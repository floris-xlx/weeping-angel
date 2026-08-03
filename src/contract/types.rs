//! Codex Security v1 document types (serde, camelCase JSON).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestDocument {
    pub document_type: String,
    pub schema_version: String,
    pub scan: ScanBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanBody {
    pub id: String,
    pub producer: Producer,
    pub status: String,
    pub started_at: String,
    pub completed_at: String,
    pub sealed_at: String,
    pub target: ScanTarget,
    pub scope: ScanScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threat_model: Option<ThreatModelSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardening: Option<HardeningRef>,
    pub coverage_ref: String,
    pub findings_ref: String,
    #[serde(default)]
    pub artifacts: Vec<ArtifactRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Producer {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTarget {
    pub kind: String,
    pub target_id: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanScope {
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifacts_reviewed: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limitations: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreatModelSummary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust_boundaries: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attacker_capabilities: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_objectives: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assumptions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardeningRef {
    pub portfolio_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRecord {
    pub path: String,
    pub sha256: String,
    pub media_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingsDocument {
    pub document_type: String,
    pub schema_version: String,
    pub scan_id: String,
    pub findings: Vec<SemanticFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticFinding {
    pub finding_id: String,
    pub occurrence_id: String,
    pub rule_id: String,
    pub identity: FindingIdentity,
    pub fingerprints: Fingerprints,
    pub title: String,
    pub summary: String,
    pub severity: SeverityBlock,
    pub confidence: ConfidenceBlock,
    pub taxonomy: Taxonomy,
    pub locations: Vec<CodeLocation>,
    pub remediation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_path: Option<Value>,
    pub provenance: Provenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writeup: Option<WriteupRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_evidence: Option<Vec<CodeEvidence>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remediation_tests: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preventive_controls: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "is_empty_object")]
    pub extensions: Value,
}

fn is_empty_object(v: &Value) -> bool {
    match v {
        Value::Object(m) => m.is_empty(),
        Value::Null => true,
        _ => false,
    }
}

impl Default for SemanticFinding {
    fn default() -> Self {
        Self {
            finding_id: String::new(),
            occurrence_id: String::new(),
            rule_id: String::new(),
            identity: FindingIdentity {
                anchor: String::new(),
                instance: None,
            },
            fingerprints: Fingerprints {
                algorithm: super::fingerprint::FINGERPRINT_ALGORITHM.into(),
                primary: String::new(),
            },
            title: String::new(),
            summary: String::new(),
            severity: SeverityBlock {
                level: "medium".into(),
                score: None,
                scoring_system: None,
                vector: None,
                rationale: None,
                change_conditions: None,
            },
            confidence: ConfidenceBlock {
                level: "medium".into(),
                rationale: String::new(),
            },
            taxonomy: Taxonomy {
                category: String::new(),
                cwe: vec![],
            },
            locations: vec![],
            remediation: String::new(),
            validation: None,
            attack_path: None,
            provenance: Provenance {
                source: "weeping-angel".into(),
            },
            writeup: None,
            code_evidence: None,
            root_cause: None,
            remediation_tests: None,
            preventive_controls: None,
            extensions: Value::Object(Default::default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingIdentity {
    pub anchor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fingerprints {
    pub algorithm: String,
    pub primary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeverityBlock {
    pub level: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scoring_system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_conditions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidenceBlock {
    pub level: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Taxonomy {
    pub category: String,
    pub cwe: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLocation {
    pub path: String,
    pub start_line: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteupRef {
    pub report_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEvidence {
    pub id: String,
    pub label: String,
    pub path: String,
    pub start_line: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub code: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageDocument {
    pub document_type: String,
    pub schema_version: String,
    pub scan_id: String,
    pub mode: String,
    pub completeness: String,
    pub inventory_strategy: String,
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub surfaces: Vec<CoverageSurface>,
    pub explicit_exclusions: Vec<ExplicitExclusion>,
    pub deferred: Vec<DeferredUnit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_questions: Vec<OpenQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageSurface {
    pub id: String,
    pub label: String,
    pub disposition: String,
    pub receipt_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_area: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplicitExclusion {
    pub pattern: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeferredUnit {
    pub id: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenQuestion {
    pub question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_up_prompt: Option<String>,
}
