//! Kleene three-state evaluation of `ApplicabilityRule` trees.

use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{
    ApplicabilityPredicate, ApplicabilityRule, AssetKind, IdentityKind, SubjectSelector,
};

use super::context::{
    ApplicabilityContext, FactKey, FactValue, InventoryCompleteness, InventoryFamily,
    asset_matches, eq_ci, identity_matches, is_falsey, is_truthy, parse_asset_kind, tag_value,
    vendor_matches,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplicabilityDecision {
    Applicable,
    NotApplicable,
    ManualDeterminationRequired,
}

impl ApplicabilityDecision {
    pub fn from_fact(value: FactValue) -> Self {
        match value {
            FactValue::True => Self::Applicable,
            FactValue::False => Self::NotApplicable,
            FactValue::Unknown => Self::ManualDeterminationRequired,
        }
    }

    /// Compile keeps Applicable and ManualDeterminationRequired; drops only NotApplicable.
    pub fn remains_in_compiled_set(self) -> bool {
        self != Self::NotApplicable
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RationaleEntry {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredicateTrace {
    pub predicate: ApplicabilityPredicate,
    pub value: FactValue,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownFact {
    pub key: FactKey,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcludedSubject {
    pub id: String,
    pub reason: String,
    pub exclusion_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicabilityOutcome {
    pub decision: ApplicabilityDecision,
    pub rationale: Vec<RationaleEntry>,
    pub predicates: Vec<PredicateTrace>,
    pub unknown_facts: Vec<UnknownFact>,
    pub selected_subjects: Vec<String>,
    pub excluded_subjects: Vec<ExcludedSubject>,
}

struct EvalScratch {
    rationale: Vec<RationaleEntry>,
    predicates: Vec<PredicateTrace>,
    unknown_facts: Vec<UnknownFact>,
}

pub fn evaluate_applicability(
    rule: &ApplicabilityRule,
    context: &ApplicabilityContext,
) -> ApplicabilityOutcome {
    evaluate_applicability_for_subjects(rule, context, None)
}

pub(crate) fn evaluate_applicability_for_subjects(
    rule: &ApplicabilityRule,
    context: &ApplicabilityContext,
    selectors: Option<&[SubjectSelector]>,
) -> ApplicabilityOutcome {
    let mut scratch = EvalScratch {
        rationale: Vec::new(),
        predicates: Vec::new(),
        unknown_facts: Vec::new(),
    };
    let value = eval_rule(rule, context, &mut scratch);
    scratch.unknown_facts.sort_by(|a, b| a.key.cmp(&b.key));
    scratch.unknown_facts.dedup_by(|a, b| a.key == b.key);

    let (selected_subjects, excluded_subjects) = select_subjects(context, selectors);
    ApplicabilityOutcome {
        decision: ApplicabilityDecision::from_fact(value),
        rationale: scratch.rationale,
        predicates: scratch.predicates,
        unknown_facts: scratch.unknown_facts,
        selected_subjects,
        excluded_subjects,
    }
}

fn eval_rule(
    rule: &ApplicabilityRule,
    context: &ApplicabilityContext,
    scratch: &mut EvalScratch,
) -> FactValue {
    match rule {
        ApplicabilityRule::Always => {
            scratch.rationale.push(RationaleEntry {
                code: "always".into(),
                message: "Always evaluates to applicable without consulting facts".into(),
            });
            FactValue::True
        }
        ApplicabilityRule::Never => {
            scratch.rationale.push(RationaleEntry {
                code: "never".into(),
                message: "Never evaluates to not applicable without consulting facts".into(),
            });
            FactValue::False
        }
        ApplicabilityRule::Predicate(predicate) => {
            let (value, source, key) = eval_predicate(predicate, context);
            scratch.predicates.push(PredicateTrace {
                predicate: predicate.clone(),
                value,
                source: source.to_string(),
            });
            scratch.rationale.push(RationaleEntry {
                code: "predicate".into(),
                message: format!("predicate {predicate:?} = {value:?} ({source})"),
            });
            if value == FactValue::Unknown {
                scratch.unknown_facts.push(UnknownFact {
                    key,
                    reason: format!("{source}: fact is unknown"),
                });
            }
            value
        }
        ApplicabilityRule::All(rules) => {
            if rules.is_empty() {
                scratch.rationale.push(RationaleEntry {
                    code: "all-empty".into(),
                    message: "empty All is vacuously true".into(),
                });
                return FactValue::True;
            }
            let mut seen_true = true;
            let mut seen_false = false;
            for child in rules {
                match eval_rule(child, context, scratch) {
                    FactValue::False => seen_false = true,
                    FactValue::True => {}
                    FactValue::Unknown => seen_true = false,
                }
            }
            let value = if seen_false {
                FactValue::False
            } else if seen_true {
                FactValue::True
            } else {
                FactValue::Unknown
            };
            scratch.rationale.push(RationaleEntry {
                code: "all".into(),
                message: format!("All combines to {value:?}"),
            });
            value
        }
        ApplicabilityRule::Any(rules) => {
            if rules.is_empty() {
                scratch.rationale.push(RationaleEntry {
                    code: "any-empty".into(),
                    message: "empty Any is vacuously false".into(),
                });
                return FactValue::False;
            }
            let mut seen_false = true;
            let mut seen_true = false;
            for child in rules {
                match eval_rule(child, context, scratch) {
                    FactValue::True => seen_true = true,
                    FactValue::False => {}
                    FactValue::Unknown => seen_false = false,
                }
            }
            let value = if seen_true {
                FactValue::True
            } else if seen_false {
                FactValue::False
            } else {
                FactValue::Unknown
            };
            scratch.rationale.push(RationaleEntry {
                code: "any".into(),
                message: format!("Any combines to {value:?}"),
            });
            value
        }
        ApplicabilityRule::Not(inner) => {
            let inner_value = eval_rule(inner, context, scratch);
            let value = inner_value.not();
            scratch.rationale.push(RationaleEntry {
                code: "not".into(),
                message: format!("Not({inner_value:?}) = {value:?}"),
            });
            value
        }
    }
}

fn eval_predicate(
    predicate: &ApplicabilityPredicate,
    context: &ApplicabilityContext,
) -> (FactValue, &'static str, FactKey) {
    match predicate {
        ApplicabilityPredicate::AssetType(name) => {
            presence(context, FactKey::AssetType(name.clone()), || {
                infer_asset_type(context, name)
            })
        }
        ApplicabilityPredicate::OrganizationAttribute { key, value } => presence(
            context,
            FactKey::OrganizationAttribute {
                key: key.clone(),
                value: value.clone(),
            },
            || infer_org_attribute(context, key, value),
        ),
        ApplicabilityPredicate::Jurisdiction(code) => {
            presence(context, FactKey::Jurisdiction(code.clone()), || {
                infer_jurisdiction(context, code)
            })
        }
        ApplicabilityPredicate::ProcessingCategory(name) => {
            presence(context, FactKey::ProcessingCategory(name.clone()), || {
                infer_tagged(
                    context,
                    InventoryFamily::ProcessingCategories,
                    &["processingCategory", "processing_category"],
                    name,
                )
            })
        }
        ApplicabilityPredicate::Technology(name) => {
            presence(context, FactKey::Technology(name.clone()), || {
                infer_tagged(
                    context,
                    InventoryFamily::Technologies,
                    &["technology"],
                    name,
                )
            })
        }
        ApplicabilityPredicate::DataCategory(name) => {
            presence(context, FactKey::DataCategory(name.clone()), || {
                infer_tagged(
                    context,
                    InventoryFamily::DataCategories,
                    &["dataCategory", "data_category"],
                    name,
                )
            })
        }
        ApplicabilityPredicate::RiskLevel(level) => {
            presence(context, FactKey::RiskLevel(level.clone()), || {
                infer_risk_level(context, level)
            })
        }
        ApplicabilityPredicate::HasVendor(expected) => {
            bool_presence(context, FactKey::VendorPresence, *expected, infer_vendors)
        }
        ApplicabilityPredicate::HasEmployees(expected) => bool_presence(
            context,
            FactKey::EmployeePresence,
            *expected,
            infer_employees,
        ),
        ApplicabilityPredicate::UsesCloudProvider(expected) => {
            bool_presence(context, FactKey::CloudUsage, *expected, infer_cloud)
        }
        ApplicabilityPredicate::ProcessesPersonalData(expected) => bool_presence(
            context,
            FactKey::PersonalData,
            *expected,
            infer_personal_data,
        ),
    }
}

fn presence(
    context: &ApplicabilityContext,
    key: FactKey,
    infer: impl FnOnce() -> FactValue,
) -> (FactValue, &'static str, FactKey) {
    if let Some(value) = context.explicit_fact(&key) {
        return (value, "explicit", key);
    }
    (infer(), "inferred", key)
}

fn bool_presence(
    context: &ApplicabilityContext,
    key: FactKey,
    expected: bool,
    infer: impl FnOnce(&ApplicabilityContext) -> FactValue,
) -> (FactValue, &'static str, FactKey) {
    if let Some(value) = context.explicit_fact(&key) {
        return (value.known_equals(expected), "explicit", key);
    }
    (infer(context).known_equals(expected), "inferred", key)
}

fn infer_vendors(context: &ApplicabilityContext) -> FactValue {
    if !context.vendors.is_empty() {
        return FactValue::True;
    }
    if context.completeness_of(InventoryFamily::Vendors) == InventoryCompleteness::Authoritative {
        FactValue::False
    } else {
        FactValue::Unknown
    }
}

fn infer_employees(context: &ApplicabilityContext) -> FactValue {
    let users = context
        .identities
        .iter()
        .any(|identity| identity.kind == IdentityKind::User);
    if users {
        return FactValue::True;
    }
    if context.completeness_of(InventoryFamily::Identities) == InventoryCompleteness::Authoritative
        || context.completeness_of(InventoryFamily::Employees)
            == InventoryCompleteness::Authoritative
    {
        FactValue::False
    } else {
        FactValue::Unknown
    }
}

fn infer_cloud(context: &ApplicabilityContext) -> FactValue {
    let present = context.assets.iter().any(|asset| {
        matches!(
            asset.kind,
            AssetKind::CloudAccount | AssetKind::CloudResource
        )
    });
    if present {
        return FactValue::True;
    }
    if context.completeness_of(InventoryFamily::Assets) == InventoryCompleteness::Authoritative
        || context.completeness_of(InventoryFamily::CloudUsage)
            == InventoryCompleteness::Authoritative
    {
        FactValue::False
    } else {
        FactValue::Unknown
    }
}

fn infer_personal_data(context: &ApplicabilityContext) -> FactValue {
    let mut saw_false = false;
    for asset in &context.assets {
        if let Some(value) = tag_value(&asset.tags, &["personalData", "processesPersonalData"]) {
            if is_truthy(value) {
                return FactValue::True;
            }
            if is_falsey(value) {
                saw_false = true;
            }
        }
    }
    if saw_false {
        return FactValue::False;
    }
    if context.completeness_of(InventoryFamily::PersonalData)
        == InventoryCompleteness::Authoritative
    {
        return FactValue::False;
    }
    FactValue::Unknown
}

fn infer_asset_type(context: &ApplicabilityContext, name: &str) -> FactValue {
    let parsed = parse_asset_kind(name);
    let present = context.assets.iter().any(|asset| match parsed {
        Some(kind) => asset.kind == kind,
        None => false,
    });
    if present {
        return FactValue::True;
    }
    if context.completeness_of(InventoryFamily::Assets) == InventoryCompleteness::Authoritative {
        FactValue::False
    } else {
        FactValue::Unknown
    }
}

fn infer_org_attribute(context: &ApplicabilityContext, key: &str, value: &str) -> FactValue {
    let present = context.assets.iter().any(|asset| {
        asset.kind == AssetKind::Organization
            && asset.tags.get(key).is_some_and(|have| have == value)
    });
    if present {
        return FactValue::True;
    }
    if context.completeness_of(InventoryFamily::OrganizationAttributes)
        == InventoryCompleteness::Authoritative
    {
        FactValue::False
    } else {
        FactValue::Unknown
    }
}

fn infer_jurisdiction(context: &ApplicabilityContext, code: &str) -> FactValue {
    let present = context.assets.iter().any(|asset| {
        tag_value(&asset.tags, &["jurisdiction", "jurisdictionCode"])
            .is_some_and(|have| eq_ci(have, code))
    });
    if present {
        return FactValue::True;
    }
    if context.completeness_of(InventoryFamily::Jurisdictions)
        == InventoryCompleteness::Authoritative
    {
        FactValue::False
    } else {
        FactValue::Unknown
    }
}

fn infer_tagged(
    context: &ApplicabilityContext,
    family: InventoryFamily,
    keys: &[&str],
    expected: &str,
) -> FactValue {
    let present = context.assets.iter().any(|asset| {
        tag_value(&asset.tags, keys).is_some_and(|have| eq_ci(have, expected))
            || tag_value(&asset.tags, &[expected]).is_some_and(is_truthy)
    });
    if present {
        return FactValue::True;
    }
    if context.completeness_of(family) == InventoryCompleteness::Authoritative {
        FactValue::False
    } else {
        FactValue::Unknown
    }
}

fn infer_risk_level(context: &ApplicabilityContext, level: &str) -> FactValue {
    let needle = format!("level={level}");
    let present = context.risks.iter().any(|risk| {
        risk.title
            .to_ascii_lowercase()
            .contains(&needle.to_ascii_lowercase())
            || risk
                .description
                .to_ascii_lowercase()
                .contains(&needle.to_ascii_lowercase())
    });
    if present {
        return FactValue::True;
    }
    if context.completeness_of(InventoryFamily::RiskLevel) == InventoryCompleteness::Authoritative
        || context.completeness_of(InventoryFamily::Risks) == InventoryCompleteness::Authoritative
    {
        FactValue::False
    } else {
        FactValue::Unknown
    }
}

pub(crate) fn select_subjects(
    context: &ApplicabilityContext,
    selectors: Option<&[SubjectSelector]>,
) -> (Vec<String>, Vec<ExcludedSubject>) {
    let excluded = context.excluded_subjects.clone();
    let selected = match selectors {
        None => context.inventory_ids(),
        Some(sels) if sels.is_empty() => Vec::new(),
        Some(sels) => {
            let mut ids = Vec::new();
            for selector in sels {
                for asset in &context.assets {
                    if asset_matches(asset, selector) {
                        ids.push(asset.id.as_str().to_string());
                    }
                }
                for identity in &context.identities {
                    if identity_matches(identity, selector) {
                        ids.push(identity.id.as_str().to_string());
                    }
                }
                for vendor in &context.vendors {
                    if vendor_matches(vendor, selector) {
                        ids.push(vendor.id.as_str().to_string());
                    }
                }
                for activity in &context.processing_activities {
                    if super::context::activity_matches(activity, selector) {
                        ids.push(activity.id.as_str().to_string());
                    }
                }
            }
            ids.sort();
            ids.dedup();
            ids
        }
    };
    (selected, excluded)
}
