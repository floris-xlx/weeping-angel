//! Explicit directed mappings. Equivalence is never inferred.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    ControlId, FrameworkVersion, MappingId, RequirementId, ASSURANCE_IR_SCHEMA,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingRelation {
    Equivalent,
    Satisfies,
    PartiallySatisfies,
    Supports,
    EvidenceFor,
    SupersetOf,
    SubsetOf,
    Related,
}

impl MappingRelation {
    pub fn from_completeness(completeness: MappingCompleteness) -> Self {
        match completeness {
            MappingCompleteness::Full => Self::Satisfies,
            MappingCompleteness::Partial => Self::PartiallySatisfies,
            MappingCompleteness::Related => Self::Related,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MappingConfidence {
    #[default]
    Unspecified,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MappingSource {
    #[default]
    BuiltIn,
    LicensedFrameworkContent,
    UserDefined,
    Imported,
    AuditorApproved,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MappingProvenance {
    pub source: MappingSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<DateTime<Utc>>,
}

impl MappingProvenance {
    pub fn has_curated_authority(&self) -> bool {
        matches!(
            self.source,
            MappingSource::AuditorApproved
                | MappingSource::LicensedFrameworkContent
                | MappingSource::BuiltIn
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MappingVersionConstraint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<FrameworkVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<FrameworkVersion>,
}

impl MappingVersionConstraint {
    pub fn is_unconstrained(&self) -> bool {
        self.from.is_none() && self.to.is_none()
    }

    pub fn contains(&self, version: &FrameworkVersion) -> bool {
        let value = version.as_str();
        if let Some(from) = &self.from
            && value < from.as_str()
        {
            return false;
        }
        if let Some(to) = &self.to
            && value > to.as_str()
        {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<MappingId>,
    from_requirement: RequirementId,
    to_control: ControlId,
    direction: MappingDirection,
    completeness: MappingCompleteness,
    relation: MappingRelation,
    rationale: String,
    #[serde(default, skip_serializing_if = "is_unspecified_confidence")]
    confidence: MappingConfidence,
    #[serde(default, skip_serializing_if = "is_default_provenance")]
    provenance: MappingProvenance,
    #[serde(default, skip_serializing_if = "MappingVersionConstraint::is_unconstrained")]
    valid_for: MappingVersionConstraint,
}

fn is_unspecified_confidence(value: &MappingConfidence) -> bool {
    matches!(value, MappingConfidence::Unspecified)
}

fn is_default_provenance(value: &MappingProvenance) -> bool {
    matches!(value.source, MappingSource::BuiltIn)
        && value.author.is_none()
        && value.reference.is_none()
        && value.reviewed_at.is_none()
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
            id: None,
            from_requirement,
            to_control,
            direction,
            completeness,
            relation: MappingRelation::from_completeness(completeness),
            rationale: match completeness {
                MappingCompleteness::Full => "explicit full mapping".into(),
                MappingCompleteness::Partial => {
                    "partial mapping; PartiallySatisfies cannot fully satisfy".into()
                }
                MappingCompleteness::Related => "related only; no satisfaction".into(),
            },
            confidence: MappingConfidence::Unspecified,
            provenance: MappingProvenance::default(),
            valid_for: MappingVersionConstraint::default(),
        }
    }

    pub fn with_relation(mut self, relation: MappingRelation) -> Self {
        self.relation = relation;
        self
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = rationale.into();
        self
    }

    pub fn with_valid_for(mut self, valid_for: MappingVersionConstraint) -> Self {
        self.valid_for = valid_for;
        self
    }

    pub fn with_provenance(mut self, provenance: MappingProvenance) -> Self {
        self.provenance = provenance;
        self
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

    pub fn relation(&self) -> MappingRelation {
        self.relation
    }

    pub fn rationale(&self) -> &str {
        &self.rationale
    }

    pub fn valid_for(&self) -> &MappingVersionConstraint {
        &self.valid_for
    }

    pub fn provenance(&self) -> &MappingProvenance {
        &self.provenance
    }
}
