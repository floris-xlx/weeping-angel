//! Security-objective governance records. Value-bearing evaluation lives in
//! `weeping-angel-assurance`; payloads here are evidence-value/v1 JSON.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ASSURANCE_IR_SCHEMA;
use crate::assessment::AssessmentScope;
use crate::evidence::EvidenceCollectionKind;
use crate::id::{
    EvidenceRequirementId, EvidenceType, ObjectiveMetricId, ObjectiveTargetId, SecurityObjectiveId,
    validate_stable_id,
};
use crate::implementation::PrincipalRef;
use crate::validation::{IrValidationError, ValidateIr};

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn scope_is_empty(scope: &AssessmentScope) -> bool {
    scope.organizations.is_empty() && scope.subjects.is_empty()
}

/// Draft records may omit owner, start, and populated scope. Active may not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjectiveLifecycle {
    Draft,
    Active,
    Retired,
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MetricKind {
    Percentage,
    Count,
    Duration,
    Boolean,
    Ratio,
    BoundedNumeric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComparisonOp {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PopulationCompleteness {
    Authoritative,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveMeasurementSource {
    pub evidence_type: EvidenceType,
    pub collection: EvidenceCollectionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_requirement_id: Option<EvidenceRequirementId>,
}

/// First-class measurable objective. Status is never stored on this record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityObjective {
    pub id: SecurityObjectiveId,
    pub schema_version: String,
    pub title: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PrincipalRef>,
    pub scope: AssessmentScope,
    pub metric_id: ObjectiveMetricId,
    pub target_id: ObjectiveTargetId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<serde_json::Value>,
    pub measurement_source: ObjectiveMeasurementSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cadence_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_at: Option<DateTime<Utc>>,
    pub lifecycle: ObjectiveLifecycle,
    pub logical_id: String,
    pub revision: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<SecurityObjectiveId>,
}

impl SecurityObjective {
    pub fn try_new(
        id: SecurityObjectiveId,
        title: impl Into<String>,
        description: impl Into<String>,
        metric_id: ObjectiveMetricId,
        target_id: ObjectiveTargetId,
        measurement_source: ObjectiveMeasurementSource,
        logical_id: impl Into<String>,
        lifecycle: ObjectiveLifecycle,
    ) -> Result<Self, IrValidationError> {
        let objective = Self {
            id,
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            title: title.into(),
            description: description.into(),
            owner: None,
            scope: AssessmentScope::default(),
            metric_id,
            target_id,
            baseline: None,
            measurement_source,
            cadence_seconds: None,
            start_at: None,
            deadline_at: None,
            review_at: None,
            lifecycle,
            logical_id: logical_id.into(),
            revision: 1,
            supersedes: None,
        };
        objective.validate()?;
        Ok(objective)
    }

    pub fn supersede(&self, new_id: SecurityObjectiveId) -> Self {
        let mut next = self.clone();
        next.supersedes = Some(self.id.clone());
        next.id = new_id;
        next.revision = self.revision.saturating_add(1);
        next.lifecycle = ObjectiveLifecycle::Active;
        next
    }
}

impl ValidateIr for SecurityObjective {
    fn validate(&self) -> Result<(), IrValidationError> {
        if self.schema_version != ASSURANCE_IR_SCHEMA {
            return Err(IrValidationError::Message(format!(
                "schema version mismatch: expected {ASSURANCE_IR_SCHEMA}, got {}",
                self.schema_version
            )));
        }
        if is_blank(&self.title) {
            return Err(IrValidationError::Message(format!(
                "empty title on security objective {}",
                self.id
            )));
        }
        if is_blank(&self.description) {
            return Err(IrValidationError::Message(format!(
                "empty description on security objective {}",
                self.id
            )));
        }
        if is_blank(&self.logical_id) {
            return Err(IrValidationError::Message(format!(
                "empty logicalId on security objective {}",
                self.id
            )));
        }
        if let Err(err) = validate_stable_id(&self.logical_id) {
            return Err(IrValidationError::Message(format!(
                "invalid logicalId on security objective {}: {err}",
                self.id
            )));
        }
        if self.revision < 1 {
            return Err(IrValidationError::Message(format!(
                "revision must start at 1 on security objective {}",
                self.id
            )));
        }
        if self.supersedes.as_ref() == Some(&self.id) {
            return Err(IrValidationError::Message(format!(
                "self-supersedes is invalid on security objective {}",
                self.id
            )));
        }
        if let Some(cadence) = self.cadence_seconds
            && cadence == 0
        {
            return Err(IrValidationError::Message(format!(
                "zero cadenceSeconds on security objective {}",
                self.id
            )));
        }
        if let (Some(start), Some(deadline)) = (self.start_at, self.deadline_at)
            && deadline < start
        {
            return Err(IrValidationError::Message(format!(
                "deadlineAt is before startAt on security objective {}",
                self.id
            )));
        }

        if self.lifecycle == ObjectiveLifecycle::Active {
            if self.owner.is_none() {
                return Err(IrValidationError::Message(format!(
                    "active security objective {} is missing owner",
                    self.id
                )));
            }
            if scope_is_empty(&self.scope) {
                return Err(IrValidationError::Message(format!(
                    "active security objective {} has empty scope",
                    self.id
                )));
            }
            if self.start_at.is_none() {
                return Err(IrValidationError::Message(format!(
                    "active security objective {} is missing startAt",
                    self.id
                )));
            }
        }
        Ok(())
    }
}
