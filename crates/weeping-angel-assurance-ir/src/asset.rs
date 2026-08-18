//! Canonical asset metadata. Collectors own provider-native detail.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{ASSURANCE_IR_SCHEMA, AssetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    Organization,
    Repository,
    Application,
    Service,
    Database,
    CloudAccount,
    CloudResource,
    Device,
    Network,
    Dataset,
    Endpoint,
    Branch,
    Deployment,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    schema_version: String,
    pub id: AssetId,
    pub kind: AssetKind,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<AssetId>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<String, String>,
}

impl Asset {
    pub fn new(id: AssetId, kind: AssetKind, name: impl Into<String>) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            kind,
            name: name.into(),
            parent: None,
            tags: BTreeMap::new(),
        }
    }
}
