//! Framework-neutral subject selection. Collectors normalize into these.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubjectKind {
    Organization,
    Asset,
    Repository,
    Service,
    Identity,
    User,
    PrivilegedIdentity,
    Device,
    Vendor,
    Dataset,
    ProcessingActivity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SelectorScope {
    #[default]
    All,
    AnyOf,
    NoneOf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubjectSelector {
    pub kind: SubjectKind,
    #[serde(default)]
    pub ids: BTreeSet<String>,
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
    #[serde(default)]
    pub scope: SelectorScope,
}

impl Default for SubjectKind {
    fn default() -> Self {
        Self::Organization
    }
}
