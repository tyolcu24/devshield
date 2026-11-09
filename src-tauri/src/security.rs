use crate::{
    db,
    model::{Finding, Package},
};
use serde_json::{json, Value};
use std::{
    io::Read,
    path::Path,
    time::{Duration, Instant},
};
const LIMIT: u64 = 4 * 1024 * 1024;
/// Fixed HTTPS origins only. No frontend URLs, credentials, redirects or telemetry.
pub struct Sources {
    client: reqwest::blocking::Client,
    requests: std::cell::Cell<usize>,
    started: Instant,
}
impl Sources {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            requests: std::cell::Cell::new(0),
            started: Instant::now(),
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(4))
                .redirect(reqwest::redirect::Policy::none())
                .user_agent("DevShield/0.2.1")
                .build()
                .map_err(|e| e.to_string())?,
        })
    }
    fn fetch(&self, path: &Path, url: &str, body: Option<&Value>) -> Result<Value, String> {
        if !allowed_url(url) {
            return Err("Kaynak izin listesinde değil".into());
        }
        let key = format!("{url}:{}", body.map(|b| b.to_string()).unwrap_or_default());
        if let Some(cached) = db::cache_get(path, &key)? {
            return serde_json::from_str(&cached).map_err(|e| e.to_string());
        }
        if self.requests.get() >= 60 || self.started.elapsed() > Duration::from_secs(120) {
            return Err("Kaynak kotası/süre sınırı; sonraki taramada tekrar deneyin".into());
        }
        self.requests.set(self.requests.get() + 1);
        let response = if let Some(body) = body {
            self.client.post(url).json(body).send()
        } else {
            self.client.get(url).send()
        }
        .map_err(|e| e.to_string())?;
        let response = response.error_for_status().map_err(|e| e.to_string())?;
        let mut bytes = vec![];
        response
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > LIMIT {
            return Err("Kaynak yanıt boyutu sınırını aştı".into());
        }
        let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        let value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        db::cache_put(path, &key, &text)?;
        Ok(value)
    }
    pub fn check(&self, path: &Path, p: &mut Package) -> Result<(), String> {
        let Some(identity) = identity(p) else {
            p.security_status = "unsupported_identity".into();
            return Ok(());
        };
        let mut token = None;
        let mut findings = vec![];
        let mut incomplete = false;
        for page in 0..5 {
            let mut body = json!({"package":{"ecosystem":"GIT","name":identity.repository},"version":identity.tag});
            if let Some(t) = token.take() {
                body["page_token"] = Value::String(t);
            }
            let response = self.fetch(path, "https://api.osv.dev/v1/query", Some(&body))?;
            let (found, unverified) = match_response(&response, &identity)?;
            findings.extend(found);
            incomplete |= unverified;
            token = response
                .get("next_page_token")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_owned);
            if token.is_none() {
                break;
            }
            if page == 4 {
                return Err("OSV sayfalama sınırı; kontrol tamamlanmadı".into());
            }
        }
        findings.sort_by(|a, b| a.id.cmp(&b.id));
        findings.dedup_by(|a, b| a.id == b.id);
        p.security_status = if !findings.is_empty() {
            "review_required"
        } else if incomplete {
            "insufficient_evidence"
        } else {
            "no_known_match"
        }
        .into();
        p.findings = findings;
        Ok(())
    }
    pub fn update(&self, path: &Path, p: &mut Package) -> Result<(), String> {
        if is_node(p) {
            return self.node_update(path, p);
        }
        let url = match p.manager.as_str() {
            "homebrew" if valid_brew_id(&p.id) => {
                format!("https://formulae.brew.sh/api/formula/{}.json", p.id)
            }
            "homebrew-cask" if valid_brew_id(&p.id) => {
                format!("https://formulae.brew.sh/api/cask/{}.json", p.id)
            }
            "winget" => {
                let Some(path) = winget_manifest_path(&p.id) else {
                    return Ok(());
                };
                format!(
                    "https://api.github.com/repos/microsoft/winget-pkgs/contents/manifests/{path}"
                )
            }
            _ => return Ok(()),
        };
        let data = self.fetch(path, &url, None)?;
        let latest = if p.manager == "homebrew" {
            let stable = data["versions"]["stable"]
                .as_str()
                .ok_or("Homebrew sürümü eksik")?;
            let rev = data["revision"].as_u64().unwrap_or(0);
            Some(if rev > 0 {
                format!("{stable}_{rev}")
            } else {
                stable.into()
            })
        } else if p.manager == "homebrew-cask" {
            data["version"].as_str().map(str::to_owned)
        } else {
            let installed = numeric_version(&p.version);
            let Some(installed) = installed else {
                return Ok(());
            };
            data.as_array()
                .ok_or("winget kaynak şeması geçersiz")?
                .iter()
                .filter(|e| e["type"] == "dir")
                .filter_map(|e| {
                    let name = e["name"].as_str()?;
                    let n = numeric_version(name)?;
                    if n.len() != installed.len() {
                        return None;
                    }
                    Some((n, name.to_owned()))
                })
                .max_by(|a, b| a.0.cmp(&b.0))
                .map(|(_, s)| s)
        };
        if let Some(latest) = latest {
            p.update_status = compare_versions(&p.version, &latest);
            p.latest = Some(latest);
        }
        Ok(())
    }
    pub fn node_update(&self, path: &Path, p: &mut Package) -> Result<(), String> {
        if !is_node(p) {
            return Ok(());
        }
        let Some(installed) = plain_version(&p.version) else {
            return Ok(());
        };
        let data = self.fetch(path, "https://nodejs.org/dist/index.json", None)?;
        let (latest, security) = node_latest(&data, &installed)?;
        if let Some(version) = latest {
            p.latest = Some(version.to_string());
            p.update_status = if version > installed {
                if security {
                    "security_update_available"
                } else {
                    "update_available"
                }
            } else {
                "current_branch"
            }
            .into();
        }
        Ok(())
    }
}
#[derive(Debug)]
pub struct Identity {
    pub repository: &'static str,
    pub tag: String,
    pub version: semver::Version,
}
fn valid_brew_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() < 128
        && id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"@+_.-".contains(&c))
        && !id.starts_with('.')
}
pub fn winget_manifest_path(id: &str) -> Option<String> {
    let parts: Vec<_> = id.split('.').collect();
    if parts.len() < 2
        || parts.len() > 8
        || id.len() > 180
        || parts.iter().any(|s| {
            s.is_empty()
                || !s
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
        })
    {
        return None;
    }
    let first = id.chars().next()?.to_ascii_lowercase();
    Some(format!("{first}/{}", parts.join("/")))
}
fn allowed_url(url: &str) -> bool {
    if [
        "https://api.osv.dev/v1/query",
        "https://nodejs.org/dist/index.json",
    ]
    .contains(&url)
    {
        return true;
    }
    for prefix in [
        "https://formulae.brew.sh/api/formula/",
        "https://formulae.brew.sh/api/cask/",
    ] {
        if let Some(id) = url
            .strip_prefix(prefix)
            .and_then(|s| s.strip_suffix(".json"))
        {
            return valid_brew_id(id);
        }
    }
    if let Some(path) =
        url.strip_prefix("https://api.github.com/repos/microsoft/winget-pkgs/contents/manifests/")
    {
        let parts: Vec<_> = path.split('/').collect();
        return parts.len() >= 3
            && parts.len() <= 10
            && parts.iter().all(|s| {
                !s.is_empty()
                    && s.bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
            });
    }
    false
}
fn numeric_version(raw: &str) -> Option<Vec<u64>> {
    let parts: Vec<_> = raw.split('.').collect();
    if !(2..=4).contains(&parts.len())
        || parts
            .iter()
            .any(|s| s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    parts.iter().map(|s| s.parse().ok()).collect()
}
pub fn compare_versions(installed: &str, latest: &str) -> String {
    if installed == latest {
        return "source_same".into();
    }
    // Only purely numeric versions with the same number of components have a safe ordering here.
    // Homebrew revisions are compared separately and never stripped for vulnerability queries.
    fn brew_parts(v: &str) -> Option<(Vec<u64>, u64)> {
        let (v, revision) = if let Some((v, r)) = v.split_once('_') {
            (v, r.parse().ok()?)
        } else {
            (v, 0)
        };
        Some((numeric_version(v)?, revision))
    }
    match (brew_parts(installed), brew_parts(latest)) {
        (Some(a), Some(b)) if a.0.len() == b.0.len() => if b > a {
            "update_available"
        } else {
            "installed_newer"
        }
        .into(),
        _ => "version_review".into(),
    }
}
fn is_node(p: &Package) -> bool {
    (p.manager == "homebrew"
        && (p.id == "node"
            || p.id
                .strip_prefix("node@")
                .is_some_and(|s| s.parse::<u64>().is_ok())))
        || (p.manager == "winget"
            && ["OpenJS.NodeJS", "OpenJS.NodeJS.LTS"].contains(&p.id.as_str()))
}
fn plain_version(raw: &str) -> Option<semver::Version> {
    let v = semver::Version::parse(raw).ok()?;
    if !v.pre.is_empty() || !v.build.is_empty() {
        None
    } else {
        Some(v)
    }
}
pub fn identity(p: &Package) -> Option<Identity> {
    let v = plain_version(&p.version)?;
    let (repo, tag) = if is_node(p) {
        ("https://github.com/nodejs/node.git", format!("v{v}"))
    } else {
        match (p.manager.as_str(), p.id.as_str()) {
            ("homebrew", "curl") => (
                "https://github.com/curl/curl.git",
                format!("curl-{}_{}_{}", v.major, v.minor, v.patch),
            ),
            ("homebrew", "git") | ("winget", "Git.Git") => {
                ("https://github.com/git/git.git", format!("v{v}"))
            }
            ("homebrew", "go") | ("winget", "GoLang.Go") => {
                ("https://github.com/golang/go.git", format!("go{v}"))
            }
            ("homebrew", "rust") | ("winget", "Rustlang.Rust.MSVC") => {
                ("https://github.com/rust-lang/rust.git", v.to_string())
            }
            ("homebrew", id)
                if id == "python@3.13" || id == "python@3.14" || id == "python@3.12" =>
            {
                ("https://github.com/python/cpython.git", format!("v{v}"))
            }
            _ => return None,
        }
    };
    Some(Identity {
        repository: repo,
        tag,
        version: v,
    })
}
fn canonical_repo(s: &str) -> &str {
    s.trim_end_matches('/').trim_end_matches(".git")
}
fn semver_affected(r: &Value, v: &semver::Version) -> Option<bool> {
    let mut active = false;
    let mut matched = false;
    for event in r["events"].as_array()? {
        if let Some(s) = event["introduced"].as_str() {
            active = s == "0" || v >= &semver::Version::parse(s).ok()?;
        } else if let Some(s) = event["fixed"].as_str() {
            if active && v < &semver::Version::parse(s).ok()? {
                matched = true;
            }
            active = false;
        } else if let Some(s) = event["last_affected"].as_str() {
            if active && v <= &semver::Version::parse(s).ok()? {
                matched = true;
            }
            active = false;
        } else {
            let s = event["limit"].as_str()?;
            if active && v < &semver::Version::parse(s).ok()? {
                matched = true;
            }
            active = false;
        }
    }
    Some(matched || active)
}
pub fn match_response(
    response: &Value,
    identity: &Identity,
) -> Result<(Vec<Finding>, bool), String> {
    let Some(vulns) = response.get("vulns") else {
        if response
            .as_object()
            .is_some_and(|o| o.keys().all(|k| k == "next_page_token"))
        {
            return Ok((vec![], false));
        }
        return Err("OSV yanıt şeması beklenmiyor".into());
    };
    let mut findings = vec![];
    let mut uncertain = false;
    for v in vulns.as_array().ok_or("OSV vulns bir dizi değil")? {
        if v.get("withdrawn").is_some_and(|w| !w.is_null()) {
            continue;
        }
        let mut confirmed = false;
        for affected in v["affected"].as_array().ok_or("OSV affected eksik")? {
            let package_matches = affected["package"]["ecosystem"].as_str() == Some("GIT")
                && affected["package"]["name"]
                    .as_str()
                    .is_some_and(|r| canonical_repo(r) == canonical_repo(identity.repository));
            let repo_matches = affected["ranges"].as_array().is_some_and(|rs| {
                rs.iter().any(|r| {
                    r["repo"]
                        .as_str()
                        .is_some_and(|r| canonical_repo(r) == canonical_repo(identity.repository))
                })
            });
            if !package_matches && !repo_matches {
                continue;
            }
            let exact = affected["versions"]
                .as_array()
                .is_some_and(|vs| vs.iter().any(|v| v.as_str() == Some(&identity.tag)));
            let semver_match = package_matches
                && affected["ranges"].as_array().is_some_and(|rs| {
                    rs.iter().any(|r| {
                        r["type"] == "SEMVER" && semver_affected(r, &identity.version) == Some(true)
                    })
                });
            if exact || semver_match {
                confirmed = true;
            } else {
                uncertain = true;
            }
        }
        if confirmed {
            let id = v["id"]
                .as_str()
                .filter(|id| {
                    !id.is_empty()
                        && id
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                })
                .ok_or("Geçersiz advisory kimliği")?;
            findings.push(Finding{id:id.into(),summary:v["summary"].as_str().unwrap_or("Güvenlik kaydı — üretici yaması doğrulanmalı").chars().take(500).collect(),url:format!("https://osv.dev/vulnerability/{id}"),source:"OSV".into(),evidence:format!("{} @ {}; upstream sürüm kanıtı. Paket üreticisinin backport/yama durumu doğrulanmadı.",identity.repository,identity.tag)});
        }
    }
    Ok((findings, uncertain))
}
pub fn node_latest(
    data: &Value,
    installed: &semver::Version,
) -> Result<(Option<semver::Version>, bool), String> {
    let mut latest = None;
    let mut security = false;
    for r in data.as_array().ok_or("Node sürüm yanıtı dizi değil")? {
        let Some(v) = r["version"]
            .as_str()
            .and_then(|s| plain_version(s.strip_prefix('v').unwrap_or(s)))
        else {
            continue;
        };
        if v.major != installed.major {
            continue;
        }
        if v > *installed && r["security"].as_bool() == Some(true) {
            security = true;
        }
        if latest.as_ref().is_none_or(|l| &v > l) {
            latest = Some(v);
        }
    }
    Ok((latest, security))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn node() -> Identity {
        identity(&Package::new("homebrew", "node", "22.1.0")).unwrap()
    }
    #[test]
    fn urls_are_constrained() {
        assert!(allowed_url(
            "https://formulae.brew.sh/api/formula/node@22.json"
        ));
        assert!(!allowed_url(
            "https://formulae.brew.sh/api/formula/../x.json"
        ));
        assert!(!allowed_url("https://api.osv.dev.evil.test/v1/query"));
        assert!(winget_manifest_path("../x").is_none());
        assert_eq!(
            winget_manifest_path("OpenJS.NodeJS").unwrap(),
            "o/OpenJS/NodeJS"
        );
    }
    #[test]
    fn numeric_versions_do_not_guess_prereleases() {
        assert_eq!(compare_versions("1.2.3_1", "1.2.3_2"), "update_available");
        assert_eq!(compare_versions("1.2.3", "1.2.4"), "update_available");
        assert_eq!(compare_versions("1.2.3-beta", "1.2.3"), "version_review");
        assert_eq!(compare_versions("1.2.3.4", "1.2.3"), "version_review");
    }
    #[test]
    fn rejects_ambiguous_revisions_and_names() {
        assert!(identity(&Package::new("homebrew", "node", "22.1.0_1")).is_none());
        assert!(identity(&Package::new("homebrew-tap", "node", "22.1.0")).is_none());
        assert!(identity(&Package::new("winget-other", "OpenJS.NodeJS", "22.1.0")).is_none());
        assert!(identity(&Package::new("homebrew", "some-node", "22.1.0")).is_none());
    }
    #[test]
    fn fuzzy_or_other_repo_never_becomes_finding() {
        let r = json!({"vulns":[{"id":"CVE-1","affected":[{"package":{"ecosystem":"npm","name":"node"},"versions":["v22.1.0"]}]}]});
        assert!(match_response(&r, &node()).unwrap().0.is_empty());
    }
    #[test]
    fn exact_tag_requires_repository() {
        let r = json!({"vulns":[{"id":"OSV-1","affected":[{"package":{"ecosystem":"GIT","name":"https://github.com/nodejs/node.git"},"versions":["v22.1.0"]}]}]});
        assert_eq!(match_response(&r, &node()).unwrap().0.len(), 1);
    }
    #[test]
    fn withdrawn_not_reported() {
        let r = json!({"vulns":[{"withdrawn":"2026-01-01"}]});
        assert!(match_response(&r, &node()).unwrap().0.is_empty());
    }
    #[test]
    fn git_ranges_without_tag_need_review() {
        let r = json!({"vulns":[{"affected":[{"ranges":[{"type":"GIT","repo":"https://github.com/nodejs/node.git","events":[]}]}]}]});
        assert_eq!(match_response(&r, &node()).unwrap(), (vec![], true));
    }
    #[test]
    fn fixed_boundary_is_excluded() {
        let r = json!({"events":[{"introduced":"22.0.0"},{"fixed":"22.1.0"}]});
        assert_eq!(semver_affected(&r, &node().version), Some(false));
    }
    #[test]
    fn node_stays_on_installed_major() {
        let r =
            json!([{"version":"v24.1.0","security":true},{"version":"v22.2.0","security":true}]);
        let (l, s) = node_latest(&r, &node().version).unwrap();
        assert_eq!(l.unwrap().to_string(), "22.2.0");
        assert!(s);
    }
}
