//! What evidence is required. Not how to interpret it.

use serde::{Deserialize, Serialize};

use crate::{EvidenceRequirementId, EvidenceType, SubjectSelector, ASSURANCE_IR_SCHEMA};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceCardinality {
    #[default]
    One,
    AtLeast(u32),
    AllSubjects,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceCollectionKind {
    #[default]
    Automated,
    Manual,
    Either,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceCriticality {
    #[default]
    Required,
    Supporting,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreshnessRequirement {
    pub max_age_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRequirement {
    schema_version: String,
    id: EvidenceRequirementId,
    evidence_type: EvidenceType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subject: Option<SubjectSelector>,
    #[serde(default, skip_serializing_if = "is_one")]
    cardinality: EvidenceCardinality,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    freshness: Option<FreshnessRequirement>,
    #[serde(default, skip_serializing_if = "is_automated")]
    collection: EvidenceCollectionKind,
    #[serde(default, skip_serializing_if = "is_required")]
    criticality: EvidenceCriticality,
}

fn is_one(value: &EvidenceCardinality) -> bool {
    matches!(value, EvidenceCardinality::One)
}

fn is_automated(value: &EvidenceCollectionKind) -> bool {
    matches!(value, EvidenceCollectionKind::Automated)
}

fn is_required(value: &EvidenceCriticality) -> bool {
    matches!(value, EvidenceCriticality::Required)
}

impl EvidenceRequirement {
    pub fn new(id: EvidenceRequirementId, evidence_type: EvidenceType) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            evidence_type,
            subject: None,
            cardinality: EvidenceCardinality::One,
            freshness: None,
            collection: EvidenceCollectionKind::Automated,
            criticality: EvidenceCriticality::Required,
        }
    }

    pub fn id(&self) -> &EvidenceRequirementId {
        &self.id
    }

    pub fn evidence_type(&self) -> &EvidenceType {
        &self.evidence_type
    }

    pub fn with_cardinality(mut self, cardinality: EvidenceCardinality) -> Self {
        self.cardinality = cardinality;
        self
    }

    pub fn with_collection(mut self, collection: EvidenceCollectionKind) -> Self {
        self.collection = collection;
        self
    }

    pub fn with_criticality(mut self, criticality: EvidenceCriticality) -> Self {
        self.criticality = criticality;
        self
    }
}
