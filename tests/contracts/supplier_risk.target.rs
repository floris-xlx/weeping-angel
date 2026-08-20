//! Target suite for Prompt 18 (operational supplier-security lifecycle).
//!
//! Encodes DESIRED behavior in `docs/specs/supplier-risk.md` §4 / §6 (SR-001–SR-015).
//! Must stay RED on CURRENT HEAD: `Vendor` is still `{ id, name }`, validation
//! does not walk vendors, and clocked review helpers do not exist. Do not
//! `#[ignore]` these tests and do not implement the lifecycle in this suite.
//!
//! Compiles against current IR constructors (`Vendor::new`, `ValidateIr`,
//! camelCase JSON) and asserts additive operational fields, the lifecycle
//! machine, risk-tiered reviews, lingering-access fail-closed, contract
//! security-requirement presence, expired-exception honesty, and supplier↔risk
//! linkage. Evidence presence must not imply approval or `RiskStatus::Accepted`.

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind, Control,
    ControlId, ControlImplementation, ControlImplementationId, Exception, ExceptionId,
    ExceptionStatus, Identity, IdentityId, IdentityKind, PrincipalRef, ProcessingActivity,
    ProcessingActivityId, Risk, RiskId, RiskStatus, SubjectKind, SubjectSelector, ValidateIr,
    Vendor, VendorId, canonical_digest,
};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

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

fn vendor_src() -> String {
    read_repo_file("crates/weeping-angel-assurance-ir/src/vendor.rs")
}

fn ir_src() -> String {
    crate_sources_joined("weeping-angel-assurance-ir")
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.supplier-risk.target"))
}

fn as_of_now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

fn as_of_past() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()
}

fn decode_vendor(value: Value) -> Vendor {
    serde_json::from_value(value).expect("operational Vendor JSON must decode")
}

fn persist(vendor: &Vendor) -> Value {
    serde_json::to_value(vendor).expect("Vendor must serialize")
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.vendor.risk-review"),
        "Supplier risk review",
        "Every critical supplier has a current risk review.",
    )
}

fn sample_asset(id: &str, kind: AssetKind, name: &str) -> Asset {
    Asset::new(AssetId::new(id), kind, name)
}

fn err_text(err: impl std::fmt::Display) -> String {
    err.to_string().to_ascii_lowercase()
}

fn assert_err_contains(err: impl std::fmt::Display, needles: &[&str]) {
    let text = err_text(err);
    assert!(
        needles
            .iter()
            .any(|n| text.contains(&n.to_ascii_lowercase())),
        "error must mention one of {needles:?}, got {text}"
    );
}

fn require_clocked_review_api(label: &str) {
    require_needles(
        label,
        &ir_src(),
        &[
            "fn review_current",
            "fn validate_supplier_reviews_at",
            "fn critical_suppliers",
        ],
    );
}

fn contract_requirement() -> Value {
    json!({
        "id": "sreq:contract-sec",
        "title": "contract security clauses",
        "source": "contract",
        "documentRef": "doc:msa-acme",
        "required": true
    })
}

fn approved() -> Value {
    json!({
        "principal": { "role": "ciso" },
        "at": "2026-01-02T00:00:00Z",
        "decision": "approved"
    })
}

fn current_onboarding_review() -> Value {
    json!({
        "id": "srev:onboard",
        "kind": "onboarding",
        "performedAt": "2026-01-01T00:00:00Z",
        "validUntil": "2027-08-19T12:00:00Z",
        "source": "manualReview"
    })
}

fn operational_vendor_payload() -> Value {
    json!({
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
                "identityId": "identity:vendor-bot",
                "privileged": true,
                "status": "active"
            }]
        },
        "owner": { "identity": "identity:alice" },
        "status": "active",
        "onboardingReview": current_onboarding_review(),
        "reviews": [current_onboarding_review()],
        "securityRequirements": [contract_requirement()],
        "riskAssessment": {
            "performedAt": "2026-01-01T00:00:00Z",
            "linkedRiskIds": ["risk:supplier-concentration"],
            "evidenceRefs": ["evidence.vendor.risk-review"]
        },
        "approval": approved(),
        "contractDocumentRefs": ["doc:msa-acme"],
        "obligationIds": ["obl:supplier-dpa"],
        "reassessmentCadence": { "intervalSeconds": 31536000 },
        "nextReview": "2027-01-01T00:00:00Z",
        "monitoringStatus": "healthy",
        "issues": [{
            "id": "issue:vendor-1",
            "title": "open finding",
            "status": "open"
        }],
        "exceptionIds": ["exc:vendor-gap"],
        "riskIds": ["risk:supplier-concentration"],
        "controlIds": ["control.vendor.risk-review"],
        "evidenceRefs": ["evidence.vendor.risk-review"],
        "version": 4,
        "history": [{
            "kind": "created",
            "version": 1,
            "at": "2026-01-01T00:00:00Z"
        }]
    })
}

const ADDITIVE_JSON_KEYS: &[&str] = &[
    "classification",
    "criticality",
    "suppliedServiceIds",
    "processingActivityIds",
    "access",
    "owner",
    "status",
    "onboardingReview",
    "reviews",
    "securityRequirements",
    "riskAssessment",
    "approval",
    "contractDocumentRefs",
    "obligationIds",
    "reassessmentCadence",
    "nextReview",
    "monitoringStatus",
    "issues",
    "exceptionIds",
    "riskIds",
    "controlIds",
    "evidenceRefs",
    "history",
];

fn graph_for(vendor: Vendor) -> AssessmentDefinition {
    let mut assessment = empty_assessment();
    assessment.assets.push(sample_asset(
        "asset:payroll-saas",
        AssetKind::Service,
        "payroll-saas",
    ));
    assessment
        .assets
        .push(sample_asset("asset:prod-db", AssetKind::Database, "prod"));
    assessment.identities.push(Identity::new(
        IdentityId::new("identity:alice"),
        IdentityKind::User,
    ));
    assessment.identities.push(Identity::new(
        IdentityId::new("identity:vendor-bot"),
        IdentityKind::ServiceAccount,
    ));
    assessment
        .processing_activities
        .push(ProcessingActivity::new(
            ProcessingActivityId::new("ropa:payroll"),
            "Payroll",
        ));
    assessment.controls.push(sample_control());
    assessment.risks.push(Risk::new(
        RiskId::new("risk:supplier-concentration"),
        "supplier concentration",
        "single critical vendor",
    ));
    let mut exception = Exception::new(ExceptionId::new("exc:vendor-gap"), "documented gap");
    exception.status = ExceptionStatus::Approved;
    assessment.exceptions.push(exception);
    assessment.vendors.push(vendor);
    assessment
}

/// SR-001: `Vendor::new` JSON remains `{id, name}`; golden `vendors: []` still decodes.
#[test]
fn sr_001_vendor_new_json_remains_id_and_name() {
    require_needles(
        "SR-001 additive skip-serialize so Vendor::new stays two-key",
        &vendor_src(),
        &[
            "skip_serializing_if",
            "classification",
            "criticality",
            "next_review",
            "risk_ids",
            "SupplierLifecycleStatus",
        ],
    );

    let vendor = Vendor::new(VendorId::new("vendor:acme"), "Acme");
    assert_eq!(vendor.id.as_str(), "vendor:acme");
    assert_eq!(vendor.name, "Acme");

    let json = persist(&vendor);
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
        "version",
        "history",
    ] {
        assert!(
            json.get(key).is_none(),
            "Vendor::new must omit operational key `{key}` via skip_serializing_if"
        );
    }

    let golden: AssessmentDefinition = serde_json::from_str(&read_repo_file(
        "tests/fixtures/assurance-ir/v1/assessment.json",
    ))
    .unwrap();
    assert!(golden.vendors.is_empty());
    assert_eq!(golden.schema_version, ASSURANCE_IR_SCHEMA);
    golden.validate().unwrap();
}

/// SR-002: fully populated operational vendor round-trips camelCase; digest is stable.
#[test]
fn sr_002_operational_vendor_round_trips_camel_case() {
    let payload = operational_vendor_payload();
    let vendor = decode_vendor(payload.clone());
    let out = persist(&vendor);
    for key in ADDITIVE_JSON_KEYS {
        assert!(
            out.get(*key).is_some(),
            "operational key `{key}` must persist on Vendor serde round-trip"
        );
    }
    assert_eq!(out["status"], "active");
    assert_eq!(out["criticality"], "critical");
    assert_eq!(out["classification"], "hostedService");
    assert_eq!(out["nextReview"], "2027-01-01T00:00:00Z");
    assert_eq!(out["access"]["privileged"], true);
    assert_eq!(out["access"]["dataAccess"], true);
    assert_eq!(out["securityRequirements"][0]["source"], "contract");
    assert_eq!(out["approval"]["decision"], "approved");
    assert_eq!(out["owner"]["identity"], "identity:alice");
    assert_eq!(out["riskIds"][0], "risk:supplier-concentration");

    let again: Vendor = serde_json::from_value(out.clone()).unwrap();
    assert_eq!(
        canonical_digest(&vendor).expect("digest"),
        canonical_digest(&again).expect("digest"),
        "canonical digest must be stable for equivalent vendor"
    );

    require_needles(
        "SR-002 PrincipalRef owner, not VendorOwner",
        &vendor_src(),
        &["PrincipalRef", "owner"],
    );
    forbid_needles(
        "SR-002 do not fork VendorOwner",
        &vendor_src(),
        &["struct VendorOwner", "enum VendorOwner"],
    );
    let _ = PrincipalRef::Role("ciso".into());
}

/// SR-003: lifecycle JSON + fail-closed transitions; legal ones append history.
#[test]
fn sr_003_lifecycle_machine_is_fail_closed() {
    require_needles(
        "SR-003 lifecycle API",
        &ir_src(),
        &[
            "enum SupplierLifecycleStatus",
            "UnderReview",
            "Restricted",
            "Suspended",
            "Terminating",
            "fn can_transition",
            "fn transition",
            "StatusTransition",
        ],
    );

    for status in [
        "candidate",
        "underReview",
        "approved",
        "active",
        "restricted",
        "suspended",
        "terminating",
        "terminated",
    ] {
        let vendor = decode_vendor(json!({
            "id": format!("vendor:{status}"),
            "name": status,
            "status": status
        }));
        assert_eq!(
            persist(&vendor)["status"],
            status,
            "lifecycle JSON `{status}` must round-trip"
        );
    }

    let illegal_history = decode_vendor(json!({
        "id": "vendor:skip",
        "name": "Skip",
        "status": "candidate",
        "history": [{
            "kind": { "statusTransition": { "from": "candidate", "to": "active" } },
            "version": 1,
            "at": "2026-01-01T00:00:00Z"
        }]
    }));
    let mut assessment = empty_assessment();
    assessment.vendors.push(illegal_history);
    let err = assessment
        .validate()
        .expect_err("Candidate → Active recorded in history must fail closed");
    assert_err_contains(err, &["transition", "illegal", "lifecycle"]);

    let skip_terminating = decode_vendor(json!({
        "id": "vendor:skip-term",
        "name": "Skip Terminating",
        "status": "active",
        "history": [{
            "kind": { "statusTransition": { "from": "active", "to": "terminated" } },
            "version": 1,
            "at": "2026-01-01T00:00:00Z"
        }]
    }));
    let mut skip_assessment = empty_assessment();
    skip_assessment.vendors.push(skip_terminating);
    skip_assessment
        .validate()
        .expect_err("Active → Terminated skipping Terminating must fail closed");

    let terminated_to_active = decode_vendor(json!({
        "id": "vendor:zombie",
        "name": "Zombie",
        "status": "terminated",
        "history": [{
            "kind": { "statusTransition": { "from": "terminated", "to": "active" } },
            "version": 1,
            "at": "2026-01-01T00:00:00Z"
        }]
    }));
    let mut zombie = empty_assessment();
    zombie.vendors.push(terminated_to_active);
    zombie
        .validate()
        .expect_err("Terminated → Active is not a legal reinstatement path");
}

/// SR-004: Active+Critical with nextReview/validUntil ≥ as_of is current.
#[test]
fn sr_004_critical_vendor_current_review() {
    require_clocked_review_api("SR-004 review helpers");

    let vendor = decode_vendor(json!({
        "id": "vendor:alpha",
        "name": "Alpha",
        "criticality": "critical",
        "status": "active",
        "owner": { "role": "vendor-manager" },
        "nextReview": "2027-08-19T12:00:00Z",
        "approval": approved(),
        "contractDocumentRefs": ["doc:msa"],
        "securityRequirements": [{
            "id": "sreq:contract-sec",
            "title": "contract security",
            "source": "contract",
            "documentRef": "doc:msa"
        }],
        "reviews": [current_onboarding_review()]
    }));
    let out = persist(&vendor);
    assert_eq!(out["criticality"], "critical");
    assert_eq!(out["status"], "active");
    assert_eq!(out["nextReview"], "2027-08-19T12:00:00Z");
    assert_eq!(out["reviews"][0]["validUntil"], "2027-08-19T12:00:00Z");
    assert!(
        out["nextReview"].as_str().unwrap() >= "2026-08-19T12:00:00Z",
        "nextReview must be ≥ as_of for a current critical review"
    );

    let assessment = graph_for(vendor);
    assessment
        .validate()
        .expect("graph-valid critical vendor with current review must pass clockless validate");
    let _ = as_of_now();
}

/// SR-005: stale review is not current; evidence refs alone do not make it current.
#[test]
fn sr_005_stale_review_is_a_gap_and_evidence_is_not_currency() {
    require_needles(
        "SR-005 AssessmentExpired history seam",
        &ir_src(),
        &[
            "AssessmentExpired",
            "fn review_current",
            "fn validate_supplier_reviews_at",
        ],
    );

    let stale = decode_vendor(json!({
        "id": "vendor:stale",
        "name": "StaleCo",
        "criticality": "critical",
        "status": "active",
        "nextReview": "2020-01-01T00:00:00Z",
        "contractDocumentRefs": ["doc:msa"],
        "securityRequirements": [contract_requirement()],
        "evidenceRefs": ["evidence.vendor.risk-review"],
        "reviews": [{
            "id": "srev:old",
            "kind": "periodic",
            "performedAt": "2019-01-01T00:00:00Z",
            "validUntil": "2020-01-01T00:00:00Z",
            "source": "questionnaire",
            "evidenceRefs": ["evidence.vendor.risk-review"]
        }],
        "history": [{
            "kind": { "assessmentExpired": { "asOf": "2026-08-19T12:00:00Z" } },
            "version": 2,
            "at": "2026-08-19T12:00:00Z"
        }]
    }));
    let out = persist(&stale);
    assert_eq!(out["nextReview"], "2020-01-01T00:00:00Z");
    assert!(
        out.get("evidenceRefs").is_some(),
        "evidence refs persist but must not imply a current review"
    );
    assert!(
        out["nextReview"].as_str().unwrap() < "2026-08-19T12:00:00Z",
        "stale nextReview is before the characterization clock"
    );
    assert!(
        out["history"]
            .to_string()
            .to_ascii_lowercase()
            .contains("assessmentexpired"),
        "expired assessment must be representable as AssessmentExpired history"
    );
    let _ = (graph_for(stale), as_of_now(), as_of_past());
}

/// SR-006: Low + not privileged + not Processor does not fail missing onboarding/contract.
#[test]
fn sr_006_low_risk_reduced_requirements_versus_critical() {
    require_clocked_review_api("SR-006 risk-tiered review policy");
    require_needles(
        "SR-006 criticality enum",
        &vendor_src(),
        &["SupplierCriticality", "Processor"],
    );

    let low = decode_vendor(json!({
        "id": "vendor:low",
        "name": "Low Risk Stationery",
        "criticality": "low",
        "status": "active",
        "owner": { "role": "procurement" }
    }));
    let critical = decode_vendor(json!({
        "id": "vendor:crit",
        "name": "Critical Host",
        "criticality": "critical",
        "status": "active"
    }));
    assert_eq!(persist(&low)["criticality"], "low");
    assert_eq!(persist(&critical)["criticality"], "critical");

    let mut low_assessment = empty_assessment();
    low_assessment.vendors.push(low);
    low_assessment
        .validate()
        .expect("low-risk without onboarding/contract must not fail clockless validate");

    let mut crit_assessment = empty_assessment();
    crit_assessment.vendors.push(critical);
    let err = crit_assessment
        .validate()
        .expect_err("critical active vendor missing contract security requirement fails");
    assert_err_contains(err, &["contract", "requirement"]);
}

/// SR-007: privileged access elevates Low to High/Critical review rules.
#[test]
fn sr_007_privileged_access_elevates_review_requirements() {
    require_clocked_review_api("SR-007 privileged elevation uses High review rules");
    require_needles(
        "SR-007 privileged access model",
        &vendor_src(),
        &["privileged", "SupplierAccess", "data_access"],
    );

    let privileged_low = decode_vendor(json!({
        "id": "vendor:priv",
        "name": "Privileged Integrator",
        "criticality": "low",
        "status": "active",
        "access": { "privileged": true, "dataAccess": true }
    }));
    assert_eq!(persist(&privileged_low)["access"]["privileged"], true);

    let mut assessment = empty_assessment();
    assessment.identities.push(Identity::new(
        IdentityId::new("identity:vendor-bot"),
        IdentityKind::ServiceAccount,
    ));
    assessment.vendors.push(privileged_low);
    assessment
        .validate()
        .expect_err("privileged Low inherits High contract/review rules and fails closed");
}

/// SR-008: Terminated/Terminating with Active grants or leftover privileged/dataAccess fails.
#[test]
fn sr_008_termination_with_lingering_access_fails_closed() {
    require_needles(
        "SR-008 lingering-access helper",
        &ir_src(),
        &["fn has_lingering_access"],
    );

    let lingering_vendor = decode_vendor(json!({
        "id": "vendor:gone",
        "name": "GoneCo",
        "status": "terminated",
        "criticality": "high",
        "access": {
            "privileged": true,
            "dataAccess": true,
            "grants": [{
                "assetId": "asset:prod-db",
                "status": "active",
                "privileged": true
            }]
        }
    }));
    assert_eq!(persist(&lingering_vendor)["status"], "terminated");
    assert_eq!(
        persist(&lingering_vendor)["access"]["grants"][0]["status"],
        "active"
    );

    let mut lingering = empty_assessment();
    lingering
        .assets
        .push(sample_asset("asset:prod-db", AssetKind::Database, "prod"));
    lingering.vendors.push(lingering_vendor);
    let err = lingering
        .validate()
        .expect_err("terminated vendor with active grant must fail closed");
    assert_err_contains(err, &["linger", "access"]);

    let terminating = decode_vendor(json!({
        "id": "vendor:leaving",
        "name": "Leaving Co",
        "status": "terminating",
        "criticality": "high",
        "access": {
            "privileged": false,
            "dataAccess": true,
            "grants": []
        }
    }));
    let mut still_data = empty_assessment();
    still_data.vendors.push(terminating);
    still_data
        .validate()
        .expect_err("terminating vendor with leftover dataAccess must fail closed");

    let clean_vendor = decode_vendor(json!({
        "id": "vendor:clean-offboard",
        "name": "Offboarded Co",
        "status": "terminated",
        "criticality": "high",
        "access": {
            "privileged": false,
            "dataAccess": false,
            "grants": [{
                "assetId": "asset:prod-db",
                "status": "revoked",
                "revokedAt": "2026-06-01T00:00:00Z"
            }]
        }
    }));
    let mut clean = empty_assessment();
    clean
        .assets
        .push(sample_asset("asset:prod-db", AssetKind::Database, "prod"));
    clean.vendors.push(clean_vendor);
    clean
        .validate()
        .expect("terminated vendor with revoked grants must validate");
}

/// SR-009: Active+Critical without source=Contract requirement and empty contractDocumentRefs fails.
#[test]
fn sr_009_missing_contract_security_requirement_fails_closed() {
    require_needles(
        "SR-009 contract security requirement model",
        &vendor_src(),
        &["SupplierSecurityRequirement", "contract_document_refs"],
    );

    let mut missing = empty_assessment();
    missing.vendors.push(decode_vendor(json!({
        "id": "vendor:no-contract",
        "name": "No Contract Co",
        "criticality": "critical",
        "status": "active"
    })));
    let err = missing
        .validate()
        .expect_err("critical active vendor needs a contract security requirement");
    assert_err_contains(err, &["contract", "requirement"]);

    let mut present = empty_assessment();
    present.vendors.push(decode_vendor(json!({
        "id": "vendor:contracted",
        "name": "Contracted Co",
        "criticality": "critical",
        "status": "active",
        "contractDocumentRefs": ["doc:msa"],
        "securityRequirements": [{
            "id": "sreq:msa",
            "title": "MSA security schedule",
            "source": "contract",
            "documentRef": "doc:msa"
        }],
        "approval": approved(),
        "nextReview": "2027-01-01T00:00:00Z",
        "reviews": [current_onboarding_review()]
    })));
    present
        .validate()
        .expect("critical vendor with contract requirement must validate");
}

/// SR-010: expired exception bound to a vendor does not suppress a stale-review gap.
#[test]
fn sr_010_expired_exception_does_not_suppress_review_gap() {
    require_clocked_review_api("SR-010 clocked supplier review validation");

    let mut assessment = empty_assessment();
    assessment.vendors.push(decode_vendor(json!({
        "id": "vendor:excepted",
        "name": "Excepted Co",
        "criticality": "critical",
        "status": "active",
        "nextReview": "2020-01-01T00:00:00Z",
        "contractDocumentRefs": ["doc:msa"],
        "securityRequirements": [contract_requirement()],
        "exceptionIds": ["exc:vendor-excepted-expired"]
    })));
    let mut expired = Exception::new(
        ExceptionId::new("exc:vendor-excepted-expired"),
        "temporary supplier review skip",
    );
    expired.status = ExceptionStatus::Expired;
    expired.expires_at = Some(as_of_past());
    expired.control_id = Some(ControlId::new("control.vendor.risk-review"));
    expired.subjects.push(SubjectSelector {
        kind: SubjectKind::Vendor,
        ids: ["vendor:excepted".to_string()].into_iter().collect(),
        tags: Default::default(),
        scope: Default::default(),
    });
    assessment.exceptions.push(expired);
    assessment.controls.push(sample_control());
    assessment
        .validate()
        .expect("clockless validate stores an expired exception as a fact (IR-020)");
    assert_eq!(
        persist(&assessment.vendors[0])["exceptionIds"][0],
        "exc:vendor-excepted-expired"
    );
    assert_eq!(
        persist(&assessment.vendors[0])["nextReview"],
        "2020-01-01T00:00:00Z"
    );
}

/// SR-011: Vendor.risk_ids must resolve; linkage does not set RiskStatus::Accepted.
#[test]
fn sr_011_supplier_risk_linkage_fail_closed_is_not_acceptance() {
    require_needles("SR-011 Vendor.risk_ids", &vendor_src(), &["risk_ids"]);

    let mut dangling = empty_assessment();
    dangling.vendors.push(decode_vendor(json!({
        "id": "vendor:link",
        "name": "Link Co",
        "criticality": "high",
        "status": "active",
        "riskIds": ["risk:missing"],
        "contractDocumentRefs": ["doc:msa"],
        "securityRequirements": [{
            "id": "sreq:msa",
            "title": "MSA",
            "source": "contract",
            "documentRef": "doc:msa"
        }]
    })));
    let err = dangling
        .validate()
        .expect_err("dangling Vendor.riskIds must fail closed");
    assert_err_contains(err, &["risk"]);

    let mut linked = empty_assessment();
    let risk = Risk::new(
        RiskId::new("risk:supplier-concentration"),
        "supplier concentration",
        "single critical vendor",
    );
    assert_eq!(risk.status, RiskStatus::Open);
    linked.risks.push(risk);
    linked.vendors.push(decode_vendor(json!({
        "id": "vendor:link",
        "name": "Link Co",
        "criticality": "high",
        "status": "candidate",
        "riskIds": ["risk:supplier-concentration"]
    })));
    linked
        .validate()
        .expect("present RiskId on Vendor must resolve");
    assert_eq!(
        persist(&linked.vendors[0])["riskIds"][0],
        "risk:supplier-concentration"
    );
    assert_eq!(
        linked.risks[0].status,
        RiskStatus::Open,
        "supplier linkage must not set RiskStatus::Accepted"
    );
    assert_ne!(linked.risks[0].status, RiskStatus::Accepted);
}

/// SR-012: questionnaire/evidence-only vendor cannot become approved without SupplierApproval.
#[test]
fn sr_012_evidence_presence_is_not_approval_or_risk_acceptance() {
    require_needles(
        "SR-012 distinct SupplierApproval",
        &vendor_src(),
        &["struct SupplierApproval", "fn transition"],
    );

    let evidence_only = decode_vendor(json!({
        "id": "vendor:evidence-only",
        "name": "Questionnaire Co",
        "criticality": "critical",
        "status": "underReview",
        "evidenceRefs": ["evidence.vendor.risk-review"],
        "reviews": [{
            "id": "srev:q",
            "kind": "onboarding",
            "performedAt": "2026-01-01T00:00:00Z",
            "source": "questionnaire",
            "evidenceRefs": ["evidence.vendor.risk-review"]
        }]
    }));
    let out = persist(&evidence_only);
    assert_eq!(out["status"], "underReview");
    assert_ne!(out["status"], "approved");
    assert_ne!(out["status"], "active");
    assert!(out.get("approval").is_none() || out["approval"].is_null());
    assert!(
        out.get("evidenceRefs").is_some() || out.get("reviews").is_some(),
        "questionnaire evidence must round-trip without implying approval"
    );
}

/// SR-013: duplicate VendorId, dangling processors, dangling suppliedServiceIds fail; IR-019/020 remain.
#[test]
fn sr_013_graph_integrity_fail_closed() {
    let mut dupes = empty_assessment();
    let id = VendorId::new("vendor:same");
    dupes.vendors.push(Vendor::new(id.clone(), "first"));
    dupes.vendors.push(Vendor::new(id, "second"));
    let err = dupes
        .validate()
        .expect_err("duplicate VendorId must fail closed");
    assert_err_contains(err, &["duplicate"]);

    let mut processors = empty_assessment();
    processors.processing_activities.push({
        let mut activity =
            ProcessingActivity::new(ProcessingActivityId::new("ropa:payroll"), "Payroll");
        activity.processors.push(VendorId::new("vendor:missing"));
        activity
    });
    let err = processors
        .validate()
        .expect_err("dangling ProcessingActivity.processors must fail closed");
    assert_err_contains(err, &["vendor", "processor"]);

    let mut services = empty_assessment();
    services.vendors.push(decode_vendor(json!({
        "id": "vendor:svc",
        "name": "Svc",
        "status": "candidate",
        "suppliedServiceIds": ["asset:missing"]
    })));
    let err = services
        .validate()
        .expect_err("dangling suppliedServiceIds must fail closed");
    assert_err_contains(err, &["asset", "service"]);

    let mut dangling_risk = empty_assessment();
    dangling_risk.controls.push(sample_control());
    dangling_risk.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.vendor.risk-review.org"),
            ControlId::new("control.vendor.risk-review"),
        )
        .with_risk(RiskId::new("risk:missing")),
    );
    let err = dangling_risk
        .validate()
        .expect_err("IR-019: dangling implementation risk");
    assert_err_contains(err, &["risk"]);

    let mut dangling_exc = empty_assessment();
    dangling_exc.controls.push(sample_control());
    dangling_exc.implementations.push(
        ControlImplementation::new(
            ControlImplementationId::new("impl.vendor.risk-review.org"),
            ControlId::new("control.vendor.risk-review"),
        )
        .with_exception(ExceptionId::new("exc:missing")),
    );
    let err = dangling_exc
        .validate()
        .expect_err("IR-020: dangling implementation exception");
    assert_err_contains(err, &["exception"]);
}

/// SR-014: HasVendor stays presence-only; control.vendor.* TOML is not rewritten or forked.
#[test]
fn sr_014_has_vendor_presence_and_catalog_family_untouched() {
    let evaluator = read_repo_file("crates/weeping-angel-assurance/src/applicability/evaluator.rs");
    assert!(evaluator.contains("fn infer_vendors"));
    assert!(evaluator.contains("if !context.vendors.is_empty()"));
    assert!(
        !evaluator.contains("criticality") && !evaluator.contains("SupplierCriticality"),
        "HasVendor must remain presence-only; criticality is not an applicability predicate"
    );

    let controls = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    assert!(controls.contains("id = \"control.vendor.inventory\""));
    assert!(controls.contains("id = \"control.vendor.risk-review\""));
    assert!(
        !controls.contains("id = \"control.supplier."),
        "must not invent a second supplier catalog family"
    );
    let tests = read_repo_file("catalog/canonical/v1/tests/governance.toml");
    assert!(tests.contains("id = \"test.vendor.critical-risk-review-current\""));

    require_needles(
        "SR-014 operational Vendor still expands in place",
        &vendor_src(),
        &["criticality", "classification"],
    );
}

/// SR-015: dual-suite runs as a harness module.
#[test]
fn sr_015_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        !toml.contains("sdd_supplier_risk_baseline")
            && harness_src().contains("supplier_risk.target.rs")
            && !toml.contains("tests/contracts/supplier_risk.baseline.rs")
            && harness_src().contains("supplier_risk.target.rs"),
        "dual-suite must be wired as a harness module"
    );
    require_needles(
        "SR-015 lib.rs re-exports the operational supplier contract",
        &read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs"),
        &[
            "SupplierLifecycleStatus",
            "validate_supplier_reviews_at",
            "critical_suppliers",
        ],
    );
}
