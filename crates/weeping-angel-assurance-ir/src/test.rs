//! Planned control-test definition. Predicate AST lives in the control-test crate.

use serde::{Deserialize, Serialize};

use crate::{
    ASSURANCE_IR_SCHEMA, ControlId, ControlTestId, EvidenceRequirementId, EvidenceType,
    FreshnessRequirement, SubjectSelector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PlannedTestKind {
    #[default]
    Automated,
    Manual,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TestFailureSeverity {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TestEvaluationRef {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedControlTest {
    schema_version: String,
    pub id: ControlTestId,
    pub control_id: ControlId,
    pub kind: PlannedTestKind,
    #[serde(default)]
    pub required_evidence: Vec<EvidenceType>,
    #[serde(default)]
    pub break_on: Vec<EvidenceType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subjects: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_requirements: Vec<EvidenceRequirementId>,
    #[serde(default, skip_serializing_if = "evaluation_empty")]
    pub evaluation: TestEvaluationRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freshness_policy: Option<FreshnessRequirement>,
    #[serde(default, skip_serializing_if = "is_medium")]
    pub severity: TestFailureSeverity,
    /// Lossless JSON `TestExpr` projected from catalog `[test.expression]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expr: Option<serde_json::Value>,
}

fn evaluation_empty(value: &TestEvaluationRef) -> bool {
    value.id.is_empty()
}

fn is_medium(value: &TestFailureSeverity) -> bool {
    matches!(value, TestFailureSeverity::Medium)
}

impl PlannedControlTest {
    pub fn new(id: ControlTestId, control_id: ControlId) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            control_id,
            kind: PlannedTestKind::Automated,
            required_evidence: Vec::new(),
            break_on: Vec::new(),
            subjects: Vec::new(),
            evidence_requirements: Vec::new(),
            evaluation: TestEvaluationRef::default(),
            freshness_policy: None,
            severity: TestFailureSeverity::Medium,
            expr: None,
        }
    }

    pub fn with_kind(mut self, kind: PlannedTestKind) -> Self {
        self.kind = kind;
        self
    }
}
