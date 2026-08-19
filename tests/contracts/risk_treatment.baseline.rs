//! Baseline suite for Operational ISMS v1 risk treatment (Prompt 08).
//!
//! Characterization of CURRENT tree (`docs/specs/risk-treatment.md` §3) on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`: `Risk` is a four-field stub,
//! `RiskStatus::Accepted` is freely assignable with no principal/expiry,
//! product crates have no treatment decision/plan/action/acceptance types or
//! state machine, `validate()` does not walk treatment control refs (IR-019
//! remains implementation→`RiskId`), and `supports_risk_treatment` is only a
//! compile capability gate.
//!
//! Target `sdd_risk_treatment_target` is the source of truth. This baseline
//! is skipped (`#[ignore = "superseded by target suite"]`).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, AssessmentRequests, Control, ControlId,
    ControlImplementation, ControlImplementationId, Exception, ExceptionId, ExceptionStatus, Risk,
    RiskId, RiskStatus, ValidateIr,
};
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkCompileError, FrameworkContext, FrameworkProfile,
    FrameworkTarget, compile_framework,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
    collect_rs_files(&crate_src(name), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
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
        collect_rs_files(&src, &mut files);
        for path in files {
            chunks.push(fs::read_to_string(&path).unwrap());
        }
    }
    chunks.join("\n")
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn golden_risk_json() -> String {
    read_repo_file("tests/fixtures/assurance-ir/v1/risk.json")
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.risk-treatment.baseline"))
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn treatment_type_needles() -> &'static [&'static str] {
    &[
        "struct RiskTreatmentDecision",
        "struct TreatmentPlan",
        "struct TreatmentAction",
        "struct RiskAcceptance",
        "enum TreatmentStrategy",
        "enum TreatmentState",
        "struct TransferEvidence",
        "struct TargetResidualRisk",
        "fn treatment_required",
        "fn acceptance_in_force",
        "fn validate_treatments_at",
        "fn active_treatment",
    ]
}

fn soc2_target(capabilities: FrameworkCapabilities) -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Soc2,
        capabilities,
        version: weeping_angel_assurance_ir::FrameworkVersion::new("2017"),
        context: FrameworkContext::default(),
    }
}

fn compile_stub(requests: AssessmentRequests) -> weeping_angel_assurance_ir::Assessment {
    let mut assessment = AssessmentDefinition::new(AssessmentId::new("assess.p08-compile"));
    assessment.requests = requests;
    assessment
}

/// P08: minimal risk.json decodes; id == risk:source-tamper; status == Open
#[test]
#[ignore = "superseded by target suite"]
fn p08_b01_golden_minimal_fixture_decodes() {
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

/// P08: RiskStatus::Accepted is assignable with no principal, rationale, or expiresAt
#[test]
#[ignore = "superseded by target suite"]
fn p08_b02_accepted_is_freely_assignable_without_acceptance_evidence() {
    let mut risk = Risk::new(
        RiskId::new("risk:accepted-without-evidence"),
        "verbal accept",
        "no principal, no expiry",
    );
    risk.status = RiskStatus::Accepted;
    assert_eq!(risk.status, RiskStatus::Accepted);

    let json = serde_json::to_value(&risk).unwrap();
    assert_eq!(json["status"], "accepted");
    for absent in [
        "principal",
        "approvedBy",
        "rationale",
        "expiresAt",
        "validFrom",
        "acceptance",
        "acceptanceId",
        "treatment",
        "treatmentId",
        "owner",
    ] {
        assert!(
            json.get(absent).is_none(),
            "found-case Accepted JSON must not contain `{absent}`"
        );
    }

    let decoded: Risk = serde_json::from_value(serde_json::json!({
        "id": "risk:accepted-without-evidence",
        "title": "verbal accept",
        "description": "no principal, no expiry",
        "status": "accepted",
        "principal": { "identity": "identity:ciso" },
        "expiresAt": "2020-01-01T00:00:00Z",
        "rationale": "we accepted this"
    }))
    .unwrap();
    assert_eq!(decoded.status, RiskStatus::Accepted);
    let out = serde_json::to_value(&decoded).unwrap();
    assert!(out.get("principal").is_none());
    assert!(out.get("expiresAt").is_none());
    assert!(out.get("rationale").is_none());
}

/// P08: product crate sources have no RiskTreatmentDecision / TreatmentPlan / TreatmentAction / RiskAcceptance
#[test]
#[ignore = "superseded by target suite"]
fn p08_b03_product_sources_have_no_treatment_types() {
    let src = product_crate_sources_joined();
    for needle in treatment_type_needles() {
        assert!(
            !src.contains(needle),
            "product crate sources must not yet expose `{needle}`"
        );
    }
    let ir_path = manifest_dir().join("crates/weeping-angel-assurance-ir/src/risk_treatment.rs");
    assert!(
        !ir_path.exists(),
        "risk_treatment.rs must be absent on characterization HEAD ({})",
        ir_path.display()
    );
}

/// P08: product sources have no treatment state machine (TreatmentState / proposed→completed)
#[test]
#[ignore = "superseded by target suite"]
fn p08_b04_no_treatment_state_machine() {
    let src = product_crate_sources_joined();
    for needle in [
        "enum TreatmentState",
        "TreatmentState::",
        "ActionState::",
        "InvalidTreatmentTransition",
        "MissingContractEvidence",
        "fn validate_treatments_at",
    ] {
        assert!(
            !src.contains(needle),
            "product sources must not contain treatment state-machine needle `{needle}`"
        );
    }

    let exception_src = read_repo_file("crates/weeping-angel-assurance-ir/src/exception.rs");
    assert!(
        exception_src.contains("Proposed") && exception_src.contains("Approved"),
        "Exception remains the neighbor status enum, not a treatment machine"
    );
    assert!(
        !exception_src.contains("fn can_transition") && !exception_src.contains("fn transition"),
        "Exception still has no transition function and is not risk acceptance"
    );
}

/// P08: id.rs has typed_id!(RiskId) and does not define treatment/acceptance/plan/action ids
#[test]
#[ignore = "superseded by target suite"]
fn p08_b05_typed_ids_have_risk_id_not_treatment_ids() {
    let src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        src.contains("typed_id!(RiskId);"),
        "RiskId must remain a typed_id!"
    );
    for absent in [
        "typed_id!(RiskTreatmentId);",
        "typed_id!(RiskAcceptanceId);",
        "typed_id!(TreatmentPlanId);",
        "typed_id!(TreatmentActionId);",
        "typed_id!(RemediationRef);",
        "RiskTreatmentId",
        "RiskAcceptanceId",
        "TreatmentPlanId",
        "TreatmentActionId",
    ] {
        assert!(
            !src.contains(absent),
            "id.rs must not define `{absent}` unless a treatment module consumes it (none exists)"
        );
    }
    let _id = RiskId::new("risk:source-tamper");
    assert_eq!(_id.as_str(), "risk:source-tamper");
}

/// P08: validate() does not fail a dangling ControlId imagined as a treatment ref; IR-019 still fails
#[test]
#[ignore = "superseded by target suite"]
fn p08_b06_validate_does_not_walk_treatment_control_refs() {
    let mut imagined = empty_assessment();
    imagined.risks.push(Risk::new(
        RiskId::new("risk:open-untreated"),
        "untreated",
        "no treatment inventory exists",
    ));
    let extra = serde_json::json!({
        "id": imagined.id.as_str(),
        "schema_version": imagined.schema_version,
        "risks": [{
            "id": "risk:open-untreated",
            "title": "untreated",
            "description": "no treatment inventory exists",
            "status": "open",
            "controlIds": ["control.missing"],
            "treatmentId": "treat:missing"
        }],
        "risk_treatments": [{
            "id": "treat:missing",
            "riskId": "risk:open-untreated",
            "strategy": "mitigate",
            "state": "completed",
            "controlIds": ["control.missing"],
            "implementationIds": ["impl.missing"]
        }]
    });
    let decoded: AssessmentDefinition = serde_json::from_value(extra).unwrap();
    decoded
        .validate()
        .expect("current validate() does not walk imagined treatment ControlIds");
    let round = serde_json::to_value(&decoded).unwrap();
    assert!(
        round.get("risk_treatments").is_none() && round.get("riskTreatments").is_none(),
        "AssessmentDefinition currently has no treatment inventory field"
    );

    let mut dangling_impl = empty_assessment();
    dangling_impl.controls.push(sample_control());
    dangling_impl.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.access.mfa.org"),
            ControlId::new("control.access.mfa"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = dangling_impl
        .validate()
        .expect_err("IR-019: dangling implementation→RiskId must still fail");
    let msg = err.to_string();
    assert!(
        msg.contains("dangling risk reference"),
        "IR-019 message: {err}"
    );
    assert!(
        msg.contains("risk:missing"),
        "IR-019 error must name risk:missing, got {msg}"
    );
}

/// P08: supports_risk_treatment default false; compile names the flag; compile does not build decisions
#[test]
#[ignore = "superseded by target suite"]
fn p08_b07_supports_risk_treatment_is_compile_capability_only() {
    let default_caps = FrameworkCapabilities::default();
    assert!(
        !default_caps.supports_risk_treatment,
        "default FrameworkCapabilities.supports_risk_treatment must be false"
    );

    let err = compile_framework(
        &compile_stub(AssessmentRequests {
            risk_treatment: true,
            ..AssessmentRequests::default()
        }),
        &soc2_target(FrameworkCapabilities::default()),
    )
    .expect_err("requesting risk_treatment without the capability must fail closed");
    match err {
        FrameworkCompileError::CapabilityViolation { capability, .. } => {
            assert_eq!(
                capability, "supports_risk_treatment",
                "capability violation must name supports_risk_treatment, got {capability}"
            );
        }
        other => panic!("expected CapabilityViolation, got {other:?}"),
    }

    let compiled = compile_framework(
        &compile_stub(AssessmentRequests {
            risk_treatment: true,
            ..AssessmentRequests::default()
        }),
        &soc2_target(FrameworkCapabilities {
            supports_risk_treatment: true,
            ..FrameworkCapabilities::default()
        }),
    )
    .expect("capability match must compile even though no treatment objects exist");
    assert!(compiled.validation.ok);
    let compiled_json = serde_json::to_value(&compiled).unwrap();
    for absent in [
        "riskTreatments",
        "risk_treatments",
        "treatmentDecisions",
        "acceptances",
    ] {
        assert!(
            compiled_json.get(absent).is_none(),
            "CompiledFramework must not grow a `{absent}` inventory today"
        );
    }
    assert!(
        !compiled
            .validation
            .stages
            .iter()
            .any(|s| s.contains("treatment")),
        "compile pipeline stages must not evaluate treatment plans, got {:?}",
        compiled.validation.stages
    );

    let framework_src = read_repo_file("crates/weeping-angel-framework/src/lib.rs");
    assert!(
        framework_src.contains("supports_risk_treatment"),
        "compile capability check still names supports_risk_treatment"
    );
    assert!(
        !framework_src.contains("RiskTreatmentDecision")
            && !framework_src.contains("TreatmentPlan"),
        "compile_framework must not construct treatment decision types"
    );

    let iso_manifest = read_repo_file("frameworks/iso-27001/2022/manifest.toml");
    assert!(
        iso_manifest.contains("risk_treatment = true"),
        "ISO pack still advertises the compile capability, not an engine"
    );
}

/// P08: lib.rs has mod risk and no mod risk_treatment
#[test]
#[ignore = "superseded by target suite"]
fn p08_b08_ir_lib_has_risk_module_not_risk_treatment() {
    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        lib.contains("pub mod risk;"),
        "IR crate still has a risk module"
    );
    assert!(
        lib.contains("pub use risk::{Risk, RiskStatus};"),
        "IR lib.rs must re-export Risk / RiskStatus"
    );
    assert!(
        !lib.contains("mod risk_treatment") && !lib.contains("risk_treatment::"),
        "IR lib.rs must not declare risk_treatment"
    );
    for needle in [
        "RiskTreatmentDecision",
        "TreatmentPlan",
        "TreatmentAction",
        "RiskAcceptance",
        "TreatmentStrategy",
        "TreatmentState",
    ] {
        assert!(
            !lib.contains(needle),
            "IR lib.rs must not export `{needle}` on characterization HEAD"
        );
    }
}

/// P08: collision fence — do not rewrite methodology/register specs; collectors have no treatment types
#[test]
#[ignore = "superseded by target suite"]
fn p08_b09_collision_fence_neighbors_and_collectors() {
    let this_src = read_repo_file("tests/contracts/risk_treatment.baseline.rs");
    let collector_import = format!("{}::", ["weeping", "angel", "collector"].join("_"));
    assert!(
        !this_src.contains(&collector_import),
        "baseline suite must not import collector types"
    );

    let methodology = read_repo_file("docs/specs/risk-methodology.md");
    assert!(
        methodology.contains("# SDD: Risk Methodology IR and Scoring"),
        "this suite must not rewrite the risk methodology spec"
    );
    let register = read_repo_file("docs/specs/risk-register.md");
    assert!(
        register.contains("# SDD: Operational Risk Register (ISMS v1)"),
        "this suite must not rewrite the risk register spec"
    );
    let identification = read_repo_file("docs/specs/risk-identification.md");
    assert!(
        identification.contains("Risk identification") || identification.contains("RiskCandidate"),
        "risk identification spec must remain a neighbor, not forked here"
    );

    let collector = crate_sources_joined("weeping-angel-collector");
    for needle in treatment_type_needles() {
        assert!(
            !collector.contains(needle),
            "collector sources must not contain `{needle}`"
        );
    }
    assert!(
        !collector.contains("RiskRating::High") && !collector.contains("enum RiskRating"),
        "collectors must not emit a global RiskRating treatment claim"
    );
}

/// P08: all four strategies are absent — enum-only Mitigate/Accept/Avoid/Transfer is not an engine
#[test]
#[ignore = "superseded by target suite"]
fn p08_four_strategies_are_not_ir_treatment_enums() {
    for unknown in ["mitigate", "avoid", "transfer", "Mitigate", "Transfer"] {
        let err = serde_json::from_str::<RiskStatus>(&format!("\"{unknown}\"")).unwrap_err();
        assert!(
            !err.to_string().is_empty(),
            "strategy-shaped status `{unknown}` must not decode as RiskStatus today"
        );
    }

    let src = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !src.contains("TreatmentStrategy::Mitigate")
            && !src.contains("TreatmentStrategy::Accept")
            && !src.contains("TreatmentStrategy::Avoid")
            && !src.contains("TreatmentStrategy::Transfer"),
        "IR must not expose four-strategy treatment enums yet"
    );
}

/// P08: expired risk acceptance does not exist — Accepted never expires or un-suppresses treatment
#[test]
#[ignore = "superseded by target suite"]
fn p08_expired_acceptance_cannot_unsuppress_because_nothing_expires() {
    let mut risk = Risk::new(RiskId::new("risk:expired-accept"), "t", "d");
    risk.status = RiskStatus::Accepted;
    let json = serde_json::to_value(&risk).unwrap();
    assert!(json.get("expiresAt").is_none());

    let validation_src = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    assert!(
        !validation_src.contains("as_of")
            && !validation_src.contains("expires_at")
            && !validation_src.contains("treatment_required")
            && !validation_src.contains("acceptance_in_force"),
        "validate() is clockless and has no acceptance expiry / treatment_required query"
    );

    let mut assessment = empty_assessment();
    assessment.risks.push(risk);
    assessment
        .validate()
        .expect("Accepted without expiry still validates — there is no treatment requirement");
}

/// P08: partially complete mitigation is not modeled — Mitigated is a free status overwrite
#[test]
#[ignore = "superseded by target suite"]
fn p08_partial_mitigation_is_a_free_status_write() {
    let mut risk = Risk::new(RiskId::new("risk:half-done"), "t", "d");
    risk.status = RiskStatus::Mitigated;
    assert_eq!(risk.status, RiskStatus::Mitigated);
    let json = serde_json::to_value(&risk).unwrap();
    assert!(json.get("actions").is_none());
    assert!(json.get("plan").is_none());
    assert_eq!(json["status"], "mitigated");
}

/// P08: transferred risk with missing contract evidence is not a validation error
#[test]
#[ignore = "superseded by target suite"]
fn p08_transfer_without_contract_is_not_a_status_or_error() {
    let mut assessment = empty_assessment();
    assessment.risks.push(Risk::new(
        RiskId::new("risk:transferred-no-contract"),
        "transfer claimed",
        "no contract artifact",
    ));
    assessment
        .validate()
        .expect("there is no Transfer strategy and no MissingContractEvidence check");
    let src = product_crate_sources_joined();
    assert!(
        !src.contains("MissingContractEvidence") && !src.contains("struct TransferEvidence"),
        "transfer contract evidence types are absent"
    );
}

/// P08: superseded treatment is not modeled on Risk or assessments
#[test]
#[ignore = "superseded by target suite"]
fn p08_superseded_treatment_is_absent() {
    let risk = Risk::new(RiskId::new("risk:supersede"), "t", "d");
    let json = serde_json::to_value(&risk).unwrap();
    assert!(json.get("supersedes").is_none());
    assert!(json.get("supersededBy").is_none());
    let src = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !src.contains("active_treatment") && !src.contains("fn supersede"),
        "no active-vs-superseded treatment path API"
    );
}

/// P08: target residual mismatch cannot fail — Risk stores no target residual
#[test]
#[ignore = "superseded by target suite"]
fn p08_target_residual_is_absent() {
    let json = serde_json::to_value(&Risk::new(RiskId::new("risk:residual"), "t", "d")).unwrap();
    assert!(json.get("targetResidual").is_none());
    assert!(json.get("residualScore").is_none());
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    assert!(!risk_src.contains("target_residual") && !risk_src.contains("TargetResidual"));
}

/// P08: Exception is a control exception, not immutable risk acceptance
#[test]
#[ignore = "superseded by target suite"]
fn p08_exception_is_not_risk_acceptance() {
    let mut exception = Exception::new(ExceptionId::new("exc.not-acceptance"), "waiver");
    exception.status = ExceptionStatus::Expired;
    assert_eq!(exception.status, ExceptionStatus::Expired);

    let mut assessment = empty_assessment();
    assessment.exceptions.push(exception);
    assessment
        .risks
        .push(Risk::new(RiskId::new("risk:still-open"), "t", "d"));
    assessment
        .validate()
        .expect("expired Exception does not interact with Risk treatment");
}

/// P08: catalog attestation of control.risk.treatment is not a GRC engine
#[test]
#[ignore = "superseded by target suite"]
fn p08_governance_catalog_attests_treatment_control_without_engine() {
    let catalog = read_repo_file("catalog/canonical/v1/tests/governance.toml");
    assert!(
        catalog.contains("control.risk.treatment") || catalog.contains("test.risk.treatment"),
        "governance catalog still attests treatment as catalog rows"
    );
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("control.risk.treatment"),
        "IR crate does not implement catalog ids as a treatment workflow"
    );
}

/// P08: dual-suite files exist beside this characterization (registration asserted in Cargo.toml at wire-up)
#[test]
#[ignore = "superseded by target suite"]
fn p08_assessment_has_no_risk_treatments_inventory() {
    let assessment = empty_assessment();
    let json = serde_json::to_value(&assessment).unwrap();
    assert!(json.get("risk_treatments").is_none());
    assert!(json.get("riskTreatments").is_none());
    assert!(assessment.risks.is_empty());

    let golden: Value = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    assert!(golden.get("risk_treatments").is_none());
    assert_eq!(golden["risks"], serde_json::json!([]));
    assert_eq!(golden["requests"]["risk_treatment"], false);
}
