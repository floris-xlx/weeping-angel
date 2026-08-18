//! SUPERSEDED by `sdd_iam_catalog_target` after the IAM catalog slice landed.
//!
//! Historical characterization of catalog absence on planning SHA
//! `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b`. Tests are ignored so
//! absence-of-catalog is not required CI green.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance_ir::{
    AssetId, ControlId, ControlImplementation, ControlImplementationId, ControlTestId, Exception,
    ExceptionId, ExceptionStatus, Identity, IdentityId, IdentityKind,
};
use weeping_angel_collector::github::GITHUB_EVIDENCE_TYPES;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, EvidenceValue, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};
use weeping_angel_framework::load_framework_pack;

const PLACEHOLDER_RATIONALE: &str = "subject coverage remains partial unless the threshold is met";

const ISO_IAM_CONTROLS: &[&str] = &[
    "access.mfa.privileged",
    "access.least-privilege",
    "access.periodic-review",
    "personnel.access-termination",
];

const CANONICAL_IDENTITY_CONTROLS: &[&str] = &[
    "control.identity.unique-user-identities",
    "control.identity.mfa",
    "control.identity.privileged-mfa",
    "control.identity.strong-authentication-policy",
    "control.identity.privileged-inventory",
    "control.identity.least-privilege",
    "control.identity.privileged-access-minimization",
    "control.identity.access-approval",
    "control.identity.periodic-access-review",
    "control.identity.inactive-account-lifecycle",
    "control.identity.terminated-user-removal",
    "control.identity.joiner-mover-leaver",
    "control.identity.service-account-inventory",
    "control.identity.service-account-ownership",
    "control.identity.service-account-credential-governance",
    "control.identity.break-glass-access",
    "control.identity.shared-account-restriction",
    "control.identity.credential-management",
    "control.identity.privileged-role-change-monitoring",
    "control.identity.external-guest-access",
    "control.identity.stale-privileged-membership",
    "control.identity.access-revocation-timeliness",
    "control.identity.segregation-of-duties",
];

const CANONICAL_IDENTITY_EVIDENCE: &[&str] = &[
    "evidence.identity.inventory",
    "evidence.identity.authentication-state",
    "evidence.identity.mfa-status",
    "evidence.identity.privileged-membership",
    "evidence.identity.role-membership",
    "evidence.identity.last-active",
    "evidence.identity.account-status",
    "evidence.identity.account-owner",
    "evidence.identity.access-review",
    "evidence.identity.lifecycle-event",
    "evidence.identity.service-account",
    "evidence.identity.external-access",
];

const CANONICAL_IDENTITY_TESTS: &[&str] = &[
    "test.identity.mfa-enabled",
    "test.identity.privileged-mfa-enabled",
    "test.identity.no-inactive-privileged-accounts",
    "test.identity.no-terminated-active-accounts",
    "test.identity.all-service-accounts-have-owner",
    "test.identity.access-review-current",
    "test.identity.no-unapproved-guest-access",
];

const IAM_FIXTURES: &[&str] = &[
    "healthy-org",
    "privileged-without-mfa",
    "inactive-admin-active",
    "terminated-employee-active",
    "service-account-without-owner",
    "partial-inventory",
    "stale-access-review",
    "break-glass-approved-exception",
];

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

fn product_rs_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&manifest_dir().join("crates"), &mut files);
    walk_rs_files(&manifest_dir().join("src"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn envelope(evidence_type: &str, asset: &str, field: &str, value: &str) -> EvidenceEnvelope {
    let obs = EvidenceObservation::new(EvidenceType::new(evidence_type)).with_fact(field, value);
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.iam-baseline".into(),
            collected_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
            scope: "baseline".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn iso_mfa_presence_test() -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new("test.access.mfa.privileged"))
        .control_id(ControlId::new("access.mfa.privileged"))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new("source.admin.permissions"))
        .build()
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn dual_suite_baseline_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_iam_catalog_baseline")
            && toml.contains("tests/sdd/iam_catalog.baseline.rs"),
        "IAM baseline suite must be listed in root Cargo.toml"
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn catalog_canonical_v1_is_absent() {
    let root = manifest_dir();
    assert!(
        !root.join("catalog").exists(),
        "current tree has no catalog/ directory"
    );
    assert!(
        !root.join("catalog/canonical/v1").exists(),
        "current tree has no catalog/canonical/v1 tree"
    );
    assert!(
        !root.join("catalog/canonical/v1/manifest.toml").exists(),
        "canonical catalog manifest is not shipped"
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn canonical_catalog_api_is_absent() {
    let rust = product_rs_joined();
    for needle in [
        "struct CanonicalCatalog",
        "CanonicalCatalog::load",
        "weeping-angel/canonical-catalog/v1",
        "weeping_angel_canonical_catalog",
    ] {
        assert!(
            !rust.contains(needle),
            "product Rust currently has no CanonicalCatalog API; found `{needle}`"
        );
    }
    assert!(
        !manifest_dir()
            .join("crates/weeping-angel-canonical-catalog")
            .exists(),
        "weeping-angel-canonical-catalog crate is not a workspace member"
    );
    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !cargo.contains("weeping-angel-canonical-catalog"),
        "root Cargo.toml currently does not declare a canonical-catalog crate"
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn no_canonical_identity_family_in_product_or_packs() {
    let rust = product_rs_joined();
    for id in CANONICAL_IDENTITY_CONTROLS
        .iter()
        .chain(CANONICAL_IDENTITY_EVIDENCE)
        .chain(CANONICAL_IDENTITY_TESTS)
    {
        assert!(
            !rust.contains(id),
            "product Rust currently has no canonical IAM id `{id}`"
        );
    }

    let iso =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        !iso.contains("control.identity.")
            && !iso.contains("evidence.identity.")
            && !iso.contains("test.identity."),
        "ISO pack must not already contain the canonical IAM family"
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn iso_pack_holds_the_iam_sliver() {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    for id in ISO_IAM_CONTROLS {
        assert!(
            control_ids.contains(id),
            "ISO pack missing IAM sliver control `{id}` (have {control_ids:?})"
        );
    }

    let tests: BTreeMap<&str, &weeping_angel_assurance_ir::PlannedControlTest> =
        pack.tests.iter().map(|t| (t.id.as_str(), t)).collect();
    let expected = [
        (
            "test.access.mfa.privileged",
            "access.mfa.privileged",
            "source.admin.permissions",
        ),
        (
            "test.access.least-privilege",
            "access.least-privilege",
            "source.collaborator.permission",
        ),
        (
            "test.access.periodic-review",
            "access.periodic-review",
            "policy.access.reviewed",
        ),
        (
            "test.personnel.access-termination",
            "personnel.access-termination",
            "personnel.access.terminated",
        ),
    ];
    for (test_id, control_id, evidence) in expected {
        let test = tests
            .get(test_id)
            .unwrap_or_else(|| panic!("missing {test_id}"));
        assert_eq!(test.control_id.as_str(), control_id);
        assert_eq!(
            test.required_evidence,
            vec![EvidenceType::new(evidence)],
            "{test_id} required evidence"
        );
    }
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn iso_pack_controls_are_id_title_description_only() {
    let pack = load_framework_pack("iso-27001", "2022").unwrap();
    for control in pack
        .controls
        .iter()
        .filter(|c| ISO_IAM_CONTROLS.contains(&c.id().as_str()))
    {
        assert!(
            control.domains().is_empty(),
            "{} currently has no domains on disk/IR",
            control.id()
        );
        assert!(
            control.objective().is_empty(),
            "{} currently has no objective field from the pack",
            control.id()
        );
    }
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn iso_hybrid_kind_in_toml_loads_as_automated() {
    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        metadata.contains("id = \"test.access.mfa.privileged\"")
            && metadata.contains("kind = \"hybrid\""),
        "pack source still marks privileged MFA as hybrid"
    );
    let pack = load_framework_pack("iso-27001", "2022").unwrap();
    let mfa = pack
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.access.mfa.privileged")
        .expect("mfa test");
    assert_eq!(
        mfa.kind,
        weeping_angel_assurance_ir::PlannedTestKind::Automated,
        "pack loader currently maps any non-manual kind to Automated"
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn iso_mfa_test_is_existence_of_github_admin_permissions() {
    let test = iso_mfa_presence_test();
    let ctx = fresh_context();

    let empty = evaluate(&test, &EvidenceSet::new(), &ctx);
    assert_eq!(empty.effectiveness, Effectiveness::InsufficientEvidence);

    let mut some_admin = EvidenceSet::new();
    some_admin.insert(envelope(
        "source.admin.permissions",
        "user:admin-1",
        "permission",
        "admin",
    ));
    let passed = evaluate(&test, &some_admin, &ctx);
    assert_eq!(
        passed.effectiveness,
        Effectiveness::Effective,
        "today a single source.admin.permissions envelope satisfies privileged MFA"
    );

    let mut identity_shaped = EvidenceSet::new();
    identity_shaped.insert(envelope(
        "evidence.identity.mfa-status",
        "user:admin-1",
        "mfa_enabled",
        "false",
    ));
    identity_shaped.insert(envelope(
        "source.admin.permissions",
        "user:admin-1",
        "permission",
        "admin",
    ));
    let still_effective = evaluate(&test, &identity_shaped, &ctx);
    assert_eq!(
        still_effective.effectiveness,
        Effectiveness::Effective,
        "IAM-shaped MFA-false facts do not change the ISO existence test"
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn coverage_at_least_is_placeholder_and_cannot_assess_privileged_mfa_population() {
    let expr = TestExpr::CoverageAtLeast {
        selector: SubjectSelector {
            kind: Some("privilegedIdentity".into()),
            id: None,
        },
        evidence: EvidenceSelector {
            evidence_type: EvidenceType::new("evidence.identity.mfa-status"),
            subject_selector: SubjectSelector {
                kind: Some("privilegedIdentity".into()),
                id: None,
            },
            field: Some("mfa_enabled".into()),
            freshness: None,
        },
        percentage: "100".into(),
    };
    let compiled = CompiledControlTest::builder()
        .id(ControlTestId::new("test.identity.privileged-mfa-enabled"))
        .control_id(ControlId::new("control.identity.privileged-mfa"))
        .kind(ControlTestKind::Automated)
        .expr(expr)
        .build();

    let mut set = EvidenceSet::new();
    set.insert(envelope(
        "evidence.identity.mfa-status",
        "user:priv-1",
        "mfa_enabled",
        "false",
    ));
    let result = evaluate(&compiled, &set, &fresh_context());
    assert_eq!(result.effectiveness, Effectiveness::PartiallyEffective);
    assert_eq!(result.rationale, PLACEHOLDER_RATIONALE);
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn test_expr_has_no_all_subjects_population_index() {
    let src = fs::read_to_string(crate_src("weeping-angel-control-test").join("expr.rs")).unwrap();
    for arm in [
        "AllSubjects",
        "AnySubject",
        "NoneSubjects",
        "CountWhere",
        "MissingSubjects",
    ] {
        assert!(
            !src.contains(arm),
            "baseline TestExpr must not contain {arm}"
        );
    }
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn exception_approved_exists_but_evaluate_never_emits_it() {
    let src = crate_sources_joined("weeping-angel-control-test");
    assert!(
        src.contains("ExceptionApproved"),
        "Effectiveness::ExceptionApproved is declared"
    );
    assert!(
        !src.contains("effectiveness: Effectiveness::ExceptionApproved"),
        "no eval arm currently assigns ExceptionApproved"
    );

    let cases = [
        TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
            "source.admin.permissions",
        ))),
        TestExpr::Missing(EvidenceSelector::of_type(EvidenceType::new(
            "source.admin.permissions",
        ))),
        TestExpr::ManualReview,
        TestExpr::CoverageAtLeast {
            selector: SubjectSelector::default(),
            evidence: EvidenceSelector::of_type(EvidenceType::new("source.admin.permissions")),
            percentage: "100".into(),
        },
    ];
    for expr in cases {
        let compiled = CompiledControlTest::builder()
            .id(ControlTestId::new("test.identity.break-glass-governed"))
            .control_id(ControlId::new("control.identity.break-glass-access"))
            .kind(ControlTestKind::Automated)
            .expr(expr)
            .build();
        let result = evaluate(&compiled, &EvidenceSet::new(), &fresh_context());
        assert_ne!(
            result.effectiveness,
            Effectiveness::ExceptionApproved,
            "evaluate must not emit ExceptionApproved today"
        );
    }
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn exception_ir_attaches_to_implementation_not_catalog_tests() {
    let ex = Exception::new(ExceptionId::new("exc:break-glass"), "timeboxed waiver");
    assert_eq!(ex.status, ExceptionStatus::Proposed);
    let json = serde_json::to_value(&ex).unwrap();
    assert!(json.get("subjects").is_none());
    assert!(json.get("appliesTo").is_none());

    let impln = ControlImplementation::new(
        ControlImplementationId::new("impl.access.mfa.org"),
        ControlId::new("access.mfa.privileged"),
    )
    .with_exception(ExceptionId::new("exc:break-glass"));
    assert_eq!(
        impln.exception_ids(),
        &[ExceptionId::new("exc:break-glass")]
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn identity_and_subject_kind_are_thin() {
    let identity = Identity::new(IdentityId::new("id:user-1"), IdentityKind::User);
    assert!(identity.display_name.is_none());
    let _ = IdentityKind::Service;
    let _ = IdentityKind::Team;
    let _ = IdentityKind::Role;
    let _ = IdentityKind::Other;

    let src =
        fs::read_to_string(crate_src("weeping-angel-assurance-ir").join("subject.rs")).unwrap();
    assert!(src.contains("PrivilegedIdentity"));
    assert!(
        !src.contains("ServiceAccount"),
        "SubjectKind currently has no ServiceAccount variant"
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn evidence_facts_are_string_map_and_parse_fact_coerces() {
    let obs = EvidenceObservation::new(EvidenceType::new("source.admin.permissions"))
        .with_fact("mfa_enabled", "true");
    assert_eq!(obs.fact("mfa_enabled"), Some("true"));
    assert!(matches!(
        obs.fact_value("mfa_enabled"),
        Some(EvidenceValue::String(s)) if s == "true"
    ));
    assert_eq!(
        obs.facts().get("mfa_enabled"),
        Some(&EvidenceValue::String("true".into()))
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn github_advertises_admin_permissions_but_collaborators_is_a_stub() {
    assert!(GITHUB_EVIDENCE_TYPES.contains(&"source.admin.permissions"));
    assert!(GITHUB_EVIDENCE_TYPES.contains(&"source.collaborator.permission"));
    assert!(
        !GITHUB_EVIDENCE_TYPES
            .iter()
            .any(|t| t.starts_with("evidence.identity.")),
        "GitHub collector does not advertise canonical identity evidence"
    );

    let collab =
        fs::read_to_string(crate_src("weeping-angel-collector").join("github/collaborators.rs"))
            .unwrap();
    assert_eq!(
        collab.trim(),
        "pub const MODULE: &str = \"collaborators\";",
        "collaborators.rs is a module stub today"
    );

    let github = crate_sources_joined("weeping-angel-collector");
    for needle in [
        "evidence.identity.inventory",
        "mfa_enabled",
        "last_active_at",
        "account_status",
        "is_service_account",
    ] {
        assert!(
            !github.contains(needle),
            "GitHub collector currently emits no IAM population facts; found `{needle}`"
        );
    }
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn no_identity_provider_collectors_or_iam_fixtures() {
    let collector_src = crate_src("weeping-angel-collector");
    for name in ["entra", "okta", "google_workspace", "workspace", "cognito"] {
        assert!(
            !collector_src.join(name).exists(),
            "current tree has no {name} collector module"
        );
    }

    let fixtures = manifest_dir().join("fixtures/assurance");
    assert!(
        !fixtures.join("canonical").exists(),
        "canonical IAM fixtures tree is absent"
    );
    for name in IAM_FIXTURES {
        assert!(
            !fixtures.join("canonical/v1/identity").join(name).exists(),
            "IAM fixture `{name}` is not shipped"
        );
    }
    let iso_only: BTreeSet<String> = fs::read_dir(&fixtures)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        iso_only,
        BTreeSet::from(["iso27001".into()]),
        "fixtures/assurance currently holds only the ISO repo pair"
    );
}

#[ignore = "superseded by sdd_iam_catalog_target"]
#[test]
fn evaluate_compiled_does_not_attach_test_expr() {
    let src = crate_sources_joined("weeping-angel-assurance");
    let start = src.find("fn evaluate_compiled").expect("evaluate_compiled");
    let body = &src[start..start + 1400.min(src.len() - start)];
    assert!(
        !body.contains(".expr("),
        "facade evaluate_compiled currently does not attach TestExpr"
    );
}
