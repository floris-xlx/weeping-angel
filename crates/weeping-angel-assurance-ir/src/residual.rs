//! Versioned residual-risk projection documents (Prompt 09).
//!
//! Residual is an explainable projection over pinned snapshots, not a live
//! score and not a field on [`crate::Risk`]. Callers supply
//! `ControlTestResult` / `Effectiveness` from `weeping-angel-control-test`;
//! this module does not fork those types and does not implement the Prompt
//! 05/06/08 engines.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ControlId, ExceptionId, PrincipalRef, ResidualRiskId, RiskId};

/// Built-in calculated methodology: effectiveness never lowers residual.
pub const NO_REDUCTION_METHODOLOGY_ID: &str = "residual-methodology:no-reduction";
/// Built-in calculated methodology: versioned control-effectiveness steps.
pub const CONTROL_EFFECTIVENESS_METHODOLOGY_ID: &str = "residual-methodology:control-effectiveness";
/// Only shipped methodology version. Unknown versions fail closed.
pub const RESIDUAL_METHODOLOGY_V1: &str = "v1";
/// Mandatory floor for `control-effectiveness/v1`. `Effective` is never zero.
pub const MIN_RESIDUAL_FLOOR: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResidualRiskMode {
    Calculated,
    Assessed,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TreatmentCompleteness {
    None,
    Partial,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InherentRiskRef {
    pub risk_id: RiskId,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InherentRiskSnapshot {
    pub pin: InherentRiskRef,
    pub rating_id: String,
    pub ordinal: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentPlanRef {
    pub plan_id: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreatmentPlanSnapshot {
    pub pin: TreatmentPlanRef,
    #[serde(default)]
    pub relevant_control_ids: Vec<ControlId>,
    pub completeness: TreatmentCompleteness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodologyRef {
    pub methodology_id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlTestSnapshotRef {
    pub digest: String,
    #[serde(default)]
    pub result_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualResidualAssessment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<PrincipalRef>,
    pub residual_ordinal: u32,
    pub residual_rating_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidualReductionStep {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_id: Option<ControlId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectiveness: Option<String>,
    pub step: u32,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidualRiskProjection {
    pub id: ResidualRiskId,
    pub risk_id: RiskId,
    pub mode: ResidualRiskMode,
    pub inherent: InherentRiskRef,
    pub treatment: TreatmentPlanRef,
    pub methodology: MethodologyRef,
    #[serde(default)]
    pub relevant_control_ids: Vec<ControlId>,
    pub control_tests: ControlTestSnapshotRef,
    pub projected_at: DateTime<Utc>,
    pub residual_ordinal: u32,
    pub residual_rating_id: String,
    #[serde(default)]
    pub reduction_trace: Vec<ResidualReductionStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual: Option<ManualResidualAssessment>,
    #[serde(default)]
    pub exception_ids: Vec<ExceptionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResidualRiskError {
    #[error("missing inherent-risk version")]
    MissingInherentRiskVersion,
    #[error("missing treatment-plan version")]
    MissingTreatmentPlanVersion,
    #[error("missing methodology version")]
    MissingMethodologyVersion,
    #[error("unknown methodology")]
    UnknownMethodology,
    #[error("missing control-test snapshot")]
    MissingControlTestSnapshot,
    #[error("dangling control")]
    DanglingControl,
    #[error("insufficient evidence")]
    InsufficientEvidence,
    #[error("not tested")]
    NotTested,
    #[error("stale evidence")]
    StaleEvidence,
    #[error("missing manual assessment")]
    MissingManualAssessment,
    #[error("missing management assessment")]
    MissingManagementAssessment,
    #[error("not applicable")]
    NotApplicableContradiction,
    #[error("manual review required")]
    ManualReviewRequired,
    #[error("inconclusive")]
    Inconclusive,
}

impl MethodologyRef {
    pub fn is_known_v1(&self) -> bool {
        self.version == RESIDUAL_METHODOLOGY_V1
            && matches!(
                self.methodology_id.as_str(),
                NO_REDUCTION_METHODOLOGY_ID | CONTROL_EFFECTIVENESS_METHODOLOGY_ID
            )
    }

    pub fn is_no_reduction_v1(&self) -> bool {
        self.methodology_id == NO_REDUCTION_METHODOLOGY_ID
            && self.version == RESIDUAL_METHODOLOGY_V1
    }

    pub fn is_control_effectiveness_v1(&self) -> bool {
        self.methodology_id == CONTROL_EFFECTIVENESS_METHODOLOGY_ID
            && self.version == RESIDUAL_METHODOLOGY_V1
    }
}

impl ResidualRiskProjection {
    pub fn rating_for_ordinal(ordinal: u32, inherent: &InherentRiskSnapshot) -> String {
        if ordinal >= inherent.ordinal {
            return inherent.rating_id.clone();
        }
        match ordinal {
            0 => "none".into(),
            1 => "low".into(),
            2 => "medium".into(),
            3 => "elevated".into(),
            _ => inherent.rating_id.clone(),
        }
    }
}
