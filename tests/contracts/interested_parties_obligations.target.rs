//! Target suite for interested parties / obligations.
//!
//! Encodes DESIRED behavior in `docs/specs/interested-parties-obligations.md`
//! §4 / §4.15 (IPO-001–018). Must stay RED on CURRENT HEAD: no standalone
//! `ObligationRegistry` in `party.rs` + `obligation.rs`; no shared
//! `ObligationId`; no current-at-T or `explain_why_control_exists` helpers.
//! Do not `#[ignore]` these tests and do not implement the feature here.
//!
//! Std-only (filesystem + JSON fixtures + source needles) so RED is a named
//! assertion even while sibling ISMS slices churn IR compile. When
//! `party.rs` / `obligation.rs` land, keep these fixture ids and assert
//! behavior through the public helpers.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

const PARTY_RS: &str = "crates/weeping-angel-assurance-ir/src/party.rs";
const OBLIGATION_RS: &str = "crates/weeping-angel-assurance-ir/src/obligation.rs";
const SCHEMA: &str = "assurance-ir/v1";

const PARTY_CUSTOMER: &str = "party.customer.acme";
const PARTY_WORKFORCE: &str = "party.workforce";
const PARTY_REGULATOR: &str = "party.regulator.dpa";
const PARTY_SUPPLIER: &str = "party.vendor.payroll";

const SRC_CUSTOMER: &str = "src.customer.acme-msa";
const SRC_EMPLOYMENT: &str = "src.employment.confidentiality";
const SRC_REGULATORY: &str = "src.regulatory.retention";
const SRC_SUPPLIER: &str = "src.supplier.payroll-dpa";

const OBL_CUSTOMER: &str = "obl.customer.security-commitment";
const OBL_CUSTOMER_2026: &str = "obl.customer.security-commitment.2026";
const OBL_EMPLOYMENT: &str = "obl.employment.confidentiality";
const OBL_REGULATORY: &str = "obl.regulatory.retention";
const OBL_SUPPLIER: &str = "obl.supplier.dpa-security";

const CONTROL_MFA: &str = "control.identity.mfa";
const DOC_CONFIDENTIALITY: &str = "doc.policy.employment-confidentiality";
const RISK_RETENTION: &str = "risk.records.retention-failure";
const CONTROL_MISSING: &str = "control.missing";

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
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

fn ir_src() -> String {
    crate_sources_joined("weeping-angel-assurance-ir")
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

fn query_needles() -> &'static [&'static str] {
    &[
        "struct InterestedParty",
        "enum InterestedPartyKind",
        "struct RequirementSource",
        "enum RequirementSourceKind",
        "struct Obligation",
        "enum ObligationLifecycle",
        "struct ObligationMapping",
        "enum ObligationMappingTarget",
        "struct ObligationRegistry",
        "struct ObligationLinkUniverse",
        "enum ObligationApplicability",
        "fn get_obligation",
        "fn current_obligations_at",
        "fn supersession_chain",
        "fn why_control_exists",
        "fn why_document_exists",
        "fn projects_as_equivalence",
        "fn projects_as_full_satisfaction",
        "fn obligation_applies",
        "typed_id!(ObligationId)",
        "typed_id!(InterestedPartyId)",
        "typed_id!(RequirementSourceId)",
        "typed_id!(ObligationMappingId)",
    ]
}

fn product_has_obligation_api() -> bool {
    crate_src("weeping-angel-assurance-ir")
        .join("party.rs")
        .is_file()
        && crate_src("weeping-angel-assurance-ir")
            .join("obligation.rs")
            .is_file()
        && query_needles().iter().all(|n| ir_src().contains(*n))
}

fn require_obligation_api(case: &str) -> (String, String) {
    assert!(
        product_has_obligation_api(),
        "{case}: InterestedParty + RequirementSource + Obligation + ObligationMapping \
         + ObligationRegistry + current-at-T / why_* helpers must exist in {PARTY_RS} \
         and {OBLIGATION_RS} (RED on characterization HEAD)"
    );
    let party = fs::read_to_string(crate_src("weeping-angel-assurance-ir").join("party.rs"))
        .unwrap_or_else(|e| panic!("read {PARTY_RS}: {e}"));
    let obligation =
        fs::read_to_string(crate_src("weeping-angel-assurance-ir").join("obligation.rs"))
            .unwrap_or_else(|e| panic!("read {OBLIGATION_RS}: {e}"));
    (party, obligation)
}

fn customer_commitment_json() -> Value {
    json!({
        "schemaVersion": SCHEMA,
        "parties": [{
            "schemaVersion": SCHEMA,
            "id": PARTY_CUSTOMER,
            "name": "Acme Customer",
            "kind": "customer",
            "notes": "Contracted production workload tenant"
        }],
        "sources": [{
            "schemaVersion": SCHEMA,
            "id": SRC_CUSTOMER,
            "kind": "customer",
            "title": "Acme MSA security schedule",
            "partyId": PARTY_CUSTOMER,
            "citation": "MSA-2024-ACME-SEC"
        }],
        "obligations": [{
            "schemaVersion": SCHEMA,
            "id": OBL_CUSTOMER,
            "sourceId": SRC_CUSTOMER,
            "title": "Customer production MFA",
            "description": "Production identities used for customer workloads require MFA.",
            "applicability": {
                "organizations": ["org:acme"],
                "subjects": [{
                    "kind": "identity",
                    "ids": ["identity:prod-admin"],
                    "scope": "anyOf"
                }],
                "exclusions": []
            },
            "owner": { "team": "security-governance" },
            "effectiveFrom": "2026-01-01T00:00:00Z",
            "lifecycle": "active"
        }],
        "mappings": [{
            "schemaVersion": SCHEMA,
            "id": "map.obl.customer.mfa",
            "from": OBL_CUSTOMER,
            "to": { "control": CONTROL_MFA },
            "direction": "forward",
            "completeness": "partial",
            "relation": "PartiallySatisfies",
            "rationale": "MFA is one supporting control for the customer commitment; not equivalence."
        }]
    })
}

fn employment_confidentiality_json() -> Value {
    json!({
        "schemaVersion": SCHEMA,
        "parties": [{
            "schemaVersion": SCHEMA,
            "id": PARTY_WORKFORCE,
            "name": "Workforce",
            "kind": "employee"
        }],
        "sources": [{
            "schemaVersion": SCHEMA,
            "id": SRC_EMPLOYMENT,
            "kind": "employment",
            "title": "Employment confidentiality terms",
            "partyId": PARTY_WORKFORCE,
            "citation": "EMP-NDA-v3"
        }],
        "obligations": [{
            "schemaVersion": SCHEMA,
            "id": OBL_EMPLOYMENT,
            "sourceId": SRC_EMPLOYMENT,
            "title": "Workforce confidentiality",
            "description": "Employees must protect customer and employer confidential information.",
            "applicability": { "organizations": ["org:acme"], "subjects": [], "exclusions": [] },
            "owner": { "role": "people-security" },
            "effectiveFrom": "2025-01-01T00:00:00Z",
            "lifecycle": "active"
        }],
        "mappings": [{
            "schemaVersion": SCHEMA,
            "id": "map.obl.employment.policy",
            "from": OBL_EMPLOYMENT,
            "to": { "document": DOC_CONFIDENTIALITY },
            "direction": "forward",
            "completeness": "full",
            "relation": "Satisfies",
            "rationale": "The employment confidentiality policy is the org-owned duty vehicle."
        }]
    })
}

fn regulatory_retention_json() -> Value {
    json!({
        "schemaVersion": SCHEMA,
        "parties": [{
            "schemaVersion": SCHEMA,
            "id": PARTY_REGULATOR,
            "name": "Supervisory authority",
            "kind": "regulator"
        }],
        "sources": [{
            "schemaVersion": SCHEMA,
            "id": SRC_REGULATORY,
            "kind": "legalRegulatory",
            "title": "Personal-data storage limitation pointer",
            "partyId": PARTY_REGULATOR,
            "citation": "GDPR Art. 5(1)(e)"
        }],
        "obligations": [{
            "schemaVersion": SCHEMA,
            "id": OBL_REGULATORY,
            "sourceId": SRC_REGULATORY,
            "title": "Retain personal data no longer than necessary",
            "description": "Org-owned paraphrase of a storage-limitation duty; not licensed statute text.",
            "applicability": { "organizations": ["org:acme"], "subjects": [], "exclusions": [] },
            "owner": { "role": "privacy-office" },
            "effectiveFrom": "2024-01-01T00:00:00Z",
            "effectiveUntil": "2026-06-30T00:00:00Z",
            "lifecycle": "active"
        }],
        "mappings": [
            {
                "schemaVersion": SCHEMA,
                "id": "map.obl.retention.risk",
                "from": OBL_REGULATORY,
                "to": { "risk": RISK_RETENTION },
                "direction": "forward",
                "completeness": "related",
                "relation": "Related",
                "rationale": "Retention failure is a related risk, not an equivalent control."
            },
            {
                "schemaVersion": SCHEMA,
                "id": "map.obl.retention.control",
                "from": OBL_REGULATORY,
                "to": { "control": "control.records.retention" },
                "direction": "forward",
                "completeness": "partial",
                "relation": "PartiallySatisfies",
                "rationale": "Retention control supports the duty without claiming full satisfaction."
            }
        ]
    })
}

fn supplier_contractual_json() -> Value {
    json!({
        "schemaVersion": SCHEMA,
        "parties": [{
            "schemaVersion": SCHEMA,
            "id": PARTY_SUPPLIER,
            "name": "Payroll processor",
            "kind": "supplier"
        }],
        "sources": [{
            "schemaVersion": SCHEMA,
            "id": SRC_SUPPLIER,
            "kind": "contractual",
            "title": "Payroll processor DPA",
            "partyId": PARTY_SUPPLIER,
            "citation": "DPA-PAYROLL-2025"
        }],
        "obligations": [{
            "schemaVersion": SCHEMA,
            "id": OBL_SUPPLIER,
            "sourceId": SRC_SUPPLIER,
            "title": "Supplier must protect payroll personal data",
            "description": "Contractual security terms for the payroll processor; not a cloud-account filter.",
            "applicability": {
                "organizations": ["org:acme"],
                "subjects": [{
                    "kind": "vendor",
                    "ids": ["vendor:payroll"],
                    "scope": "anyOf"
                }],
                "exclusions": []
            },
            "owner": { "team": "vendor-risk" },
            "effectiveFrom": "2025-03-01T00:00:00Z",
            "lifecycle": "active"
        }],
        "mappings": [{
            "schemaVersion": SCHEMA,
            "id": "map.obl.supplier.dpa",
            "from": OBL_SUPPLIER,
            "to": { "document": "doc.contract.payroll-dpa" },
            "direction": "forward",
            "completeness": "partial",
            "relation": "PartiallySatisfies",
            "rationale": "The executed DPA is a partial contractual vehicle, not control equivalence."
        }]
    })
}

fn assert_camel_case_document(label: &str, value: &Value) {
    assert_eq!(
        value["schemaVersion"], SCHEMA,
        "{label}: schemaVersion must be {SCHEMA}"
    );
    assert!(
        value.get("schema_version").is_none(),
        "{label}: obligation documents use camelCase schemaVersion"
    );
}

fn assert_honesty_preserved(label: &str, mapping: &Value, relation: &str, completeness: &str) {
    assert_eq!(mapping["relation"], relation, "{label}: relation");
    assert_eq!(
        mapping["completeness"], completeness,
        "{label}: completeness"
    );
    assert!(
        mapping.get("direction").is_some(),
        "{label}: mapping direction must be explicit"
    );
    let rationale = mapping["rationale"].as_str().unwrap_or("");
    assert!(
        !rationale.is_empty(),
        "{label}: rationale is required and non-empty"
    );
}

/// IPO-001: customer security commitment fixture round-trips and explains why MFA exists.
#[test]
fn ipo_001_customer_security_commitment() {
    let (_party, obligation) = require_obligation_api(
        "IPO-001: customer security commitment constructs, validates, explains mapped control",
    );
    require_needles(
        "IPO-001",
        &obligation,
        &[
            "fn why_control_exists",
            "fn validate",
            "AssessmentScope",
            "PrincipalRef",
            "Customer",
        ],
    );

    let fixture = customer_commitment_json();
    assert_camel_case_document("IPO-001", &fixture);
    assert_eq!(fixture["parties"][0]["id"], PARTY_CUSTOMER);
    assert_eq!(fixture["parties"][0]["kind"], "customer");
    assert_eq!(fixture["sources"][0]["kind"], "customer");
    assert_eq!(fixture["obligations"][0]["id"], OBL_CUSTOMER);
    assert_eq!(fixture["obligations"][0]["lifecycle"], "active");
    assert_honesty_preserved(
        "IPO-001",
        &fixture["mappings"][0],
        "PartiallySatisfies",
        "partial",
    );
    assert_eq!(fixture["mappings"][0]["to"]["control"], CONTROL_MFA);
    assert_eq!(
        fixture["obligations"][0]["applicability"]["subjects"][0]["kind"], "identity",
        "IPO-001: applicability is IR SubjectSelector, not a provider filter"
    );
    require_needles(
        "IPO-001-explain",
        &crate_sources_joined("weeping-angel-assurance"),
        &["fn explain_why_control_exists", "struct ObligationExplain"],
    );
}

/// IPO-002: employment confidentiality explains why the linked policy/document exists.
#[test]
fn ipo_002_employment_confidentiality() {
    let (_party, obligation) = require_obligation_api(
        "IPO-002: employment confidentiality maps to a policy/document with explicit relation",
    );
    require_needles(
        "IPO-002",
        &obligation,
        &[
            "fn why_document_exists",
            "Employment",
            "ControlledDocumentId",
            "Document(",
        ],
    );

    let fixture = employment_confidentiality_json();
    assert_camel_case_document("IPO-002", &fixture);
    assert_eq!(fixture["parties"][0]["id"], PARTY_WORKFORCE);
    assert_eq!(fixture["parties"][0]["kind"], "employee");
    assert_eq!(fixture["sources"][0]["kind"], "employment");
    assert_eq!(fixture["obligations"][0]["id"], OBL_EMPLOYMENT);
    assert_eq!(
        fixture["mappings"][0]["to"]["document"],
        DOC_CONFIDENTIALITY
    );
    assert_honesty_preserved("IPO-002", &fixture["mappings"][0], "Satisfies", "full");
    assert_ne!(
        fixture["mappings"][0]["relation"], "Equivalent",
        "IPO-002: org policy Satisfies the duty; it is not silent ISO-clause equivalence"
    );
}

/// IPO-003: regulatory retention uses LegalRegulatory + citation pointer, no protected text.
#[test]
fn ipo_003_regulatory_retention() {
    let (_party, obligation) =
        require_obligation_api("IPO-003: regulatory retention citation pointer without body text");
    require_needles(
        "IPO-003",
        &obligation,
        &["LegalRegulatory", "citation", "protected"],
    );

    let fixture = regulatory_retention_json();
    assert_eq!(fixture["sources"][0]["kind"], "legalRegulatory");
    assert_eq!(fixture["sources"][0]["citation"], "GDPR Art. 5(1)(e)");
    assert_eq!(fixture["parties"][0]["kind"], "regulator");
    let blob = fixture.to_string().to_ascii_lowercase();
    assert!(
        !blob.contains("the organization shall")
            && !blob.contains("annex a")
            && !blob.contains("iso/iec 27001"),
        "IPO-003: must not store protected normative text on generic IR"
    );
    assert_honesty_preserved(
        "IPO-003-risk",
        &fixture["mappings"][0],
        "Related",
        "related",
    );
    assert_honesty_preserved(
        "IPO-003-control",
        &fixture["mappings"][1],
        "PartiallySatisfies",
        "partial",
    );
}

/// IPO-004: supplier contractual fixture is provider-neutral with explicit relation/rationale.
#[test]
fn ipo_004_supplier_contractual() {
    let (_party, obligation) =
        require_obligation_api("IPO-004: supplier contractual fixture is provider-neutral");
    require_needles("IPO-004", &obligation, &["Contractual", "Supplier"]);

    let fixture = supplier_contractual_json();
    assert_eq!(fixture["parties"][0]["id"], PARTY_SUPPLIER);
    assert_eq!(fixture["parties"][0]["kind"], "supplier");
    assert_eq!(fixture["sources"][0]["kind"], "contractual");
    assert_eq!(fixture["obligations"][0]["id"], OBL_SUPPLIER);
    assert_honesty_preserved(
        "IPO-004",
        &fixture["mappings"][0],
        "PartiallySatisfies",
        "partial",
    );

    let ir = ir_src();
    forbid_needles(
        "IPO-004",
        &ir,
        &[
            "aws-sdk",
            "octocrab",
            "githubOrg",
            "awsAccountFilter",
            "struct ObligationProviderFilter",
        ],
    );
    let blob = fixture.to_string();
    assert!(
        !blob.contains("arn:aws") && !blob.contains("github.com") && !blob.contains("octocrab"),
        "IPO-004: fixture itself must stay provider-neutral"
    );
}

/// IPO-005: superseded predecessor remains get(); current_at excludes it.
#[test]
fn ipo_005_supersession_replayable_not_current() {
    let (_party, obligation) = require_obligation_api(
        "IPO-005: superseded obligations remain get-addressable and leave current_at",
    );
    require_needles(
        "IPO-005",
        &obligation,
        &[
            "fn get_obligation",
            "fn current_obligations_at",
            "fn supersession_chain",
            "Superseded",
            "supersedes",
            "Active",
            "Retired",
        ],
    );
    assert_ne!(OBL_CUSTOMER, OBL_CUSTOMER_2026);
    let successor = json!({
        "id": OBL_CUSTOMER_2026,
        "supersedes": OBL_CUSTOMER,
        "lifecycle": "active"
    });
    let predecessor = json!({
        "id": OBL_CUSTOMER,
        "lifecycle": "superseded"
    });
    assert_eq!(successor["supersedes"], OBL_CUSTOMER);
    assert_eq!(predecessor["lifecycle"], "superseded");
}

/// IPO-006: expired applicability is addressable and not current.
#[test]
fn ipo_006_expired_applicability_addressable() {
    let (_party, obligation) = require_obligation_api(
        "IPO-006: expired applicability is addressable and excluded from current_at",
    );
    require_needles(
        "IPO-006",
        &obligation,
        &[
            "effective_until",
            "Expired",
            "NotCurrent",
            "fn current_obligations_at",
            "fn get_obligation",
            "enum ObligationApplicability",
        ],
    );

    let fixture = regulatory_retention_json();
    let until = fixture["obligations"][0]["effectiveUntil"]
        .as_str()
        .expect("effectiveUntil");
    assert!(
        until < "2026-08-19T12:00:00Z",
        "IPO-006: fixture effectiveUntil must precede T=2026-08-19 so current_at excludes it"
    );
}

/// IPO-007: dangling mapping targets fail closed.
#[test]
fn ipo_007_dangling_mapping_fails_closed() {
    let (_party, obligation) =
        require_obligation_api("IPO-007: dangling mapping target fails closed");
    require_needles(
        "IPO-007",
        &obligation,
        &["struct ObligationLinkUniverse", "fn validate", "dangling"],
    );
    let dangling = json!({
        "from": OBL_CUSTOMER,
        "to": { "control": CONTROL_MISSING },
        "direction": "forward",
        "completeness": "partial",
        "relation": "PartiallySatisfies",
        "rationale": "maps to a control that is not in the universe"
    });
    assert_eq!(dangling["to"]["control"], CONTROL_MISSING);
}

/// IPO-008: duplicate stable ids fail closed.
#[test]
fn ipo_008_duplicate_stable_ids_fail_closed() {
    let (_party, obligation) = require_obligation_api("IPO-008: duplicate stable ids fail closed");
    require_needles(
        "IPO-008",
        &obligation,
        &["duplicate", "fn validate", "ObligationId"],
    );
    let id_src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    require_needles(
        "IPO-008-ids",
        &id_src,
        &[
            "typed_id!(ObligationId)",
            "typed_id!(InterestedPartyId)",
            "typed_id!(RequirementSourceId)",
            "typed_id!(ObligationMappingId)",
        ],
    );
}

/// IPO-009: PartiallySatisfies / Supports never project as equivalence or full satisfaction.
#[test]
fn ipo_009_partial_mapping_never_equivalence() {
    let (_party, obligation) = require_obligation_api(
        "IPO-009: PartiallySatisfies/Supports never project as equivalence or full satisfaction",
    );
    require_needles(
        "IPO-009",
        &obligation,
        &[
            "fn projects_as_equivalence",
            "fn projects_as_full_satisfaction",
            "PartiallySatisfies",
            "Supports",
        ],
    );
    let mapping_src = read_repo_file("crates/weeping-angel-assurance-ir/src/mapping.rs");
    require_needles(
        "IPO-009-reuse",
        &mapping_src,
        &[
            "enum MappingRelation",
            "PartiallySatisfies",
            "fn from_completeness",
            "MappingCompleteness::Partial => Self::PartiallySatisfies",
        ],
    );

    let fixture = customer_commitment_json();
    let mapping = &fixture["mappings"][0];
    assert_eq!(mapping["relation"], "PartiallySatisfies");
    assert_eq!(mapping["completeness"], "partial");
    assert_eq!(mapping["direction"], "forward");
    assert_ne!(mapping["relation"], "Equivalent");
    assert_ne!(mapping["relation"], "Satisfies");
}

/// IPO-010: illegal Equivalent+partial (and listed honesty pairs) fail validate.
#[test]
fn ipo_010_illegal_equivalent_partial_fails() {
    let (_party, obligation) =
        require_obligation_api("IPO-010: illegal Equivalent+partial fails validate");
    require_needles(
        "IPO-010",
        &obligation,
        &["fn validate", "Equivalent", "mapping honesty"],
    );
    for (relation, completeness) in [
        ("Equivalent", "partial"),
        ("Satisfies", "related"),
        ("PartiallySatisfies", "full"),
    ] {
        assert!(
            obligation.contains("MappingRelation") || obligation.contains(relation),
            "IPO-010: honesty table must reject {relation}+{completeness}"
        );
    }
}

/// IPO-011: applicability is IR AssessmentScope / SubjectSelector, not provider filters.
#[test]
fn ipo_011_applicability_uses_scope_engine_selectors() {
    let (_party, obligation) = require_obligation_api(
        "IPO-011: applicability uses AssessmentScope/SubjectSelector resolved via the scope engine",
    );
    require_needles(
        "IPO-011",
        &obligation,
        &[
            "pub applicability",
            "AssessmentScope",
            "SubjectSelector",
            "fn obligation_applies",
        ],
    );
    let product = product_crate_sources_joined();
    forbid_needles(
        "IPO-011",
        &product,
        &[
            "struct ObligationProviderFilter",
            "githubOrg",
            "awsAccountFilter",
        ],
    );
    if product.contains("struct ScopeResolution") {
        assert!(
            obligation.contains("ScopeResolution"),
            "IPO-011: when ScopeResolution is landed, obligation resolution must call it"
        );
    }
    let fixture = customer_commitment_json();
    assert_eq!(
        fixture["obligations"][0]["applicability"]["organizations"][0],
        "org:acme"
    );
    assert_eq!(
        fixture["obligations"][0]["applicability"]["subjects"][0]["kind"],
        "identity"
    );
}

/// IPO-012: overlapping Active obligations may coexist.
#[test]
fn ipo_012_overlapping_active_obligations_coexist() {
    let (_party, obligation) =
        require_obligation_api("IPO-012: overlapping/conflicting Active obligations may coexist");
    require_needles(
        "IPO-012",
        &obligation,
        &["ObligationLifecycle", "Active", "fn current_obligations_at"],
    );
    forbid_needles(
        "IPO-012",
        &obligation,
        &["fn merge_overlapping_obligations", "reject overlap"],
    );
}

/// IPO-013: collectors and framework packs cannot set obligation satisfaction or lifecycle.
#[test]
fn ipo_013_collectors_and_packs_cannot_satisfy_obligations() {
    let (_party, obligation) =
        require_obligation_api("IPO-013: collectors/packs cannot mutate obligation satisfaction");
    forbid_needles(
        "IPO-013-obligation-fields",
        &obligation,
        &["pub satisfied", "obligationStatus", "pub effectiveness:"],
    );
    let collector = crate_sources_joined("weeping-angel-collector");
    let framework = crate_sources_joined("weeping-angel-framework");
    forbid_needles(
        "IPO-013-collector",
        &collector,
        &[
            "fn satisfy_obligation",
            "obligationStatus",
            "mark_obligation_satisfied",
            "Obligation.lifecycle",
        ],
    );
    forbid_needles(
        "IPO-013-framework",
        &framework,
        &[
            "fn satisfy_obligation",
            "obligationStatus",
            "mark_obligation_satisfied",
            "Obligation.lifecycle",
        ],
    );
}

/// IPO-014: explain_why_control_exists is deterministic via canon/v1.
#[test]
fn ipo_014_explain_is_deterministic() {
    let (_party, _obligation) =
        require_obligation_api("IPO-014: explain is deterministic via canon/v1");
    let assurance = crate_sources_joined("weeping-angel-assurance");
    require_needles(
        "IPO-014",
        &assurance,
        &[
            "fn explain_why_control_exists",
            "ObligationExplain",
            "canon/v1",
        ],
    );
    require_needles(
        "IPO-014-digest",
        &ir_src(),
        &["fn canonical_digest", "canon/v1"],
    );
}

/// IPO-015: dual-suite registered; schema remains assurance-ir/v1; no Annex A on Obligation.
#[test]
fn ipo_015_dual_suite_and_schema() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        !cargo.contains("name = \"sdd_interested_parties_obligations_baseline\"")
            && !cargo
                .contains("path = \"tests/contracts/interested_parties_obligations.baseline.rs\"")
            && harness_src().contains("interested_parties_obligations.target.rs"),
        "IPO-015: dual-suite names must be wired as a harness module"
    );
    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        lib.contains("assurance-ir/v1"),
        "IPO-015: schema remains {SCHEMA}"
    );

    let (_party, obligation) =
        require_obligation_api("IPO-015: Obligation record exists and stays assurance-ir/v1");
    assert!(
        !obligation.to_ascii_lowercase().contains("annex a"),
        "IPO-015: Obligation must not carry Annex A fields"
    );
    require_needles("IPO-015-mods", &lib, &["mod party", "mod obligation"]);
}

/// IPO-016: ObligationId is a single typed_id! alias (shared; do not fork).
#[test]
fn ipo_016_obligation_id_is_shared_typed_id() {
    let _ = require_obligation_api("IPO-016: shared ObligationId typed_id! alias");
    let id_src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    let count = id_src.matches("typed_id!(ObligationId)").count();
    assert_eq!(
        count, 1,
        "IPO-016: exactly one typed_id!(ObligationId) alias, found {count}"
    );
}

/// IPO-017: Requirement stays a framework clause; Obligation is a distinct type.
#[test]
fn ipo_017_requirement_unchanged_obligation_distinct() {
    let req = read_repo_file("crates/weeping-angel-assurance-ir/src/requirement.rs");
    assert!(
        !req.contains("Obligation"),
        "IPO-017: do not bolt RequirementKind::Obligation onto framework requirements"
    );
    assert!(
        req.contains("struct Requirement") && req.contains("FrameworkRef"),
        "IPO-017: Requirement remains a framework clause"
    );
    let (_party, obligation) = require_obligation_api("IPO-017: Obligation is a distinct IR type");
    require_needles("IPO-017", &obligation, &["struct Obligation", "source_id"]);
}

/// IPO-018: retired/superseded obligations have no delete API and remain get(id).
#[test]
fn ipo_018_no_delete_api_historical_addressable() {
    let (_party, obligation) = require_obligation_api(
        "IPO-018: retired/superseded obligations remain get-addressable; no delete API",
    );
    require_needles(
        "IPO-018",
        &obligation,
        &["fn get_obligation", "Retired", "Superseded"],
    );
    let ir = ir_src();
    forbid_needles(
        "IPO-018",
        &ir,
        &["fn delete_obligation", "fn remove_obligation"],
    );
    assert!(
        !ir.contains("ObligationLifecycle::Deleted") && !ir.contains("lifecycle: Deleted"),
        "IPO-018: no Deleted lifecycle variant"
    );
}
