//! Minimal vendor node for the compliance graph.

use serde::{Deserialize, Serialize};

use crate::VendorId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vendor {
    pub id: VendorId,
    pub name: String,
}

impl Vendor {
    pub fn new(id: VendorId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}
