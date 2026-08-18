//! Framework readiness projection. Never certification.

use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{AssessmentId, ControlId, MappingRelation, RequirementId};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkReadinessSnapshot {
    pub assessment_id: AssessmentId,
    pub framework: String,
    pub framework_version: String,
    pub framework_pack_digest: String,
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
        let mapped: Vec<ControlId> = compiled.controls.iter().map(|c| c.id().clone()).collect();
        let mut status = "insufficient evidence".to_string();
        let related: Vec<_> = results
            .iter()
            .filter(|r| mapped.iter().any(|c| c == &r.control_id))
            .collect();
        let any_manual = related
            .iter()
            .any(|r| r.effectiveness == Effectiveness::ManualReviewRequired);
        let any_ineff = related
            .iter()
            .any(|r| r.effectiveness == Effectiveness::Ineffective);
        let all_eff = !related.is_empty()
            && related
                .iter()
                .all(|r| r.effectiveness == Effectiveness::Effective);
        // A partial mapping cannot fully satisfy a requirement.
        let has_partial = true;
        if any_ineff {
            status = "ineffective".into();
        } else if any_manual {
            status = "manual review required".into();
        } else if all_eff && has_partial {
            status = "partially covered".into();
        } else if all_eff {
            status = "effective".into();
        }
        let _ = MappingRelation::PartiallySatisfies;
        requirements.push(RequirementReadiness {
            id: req.id().clone(),
            status,
            mapped_controls: mapped,
        });
    }

    let total = results.len().max(1) as f64;
    let automated = results
        .iter()
        .filter(|r| {
            !matches!(
                r.effectiveness,
                Effectiveness::ManualReviewRequired | Effectiveness::NotTested
            )
        })
        .count() as f64;
    let evidenced = results
        .iter()
        .filter(|r| {
            !matches!(
                r.effectiveness,
                Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence
            )
        })
        .count() as f64;

    FrameworkReadinessSnapshot {
        assessment_id,
        framework: framework.into(),
        framework_version: framework_version.into(),
        framework_pack_digest: framework_pack_digest.into(),
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
        automation_coverage: format!("{:.0}%", (automated / total) * 100.0),
        evidence_coverage: format!("{:.0}%", (evidenced / total) * 100.0),
    }
}
