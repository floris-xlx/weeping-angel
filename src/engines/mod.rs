//! Algorithmic detection engines (code SAST + adapters).

pub mod authz_routes;
pub mod code_scan;
pub mod cmd_injection;
pub mod depcheck_engine;
pub mod git_diff;
pub mod path_traversal;
pub mod secrets_code;
pub mod security_md;
pub mod sql_injection;
pub mod ssrf;
pub mod taint_lite;
pub mod web_adapt;
pub mod xss_template;

use serde_json::json;

use crate::contract::{
    Candidate, CandidateLocation, CodeLocation, ConfidenceBlock, FindingIdentity, Provenance,
    SemanticFinding, SeverityBlock, Taxonomy,
};

/// Intermediate hit from a rule pack (before ledger merge).
#[derive(Debug, Clone)]
pub struct EngineHit {
    pub rule_id: String,
    pub anchor: String,
    pub instance: Option<String>,
    pub title: String,
    pub summary: String,
    pub evidence: String,
    pub severity: &'static str,
    pub confidence: &'static str,
    pub confidence_rationale: String,
    pub category: String,
    pub cwe: Vec<String>,
    pub remediation: String,
    pub path: String,
    pub start_line: u32,
    pub end_line: Option<u32>,
    pub role: &'static str,
    pub snippet: String,
    /// Filled by taint_lite enrichment.
    pub validation_json: Option<serde_json::Value>,
    pub attack_path_json: Option<serde_json::Value>,
}

impl EngineHit {
    pub fn to_raw_candidate(&self) -> serde_json::Value {
        let mut loc = json!({
            "path": self.path,
            "start_line": self.start_line,
            "role": self.role,
        });
        if let Some(end) = self.end_line {
            loc["end_line"] = json!(end);
        }
        let mut row = json!({
            "cwe_ids": self.cwe,
            "locations": [loc],
            "summary": self.summary,
            "evidence": self.evidence,
        });
        if let Some(inst) = &self.instance {
            row["instance"] = json!(inst);
        }
        row
    }

    pub fn to_semantic_finding(&self) -> SemanticFinding {
        let mut finding = SemanticFinding::default();
        finding.rule_id = self.rule_id.clone();
        finding.identity = FindingIdentity {
            anchor: self.anchor.clone(),
            instance: self.instance.clone(),
        };
        finding.title = self.title.clone();
        finding.summary = self.summary.clone();
        finding.severity = SeverityBlock {
            level: self.severity.into(),
            score: None,
            scoring_system: None,
            vector: None,
            rationale: Some(format!(
                "Algorithmic rule {} matched source evidence.",
                self.rule_id
            )),
            change_conditions: None,
        };
        finding.confidence = ConfidenceBlock {
            level: self.confidence.into(),
            rationale: self.confidence_rationale.clone(),
        };
        finding.taxonomy = Taxonomy {
            category: self.category.clone(),
            cwe: self.cwe.clone(),
        };
        finding.locations = vec![CodeLocation {
            path: self.path.clone(),
            start_line: self.start_line,
            end_line: self.end_line,
            role: Some(self.role.into()),
        }];
        finding.remediation = self.remediation.clone();
        finding.provenance = Provenance {
            source: "weeping-angel-engine".into(),
        };
        finding.validation = self.validation_json.clone().or_else(|| {
            Some(json!({
                "disposition": "reportable",
                "method": "static-pattern",
                "confidence": self.confidence,
                "confidence_rationale": self.confidence_rationale,
                "rubric": ["pattern match on known dangerous sink or secret form"],
                "evidence": self.evidence,
                "counterevidence_or_proof_gap": "No dynamic reproduction; static pattern only.",
                "remaining_uncertainty": "May be dead code, test-only, or mitigated by unmodeled controls.",
            }))
        });
        finding.attack_path = self.attack_path_json.clone().or_else(|| {
            Some(json!({
                "decision": "reportable",
                "dataflow": {
                    "source": "attacker-controlled or embedded secret material",
                    "sink": self.rule_id,
                    "narrative": self.summary,
                },
                "reachability": {
                    "attacker": "depends on product surface",
                    "preconditions": "code path reachable in supported deployment",
                    "narrative": "Static match; product reachability not fully proven.",
                },
                "severity": self.severity,
                "severity_rationale": "Rule pack default severity for this family.",
            }))
        });
        finding.extensions = json!({
            "engine": "algorithmic",
            "snippet": self.snippet,
            "validationMethod": self.validation_json.as_ref()
                .and_then(|v| v.get("method"))
                .cloned()
                .unwrap_or(json!("static-pattern")),
        });
        finding
    }
}

pub fn hit_to_candidate(hit: &EngineHit, candidate_id: String) -> Candidate {
    Candidate {
        candidate_id,
        cwe_ids: hit.cwe.clone(),
        locations: vec![CandidateLocation {
            path: hit.path.clone(),
            start_line: hit.start_line,
            end_line: hit.end_line,
            role: Some(hit.role.into()),
        }],
        summary: hit.summary.clone(),
        evidence: hit.evidence.clone(),
        context: None,
        instance: hit.instance.clone(),
        validation: hit.to_semantic_finding().validation.clone(),
        attack_path: hit.to_semantic_finding().attack_path.clone(),
    }
}

/// Scan one file's text with all code rule packs.
pub fn scan_source_file(rel_path: &str, content: &str) -> Vec<EngineHit> {
    let mut hits = Vec::new();
    hits.extend(path_traversal::scan(rel_path, content));
    hits.extend(cmd_injection::scan(rel_path, content));
    hits.extend(secrets_code::scan(rel_path, content));
    hits.extend(sql_injection::scan(rel_path, content));
    hits.extend(ssrf::scan(rel_path, content));
    hits.extend(xss_template::scan(rel_path, content));
    hits.extend(authz_routes::scan(rel_path, content));
    hits
}

/// Rank severity strings for fail-on comparisons (higher = worse).
pub fn severity_rank(level: &str) -> u8 {
    match level {
        "critical" => 5,
        "high" => 4,
        "medium" => 3,
        "low" => 2,
        "informational" | "info" => 1,
        _ => 0,
    }
}

/// True if any finding severity is at or above the fail_on threshold.
pub fn findings_meet_fail_on(max_finding: &str, fail_on: &str) -> bool {
    severity_rank(max_finding) >= severity_rank(fail_on) && severity_rank(fail_on) > 0
}

/// Max file size read for static engines (bytes).
pub const MAX_ENGINE_FILE_BYTES: u64 = 1_500_000;
