//! Public assurance facade. Callers select a profile + capabilities, not an adapter.

pub mod bridge;

use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    AssessmentId, AssetId, Control, ControlId, EvidenceRequirement, EvidenceRequirementId,
    EvidenceType, FrameworkId, Mapping, MappingCompleteness, MappingDirection, Requirement,
    RequirementId, ASSURANCE_IR_SCHEMA,
};
use weeping_angel_collector::{CollectorError, CollectorScope, EvidenceCollector};
use weeping_angel_control_test::{
    evaluate, AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult,
    Effectiveness, EvidenceSet,
};
use weeping_angel_framework::{
    compile_framework, Assessment, AssessmentRequests, CompiledFramework, FrameworkCompileError,
    FrameworkTarget,
};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentReport {
    pub assessment_id: AssessmentId,
    pub profile: String,
    pub digest: String,
    pub results: Vec<ControlTestResult>,
    pub evidence_count: usize,
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
                    weeping_angel_assurance_ir::PlannedTestKind::Automated => {
                        ControlTestKind::Automated
                    }
                });
            for req in &test.required {
                builder = builder.require(req.clone());
            }
            for brk in &test.break_on {
                builder = builder.break_on(brk.clone());
            }
            evaluate(&builder.build(), set, ctx)
        })
        .collect()
}

fn assessment_for_target(target: &FrameworkTarget) -> Assessment {
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
    Assessment {
        id: AssessmentId::new("assess-runtime-1"),
        schema_version: ASSURANCE_IR_SCHEMA.into(),
        requirements: vec![requirement],
        controls: vec![control],
        mappings: vec![mapping],
        evidence_requirements: vec![evidence_req],
        tests: vec![],
        requests: AssessmentRequests::default(),
    }
}

/// Effectiveness is never inferred from an empty result set.
pub fn empty_is_not_effective(results: &[ControlTestResult]) -> bool {
    results
        .iter()
        .all(|r| r.effectiveness != Effectiveness::Effective)
}
