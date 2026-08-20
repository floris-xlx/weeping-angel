//! Target suite for Operational ISMS v1 Prompt 11
//! (`docs/specs/operational-soa.md` §4 / §5 / §6.2).
//!
//! Encodes SOA-T01–T12 on CURRENT HEAD. Must stay RED for semantic reasons:
//! no `project_operational_soa`, implementation hardcoded `assessed`,
//! no NA approval/gaps, no SoA cause taxonomy, live `project_soa` still
//! copies pack TOML, snapshot digest is caller-supplied.
//!
//! Compile-safe: do not import symbols that do not exist yet.
//! Do not implement the projector in this file and do not `#[ignore]`.

use serde_json::Value;
use weeping_angel_assurance::soa::Applicability;
use weeping_angel_assurance::{StatementOfApplicability, project_soa};
use weeping_angel_assurance_ir::{Control, ControlImplementation};
use weeping_angel_control_test::Effectiveness;

fn soa_src() -> String {
    read_repo_file("crates/weeping-angel-assurance/src/soa.rs")
}

fn lib_src() -> String {
    read_repo_file("crates/weeping-angel-assurance/src/lib.rs")
}

fn snapshot_src() -> String {
    read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs")
}

fn lineage_src() -> String {
    read_repo_file("crates/weeping-angel-assurance/src/lineage.rs")
}

fn live_iso_soa() -> StatementOfApplicability {
    project_soa("iso-27001", "2022")
}

fn live_json() -> Value {
    serde_json::to_value(live_iso_soa()).expect("serialize live SoA")
}

fn live_entry(reference: &str) -> Value {
    live_json()
        .get("entries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|e| e.get("reference").and_then(Value::as_str) == Some(reference))
        .cloned()
        .unwrap_or_else(|| panic!("expected live SoA row {reference}"))
}

fn live_soa_entry<'a>(
    soa: &'a StatementOfApplicability,
    reference: &str,
) -> &'a weeping_angel_assurance::soa::SoaEntry {
    soa.entries
        .iter()
        .find(|e| e.reference == reference)
        .unwrap_or_else(|| panic!("expected SoA row {reference}"))
}

fn field_str(entry: &Value, keys: &[&str]) -> String {
    for key in keys {
        if let Some(s) = entry.get(*key).and_then(Value::as_str) {
            return s.to_ascii_lowercase();
        }
    }
    String::new()
}

fn blob(entry: &Value) -> String {
    entry.to_string().to_ascii_lowercase()
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mod.rs"));

fn impl_status_is_not_implemented(value: &str) -> bool {
    let n = value.to_ascii_lowercase().replace(['_', '-'], "");
    n.contains("notimplemented")
}

fn applicability_is_na(value: &str) -> bool {
    let n = value.to_ascii_lowercase().replace(['_', '-'], "");
    n == "notapplicable"
}

/// Operational JSON keys every ISO SoA row must expose after implement.
const OPERATIONAL_ROW_KEYS: &[&str] = &[
    "applicability",
    "applicabilityRationale",
    "linkedRisks",
    "treatmentRationale",
    "treatmentRefs",
    "implementationRefs",
    "owner",
    "effectivenessStatus",
    "evidenceLineage",
    "exceptions",
    "approval",
    "readinessGaps",
];

#[test]
fn soa_t01_applicable_and_effective() {
    let src = soa_src();
    let lib = lib_src();
    require_needles(
        "SOA-T01",
        &format!("{src}\n{lib}"),
        &[
            "fn project_operational_soa",
            "OperationalSoaInput",
            "weeping-angel/operational-soa-input/v1",
            "Effectiveness",
            "implementation_status",
            "effectiveness_status",
            "linked_risks",
        ],
    );
    assert!(
        src.contains("Effectiveness::Effective")
            || src.contains("Effectiveness::Effective")
            || (src.contains("Effective") && src.contains("Implemented")),
        "SOA-T01: projector must wire Effectiveness::Effective + ImplementationStatus::Implemented"
    );
    assert!(
        src.contains("ImplementationStatus") || src.contains("Implemented"),
        "SOA-T01: Implemented does not imply NA or Effective by itself"
    );

    let json = live_json();
    let blob = json.to_string();
    assert!(
        blob.contains("linkedRisks")
            && blob.contains("effectivenessStatus")
            && blob.contains("implementationRefs"),
        "SOA-T01: SoaEntry JSON must grow operational fields so Implemented+Effective is representable"
    );
    let _ = Effectiveness::Effective;
}

#[test]
fn soa_t02_applicable_and_not_implemented() {
    let soa = live_iso_soa();
    let row = live_soa_entry(&soa, "A.5.1");
    assert_eq!(
        row.applicability,
        Applicability::Applicable,
        "SOA-T02: A.5.1 stays Applicable (found case: missing implementation)"
    );
    assert_ne!(
        row.applicability,
        Applicability::NotApplicable,
        "SOA-T02: applicable + not implemented MUST NOT map to NA"
    );
    assert!(
        impl_status_is_not_implemented(&row.implementation_state),
        "SOA-T02: missing implementation is first-class notImplemented, not '{}'",
        row.implementation_state
    );

    let json = live_entry("A.5.1");
    let status = field_str(
        &json,
        &[
            "implementationStatus",
            "implementationState",
            "implementation_status",
        ],
    );
    assert!(
        impl_status_is_not_implemented(&status),
        "SOA-T02: JSON implementation status must be notImplemented, got {json}"
    );
    assert!(
        !applicability_is_na(&field_str(&json, &["applicability", "applicabilityState"])),
        "SOA-T02: JSON applicability must not become notApplicable, got {json}"
    );
}

#[test]
fn soa_t03_applicable_and_insufficient_evidence() {
    let src = soa_src();
    require_needles(
        "SOA-T03",
        &src,
        &["InsufficientEvidence", "evidence_lineage", "readiness_gaps"],
    );
    assert!(
        !src.contains("insufficient evidence is not applicable")
            && (src.contains("MUST NOT")
                || src.contains("must not")
                || src.contains("not become not applicable")
                || src.contains("InsufficientEvidence")),
        "SOA-T03: insufficient evidence is first-class Effectiveness, never a NA justification"
    );

    let a51 = live_entry("A.5.1");
    let status = field_str(&a51, &["applicability", "applicabilityState"]);
    assert!(
        status.contains("applicable") && !applicability_is_na(&status),
        "SOA-T03: live applicable rows must remain Applicable when evidence is empty, got {a51}"
    );
}

#[test]
fn soa_t04_non_applicable_approved() {
    let src = soa_src();
    require_needles("SOA-T04", &src, &["approval"]);
    assert!(
        src.contains("principal") || src.contains("approved_by") || src.contains("NaApproval"),
        "SOA-T04: NA requires accountable principal + review semantics"
    );

    let soa = live_iso_soa();
    let row = live_soa_entry(&soa, "A.5.19");
    assert_eq!(
        row.applicability,
        Applicability::NotApplicable,
        "SOA-T04: live A.5.19 remains NotApplicable (remap ISO-R-009 / g07)"
    );
    assert!(
        !row.applicability_rationale.is_empty(),
        "SOA-T04: approved NA needs explicit rationale"
    );
    let rationale = row.applicability_rationale.to_ascii_lowercase();
    assert!(
        !rationale.contains("no evidence")
            && !rationale.contains("missing evidence")
            && !rationale.contains("insufficient evidence"),
        "SOA-T04: NA rationale is organization context, not missing evidence"
    );

    let json = live_entry("A.5.19");
    let has_principal = json.pointer("/approval/principal").is_some()
        || json
            .get("approval")
            .and_then(|v| v.get("principal"))
            .is_some()
        || blob(&json).contains("principal");
    assert!(
        has_principal,
        "SOA-T04: NA row must carry accountable principal/approval metadata, got {json}"
    );
}

#[test]
fn soa_t05_non_applicable_expired_is_readiness_gap() {
    let src = soa_src();
    require_needles("SOA-T05", &src, &["readiness_gaps", "expires_at"]);
    assert!(
        src.contains("expiredNaApproval")
            || src.contains("expired_na_approval")
            || src.contains("missingNaApproval")
            || src.contains("missing_na_approval"),
        "SOA-T05: expired/missing NA approval must surface a named readiness gap, not silent NA"
    );
    assert!(
        src.contains("ExceptionStatus") || src.contains("Expired"),
        "SOA-T05: projector must observe exception expiry"
    );
}

#[test]
fn soa_t06_partial_canonical_mapping() {
    let src = soa_src();
    assert!(
        src.contains("partialCanonicalMapping")
            || src.contains("partial_canonical_mapping")
            || src.contains("PartiallySatisfies"),
        "SOA-T06: partial mapping must be a first-class SoA note/gap"
    );

    let json = live_entry("A.8.5");
    let status = field_str(&json, &["applicability", "applicabilityState"]);
    assert!(
        status.contains("applicable") && !applicability_is_na(&status),
        "SOA-T06: partial mapping must not coerce A.8.5 to NA, got {json}"
    );
    let mapped = json
        .get("mappedControls")
        .or_else(|| json.get("canonicalControls"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(
        mapped
            .iter()
            .any(|c| c.as_str().is_some_and(|s| s.starts_with("control."))),
        "SOA-T06: A.8.5 lists canonical catalog ids, got {mapped:?}"
    );
}

#[test]
fn soa_t07_risk_treatment_driven_applicability_fail_closed() {
    let src = soa_src();
    require_needles(
        "SOA-T07",
        &src,
        &[
            "RiskTreatmentRef",
            "weeping-angel/risk-treatment-ref/v1",
            "RiskRegisterRef",
            "weeping-angel/risk-register-ref/v1",
            "OperationalSoaError",
            "MissingRiskTreatment",
            "MissingRiskRegister",
        ],
    );
    assert!(
        !src.contains("residual_risk") && !src.contains("ResidualRisk"),
        "SOA-T07: must not implement residual-risk (Prompt 09 collision fence)"
    );
    assert!(
        !src.contains("fn treat_risk") && !src.contains("TreatmentStateMachine"),
        "SOA-T07: must not implement the Prompt 08 treatment engine"
    );
}

#[test]
fn soa_t08_snapshot_diff_with_causes() {
    let joined = format!("{}\n{}\n{}", snapshot_src(), soa_src(), lib_src());
    assert!(
        joined.contains("SoaDiffCause")
            || joined.contains("fn diff_soa_snapshots")
            || joined.contains("soa_causes"),
        "SOA-T08: SoA snapshot diff must name a cause taxonomy"
    );
    for cause in [
        "ApplicabilityChange",
        "ImplementationChange",
        "EffectivenessRegression",
        "ExceptionExpiry",
        "MappingChange",
        "TreatmentChange",
    ] {
        assert!(
            joined.contains(cause),
            "SOA-T08: missing SoA diff cause {cause}"
        );
    }
}

#[test]
fn soa_t09_missing_implementation_is_not_not_applicable() {
    let soa = live_iso_soa();
    let applicable: Vec<_> = soa
        .entries
        .iter()
        .filter(|e| e.applicability == Applicability::Applicable)
        .collect();
    assert!(
        !applicable.is_empty(),
        "SOA-T09: live ISO SoA still has Applicable rows"
    );
    for row in applicable {
        assert_ne!(
            row.applicability,
            Applicability::NotApplicable,
            "SOA-T09: applicable row {} must not be NA",
            row.reference
        );
        let impl_state = row.implementation_state.to_ascii_lowercase();
        assert!(
            !impl_state.contains("notapplicable") && !impl_state.contains("not-applicable"),
            "SOA-T09: missing implementation must not be encoded as implementation NA ({})",
            row.reference
        );
        assert_ne!(
            impl_state, "assessed",
            "SOA-T09: hardcoded assessed is not a first-class missing-implementation row ({})",
            row.reference
        );
        assert!(
            impl_status_is_not_implemented(&row.implementation_state),
            "SOA-T09: empty implementations ⇒ notImplemented, not assessed ({})",
            row.reference
        );
    }

    let src = soa_src();
    assert!(
        src.contains("NotImplemented") || src.contains("notImplemented"),
        "SOA-T09: projector must name ImplementationStatus::NotImplemented"
    );
    assert!(
        !src.contains("ImplementationStatus::NotApplicable")
            || src.contains("does not set")
            || src.contains("must not")
            || src.contains("MUST NOT"),
        "SOA-T09: IR implementation NotApplicable must not flip SoA applicability"
    );
}

#[test]
fn soa_t10_pinned_snapshot_not_live_project_soa() {
    let src = soa_src();
    let lib = lib_src();
    assert!(
        src.contains("fn pin_soa_snapshot")
            || src.contains("fn seal_soa_snapshot")
            || lib.contains("pin_soa_snapshot"),
        "SOA-T10: pin_soa_snapshot must compute an immutable digest"
    );
    assert!(
        lib.contains("project_soa_from_snapshot"),
        "SOA-T10: project_soa_from_snapshot must be crate-root re-exported"
    );
    let from_snap = {
        let start = src
            .find("pub fn project_soa_from_snapshot")
            .expect("from_snapshot");
        let rest = &src[start..];
        rest.split("pub fn project_soa(").next().unwrap_or(rest)
    };
    assert!(
        !from_snap.contains("resolve_pack_dir") && !from_snap.contains("applicability.toml"),
        "SOA-T10: historical reconstruction must not reread live pack files"
    );
    assert!(
        src.contains("typed_canonical_digest")
            || src.contains("snapshot_digest")
            || src.contains("soa-snapshot"),
        "SOA-T10: digest must be computed from pinned SoA body, not caller-supplied empty string"
    );
    assert!(
        lineage_src().contains("struct StatementOfApplicabilitySnapshot"),
        "SOA-T10: StatementOfApplicabilitySnapshot remains the historical document"
    );
}

#[test]
fn soa_t11_live_project_soa_is_not_sole_historical_path() {
    let src = soa_src();
    assert!(
        src.contains("project_soa_from_snapshot"),
        "SOA-T11: live project_soa must not be the sole reconstruction path"
    );
    let live_fn = {
        let start = src.find("pub fn project_soa(").expect("project_soa");
        &src[start..]
    };
    assert!(
        live_fn.contains("notImplemented")
            || live_fn.contains("NotImplemented")
            || live_fn.contains("project_operational_soa"),
        "SOA-T11: live convenience must project applicable + notImplemented instead of assessed stubs"
    );
    assert!(
        src.contains("ApplicabilityDecision")
            || src.contains("ManualDeterminationRequired") && src.contains("ApplicabilitySnapshot"),
        "SOA-T11: live path consumes Kleene results; pack TOML is default/structural flags only"
    );

    let soa = live_iso_soa();
    assert_eq!(
        live_soa_entry(&soa, "A.8.13").applicability,
        Applicability::Unresolved,
        "SOA-T11: Unresolved stays representable (A.8.13 / remap g08)"
    );
    let d = soa.disclaimer.to_ascii_lowercase();
    assert!(
        d.contains("readiness") && d.contains("not certification"),
        "SOA-T11: disclaimer remains a readiness-not-certification statement, got {}",
        soa.disclaimer
    );
    let banned = [
        "iso 27001 certified",
        "iso 27001 compliant",
        "certification guaranteed",
        "audit passed",
    ];
    let blob = format!("{}\n{}", src.to_ascii_lowercase(), d);
    for phrase in banned {
        assert!(
            !blob.contains(phrase),
            "SOA-T11: must not emit certification claim {phrase:?}"
        );
    }
}

#[test]
fn soa_t12_every_row_exposes_operational_dimensions() {
    let src = soa_src();
    require_needles(
        "SOA-T12",
        &src,
        &[
            "linked_risks",
            "treatment_rationale",
            "treatment_refs",
            "implementation_refs",
            "implementation_status",
            "effectiveness_status",
            "evidence_lineage",
            "readiness_gaps",
            "review_state",
        ],
    );

    let json = live_json();
    let entries = json
        .get("entries")
        .and_then(Value::as_array)
        .expect("SoA entries");
    assert!(!entries.is_empty(), "SOA-T12: ISO projection is non-empty");
    for entry in entries {
        let reference = entry
            .get("reference")
            .and_then(Value::as_str)
            .unwrap_or("?");
        for key in OPERATIONAL_ROW_KEYS {
            assert!(
                entry.get(*key).is_some()
                    || (*key == "implementationStatus"
                        && entry.get("implementationState").is_some()
                        && impl_status_is_not_implemented(&field_str(
                            entry,
                            &["implementationState"]
                        )))
                    || (*key == "canonicalControls" && entry.get("mappedControls").is_some()),
                "SOA-T12: row {reference} missing operational field {key}, got {entry}"
            );
        }
        let impl_status = field_str(entry, &["implementationStatus", "implementationState"]);
        if field_str(entry, &["applicability"]) == "applicable" {
            assert!(
                !applicability_is_na(&impl_status),
                "SOA-T12: applicable row {reference} must not encode missing impl as NA"
            );
        }
    }

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.contains("annex_a") && !ir.contains("annexA") && !ir.contains("isoAnnex"),
        "SOA-T12: generic IR Control / ControlImplementation must not grow ISO Annex A fields"
    );
    let _ = std::any::type_name::<Control>();
    let _ = std::any::type_name::<ControlImplementation>();
}
