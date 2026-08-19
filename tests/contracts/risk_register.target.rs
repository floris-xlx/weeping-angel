//! Target suite for Prompt 06 (operational risk register).
//!
//! Encodes DESIRED behavior in `docs/specs/risk-register.md` §4 / §6 (RR-001–RR-015)
//! and ADR 0005. Must stay RED on the current four-field `Risk` stub. Do not
//! `#[ignore]` these tests and do not implement the register in this suite.
//!
//! Compiles against current IR (`Risk::new`, `ValidateIr`, camelCase JSON) and
//! asserts additive operational fields, status machine, graph integrity, Prompt 05
//! scoring consumption, finding N:N contributors, and history/supersession.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};
use weeping_angel::finding::Finding;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind, Control, ControlId,
    ControlImplementation, ControlImplementationId, EvidenceRequirement, EvidenceRequirementId,
    EvidenceType, Identity, IdentityId, IdentityKind, PrincipalRef, ProcessingActivity,
    ProcessingActivityId, Risk, RiskId, RiskStatus, ValidateIr, Vendor, VendorId, canonical_digest,
};
use weeping_angel_control_test::Effectiveness;

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

fn golden_risk_json() -> String {
    read_repo_file("tests/fixtures/assurance-ir/v1/risk.json")
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.risk-register.target"))
}

fn sample_control() -> Control {
    Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    )
}

fn sample_asset() -> Asset {
    Asset::new(
        AssetId::new("asset:repo:source"),
        AssetKind::Repository,
        "source-of-record",
    )
}

fn as_of() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

fn round_trip_json(value: &Value) -> Value {
    let risk: Risk = serde_json::from_value(value.clone()).unwrap();
    serde_json::to_value(&risk).unwrap()
}

fn operational_payload() -> Value {
    json!({
        "id": "risk:source-tamper",
        "title": "Source tampering",
        "description": "Unauthorized change to the source of record.",
        "status": "open",
        "scenario": "attacker tampers with the source of record",
        "threat": "insider",
        "weaknessRefs": ["CWE-284", "CWE-345"],
        "assetIds": ["asset:repo:source"],
        "processingActivityIds": ["ropa:source-control"],
        "vendorIds": ["vendor:git-host"],
        "cia": { "confidentiality": 2, "integrity": 5, "availability": 1 },
        "likelihood": { "levelId": "possible" },
        "impact": { "levelId": "major" },
        "inherentScore": { "kind": "qualitative", "cellId": "possible-major" },
        "inherentRating": { "methodologyId": "meth.isms.v1", "revision": 1, "ratingId": "high" },
        "methodologyVersion": "meth.isms.v1:1",
        "owner": { "identity": "identity:alice" },
        "source": "finding",
        "discoveredAt": "2026-01-01T00:00:00Z",
        "reviewCadence": { "intervalSeconds": 7776000 },
        "nextReview": "2026-12-01T00:00:00Z",
        "treatmentId": "treat:source-tamper",
        "controlIds": ["control.access.mfa"],
        "evidenceRefs": ["evidence.req.source-integrity"],
        "findingRefs": ["finding:unprotected-branch", "finding:unsigned-commits"],
        "tags": ["integrity", "isms"],
        "classification": "confidential",
        "version": 1,
        "supersedes": "risk:source-tamper-v0",
        "history": [
            {
                "version": 1,
                "at": "2026-01-01T00:00:00Z",
                "kind": "created"
            }
        ]
    })
}

const ADDITIVE_JSON_KEYS: &[&str] = &[
    "scenario",
    "threat",
    "weaknessRefs",
    "assetIds",
    "processingActivityIds",
    "vendorIds",
    "cia",
    "likelihood",
    "impact",
    "inherentScore",
    "inherentRating",
    "methodologyVersion",
    "owner",
    "source",
    "discoveredAt",
    "reviewCadence",
    "nextReview",
    "treatmentId",
    "controlIds",
    "evidenceRefs",
    "findingRefs",
    "tags",
    "classification",
    "version",
    "supersedes",
    "history",
];

/// RR-001: old minimal `risk.json` still decodes; missing additive keys default empty.
#[test]
fn rr_001_old_minimal_fixture_decodes_with_empty_additive_defaults() {
    let risk: Risk = serde_json::from_str(&golden_risk_json()).unwrap();
    assert_eq!(risk.id.as_str(), "risk:source-tamper");
    assert_eq!(risk.title, "Source tampering");
    assert_eq!(
        risk.description,
        "Unauthorized change to the source of record."
    );
    assert_eq!(risk.status, RiskStatus::Open);

    let out = serde_json::to_value(&risk).unwrap();
    for key in ADDITIVE_JSON_KEYS {
        assert!(
            out.get(*key).is_none() || out[*key] == json!([]) || out[*key] == json!(null),
            "missing additive key `{key}` must default empty/omitted on old fixtures, got {:?}",
            out.get(*key)
        );
    }

    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    require_needles(
        "RR-001 additive fields with serde defaults",
        &risk_src,
        &[
            "scenario",
            "threat",
            "weakness_refs",
            "asset_ids",
            "treatment_id",
            "finding_refs",
            "next_review",
            "serde(default)",
        ],
    );
}

/// RR-002: `Risk::new` remains; default status Open; constructor JSON omits owner / treatment / residual.
#[test]
fn rr_002_risk_new_stays_minimal_and_defaults_open() {
    let risk = Risk::new(
        RiskId::new("risk:org-1"),
        "supplier concentration",
        "single critical vendor",
    );
    assert_eq!(risk.status, RiskStatus::Open);
    assert_eq!(risk.id.as_str(), "risk:org-1");

    let json = serde_json::to_value(&risk).unwrap();
    assert_eq!(json["status"], "open");
    assert!(json.get("owner").is_none());
    assert!(json.get("treatment").is_none());
    assert!(json.get("treatmentId").is_none());
    assert!(json.get("residualScore").is_none());
    assert!(json.get("residualRating").is_none());

    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    require_needles(
        "RR-002 constructor keeps additive defaults",
        &risk_src,
        &[
            "pub fn new",
            "version",
            "skip_serializing_if",
            "owner",
            "treatment_id",
            "residual_score",
        ],
    );
}

/// RR-003: fully populated operational Risk serde round-trips; digest stable under BTree ordering.
#[test]
fn rr_003_fully_populated_operational_risk_round_trips_with_stable_digest() {
    let payload = operational_payload();
    let out = round_trip_json(&payload);
    for key in [
        "scenario",
        "threat",
        "weaknessRefs",
        "assetIds",
        "controlIds",
        "findingRefs",
        "owner",
        "methodologyVersion",
        "inherentScore",
        "inherentRating",
        "history",
        "tags",
    ] {
        assert!(
            out.get(key).is_some(),
            "operational key `{key}` must survive serde round-trip"
        );
        assert_eq!(out[key], payload[key], "round-trip mismatch for `{key}`");
    }

    let mut a = payload.clone();
    let mut b = payload;
    a["tags"] = json!(["isms", "integrity"]);
    b["tags"] = json!(["integrity", "isms"]);
    let risk_a: Risk = serde_json::from_value(a).unwrap();
    let risk_b: Risk = serde_json::from_value(b).unwrap();
    let out_a = serde_json::to_value(&risk_a).unwrap();
    assert!(
        out_a.get("tags").is_some(),
        "tags must persist so BTree ordering is observable"
    );
    assert_eq!(
        canonical_digest(&risk_a).unwrap(),
        canonical_digest(&risk_b).unwrap(),
        "equivalent BTree tag ordering must not change canon/v1 digest"
    );
}

/// RR-004: Draft | Open | UnderTreatment | Accepted | Mitigated | Closed | Retired.
#[test]
fn rr_004_risk_status_includes_draft_under_treatment_and_retired() {
    assert_eq!(RiskStatus::default(), RiskStatus::Open);
    assert_eq!(
        serde_json::from_str::<RiskStatus>("\"open\"").unwrap(),
        RiskStatus::Open
    );

    for (raw, label) in [
        ("\"draft\"", "Draft"),
        ("\"underTreatment\"", "UnderTreatment"),
        ("\"accepted\"", "Accepted"),
        ("\"mitigated\"", "Mitigated"),
        ("\"closed\"", "Closed"),
        ("\"retired\"", "Retired"),
    ] {
        let decoded: RiskStatus = serde_json::from_str(raw)
            .unwrap_or_else(|e| panic!("{label} must decode from {raw}: {e}"));
        let encoded = serde_json::to_string(&decoded).unwrap();
        assert_eq!(encoded, raw, "{label} must round-trip as {raw}");
    }

    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    require_needles(
        "RR-004 status variants",
        &risk_src,
        &["Draft", "UnderTreatment", "Retired"],
    );
}

/// RR-005: illegal transitions fail; legal transitions append history.
#[test]
fn rr_005_illegal_transitions_fail_and_legal_ones_append_history() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    require_needles(
        "RR-005 transition API",
        &risk_src,
        &["fn can_transition", "fn transition", "StatusTransition"],
    );

    let illegal = json!({
        "id": "risk:illegal",
        "title": "t",
        "description": "d",
        "status": "mitigated",
        "history": [{
            "version": 2,
            "at": "2026-01-02T00:00:00Z",
            "kind": { "statusTransition": { "from": "open", "to": "mitigated" } }
        }]
    });
    let mut assessment = empty_assessment();
    assessment
        .risks
        .push(serde_json::from_value(illegal).unwrap());
    let err = assessment
        .validate()
        .expect_err("Open → Mitigated must fail closed");
    assert!(
        !err.to_string().is_empty(),
        "illegal transition error must be deterministic"
    );

    for (from, to) in [("open", "closed"), ("draft", "closed"), ("retired", "open")] {
        let payload = json!({
            "id": format!("risk:{from}-to-{to}"),
            "title": "t",
            "description": "d",
            "status": to,
            "history": [{
                "version": 2,
                "at": "2026-01-02T00:00:00Z",
                "kind": { "statusTransition": { "from": from, "to": to } }
            }]
        });
        let mut a = empty_assessment();
        a.risks.push(serde_json::from_value(payload).unwrap());
        a.validate()
            .expect_err(&format!("{from} → {to} must be illegal"));
    }

    let legal = json!({
        "id": "risk:legal",
        "title": "t",
        "description": "d",
        "status": "underTreatment",
        "history": [{
            "version": 2,
            "at": "2026-01-02T00:00:00Z",
            "kind": { "statusTransition": { "from": "open", "to": "underTreatment" } }
        }]
    });
    let risk: Risk = serde_json::from_value(legal).unwrap();
    let out = serde_json::to_value(&risk).unwrap();
    assert_eq!(out["status"], "underTreatment");
    assert!(
        out.get("history")
            .and_then(|h| h.as_array())
            .is_some_and(|h| !h.is_empty()),
        "legal Open → UnderTreatment must retain history"
    );
}

/// RR-006: dangling AssetId / ControlId / treatmentId on a risk fail validate().
#[test]
fn rr_006_dangling_asset_control_and_treatment_refs_fail_closed() {
    let mut dangling_asset = empty_assessment();
    dangling_asset.risks.push(
        serde_json::from_value(json!({
            "id": "risk:orphan-asset",
            "title": "t",
            "description": "d",
            "status": "open",
            "assetIds": ["asset:missing"]
        }))
        .unwrap(),
    );
    dangling_asset
        .validate()
        .expect_err("dangling AssetId on a risk must fail validate()");

    let mut dangling_control = empty_assessment();
    dangling_control.controls.push(sample_control());
    dangling_control.risks.push(
        serde_json::from_value(json!({
            "id": "risk:orphan-control",
            "title": "t",
            "description": "d",
            "status": "open",
            "controlIds": ["control.missing"]
        }))
        .unwrap(),
    );
    dangling_control
        .validate()
        .expect_err("dangling ControlId on a risk must fail validate()");

    let mut dangling_treatment = empty_assessment();
    dangling_treatment.risks.push(
        serde_json::from_value(json!({
            "id": "risk:orphan-treatment",
            "title": "t",
            "description": "d",
            "status": "open",
            "treatmentId": "treat:missing"
        }))
        .unwrap(),
    );
    dangling_treatment
        .validate()
        .expect_err("Some(treatmentId) with no treatment record must fail");

    let mut dangling_evidence = empty_assessment();
    dangling_evidence.risks.push(
        serde_json::from_value(json!({
            "id": "risk:orphan-evidence",
            "title": "t",
            "description": "d",
            "status": "open",
            "evidenceRefs": ["evidence.req.missing"]
        }))
        .unwrap(),
    );
    dangling_evidence
        .validate()
        .expect_err("dangling EvidenceRequirementId on a risk must fail validate()");
}

/// RR-007: IR-019 still fails; duplicate RiskId fails.
#[test]
fn rr_007_ir_019_and_duplicate_risk_ids_fail_closed() {
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
    dupes.risks.push(Risk::new(id, "second", "second copy"));
    dupes
        .validate()
        .expect_err("duplicate RiskId in assessment.risks must fail");
}

/// RR-008: review_overdue(as_of) iff nextReview < as_of; unscheduled is not overdue.
#[test]
fn rr_008_review_overdue_and_clocked_validation() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    require_needles(
        "RR-008 review API",
        &risk_src,
        &["fn review_overdue", "next_review", "review_cadence"],
    );
    let validation_src = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    require_needles(
        "RR-008 clocked review validation",
        &validation_src,
        &["validate_risk_reviews"],
    );

    let unscheduled = Risk::new(RiskId::new("risk:unscheduled"), "t", "d");
    let unscheduled_json = serde_json::to_value(&unscheduled).unwrap();
    assert!(
        unscheduled_json.get("nextReview").is_none(),
        "unscheduled Risk::new must omit nextReview"
    );

    let overdue: Risk = serde_json::from_value(json!({
        "id": "risk:overdue",
        "title": "t",
        "description": "d",
        "status": "open",
        "nextReview": "2020-01-01T00:00:00Z"
    }))
    .unwrap();
    let overdue_json = serde_json::to_value(&overdue).unwrap();
    let next = overdue_json["nextReview"]
        .as_str()
        .expect("nextReview must round-trip so overdue can be evaluated");
    let next_at = DateTime::parse_from_rfc3339(next)
        .unwrap()
        .with_timezone(&Utc);
    assert!(
        next_at < as_of(),
        "stored nextReview must be in the past relative to as_of"
    );

    let future: Risk = serde_json::from_value(json!({
        "id": "risk:future",
        "title": "t",
        "description": "d",
        "status": "open",
        "nextReview": "2027-01-01T00:00:00Z"
    }))
    .unwrap();
    let future_json = serde_json::to_value(&future).unwrap();
    let future_next = future_json["nextReview"]
        .as_str()
        .expect("scheduled future nextReview must persist");
    assert!(
        DateTime::parse_from_rfc3339(future_next)
            .unwrap()
            .with_timezone(&Utc)
            >= as_of()
    );

    let mut clocked = empty_assessment();
    clocked.risks.push(overdue);
    let validation_src = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    assert!(
        validation_src.contains("Closed") && validation_src.contains("Retired"),
        "clocked review validation must spare terminal Closed/Retired statuses"
    );
}

/// RR-009: inherent score/rating derived from raw inputs + methodology version; no hardcoded 5×5.
#[test]
fn rr_009_inherent_score_is_derived_via_methodology_not_a_hardcoded_matrix() {
    let ir_src = crate_sources_joined("weeping-angel-assurance-ir");
    let has_prompt05 = ir_src.contains("fn score_risk") || ir_src.contains("fn score_inherent");
    assert!(
        has_prompt05,
        "inherent scoring must call Prompt 05 score_risk or a thin score_inherent adapter"
    );

    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    forbid_needles(
        "RR-009 no second scoring model in risk.rs",
        &risk_src,
        &[
            "LIKELIHOOD_MAX = 5",
            "IMPACT_MAX = 5",
            "enum RiskRating",
            "RiskRating::High",
        ],
    );

    let rating_only = json!({
        "id": "risk:rating-only",
        "title": "t",
        "description": "d",
        "status": "open",
        "inherentRating": { "ratingId": "high" }
    });
    let mut assessment = empty_assessment();
    assessment
        .risks
        .push(serde_json::from_value(rating_only).unwrap());
    assessment
        .validate()
        .expect_err("derived rating must not be the only authoring input");

    let raw = json!({
        "id": "risk:scored",
        "title": "t",
        "description": "d",
        "status": "open",
        "likelihood": { "levelId": "possible" },
        "impact": { "levelId": "major" },
        "methodologyVersion": "meth.isms.v1:1"
    });
    let a = round_trip_json(&raw);
    let b = round_trip_json(&raw);
    assert_eq!(a["likelihood"], raw["likelihood"]);
    assert_eq!(a["impact"], raw["impact"]);
    assert_eq!(a["methodologyVersion"], raw["methodologyVersion"]);
    assert_eq!(
        a.get("inherentScore"),
        b.get("inherentScore"),
        "equal raw inputs + methodology version must yield equal inherent scores"
    );
}

/// RR-010: residualScore/residualRating are optional placeholders; Effective ≠ residual zero.
#[test]
fn rr_010_residual_fields_are_optional_placeholders() {
    let omitted = Risk::new(RiskId::new("risk:no-residual"), "t", "d");
    let omitted_json = serde_json::to_value(&omitted).unwrap();
    assert!(omitted_json.get("residualScore").is_none());
    assert!(omitted_json.get("residualRating").is_none());

    let with_placeholder = json!({
        "id": "risk:placeholder-residual",
        "title": "t",
        "description": "d",
        "status": "open",
        "residualScore": { "kind": "placeholder", "value": 4 },
        "residualRating": { "ratingId": "medium" }
    });
    let out = round_trip_json(&with_placeholder);
    assert_eq!(out["residualScore"], with_placeholder["residualScore"]);
    assert_eq!(out["residualRating"], with_placeholder["residualRating"]);

    let _ = Effectiveness::Effective;
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    forbid_needles(
        "RR-010 residual is not control-effectiveness math",
        &risk_src,
        &[
            "Effectiveness::Effective",
            "residual_score = 0",
            "residualScore\": 0",
        ],
    );
}

/// RR-011: Finding refs are N:N; no From<Finding> for Risk; no Finding struct in IR.
#[test]
fn rr_011_finding_refs_are_n_to_n_without_auto_promotion() {
    let finding = Finding::builder("recon", "unprotected-branch")
        .title("Unprotected default branch")
        .description("scanner output is not an ISMS risk")
        .build();
    assert_eq!(finding.id, "unprotected-branch");

    let ir_src = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir_src.contains("pub struct Finding "),
        "weeping-angel-assurance-ir must not declare a Finding document"
    );
    forbid_needles(
        "RR-011 no auto-promotion",
        &ir_src,
        &["impl From<Finding> for Risk", "From<Finding> for Risk"],
    );
    assert!(
        ir_src.contains("FindingRef") || ir_src.contains("typed_id!(FindingRef)"),
        "IR must expose FindingRef as a typed id, not a Finding struct"
    );

    let one_risk_two_findings = json!({
        "id": "risk:aggregate",
        "title": "t",
        "description": "d",
        "status": "open",
        "findingRefs": ["finding:unprotected-branch", "finding:unsigned-commits"]
    });
    let out = round_trip_json(&one_risk_two_findings);
    assert_eq!(
        out["findingRefs"],
        json!(["finding:unprotected-branch", "finding:unsigned-commits"])
    );

    let shared = json!("finding:unprotected-branch");
    let r1 = round_trip_json(&json!({
        "id": "risk:one",
        "title": "t",
        "description": "d",
        "status": "open",
        "findingRefs": [shared]
    }));
    let r2 = round_trip_json(&json!({
        "id": "risk:two",
        "title": "t",
        "description": "d",
        "status": "open",
        "findingRefs": [shared]
    }));
    assert_eq!(r1["findingRefs"], json!([shared]));
    assert_eq!(r2["findingRefs"], json!([shared]));
}

/// RR-012: revise/transition preserves prior state; version increments; history is not cleared.
#[test]
fn rr_012_revise_and_transition_preserve_history_and_increment_version() {
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    require_needles(
        "RR-012 history API",
        &risk_src,
        &[
            "fn revise",
            "history",
            "supersedes",
            "superseded_by",
            "version",
        ],
    );

    let original = json!({
        "id": "risk:history",
        "title": "original",
        "description": "body",
        "status": "open",
        "version": 1,
        "inherentScore": { "cellId": "possible-major" },
        "history": [{
            "version": 1,
            "at": "2026-01-01T00:00:00Z",
            "kind": "created"
        }]
    });
    let risk: Risk = serde_json::from_value(original.clone()).unwrap();
    let before = serde_json::to_value(&risk).unwrap();
    assert_eq!(before["version"], 1);
    assert_eq!(
        before["history"].as_array().map(|h| h.len()),
        Some(1),
        "seeded history must round-trip"
    );

    let revised = json!({
        "id": "risk:history",
        "title": "revised",
        "description": "body",
        "status": "open",
        "version": 2,
        "inherentScore": { "cellId": "possible-major" },
        "history": [
            {
                "version": 1,
                "at": "2026-01-01T00:00:00Z",
                "kind": "created"
            },
            {
                "version": 2,
                "at": "2026-02-01T00:00:00Z",
                "kind": "fieldsRevised",
                "previous": { "title": "original" }
            }
        ]
    });
    let out = round_trip_json(&revised);
    assert_eq!(out["version"], 2);
    assert_eq!(out["title"], "revised");
    let history = out["history"].as_array().expect("history must persist");
    assert!(
        history.len() >= 2,
        "revise must append, not clear, history (got {history:?})"
    );
    assert!(
        history.iter().any(|ev| ev.to_string().contains("original")),
        "prior title must remain represented in history"
    );
}

/// RR-013: CIA raw inputs serialize when present and omit when unset.
#[test]
fn rr_013_cia_raw_inputs_omit_when_unset_and_do_not_replace_ratings() {
    let omitted = serde_json::to_value(&Risk::new(RiskId::new("risk:cia"), "t", "d")).unwrap();
    assert!(omitted.get("cia").is_none());

    let with_cia = json!({
        "id": "risk:cia-set",
        "title": "t",
        "description": "d",
        "status": "open",
        "cia": { "confidentiality": 2, "integrity": 5, "availability": 1 }
    });
    let out = round_trip_json(&with_cia);
    assert_eq!(out["cia"], with_cia["cia"]);
    assert!(
        out.get("inherentRating").is_none(),
        "CIA raw inputs must not substitute for methodology ratings"
    );
}

/// RR-014: owner is PrincipalRef; dangling Identity owner fails closed.
#[test]
fn rr_014_owner_is_principal_ref_and_dangling_identity_fails_closed() {
    let _owner = PrincipalRef::Identity(IdentityId::new("identity:alice"));
    let risk_src = read_repo_file("crates/weeping-angel-assurance-ir/src/risk.rs");
    require_needles("RR-014 owner type", &risk_src, &["PrincipalRef", "owner"]);

    let owned = json!({
        "id": "risk:owned",
        "title": "t",
        "description": "d",
        "status": "open",
        "owner": { "identity": "identity:alice" }
    });
    let out = round_trip_json(&owned);
    assert_eq!(out["owner"], owned["owner"]);

    let mut dangling = empty_assessment();
    dangling.risks.push(serde_json::from_value(owned).unwrap());
    dangling
        .validate()
        .expect_err("dangling Identity owner must fail closed");

    let mut resolved = empty_assessment();
    resolved.identities.push(Identity::new(
        IdentityId::new("identity:alice"),
        IdentityKind::User,
    ));
    resolved.assets.push(sample_asset());
    resolved.controls.push(sample_control());
    resolved.processing_activities.push(ProcessingActivity::new(
        ProcessingActivityId::new("ropa:source-control"),
        "Source control",
    ));
    resolved
        .vendors
        .push(Vendor::new(VendorId::new("vendor:git-host"), "Git host"));
    resolved
        .evidence_requirements
        .push(EvidenceRequirement::new(
            EvidenceRequirementId::new("evidence.req.source-integrity"),
            EvidenceType::new("evidence.source.integrity"),
        ));
    let populated = operational_payload();
    let mut without_treatment = populated.as_object().cloned().unwrap();
    without_treatment.remove("treatmentId");
    without_treatment.remove("supersedes");
    resolved
        .risks
        .push(serde_json::from_value(Value::Object(without_treatment)).unwrap());
    resolved.validate().expect(
        "resolved identity/asset/control/evidence refs must be valid when treatmentId is omitted",
    );
}

/// RR-015: dual-suite names are listed in root Cargo.toml.
#[test]
fn rr_015_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_risk_register_baseline")
            && toml.contains("sdd_risk_register_target")
            && toml.contains("tests/contracts/risk_register.baseline.rs")
            && toml.contains("tests/contracts/risk_register.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
}
