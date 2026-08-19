//! Derived organization context over existing IR inventories. Not a second inventory.

use std::collections::BTreeMap;
use std::ops::Not;

use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, AssessmentScope, Asset, AssetKind, Identity, IdentityKind,
    ProcessingActivity, Risk, SelectorScope, SubjectKind, SubjectSelector, Vendor,
};

use super::evaluator::ExcludedSubject;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FactValue {
    True,
    False,
    Unknown,
}

impl FactValue {
    pub fn from_bool(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }

    pub fn known_equals(self, expected: bool) -> Self {
        match self {
            Self::Unknown => Self::Unknown,
            known => {
                if known == Self::from_bool(expected) {
                    Self::True
                } else {
                    Self::False
                }
            }
        }
    }
}

impl Not for FactValue {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FactKey {
    AssetType(String),
    OrganizationAttribute { key: String, value: String },
    Jurisdiction(String),
    ProcessingCategory(String),
    Technology(String),
    DataCategory(String),
    RiskLevel(String),
    VendorPresence,
    EmployeePresence,
    CloudUsage,
    PersonalData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InventoryCompleteness {
    Authoritative,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InventoryFamily {
    Assets,
    Identities,
    Vendors,
    ProcessingActivities,
    Risks,
    OrganizationAttributes,
    Jurisdictions,
    Technologies,
    DataCategories,
    ProcessingCategories,
    PersonalData,
    CloudUsage,
    Employees,
    RiskLevel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextExtras {
    #[serde(default)]
    completeness: BTreeMap<InventoryFamily, InventoryCompleteness>,
    #[serde(default)]
    facts: BTreeMap<FactKey, FactValue>,
    #[serde(default)]
    pack_entries: Vec<super::snapshot::PackApplicabilityEntry>,
}

impl ContextExtras {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_completeness(
        mut self,
        family: InventoryFamily,
        completeness: InventoryCompleteness,
    ) -> Self {
        self.completeness.insert(family, completeness);
        self
    }

    pub fn with_fact(mut self, key: FactKey, value: FactValue) -> Self {
        self.facts.insert(key, value);
        self
    }

    pub fn with_pack_entries(
        mut self,
        entries: Vec<super::snapshot::PackApplicabilityEntry>,
    ) -> Self {
        self.pack_entries = entries;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicabilityContext {
    pub assessment_id: AssessmentId,
    pub scope: AssessmentScope,
    pub organizations: Vec<String>,
    pub assets: Vec<Asset>,
    pub identities: Vec<Identity>,
    pub vendors: Vec<Vendor>,
    pub processing_activities: Vec<ProcessingActivity>,
    pub risks: Vec<Risk>,
    pub completeness: BTreeMap<InventoryFamily, InventoryCompleteness>,
    pub facts: BTreeMap<FactKey, FactValue>,
    pub excluded_subjects: Vec<ExcludedSubject>,
    #[serde(default)]
    pub pack_entries: Vec<super::snapshot::PackApplicabilityEntry>,
}

impl ApplicabilityContext {
    pub fn completeness_of(&self, family: InventoryFamily) -> InventoryCompleteness {
        self.completeness
            .get(&family)
            .copied()
            .unwrap_or(InventoryCompleteness::Unknown)
    }

    pub fn explicit_fact(&self, key: &FactKey) -> Option<FactValue> {
        self.facts.get(key).copied()
    }

    pub fn inventory_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        for asset in &self.assets {
            ids.push(asset.id.as_str().to_string());
        }
        for identity in &self.identities {
            ids.push(identity.id.as_str().to_string());
        }
        for vendor in &self.vendors {
            ids.push(vendor.id.as_str().to_string());
        }
        for activity in &self.processing_activities {
            ids.push(activity.id.as_str().to_string());
        }
        for risk in &self.risks {
            ids.push(risk.id.as_str().to_string());
        }
        ids.sort();
        ids.dedup();
        ids
    }
}

pub fn build_applicability_context(
    definition: &AssessmentDefinition,
    extras: ContextExtras,
) -> ApplicabilityContext {
    let scope = definition.scope.clone();
    let mut assets = definition.assets.clone();
    let mut identities = definition.identities.clone();
    let mut vendors = definition.vendors.clone();
    let mut processing_activities = definition.processing_activities.clone();
    let mut risks = definition.risks.clone();

    let mut excluded_subjects = Vec::new();
    {
        let mut inventories = Inventories {
            assets: &mut assets,
            identities: &mut identities,
            vendors: &mut vendors,
            processing_activities: &mut processing_activities,
            risks: &mut risks,
        };
        if !scope.subjects.is_empty() {
            inventories.retain_included(&scope.subjects);
        }
        inventories.apply_exclusions(&scope, &mut excluded_subjects);
    }
    excluded_subjects.sort_by(|a, b| a.id.cmp(&b.id));

    ApplicabilityContext {
        assessment_id: definition.id.clone(),
        organizations: scope.organizations.clone(),
        scope,
        assets,
        identities,
        vendors,
        processing_activities,
        risks,
        completeness: extras.completeness,
        facts: extras.facts,
        excluded_subjects,
        pack_entries: extras.pack_entries,
    }
}

struct Inventories<'a> {
    assets: &'a mut Vec<Asset>,
    identities: &'a mut Vec<Identity>,
    vendors: &'a mut Vec<Vendor>,
    processing_activities: &'a mut Vec<ProcessingActivity>,
    risks: &'a mut Vec<Risk>,
}

impl Inventories<'_> {
    fn retain_included(&mut self, includes: &[SubjectSelector]) {
        self.assets
            .retain(|asset| includes.iter().any(|sel| asset_matches(asset, sel)));
        self.identities
            .retain(|identity| includes.iter().any(|sel| identity_matches(identity, sel)));
        self.vendors
            .retain(|vendor| includes.iter().any(|sel| vendor_matches(vendor, sel)));
        self.processing_activities
            .retain(|activity| includes.iter().any(|sel| activity_matches(activity, sel)));
        self.risks
            .retain(|risk| includes.iter().any(|sel| risk_matches(risk, sel)));
    }

    fn apply_exclusions(&mut self, scope: &AssessmentScope, excluded: &mut Vec<ExcludedSubject>) {
        for (index, exclusion) in scope.exclusions.iter().enumerate() {
            let Some(reason) = exclusion
                .rationale
                .as_deref()
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .map(ToString::to_string)
            else {
                continue;
            };
            self.drop_matching(&exclusion.subjects, index, &reason, excluded);
        }
    }

    fn drop_matching(
        &mut self,
        selectors: &[SubjectSelector],
        exclusion_index: usize,
        reason: &str,
        excluded: &mut Vec<ExcludedSubject>,
    ) {
        record_drops(
            self.assets,
            asset_matches,
            |asset| asset.id.as_str().to_string(),
            selectors,
            exclusion_index,
            reason,
            excluded,
        );
        record_drops(
            self.identities,
            identity_matches,
            |identity| identity.id.as_str().to_string(),
            selectors,
            exclusion_index,
            reason,
            excluded,
        );
        record_drops(
            self.vendors,
            vendor_matches,
            |vendor| vendor.id.as_str().to_string(),
            selectors,
            exclusion_index,
            reason,
            excluded,
        );
        record_drops(
            self.processing_activities,
            activity_matches,
            |activity| activity.id.as_str().to_string(),
            selectors,
            exclusion_index,
            reason,
            excluded,
        );
        record_drops(
            self.risks,
            risk_matches,
            |risk| risk.id.as_str().to_string(),
            selectors,
            exclusion_index,
            reason,
            excluded,
        );
    }
}

fn record_drops<T>(
    items: &mut Vec<T>,
    matches: impl Fn(&T, &SubjectSelector) -> bool,
    id_of: impl Fn(&T) -> String,
    selectors: &[SubjectSelector],
    exclusion_index: usize,
    reason: &str,
    excluded: &mut Vec<ExcludedSubject>,
) {
    items.retain(|item| {
        if selectors.iter().any(|sel| matches(item, sel)) {
            excluded.push(ExcludedSubject {
                id: id_of(item),
                reason: reason.to_string(),
                exclusion_index,
            });
            false
        } else {
            true
        }
    });
}

pub(crate) fn asset_matches(asset: &Asset, selector: &SubjectSelector) -> bool {
    if !asset_kind_matches(asset.kind, selector.kind) {
        return false;
    }
    if !tags_match(&asset.tags, &selector.tags) {
        return false;
    }
    id_in_scope(asset.id.as_str(), selector)
}

pub(crate) fn identity_matches(identity: &Identity, selector: &SubjectSelector) -> bool {
    if !identity_kind_matches(identity.kind, selector.kind) {
        return false;
    }
    if !selector.tags.is_empty() {
        return false;
    }
    id_in_scope(identity.id.as_str(), selector)
}

pub(crate) fn vendor_matches(vendor: &Vendor, selector: &SubjectSelector) -> bool {
    if selector.kind != SubjectKind::Vendor {
        return false;
    }
    if !selector.tags.is_empty() {
        return false;
    }
    id_in_scope(vendor.id.as_str(), selector)
}

pub(crate) fn activity_matches(activity: &ProcessingActivity, selector: &SubjectSelector) -> bool {
    if selector.kind != SubjectKind::ProcessingActivity {
        return false;
    }
    if !selector.tags.is_empty() {
        return false;
    }
    id_in_scope(activity.id.as_str(), selector)
}

pub(crate) fn risk_matches(risk: &Risk, selector: &SubjectSelector) -> bool {
    if !selector.tags.is_empty() {
        return false;
    }
    if selector.kind != SubjectKind::Asset {
        return false;
    }
    id_in_scope(risk.id.as_str(), selector)
}

fn id_in_scope(id: &str, selector: &SubjectSelector) -> bool {
    match selector.scope {
        SelectorScope::All => true,
        SelectorScope::AnyOf => selector.ids.is_empty() || selector.ids.contains(id),
        SelectorScope::NoneOf => !selector.ids.contains(id),
    }
}

fn tags_match(have: &BTreeMap<String, String>, want: &BTreeMap<String, String>) -> bool {
    want.iter()
        .all(|(k, v)| have.get(k).is_some_and(|have_v| have_v == v))
}

pub(crate) fn asset_kind_matches(kind: AssetKind, subject: SubjectKind) -> bool {
    matches!(
        (kind, subject),
        (_, SubjectKind::Asset)
            | (AssetKind::Organization, SubjectKind::Organization)
            | (AssetKind::Repository, SubjectKind::Repository)
            | (AssetKind::Application, SubjectKind::Application)
            | (AssetKind::Service, SubjectKind::Service)
            | (AssetKind::Database, SubjectKind::Database)
            | (AssetKind::CloudAccount, SubjectKind::CloudAccount)
            | (AssetKind::CloudResource, SubjectKind::CloudResource)
            | (AssetKind::Device, SubjectKind::Device)
            | (AssetKind::Network, SubjectKind::Network)
            | (
                AssetKind::Dataset,
                SubjectKind::Dataset | SubjectKind::DataStore
            )
            | (AssetKind::Endpoint, SubjectKind::Endpoint)
            | (AssetKind::Branch, SubjectKind::Branch)
            | (AssetKind::Deployment, SubjectKind::Deployment)
    )
}

fn identity_kind_matches(kind: IdentityKind, subject: SubjectKind) -> bool {
    matches!(
        (kind, subject),
        (_, SubjectKind::Identity)
            | (IdentityKind::User, SubjectKind::User)
            | (
                IdentityKind::Service | IdentityKind::ServiceAccount,
                SubjectKind::ServiceAccount | SubjectKind::Service,
            )
    )
}

pub(crate) fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

pub(crate) fn tag_value<'a>(tags: &'a BTreeMap<String, String>, names: &[&str]) -> Option<&'a str> {
    let wanted: Vec<String> = names.iter().map(|n| normalize_token(n)).collect();
    for (key, value) in tags {
        if wanted.contains(&normalize_token(key)) {
            return Some(value.as_str());
        }
    }
    None
}

pub(crate) fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes"
    )
}

pub(crate) fn is_falsey(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "false" | "0" | "no"
    )
}

pub(crate) fn parse_asset_kind(name: &str) -> Option<AssetKind> {
    Some(match normalize_token(name).as_str() {
        "organization" => AssetKind::Organization,
        "repository" => AssetKind::Repository,
        "application" => AssetKind::Application,
        "service" => AssetKind::Service,
        "database" => AssetKind::Database,
        "cloudaccount" => AssetKind::CloudAccount,
        "cloudresource" => AssetKind::CloudResource,
        "device" => AssetKind::Device,
        "network" => AssetKind::Network,
        "dataset" | "datastore" => AssetKind::Dataset,
        "endpoint" => AssetKind::Endpoint,
        "branch" => AssetKind::Branch,
        "deployment" => AssetKind::Deployment,
        "other" => AssetKind::Other,
        _ => return None,
    })
}

pub(crate) fn eq_ci(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}
