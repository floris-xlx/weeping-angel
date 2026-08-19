//! Immutable assessment runs and snapshot comparison.

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use weeping_angel_assurance_ir::AssessmentId;
use weeping_angel_control_test::Effectiveness;

use crate::readiness::FrameworkReadinessSnapshot;

/// Live catalog walk for establishing a pin at assess start. Serialize must not call this.
pub fn catalog_digest() -> String {
    use std::path::PathBuf;
    use weeping_angel_canonical_catalog::CanonicalCatalog;

    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = PathBuf::from(dir);
        roots.push(base.join("catalog/canonical/v1"));
        roots.push(base.join("../..").join("catalog/canonical/v1"));
        roots.push(base.join("..").join("catalog/canonical/v1"));
    }
    roots.push(PathBuf::from("catalog/canonical/v1"));
    for root in roots {
        if let Ok(catalog) = CanonicalCatalog::load(&root)
            && let Ok(digest) = catalog.digest()
        {
            return digest.to_string();
        }
    }
    "catalog-unavailable".into()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentRun {
    // `as_of` / asOf is serialized from the as_of field (pinned evaluation clock).
    pub id: AssessmentId,
    pub framework: String,
    pub framework_pack_digest: String,
    pub assessment_definition_digest: String,
    pub started_at: String,
    pub completed_at: String,
    pub scope: String,
    pub collector_runs: Vec<String>,
    pub evidence_snapshot_digest: String, // evidenceSnapshotDigest
    pub result_digest: String,
    pub status: String,
    /// Pinned canonical catalog identity. JSON names are `canonicalCatalogDigest` and `catalogDigest`.
    #[serde(default, alias = "catalogDigest", alias = "canonicalCatalogDigest")]
    pub canonical_catalog_pin: String,
    #[serde(default, rename = "applicabilitySnapshotId")]
    pub applicability_snapshot_id: String,
    /// Pinned evaluation clock. JSON `asOf`.
    #[serde(default)]
    pub as_of: String,
}

impl Serialize for AssessmentRun {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let catalog = self.canonical_catalog_pin.clone();
        let mut state = serializer.serialize_struct("AssessmentRun", 15)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("framework", &self.framework)?;
        state.serialize_field("frameworkPackDigest", &self.framework_pack_digest)?;
        state.serialize_field(
            "assessmentDefinitionDigest",
            &self.assessment_definition_digest,
        )?;
        state.serialize_field("startedAt", &self.started_at)?;
        state.serialize_field("completedAt", &self.completed_at)?;
        state.serialize_field("scope", &self.scope)?;
        state.serialize_field("collectorRuns", &self.collector_runs)?;
        state.serialize_field("evidenceSnapshotDigest", &self.evidence_snapshot_digest)?;
        state.serialize_field("resultDigest", &self.result_digest)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("canonicalCatalogDigest", &catalog)?;
        state.serialize_field("catalogDigest", &catalog)?;
        state.serialize_field("applicabilitySnapshotId", &self.applicability_snapshot_id)?;
        // Pinned evaluation clock (`as_of`) for historical replay; independent of startedAt.
        state.serialize_field("asOf", &self.as_of)?;
        state.end()
    }
}

impl Default for AssessmentRun {
    fn default() -> Self {
        Self {
            id: AssessmentId::new("assess-unset"),
            framework: String::new(),
            framework_pack_digest: String::new(),
            assessment_definition_digest: String::new(),
            started_at: String::new(),
            completed_at: String::new(),
            scope: String::new(),
            collector_runs: Vec::new(),
            evidence_snapshot_digest: String::new(),
            result_digest: String::new(),
            status: String::new(),
            canonical_catalog_pin: String::new(),
            applicability_snapshot_id: String::new(),
            as_of: String::new(),
        }
    }
}

/// Causes of a material Statement-of-Applicability snapshot change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SoaDiffCause {
    ApplicabilityChange,
    ImplementationChange,
    EffectivenessRegression,
    ExceptionExpiry,
    MappingChange,
    TreatmentChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDiff {
    pub control_became_effective: Vec<String>,
    pub control_became_ineffective: Vec<String>,
    pub evidence_became_stale: Vec<String>,
    pub new_subjects: Vec<String>,
    pub disappeared_subjects: Vec<String>,
    pub requirement_became_applicable: Vec<String>,
    pub requirement_became_not_applicable: Vec<String>,
    pub manual_review_resolved: Vec<String>,
    pub new_exceptions: Vec<String>,
    pub expired_exceptions: Vec<String>,
    #[serde(default)]
    pub evidence_added: Vec<String>,
    #[serde(default)]
    pub evidence_removed: Vec<String>,
    #[serde(default)]
    pub evidence_superseded: Vec<String>,
    #[serde(default)]
    pub framework_pack_digest_changed: bool,
    #[serde(default)]
    pub canonical_catalog_digest_changed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soa_causes: Vec<SoaDiffCause>,
}

pub fn compare(
    previous: &FrameworkReadinessSnapshot,
    next: &FrameworkReadinessSnapshot,
) -> SnapshotDiff {
    let mut diff = SnapshotDiff::default();
    for ctl in &next.controls {
        let prior = previous.controls.iter().find(|c| c.id == ctl.id);
        match (prior.map(|c| c.effectiveness), ctl.effectiveness) {
            (Some(Effectiveness::Ineffective), Effectiveness::Effective) => diff
                .control_became_effective
                .push(format!("{} became effective", ctl.id)),
            (Some(Effectiveness::Effective), Effectiveness::Ineffective) => diff
                .control_became_ineffective
                .push(format!("{} became ineffective", ctl.id)),
            (_, Effectiveness::StaleEvidence) => {
                diff.evidence_became_stale.push(ctl.id.to_string())
            }
            (prior, Effectiveness::ExceptionApproved)
                if prior != Some(Effectiveness::ExceptionApproved) =>
            {
                diff.new_exceptions
                    .push(format!("{} exception approved", ctl.id));
            }
            (Some(Effectiveness::ExceptionApproved), next_eff)
                if next_eff != Effectiveness::ExceptionApproved =>
            {
                diff.expired_exceptions
                    .push(format!("{} exception expired", ctl.id));
            }
            _ => {}
        }
        if prior.is_none() {
            diff.control_became_effective
                .push(format!("{} became effective", ctl.id));
        }
    }

    for req in &next.requirements {
        let prior = previous.requirements.iter().find(|r| r.id == req.id);
        let next_applicable = !req.status.eq_ignore_ascii_case("not applicable");
        match prior {
            None if next_applicable => {
                diff.requirement_became_applicable.push(req.id.to_string());
            }
            Some(prev) if prev.status.eq_ignore_ascii_case("not applicable") && next_applicable => {
                diff.requirement_became_applicable.push(req.id.to_string());
            }
            Some(prev)
                if !prev.status.eq_ignore_ascii_case("not applicable") && !next_applicable =>
            {
                diff.requirement_became_not_applicable
                    .push(req.id.to_string());
            }
            _ => {}
        }
        if prior.is_some_and(|p| p.status.contains("manual") && !req.status.contains("manual")) {
            diff.manual_review_resolved.push(req.id.to_string());
        }
    }

    let prev_subjects = subject_ids(previous);
    let next_subjects = subject_ids(next);
    for id in &next_subjects {
        if !prev_subjects.contains(id) {
            diff.new_subjects.push(id.clone());
        }
    }
    for id in &prev_subjects {
        if !next_subjects.contains(id) {
            diff.disappeared_subjects.push(id.clone());
        }
    }

    if previous.framework_pack_digest != next.framework_pack_digest
        || previous.assessment_digest != next.assessment_digest
    {
        diff.framework_pack_digest_changed = true;
        if previous.assessment_digest != next.assessment_digest {
            diff.canonical_catalog_digest_changed = true;
        }
    }

    diff
}

fn subject_ids(snapshot: &FrameworkReadinessSnapshot) -> Vec<String> {
    snapshot.controls.iter().map(|c| c.id.to_string()).collect()
}

/// Compare two execution records for catalog/pack digest and collector lineage.
pub fn compare_runs(previous: &AssessmentRun, next: &AssessmentRun) -> SnapshotDiff {
    let mut diff = SnapshotDiff::default();
    if previous.framework_pack_digest != next.framework_pack_digest {
        diff.framework_pack_digest_changed = true;
    }
    if previous.canonical_catalog_pin != next.canonical_catalog_pin {
        diff.canonical_catalog_digest_changed = true;
    }
    diff
}

pub fn compare_lineage(previous: &AssessmentRun, next: &AssessmentRun) -> SnapshotDiff {
    compare_runs(previous, next)
}
