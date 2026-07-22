use anyhow::Result;
use serde_json::{json, Value};

use crate::finding::{ScanReport, Severity};

pub fn to_string(report: &ScanReport) -> Result<String> {
    let rules: Vec<Value> = {
        let mut seen = std::collections::HashSet::new();
        let mut rules = Vec::new();
        for f in &report.findings {
            let key = format!("{}:{}", f.module, f.id);
            if !seen.insert(key.clone()) {
                continue;
            }
            rules.push(json!({
                "id": key,
                "name": f.title,
                "shortDescription": { "text": f.title },
                "fullDescription": { "text": f.description },
                "defaultConfiguration": {
                    "level": sarif_level(f.severity)
                },
                "properties": {
                    "tags": ["security", f.module],
                    "cwe": f.cwe,
                }
            }));
        }
        rules
    };

    let results: Vec<Value> = report
        .findings
        .iter()
        .map(|f| {
            json!({
                "ruleId": format!("{}:{}", f.module, f.id),
                "level": sarif_level(f.severity),
                "message": { "text": format!("{} — {}", f.title, f.description) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": f.url }
                    }
                }]
            })
        })
        .collect();

    let doc = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": report.tool,
                    "version": report.version,
                    "informationUri": "https://github.com/weeping-angel/weeping-angel",
                    "rules": rules
                }
            },
            "results": results
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
