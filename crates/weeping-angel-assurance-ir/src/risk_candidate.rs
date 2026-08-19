//! Risk candidate proposals. Distinct from the operational [`crate::Risk`] record.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::RiskMethodologyId;
use crate::{
    ASSURANCE_IR_SCHEMA, RiskCandidateId, RiskId, RiskScoreInput, ScoredRisk, SubjectKind,
};

/// Deterministic clustering identity: digest of sorted subjects + scenario key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CorrelationKey(String);

impl CorrelationKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CorrelationKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CandidateStatus {
    Proposed,
    ClusteredDuplicate,
    Promoted,
    Dismissed,
    Stale,
    Resurfaced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CandidateConfidence {
    Low,
    Medium,
    High,
}

/// Suggested CIA/family category. Not an ISO annex or clause.
///
/// When clustered members disagree, the survivor uses [`SuggestedRiskCategory::Other`]
/// with `"mixed"` (documented tie-break; not a model score).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SuggestedRiskCategory {
    Confidentiality,
    Integrity,
    Availability,
    Identity,
    Supplier,
    Vulnerability,
    Other(String),
}

impl SuggestedRiskCategory {
    pub fn mixed() -> Self {
        Self::Other("mixed".into())
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::Other(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return false;
                }
                let folded = trimmed.to_ascii_lowercase();
                !folded.starts_with("iso27001:") && !folded.contains("annex-a")
            }
            _ => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectRef {
    pub kind: SubjectKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRef {
    pub evidence_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collector_id: Option<String>,
}

/// Non-temporal observation identity. Excludes `collected_at` and run ids.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationIdentity {
    pub evidence_type: String,
    #[serde(default)]
    pub facts: BTreeMap<String, String>,
    #[serde(default)]
    pub narrative_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioProposal {
    pub scenario_key: String,
    pub title: String,
    pub narrative: String,
}

/// Optional methodology-shaped suggestion. Derived ratings come only from Prompt 05 `score_risk`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreSuggestion {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methodology_id: Option<RiskMethodologyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methodology_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<RiskScoreInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived: Option<ScoredRisk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskCandidate {
    pub id: RiskCandidateId,
    pub schema_version: String,
    pub status: CandidateStatus,
    pub correlation_key: CorrelationKey,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_lineage: Vec<SourceRef>,
    pub scenario_proposal: ScenarioProposal,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impacted_subjects: Vec<SubjectRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_observations: Vec<ObservationIdentity>,
    pub confidence: CandidateConfidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duplicate_candidate_ids: Vec<RiskCandidateId>,
    pub suggested_risk_category: SuggestedRiskCategory,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_suggestion: Option<ScoreSuggestion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matches_existing_risk_ids: Vec<RiskId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resulting_risk_id: Option<RiskId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_seen_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub stale: bool,
}

impl RiskCandidate {
    pub fn new(
        id: RiskCandidateId,
        correlation_key: CorrelationKey,
        scenario_proposal: ScenarioProposal,
        suggested_risk_category: SuggestedRiskCategory,
    ) -> Self {
        Self {
            id,
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            status: CandidateStatus::Proposed,
            correlation_key,
            source_lineage: Vec::new(),
            scenario_proposal,
            impacted_subjects: Vec::new(),
            supporting_observations: Vec::new(),
            confidence: CandidateConfidence::Low,
            duplicate_candidate_ids: Vec::new(),
            suggested_risk_category,
            score_suggestion: None,
            matches_existing_risk_ids: Vec::new(),
            resulting_risk_id: None,
            first_seen_at: None,
            last_seen_at: None,
            stale: false,
        }
    }
}
