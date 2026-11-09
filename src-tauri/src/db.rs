use crate::model::{History, Scan, Settings};
use rusqlite::{params, Connection};
use std::path::Path;
fn open(path: &Path) -> Result<Connection, String> {
    if std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err("Veritabanı dosyası sembolik bağlantı olamaz".into());
    }
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    c.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    c.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|e| e.to_string())?;
    let version: i64 = c
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if version > 2 {
        return Err("Veritabanı bu uygulamadan daha yeni; eski sürümle açılmaz".into());
    }
    if version < 1 {
        c.execute_batch(include_str!("../migrations/001_init.sql"))
            .map_err(|e| e.to_string())?;
    }
    if version < 2 {
        c.execute_batch(include_str!("../migrations/002_reports.sql"))
            .map_err(|e| e.to_string())?;
    };
    Ok(c)
}
pub fn save(path: &Path, scan: &Scan) -> Result<(), String> {
    let mut c = open(path)?;
    let tx = c.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO scans(started_at,status) VALUES (?1,?2)",
        params![
            i64::try_from(scan.checked_at).map_err(|_| "Tarama zamanı SQLite sınırını aşıyor")?,
            if scan.warnings.is_empty() {
                "complete"
            } else {
                "partial"
            }
        ],
    )
    .map_err(|e| e.to_string())?;
    let id = tx.last_insert_rowid();
    for p in &scan.packages {
        tx.execute("INSERT OR IGNORE INTO packages(scan_id,manager,package_id,version) VALUES (?1,?2,?3,?4)",params![id,p.manager,p.id,p.version]).map_err(|e|e.to_string())?;
    }
    tx.execute(
        "INSERT INTO scan_reports(scan_id,report_json) VALUES (?1,?2)",
        params![id, serde_json::to_string(scan).map_err(|e| e.to_string())?],
    )
    .map_err(|e| e.to_string())?;
    // Retain at most 100 scans. Delete dependent rows explicitly for the v1 schema.
    tx.execute_batch("DELETE FROM packages WHERE scan_id IN (SELECT id FROM scans ORDER BY id DESC LIMIT -1 OFFSET 100); DELETE FROM findings WHERE scan_id IN (SELECT id FROM scans ORDER BY id DESC LIMIT -1 OFFSET 100); DELETE FROM scan_reports WHERE scan_id IN (SELECT id FROM scans ORDER BY id DESC LIMIT -1 OFFSET 100); DELETE FROM scans WHERE id IN (SELECT id FROM scans ORDER BY id DESC LIMIT -1 OFFSET 100);").map_err(|e|e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}
pub fn latest(path: &Path) -> Result<Option<Scan>, String> {
    let c = open(path)?;
    let mut stmt = c
        .prepare("SELECT report_json FROM scan_reports ORDER BY scan_id DESC LIMIT 1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    rows.next()
        .map_err(|e| e.to_string())?
        .map(|r| {
            let text: String = r.get(0).map_err(|e| e.to_string())?;
            serde_json::from_str(&text).map_err(|e| e.to_string())
        })
        .transpose()
}
pub fn history(path: &Path) -> Result<Vec<History>, String> {
    let c = open(path)?;
    let mut stmt=c.prepare("SELECT s.id,s.started_at,r.report_json FROM scans s JOIN scan_reports r ON s.id=r.scan_id ORDER BY s.id DESC LIMIT 100").map_err(|e|e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut out = vec![];
    while let Some(r) = rows.next().map_err(|e| e.to_string())? {
        let text: String = r.get(2).map_err(|e| e.to_string())?;
        let s: Scan = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        out.push(History {
            id: r.get(0).map_err(|e| e.to_string())?,
            started_at: s.checked_at,
            package_count: s.packages.len() as i64,
            finding_count: s.packages.iter().map(|p| p.findings.len() as i64).sum(),
        });
    }
    Ok(out)
}
pub fn settings(path: &Path) -> Result<Settings, String> {
    let c = open(path)?;
    match c.query_row("SELECT value FROM settings WHERE id=1", [], |r| {
        r.get::<_, String>(0)
    }) {
        Ok(s) => serde_json::from_str(&s).map_err(|e| e.to_string()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Settings::default()),
        Err(e) => Err(e.to_string()),
    }
}
pub fn set_settings(path: &Path, s: &Settings) -> Result<(), String> {
    open(path)?.execute("INSERT INTO settings(id,value) VALUES (1,?1) ON CONFLICT(id) DO UPDATE SET value=excluded.value",params![serde_json::to_string(s).map_err(|e|e.to_string())?]).map(|_|()).map_err(|e|e.to_string())
}
pub fn cache_get(path: &Path, key: &str) -> Result<Option<String>, String> {
    let c = open(path)?;
    match c.query_row(
        "SELECT body FROM source_cache WHERE key=?1 AND fetched_at>?2",
        params![key, crate::model::now().saturating_sub(3600) as i64],
        |r| r.get(0),
    ) {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
pub fn cache_put(path: &Path, key: &str, body: &str) -> Result<(), String> {
    let c = open(path)?;
    c.execute("INSERT INTO source_cache(key,fetched_at,body) VALUES (?1,?2,?3) ON CONFLICT(key) DO UPDATE SET fetched_at=excluded.fetched_at,body=excluded.body",params![key,crate::model::now() as i64,body]).map_err(|e|e.to_string())?;
    c.execute(
        "DELETE FROM source_cache WHERE fetched_at<?1",
        params![crate::model::now().saturating_sub(86400) as i64],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persists_settings_and_retains_history() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("test.sqlite");
        assert!(!settings(&path).unwrap().online);
        set_settings(
            &path,
            &Settings {
                online: true,
                periodic: false,
                notifications: false,
            },
        )
        .unwrap();
        assert!(settings(&path).unwrap().online);
        for i in 0..102 {
            save(
                &path,
                &Scan {
                    packages: vec![],
                    warnings: vec![],
                    checked_at: i,
                    online: false,
                },
            )
            .unwrap();
        }
        assert_eq!(history(&path).unwrap().len(), 100);
        assert_eq!(latest(&path).unwrap().unwrap().checked_at, 101);
    }
    #[test]
    fn migrates_v1_without_losing_scans_and_rejects_newer_schema() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("test.sqlite");
        let c = Connection::open(&p).unwrap();
        c.execute_batch(include_str!("../migrations/001_init.sql"))
            .unwrap();
        c.execute(
            "INSERT INTO scans(started_at,status) VALUES (1,'inventory_only')",
            [],
        )
        .unwrap();
        drop(c);
        let c = open(&p).unwrap();
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM scans", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            c.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2
        );
        c.execute_batch("PRAGMA user_version=3").unwrap();
        drop(c);
        assert!(open(&p).is_err());
    }
    #[test]
    fn cache_round_trip() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("test.sqlite");
        assert_eq!(cache_get(&p, "x").unwrap(), None);
        cache_put(&p, "x", "{}").unwrap();
        assert_eq!(cache_get(&p, "x").unwrap(), Some("{}".into()));
    }
}
