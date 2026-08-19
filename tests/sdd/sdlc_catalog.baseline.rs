//! Baseline characterization of the SDLC / source-control catalog surface.
//!
//! Encodes what the tree does *today* (Prompt 01 fixture + Prompt 04 IAM):
//! no SDLC population family, exists-only `control.source.protected-branch`,
//! ISO pack GitHub-shaped source sliver, collector still `source.*`, and
//! Prompt 03 generic inventory (no `resolve_repository_inventory`).
//!
//! Do not assert absence of every `control.source.*` string (collides with
//! the fixture). Do not implement Prompt 05 product content here.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance_ir::{
    AssetId, ControlDomain, ControlId, ControlTestId, Exception, ExceptionId, ExceptionStatus,
    PlannedTestKind, SelectorScope, SubjectKind,
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

const POPULATION_DEFAULT_BRANCH: &str = "control.source.default-branch-protection";

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

const SDLC_FIXTURES: &[&str] = &[
    "healthy-org",
    "degraded-org",
    "partial-coverage",
    "unprotected-default-branch",
    "missing-scan-evidence",
    "stale-dependency-scan",
    "approved-exception",
];

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

const GITHUB_SOURCE_TYPES: &[&str] = &[
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

const SDLC_POPULATION_CONTROLS: &[&str] = &[
    "control.source.repository-inventory",
    "control.source.visibility-governance",
    "control.source.default-branch-protection",
    "control.source.force-push-restricted",
    "control.source.branch-deletion-restricted",
    "control.source.required-review",
    "control.source.minimum-reviewer-count",
    "control.source.review-ownership",
    "control.source.required-status-checks",
    "control.source.admin-bypass-governance",
    "control.source.signed-commits",
    "control.source.secret-scanning",
    "control.source.code-scanning",
    "control.source.dependency-scanning",
    "control.source.dependency-update-monitoring",
    "control.supply-chain.dependency-integrity",
    "control.cicd.workflow-permissions",
    "control.release.protected-environment",
    "control.release.authorization",
    "control.release.authority-separation",
    "control.supply-chain.build-provenance",
    "control.supply-chain.artifact-integrity",
    "control.source.change-traceability",
    "control.source.security-review",
    "control.source.secure-development-policy",
    "control.supply-chain.unsupported-components",
];

const SDLC_EVIDENCE: &[&str] = &[
    "evidence.repository.inventory",
    "evidence.repository.visibility",
    "evidence.repository.default-branch",
    "evidence.repository.branch-protection",
    "evidence.repository.review-policy",
    "evidence.repository.review-ownership",
    "evidence.repository.security-scanning",
    "evidence.repository.dependency-scanning",
    "evidence.repository.commit-signing",
    "evidence.repository.change-trace",
    "evidence.repository.security-review",
    "evidence.repository.secure-development-policy",
    "evidence.cicd.workflow-permissions",
    "evidence.cicd.status-checks",
    "evidence.deployment.environment-protection",
    "evidence.release.authorization",
    "evidence.supply-chain.build-provenance",
    "evidence.supply-chain.artifact-integrity",
    "evidence.supply-chain.lockfile-state",
    "evidence.supply-chain.component-support",
];

const SDLC_TESTS: &[&str] = &[
    "test.source.repository-inventory-complete",
    "test.source.visibility-governed",
    "test.source.default-branches-protected",
    "test.source.force-push-restricted",
    "test.source.branch-deletion-restricted",
    "test.source.reviews-required",
    "test.source.minimum-reviewer-count",
    "test.source.review-ownership-present",
    "test.source.required-status-checks",
    "test.source.admin-bypass-governed",
    "test.source.signed-commits-required",
    "test.source.secret-scanning-enabled",
    "test.source.code-scanning-enabled",
    "test.source.dependency-scanning-current",
    "test.source.dependency-updates-monitored",
    "test.supply-chain.lockfile-integrity",
    "test.cicd.workflow-permissions-minimized",
    "test.release.environments-protected",
    "test.release.authorization-recorded",
    "test.release.authority-separated",
    "test.supply-chain.provenance-present",
    "test.supply-chain.artifacts-have-integrity",
    "test.source.changes-traceable",
    "test.source.security-review-recorded",
    "test.source.secure-development-policy-attested",
    "test.supply-chain.unsupported-components-handled",
];

const RESERVED_PROVIDER_SEGMENTS: &[&str] =
    &["github", "gitlab", "bitbucket", "azure-devops", "gitea"];

const RESERVED_FRAMEWORK_SEGMENTS: &[&str] = &[
    "iso27001",
    "iso-27001",
    "soc2",
    "soc-2",
    "nis2",
    "dora",
    "gdpr",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn catalog_v1() -> PathBuf {
    manifest_dir().join("catalog/canonical/v1")
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1()).expect("canonical catalog v1 loads")
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    walk_files(dir, out);
    out.retain(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"));
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
        now: Utc.with_ymd_and_hms(2026, 8, 19, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn collected(hours_ago: i64) -> chrono::DateTime<Utc> {
    fresh_context().now - chrono::Duration::hours(hours_ago)
}

fn seal(evidence_type: &str, asset: &str, facts: &[(&str, &str)]) -> EvidenceEnvelope {
    seal_at(evidence_type, asset, facts, collected(1))
}

fn seal_at(
    evidence_type: &str,
    asset: &str,
    facts: &[(&str, &str)],
    collected_at: chrono::DateTime<Utc>,
) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new(evidence_type));
    for (k, v) in facts {
        obs = obs.with_fact(*k, *v);
    }
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.sdlc-baseline".into(),
            collected_at,
            scope: "baseline".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn compiled(id: &str, control: &str, expr: TestExpr) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new(id))
        .control_id(ControlId::new(control))
        .kind(ControlTestKind::Automated)
        .expr(expr)
        .build()
}

fn exists_test(evidence_type: &str) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new(PINNED_TEST))
        .control_id(ControlId::new(PINNED_CONTROL))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new(evidence_type))
        .expr(TestExpr::Exists(EvidenceSelector::of_type(
            EvidenceType::new(evidence_type),
        )))
        .build()
}

fn repo_all_subjects(evidence_type: &str, field: &str) -> TestExpr {
    TestExpr::AllSubjects {
        selector: SubjectSelector {
            kind: Some("repository".into()),
            id: None,
        },
        evidence: EvidenceSelector {
            evidence_type: EvidenceType::new(evidence_type),
            subject_selector: SubjectSelector {
                kind: Some("repository".into()),
                id: None,
            },
            field: Some(field.into()),
            freshness: None,
        },
    }
}

fn authoritative_repos(set: &mut EvidenceSet, ids: &[&str]) {
    for id in ids {
        set.insert(seal(
            "inventory.subject",
            id,
            &[("kind", "repository"), ("id", id)],
        ));
    }
    set.insert(seal(
        "inventory.complete",
        "org:baseline",
        &[("kind", "repository"), ("authoritative", "true")],
    ));
}

fn id_has_reserved_segment(id: &str, tokens: &[&str]) -> Option<String> {
    let parts: Vec<&str> = id.split('.').collect();
    for token in tokens {
        if parts.iter().any(|p| p.eq_ignore_ascii_case(token)) {
            return Some((*token).into());
        }
    }
    None
}

fn bound_exception(
    status: ExceptionStatus,
    expires_hours: Option<i64>,
    subject: &str,
) -> Exception {
    let mut ex = Exception::new(
        ExceptionId::new(format!("exc:{subject}")),
        "timeboxed waiver",
    );
    ex.status = status;
    ex.control_id = Some(ControlId::new(PINNED_CONTROL));
    ex.expires_at = expires_hours.map(|h| fresh_context().now + chrono::Duration::hours(h));
    let mut ids = BTreeSet::new();
    ids.insert(subject.into());
    ex.subjects
        .push(weeping_angel_assurance_ir::SubjectSelector {
            kind: SubjectKind::Repository,
            ids,
            tags: Default::default(),
            scope: SelectorScope::AnyOf,
        });
    ex
}

#[test]
fn dual_suite_baseline_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_sdlc_catalog_baseline")
            && toml.contains("tests/sdd/sdlc_catalog.baseline.rs")
            && toml.contains("sdd_sdlc_catalog_target")
            && toml.contains("tests/sdd/sdlc_catalog.target.rs"),
        "SDLC dual-suite must be listed in root Cargo.toml (tests/sdd/*.rs is not auto-discovered)"
    );
}

#[test]
fn catalog_has_no_sdlc_population_family() {
    let root = catalog_v1();
    for rel in [
        "controls/sdlc.toml",
        "evidence/sdlc.toml",
        "tests/sdlc.toml",
        "controls/source.toml",
        "evidence/repository.toml",
        "tests/source.toml",
    ] {
        assert!(
            !root.join(rel).exists(),
            "current tree has no Prompt 05 catalog file `{rel}`"
        );
    }

    let manifest = fs::read_to_string(root.join("manifest.toml")).unwrap();
    assert!(
        !manifest.contains("sdlc.toml") && !manifest.contains("repository.toml"),
        "manifest.toml currently lists only fixture.example + identity, not SDLC files"
    );

    let catalog = load_catalog();
    for id in SDLC_POPULATION_CONTROLS {
        assert!(
            catalog.control(id).is_err(),
            "canonical catalog currently has no SDLC population control `{id}`"
        );
    }
    for id in SDLC_EVIDENCE {
        assert!(
            !catalog.evidence().contains_key(*id),
            "canonical catalog currently has no SDLC evidence `{id}`"
        );
    }
    for id in SDLC_TESTS {
        assert!(
            !catalog.tests().contains_key(*id),
            "canonical catalog currently has no SDLC test `{id}`"
        );
    }

    let source_controls: Vec<&str> = catalog
        .controls()
        .keys()
        .filter(|id| id.starts_with("control.source."))
        .map(String::as_str)
        .collect();
    assert_eq!(
        source_controls,
        vec![PINNED_CONTROL],
        "the only source-shaped canonical control today is the exists-only fixture"
    );
}

#[test]
fn sdlc_population_fixtures_are_absent() {
    let fixtures = manifest_dir().join("fixtures/assurance/canonical/v1/sdlc");
    assert!(
        !fixtures.exists(),
        "fixtures/assurance/canonical/v1/sdlc is not shipped"
    );
    for name in SDLC_FIXTURES {
        assert!(
            !fixtures.join(name).exists(),
            "SDLC fixture `{name}` is not shipped"
        );
    }
}

#[test]
fn catalog_loader_validate_and_digest_remain_the_single_ssot() {
    let catalog = load_catalog();
    catalog
        .validate()
        .expect("CanonicalCatalog::validate accepts the shipped tree");
    assert_eq!(CATALOG_SCHEMA, "weeping-angel/canonical-catalog/v1");
    let digest = catalog.digest().expect("digest");
    assert!(
        digest.to_string().starts_with(DIGEST_PREFIX),
        "digest must use {DIGEST_PREFIX}, got {digest}"
    );

    let again = load_catalog();
    assert_eq!(
        digest.to_string(),
        again.digest().expect("digest").to_string(),
        "CanonicalCatalog::digest is deterministic for the same on-disk tree"
    );

    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    assert_eq!(
        rust.matches("struct CanonicalCatalog").count(),
        1,
        "there is one CanonicalCatalog type; this slice must not invent a second loader"
    );

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
            "manifest.toml must keep listing `{listed}`"
        );
    }
}

#[test]
fn fixture_protected_branch_is_exists_only_and_survives() {
    let catalog = load_catalog();
    let control = catalog
        .control(PINNED_CONTROL)
        .expect("CAT-015 fixture control remains");
    assert_eq!(control.title, "Protected default branch");
    assert!(
        control
            .domains
            .iter()
            .any(|d| d == "secureDevelopment" || d.eq_ignore_ascii_case("securedevelopment")),
        "fixture control keeps SecureDevelopment domain, got {:?}",
        control.domains
    );
    assert_eq!(control.evidence, vec![PINNED_EVIDENCE.to_string()]);
    assert_eq!(control.tests, vec![PINNED_TEST.to_string()]);
    let _ = ControlDomain::SecureDevelopment;

    let evidence = catalog
        .evidence()
        .get(PINNED_EVIDENCE)
        .expect("fixture evidence remains");
    assert_eq!(
        evidence.evidence_type, PINNED_OBSERVATION_TYPE,
        "fixture still declares the GitHub-shaped envelope type, not a population contract"
    );

    let test = catalog
        .tests()
        .get(PINNED_TEST)
        .expect("fixture test remains");
    assert_eq!(test.control, PINNED_CONTROL);
    assert_eq!(test.required_evidence, vec![PINNED_EVIDENCE.to_string()]);
    let op = test
        .expression
        .get("op")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        op, "exists",
        "test.source.protected-branch stays exists-only (population lives on a different id)"
    );

    let toml = fs::read_to_string(catalog_v1().join("tests/fixture.example.toml")).unwrap();
    assert!(
        toml.contains("op = \"exists\"") && toml.contains(PINNED_EVIDENCE),
        "fixture test TOML still encodes Exists(evidence.source.protected-branch)"
    );
}

#[test]
fn identity_family_and_iam_fixtures_remain() {
    let catalog = load_catalog();
    for id in CANONICAL_IDENTITY_CONTROLS {
        catalog
            .control(id)
            .unwrap_or_else(|_| panic!("IAM control `{id}` must remain"));
    }
    assert_eq!(
        CANONICAL_IDENTITY_CONTROLS.len(),
        23,
        "Prompt 04 shipped 23 independently assessable identity controls"
    );
    assert!(catalog.controls().contains_key(PINNED_CONTROL));
    assert!(catalog.evidence().contains_key(PINNED_EVIDENCE));
    assert!(catalog.tests().contains_key(PINNED_TEST));

    let fixtures = manifest_dir().join("fixtures/assurance/canonical/v1/identity");
    for name in IAM_FIXTURES {
        assert!(
            fixtures.join(name).join("evidence.json").is_file(),
            "IAM fixture `{name}` remains on disk"
        );
    }

    let iso =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        !iso.contains("control.identity."),
        "ISO pack still does not host the canonical IAM family"
    );
}

#[test]
fn prompt_01_and_04_ssot_docs_are_not_overwritten() {
    let cat = fs::read_to_string(manifest_dir().join("docs/sdd/canonical-assurance-catalog-v1.md"))
        .unwrap();
    assert!(
        cat.starts_with("# SDD: Canonical Assurance Catalog v1 infrastructure"),
        "Prompt 01 SSOT title must remain"
    );
    assert!(
        cat.contains("This document is the durable SSOT for **catalog infrastructure only**"),
        "Prompt 01 SSOT mission sentence must remain"
    );
    assert!(
        cat.contains("does **not** own IAM / SDLC") || cat.contains("does not own IAM / SDLC"),
        "Prompt 01 SSOT still disclaims SDLC domain ownership"
    );

    let iam =
        fs::read_to_string(manifest_dir().join("docs/sdd/iam-canonical-assurance-catalog.md"))
            .unwrap();
    assert!(
        iam.starts_with("# SDD: IAM Canonical Assurance Catalog (v1 slice)"),
        "Prompt 04 SSOT title must remain"
    );
    assert!(
        iam.contains("This document is the durable SSOT for the **IAM catalog slice**"),
        "Prompt 04 SSOT mission sentence must remain"
    );
    assert!(
        iam.contains("control.identity.mfa"),
        "Prompt 04 SSOT still describes the identity family"
    );
}

#[test]
fn iso_source_sliver_ids_tests_and_mappings_are_unchanged() {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    for id in ISO_SOURCE_CONTROLS {
        assert!(
            control_ids.contains(id),
            "ISO source sliver `{id}` must remain (have {control_ids:?})"
        );
    }
    assert!(
        !control_ids.contains(PINNED_CONTROL),
        "ISO pack must not host the catalog fixture id `{PINNED_CONTROL}`"
    );
    assert!(
        !control_ids.contains(POPULATION_DEFAULT_BRANCH),
        "ISO pack must not host the Prompt 05 population id `{POPULATION_DEFAULT_BRANCH}`"
    );
    assert!(
        !control_ids
            .iter()
            .any(|id| id.starts_with("control.source.")
                || id.starts_with("control.cicd.")
                || id.starts_with("control.release.")
                || id.starts_with("control.supply-chain.")),
        "ISO pack must not grow control.source|cicd|release|supply-chain.* rows"
    );

    let tests: BTreeMap<&str, &weeping_angel_assurance_ir::PlannedControlTest> =
        pack.tests.iter().map(|t| (t.id.as_str(), t)).collect();
    for (test_id, control_id, evidence) in ISO_SOURCE_TESTS {
        let test = tests
            .get(test_id)
            .unwrap_or_else(|| panic!("ISO pack missing {test_id}"));
        assert_eq!(test.control_id.as_str(), *control_id);
        assert_eq!(
            test.required_evidence,
            vec![EvidenceType::new(*evidence)],
            "{test_id} required evidence stays GitHub-shaped `{evidence}`"
        );
    }

    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        !metadata.contains("[test.expression]")
            && !metadata.contains("op = \"all-subjects\"")
            && !metadata.contains("op = \"coverage-at-least\""),
        "ISO source tests remain presence/hybrid stubs, not population expressions"
    );

    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    for (from, to) in ISO_SOURCE_MAPPINGS {
        assert!(
            mappings.contains(&format!("from = \"{from}\""))
                && mappings.contains(&format!("to = \"{to}\"")),
            "ISO mapping {from} → {to} must stay"
        );
    }
    assert!(
        !mappings.contains("to = \"control.source.")
            && !mappings.contains("to = \"control.cicd.")
            && !mappings.contains("to = \"control.release.")
            && !mappings.contains("to = \"control.supply-chain."),
        "ISO mappings stay on pack-local source.* ids until Prompt 12"
    );
}

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
        .expect("iso mfa test");
    assert_eq!(
        mfa.kind,
        PlannedTestKind::Automated,
        "pack loader currently maps any non-manual kind to Automated"
    );
    let branch = pack
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.source.branch-protection")
        .expect("iso source test");
    assert_eq!(branch.kind, PlannedTestKind::Automated);
    let _ = PlannedTestKind::Hybrid;
    let _ = PlannedTestKind::Manual;
}

#[test]
fn iso_and_fixture_existence_pass_on_one_envelope() {
    let ctx = fresh_context();
    let iso = exists_test(PINNED_OBSERVATION_TYPE);

    let empty = evaluate(&iso, &EvidenceSet::new(), &ctx);
    assert_eq!(
        empty.effectiveness,
        Effectiveness::InsufficientEvidence,
        "missing required envelope is InsufficientEvidence, not Ineffective"
    );
    assert_ne!(empty.effectiveness, Effectiveness::Ineffective);

    let mut one = EvidenceSet::new();
    one.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:only",
        &[("protected", "true")],
    ));
    authoritative_repos(&mut one, &["repo:only", "repo:unprotected"]);
    one.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:unprotected",
        &[("protected", "false")],
    ));
    let passed = evaluate(&iso, &one, &ctx);
    assert_eq!(
        passed.effectiveness,
        Effectiveness::Effective,
        "today one {PINNED_OBSERVATION_TYPE} envelope satisfies Exists even when another repo is unprotected"
    );

    let mut unprotected = EvidenceSet::new();
    unprotected.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:open",
        &[("protected", "false")],
    ));
    let still = evaluate(&iso, &unprotected, &ctx);
    assert_eq!(
        still.effectiveness,
        Effectiveness::Effective,
        "Exists ignores the protected fact; one envelope still passes"
    );
}

#[test]
fn github_collector_still_advertises_source_star_types() {
    assert_eq!(
        GITHUB_EVIDENCE_TYPES, GITHUB_SOURCE_TYPES,
        "GitHub collector advertisement is frozen for this slice"
    );
    assert!(
        GITHUB_EVIDENCE_TYPES
            .iter()
            .all(|t| t.starts_with("source.")),
        "collector still emits GitHub-shaped source.* names"
    );
    for prefix in [
        "evidence.repository.",
        "evidence.cicd.",
        "evidence.deployment.",
        "evidence.release.",
        "evidence.supply-chain.",
    ] {
        assert!(
            !GITHUB_EVIDENCE_TYPES.iter().any(|t| t.starts_with(prefix)),
            "collector must not be expanded to `{prefix}` in this slice"
        );
    }
}

#[test]
fn catalog_ids_and_validator_stay_provider_and_framework_neutral() {
    let catalog = load_catalog();
    let mut ids = Vec::new();
    ids.extend(catalog.controls().keys().cloned());
    ids.extend(catalog.evidence().keys().cloned());
    ids.extend(catalog.tests().keys().cloned());
    for id in &ids {
        if let Some(token) = id_has_reserved_segment(id, RESERVED_PROVIDER_SEGMENTS) {
            panic!("catalog id `{id}` contains provider segment `{token}`");
        }
        if let Some(token) = id_has_reserved_segment(id, RESERVED_FRAMEWORK_SEGMENTS) {
            panic!("catalog id `{id}` contains framework segment `{token}`");
        }
        assert!(
            !id.starts_with("evidence.github."),
            "canonical evidence ids are not evidence.github.*"
        );
    }

    let rust = crate_sources_joined("weeping-angel-canonical-catalog");
    for token in ["github", "gitlab", "bitbucket", "iso27001", "soc2", "nis2"] {
        assert!(
            rust.contains(token),
            "Prompt 01 validator still reserves `{token}`"
        );
    }
}

#[test]
fn no_repository_inventory_resolver_or_second_value_enum() {
    let product = product_rs_joined();
    assert!(
        !product.contains("resolve_repository_inventory"),
        "Prompt 03 has no repository-inventory special case; this slice must not add one"
    );
    assert!(
        !product.contains("struct SdlcPopulation"),
        "do not fork a second population type"
    );

    let expr = fs::read_to_string(crate_src("weeping-angel-control-test").join("expr.rs")).unwrap();
    assert!(
        expr.contains("pub use weeping_angel_evidence::EvidenceValue"),
        "control-test re-exports the single EvidenceValue"
    );
    assert!(
        !expr.contains("pub enum EvidenceValue"),
        "control-test must not define a second EvidenceValue enum"
    );

    let value = fs::read_to_string(crate_src("weeping-angel-evidence").join("value.rs")).unwrap();
    assert!(
        value.contains("pub enum EvidenceValue"),
        "the one EvidenceValue lives in weeping-angel-evidence"
    );
    let _ = EvidenceValue::Bool(true);

    let pop =
        fs::read_to_string(crate_src("weeping-angel-control-test").join("population.rs")).unwrap();
    assert!(
        pop.contains("fn resolve_identity_inventory"),
        "identity inventory special-case remains"
    );
    assert!(
        pop.contains("inventory.subject") && pop.contains("inventory.complete"),
        "generic inventory.subject / inventory.complete path must remain"
    );
    assert!(pop.contains("pub fn resolve_population"));
    assert!(pop.contains("pub fn evaluate_coverage"));
}

#[test]
fn generic_inventory_population_resolves_repositories() {
    let mut set = EvidenceSet::new();
    authoritative_repos(&mut set, &["repo:alpha", "repo:beta"]);
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
fn population_uses_inventory_subject_and_complete() {
    let ctx = fresh_context();
    let expr = repo_all_subjects(PINNED_OBSERVATION_TYPE, "protected");
    let test = compiled(
        "test.source.branch-protection.coverage",
        "source.branch-protection",
        expr,
    );

    let mut inferred = EvidenceSet::new();
    inferred.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:a",
        &[("protected", "true")],
    ));
    let unknown = evaluate(&test, &inferred, &ctx);
    assert_eq!(
        unknown.effectiveness,
        Effectiveness::Inconclusive,
        "observations without inventory.complete stay Unknown → Inconclusive on all-subjects"
    );
    assert_ne!(unknown.effectiveness, Effectiveness::Effective);

    let mut partial = EvidenceSet::new();
    partial.insert(seal(
        "inventory.subject",
        "repo:a",
        &[("kind", "repository"), ("id", "repo:a")],
    ));
    partial.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:a",
        &[("protected", "true")],
    ));
    let partial_r = evaluate(&test, &partial, &ctx);
    assert_eq!(
        partial_r.effectiveness,
        Effectiveness::InsufficientEvidence,
        "inventory.subject without inventory.complete is Partial → InsufficientEvidence"
    );
    assert_ne!(partial_r.effectiveness, Effectiveness::Effective);

    let mut healthy = EvidenceSet::new();
    authoritative_repos(&mut healthy, &["repo:a", "repo:b"]);
    healthy.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:a",
        &[("protected", "true")],
    ));
    healthy.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:b",
        &[("protected", "true")],
    ));
    let ok = evaluate(&test, &healthy, &ctx);
    assert_eq!(
        ok.effectiveness,
        Effectiveness::Effective,
        "authoritative inventory.subject + inventory.complete can yield Effective"
    );
}

#[test]
fn evaluator_distinguishes_missing_stale_fail_manual_and_exception() {
    let ctx = fresh_context();
    let expr = repo_all_subjects(PINNED_OBSERVATION_TYPE, "protected");
    let test = compiled(
        "test.source.default-branches-protected",
        PINNED_CONTROL,
        expr,
    );

    let mut missing = EvidenceSet::new();
    authoritative_repos(&mut missing, &["repo:a", "repo:b"]);
    missing.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:a",
        &[("protected", "true")],
    ));
    let missing_r = evaluate(&test, &missing, &ctx);
    assert_eq!(
        missing_r.effectiveness,
        Effectiveness::InsufficientEvidence,
        "known repo without a protection envelope is missing, not Ineffective"
    );
    assert_ne!(missing_r.effectiveness, Effectiveness::Ineffective);

    let mut fail = EvidenceSet::new();
    authoritative_repos(&mut fail, &["repo:a", "repo:b"]);
    fail.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:a",
        &[("protected", "true")],
    ));
    fail.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:b",
        &[("protected", "false")],
    ));
    let fail_r = evaluate(&test, &fail, &ctx);
    assert_eq!(
        fail_r.effectiveness,
        Effectiveness::Ineffective,
        "authoritative unprotected subject is Ineffective"
    );
    let fail_json = serde_json::to_value(&fail_r).unwrap();
    let failing = fail_json
        .get("population")
        .and_then(|p| p.get("failingSubjects"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert!(
        failing.to_string().contains("repo:b"),
        "failing subject is named, got {failing}"
    );

    let mut stale = EvidenceSet::new();
    authoritative_repos(&mut stale, &["repo:a"]);
    stale.insert(seal_at(
        PINNED_OBSERVATION_TYPE,
        "repo:a",
        &[("protected", "true")],
        collected(48),
    ));
    let stale_r = evaluate(&test, &stale, &ctx);
    assert_eq!(
        stale_r.effectiveness,
        Effectiveness::StaleEvidence,
        "envelope older than max_age is StaleEvidence, not missing/fail"
    );

    let mut scan_missing = EvidenceSet::new();
    authoritative_repos(&mut scan_missing, &["repo:a"]);
    let scan = compiled(
        "test.source.secret-scanning-enabled",
        "source.security-scanning",
        repo_all_subjects("source.security.secret_scanning.enabled", "enabled"),
    );
    let scan_r = evaluate(&scan, &scan_missing, &ctx);
    assert_eq!(
        scan_r.effectiveness,
        Effectiveness::InsufficientEvidence,
        "missing scan evidence is InsufficientEvidence, never a technical failure"
    );
    assert_ne!(scan_r.effectiveness, Effectiveness::Ineffective);

    let manual = compiled(
        "test.source.secure-development-policy-attested",
        "control.source.secure-development-policy",
        TestExpr::ManualReview,
    );
    let mut attested = EvidenceSet::new();
    attested.insert(seal(
        "evidence.repository.secure-development-policy",
        "org:baseline",
        &[("attested", "true")],
    ));
    let manual_r = evaluate(&manual, &attested, &ctx);
    assert_eq!(
        manual_r.effectiveness,
        Effectiveness::ManualReviewRequired,
        "ManualReview cannot auto-pass from a single technical flag"
    );

    let mut excepted = EvidenceSet::new();
    authoritative_repos(&mut excepted, &["repo:a", "repo:b"]);
    excepted.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:a",
        &[("protected", "true")],
    ));
    excepted.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:b",
        &[("protected", "false")],
    ));
    excepted.insert_exception(bound_exception(
        ExceptionStatus::Approved,
        Some(24),
        "repo:b",
    ));
    let except_r = evaluate(&test, &excepted, &ctx);
    assert_eq!(
        except_r.effectiveness,
        Effectiveness::Effective,
        "today an approved bound exception removes the subject; remaining pass → Effective (not ExceptionApproved)"
    );
    assert_ne!(except_r.effectiveness, Effectiveness::Ineffective);
    let except_json = serde_json::to_value(&except_r).unwrap();
    let excepted_subjects = except_json
        .get("population")
        .and_then(|p| p.get("exceptedSubjects"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert!(
        excepted_subjects.to_string().contains("repo:b"),
        "excepted subject is first-class, got {excepted_subjects}"
    );
}

#[test]
fn expired_and_revoked_exceptions_do_not_except() {
    let ctx = fresh_context();
    let test = compiled(
        "test.source.default-branches-protected",
        PINNED_CONTROL,
        repo_all_subjects(PINNED_OBSERVATION_TYPE, "protected"),
    );

    let mut failing = EvidenceSet::new();
    authoritative_repos(&mut failing, &["repo:a", "repo:b"]);
    failing.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:a",
        &[("protected", "true")],
    ));
    failing.insert(seal(
        PINNED_OBSERVATION_TYPE,
        "repo:b",
        &[("protected", "false")],
    ));

    let mut expired_set = failing.clone();
    expired_set.insert_exception(bound_exception(ExceptionStatus::Expired, None, "repo:b"));
    assert_eq!(
        evaluate(&test, &expired_set, &ctx).effectiveness,
        Effectiveness::Ineffective,
        "expired exceptions do not pass"
    );

    let mut revoked = failing.clone();
    revoked.insert_exception(bound_exception(
        ExceptionStatus::Revoked,
        Some(24),
        "repo:b",
    ));
    assert_eq!(
        evaluate(&test, &revoked, &ctx).effectiveness,
        Effectiveness::Ineffective,
        "revoked exceptions do not pass"
    );

    let mut expired_approved = failing;
    expired_approved.insert_exception(bound_exception(
        ExceptionStatus::Approved,
        Some(-1),
        "repo:b",
    ));
    assert_eq!(
        evaluate(&test, &expired_approved, &ctx).effectiveness,
        Effectiveness::Ineffective,
        "an Approved exception that is already expired does not pass"
    );
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
