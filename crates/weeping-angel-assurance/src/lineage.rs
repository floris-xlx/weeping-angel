//! Immutable assessment lineage: snapshots, result identity, explain, replay.

use chrono::{DateTime, Utc};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use weeping_angel_assurance_ir::obligation::{ObligationRegistry, ObligationWhyEdge};
use weeping_angel_assurance_ir::party::InterestedParty;
use weeping_angel_assurance_ir::{
    ApplicabilityRule, AssessmentId, CanonicalizationVersion, Control, ControlId,
    ControlImplementation, ControlTestId, ControlledDocumentId, Exception, Mapping,
    canonical_digest, typed_canonical_digest,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness, PopulationEvaluation};
use weeping_angel_framework::{Assessment, CompiledFramework};

use crate::readiness::FrameworkReadinessSnapshot;
use crate::snapshot::AssessmentRun;
use crate::soa::StatementOfApplicability;
use crate::{AssessmentReport, AssuranceError};

pub const LINEAGE_SNAPSHOT_SCHEMA: &str = "weeping-angel/assessment-lineage/v1";

/// Detected when current pack/catalog files no longer match a pinned digest.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[error("digest mismatch: expected {expected}, got {actual}")]
pub struct DigestMismatch {
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkPackSnapshot {
    pub schema: String,
    pub framework: String,
    pub version: String,
    pub digest: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalCatalogSnapshot {
    pub schema: String,
    pub digest: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentDefinitionSnapshot {
    pub schema: String,
    pub assessment_id: AssessmentId,
    pub digest: String,
    pub definition: Assessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Lineage pin for a static applicability decision (DUP-004: distinct from
/// `applicability::ApplicabilityDecision` engine outcomes).
pub struct LineageApplicabilityDecision {
    pub id: String,
    pub rule: ApplicabilityRule,
    pub static_outcome: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineagePackApplicabilityEntry {
    pub reference: String,
    pub applicable: bool,
    pub applicability_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Pinned applicability material for lineage replay (DUP-004: distinct from
/// `applicability::ApplicabilitySnapshot`).
pub struct LineageApplicabilitySnapshot {
    pub schema: String,
    pub assessment_id: AssessmentId,
    pub scope: String,
    pub requirement_decisions: Vec<LineageApplicabilityDecision>,
    pub control_decisions: Vec<LineageApplicabilityDecision>,
    pub pack_entries: Vec<LineagePackApplicabilityEntry>,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSnapshot {
    pub schema: String,
    pub envelope_digests: Vec<String>,
    pub collection_run_ids: Vec<String>,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlTestRun {
    pub id: String,
    pub test_id: ControlTestId,
    pub test_version: String,
    pub input_digest: String,
    pub control_id: ControlId,
    pub effectiveness: Effectiveness,
    pub evidence_refs: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub population: Option<PopulationEvaluation>,
    pub exception_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementOfApplicabilitySnapshot {
    pub schema: String,
    pub digest: String,
    pub framework_pack_digest: String,
    pub soa: StatementOfApplicability,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetricFamily {
    pub covered: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CoverageMetrics {
    pub control_effectiveness: MetricFamily,
    pub evidence: MetricFamily,
    pub automation: MetricFamily,
    pub subject: MetricFamily,
    pub framework_requirement: MetricFamily,
    pub fresh_evidence: MetricFamily,
    pub manual_review_burden: MetricFamily,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssessmentSummary {
    pub assessment_id: Option<AssessmentId>,
    pub status: String,
    pub effective: u32,
    pub ineffective: u32,
    pub partial: u32,
    pub manual_review: u32,
    pub insufficient_evidence: u32,
    pub not_applicable: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainedTest {
    pub id: ControlTestId,
    pub test_version: String,
    pub input_digest: String,
    pub expr_identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlExplanation {
    pub control: Control,
    pub applicability: LineageApplicabilityDecision,
    pub implementation: Option<ControlImplementation>,
    pub population: Option<PopulationEvaluation>,
    pub tests: Vec<ExplainedTest>,
    pub evidence_requirements: Vec<String>,
    pub evidence: Vec<String>,
    pub missing_evidence: Vec<String>,
    pub failing_subjects: Vec<String>,
    pub missing_subjects: Vec<String>,
    pub exceptions: Vec<Exception>,
    pub mappings: Vec<Mapping>,
    pub effectiveness: Effectiveness,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligations: Vec<ObligationExplainEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineageBundle {
    pub pack: FrameworkPackSnapshot,
    pub catalog: CanonicalCatalogSnapshot,
    pub definition: AssessmentDefinitionSnapshot,
    pub applicability: LineageApplicabilitySnapshot,
    pub evidence: EvidenceSnapshot,
    pub tests: Vec<ControlTestRun>,
    pub run: AssessmentRun,
    pub readiness: FrameworkReadinessSnapshot,
    pub soa: StatementOfApplicabilitySnapshot,
    pub results: Vec<ControlTestResult>,
}

pub fn static_outcome_label(rule: &ApplicabilityRule) -> String {
    match rule.statically_applicable() {
        Some(true) => "applicable".into(),
        Some(false) => "not applicable".into(),
        None => "unresolved".into(),
    }
}

pub fn snapshot_applicability(assessment: &Assessment, scope: &str) -> LineageApplicabilitySnapshot {
    let requirement_decisions = assessment
        .requirements
        .iter()
        .map(|req| LineageApplicabilityDecision {
            id: req.id().to_string(),
            rule: req.applicability().clone(),
            static_outcome: static_outcome_label(req.applicability()),
            rationale:
                "static applicability from ApplicabilityRule; unresolved predicates stay included"
                    .to_string(),
        })
        .collect::<Vec<_>>();
    let control_decisions = assessment
        .controls
        .iter()
        .map(|ctl| LineageApplicabilityDecision {
            id: ctl.id().to_string(),
            rule: ctl.applicability().clone(),
            static_outcome: static_outcome_label(ctl.applicability()),
            rationale: "static applicability from ApplicabilityRule".into(),
        })
        .collect::<Vec<_>>();
    let mut snapshot = LineageApplicabilitySnapshot {
        schema: LINEAGE_SNAPSHOT_SCHEMA.into(),
        assessment_id: assessment.id.clone(),
        scope: scope.into(),
        requirement_decisions,
        control_decisions,
        pack_entries: Vec::new(),
        digest: String::new(),
    };
    snapshot.digest = snapshot_digest("applicability-snapshot", &snapshot);
    snapshot
}

pub fn seal_evidence_snapshot(
    envelope_digests: impl IntoIterator<Item = String>,
    collection_run_ids: impl IntoIterator<Item = String>,
) -> EvidenceSnapshot {
    let mut envelope_digests: Vec<String> = envelope_digests.into_iter().collect();
    envelope_digests.sort();
    envelope_digests.dedup();
    let mut collection_run_ids: Vec<String> = collection_run_ids.into_iter().collect();
    collection_run_ids.sort();
    collection_run_ids.dedup();
    let mut snapshot = EvidenceSnapshot {
        schema: LINEAGE_SNAPSHOT_SCHEMA.into(),
        envelope_digests,
        collection_run_ids,
        digest: String::new(),
    };
    snapshot.digest = snapshot_digest("evidence-snapshot", &snapshot);
    snapshot
}

pub fn definition_snapshot(assessment: &Assessment) -> AssessmentDefinitionSnapshot {
    let mut snap = AssessmentDefinitionSnapshot {
        schema: LINEAGE_SNAPSHOT_SCHEMA.into(),
        assessment_id: assessment.id.clone(),
        digest: String::new(),
        definition: assessment.clone(),
    };
    snap.digest = snapshot_digest("assessment-definition", &snap.definition);
    snap
}

pub fn pack_snapshot(
    framework: &str,
    version: &str,
    digest: &str,
    payload: serde_json::Value,
) -> FrameworkPackSnapshot {
    FrameworkPackSnapshot {
        schema: LINEAGE_SNAPSHOT_SCHEMA.into(),
        framework: framework.into(),
        version: version.into(),
        digest: digest.into(),
        payload,
    }
}

pub fn catalog_snapshot(digest: &str, payload: serde_json::Value) -> CanonicalCatalogSnapshot {
    CanonicalCatalogSnapshot {
        schema: LINEAGE_SNAPSHOT_SCHEMA.into(),
        digest: digest.into(),
        payload,
    }
}

pub fn control_test_run_from_result(
    result: &ControlTestResult,
    exception_ids: Vec<String>,
) -> ControlTestRun {
    ControlTestRun {
        id: format!("{}:{}", result.test_id, result.input_digest),
        test_id: result.test_id.clone(),
        test_version: result.test_version.clone(),
        input_digest: result.input_digest.clone(),
        control_id: result.control_id.clone(),
        effectiveness: result.effectiveness,
        evidence_refs: result.evidence_refs.clone(),
        missing_evidence: result.missing_evidence.clone(),
        population: result.population.clone(),
        exception_ids,
    }
}

fn snapshot_digest<T: Serialize>(type_name: &str, value: &T) -> String {
    typed_canonical_digest(type_name, value).unwrap_or_default()
}

/// Result identity: SHA-256 of canonical JSON. Excludes wall-clock `duration`
/// and `evaluatedAt` (`checked_at`) so two processes with the same semantic
/// results produce the same digest.
pub fn assessment_result_digest(results: &[ControlTestResult]) -> String {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Identity<'a> {
        schema: &'static str,
        results: Vec<ResultPin<'a>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ResultPin<'a> {
        test_id: &'a ControlTestId,
        control_id: &'a ControlId,
        effectiveness: Effectiveness,
        evidence_refs: &'a [String],
        missing_evidence: &'a [String],
        test_version: &'a str,
        input_digest: &'a str,
        population: &'a Option<PopulationEvaluation>,
    }
    let body = Identity {
        schema: LINEAGE_SNAPSHOT_SCHEMA,
        results: results
            .iter()
            .map(|r| {
                let _duration = &r.duration;
                let _evaluated_at = &r.checked_at;
                ResultPin {
                    test_id: &r.test_id,
                    control_id: &r.control_id,
                    effectiveness: r.effectiveness,
                    evidence_refs: &r.evidence_refs,
                    missing_evidence: &r.missing_evidence,
                    test_version: &r.test_version,
                    input_digest: &r.input_digest,
                    population: &r.population,
                }
            })
            .collect(),
    };
    typed_canonical_digest("assessment-result", &body).unwrap_or_default()
}

pub fn coverage_metrics(
    results: &[ControlTestResult],
    compiled: Option<&CompiledFramework>,
) -> CoverageMetrics {
    let total = results.len() as u64;
    let effective = results
        .iter()
        .filter(|r| r.effectiveness == Effectiveness::Effective)
        .count() as u64;
    let evidenced = results
        .iter()
        .filter(|r| {
            !matches!(
                r.effectiveness,
                Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence
            )
        })
        .count() as u64;
    let automated = results
        .iter()
        .filter(|r| {
            !matches!(
                r.effectiveness,
                Effectiveness::ManualReviewRequired | Effectiveness::NotTested
            )
        })
        .count() as u64;
    let with_subjects = results
        .iter()
        .filter(|r| {
            r.population
                .as_ref()
                .is_some_and(|p| p.evaluated > 0 || p.population > 0)
        })
        .count() as u64;
    let req_total = compiled
        .map(|c| c.applicable_requirements.len() as u64)
        .unwrap_or(total);
    let fresh = results
        .iter()
        .filter(|r| r.effectiveness != Effectiveness::StaleEvidence)
        .count() as u64;
    let manual = results
        .iter()
        .filter(|r| r.effectiveness == Effectiveness::ManualReviewRequired)
        .count() as u64;
    CoverageMetrics {
        control_effectiveness: MetricFamily {
            covered: effective,
            total,
        },
        evidence: MetricFamily {
            covered: evidenced,
            total,
        },
        automation: MetricFamily {
            covered: automated,
            total,
        },
        subject: MetricFamily {
            covered: with_subjects,
            total,
        },
        framework_requirement: MetricFamily {
            covered: req_total,
            total: req_total,
        },
        fresh_evidence: MetricFamily {
            covered: fresh,
            total,
        },
        manual_review_burden: MetricFamily {
            covered: manual,
            total,
        },
    }
}

pub fn assessment_summary(report: &AssessmentReport) -> AssessmentSummary {
    let mut summary = AssessmentSummary {
        assessment_id: Some(report.assessment_id.clone()),
        status: report
            .run
            .as_ref()
            .map(|r| r.status.clone())
            .unwrap_or_else(|| "completed".into()),
        ..AssessmentSummary::default()
    };
    for result in &report.results {
        match result.effectiveness {
            Effectiveness::Effective => summary.effective += 1,
            Effectiveness::Ineffective => summary.ineffective += 1,
            Effectiveness::PartiallyEffective => summary.partial += 1,
            Effectiveness::ManualReviewRequired => summary.manual_review += 1,
            Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence => {
                summary.insufficient_evidence += 1
            }
            Effectiveness::NotApplicable => summary.not_applicable += 1,
            _ => {}
        }
    }
    summary
}

pub fn explain_control(
    report: &AssessmentReport,
    control_id: &str,
    assessment: Option<&Assessment>,
    applicability: Option<&LineageApplicabilitySnapshot>,
) -> Result<ControlExplanation, AssuranceError> {
    let result = report
        .results
        .iter()
        .find(|r| r.control_id.as_str() == control_id)
        .ok_or_else(|| AssuranceError::UnknownControl {
            assessment: report.assessment_id.to_string(),
            control: control_id.into(),
        })?;
    let control = assessment
        .and_then(|a| a.controls.iter().find(|c| c.id().as_str() == control_id))
        .cloned()
        .unwrap_or_else(|| {
            Control::new(
                result.control_id.clone(),
                control_id,
                "control from pinned assessment run",
            )
        });
    let applicability = applicability
        .and_then(|snap| {
            snap.control_decisions
                .iter()
                .find(|d| d.id == control_id)
                .cloned()
        })
        .unwrap_or_else(|| LineageApplicabilityDecision {
            id: control_id.into(),
            rule: ApplicabilityRule::Always,
            static_outcome: "applicable".into(),
            rationale: "evaluated because the control was in the pinned assessment".into(),
        });
    let implementation = assessment.and_then(|a| {
        a.implementations
            .iter()
            .find(|imp| imp.control_id().as_str() == control_id)
            .cloned()
    });
    let mappings = assessment
        .map(|a| {
            a.mappings
                .iter()
                .filter(|m| m.to_control().as_str() == control_id)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let evidence_requirements = assessment
        .map(|a| {
            a.evidence_requirements
                .iter()
                .map(|e| e.id().to_string())
                .collect()
        })
        .unwrap_or_default();
    let exceptions = assessment
        .map(|a| {
            a.exceptions
                .iter()
                .filter(|e| {
                    e.control_id
                        .as_ref()
                        .is_some_and(|id| id.as_str() == control_id)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let population = result.population.clone();
    let failing_subjects = population
        .as_ref()
        .map(|p| p.failing_subjects.clone())
        .unwrap_or_default();
    let missing_subjects = population
        .as_ref()
        .map(|p| p.missing_subjects.clone())
        .unwrap_or_default();
    Ok(ControlExplanation {
        control,
        applicability,
        implementation,
        population,
        tests: vec![ExplainedTest {
            id: result.test_id.clone(),
            test_version: result.test_version.clone(),
            input_digest: result.input_digest.clone(),
            expr_identity: None,
        }],
        evidence_requirements,
        evidence: result.evidence_refs.clone(),
        missing_evidence: result.missing_evidence.clone(),
        failing_subjects,
        missing_subjects,
        exceptions,
        mappings,
        effectiveness: result.effectiveness,
        obligations: Vec::new(),
    })
}

/// Target of an organizational-duty explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObligationExplainTarget {
    Control(ControlId),
    Document(ControlledDocumentId),
}

/// One party → source → obligation → mapping edge in an explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationExplainEdge {
    pub party: InterestedParty,
    pub source: weeping_angel_assurance_ir::obligation::RequirementSource,
    pub obligation: weeping_angel_assurance_ir::obligation::Obligation,
    pub mapping: weeping_angel_assurance_ir::obligation::ObligationMapping,
    pub applicability: weeping_angel_assurance_ir::obligation::ObligationApplicability,
    pub projects_as_equivalence: bool,
    pub projects_as_full_satisfaction: bool,
}

impl From<ObligationWhyEdge> for ObligationExplainEdge {
    fn from(edge: ObligationWhyEdge) -> Self {
        Self {
            party: edge.party,
            source: edge.source,
            obligation: edge.obligation,
            mapping: edge.mapping,
            applicability: edge.applicability,
            projects_as_equivalence: edge.projects_as_equivalence,
            projects_as_full_satisfaction: edge.projects_as_full_satisfaction,
        }
    }
}

/// Deterministic answer to “why does this control/policy exist?”
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationExplain {
    pub target: ObligationExplainTarget,
    pub at: DateTime<Utc>,
    pub current: Vec<ObligationExplainEdge>,
    pub historical: Vec<ObligationExplainEdge>,
}

impl ObligationExplain {
    pub fn canonical_digest(
        &self,
    ) -> Result<String, weeping_angel_assurance_ir::CanonicalDigestError> {
        debug_assert_eq!(CanonicalizationVersion::CURRENT.as_str(), "canon/v1");
        canonical_digest(self)
    }
}

/// Explain why a canonical control exists using the obligation registry (canon/v1).
pub fn explain_why_control_exists(
    control_id: &ControlId,
    registry: &ObligationRegistry,
    t: DateTime<Utc>,
) -> ObligationExplain {
    let current = registry.why_control_exists(control_id, t);
    let current_ids: std::collections::BTreeSet<_> = current
        .iter()
        .map(|edge| edge.mapping.id.as_str().to_string())
        .collect();
    let historical = registry
        .why_control_exists_including_historical(control_id, t)
        .into_iter()
        .filter(|edge| !current_ids.contains(edge.mapping.id.as_str()))
        .collect::<Vec<_>>();
    ObligationExplain {
        target: ObligationExplainTarget::Control(control_id.clone()),
        at: t,
        current: current
            .into_iter()
            .map(ObligationExplainEdge::from)
            .collect(),
        historical: historical
            .into_iter()
            .map(ObligationExplainEdge::from)
            .collect(),
    }
}

/// Explain why a governed document/policy exists using the obligation registry.
pub fn explain_why_document_exists(
    document_id: &ControlledDocumentId,
    registry: &ObligationRegistry,
    t: DateTime<Utc>,
) -> ObligationExplain {
    let current = registry.why_document_exists(document_id, t);
    let current_ids: std::collections::BTreeSet<_> = current
        .iter()
        .map(|edge| edge.mapping.id.as_str().to_string())
        .collect();
    let historical = registry
        .why_document_exists_including_historical(document_id, t)
        .into_iter()
        .filter(|edge| !current_ids.contains(edge.mapping.id.as_str()))
        .collect::<Vec<_>>();
    ObligationExplain {
        target: ObligationExplainTarget::Document(document_id.clone()),
        at: t,
        current: current
            .into_iter()
            .map(ObligationExplainEdge::from)
            .collect(),
        historical: historical
            .into_iter()
            .map(ObligationExplainEdge::from)
            .collect(),
    }
}

/// Reconstruct a report from pinned snapshots. Does not consult current files.
pub fn reconstruct(bundle: &LineageBundle) -> AssessmentReport {
    let mut report = AssessmentReport {
        assessment_id: bundle.run.id.clone(),
        profile: bundle.run.framework.clone(),
        digest: bundle.run.result_digest.clone(),
        results: bundle.results.clone(),
        evidence_count: bundle.evidence.envelope_digests.len(),
        run: Some(bundle.run.clone()),
        summary: None,
        coverage_metrics: None,
        framework_pack_digest: bundle.pack.digest.clone(),
        canonical_catalog_digest: bundle.catalog.digest.clone(),
    };
    report.summary = Some(assessment_summary(&report));
    report.coverage_metrics = Some(coverage_metrics(&report.results, None));
    report
}

/// Replay from pins. Verifies lineage then reconstructs. Never loads current files.
pub fn replay_assessment(bundle: &LineageBundle) -> Result<AssessmentReport, AssuranceError> {
    verify_replay_bundle(bundle)?;
    let report = reconstruct(bundle);
    Ok(report)
}

fn pin_missing(pin: &str) -> bool {
    let trimmed = pin.trim();
    trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unpinned")
}

fn snapshot_schema_ok(schema: &str) -> bool {
    schema == LINEAGE_SNAPSHOT_SCHEMA
}

fn verify_replay_bundle(bundle: &LineageBundle) -> Result<(), AssuranceError> {
    if pin_missing(&bundle.run.framework_pack_digest)
        || pin_missing(&bundle.run.canonical_catalog_pin)
        || pin_missing(&bundle.run.assessment_definition_digest)
        || pin_missing(&bundle.run.evidence_snapshot_digest)
        || pin_missing(&bundle.run.applicability_snapshot_id)
        || pin_missing(&bundle.run.result_digest)
        || bundle.run.as_of.trim().is_empty()
    {
        return Err(crate::ReplayFailure::MissingPinnedMaterial(
            "required run pin, snapshot identity, or asOf is absent".into(),
        )
        .into());
    }
    for (label, schema) in [
        ("pack", bundle.pack.schema.as_str()),
        ("catalog", bundle.catalog.schema.as_str()),
        ("definition", bundle.definition.schema.as_str()),
        ("applicability", bundle.applicability.schema.as_str()),
        ("evidence", bundle.evidence.schema.as_str()),
        ("soa", bundle.soa.schema.as_str()),
    ] {
        if !snapshot_schema_ok(schema) {
            return Err(crate::ReplayFailure::IncompatibleSchema(format!(
                "{label} schema {schema}"
            ))
            .into());
        }
    }
    detect_digest_mismatch(&bundle.run.framework_pack_digest, &bundle.pack.digest)?;
    detect_digest_mismatch(&bundle.run.canonical_catalog_pin, &bundle.catalog.digest)?;
    detect_digest_mismatch(
        &bundle.run.assessment_definition_digest,
        &bundle.definition.digest,
    )?;
    detect_digest_mismatch(
        &bundle.run.evidence_snapshot_digest,
        &bundle.evidence.digest,
    )?;
    detect_digest_mismatch(
        &bundle.run.applicability_snapshot_id,
        &bundle.applicability.digest,
    )?;
    let result_identity = assessment_result_digest(&bundle.results);
    detect_digest_mismatch(&bundle.run.result_digest, &result_identity)?;
    if !bundle.evidence.envelope_digests.is_empty() {
        let sealed = seal_evidence_snapshot(
            bundle.evidence.envelope_digests.iter().cloned(),
            bundle.evidence.collection_run_ids.iter().cloned(),
        );
        if sealed.digest != bundle.evidence.digest {
            return Err(crate::ReplayFailure::InconsistentLineage(
                "envelope digest list does not match evidence snapshot identity".into(),
            )
            .into());
        }
    }
    if !bundle.soa.framework_pack_digest.is_empty()
        && bundle.soa.framework_pack_digest != bundle.run.framework_pack_digest
    {
        return Err(crate::ReplayFailure::InconsistentLineage(
            "SoA pack pin does not match run framework pack pin".into(),
        )
        .into());
    }
    if bundle.definition.digest.is_empty() || bundle.applicability.digest.is_empty() {
        return Err(crate::ReplayFailure::IncompleteLineage(
            "definition or applicability snapshot missing identity".into(),
        )
        .into());
    }
    Ok(())
}

/// Deprecated alias of [`reconstruct`] without pin verification.
/// Prefer [`replay_assessment`] for historical rebuild (DUP-005).
#[deprecated(note = "use replay_assessment for verified rebuild, or reconstruct only when pins are already verified")]
pub fn load_lineage(bundle: &LineageBundle) -> AssessmentReport {
    reconstruct(bundle)
}

/// Compare pinned digests to current files. Mismatch is detected, never rewritten.
pub fn detect_digest_mismatch(pinned: &str, current: &str) -> Result<(), DigestMismatch> {
    if pinned != current {
        Err(DigestMismatch {
            expected: pinned.into(),
            actual: current.into(),
        })
    } else {
        Ok(())
    }
}

const NOT_CERTIFICATION: &str = "This is a readiness assessment and is not certification.";

fn carried_pack_pin(report: &AssessmentReport) -> String {
    if !report.framework_pack_digest.is_empty() {
        return report.framework_pack_digest.clone();
    }
    if let Some(run) = &report.run
        && !run.framework_pack_digest.is_empty()
    {
        return run.framework_pack_digest.clone();
    }
    if !report.digest.is_empty() {
        report.digest.clone()
    } else {
        "unpinned".into()
    }
}

fn carried_catalog_pin(report: &AssessmentReport) -> String {
    if !report.canonical_catalog_digest.is_empty() {
        return report.canonical_catalog_digest.clone();
    }
    if let Some(run) = &report.run
        && !run.canonical_catalog_pin.is_empty()
    {
        return run.canonical_catalog_pin.clone();
    }
    String::new()
}

/// Pure report serialization: only values already on the report / run.
/// No pack load, catalog load, network, or filesystem lookup.
pub fn serialize_assessment_report<S: Serializer>(
    report: &AssessmentReport,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let pack_pin = carried_pack_pin(report);
    let catalog_pin = carried_catalog_pin(report);
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
    let evidence_refs: Vec<String> = report
        .results
        .iter()
        .flat_map(|r| r.evidence_refs.iter().cloned())
        .collect();
    let readiness = FrameworkReadinessSnapshot {
        assessment_id: report.assessment_id.clone(),
        framework: report.profile.clone(),
        framework_version: String::new(),
        framework_pack_digest: pack_pin.clone(),
        catalog_digest: String::new(),
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
        automation_coverage: metrics.automation.covered.to_string(),
        evidence_coverage: metrics.evidence.covered.to_string(),
    };

    let mut state = serializer.serialize_struct("AssessmentReport", 30)?;
    state.serialize_field("assessmentId", &report.assessment_id)?;
    state.serialize_field("profile", &report.profile)?;
    state.serialize_field("digest", &report.digest)?;
    state.serialize_field("results", &report.results)?;
    state.serialize_field("evidenceCount", &report.evidence_count)?;
    state.serialize_field("disclaimer", &NOT_CERTIFICATION)?;
    state.serialize_field("banner", &NOT_CERTIFICATION)?;
    state.serialize_field("frameworkPackDigest", &pack_pin)?;
    state.serialize_field("canonicalCatalogDigest", &catalog_pin)?;
    state.serialize_field("catalogDigest", &catalog_pin)?;
    state.serialize_field("resultDigest", &report.digest)?;
    state.serialize_field("summary", &summary)?;
    state.serialize_field("coverageMetrics", &metrics)?;
    state.serialize_field("assessmentRun", &report.run)?;
    state.serialize_field("status", &status)?;
    state.serialize_field("collectionRuns", &collection_runs)?;
    state.serialize_field("collectionRunId", &collection_runs.first())?;
    state.serialize_field("evidenceRefs", &evidence_refs)?;
    state.serialize_field("readiness", &readiness)?;
    state.serialize_field("requirements", &requirement_ids)?;
    state.serialize_field("controls", &controls)?;
    state.serialize_field("insufficientEvidence", &summary.insufficient_evidence)?;
    state.serialize_field("manualReview", &summary.manual_review)?;
    state.serialize_field("effective", &summary.effective)?;
    state.serialize_field("ineffective", &summary.ineffective)?;
    state.serialize_field("automationCoverage", &metrics.automation)?;
    state.serialize_field("evidenceCoverage", &metrics.evidence)?;
    state.serialize_field("subjectCoverage", &metrics.subject)?;
    state.serialize_field("controlCoverage", &metrics.control_effectiveness)?;
    state.serialize_field(
        "frameworkRequirementCoverage",
        &metrics.framework_requirement,
    )?;
    state.end()
}

pub fn verify_current_against_pins(
    bundle: &LineageBundle,
    current_pack_digest: Option<&str>,
    current_catalog_digest: Option<&str>,
) -> Result<(), DigestMismatch> {
    if let Some(current) = current_pack_digest {
        detect_digest_mismatch(&bundle.pack.digest, current)?;
    }
    if let Some(current) = current_catalog_digest {
        detect_digest_mismatch(&bundle.catalog.digest, current)?;
    }
    Ok(())
}
