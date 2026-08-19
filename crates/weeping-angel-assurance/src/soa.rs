//! Statement of Applicability projection. Not a certification document.

use serde::{Deserialize, Serialize};
use weeping_angel_control_test::Effectiveness;
use weeping_angel_framework::load_framework_pack;

/// Generic three-state applicability consumed by SoA projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Applicability {
    Applicable,
    NotApplicable,
    Unresolved,
}

impl Applicability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Applicable => "applicable",
            Self::NotApplicable => "notApplicable",
            Self::Unresolved => "unresolved",
        }
    }

    fn from_pack(raw: &str, fallback_bool: Option<bool>) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "applicable" | "true" => Self::Applicable,
            "notapplicable" | "not-applicable" | "not_applicable" | "false" => Self::NotApplicable,
            "unresolved" | "manual" | "manualdeterminationrequired" => Self::Unresolved,
            "" => match fallback_bool {
                Some(true) => Self::Applicable,
                Some(false) => Self::NotApplicable,
                None => Self::Unresolved,
            },
            _ => Self::Unresolved,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementOfApplicability {
    pub framework: String,
    pub framework_version: String,
    pub entries: Vec<SoaEntry>,
    pub disclaimer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoaEntry {
    pub reference: String,
    pub applicability: Applicability,
    pub applicable: bool,
    pub applicability_rationale: String,
    pub implementation_state: String,
    pub automated_effectiveness: Option<Effectiveness>,
    pub manual_review_state: String,
    pub evidence: Vec<String>,
    pub exceptions: Vec<String>,
    pub mapped_controls: Vec<String>,
    pub notes: String,
}

pub fn project_soa_from_snapshot(
    snapshot: &crate::lineage::StatementOfApplicabilitySnapshot,
) -> StatementOfApplicability {
    snapshot.soa.clone()
}

pub fn project_soa(framework: &str, version: &str) -> StatementOfApplicability {
    // When a pinned StatementOfApplicabilitySnapshot is available, callers
    // should use project_soa_from_snapshot so historical SoA is not rewritten
    // by later pack edits. Digest identity lives on the snapshot, not live disk.
    let mapped = load_framework_pack(framework, version)
        .map(|pack| {
            pack.mappings
                .iter()
                .map(|m| {
                    (
                        m.from_requirement().as_str().to_string(),
                        m.to_control().as_str().to_string(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let pack_dir = weeping_angel_framework::pack::resolve_pack_dir(framework, version).ok();
    let mut entries = Vec::new();
    if let Some(dir) = pack_dir {
        let path = dir.join("applicability.toml");
        if let Ok(text) = std::fs::read_to_string(path)
            && let Ok(parsed) = toml::from_str::<toml::Value>(&text)
            && let Some(arr) = parsed.get("entry").and_then(|v| v.as_array())
        {
            for item in arr {
                let requirement = item
                    .get("requirement")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let raw_state = item
                    .get("applicability")
                    .or_else(|| item.get("state"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let fallback = item.get("applicable").and_then(|v| v.as_bool());
                let applicability = Applicability::from_pack(raw_state, fallback);
                // applicability rationale is preserved verbatim from the pack.
                let rationale = item
                    .get("applicability_rationale")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mapped_controls = mapped
                    .iter()
                    .filter(|(from, _)| from == &requirement)
                    .map(|(_, to)| to.clone())
                    .collect();
                entries.push(SoaEntry {
                    reference: item
                        .get("reference")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    applicability,
                    applicable: matches!(applicability, Applicability::Applicable),
                    applicability_rationale: rationale,
                    implementation_state: "assessed".into(),
                    automated_effectiveness: None,
                    manual_review_state: match applicability {
                        Applicability::Unresolved => "manual determination required".into(),
                        Applicability::NotApplicable => "not applicable".into(),
                        Applicability::Applicable => "pending".into(),
                    },
                    evidence: Vec::new(),
                    exceptions: Vec::new(),
                    mapped_controls,
                    notes: String::new(),
                });
            }
        }
    }
    StatementOfApplicability {
        framework: framework.into(),
        framework_version: version.into(),
        entries,
        disclaimer: "This Statement of Applicability projection is a readiness aid and is not certification.".into(),
    }
}
