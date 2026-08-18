//! Immutable assessment runs and snapshot comparison.

use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::AssessmentId;

use crate::readiness::FrameworkReadinessSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
