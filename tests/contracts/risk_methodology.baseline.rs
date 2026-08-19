//! Baseline suite for Operational ISMS v1 risk methodology — risk methodology IR.
//!
//! SUPERSEDED by `sdd_risk_methodology_target` after Prompt 05 landed
//! `RiskMethodology` / `score_risk`. Absence-of-API characterization is
//! kept registered but ignored so CI does not require the pre-implement HEAD.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, Control, ControlId, ControlImplementation,
    ControlImplementationId, Risk, RiskId, RiskStatus, ValidateIr,
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

/// P05: risk.rs module docs still contain `Minimal risk record. Not a risk engine.`
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b01_risk_module_is_not_a_risk_engine() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        src.contains("//! Minimal risk record. Not a risk engine."),
        "risk.rs must keep the found-case module comment"
    );
    assert!(
        !src.contains("level"),
        "Risk has no level field on characterization HEAD"
    );
}

/// P05: Risk::new yields four fields and Open; JSON has no scoring fields
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b02_risk_new_is_four_fields_and_open() {
    let risk = Risk::new(
        RiskId::new("risk:source-tamper"),
        "Source tampering",
        "Unauthorized change to the source of record.",
    );
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.title, "Source tampering");
    assert_eq!(
        risk.description,
        "Unauthorized change to the source of record."
    );
    assert_eq!(risk.status, RiskStatus::Open);

    let json = serde_json::to_value(&risk).unwrap();
    assert_eq!(json["id"], "risk:source-tamper");
    assert_eq!(json["title"], "Source tampering");
    assert_eq!(
        json["description"],
        "Unauthorized change to the source of record."
    );
    assert_eq!(json["status"], "open");
    let obj = json.as_object().expect("Risk serializes as an object");
    assert_eq!(
        obj.keys().count(),
        4,
        "found-case Risk JSON is exactly four keys, got {obj:?}"
    );
    for absent in [
        "likelihood",
        "impact",
        "score",
        "rating",
        "methodology",
        "appetite",
        "residual",
        "treatment",
        "owner",
        "residualScore",
    ] {
        assert!(
            obj.get(absent).is_none(),
            "found-case Risk JSON must not contain `{absent}`"
        );
    }

    let _ = RiskStatus::Accepted;
    let _ = RiskStatus::Mitigated;
    let _ = RiskStatus::Closed;
}

/// P05: tests/fixtures/assurance-ir/v1/risk.json decodes; id == risk:source-tamper
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b03_golden_risk_json_decodes_four_keys() {
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
    let obj = value.as_object().expect("risk.json is an object");
    assert_eq!(obj.keys().count(), 4);
    assert_eq!(obj["id"], "risk:source-tamper");
    assert_eq!(obj["status"], "open");
}

/// P05: product crate sources have no RiskMethodology / ScoringMode / score_risk
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b04_product_sources_have_no_methodology_scoring_apis() {
    let src = product_crate_sources_joined();
    for needle in [
        "struct RiskMethodology",
        "RiskMethodologyId",
        "ScoringMode",
        "fn score_risk",
        "fn validate_risk_methodology",
        "struct LikelihoodScale",
        "struct ImpactScale",
        "struct RiskMatrix",
        "struct RiskScore",
        "enum RiskRating",
        "struct ScoredRisk",
        "struct RiskAppetite",
        "struct RiskTolerance",
        "struct AcceptanceThreshold",
    ] {
        assert!(
            !src.contains(needle),
            "product crate sources must not yet expose `{needle}`"
        );
    }
}

/// P05: lib.rs re-exports Risk / RiskStatus and does not export methodology types
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b05_ir_lib_reexports_only_the_minimal_risk_record() {
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
        !lib.contains("pub mod risk_methodology"),
        "IR lib.rs must not declare risk_methodology"
    );
    for needle in [
        "RiskMethodology",
        "RiskMethodologyId",
        "ScoringMode",
        "score_risk",
        "ScoredRisk",
        "DerivedRating",
    ] {
        assert!(
            !lib.contains(needle),
            "IR lib.rs must not export `{needle}` on characterization HEAD"
        );
    }
}

/// P05: collector crate sources have no RiskRating / RiskMethodology / score_risk
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b06_collectors_have_no_risk_types() {
    let src = crate_sources_joined("weeping-angel-collector");
    assert!(
        !src.contains("risk"),
        "collector Rust sources have zero matches for `risk` on characterization HEAD"
    );
    for needle in [
        "RiskRating",
        "RiskMethodology",
        "score_risk",
        "DerivedRating",
    ] {
        assert!(
            !src.contains(needle),
            "collector sources must not contain `{needle}`"
        );
    }
}

/// P05: IR-019 still fails closed on dangling RiskId (risk:missing on an implementation)
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b07_ir_019_dangling_risk_id_still_fails_closed() {
    let mut assessment = AssessmentDefinition::new(AssessmentId::new("assess.ir-019"));
    assessment.controls.push(Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    ));
    assessment.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = assessment
        .validate()
        .expect_err("IR-019: dangling risk must fail closed");
    let msg = err.to_string();
    assert!(
        msg.contains("dangling risk"),
        "IR-019 error must mention dangling risk, got {msg}"
    );
    assert!(
        msg.contains("risk:missing"),
        "IR-019 error must name risk:missing, got {msg}"
    );
}

/// P05: id.rs has typed_id!(RiskId) and does not have RiskMethodologyId
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b08_risk_id_exists_methodology_id_does_not() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        src.contains("typed_id!(RiskId);"),
        "RiskId must remain a typed_id!"
    );
    assert!(
        !src.contains("RiskMethodologyId"),
        "RiskMethodologyId must not exist on characterization HEAD"
    );
    let _id = RiskId::new("risk:source-tamper");
    assert_eq!(_id.as_str(), "risk:source-tamper");
}

/// P05: no risk_methodology.rs module under the IR crate
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b09_no_risk_methodology_module_file() {
    let path = manifest_dir().join("crates/weeping-angel-assurance-ir/src/risk_methodology.rs");
    assert!(
        !path.exists(),
        "risk_methodology.rs must be absent on characterization HEAD ({})",
        path.display()
    );
}

/// P05: collision fence — this suite does not import GitHub collector types or change severity_policy.rs
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_b10_collision_fence_github_collector_and_severity_policy() {
    let this_src = read_repo_file("tests/contracts/risk_methodology.baseline.rs");
    let collector_import = format!("{}::", ["weeping", "angel", "collector"].join("_"));
    assert!(
        !this_src.contains(&collector_import),
        "baseline suite must not import collector types"
    );
    let github_types = ["GITHUB", "EVIDENCE", "TYPES"].join("_");
    assert!(
        !this_src.contains(&github_types),
        "baseline suite must not touch GitHub evidence types"
    );

    let severity = read_repo_file("src/contract/severity_policy.rs");
    assert!(
        !severity.contains("RiskMethodology")
            && !severity.contains("score_risk")
            && !severity.contains("IsmsContext"),
        "scanner severity_policy.rs must stay a Codex attack-path matrix, not an ISMS methodology"
    );

    let assurance = crate_sources_joined("weeping-angel-assurance");
    assert!(
        !assurance.contains("fn score_risk") && !assurance.contains("struct RiskMethodology"),
        "assurance crate owns Kleene applicability, not ISMS scoring"
    );
    let control_test = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !control_test.contains("fn score_risk") && !control_test.contains("struct RiskMethodology"),
        "control-test must not derive ISMS ratings"
    );
}

/// P05: assessment inventory is Vec<Risk>, empty by default; golden assessment.json has risks: []
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_assessment_risks_inventory_is_empty_by_default() {
    let assessment = AssessmentDefinition::new(AssessmentId::new("assess.p05-baseline"));
    assert!(
        assessment.risks.is_empty(),
        "AssessmentDefinition.risks is empty by default"
    );

    let raw = fs::read_to_string(ir_fixture("assessment.json")).unwrap();
    let value: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        value["risks"],
        serde_json::json!([]),
        "golden assessment.json must keep an empty risks inventory"
    );

    let decoded: AssessmentDefinition = serde_json::from_str(&raw).unwrap();
    assert!(decoded.risks.is_empty());
}

/// P05: duplicate risk ids on the assessment are not currently rejected
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_duplicate_risk_ids_are_not_rejected() {
    let mut assessment = AssessmentDefinition::new(AssessmentId::new("assess.p05-dup-risks"));
    assessment.risks.push(Risk::new(
        RiskId::new("risk:source-tamper"),
        "one",
        "first copy",
    ));
    assessment.risks.push(Risk::new(
        RiskId::new("risk:source-tamper"),
        "two",
        "second copy",
    ));
    assessment
        .validate()
        .expect("found case: duplicate risk ids are membership-only, not uniqueness-checked");
}

/// P05: methodology fixtures and ISMS context IR IsmsContext are absent
#[test]
#[ignore = "superseded by sdd_risk_methodology_target"]
fn p05_methodology_fixtures_and_isms_context_are_absent() {
    for name in [
        "risk-methodology-3x3.json",
        "risk-methodology-5x5.json",
        "risk-methodology-expected-loss.json",
    ] {
        let path = ir_fixture(name);
        assert!(
            !path.exists(),
            "methodology fixture {name} must not exist on characterization HEAD"
        );
    }

    let product = product_crate_sources_joined();
    assert!(
        !product.contains("IsmsContext"),
        "ISMS context IR IsmsContext is absent in product crates"
    );
    assert!(
        !product.contains("RiskCandidate"),
        "risk identification RiskCandidate is absent"
    );
}
