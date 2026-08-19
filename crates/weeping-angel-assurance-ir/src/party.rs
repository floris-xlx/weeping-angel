//! Provider-neutral interested parties. Canonical records for the obligation registry.
//!
//! Distinct from the membership graph stubs on [`crate::IsmsContext`]: this module
//! owns party identity used by `ObligationRegistry`. Share [`crate::InterestedPartyId`].

use serde::{Deserialize, Serialize};

use crate::{ASSURANCE_IR_SCHEMA, InterestedPartyId};

fn schema_version_default() -> String {
    ASSURANCE_IR_SCHEMA.to_string()
}

/// Who the ISMS owes a duty to, or who imposes a duty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InterestedPartyKind {
    Internal,
    External,
    Customer,
    Regulator,
    Insurer,
    Supplier,
    Employee,
    Other(String),
}

/// Provider-neutral party. Not a vendor inventory row and not a collector identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestedParty {
    #[serde(default = "schema_version_default")]
    pub schema_version: String,
    pub id: InterestedPartyId,
    pub name: String,
    pub kind: InterestedPartyKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl InterestedParty {
    pub fn new(id: InterestedPartyId, name: impl Into<String>, kind: InterestedPartyKind) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            name: name.into(),
            kind,
            notes: None,
        }
    }
}
