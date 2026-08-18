//! Control exception. Attaches to implementations, not Control definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ControlId, ExceptionId, PrincipalRef, SubjectSelector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exception {
    pub id: ExceptionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_id: Option<ControlId>,
    pub rationale: String,
    #[serde(default)]
    pub status: ExceptionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Subject binding. Empty does **not** mean the entire inventory.
    #[serde(default)]
    pub subjects: Vec<SubjectSelector>,
}

impl Exception {
    pub fn new(id: ExceptionId, rationale: impl Into<String>) -> Self {
        Self {
            id,
            control_id: None,
            rationale: rationale.into(),
            status: ExceptionStatus::Proposed,
            approved_by: None,
            expires_at: None,
            subjects: Vec::new(),
        }
    }
}
