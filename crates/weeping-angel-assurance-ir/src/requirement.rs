//! Framework-specific requirement. Not a canonical control.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ApplicabilityRule, ExtensionMap, FrameworkId, FrameworkRef, FrameworkVersion, RequirementId,
    ASSURANCE_IR_SCHEMA,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RequirementKind {
    #[default]
    Requirement,
    ControlObjective,
    Control,
    Clause,
    Article,
    Principle,
    Procedure,
    AuditRequirement,
    Guidance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirement {
    schema_version: String,
    id: RequirementId,
    #[serde(flatten)]
    framework: FrameworkRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    external_id: Option<String>,
    title: String,
    description: String,
    #[serde(default, skip_serializing_if = "is_default_kind")]
    kind: RequirementKind,
    #[serde(default, skip_serializing_if = "is_always")]
    applicability: ApplicabilityRule,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<RequirementId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    tags: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "ExtensionMap::is_empty")]
    extensions: ExtensionMap,
}

fn is_always(rule: &ApplicabilityRule) -> bool {
    matches!(rule, ApplicabilityRule::Always)
}

fn is_default_kind(kind: &RequirementKind) -> bool {
    matches!(kind, RequirementKind::Requirement)
}

impl Requirement {
    pub fn new(
        id: RequirementId,
        framework_id: FrameworkId,
        framework_version: FrameworkVersion,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            framework: FrameworkRef::new(framework_id, framework_version),
            external_id: None,
            title: title.into(),
            description: description.into(),
            kind: RequirementKind::Requirement,
            applicability: ApplicabilityRule::Always,
            parent: None,
            tags: BTreeSet::new(),
            extensions: ExtensionMap::new(),
        }
    }

    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn id(&self) -> &RequirementId {
        &self.id
    }

    pub fn framework(&self) -> &FrameworkRef {
        &self.framework
    }

    pub fn framework_id(&self) -> &FrameworkId {
        self.framework.id()
    }

    pub fn framework_version(&self) -> &FrameworkVersion {
        self.framework.version()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn kind(&self) -> RequirementKind {
        self.kind
    }

    pub fn applicability(&self) -> &ApplicabilityRule {
        &self.applicability
    }

    pub fn external_id(&self) -> Option<&str> {
        self.external_id.as_deref()
    }

    pub fn with_external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    pub fn with_kind(mut self, kind: RequirementKind) -> Self {
        self.kind = kind;
        self
    }
}
