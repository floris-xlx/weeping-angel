//! Public assurance facade. Callers select a profile + capabilities, not an adapter.

pub mod bridge;
pub mod lineage;
pub mod readiness;
pub mod snapshot;
pub mod soa;

use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_collector::{CollectorError, CollectorScope, EvidenceCollector};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSet, evaluate,
};
use weeping_angel_evidence::CollectionRun;
use weeping_angel_framework::{
    Assessment, CompiledFramework, FrameworkCompileError, FrameworkTarget, compile_framework,
    load_framework_pack,
};
use weeping_angel_framework::pack::{PackError, resolve_pack_dir};

pub use lineage::{
    ApplicabilitySnapshot, AssessmentDefinitionSnapshot, AssessmentSummary, CanonicalCatalogSnapshot,
    ControlExplanation, ControlTestRun, CoverageMetrics, DigestMismatch, EvidenceSnapshot,
    FrameworkPackSnapshot, LineageBundle, StatementOfApplicabilitySnapshot, assessment_result_digest,
    explain_control, load_lineage, reconstruct, replay_assessment,
};
pub use readiness::FrameworkReadinessSnapshot;
pub use snapshot::{AssessmentRun, SnapshotDiff, compare, compare_lineage, compare_runs};
pub use soa::{StatementOfApplicability, project_soa};

use crate::lineage::{
    assessment_summary, catalog_snapshot, coverage_metrics, definition_snapshot, pack_snapshot,
    seal_evidence_snapshot, snapshot_applicability,
};
use weeping_angel_assurance_ir::AssessmentId;
use weeping_angel_assurance_ir::AssetId;

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
    #[error("unknown pack: {0}")]
    UnknownPack(String),
    #[error(transparent)]
    DigestMismatch(#[from] DigestMismatch),
    #[error("unknown control {control} in assessment {assessment}")]
    UnknownControl { assessment: String, control: String },
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

    pub fn describe(&self) -> String {
        if self.allowed.is_empty() {
            "assess".into()
        } else {
            self.allowed
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(",")
        }
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
    #[serde(default)]
    pub run: Option<AssessmentRun>,
    #[serde(default)]
    pub summary: Option<AssessmentSummary>,
    #[serde(default)]
    pub coverage_metrics: Option<CoverageMetrics>,
    #[serde(default)]
    pub framework_pack_digest: String,
    #[serde(default)]
    pub canonical_catalog_digest: String,
}

impl Default for AssessmentReport {
    fn default() -> Self {
        Self {
            assessment_id: AssessmentId::new("assess-unset"),
            profile: String::new(),
            digest: String::new(),
            results: Vec::new(),
            evidence_count: 0,
            run: None,
            summary: None,
            coverage_metrics: None,
            framework_pack_digest: String::new(),
            canonical_catalog_digest: String::new(),
        }
    }
}

impl Serialize for AssessmentReport {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let projections = report_projections(self);
        let mut state = serializer.serialize_struct("AssessmentReport", 18)?;
        state.serialize_field("assessmentId", &self.assessment_id)?;
        state.serialize_field("profile", &self.profile)?;
        state.serialize_field("digest", &self.digest)?;
        state.serialize_field("results", &self.results)?;
        state.serialize_field("evidenceCount", &self.evidence_count)?;
        state.serialize_field("disclaimer", &NOT_CERTIFICATION)?;
        state.serialize_field("banner", &NOT_CERTIFICATION)?;
        state.serialize_field("frameworkPackDigest", &projections.pack_pin)?;
        state.serialize_field("canonicalCatalogDigest", &projections.catalog_pin)?;
        state.serialize_field("resultDigest", &self.digest)?;
        state.serialize_field("summary", &projections.summary)?;
        state.serialize_field("coverageMetrics", &projections.metrics)?;
        state.serialize_field("assessmentRun", &self.run)?;
        state.serialize_field("status", &projections.status)?;
        state.serialize_field("collectionRuns", &projections.collection_runs)?;
        state.serialize_field("readiness", &projections.readiness)?;
        state.serialize_field("requirements", &projections.requirement_ids)?;
        state.serialize_field("controls", &projections.controls)?;
        state.serialize_field("insufficientEvidence", &projections.summary.insufficient_evidence)?;
        state.serialize_field("manualReview", &projections.summary.manual_review)?;
        state.serialize_field("effective", &projections.summary.effective)?;
        state.serialize_field("ineffective", &projections.summary.ineffective)?;
        state.end()
    }
}

struct ReportProjections {
    pack_pin: String,
    catalog_pin: String,
    summary: AssessmentSummary,
    metrics: CoverageMetrics,
    status: String,
    collection_runs: Vec<String>,
    readiness: FrameworkReadinessSnapshot,
    requirement_ids: Vec<String>,
    controls: Vec<serde_json::Value>,
}

fn report_projections(report: &AssessmentReport) -> ReportProjections {
    let pack_pin = if !report.framework_pack_digest.is_empty() {
        report.framework_pack_digest.clone()
    } else if let Some(run) = &report.run {
        run.framework_pack_digest.clone()
    } else if !report.digest.is_empty() {
        report.digest.clone()
    } else {
        "unpinned".into()
    };
    let catalog_pin = if !report.canonical_catalog_digest.is_empty() {
        report.canonical_catalog_digest.clone()
    } else if let Some(run) = &report.run {
        run.canonical_catalog_pin.clone()
    } else if !report.digest.is_empty() {
        report.digest.clone()
    } else {
        "unpinned".into()
    };
    let summary = report
        .summary
        .clone()
        .unwrap_or_else(|| assessment_summary(report));
    let metrics = report
        .coverage_metrics
        .clone()
        .unwrap_or_else(|| coverage_metrics(&report.results, None));
    let status = report
        .run
        .as_ref()
        .map(|r| r.status.clone())
        .unwrap_or_else(|| summary.status.clone());
    let collection_runs = report
        .run
        .as_ref()
        .map(|r| r.collector_runs.clone())
        .unwrap_or_default();
    let controls: Vec<serde_json::Value> = report
        .results
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.control_id.as_str(),
                "effectiveness": r.effectiveness,
            })
        })
        .collect();
    let requirement_ids: Vec<String> = report
        .results
        .iter()
        .map(|r| r.control_id.to_string())
        .collect();
    let readiness = FrameworkReadinessSnapshot {
        assessment_id: report.assessment_id.clone(),
        framework: report.profile.clone(),
        framework_version: String::new(),
        framework_pack_digest: pack_pin.clone(),
        assessment_digest: report.digest.clone(),
        evaluated_at: report
            .run
            .as_ref()
            .map(|r| r.completed_at.clone())
            .unwrap_or_default(),
        requirements: Vec::new(),
        controls: report
            .results
            .iter()
            .map(|r| crate::readiness::ControlReadiness {
                id: r.control_id.clone(),
                effectiveness: r.effectiveness,
            })
            .collect(),
        effective: summary.effective,
        ineffective: summary.ineffective,
        partial: summary.partial,
        manual_review: summary.manual_review,
        insufficient_evidence: summary.insufficient_evidence,
        not_applicable: summary.not_applicable,
        automation_coverage: format!("{}", metrics.automation.covered),
        evidence_coverage: format!("{}", metrics.evidence.covered),
    };
    ReportProjections {
        pack_pin,
        catalog_pin,
        summary,
        metrics,
        status,
        collection_runs,
        readiness,
        requirement_ids,
        controls,
    }
}

pub struct AssuranceEngineBuilder<C> {
    collector: Option<C>,
    target: Option<FrameworkTarget>,
    definition: Option<Assessment>,
}

pub struct AssuranceEngine;

impl AssuranceEngine {
    pub fn builder() -> AssuranceEngineBuilder<()> {
        AssuranceEngineBuilder {
            collector: None,
            target: None,
            definition: None,
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

    pub fn definition(mut self, assessment: Assessment) -> Self {
        self.definition = Some(assessment);
        self
    }

    pub fn collector<N>(self, collector: N) -> AssuranceEngineBuilder<N> {
        AssuranceEngineBuilder {
            collector: Some(collector),
            target: self.target,
            definition: self.definition,
        }
    }
}

impl<C: EvidenceCollector> AssuranceEngineBuilder<C> {
    pub fn assess(self, scope: AssessmentScope) -> Result<AssessmentReport, AssuranceError> {
        let collector = self.collector.ok_or(AssuranceError::MissingCollector)?;
        let target = self.target.ok_or(AssuranceError::MissingFramework)?;
        let started_at = Utc::now().to_rfc3339();
        let assessment = match self.definition {
            Some(def) => def,
            None => assessment_for_target(&target)?,
        };
        let compiled = compile_framework(&assessment, &target)?;
        let descriptor = collector.descriptor();
        let mut collection_run = CollectionRun::new(&descriptor.id, &descriptor.version);
        collection_run.scope = scope.describe();
        let envelopes = match collector.collect(&scope.to_collector_scope()) {
            Ok(envs) => {
                collection_run.evidence_count = envs.len() as u32;
                collection_run.status = "completed".into();
                collection_run.completed_at = Some(Utc::now());
                envs
            }
            Err(_err) => {
                collection_run.error_count = 1;
                collection_run.status = if collection_run.evidence_count > 0 {
                    "partial".into()
                } else {
                    "failed".into()
                };
                collection_run.completed_at = Some(Utc::now());
                Vec::new()
            }
        };
        let mut set = EvidenceSet::new();
        for env in &envelopes {
            set.insert(env.clone());
        }
        for exception in &assessment.exceptions {
            set.insert_exception(exception.clone());
        }
        let ctx = AssessmentContext {
            now: Utc::now(),
            max_age: Duration::from_secs(24 * 3600),
        };
        let results = evaluate_compiled(&compiled, &set, &ctx);
        let status = if collection_run.status == "failed" {
            "failed"
        } else if collection_run.status == "partial" || collection_run.error_count > 0 {
            "partial"
        } else {
            "completed"
        };
        let loaded_pack = load_framework_pack(target.profile.as_selector(), target.version.as_str());
        let pack_digest = loaded_pack
            .as_ref()
            .map(|p| p.digest.0.clone())
            .unwrap_or_else(|_| "unpinned".into());
        let catalog_digest = load_catalog_pin();
        let definition_digest = definition_snapshot(&assessment).digest;
        let evidence_snapshot = seal_evidence_snapshot(
            envelopes.iter().map(|e| e.digest().to_string()),
            [collection_run.run_id.clone()],
        );
        let result_digest = assessment_result_digest(&results);
        let mut applicability = snapshot_applicability(&assessment, &scope.describe());
        if let Ok(entries) = load_pack_applicability_rows(
            target.profile.as_selector(),
            target.version.as_str(),
        ) {
            applicability.pack_entries = entries;
        }
        let collector_runs = vec![collection_run.run_id.clone()];
        let run = AssessmentRun {
            id: assessment.id.clone(),
            framework: target.profile.as_selector().into(),
            framework_pack_digest: pack_digest.clone(),
            assessment_definition_digest: definition_digest,
            started_at,
            completed_at: Utc::now().to_rfc3339(),
            scope: scope.describe(),
            collector_runs,
            evidence_snapshot_digest: evidence_snapshot.digest.clone(),
            result_digest: result_digest.clone(),
            status: status.into(),
            canonical_catalog_pin: catalog_digest.clone(),
            applicability_snapshot_id: applicability.digest.clone(),
        };
        let mut report = AssessmentReport {
            assessment_id: assessment.id.clone(),
            profile: target.profile.as_selector().to_string(),
            digest: result_digest,
            results,
            evidence_count: set.len(),
            run: Some(run),
            summary: None,
            coverage_metrics: None,
            framework_pack_digest: pack_digest,
            canonical_catalog_digest: catalog_digest,
        };
        report.summary = Some(assessment_summary(&report));
        report.coverage_metrics = Some(coverage_metrics(&report.results, Some(&compiled)));
        let _ = (
            pack_snapshot(
                target.profile.as_selector(),
                target.version.as_str(),
                &report.framework_pack_digest,
                serde_json::json!({ "id": target.profile.as_selector() }),
            ),
            catalog_snapshot(
                &report.canonical_catalog_digest,
                serde_json::json!({ "schema": "weeping-angel/canonical-catalog/v1" }),
            ),
        );
        Ok(report)
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

fn assessment_for_target(target: &FrameworkTarget) -> Result<Assessment, AssuranceError> {
    let pack = load_framework_pack(target.profile.as_selector(), target.version.as_str()).map_err(
        |err| match err {
            PackError::UnknownPack(message) => AssuranceError::UnknownPack(message),
            other => AssuranceError::Compile(other.into()),
        },
    )?;
    Ok(weeping_angel_framework::assessment_from_pack(&pack, target))
}

fn load_catalog_pin() -> String {
    for root in catalog_search_roots() {
        if let Ok(catalog) = CanonicalCatalog::load(&root)
            && let Ok(digest) = catalog.digest()
        {
            return digest.to_string();
        }
    }
    "catalog-unavailable".into()
}

fn catalog_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = PathBuf::from(dir);
        roots.push(base.join("catalog/canonical/v1"));
        roots.push(base.join("../..").join("catalog/canonical/v1"));
        roots.push(base.join("..").join("catalog/canonical/v1"));
    }
    roots.push(PathBuf::from("catalog/canonical/v1"));
    roots
}

fn load_pack_applicability_rows(
    framework: &str,
    version: &str,
) -> Result<Vec<crate::lineage::PackApplicabilityEntry>, PackError> {
    let dir = resolve_pack_dir(framework, version)?;
    let path = dir.join("applicability.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| PackError::Io(e.to_string()))?;
    let parsed: toml::Value = toml::from_str(&text).map_err(|e| PackError::Parse(e.to_string()))?;
    let mut entries = Vec::new();
    if let Some(arr) = parsed.get("entry").and_then(|v| v.as_array()) {
        for item in arr {
            entries.push(crate::lineage::PackApplicabilityEntry {
                reference: item
                    .get("reference")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                applicable: item
                    .get("applicable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                applicability_rationale: item
                    .get("applicability_rationale")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
            });
        }
    }
    Ok(entries)
}

/// Effectiveness is never inferred from an empty result set.
pub fn empty_is_not_effective(results: &[ControlTestResult]) -> bool {
    results
        .iter()
        .all(|r| r.effectiveness != Effectiveness::Effective)
}
