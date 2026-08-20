//! Target suite for Prompt 12 (controlled document and policy registry).
//!
//! Encodes DESIRED behavior in `docs/specs/controlled-documents.md` §4 / §4.12
//! (CD-001–014). Calls the public IR helpers (`effective_version_at`,
//! `is_operational_current_at`, `approve`, `append_version`,
//! `acknowledgement_coverage`, `validate`). Do not `#[ignore]`.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AcknowledgementRecord, ControlId, ControlledDocument,
    ControlledDocumentId, DocumentControlError, DocumentControlRegistry, DocumentLinkUniverse,
    DocumentType, DocumentVersion, DocumentVersionStatus, IdentityId, InformationClassification,
    ObligationId, PrincipalRef, RetentionMetadata, RiskId, SubjectKind, SubjectSelector,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};

const DOCUMENT_RS: &str = "crates/weeping-angel-assurance-ir/src/document.rs";

fn crate_src(name: &str) -> PathBuf {
    let path = manifest_dir().join("crates").join(name).join("src");
    assert!(
        path.is_dir(),
        "expected crate sources at {}",
        path.display()
    );
    path
}

fn document_rs_path() -> PathBuf {
    crate_src("weeping-angel-assurance-ir").join("document.rs")
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

fn t() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

fn effective_from() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
}

fn review_by_in_window() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap()
}

fn review_by_overdue() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()
}

fn owner() -> PrincipalRef {
    PrincipalRef::Identity(IdentityId::new("identity:ciso"))
}

fn policy_id() -> ControlledDocumentId {
    ControlledDocumentId::new("doc.policy.information-security")
}

fn digest_a() -> &'static str {
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}

fn digest_b() -> &'static str {
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
}

fn approval_digest() -> String {
    "sha256:approval-evidence-1".into()
}

fn new_policy() -> ControlledDocument {
    ControlledDocument::new(
        policy_id(),
        DocumentType::Policy,
        "Information Security Policy",
        owner(),
    )
}

fn approve_current(doc: &mut ControlledDocument, version: &str, review_by: DateTime<Utc>) {
    doc.approve(
        version,
        vec![owner()],
        vec![approval_digest()],
        effective_from(),
        Some(review_by),
    )
    .expect("approve");
}

fn current_policy_registry() -> DocumentControlRegistry {
    let mut doc = new_policy();
    doc.append_version(DocumentVersion::draft("1.0", digest_a()))
        .unwrap();
    approve_current(&mut doc, "1.0", review_by_in_window());
    let mut registry = DocumentControlRegistry::new();
    registry.insert(doc).unwrap();
    registry
}

/// CD-001: approved + effective-at-T + in review window + current pointer is operational-current.
#[test]
fn cd_001_current_policy_approved_effective_in_review_window() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-001",
        &src,
        &[
            "pub struct ControlledDocument",
            "pub struct DocumentVersion",
            "pub struct DocumentControlRegistry",
            "fn is_approved",
            "fn is_effective_at",
            "fn within_review_window",
            "fn is_operational_current_at",
            "fn effective_version_at",
            "current_version",
            "effective_from",
            "review_by",
        ],
    );
    assert!(
        src.contains("DocumentVersionStatus") && src.contains("Approved"),
        "CD-001: operational-current requires status Approved (derived currency, not stored Effective)"
    );

    let registry = current_policy_registry();
    let id = policy_id();
    let at = t();
    let version = registry
        .effective_version_at(&id, at)
        .expect("CD-001: current policy must be selected at T");
    assert_eq!(version.version, "1.0");
    assert!(version.is_approved());
    assert!(version.is_effective_at(at));
    assert!(version.within_review_window(at));
    assert!(version.is_operational_current_at(at));
    assert_eq!(
        registry.current(&id).map(|v| v.version.as_str()),
        Some("1.0")
    );
    assert!(
        registry
            .get(&id)
            .expect("doc")
            .is_operational_current_at(at)
    );
}

/// CD-002: review-overdue approved policy is stale (not operational-current) and still queryable.
#[test]
fn cd_002_stale_policy_review_overdue() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-002",
        &src,
        &[
            "fn is_operational_current_at",
            "fn within_review_window",
            "fn is_effective_at",
            "review_by",
            "fn version",
        ],
    );
    assert!(
        !src.contains("Effectiveness::StaleEvidence"),
        "CD-002: document stale is review-window metadata, not Effectiveness::StaleEvidence"
    );

    let mut doc = new_policy();
    doc.append_version(DocumentVersion::draft("1.0", digest_a()))
        .unwrap();
    approve_current(&mut doc, "1.0", review_by_overdue());
    let mut registry = DocumentControlRegistry::new();
    registry.insert(doc).unwrap();
    let id = policy_id();
    let at = t();
    let version = registry
        .version(&id, "1.0")
        .expect("CD-002: stale version remains queryable");
    assert!(version.is_approved());
    assert!(version.is_effective_at(at));
    assert!(!version.within_review_window(at));
    assert!(!version.is_operational_current_at(at));
    assert!(registry.effective_version_at(&id, at).is_none());
}

/// CD-003: draft-only identity is not treated as effective.
#[test]
fn cd_003_draft_only_policy_not_effective() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-003",
        &src,
        &[
            "enum DocumentVersionStatus",
            "Draft",
            "fn effective_version_at",
            "fn is_effective_at",
            "fn is_operational_current_at",
        ],
    );
    assert!(
        src.contains("current_version"),
        "CD-003: constructor starts with no versions and current_version = None"
    );

    let mut doc = new_policy();
    assert!(doc.current_version.is_none());
    doc.append_version(DocumentVersion::draft("1.0", digest_a()))
        .unwrap();
    assert_eq!(
        doc.version("1.0").map(|v| v.status),
        Some(DocumentVersionStatus::Draft)
    );
    assert!(!doc.version("1.0").unwrap().is_effective_at(t()));
    assert!(!doc.version("1.0").unwrap().is_operational_current_at(t()));
    assert!(doc.effective_version_at(t()).is_none());
    let mut registry = DocumentControlRegistry::new();
    registry.insert(doc).unwrap();
    assert!(registry.effective_version_at(&policy_id(), t()).is_none());
}

/// CD-004: empty approvers or empty approval evidence cannot approve.
#[test]
fn cd_004_missing_approval() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-004",
        &src,
        &[
            "MissingApproval",
            "approvers",
            "approval_evidence_digests",
            "fn approve",
            "DocumentControlError",
        ],
    );

    let mut doc = new_policy();
    doc.append_version(DocumentVersion::draft("1.0", digest_a()))
        .unwrap();
    let empty_approvers = doc.approve(
        "1.0",
        vec![],
        vec![approval_digest()],
        effective_from(),
        Some(review_by_in_window()),
    );
    assert!(
        matches!(
            empty_approvers,
            Err(DocumentControlError::MissingApproval { .. })
        ),
        "CD-004: empty approvers cannot approve: {empty_approvers:?}"
    );

    let empty_evidence = doc.approve(
        "1.0",
        vec![owner()],
        vec![],
        effective_from(),
        Some(review_by_in_window()),
    );
    assert!(
        matches!(
            empty_evidence,
            Err(DocumentControlError::MissingApproval { .. })
        ),
        "CD-004: empty approval evidence cannot approve: {empty_evidence:?}"
    );
    assert_eq!(
        doc.version("1.0").map(|v| v.status),
        Some(DocumentVersionStatus::Draft)
    );
    assert!(doc.effective_version_at(t()).is_none());
}

/// CD-005: superseded version remains addressable; current pointer moves.
#[test]
fn cd_005_superseded_document_remains_addressable() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-005",
        &src,
        &[
            "supersedes_version",
            "current_version",
            "fn current",
            "fn version",
            "fn append_version",
        ],
    );

    let mut doc = new_policy();
    doc.append_version(DocumentVersion::draft("1.0", digest_a()))
        .unwrap();
    approve_current(&mut doc, "1.0", review_by_in_window());
    doc.append_version(DocumentVersion::draft("1.1", digest_b()).with_supersedes("1.0"))
        .unwrap();
    doc.approve(
        "1.1",
        vec![owner()],
        vec![approval_digest()],
        Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        Some(Utc.with_ymd_and_hms(2027, 9, 1, 0, 0, 0).unwrap()),
    )
    .unwrap();

    let mut registry = DocumentControlRegistry::new();
    registry.insert(doc).unwrap();
    let id = policy_id();
    assert!(
        registry.version(&id, "1.0").is_some(),
        "CD-005: superseded 1.0 remains addressable"
    );
    assert_eq!(
        registry.current(&id).map(|v| v.version.as_str()),
        Some("1.1")
    );
    assert_eq!(
        registry
            .version(&id, "1.1")
            .and_then(|v| v.supersedes_version.as_deref()),
        Some("1.0")
    );
    assert_eq!(
        registry
            .effective_version_at(&id, t())
            .map(|v| v.version.as_str()),
        Some("1.0"),
        "CD-005: at T before 1.1 effective_from, 1.0 is still the version in force"
    );
}

/// CD-006: changing digest after approval creates a new version; approved artifact_digest is immutable.
#[test]
fn cd_006_changed_digest_creates_new_version() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-006",
        &src,
        &[
            "artifact_digest",
            "fn append_version",
            "Approved",
            "Immutable",
        ],
    );

    let mut doc = new_policy();
    doc.append_version(DocumentVersion::draft("1.0", digest_a()))
        .unwrap();
    approve_current(&mut doc, "1.0", review_by_in_window());
    let overwrite = doc.set_artifact_digest("1.0", digest_b());
    assert!(
        matches!(
            overwrite,
            Err(DocumentControlError::ImmutableApprovedArtifact { .. })
        ),
        "CD-006: approved digest is immutable: {overwrite:?}"
    );
    assert_eq!(doc.version("1.0").unwrap().artifact_digest(), digest_a());

    doc.append_version(DocumentVersion::draft("1.1", digest_b()).with_supersedes("1.0"))
        .unwrap();
    assert_eq!(doc.version("1.0").unwrap().artifact_digest(), digest_a());
    assert_eq!(doc.version("1.1").unwrap().artifact_digest(), digest_b());
}

/// CD-007: required acknowledgement gaps report incomplete coverage.
#[test]
fn cd_007_required_acknowledgement_gaps() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-007",
        &src,
        &[
            "fn acknowledgement_coverage",
            "acknowledgement_required",
            "required_acknowledgement_subjects",
            "pub struct AcknowledgementRecord",
            "pub struct AcknowledgementCoverage",
            "complete",
        ],
    );

    let mut doc = new_policy();
    let mut version = DocumentVersion::draft("1.0", digest_a());
    version.acknowledgement_required = true;
    version.required_acknowledgement_subjects = vec!["alice".into(), "bob".into()];
    version.acknowledgements.push(AcknowledgementRecord {
        subject_id: "alice".into(),
        acknowledged_at: t(),
        evidence_digest: None,
    });
    doc.append_version(version).unwrap();
    let coverage = doc.version("1.0").unwrap().acknowledgement_coverage();
    assert!(!coverage.complete, "CD-007: bob missing");
    assert_eq!(coverage.required, 2);
    assert_eq!(coverage.recorded, 1);
}

/// CD-008: retention metadata is present and queryable.
#[test]
fn cd_008_retention_metadata_queryable() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-008",
        &src,
        &[
            "pub struct RetentionMetadata",
            "retain_until",
            "legal_hold",
            "retention_period_seconds",
            "fn retention",
        ],
    );

    let retain_until = Utc.with_ymd_and_hms(2031, 8, 19, 0, 0, 0).unwrap();
    let mut doc = new_policy();
    doc.append_version(DocumentVersion::draft("1.0", digest_a()).with_retention(
        RetentionMetadata {
            retain_until: Some(retain_until),
            retention_period_seconds: Some(5 * 365 * 24 * 3600),
            legal_hold: true,
            disposition: Some("review-then-destroy".into()),
        },
    ))
    .unwrap();
    let retention = doc.version("1.0").unwrap().retention();
    assert_eq!(retention.retain_until, Some(retain_until));
    assert!(retention.legal_hold);
    assert!(retention.retention_period_seconds.is_some());
}

/// CD-009: linked/present document does not imply Effectiveness::Effective for execution-evidence tests.
#[test]
fn cd_009_document_does_not_imply_control_effective() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-009",
        &src,
        &["pub struct ControlledDocument", "control_ids"],
    );
    forbid_needles(
        "CD-009",
        &src,
        &["effectiveness: Effectiveness", "pub effectiveness:"],
    );

    let tests_toml = read_repo_file("catalog/canonical/v1/tests/governance.toml");
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
        "CD-009: catalog document-control stays manual-review"
    );

    let control_test = crate_sources_joined("weeping-angel-control-test");
    assert!(
        control_test.contains("ManualReviewRequired"),
        "CD-009: ManualReview must remain ManualReviewRequired"
    );
    assert!(
        control_test.contains("InsufficientEvidence"),
        "CD-009: missing execution evidence stays InsufficientEvidence, not Effective"
    );

    let mut doc = new_policy();
    let mut version = DocumentVersion::draft("1.0", digest_a());
    version.control_ids = vec![ControlId::new("control.source.default-branch-protection")];
    version.classification = InformationClassification::Confidential;
    doc.append_version(version).unwrap();
    approve_current(&mut doc, "1.0", review_by_in_window());
    assert!(
        doc.effective_version_at(t()).is_some(),
        "CD-009: a current policy may be linked; it still is not a test result"
    );

    let mut obs = EvidenceObservation::new(EvidenceType::new("evidence.governance.policy"));
    obs = obs.with_fact("policy_kind", "information-security");
    let env = EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.controlled-documents-target".into(),
            collected_at: t(),
            scope: "org:weeping-angel".into(),
            asset: weeping_angel_assurance_ir::AssetId::new("org:weeping-angel"),
        },
    )
    .unwrap();
    let mut evidence = EvidenceSet::new();
    evidence.insert(env);

    let test = CompiledControlTest::builder()
        .id(weeping_angel_assurance_ir::ControlTestId::new(
            "test.source.default-branch-protection",
        ))
        .control_id(ControlId::new("control.source.default-branch-protection"))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new("source.branch.protection"))
        .expr(TestExpr::Exists(EvidenceSelector::of_type(
            EvidenceType::new("source.branch.protection"),
        )))
        .build();
    let ctx = AssessmentContext {
        now: t(),
        max_age: Duration::from_secs(24 * 3600),
    };
    let result = evaluate(&test, &evidence, &ctx);
    assert_ne!(result.effectiveness, Effectiveness::Effective);
    assert_eq!(result.effectiveness, Effectiveness::InsufficientEvidence);

    let manual = CompiledControlTest::builder()
        .id(weeping_angel_assurance_ir::ControlTestId::new(
            "test.governance.document-control-attested",
        ))
        .control_id(ControlId::new("control.governance.document-control"))
        .kind(ControlTestKind::Manual)
        .require(EvidenceType::new("evidence.manual.attestation"))
        .expr(TestExpr::ManualReview)
        .build();
    let manual_result = evaluate(&manual, &EvidenceSet::new(), &ctx);
    assert_eq!(
        manual_result.effectiveness,
        Effectiveness::ManualReviewRequired
    );
}

/// CD-010: dangling control, obligation, risk, or scope ids fail closed.
#[test]
fn cd_010_dangling_refs_fail_closed() {
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles(
        "CD-010",
        &src,
        &[
            "pub struct DocumentLinkUniverse",
            "obligation_ids",
            "risk_ids",
            "control_ids",
            "fn validate",
        ],
    );
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    require_needles(
        "CD-010",
        &ir,
        &["typed_id!(ObligationId)", "typed_id!(ControlledDocumentId)"],
    );

    let mut doc = new_policy();
    let mut version = DocumentVersion::draft("1.0", digest_a());
    version.control_ids = vec![ControlId::new("control.governance.document-control")];
    version.obligation_ids = vec![ObligationId::new("obligation.customer-dpa")];
    version.risk_ids = vec![RiskId::new("risk:source-tamper")];
    version.applicability = vec![SubjectSelector {
        kind: SubjectKind::Identity,
        ids: ["identity:alice".into()].into_iter().collect(),
        tags: Default::default(),
        scope: Default::default(),
    }];
    doc.append_version(version).unwrap();
    let mut registry = DocumentControlRegistry::new();
    registry.insert(doc).unwrap();

    let empty = DocumentLinkUniverse::default();
    let err = registry
        .validate(&empty)
        .expect_err("CD-010: empty universe + linked ids must fail closed");
    assert!(
        matches!(err, DocumentControlError::DanglingReference { .. }),
        "CD-010: expected dangling reference, got {err:?}"
    );

    let mut universe = DocumentLinkUniverse::default();
    universe
        .control_ids
        .insert(ControlId::new("control.governance.document-control"));
    universe
        .obligation_ids
        .insert(ObligationId::new("obligation.customer-dpa"));
    universe.risk_ids.insert(RiskId::new("risk:source-tamper"));
    universe.subject_ids.insert("identity:alice".into());
    registry
        .validate(&universe)
        .expect("CD-010: populated universe accepts the same links");

    let mut empty_links = new_policy();
    empty_links
        .append_version(DocumentVersion::draft("1.0", digest_a()))
        .unwrap();
    empty_links
        .validate(&DocumentLinkUniverse::default())
        .expect("CD-010: empty link lists are valid against an empty universe");
}

/// CD-011: dual-suite runs as a harness module.
#[test]
fn cd_011_dual_suite_registered() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        !cargo.contains("name = \"sdd_controlled_documents_baseline\""),
        "CD-011: baseline suite must be listed"
    );
    assert!(!cargo.contains("path = \"tests/contracts/controlled_documents.baseline.rs\""));
    assert!(
        harness_src().contains("controlled_documents.target.rs"),
        "CD-011: target suite must be listed"
    );
    assert!(harness_src().contains("controlled_documents.target.rs"));
    let spec = read_repo_file("docs/specs/controlled-documents.md");
    assert!(
        spec.contains("Controlled Document and Policy Registry"),
        "CD-011: human spec SSOT must exist"
    );
}

/// CD-012: consume catalog document-control; do not rewrite TOML.
#[test]
fn cd_012_catalog_document_control_consumed_not_rewritten() {
    let controls = read_repo_file("catalog/canonical/v1/controls/governance.toml");
    assert!(
        controls.contains("id = \"control.governance.document-control\""),
        "CD-012: consume control.governance.document-control"
    );
    let control_block = controls
        .split("id = \"control.governance.document-control\"")
        .nth(1)
        .expect("control block");
    let control_block = control_block
        .split("[[control]]")
        .next()
        .expect("until next control");
    assert!(
        control_block.contains("automation = \"hybrid\""),
        "CD-012: document-control stays hybrid"
    );
    assert!(
        control_block.contains("test.governance.document-control-attested"),
        "CD-012: attested test remains linked"
    );

    let tests_toml = read_repo_file("catalog/canonical/v1/tests/governance.toml");
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
        "CD-012: document-control test stays manual-review"
    );
    assert!(
        attested_block.contains("kind = \"hybrid\""),
        "CD-012: document-control test stays hybrid"
    );
}

/// CD-013: schema stays assurance-ir/v1; no ISO clause / Annex A text on generic IR documents.
#[test]
fn cd_013_schema_stays_assurance_ir_v1() {
    let lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        lib.contains("assurance-ir/v1"),
        "CD-013: do not fork ASSURANCE_IR_SCHEMA"
    );
    let src = fs::read_to_string(document_rs_path()).expect(DOCUMENT_RS);
    require_needles("CD-013", &src, &["schema_version", "ASSURANCE_IR_SCHEMA"]);
    assert!(
        !src.to_ascii_lowercase().contains("annex a"),
        "CD-013: ControlledDocument IR must not carry Annex A"
    );
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        !ir.to_ascii_lowercase().contains("annex a."),
        "CD-013: generic IR must not carry Annex A identifiers"
    );
    forbid_needles("CD-013", &src, &["ISO/IEC 27001", "Annex A.", "clause 7.5"]);
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
    let doc = new_policy();
    assert_eq!(doc.schema_version, ASSURANCE_IR_SCHEMA);
}

/// CD-014: evidence crate stays conclusion-free.
#[test]
fn cd_014_evidence_crate_stays_conclusion_free() {
    let evidence = crate_sources_joined("weeping-angel-evidence");
    forbid_needles(
        "CD-014",
        &evidence,
        &[
            "struct ControlledDocument",
            "Effectiveness::Effective",
            "fn mark_effective",
            "approvedPolicy",
        ],
    );
    assert!(
        evidence.contains("never compliance"),
        "CD-014: envelopes remain claim-rejected observations"
    );
}
