//! SUPERSEDED by `sdd_supplier_risk_target`.
//!
//! Historical characterization of SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`
//! (`docs/specs/supplier-risk.md` §3): `Vendor` was `{ id, name }`
//! (*“Minimal vendor node for the compliance graph.”*),
//! `AssessmentDefinition.vendors` was an unvalidated bag,
//! `ProcessingActivity.processors` was a `VendorId` list with no graph check,
//! `HasVendor` was presence-only, and `validation.rs` did not walk vendors.
//!
//! Target `sdd_supplier_risk_target` is the source of truth. This baseline
//! is skipped (`#[ignore = "superseded by target suite"]`).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind, Control, ControlId,
    ControlImplementation, ControlImplementationId, Exception, ExceptionId, Identity, IdentityId,
    IdentityKind, ProcessingActivity, ProcessingActivityId, Risk, RiskId, ValidateIr, Vendor,
    VendorId,
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

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.supplier-risk.baseline"))
}

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("Vendor JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn vendor_src() -> String {
    read_repo_file("crates/weeping-angel-assurance-ir/src/vendor.rs")
}

fn validation_src() -> String {
    read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs")
}

/// SR-001 found case: `Vendor::new` is two camelCase fields; constructor JSON has no lifecycle keys.
#[ignore = "superseded by target suite"]
#[test]
fn sr_001_vendor_new_is_id_and_name_only() {
    let vendor = Vendor::new(VendorId::new("vendor:acme"), "Acme");
    assert_eq!(vendor.id.as_str(), "vendor:acme");
    assert_eq!(vendor.name, "Acme");

    let json = serde_json::to_value(&vendor).unwrap();
    let mut keys = json_object_keys(&json);
    keys.sort();
    assert_eq!(keys, vec!["id".to_string(), "name".to_string()]);
    assert_eq!(json["id"], "vendor:acme");
    assert_eq!(json["name"], "Acme");
    for key in [
        "status",
        "criticality",
        "classification",
        "owner",
        "nextReview",
        "riskIds",
        "access",
        "approval",
        "securityRequirements",
        "onboardingReview",
        "reassessmentCadence",
        "history",
        "version",
        "exceptionIds",
        "contractDocumentRefs",
        "monitoringStatus",
        "issues",
        "suppliedServiceIds",
    ] {
        assert!(
            json.get(key).is_none(),
            "Vendor::new must omit operational key `{key}`"
        );
    }

    let golden: AssessmentDefinition = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    assert!(golden.vendors.is_empty());
    golden.validate().unwrap();
}

/// SR-002 found case: operational payload keys are dropped on decode.
#[ignore = "superseded by target suite"]
#[test]
fn sr_002_operational_payload_is_dropped_on_decode() {
    let payload = serde_json::json!({
        "id": "vendor:critical-saas",
        "name": "Critical SaaS",
        "classification": "hostedService",
        "criticality": "critical",
        "suppliedServiceIds": ["asset:payroll-saas"],
        "processingActivityIds": ["ropa:payroll"],
        "access": {
            "privileged": true,
            "dataAccess": true,
            "grants": [{
                "assetId": "asset:prod-db",
                "status": "active",
                "privileged": true
            }]
        },
        "owner": { "identity": "identity:alice" },
        "status": "active",
        "onboardingReview": { "kind": "onboarding", "performedAt": "2026-01-01T00:00:00Z" },
        "securityRequirements": [{
            "id": "sreq:contract-sec",
            "title": "contract security",
            "source": "contract"
        }],
        "approval": {
            "principal": { "role": "ciso" },
            "at": "2026-01-02T00:00:00Z",
            "decision": "approved"
        },
        "contractDocumentRefs": ["doc:msa-acme"],
        "obligationIds": ["obl:supplier-dpa"],
        "reassessmentCadence": { "intervalSeconds": 31536000 },
        "nextReview": "2020-01-01T00:00:00Z",
        "monitoringStatus": "healthy",
        "issues": [{ "id": "issue:1", "title": "open finding", "status": "open" }],
        "exceptionIds": ["exc:vendor-gap"],
        "riskIds": ["risk:supplier-concentration"],
        "controlIds": ["control.vendor.risk-review"],
        "evidenceRefs": ["evidence.vendor.risk-review"],
        "version": 4,
        "history": [{ "kind": "created", "version": 1 }]
    });
    let vendor: Vendor = serde_json::from_value(payload).unwrap();
    let out = serde_json::to_value(&vendor).unwrap();
    assert_eq!(out["id"], "vendor:critical-saas");
    assert_eq!(out["name"], "Critical SaaS");
    for key in [
        "classification",
        "criticality",
        "suppliedServiceIds",
        "access",
        "owner",
        "status",
        "onboardingReview",
        "securityRequirements",
        "approval",
        "nextReview",
        "riskIds",
        "history",
        "version",
        "exceptionIds",
        "contractDocumentRefs",
    ] {
        assert!(
            out.get(key).is_none(),
            "current Vendor drops unknown operational key `{key}`"
        );
    }
}

/// SR-003 found case: no lifecycle enum or transition table on Vendor.
#[ignore = "superseded by target suite"]
#[test]
fn sr_003_vendor_has_no_lifecycle_machine() {
    let src = vendor_src();
    for needle in [
        "SupplierLifecycleStatus",
        "UnderReview",
        "Terminating",
        "fn can_transition",
        "fn transition",
        "Candidate",
        "Terminated",
    ] {
        assert!(
            !src.contains(needle),
            "current vendor.rs must not contain `{needle}`"
        );
    }
    assert!(
        src.contains("Minimal vendor node for the compliance graph."),
        "module docs are the found-case product statement"
    );
}

/// SR-004 / SR-005 found case: no review clock, no current-review helper.
#[ignore = "superseded by target suite"]
#[test]
fn sr_004_sr_005_vendor_has_no_review_current_semantics() {
    let src = vendor_src();
    for needle in [
        "review_current",
        "next_review",
        "nextReview",
        "reassessment_cadence",
        "valid_until",
        "SupplierReview",
    ] {
        assert!(
            !src.contains(needle),
            "current vendor.rs must not contain `{needle}`"
        );
    }
    assert!(
        !validation_src().contains("validate_supplier_reviews_at")
            && !validation_src().contains("review_current"),
        "current validate() has no supplier review clock"
    );
}

/// SR-006 found case: no criticality / reduced-requirements policy.
#[ignore = "superseded by target suite"]
#[test]
fn sr_006_vendor_has_no_criticality_tier() {
    let src = vendor_src();
    assert!(
        !src.contains("criticality")
            && !src.contains("SupplierCriticality")
            && !src.contains("Critical"),
        "current Vendor has no risk-tier field"
    );
}

/// SR-007 found case: no privileged-access model.
#[ignore = "superseded by target suite"]
#[test]
fn sr_007_vendor_has_no_access_grants() {
    let src = vendor_src();
    assert!(
        !src.contains("privileged")
            && !src.contains("SupplierAccess")
            && !src.contains("data_access")
            && !src.contains("grants"),
        "current Vendor has no access/privileged model"
    );
}

/// SR-008 found case: termination with lingering access is not validated.
#[ignore = "superseded by target suite"]
#[test]
fn sr_008_terminated_payload_with_lingering_access_validates() {
    let mut assessment = empty_assessment();
    assessment.assets.push(Asset::new(
        AssetId::new("asset:prod-db"),
        AssetKind::Database,
        "prod",
    ));
    let vendor: Vendor = serde_json::from_value(serde_json::json!({
        "id": "vendor:gone",
        "name": "GoneCo",
        "status": "terminated",
        "access": {
            "privileged": true,
            "dataAccess": true,
            "grants": [{ "assetId": "asset:prod-db", "status": "active" }]
        }
    }))
    .unwrap();
    assessment.vendors.push(vendor);
    assessment
        .validate()
        .expect("current validate() does not inspect vendor access after termination");
    assert!(
        !vendor_src().contains("lingering") && !validation_src().contains("lingering"),
        "current IR has no lingering-access helper"
    );
}

/// SR-009 found case: missing contract security requirement is not a validation error.
#[ignore = "superseded by target suite"]
#[test]
fn sr_009_active_critical_vendor_without_contract_requirement_validates() {
    let mut assessment = empty_assessment();
    let vendor: Vendor = serde_json::from_value(serde_json::json!({
        "id": "vendor:no-contract",
        "name": "No Contract Co",
        "criticality": "critical",
        "status": "active",
        "securityRequirements": []
    }))
    .unwrap();
    assessment.vendors.push(vendor);
    assessment
        .validate()
        .expect("current validate() does not require contract security requirements");
    assert!(
        !vendor_src().contains("security_requirements")
            && !vendor_src().contains("SupplierSecurityRequirement"),
        "current Vendor has no security-requirement list"
    );
}

/// SR-010 found case: expired exceptions are stored facts; validation does not apply them to vendors.
#[ignore = "superseded by target suite"]
#[test]
fn sr_010_expired_exception_is_not_applied_to_vendor_review() {
    let mut assessment = empty_assessment();
    assessment
        .vendors
        .push(Vendor::new(VendorId::new("vendor:excepted"), "Excepted Co"));
    assessment.exceptions.push(Exception::new(
        ExceptionId::new("exc:vendor-excepted-expired"),
        "stale supplier exception",
    ));
    assessment.validate().unwrap();
    let src = validation_src();
    assert!(
        !src.contains("validate_supplier")
            && !src.contains("validate_supplier_reviews_at")
            && !src.contains("SupplierAssessmentExpired"),
        "no supplier-exception honesty / review-clock helper today"
    );
}

/// SR-011 found case: Vendor has no risk linkage; Risk is a four-field stub.
#[ignore = "superseded by target suite"]
#[test]
fn sr_011_vendor_has_no_risk_ids_and_risk_has_no_vendor_ids() {
    let vendor_json =
        serde_json::to_value(&Vendor::new(VendorId::new("vendor:link"), "Link")).unwrap();
    assert!(vendor_json.get("riskIds").is_none());
    let risk_json = serde_json::to_value(&Risk::new(
        RiskId::new("risk:supplier-concentration"),
        "supplier concentration",
        "single critical vendor",
    ))
    .unwrap();
    assert!(risk_json.get("vendorIds").is_none());

    let mut assessment = empty_assessment();
    assessment.vendors.push(
        serde_json::from_value(serde_json::json!({
            "id": "vendor:link",
            "name": "Link",
            "riskIds": ["risk:missing"]
        }))
        .unwrap(),
    );
    assessment
        .validate()
        .expect("dangling vendor→risk ids are ignored because they are not fields");

    assert!(!vendor_src().contains("risk_ids") && !vendor_src().contains("RiskId"));
}

/// SR-012 found case: evidence cannot be distinguished from acceptance because neither exists.
#[ignore = "superseded by target suite"]
#[test]
fn sr_012_vendor_has_no_approval_distinct_from_evidence() {
    let src = vendor_src();
    assert!(
        !src.contains("approval")
            && !src.contains("evidence_refs")
            && !src.contains("SupplierApproval"),
        "current Vendor has no approval or evidence-ref fields"
    );
}

/// SR-013 found case: duplicate VendorIds and dangling processors are silent.
#[ignore = "superseded by target suite"]
#[test]
fn sr_013_duplicate_vendor_ids_and_dangling_processors_are_silent() {
    let mut dupes = empty_assessment();
    let id = VendorId::new("vendor:same");
    dupes.vendors.push(Vendor::new(id.clone(), "first"));
    dupes.vendors.push(Vendor::new(id, "second"));
    dupes
        .validate()
        .expect("duplicate VendorIds are not an error today");

    let mut processors = empty_assessment();
    processors.processing_activities.push({
        let mut activity =
            ProcessingActivity::new(ProcessingActivityId::new("ropa:payroll"), "Payroll");
        activity.processors.push(VendorId::new("vendor:missing"));
        activity
    });
    processors
        .validate()
        .expect("dangling ProcessingActivity.processors are not an error today");

    let mut services = empty_assessment();
    services.vendors.push(
        serde_json::from_value(serde_json::json!({
            "id": "vendor:svc",
            "name": "Svc",
            "suppliedServiceIds": ["asset:missing"]
        }))
        .unwrap(),
    );
    services
        .validate()
        .expect("dangling suppliedServiceIds are dropped and not validated");

    assert!(
        !validation_src().contains("VendorId") && !validation_src().contains("vendors"),
        "validation.rs must not currently walk the vendor inventory"
    );
}

/// SR-014 found case: HasVendor is presence-only; catalog family already exists as TOML, not IR.
#[ignore = "superseded by target suite"]
#[test]
fn sr_014_has_vendor_is_presence_only_and_catalog_family_is_not_rewritten_here() {
    let evaluator = read_repo_file("crates/weeping-angel-assurance/src/applicability/evaluator.rs");
    assert!(
        evaluator.contains("ApplicabilityPredicate::HasVendor")
            && evaluator.contains("fn infer_vendors"),
        "HasVendor remains an applicability presence predicate"
    );
    assert!(
        evaluator.contains("if !context.vendors.is_empty()")
            && evaluator.contains("InventoryFamily::Vendors"),
        "infer_vendors is non-empty vs authoritative-empty"
    );

    let context = read_repo_file("crates/weeping-angel-assurance/src/applicability/context.rs");
    assert!(
        context.contains("fn vendor_matches")
            && context.contains("SubjectKind::Vendor")
            && context.contains("if !selector.tags.is_empty()"),
        "vendor_matches is id-only and ignores tags (no criticality selector)"
    );

    let controls = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    assert!(
        controls.contains("id = \"control.vendor.inventory\"")
            && controls.contains("id = \"control.vendor.risk-review\"")
            && controls.contains("id = \"control.vendor.security-requirements\""),
        "governance catalog already owns control.vendor.*"
    );
    let tests = read_repo_file("catalog/canonical/v1/tests/governance.toml");
    assert!(
        tests.contains("id = \"test.vendor.critical-risk-review-current\""),
        "catalog already owns the critical-review population test"
    );
}

/// SR-015: dual-suite names are listed in root Cargo.toml.
#[ignore = "superseded by target suite"]
#[test]
fn sr_015_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_supplier_risk_baseline")
            && toml.contains("sdd_supplier_risk_target")
            && toml.contains("tests/contracts/supplier_risk.baseline.rs")
            && toml.contains("tests/contracts/supplier_risk.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/supplier_risk.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/supplier_risk.target.rs")
            .is_file()
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn sr_assessment_vendors_default_empty() {
    let assessment = empty_assessment();
    assert!(assessment.vendors.is_empty());
    assessment.validate().unwrap();
}

#[ignore = "superseded by target suite"]
#[test]
fn sr_processing_activity_processors_default_empty() {
    let activity = ProcessingActivity::new(ProcessingActivityId::new("ropa:hr"), "HR");
    assert!(activity.processors.is_empty());
    let json = serde_json::to_value(&activity).unwrap();
    assert!(json.get("processors").is_none());
}

#[ignore = "superseded by target suite"]
#[test]
fn sr_ir_crate_has_no_obligation_or_controlled_document_or_treatment_plan_types() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("pub struct InterestedParty")
            && !ir.contains("pub struct Obligation")
            && !ir.contains("pub struct ControlledDocument")
            && !ir.contains("pub struct TreatmentPlan"),
        "Prompts 03/08/12 types are not landed; baseline must not pretend they exist"
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn sr_identity_and_asset_seams_exist_for_later_grants() {
    let _ = (
        Identity::new(IdentityId::new("identity:alice"), IdentityKind::User),
        Asset::new(AssetId::new("asset:svc"), AssetKind::Service, "payroll"),
        Control::new(
            ControlId::new("control.vendor.risk-review"),
            "Supplier risk review",
            "Every critical supplier has a current risk review.",
        ),
        ControlImplementation::new(
            ControlImplementationId::new("impl.vendor.risk-review"),
            ControlId::new("control.vendor.risk-review"),
        ),
    );
}
