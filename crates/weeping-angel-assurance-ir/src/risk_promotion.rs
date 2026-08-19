//! Promotion and dismissal records. The only path from a candidate to a [`crate::Risk`].

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::risk_candidate::{CorrelationKey, ScoreSuggestion};
use crate::{DismissalId, PrincipalRef, PromotionId, RiskCandidateId, RiskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionRecord {
    pub id: PromotionId,
    pub candidate_id: RiskCandidateId,
    pub correlation_key: CorrelationKey,
    pub risk_id: RiskId,
    pub principal: PrincipalRef,
    pub at: DateTime<Utc>,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methodology_inputs: Option<ScoreSuggestion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methodology_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DismissalRecord {
    pub id: DismissalId,
    pub candidate_id: RiskCandidateId,
    pub correlation_key: CorrelationKey,
    #[serde(default)]
    pub observation_identities: BTreeSet<String>,
    pub subject_key: String,
    pub scenario_key: String,
    pub principal: PrincipalRef,
    pub at: DateTime<Utc>,
    pub rationale: String,
}
