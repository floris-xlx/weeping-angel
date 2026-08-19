//! SUPERSEDED by `sdd_continuity_resilience_target`.
//!
//! Historical characterization of SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`
//! (`docs/specs/continuity-resilience.md` §3): `AssetKind::Service` existed
//! without criticality/RTO/RPO; IR had no continuity domain types; `Risk`
//! was a four-field stub; Prompt 12/16 types were absent; catalog
//! plan-presence / freshness IDs could pass without demonstrated restore.
//!
//! Target `sdd_continuity_resilience_target` is the source of truth. This
//! baseline is skipped (`#[ignore = "superseded by target suite"]`). Dual-suite
//! registration remains. Does not implement continuity evaluation.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind, ControlId, ControlTestId, Risk,
    RiskId, RiskStatus, ValidateIr,
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

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn ir_src() -> String {
    crate_sources_joined("weeping-angel-assurance-ir")
}

fn as_of() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 19, 12, 30, 0).unwrap(),
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
            collector_id: "fixture.continuity-resilience-baseline".into(),
            collected_at: as_of().now - chrono::Duration::hours(1),
            scope: "baseline".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn compiled(test_id: &str, control_id: &str, expr: TestExpr) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new(test_id))
        .control_id(ControlId::new(control_id))
        .kind(ControlTestKind::Automated)
        .expr(expr)
        .build()
}

fn org_selector() -> SubjectSelector {
    SubjectSelector {
        kind: Some("organization".into()),
        id: None,
    }
}

fn inventory_org(set: &mut EvidenceSet) {
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
}

const CONTINUITY_ABSENT_NEEDLES: &[&str] = &[
    "struct RecoveryObjective",
    "struct ContinuityResilienceProfile",
    "struct ContinuityResilienceVerdict",
    "struct ServiceDependency",
    "enum ServiceCriticality",
    "struct BackupExpectation",
    "struct ContinuityExercise",
    "struct ExerciseResult",
    "fn evaluate_continuity_resilience",
    "observed_recovery_duration",
    "observed_data_loss",
    "pub struct DocumentRef",
    "pub struct ControlledDocument",
];

/// P20-B01: AssetKind::Service exists; Asset::new JSON has no continuity fields.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b01_service_asset_has_no_criticality_or_objectives() {
    let _ = AssetKind::Service;
    let asset = Asset::new(AssetId::new("svc:checkout"), AssetKind::Service, "checkout");
    assert_eq!(asset.kind, AssetKind::Service);
    assert_eq!(asset.id.as_str(), "svc:checkout");

    let json = serde_json::to_value(&asset).unwrap();
    assert_eq!(json["kind"], "service");
    for absent in [
        "criticality",
        "rto",
        "rpo",
        "rtoSeconds",
        "rpoSeconds",
        "dependencies",
        "backupExpectation",
        "exercises",
    ] {
        assert!(
            json.get(absent).is_none(),
            "found-case Asset JSON must not contain `{absent}`: {json}"
        );
    }
}

/// P20-B02: IR sources have no continuity domain types.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b02_ir_has_no_continuity_domain_types() {
    let src = ir_src();
    for needle in CONTINUITY_ABSENT_NEEDLES {
        assert!(
            !src.contains(needle),
            "IR must not declare `{needle}` on characterization HEAD"
        );
    }
    assert!(
        !crate_src("weeping-angel-assurance-ir")
            .join("continuity.rs")
            .is_file(),
        "continuity.rs must not exist on characterization HEAD"
    );
    let ids = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    for needle in [
        "typed_id!(RecoveryObjectiveId)",
        "typed_id!(ContinuityExerciseId)",
        "typed_id!(ContinuityProfileId)",
    ] {
        assert!(
            !ids.contains(needle),
            "id.rs must not declare `{needle}` on characterization HEAD"
        );
    }
}

/// P20-B03: Risk remains the four-field stub.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b03_risk_is_still_a_four_field_stub() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        risk_src.contains("Minimal risk record. Not a risk engine."),
        "risk.rs must keep the found-case module comment"
    );

    let risk = Risk::new(
        RiskId::new("risk:dr-unproven"),
        "unproven recovery",
        "plan exists without restore",
    );
    assert_eq!(risk.status, RiskStatus::Open);
    let json = serde_json::to_value(&risk).unwrap();
    let obj = json.as_object().expect("Risk serializes as an object");
    assert_eq!(obj.keys().count(), 4, "found-case Risk JSON is four keys");
    for absent in ["rto", "rpo", "exercise", "remediationRefs", "continuity"] {
        assert!(obj.get(absent).is_none(), "Risk JSON must omit `{absent}`");
    }
}

/// P20-B04: Prompt 12 / 16 product types are absent from IR.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b04_document_and_remediation_engines_are_not_landed() {
    let src = ir_src();
    assert!(
        !src.contains("pub struct DocumentRef") && !src.contains("pub struct ControlledDocument"),
        "Prompt 12 document types must be absent from IR on this HEAD"
    );
    assert!(
        !src.contains("pub struct Remediation {")
            && !src.contains("pub struct Remediation\n")
            && !src.contains("pub struct Remediation\r"),
        "Prompt 16 Remediation workflow type must be absent from IR on this HEAD"
    );
    let impl_src = read_repo_file("crates/weeping-angel-assurance-ir/src/implementation.rs");
    assert!(
        !impl_src.contains("document_refs") && !impl_src.contains("DocumentRef"),
        "CIR DocumentRef is specified but not landed on implementation.rs"
    );
}

/// P20-B05: Catalog resilience / backup / governance IDs exist.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b05_catalog_plan_presence_ids_exist() {
    let resilience = read_repo_file("catalog/canonical/v1/controls/resilience.toml");
    let gov = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    for id in [
        "control.resilience.recovery-procedure",
        "control.resilience.disaster-recovery-exercise",
        "control.resilience.recovery-objectives",
        "control.resilience.recovery-evidence-freshness",
    ] {
        assert!(resilience.contains(id), "missing operational control {id}");
    }
    for id in [
        "control.resilience.business-continuity-plan",
        "control.resilience.disaster-recovery-governance",
    ] {
        assert!(gov.contains(id), "missing governance control {id}");
    }
    let backup = read_repo_file("catalog/canonical/v1/controls/backup.toml");
    assert!(backup.contains("control.backup.restore-testing"));
}

/// P20-B06: DR exercise and RTO/RPO catalog tests are manual-review.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b06_dr_exercise_and_objectives_are_manual_review() {
    let tests = read_repo_file("catalog/canonical/v1/tests/resilience.toml");
    for id in [
        "test.resilience.dr-exercise-recorded",
        "test.resilience.recovery-objectives-documented",
    ] {
        let idx = tests
            .find(id)
            .unwrap_or_else(|| panic!("missing catalog test {id}"));
        let window = &tests[idx..idx.saturating_add(280).min(tests.len())];
        assert!(
            window.contains("manual-review"),
            "{id} must be manual-review on characterization HEAD; window={window}"
        );
    }

    let mut set = EvidenceSet::new();
    inventory_org(&mut set);
    set.insert(seal(
        "evidence.resilience.recovery-plan",
        "org:weeping",
        &[
            ("procedure_present", EvidenceValue::Bool(true)),
            ("objectives_documented", EvidenceValue::Bool(true)),
        ],
    ));
    for (test_id, control_id) in [
        (
            "test.resilience.dr-exercise-recorded",
            "control.resilience.disaster-recovery-exercise",
        ),
        (
            "test.resilience.recovery-objectives-documented",
            "control.resilience.recovery-objectives",
        ),
    ] {
        let result = evaluate(
            &compiled(test_id, control_id, TestExpr::ManualReview),
            &set,
            &as_of(),
        );
        assert_eq!(
            result.effectiveness,
            Effectiveness::ManualReviewRequired,
            "found case: {test_id} never auto-concludes RTO/RPO or exercise achievement"
        );
    }
}

/// P20-B07: recovery-procedure-present is procedure_present only.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b07_procedure_present_has_no_restore_predicate() {
    let tests = read_repo_file("catalog/canonical/v1/tests/resilience.toml");
    let idx = tests
        .find("test.resilience.recovery-procedure-present")
        .expect("recovery-procedure-present");
    let window = &tests[idx..idx.saturating_add(350).min(tests.len())];
    assert!(
        window.contains("field = \"procedure_present\""),
        "found case: procedure_present is the only predicate"
    );
    assert!(
        !window.contains("restore") && !window.contains("rto") && !window.contains("rpo"),
        "procedure-present test must not encode restore/RTO/RPO on this HEAD"
    );

    let mut set = EvidenceSet::new();
    inventory_org(&mut set);
    set.insert(seal(
        "evidence.resilience.recovery-plan",
        "org:weeping",
        &[("procedure_present", EvidenceValue::Bool(true))],
    ));
    assert!(
        set.iter()
            .all(|env| env.observation().evidence_type().as_str() != "evidence.backup.restore-test"),
        "found case must omit restore-test envelopes"
    );
    let selector = org_selector();
    let result = evaluate(
        &compiled(
            "test.resilience.recovery-procedure-present",
            "control.resilience.recovery-procedure",
            TestExpr::AllSubjects {
                selector: selector.clone(),
                evidence: EvidenceSelector {
                    evidence_type: EvidenceType::new("evidence.resilience.recovery-plan"),
                    subject_selector: selector,
                    field: Some("procedure_present".into()),
                    freshness: None,
                },
            },
        ),
        &set,
        &as_of(),
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::Effective,
        "found case: procedure_present=true is Effective with no restore-test; got {:?} {}",
        result.effectiveness,
        result.rationale
    );
}

/// P20-B08: continuity-plan-current is freshness on reviewed_at only.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b08_continuity_plan_current_is_freshness_only() {
    let tests = read_repo_file("catalog/canonical/v1/tests/governance.toml");
    let idx = tests
        .find("test.resilience.continuity-plan-current")
        .expect("continuity-plan-current");
    let window = &tests[idx..idx.saturating_add(400).min(tests.len())];
    assert!(
        window.contains("fresh-within") && window.contains("reviewed_at"),
        "found case: continuity-plan-current is reviewed_at freshness"
    );
    assert!(
        !window.contains("restore") && !window.contains("rto_seconds"),
        "continuity-plan-current must not encode demonstrated restore"
    );

    let mut set = EvidenceSet::new();
    set.insert(seal(
        "evidence.resilience.continuity-plan",
        "org:weeping",
        &[
            (
                "reviewed_at",
                EvidenceValue::Timestamp(as_of().now - chrono::Duration::days(30)),
            ),
            ("plan_kind", EvidenceValue::String("bcp".into())),
        ],
    ));
    let result = evaluate(
        &compiled(
            "test.resilience.continuity-plan-current",
            "control.resilience.business-continuity-plan",
            TestExpr::FreshWithin {
                selector: EvidenceSelector {
                    evidence_type: EvidenceType::new("evidence.resilience.continuity-plan"),
                    subject_selector: org_selector(),
                    field: Some("reviewed_at".into()),
                    freshness: None,
                },
                duration: Duration::from_secs(365 * 24 * 3600),
            },
        ),
        &set,
        &as_of(),
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::Effective,
        "found case: continuity-plan-current passes on freshness with no restore; got {:?} {}",
        result.effectiveness,
        result.rationale
    );
}

/// P20-B09: AssessmentDefinition has no continuity inventory.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b09_assessment_has_no_continuity_collection() {
    let assessment_src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    assert!(
        !assessment_src.contains("continuity")
            && !assessment_src.contains("recovery_objective")
            && !assessment_src.contains("continuity_profiles"),
        "AssessmentDefinition must not carry a continuity inventory today"
    );

    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.continuity.baseline"));
    assessment.validate().unwrap();
    let json = serde_json::to_value(&assessment).unwrap();
    assert!(json.get("continuityProfiles").is_none());
    assert!(json.get("recoveryObjectives").is_none());
}

/// P20-B10: workbench remediation is a scanner patch helper, not IR.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b10_workbench_remediation_is_not_isms() {
    let wb = read_repo_file("src/workbench/remediation.rs");
    assert!(
        wb.contains("pub struct RemediationRequest") && wb.contains("pub struct RemediationResult"),
        "scanner remediation helper remains in src/workbench/remediation.rs"
    );
    assert!(
        !ir_src().contains("workbench::remediation"),
        "IR must not import scanner workbench remediation"
    );
}

/// P20-B11: dual-suite is registered.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b11_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_continuity_resilience_baseline")
            && toml.contains("sdd_continuity_resilience_target")
            && toml.contains("tests/contracts/continuity_resilience.baseline.rs")
            && toml.contains("tests/contracts/continuity_resilience.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
}

/// P20-B12: collision fence — catalog backup/resilience IDs remain as files.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b12_collision_fence_catalog_files_present() {
    for rel in [
        "catalog/canonical/v1/controls/backup.toml",
        "catalog/canonical/v1/evidence/backup.toml",
        "catalog/canonical/v1/tests/backup.toml",
        "catalog/canonical/v1/controls/resilience.toml",
        "catalog/canonical/v1/evidence/resilience.toml",
        "catalog/canonical/v1/tests/resilience.toml",
    ] {
        assert!(
            manifest_dir().join(rel).is_file(),
            "collision fence: leave catalog file in place ({rel})"
        );
    }
}

/// Found case: golden risk fixture still decodes beside this slice.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b_golden_risk_still_decodes() {
    let raw = read_repo_file("tests/fixtures/assurance-ir/v1/risk.json");
    let risk: Risk = serde_json::from_str(&raw).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    let fixture: Value = serde_json::from_str(&raw).unwrap();
    assert!(fixture.get("rtoSeconds").is_none());
}

/// Found case: assurance crate has no continuity evaluation module.
#[ignore = "superseded by target suite"]
#[test]
fn p20_b_assurance_has_no_continuity_module() {
    let assurance = crate_src("weeping-angel-assurance");
    assert!(
        !assurance.join("continuity.rs").is_file(),
        "weeping-angel-assurance/src/continuity.rs must be absent today"
    );
    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !lib.contains("mod continuity") && !lib.contains("evaluate_continuity_resilience"),
        "assurance facade must not export continuity evaluation on this HEAD"
    );
}
