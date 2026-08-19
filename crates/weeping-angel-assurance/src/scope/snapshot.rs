//! Deterministic scope-resolution snapshot (`weeping-angel/scope-resolution/v1`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{
    AssessmentId, PrincipalRef, ScopeId, SubjectKind, canonical_digest,
};

pub const SCOPE_RESOLUTION_SCHEMA: &str = "weeping-angel/scope-resolution/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScopeDecision {
    InScope,
    OutOfScope,
    Conditional,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InfluencingRuleClass {
    Inclusion,
    Exclusion,
    Inheritance,
    Organization,
    InvalidExclusion,
    ExpiredExclusion,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineageHop {
    pub kind: SubjectKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfluencingRule {
    pub class: InfluencingRuleClass,
    pub rank: u16,
    pub selector_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclusion_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_by: Option<DateTime<Utc>>,
    pub applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectScopeDecision {
    pub kind: SubjectKind,
    pub id: String,
    pub decision: ScopeDecision,
    pub rationale: String,
    pub lineage: Vec<LineageHop>,
    pub explain: String,
    pub influencing_rules: Vec<InfluencingRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeResolution {
    pub schema: String,
    pub assessment_id: AssessmentId,
    pub as_of: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<ScopeId>,
    pub subjects: Vec<SubjectScopeDecision>,
    pub digest: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScopeResolutionDigestBody<'a> {
    schema: &'a str,
    assessment_id: &'a AssessmentId,
    as_of: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope_id: Option<&'a ScopeId>,
    subjects: &'a [SubjectScopeDecision],
}

impl ScopeResolution {
    pub(crate) fn seal(
        assessment_id: AssessmentId,
        as_of: DateTime<Utc>,
        scope_id: Option<ScopeId>,
        mut subjects: Vec<SubjectScopeDecision>,
    ) -> Self {
        subjects.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.kind.cmp(&b.kind)));
        let schema = SCOPE_RESOLUTION_SCHEMA.to_string();
        let body = ScopeResolutionDigestBody {
            schema: SCOPE_RESOLUTION_SCHEMA,
            assessment_id: &assessment_id,
            as_of,
            scope_id: scope_id.as_ref(),
            subjects: &subjects,
        };
        let digest = canonical_digest(&body).unwrap_or_default();
        Self {
            schema,
            assessment_id,
            as_of,
            scope_id,
            subjects,
            digest,
        }
    }
}
