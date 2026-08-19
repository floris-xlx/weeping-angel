//! Baseline characterization of the SDLC / source-control catalog surface.
//!
//! Encodes what the tree does *today* on the Prompt 01+04 catalog: a single
//! exists-only `control.source.protected-branch` fixture, the ISO pack
//! GitHub-shaped source sliver, and Prompt 03 population semantics. Does not
//! require the Prompt 05 family to be absent after it lands — additive IDs
//! stay compatible. Does not assert harness `[[test]]` registration.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance_ir::{
    AssetId, ControlDomain, ControlId, ControlTestId, Exception, ExceptionId, ExceptionStatus,
    SelectorScope, SubjectKind, SubjectSelector as IrSubjectSelector,
};
use weeping_angel_canonical_catalog::{CATALOG_SCHEMA, CanonicalCatalog, DIGEST_PREFIX};
use weeping_angel_collector::github::GITHUB_EVIDENCE_TYPES;
use weeping_angel_control_test::population::resolve_population;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, PopulationCompleteness, SubjectSelector, TestExpr, build_index, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType, EvidenceValue,
};
use weeping_angel_framework::load_framework_pack;

const PINNED_CONTROL: &str = "control.source.protected-branch";
const PINNED_EVIDENCE: &str = "evidence.source.protected-branch";
const PINNED_TEST: &str = "test.source.protected-branch";
const PINNED_OBSERVATION_TYPE: &str = "source.branch.protection";

const ISO_SOURCE_CONTROLS: &[&str] = &[
    "source.branch-protection",
    "source.required-review",
    "source.code-ownership",
    "source.security-scanning",
    "source.commit-signing",
];

const ISO_SOURCE_TESTS: &[(&str, &str, &str)] = &[
    (
        "test.source.branch-protection",
        "source.branch-protection",
        "source.branch.protection",
    ),
    (
        "test.source.required-review",
        "source.required-review",
        "source.branch.required_reviews",
    ),
    (
        "test.source.code-ownership",
        "source.code-ownership",
        "source.codeowners.present",
    ),
    (
        "test.source.security-scanning",
        "source.security-scanning",
        "source.security.secret_scanning.enabled",
    ),
    (
        "test.source.commit-signing",
        "source.commit-signing",
        "source.commit.signing",
    ),
];

const ISO_SOURCE_MAPPINGS: &[(&str, &str)] = &[
    ("iso27001:a.8.25", "source.branch-protection"),
    ("iso27001:a.8.25", "source.required-review"),
    ("iso27001:a.8.25", "source.code-ownership"),
    ("iso27001:a.8.26", "source.security-scanning"),
    ("iso27001:a.8.8", "source.security-scanning"),
    ("iso27001:a.8.25", "source.commit-signing"),
];

const IDENTITY_CONTROLS: &[&str] = &[
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

const GITHUB_TYPES: &[&str] = &[
    "source.repository.exists",
    "source.repository.visibility",
    "source.default_branch",
    "source.branch.protection",
    "source.branch.required_reviews",
    "source.branch.required_status_checks",
    "source.branch.force_push_protection",
    "source.branch.deletion_protection",
    "source.codeowners.present",
    "source.admin.permissions",
    "source.collaborator.permission",
    "source.security.dependabot.enabled",
    "source.security.secret_scanning.enabled",
    "source.security.code_scanning.configured",
    "source.workflow.permissions",
    "source.workflow.review_requirement",
    "source.ruleset.present",
    "source.repository.archived",
    "source.commit.signing",
];

const PROVIDER_SEGMENTS: &[&str] = &[
    "github",
    "gitlab",
    "bitbucket",
    "azure-devops",
    "okta",
    "entra",
];
const FRAMEWORK_SEGMENTS: &[&str] = &["iso27001", "iso-27001", "soc2", "soc-2", "nis2", "nis-2"];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn catalog_v1() -> PathBuf {
    manifest_dir().join("catalog/canonical/v1")
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1()).expect("canonical catalog v1 loads")
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
    manifest_dir().join("crates").join(name).join("src")
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

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 19, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn seal(
    evidence_type: &str,
    asset: &str,
    facts: &[(&str, &str)],
    hours_ago: i64,
) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new(evidence_type));
    for (k, v) in facts {
        obs = obs.with_fact(*k, *v);
    }
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.sdlc-baseline".into(),
            collected_at: fresh_context().now - chrono::Duration::hours(hours_ago),
            scope: "baseline".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn compiled(test_id: &str, control_id: &str, expr: TestExpr) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new(test_id))
        .control_id(ControlId::new(control_id))
        .kind(ControlTestKind::Automated)
        .expr(expr)
        .build()
}

fn all_subjects(kind: &str, evidence_type: &str, field: &str) -> TestExpr {
    TestExpr::AllSubjects {
        selector: SubjectSelector {
            kind: Some(kind.into()),
            id: None,
        },
        evidence: EvidenceSelector {
            evidence_type: EvidenceType::new(evidence_type),
            subject_selector: SubjectSelector {
                kind: Some(kind.into()),
                id: None,
            },
            field: Some(field.into()),
            freshness: None,
        },
    }
}

fn inventory_repo(set: &mut EvidenceSet, id: &str) {
    set.insert(seal(
        "inventory.subject",
        id,
        &[("id", id), ("kind", "repository")],
        1,
    ));
}

fn complete_repos(set: &mut EvidenceSet) {
    set.insert(seal(
        "inventory.complete",
        "org:scope",
        &[("kind", "repository"), ("authoritative", "true")],
        1,
    ));
}

fn protected(set: &mut EvidenceSet, id: &str, ok: bool, hours_ago: i64) {
    set.insert(seal(
        "evidence.repository.branch-protection",
        id,
        &[("protected", if ok { "protected" } else { "unprotected" })],
        hours_ago,
    ));
}

fn repo_exception(
    id: &str,
    status: ExceptionStatus,
    expires_hours_from_now: Option<i64>,
) -> Exception {
    let mut ex = Exception::new(ExceptionId::new(format!("exc:{id}")), "timeboxed waiver");
    ex.status = status;
    ex.control_id = Some(ControlId::new("control.source.default-branch-protection"));
    if let Some(hours) = expires_hours_from_now {
        ex.expires_at = Some(fresh_context().now + chrono::Duration::hours(hours));
    }
    let mut ids = BTreeSet::new();
    ids.insert(id.to_string());
    ex.subjects.push(IrSubjectSelector {
        kind: SubjectKind::Repository,
        ids,
        tags: BTreeMap::new(),
        scope: SelectorScope::AnyOf,
    });
    ex
}

fn catalog_ids_joined() -> String {
    let catalog = load_catalog();
    catalog
        .controls()
        .keys()
        .chain(catalog.evidence().keys())
        .chain(catalog.tests().keys())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn canonical_catalog_loads_fixture_and_identity_and_validates() {
    let catalog = load_catalog();
    catalog.validate().expect("current catalog validates");
    let digest = catalog.digest().expect("digest");
    assert!(
        digest.to_string().starts_with(DIGEST_PREFIX),
        "digest must use {DIGEST_PREFIX}, got {digest}"
    );
    assert_eq!(CATALOG_SCHEMA, "weeping-angel/canonical-catalog/v1");

    let manifest = fs::read_to_string(catalog_v1().join("manifest.toml")).unwrap();
    for listed in [
        "controls/fixture.example.toml",
        "controls/identity.toml",
        "evidence/fixture.example.toml",
        "evidence/identity.toml",
        "tests/fixture.example.toml",
        "tests/identity.toml",
    ] {
        assert!(
            manifest.contains(listed),
            "manifest.toml must list {listed}"
        );
    }
}

#[test]
fn protected_branch_fixture_remains_exists_only() {
    let catalog = load_catalog();
    let control = catalog.control(PINNED_CONTROL).expect(PINNED_CONTROL);
    assert_eq!(control.tests, vec![PINNED_TEST.to_string()]);
    assert_eq!(control.evidence, vec![PINNED_EVIDENCE.to_string()]);
    assert!(
        control.domains.iter().any(|d| d == "secureDevelopment"),
        "fixture domains include secureDevelopment, got {:?}",
        control.domains
    );
    let _ = ControlDomain::SecureDevelopment;

    let evidence = catalog
        .evidence()
        .get(PINNED_EVIDENCE)
        .expect(PINNED_EVIDENCE);
    assert_eq!(evidence.evidence_type, PINNED_OBSERVATION_TYPE);

    let test = catalog.tests().get(PINNED_TEST).expect(PINNED_TEST);
    assert_eq!(test.control, PINNED_CONTROL);
    assert_eq!(
        test.expression.get("op").and_then(|v| v.as_str()),
        Some("exists"),
        "CAT-015 fixture stays op=exists, not a population predicate"
    );
    assert_eq!(test.required_evidence, vec![PINNED_EVIDENCE.to_string()]);
}

#[test]
fn exists_fixture_is_not_a_population_predicate() {
    let expr = TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
        PINNED_OBSERVATION_TYPE,
    )));
    let test = compiled(PINNED_TEST, PINNED_CONTROL, expr);
    let ctx = fresh_context();

    let empty = evaluate(&test, &EvidenceSet::new(), &ctx);
    assert_eq!(empty.effectiveness, Effectiveness::InsufficientEvidence);

    let mut one = EvidenceSet::new();
    one.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:alpha",
        &[("protected", "true")],
        1,
    ));
    inventory_repo(&mut one, "repo:alpha");
    inventory_repo(&mut one, "repo:unprotected");
    complete_repos(&mut one);
    protected(&mut one, "repo:unprotected", false, 1);
    let passed = evaluate(&test, &one, &ctx);
    assert_eq!(
        passed.effectiveness,
        Effectiveness::Effective,
        "today one {PINNED_OBSERVATION_TYPE} envelope satisfies the shipped fixture even when another repo is unprotected"
    );
}

#[test]
fn missing_required_scan_evidence_is_not_a_technical_failure() {
    let presence = CompiledControlTest::builder()
        .id(ControlTestId::new("test.source.security-scanning"))
        .control_id(ControlId::new("source.security-scanning"))
        .require(EvidenceType::new("source.security.secret_scanning.enabled"))
        .build();
    let empty = evaluate(&presence, &EvidenceSet::new(), &fresh_context());
    assert_eq!(empty.effectiveness, Effectiveness::InsufficientEvidence);
    assert_ne!(empty.effectiveness, Effectiveness::Ineffective);

    let expr = all_subjects(
        "repository",
        "evidence.repository.security-scanning",
        "enabled",
    );
    let mut set = EvidenceSet::new();
    inventory_repo(&mut set, "repo:scanned");
    inventory_repo(&mut set, "repo:unscanned");
    complete_repos(&mut set);
    set.insert(seal(
        "evidence.repository.security-scanning",
        "repo:scanned",
        &[("enabled", "true")],
        1,
    ));
    let result = evaluate(
        &compiled(
            "test.source.security-scanning-coverage",
            "control.source.secret-scanning",
            expr,
        ),
        &set,
        &fresh_context(),
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::InsufficientEvidence,
        "missing scan observation on an in-scope repo is not Ineffective; got {:?}",
        result.effectiveness
    );
}

#[test]
fn stale_required_observation_is_stale_not_fail() {
    let test = compiled(
        PINNED_TEST,
        PINNED_CONTROL,
        TestExpr::Exists(EvidenceSelector::of_type(EvidenceType::new(
            PINNED_OBSERVATION_TYPE,
        ))),
    );
    let mut set = EvidenceSet::new();
    set.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:alpha",
        &[("protected", "true")],
        48,
    ));
    let result = evaluate(&test, &set, &fresh_context());
    assert_eq!(result.effectiveness, Effectiveness::StaleEvidence);
}

#[test]
fn iso_source_sliver_remains_github_shaped_presence_checks() {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    for id in ISO_SOURCE_CONTROLS {
        assert!(
            control_ids.contains(id),
            "ISO pack missing source sliver `{id}`"
        );
    }
    assert!(
        !control_ids.contains("control.source.protected-branch")
            && !control_ids.contains("control.source.default-branch-protection"),
        "ISO pack must not already hold canonical catalog source IDs"
    );

    let tests: BTreeMap<&str, &weeping_angel_assurance_ir::PlannedControlTest> =
        pack.tests.iter().map(|t| (t.id.as_str(), t)).collect();
    for (test_id, control_id, evidence) in ISO_SOURCE_TESTS {
        let test = tests
            .get(test_id)
            .unwrap_or_else(|| panic!("missing {test_id}"));
        assert_eq!(test.control_id.as_str(), *control_id);
        assert_eq!(
            test.required_evidence,
            vec![EvidenceType::new(*evidence)],
            "{test_id} required evidence stays GitHub-shaped"
        );
    }

    let iso_presence = CompiledControlTest::builder()
        .id(ControlTestId::new("test.source.branch-protection"))
        .control_id(ControlId::new("source.branch-protection"))
        .require(EvidenceType::new("source.branch.protection"))
        .build();
    let mut set = EvidenceSet::new();
    set.insert(seal(
        "source.branch.protection",
        "repo:alpha",
        &[("protected", "true")],
        1,
    ));
    assert_eq!(
        evaluate(&iso_presence, &set, &fresh_context()).effectiveness,
        Effectiveness::Effective
    );
}

#[test]
fn iso_source_mappings_are_unchanged() {
    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    for (from, to) in ISO_SOURCE_MAPPINGS {
        assert!(
            mappings.contains(&format!("from = \"{from}\""))
                && mappings.contains(&format!("to = \"{to}\"")),
            "ISO mapping {from} → {to} must remain"
        );
    }
    assert!(
        !mappings.contains("control.source.default-branch-protection")
            && !mappings.contains("control.source.required-review"),
        "ISO mappings must not already retarget the Prompt 05 catalog IDs"
    );
}

#[test]
fn github_collector_advertises_the_same_source_star_types() {
    assert_eq!(GITHUB_EVIDENCE_TYPES, GITHUB_TYPES);
    for prefix in [
        "evidence.repository.",
        "evidence.cicd.",
        "evidence.deployment.",
        "evidence.release.",
        "evidence.supply-chain.",
    ] {
        assert!(
            GITHUB_EVIDENCE_TYPES.iter().all(|t| !t.starts_with(prefix)),
            "GitHub collector must not advertise {prefix}* (Prompt 05 does not expand it)"
        );
    }
}

#[test]
fn catalog_toml_does_not_read_github_evidence_types() {
    let toml = fs::read_to_string(catalog_v1().join("manifest.toml")).unwrap()
        + &fs::read_to_string(catalog_v1().join("controls/fixture.example.toml")).unwrap()
        + &fs::read_to_string(catalog_v1().join("tests/fixture.example.toml")).unwrap()
        + &fs::read_to_string(catalog_v1().join("evidence/fixture.example.toml")).unwrap();
    assert!(
        !toml.contains("GITHUB_EVIDENCE_TYPES"),
        "catalog documents must not couple to collector internals"
    );
    let catalog_crate = fs::read_to_string(
        manifest_dir().join("crates/weeping-angel-canonical-catalog/Cargo.toml"),
    )
    .unwrap();
    assert!(
        !catalog_crate.contains("weeping-angel-collector"),
        "canonical-catalog crate stays collector-free"
    );
}

#[test]
fn population_resolver_has_no_repository_special_case() {
    let src =
        fs::read_to_string(crate_src("weeping-angel-control-test").join("population.rs")).unwrap();
    assert!(
        src.contains("inventory.subject") && src.contains("inventory.complete"),
        "generic inventory.subject / inventory.complete path must remain"
    );
    assert!(
        src.contains("fn resolve_identity_inventory"),
        "identity special-case remains"
    );
    assert!(
        !src.contains("resolve_repository_inventory"),
        "Prompt 03 forbids a repository-specific resolver"
    );
}

#[test]
fn generic_inventory_population_resolves_repositories() {
    let mut set = EvidenceSet::new();
    inventory_repo(&mut set, "repo:alpha");
    inventory_repo(&mut set, "repo:beta");
    complete_repos(&mut set);
    let index = build_index(&set);
    let pop = resolve_population(
        &SubjectSelector {
            kind: Some("repository".into()),
            id: None,
        },
        &set,
        &index,
        Some(&EvidenceType::new("evidence.repository.branch-protection")),
        fresh_context().now,
    );
    assert_eq!(pop.completeness, PopulationCompleteness::Authoritative);
    assert_eq!(
        pop.subject_ids,
        vec!["repo:alpha".to_string(), "repo:beta".to_string()]
    );
}

#[test]
fn all_subjects_distinguishes_healthy_fail_partial_and_stale() {
    let expr = all_subjects(
        "repository",
        "evidence.repository.branch-protection",
        "protected",
    );
    let test = compiled(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        expr,
    );
    let ctx = fresh_context();

    let mut healthy = EvidenceSet::new();
    inventory_repo(&mut healthy, "repo:alpha");
    inventory_repo(&mut healthy, "repo:beta");
    complete_repos(&mut healthy);
    protected(&mut healthy, "repo:alpha", true, 1);
    protected(&mut healthy, "repo:beta", true, 1);
    assert_eq!(
        evaluate(&test, &healthy, &ctx).effectiveness,
        Effectiveness::Effective
    );

    let mut degraded = healthy.clone();
    protected(&mut degraded, "repo:beta", false, 0);
    assert_eq!(
        evaluate(&test, &degraded, &ctx).effectiveness,
        Effectiveness::Ineffective,
        "one unprotected in-scope repo is a technical fail"
    );

    let mut partial = EvidenceSet::new();
    inventory_repo(&mut partial, "repo:alpha");
    protected(&mut partial, "repo:alpha", true, 1);
    assert_eq!(
        evaluate(&test, &partial, &ctx).effectiveness,
        Effectiveness::InsufficientEvidence,
        "inventory.subject without inventory.complete is partial/unknown, not Effective"
    );

    let mut stale = EvidenceSet::new();
    inventory_repo(&mut stale, "repo:alpha");
    complete_repos(&mut stale);
    protected(&mut stale, "repo:alpha", true, 48);
    assert_eq!(
        evaluate(&test, &stale, &ctx).effectiveness,
        Effectiveness::StaleEvidence
    );
}

#[test]
fn approved_ir_exception_excludes_subject_expired_and_revoked_do_not() {
    let expr = all_subjects(
        "repository",
        "evidence.repository.branch-protection",
        "protected",
    );
    let test = compiled(
        "test.source.default-branches-protected",
        "control.source.default-branch-protection",
        expr,
    );
    let ctx = fresh_context();

    let mut set = EvidenceSet::new();
    inventory_repo(&mut set, "repo:ok");
    inventory_repo(&mut set, "repo:waived");
    complete_repos(&mut set);
    protected(&mut set, "repo:ok", true, 1);
    protected(&mut set, "repo:waived", false, 1);

    let mut approved = set.clone();
    approved.insert_exception(repo_exception(
        "repo:waived",
        ExceptionStatus::Approved,
        Some(24),
    ));
    let approved_out = evaluate(&test, &approved, &ctx);
    assert_eq!(
        approved_out.effectiveness,
        Effectiveness::Effective,
        "today an approved bound exception removes the subject; remaining pass → Effective (not ExceptionApproved). got {:?}",
        approved_out.effectiveness
    );
    assert_ne!(approved_out.effectiveness, Effectiveness::Ineffective);

    let mut expired = set.clone();
    expired.insert_exception(repo_exception(
        "repo:waived",
        ExceptionStatus::Approved,
        Some(-1),
    ));
    assert_eq!(
        evaluate(&test, &expired, &ctx).effectiveness,
        Effectiveness::Ineffective,
        "expired exceptions do not pass"
    );

    let mut revoked = set;
    revoked.insert_exception(repo_exception(
        "repo:waived",
        ExceptionStatus::Revoked,
        Some(24),
    ));
    assert_eq!(
        evaluate(&test, &revoked, &ctx).effectiveness,
        Effectiveness::Ineffective,
        "revoked exceptions do not pass"
    );
}

#[test]
fn manual_review_cannot_auto_pass_without_attestation() {
    let test = compiled(
        "test.release.authorization",
        "control.release.authorization",
        TestExpr::ManualReview,
    );
    let mut set = EvidenceSet::new();
    set.insert(seal(
        "evidence.release.authorization",
        "release:1",
        &[("authorized", "true")],
        1,
    ));
    let result = evaluate(&test, &set, &fresh_context());
    assert_eq!(
        result.effectiveness,
        Effectiveness::ManualReviewRequired,
        "ManualReview cannot auto-pass from a technical envelope"
    );
}

#[test]
fn identity_family_and_cat_fixtures_remain() {
    let catalog = load_catalog();
    for id in IDENTITY_CONTROLS {
        assert!(
            catalog.controls().contains_key(*id),
            "IAM control `{id}` must remain"
        );
    }
    assert!(catalog.controls().contains_key(PINNED_CONTROL));
    assert!(catalog.evidence().contains_key(PINNED_EVIDENCE));
    assert!(catalog.tests().contains_key(PINNED_TEST));

    let fixtures = manifest_dir().join("fixtures/assurance/canonical/v1/identity");
    for name in IAM_FIXTURES {
        assert!(
            fixtures.join(name).join("evidence.json").is_file(),
            "IAM fixture `{name}` must remain"
        );
    }
}

#[test]
fn prompt_01_and_04_ssot_documents_are_not_overwritten() {
    let cat = fs::read_to_string(manifest_dir().join("docs/sdd/canonical-assurance-catalog-v1.md"))
        .unwrap();
    assert!(
        cat.starts_with("# SDD: Canonical Assurance Catalog v1 infrastructure"),
        "Prompt 01 SSOT title must remain"
    );
    assert!(
        cat.contains("does **not** own IAM / SDLC") || cat.contains("does not own IAM / SDLC"),
        "Prompt 01 SSOT still disclaims SDLC domain ownership"
    );

    let iam =
        fs::read_to_string(manifest_dir().join("docs/sdd/iam-canonical-assurance-catalog.md"))
            .unwrap();
    assert!(
        iam.contains("IAM Canonical Assurance Catalog"),
        "Prompt 04 SSOT must remain"
    );
    assert!(
        iam.contains("control.identity.mfa"),
        "Prompt 04 SSOT still describes the identity family"
    );
}

#[test]
fn single_canonical_catalog_loader_and_single_evidence_value() {
    let catalog_src = crate_sources_joined("weeping-angel-canonical-catalog");
    assert_eq!(
        catalog_src.matches("struct CanonicalCatalog").count(),
        1,
        "exactly one CanonicalCatalog loader"
    );
    let control_test = crate_sources_joined("weeping-angel-control-test");
    assert!(
        control_test.contains("pub use weeping_angel_evidence::EvidenceValue"),
        "control-test re-exports EvidenceValue; it must not define a second enum"
    );
    assert!(
        !control_test.contains("enum EvidenceValue {"),
        "no second EvidenceValue enum in control-test"
    );
    let _ = EvidenceValue::Bool(true);
}

#[test]
fn current_catalog_ids_have_no_provider_or_framework_segments() {
    let ids = catalog_ids_joined();
    for token in PROVIDER_SEGMENTS.iter().chain(FRAMEWORK_SEGMENTS) {
        for id in ids.lines() {
            let parts: Vec<&str> = id.split('.').collect();
            assert!(
                !parts.iter().any(|p| p.eq_ignore_ascii_case(token)),
                "canonical id `{id}` must not contain reserved segment `{token}`"
            );
        }
    }
}

#[test]
fn subject_kinds_already_cover_repository_branch_and_deployment() {
    assert_eq!(
        SubjectKind::parse_name("repository"),
        Some(SubjectKind::Repository)
    );
    assert_eq!(SubjectKind::parse_name("branch"), Some(SubjectKind::Branch));
    assert_eq!(
        SubjectKind::parse_name("deployment"),
        Some(SubjectKind::Deployment)
    );
    let _ = SubjectKind::Organization;
}
