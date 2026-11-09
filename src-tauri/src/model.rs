use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Package {
    pub manager: String,
    pub id: String,
    pub version: String,
    #[serde(default = "unknown")]
    pub install_reason: String,
    #[serde(default = "unverified")]
    pub verification: String,
    #[serde(default)]
    pub evidence_path: Option<String>,
    pub latest: Option<String>,
    pub update_status: String,
    pub security_status: String,
    pub findings: Vec<Finding>,
}
impl Package {
    pub fn new(manager: &str, id: &str, version: &str) -> Self {
        Self {
            manager: manager.into(),
            id: id.into(),
            version: version.into(),
            install_reason: "unknown".into(),
            verification: "manager_reported".into(),
            evidence_path: None,
            latest: None,
            update_status: "unknown".into(),
            security_status: "not_checked".into(),
            findings: vec![],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    pub id: String,
    pub summary: String,
    pub url: String,
    pub source: String,
    pub evidence: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scan {
    pub packages: Vec<Package>,
    pub warnings: Vec<String>,
    pub checked_at: u64,
    pub online: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    pub online: bool,
    pub periodic: bool,
    #[serde(default)]
    pub notifications: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub id: i64,
    pub started_at: u64,
    pub package_count: i64,
    pub finding_count: i64,
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn unknown() -> String {
    "unknown".into()
}
fn unverified() -> String {
    "legacy_unverified".into()
}
