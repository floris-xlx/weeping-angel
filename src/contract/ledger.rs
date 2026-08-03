//! Candidate ledger normalize/merge (compact standard-scan JSONL).

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateLocation {
    pub path: String,
    pub start_line: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Normalized durable candidate row (discovery fields only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub candidate_id: String,
    pub cwe_ids: Vec<String>,
    pub locations: Vec<CandidateLocation>,
    pub summary: String,
    pub evidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// Compact validation (added after discovery; never re-normalized).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<Value>,
    /// Compact attack_path (added after validation for reportable/deferred).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_path: Option<Value>,
}

/// Normalize one raw discovery candidate (no candidate_id yet).
pub fn normalize_raw_candidate(
    row: &Value,
    scope: &BTreeSet<String>,
) -> Result<Candidate> {
    let obj = row
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("candidate row must be an object"))?;

    let allowed = [
        "cwe_ids",
        "locations",
        "summary",
        "evidence",
        "context",
        "instance",
        "candidate_id",
    ];
    for k in obj.keys() {
        if !allowed.contains(&k.as_str()) {
            bail!("unsupported field `{k}` in raw candidate");
        }
    }

    let cwe_ids = obj
        .get("cwe_ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    for cwe in &cwe_ids {
        if !cwe.starts_with("CWE-") {
            bail!("cwe_ids entry must look like CWE-N, got `{cwe}`");
        }
    }

    let locations = parse_locations(obj.get("locations"))?;
    if !locations.iter().any(|l| scope.contains(&l.path)) {
        bail!("locations: expected at least one in-scope file");
    }

    let summary = require_str(obj, "summary")?;
    let evidence = require_str(obj, "evidence")?;
    let context = optional_str(obj, "context");
    let instance = optional_str(obj, "instance");

    Ok(Candidate {
        candidate_id: String::new(),
        cwe_ids,
        locations,
        summary,
        evidence,
        context,
        instance,
        validation: None,
        attack_path: None,
    })
}

fn require_str(obj: &serde_json::Map<String, Value>, key: &str) -> Result<String> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("{key}: expected non-empty string"))
}

fn optional_str(obj: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn parse_locations(value: Option<&Value>) -> Result<Vec<CandidateLocation>> {
    let arr = value
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("locations: expected array"))?;
    if arr.is_empty() {
        bail!("locations: expected at least one location");
    }
    let mut out = Vec::with_capacity(arr.len());
    for (i, item) in arr.iter().enumerate() {
        let o = item
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("locations[{i}]: expected object"))?;
        let path = o
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("locations[{i}].path required"))?
            .replace('\\', "/");
        let start_line = o
            .get("start_line")
            .or_else(|| o.get("startLine"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("locations[{i}].start_line required"))?
            as u32;
        if start_line < 1 {
            bail!("locations[{i}].start_line must be >= 1");
        }
        let end_line = o
            .get("end_line")
            .or_else(|| o.get("endLine"))
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);
        let role = o
            .get("role")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        out.push(CandidateLocation {
            path,
            start_line,
            end_line,
            role,
        });
    }
    Ok(out)
}

fn identity_key(c: &Candidate) -> String {
    let mut payload = serde_json::json!({
        "cwe_ids": c.cwe_ids,
        "locations": c.locations,
        "instance": c.instance,
    });
    // Stable serialization
    if let Some(obj) = payload.as_object_mut() {
        // already structured
        let _ = obj;
    }
    serde_json::to_string(&payload).unwrap_or_default()
}

/// Merge candidates with the same CWE/locations/instance; assign candidate_id.
pub fn combine_candidates(rows: Vec<Candidate>) -> Vec<Candidate> {
    let mut groups: BTreeMap<String, Vec<Candidate>> = BTreeMap::new();
    for row in rows {
        groups.entry(identity_key(&row)).or_default().push(row);
    }
    let mut combined = Vec::new();
    for (key, group) in groups {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let digest = hex::encode(hasher.finalize());
        let candidate_id = format!("candidate-{}", &digest[..16]);

        let mut summaries = BTreeSet::new();
        let mut evidences = BTreeSet::new();
        let mut contexts = BTreeSet::new();
        for g in &group {
            summaries.insert(g.summary.clone());
            evidences.insert(g.evidence.clone());
            if let Some(c) = &g.context {
                contexts.insert(c.clone());
            }
        }
        let first = &group[0];
        combined.push(Candidate {
            candidate_id,
            cwe_ids: first.cwe_ids.clone(),
            locations: first.locations.clone(),
            summary: summaries.into_iter().collect::<Vec<_>>().join("\n"),
            evidence: evidences.into_iter().collect::<Vec<_>>().join("\n"),
            context: if contexts.is_empty() {
                None
            } else {
                Some(contexts.into_iter().collect::<Vec<_>>().join("\n"))
            },
            instance: first.instance.clone(),
            validation: None,
            attack_path: None,
        });
    }
    combined
}

pub fn read_scope_file(path: &Path) -> Result<BTreeSet<String>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut scope = BTreeSet::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        let line = line.trim().trim_start_matches("./").replace('\\', "/");
        if !line.is_empty() {
            scope.insert(line);
        }
    }
    Ok(scope)
}

pub fn write_candidate_ledger(path: &Path, candidates: &[Candidate]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path).with_context(|| format!("create {}", path.display()))?;
    for c in candidates {
        let line = serde_json::to_string(c)?;
        writeln!(file, "{line}")?;
    }
    Ok(())
}

pub fn read_candidate_ledger(path: &Path) -> Result<Vec<Candidate>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let c: Candidate = serde_json::from_str(&line)
            .with_context(|| format!("{}:{} invalid candidate JSON", path.display(), i + 1))?;
        out.push(c);
    }
    Ok(out)
}

/// Load raw JSONL candidates, normalize against scope, combine, write ledger.
pub fn build_ledger_from_raw_inputs(
    inputs: &[PathBuf],
    scope_file: &Path,
    out: &Path,
) -> Result<Vec<Candidate>> {
    let scope = read_scope_file(scope_file)?;
    let mut rows = Vec::new();
    for input in inputs {
        let file = File::open(input).with_context(|| format!("open {}", input.display()))?;
        for (i, line) in BufReader::new(file).lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let value: Value = serde_json::from_str(&line)
                .with_context(|| format!("{}:{} bad JSON", input.display(), i + 1))?;
            rows.push(
                normalize_raw_candidate(&value, &scope)
                    .with_context(|| format!("{}:{}", input.display(), i + 1))?,
            );
        }
    }
    let combined = combine_candidates(rows);
    write_candidate_ledger(out, &combined)?;
    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn combine_merges_same_identity() {
        let mut scope = BTreeSet::new();
        scope.insert("src/a.py".into());
        let raw = json!({
            "cwe_ids": ["CWE-22"],
            "locations": [{"path": "src/a.py", "start_line": 10, "role": "sink"}],
            "summary": "path issue",
            "evidence": "join without check"
        });
        let a = normalize_raw_candidate(&raw, &scope).unwrap();
        let mut b = a.clone();
        b.summary = "also path issue".into();
        let combined = combine_candidates(vec![a, b]);
        assert_eq!(combined.len(), 1);
        assert!(combined[0].candidate_id.starts_with("candidate-"));
        assert!(combined[0].summary.contains("path issue"));
    }
}
