//! Framework-neutral compliance IR (Athena `Statement` analogue).
//!
//! Requirement → Mapping → Canonical Control → Control Test → Evidence Requirement.
//! No provider/SDK types. Control has no ISO-specific fields.

pub mod crosswalk;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Explicit schema version on every serialized IR document.
pub const ASSURANCE_IR_SCHEMA: &str = "assurance-ir/v1";

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

typed_id!(FrameworkId);
typed_id!(FrameworkVersion);
typed_id!(RequirementId);
typed_id!(ControlId);
typed_id!(ControlImplementationId);
typed_id!(ControlTestId);
typed_id!(AssetId);
typed_id!(IdentityId);
typed_id!(VendorId);
typed_id!(ProcessingActivityId);
typed_id!(EvidenceRequirementId);
typed_id!(RiskId);
typed_id!(ExceptionId);
typed_id!(AssessmentId);
typed_id!(AuditProgramId);

/// Evidence kind advertised by collectors (not a framework name).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EvidenceType(String);

impl EvidenceType {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EvidenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MappingDirection {
    Forward,
    Reverse,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MappingCompleteness {
    Full,
    Partial,
    Related,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PlannedTestKind {
    #[default]
    Automated,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Control {
    schema_version: String,
    id: ControlId,
    title: String,
    description: String,
}

impl Control {
    pub fn new(
        id: ControlId,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            title: title.into(),
            description: description.into(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirement {
    schema_version: String,
    id: RequirementId,
    framework_id: FrameworkId,
    framework_version: FrameworkVersion,
    title: String,
    description: String,
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
            framework_id,
            framework_version,
            title: title.into(),
            description: description.into(),
        }
    }

    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn id(&self) -> &RequirementId {
        &self.id
    }

    pub fn framework_id(&self) -> &FrameworkId {
        &self.framework_id
    }

    pub fn framework_version(&self) -> &FrameworkVersion {
        &self.framework_version
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    schema_version: String,
    from_requirement: RequirementId,
    to_control: ControlId,
    direction: MappingDirection,
    completeness: MappingCompleteness,
}

impl Mapping {
    pub fn new(
        from_requirement: RequirementId,
        to_control: ControlId,
        direction: MappingDirection,
        completeness: MappingCompleteness,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            from_requirement,
            to_control,
            direction,
            completeness,
        }
    }

    pub fn from_requirement(&self) -> &RequirementId {
        &self.from_requirement
    }

    pub fn to_control(&self) -> &ControlId {
        &self.to_control
    }

    pub fn direction(&self) -> MappingDirection {
        self.direction
    }

    pub fn completeness(&self) -> MappingCompleteness {
        self.completeness
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRequirement {
    schema_version: String,
    id: EvidenceRequirementId,
    evidence_type: EvidenceType,
}

impl EvidenceRequirement {
    pub fn new(id: EvidenceRequirementId, evidence_type: EvidenceType) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            evidence_type,
        }
    }

    pub fn id(&self) -> &EvidenceRequirementId {
        &self.id
    }

    pub fn evidence_type(&self) -> &EvidenceType {
        &self.evidence_type
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedControlTest {
    schema_version: String,
    pub id: ControlTestId,
    pub control_id: ControlId,
    pub kind: PlannedTestKind,
    pub required_evidence: Vec<EvidenceType>,
    pub break_on: Vec<EvidenceType>,
}

impl PlannedControlTest {
    pub fn new(id: ControlTestId, control_id: ControlId) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            control_id,
            kind: PlannedTestKind::Automated,
            required_evidence: Vec::new(),
            break_on: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CanonicalDigestError {
    #[error("canonical serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// SHA-256 hex of deterministic serde JSON (struct field order + BTree maps).
pub fn canonical_digest<T: Serialize>(value: &T) -> Result<String, CanonicalDigestError> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}
