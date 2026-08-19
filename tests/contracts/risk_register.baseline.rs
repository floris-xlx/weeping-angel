//! Baseline suite for risk register (operational risk register).
//!
//! Characterization of CURRENT tree (`docs/specs/risk-register.md` §3) on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`: `Risk` is a four-field inventory
//! stub (*“Not a risk engine”*), `RiskStatus` is Open|Accepted|Mitigated|Closed,
//! validation treats `assessment.risks` as an id bag for IR-019 only, scanner
//! `Finding` is not IR, and risk methodology methodology types are absent from `risk.rs`.
//!
//! Target `sdd_risk_register_target` is the source of truth. This baseline
//! is skipped (`#[ignore = "superseded by target suite"]`). Does not implement
//! the operational register.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel::finding::Finding;
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

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn golden_risk_json() -> String {
    read_repo_file("tests/fixtures/assurance-ir/v1/risk.json")
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.risk-register.baseline"))
}

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("Risk JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn risk_status_json_name(status: RiskStatus) -> &'static str {
    match status {
        RiskStatus::Open => "open",
        RiskStatus::Accepted => "accepted",
        RiskStatus::Mitigated => "mitigated",
        RiskStatus::Closed => "closed",
        _ => "other",
    }
}

/// RR-001 found case: golden `risk.json` is four camelCase fields; `id` is `risk:source-tamper`.
#[test]
#[ignore = "superseded by target suite"]
fn rr_001_golden_minimal_fixture_decodes() {
    let risk: Risk = serde_json::from_str(&golden_risk_json()).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.title, "Source tampering");
    assert_eq!(
        risk.description,
        "Unauthorized change to the source of record."
    );
    assert_eq!(risk.status, RiskStatus::Open);

    let fixture: Value = serde_json::from_str(&golden_risk_json()).unwrap();
    let mut keys = json_object_keys(&fixture);
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

/// RR-002 found case: `Risk::new` defaults `Open` and omits owner / treatment / residualScore.
#[test]
#[ignore = "superseded by target suite"]
fn rr_002_risk_new_is_a_minimal_record_not_a_grc_engine() {
    let risk = Risk::new(
        RiskId::new("risk:org-1"),
        "supplier concentration",
        "single critical vendor",
    );
    assert_eq!(risk.status, RiskStatus::Open);
    assert_eq!(risk.id.as_str(), "risk:org-1");
    assert_eq!(risk.title, "supplier concentration");
    assert_eq!(risk.description, "single critical vendor");

    let json = serde_json::to_value(&risk).unwrap();
    assert_eq!(json["id"], "risk:org-1");
    assert_eq!(json["title"], "supplier concentration");
    assert_eq!(json["status"], "open");
    assert!(json.get("treatment").is_none());
    assert!(json.get("treatmentId").is_none());
    assert!(json.get("owner").is_none());
    assert!(json.get("residualScore").is_none());
    assert!(json.get("residualRating").is_none());
    assert!(json.get("scenario").is_none());
    assert!(json.get("threat").is_none());
    assert!(json.get("assetIds").is_none());
    assert!(json.get("controlIds").is_none());
    assert!(json.get("findingRefs").is_none());
    assert!(json.get("history").is_none());
    assert!(json.get("version").is_none());
    assert!(json.get("nextReview").is_none());
    assert!(json.get("methodologyVersion").is_none());
    assert!(json.get("inherentScore").is_none());
    assert!(json.get("inherentRating").is_none());

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
}

/// RR-003 found case: round-trip is the four-field stub; operational fields are not stored.
#[test]
#[ignore = "superseded by target suite"]
fn rr_003_complete_operational_payload_is_dropped_on_decode() {
    let payload = serde_json::json!({
        "id": "risk:source-tamper",
        "title": "Source tampering",
        "description": "Unauthorized change to the source of record.",
        "status": "open",
        "scenario": "attacker tampers with the source of record",
        "threat": "insider",
        "weaknessRefs": ["CWE-284"],
        "assetIds": ["asset:missing"],
        "processingActivityIds": ["ropa:missing"],
        "vendorIds": ["vendor:missing"],
        "cia": { "confidentiality": 5, "integrity": 5, "availability": 1 },
        "likelihood": 5,
        "impact": 5,
        "inherentScore": 25,
        "inherentRating": "high",
        "residualScore": 1,
        "residualRating": "low",
        "methodologyVersion": "meth.v1",
        "owner": { "identity": "identity:alice" },
        "source": "finding",
        "discoveredAt": "2026-01-01T00:00:00Z",
        "reviewCadence": { "intervalSeconds": 86400 },
        "nextReview": "2020-01-01T00:00:00Z",
        "treatmentId": "treat:missing",
        "controlIds": ["control.missing"],
        "evidenceRefs": ["evidence.req.missing"],
        "findingRefs": ["finding:unprotected-branch"],
        "tags": ["isms"],
        "classification": "confidential",
        "version": 9,
        "supersedes": "risk:old",
        "supersededBy": "risk:new",
        "history": [{ "kind": "created", "version": 1 }]
    });
    let risk: Risk = serde_json::from_value(payload).unwrap();
    let out = serde_json::to_value(&risk).unwrap();
    assert_eq!(out["id"], "risk:source-tamper");
    assert_eq!(out["status"], "open");
    for key in [
        "scenario",
        "threat",
        "weaknessRefs",
        "assetIds",
        "controlIds",
        "treatmentId",
        "owner",
        "residualScore",
        "inherentScore",
        "findingRefs",
        "history",
        "version",
        "nextReview",
        "cia",
        "methodologyVersion",
    ] {
        assert!(
            out.get(key).is_none(),
            "current Risk drops unknown operational key `{key}`"
        );
    }
}

/// RR-004 found case: four statuses, camelCase JSON; Draft / UnderTreatment / Retired do not decode.
#[test]
#[ignore = "superseded by target suite"]
fn rr_004_risk_status_is_four_camel_case_variants() {
    assert_eq!(RiskStatus::default(), RiskStatus::Open);
    for status in [
        RiskStatus::Open,
        RiskStatus::Accepted,
        RiskStatus::Mitigated,
        RiskStatus::Closed,
    ] {
        let encoded = serde_json::to_string(&status).unwrap();
        assert_eq!(encoded, format!("\"{}\"", risk_status_json_name(status)));
        let decoded: RiskStatus = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, status);
    }

    for unknown in ["draft", "underTreatment", "retired", "Draft", "open "] {
        let err = serde_json::from_str::<RiskStatus>(&format!("\"{unknown}\"")).unwrap_err();
        let msg = err.to_string();
        assert!(
            !msg.is_empty(),
            "unknown status `{unknown}` must fail closed today"
        );
    }
}

/// RR-005 found case: no transition table; status is a public field anyone can overwrite.
#[test]
#[ignore = "superseded by target suite"]
fn rr_005_status_is_a_public_field_with_no_transition_validator() {
    let mut risk = Risk::new(RiskId::new("risk:open"), "t", "d");
    risk.status = RiskStatus::Mitigated;
    assert_eq!(risk.status, RiskStatus::Mitigated);
    risk.status = RiskStatus::Closed;
    assert_eq!(risk.status, RiskStatus::Closed);
    risk.status = RiskStatus::Accepted;
    assert_eq!(risk.status, RiskStatus::Accepted);

    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        !risk_src.contains("fn can_transition") && !risk_src.contains("fn transition"),
        "current Risk has no transition validator"
    );
    assert!(
        !risk_src.contains("Draft")
            && !risk_src.contains("UnderTreatment")
            && !risk_src.contains("Retired"),
        "current RiskStatus does not name Draft / UnderTreatment / Retired"
    );
}

/// RR-006 found case: dangling asset / control / treatment ids on a risk JSON are ignored, not validated.
#[test]
#[ignore = "superseded by target suite"]
fn rr_006_validate_does_not_walk_risk_asset_control_or_treatment_refs() {
    let mut assessment = empty_assessment();
    let risk_json = serde_json::json!({
        "id": "risk:orphan-graph",
        "title": "orphan",
        "description": "dangling refs are not fields today",
        "status": "open",
        "assetIds": ["asset:missing"],
        "controlIds": ["control.missing"],
        "treatmentId": "treat:missing"
    });
    assessment
        .risks
        .push(serde_json::from_value(risk_json).unwrap());
    assessment
        .validate()
        .expect("current validate() does not inspect Risk graph refs");
}

/// RR-007 found case: IR-019 still fails on dangling implementation→risk; duplicate RiskIds collapse.
#[test]
#[ignore = "superseded by target suite"]
fn rr_007_implementation_risk_ids_must_resolve_and_duplicate_risk_ids_collapse() {
    let mut dangling = empty_assessment();
    dangling.controls.push(sample_control());
    dangling.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = dangling.validate().expect_err("IR-019: dangling risk");
    assert!(
        err.to_string().contains("dangling risk reference"),
        "IR-019 message: {err}"
    );

    let mut dupes = empty_assessment();
    let id = RiskId::new("risk:same");
    dupes
        .risks
        .push(Risk::new(id.clone(), "first", "first copy"));
    dupes
        .risks
        .push(Risk::new(id.clone(), "second", "second copy"));
    dupes.controls.push(sample_control());
    dupes.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(id),
    );
    dupes
        .validate()
        .expect("duplicate RiskIds silently collapse into the IR-019 id bag");
}

/// RR-008 found case: no review clock; unscheduled vs overdue is not a Risk API.
#[test]
#[ignore = "superseded by target suite"]
fn rr_008_risk_has_no_review_overdue_semantics() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        !risk_src.contains("review_overdue")
            && !risk_src.contains("next_review")
            && !risk_src.contains("nextReview")
            && !risk_src.contains("review_cadence"),
        "current Risk has no review cadence / overdue helper"
    );
    let validation_src = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    assert!(
        !validation_src.contains("validate_risk_reviews") && !validation_src.contains("overdue"),
        "current validate() is clockless and does not mention overdue reviews"
    );
}

/// RR-009 found case: no methodology scoring surface on Risk; risk methodology types are not in risk.rs.
#[test]
#[ignore = "superseded by target suite"]
fn rr_009_risk_has_no_methodology_scoring_fields_or_adapter() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    for needle in [
        "likelihood",
        "inherent_score",
        "inherent_rating",
        "methodology_version",
        "score_inherent",
        "RiskScore",
        "RiskRating",
        "LikelihoodScale",
        "ImpactScale",
        "RiskMatrix",
    ] {
        assert!(
            !risk_src.contains(needle),
            "current risk.rs must not contain `{needle}`"
        );
    }
}

/// RR-010 found case: residual score is not a constructor/JSON field.
#[test]
#[ignore = "superseded by target suite"]
fn rr_010_residual_score_is_absent_from_the_stub() {
    let risk = Risk::new(RiskId::new("risk:residual"), "t", "d");
    let json = serde_json::to_value(&risk).unwrap();
    assert!(json.get("residualScore").is_none());
    assert!(json.get("residualRating").is_none());
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(!risk_src.contains("residual"));
}

/// RR-011 found case: scanner Finding lives in `src/finding.rs`; IR has no Finding type or promotion.
#[test]
#[ignore = "superseded by target suite"]
fn rr_011_scanner_finding_is_not_an_ir_risk() {
    let finding = Finding::builder("recon", "unprotected-branch")
        .title("Unprotected default branch")
        .description("scanner output is not an ISMS risk")
        .build();
    assert_eq!(finding.id, "unprotected-branch");
    assert!(!finding.title.is_empty());

    let ir_src = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir_src.contains("pub struct Finding") && !ir_src.contains("struct FindingRef"),
        "weeping-angel-assurance-ir must not declare Finding / FindingRef today"
    );
    assert!(
        !ir_src.contains("From<Finding>") && !ir_src.contains("impl From<weeping_angel"),
        "IR must not promote scanner Finding to Risk"
    );
    let finding_src = read_repo_file("src/finding.rs");
    assert!(
        finding_src.contains("pub struct Finding"),
        "scanner Finding remains in src/finding.rs"
    );
}

/// RR-012 found case: edits overwrite in place; no version / history / revise.
#[test]
#[ignore = "superseded by target suite"]
fn rr_012_edits_overwrite_public_fields_with_no_history() {
    let mut risk = Risk::new(RiskId::new("risk:history"), "original", "body");
    risk.title = "revised".into();
    assert_eq!(risk.title, "revised");
    let json = serde_json::to_value(&risk).unwrap();
    assert!(json.get("history").is_none());
    assert!(json.get("version").is_none());
    assert!(json.get("supersedes").is_none());
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        !risk_src.contains("fn revise")
            && !risk_src.contains("history")
            && !risk_src.contains("supersedes"),
        "current Risk has no history/supersession API"
    );
}

/// RR-013 found case: no CIA impact dimensions on the stub.
#[test]
#[ignore = "superseded by target suite"]
fn rr_013_cia_dimensions_are_absent() {
    let json = serde_json::to_value(&Risk::new(RiskId::new("risk:cia"), "t", "d")).unwrap();
    assert!(json.get("cia").is_none());
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(!risk_src.contains("cia") && !risk_src.contains("confidentiality"));
}

/// RR-014 found case: Risk has no owner; PrincipalRef is not used by the constructor.
#[test]
#[ignore = "superseded by target suite"]
fn rr_014_risk_has_no_owner_principal() {
    let json = serde_json::to_value(&Risk::new(RiskId::new("risk:owner"), "t", "d")).unwrap();
    assert!(json.get("owner").is_none());
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(!risk_src.contains("owner") && !risk_src.contains("PrincipalRef"));
}

/// RR-015: dual-suite names are listed in root Cargo.toml (baseline + target files).
#[test]
#[ignore = "superseded by target suite"]
fn rr_015_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_risk_register_baseline")
            && toml.contains("sdd_risk_register_target")
            && toml.contains("tests/contracts/risk_register.baseline.rs")
            && toml.contains("tests/contracts/risk_register.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/risk_register.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/risk_register.target.rs")
            .is_file()
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn rr_module_docs_declare_not_a_risk_engine() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(
        risk_src.contains("Minimal risk record. Not a risk engine."),
        "module docs are the found-case product statement"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn rr_missing_status_defaults_to_open() {
    let risk: Risk = serde_json::from_value(serde_json::json!({
        "id": "risk:no-status",
        "title": "t",
        "description": "d"
    }))
    .unwrap();
    assert_eq!(risk.status, RiskStatus::Open);
}

#[test]
#[ignore = "superseded by target suite"]
fn rr_assessment_risks_default_empty_and_golden_assessment_has_empty_vec() {
    let assessment = empty_assessment();
    assert!(assessment.risks.is_empty());
    assessment.validate().unwrap();

    let golden: AssessmentDefinition = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    assert!(golden.risks.is_empty());
    golden.validate().unwrap();
}

#[test]
#[ignore = "superseded by target suite"]
fn rr_validation_builds_risk_id_bag_without_duplicate_error() {
    let validation_src = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    assert!(
        validation_src.contains("dangling risk reference"),
        "IR-019 remains implementation→risk"
    );
    assert!(
        !validation_src.contains("duplicate risk"),
        "current validate() does not error on duplicate RiskId"
    );
    let risk_block = validation_src
        .split("let risk_ids:")
        .nth(1)
        .expect("validation collects risk_ids");
    let risk_block = risk_block.split("let exception_ids:").next().unwrap();
    assert!(
        !risk_block.contains("duplicate"),
        "risk id collection must silently collapse duplicates today"
    );
}
