//! Canonical, framework-neutral control definition.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ASSURANCE_IR_SCHEMA, ApplicabilityRule, ControlId, ControlTestId, EvidenceRequirementId,
    ExtensionMap, SubjectSelector,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlDomain {
    AccessControl,
    Authentication,
    Authorization,
    AssetManagement,
    ChangeManagement,
    Cryptography,
    DataProtection,
    IncidentResponse,
    LoggingMonitoring,
    NetworkSecurity,
    PersonnelSecurity,
    PhysicalSecurity,
    SecureDevelopment,
    SupplierManagement,
    VulnerabilityManagement,
    Governance,
    Privacy,
    Resilience,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ControlExpectation {
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Control {
    #[serde(default = "default_schema_version")]
    schema_version: String,
    id: ControlId,
    title: String,
    description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    objective: String,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    domains: BTreeSet<ControlDomain>,
    #[serde(default, skip_serializing_if = "is_always")]
    applicability: ApplicabilityRule,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    subjects: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "expectation_empty")]
    implementation_expectation: ControlExpectation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    evidence_requirements: Vec<EvidenceRequirementId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tests: Vec<ControlTestId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    tags: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "ExtensionMap::is_empty")]
    extensions: ExtensionMap,
}

fn default_schema_version() -> String {
    ASSURANCE_IR_SCHEMA.into()
}

fn is_always(rule: &ApplicabilityRule) -> bool {
    matches!(rule, ApplicabilityRule::Always)
}

fn expectation_empty(value: &ControlExpectation) -> bool {
    value.summary.is_empty()
}

impl Control {
    pub fn new(id: ControlId, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            title: title.into(),
            description: description.into(),
            objective: String::new(),
            domains: BTreeSet::new(),
            applicability: ApplicabilityRule::Always,
            subjects: Vec::new(),
            implementation_expectation: ControlExpectation::default(),
            evidence_requirements: Vec::new(),
            tests: Vec::new(),
            tags: BTreeSet::new(),
            extensions: ExtensionMap::new(),
        }
    }

    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn id(&self) -> &ControlId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn objective(&self) -> &str {
        &self.objective
    }

    pub fn domains(&self) -> &BTreeSet<ControlDomain> {
        &self.domains
    }

    pub fn applicability(&self) -> &ApplicabilityRule {
        &self.applicability
    }

    pub fn subjects(&self) -> &[SubjectSelector] {
        &self.subjects
    }

    pub fn evidence_requirements(&self) -> &[EvidenceRequirementId] {
        &self.evidence_requirements
    }

    pub fn extensions(&self) -> &ExtensionMap {
        &self.extensions
    }

    pub fn with_objective(mut self, objective: impl Into<String>) -> Self {
        self.objective = objective.into();
        self
    }

    pub fn with_domain(mut self, domain: ControlDomain) -> Self {
        self.domains.insert(domain);
        self
    }

    pub fn with_extensions(mut self, extensions: ExtensionMap) -> Self {
        self.extensions = extensions;
        self
    }
}
