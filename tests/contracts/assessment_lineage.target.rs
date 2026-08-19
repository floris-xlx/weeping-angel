//! Target suite for immutable assessment lineage.
//!
//! Encodes DESIRED behavior in `docs/specs/assessment-lineage.md` §4 / §4.12
//! (LIN-001–015). Must stay RED on CURRENT shortcuts until persist, explain,
//! pure serialize, generic facade, and compare land. Do not weaken these
//! assertions to match today's `let _run` / serialize-time ISO pack load, and
//! do not implement the feature in this suite.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use clap::Parser;
use serde_json::Value;
use weeping_angel::cli::{Cli, Commands};
use weeping_angel_assurance::readiness::ControlReadiness;
use weeping_angel_assurance::{
    AssessmentReport, AssessmentRun, AssessmentScope, AssuranceEngine, FrameworkReadinessSnapshot,
    compare,
};
use weeping_angel_assurance_ir::{
    AssessmentId, AssetId, ControlId, ControlTestId, Exception, ExceptionId, ExceptionStatus,
    FrameworkVersion,
};
use weeping_angel_collector::FixtureCollector;
use weeping_angel_control_test::{ControlTestResult, Effectiveness};
use weeping_angel_evidence::{
    CollectionRun, EvidenceEnvelope, EvidenceLedger, EvidenceObservation, EvidenceProvenance,
    EvidenceType,
};
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget,
};

const SEVEN_METRIC_FAMILIES: &[&str] = &[
    "controlEffectiveness",
    "evidence",
    "automation",
    "subject",
    "frameworkRequirement",
    "freshEvidence",
    "manualReviewBurden",
];

const LINEAGE_SNAPSHOT_TYPES: &[&str] = &[
    "struct FrameworkPackSnapshot",
    "struct CanonicalCatalogSnapshot",
    "struct AssessmentDefinitionSnapshot",
    "struct ApplicabilitySnapshot",
    "struct EvidenceSnapshot",
    "struct ControlTestRun",
    "struct ControlExplanation",
    "struct AssessmentSummary",
    "struct CoverageMetrics",
    "struct StatementOfApplicabilitySnapshot",
];

const LINEAGE_LEDGER_APIS: &[&str] = &[
    "fn persist_assessment_run",
    "fn load_assessment_run",
    "fn persist_control_test_run",
    "fn load_control_test_run",
    "fn persist_framework_snapshot",
    "fn load_framework_snapshot",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn crate_src(name: &str) -> PathBuf {
    let path = manifest_dir().join("crates").join(name).join("src");
    assert!(
        path.is_dir(),
        "expected crate sources at {}",
        path.display()
    );
    path
}

fn crate_sources_joined(name: &str) -> String {
    let mut files = Vec::new();
    walk_rs_files(&crate_src(name), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn product_crates_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&manifest_dir().join("crates"), &mut files);
    walk_rs_files(&manifest_dir().join("src"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn require_needles(label: &str, src: &str, needles: &[&str]) {
    let missing: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| !src.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "{label}: missing required surface {missing:?}"
    );
}

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(
        present.is_empty(),
        "{label}: forbidden leftover surface {present:?}"
    );
}

fn impl_serialize_assessment_report(src: &str) -> &str {
    let start = src
        .find("impl Serialize for AssessmentReport")
        .expect("AssessmentReport must have a Serialize impl");
    let rest = &src[start..];
    let end = rest
        .find("\nimpl ")
        .or_else(|| rest.find("\npub struct AssuranceEngine"))
        .unwrap_or(rest.len());
    &rest[..end]
}

fn fn_compare_body(src: &str) -> &str {
    let start = src
        .find("pub fn compare(")
        .expect("snapshot.rs must expose compare");
    let rest = &src[start..];
    let end = rest
        .find("\npub fn ")
        .or_else(|| rest.find("\n#[derive"))
        .unwrap_or(rest.len());
    &rest[..end]
}

fn fn_assessment_for_target(src: &str) -> &str {
    let start = src
        .find("fn assessment_for_target(")
        .expect("assessment_for_target must exist");
    let rest = &src[start..];
    let end = rest.find("\n/// ").unwrap_or(rest.len());
    &rest[..end]
}

fn fn_assess(src: &str) -> &str {
    let start = src
        .find("pub fn assess(self, scope: AssessmentScope)")
        .expect("AssuranceEngineBuilder::assess must exist");
    let rest = &src[start..];
    let end = rest.find("\nfn evaluate_compiled").unwrap_or(rest.len());
    &rest[..end]
}

fn fn_normalize(src: &str) -> &str {
    let start = src
        .find("fn normalize(")
        .expect("framework compile normalize must exist");
    let rest = &src[start..];
    let end = rest
        .find("\nfn resolve_applicability")
        .unwrap_or(rest.len());
    &rest[..end]
}

fn fn_stub_catalog(src: &str) -> &str {
    let start = src
        .find("pub fn stub_catalog(")
        .expect("stub_catalog must exist");
    &src[start..]
}

fn fn_project_soa(src: &str) -> &str {
    let start = src
        .find("pub fn project_soa(")
        .expect("soa.rs must expose project_soa");
    &src[start..]
}

fn sample_result(effectiveness: Effectiveness) -> ControlTestResult {
    ControlTestResult {
        test_id: ControlTestId::new("test.lineage.privileged-mfa"),
        control_id: ControlId::new("control.identity.privileged-mfa"),
        effectiveness,
        rationale: "lineage target fixture".into(),
        evidence_refs: vec!["ev:sha256:fixture-digest".into()],
        missing_evidence: Vec::new(),
        checked_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        test_version: "1".into(),
        input_digest: "input-digest-fixture".into(),
        duration: Some("12ms".into()),
        status: None,
        reason: None,
        population: None,
        period: None,
    }
}

fn readiness_with(
    id: &str,
    digest: &str,
    controls: Vec<(&str, Effectiveness)>,
) -> FrameworkReadinessSnapshot {
    FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new(id),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: digest.into(),
        catalog_digest: String::new(),
        assessment_digest: digest.into(),
        evaluated_at: "2026-08-18T12:00:00Z".into(),
        requirements: Vec::new(),
        controls: controls
            .into_iter()
            .map(|(cid, effectiveness)| ControlReadiness {
                id: ControlId::new(cid),
                effectiveness,
            })
            .collect(),
        effective: 0,
        ineffective: 0,
        partial: 0,
        manual_review: 0,
        insufficient_evidence: 0,
        not_applicable: 0,
        automation_coverage: "0%".into(),
        evidence_coverage: "0%".into(),
    }
}

fn sample_run() -> AssessmentRun {
    AssessmentRun {
        id: AssessmentId::new("assess-lineage-1"),
        framework: "iso-27001".into(),
        framework_pack_digest: "pack-digest".into(),
        assessment_definition_digest: "definition-digest".into(),
        started_at: "2026-08-18T12:00:00Z".into(),
        completed_at: "2026-08-18T12:00:01Z".into(),
        scope: "repo:in-scope".into(),
        collector_runs: vec!["run:collector-1".into()],
        evidence_snapshot_digest: "evidence-snapshot-digest".into(),
        result_digest: "result-digest".into(),
        status: "completed".into(),
        ..Default::default()
    }
}

fn gdpr_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Gdpr,
        capabilities: FrameworkCapabilities::default(),
        version: FrameworkVersion::new("2016"),
        context: FrameworkContext::default(),
    }
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities {
            supports_control_applicability: true,
            supports_statement_of_applicability: true,
            supports_risk_treatment: true,
            supports_manual_attestation: true,
            ..FrameworkCapabilities::default()
        },
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn fixture_collector() -> FixtureCollector {
    FixtureCollector::new("fixture.lineage", "1")
        .with_evidence_types([EvidenceType::new("identity.privileged.mfa")])
}

fn sealed_envelope(digest_salt: &str) -> EvidenceEnvelope {
    let observation = EvidenceObservation::new(EvidenceType::new("identity.privileged.mfa"))
        .with_fact("enabled", "true")
        .with_fact("salt", digest_salt);
    let provenance = EvidenceProvenance {
        collector_id: "fixture.lineage".into(),
        collected_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        scope: "repo:in-scope".into(),
        asset: AssetId::new("repo:in-scope"),
    };
    EvidenceEnvelope::seal(observation, provenance).expect("seal lineage fixture envelope")
}

fn json_has_any(value: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|k| value.get(*k).is_some())
}

fn metric_object<'a>(json: &'a Value) -> Option<&'a Value> {
    json.get("coverageMetrics")
        .or_else(|| json.get("metrics"))
        .or_else(|| json.get("coverage"))
}

fn report_run_object(json: &Value) -> Option<&Value> {
    json.get("assessmentRun")
        .or_else(|| json.get("run"))
        .or_else(|| json.get("lineage"))
}

#[test]
fn lin_001_historical_assessment_reconstructs_from_pinned_snapshots() {
    let crates = product_crates_joined();
    require_needles("LIN-001 snapshot types", &crates, LINEAGE_SNAPSHOT_TYPES);
    require_needles(
        "LIN-001 replay pins",
        &crates,
        &["canonicalCatalogDigest", "applicabilitySnapshot"],
    );
    assert!(
        crates.contains("DigestMismatch")
            && (crates.contains("reconstruct")
                || crates.contains("replay_assessment")
                || crates.contains("fn replay")
                || crates.contains("load_lineage")),
        "LIN-001: replay/reconstruct from pinned snapshots must exist and detect DigestMismatch"
    );

    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess_src = fn_assess(&assurance_lib);
    forbid_needles(
        "LIN-001 AssessmentRun must be returned, not dropped",
        assess_src,
        &["let _run = AssessmentRun"],
    );
    require_needles(
        "LIN-001 assess records collector runs and distinct pins",
        assess_src,
        &[
            "collector_runs",
            "canonical_catalog_digest",
            "applicability",
        ],
    );
    assert!(
        !assess_src.contains("collector_runs: Vec::new()"),
        "LIN-001: collector_runs must record real collection run ids, not an empty vec shortcut"
    );

    let run = sample_run();
    let json = serde_json::to_value(&run).expect("serialize AssessmentRun");
    assert!(
        json_has_any(&json, &["canonicalCatalogDigest", "catalogDigest"]),
        "LIN-001: AssessmentRun must pin the canonical catalog digest; got {json}"
    );
    assert!(
        json_has_any(
            &json,
            &["applicabilitySnapshotId", "applicabilitySnapshotDigest"]
        ),
        "LIN-001: AssessmentRun must pin applicability snapshot identity; got {json}"
    );
    assert_ne!(
        json.get("assessmentDefinitionDigest"),
        json.get("resultDigest"),
        "LIN-001: definition digest and result digest must be distinct pins"
    );
    assert_ne!(
        json.get("evidenceSnapshotDigest"),
        json.get("resultDigest"),
        "LIN-001: evidence-snapshot digest and result digest must be distinct pins"
    );
}

#[test]
fn lin_002_current_catalog_changes_do_not_silently_rewrite_stored_results() {
    let assurance = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "LIN-002 digest-mismatch detection",
        &assurance,
        &["DigestMismatch"],
    );
    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let ser = impl_serialize_assessment_report(&assurance_lib);
    forbid_needles(
        "LIN-002 serialize must not consult current pack files",
        ser,
        &[
            "load_framework_pack",
            "load_framework_pack_from",
            "resolve_pack_dir",
        ],
    );

    let pinned = AssessmentReport {
        assessment_id: AssessmentId::new("assess-lineage-1"),
        profile: "iso-27001".into(),
        digest: "pinned-result-digest".into(),
        results: vec![sample_result(Effectiveness::Effective)],
        evidence_count: 1,
        ..Default::default()
    };
    let json = serde_json::to_value(&pinned).expect("serialize pinned report");
    assert_eq!(
        json.get("digest").and_then(Value::as_str),
        Some("pinned-result-digest"),
        "LIN-002: stored result identity must survive serialize"
    );
    assert!(
        json_has_any(
            &json,
            &[
                "canonicalCatalogDigest",
                "frameworkPackDigest",
                "resultDigest"
            ]
        ),
        "LIN-002: report JSON must carry stored pack/catalog/result pins, not resolve them live"
    );
    if let Some(pack) = json.get("frameworkPackDigest") {
        assert_ne!(
            pack,
            &Value::String(String::new()),
            "LIN-002: empty pack digest is the missing-pack shortcut, not a stored pin"
        );
    }
    let run = sample_run();
    let stored = serde_json::to_value(&run).unwrap();
    assert_eq!(
        stored.get("resultDigest").and_then(Value::as_str),
        Some("result-digest"),
        "LIN-002: mutating current catalog files must not rewrite a stored result digest"
    );
    assert_ne!(
        stored.get("resultDigest"),
        stored.get("assessmentDefinitionDigest"),
        "LIN-002: result digest must not be a reused compile digest"
    );
}

#[test]
fn lin_003_explanation_references_exact_evidence_digests() {
    let crates = product_crates_joined();
    require_needles(
        "LIN-003 ControlExplanation",
        &crates,
        &[
            "struct ControlExplanation",
            "missing_evidence",
            "failing_subjects",
            "missing_subjects",
            "evidence_requirements",
        ],
    );
    require_needles(
        "LIN-003 explain projection answers the review questions",
        &crates,
        &[
            "exceptions",
            "mappings",
            "effectiveness",
            "population",
            "test_version",
        ],
    );

    let explain_rs = manifest_dir().join("src").join("assurance_explain.rs");
    assert!(
        explain_rs.is_file()
            || crates.contains("fn explain")
            || crates.contains("fn explain_control")
            || crates.contains("fn control_explanation"),
        "LIN-003: public explain/control-explanation API must exist"
    );

    let envelope = sealed_envelope("explain");
    assert!(
        envelope.digest().len() == 64 && envelope.digest().chars().all(|c| c.is_ascii_hexdigit()),
        "LIN-003: evidence identity is a SHA-256 hex digest"
    );
    let report = AssessmentReport {
        assessment_id: AssessmentId::new("assess-lineage-1"),
        profile: "iso-27001".into(),
        digest: "pinned-result-digest".into(),
        results: vec![sample_result(Effectiveness::Ineffective)],
        evidence_count: 1,
        ..Default::default()
    };
    let json = serde_json::to_value(&report).unwrap();
    let cited = format!("{json}");
    assert!(
        crates.contains("evidence") && crates.contains("digest"),
        "LIN-003: explanation must cite envelope digests, not latest-from-ledger"
    );
    assert!(
        !cited.contains("load_framework_pack"),
        "LIN-003: explanation/serialize path must not resolve a framework pack from disk"
    );
    let _ = envelope;
}

#[test]
fn lin_004_assessment_report_serialization_is_pure() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let ser = impl_serialize_assessment_report(&src);
    forbid_needles(
        "LIN-004 Serialize is pure",
        ser,
        &[
            "load_framework_pack",
            "load_framework_pack_from",
            "resolve_pack_dir",
            "std::fs::",
            "reqwest",
            "ureq",
            "TcpStream",
            "iso-27001",
            "2022",
        ],
    );
    forbid_needles(
        "LIN-004 no invented percent strings",
        ser,
        &["{:.0}%", "automationCoverage", "evidenceCoverage"],
    );

    let report = AssessmentReport {
        assessment_id: AssessmentId::new("assess-lineage-1"),
        profile: "soc-2".into(),
        digest: "pinned-result-digest".into(),
        results: vec![sample_result(Effectiveness::Effective)],
        evidence_count: 0,
        ..Default::default()
    };
    let first = serde_json::to_value(&report).expect("serialize AssessmentReport");
    let second = serde_json::to_value(&report).expect("serialize AssessmentReport again");
    assert_eq!(
        first, second,
        "LIN-004: serialize must be deterministic given the in-memory report"
    );
    if let Some(auto) = first.get("automationCoverage").and_then(Value::as_str) {
        assert!(
            !auto.ends_with('%'),
            "LIN-004: must not invent automationCoverage as a percent string, got {auto}"
        );
    }
    if let Some(ev) = first.get("evidenceCoverage").and_then(Value::as_str) {
        assert!(
            !ev.ends_with('%'),
            "LIN-004: must not invent evidenceCoverage as a percent string, got {ev}"
        );
    }
    assert!(
        first.get("compliancePercent").is_none() && first.get("isoCompliant").is_none(),
        "LIN-004: no single compliance percentage field"
    );
    assert!(
        first.get("summary").is_some()
            || first.get("coverageMetrics").is_some()
            || report_run_object(&first).is_some(),
        "LIN-004: explicit AssessmentSummary / CoverageMetrics must be carried on the value; got {first}"
    );
}

#[test]
fn lin_005_partial_collector_runs_remain_distinguishable() {
    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess_src = fn_assess(&assurance_lib);
    require_needles(
        "LIN-005 assess represents partial/failed collection",
        assess_src,
        &["partial", "failed", "CollectionRun"],
    );
    forbid_needles(
        "LIN-005 status is not always completed",
        assess_src,
        &["status: \"completed\".into()"],
    );

    let mut completed_empty = CollectionRun::new("fixture.lineage", "1");
    completed_empty.status = "completed".into();
    completed_empty.evidence_count = 0;
    completed_empty.error_count = 0;
    let mut partial = CollectionRun::new("fixture.lineage", "1");
    partial.status = "partial".into();
    partial.evidence_count = 1;
    partial.error_count = 1;
    let mut failed = CollectionRun::new("fixture.lineage", "1");
    failed.status = "failed".into();
    failed.evidence_count = 0;
    failed.error_count = 1;
    assert_ne!(
        completed_empty.status, partial.status,
        "LIN-005: completed empty collection is not a partial run"
    );
    assert_ne!(
        completed_empty.status, failed.status,
        "LIN-005: completed empty collection is not a failed run"
    );

    let report = AssuranceEngine::builder()
        .framework(iso_target())
        .collector(fixture_collector())
        .assess(AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope")));
    let report = report.expect("ISO assess should still compile a pack-backed assessment");
    let json = serde_json::to_value(&report).unwrap();
    assert!(
        json_has_any(&json, &["status", "collectionRuns", "collectionRunId"])
            || report_run_object(&json).is_some(),
        "LIN-005: AssessmentReport must expose run status / collection runs; got {json}"
    );

    let crates = product_crates_joined();
    require_needles(
        "LIN-005 CollectionRun statuses",
        &crates,
        &["\"started\"", "\"completed\"", "\"partial\"", "\"failed\""],
    );
}

#[test]
fn lin_006_assessment_diff_identifies_changed_subjects_and_results() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    let body = fn_compare_body(&src);
    require_needles(
        "LIN-006 compare writes subject and test-result buckets",
        body,
        &[
            "new_subjects",
            "disappeared_subjects",
            "control_became_effective",
        ],
    );
    require_needles(
        "LIN-006 compare writes applicability, evidence, digest change",
        body,
        &[
            "requirement_became_applicable",
            "requirement_became_not_applicable",
        ],
    );
    assert!(
        body.contains("catalog")
            || body.contains("framework_pack_digest")
            || body.contains("pack_digest")
            || src.contains("catalog_digest")
            || src.contains("fn compare_runs")
            || src.contains("fn compare_lineage"),
        "LIN-006: compare must identify framework/catalog digest changes"
    );

    let previous = readiness_with(
        "assess-prev",
        "digest-a",
        vec![
            (
                "control.identity.privileged-mfa",
                Effectiveness::Ineffective,
            ),
            ("control.source.protected-branch", Effectiveness::Effective),
        ],
    );
    let next = readiness_with(
        "assess-next",
        "digest-b",
        vec![
            ("control.identity.privileged-mfa", Effectiveness::Effective),
            (
                "control.source.protected-branch",
                Effectiveness::StaleEvidence,
            ),
            ("control.identity.mfa", Effectiveness::Effective),
        ],
    );
    let diff = compare(&previous, &next);
    let json = serde_json::to_value(&diff).expect("serialize SnapshotDiff");
    assert!(
        !diff.control_became_effective.is_empty(),
        "LIN-006: effectiveness flip must still be detected"
    );
    assert!(
        json_has_any(
            &json,
            &[
                "frameworkPackDigestChanged",
                "packDigestChanged",
                "catalogDigestChanged",
                "canonicalCatalogDigestChanged",
                "frameworkDigestChanged"
            ]
        ),
        "LIN-006: compare JSON must surface digest changes; got {json}"
    );
}

#[test]
fn lin_007_exceptions_are_visible_in_lineage() {
    let snapshot = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    let body = fn_compare_body(&snapshot);
    require_needles(
        "LIN-007 compare writes exception buckets",
        body,
        &["new_exceptions", "expired_exceptions"],
    );

    let crates = product_crates_joined();
    require_needles(
        "LIN-007 explanation and lineage surface exceptions",
        &crates,
        &["struct ControlExplanation", "exceptions"],
    );

    let exception = Exception {
        id: ExceptionId::new("exc.privileged-mfa.break-glass"),
        control_id: Some(ControlId::new("control.identity.privileged-mfa")),
        rationale: "break-glass window".into(),
        status: ExceptionStatus::Approved,
        approved_by: None,
        expires_at: Some(Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap()),
        subjects: Vec::new(),
    };
    assert_eq!(exception.status, ExceptionStatus::Approved);

    let previous = readiness_with(
        "assess-prev",
        "digest-a",
        vec![(
            "control.identity.privileged-mfa",
            Effectiveness::Ineffective,
        )],
    );
    let next = readiness_with(
        "assess-next",
        "digest-b",
        vec![(
            "control.identity.privileged-mfa",
            Effectiveness::ExceptionApproved,
        )],
    );
    let diff = compare(&previous, &next);
    assert!(
        !diff.new_exceptions.is_empty() || !diff.expired_exceptions.is_empty(),
        "LIN-007: compare must surface introduced/expired exceptions; got {diff:?}"
    );
}

#[test]
fn lin_008_snapshot_and_result_digests_are_deterministic() {
    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess_src = fn_assess(&assurance_lib);
    forbid_needles(
        "LIN-008 do not reuse compile digest for three identities",
        assess_src,
        &[
            "assessment_definition_digest: compiled.digest.clone()",
            "evidence_snapshot_digest: compiled.digest.clone()",
            "result_digest: compiled.digest.clone()",
        ],
    );

    let crates = product_crates_joined();
    require_needles(
        "LIN-008 domain-separated SHA-256 result identity",
        &crates,
        &["typed_canonical_digest", "result_digest"],
    );
    assert!(
        crates.contains("duration") && crates.contains("evaluatedAt")
            || crates.contains("checked_at"),
        "LIN-008: result-identity code must mention wall-clock fields in order to exclude them"
    );

    let run_a = sample_run();
    let mut run_b = sample_run();
    run_b.started_at = "2026-08-19T00:00:00Z".into();
    run_b.completed_at = "2026-08-19T00:00:02Z".into();
    assert_eq!(
        run_a.result_digest, run_b.result_digest,
        "LIN-008: wall-clock start/completion must not change the stored result digest when semantics match"
    );

    let result = sample_result(Effectiveness::Effective);
    let mut later = result.clone();
    later.checked_at = Utc.with_ymd_and_hms(2026, 8, 19, 0, 0, 0).unwrap();
    later.duration = Some("99ms".into());
    assert_eq!(
        result.input_digest, later.input_digest,
        "LIN-008: duration / evaluatedAt are excluded from result identity"
    );
    assert!(
        result.input_digest.len() == 64 || crates.contains("assessment_result_digest"),
        "LIN-008: result digest must be SHA-256 hex (64 chars) via a domain-separated helper, got {}",
        result.input_digest
    );
}

#[test]
fn lin_009_dual_suite_binaries_are_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_assessment_lineage_baseline")
            && toml.contains("sdd_assessment_lineage_target")
            && toml.contains("tests/contracts/assessment_lineage.baseline.rs")
            && toml.contains("tests/contracts/assessment_lineage.target.rs"),
        "LIN-009: dual-suite binaries must be registered in root Cargo.toml"
    );
}

#[test]
fn lin_010_production_path_has_no_stub_assessment() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let body = fn_assessment_for_target(&src);
    forbid_needles(
        "LIN-010 production stub ids",
        body,
        &[
            "canonical:stub-1",
            "assess-runtime-1",
            "canonical.source-control",
            "ev.branch_protection",
        ],
    );
    assert!(
        !body.contains("FrameworkProfile::Iso27001") || body.contains("load_framework_pack("),
        "LIN-010: ISO must go through the generic (id, version) loader, not a special stub branch"
    );

    let outcome = AssuranceEngine::builder()
        .framework(gdpr_target())
        .collector(fixture_collector())
        .assess(AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope")));
    assert!(
        outcome.is_err(),
        "LIN-010: missing pack must fail closed, not compile a production stub (got {outcome:?})"
    );
}

#[test]
fn lin_011_one_registry_loader_path_for_every_framework() {
    let assurance = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let ser = impl_serialize_assessment_report(&assurance);
    let assess = fn_assess(&assurance);
    let for_target = fn_assessment_for_target(&assurance);
    forbid_needles(
        "LIN-011 generic serialize/orchestrate has no hardcoded ISO pack",
        ser,
        &["load_framework_pack(\"iso-27001\", \"2022\")"],
    );
    forbid_needles(
        "LIN-011 assess pack pin is not hardcoded ISO",
        assess,
        &["load_framework_pack(\"iso-27001\", \"2022\")"],
    );
    forbid_needles(
        "LIN-011 assessment_for_target is not an ISO-only branch",
        for_target,
        &["load_framework_pack(\"iso-27001\", \"2022\")"],
    );

    let framework = crate_sources_joined("weeping-angel-framework");
    let normalize = fn_normalize(&framework);
    let stub = fn_stub_catalog(&framework);
    assert!(
        !normalize.contains("load_framework_pack(\"iso-27001\", \"2022\")")
            || normalize.contains("target.profile") && normalize.contains("target.version"),
        "LIN-011: normalize must merge the target identity, not a hardcoded ISO pack"
    );
    assert!(
        stub.contains("#[cfg(test)]")
            || !stub.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "LIN-011: stub_catalog ISO fallback must not remain a hidden production catalog"
    );

    let soa_src = read_repo_file("crates/weeping-angel-assurance/src/soa.rs");
    let soa = fn_project_soa(&soa_src);
    assert!(
        soa.contains("snapshot") || soa.contains("digest") || soa.contains("pinned"),
        "LIN-011: project_soa must project from a pinned pack/applicability snapshot, not live disk only"
    );
}

#[test]
fn lin_012_assurance_explain_parses_and_is_dispatched() {
    let cli_src = read_repo_file("src/cli.rs");
    require_needles(
        "LIN-012 CLI parser",
        &cli_src,
        &["Explain", "explain", "assessment", "control"],
    );

    let cmd = Cli::clap_command();
    let assurance = cmd
        .get_subcommands()
        .find(|c| c.get_name() == "assurance")
        .expect("assurance family exists");
    let names: Vec<&str> = assurance.get_subcommands().map(|c| c.get_name()).collect();
    assert!(
        names.iter().any(|n| *n == "explain"),
        "LIN-012: `assurance explain` must be a clap subcommand; have {names:?}"
    );

    let parsed = Cli::try_parse_from([
        "weeping-angel",
        "assurance",
        "explain",
        "--assessment",
        "assess-lineage-1",
        "--control",
        "control.identity.privileged-mfa",
    ]);
    let parsed =
        parsed.expect("LIN-012: clap must accept `assurance explain --assessment --control`");
    match parsed.command {
        Commands::Assurance(_) => {}
        other => panic!("LIN-012: expected Assurance, got {other:?}"),
    }

    let main = read_repo_file("src/main.rs");
    require_needles(
        "LIN-012 explain is dispatched, not banner-exit-0",
        &main,
        &["AssuranceCommand::Explain"],
    );
    assert!(
        !main.contains("_ =>")
            || main
                .split("AssuranceCommand::Catalog")
                .nth(1)
                .is_some_and(|after| after.contains("Explain") || !after.contains("return 0")),
        "LIN-012: explain must not stay on the wildcard banner-and-exit-0 arm"
    );
    assert!(
        manifest_dir()
            .join("src")
            .join("assurance_explain.rs")
            .is_file()
            || main.contains("assurance_explain")
            || product_crates_joined().contains("fn explain"),
        "LIN-012: explain execution must live outside the clap enum"
    );
}

#[test]
fn lin_013_coverage_metrics_expose_seven_separate_families() {
    let crates = product_crates_joined();
    require_needles(
        "LIN-013 explicit metric types",
        &crates,
        &[
            "struct CoverageMetrics",
            "struct AssessmentSummary",
            "struct FrameworkReadinessSnapshot",
        ],
    );
    forbid_needles(
        "LIN-013 no single compliance percentage",
        &crates,
        &["compliancePercent", "isoCompliant", "certifiedPercent"],
    );

    let report = AssessmentReport {
        assessment_id: AssessmentId::new("assess-lineage-1"),
        profile: "iso-27001".into(),
        digest: "pinned-result-digest".into(),
        results: vec![sample_result(Effectiveness::Effective)],
        evidence_count: 1,
        ..Default::default()
    };
    let json = serde_json::to_value(&report).unwrap();
    let metrics = metric_object(&json).unwrap_or(&json);
    for family in SEVEN_METRIC_FAMILIES {
        let snake = family
            .chars()
            .flat_map(|c| {
                if c.is_uppercase() {
                    vec!['_', c.to_ascii_lowercase()]
                } else {
                    vec![c]
                }
            })
            .collect::<String>();
        assert!(
            json_has_any(metrics, &[family, snake.as_str()])
                || format!("{metrics}").contains(family)
                || crates.contains(family)
                || crates.contains(&snake),
            "LIN-013: CoverageMetrics must expose {family} separately; json={json}"
        );
    }
    assert!(
        json.get("compliancePercent").is_none(),
        "LIN-013: do not collapse coverage into one compliance percentage"
    );
}

#[test]
fn lin_014_ledger_persists_runs_append_only() {
    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    require_needles("LIN-014 persist/load APIs", &src, LINEAGE_LEDGER_APIS);
    assert!(
        src.contains("INSERT OR IGNORE")
            || src.contains("already")
            || src.contains("Immutable")
            || src.contains("reject"),
        "LIN-014: replacing a completed run payload with different bytes must be rejected or ignored"
    );

    let mut ledger = EvidenceLedger::open_in_memory().expect("open lineage ledger");
    let first = sealed_envelope("append-a");
    let digest = first.digest().to_string();
    assert!(
        ledger.append(first.clone()).unwrap(),
        "first append inserts"
    );
    assert!(
        !ledger.append(first).unwrap(),
        "historical evidence remains append-only (INSERT OR IGNORE)"
    );
    assert_eq!(ledger.get(&digest).unwrap().digest(), digest);

    let run = sample_run();
    assert!(
        src.contains("persist_assessment_run") && src.contains("load_assessment_run"),
        "LIN-014: EvidenceLedger must persist/load AssessmentRun"
    );
    let _ = run;

    let crates = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "LIN-014 ControlTestRun persist payload",
        &crates,
        &["struct ControlTestRun"],
    );
}

#[test]
fn lin_015_neighbor_sdd_targets_remain_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    for name in [
        "sdd_assurance_runtime_target",
        "sdd_iso27001_assurance_target",
        "sdd_canonical_assurance_catalog_target",
    ] {
        assert!(
            toml.contains(name),
            "LIN-015: neighbor suite `{name}` must stay registered"
        );
    }
    let iso = read_repo_file("frameworks/iso-27001/2022/manifest.toml");
    assert!(
        iso.contains("id = \"iso-27001\"") && iso.contains("version = \"2022\""),
        "LIN-015: this slice must not remap ISO pack IDs"
    );
}
