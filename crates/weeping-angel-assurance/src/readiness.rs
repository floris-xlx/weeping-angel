//! Framework readiness projection. Never certification.

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use weeping_angel_assurance_ir::{
    AssessmentId, ControlId, MappingCompleteness, MappingRelation, RequirementId,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};
use weeping_angel_framework::CompiledFramework;

/// Explicit graph verbs used when projecting readiness.
#[allow(dead_code)]
const GRAPH_VERBS: &[&str] = &[
    "MapsTo",
    "TestedBy",
    "RequiresEvidence",
    "PartiallySatisfies",
    "Satisfies",
    "Supports",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkReadinessSnapshot {
    pub assessment_id: AssessmentId,
    pub framework: String,
    pub framework_version: String,
    pub framework_pack_digest: String,
    #[serde(default)]
    pub catalog_digest: String,
    pub assessment_digest: String,
    pub evaluated_at: String,
    pub requirements: Vec<RequirementReadiness>,
    pub controls: Vec<ControlReadiness>,
    pub effective: u32,
    pub ineffective: u32,
    pub partial: u32,
    pub manual_review: u32,
    pub insufficient_evidence: u32,
    pub not_applicable: u32,
    pub automation_coverage: String,
    pub evidence_coverage: String,
}

impl Serialize for FrameworkReadinessSnapshot {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let total_controls = self.controls.len();
        let automated = self
            .controls
            .iter()
            .filter(|c| {
                !matches!(
                    c.effectiveness,
                    Effectiveness::ManualReviewRequired | Effectiveness::NotTested
                )
            })
            .count();
        let evidenced = self
            .controls
            .iter()
            .filter(|c| {
                !matches!(
                    c.effectiveness,
                    Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence
                )
            })
            .count();
        let subjects = self
            .controls
            .iter()
            .filter(|c| {
                !matches!(
                    c.effectiveness,
                    Effectiveness::InsufficientEvidence | Effectiveness::NotTested
                )
            })
            .count();
        let req_total = self.requirements.len();
        let req_covered = self
            .requirements
            .iter()
            .filter(|r| !r.mapped_controls.is_empty())
            .count();
        let mut state = serializer.serialize_struct("FrameworkReadinessSnapshot", 22)?;
        state.serialize_field("assessmentId", &self.assessment_id)?;
        state.serialize_field("framework", &self.framework)?;
        state.serialize_field("frameworkVersion", &self.framework_version)?;
        state.serialize_field("frameworkPackDigest", &self.framework_pack_digest)?;
        state.serialize_field("catalogDigest", &self.catalog_digest)?;
        state.serialize_field("assessmentDigest", &self.assessment_digest)?;
        state.serialize_field("evaluatedAt", &self.evaluated_at)?;
        state.serialize_field("requirements", &self.requirements)?;
        state.serialize_field("controls", &self.controls)?;
        state.serialize_field("effective", &self.effective)?;
        state.serialize_field("ineffective", &self.ineffective)?;
        state.serialize_field("partial", &self.partial)?;
        state.serialize_field("manualReview", &self.manual_review)?;
        state.serialize_field("insufficientEvidence", &self.insufficient_evidence)?;
        state.serialize_field("notApplicable", &self.not_applicable)?;
        state.serialize_field(
            "automationCoverage",
            &coverage_counts(automated, total_controls),
        )?;
        state.serialize_field(
            "evidenceCoverage",
            &coverage_counts(evidenced, total_controls),
        )?;
        state.serialize_field(
            "subjectCoverage",
            &coverage_counts(subjects, total_controls),
        )?;
        state.serialize_field(
            "controlCoverage",
            &coverage_counts(total_controls, total_controls),
        )?;
        state.serialize_field(
            "frameworkRequirementCoverage",
            &coverage_counts(req_covered, req_total),
        )?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementReadiness {
    pub id: RequirementId,
    pub status: String,
    pub mapped_controls: Vec<ControlId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlReadiness {
    pub id: ControlId,
    pub effectiveness: Effectiveness,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CoverageCounts {
    covered: usize,
    total: usize,
    count: usize,
}

fn coverage_counts(covered: usize, total: usize) -> CoverageCounts {
    CoverageCounts {
        covered,
        total,
        count: covered,
    }
}

fn relation_may_fully_satisfy(
    relation: MappingRelation,
    completeness: MappingCompleteness,
) -> bool {
    match relation {
        MappingRelation::Equivalent | MappingRelation::Satisfies | MappingRelation::SupersetOf => {
            completeness == MappingCompleteness::Full
        }
        MappingRelation::PartiallySatisfies
        | MappingRelation::Supports
        | MappingRelation::Related
        | MappingRelation::EvidenceFor
        | MappingRelation::SubsetOf => false,
    }
}

pub fn project_readiness(
    compiled: &CompiledFramework,
    results: &[ControlTestResult],
    framework: &str,
    framework_version: &str,
    framework_pack_digest: &str,
    assessment_id: AssessmentId,
) -> FrameworkReadinessSnapshot {
    let _ = GRAPH_VERBS;
    let mut effective = 0;
    let mut ineffective = 0;
    let mut partial = 0;
    let mut manual_review = 0;
    let mut insufficient_evidence = 0;
    let mut not_applicable = 0;
    let mut controls = Vec::new();
    for result in results {
        match result.effectiveness {
            Effectiveness::Effective => effective += 1,
            Effectiveness::Ineffective => ineffective += 1,
            Effectiveness::PartiallyEffective => partial += 1,
            Effectiveness::ManualReviewRequired => manual_review += 1,
            Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence => {
                insufficient_evidence += 1
            }
            Effectiveness::NotApplicable => not_applicable += 1,
            _ => {}
        }
        controls.push(ControlReadiness {
            id: result.control_id.clone(),
            effectiveness: result.effectiveness,
        });
    }

    let mut requirements = Vec::new();
    for req in &compiled.applicable_requirements {
        let req_mappings: Vec<_> = compiled
            .mappings
            .iter()
            .filter(|m| m.from_requirement() == req.id())
            .collect();
        let mapped: Vec<ControlId> = req_mappings
            .iter()
            .map(|m| m.to_control().clone())
            .collect();
        let related: Vec<_> = results
            .iter()
            .filter(|r| mapped.iter().any(|c| c == &r.control_id))
            .collect();
        let any_ineff = related
            .iter()
            .any(|r| r.effectiveness == Effectiveness::Ineffective);
        let any_stale = related
            .iter()
            .any(|r| r.effectiveness == Effectiveness::StaleEvidence);
        let any_insufficient = related
            .iter()
            .any(|r| r.effectiveness == Effectiveness::InsufficientEvidence);
        let any_manual = related
            .iter()
            .any(|r| r.effectiveness == Effectiveness::ManualReviewRequired);
        let all_eff = !related.is_empty()
            && related
                .iter()
                .all(|r| r.effectiveness == Effectiveness::Effective);
        let has_partial = req_mappings.is_empty()
            || req_mappings
                .iter()
                .any(|m| !relation_may_fully_satisfy(m.relation(), m.completeness()));
        let status = if mapped.is_empty() {
            "manual review required".to_string()
        } else if any_ineff {
            "ineffective".into()
        } else if any_stale {
            "stale evidence".into()
        } else if any_insufficient {
            "insufficient evidence".into()
        } else if any_manual {
            "manual review required".into()
        } else if all_eff && has_partial {
            "partially covered".into()
        } else if all_eff {
            "effective".into()
        } else {
            "insufficient evidence".into()
        };
        requirements.push(RequirementReadiness {
            id: req.id().clone(),
            status,
            mapped_controls: mapped,
        });
    }

    FrameworkReadinessSnapshot {
        assessment_id,
        framework: framework.into(),
        framework_version: framework_version.into(),
        framework_pack_digest: framework_pack_digest.into(),
        catalog_digest: compiled.catalog_digest.clone(),
        assessment_digest: compiled.digest.clone(),
        evaluated_at: chrono::Utc::now().to_rfc3339(),
        requirements,
        controls,
        effective,
        ineffective,
        partial,
        manual_review,
        insufficient_evidence,
        not_applicable,
        automation_coverage: String::new(),
        evidence_coverage: String::new(),
    }
}
