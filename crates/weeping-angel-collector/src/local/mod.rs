//! Local filesystem / manual evidence. Structural checks only.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use chrono::Utc;
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceType, EvidenceValue};

use crate::application::EnvelopeFactory;
use crate::domain::{
    CollectionCoverage, CollectionRequest, CollectorCapabilities, CollectorDescriptor,
    CollectorInstance, CollectorScope, CredentialRef, ObservationBatch, ObservationCandidate,
};
use crate::ports::CollectorAdapter;
use crate::{CollectorError, EvidenceCollector};

/// Local collector: CODEOWNERS, SECURITY.md, CI workflow presence.
pub struct LocalCollector {
    root: PathBuf,
}

impl LocalCollector {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn exists(&self, rel: &str) -> bool {
        let path = self.root.join(rel);
        path.is_file()
    }

    fn instance(&self) -> CollectorInstance {
        CollectorInstance::new(
            "local:default",
            "collector.local",
            CredentialRef::new("local:default"),
        )
    }
}

impl CollectorAdapter for LocalCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorDescriptor {
            id: "collector.local".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            evidence_types: [
                "source.codeowners.present",
                "policy.security.reviewed",
                "source.workflow.permissions",
            ]
            .into_iter()
            .map(EvidenceType::new)
            .collect(),
            provider_family: "local-fs".into(),
            subject_types: BTreeSet::from(["repository".into()]),
            capabilities: CollectorCapabilities {
                offline: true,
                worker_safe: true,
                ..CollectorCapabilities::default()
            },
            required_permissions: Vec::new(),
        }
    }

    fn collect_observations(
        &self,
        _instance: &CollectorInstance,
        request: &CollectionRequest,
    ) -> Result<ObservationBatch, CollectorError> {
        let collected_at = Utc::now();
        let asset = AssetId::new("repo:local");
        if !request.scope.allows(&asset) {
            return Err(CollectorError::OutOfScope {
                asset: asset.to_string(),
            });
        }
        let codeowners = self.exists("CODEOWNERS")
            || self.exists(".github/CODEOWNERS")
            || self.exists("docs/CODEOWNERS");
        let candidate = ObservationCandidate {
            asset,
            evidence_type: EvidenceType::new("source.codeowners.present"),
            facts: [(
                "present".into(),
                EvidenceValue::String(if codeowners { "true" } else { "false" }.into()),
            )]
            .into_iter()
            .collect(),
            narrative: "CODEOWNERS presence is structural, not effectiveness".into(),
            observed_at: Some(collected_at),
            valid_from: None,
            valid_until: None,
            source_revision: None,
        };
        Ok(ObservationBatch {
            candidates: vec![candidate],
            diagnostics: Vec::new(),
            coverage: CollectionCoverage {
                hole: false,
                strict_scope: true,
            },
            collected_at: Some(collected_at),
        })
    }
}

impl EvidenceCollector for LocalCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorAdapter::descriptor(self)
    }

    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        crate::application::engine::collect_envelopes(self, &self.instance(), scope)
    }
}

/// Manual evidence producer. Attestation is never synthesized.
pub struct ManualEvidence {
    pub evidence_type: EvidenceType,
    pub subject: AssetId,
    /// CLI flag: --attested-by
    pub attested_by: String,
    pub reason: String,
    pub artifact: Option<PathBuf>,
}

impl ManualEvidence {
    pub fn seal(
        &self,
        collected_at: chrono::DateTime<Utc>,
    ) -> Result<EvidenceEnvelope, CollectorError> {
        if self.attested_by.trim().is_empty() {
            return Err(CollectorError::InsufficientEvidence {
                detail: "manual evidence requires attested-by; attestation is never synthesized"
                    .into(),
            });
        }
        let candidate = ObservationCandidate {
            asset: self.subject.clone(),
            evidence_type: self.evidence_type.clone(),
            facts: [
                (
                    "attested_by".into(),
                    EvidenceValue::String(self.attested_by.clone()),
                ),
                ("reason".into(), EvidenceValue::String(self.reason.clone())),
            ]
            .into_iter()
            .collect(),
            narrative: format!(
                "manual evidence attested-by {} for {}",
                self.attested_by, self.subject
            ),
            observed_at: Some(collected_at),
            valid_from: None,
            valid_until: None,
            source_revision: None,
        };
        let scope = CollectorScope::new().allow_asset(self.subject.clone());
        let instance = CollectorInstance::new(
            "manual:default",
            "collector.manual",
            CredentialRef::new("manual:default"),
        );
        let batch = ObservationBatch {
            candidates: vec![candidate.clone()],
            diagnostics: Vec::new(),
            coverage: CollectionCoverage::default(),
            collected_at: Some(collected_at),
        };
        EnvelopeFactory::new().seal_candidate(&instance, &scope, &batch, &candidate)
    }

    pub fn from_file(
        evidence_type: EvidenceType,
        subject: AssetId,
        attested_by: String,
        file: &Path,
    ) -> Result<Self, CollectorError> {
        if !file.is_file() {
            return Err(CollectorError::InsufficientEvidence {
                detail: format!("manual artifact missing: {}", file.display()),
            });
        }
        Ok(Self {
            evidence_type,
            subject,
            attested_by,
            reason: "user-supplied artifact".into(),
            artifact: Some(file.to_path_buf()),
        })
    }
}
