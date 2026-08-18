//! Compare two sealed scans by primary fingerprint.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct FindingLite {
    pub finding_id: String,
    pub fingerprint: String,
    pub rule_id: String,
    pub title: String,
    pub severity: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanCompare {
    pub before_scan_id: String,
    pub after_scan_id: String,
    pub before_count: usize,
    pub after_count: usize,
    pub introduced: Vec<FindingLite>,
    pub resolved: Vec<FindingLite>,
    pub persistent: Vec<FindingLite>,
    pub severity_introduced: BTreeMap<String, usize>,
    pub severity_resolved: BTreeMap<String, usize>,
}

pub fn load_findings(scan_dir: &Path) -> Result<(String, Vec<FindingLite>)> {
    let findings_path = scan_dir.join("findings.json");
    let manifest_path = scan_dir.join("scan-manifest.json");
    if !findings_path.is_file() {
        bail!("missing findings.json in {}", scan_dir.display());
    }
    let findings: Value = serde_json::from_str(
        &fs::read_to_string(&findings_path)
            .with_context(|| format!("read {}", findings_path.display()))?,
    )?;
    let scan_id = if manifest_path.is_file() {
        let m: Value = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
        m["scan"]["id"]
            .as_str()
            .or_else(|| findings["scanId"].as_str())
            .unwrap_or("unknown")
            .to_string()
    } else {
        findings["scanId"].as_str().unwrap_or("unknown").to_string()
    };

    let mut out = Vec::new();
    if let Some(arr) = findings["findings"].as_array() {
        for f in arr {
            let fingerprint = f["fingerprints"]["primary"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if fingerprint.is_empty() {
                continue;
            }
            let path = f["locations"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|l| l["path"].as_str())
                .map(str::to_string);
            out.push(FindingLite {
                finding_id: f["findingId"].as_str().unwrap_or("").into(),
                fingerprint,
                rule_id: f["ruleId"].as_str().unwrap_or("").into(),
                title: f["title"].as_str().unwrap_or("").into(),
                severity: f["severity"]["level"].as_str().unwrap_or("unknown").into(),
                path,
            });
        }
    }
    Ok((scan_id, out))
}

pub fn compare_scan_dirs(before_dir: &Path, after_dir: &Path) -> Result<ScanCompare> {
    let (before_id, before) = load_findings(before_dir)?;
    let (after_id, after) = load_findings(after_dir)?;
    compare_sets(&before_id, &before, &after_id, &after)
}

pub fn compare_sets(
    before_id: &str,
    before: &[FindingLite],
    after_id: &str,
    after: &[FindingLite],
) -> Result<ScanCompare> {
    let before_map: BTreeMap<_, _> = before
        .iter()
        .map(|f| (f.fingerprint.clone(), f.clone()))
        .collect();
    let after_map: BTreeMap<_, _> = after
        .iter()
        .map(|f| (f.fingerprint.clone(), f.clone()))
        .collect();

    let before_fps: BTreeSet<_> = before_map.keys().cloned().collect();
    let after_fps: BTreeSet<_> = after_map.keys().cloned().collect();

    let introduced: Vec<_> = after_fps
        .difference(&before_fps)
        .filter_map(|fp| after_map.get(fp).cloned())
        .collect();
    let resolved: Vec<_> = before_fps
        .difference(&after_fps)
        .filter_map(|fp| before_map.get(fp).cloned())
        .collect();
    let persistent: Vec<_> = before_fps
        .intersection(&after_fps)
        .filter_map(|fp| after_map.get(fp).cloned())
        .collect();

    Ok(ScanCompare {
        before_scan_id: before_id.into(),
        after_scan_id: after_id.into(),
        before_count: before.len(),
        after_count: after.len(),
        severity_introduced: count_severity(&introduced),
        severity_resolved: count_severity(&resolved),
        introduced,
        resolved,
        persistent,
    })
}

fn count_severity(items: &[FindingLite]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for i in items {
        *m.entry(i.severity.clone()).or_default() += 1;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_introduced_and_resolved() {
        let before = vec![FindingLite {
            finding_id: "a".into(),
            fingerprint: "fp1".into(),
            rule_id: "r1".into(),
            title: "old".into(),
            severity: "high".into(),
            path: Some("a.py".into()),
        }];
        let after = vec![FindingLite {
            finding_id: "b".into(),
            fingerprint: "fp2".into(),
            rule_id: "r2".into(),
            title: "new".into(),
            severity: "medium".into(),
            path: Some("b.py".into()),
        }];
        let c = compare_sets("s1", &before, "s2", &after).unwrap();
        assert_eq!(c.introduced.len(), 1);
        assert_eq!(c.resolved.len(), 1);
        assert!(c.persistent.is_empty());
    }
}
