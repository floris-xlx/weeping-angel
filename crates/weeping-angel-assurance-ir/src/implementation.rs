//! Organizational implementation state. Not control effectiveness.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    ASSURANCE_IR_SCHEMA, ControlId, ControlImplementationId, ExceptionId, IdentityId, RiskId,
    SubjectSelector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ImplementationStatus {
    #[default]
    NotImplemented,
    Planned,
    PartiallyImplemented,
    Implemented,
    NotApplicable,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrincipalRef {
    Identity(IdentityId),
    Team(String),
    Role(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlImplementation {
    schema_version: String,
    id: ControlImplementationId,
    control_id: ControlId,
    status: ImplementationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    implemented_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    applies_to: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    compensating_controls: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    exception_ids: Vec<ExceptionId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    risk_ids: Vec<RiskId>,
}

impl ControlImplementation {
    pub fn new(id: ControlImplementationId, control_id: ControlId) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            control_id,
            status: ImplementationStatus::NotImplemented,
            owner: None,
            description: None,
            implemented_at: None,
            applies_to: Vec::new(),
            compensating_controls: Vec::new(),
            exception_ids: Vec::new(),
            risk_ids: Vec::new(),
        }
    }

    pub fn id(&self) -> &ControlImplementationId {
        &self.id
    }

    pub fn control_id(&self) -> &ControlId {
        &self.control_id
    }

    pub fn status(&self) -> ImplementationStatus {
        self.status
    }

    pub fn risk_ids(&self) -> &[RiskId] {
        &self.risk_ids
    }

    pub fn exception_ids(&self) -> &[ExceptionId] {
        &self.exception_ids
    }

    pub fn with_status(mut self, status: ImplementationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_risk(mut self, risk: RiskId) -> Self {
        self.risk_ids.push(risk);
        self
    }

    pub fn with_exception(mut self, exception: ExceptionId) -> Self {
        self.exception_ids.push(exception);
        self
    }
}
