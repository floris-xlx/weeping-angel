//! Baseline characterization of the infrastructure catalog surface (Prompt 07).
//!
//! SUPERSEDED by `sdd_infrastructure_catalog_target` after the infrastructure
//! catalog slice landed. Historical characterization of catalog absence.
//! Tests are ignored so absence-of-catalog is not required CI green.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance_ir::{
    AssetId, ControlDomain, ControlId, ControlTestId, PlannedTestKind, SubjectKind,
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

const INFRA_FAMILY_FILES: &[&str] = &[
    "network.toml",
    "crypto.toml",
    "data.toml",
    "database.toml",
    "logging.toml",
    "backup.toml",
    "resilience.toml",
];

const INFRA_CONTROL_PREFIXES: &[&str] = &[
    "control.network.",
    "control.crypto.",
    "control.secret.",
    "control.data.",
    "control.database.",
    "control.logging.",
    "control.backup.",
    "control.resilience.",
];

const REQUIRED_EVIDENCE: &[&str] = &[
    "evidence.network.exposure",
    "evidence.network.firewall-policy",
    "evidence.network.tls-configuration",
    "evidence.data.encryption-at-rest",
    "evidence.data.encryption-in-transit",
    "evidence.crypto.key-state",
    "evidence.secret.storage-configuration",
    "evidence.database.inventory",
    "evidence.database.access-configuration",
    "evidence.logging.configuration",
    "evidence.logging.retention",
    "evidence.logging.alerting",
    "evidence.backup.configuration",
    "evidence.backup.run",
    "evidence.backup.restore-test",
    "evidence.resilience.recovery-plan",
];

const REQUIRED_POPULATION_TESTS: &[&str] = &[
    "test.database.critical-encrypt-at-rest",
    "test.network.public-endpoints-acceptable-tls",
    "test.logging.critical-assets-audit-current",
    "test.logging.retention-meets-threshold",
    "test.backup.required-stores-current",
    "test.backup.restore-test-fresh",
    "test.network.no-prohibited-public-databases",
    "test.secret.approved-storage",
];

const INFRA_FIXTURE_FAMILIES: &[&str] = &[
    "network",
    "crypto",
    "data",
    "database",
    "logging",
    "backup",
    "resilience",
];

const ISO_INFRA_CONTROLS: &[&str] = &[
    "logging.security-events",
    "logging.audit-trail",
    "backup.recovery-testing",
    "encryption.data-at-rest",
    "encryption.data-in-transit",
    "security.tls",
];

const ISO_INFRA_TESTS: &[(&str, &str, &str)] = &[
    (
        "test.logging.security-events",
        "logging.security-events",
        "logging.security-events",
    ),
    (
        "test.logging.audit-trail",
        "logging.audit-trail",
        "logging.audit-trail",
    ),
    (
        "test.backup.recovery-testing",
        "backup.recovery-testing",
        "backup.configuration.present",
    ),
    (
        "test.encryption.data-at-rest",
        "encryption.data-at-rest",
        "encryption.at-rest.configured",
    ),
    (
        "test.encryption.data-in-transit",
        "encryption.data-in-transit",
        "encryption.in-transit.configured",
    ),
    (
        "test.security.tls",
        "security.tls",
        "security.tls.misconfiguration",
    ),
];

const ISO_INFRA_MAPPINGS: &[(&str, &str)] = &[
    ("iso27001:a.8.13", "backup.recovery-testing"),
    ("iso27001:a.8.15", "logging.security-events"),
    ("iso27001:a.8.24", "encryption.data-at-rest"),
    ("iso27001:a.8.24", "encryption.data-in-transit"),
    ("iso27001:a.8.24", "security.tls"),
];

const CLOUD_COLLECTOR_DIRS: &[&str] = &["aws", "azure", "gcp", "google", "cloudflare"];

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

fn catalog_toml_joined() -> String {
    let mut files = Vec::new();
    walk_files(&catalog_v1(), &mut files);
    files.retain(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"));
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn fixture_json_joined() -> String {
    let root = manifest_dir().join("fixtures/assurance/canonical");
    if !root.is_dir() {
        return String::new();
    }
    let mut files = Vec::new();
    walk_files(&root, &mut files);
    files.retain(|p| p.extension().and_then(|e| e.to_str()) == Some("json"));
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn iso_pack_text() -> String {
    let dir = manifest_dir().join("frameworks/iso-27001/2022");
    let mut files = Vec::new();
    walk_files(&dir, &mut files);
    files.retain(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"));
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

fn seal(evidence_type: &str, asset: &str, facts: &[(&str, EvidenceValue)]) -> EvidenceEnvelope {
    seal_at(evidence_type, asset, facts, collected(1))
}

fn seal_at(
    evidence_type: &str,
    asset: &str,
    facts: &[(&str, EvidenceValue)],
    collected_at: chrono::DateTime<Utc>,
) -> EvidenceEnvelope {
    let mut obs = EvidenceObservation::new(EvidenceType::new(evidence_type));
    for (k, v) in facts {
        obs = obs.with_value(*k, v.clone());
    }
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.infrastructure-baseline".into(),
            collected_at,
            scope: "baseline".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn exists_test(id: &str, control: &str, evidence_type: &str) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new(id))
        .control_id(ControlId::new(control))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new(evidence_type))
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

fn compiled(id: &str, control: &str, expr: TestExpr) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new(id))
        .control_id(ControlId::new(control))
        .kind(ControlTestKind::Automated)
        .expr(expr)
        .build()
}

fn authoritative_kind(set: &mut EvidenceSet, kind: &str, ids: &[&str]) {
    for id in ids {
        set.insert(seal(
            "inventory.subject",
            id,
            &[
                ("kind", EvidenceValue::String(kind.into())),
                ("id", EvidenceValue::String((*id).into())),
            ],
        ));
    }
    set.insert(seal(
        "inventory.complete",
        "org:infra-baseline",
        &[
            ("kind", EvidenceValue::String(kind.into())),
            ("authoritative", EvidenceValue::Bool(true)),
        ],
    ));
}

fn product_mentions(needle: &str) -> bool {
    catalog_toml_joined().contains(needle)
        || product_rs_joined().contains(needle)
        || fixture_json_joined().contains(needle)
        || iso_pack_text().contains(needle)
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn dual_suite_baseline_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_infrastructure_catalog_baseline")
            && toml.contains("tests/sdd/infrastructure_catalog.baseline.rs")
            && toml.contains("sdd_infrastructure_catalog_target")
            && toml.contains("tests/sdd/infrastructure_catalog.target.rs"),
        "infrastructure dual-suite must be listed in root Cargo.toml"
    );
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
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
    for family in INFRA_FAMILY_FILES {
        assert!(
            !manifest.contains(family),
            "manifest.toml currently does not list infrastructure family `{family}`"
        );
    }
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn fixture_example_and_identity_are_the_only_catalog_families() {
    let catalog = load_catalog();
    catalog
        .control(PINNED_CONTROL)
        .expect("CAT-015 fixture control remains");
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

    let listed_controls: BTreeSet<String> = catalog.controls().keys().cloned().collect();
    for prefix in INFRA_CONTROL_PREFIXES {
        let leaked: Vec<&String> = listed_controls
            .iter()
            .filter(|id| id.starts_with(prefix))
            .collect();
        assert!(
            leaked.is_empty(),
            "canonical catalog currently has no `{prefix}*` controls, found {leaked:?}"
        );
    }

    for dir in ["controls", "evidence", "tests"] {
        for family in INFRA_FAMILY_FILES {
            let path = catalog_v1().join(dir).join(family);
            assert!(
                !path.exists(),
                "current tree has no catalog family file {}",
                path.display()
            );
        }
        assert!(
            !catalog_v1().join(dir).join("secret.toml").exists(),
            "current tree has no {dir}/secret.toml"
        );
        assert!(
            !catalog_v1().join(dir).join("infrastructure.toml").exists(),
            "current tree has no {dir}/infrastructure.toml"
        );
    }
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn required_infrastructure_evidence_and_population_tests_are_undeclared() {
    let catalog = load_catalog();
    for id in REQUIRED_EVIDENCE {
        assert!(
            !catalog.evidence().contains_key(*id),
            "canonical evidence `{id}` is not declared today"
        );
        assert!(
            !product_mentions(id),
            "product catalog/crates/fixtures/ISO pack currently have no `{id}` contract"
        );
    }
    for id in REQUIRED_POPULATION_TESTS {
        assert!(
            !catalog.tests().contains_key(*id),
            "canonical test `{id}` is not declared today"
        );
        assert!(
            !catalog_toml_joined().contains(id),
            "catalog TOML currently has no `{id}`"
        );
    }
    assert!(
        !catalog.evidence().contains_key("evidence.secret.exposure"),
        "this slice must not create Prompt 06 evidence.secret.exposure"
    );
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn identity_fixtures_exist_and_infrastructure_fixtures_do_not() {
    let identity = manifest_dir().join("fixtures/assurance/canonical/v1/identity");
    for name in IAM_FIXTURES {
        assert!(
            identity.join(name).join("evidence.json").is_file(),
            "IAM fixture `{name}` remains on disk"
        );
    }

    let root = manifest_dir().join("fixtures/assurance/canonical/v1");
    for family in INFRA_FIXTURE_FAMILIES {
        assert!(
            !root.join(family).exists(),
            "infrastructure fixture family `{family}` is not shipped"
        );
    }
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn iso_pack_holds_the_logging_crypto_backup_tls_sliver() {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    for id in ISO_INFRA_CONTROLS {
        assert!(
            control_ids.contains(id),
            "ISO pack missing infrastructure sliver `{id}` (have {control_ids:?})"
        );
    }
    for prefix in INFRA_CONTROL_PREFIXES {
        assert!(
            !control_ids.iter().any(|id| id.starts_with(prefix)),
            "ISO pack must not host canonical `{prefix}*` ids"
        );
    }

    let tests: BTreeMap<&str, &weeping_angel_assurance_ir::PlannedControlTest> =
        pack.tests.iter().map(|t| (t.id.as_str(), t)).collect();
    for (test_id, control_id, evidence) in ISO_INFRA_TESTS {
        let test = tests
            .get(test_id)
            .unwrap_or_else(|| panic!("ISO pack missing {test_id}"));
        assert_eq!(test.control_id.as_str(), *control_id);
        assert_eq!(
            test.required_evidence,
            vec![EvidenceType::new(*evidence)],
            "{test_id} required evidence stays pack-local `{evidence}`"
        );
    }

    let tls = tests
        .get("test.security.tls")
        .expect("ISO TLS test remains");
    assert_eq!(
        tls.break_on,
        vec![EvidenceType::new("security.tls.misconfiguration")],
        "security.tls is finding-shaped break_on, not a TLS-policy population"
    );

    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        !metadata.contains("[test.expression]")
            && !metadata.contains("op = \"all-subjects\"")
            && !metadata.contains("op = \"coverage-at-least\""),
        "ISO logging/crypto/backup/TLS tests remain presence/hybrid stubs"
    );

    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    for (from, to) in ISO_INFRA_MAPPINGS {
        assert!(
            mappings.contains(&format!("from = \"{from}\""))
                && mappings.contains(&format!("to = \"{to}\"")),
            "ISO mapping {from} → {to} must stay"
        );
    }
    assert!(
        !mappings.contains("to = \"control.network.")
            && !mappings.contains("to = \"control.crypto.")
            && !mappings.contains("to = \"control.database.")
            && !mappings.contains("to = \"control.logging.")
            && !mappings.contains("to = \"control.backup.")
            && !mappings.contains("to = \"control.resilience."),
        "ISO mappings stay on pack-local ids until Prompt 12"
    );
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn iso_hybrid_kind_in_toml_loads_as_automated() {
    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        metadata.contains("id = \"test.logging.security-events\"")
            && metadata.contains("kind = \"hybrid\""),
        "pack source still marks security-event logging as hybrid"
    );
    let pack = load_framework_pack("iso-27001", "2022").unwrap();
    let logging = pack
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.logging.security-events")
        .expect("iso logging test");
    assert_eq!(
        logging.kind,
        PlannedTestKind::Automated,
        "pack loader currently maps any non-manual kind to Automated"
    );
    let transit = pack
        .tests
        .iter()
        .find(|t| t.id.as_str() == "test.encryption.data-in-transit")
        .expect("iso transit test");
    assert_eq!(transit.kind, PlannedTestKind::Automated);
    let _ = PlannedTestKind::Hybrid;
    let _ = PlannedTestKind::Manual;
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn iso_encryption_and_logging_existence_pass_on_one_envelope() {
    let ctx = fresh_context();
    let at_rest = exists_test(
        "test.encryption.data-at-rest",
        "encryption.data-at-rest",
        "encryption.at-rest.configured",
    );

    let empty = evaluate(&at_rest, &EvidenceSet::new(), &ctx);
    assert_eq!(
        empty.effectiveness,
        Effectiveness::InsufficientEvidence,
        "missing required envelope is InsufficientEvidence, not Ineffective"
    );

    let mut one = EvidenceSet::new();
    one.insert(seal(
        "encryption.at-rest.configured",
        "db:only",
        &[("encrypted", EvidenceValue::Bool(true))],
    ));
    authoritative_kind(&mut one, "database", &["db:only", "db:plain"]);
    one.insert(seal(
        "encryption.at-rest.configured",
        "db:plain",
        &[("encrypted", EvidenceValue::Bool(false))],
    ));
    let passed = evaluate(&at_rest, &one, &ctx);
    assert_eq!(
        passed.effectiveness,
        Effectiveness::Effective,
        "today one encryption.at-rest.configured envelope satisfies the ISO existence test"
    );

    let mut logging = EvidenceSet::new();
    logging.insert(seal(
        "logging.security-events",
        "asset:one",
        &[("audit_enabled", EvidenceValue::Bool(false))],
    ));
    let log_test = exists_test(
        "test.logging.security-events",
        "logging.security-events",
        "logging.security-events",
    );
    let log_ok = evaluate(&log_test, &logging, &ctx);
    assert_eq!(
        log_ok.effectiveness,
        Effectiveness::Effective,
        "ISO logging existence ignores fact values; one envelope still passes"
    );
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn iso_tls_is_finding_shaped_break_on() {
    let ctx = fresh_context();
    let tls = CompiledControlTest::builder()
        .id(ControlTestId::new("test.security.tls"))
        .control_id(ControlId::new("security.tls"))
        .kind(ControlTestKind::Automated)
        .require(EvidenceType::new("security.tls.misconfiguration"))
        .break_on(EvidenceType::new("security.tls.misconfiguration"))
        .build();

    let empty = evaluate(&tls, &EvidenceSet::new(), &ctx);
    assert_eq!(
        empty.effectiveness,
        Effectiveness::InsufficientEvidence,
        "no TLS finding is missing evidence, not a policy population miss"
    );

    let mut finding = EvidenceSet::new();
    finding.insert(seal(
        "security.tls.misconfiguration",
        "endpoint:web",
        &[("insecure_protocol", EvidenceValue::Bool(true))],
    ));
    let broken = evaluate(&tls, &finding, &ctx);
    assert_eq!(
        broken.effectiveness,
        Effectiveness::Ineffective,
        "presence of security.tls.misconfiguration breaks the ISO TLS test"
    );
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn github_collector_still_advertises_source_star_only() {
    assert!(
        GITHUB_EVIDENCE_TYPES
            .iter()
            .all(|t| t.starts_with("source.")),
        "GitHub collector still emits source.* names only"
    );
    for prefix in [
        "evidence.network.",
        "evidence.database.",
        "evidence.backup.",
        "evidence.logging.",
        "evidence.crypto.",
        "evidence.resilience.",
        "evidence.secret.",
        "evidence.data.",
    ] {
        assert!(
            !GITHUB_EVIDENCE_TYPES.iter().any(|t| t.starts_with(prefix)),
            "collector must not advertise `{prefix}`"
        );
    }
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn no_cloud_collectors_or_database_inventory_resolver() {
    let collector_src = crate_src("weeping-angel-collector");
    for name in CLOUD_COLLECTOR_DIRS {
        assert!(
            !collector_src.join(name).exists(),
            "current tree has no {name} collector module"
        );
    }

    let product = product_rs_joined();
    assert!(
        !product.contains("resolve_database_inventory"),
        "Prompt 03 has no database-inventory special case"
    );
    assert!(
        !product.contains("resolve_network_inventory"),
        "Prompt 03 has no network-inventory special case"
    );
    assert!(
        !product.contains("struct InfraPopulation"),
        "do not fork a second population type"
    );

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

    let value = fs::read_to_string(crate_src("weeping-angel-evidence").join("value.rs")).unwrap();
    assert!(
        value.contains("pub enum EvidenceValue"),
        "the one EvidenceValue lives in weeping-angel-evidence"
    );
    let _ = EvidenceValue::Bool(true);
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn subject_kinds_and_domains_already_cover_infrastructure() {
    let _ = SubjectKind::Database;
    let _ = SubjectKind::DataStore;
    let _ = SubjectKind::Endpoint;
    let _ = SubjectKind::Network;
    let _ = SubjectKind::Asset;
    let _ = SubjectKind::Service;
    let _ = SubjectKind::CloudResource;
    let _ = SubjectKind::CloudAccount;
    let _ = ControlDomain::NetworkSecurity;
    let _ = ControlDomain::Cryptography;
    let _ = ControlDomain::DataProtection;
    let _ = ControlDomain::LoggingMonitoring;
    let _ = ControlDomain::Resilience;
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn generic_inventory_resolves_databases_without_a_family_resolver() {
    let mut set = EvidenceSet::new();
    authoritative_kind(&mut set, "database", &["db:orders", "db:billing"]);
    let index = build_index(&set);
    let pop = resolve_population(
        &SubjectSelector {
            kind: Some("database".into()),
            id: None,
        },
        &set,
        &index,
        Some(&EvidenceType::new("evidence.data.encryption-at-rest")),
        fresh_context().now,
    );
    assert_eq!(pop.completeness, PopulationCompleteness::Authoritative);
    assert_eq!(
        pop.subject_ids,
        vec!["db:billing".to_string(), "db:orders".to_string()]
    );
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn unknown_and_partial_populations_cannot_be_effective_on_all_subjects() {
    let ctx = fresh_context();
    let test = compiled(
        "test.database.critical-encrypt-at-rest",
        "control.database.encryption",
        all_subjects("database", "evidence.data.encryption-at-rest", "encrypted"),
    );

    let mut inferred = EvidenceSet::new();
    inferred.insert(seal(
        "evidence.data.encryption-at-rest",
        "db:a",
        &[("encrypted", EvidenceValue::Bool(true))],
    ));
    let unknown = evaluate(&test, &inferred, &ctx);
    assert_eq!(
        unknown.effectiveness,
        Effectiveness::Inconclusive,
        "observations without inventory.complete stay Unknown → Inconclusive"
    );
    assert_ne!(unknown.effectiveness, Effectiveness::Effective);

    let mut partial = EvidenceSet::new();
    partial.insert(seal(
        "inventory.subject",
        "db:a",
        &[
            ("kind", EvidenceValue::String("database".into())),
            ("id", EvidenceValue::String("db:a".into())),
        ],
    ));
    partial.insert(seal(
        "evidence.data.encryption-at-rest",
        "db:a",
        &[("encrypted", EvidenceValue::Bool(true))],
    ));
    let partial_r = evaluate(&test, &partial, &ctx);
    assert_eq!(
        partial_r.effectiveness,
        Effectiveness::InsufficientEvidence,
        "inventory.subject without inventory.complete is Partial → InsufficientEvidence"
    );
    assert_ne!(partial_r.effectiveness, Effectiveness::Effective);
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn classify_value_treats_retention_day_integers_as_technical() {
    let ctx = fresh_context();
    let test = compiled(
        "test.logging.retention-meets-threshold",
        "control.logging.retention-meets-policy",
        all_subjects("asset", "evidence.logging.retention", "retention_days"),
    );

    let mut set = EvidenceSet::new();
    authoritative_kind(&mut set, "asset", &["asset:app"]);
    set.insert(seal(
        "evidence.logging.retention",
        "asset:app",
        &[("retention_days", EvidenceValue::Integer(90))],
    ));
    let result = evaluate(&test, &set, &ctx);
    assert_eq!(
        result.effectiveness,
        Effectiveness::Ineffective,
        "AllSubjects classifies Integer(90) as Technical, not a threshold comparison"
    );
    let json = serde_json::to_value(&result).unwrap();
    let technical = json
        .get("population")
        .and_then(|p| p.get("technicalSubjects"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    assert!(
        technical.to_string().contains("asset:app"),
        "technical subject is named, got {technical}"
    );

    let bool_test = compiled(
        "test.logging.retention-meets-threshold",
        "control.logging.retention-meets-policy",
        all_subjects("asset", "evidence.logging.retention", "meets_threshold"),
    );
    let mut bool_set = EvidenceSet::new();
    authoritative_kind(&mut bool_set, "asset", &["asset:app"]);
    bool_set.insert(seal(
        "evidence.logging.retention",
        "asset:app",
        &[
            ("retention_days", EvidenceValue::Integer(90)),
            ("meets_threshold", EvidenceValue::Bool(true)),
        ],
    ));
    let bool_ok = evaluate(&bool_test, &bool_set, &ctx);
    assert_eq!(
        bool_ok.effectiveness,
        Effectiveness::Effective,
        "a boolean meets_threshold fact is classifiable today"
    );
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn public_contract_documents_iam_not_infrastructure() {
    let contract =
        fs::read_to_string(manifest_dir().join("docs/contracts/assurance-runtime.md")).unwrap();
    assert!(
        contract.contains("control.identity.")
            && contract.contains("fixtures/assurance/canonical/v1/identity/"),
        "public contract still documents the landed IAM family"
    );
    assert!(
        contract.contains(PINNED_CONTROL),
        "public contract still names the fixture control"
    );
    for needle in [
        "control.network.",
        "evidence.database.",
        "fixtures/assurance/canonical/v1/network",
        "fixtures/assurance/canonical/v1/database",
    ] {
        assert!(
            !contract.contains(needle),
            "public contract currently does not name infrastructure `{needle}`"
        );
    }
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
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
}

#[ignore = "superseded by sdd_infrastructure_catalog_target"]
#[test]
fn no_framework_retention_or_tls_constants_in_product_crates() {
    let product = product_rs_joined();
    assert!(
        !product.contains("ISO_RETENTION_DAYS"),
        "product crates currently have no ISO_RETENTION_DAYS constant"
    );
    assert!(
        !product.contains("const MIN_TLS"),
        "product crates currently have no MIN_TLS framework constant"
    );
}
