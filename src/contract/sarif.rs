//! SARIF 2.1.0 projection from sealed codex-security findings.

use anyhow::Result;
use serde_json::{json, Value};

use crate::contract::types::{FindingsDocument, ManifestDocument, SemanticFinding};

pub fn findings_to_sarif(
    findings: &FindingsDocument,
    manifest: &ManifestDocument,
    tool_version: &str,
) -> Result<String> {
    let mut rules: Vec<Value> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for f in &findings.findings {
        if !seen.insert(f.rule_id.clone()) {
            continue;
        }
        let mut rule = json!({
            "id": f.rule_id,
            "name": f.title,
            "shortDescription": { "text": f.title },
            "fullDescription": { "text": f.summary },
            "defaultConfiguration": {
                "level": sarif_level(&f.severity.level)
            },
            "properties": {
                "tags": ["security", f.taxonomy.category.as_str()],
                "precision": f.confidence.level,
            }
        });
        if !f.taxonomy.cwe.is_empty() {
            rule["properties"]["cwe"] = json!(f.taxonomy.cwe);
        }
        if !f.remediation.is_empty() {
            rule["help"] = json!({
                "text": f.remediation,
                "markdown": f.remediation
            });
        }
        rules.push(rule);
    }

    let results: Vec<Value> = findings
        .findings
        .iter()
        .map(finding_to_result)
        .collect();

    let doc = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "weeping-angel",
                    "version": tool_version,
                    "informationUri": "https://github.com/floris-xlx/weeping-angel",
                    "rules": rules
                }
            },
            "results": results,
            "invocations": [{
                "executionSuccessful": true,
                "commandLine": format!(
                    "weeping-angel scan-code/diff {}",
                    manifest.scan.target.display_name
                ),
            }],
            "properties": {
                "scanId": findings.scan_id,
                "targetId": manifest.scan.target.target_id,
                "mode": manifest.scan.scope.summary,
            }
        }]
    });

    Ok(serde_json::to_string_pretty(&doc)?)
}

fn finding_to_result(f: &SemanticFinding) -> Value {
    let locations: Vec<Value> = f
        .locations
        .iter()
        .map(|loc| {
            json!({
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": loc.path,
                        "uriBaseId": "%SRCROOT%"
                    },
                    "region": {
                        "startLine": loc.start_line,
                        "endLine": loc.end_line.unwrap_or(loc.start_line)
                    }
                }
            })
        })
        .collect();

    let fingerprint = f
        .fingerprints
        .primary
        .strip_prefix("codex-security/v1:sha256:")
        .unwrap_or(&f.fingerprints.primary);

    json!({
        "ruleId": f.rule_id,
        "level": sarif_level(&f.severity.level),
        "message": { "text": format!("{} — {}", f.title, f.summary) },
        "locations": locations,
        "partialFingerprints": {
            "primaryLocationLineHash": fingerprint
        },
        "fingerprints": {
            "weepingAngel/v1": f.fingerprints.primary
        },
        "properties": {
            "findingId": f.finding_id,
            "occurrenceId": f.occurrence_id,
            "confidence": f.confidence.level,
            "category": f.taxonomy.category,
            "cwe": f.taxonomy.cwe,
        }
    })
}

fn sarif_level(level: &str) -> &'static str {
    match level {
        "critical" | "high" => "error",
        "medium" => "warning",
        "low" => "note",
        _ => "none",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::types::*;

    #[test]
    fn sarif_contains_rule_and_result() {
        let findings = FindingsDocument {
            document_type: "codex-security.findings".into(),
            schema_version: "1.0".into(),
            scan_id: "s1".into(),
            findings: vec![SemanticFinding {
                finding_id: "csf_aaaaaaaaaaaaaaaaaaaaaaaa".into(),
                occurrence_id: "occ_bbbbbbbbbbbbbbbbbbbbbbbb".into(),
                rule_id: "sql-injection.format-fstring".into(),
                identity: FindingIdentity {
                    anchor: "sql-query-fstring-or-format".into(),
                    instance: None,
                },
                fingerprints: Fingerprints {
                    algorithm: "codex-security/v1".into(),
                    primary: "codex-security/v1:sha256:".to_string() + &"ab".repeat(32),
                },
                title: "SQL fstring".into(),
                summary: "bad".into(),
                severity: SeverityBlock {
                    level: "high".into(),
                    score: None,
                    scoring_system: None,
                    vector: None,
                    rationale: None,
                    change_conditions: None,
                },
                confidence: ConfidenceBlock {
                    level: "high".into(),
                    rationale: "taint".into(),
                },
                taxonomy: Taxonomy {
                    category: "sql-injection".into(),
                    cwe: vec!["CWE-89".into()],
                },
                locations: vec![CodeLocation {
                    path: "db.py".into(),
                    start_line: 3,
                    end_line: Some(3),
                    role: Some("sink".into()),
                }],
                remediation: "use params".into(),
                validation: None,
                attack_path: None,
                provenance: Provenance {
                    source: "test".into(),
                },
                writeup: None,
                code_evidence: None,
                root_cause: None,
                remediation_tests: None,
                preventive_controls: None,
                extensions: serde_json::json!({}),
            }],
        };
        let manifest = ManifestDocument {
            document_type: "codex-security.scan-manifest".into(),
            schema_version: "1.0".into(),
            scan: ScanBody {
                id: "s1".into(),
                producer: Producer {
                    name: "weeping-angel".into(),
                    version: "0.1.2".into(),
                },
                status: "completed".into(),
                started_at: String::new(),
                completed_at: String::new(),
                sealed_at: String::new(),
                target: ScanTarget {
                    kind: "directory_snapshot".into(),
                    target_id: "t".into(),
                    display_name: "toy".into(),
                    remote: None,
                    revision: None,
                    base_revision: None,
                    head_revision: None,
                    snapshot_digest: Some(
                        "codex-security-snapshot/v1:sha256:".to_string() + &"cd".repeat(32),
                    ),
                },
                scope: ScanScope {
                    include_paths: vec![".".into()],
                    exclude_paths: vec![],
                    summary: Some("test".into()),
                    artifacts_reviewed: None,
                    runtime_status: None,
                    validation_mode: None,
                    context: None,
                    limitations: None,
                },
                threat_model: None,
                hardening: None,
                coverage_ref: "coverage.json".into(),
                findings_ref: "findings.json".into(),
                artifacts: vec![],
            },
        };
        let s = findings_to_sarif(&findings, &manifest, "0.1.2").unwrap();
        assert!(s.contains("sql-injection.format-fstring"));
        assert!(s.contains("db.py"));
        assert!(s.contains("partialFingerprints"));
    }
}
