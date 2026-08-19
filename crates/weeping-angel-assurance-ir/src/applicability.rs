//! Declarative applicability. The IR does not evaluate platform facts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ApplicabilityRule {
    #[default]
    Always,
    Never,
    All(Vec<ApplicabilityRule>),
    Any(Vec<ApplicabilityRule>),
    Not(Box<ApplicabilityRule>),
    Predicate(ApplicabilityPredicate),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApplicabilityPredicate {
    AssetType(String),
    OrganizationAttribute { key: String, value: String },
    Jurisdiction(String),
    ProcessingCategory(String),
    Technology(String),
    DataCategory(String),
    RiskLevel(String),
    HasVendor(bool),
    HasEmployees(bool),
    UsesCloudProvider(bool),
    ProcessesPersonalData(bool),
}

impl ApplicabilityRule {
    pub fn jurisdiction(code: impl Into<String>) -> Self {
        Self::Predicate(ApplicabilityPredicate::Jurisdiction(code.into()))
    }

    pub fn processes_personal_data(value: bool) -> Self {
        Self::Predicate(ApplicabilityPredicate::ProcessesPersonalData(value))
    }

    /// `Some(false)` only when the tree is statically Never. Predicates stay unknown.
    pub fn statically_applicable(&self) -> Option<bool> {
        match self {
            Self::Always => Some(true),
            Self::Never => Some(false),
            Self::All(rules) => {
                let mut seen_true: bool = true;
                for rule in rules {
                    match rule.statically_applicable() {
                        Some(false) => return Some(false),
                        Some(true) => {}
                        None => seen_true = false,
                    }
                }
                if seen_true { Some(true) } else { None }
            }
            Self::Any(rules) => {
                let mut seen_false = true;
                for rule in rules {
                    match rule.statically_applicable() {
                        Some(true) => return Some(true),
                        Some(false) => {}
                        None => seen_false = false,
                    }
                }
                if seen_false { Some(false) } else { None }
            }
            Self::Not(inner) => inner.statically_applicable().map(|v| !v),
            Self::Predicate(_) => None,
        }
    }
}
