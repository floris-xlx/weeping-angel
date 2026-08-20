//! Controlled-document registry: immutable versioned artifacts with governance
//! metadata. Not a document editor. Not a control. Not an effectiveness verdict.
//!
//! Artifact identity is `EvidenceEnvelope.content_digest` (IR `canonical_digest`).

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::id::{ControlledDocumentId, ObligationId};
use crate::{ASSURANCE_IR_SCHEMA, ControlId, PrincipalRef, RiskId, SubjectSelector};

fn schema_version_default() -> String {
    ASSURANCE_IR_SCHEMA.to_string()
}

/// Governed document class. Extensible via [`DocumentType::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentType {
    Policy,
    Standard,
    Procedure,
    Plan,
    Runbook,
    Guideline,
    Record,
    Other(String),
}

/// Lifecycle of one version's metadata. Operational currency is derived at T.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentVersionStatus {
    Draft,
    Approved,
    Retired,
}

/// Confidentiality / classification. Extensible via [`InformationClassification::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum InformationClassification {
    Public,
    #[default]
    Internal,
    Confidential,
    Restricted,
    Other(String),
}

/// Fail-closed document-control errors. `MissingApproval` and `Immutable*` are matchable.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DocumentControlError {
    #[error("missing approval: empty approvers or empty approval evidence for version {version}")]
    MissingApproval { version: String },
    #[error("approved artifact digest is immutable for version {version}")]
    ImmutableApprovedArtifact { version: String },
    #[error("duplicate document id {id}")]
    DuplicateDocument { id: String },
    #[error("duplicate version {version} on document {id}")]
    DuplicateVersion { id: String, version: String },
    #[error("dangling {kind} id {id}")]
    DanglingReference { kind: &'static str, id: String },
    #[error("current version {version} does not exist on document {id}")]
    UnknownCurrentVersion { id: String, version: String },
    #[error("supersedes version {supersedes} does not exist on document {id}")]
    UnknownSupersedes { id: String, supersedes: String },
    #[error("supersession cycle on document {id}")]
    SupersessionCycle { id: String },
    #[error("artifact digest is empty on version {version}")]
    EmptyArtifactDigest { version: String },
    #[error("acknowledgement required but no subjects listed on version {version}")]
    MissingAcknowledgementSubjects { version: String },
    #[error("version {version} not found")]
    VersionNotFound { version: String },
    #[error("cannot approve version {version} in status {status:?}")]
    NotDraft {
        version: String,
        status: DocumentVersionStatus,
    },
}

/// Caller-supplied inventory for fail-closed link checks. Empty + any link ⇒ dangling.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinkUniverse {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub control_ids: BTreeSet<ControlId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub obligation_ids: BTreeSet<ObligationId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub risk_ids: BTreeSet<RiskId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub subject_ids: BTreeSet<String>,
}

/// Recorded acknowledgement against one document version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgementRecord {
    pub subject_id: String,
    pub acknowledged_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_digest: Option<String>,
}

/// Coverage of required acknowledgements for one version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgementCoverage {
    pub required: usize,
    pub recorded: usize,
    pub complete: bool,
}

/// Retention schedule. The struct is required even when periods are unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RetentionMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retain_until: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_period_seconds: Option<u64>,
    #[serde(default)]
    pub legal_hold: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,
}

/// One immutable version of a controlled document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersion {
    pub version: String,
    artifact_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_ref: Option<String>,
    pub status: DocumentVersionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_by: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approvers: Vec<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_evidence_digests: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applicability: Vec<SubjectSelector>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligation_ids: Vec<ObligationId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_ids: Vec<RiskId>,
    #[serde(default)]
    pub acknowledgement_required: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_acknowledgement_subjects: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acknowledgements: Vec<AcknowledgementRecord>,
    #[serde(default)]
    pub classification: InformationClassification,
    #[serde(default)]
    pub retention: RetentionMetadata,
}

impl DocumentVersion {
    /// New draft version. Artifact digest is the sealed envelope `content_digest`.
    pub fn draft(version: impl Into<String>, artifact_digest: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            artifact_digest: artifact_digest.into(),
            artifact_ref: None,
            status: DocumentVersionStatus::Draft,
            effective_from: None,
            review_by: None,
            approvers: Vec::new(),
            approval_evidence_digests: Vec::new(),
            supersedes_version: None,
            applicability: Vec::new(),
            control_ids: Vec::new(),
            obligation_ids: Vec::new(),
            risk_ids: Vec::new(),
            acknowledgement_required: false,
            required_acknowledgement_subjects: Vec::new(),
            acknowledgements: Vec::new(),
            classification: InformationClassification::Internal,
            retention: RetentionMetadata::default(),
        }
    }

    pub fn artifact_digest(&self) -> &str {
        &self.artifact_digest
    }

    pub fn retention(&self) -> &RetentionMetadata {
        &self.retention
    }

    /// Draft-only replacement. Approved/retired bytes are immutable.
    pub fn set_artifact_digest(
        &mut self,
        digest: impl Into<String>,
    ) -> Result<(), DocumentControlError> {
        if self.status != DocumentVersionStatus::Draft {
            return Err(DocumentControlError::ImmutableApprovedArtifact {
                version: self.version.clone(),
            });
        }
        let digest = digest.into();
        if digest.trim().is_empty() {
            return Err(DocumentControlError::EmptyArtifactDigest {
                version: self.version.clone(),
            });
        }
        self.artifact_digest = digest;
        Ok(())
    }

    pub fn with_supersedes(mut self, version: impl Into<String>) -> Self {
        self.supersedes_version = Some(version.into());
        self
    }

    pub fn with_artifact_ref(mut self, artifact_ref: impl Into<String>) -> Self {
        self.artifact_ref = Some(artifact_ref.into());
        self
    }

    pub fn with_retention(mut self, retention: RetentionMetadata) -> Self {
        self.retention = retention;
        self
    }

    pub fn with_classification(mut self, classification: InformationClassification) -> Self {
        self.classification = classification;
        self
    }

    pub fn is_approved(&self) -> bool {
        self.status == DocumentVersionStatus::Approved
    }

    /// Approved and `effective_from <= t`.
    pub fn is_effective_at(&self, t: DateTime<Utc>) -> bool {
        self.is_approved() && self.effective_from.is_some_and(|start| start <= t)
    }

    /// `review_by` is scheduled and `t <= review_by`. Unscheduled is not in-window.
    pub fn within_review_window(&self, t: DateTime<Utc>) -> bool {
        self.review_by.is_some_and(|end| t <= end)
    }

    /// Approved + dated-effective + in review window. Not `Effectiveness`.
    pub fn is_operational_current_at(&self, t: DateTime<Utc>) -> bool {
        self.is_effective_at(t) && self.within_review_window(t)
    }

    pub fn acknowledgement_coverage(&self) -> AcknowledgementCoverage {
        if !self.acknowledgement_required {
            return AcknowledgementCoverage {
                required: 0,
                recorded: 0,
                complete: true,
            };
        }
        let required = self.required_acknowledgement_subjects.len();
        let recorded = self
            .required_acknowledgement_subjects
            .iter()
            .filter(|subject| {
                self.acknowledgements
                    .iter()
                    .any(|ack| &ack.subject_id == *subject)
            })
            .count();
        AcknowledgementCoverage {
            required,
            recorded,
            complete: required > 0 && recorded == required,
        }
    }
}

/// Stable identity with an append-only version list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlledDocument {
    #[serde(default = "schema_version_default")]
    pub schema_version: String,
    pub id: ControlledDocumentId,
    pub document_type: DocumentType,
    pub title: String,
    pub owner: PrincipalRef,
    #[serde(default)]
    pub versions: Vec<DocumentVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_version: Option<String>,
}

impl ControlledDocument {
    pub fn new(
        id: ControlledDocumentId,
        document_type: DocumentType,
        title: impl Into<String>,
        owner: PrincipalRef,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.to_string(),
            id,
            document_type,
            title: title.into(),
            owner,
            versions: Vec::new(),
            current_version: None,
        }
    }

    pub fn append_version(
        &mut self,
        mut version: DocumentVersion,
    ) -> Result<(), DocumentControlError> {
        if version.artifact_digest.trim().is_empty() {
            return Err(DocumentControlError::EmptyArtifactDigest {
                version: version.version.clone(),
            });
        }
        if self.version(&version.version).is_some() {
            return Err(DocumentControlError::DuplicateVersion {
                id: self.id.as_str().to_string(),
                version: version.version.clone(),
            });
        }
        version.status = DocumentVersionStatus::Draft;
        version.effective_from = None;
        self.versions.push(version);
        Ok(())
    }

    pub fn approve(
        &mut self,
        version: &str,
        approvers: Vec<PrincipalRef>,
        approval_evidence_digests: Vec<String>,
        effective_from: DateTime<Utc>,
        review_by: Option<DateTime<Utc>>,
    ) -> Result<(), DocumentControlError> {
        if approvers.is_empty() || approval_evidence_digests.is_empty() {
            return Err(DocumentControlError::MissingApproval {
                version: version.to_string(),
            });
        }
        let idx = self
            .versions
            .iter()
            .position(|v| v.version == version)
            .ok_or_else(|| DocumentControlError::VersionNotFound {
                version: version.to_string(),
            })?;
        if self.versions[idx].status != DocumentVersionStatus::Draft {
            return Err(DocumentControlError::NotDraft {
                version: version.to_string(),
                status: self.versions[idx].status,
            });
        }
        if self.versions[idx].artifact_digest.trim().is_empty() {
            return Err(DocumentControlError::EmptyArtifactDigest {
                version: version.to_string(),
            });
        }
        let v = &mut self.versions[idx];
        v.approvers = approvers;
        v.approval_evidence_digests = approval_evidence_digests;
        v.effective_from = Some(effective_from);
        v.review_by = review_by;
        v.status = DocumentVersionStatus::Approved;
        self.current_version = Some(version.to_string());
        Ok(())
    }

    pub fn set_artifact_digest(
        &mut self,
        version: &str,
        digest: impl Into<String>,
    ) -> Result<(), DocumentControlError> {
        let v = self
            .versions
            .iter_mut()
            .find(|v| v.version == version)
            .ok_or_else(|| DocumentControlError::VersionNotFound {
                version: version.to_string(),
            })?;
        v.set_artifact_digest(digest)
    }

    pub fn version(&self, version: &str) -> Option<&DocumentVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    pub fn version_mut(&mut self, version: &str) -> Option<&mut DocumentVersion> {
        self.versions.iter_mut().find(|v| v.version == version)
    }

    pub fn current(&self) -> Option<&DocumentVersion> {
        self.current_version
            .as_deref()
            .and_then(|v| self.version(v))
    }

    /// Version that was operational at `t` (approved, effective, in review window,
    /// not superseded by another such version).
    pub fn effective_version_at(&self, t: DateTime<Utc>) -> Option<&DocumentVersion> {
        self.effective_at(t)
    }

    pub fn effective_at(&self, t: DateTime<Utc>) -> Option<&DocumentVersion> {
        let candidates: Vec<&DocumentVersion> = self
            .versions
            .iter()
            .filter(|v| v.is_operational_current_at(t))
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let superseded: BTreeSet<&str> = candidates
            .iter()
            .filter_map(|v| v.supersedes_version.as_deref())
            .collect();
        let mut remaining: Vec<&DocumentVersion> = candidates
            .into_iter()
            .filter(|v| !superseded.contains(v.version.as_str()))
            .collect();
        if remaining.is_empty() {
            return None;
        }
        if remaining.len() == 1 {
            return remaining.pop();
        }
        if let Some(current) = self.current_version.as_deref()
            && let Some(idx) = remaining.iter().position(|v| v.version == current)
        {
            return Some(remaining[idx]);
        }
        remaining.sort_by_key(|v| v.effective_from);
        remaining.pop()
    }

    /// Current pointer is operational at `t`.
    pub fn is_operational_current_at(&self, t: DateTime<Utc>) -> bool {
        self.current()
            .is_some_and(|v| v.is_operational_current_at(t))
    }

    pub fn validate(&self, universe: &DocumentLinkUniverse) -> Result<(), DocumentControlError> {
        let mut seen_versions = BTreeSet::new();
        for version in &self.versions {
            if !seen_versions.insert(version.version.as_str()) {
                return Err(DocumentControlError::DuplicateVersion {
                    id: self.id.as_str().to_string(),
                    version: version.version.clone(),
                });
            }
            if version.artifact_digest.trim().is_empty() {
                return Err(DocumentControlError::EmptyArtifactDigest {
                    version: version.version.clone(),
                });
            }
            if version.is_approved()
                && (version.approvers.is_empty() || version.approval_evidence_digests.is_empty())
            {
                return Err(DocumentControlError::MissingApproval {
                    version: version.version.clone(),
                });
            }
            if version.acknowledgement_required
                && version.required_acknowledgement_subjects.is_empty()
            {
                return Err(DocumentControlError::MissingAcknowledgementSubjects {
                    version: version.version.clone(),
                });
            }
            for control_id in &version.control_ids {
                if !universe.control_ids.contains(control_id) {
                    return Err(DocumentControlError::DanglingReference {
                        kind: "control",
                        id: control_id.as_str().to_string(),
                    });
                }
            }
            for obligation_id in &version.obligation_ids {
                if !universe.obligation_ids.contains(obligation_id) {
                    return Err(DocumentControlError::DanglingReference {
                        kind: "obligation",
                        id: obligation_id.as_str().to_string(),
                    });
                }
            }
            for risk_id in &version.risk_ids {
                if !universe.risk_ids.contains(risk_id) {
                    return Err(DocumentControlError::DanglingReference {
                        kind: "risk",
                        id: risk_id.as_str().to_string(),
                    });
                }
            }
            for selector in &version.applicability {
                for subject_id in &selector.ids {
                    if !subject_id.is_empty() && !universe.subject_ids.contains(subject_id) {
                        return Err(DocumentControlError::DanglingReference {
                            kind: "subject",
                            id: subject_id.clone(),
                        });
                    }
                }
            }
            for subject_id in &version.required_acknowledgement_subjects {
                if !subject_id.is_empty() && !universe.subject_ids.contains(subject_id) {
                    return Err(DocumentControlError::DanglingReference {
                        kind: "subject",
                        id: subject_id.clone(),
                    });
                }
            }
            if let Some(supersedes) = version.supersedes_version.as_deref()
                && self.version(supersedes).is_none()
            {
                return Err(DocumentControlError::UnknownSupersedes {
                    id: self.id.as_str().to_string(),
                    supersedes: supersedes.to_string(),
                });
            }
        }
        if let Some(current) = self.current_version.as_deref()
            && self.version(current).is_none()
        {
            return Err(DocumentControlError::UnknownCurrentVersion {
                id: self.id.as_str().to_string(),
                version: current.to_string(),
            });
        }
        if has_supersession_cycle(self) {
            return Err(DocumentControlError::SupersessionCycle {
                id: self.id.as_str().to_string(),
            });
        }
        Ok(())
    }
}

fn has_supersession_cycle(doc: &ControlledDocument) -> bool {
    for start in &doc.versions {
        let mut seen = BTreeSet::new();
        let mut cursor = Some(start.version.as_str());
        while let Some(version) = cursor {
            if !seen.insert(version) {
                return true;
            }
            cursor = doc
                .version(version)
                .and_then(|v| v.supersedes_version.as_deref());
        }
    }
    false
}

/// Standalone document-control registry. Not an `AssessmentDefinition` field.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentControlRegistry {
    #[serde(default)]
    pub documents: Vec<ControlledDocument>,
}

impl DocumentControlRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, document: ControlledDocument) -> Result<(), DocumentControlError> {
        if self.get(&document.id).is_some() {
            return Err(DocumentControlError::DuplicateDocument {
                id: document.id.as_str().to_string(),
            });
        }
        self.documents.push(document);
        Ok(())
    }

    pub fn get(&self, id: &ControlledDocumentId) -> Option<&ControlledDocument> {
        self.documents.iter().find(|d| d.id == *id)
    }

    pub fn get_mut(&mut self, id: &ControlledDocumentId) -> Option<&mut ControlledDocument> {
        self.documents.iter_mut().find(|d| d.id == *id)
    }

    pub fn version(&self, id: &ControlledDocumentId, version: &str) -> Option<&DocumentVersion> {
        self.get(id).and_then(|d| d.version(version))
    }

    pub fn current(&self, id: &ControlledDocumentId) -> Option<&DocumentVersion> {
        self.get(id).and_then(ControlledDocument::current)
    }

    pub fn effective_version_at(
        &self,
        id: &ControlledDocumentId,
        t: DateTime<Utc>,
    ) -> Option<&DocumentVersion> {
        self.get(id).and_then(|d| d.effective_version_at(t))
    }

    pub fn validate(&self, universe: &DocumentLinkUniverse) -> Result<(), DocumentControlError> {
        let mut seen = BTreeSet::new();
        for document in &self.documents {
            if !seen.insert(document.id.as_str()) {
                return Err(DocumentControlError::DuplicateDocument {
                    id: document.id.as_str().to_string(),
                });
            }
            document.validate(universe)?;
        }
        Ok(())
    }
}
