PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS scans (id INTEGER PRIMARY KEY, started_at INTEGER NOT NULL, status TEXT NOT NULL CHECK(status IN ('inventory_only','complete','partial','failed')));
CREATE TABLE IF NOT EXISTS packages (scan_id INTEGER NOT NULL REFERENCES scans(id), manager TEXT NOT NULL, package_id TEXT NOT NULL, version TEXT NOT NULL, PRIMARY KEY(scan_id, manager, package_id, version));
CREATE TABLE IF NOT EXISTS identities (manager TEXT NOT NULL, package_id TEXT NOT NULL, ecosystem TEXT NOT NULL, canonical_name TEXT NOT NULL, evidence_url TEXT NOT NULL, reviewed_at INTEGER NOT NULL, PRIMARY KEY(manager,package_id));
CREATE TABLE IF NOT EXISTS findings (scan_id INTEGER NOT NULL REFERENCES scans(id), package_id TEXT NOT NULL, advisory_id TEXT NOT NULL, source TEXT NOT NULL, evidence_json TEXT NOT NULL, PRIMARY KEY(scan_id,package_id,advisory_id));
PRAGMA user_version = 1;
