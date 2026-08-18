//! Local filesystem / manual evidence. Structural checks only.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use chrono::Utc;
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};

use crate::{
    CollectorCapabilities, CollectorDescriptor, CollectorError, CollectorScope, EvidenceCollector,
};

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
}

impl EvidenceCollector for LocalCollector {
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

    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        let mut out = Vec::new();
        let collected_at = Utc::now();
        let asset = AssetId::new("repo:local");
        if !scope.allows(&asset) {
            return Err(CollectorError::OutOfScope {
                asset: asset.to_string(),
            });
        }
        let codeowners = self.exists("CODEOWNERS")
            || self.exists(".github/CODEOWNERS")
            || self.exists("docs/CODEOWNERS");
        let obs = EvidenceObservation::new(EvidenceType::new("source.codeowners.present"))
            .with_fact("present", if codeowners { "true" } else { "false" })
            .with_narrative("CODEOWNERS presence is structural, not effectiveness");
        let prov = EvidenceProvenance {
            collector_id: "collector.local".into(),
            collected_at,
            scope: scope.as_label(),
            asset,
        };
        out.push(EvidenceEnvelope::seal(obs, prov)?);
        Ok(out)
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
        let obs = EvidenceObservation::new(self.evidence_type.clone())
            .with_fact("attested_by", &self.attested_by)
            .with_fact("reason", &self.reason)
            .with_narrative(format!(
                "manual evidence attested-by {} for {}",
                self.attested_by, self.subject
            ));
        let prov = EvidenceProvenance {
            collector_id: "collector.manual".into(),
            collected_at,
            scope: self.subject.to_string(),
            asset: self.subject.clone(),
        };
        Ok(EvidenceEnvelope::seal(obs, prov)?)
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
