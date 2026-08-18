//! Minimal risk record. Not a risk engine.

use serde::{Deserialize, Serialize};

use crate::RiskId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RiskStatus {
    #[default]
    Open,
    Accepted,
    Mitigated,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Risk {
    pub id: RiskId,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub status: RiskStatus,
}

impl Risk {
    pub fn new(id: RiskId, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            status: RiskStatus::Open,
        }
    }
}
