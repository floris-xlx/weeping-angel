//! Canonical assessment input. Compile targets stay in the framework crate.

use serde::{Deserialize, Serialize};

use crate::{
    AssessmentId, Asset, Control, ControlImplementation, EvidenceRequirement, Exception, Identity,
    PlannedControlTest, ProcessingActivity, Requirement, Risk, SubjectSelector, Vendor,
    ASSURANCE_IR_SCHEMA,
};

use super::mapping::Mapping;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssessmentRequests {
    pub statement_of_applicability: bool,
    pub control_applicability: bool,
    pub privacy_processing: bool,
    pub risk_treatment: bool,
    pub manual_attestation: bool,
    pub sampling: bool,
    pub audit_program: bool,
    pub nonconformities: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScopeExclusion {
    #[serde(default)]
    pub subjects: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentScope {
    #[serde(default)]
    pub organizations: Vec<String>,
    #[serde(default)]
    pub subjects: Vec<SubjectSelector>,
    #[serde(default)]
    pub exclusions: Vec<ScopeExclusion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentDefinition {
    pub id: AssessmentId,
    pub schema_version: String,
    #[serde(default)]
    pub requirements: Vec<Requirement>,
    #[serde(default)]
    pub controls: Vec<Control>,
    #[serde(default)]
    pub mappings: Vec<Mapping>,
    #[serde(default)]
    pub evidence_requirements: Vec<EvidenceRequirement>,
    #[serde(default)]
    pub tests: Vec<PlannedControlTest>,
    #[serde(default)]
    pub requests: AssessmentRequests,
    #[serde(default)]
    pub implementations: Vec<ControlImplementation>,
    #[serde(default)]
    pub scope: AssessmentScope,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub identities: Vec<Identity>,
    #[serde(default)]
    pub vendors: Vec<Vendor>,
    #[serde(default)]
    pub risks: Vec<Risk>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
    #[serde(default)]
    pub processing_activities: Vec<ProcessingActivity>,
}

impl AssessmentDefinition {
    pub fn new(id: AssessmentId) -> Self {
        Self {
            id,
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            requirements: Vec::new(),
            controls: Vec::new(),
            mappings: Vec::new(),
            evidence_requirements: Vec::new(),
            tests: Vec::new(),
            requests: AssessmentRequests::default(),
            implementations: Vec::new(),
            scope: AssessmentScope::default(),
            assets: Vec::new(),
            identities: Vec::new(),
            vendors: Vec::new(),
            risks: Vec::new(),
            exceptions: Vec::new(),
            processing_activities: Vec::new(),
        }
    }
}

/// Compatibility name used by the framework compiler.
pub type Assessment = AssessmentDefinition;
