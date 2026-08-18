//! Statement of Applicability projection. Not a certification document.

use serde::{Deserialize, Serialize};
use weeping_angel_control_test::Effectiveness;

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
    pub applicable: bool,
    pub applicability_rationale: String,
    pub implementation_state: String,
    pub automated_effectiveness: Option<Effectiveness>,
    pub manual_review_state: String,
    pub evidence: Vec<String>,
    pub exceptions: Vec<String>,
    pub notes: String,
}

pub fn project_soa(framework: &str, version: &str) -> StatementOfApplicability {
    let pack_dir = weeping_angel_framework::pack::resolve_pack_dir(framework, version).ok();
    let mut entries = Vec::new();
    if let Some(dir) = pack_dir {
        let path = dir.join("applicability.toml");
        if let Ok(text) = std::fs::read_to_string(path)
            && let Ok(parsed) = toml::from_str::<toml::Value>(&text)
            && let Some(arr) = parsed.get("entry").and_then(|v| v.as_array())
        {
            for item in arr {
                entries.push(SoaEntry {
                    reference: item
                        .get("reference")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    applicable: item
                        .get("applicable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    // applicability rationale is preserved verbatim from the pack.
                    applicability_rationale: item
                        .get("applicability_rationale")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    implementation_state: "assessed".into(),
                    automated_effectiveness: None,
                    manual_review_state: "pending".into(),
                    evidence: Vec::new(),
                    exceptions: Vec::new(),
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
