//! Framework-neutral subject selection. Collectors normalize into these.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubjectKind {
    #[default]
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
    Branch,
    Application,
    Database,
    CloudAccount,
    CloudResource,
    ServiceAccount,
    Endpoint,
    DataStore,
    Network,
    Deployment,
    BusinessUnit,
    Location,
    DataDomain,
    PersonnelPopulation,
}

impl SubjectKind {
    pub fn parse_name(name: &str) -> Option<Self> {
        let key: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect();
        Some(match key.as_str() {
            "organization" => Self::Organization,
            "asset" => Self::Asset,
            "repository" => Self::Repository,
            "service" => Self::Service,
            "identity" => Self::Identity,
            "user" => Self::User,
            "privilegedidentity" => Self::PrivilegedIdentity,
            "device" => Self::Device,
            "vendor" => Self::Vendor,
            "dataset" => Self::Dataset,
            "processingactivity" => Self::ProcessingActivity,
            "branch" => Self::Branch,
            "application" => Self::Application,
            "database" => Self::Database,
            "cloudaccount" => Self::CloudAccount,
            "cloudresource" => Self::CloudResource,
            "serviceaccount" => Self::ServiceAccount,
            "endpoint" => Self::Endpoint,
            "datastore" => Self::DataStore,
            "network" => Self::Network,
            "deployment" => Self::Deployment,
            "businessunit" => Self::BusinessUnit,
            "location" => Self::Location,
            "datadomain" => Self::DataDomain,
            "personnelpopulation" | "population" => Self::PersonnelPopulation,
            _ => return None,
        })
    }
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
