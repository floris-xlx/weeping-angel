//! Baseline suite for Operational ISMS v1 internal audit (Prompt 21).
//!
//! Characterization of CURRENT tree (`docs/specs/internal-audit.md` §3) on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`: `AuditProgramId` is a typed id
//! only; `AssessmentRequests.audit_program` / `FrameworkCapabilities.supports_audit_program`
//! (and sampling) are fail-closed booleans that compile no program/sample
//! objects; `Iso27007` is a pack-less compile selector; `AssessmentDefinition`
//! has no audits inventory; governance catalog only freshness-tests
//! `evidence.governance.internal-audit` / `control.governance.audit-program`
//! attestation; lineage `EvidenceSnapshot` / `AssessmentRun` pins exist but
//! are not bound to an auditor; there is no independence declaration, audit
//! finding type, snapshot pin-on-audit, incomplete gate, or human sign-off.
//!
//! Skip-superseded by `sdd_internal_audit_target` (`#[ignore = "superseded by target suite"]`).
//! Does **not** implement the internal-audit domain.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use weeping_angel::finding::Finding;
use weeping_angel_assurance::AssessmentRun;
use weeping_angel_assurance::lineage::{
    EvidenceSnapshot, LINEAGE_SNAPSHOT_SCHEMA, seal_evidence_snapshot,
};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AssessmentRequests, AuditProgramId,
    Control, ControlId, FrameworkVersion, ValidateIr,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::Effectiveness;
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkCompileError, FrameworkContext, FrameworkProfile,
    FrameworkTarget, compile_framework, load_framework_pack,
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

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(
        present.is_empty(),
        "{label}: internal-audit product types must be absent on characterization HEAD; found {present:?}"
    );
}

fn audit_engine_needles() -> &'static [&'static str] {
    &[
        "pub struct AuditProgram {",
        "struct AuditProgram {",
        "pub struct Audit {",
        "enum AuditStatus",
        "enum AuditProgramStatus",
        "enum AuditConclusion",
        "struct IndependenceRecord",
        "struct AuditSample",
        "struct AuditSampleProposal",
        "struct AuditEvidencePin",
        "struct AuditFinding",
        "struct AuditSignOff",
        "struct AuditPrepareBundle",
        "typed_id!(AuditId)",
        "typed_id!(AuditFindingId)",
        "fn prepare_audit_program",
        "fn prepare_audit(",
        "fn propose_sample",
        "fn accept_sample",
        "fn pin_evidence",
        "fn record_finding",
        "fn conclude_audit",
        "fn sign_off",
        "pub mod audit",
    ]
}

fn empty_assessment() -> AssessmentDefinition {
    AssessmentDefinition::new(AssessmentId::new("assess.internal-audit.baseline"))
}

fn json_object_keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("JSON must be an object")
        .keys()
        .cloned()
        .collect()
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("canonical catalog must load")
}

fn fail_closed_iso27001() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

/// IA-001 found case: `AuditProgramId` exists; there is no program document.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b001_audit_program_id_is_typed_id_only() {
    let id = AuditProgramId::new("audit:2026");
    assert_eq!(id.as_str(), "audit:2026");

    let ir_src = crate_src("weeping-angel-assurance-ir");
    assert!(
        !ir_src.join("audit.rs").is_file(),
        "IA-B001: audit.rs must not exist on characterization HEAD"
    );

    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles("IA-B001", &ir, audit_engine_needles());
    assert!(
        ir.contains("typed_id!(AuditProgramId)"),
        "IA-B001: AuditProgramId remains a typed_id!"
    );

    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        lib.contains("AuditProgramId")
            && !lib.contains("pub mod audit")
            && !lib.contains("AuditId"),
        "IA-B001: lib.rs re-exports AuditProgramId and has no audit module"
    );

    let ids = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(ids.contains("typed_id!(AuditProgramId)"));
    assert!(
        !ids.contains("typed_id!(AuditId)") && !ids.contains("typed_id!(AuditFindingId)"),
        "IA-B001: id.rs has no AuditId / AuditFindingId"
    );
}

/// IA-002 found case: request/capability bits fail-closed; enabling both still yields no program objects.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b002_audit_program_flags_compile_no_objects() {
    let requests = AssessmentRequests::default();
    assert!(
        !requests.audit_program && !requests.sampling && !requests.nonconformities,
        "IA-B002: AssessmentRequests defaults are fail-closed"
    );
    let req_json = serde_json::to_value(&requests).unwrap();
    assert_eq!(req_json["audit_program"], false);
    assert_eq!(req_json["sampling"], false);

    let caps = FrameworkCapabilities::default();
    assert!(
        !caps.supports_audit_program && !caps.supports_sampling && !caps.supports_nonconformities,
        "IA-B002: FrameworkCapabilities default is fail-closed"
    );

    let mut requested = empty_assessment();
    requested.requests.audit_program = true;
    let err = compile_framework(&requested, &fail_closed_iso27001())
        .expect_err("IA-B002: audit_program without support is CapabilityViolation");
    match err {
        FrameworkCompileError::CapabilityViolation { capability, .. } => {
            assert_eq!(capability, "supports_audit_program");
        }
        other => panic!("expected CapabilityViolation, got {other:?}"),
    }

    requested.requests.audit_program = false;
    requested.requests.sampling = true;
    let err = compile_framework(&requested, &fail_closed_iso27001())
        .expect_err("IA-B002: sampling without support is CapabilityViolation");
    match err {
        FrameworkCompileError::CapabilityViolation { capability, .. } => {
            assert_eq!(capability, "supports_sampling");
        }
        other => panic!("expected CapabilityViolation, got {other:?}"),
    }

    requested.requests.sampling = false;
    requested.requests.audit_program = true;
    let enabled = FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities {
            supports_audit_program: true,
            ..FrameworkCapabilities::default()
        },
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    };
    let compiled = compile_framework(&requested, &enabled)
        .expect("IA-B002: request + support compiles; it still constructs no program");
    assert!(compiled.validation.ok);
    assert!(!compiled.digest.is_empty());
    let compiled_json = serde_json::to_value(&compiled).unwrap();
    assert!(compiled_json.get("auditPrograms").is_none());
    assert!(compiled_json.get("audit_programs").is_none());
    assert!(compiled_json.get("audits").is_none());
}

/// IA-003 found case: Iso27007 is a pack-less compile selector.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b003_iso27007_is_packless_compile_selector() {
    assert_eq!(
        FrameworkProfile::try_from("iso27007").unwrap(),
        FrameworkProfile::Iso27007
    );
    assert_eq!(
        FrameworkProfile::try_from("iso-27007").unwrap(),
        FrameworkProfile::Iso27007
    );
    assert_eq!(FrameworkProfile::Iso27007.as_selector(), "iso-27007");

    let err = load_framework_pack("iso-27007", "2022").expect_err("no iso-27007 pack on disk");
    let msg = err.to_string();
    assert!(
        msg.contains("iso-27007") || msg.to_ascii_lowercase().contains("unknown"),
        "IA-B003: load_framework_pack reports unknown pack: {msg}"
    );
    assert!(
        !manifest_dir().join("frameworks/iso-27007").exists(),
        "IA-B003: frameworks/iso-27007/ must not exist"
    );

    let mut assessment = empty_assessment();
    assessment.requests.audit_program = true;
    let target = FrameworkTarget {
        profile: FrameworkProfile::Iso27007,
        capabilities: FrameworkCapabilities {
            supports_audit_program: true,
            ..FrameworkCapabilities::default()
        },
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    };
    let compiled = compile_framework(&assessment, &target)
        .expect("IA-B003: unknown pack is skipped; compile continues on in-memory assessment");
    assert!(compiled.applicable_requirements.is_empty());
    assert!(compiled.controls.is_empty());
    let compiled_json = serde_json::to_value(&compiled).unwrap();
    assert!(compiled_json.get("auditPrograms").is_none());

    let catalog_src = read_repo_file("crates/weeping-angel-canonical-catalog/src/lib.rs");
    assert!(
        catalog_src.contains("\"iso27007\"") && catalog_src.contains("\"iso-27007\""),
        "IA-B003: catalog still forbids iso27007 as a catalog segment"
    );
}

/// IA-004 found case: AssessmentDefinition has no audits / audit_programs inventory.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b004_assessment_definition_has_no_audits_inventory() {
    let assessment = empty_assessment();
    let json = serde_json::to_value(&assessment).unwrap();
    let mut keys = json_object_keys(&json);
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "assets".to_string(),
            "controls".to_string(),
            "evidence_requirements".to_string(),
            "exceptions".to_string(),
            "id".to_string(),
            "identities".to_string(),
            "implementations".to_string(),
            "mappings".to_string(),
            "processing_activities".to_string(),
            "requests".to_string(),
            "requirements".to_string(),
            "risks".to_string(),
            "schema_version".to_string(),
            "scope".to_string(),
            "tests".to_string(),
            "vendors".to_string(),
        ]
    );
    assert!(json.get("audits").is_none());
    assert!(json.get("audit_programs").is_none());
    assert!(json.get("auditPrograms").is_none());
    assert!(json.get("audit_findings").is_none());
    assert_eq!(json["schema_version"], ASSURANCE_IR_SCHEMA);

    let with_unknown = serde_json::from_value::<AssessmentDefinition>(json!({
        "id": "assess.internal-audit.unknown-key",
        "schema_version": ASSURANCE_IR_SCHEMA,
        "audit_programs": [{ "id": "audit:2026", "title": "Annual" }],
        "audits": [{ "id": "audit.q1", "programId": "audit:2026" }]
    }))
    .expect("unknown audit inventories are ignored; there is no field");
    let round = serde_json::to_value(&with_unknown).unwrap();
    assert!(round.get("audits").is_none());
    assert!(round.get("audit_programs").is_none());

    let assessment_src = read_repo_file("crates/weeping-angel-assurance-ir/src/assessment.rs");
    assert!(
        !assessment_src.contains("audit_programs")
            && !assessment_src.contains("audits")
            && !assessment_src.contains("audit_findings"),
        "IA-B004: assessment.rs must not name audit inventories"
    );
    assert!(assessment_src.contains("pub audit_program: bool"));

    empty_assessment()
        .validate()
        .expect("empty assessment still validates with no audit walk");

    let validation = read_repo_file("crates/weeping-angel-assurance-ir/src/validation.rs");
    for needle in [
        "audit_program",
        "audit_programs",
        "audits",
        "independence",
        "sign_off",
        "sample_digest",
    ] {
        assert!(
            !validation.contains(needle),
            "IA-B004: validation.rs must not mention `{needle}` today"
        );
    }
}

/// IA-005 found case: governance catalog is freshness / attestation, not audit quality.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b005_governance_catalog_is_freshness_attestation_only() {
    let catalog = load_catalog();
    let control = catalog
        .control("control.governance.internal-audit")
        .expect("governance catalog already publishes internal-audit capability");
    assert_eq!(control.automation, "hybrid");
    assert!(
        control
            .description
            .contains("internal-audit record exists inside the required window")
    );
    assert!(
        control
            .objective
            .to_ascii_lowercase()
            .contains("independence is not inferred from a file")
    );
    assert!(
        control
            .evidence
            .iter()
            .any(|e| e == "evidence.governance.internal-audit")
    );
    assert!(
        control
            .tests
            .iter()
            .any(|t| t == "test.governance.internal-audit-current")
    );

    let test = catalog
        .tests()
        .get("test.governance.internal-audit-current")
        .expect("freshness test remains");
    assert_eq!(test.control, "control.governance.internal-audit");
    assert_eq!(
        test.expression.get("op").and_then(|v| v.as_str()),
        Some("fresh-within")
    );
    assert_eq!(
        test.expression.get("evidence").and_then(|v| v.as_str()),
        Some("evidence.governance.internal-audit")
    );
    assert_eq!(
        test.expression.get("field").and_then(|v| v.as_str()),
        Some("audited_at")
    );
    assert_eq!(
        test.expression.get("duration").and_then(|v| v.as_str()),
        Some("365d")
    );

    let program = catalog
        .control("control.governance.audit-program")
        .expect("governance catalog owns control.governance.audit-program");
    assert_eq!(program.automation, "manual");
    assert!(
        program
            .description
            .contains("attested in addition to a current audit record")
    );
    assert!(
        program
            .objective
            .contains("A single audit file does not prove a program exists")
    );
    assert!(
        program
            .tests
            .iter()
            .any(|t| t == "test.governance.audit-program-attested")
    );

    let attested = catalog
        .tests()
        .get("test.governance.audit-program-attested")
        .expect("manual-review program test remains");
    assert_eq!(
        attested.expression.get("op").and_then(|v| v.as_str()),
        Some("manual-review")
    );
    assert!(
        attested
            .required_evidence
            .iter()
            .any(|e| e == "evidence.governance.internal-audit")
    );
    assert!(
        attested
            .required_evidence
            .iter()
            .any(|e| e == "evidence.manual.attestation")
    );

    let evidence = catalog
        .evidence()
        .get("evidence.governance.internal-audit")
        .expect("evidence.governance.internal-audit remains in the catalog");
    assert_eq!(evidence.evidence_type, "governance.internal-audit");

    let evidence_toml = read_repo_file("catalog/canonical/v1/evidence/governance.toml");
    let tests_toml = read_repo_file("catalog/canonical/v1/tests/governance.toml");
    let controls_toml = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    for blob in [&evidence_toml, &tests_toml, &controls_toml] {
        let folded = blob.to_ascii_lowercase();
        assert!(
            !folded.contains("audit passed") && !folded.contains("audit-passed"),
            "IA-B005: catalog must not encode an audit-passed conclusion"
        );
    }
}

/// IA-006 found case: sampling is a capability flag; there is no sample engine.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b006_sampling_is_a_flag_without_engine() {
    let product = product_crate_sources_joined();
    forbid_needles(
        "IA-B006",
        &product,
        &[
            "struct AuditSample",
            "struct AuditSampleProposal",
            "enum SampleMethod",
            "fn propose_sample",
            "fn accept_sample",
            "sampleDigest",
            "populationDigest",
        ],
    );
    assert!(
        crate_sources_joined("weeping-angel-framework").contains("supports_sampling")
            && crate_sources_joined("weeping-angel-assurance-ir").contains("pub sampling: bool"),
        "IA-B006: sampling remains a request/capability pair"
    );
}

/// IA-007 found case: lineage snapshots exist; audits do not pin them to an auditor.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b007_snapshots_exist_audits_do_not_pin_them() {
    let snapshot = seal_evidence_snapshot(
        ["sha256:env-a".to_string(), "sha256:env-b".to_string()],
        ["run-1".to_string()],
    );
    assert_eq!(snapshot.schema, LINEAGE_SNAPSHOT_SCHEMA);
    assert_eq!(
        snapshot.envelope_digests,
        vec!["sha256:env-a".to_string(), "sha256:env-b".to_string()]
    );
    assert!(!snapshot.digest.is_empty());
    let snap_json = serde_json::to_value(&snapshot).unwrap();
    assert!(snap_json.get("auditor").is_none());
    assert!(snap_json.get("pinnedBy").is_none());
    assert!(snap_json.get("auditId").is_none());

    let run = AssessmentRun {
        id: AssessmentId::new("run.internal-audit.baseline"),
        framework: "iso-27001".into(),
        framework_pack_digest: "pack".into(),
        assessment_definition_digest: "def".into(),
        started_at: "2026-01-01T00:00:00Z".into(),
        completed_at: "2026-01-01T00:00:01Z".into(),
        scope: "org".into(),
        collector_runs: Vec::new(),
        evidence_snapshot_digest: snapshot.digest.clone(),
        result_digest: "result".into(),
        status: "completed".into(),
        canonical_catalog_pin: "catalog".into(),
        applicability_snapshot_id: String::new(),
        as_of: "2026-01-01T00:00:00Z".into(),
    };
    assert_eq!(run.evidence_snapshot_digest, snapshot.digest);
    let run_json = serde_json::to_value(&run).unwrap();
    assert!(run_json.get("auditor").is_none());
    assert_eq!(run_json["evidenceSnapshotDigest"], snapshot.digest);

    let _ = EvidenceSnapshot {
        schema: LINEAGE_SNAPSHOT_SCHEMA.into(),
        envelope_digests: Vec::new(),
        collection_run_ids: Vec::new(),
        digest: String::new(),
    };

    let product = product_crate_sources_joined();
    forbid_needles(
        "IA-B007",
        &product,
        &["struct AuditEvidencePin", "fn pin_evidence", "pinnedBy"],
    );
}

/// IA-008 found case: no independence, audit findings, incomplete gate, or human sign-off.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b008_no_independence_findings_or_human_sign_off() {
    let product = product_crate_sources_joined();
    forbid_needles(
        "IA-B008",
        &product,
        &[
            "struct IndependenceRecord",
            "struct AuditFinding",
            "struct AuditSignOff",
            "enum AuditConclusion",
            "fn record_finding",
            "fn conclude_audit",
            "fn sign_off",
            "fn replay_audit",
        ],
    );

    assert!(
        !crate_src("weeping-angel-assurance")
            .join("audit.rs")
            .is_file(),
        "IA-B008: assurance crate has no audit engine module"
    );

    let finding = Finding::builder("recon", "unprotected-branch")
        .title("Unprotected default branch")
        .description("scanner output is not an audit finding")
        .build();
    assert_eq!(finding.id, "unprotected-branch");
    let finding_src = read_repo_file("src/finding.rs");
    assert!(finding_src.contains("pub struct Finding"));
    assert!(
        !finding_src.contains("AuditFinding") && !finding_src.contains("IndependenceRecord"),
        "src/finding.rs must not promote into audit IR"
    );

    let _ = Effectiveness::Effective;
    let control = Control::new(
        ControlId::new("control.access.mfa"),
        "MFA",
        "Require multi-factor authentication.",
    );
    let json = serde_json::to_value(&control).unwrap();
    assert!(json.get("signOff").is_none());
    assert!(json.get("conclusion").is_none());
}

/// Schema lock: IR stays assurance-ir/v1; lineage snapshot schema is reused, not forked.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b_schema_pins_remain() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    assert_eq!(
        LINEAGE_SNAPSHOT_SCHEMA,
        "weeping-angel/assessment-lineage/v1"
    );
}

/// Dual-suite names are listed in root Cargo.toml.
#[ignore = "superseded by target suite"]
#[test]
fn ia_b009_dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_internal_audit_baseline")
            && toml.contains("sdd_internal_audit_target")
            && toml.contains("tests/contracts/internal_audit.baseline.rs")
            && toml.contains("tests/contracts/internal_audit.target.rs"),
        "dual-suite must be listed in root Cargo.toml"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/internal_audit.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/internal_audit.target.rs")
            .is_file()
    );
}
