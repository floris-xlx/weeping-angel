//! Minimal processing-activity node. Full privacy catalogs live in adapters.

use serde::{Deserialize, Serialize};

use crate::{AssetId, ProcessingActivityId, VendorId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingActivity {
    pub id: ProcessingActivityId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub systems: Vec<AssetId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub processors: Vec<VendorId>,
}

impl ProcessingActivity {
    pub fn new(id: ProcessingActivityId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            systems: Vec::new(),
            processors: Vec::new(),
        }
    }
}
