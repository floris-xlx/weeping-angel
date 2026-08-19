//! SUPERSEDED by `sdd_controlled_documents_target`.
//!
//! Historical characterization of SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`
//! (`docs/specs/controlled-documents.md` §3): no `ControlledDocument` IR, no
//! eval-at-T helpers, catalog `control.governance.document-control` was
//! hybrid/manual-review attestation (consumed, not a registry), evidence
//! envelopes already carried `content_digest`, and a policy observation did not
//! make an execution-required control `Effectiveness::Effective`.
//!
//! Target `sdd_controlled_documents_target` is the source of truth. This
//! baseline is skipped (`#[ignore = "superseded by target suite"]`).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentId, AssetId, ControlId, ControlTestId,
    canonical_digest, validate_assessment_ir,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
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

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(
        present.is_empty(),
        "{label}: document-control IR must be absent on characterization HEAD; found {present:?}"
    );
}

fn fresh_provenance() -> EvidenceProvenance {
    EvidenceProvenance {
        collector_id: "fixture.controlled-documents-baseline".into(),
        collected_at: Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap(),
        scope: "org:weeping-angel".into(),
        asset: AssetId::new("org:weeping-angel"),
    }
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 19, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

/// CD-B001 found case: IR has no ControlledDocument type or document module.
#[ignore = "superseded by target suite"]
#[test]
fn cd_b001_no_controlled_document_ir_type() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "CD-B001",
        &ir,
        &[
            "struct ControlledDocument",
            "struct DocumentVersion",
            "struct DocumentControlRegistry",
            "enum DocumentType",
            "fn is_operational_current_at",
            "fn effective_version_at",
            "fn acknowledgement_coverage",
            "typed_id!(ControlledDocumentId)",
        ],
    );
    assert!(
        !crate_src("weeping-angel-assurance-ir")
            .join("document.rs")
            .is_file(),
        "CD-B001: document.rs must not exist on characterization HEAD"
    );
    assert!(
        !ir.contains("pub mod document;"),
        "CD-B001: lib.rs must not declare mod document"
    );
}

/// CD-B002 found case: no eval-at-T / supersession / approval helpers for documents.
#[ignore = "superseded by target suite"]
#[test]
fn cd_b002_no_document_evaluation_helpers() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    forbid_needles(
        "CD-B002",
        &ir,
        &[
            "DocumentLinkUniverse",
            "MissingApproval",
            "is_operational_current_at",
            "within_review_window",
            "acknowledgement_required",
            "approval_evidence_digests",
            "supersedes_version",
        ],
    );
}

/// CD-B003 found case: catalog document-control is hybrid attestation, not a registry.
#[ignore = "superseded by target suite"]
#[test]
fn cd_b003_catalog_document_control_is_attestation() {
    let catalog = CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1")).unwrap();
    let control = catalog
        .control("control.governance.document-control")
        .expect("governance catalog already publishes document-control; consume it");
    assert_eq!(control.automation, "hybrid");
    assert!(
        control
            .evidence
            .iter()
            .any(|e| e == "evidence.manual.attestation")
    );
    assert!(
        control
            .tests
            .iter()
            .any(|t| t == "test.governance.document-control-attested")
    );

    let tests_toml = read_repo_file("catalog/canonical/v1/tests/governance.toml");
    assert!(tests_toml.contains("id = \"test.governance.document-control-attested\""));
    assert!(tests_toml.contains("control = \"control.governance.document-control\""));
    assert!(tests_toml.contains("kind = \"hybrid\""));
    assert!(tests_toml.contains("required_evidence = [\"evidence.manual.attestation\"]"));
    let attested_block = tests_toml
        .split("id = \"test.governance.document-control-attested\"")
        .nth(1)
        .expect("attested test block");
    let attested_block = attested_block
        .split("[[test]]")
        .next()
        .expect("until next test");
    assert!(
        attested_block.contains("op = \"manual-review\""),
        "CD-B003: document-control test stays manual-review"
    );
}

/// CD-B004 found case: envelopes already expose content_digest via canonical_digest.
#[ignore = "superseded by target suite"]
#[test]
fn cd_b004_evidence_envelope_already_has_content_digest() {
    let mut obs = EvidenceObservation::new(EvidenceType::new("evidence.governance.policy"));
    obs = obs.with_fact("policy_kind", "information-security");
    let env = EvidenceEnvelope::seal(obs, fresh_provenance()).unwrap();
    assert!(!env.content_digest().is_empty());
    assert_eq!(env.content_digest(), env.digest());
    assert!(env.evidence_id().starts_with("ev:sha256:"));

    let evidence_src = crate_sources_joined("weeping-angel-evidence");
    assert!(evidence_src.contains("content_digest"));
    assert!(evidence_src.contains("struct EvidenceEnvelope"));
    assert!(evidence_src.contains("struct EvidenceLedger"));
    assert!(evidence_src.contains("canonical_digest"));
    assert!(
        !evidence_src.contains("struct ControlledDocument"),
        "CD-B004: evidence crate must not grow a document registry"
    );
}

/// CD-B005 found case: policy envelope does not make execution-required tests Effective.
#[ignore = "superseded by target suite"]
#[test]
fn cd_b005_policy_observation_does_not_imply_effectiveness() {
    let mut obs = EvidenceObservation::new(EvidenceType::new("evidence.governance.policy"));
    obs = obs.with_fact("policy_kind", "information-security");
    let env = EvidenceEnvelope::seal(obs, fresh_provenance()).unwrap();

    let mut evidence = EvidenceSet::new();
    evidence.insert(env);

    let test = CompiledControlTest::builder()
        .id(ControlTestId::new("test.source.default-branch-protection"))
        .control_id(ControlId::new("control.source.default-branch-protection"))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new("source.branch.protection"))
        .expr(TestExpr::Exists(EvidenceSelector::of_type(
            EvidenceType::new("source.branch.protection"),
        )))
        .build();

    let result = evaluate(&test, &evidence, &fresh_context());
    assert_ne!(
        result.effectiveness,
        Effectiveness::Effective,
        "CD-B005: a policy observation must not satisfy execution evidence"
    );
    assert_eq!(result.effectiveness, Effectiveness::InsufficientEvidence);
}

/// CD-B006 found case: ManualReview stays ManualReviewRequired (catalog document-control shape).
#[ignore = "superseded by target suite"]
#[test]
fn cd_b006_manual_review_is_not_auto_effective() {
    let test = CompiledControlTest::builder()
        .id(ControlTestId::new(
            "test.governance.document-control-attested",
        ))
        .control_id(ControlId::new("control.governance.document-control"))
        .kind(ControlTestKind::Manual)
        .require(EvidenceType::new("evidence.manual.attestation"))
        .expr(TestExpr::ManualReview)
        .build();
    let result = evaluate(&test, &EvidenceSet::new(), &fresh_context());
    assert_eq!(result.effectiveness, Effectiveness::ManualReviewRequired);
}

/// CD-B007 found case: generic IR schema is still assurance-ir/v1; no document ISO clauses.
#[ignore = "superseded by target suite"]
#[test]
fn cd_b007_schema_remains_assurance_ir_v1() {
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(ir.contains("assurance-ir/v1"));
    assert!(
        !ir.to_ascii_lowercase().contains("annex a."),
        "CD-B007: generic IR must not carry Annex A identifiers"
    );
    let _ = canonical_digest(&"controlled-documents-baseline").unwrap();
}

/// CD-B008 found case: IR validation has no document link graph.
#[ignore = "superseded by target suite"]
#[test]
fn cd_b008_validation_has_no_document_link_graph() {
    let validation =
        fs::read_to_string(crate_src("weeping-angel-assurance-ir").join("validation.rs")).unwrap();
    forbid_needles(
        "CD-B008",
        &validation,
        &[
            "ControlledDocument",
            "DocumentControlRegistry",
            "DocumentLinkUniverse",
            "dangling document",
            "obligation_ids",
        ],
    );

    let assessment_src =
        fs::read_to_string(crate_src("weeping-angel-assurance-ir").join("assessment.rs")).unwrap();
    assert!(
        !assessment_src.contains("pub documents:"),
        "CD-B008: AssessmentDefinition must not carry a documents vec on characterization HEAD"
    );

    let assessment =
        AssessmentDefinition::new(AssessmentId::new("assess.controlled-documents.baseline"));
    validate_assessment_ir(&assessment)
        .expect("empty assessment remains valid without a document graph");
}

/// CD-B009 found case: dual-suite binaries are listed; this characterization is not auto-discovered.
#[ignore = "superseded by target suite"]
#[test]
fn cd_b009_dual_suite_registered() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(cargo.contains("name = \"sdd_controlled_documents_baseline\""));
    assert!(cargo.contains("path = \"tests/contracts/controlled_documents.baseline.rs\""));
    assert!(cargo.contains("name = \"sdd_controlled_documents_target\""));
    assert!(cargo.contains("path = \"tests/contracts/controlled_documents.target.rs\""));
}
