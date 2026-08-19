//! Target suite for continuity / resilience assurance (Prompt 20).
//!
//! Encodes DESIRED behavior in `docs/specs/continuity-resilience.md` §4 / §5
//! (P20-T01–T16). Compile-safe on characterization HEAD: does not import
//! types that do not exist. Fails because `AssessmentDefinition` does not
//! persist `continuityProfiles`, IR/eval types are absent, and
//! `evaluate_continuity_resilience` is not callable. Do not `#[ignore]`.
//! Do not implement product evaluation here.
//!
//! Implement replaces `require_continuity_eval` / `assert_expected_verdict`
//! with calls to `evaluate_continuity_resilience` while keeping these titles.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel_assurance::evaluate_continuity_resilience;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind,
    DocumentKind, DocumentRef, Risk, RiskId, ValidateIr, Vendor, VendorId,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType, EvidenceValue,
};

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
    manifest_dir().join("crates").join(name).join("src")
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

fn product_crate_sources_joined() -> String {
    let crates_dir = manifest_dir().join("crates");
    let mut chunks = Vec::new();
    for entry in fs::read_dir(&crates_dir).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let src = entry.path().join("src");
        if !src.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        walk_rs_files(&src, &mut files);
        for path in files {
            chunks.push(fs::read_to_string(&path).unwrap());
        }
    }
    chunks.join("\n")
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
        "{label}: missing continuity/resilience surface {missing:?}"
    );
}

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(present.is_empty(), "{label}: forbidden surface {present:?}");
}

const CONTINUITY_IR_NEEDLES: &[&str] = &[
    "struct ContinuityResilienceProfile",
    "struct ContinuityResilienceVerdict",
    "enum ServiceCriticality",
    "struct ServiceDependency",
    "struct RecoveryObjective",
    "typed_id!(RecoveryObjectiveId)",
    "typed_id!(ContinuityExerciseId)",
    "typed_id!(ContinuityProfileId)",
    "struct BackupExpectation",
    "struct ContinuityExercise",
    "enum ExerciseKind",
    "TechnicalRecovery",
    "RestoreTest",
    "Tabletop",
    "Walkthrough",
    "struct ExerciseResult",
    "observed_recovery_duration_seconds",
    "observed_data_loss_window_seconds",
    "struct ExerciseIssue",
    "struct ContinuityGap",
    "struct RecoveryProcedureRef",
    "continuity_profiles",
    "rto_seconds",
    "rpo_seconds",
    "demonstrated_recovery",
];

const CONTINUITY_EVAL_NEEDLES: &[&str] = &[
    "fn evaluate_continuity_resilience",
    "struct ContinuityResilienceVerdict",
    "plan_existence",
    "backup_configuration",
    "successful_restore",
    "exercise_cadence",
    "rto_achievement",
    "rpo_achievement",
    "unresolved_exercise_findings",
    "dependency_coverage",
    "demonstrated_recovery",
];

fn require_continuity_eval(label: &str) {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(label, &ir, CONTINUITY_IR_NEEDLES);
    assert!(
        crate_src("weeping-angel-assurance-ir")
            .join("continuity.rs")
            .is_file(),
        "{label}: expected crates/weeping-angel-assurance-ir/src/continuity.rs"
    );
    let assurance = crate_sources_joined("weeping-angel-assurance");
    require_needles(label, &assurance, CONTINUITY_EVAL_NEEDLES);
    assert!(
        crate_src("weeping-angel-assurance")
            .join("continuity.rs")
            .is_file(),
        "{label}: expected crates/weeping-angel-assurance/src/continuity.rs"
    );
    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        lib.contains("pub mod continuity") || lib.contains("evaluate_continuity_resilience"),
        "{label}: weeping-angel-assurance must export evaluate_continuity_resilience"
    );
}

fn as_of() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 30, 0).unwrap()
}

fn as_of_ctx() -> AssessmentContext {
    AssessmentContext {
        now: as_of(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn seal(evidence_type: &str, asset: &str, facts: &[(&str, EvidenceValue)]) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new(evidence_type));
    for (k, v) in facts {
        obs = obs.with_value(*k, v.clone());
    }
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.continuity-resilience-target".into(),
            collected_at: as_of() - chrono::Duration::hours(1),
            scope: "target".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn graph_assessment() -> AssessmentDefinition {
    let mut assessment =
        AssessmentDefinition::new(AssessmentId::new("assess.continuity-resilience.target"));
    assessment.assets.push(Asset::new(
        AssetId::new("asset:payments"),
        AssetKind::Service,
        "payments",
    ));
    assessment.assets.push(Asset::new(
        AssetId::new("asset:payments-db"),
        AssetKind::Database,
        "payments-db",
    ));
    assessment
        .vendors
        .push(Vendor::new(VendorId::new("vendor:stripe"), "Stripe"));
    assessment.risks.push(Risk::new(
        RiskId::new("risk:unproven-recovery"),
        "unproven recovery",
        "plan exists without demonstrated restore",
    ));
    assessment
}

fn decode_with_profiles(profiles: Vec<Value>) -> AssessmentDefinition {
    let mut encoded = serde_json::to_value(graph_assessment()).unwrap();
    let obj = encoded.as_object_mut().expect("assessment JSON object");
    obj.insert("continuityProfiles".into(), Value::Array(profiles));
    serde_json::from_value(encoded).expect("assessment with continuityProfiles must decode")
}

fn retained_profiles(assessment: &AssessmentDefinition) -> Vec<Value> {
    serde_json::to_value(assessment)
        .unwrap()
        .get("continuityProfiles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn require_retained_profile(label: &str, assessment: &AssessmentDefinition) -> Value {
    let rows = retained_profiles(assessment);
    assert_eq!(
        rows.len(),
        1,
        "{label}: AssessmentDefinition.continuityProfiles must persist the fixture profile"
    );
    rows.into_iter().next().unwrap()
}

fn payments_dependency(critical: bool) -> Value {
    json!({
        "from": "asset:payments",
        "to": { "asset": "asset:payments-db" },
        "kind": "data",
        "critical": critical
    })
}

fn stripe_dependency() -> Value {
    json!({
        "from": "asset:payments",
        "to": { "vendor": "vendor:stripe" },
        "kind": "supplier",
        "critical": true
    })
}

fn payments_objective() -> Value {
    json!({
        "id": "rto.payments",
        "subject": "asset:payments",
        "rtoSeconds": 3600,
        "rpoSeconds": 300
    })
}

fn backup_expectation() -> Value {
    json!({
        "subject": "asset:payments-db",
        "required": true,
        "evidenceType": "evidence.backup.configuration"
    })
}

fn bcp_procedure() -> Value {
    json!({
        "document": {
            "id": "doc:bcp-payments",
            "title": "Payments BCP",
            "kind": "plan"
        },
        "role": "businessContinuityPlan"
    })
}

fn profile_shell(exercises: Vec<Value>, results: Vec<Value>, deps: Vec<Value>) -> Value {
    json!({
        "id": "crp.payments",
        "service": "asset:payments",
        "criticality": "missionCritical",
        "dependencies": deps,
        "objectives": [payments_objective()],
        "backupExpectations": [backup_expectation()],
        "procedures": [bcp_procedure()],
        "exerciseCadenceSeconds": 7_776_000,
        "exercises": exercises,
        "results": results
    })
}

fn technical_exercise(at: &str, in_scope: Vec<Value>) -> Value {
    json!({
        "id": "ex.payments.restore",
        "subject": "asset:payments",
        "kind": "restoreTest",
        "conductedAt": at,
        "procedure": bcp_procedure(),
        "inScopeDependencies": in_scope
    })
}

fn tabletop_exercise(at: &str, in_scope: Vec<Value>) -> Value {
    json!({
        "id": "ex.payments.tabletop",
        "subject": "asset:payments",
        "kind": "tabletop",
        "conductedAt": at,
        "procedure": bcp_procedure(),
        "inScopeDependencies": in_scope
    })
}

fn covered_scope() -> Vec<Value> {
    vec![
        json!({ "asset": "asset:payments-db" }),
        json!({ "vendor": "vendor:stripe" }),
    ]
}

fn passed_technical_result(
    duration: u64,
    data_loss: u64,
    issues: Vec<Value>,
    remediation_refs: Vec<Value>,
) -> Value {
    json!({
        "exerciseId": "ex.payments.restore",
        "outcome": "passed",
        "observedRecoveryDurationSeconds": duration,
        "observedDataLossWindowSeconds": data_loss,
        "issues": issues,
        "remediationRefs": remediation_refs,
        "riskRefs": [{ "id": "risk:unproven-recovery" }]
    })
}

fn t01_plan_no_exercise() -> Value {
    profile_shell(
        vec![],
        vec![],
        vec![payments_dependency(true), stripe_dependency()],
    )
}

fn t02_successful_restore() -> Value {
    profile_shell(
        vec![technical_exercise("2026-08-01T12:00:00Z", covered_scope())],
        vec![passed_technical_result(1800, 60, vec![], vec![])],
        vec![payments_dependency(true), stripe_dependency()],
    )
}

fn t03_failed_restore() -> Value {
    let mut result = passed_technical_result(1800, 60, vec![], vec![]);
    result["outcome"] = json!("failed");
    profile_shell(
        vec![technical_exercise("2026-08-01T12:00:00Z", covered_scope())],
        vec![result],
        vec![payments_dependency(true), stripe_dependency()],
    )
}

fn t04_stale_exercise() -> Value {
    profile_shell(
        vec![technical_exercise("2025-01-01T12:00:00Z", covered_scope())],
        vec![passed_technical_result(1800, 60, vec![], vec![])],
        vec![payments_dependency(true), stripe_dependency()],
    )
}

fn t05_uncovered_dependency() -> Value {
    profile_shell(
        vec![technical_exercise(
            "2026-08-01T12:00:00Z",
            vec![json!({ "asset": "asset:payments-db" })],
        )],
        vec![passed_technical_result(1800, 60, vec![], vec![])],
        vec![payments_dependency(true), stripe_dependency()],
    )
}

fn t06_missing_backup() -> Value {
    t02_successful_restore()
}

fn t07_tabletop_only() -> Value {
    profile_shell(
        vec![tabletop_exercise("2026-08-01T12:00:00Z", covered_scope())],
        vec![json!({
            "exerciseId": "ex.payments.tabletop",
            "outcome": "passed",
            "observedRecoveryDurationSeconds": 900,
            "observedDataLossWindowSeconds": 0,
            "issues": [],
            "remediationRefs": [],
            "riskRefs": []
        })],
        vec![payments_dependency(true), stripe_dependency()],
    )
}

fn t08_open_finding() -> Value {
    let issue = json!({
        "id": "issue.restore-runbook-gap",
        "summary": "restore runbook skipped a replica",
        "open": true,
        "remediationRefs": [{ "id": "rem.restore-runbook" }]
    });
    profile_shell(
        vec![technical_exercise("2026-08-01T12:00:00Z", covered_scope())],
        vec![passed_technical_result(
            1800,
            60,
            vec![issue],
            vec![json!({ "id": "rem.restore-runbook" })],
        )],
        vec![payments_dependency(true), stripe_dependency()],
    )
}

fn t08_untracked_finding() -> Value {
    let issue = json!({
        "id": "issue.untracked",
        "summary": "open finding with no remediation",
        "open": true,
        "remediationRefs": []
    });
    profile_shell(
        vec![technical_exercise("2026-08-01T12:00:00Z", covered_scope())],
        vec![passed_technical_result(1800, 60, vec![issue], vec![])],
        vec![payments_dependency(true), stripe_dependency()],
    )
}

fn intention_evidence() -> EvidenceSet {
    let mut set = EvidenceSet::new();
    set.insert(seal(
        "evidence.resilience.recovery-plan",
        "asset:payments",
        &[
            ("procedure_present", EvidenceValue::Bool(true)),
            ("objectives_documented", EvidenceValue::Bool(true)),
            (
                "reviewed_at",
                EvidenceValue::Timestamp(as_of() - chrono::Duration::days(10)),
            ),
        ],
    ));
    set.insert(seal(
        "evidence.resilience.continuity-plan",
        "asset:payments",
        &[
            (
                "reviewed_at",
                EvidenceValue::Timestamp(as_of() - chrono::Duration::days(30)),
            ),
            ("plan_kind", EvidenceValue::String("bcp".into())),
        ],
    ));
    set
}

fn capability_evidence(include_backup: bool, restore_success: Option<bool>) -> EvidenceSet {
    let mut set = intention_evidence();
    if include_backup {
        set.insert(seal(
            "evidence.backup.configuration",
            "asset:payments-db",
            &[
                ("enabled", EvidenceValue::Bool(true)),
                ("retention_days", EvidenceValue::Integer(30)),
            ],
        ));
    }
    if let Some(success) = restore_success {
        set.insert(seal(
            "evidence.backup.restore-test",
            "asset:payments-db",
            &[
                (
                    "tested_at",
                    EvidenceValue::Timestamp(Utc.with_ymd_and_hms(2026, 8, 1, 12, 0, 0).unwrap()),
                ),
                ("success", EvidenceValue::Bool(success)),
            ],
        ));
    }
    set
}

fn assert_expected_verdict(
    label: &str,
    assessment: &AssessmentDefinition,
    evidence: &EvidenceSet,
    expected: Value,
) {
    require_continuity_eval(label);
    let profile = assessment
        .continuity_profiles
        .first()
        .unwrap_or_else(|| panic!("{label}: missing continuity profile"));
    let verdict = evaluate_continuity_resilience(assessment, profile, evidence, as_of())
        .unwrap_or_else(|err| panic!("{label}: evaluate_continuity_resilience failed: {err}"));
    let actual = serde_json::to_value(&verdict).unwrap();
    assert_verdict_matches(label, &actual, &expected);
}

fn assert_verdict_matches(label: &str, actual: &Value, expected: &Value) {
    let exp = expected
        .as_object()
        .unwrap_or_else(|| panic!("{label}: expected verdict object"));
    let act = actual
        .as_object()
        .unwrap_or_else(|| panic!("{label}: actual verdict object"));
    for (key, ev) in exp {
        if key == "gaps" {
            let exp_gaps = ev.as_array().cloned().unwrap_or_default();
            let act_gaps = act
                .get("gaps")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if exp_gaps.is_empty() {
                assert!(
                    act_gaps.is_empty(),
                    "{label}: expected no gaps, got {act_gaps:?}"
                );
                continue;
            }
            for eg in &exp_gaps {
                let dim = eg
                    .get("dimension")
                    .and_then(Value::as_str)
                    .unwrap_or_else(|| panic!("{label}: expected gap missing dimension"));
                let found = act_gaps
                    .iter()
                    .find(|ag| ag.get("dimension").and_then(Value::as_str) == Some(dim));
                assert!(
                    found.is_some(),
                    "{label}: every failing dimension must emit a ContinuityGap; missing {dim} in {act_gaps:?}"
                );
                let ag = found.unwrap();
                if let Some(obj) = eg.as_object() {
                    for (field, fv) in obj {
                        if field == "dimension" {
                            continue;
                        }
                        assert_eq!(
                            ag.get(field),
                            Some(fv),
                            "{label}: gap {dim} field {field} mismatch; actual={ag}"
                        );
                    }
                }
            }
            continue;
        }
        assert_eq!(
            act.get(key),
            Some(ev),
            "{label}: field {key} mismatch; actual={actual} expected={expected}"
        );
    }
}

fn assert_gap_on(label: &str, expected: &Value, dimension: &str) {
    let gaps = expected
        .get("gaps")
        .and_then(Value::as_array)
        .expect("expected.gaps");
    assert!(
        gaps.iter()
            .any(|g| g.get("dimension").and_then(Value::as_str) == Some(dimension)),
        "{label}: every failing dimension must emit a ContinuityGap; missing {dimension} in {gaps:?}"
    );
}

/// P20-T11: dual-suite + spec + CANONICAL_SPECS so cargo can invoke both binaries.
#[test]
fn p20_t11_dual_suite_is_registered_and_specified() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_continuity_resilience_baseline")
            && toml.contains("sdd_continuity_resilience_target")
            && toml.contains("tests/contracts/continuity_resilience.baseline.rs")
            && toml.contains("tests/contracts/continuity_resilience.target.rs"),
        "target suite must be listed in root Cargo.toml"
    );
    let spec = read_repo_file("docs/specs/continuity-resilience.md");
    for id in [
        "P20-T01", "P20-T02", "P20-T03", "P20-T04", "P20-T05", "P20-T06", "P20-T07", "P20-T08",
        "P20-T09", "P20-T10", "P20-T11", "P20-T12", "P20-T13", "P20-T14", "P20-T15", "P20-T16",
    ] {
        assert!(spec.contains(id), "spec must list target case {id}");
    }
    assert!(
        spec.contains("A plan document alone MUST NEVER prove recovery capability"),
        "spec must state the plan≠capability law"
    );
    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/continuity-resilience.md"),
        "P20-T11: spec must remain in CANONICAL_SPECS"
    );
}

/// P20-T01: current plan but no exercise is not demonstrated recovery.
#[test]
fn p20_t01_current_plan_without_exercise_is_not_demonstrated_recovery() {
    let assessment = decode_with_profiles(vec![t01_plan_no_exercise()]);
    let row = require_retained_profile("P20-T01", &assessment);
    assert_eq!(row["id"], "crp.payments");
    assert_eq!(row["service"], "asset:payments");
    assert_eq!(row["criticality"], "missionCritical");
    assert_eq!(row["procedures"][0]["document"]["id"], "doc:bcp-payments");
    assert!(
        row["exercises"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(true),
        "P20-T01 fixture has no exercise"
    );
    let evidence = capability_evidence(true, None);
    let expected = json!({
        "planExistence": "satisfied",
        "backupConfiguration": "satisfied",
        "successfulRestore": "missing",
        "exerciseCadence": "missing",
        "rtoAchievement": "notMeasured",
        "rpoAchievement": "notMeasured",
        "unresolvedExerciseFindings": "none",
        "dependencyCoverage": "gap",
        "demonstratedRecovery": false,
        "gaps": [
            { "dimension": "successfulRestore" },
            { "dimension": "exerciseCadence" },
            { "dimension": "rtoAchievement" },
            { "dimension": "rpoAchievement" },
            { "dimension": "dependencyCoverage" }
        ]
    });
    for dim in [
        "successfulRestore",
        "exerciseCadence",
        "rtoAchievement",
        "rpoAchievement",
        "dependencyCoverage",
    ] {
        assert_gap_on("P20-T01", &expected, dim);
    }
    assert_eq!(expected["demonstratedRecovery"], false);
    assert_eq!(expected["planExistence"], "satisfied");
    assert_expected_verdict(
        "P20-T01: current plan but no exercise",
        &assessment,
        &evidence,
        expected,
    );
}

/// P20-T02: successful technical exercise within RTO/RPO demonstrates recovery.
#[test]
fn p20_t02_successful_exercise_within_rto_rpo_demonstrates_recovery() {
    let assessment = decode_with_profiles(vec![t02_successful_restore()]);
    let row = require_retained_profile("P20-T02", &assessment);
    assert_eq!(row["exercises"][0]["kind"], "restoreTest");
    assert_eq!(row["results"][0]["outcome"], "passed");
    assert_eq!(row["results"][0]["observedRecoveryDurationSeconds"], 1800);
    assert_eq!(row["results"][0]["observedDataLossWindowSeconds"], 60);
    assert!(1800 < 3600 && 60 < 300, "fixture is inside RTO/RPO");
    let evidence = capability_evidence(true, Some(true));
    let expected = json!({
        "planExistence": "satisfied",
        "backupConfiguration": "satisfied",
        "successfulRestore": "demonstrated",
        "exerciseCadence": "current",
        "rtoAchievement": "met",
        "rpoAchievement": "met",
        "unresolvedExerciseFindings": "none",
        "dependencyCoverage": "covered",
        "demonstratedRecovery": true,
        "gaps": []
    });
    assert_eq!(expected["demonstratedRecovery"], true);
    assert_expected_verdict(
        "P20-T02: successful exercise within RTO/RPO",
        &assessment,
        &evidence,
        expected,
    );
}

/// P20-T03: failed restore cannot demonstrate recovery.
#[test]
fn p20_t03_failed_restore_is_not_demonstrated_recovery() {
    let assessment = decode_with_profiles(vec![t03_failed_restore()]);
    let row = require_retained_profile("P20-T03", &assessment);
    assert_eq!(row["results"][0]["outcome"], "failed");
    let evidence = capability_evidence(true, Some(false));
    let expected = json!({
        "successfulRestore": "failed",
        "demonstratedRecovery": false,
        "gaps": [{ "dimension": "successfulRestore" }]
    });
    assert_gap_on("P20-T03", &expected, "successfulRestore");
    assert_expected_verdict("P20-T03: failed restore", &assessment, &evidence, expected);
}

/// P20-T04: stale exercise fails cadence and capability.
#[test]
fn p20_t04_stale_exercise_is_not_demonstrated_recovery() {
    let assessment = decode_with_profiles(vec![t04_stale_exercise()]);
    let row = require_retained_profile("P20-T04", &assessment);
    assert_eq!(row["exercises"][0]["conductedAt"], "2025-01-01T12:00:00Z");
    assert_eq!(row["exerciseCadenceSeconds"], 7_776_000);
    let age = as_of() - Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
    assert!(
        age.num_seconds() > 7_776_000,
        "P20-T04 fixture must be older than cadence at as_of"
    );
    let evidence = capability_evidence(true, Some(true));
    let expected = json!({
        "exerciseCadence": "stale",
        "demonstratedRecovery": false,
        "gaps": [{ "dimension": "exerciseCadence" }]
    });
    assert_gap_on("P20-T04", &expected, "exerciseCadence");
    assert_expected_verdict("P20-T04: stale exercise", &assessment, &evidence, expected);
}

/// P20-T05: critical dependency omitted from the exercise is a coverage gap.
#[test]
fn p20_t05_critical_dependency_not_covered_is_a_gap() {
    let assessment = decode_with_profiles(vec![t05_uncovered_dependency()]);
    let row = require_retained_profile("P20-T05", &assessment);
    let deps = row["dependencies"].as_array().cloned().unwrap_or_default();
    assert!(
        deps.iter()
            .any(|d| d["to"]["vendor"] == "vendor:stripe" && d["critical"] == true),
        "P20-T05: stripe is a critical supplier dependency"
    );
    let scope = row["exercises"][0]["inScopeDependencies"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        !scope
            .iter()
            .any(|s| s.get("vendor") == Some(&json!("vendor:stripe"))),
        "P20-T05: stripe must be omitted from exercise scope"
    );
    let evidence = capability_evidence(true, Some(true));
    let expected = json!({
        "dependencyCoverage": "gap",
        "demonstratedRecovery": false,
        "gaps": [{ "dimension": "dependencyCoverage", "riskRefs": [{ "id": "risk:unproven-recovery" }] }]
    });
    assert_gap_on("P20-T05", &expected, "dependencyCoverage");
    assert_expected_verdict(
        "P20-T05: critical dependency not covered",
        &assessment,
        &evidence,
        expected,
    );
}

/// P20-T06: required backup configuration evidence missing fails closed.
#[test]
fn p20_t06_backup_evidence_missing_fails_closed() {
    let assessment = decode_with_profiles(vec![t06_missing_backup()]);
    let row = require_retained_profile("P20-T06", &assessment);
    assert_eq!(row["backupExpectations"][0]["required"], true);
    assert_eq!(
        row["backupExpectations"][0]["evidenceType"],
        "evidence.backup.configuration"
    );
    let evidence = capability_evidence(false, Some(true));
    assert!(
        evidence.iter().all(
            |env| env.observation().evidence_type().as_str() != "evidence.backup.configuration"
        ),
        "P20-T06 fixture omits backup configuration evidence"
    );
    let expected = json!({
        "backupConfiguration": "missing",
        "demonstratedRecovery": false,
        "gaps": [{ "dimension": "backupConfiguration" }]
    });
    assert_gap_on("P20-T06", &expected, "backupConfiguration");
    assert_expected_verdict(
        "P20-T06: backup evidence missing",
        &assessment,
        &evidence,
        expected,
    );
}

/// P20-T07: tabletop cannot satisfy technical RTO/RPO.
#[test]
fn p20_t07_tabletop_cannot_satisfy_technical_rto_rpo() {
    let assessment = decode_with_profiles(vec![t07_tabletop_only()]);
    let row = require_retained_profile("P20-T07", &assessment);
    assert_eq!(row["exercises"][0]["kind"], "tabletop");
    assert_eq!(row["results"][0]["outcome"], "passed");
    let evidence = capability_evidence(true, None);
    let expected = json!({
        "exerciseCadence": "current",
        "successfulRestore": "notApplicable",
        "rtoAchievement": "notMeasured",
        "rpoAchievement": "notMeasured",
        "demonstratedRecovery": false,
        "gaps": [
            { "dimension": "successfulRestore" },
            { "dimension": "rtoAchievement" },
            { "dimension": "rpoAchievement" }
        ]
    });
    assert_ne!(expected["rtoAchievement"], "met");
    assert_ne!(expected["successfulRestore"], "demonstrated");
    assert_expected_verdict(
        "P20-T07: manual tabletop vs technical recovery test",
        &assessment,
        &evidence,
        expected,
    );
}

/// P20-T08: unresolved exercise remediation blocks demonstrated recovery.
#[test]
fn p20_t08_unresolved_exercise_remediation_blocks_capability() {
    let tracked = decode_with_profiles(vec![t08_open_finding()]);
    let row = require_retained_profile("P20-T08", &tracked);
    assert_eq!(row["results"][0]["issues"][0]["open"], true);
    assert_eq!(
        row["results"][0]["issues"][0]["remediationRefs"][0]["id"],
        "rem.restore-runbook"
    );
    let expected = json!({
        "unresolvedExerciseFindings": "open",
        "demonstratedRecovery": false,
        "gaps": [{
            "dimension": "unresolvedExerciseFindings",
            "remediationRefs": [{ "id": "rem.restore-runbook" }]
        }]
    });
    assert_gap_on("P20-T08", &expected, "unresolvedExerciseFindings");
    let evidence = capability_evidence(true, Some(true));
    assert_expected_verdict(
        "P20-T08: unresolved exercise remediation",
        &tracked,
        &evidence,
        expected,
    );
}

/// Open exercise findings without a RemediationRef fail closed as untracked.
#[test]
fn p20_t08b_untracked_open_finding_fails_closed() {
    let assessment = decode_with_profiles(vec![t08_untracked_finding()]);
    let row = require_retained_profile("P20-T08b", &assessment);
    assert_eq!(row["results"][0]["issues"][0]["open"], true);
    assert!(
        row["results"][0]["issues"][0]["remediationRefs"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(true),
        "P20-T08b: untracked finding has no RemediationRef"
    );
    require_continuity_eval("P20-T08b: untracked exercise finding");
    let profile = assessment
        .continuity_profiles
        .first()
        .expect("P20-T08b: missing continuity profile");
    let err = evaluate_continuity_resilience(
        &assessment,
        profile,
        &capability_evidence(true, Some(true)),
        as_of(),
    )
    .expect_err(
        "P20-T08b: evaluate_continuity_resilience must fail closed on untracked exercise finding",
    );
    let text = err.to_string().to_ascii_lowercase();
    assert!(
        text.contains("untracked"),
        "P20-T08b: error must mention untracked exercise finding, got {text}"
    );
}

/// P20-T09: plan document / procedure_present / current BCP never proves capability.
#[test]
fn p20_t09_plan_document_never_proves_recovery_capability() {
    let assessment = decode_with_profiles(vec![t01_plan_no_exercise()]);
    let _ = require_retained_profile("P20-T09", &assessment);

    let spec = read_repo_file("docs/specs/continuity-resilience.md");
    assert!(
        spec.contains("plan_existence is not an input to `demonstrated_recovery`")
            || spec.contains("**excludes** plan existence")
            || spec.contains("demonstrated_recovery` is derived and **excludes**"),
        "SSOT must keep plan_existence out of demonstrated_recovery"
    );

    let mut set = EvidenceSet::new();
    set.insert(seal(
        "inventory.subject",
        "org:weeping",
        &[
            ("id", EvidenceValue::String("org:weeping".into())),
            ("kind", EvidenceValue::String("organization".into())),
        ],
    ));
    set.insert(seal(
        "inventory.complete",
        "org:weeping",
        &[
            ("kind", EvidenceValue::String("organization".into())),
            ("authoritative", EvidenceValue::Bool(true)),
        ],
    ));
    set.insert(seal(
        "evidence.resilience.recovery-plan",
        "org:weeping",
        &[("procedure_present", EvidenceValue::Bool(true))],
    ));
    let org = SubjectSelector {
        kind: Some("organization".into()),
        id: None,
    };
    let procedure = evaluate(
        &CompiledControlTest::builder()
            .id(weeping_angel_assurance_ir::ControlTestId::new(
                "test.resilience.recovery-procedure-present",
            ))
            .control_id(weeping_angel_assurance_ir::ControlId::new(
                "control.resilience.recovery-procedure",
            ))
            .kind(ControlTestKind::Automated)
            .expr(TestExpr::AllSubjects {
                selector: org.clone(),
                evidence: EvidenceSelector {
                    evidence_type: EvidenceType::new("evidence.resilience.recovery-plan"),
                    subject_selector: org.clone(),
                    field: Some("procedure_present".into()),
                    freshness: None,
                },
            })
            .build(),
        &set,
        &as_of_ctx(),
    );
    assert_eq!(
        procedure.effectiveness,
        Effectiveness::Effective,
        "P20-T09: catalog procedure_present may still be Effective"
    );

    let mut plan_set = EvidenceSet::new();
    plan_set.insert(seal(
        "evidence.resilience.continuity-plan",
        "org:weeping",
        &[
            (
                "reviewed_at",
                EvidenceValue::Timestamp(as_of() - chrono::Duration::days(30)),
            ),
            ("plan_kind", EvidenceValue::String("bcp".into())),
        ],
    ));
    let plan = evaluate(
        &CompiledControlTest::builder()
            .id(weeping_angel_assurance_ir::ControlTestId::new(
                "test.resilience.continuity-plan-current",
            ))
            .control_id(weeping_angel_assurance_ir::ControlId::new(
                "control.resilience.business-continuity-plan",
            ))
            .kind(ControlTestKind::Automated)
            .expr(TestExpr::FreshWithin {
                selector: EvidenceSelector {
                    evidence_type: EvidenceType::new("evidence.resilience.continuity-plan"),
                    subject_selector: org,
                    field: Some("reviewed_at".into()),
                    freshness: None,
                },
                duration: Duration::from_secs(365 * 24 * 3600),
            })
            .build(),
        &plan_set,
        &as_of_ctx(),
    );
    assert_eq!(
        plan.effectiveness,
        Effectiveness::Effective,
        "P20-T09: catalog continuity-plan-current may still be Effective"
    );

    let doc = DocumentRef::new("doc:bcp-payments");
    assert_eq!(doc.id, "doc:bcp-payments");
    let _ = DocumentKind::Policy;

    let expected = json!({
        "planExistence": "satisfied",
        "demonstratedRecovery": false
    });
    assert_expected_verdict(
        "P20-T09: plan document never proves recovery capability",
        &assessment,
        &capability_evidence(true, None),
        expected,
    );
}

/// P20-T10: collision fence — do not retarget landed catalog IDs.
#[test]
fn p20_t10_catalog_plan_presence_semantics_unchanged() {
    let proc = read_repo_file("catalog/canonical/v1/tests/resilience.toml");
    assert!(
        proc.contains("id = \"test.resilience.recovery-procedure-present\"")
            && proc.contains("field = \"procedure_present\""),
        "P20-T10: do not rewrite recovery-procedure-present into a restore test"
    );
    let gov = read_repo_file("catalog/canonical/v1/tests/governance.toml");
    assert!(
        gov.contains("id = \"test.resilience.continuity-plan-current\"")
            && gov.contains("fresh-within")
            && gov.contains("reviewed_at"),
        "P20-T10: do not rewrite continuity-plan-current into a capability test"
    );
    forbid_needles(
        "P20-T10: no backup-vendor types in continuity IR",
        &product_crate_sources_joined(),
        &[
            "AwsBackup",
            "AzureSiteRecovery",
            "struct Veeam",
            "pub struct BusinessService",
        ],
    );
}

/// P20-T12: business service is AssetKind::Service, not a parallel inventory.
#[test]
fn p20_t12_business_service_is_asset_kind_service() {
    let service = Asset::new(
        AssetId::new("asset:payments"),
        AssetKind::Service,
        "payments",
    );
    assert!(matches!(service.kind, AssetKind::Service));
    let encoded = serde_json::to_value(&service).unwrap();
    assert_eq!(encoded["kind"], "service");
    assert!(encoded.get("criticality").is_none());
    assert!(encoded.get("rtoSeconds").is_none());

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "P20-T12: do not invent a parallel BusinessService inventory",
        &ir,
        &["pub struct BusinessService"],
    );
    require_needles(
        "P20-T12: AssetKind::Service is the business service",
        &ir,
        &["struct ContinuityResilienceProfile", "service: AssetId"],
    );
    let assessment = decode_with_profiles(vec![t01_plan_no_exercise()]);
    let row = require_retained_profile("P20-T12", &assessment);
    assert_eq!(row["service"], "asset:payments");
}

/// P20-T13: capability gaps are ContinuityGap records with remediation refs on open findings.
#[test]
fn p20_t13_gaps_surface_as_risk_and_remediation_refs() {
    let assessment = decode_with_profiles(vec![t08_open_finding()]);
    let row = require_retained_profile("P20-T13", &assessment);
    assert_eq!(
        row["results"][0]["issues"][0]["remediationRefs"][0]["id"],
        "rem.restore-runbook"
    );
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "P20-T13: ContinuityGap + refs",
        &ir,
        &[
            "struct ContinuityGap",
            "risk_refs",
            "remediation_refs",
            "untracked",
        ],
    );
    let expected = json!({
        "unresolvedExerciseFindings": "open",
        "demonstratedRecovery": false,
        "gaps": [{
            "dimension": "unresolvedExerciseFindings",
            "riskRefs": [{ "id": "risk:unproven-recovery" }],
            "remediationRefs": [{ "id": "rem.restore-runbook" }]
        }]
    });
    assert_gap_on("P20-T13", &expected, "unresolvedExerciseFindings");
    assert_expected_verdict(
        "P20-T13: ContinuityGap + remediation refs",
        &assessment,
        &capability_evidence(true, Some(true)),
        expected,
    );
}

/// P20-T14: DocumentRef is opaque; existence is not capability.
#[test]
fn p20_t14_document_ref_is_opaque_not_capability() {
    let assessment = decode_with_profiles(vec![t01_plan_no_exercise()]);
    let row = require_retained_profile("P20-T14", &assessment);
    assert_eq!(row["procedures"][0]["document"]["id"], "doc:bcp-payments");
    assert_eq!(row["procedures"][0]["document"]["kind"], "plan");
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "P20-T14: reuse CIR DocumentRef; add Plan/Runbook kinds",
        &ir,
        &["pub struct DocumentRef", "Plan", "Runbook"],
    );
    forbid_needles(
        "P20-T14: do not land Prompt 12 registry here",
        &fs::read_to_string(crate_src("weeping-angel-assurance-ir").join("continuity.rs")).unwrap(),
        &["pub struct ControlledDocument"],
    );
    let expected = json!({
        "planExistence": "satisfied",
        "demonstratedRecovery": false
    });
    assert_expected_verdict(
        "P20-T14: opaque DocumentRef",
        &assessment,
        &capability_evidence(true, None),
        expected,
    );
}

/// P20-T15: schema remains assurance-ir/v1.
#[test]
fn p20_t15_schema_stays_assurance_ir_v1() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    let ir_lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        ir_lib.contains("assurance-ir/v1"),
        "P20-T15: do not fork ASSURANCE_IR_SCHEMA"
    );
    let golden = read_repo_file("tests/fixtures/assurance-ir/v1/risk.json");
    let risk: Risk = serde_json::from_str(&golden).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
}

/// P20-T16: evidence crate stays conclusion-free.
#[test]
fn p20_t16_evidence_crate_has_no_demonstrated_recovery_conclusion() {
    let evidence = crate_sources_joined("weeping-angel-evidence");
    assert!(
        !evidence.contains("demonstrated_recovery") && !evidence.contains("DemonstratedRecovery"),
        "P20-T16: do not put demonstrated recovery on evidence envelopes"
    );
    let collector = crate_sources_joined("weeping-angel-collector");
    forbid_needles(
        "P20-T16: collectors stay conclusion-free",
        &collector,
        &[
            "demonstrated_recovery",
            "DemonstratedRecovery",
            "evaluate_continuity_resilience",
        ],
    );
    let wb = read_repo_file("src/workbench/remediation.rs");
    assert!(
        wb.contains("pub struct RemediationRequest"),
        "P20-T16: scanner workbench remediation stays a patch helper"
    );
    assert!(
        !crate_sources_joined("weeping-angel-assurance-ir").contains("workbench::remediation"),
        "IR must not import scanner workbench remediation"
    );
}

/// Additive validation: rto_seconds == 0 fails closed.
#[test]
fn p20_t_rto_zero_fails_validate() {
    let mut profile = t01_plan_no_exercise();
    profile["objectives"][0]["rtoSeconds"] = json!(0);
    let assessment = decode_with_profiles(vec![profile]);
    let _ = require_retained_profile("P20-T-rto-zero", &assessment);
    let err = assessment
        .validate()
        .expect_err("rto_seconds == 0 must fail closed");
    let text = err.to_string().to_ascii_lowercase();
    assert!(
        text.contains("rto"),
        "validation error must mention rto, got {text}"
    );
}

/// MissionCritical / High without cadence fails closed.
#[test]
fn p20_t_mission_critical_requires_cadence() {
    let mut profile = t01_plan_no_exercise();
    profile
        .as_object_mut()
        .unwrap()
        .remove("exerciseCadenceSeconds");
    let assessment = decode_with_profiles(vec![profile]);
    let _ = require_retained_profile("P20-T-cadence", &assessment);
    let err = assessment
        .validate()
        .expect_err("MissionCritical without cadence must fail closed");
    let text = err.to_string().to_ascii_lowercase();
    assert!(
        text.contains("exercise cadence") || text.contains("cadence"),
        "validation error must mention exercise cadence, got {text}"
    );
}

/// Profile service must be AssetKind::Service.
#[test]
fn p20_t_profile_service_must_be_asset_kind_service() {
    let mut profile = t01_plan_no_exercise();
    profile["service"] = json!("asset:payments-db");
    let assessment = decode_with_profiles(vec![profile]);
    let _ = require_retained_profile("P20-T-service-kind", &assessment);
    let err = assessment
        .validate()
        .expect_err("non-Service continuity subject must fail closed");
    let text = err.to_string().to_ascii_lowercase();
    assert!(
        text.contains("continuity service") || text.contains("service"),
        "validation error must mention continuity service, got {text}"
    );
}
