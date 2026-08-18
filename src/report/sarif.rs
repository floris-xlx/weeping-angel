use anyhow::Result;
use serde_json::{Value, json};

use crate::finding::{ScanReport, Severity};
use crate::report::security_findings;

pub fn to_string(report: &ScanReport) -> Result<String> {
    let findings = security_findings(report);

    let rules: Vec<Value> = {
        let mut seen = std::collections::HashSet::new();
        let mut rules = Vec::new();
        for f in &findings {
            let key = format!("{}/{}", f.module, f.id);
            if !seen.insert(key.clone()) {
                continue;
            }
            let mut rule = json!({
                "id": key,
                "name": f.title,
                "shortDescription": { "text": f.title },
                "fullDescription": { "text": f.description },
                "defaultConfiguration": {
                    "level": sarif_level(f.severity)
                },
                "properties": {
                    "tags": ["security", f.module.as_str()],
                    "precision": "medium",
                }
            });
            if let Some(cwe) = &f.cwe {
                rule["properties"]["cwe"] = json!(cwe);
            }
            if let Some(rem) = &f.remediation {
                rule["help"] = json!({
                    "text": rem,
                    "markdown": rem
                });
            }
            rules.push(rule);
        }
        rules
    };

    let results: Vec<Value> = findings
        .iter()
        .map(|f| {
            let rule_id = format!("{}/{}", f.module, f.id);
            let fingerprint = simple_fingerprint(&f.module, &f.id, &f.url);
            let mut result = json!({
                "ruleId": rule_id,
                "level": sarif_level(f.severity),
                "message": { "text": format!("{} — {}", f.title, f.description) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": f.url }
                    }
                }],
                "partialFingerprints": {
                    "primaryLocationLineHash": fingerprint
                }
            });
            if let Some(cwe) = &f.cwe {
                result["properties"] = json!({ "cwe": cwe, "module": f.module });
            } else {
                result["properties"] = json!({ "module": f.module });
            }
            result
        })
        .collect();

    let repo = option_env!("CARGO_PKG_REPOSITORY")
        .filter(|s| !s.is_empty())
        .unwrap_or("https://github.com/floris-xlx/weeping-angel");

    let doc = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": report.tool,
                    "version": report.version,
                    "informationUri": repo,
                    "rules": rules
                }
            },
            "results": results,
            "invocations": [{
                "executionSuccessful": true,
                "commandLine": format!("weeping-angel scan {}", report.target),
            }]
        }]
    });

    Ok(serde_json::to_string_pretty(&doc)?)
}

fn sarif_level(s: Severity) -> &'static str {
    match s {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low | Severity::Info => "note",
    }
}

fn simple_fingerprint(module: &str, id: &str, url: &str) -> String {
    // Stable, non-crypto fingerprint for partialFingerprints
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    module.hash(&mut h);
    id.hash(&mut h);
    url.hash(&mut h);
    format!("{:016x}", h.finish())
}
