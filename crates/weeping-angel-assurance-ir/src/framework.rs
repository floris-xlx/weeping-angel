//! Framework identity. Not a compile target and not a catalog.

use serde::{Deserialize, Serialize};

use crate::{FrameworkId, FrameworkVersion};

/// Pair that appears on every framework-specific requirement.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FrameworkRef {
    #[serde(rename = "frameworkId")]
    pub id: FrameworkId,
    #[serde(rename = "frameworkVersion")]
    pub version: FrameworkVersion,
}

impl FrameworkRef {
    pub fn new(id: FrameworkId, version: FrameworkVersion) -> Self {
        Self { id, version }
    }

    pub fn id(&self) -> &FrameworkId {
        &self.id
    }

    pub fn version(&self) -> &FrameworkVersion {
        &self.version
    }
}

/// Framework-native identifier, distinct from internal [`crate::RequirementId`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalRequirementRef {
    pub framework: FrameworkRef,
    pub external_id: String,
}
