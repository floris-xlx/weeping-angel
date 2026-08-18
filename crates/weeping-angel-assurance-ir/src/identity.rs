//! Minimal principal identity. Not an IAM platform.

use serde::{Deserialize, Serialize};

use crate::IdentityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IdentityKind {
    User,
    Service,
    Team,
    Role,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: IdentityId,
    pub kind: IdentityKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl Identity {
    pub fn new(id: IdentityId, kind: IdentityKind) -> Self {
        Self {
            id,
            kind,
            display_name: None,
        }
    }
}
