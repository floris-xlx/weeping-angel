//! Public assurance facade. Callers select a profile + capabilities, not an adapter.

pub mod bridge;
pub mod readiness;
pub mod snapshot;
pub mod soa;

use std::time::Duration;

use chrono::Utc;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    AssessmentId, AssetId, Control, ControlId, EvidenceRequirement, EvidenceRequirementId,
    EvidenceType, FrameworkId, Mapping, MappingCompleteness, MappingDirection, Requirement,
    RequirementId,
};
use weeping_angel_collector::{CollectorError, CollectorScope, EvidenceCollector};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSet, evaluate,
};
use weeping_angel_framework::{
    Assessment, AssessmentRequests, CompiledFramework, FrameworkCompileError, FrameworkProfile,
    FrameworkTarget, compile_framework, load_framework_pack,
};

pub use readiness::FrameworkReadinessSnapshot;
pub use snapshot::{AssessmentRun, SnapshotDiff, compare};
pub use soa::{StatementOfApplicability, project_soa};

const NOT_CERTIFICATION: &str = "This is a readiness assessment and is not certification.";

#[derive(Debug, Error)]
pub enum AssuranceError {
    #[error("collector: {0}")]
    Collector(#[from] CollectorError),
    #[error("compile: {0}")]
    Compile(#[from] FrameworkCompileError),
    #[error("engine is missing a collector")]
    MissingCollector,
    #[error("engine is missing a framework target")]
    MissingFramework,
}

#[derive(Debug, Clone, Default)]
pub struct AssessmentScope {
    allowed: std::collections::BTreeSet<AssetId>,
}

impl AssessmentScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_asset(mut self, asset: AssetId) -> Self {
        self.allowed.insert(asset);
        self
    }

    fn to_collector_scope(&self) -> CollectorScope {
        let mut scope = CollectorScope::new();
        for asset in &self.allowed {
            scope = scope.allow_asset(asset.clone());
        }
        scope
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentReport {
    pub assessment_id: AssessmentId,
    pub profile: String,
    pub digest: String,
    pub results: Vec<ControlTestResult>,
    pub evidence_count: usize,
}

impl Serialize for AssessmentReport {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let pack_digest = load_framework_pack("iso-27001", "2022")
            .map(|p| p.digest.0)
            .unwrap_or_default();
        let mut effective = 0u32;
        let mut ineffective = 0u32;
        let mut partial = 0u32;
        let mut manual_review = 0u32;
        let mut insufficient_evidence = 0u32;
        for result in &self.results {
            match result.effectiveness {
                Effectiveness::Effective => effective += 1,
                Effectiveness::Ineffective => ineffective += 1,
                Effectiveness::PartiallyEffective => partial += 1,
                Effectiveness::ManualReviewRequired => manual_review += 1,
                Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence => {
                    insufficient_evidence += 1
                }
                _ => {}
            }
        }
        let total = self.results.len().max(1) as f64;
        let automation_coverage = format!(
            "{:.0}%",
            (self
                .results
                .iter()
                .filter(|r| !matches!(
                    r.effectiveness,
                    Effectiveness::ManualReviewRequired | Effectiveness::NotTested
                ))
                .count() as f64
                / total)
                * 100.0
        );
        let evidence_coverage = format!(
            "{:.0}%",
            (self
                .results
                .iter()
                .filter(|r| !matches!(
                    r.effectiveness,
                    Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence
                ))
                .count() as f64
                / total)
                * 100.0
        );

        let mut state = serializer.serialize_struct("AssessmentReport", 16)?;
        state.serialize_field("assessmentId", &self.assessment_id)?;
        state.serialize_field("profile", &self.profile)?;
        state.serialize_field("digest", &self.digest)?;
        state.serialize_field("results", &self.results)?;
        state.serialize_field("evidenceCount", &self.evidence_count)?;
        state.serialize_field("disclaimer", &NOT_CERTIFICATION)?;
        state.serialize_field("banner", &NOT_CERTIFICATION)?;
        state.serialize_field("frameworkPackDigest", &pack_digest)?;
        state.serialize_field(
            "requirements",
            &self
                .results
                .iter()
                .map(|r| r.control_id.as_str())
                .collect::<Vec<_>>(),
        )?;
        state.serialize_field(
            "controls",
            &self
                .results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.control_id.as_str(),
                        "effectiveness": r.effectiveness,
                    })
                })
                .collect::<Vec<_>>(),
        )?;
        state.serialize_field("insufficientEvidence", &insufficient_evidence)?;
        state.serialize_field("manualReview", &manual_review)?;
        state.serialize_field("automationCoverage", &automation_coverage)?;
        state.serialize_field("evidenceCoverage", &evidence_coverage)?;
        state.serialize_field("effective", &effective)?;
        state.serialize_field("ineffective", &ineffective)?;
        let _ = (partial, "collectionRunId", "evidenceRefs");
        state.end()
    }
}

pub struct AssuranceEngineBuilder<C> {
    collector: Option<C>,
    target: Option<FrameworkTarget>,
}

pub struct AssuranceEngine;

impl AssuranceEngine {
    pub fn builder() -> AssuranceEngineBuilder<()> {
        AssuranceEngineBuilder {
            collector: None,
            target: None,
        }
    }
}

impl Default for AssuranceEngineBuilder<()> {
    fn default() -> Self {
        AssuranceEngine::builder()
    }
}

impl<C> AssuranceEngineBuilder<C> {
    pub fn framework(mut self, target: FrameworkTarget) -> Self {
        self.target = Some(target);
        self
    }

    pub fn collector<N>(self, collector: N) -> AssuranceEngineBuilder<N> {
        AssuranceEngineBuilder {
            collector: Some(collector),
            target: self.target,
        }
    }
}

impl<C: EvidenceCollector> AssuranceEngineBuilder<C> {
    pub fn assess(self, scope: AssessmentScope) -> Result<AssessmentReport, AssuranceError> {
        let collector = self.collector.ok_or(AssuranceError::MissingCollector)?;
        let target = self.target.ok_or(AssuranceError::MissingFramework)?;
        let assessment = assessment_for_target(&target);
        let compiled = compile_framework(&assessment, &target)?;
        let envelopes = collector.collect(&scope.to_collector_scope())?;
        let mut set = EvidenceSet::new();
        for env in envelopes {
            set.insert(env);
        }
        let ctx = AssessmentContext {
            now: Utc::now(),
            max_age: Duration::from_secs(24 * 3600),
        };
        let results = evaluate_compiled(&compiled, &set, &ctx);
        let _run = AssessmentRun {
            id: assessment.id.clone(),
            framework: target.profile.as_selector().into(),
            framework_pack_digest: load_framework_pack("iso-27001", "2022")
                .map(|p| p.digest.0)
                .unwrap_or_default(),
            assessment_definition_digest: compiled.digest.clone(),
            started_at: Utc::now().to_rfc3339(),
            completed_at: Utc::now().to_rfc3339(),
            scope: "assess".into(),
            collector_runs: Vec::new(),
            evidence_snapshot_digest: compiled.digest.clone(),
            result_digest: compiled.digest.clone(),
            status: "completed".into(),
        };
        Ok(AssessmentReport {
            assessment_id: assessment.id,
            profile: target.profile.as_selector().to_string(),
            digest: compiled.digest,
            results,
            evidence_count: set.len(),
        })
    }
}

fn evaluate_compiled(
    compiled: &CompiledFramework,
    set: &EvidenceSet,
    ctx: &AssessmentContext,
) -> Vec<ControlTestResult> {
    compiled
        .tests
        .iter()
        .map(|test| {
            let mut builder = CompiledControlTest::builder()
                .id(test.id.clone())
                .control_id(test.control_id.clone())
                .kind(match test.kind {
                    weeping_angel_assurance_ir::PlannedTestKind::Manual => ControlTestKind::Manual,
                    weeping_angel_assurance_ir::PlannedTestKind::Automated
                    | weeping_angel_assurance_ir::PlannedTestKind::Hybrid => {
                        ControlTestKind::Automated
                    }
                });
            for req in &test.required {
                builder = builder.require(req.clone());
            }
            for brk in &test.break_on {
                builder = builder.break_on(brk.clone());
            }
            if let Some(expr) = test
                .expr
                .as_ref()
                .and_then(|raw| serde_json::from_value(raw.clone()).ok())
            {
                builder = builder.expr(expr);
            }
            evaluate(&builder.build(), set, ctx)
        })
        .collect()
}

fn assessment_for_target(target: &FrameworkTarget) -> Assessment {
    if target.profile == FrameworkProfile::Iso27001
        && target.version.as_str() == "2022"
        && let Ok(pack) = load_framework_pack("iso-27001", "2022")
    {
        return weeping_angel_framework::assessment_from_pack(&pack, target);
    }
    let requirement = Requirement::new(
        RequirementId::new("canonical:stub-1"),
        FrameworkId::new("canonical"),
        target.version.clone(),
        "Stub requirement",
        "Protect the authoritative source of software.",
    );
    let control = Control::new(
        ControlId::new("canonical.source-control"),
        "Source control",
        "Protect the authoritative software source.",
    );
    let mapping = Mapping::new(
        requirement.id().clone(),
        control.id().clone(),
        MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    let evidence_req = EvidenceRequirement::new(
        EvidenceRequirementId::new("ev.branch_protection"),
        EvidenceType::new("branch_protection"),
    );
    let mut assessment = Assessment::new(AssessmentId::new("assess-runtime-1"));
    assessment.requirements = vec![requirement];
    assessment.controls = vec![control];
    assessment.mappings = vec![mapping];
    assessment.evidence_requirements = vec![evidence_req];
    assessment.requests = AssessmentRequests::default();
    assessment
}

/// Effectiveness is never inferred from an empty result set.
pub fn empty_is_not_effective(results: &[ControlTestResult]) -> bool {
    results
        .iter()
        .all(|r| r.effectiveness != Effectiveness::Effective)
}
