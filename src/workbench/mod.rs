//! Minimal local scan workbench (SQLite): register, list, show, compare, remediate.

pub mod compare;
pub mod remediation;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::{params, Connection};
use serde::Serialize;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS scans (
  scan_id TEXT PRIMARY KEY,
  mode TEXT NOT NULL,
  display_name TEXT NOT NULL,
  scan_dir TEXT NOT NULL,
  report_path TEXT,
  finding_count INTEGER NOT NULL DEFAULT 0,
  max_severity TEXT NOT NULL DEFAULT 'none',
  sealed_at TEXT,
  registered_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_scans_registered ON scans(registered_at DESC);
"#;

#[derive(Debug, Clone, Serialize)]
pub struct ScanRow {
    pub scan_id: String,
    pub mode: String,
    pub display_name: String,
    pub scan_dir: String,
    pub report_path: Option<String>,
    pub finding_count: i64,
    pub max_severity: String,
    pub sealed_at: Option<String>,
    pub registered_at: String,
}

fn default_db_path() -> PathBuf {
    dirs_next_home()
        .map(|h| h.join(".weeping-angel").join("workbench.sqlite3"))
        .unwrap_or_else(|| PathBuf::from("weeping-angel-workbench.sqlite3"))
}

fn dirs_next_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

pub fn open_db(path: Option<&Path>) -> Result<Connection> {
    let path = path.map(Path::to_path_buf).unwrap_or_else(default_db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&path)
        .with_context(|| format!("open workbench db {}", path.display()))?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

/// Register a sealed scan directory into the workbench index.
pub fn register_scan(conn: &Connection, scan_dir: &Path) -> Result<ScanRow> {
    let manifest_path = scan_dir.join("scan-manifest.json");
    let findings_path = scan_dir.join("findings.json");
    if !manifest_path.is_file() || !findings_path.is_file() {
        bail!(
            "scan-dir missing sealed artifacts: {}",
            scan_dir.display()
        );
    }
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?;
    let findings: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&findings_path)?)?;

    let scan_id = manifest["scan"]["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("manifest.scan.id missing"))?
        .to_string();
    let display_name = manifest["scan"]["target"]["displayName"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let sealed_at = manifest["scan"]["sealedAt"].as_str().map(str::to_string);
    let finding_count = findings["findings"]
        .as_array()
        .map(|a| a.len() as i64)
        .unwrap_or(0);
    let max_severity = findings["findings"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|f| f["severity"]["level"].as_str())
                .max_by_key(|l| crate::engines::severity_rank(l))
                .unwrap_or("none")
                .to_string()
        })
        .unwrap_or_else(|| "none".into());

    // infer mode from coverage if present
    let mode = scan_dir
        .join("coverage.json")
        .exists()
        .then(|| std::fs::read_to_string(scan_dir.join("coverage.json")).ok())
        .flatten()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v["mode"].as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".into());

    let report_path = {
        let p = scan_dir.join("report.md");
        if p.is_file() {
            Some(p.display().to_string())
        } else {
            None
        }
    };
    let registered_at = chrono::Utc::now().to_rfc3339();
    let scan_dir_s = scan_dir
        .canonicalize()
        .unwrap_or_else(|_| scan_dir.to_path_buf())
        .display()
        .to_string();

    conn.execute(
        r#"INSERT INTO scans
        (scan_id, mode, display_name, scan_dir, report_path, finding_count, max_severity, sealed_at, registered_at)
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
        ON CONFLICT(scan_id) DO UPDATE SET
          mode=excluded.mode,
          display_name=excluded.display_name,
          scan_dir=excluded.scan_dir,
          report_path=excluded.report_path,
          finding_count=excluded.finding_count,
          max_severity=excluded.max_severity,
          sealed_at=excluded.sealed_at,
          registered_at=excluded.registered_at
        "#,
        params![
            scan_id,
            mode,
            display_name,
            scan_dir_s,
            report_path,
            finding_count,
            max_severity,
            sealed_at,
            registered_at,
        ],
    )?;

    get_scan(conn, &scan_id)?.ok_or_else(|| anyhow::anyhow!("register failed to read back"))
}

pub fn list_scans(conn: &Connection, limit: usize) -> Result<Vec<ScanRow>> {
    let mut stmt = conn.prepare(
        r#"SELECT scan_id, mode, display_name, scan_dir, report_path, finding_count,
                  max_severity, sealed_at, registered_at
           FROM scans ORDER BY registered_at DESC LIMIT ?1"#,
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok(ScanRow {
            scan_id: row.get(0)?,
            mode: row.get(1)?,
            display_name: row.get(2)?,
            scan_dir: row.get(3)?,
            report_path: row.get(4)?,
            finding_count: row.get(5)?,
            max_severity: row.get(6)?,
            sealed_at: row.get(7)?,
            registered_at: row.get(8)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_scan(conn: &Connection, scan_id: &str) -> Result<Option<ScanRow>> {
    let mut stmt = conn.prepare(
        r#"SELECT scan_id, mode, display_name, scan_dir, report_path, finding_count,
                  max_severity, sealed_at, registered_at
           FROM scans WHERE scan_id = ?1"#,
    )?;
    let mut rows = stmt.query_map(params![scan_id], |row| {
        Ok(ScanRow {
            scan_id: row.get(0)?,
            mode: row.get(1)?,
            display_name: row.get(2)?,
            scan_dir: row.get(3)?,
            report_path: row.get(4)?,
            finding_count: row.get(5)?,
            max_severity: row.get(6)?,
            sealed_at: row.get(7)?,
            registered_at: row.get(8)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn register_and_list() {
        let dir = tempdir().unwrap();
        let db = dir.path().join("wb.sqlite3");
        let conn = open_db(Some(&db)).unwrap();

        // minimal sealed-looking bundle
        let scan = dir.path().join("scan");
        std::fs::create_dir_all(&scan).unwrap();
        std::fs::write(
            scan.join("scan-manifest.json"),
            r#"{"documentType":"codex-security.scan-manifest","schemaVersion":"1.0","scan":{"id":"wa_test","producer":{"name":"wa","version":"0"},"status":"completed","startedAt":"","completedAt":"","sealedAt":"2026-01-01T00:00:00Z","target":{"kind":"directory_snapshot","targetId":"t","displayName":"toy","snapshotDigest":"codex-security-snapshot/v1:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"scope":{"includePaths":["."],"excludePaths":[]},"coverageRef":"coverage.json","findingsRef":"findings.json","artifacts":[]}}"#,
        )
        .unwrap();
        std::fs::write(
            scan.join("findings.json"),
            r#"{"documentType":"codex-security.findings","schemaVersion":"1.0","scanId":"wa_test","findings":[]}"#,
        )
        .unwrap();
        std::fs::write(scan.join("report.md"), "# ok\n").unwrap();

        let row = register_scan(&conn, &scan).unwrap();
        assert_eq!(row.scan_id, "wa_test");
        let list = list_scans(&conn, 10).unwrap();
        assert_eq!(list.len(), 1);
    }
}
