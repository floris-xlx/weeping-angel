//! Baseline suite for Prompt 08 (governance / vendor / personnel / incident /
//! continuity-governance catalog).
//!
//! Characterization of CURRENT tree (spec SHA `e430980c0d27a8138a153d49b62ddf3c57827891`):
//! Prompt 01 catalog ships `fixture.example` + IAM `identity` only.
//! There is no governance/risk/personnel/vendor/incident family TOML, no
//! `evidence.manual.attestation` catalog type, and no
//! `fixtures/assurance/canonical/v1/governance/*`. The ISO-pack
//! organizational sliver and Prompt 04 IAM sibling remain.
//!
//! After the target suite is GREEN, ignore this file
//! (`#[ignore = "superseded by sdd_governance_catalog_target"]`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use weeping_angel_assurance_ir::{
    AssessmentRequests, AssetId, ControlDomain, ControlId, ControlTestId, Exception, ExceptionId,
    ExceptionStatus, PlannedTestKind, Risk, RiskId, RiskStatus, SelectorScope, SubjectKind,
    SubjectSelector as IrSubjectSelector,
};
use weeping_angel_canonical_catalog::{
    CATALOG_SCHEMA, CanonicalCatalog, CatalogError, DIGEST_PREFIX,
};
use weeping_angel_collector::ManualEvidence;
use weeping_angel_control_test::population::resolve_population;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceSelector,
    EvidenceSet, PopulationCompleteness, SubjectSelector, TestExpr, build_index, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType, EvidenceValue,
};
use weeping_angel_framework::{FrameworkCapabilities, load_framework_pack};

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

const GOVERNANCE_FAMILY_FILES: &[&str] = &[
    "governance.toml",
    "risk.toml",
    "personnel.toml",
    "vendor.toml",
    "incident.toml",
];

const GOVERNANCE_CONTROL_PREFIXES: &[&str] = &[
    "control.governance.",
    "control.risk.",
    "control.personnel.",
    "control.vendor.",
    "control.incident.",
];

const PROMPT08_CONTROLS: &[&str] = &[
    "control.governance.information-security-policy",
    "control.governance.policy-review",
    "control.governance.roles-and-responsibilities",
    "control.governance.security-objectives",
    "control.governance.documented-scope",
    "control.governance.internal-audit",
    "control.governance.management-review",
    "control.governance.corrective-action",
    "control.governance.continual-improvement",
    "control.governance.data-classification-policy",
    "control.governance.acceptable-use-policy",
    "control.governance.asset-ownership",
    "control.governance.document-control",
    "control.governance.evidence-retention",
    "control.governance.audit-program",
    "control.risk.assessment",
    "control.risk.treatment",
    "control.risk.ownership",
    "control.risk.acceptance",
    "control.incident.response-plan",
    "control.incident.exercise",
    "control.incident.postmortem",
    "control.personnel.security-awareness",
    "control.personnel.role-specific-training",
    "control.personnel.onboarding-offboarding",
    "control.personnel.confidentiality-commitment",
    "control.personnel.policy-acknowledgement",
    "control.vendor.inventory",
    "control.vendor.risk-review",
    "control.vendor.security-requirements",
    "control.vendor.reassessment",
    "control.vendor.cloud-governance",
    "control.resilience.business-continuity-plan",
    "control.resilience.disaster-recovery-governance",
];

const PROMPT08_EVIDENCE: &[&str] = &[
    "evidence.manual.attestation",
    "evidence.governance.policy",
    "evidence.governance.policy-review",
    "evidence.governance.management-review",
    "evidence.governance.internal-audit",
    "evidence.risk.assessment",
    "evidence.risk.treatment",
    "evidence.personnel.training",
    "evidence.personnel.acknowledgement",
    "evidence.vendor.inventory",
    "evidence.vendor.risk-review",
    "evidence.incident.exercise",
    "evidence.resilience.continuity-plan",
];

const PROMPT08_TESTS: &[&str] = &[
    "test.governance.policy-current",
    "test.governance.management-review-current",
    "test.governance.internal-audit-current",
    "test.personnel.training-current-all",
    "test.vendor.critical-risk-review-current",
    "test.incident.exercise-current",
];

const GOVERNANCE_FIXTURES: &[&str] = &[
    "current-documents",
    "stale-documents",
    "missing-documents",
    "incomplete-training-population",
    "vendor-review-gaps",
    "approved-exception",
    "expired-exception",
    "manual-review-despite-evidence",
];

const ISO_ORG_CONTROLS: &[&str] = &[
    "incident.response-process",
    "supplier.security-assessment",
    "personnel.access-termination",
    "access.periodic-review",
];

const ISO_ORG_TESTS: &[(&str, &str, &str, PlannedTestKind)] = &[
    (
        "test.incident.response-process",
        "incident.response-process",
        "policy.security.reviewed",
        PlannedTestKind::Manual,
    ),
    (
        "test.supplier.security-assessment",
        "supplier.security-assessment",
        "policy.supplier.assessed",
        PlannedTestKind::Manual,
    ),
    (
        "test.personnel.access-termination",
        "personnel.access-termination",
        "personnel.access.terminated",
        PlannedTestKind::Automated,
    ),
    (
        "test.access.periodic-review",
        "access.periodic-review",
        "policy.access.reviewed",
        PlannedTestKind::Manual,
    ),
];

const ISO_ORG_MAPPINGS: &[(&str, &str)] = &[
    ("iso27001:a.5.19", "supplier.security-assessment"),
    ("iso27001:a.5.24", "incident.response-process"),
    ("iso27001:5.2", "incident.response-process"),
    ("iso27001:a.5.1", "incident.response-process"),
    ("iso27001:a.5.16", "personnel.access-termination"),
    ("iso27001:a.6.5", "personnel.access-termination"),
    ("iso27001:a.5.18", "access.periodic-review"),
];

const GRC_PRODUCT_TOKENS: &[&str] = &["vanta", "drata", "servicenow", "jira"];

const SSOT_01_04: &[&str] = &[
    "docs/sdd/canonical-assurance-catalog-v1.md",
    "docs/sdd/typed-evidence.md",
    "docs/sdd/population-runtime.md",
    "docs/sdd/iam-canonical-assurance-catalog.md",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn catalog_v1() -> PathBuf {
    manifest_dir().join("catalog/canonical/v1")
}

fn walk_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_files(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

fn crate_src(name: &str) -> PathBuf {
    manifest_dir().join("crates").join(name).join("src")
}

fn crate_sources_joined(name: &str) -> String {
    let mut files = Vec::new();
    walk_files(&crate_src(name), "rs", &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1()).unwrap_or_else(|e| {
        panic!(
            "Prompt 01 catalog must already load offline at {}: {e}",
            catalog_v1().display()
        )
    })
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap(),
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
            collector_id: "fixture.governance-baseline".into(),
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
        "org:governance-baseline",
        &[
            ("kind", EvidenceValue::String(kind.into())),
            ("authoritative", EvidenceValue::Bool(true)),
        ],
    ));
}

fn bound_exception(id: &str, subject_kind: SubjectKind, subject_id: &str) -> Exception {
    let mut ex = Exception::new(ExceptionId::new(id), "timeboxed organizational waiver");
    ex.status = ExceptionStatus::Approved;
    ex.expires_at = Some(fresh_context().now + chrono::Duration::days(30));
    let mut ids = BTreeSet::new();
    ids.insert(subject_id.to_string());
    ex.subjects.push(IrSubjectSelector {
        kind: subject_kind,
        ids,
        tags: BTreeMap::new(),
        scope: SelectorScope::AnyOf,
    });
    ex
}

#[test]
fn dual_suite_baseline_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_governance_catalog_baseline")
            && toml.contains("tests/sdd/governance_catalog.baseline.rs")
            && toml.contains("sdd_governance_catalog_target")
            && toml.contains("tests/sdd/governance_catalog.target.rs"),
        "Prompt 08 dual-suite must be listed in root Cargo.toml (tests/sdd/*.rs is not auto-discovered)"
    );
}

#[test]
fn catalog_manifest_lists_only_fixture_example_and_identity() {
    let manifest = fs::read_to_string(catalog_v1().join("manifest.toml")).unwrap();
    assert!(
        manifest.contains(CATALOG_SCHEMA),
        "Prompt 01 schema must remain on the shipped manifest"
    );
    assert!(
        manifest.contains("controls/fixture.example.toml")
            && manifest.contains("controls/identity.toml")
            && manifest.contains("evidence/fixture.example.toml")
            && manifest.contains("evidence/identity.toml")
            && manifest.contains("tests/fixture.example.toml")
            && manifest.contains("tests/identity.toml"),
        "manifest currently lists fixture.example + identity"
    );
    for family in GOVERNANCE_FAMILY_FILES {
        assert!(
            !manifest.contains(family),
            "manifest currently lists no governance-family `{family}`:\n{manifest}"
        );
    }

    for section in ["controls", "evidence", "tests"] {
        let dir = catalog_v1().join(section);
        let mut names = BTreeSet::new();
        for entry in fs::read_dir(&dir).unwrap() {
            names.insert(entry.unwrap().file_name().to_string_lossy().into_owned());
        }
        assert_eq!(
            names,
            BTreeSet::from(["fixture.example.toml".into(), "identity.toml".into()]),
            "{section}/ currently holds only fixture.example + identity"
        );
        for family in GOVERNANCE_FAMILY_FILES {
            assert!(
                !dir.join(family).exists(),
                "current tree has no {section}/{family}"
            );
        }
    }
}

#[test]
fn canonical_catalog_load_validate_digest_exist() {
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
}

#[test]
fn loaded_catalog_has_no_governance_family() {
    let catalog = load_catalog();
    catalog
        .control(PINNED_CONTROL)
        .expect("fixture.example control remains");

    for prefix in GOVERNANCE_CONTROL_PREFIXES {
        let leaked: Vec<&String> = catalog
            .controls()
            .keys()
            .filter(|id| id.starts_with(prefix))
            .collect();
        assert!(
            leaked.is_empty(),
            "canonical catalog currently has no `{prefix}*` controls, found {leaked:?}"
        );
    }
    for id in PROMPT08_CONTROLS {
        assert!(
            !catalog.controls().contains_key(*id),
            "Prompt 08 control `{id}` is not in the shipped catalog"
        );
    }
    match catalog.control("control.governance.information-security-policy") {
        Err(CatalogError::UnknownControl(id)) => {
            assert_eq!(id, "control.governance.information-security-policy");
        }
        other => panic!("expected UnknownControl, got {other:?}"),
    }
    assert_eq!(
        catalog
            .controls()
            .keys()
            .filter(|id| id.starts_with("control.governance."))
            .count(),
        0,
        "governance control count is 0 today, not 30–45"
    );
}

#[test]
fn evidence_manual_attestation_is_not_catalog_content() {
    let catalog = load_catalog();
    for id in PROMPT08_EVIDENCE {
        assert!(
            !catalog.evidence().contains_key(*id),
            "canonical evidence `{id}` is not declared"
        );
    }
    for id in catalog.evidence().keys() {
        assert!(
            !id.starts_with("evidence.governance.")
                && !id.starts_with("evidence.risk.")
                && !id.starts_with("evidence.personnel.")
                && !id.starts_with("evidence.vendor.")
                && !id.starts_with("evidence.incident.")
                && *id != "evidence.manual.attestation"
                && *id != "evidence.resilience.continuity-plan",
            "unexpected governance-family evidence `{id}`"
        );
    }
    assert!(
        catalog
            .evidence()
            .contains_key("evidence.source.protected-branch")
            || catalog
                .evidence()
                .keys()
                .any(|id| id.starts_with("evidence.identity.")),
        "fixture/IAM evidence remains"
    );
}

#[test]
fn required_governance_tests_are_undeclared() {
    let catalog = load_catalog();
    for id in PROMPT08_TESTS {
        assert!(
            !catalog.tests().contains_key(*id),
            "Prompt 08 test `{id}` is not declared"
        );
    }
    for test in catalog.tests().values() {
        assert!(
            !test.id.starts_with("test.governance.")
                && !test.id.starts_with("test.risk.")
                && !test.id.starts_with("test.personnel.")
                && !test.id.starts_with("test.vendor.")
                && !test.id.starts_with("test.incident."),
            "catalog already has a governance-family test {}",
            test.id
        );
    }
}

#[test]
fn iam_sibling_and_identity_fixtures_remain() {
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

    let identity = manifest_dir().join("fixtures/assurance/canonical/v1/identity");
    for name in IAM_FIXTURES {
        assert!(
            identity.join(name).join("evidence.json").is_file(),
            "IAM fixture `{name}` remains on disk"
        );
    }
}

#[test]
fn governance_fixtures_are_absent() {
    let root = manifest_dir().join("fixtures/assurance/canonical/v1");
    assert!(
        root.join("identity").is_dir(),
        "IAM fixtures already exist and must not be treated as the governance library"
    );
    assert!(
        !root.join("governance").exists(),
        "fixtures/assurance/canonical/v1/governance is not shipped"
    );
    for name in GOVERNANCE_FIXTURES {
        assert!(
            !root.join("governance").join(name).exists(),
            "golden fixture `{name}` is absent"
        );
    }
}

#[test]
fn iso_pack_holds_the_organizational_sliver() {
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<&str> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    for id in ISO_ORG_CONTROLS {
        assert!(
            control_ids.contains(id),
            "ISO pack missing organizational sliver `{id}` (have {control_ids:?})"
        );
    }
    for prefix in GOVERNANCE_CONTROL_PREFIXES {
        assert!(
            !control_ids.iter().any(|id| id.starts_with(prefix)),
            "ISO pack must not host canonical `{prefix}*` ids"
        );
    }

    let tests: BTreeMap<&str, &weeping_angel_assurance_ir::PlannedControlTest> =
        pack.tests.iter().map(|t| (t.id.as_str(), t)).collect();
    for (test_id, control_id, evidence, kind) in ISO_ORG_TESTS {
        let test = tests
            .get(test_id)
            .unwrap_or_else(|| panic!("ISO pack missing {test_id}"));
        assert_eq!(test.control_id.as_str(), *control_id);
        assert_eq!(
            test.required_evidence,
            vec![EvidenceType::new(*evidence)],
            "{test_id} required evidence stays pack-local `{evidence}`"
        );
        assert_eq!(
            test.kind, *kind,
            "{test_id}: pack loader keeps manual as Manual and maps hybrid to Automated"
        );
    }

    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        metadata.contains("id = \"incident.response-process\"")
            && metadata.contains("automation = \"Manual\"")
            && metadata.contains("id = \"test.incident.response-process\"")
            && metadata.contains("required = [\"policy.security.reviewed\"]")
            && metadata.contains("id = \"supplier.security-assessment\"")
            && metadata.contains("required = [\"policy.supplier.assessed\"]")
            && metadata.contains("id = \"personnel.access-termination\"")
            && metadata.contains("required = [\"personnel.access.terminated\"]")
            && metadata.contains("id = \"access.periodic-review\"")
            && metadata.contains("required = [\"policy.access.reviewed\"]"),
        "ISO pack still owns the organizational sliver"
    );
    assert!(
        !metadata.contains("control.governance.")
            && !metadata.contains("evidence.manual.attestation")
            && !metadata.contains("evidence.governance.")
            && !metadata.contains("test.governance.")
            && !metadata.contains("test.personnel.training-current-all"),
        "ISO pack must not already contain the canonical governance library"
    );
    assert!(
        !metadata.contains("[test.expression]")
            && !metadata.contains("op = \"all-subjects\"")
            && !metadata.contains("op = \"fresh-within\""),
        "ISO organizational tests remain presence/hybrid/manual stubs"
    );

    let incident = pack
        .controls
        .iter()
        .find(|c| c.id().as_str() == "incident.response-process")
        .expect("incident.response-process remains");
    assert!(
        incident.domains().is_empty(),
        "ISO pack control remains id/title/description (no catalog domains)"
    );
}

#[test]
fn iso_mappings_still_point_at_pack_organizational_ids() {
    let mappings =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/mappings.toml")).unwrap();
    for (from, to) in ISO_ORG_MAPPINGS {
        assert!(
            mappings.contains(&format!("from = \"{from}\""))
                && mappings.contains(&format!("to = \"{to}\"")),
            "ISO mapping {from} → {to} must stay"
        );
    }
    assert!(
        !mappings.contains("to = \"control.governance.")
            && !mappings.contains("to = \"control.incident.")
            && !mappings.contains("to = \"control.vendor.")
            && !mappings.contains("to = \"control.personnel.")
            && !mappings.contains("to = \"control.risk."),
        "ISO mappings stay on pack-local ids until Prompt 12"
    );
}

#[test]
fn iso_incident_manual_requires_legacy_manual_attestation_type() {
    let ctx = fresh_context();
    let manual = CompiledControlTest::builder()
        .id(ControlTestId::new("test.incident.response-process"))
        .control_id(ControlId::new("incident.response-process"))
        .kind(ControlTestKind::Manual)
        .require(EvidenceType::new("policy.security.reviewed"))
        .build();

    let empty = evaluate(&manual, &EvidenceSet::new(), &ctx);
    assert_eq!(
        empty.effectiveness,
        Effectiveness::ManualReviewRequired,
        "Manual kind without `manual_attestation` is ManualReviewRequired, not a catalog attestation"
    );

    let mut policy_only = EvidenceSet::new();
    policy_only.insert(seal(
        "policy.security.reviewed",
        "org:acme",
        &[("reviewed", EvidenceValue::Bool(true))],
    ));
    let still_manual = evaluate(&manual, &policy_only, &ctx);
    assert_eq!(
        still_manual.effectiveness,
        Effectiveness::ManualReviewRequired,
        "a policy.security.reviewed envelope is not a first-class evidence.manual.attestation"
    );

    let mut attested = policy_only;
    attested.insert(seal(
        "manual_attestation",
        "org:acme",
        &[("attested_by", EvidenceValue::String("auditor".into()))],
    ));
    let passed = evaluate(&manual, &attested, &ctx);
    assert_eq!(
        passed.effectiveness,
        Effectiveness::Effective,
        "today Manual kind auto-passes once the legacy `manual_attestation` type is present"
    );
}

#[test]
fn iso_policy_existence_passes_on_one_envelope() {
    let ctx = fresh_context();
    let incident = exists_test(
        "test.incident.response-process",
        "incident.response-process",
        "policy.security.reviewed",
    );
    let empty = evaluate(&incident, &EvidenceSet::new(), &ctx);
    assert_eq!(
        empty.effectiveness,
        Effectiveness::InsufficientEvidence,
        "missing required envelope is InsufficientEvidence, not Ineffective"
    );

    let mut one = EvidenceSet::new();
    one.insert(seal(
        "policy.security.reviewed",
        "org:acme",
        &[("reviewed", EvidenceValue::Bool(false))],
    ));
    let passed = evaluate(&incident, &one, &ctx);
    assert_eq!(
        passed.effectiveness,
        Effectiveness::Effective,
        "today one policy.security.reviewed envelope satisfies the ISO existence test"
    );
}

#[test]
fn document_present_is_not_a_population_or_freshness_test() {
    let ctx = fresh_context();
    let exists = exists_test(
        "test.personnel.training-current-all",
        "control.personnel.role-specific-training",
        "evidence.personnel.training",
    );

    let mut one_of_two = EvidenceSet::new();
    authoritative_kind(&mut one_of_two, "user", &["user:alice", "user:bob"]);
    one_of_two.insert(seal(
        "evidence.personnel.training",
        "user:alice",
        &[("current", EvidenceValue::Bool(true))],
    ));
    let existence = evaluate(&exists, &one_of_two, &ctx);
    assert_eq!(
        existence.effectiveness,
        Effectiveness::Effective,
        "Exists/require today treats one training envelope as a pass — that is not population coverage"
    );

    let population = compiled(
        "test.personnel.training-current-all",
        "control.personnel.role-specific-training",
        all_subjects("user", "evidence.personnel.training", "current"),
    );
    let pop = evaluate(&population, &one_of_two, &ctx);
    assert_ne!(
        pop.effectiveness,
        Effectiveness::Effective,
        "Prompt 03 AllSubjects already refuses Effective on an incomplete training population"
    );
    assert_eq!(
        pop.effectiveness,
        Effectiveness::InsufficientEvidence,
        "missing training for a known subject is InsufficientEvidence, got {:?}",
        pop.effectiveness
    );
}

#[test]
fn test_expr_manual_review_always_yields_manual_review_required() {
    let compiled = CompiledControlTest::builder()
        .id(ControlTestId::new("test.governance.roles-attested"))
        .control_id(ControlId::new(
            "control.governance.roles-and-responsibilities",
        ))
        .kind(ControlTestKind::Automated)
        .expr(TestExpr::ManualReview)
        .build();
    let mut supporting = EvidenceSet::new();
    supporting.insert(seal(
        "evidence.governance.policy",
        "org:acme",
        &[(
            "policy_kind",
            EvidenceValue::String("information-security".into()),
        )],
    ));
    let result = evaluate(&compiled, &supporting, &fresh_context());
    assert_eq!(
        result.effectiveness,
        Effectiveness::ManualReviewRequired,
        "TestExpr::ManualReview always yields ManualReviewRequired"
    );
    assert_eq!(result.rationale, "expression requires manual review");
}

#[test]
fn exception_approved_promotion_is_identity_break_glass_shaped() {
    let ctx = fresh_context();

    let mut bg = EvidenceSet::new();
    bg.insert(seal(
        "evidence.identity.inventory",
        "org:acme",
        &[
            ("account_kind", EvidenceValue::String("organization".into())),
            ("authoritative", EvidenceValue::Bool(true)),
        ],
    ));
    bg.insert(seal(
        "evidence.identity.inventory",
        "user:break-glass",
        &[("account_kind", EvidenceValue::String("break-glass".into()))],
    ));
    bg.insert(seal(
        "evidence.identity.privileged-membership",
        "user:break-glass",
        &[("privileged", EvidenceValue::Bool(true))],
    ));
    bg.insert(seal(
        "evidence.identity.mfa-status",
        "user:break-glass",
        &[("mfa_enabled", EvidenceValue::Bool(false))],
    ));
    let bg_r = evaluate(
        &compiled(
            "test.identity.privileged-mfa-enabled",
            "control.identity.privileged-mfa",
            all_subjects(
                "privilegedIdentity",
                "evidence.identity.mfa-status",
                "mfa_enabled",
            ),
        ),
        &bg,
        &ctx,
    );
    assert_eq!(
        bg_r.effectiveness,
        Effectiveness::ExceptionApproved,
        "evaluate_coverage promotes Ineffective → ExceptionApproved only for identity break-glass"
    );

    let mut vendors = EvidenceSet::new();
    authoritative_kind(&mut vendors, "vendor", &["vendor:a", "vendor:b"]);
    vendors.insert(seal(
        "evidence.vendor.risk-review",
        "vendor:a",
        &[("current", EvidenceValue::Bool(true))],
    ));
    vendors.insert_exception(bound_exception(
        "exc:vendor-b",
        SubjectKind::Vendor,
        "vendor:b",
    ));
    let silent = evaluate(
        &compiled(
            "test.vendor.critical-risk-review-current",
            "control.vendor.risk-review",
            all_subjects("vendor", "evidence.vendor.risk-review", "current"),
        ),
        &vendors,
        &ctx,
    );
    assert_eq!(
        silent.effectiveness,
        Effectiveness::Effective,
        "today an approved bound vendor exception is removed from the denominator and the rest can silently be Effective"
    );
    assert_ne!(
        silent.effectiveness,
        Effectiveness::ExceptionApproved,
        "ExceptionApproved promotion is not generic for vendor/personnel subjects"
    );
}

#[test]
fn expired_and_empty_exceptions_do_not_suppress_missing() {
    let ctx = fresh_context();
    let expr = all_subjects("vendor", "evidence.vendor.risk-review", "current");
    let test = compiled(
        "test.vendor.critical-risk-review-current",
        "control.vendor.risk-review",
        expr,
    );

    let mut expired_set = EvidenceSet::new();
    authoritative_kind(&mut expired_set, "vendor", &["vendor:a", "vendor:b"]);
    expired_set.insert(seal(
        "evidence.vendor.risk-review",
        "vendor:a",
        &[("current", EvidenceValue::Bool(true))],
    ));
    let mut expired = bound_exception("exc:vendor-b-expired", SubjectKind::Vendor, "vendor:b");
    expired.status = ExceptionStatus::Expired;
    expired_set.insert_exception(expired);
    let expired_r = evaluate(&test, &expired_set, &ctx);
    assert_eq!(
        expired_r.effectiveness,
        Effectiveness::InsufficientEvidence,
        "expired exceptions must not suppress a missing vendor review"
    );
    assert_ne!(expired_r.effectiveness, Effectiveness::Effective);

    let mut empty_subjects = EvidenceSet::new();
    authoritative_kind(&mut empty_subjects, "vendor", &["vendor:a", "vendor:b"]);
    empty_subjects.insert(seal(
        "evidence.vendor.risk-review",
        "vendor:a",
        &[("current", EvidenceValue::Bool(true))],
    ));
    let mut blanket = Exception::new(ExceptionId::new("exc:empty"), "no subjects bound");
    blanket.status = ExceptionStatus::Approved;
    blanket.expires_at = Some(ctx.now + chrono::Duration::days(30));
    empty_subjects.insert_exception(blanket);
    let empty_r = evaluate(&test, &empty_subjects, &ctx);
    assert_eq!(
        empty_r.effectiveness,
        Effectiveness::InsufficientEvidence,
        "empty Exception.subjects does not mean the entire inventory"
    );
}

#[test]
fn evidence_value_with_value_exists_and_is_the_only_enum() {
    let obs = EvidenceObservation::new(EvidenceType::new("evidence.governance.policy"))
        .with_value("current", EvidenceValue::Bool(true))
        .with_value("version", EvidenceValue::Integer(3))
        .with_fact("attested_by", "ciso");
    assert!(matches!(
        obs.fact_value("current"),
        Some(EvidenceValue::Bool(true))
    ));
    assert!(matches!(
        obs.fact_value("version"),
        Some(EvidenceValue::Integer(3))
    ));
    assert_eq!(obs.fact("attested_by"), Some("ciso"));
    assert!(matches!(
        obs.fact_value("attested_by"),
        Some(EvidenceValue::String(s)) if s == "ciso"
    ));

    let mut rust_files = Vec::new();
    walk_files(&manifest_dir().join("crates"), "rs", &mut rust_files);
    walk_files(&manifest_dir().join("src"), "rs", &mut rust_files);
    let mut catalog_structs = 0usize;
    let mut evidence_value_enums = 0usize;
    for path in &rust_files {
        let text = fs::read_to_string(path).unwrap();
        if text.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("pub struct CanonicalCatalog") || t.starts_with("struct CanonicalCatalog")
        }) {
            catalog_structs += 1;
            assert!(
                path.components()
                    .any(|c| c.as_os_str() == "weeping-angel-canonical-catalog"),
                "CanonicalCatalog must stay in weeping-angel-canonical-catalog ({})",
                path.display()
            );
        }
        if text.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("pub enum EvidenceValue") || t.starts_with("enum EvidenceValue {")
        }) {
            evidence_value_enums += 1;
            assert!(
                path.components()
                    .any(|c| c.as_os_str() == "weeping-angel-evidence"),
                "EvidenceValue must stay in weeping-angel-evidence ({})",
                path.display()
            );
        }
        assert!(
            !text.contains("struct GovernanceCatalog")
                && !text.contains("fn resolve_personnel_inventory")
                && !text.contains("fn resolve_vendor_inventory")
                && !text.contains("struct GovernanceException"),
            "no local governance catalog/population/exception fork in {}",
            path.display()
        );
    }
    assert_eq!(catalog_structs, 1, "exactly one CanonicalCatalog loader");
    assert_eq!(evidence_value_enums, 1, "exactly one EvidenceValue enum");
}

#[test]
fn generic_inventory_resolves_personnel_and_vendors_without_a_family_resolver() {
    let mut set = EvidenceSet::new();
    authoritative_kind(&mut set, "user", &["user:alice", "user:bob"]);
    authoritative_kind(&mut set, "vendor", &["vendor:critical"]);
    let index = build_index(&set);
    let people = resolve_population(
        &SubjectSelector {
            kind: Some("user".into()),
            id: None,
        },
        &set,
        &index,
        Some(&EvidenceType::new("evidence.personnel.training")),
        fresh_context().now,
    );
    assert_eq!(people.completeness, PopulationCompleteness::Authoritative);
    assert_eq!(
        people.subject_ids,
        vec!["user:alice".to_string(), "user:bob".to_string()]
    );

    let vendors = resolve_population(
        &SubjectSelector {
            kind: Some("vendor".into()),
            id: None,
        },
        &set,
        &index,
        Some(&EvidenceType::new("evidence.vendor.risk-review")),
        fresh_context().now,
    );
    assert_eq!(vendors.completeness, PopulationCompleteness::Authoritative);
    assert_eq!(vendors.subject_ids, vec!["vendor:critical".to_string()]);

    let pop = crate_sources_joined("weeping-angel-control-test");
    assert!(
        pop.contains("fn resolve_identity_inventory"),
        "identity inventory special-case remains"
    );
    assert!(
        pop.contains("inventory.subject") && pop.contains("inventory.complete"),
        "generic inventory.subject / inventory.complete path must remain"
    );
    assert!(pop.contains("pub fn evaluate_coverage"));
}

#[test]
fn risk_ir_is_a_minimal_record_not_a_grc_engine() {
    let risk = Risk::new(
        RiskId::new("risk:org-1"),
        "supplier concentration",
        "single critical vendor",
    );
    assert_eq!(risk.status, RiskStatus::Open);
    let json = serde_json::to_value(&risk).unwrap();
    assert_eq!(json["id"], "risk:org-1");
    assert_eq!(json["title"], "supplier concentration");
    assert!(json.get("treatment").is_none());
    assert!(json.get("owner").is_none());
    assert!(json.get("residualScore").is_none());
    let _ = RiskStatus::Accepted;
    let _ = RiskStatus::Mitigated;
    let _ = RiskStatus::Closed;
}

#[test]
fn validator_omits_grc_product_tokens() {
    let src = crate_sources_joined("weeping-angel-canonical-catalog");
    assert!(
        src.contains("const PROVIDER_SEGMENTS") && src.contains("const FRAMEWORK_SEGMENTS"),
        "Prompt 01 reserved-segment lists remain the only validator token gates"
    );
    for token in GRC_PRODUCT_TOKENS {
        let needle = format!("\"{token}\"");
        assert!(
            !src.contains(&needle),
            "catalog validator currently does not reserve GRC product `{token}`"
        );
    }
}

#[test]
fn manual_attestation_is_capability_and_legacy_type() {
    let requests = AssessmentRequests {
        manual_attestation: true,
        ..AssessmentRequests::default()
    };
    assert!(requests.manual_attestation);
    let caps = FrameworkCapabilities {
        supports_manual_attestation: true,
        ..FrameworkCapabilities::default()
    };
    assert!(caps.supports_manual_attestation);

    let iso_manifest =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/manifest.toml")).unwrap();
    assert!(
        iso_manifest.contains("manual_attestation = true"),
        "ISO pack still advertises the compile capability"
    );

    let catalog = load_catalog();
    assert!(
        !catalog
            .evidence()
            .contains_key("evidence.manual.attestation"),
        "legacy `manual_attestation` is not catalog evidence.manual.attestation"
    );
}

#[test]
fn collector_manual_does_not_emit_catalog_attestation() {
    let sealed = ManualEvidence {
        evidence_type: EvidenceType::new("policy.security.reviewed"),
        subject: AssetId::new("org:acme"),
        attested_by: "ciso".into(),
        reason: "policy reviewed".into(),
        artifact: None,
    }
    .seal(collected(1))
    .expect("attested-by is present");
    assert_eq!(
        sealed.observation().evidence_type().as_str(),
        "policy.security.reviewed"
    );
    assert_eq!(sealed.provenance().collector_id, "collector.manual");
    assert_eq!(sealed.observation().fact("attested_by"), Some("ciso"));
    assert_eq!(sealed.observation().fact("reason"), Some("policy reviewed"));
    assert!(sealed.observation().fact("review_state").is_none());
    assert!(sealed.observation().fact("kind").is_none());
    assert!(sealed.observation().fact("artifact_ref").is_none());

    let missing = ManualEvidence {
        evidence_type: EvidenceType::new("policy.security.reviewed"),
        subject: AssetId::new("org:acme"),
        attested_by: "  ".into(),
        reason: "no".into(),
        artifact: None,
    }
    .seal(collected(1));
    assert!(
        missing.is_err(),
        "manual evidence requires --attested-by; attestation is never synthesized"
    );
}

#[test]
fn subject_kinds_and_domains_already_cover_governance() {
    let _ = SubjectKind::Organization;
    let _ = SubjectKind::User;
    let _ = SubjectKind::Identity;
    let _ = SubjectKind::Vendor;
    let _ = ControlDomain::Governance;
    let _ = ControlDomain::PersonnelSecurity;
    let _ = ControlDomain::SupplierManagement;
    let _ = ControlDomain::IncidentResponse;
    let _ = ControlDomain::Resilience;
    let _ = ControlDomain::AssetManagement;
    assert!(SubjectKind::parse_name("organization").is_some());
    assert!(SubjectKind::parse_name("vendor").is_some());
}

#[test]
fn public_contract_documents_iam_not_governance() {
    let contract =
        fs::read_to_string(manifest_dir().join("docs/contracts/assurance-runtime.md")).unwrap();
    assert!(
        contract.contains("control.identity.")
            && contract.contains("fixtures/assurance/canonical/v1/identity/"),
        "public contract still documents the landed IAM family"
    );
    assert!(
        contract.contains(PINNED_CONTROL) || contract.contains("evidence.identity."),
        "public contract still names fixture/IAM evidence"
    );
    for needle in [
        "control.governance.",
        "evidence.manual.attestation",
        "fixtures/assurance/canonical/v1/governance",
        "evidence.resilience.continuity-plan",
    ] {
        assert!(
            !contract.contains(needle),
            "public contract currently does not name governance `{needle}`"
        );
    }
}

#[test]
fn prompt_01_and_04_ssot_docs_are_not_overwritten() {
    for rel in SSOT_01_04 {
        let path = manifest_dir().join(rel);
        assert!(path.is_file(), "Prompt 01–04 SSOT must remain at {rel}");
    }
    let cat = fs::read_to_string(manifest_dir().join("docs/sdd/canonical-assurance-catalog-v1.md"))
        .unwrap();
    assert!(
        cat.starts_with("# SDD: Canonical Assurance Catalog v1 infrastructure"),
        "Prompt 01 SSOT title must remain"
    );
    let iam =
        fs::read_to_string(manifest_dir().join("docs/sdd/iam-canonical-assurance-catalog.md"))
            .unwrap();
    assert!(
        iam.starts_with("# SDD: IAM Canonical Assurance Catalog (v1 slice)"),
        "Prompt 04 SSOT title must remain"
    );
}

#[test]
fn spec_and_draft_adr_exist_as_spec_phase_artifacts() {
    let spec = manifest_dir().join("docs/sdd/governance-canonical-assurance-catalog.md");
    let adr = manifest_dir().join("docs/adr/0003-governance-canonical-assurance-catalog.md");
    assert!(
        spec.is_file(),
        "spec-phase SSOT exists; baseline must not assert this path is missing"
    );
    assert!(
        adr.is_file(),
        "draft ADR exists; baseline must not assert this path is missing"
    );
    let spec_text = fs::read_to_string(&spec).unwrap();
    assert!(
        spec_text.starts_with("# SDD: Governance Canonical Assurance Catalog (v1 slice)"),
        "governance SSOT title must remain"
    );
    let adr_text = fs::read_to_string(&adr).unwrap();
    assert!(
        adr_text.contains("**Draft**"),
        "ADR stays Draft until target GREEN"
    );
}

#[test]
fn no_grc_collectors() {
    let collector = crate_src("weeping-angel-collector");
    for name in ["vanta", "drata", "servicenow", "jira", "servicenow_itsm"] {
        assert!(
            !collector.join(name).exists(),
            "current tree has no {name} collector module"
        );
    }
}

#[test]
fn sibling_sdd_suites_stay_registered() {
    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        cargo.contains("sdd_iso27001_assurance_target")
            && cargo.contains("sdd_iam_catalog_target")
            && cargo.contains("sdd_canonical_assurance_catalog_target"),
        "sibling SDD suites stay registered"
    );
}
