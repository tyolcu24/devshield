use crate::model::Package;
#[cfg(windows)]
use crate::process;
use serde_json::Value;
pub trait InventoryAdapter {
    fn discover(&self) -> Result<Vec<Package>, String>;
}
pub struct Homebrew;
pub struct Winget;
impl InventoryAdapter for Homebrew {
    fn discover(&self) -> Result<Vec<Package>, String> {
        #[cfg(target_os = "macos")]
        {
            let mut result = vec![];
            let mut roots = std::collections::HashSet::new();
            for prefix in ["/opt/homebrew", "/usr/local"] {
                let prefix = std::path::Path::new(prefix);
                if !prefix.join("Cellar").is_dir() && !prefix.join("Caskroom").is_dir() {
                    continue;
                }
                let canonical = prefix.canonicalize().map_err(|e| e.to_string())?;
                if roots.insert(canonical) {
                    result.extend(read_brew_prefix(prefix)?);
                }
            }
            if roots.is_empty() {
                return Err("Standart Homebrew Cellar/Caskroom dizini bulunamadı".into());
            }
            result.sort_by(|a, b| {
                (&a.manager, &a.id, &a.version, &a.evidence_path).cmp(&(
                    &b.manager,
                    &b.id,
                    &b.version,
                    &b.evidence_path,
                ))
            });
            Ok(result)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err("Homebrew bu platformda desteklenmiyor".into())
        }
    }
}
impl InventoryAdapter for Winget {
    fn discover(&self) -> Result<Vec<Package>, String> {
        #[cfg(windows)]
        {
            use std::{ffi::OsStr, io::Read};
            // Microsoft App Installer's per-user app execution alias; never search PATH.
            let local = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA bulunamadı")?;
            let executable =
                std::path::PathBuf::from(local).join("Microsoft/WindowsApps/winget.exe");
            if !executable.is_file() {
                return Err("Microsoft App Installer / winget bulunamadı".into());
            }
            let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
            let output = dir.path().join("inventory.json");
            process::run(
                &executable,
                &[
                    OsStr::new("export"),
                    OsStr::new("--output"),
                    output.as_os_str(),
                    OsStr::new("--include-versions"),
                    OsStr::new("--source"),
                    OsStr::new("winget"),
                    OsStr::new("--disable-interactivity"),
                ],
            )?;
            let mut data = String::new();
            std::fs::File::open(output)
                .map_err(|e| e.to_string())?
                .take(8 * 1024 * 1024 + 1)
                .read_to_string(&mut data)
                .map_err(|e| e.to_string())?;
            if data.len() > 8 * 1024 * 1024 {
                return Err("winget çıktı sınırı aşıldı".into());
            }
            parse_winget(data.trim_start_matches('\u{feff}'))
        }
        #[cfg(not(windows))]
        {
            Err("winget bu platformda desteklenmiyor".into())
        }
    }
}
pub fn parse_brew(text: &str) -> Result<Vec<Package>, String> {
    let data: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let formulae = data["formulae"].as_array().ok_or("formulae alanı eksik")?;
    let mut result = vec![];
    for f in formulae {
        let id = f["full_name"].as_str().ok_or("Paket kimliği eksik")?;
        // Third-party taps remain visible but never enter the curated mapping.
        let manager = if f["tap"].as_str() == Some("homebrew/core") {
            "homebrew"
        } else {
            "homebrew-tap"
        };
        for installed in f["installed"].as_array().ok_or("Kurulu sürüm eksik")? {
            let mut package = Package::new(
                manager,
                id,
                installed["version"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("unknown"),
            );
            package.install_reason = match installed["installed_on_request"].as_bool() {
                Some(true) => "requested",
                Some(false) => "dependency",
                None => "unknown",
            }
            .into();
            if let Some(stable) = f["versions"]["stable"].as_str() {
                let revision = f["revision"].as_u64().unwrap_or(0);
                let latest = if revision > 0 {
                    format!("{stable}_{revision}")
                } else {
                    stable.into()
                };
                package.update_status = if package.version == latest {
                    "catalog_same"
                } else {
                    match crate::security::compare_versions(&package.version, &latest).as_str() {
                        "update_available" => "catalog_update_available",
                        "installed_newer" => "installed_newer",
                        _ => "version_review",
                    }
                }
                .into();
                package.latest = Some(latest);
            }
            result.push(package);
        }
    }
    if let Some(casks) = data["casks"].as_array() {
        for c in casks {
            let id = c["token"].as_str().ok_or("Cask kimliği eksik")?;
            if let Some(v) = c["installed"].as_str() {
                result.push(Package::new(
                    if c["tap"].as_str() == Some("homebrew/cask") {
                        "homebrew-cask"
                    } else {
                        "homebrew-cask-tap"
                    },
                    id,
                    v,
                ));
            } else if let Some(versions) = c["installed"].as_array() {
                for v in versions {
                    if let Some(v) = v.as_str() {
                        result.push(Package::new(
                            if c["tap"].as_str() == Some("homebrew/cask") {
                                "homebrew-cask"
                            } else {
                                "homebrew-cask-tap"
                            },
                            id,
                            v,
                        ));
                    }
                }
            }
        }
    }
    result.sort_by(|a, b| (&a.manager, &a.id, &a.version).cmp(&(&b.manager, &b.id, &b.version)));
    result.dedup();
    Ok(result)
}
pub fn parse_winget(text: &str) -> Result<Vec<Package>, String> {
    let data: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut result = vec![];
    for source in data["Sources"].as_array().ok_or("Sources eksik")? {
        let trusted = source["SourceDetails"]["Identifier"].as_str()
            == Some("Microsoft.Winget.Source_8wekyb3d8bbwe");
        for p in source["Packages"].as_array().ok_or("Packages eksik")? {
            let id = p["PackageIdentifier"]
                .as_str()
                .ok_or("Paket kimliği eksik")?;
            result.push(Package::new(
                if trusted { "winget" } else { "winget-other" },
                id,
                p["Version"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("unknown"),
            ));
        }
    }
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result.dedup();
    Ok(result)
}
/// Read local installation receipts, not the remote catalog and not PATH binaries.
pub fn read_brew_prefix(prefix: &std::path::Path) -> Result<Vec<Package>, String> {
    let mut packages = vec![];
    for (kind, root) in [
        ("formula", prefix.join("Cellar")),
        ("cask", prefix.join("Caskroom")),
    ] {
        if !root.is_dir() {
            continue;
        }
        let base = root.canonicalize().map_err(|e| e.to_string())?;
        let mut entries = 0usize;
        for item in std::fs::read_dir(&root).map_err(|e| e.to_string())? {
            let item = item.map_err(|e| e.to_string())?;
            entries += 1;
            if entries > 10000 {
                return Err("Homebrew envanter sınırı aşıldı".into());
            }
            let name = item.file_name().to_string_lossy().into_owned();
            if !safe_component(&name) || !item.path().is_dir() {
                continue;
            }
            let item_path = item.path().canonicalize().map_err(|e| e.to_string())?;
            if !item_path.starts_with(&base) {
                continue;
            }
            if kind == "formula" {
                for version in std::fs::read_dir(&item_path).map_err(|e| e.to_string())? {
                    let version = version.map_err(|e| e.to_string())?;
                    if !version.path().is_dir() {
                        continue;
                    }
                    let v = version.file_name().to_string_lossy().into_owned();
                    if !safe_component(&v) {
                        continue;
                    }
                    packages.push(Package::new("homebrew", &name, &v));
                    if packages.len() > 10000 {
                        return Err("Homebrew envanter sınırı aşıldı".into());
                    }
                }
            } else {
                let path = item_path.join(".metadata/INSTALL_RECEIPT.json");
                let Some((receipt, path)) = read_receipt(&path, &base) else {
                    continue;
                };
                let v = receipt["source"]["version"].as_str().unwrap_or("unknown");
                let mut package = Package::new(
                    if receipt["source"]["tap"].as_str() == Some("homebrew/cask") {
                        "homebrew-cask"
                    } else {
                        "homebrew-cask-tap"
                    },
                    &name,
                    v,
                );
                package.install_reason = match receipt["installed_on_request"].as_bool() {
                    Some(true) => "requested",
                    Some(false) => "dependency",
                    None => "unknown",
                }
                .into();
                package.verification = "receipt_found".into();
                package.evidence_path = Some(path.to_string_lossy().into());
                packages.push(package);
            }
        }
    }
    // Cask receipts were checked above; formula receipts are independently checked here.
    verify_brew_receipts(&mut packages, prefix)?;
    Ok(packages)
}
fn read_receipt(
    path: &std::path::Path,
    base: &std::path::Path,
) -> Option<(Value, std::path::PathBuf)> {
    let path = path.canonicalize().ok()?;
    if !path.starts_with(base)
        || !path.is_file()
        || std::fs::metadata(&path).ok()?.len() > 512 * 1024
    {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    let data = serde_json::from_slice(&bytes).ok()?;
    Some((data, path))
}
fn safe_component(s: &str) -> bool {
    !s.is_empty()
        && s.len() < 200
        && !s.starts_with('.')
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"@+_.,-".contains(&c))
}
pub fn verify_brew_receipts(
    packages: &mut [Package],
    prefix: &std::path::Path,
) -> Result<(), String> {
    let cellar = prefix.join("Cellar");
    for p in packages {
        let name = p.id.rsplit('/').next().unwrap_or("");
        if !safe_component(name) || !safe_component(&p.version) {
            p.verification = "unverified".into();
            continue;
        }
        if p.manager.starts_with("homebrew-cask") {
            continue;
        }
        let receipt = cellar
            .join(name)
            .join(&p.version)
            .join("INSTALL_RECEIPT.json");
        let Ok(base) = cellar.canonicalize() else {
            p.verification = "missing_receipt".into();
            continue;
        };
        let Ok(path) = receipt.canonicalize() else {
            p.verification = "missing_receipt".into();
            continue;
        };
        if !path.starts_with(&base)
            || std::fs::metadata(&path)
                .map(|m| m.len() > 512 * 1024)
                .unwrap_or(true)
        {
            p.verification = "unverified".into();
            continue;
        }
        let receipt: Value = match std::fs::read(&path)
            .ok()
            .and_then(|s| serde_json::from_slice(&s).ok())
        {
            Some(v) => v,
            None => {
                p.verification = "unverified".into();
                continue;
            }
        };
        p.install_reason = match receipt["installed_on_request"].as_bool() {
            Some(true) => "requested",
            Some(false) => "dependency",
            None => "unknown",
        }
        .into();
        if receipt["source"]["tap"].as_str() != Some("homebrew/core") {
            p.manager = "homebrew-tap".into();
        }
        p.verification = "receipt_found".into();
        p.evidence_path = Some(path.to_string_lossy().into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn brew_preserves_revisions_and_casks() {
        let p=parse_brew(r#"{"formulae":[{"full_name":"node","tap":"homebrew/core","installed":[{"version":"22.1.0_1"}]}],"casks":[{"token":"visual-studio-code","installed":"1.2.3"}]}"#).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].version, "22.1.0_1");
    }
    #[test]
    fn tap_cannot_impersonate_core() {
        let p=parse_brew(r#"{"formulae":[{"full_name":"node","tap":"evil/tap","installed":[{"version":"22.0.0"}]}]}"#).unwrap();
        assert_eq!(p[0].manager, "homebrew-tap");
    }
    #[test]
    fn winget_unknown_is_preserved() {
        let p=parse_winget(r#"{"Sources":[{"SourceDetails":{"Identifier":"Microsoft.Winget.Source_8wekyb3d8bbwe"},"Packages":[{"PackageIdentifier":"OpenJS.NodeJS"}]}]}"#).unwrap();
        assert_eq!(p[0].version, "unknown");
        assert_eq!(p[0].manager, "winget");
    }
    #[test]
    fn catalog_downgrade_is_not_an_update() {
        let p=parse_brew(r#"{"formulae":[{"full_name":"openssl@3","tap":"homebrew/core","versions":{"stable":"3.5.9"},"outdated":true,"installed":[{"version":"3.6.3"}]}]}"#).unwrap();
        assert_eq!(p[0].update_status, "installed_newer");
    }
    #[test]
    fn verifies_local_receipt_and_distinguishes_dependencies() {
        let d = tempfile::tempdir().unwrap();
        let folder = d.path().join("Cellar/node/22.1.0");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join("INSTALL_RECEIPT.json"),
            r#"{"installed_on_request":false,"source":{"tap":"homebrew/core"}}"#,
        )
        .unwrap();
        let mut packages = vec![
            Package::new("homebrew", "node", "22.1.0"),
            Package::new("homebrew", "absent", "1.0.0"),
        ];
        verify_brew_receipts(&mut packages, d.path()).unwrap();
        assert_eq!(packages[0].verification, "receipt_found");
        assert_eq!(packages[0].install_reason, "dependency");
        assert_eq!(packages[1].verification, "missing_receipt");
    }
    #[test]
    fn native_inventory_ignores_catalog_and_checks_receipts() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("Cellar/node/22.1.0");
        std::fs::create_dir_all(&f).unwrap();
        std::fs::write(
            f.join("INSTALL_RECEIPT.json"),
            r#"{"installed_on_request":true,"source":{"tap":"homebrew/core"}}"#,
        )
        .unwrap();
        let c = d.path().join("Caskroom/example/.metadata");
        std::fs::create_dir_all(&c).unwrap();
        std::fs::write(
            c.join("INSTALL_RECEIPT.json"),
            r#"{"installed_on_request":true,"source":{"tap":"homebrew/cask","version":"1.2.3"}}"#,
        )
        .unwrap();
        let p = read_brew_prefix(d.path()).unwrap();
        assert_eq!(p.len(), 2);
        assert!(p.iter().all(|p| p.verification == "receipt_found"));
        assert!(p.iter().all(|p| p.latest.is_none()));
    }
    #[test]
    fn paths_cannot_escape_cellar() {
        assert!(!safe_component("../secret"));
        assert!(!safe_component("/etc/passwd"));
        assert!(safe_component("22.1.0_1"));
    }
    #[test]
    fn invalid_inventory_is_error() {
        assert!(parse_brew("{}").is_err());
        assert!(parse_winget("{}").is_err());
    }
}
