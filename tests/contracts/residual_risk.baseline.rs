//! Baseline suite for Operational ISMS v1 residual risk (Prompt 09).
//!
//! Characterization of CURRENT tree (`docs/specs/residual-risk.md` §3) on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`: `Risk` is a four-field inventory
//! stub (*“Minimal risk record. Not a risk engine.”*). There is no residual-risk
//! projection, mode, methodology version, treatment-plan version, inherent-risk
//! snapshot pin, or control-effectiveness reduction. Canonical
//! `ControlTestResult` / `Effectiveness` already exist and are unused by risk.
//!
//! Target `sdd_residual_risk_target` is the source of truth. This baseline
//! is skipped (`#[ignore = "superseded by target suite"]`).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, Exception, ExceptionId, ExceptionStatus, IdentityId, PrincipalRef, Risk,
    RiskId, RiskStatus,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};

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

fn product_crate_sources_joined() -> String {
    let crates_dir = manifest_dir().join("crates");
    let entries = fs::read_dir(&crates_dir).unwrap_or_else(|e| {
        panic!("read {}: {e}", crates_dir.display());
    });
    let mut chunks = Vec::new();
    for entry in entries {
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

fn ir_fixture(name: &str) -> PathBuf {
    manifest_dir()
        .join("tests/fixtures/assurance-ir/v1")
        .join(name)
}

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn sample_risk() -> Risk {
    Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    )
}

fn sample_test_result(effectiveness: Effectiveness) -> ControlTestResult {
    ControlTestResult {
        test_id: weeping_angel_assurance_ir::ControlTestId::new("test.access.mfa"),
        control_id: weeping_angel_assurance_ir::ControlId::new("control.access.mfa"),
        effectiveness,
        rationale: "found-case control-test result is unused by Risk".into(),
        evidence_refs: Vec::new(),
        missing_evidence: Vec::new(),
        checked_at: chrono::Utc::now(),
        test_version: "1".into(),
        input_digest: "sha256:unused-by-risk".into(),
        duration: None,
        status: None,
        reason: None,
        population: None,
        period: None,
    }
}

/// P09-B01: `risk.rs` module docs still contain `Minimal risk record. Not a risk engine.`
#[ignore = "superseded by target suite"]
#[test]
fn p09_b01_risk_module_is_not_a_risk_engine() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        src.contains("//! Minimal risk record. Not a risk engine."),
        "risk.rs must keep the found-case module comment"
    );
    assert!(
        !src.contains("residual"),
        "risk.rs must not mention residual on characterization HEAD"
    );
}

/// P09-B02: `Risk::new` JSON has no residual / mode / methodology / treatment / projection fields.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b02_risk_new_json_has_no_residual_lineage_fields() {
    let risk = sample_risk();
    assert_eq!(risk.status, RiskStatus::Open);

    let json = serde_json::to_value(&risk).unwrap();
    let obj = json.as_object().expect("Risk serializes as an object");
    let mut keys = json_object_keys(&json);
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "description".to_string(),
            "id".to_string(),
            "status".to_string(),
            "title".to_string()
        ]
    );
    assert_eq!(obj.keys().count(), 4);

    for absent in [
        "residual",
        "residualScore",
        "residualRating",
        "mode",
        "residualMode",
        "methodology",
        "methodologyId",
        "methodologyVersion",
        "treatment",
        "treatmentId",
        "treatmentPlanVersion",
        "inherent",
        "inherentRiskVersion",
        "projection",
        "projectedAt",
        "controlTests",
        "reductionTrace",
    ] {
        assert!(
            obj.get(absent).is_none(),
            "found-case Risk JSON must not contain `{absent}`"
        );
    }
}

/// P09-B03: Golden `risk.json` still decodes; `id == "risk:source-tamper"`.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b03_golden_risk_json_decodes() {
    let raw = fs::read_to_string(ir_fixture("risk.json")).unwrap();
    let risk: Risk = serde_json::from_str(&raw).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.title, "Source tampering");
    assert_eq!(
        risk.description,
        "Unauthorized change to the source of record."
    );
    assert_eq!(risk.status, RiskStatus::Open);

    let value: Value = serde_json::from_str(&raw).unwrap();
    let mut keys = json_object_keys(&value);
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "description".to_string(),
            "id".to_string(),
            "status".to_string(),
            "title".to_string()
        ]
    );
}

/// P09-B04: Product crate sources have no residual-risk projection types or API.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b04_product_sources_have_no_residual_projection_api() {
    let src = product_crate_sources_joined();
    for needle in [
        "ResidualRiskProjection",
        "ResidualRiskMode",
        "ResidualRiskRequest",
        "ResidualRiskError",
        "fn project_residual_risk",
        "fn query_residual_risk",
        "InherentRiskRef",
        "TreatmentPlanRef",
        "ControlTestSnapshotRef",
        "ManualResidualAssessment",
        "residual-methodology:no-reduction",
        "residual-methodology:control-effectiveness",
    ] {
        assert!(
            !src.contains(needle),
            "product crate sources must not yet expose `{needle}`"
        );
    }
}

/// P09-B05: `lib.rs` re-exports `Risk` / `RiskStatus` only (no residual types).
#[ignore = "superseded by target suite"]
#[test]
fn p09_b05_ir_lib_reexports_only_the_minimal_risk_record() {
    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        lib.contains("pub use risk::{Risk, RiskStatus};"),
        "IR lib.rs must re-export Risk / RiskStatus"
    );
    assert!(
        lib.contains("pub mod risk;"),
        "IR crate still has a risk module"
    );
    assert!(
        !lib.contains("pub mod residual"),
        "IR lib.rs must not declare a residual module"
    );
    for needle in [
        "ResidualRiskProjection",
        "ResidualRiskMode",
        "InherentRiskRef",
        "TreatmentPlanRef",
        "MethodologyRef",
        "project_residual_risk",
    ] {
        assert!(
            !lib.contains(needle),
            "IR lib.rs must not export `{needle}` on characterization HEAD"
        );
    }
}

/// P09-B06: `Effectiveness` still declares the canonical variants (reuse lock).
#[ignore = "superseded by target suite"]
#[test]
fn p09_b06_effectiveness_enum_is_the_canonical_control_test_contract() {
    let src = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    for variant in [
        "Effective",
        "Ineffective",
        "PartiallyEffective",
        "NotApplicable",
        "NotTested",
        "InsufficientEvidence",
        "StaleEvidence",
        "ManualReviewRequired",
        "ExceptionApproved",
        "Inconclusive",
    ] {
        assert!(
            src.contains(variant),
            "Effectiveness must still declare `{variant}`"
        );
    }

    let _ = Effectiveness::Effective;
    let _ = Effectiveness::Ineffective;
    let _ = Effectiveness::PartiallyEffective;
    let _ = Effectiveness::NotApplicable;
    let _ = Effectiveness::NotTested;
    let _ = Effectiveness::InsufficientEvidence;
    let _ = Effectiveness::StaleEvidence;
    let _ = Effectiveness::ManualReviewRequired;
    let _ = Effectiveness::ExceptionApproved;
    let _ = Effectiveness::Inconclusive;

    let encoded = serde_json::to_string(&Effectiveness::ExceptionApproved).unwrap();
    assert_eq!(encoded, "\"exceptionApproved\"");
}

/// P09-B07: No residual reduction mapping in `risk.rs`; assurance residual module absent.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b07_no_residual_reduction_mapping_or_module() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(!risk_src.contains("Effectiveness"));
    assert!(!risk_src.contains("ControlTestResult"));
    assert!(!risk_src.contains("reduction"));

    let residual_ir = manifest_dir().join("crates/weeping-angel-assurance-ir/src/residual.rs");
    assert!(
        !residual_ir.exists(),
        "assurance-ir residual.rs must be absent on characterization HEAD"
    );
    let residual_assurance = manifest_dir().join("crates/weeping-angel-assurance/src/residual.rs");
    assert!(
        !residual_assurance.exists(),
        "assurance residual.rs must be absent on characterization HEAD"
    );

    let assurance_lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !assurance_lib.contains("pub mod residual"),
        "assurance crate must not declare a residual module"
    );
}

/// P09-B08: Collision fence — GitHub collector / ISO remap paths stay out of this slice.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b08_collision_fence_github_collector_and_iso_remap() {
    assert!(
        manifest_dir()
            .join("crates/weeping-angel-collector/src/github")
            .is_dir(),
        "GitHub collector crate path remains a collision fence"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/iso27001_remap.baseline.rs")
            .is_file()
            && manifest_dir()
                .join("tests/contracts/iso27001_remap.target.rs")
                .is_file(),
        "ISO remap dual-suite remains a collision fence"
    );

    let collector = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "ResidualRiskProjection",
        "project_residual_risk",
        "ResidualRiskMode",
    ] {
        assert!(
            !collector.contains(needle),
            "collector sources must not contain `{needle}`"
        );
    }
}

/// Found case: Effective control-test results do not change `Risk` (no mapping to zero residual).
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_effective_control_does_not_project_residual() {
    let before = serde_json::to_value(&sample_risk()).unwrap();
    let result = sample_test_result(Effectiveness::Effective);
    assert_eq!(result.effectiveness, Effectiveness::Effective);

    let after = serde_json::to_value(&sample_risk()).unwrap();
    assert_eq!(before, after);
    assert!(after.get("residualScore").is_none());
    assert!(after.get("residualRating").is_none());
}

/// Found case: Ineffective / missing control-test results are unused by `Risk`.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_ineffective_and_missing_controls_do_not_project() {
    let ineffective = sample_test_result(Effectiveness::Ineffective);
    assert_eq!(ineffective.effectiveness, Effectiveness::Ineffective);

    let risk = sample_risk();
    let json = serde_json::to_value(&risk).unwrap();
    assert!(json.get("controlIds").is_none());
    assert!(json.get("missingControls").is_none());
}

/// Found case: partial treatment is not a Risk / residual field.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_partial_treatment_is_absent_from_risk() {
    let json = serde_json::to_value(&sample_risk()).unwrap();
    for absent in [
        "treatmentCompleteness",
        "partialTreatment",
        "treatmentPlanVersion",
    ] {
        assert!(json.get(absent).is_none());
    }
    let src = product_crate_sources_joined();
    assert!(
        !src.contains("struct TreatmentPlanRef"),
        "TreatmentPlanRef must be absent (Prompt 08 engine is not consumed here)"
    );
}

/// Found case: no Assessed residual mode / accountable manual residual evidence.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_manual_residual_assessment_is_absent() {
    let src = product_crate_sources_joined();
    for needle in [
        "ManualResidualAssessment",
        "ResidualRiskMode",
        "enum ResidualRiskMode",
    ] {
        assert!(!src.contains(needle), "must not contain `{needle}`");
    }
    let json = serde_json::to_value(&sample_risk()).unwrap();
    assert!(json.get("assessedBy").is_none());
    assert!(json.get("residualRationale").is_none());
}

/// Found case: historical residual query does not exist; Risk overwrites in place.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_historical_residual_is_not_queryable() {
    let mut risk = sample_risk();
    risk.title = "revised in place".into();
    assert_eq!(risk.title, "revised in place");
    let json = serde_json::to_value(&risk).unwrap();
    assert!(json.get("history").is_none());
    assert!(json.get("projections").is_none());

    let src = product_crate_sources_joined();
    assert!(!src.contains("fn query_residual_risk"));
    assert!(!src.contains("ResidualRiskProjection"));
}

/// Found case: `StaleEvidence` exists on control-test and is unused by risk.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_stale_evidence_is_unused_by_risk() {
    let result = sample_test_result(Effectiveness::StaleEvidence);
    assert_eq!(result.effectiveness, Effectiveness::StaleEvidence);
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(!risk_src.contains("StaleEvidence"));
    assert!(!risk_src.contains("stale"));
}

/// Found case: multiple controls do not compose into residual.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_multiple_controls_do_not_compose_residual() {
    let a = sample_test_result(Effectiveness::Effective);
    let mut b = sample_test_result(Effectiveness::PartiallyEffective);
    b.control_id = weeping_angel_assurance_ir::ControlId::new("control.access.logging");
    let _ = [a, b];
    let json = serde_json::to_value(&sample_risk()).unwrap();
    assert!(json.get("controlIds").is_none());
    assert!(json.get("reductionTrace").is_none());
}

/// Found case: no-reduction methodology id is absent.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_no_reduction_methodology_is_absent() {
    let src = product_crate_sources_joined();
    assert!(!src.contains("residual-methodology:no-reduction"));
    assert!(!src.contains("residual-methodology:control-effectiveness"));
}

/// Found case: approved exception is governance evidence and does not set residual Low.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_approved_exception_does_not_set_residual_low() {
    let mut exception = Exception::new(ExceptionId::new("exc.mfa.break-glass"), "break-glass");
    exception.status = ExceptionStatus::Approved;
    exception.approved_by = Some(PrincipalRef::Identity(IdentityId::new("identity:ciso")));
    assert_eq!(exception.status, ExceptionStatus::Approved);

    let result = sample_test_result(Effectiveness::ExceptionApproved);
    assert_eq!(result.effectiveness, Effectiveness::ExceptionApproved);

    let json = serde_json::to_value(&sample_risk()).unwrap();
    assert!(json.get("residualRating").is_none());
    assert!(json.get("residualScore").is_none());
    assert_ne!(json.get("residualRating"), Some(&Value::from("low")));
}

/// Found case: Calculated / Assessed / Hybrid residual modes do not exist.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_calculated_assessed_hybrid_modes_are_absent() {
    let src = product_crate_sources_joined();
    assert!(!src.contains("ResidualRiskMode"));
    // PlannedTestKind::Hybrid exists; residual Hybrid mode must not.
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(!ir.contains("ResidualRiskMode::Hybrid"));
    assert!(!ir.contains("ResidualRiskMode::Calculated"));
    assert!(!ir.contains("ResidualRiskMode::Assessed"));
}

/// Found case: missing methodology / treatment / inherent / control-test snapshot
/// cannot fail closed because there is no projection API.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_missing_pins_have_no_fail_closed_projection() {
    let src = product_crate_sources_joined();
    for needle in [
        "missing inherent-risk version",
        "missing treatment-plan version",
        "missing methodology version",
        "missing control-test snapshot",
        "missing management assessment",
        "missing manual assessment",
    ] {
        assert!(
            !src.contains(needle),
            "fail-closed residual error needle `{needle}` must be absent today"
        );
    }
}

/// Found case: assessment lineage exists and does not pin residual projections.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_lineage_does_not_include_residual() {
    let lineage = read_repo_file("crates/weeping-angel-assurance/src/lineage.rs");
    assert!(
        lineage.contains("Immutable assessment lineage"),
        "lineage module remains the landed snapshot model"
    );
    assert!(
        !lineage.to_lowercase().contains("residual"),
        "lineage must not mention residual on characterization HEAD"
    );
}

/// Found case: IR schema remains `assurance-ir/v1`.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_ir_schema_remains_v1() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
}

/// Found case: evidence crate stays conclusion-free (no residual rating types).
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_evidence_crate_has_no_residual_rating() {
    let src = crate_sources_joined("weeping-angel-evidence");
    assert!(
        src.contains("Observations are facts, never compliance claims")
            || src.contains("never compliance claims"),
        "evidence crate must remain observation-only"
    );
    for needle in [
        "ResidualRisk",
        "residualRating",
        "RiskRating",
        "project_residual_risk",
    ] {
        assert!(
            !src.contains(needle),
            "evidence crate must not contain `{needle}`"
        );
    }
}

/// Dual-suite binaries are listed in root Cargo.toml (directory is not auto-discovered).
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_residual_risk_baseline")
            && toml.contains("sdd_residual_risk_target")
            && toml.contains("tests/contracts/residual_risk.baseline.rs")
            && toml.contains("tests/contracts/residual_risk.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/residual_risk.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/residual_risk.target.rs")
            .is_file()
    );
}

/// Human SSOT is registered in documentation_layout CANONICAL_SPECS.
#[ignore = "superseded by target suite"]
#[test]
fn p09_b_spec_is_canonical() {
    let layout = read_repo_file("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/residual-risk.md"),
        "residual-risk spec must be in CANONICAL_SPECS"
    );
    assert!(manifest_dir().join("docs/specs/residual-risk.md").is_file());
    assert!(
        manifest_dir()
            .join("docs/adr/0003-residual-risk.md")
            .is_file()
    );
}
