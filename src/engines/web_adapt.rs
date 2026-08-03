//! Map live web DAST findings into Codex-compatible SemanticFinding records.

use serde_json::json;

use crate::contract::{
    CodeLocation, ConfidenceBlock, FindingIdentity, Provenance, SemanticFinding, SeverityBlock,
    Taxonomy,
};
use crate::finding::{Finding, Severity};

pub fn web_finding_to_semantic(f: &Finding) -> SemanticFinding {
    let level = match f.severity {
        Severity::Critical => "critical",
        Severity::High => "high",
        Severity::Medium => "medium",
        Severity::Low => "low",
        Severity::Info => "informational",
    };
    let rule_id = format!(
        "web.{}.{}",
        slug(&f.module),
        slug(&f.id)
    );
    let anchor = slug(&format!("{}-{}", f.module, f.id));
    let path_hint = f
        .evidence
        .first()
        .map(|e| e.location.clone())
        .unwrap_or_else(|| f.url.clone());

    let mut finding = SemanticFinding::default();
    finding.rule_id = rule_id;
    finding.identity = FindingIdentity {
        anchor,
        instance: Some(slug(&f.url)),
    };
    finding.title = f.title.clone();
    finding.summary = f.description.clone();
    finding.severity = SeverityBlock {
        level: level.into(),
        score: None,
        scoring_system: None,
        vector: None,
        rationale: Some("Derived from weeping-angel live web scan evidence.".into()),
        change_conditions: None,
    };
    finding.confidence = ConfidenceBlock {
        level: if f.severity >= Severity::High {
            "high".into()
        } else {
            "medium".into()
        },
        rationale: "Live HTTP response evidence from authorized scan.".into(),
    };
    finding.taxonomy = Taxonomy {
        category: format!("web-{}", slug(&f.module)),
        cwe: f
            .cwe
            .as_ref()
            .map(|c| vec![c.clone()])
            .unwrap_or_default(),
    };
    finding.locations = vec![CodeLocation {
        path: path_hint,
        start_line: 1,
        end_line: None,
        role: Some("entrypoint".into()),
    }];
    finding.remediation = f
        .remediation
        .clone()
        .unwrap_or_else(|| "Remediate according to the finding description.".into());
    finding.provenance = Provenance {
        source: "weeping-angel-web".into(),
    };
    finding.validation = Some(json!({
        "disposition": "reportable",
        "method": "http-probe",
        "confidence": if f.severity >= Severity::High { "high" } else { "medium" },
        "confidence_rationale": "Observed over the network against an allowlisted target.",
        "evidence": f.evidence.iter().map(|e| format!("{}: {}", e.location, e.snippet)).collect::<Vec<_>>(),
    }));
    finding.attack_path = Some(json!({
        "decision": "reportable",
        "dataflow": {
            "source": "remote HTTP client",
            "sink": f.module,
            "narrative": f.description,
        },
        "reachability": {
            "attacker": "network client in scope",
            "entry_point": f.url,
            "narrative": "Live scan reached this URL under authorized consent.",
        },
        "severity": level,
    }));
    finding.extensions = json!({
        "module": f.module,
        "url": f.url,
        "webFindingId": f.id,
    });
    finding
}

fn slug(s: &str) -> String {
    // Codex ruleId/identity: lowercase slug [a-z0-9._/-]
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if matches!(c, '.' | '_' | '/') {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "item".into()
    } else if trimmed
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric())
    {
        trimmed
    } else {
        format!("x{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn maps_web_finding() {
        let f = Finding {
            id: "missing-csp".into(),
            title: "Missing CSP".into(),
            severity: Severity::Medium,
            url: "https://example.com/".into(),
            module: "headers".into(),
            description: "No Content-Security-Policy".into(),
            remediation: Some("Add CSP".into()),
            cwe: Some("CWE-693".into()),
            evidence: vec![],
            found_at: Utc::now(),
        };
        let s = web_finding_to_semantic(&f);
        assert!(s.rule_id.starts_with("web.headers."));
        assert_eq!(s.severity.level, "medium");
    }
}
