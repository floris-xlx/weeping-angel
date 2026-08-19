//! Immutable assessment runs and snapshot comparison.

use std::path::PathBuf;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use weeping_angel_assurance_ir::AssessmentId;
use weeping_angel_canonical_catalog::CanonicalCatalog;

use crate::readiness::FrameworkReadinessSnapshot;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentRun {
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
}

impl AssessmentRun {
    pub fn catalog_digest(&self) -> String {
        catalog_digest()
    }
}

impl Serialize for AssessmentRun {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AssessmentRun", 12)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("framework", &self.framework)?;
        state.serialize_field("frameworkPackDigest", &self.framework_pack_digest)?;
        state.serialize_field("catalogDigest", &self.catalog_digest())?;
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
        state.end()
    }
}

pub fn catalog_digest() -> String {
    for root in catalog_search_roots() {
        if let Ok(catalog) = CanonicalCatalog::load(&root)
            && let Ok(digest) = catalog.digest()
        {
            return digest.to_string();
        }
    }
    String::new()
}

fn catalog_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = PathBuf::from(dir);
        roots.push(base.join("catalog/canonical/v1"));
        roots.push(base.join("..").join("catalog/canonical/v1"));
        roots.push(base.join("..").join("..").join("catalog/canonical/v1"));
    }
    roots.push(PathBuf::from("catalog/canonical/v1"));
    roots
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
}

pub fn compare(
    previous: &FrameworkReadinessSnapshot,
    next: &FrameworkReadinessSnapshot,
) -> SnapshotDiff {
    let mut diff = SnapshotDiff::default();
    for ctl in &next.controls {
        let prior = previous.controls.iter().find(|c| c.id == ctl.id);
        match (prior.map(|c| c.effectiveness), ctl.effectiveness) {
            (
                Some(weeping_angel_control_test::Effectiveness::Ineffective),
                weeping_angel_control_test::Effectiveness::Effective,
            ) => diff
                .control_became_effective
                .push(format!("{} became effective", ctl.id)),
            (
                Some(weeping_angel_control_test::Effectiveness::Effective),
                weeping_angel_control_test::Effectiveness::Ineffective,
            ) => diff
                .control_became_ineffective
                .push(format!("{} became ineffective", ctl.id)),
            (_, weeping_angel_control_test::Effectiveness::StaleEvidence) => {
                diff.evidence_became_stale.push(ctl.id.to_string())
            }
            _ => {}
        }
    }
    diff
}
