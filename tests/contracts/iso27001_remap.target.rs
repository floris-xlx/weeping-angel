//! Target suite for ISO 27001:2022 remapping onto the Canonical Assurance
//! Catalog (Prompt 12 / `docs/sdd/iso-27001-canonical-remap.md` §4).
//!
//! Encodes ISO-R-001…020, golden scenarios 1–10, and architecture-boundary
//! asserts. Must stay RED on current sliver HEAD (pack-local `access.*`
//! targets, ISO serialize special-case, no catalog digest, SoA booleans,
//! loader rejects EvidenceFor/SupersetOf/SubsetOf). Do not implement the
//! remap in this file and do not weaken assertions to match today's slivers.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance::readiness::project_readiness;
use weeping_angel_assurance::{AssessmentReport, AssessmentRun, project_soa};
use weeping_angel_assurance_ir::crosswalk::ComplianceGraph;
use weeping_angel_assurance_ir::{
    AssessmentId, ControlId, ControlTestId, Mapping, MappingCompleteness, MappingRelation,
    RequirementId,
};
use weeping_angel_canonical_catalog::CanonicalCatalog;
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSelector, EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType,
};
use weeping_angel_framework::pack::PackError;
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget, compile_framework,
    load_framework_pack, load_framework_pack_from, validate_framework_pack,
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

fn iso_pack_dir() -> PathBuf {
    manifest_dir().join("frameworks/iso-27001/2022")
}

fn catalog_v1() -> PathBuf {
    manifest_dir().join("catalog/canonical/v1")
}

fn load_catalog() -> CanonicalCatalog {
    CanonicalCatalog::load(catalog_v1()).expect("canonical catalog v1 must load")
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: weeping_angel_assurance_ir::FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn load_iso_pack() -> weeping_angel_framework::LoadedPack {
    load_framework_pack("iso-27001", "2022").expect("ISO pack must load")
}

const RETIRED_SLIVERS: &[&str] = &[
    "access.mfa.privileged",
    "access.least-privilege",
    "access.periodic-review",
    "source.branch-protection",
    "source.required-review",
    "source.code-ownership",
    "source.security-scanning",
    "source.commit-signing",
    "vulnerability.remediation",
    "logging.security-events",
    "logging.audit-trail",
    "incident.response-process",
    "backup.recovery-testing",
    "encryption.data-at-rest",
    "encryption.data-in-transit",
    "supplier.security-assessment",
    "personnel.access-termination",
    "asset.inventory",
    "change.approval",
    "security.headers",
    "security.tls",
    "security.secret-exposure",
];

const REQUIRED_IDENTITY_REMAPS: &[(&str, &[&str])] = &[
    (
        "iso27001:a.8.5",
        &[
            "control.identity.privileged-mfa",
            "control.identity.mfa",
            "control.identity.strong-authentication-policy",
        ],
    ),
    (
        "iso27001:a.8.2",
        &[
            "control.identity.privileged-access-minimization",
            "control.identity.least-privilege",
        ],
    ),
    ("iso27001:a.8.3", &["control.identity.least-privilege"]),
    ("iso27001:a.5.15", &["control.identity.least-privilege"]),
    (
        "iso27001:a.5.18",
        &[
            "control.identity.periodic-access-review",
            "control.identity.access-approval",
        ],
    ),
    (
        "iso27001:a.5.16",
        &[
            "control.identity.unique-user-identities",
            "control.identity.joiner-mover-leaver",
        ],
    ),
    (
        "iso27001:a.6.5",
        &[
            "control.identity.terminated-user-removal",
            "control.identity.access-revocation-timeliness",
        ],
    ),
];

const UNMAPPED_UNLESS_FAMILY_LANDS: &[(&str, &str)] = &[
    ("iso27001:a.8.8", "control.vulnerability."),
    ("iso27001:a.8.26", "control.source."),
    ("iso27001:a.5.9", "control."),
    ("iso27001:a.5.19", "control."),
    ("iso27001:a.5.24", "control."),
    ("iso27001:a.8.13", "control."),
    ("iso27001:a.8.15", "control."),
    ("iso27001:a.8.24", "control."),
    ("iso27001:a.8.32", "control."),
    ("iso27001:5.2", "control."),
    ("iso27001:a.5.1", "control."),
];

const GOVERNANCE_REQUIREMENTS: &[&str] = &[
    "iso27001:4.1",
    "iso27001:5.1",
    "iso27001:6.1",
    "iso27001:7.2",
    "iso27001:8.1",
    "iso27001:9.1",
    "iso27001:10.1",
];

const FORBIDDEN_CERTIFICATION: &[&str] = &[
    "iso 27001 certified",
    "iso 27001 compliant",
    "certification guaranteed",
    "audit passed",
];

const NEVER_FULLY_SATISFY: &[MappingRelation] = &[
    MappingRelation::PartiallySatisfies,
    MappingRelation::Supports,
    MappingRelation::Related,
    MappingRelation::EvidenceFor,
    MappingRelation::SubsetOf,
];

const FIVE_COVERAGE_KEYS: &[&str] = &[
    "automationCoverage",
    "evidenceCoverage",
    "subjectCoverage",
    "controlCoverage",
    "frameworkRequirementCoverage",
];

fn mapped_tos(pack: &weeping_angel_framework::LoadedPack, from: &str) -> BTreeSet<String> {
    pack.mappings
        .iter()
        .filter(|m| m.from_requirement().as_str() == from)
        .map(|m| m.to_control().as_str().to_string())
        .collect()
}

fn mapping_rows<'a>(pack: &'a weeping_angel_framework::LoadedPack, from: &str) -> Vec<&'a Mapping> {
    pack.mappings
        .iter()
        .filter(|m| m.from_requirement().as_str() == from)
        .collect()
}

fn catalog_has_prefix(catalog: &CanonicalCatalog, prefix: &str) -> bool {
    catalog
        .controls()
        .keys()
        .any(|id| id.starts_with(prefix) && id != "control.source.protected-branch")
}

fn relation_may_fully_satisfy(
    relation: MappingRelation,
    completeness: MappingCompleteness,
) -> bool {
    match relation {
        MappingRelation::Equivalent | MappingRelation::Satisfies | MappingRelation::SupersetOf => {
            completeness == MappingCompleteness::Full
        }
        MappingRelation::PartiallySatisfies
        | MappingRelation::Supports
        | MappingRelation::Related
        | MappingRelation::EvidenceFor
        | MappingRelation::SubsetOf => false,
    }
}

fn synthetic_result(control: &str, test: &str, effectiveness: Effectiveness) -> ControlTestResult {
    ControlTestResult {
        test_id: ControlTestId::new(test),
        control_id: ControlId::new(control),
        effectiveness,
        rationale: format!("iso remap target synthetic ({effectiveness:?})"),
        evidence_refs: Vec::new(),
        missing_evidence: Vec::new(),
        checked_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        test_version: "1".into(),
        input_digest: "iso-remap-target".into(),
        duration: None,
        status: Some(effectiveness),
        reason: None,
        population: None,
    }
}

fn fresh_context() -> AssessmentContext {
    AssessmentContext {
        now: Utc.with_ymd_and_hms(2026, 8, 18, 12, 30, 0).unwrap(),
        max_age: Duration::from_secs(24 * 3600),
    }
}

fn seal(
    evidence_type: &str,
    asset: &str,
    facts: &[(&str, &str)],
    hours_ago: i64,
) -> EvidenceEnvelope {
    let at =
        Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap() - chrono::Duration::hours(hours_ago);
    let mut obs = EvidenceObservation::new(EvidenceType::new(evidence_type));
    for (k, v) in facts {
        obs = obs.with_fact(*k, *v);
    }
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.iso-remap-target".into(),
            collected_at: at,
            scope: "target".into(),
            asset: weeping_angel_assurance_ir::AssetId::new(asset),
        },
    )
    .unwrap()
}

fn privileged_mfa_expr() -> TestExpr {
    TestExpr::CoverageAtLeast {
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
    }
}

fn evaluate_privileged_mfa(set: &EvidenceSet) -> ControlTestResult {
    let compiled = CompiledControlTest::builder()
        .id(ControlTestId::new("test.identity.privileged-mfa-enabled"))
        .control_id(ControlId::new("control.identity.privileged-mfa"))
        .kind(ControlTestKind::Automated)
        .expr(privileged_mfa_expr())
        .build();
    evaluate(&compiled, set, &fresh_context())
}

fn compile_iso(
    pack: &weeping_angel_framework::LoadedPack,
) -> weeping_angel_framework::CompiledFramework {
    compile_framework(
        &weeping_angel_framework::assessment_from_pack(pack, &iso_target()),
        &iso_target(),
    )
    .expect("ISO pack must compile")
}

fn pack_and_product_text() -> String {
    let mut chunks = vec![
        fs::read_to_string(iso_pack_dir().join("manifest.toml")).unwrap_or_default(),
        fs::read_to_string(iso_pack_dir().join("requirements.toml")).unwrap_or_default(),
        fs::read_to_string(iso_pack_dir().join("mappings.toml")).unwrap_or_default(),
        fs::read_to_string(iso_pack_dir().join("applicability.toml")).unwrap_or_default(),
        fs::read_to_string(iso_pack_dir().join("metadata.toml")).unwrap_or_default(),
        crate_sources_joined("weeping-angel-assurance"),
        crate_sources_joined("weeping-angel-framework"),
    ];
    chunks.push(crate_sources_joined("weeping-angel-collector"));
    chunks.join("\n")
}

fn serialize_empty_iso_report() -> Value {
    let report = AssessmentReport {
        assessment_id: AssessmentId::new("assess-iso-remap-target"),
        profile: "iso-27001".into(),
        digest: "digest".into(),
        results: Vec::new(),
        evidence_count: 0,
        ..Default::default()
    };
    serde_json::to_value(&report).expect("serialize AssessmentReport")
}

fn coverage_is_inspectable(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            map.contains_key("covered")
                || map.contains_key("total")
                || map.contains_key("count")
                || map.contains_key("numerator")
        }
        Value::Number(_) => true,
        Value::String(s) => !s.trim().is_empty() && !s.trim().ends_with('%'),
        _ => false,
    }
}

fn write_relation_pack(dir: &Path, relation: &str, to: &str, extra: &str) {
    fs::write(
        dir.join("manifest.toml"),
        r#"schema = "weeping-angel/framework-pack/v1"

[framework]
id = "iso-27001"
version = "2022"
content_mode = "StructuralOnly"
"#,
    )
    .unwrap();
    fs::write(
        dir.join("requirements.toml"),
        r#"schema = "weeping-angel/framework-pack/v1"

[[requirement]]
id = "iso27001:a.8.5"
title = "Authentication (structural)"
kind = "annex"
"#,
    )
    .unwrap();
    fs::write(
        dir.join("metadata.toml"),
        r#"schema = "weeping-angel/framework-pack/v1"
"#,
    )
    .unwrap();
    fs::write(
        dir.join("mappings.toml"),
        format!(
            r#"schema = "weeping-angel/framework-pack/v1"

[[mapping]]
from = "iso27001:a.8.5"
to = "{to}"
direction = "forward"
completeness = "partial"
relation = "{relation}"
rationale = "target-suite mapping row"
{extra}
"#
        ),
    )
    .unwrap();
}

#[test]
fn iso_r_000_dual_suite_remains_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_iso27001_remap_baseline")
            && toml.contains("sdd_iso27001_remap_target")
            && toml.contains("tests/sdd/iso27001_remap.baseline.rs")
            && toml.contains("tests/sdd/iso27001_remap.target.rs"),
        "ISO-R: dual-suite sdd_iso27001_remap_{{baseline,target}} must stay registered"
    );
    assert!(
        toml.contains("sdd_iso27001_assurance_target")
            && toml.contains("sdd_iso27001_assurance_baseline"),
        "ISO-R: must not reuse or delete the MVP iso27001_assurance dual-suite"
    );
    assert!(
        !toml.contains("name = \"sdd_iso27001_assurance_target\"")
            || toml.contains("path = \"tests/sdd/iso27001_assurance.target.rs\""),
        "ISO-R: remap suite files are not iso27001_assurance.*"
    );
}

#[test]
fn iso_r_001_mappings_target_existing_catalog_identity_ids() {
    let catalog = load_catalog();
    let pack = load_iso_pack();
    let mappings =
        fs::read_to_string(iso_pack_dir().join("mappings.toml")).expect("read mappings.toml");

    for sliver in RETIRED_SLIVERS {
        assert!(
            !mappings.contains(&format!("to = \"{sliver}\"")),
            "ISO-R-001: mapping must not target retired pack sliver `{sliver}`"
        );
    }
    assert!(
        mappings.contains("control.identity.privileged-mfa"),
        "ISO-R-001: A.8.5 must remap onto control.identity.privileged-mfa"
    );

    for (from, tos) in REQUIRED_IDENTITY_REMAPS {
        let got = mapped_tos(&pack, from);
        for to in *tos {
            catalog
                .control(to)
                .unwrap_or_else(|_| panic!("ISO-R-001: catalog must contain `{to}`"));
            assert!(
                got.contains(*to),
                "ISO-R-001: `{from}` must map to `{to}` (have {got:?})"
            );
        }
    }

    let compiled = compile_iso(&pack);
    let compiled_ids: BTreeSet<_> = compiled.controls.iter().map(|c| c.id().as_str()).collect();
    assert!(
        compiled_ids.contains("control.identity.privileged-mfa"),
        "ISO-R-001: compiled ISO assessment must resolve control.identity.privileged-mfa"
    );
    assert!(
        !compiled_ids.contains("access.mfa.privileged"),
        "ISO-R-001: compiled ISO assessment must not own access.mfa.privileged"
    );
}

#[test]
fn iso_r_002_relations_are_honest_and_equivalent_is_not_convenience() {
    let pack = load_iso_pack();
    assert!(
        !pack.mappings.is_empty(),
        "ISO-R-002: remapped pack must still carry material mapping rows"
    );

    let mut saw_non_equivalent = false;
    for mapping in &pack.mappings {
        let rel = mapping.relation();
        if rel != MappingRelation::Equivalent {
            saw_non_equivalent = true;
        }
        if rel == MappingRelation::Equivalent {
            assert_eq!(
                mapping.completeness(),
                MappingCompleteness::Full,
                "ISO-R-002: Equivalent requires completeness full"
            );
            assert!(
                !mapping.rationale().trim().is_empty(),
                "ISO-R-002: Equivalent rows need an explicit rationale"
            );
            assert!(
                mapping.provenance().reference.is_some() || mapping.provenance().author.is_some(),
                "ISO-R-002: Equivalent rows need provenance, not a convenience default"
            );
        }
        if mapping
            .from_requirement()
            .as_str()
            .starts_with("iso27001:a.8.")
            && mapping
                .to_control()
                .as_str()
                .starts_with("control.identity.")
        {
            assert_ne!(
                rel,
                MappingRelation::Equivalent,
                "ISO-R-002: IAM controls are slices of Annex A, not Equivalent"
            );
        }
    }
    assert!(
        saw_non_equivalent,
        "ISO-R-002: at least one material mapping must be a non-Equivalent relation"
    );
    assert!(
        pack.mappings.iter().any(|m| {
            m.to_control().as_str().starts_with("control.identity.")
                && m.relation() != MappingRelation::Equivalent
        }),
        "ISO-R-002: identity remaps must exist and must not use Equivalent as a convenience"
    );
}

#[test]
fn iso_r_003_partial_supports_related_evidence_for_subset_cannot_fully_satisfy() {
    let pack = load_iso_pack();
    let compiled = compile_iso(&pack);
    let mut results = Vec::new();
    for control in &compiled.controls {
        results.push(synthetic_result(
            control.id().as_str(),
            &format!("test.{}", control.id().as_str()),
            Effectiveness::Effective,
        ));
    }
    let snapshot = project_readiness(
        &compiled,
        &results,
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-iso-r-003"),
    );

    for mapping in &pack.mappings {
        if !NEVER_FULLY_SATISFY.contains(&mapping.relation())
            && relation_may_fully_satisfy(mapping.relation(), mapping.completeness())
        {
            continue;
        }
        let req = snapshot
            .requirements
            .iter()
            .find(|r| r.id.as_str() == mapping.from_requirement().as_str())
            .unwrap_or_else(|| {
                panic!(
                    "ISO-R-003: readiness missing requirement {}",
                    mapping.from_requirement()
                )
            });
        assert_ne!(
            req.status,
            "effective",
            "ISO-R-003: {:?} mapping {} → {} must not fully satisfy (status={})",
            mapping.relation(),
            mapping.from_requirement(),
            mapping.to_control(),
            req.status
        );
        assert_ne!(
            req.status.to_ascii_lowercase(),
            "fully satisfied",
            "ISO-R-003: partial/support/related/evidence-for/subset cannot become equivalence"
        );
    }

    let a85 = snapshot
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("ISO-R-003: A.8.5 must appear in readiness");
    assert!(
        a85.mapped_controls
            .iter()
            .any(|c| c.as_str() == "control.identity.privileged-mfa"),
        "ISO-R-003: readiness must walk the mapping graph onto control.identity.privileged-mfa, not every compiled control"
    );
    assert!(
        !a85.mapped_controls
            .iter()
            .any(|c| c.as_str() == "access.mfa.privileged"),
        "ISO-R-003: retired sliver must not be attached to A.8.5"
    );

    let r41 = snapshot
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:4.1")
        .expect("ISO-R-003: 4.1 must appear in readiness");
    assert!(
        r41.mapped_controls.is_empty()
            || r41
                .mapped_controls
                .iter()
                .all(|c| c.as_str().starts_with("control.governance.")
                    || c.as_str().contains(".governance.")),
        "ISO-R-003: unmapped/governance 4.1 must not inherit every compiled control (have {:?})",
        r41.mapped_controls
    );
    assert_ne!(
        r41.status, "effective",
        "ISO-R-003: 4.1 must not become effective from unrelated technical tests"
    );

    let mut graph = ComplianceGraph::new();
    graph.link(
        RequirementId::new("iso27001:a.8.5"),
        RequirementId::new("iso27001:a.8.2"),
        weeping_angel_assurance_ir::MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    graph.link(
        RequirementId::new("iso27001:a.8.2"),
        RequirementId::new("iso27001:a.8.5"),
        weeping_angel_assurance_ir::MappingDirection::Forward,
        MappingCompleteness::Partial,
    );
    assert!(
        !graph.equivalent(
            &RequirementId::new("iso27001:a.8.5"),
            &RequirementId::new("iso27001:a.8.2")
        ),
        "ISO-R-003: ComplianceGraph::equivalent must stay fail-closed on partial paths"
    );
}

#[test]
fn iso_r_004_mapping_targets_resolve_in_canonical_catalog() {
    let catalog = load_catalog();
    let pack = load_iso_pack();
    for mapping in &pack.mappings {
        let to = mapping.to_control().as_str();
        catalog.control(to).unwrap_or_else(|_| {
            panic!("ISO-R-004: mapping to `{to}` must exist in CanonicalCatalog")
        });
        assert!(
            to.starts_with("control."),
            "ISO-R-004: mapping to `{to}` must be a catalog control id"
        );
    }

    let tmp = tempfile::tempdir().expect("temp pack");
    write_relation_pack(
        tmp.path(),
        "PartiallySatisfies",
        "control.identity.does-not-exist",
        "",
    );
    let unknown = load_framework_pack_from(tmp.path());
    assert!(
        unknown.is_err(),
        "ISO-R-004: unknown catalog id must fail pack load/validate closed, got {unknown:?}"
    );
}

#[test]
fn iso_r_005_metadata_is_not_a_competing_control_library() {
    let metadata =
        fs::read_to_string(iso_pack_dir().join("metadata.toml")).expect("read metadata.toml");
    for sliver in RETIRED_SLIVERS {
        assert!(
            !metadata.contains(&format!("id = \"{sliver}\"")),
            "ISO-R-005: metadata.toml must not declare competing sliver `{sliver}`"
        );
    }
    assert!(
        !metadata.contains("id = \"test.access.mfa.privileged\""),
        "ISO-R-005: pack-local privileged MFA test must be gone"
    );
    assert!(
        !metadata.contains("source.admin.permissions"),
        "ISO-R-005: privileged MFA must not require GitHub admin-permission existence"
    );

    let pack = load_iso_pack();
    let control_ids: BTreeSet<_> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    for sliver in RETIRED_SLIVERS {
        assert!(
            !control_ids.contains(sliver),
            "ISO-R-005: loaded pack must not expose sliver `{sliver}`"
        );
    }
    assert!(
        !control_ids.contains("access.mfa.privileged")
            || !control_ids.contains("control.identity.privileged-mfa"),
        "ISO-R-005: must not keep two public IDs for privileged MFA"
    );
    assert!(
        !control_ids.contains("access.mfa.privileged"),
        "ISO-R-005: access.mfa.privileged must not remain as a second library"
    );
}

#[test]
fn iso_r_006_loader_accepts_evidence_for_superset_of_subset_of() {
    let pack_src =
        fs::read_to_string(crate_src("weeping-angel-framework").join("pack.rs")).unwrap();
    let relation_match = pack_src
        .split("let relation = match row.relation.as_str()")
        .nth(1)
        .expect("ISO-R-006: pack loader must still match mapping relations");
    let relation_match = relation_match
        .split("let direction = match")
        .next()
        .unwrap();
    for accepted in [
        "Equivalent",
        "Satisfies",
        "PartiallySatisfies",
        "Supports",
        "Related",
        "EvidenceFor",
        "SupersetOf",
        "SubsetOf",
    ] {
        assert!(
            relation_match.contains(&format!("\"{accepted}\"")),
            "ISO-R-006: loader must accept relation {accepted}"
        );
    }

    let tmp = tempfile::tempdir().expect("temp pack");
    for rel in ["EvidenceFor", "SupersetOf", "SubsetOf"] {
        write_relation_pack(tmp.path(), rel, "control.identity.privileged-mfa", "");
        let loaded = load_framework_pack_from(tmp.path());
        assert!(
            loaded.is_ok(),
            "ISO-R-006: relation {rel} must load, got {loaded:?}"
        );
        let loaded = loaded.unwrap();
        assert_eq!(
            format!("{:?}", loaded.mappings[0].relation()),
            rel,
            "ISO-R-006: loaded relation must round-trip as {rel}"
        );
    }

    write_relation_pack(
        tmp.path(),
        "NotARelation",
        "control.identity.privileged-mfa",
        "",
    );
    match load_framework_pack_from(tmp.path()) {
        Err(PackError::UnsupportedRelation(rel)) => {
            assert_eq!(rel, "NotARelation");
        }
        other => panic!("ISO-R-006: unknown relation must still be rejected, got {other:?}"),
    }
}

#[test]
fn iso_r_007_material_mappings_carry_rationale_provenance_and_version_constraints() {
    let mappings =
        fs::read_to_string(iso_pack_dir().join("mappings.toml")).expect("read mappings.toml");
    assert!(
        mappings.contains("provenance") && mappings.contains("valid_for"),
        "ISO-R-007: mappings.toml must serialize provenance and valid_for, not rely on IR defaults"
    );

    let pack = load_iso_pack();
    for mapping in &pack.mappings {
        assert!(
            !mapping.rationale().trim().is_empty(),
            "ISO-R-007: mapping {} → {} needs a non-empty rationale",
            mapping.from_requirement(),
            mapping.to_control()
        );
        assert!(
            !mapping.valid_for().is_unconstrained(),
            "ISO-R-007: ISO 27001:2022 mappings must carry an edition constraint, not unconstrained valid_for"
        );
        assert!(
            mapping
                .valid_for()
                .contains(&weeping_angel_assurance_ir::FrameworkVersion::new("2022")),
            "ISO-R-007: valid_for must include edition 2022"
        );
        let source = mapping.provenance().source;
        assert!(
            matches!(
                source,
                weeping_angel_assurance_ir::MappingSource::BuiltIn
                    | weeping_angel_assurance_ir::MappingSource::UserDefined
                    | weeping_angel_assurance_ir::MappingSource::LicensedFrameworkContent
            ),
            "ISO-R-007: provenance.source must be explicit, got {source:?}"
        );
        assert!(
            mapping.provenance().reference.is_some()
                || mapping.provenance().author.is_some()
                || !matches!(
                    mapping.provenance().source,
                    weeping_angel_assurance_ir::MappingSource::BuiltIn
                )
                || mappings.contains(&format!("to = \"{}\"", mapping.to_control().as_str())),
            "ISO-R-007: loader must populate provenance from the pack file"
        );
    }

    let pack_src =
        fs::read_to_string(crate_src("weeping-angel-framework").join("pack.rs")).unwrap();
    let mapping_row = pack_src
        .split("struct MappingRow")
        .nth(1)
        .expect("MappingRow")
        .split('}')
        .next()
        .unwrap();
    assert!(
        mapping_row.contains("provenance") && mapping_row.contains("valid_for"),
        "ISO-R-007: MappingRow must deserialize provenance and valid_for"
    );
}

#[test]
fn iso_r_008_generic_serialize_and_assess_have_no_iso_pack_literal() {
    let assurance = crate_sources_joined("weeping-angel-assurance");
    let serialize_window = assurance
        .split("impl Serialize for AssessmentReport")
        .nth(1)
        .unwrap_or("");
    assert!(
        !serialize_window.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "ISO-R-008: AssessmentReport serialize must not hard-load the ISO pack"
    );
    assert!(
        !assurance.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "ISO-R-008: generic assess/serialize must not contain an ISO pack-load literal"
    );

    let framework = crate_sources_joined("weeping-angel-framework");
    assert!(
        !framework.contains(
            "target.profile == FrameworkProfile::Iso27001 && target.version.as_str() == \"2022\""
        ) && !framework
            .contains("FrameworkProfile::Iso27001 && target.version.as_str() == \"2022\""),
        "ISO-R-008: normalize must key off target identity, not an Iso27001+2022 branch"
    );

    let iso = load_iso_pack();
    let other = AssessmentReport {
        assessment_id: AssessmentId::new("assess-not-iso"),
        profile: "soc-2".into(),
        digest: "other".into(),
        results: Vec::new(),
        evidence_count: 0,
        ..Default::default()
    };
    let value = serde_json::to_value(&other).expect("serialize non-ISO report");
    assert_ne!(
        value.get("frameworkPackDigest").and_then(Value::as_str),
        Some(iso.digest.as_str()),
        "ISO-R-008: generic serialize must not pin the live ISO pack digest onto a non-ISO report"
    );
}

#[test]
fn iso_r_009_soa_uses_generic_three_state_applicability() {
    let soa_src = fs::read_to_string(crate_src("weeping-angel-assurance").join("soa.rs")).unwrap();
    assert!(
        soa_src.contains("Unresolved")
            || soa_src.contains("ManualDeterminationRequired")
            || soa_src.contains("NotApplicable"),
        "ISO-R-009: SoA projection must consume generic three-state applicability"
    );
    assert!(
        !soa_src.contains("as_bool")
            || soa_src.contains("Applicability") && soa_src.contains("NotApplicable"),
        "ISO-R-009: SoA must not copy applicability.toml booleans as the public type"
    );

    let soa = project_soa("iso-27001", "2022");
    let json = serde_json::to_value(&soa).expect("serialize SoA");
    let entries = json
        .get("entries")
        .and_then(Value::as_array)
        .expect("SoA entries");
    assert!(
        !entries.is_empty(),
        "ISO-R-009: Annex A / SoA-oriented output must not be empty"
    );
    for entry in entries {
        let status = entry
            .get("applicability")
            .or_else(|| entry.get("applicabilityState"))
            .or_else(|| entry.get("decision"))
            .or_else(|| entry.get("status"))
            .cloned()
            .or_else(|| {
                entry
                    .get("applicable")
                    .and_then(|v| v.as_str().map(|s| Value::String(s.to_string())))
            });
        let status_str = status
            .as_ref()
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_ascii_lowercase();
        assert!(
            matches!(
                status_str.as_str(),
                "applicable"
                    | "notapplicable"
                    | "not-applicable"
                    | "unresolved"
                    | "manualdeterminationrequired"
                    | "manual_determination_required"
            ),
            "ISO-R-009: SoA entry must be three-state, not a raw bool (entry={entry})"
        );
        if status_str.contains("notapplicable") || status_str.contains("not-applicable") {
            let rationale = entry
                .get("applicabilityRationale")
                .or_else(|| entry.get("rationale"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            assert!(
                !rationale.contains("no evidence")
                    && !rationale.contains("missing evidence")
                    && !rationale.contains("insufficient evidence"),
                "ISO-R-009: NotApplicable must be justified by context, not missing evidence"
            );
        }
    }
}

#[test]
fn iso_r_010_lineage_pins_pack_digest_and_catalog_digest() {
    let catalog = load_catalog();
    let catalog_digest = catalog.digest().expect("catalog digest").to_string();
    let pack = load_iso_pack();

    let report = serialize_empty_iso_report();
    assert_eq!(
        report.get("frameworkPackDigest").and_then(Value::as_str),
        Some(pack.digest.as_str()),
        "ISO-R-010: ISO report lineage must pin the assessed pack digest without a serialize-time ISO load"
    );
    assert_eq!(
        report.get("catalogDigest").and_then(Value::as_str),
        Some(catalog_digest.as_str()),
        "ISO-R-010: AssessmentReport must pin catalogDigest"
    );

    let snapshot_src =
        fs::read_to_string(crate_src("weeping-angel-assurance").join("snapshot.rs")).unwrap();
    assert!(
        snapshot_src.contains("catalog_digest"),
        "ISO-R-010: AssessmentRun must have catalog_digest"
    );

    let run = AssessmentRun {
        id: AssessmentId::new("run-iso-remap-target"),
        framework: "iso-27001".into(),
        framework_pack_digest: pack.digest.0.clone(),
        assessment_definition_digest: "def".into(),
        started_at: "2026-08-18T12:00:00Z".into(),
        completed_at: "2026-08-18T12:00:00Z".into(),
        scope: "assess".into(),
        collector_runs: Vec::new(),
        evidence_snapshot_digest: "ev".into(),
        result_digest: "res".into(),
        status: "completed".into(),
        ..Default::default()
    };
    let run_json = serde_json::to_value(&run).expect("serialize AssessmentRun");
    assert_eq!(
        run_json.get("frameworkPackDigest").and_then(Value::as_str),
        Some(pack.digest.as_str())
    );
    assert!(
        run_json
            .get("catalogDigest")
            .and_then(Value::as_str)
            .is_some(),
        "ISO-R-010: serialized AssessmentRun must emit catalogDigest (got {run_json})"
    );

    let compiled = compile_iso(&pack);
    let readiness = project_readiness(
        &compiled,
        &[],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-iso-r-010"),
    );
    let ready_json = serde_json::to_value(&readiness).unwrap();
    assert_eq!(
        ready_json
            .get("frameworkPackDigest")
            .and_then(Value::as_str),
        Some(pack.digest.as_str())
    );
    assert!(
        ready_json.get("catalogDigest").is_some(),
        "ISO-R-010: readiness snapshot must pin catalogDigest"
    );
}

#[test]
fn iso_r_011_g01_technically_strong_with_governance_stays_partial_where_mapped_partial() {
    let pack = load_iso_pack();
    let compiled = compile_iso(&pack);
    let mut results = Vec::new();
    for control in &compiled.controls {
        let id = control.id().as_str();
        results.push(synthetic_result(
            id,
            &format!("test.{id}"),
            Effectiveness::Effective,
        ));
    }
    results.push(synthetic_result(
        "control.identity.privileged-mfa",
        "test.identity.privileged-mfa-enabled",
        Effectiveness::Effective,
    ));
    let snapshot = project_readiness(
        &compiled,
        &results,
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-golden-1"),
    );
    let a85 = snapshot
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("golden-1: A.8.5");
    assert_eq!(
        a85.status, "partially covered",
        "golden-1: A.8.5 identity remaps are Partial/Supports and must stay partially covered even when tests are Effective"
    );
    assert!(
        a85.mapped_controls
            .iter()
            .any(|c| c.as_str() == "control.identity.privileged-mfa")
    );
}

#[test]
fn iso_r_011_g02_missing_manual_governance_does_not_fully_satisfy() {
    let pack = load_iso_pack();
    let compiled = compile_iso(&pack);
    let mut results = vec![synthetic_result(
        "control.identity.privileged-mfa",
        "test.identity.privileged-mfa-enabled",
        Effectiveness::Effective,
    )];
    for req in GOVERNANCE_REQUIREMENTS {
        let _ = req;
        results.push(synthetic_result(
            "control.identity.mfa",
            "test.identity.mfa-enabled",
            Effectiveness::Effective,
        ));
    }
    results.push(synthetic_result(
        "control.identity.strong-authentication-policy",
        "test.identity.strong-authentication-policy",
        Effectiveness::InsufficientEvidence,
    ));
    let snapshot = project_readiness(
        &compiled,
        &results,
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-golden-2"),
    );
    for req in snapshot.requirements.iter().filter(|r| {
        r.id.as_str().starts_with("iso27001:4.")
            || r.id.as_str().starts_with("iso27001:5.")
            || r.id.as_str() == "iso27001:a.8.5"
    }) {
        assert_ne!(
            req.status, "effective",
            "golden-2: {} must not fully satisfy when governance/manual evidence is missing or mappings are partial",
            req.id
        );
    }
    let a85 = snapshot
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("golden-2 A.8.5");
    assert!(
        a85.status == "partially covered"
            || a85.status == "insufficient evidence"
            || a85.status == "manual review required",
        "golden-2: A.8.5 status must stay partial/weaker, got {}",
        a85.status
    );
    assert!(
        a85.mapped_controls
            .iter()
            .any(|c| c.as_str() == "control.identity.privileged-mfa"),
        "golden-2: A.8.5 must trace to control.identity.privileged-mfa"
    );
}

#[test]
fn iso_r_011_g03_partial_population_is_insufficient_evidence() {
    let mut set = EvidenceSet::new();
    set.insert(seal(
        "evidence.identity.inventory",
        "user:alice",
        &[
            ("subject_id", "user:alice"),
            ("account_kind", "user"),
            ("unique_key", "user:alice"),
            ("authoritative", "false"),
        ],
        1,
    ));
    set.insert(seal(
        "evidence.identity.mfa-status",
        "user:alice",
        &[("subject_id", "user:alice"), ("mfa_enabled", "true")],
        1,
    ));
    let compiled_mfa = CompiledControlTest::builder()
        .id(ControlTestId::new("test.identity.mfa-enabled"))
        .control_id(ControlId::new("control.identity.mfa"))
        .kind(ControlTestKind::Automated)
        .expr(TestExpr::CoverageAtLeast {
            selector: SubjectSelector {
                kind: Some("user".into()),
                id: None,
            },
            evidence: EvidenceSelector {
                evidence_type: EvidenceType::new("evidence.identity.mfa-status"),
                subject_selector: SubjectSelector {
                    kind: Some("user".into()),
                    id: None,
                },
                field: Some("mfa_enabled".into()),
                freshness: None,
            },
            percentage: "100".into(),
        })
        .build();
    let result = evaluate(&compiled_mfa, &set, &fresh_context());
    assert_eq!(
        result.effectiveness,
        Effectiveness::InsufficientEvidence,
        "golden-3: partial privileged population must be InsufficientEvidence, got {:?}",
        result.effectiveness
    );
    assert_ne!(result.effectiveness, Effectiveness::Effective);
    assert_ne!(result.effectiveness, Effectiveness::Ineffective);

    let pack = load_iso_pack();
    let compiled = compile_iso(&pack);
    let snapshot = project_readiness(
        &compiled,
        &[result],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-golden-3"),
    );
    let a85 = snapshot
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("golden-3 A.8.5");
    assert_ne!(a85.status, "effective");
    assert!(
        a85.mapped_controls
            .iter()
            .any(|c| c.as_str() == "control.identity.privileged-mfa")
    );
}

#[test]
fn iso_r_011_g04_privileged_mfa_failure_maps_through_canonical_iam() {
    let pack = load_iso_pack();
    assert!(
        mapped_tos(&pack, "iso27001:a.8.5").contains("control.identity.privileged-mfa"),
        "golden-4: iso27001:a.8.5 → control.identity.privileged-mfa"
    );
    let compiled = compile_iso(&pack);
    assert!(
        compiled
            .tests
            .iter()
            .any(|t| t.id.as_str() == "test.identity.privileged-mfa-enabled"
                && t.control_id.as_str() == "control.identity.privileged-mfa"),
        "golden-4: compiled tests must include test.identity.privileged-mfa-enabled"
    );
    assert!(
        compiled
            .tests
            .iter()
            .all(|t| t.id.as_str() != "test.access.mfa.privileged"),
        "golden-4: pack sliver test.access.mfa.privileged must be gone"
    );

    let mut set = EvidenceSet::new();
    set.insert(seal(
        "evidence.identity.inventory",
        "org:healthy",
        &[
            ("population_id", "org:healthy"),
            ("authoritative", "true"),
            ("account_kind", "organization"),
        ],
        1,
    ));
    for id in ["user:alice", "user:admin"] {
        set.insert(seal(
            "evidence.identity.inventory",
            id,
            &[
                ("subject_id", id),
                ("account_kind", "user"),
                ("unique_key", id),
                ("authoritative", "true"),
            ],
            1,
        ));
    }
    set.insert(seal(
        "evidence.identity.privileged-membership",
        "user:admin",
        &[
            ("subject_id", "user:admin"),
            ("privileged", "true"),
            ("roles", "admin"),
            ("membership_observed_at", "2026-08-18T12:00:00Z"),
        ],
        1,
    ));
    set.insert(seal(
        "evidence.identity.mfa-status",
        "user:alice",
        &[("subject_id", "user:alice"), ("mfa_enabled", "true")],
        1,
    ));
    set.insert(seal(
        "evidence.identity.mfa-status",
        "user:admin",
        &[("subject_id", "user:admin"), ("mfa_enabled", "false")],
        1,
    ));
    let result = evaluate_privileged_mfa(&set);
    assert_eq!(
        result.effectiveness,
        Effectiveness::Ineffective,
        "golden-4: privileged user without MFA must be Ineffective"
    );
    let json = serde_json::to_value(&result).unwrap();
    let named = json.to_string();
    assert!(
        named.contains("user:admin") || named.contains("admin"),
        "golden-4: Ineffective result must name the subject, got {json}"
    );

    let snapshot = project_readiness(
        &compiled,
        &[result],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-golden-4"),
    );
    let a85 = snapshot
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("golden-4 A.8.5");
    assert_eq!(a85.status, "ineffective");
    assert!(
        a85.mapped_controls
            .iter()
            .any(|c| c.as_str() == "control.identity.privileged-mfa")
    );
}

#[test]
fn iso_r_011_g05_stale_evidence_is_stale_not_ineffective_as_missing() {
    let pack = load_iso_pack();
    let compiled = compile_iso(&pack);
    let result = synthetic_result(
        "control.identity.privileged-mfa",
        "test.identity.privileged-mfa-enabled",
        Effectiveness::StaleEvidence,
    );
    assert_eq!(result.effectiveness, Effectiveness::StaleEvidence);
    let snapshot = project_readiness(
        &compiled,
        &[result],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-golden-5"),
    );
    let a85 = snapshot
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.5")
        .expect("golden-5 A.8.5");
    assert_ne!(a85.status, "ineffective");
    assert!(
        a85.status.contains("stale")
            || a85.status == "insufficient evidence"
            || snapshot.insufficient_evidence >= 1,
        "golden-5: stale evidence must not be treated as Ineffective-as-missing (status={})",
        a85.status
    );
    assert!(
        a85.mapped_controls
            .iter()
            .any(|c| c.as_str() == "control.identity.privileged-mfa"),
        "golden-5: stale evidence for privileged MFA must trace through the catalog control"
    );
}

#[test]
fn iso_r_011_g06_approved_exception_is_bound_and_expired_does_not_pass() {
    let pack = load_iso_pack();
    let compiled = compile_iso(&pack);
    let approved = synthetic_result(
        "control.identity.break-glass-access",
        "test.identity.break-glass-governed",
        Effectiveness::ExceptionApproved,
    );
    let snapshot = project_readiness(
        &compiled,
        &[approved],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-golden-6"),
    );
    assert!(
        snapshot
            .controls
            .iter()
            .any(|c| c.id.as_str() == "control.identity.break-glass-access"
                && c.effectiveness == Effectiveness::ExceptionApproved),
        "golden-6: approved unexpired exception must surface ExceptionApproved"
    );

    let expired = synthetic_result(
        "control.identity.break-glass-access",
        "test.identity.break-glass-governed",
        Effectiveness::Ineffective,
    );
    let expired_snap = project_readiness(
        &compiled,
        &[expired],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-golden-6-expired"),
    );
    assert!(
        expired_snap
            .controls
            .iter()
            .any(|c| c.effectiveness == Effectiveness::Ineffective),
        "golden-6: expired/revoked exception must not pass"
    );
    assert!(
        snapshot
            .requirements
            .iter()
            .any(|r| r.id.as_str() == "iso27001:a.8.5"
                && r.mapped_controls
                    .iter()
                    .any(|c| c.as_str().starts_with("control.identity."))),
        "golden-6: exception state must still sit on the ISO→catalog mapping graph"
    );
}

#[test]
fn iso_r_011_g07_applicability_driven_not_applicable_is_context_justified() {
    let soa = project_soa("iso-27001", "2022");
    let json = serde_json::to_value(&soa).unwrap();
    let entries = json
        .get("entries")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let na = entries.iter().find(|e| {
        let blob = e.to_string().to_ascii_lowercase();
        blob.contains("notapplicable")
            || blob.contains("not-applicable")
            || blob.contains("\"applicable\":false")
    });
    assert!(
        na.is_some(),
        "golden-7: SoA must be able to emit NotApplicable (got {json})"
    );
    let na = na.unwrap();
    let rationale = na
        .get("applicabilityRationale")
        .or_else(|| na.get("rationale"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    assert!(
        !rationale.is_empty(),
        "golden-7: NotApplicable needs a context rationale"
    );
    assert!(
        !rationale.contains("no evidence") && !rationale.contains("missing evidence"),
        "golden-7: NA must not be justified by missing evidence"
    );
}

#[test]
fn iso_r_011_g08_incomplete_org_context_stays_unresolved() {
    let soa_src = fs::read_to_string(crate_src("weeping-angel-assurance").join("soa.rs")).unwrap();
    assert!(
        soa_src.contains("Unresolved") || soa_src.contains("ManualDeterminationRequired"),
        "golden-8: incomplete org context must map to Unresolved / ManualDeterminationRequired"
    );
    let soa = project_soa("iso-27001", "2022");
    let json = serde_json::to_value(&soa).unwrap();
    let blob = json.to_string().to_ascii_lowercase();
    assert!(
        blob.contains("unresolved")
            || blob.contains("manualdeterminationrequired")
            || blob.contains("manual_determination_required")
            || blob.contains("manual determination"),
        "golden-8: SoA JSON must be able to represent unresolved applicability, got {json}"
    );
}

#[test]
fn iso_r_011_g09_historical_snapshot_pins_both_digests() {
    let catalog = load_catalog();
    let pack = load_iso_pack();
    let catalog_digest = catalog.digest().unwrap().to_string();
    let report = serialize_empty_iso_report();
    assert_eq!(
        report.get("frameworkPackDigest").and_then(Value::as_str),
        Some(pack.digest.as_str()),
        "golden-9: replay identity includes the pack digest"
    );
    assert_eq!(
        report.get("catalogDigest").and_then(Value::as_str),
        Some(catalog_digest.as_str()),
        "golden-9: replay identity includes the catalog digest; mismatch after file change must be detectable"
    );
}

#[test]
fn iso_r_011_g10_empty_scanner_findings_are_not_false_positive_effective() {
    let catalog = load_catalog();
    let pack = load_iso_pack();
    let vuln_landed = catalog_has_prefix(&catalog, "control.vulnerability.");
    let a88 = mapped_tos(&pack, "iso27001:a.8.8");
    if !vuln_landed {
        assert!(
            a88.is_empty(),
            "golden-10: unlanded vuln family must leave A.8.8 unmapped, not stubbed (have {a88:?})"
        );
        assert!(
            !a88.contains("vulnerability.remediation"),
            "golden-10: leftover vulnerability.remediation sliver cannot become Effective"
        );
    }
    let compiled = compile_iso(&pack);
    assert!(
        compiled
            .controls
            .iter()
            .all(|c| c.id().as_str() != "vulnerability.remediation"),
        "golden-10: pack sliver vulnerability.remediation must not compile as a control"
    );
    let empty = project_readiness(
        &compiled,
        &[],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-golden-10"),
    );
    if let Some(req) = empty
        .requirements
        .iter()
        .find(|r| r.id.as_str() == "iso27001:a.8.8")
    {
        assert_ne!(
            req.status, "effective",
            "golden-10: unknown coverage / empty findings must not be Effective"
        );
    }
}

#[test]
fn iso_r_012_architecture_boundaries() {
    let pack_rs = crate_sources_joined("weeping-angel-framework");
    for token in ["octocrab", "aws-sdk", "octorust", "reqwest::"] {
        assert!(
            !pack_rs.contains(token),
            "ISO-R-012: framework pack loader must not contain provider type `{token}`"
        );
    }
    let pack_text = [
        fs::read_to_string(iso_pack_dir().join("manifest.toml")).unwrap(),
        fs::read_to_string(iso_pack_dir().join("mappings.toml")).unwrap(),
        fs::read_to_string(iso_pack_dir().join("metadata.toml")).unwrap(),
    ]
    .join("\n");
    for token in ["octocrab", "aws-sdk-", "octorust"] {
        assert!(
            !pack_text.contains(token),
            "ISO-R-012: pack files must not embed provider types (`{token}`)"
        );
    }

    let collector = crate_sources_joined("weeping-angel-collector");
    assert!(
        !collector.contains("iso27001:"),
        "ISO-R-012: collectors must contain no ISO requirement IDs"
    );
    let control_test = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !control_test.contains("iso27001") && !control_test.contains("Iso27001"),
        "ISO-R-012: control-test runtime must contain no ISO branches"
    );

    let catalog = load_catalog();
    let pack = load_iso_pack();
    for mapping in &pack.mappings {
        catalog
            .control(mapping.to_control().as_str())
            .expect("ISO-R-012: every mapping to must resolve via CanonicalCatalog::control");
    }

    let serialize = crate_sources_joined("weeping-angel-assurance");
    assert!(
        !serialize.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "ISO-R-012: generic serialize/assess has no ISO pack-load literal"
    );
}

#[test]
fn iso_r_013_no_forbidden_certification_language() {
    let blob = pack_and_product_text().to_ascii_lowercase();
    for phrase in FORBIDDEN_CERTIFICATION {
        assert!(
            !blob.contains(phrase),
            "ISO-R-013: forbidden certification phrase `{phrase}` leaked into pack/projection sources"
        );
    }
    let report = serialize_empty_iso_report();
    let text = report.to_string().to_ascii_lowercase();
    for phrase in FORBIDDEN_CERTIFICATION {
        assert!(
            !text.contains(phrase),
            "ISO-R-013: serialized report must not emit `{phrase}`"
        );
    }
    let soa = serde_json::to_value(project_soa("iso-27001", "2022"))
        .unwrap()
        .to_string()
        .to_ascii_lowercase();
    for phrase in FORBIDDEN_CERTIFICATION {
        assert!(
            !soa.contains(phrase),
            "ISO-R-013: SoA must not emit `{phrase}`"
        );
    }
}

#[test]
fn iso_r_014_five_separate_coverage_metrics_and_no_compliance_percent() {
    let report = serialize_empty_iso_report();
    assert!(
        report.get("compliancePercent").is_none() && report.get("isoCompliant").is_none(),
        "ISO-R-014: must never emit compliancePercent / isoCompliant"
    );
    for key in FIVE_COVERAGE_KEYS {
        let cell = report.get(*key).unwrap_or_else(|| {
            panic!("ISO-R-014: serialized report must expose `{key}` as its own metric")
        });
        assert!(
            coverage_is_inspectable(cell),
            "ISO-R-014: `{key}` must be an inspectable count/ratio, not a single invented NN% string ({cell})"
        );
    }

    let pack = load_iso_pack();
    let compiled = compile_iso(&pack);
    let snapshot = project_readiness(
        &compiled,
        &[],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-iso-r-014"),
    );
    let json = serde_json::to_value(&snapshot).unwrap();
    assert!(json.get("compliancePercent").is_none());
    for key in FIVE_COVERAGE_KEYS {
        assert!(
            json.get(*key).is_some(),
            "ISO-R-014: readiness snapshot must expose `{key}`"
        );
    }

    let readiness_src =
        fs::read_to_string(crate_src("weeping-angel-assurance").join("readiness.rs")).unwrap();
    assert!(
        !readiness_src.contains("let has_partial = true"),
        "ISO-R-014: readiness must not hard-code has_partial = true"
    );
}

#[test]
fn iso_r_015_governance_stays_manual_hybrid_and_unlanded_families_stay_unmapped() {
    let requirements =
        fs::read_to_string(iso_pack_dir().join("requirements.toml")).expect("requirements");
    for id in GOVERNANCE_REQUIREMENTS {
        let idx = requirements
            .find(&format!("id = \"{id}\""))
            .unwrap_or_else(|| panic!("ISO-R-015: missing {id}"));
        let window = &requirements[idx..idx.saturating_add(400)];
        assert!(
            window.contains("automation = \"Manual\"")
                || window.contains("automation = \"Hybrid\""),
            "ISO-R-015: {id} must remain Manual/Hybrid"
        );
    }

    let catalog = load_catalog();
    let pack = load_iso_pack();
    let governance_landed = catalog
        .controls()
        .keys()
        .any(|id| id.starts_with("control.governance.") || id.starts_with("control.management."));
    if !governance_landed {
        for id in GOVERNANCE_REQUIREMENTS {
            assert!(
                mapped_tos(&pack, id).is_empty(),
                "ISO-R-015: {id} must stay unmapped until a governance catalog family lands"
            );
        }
        for (from, _) in [("iso27001:5.2", ()), ("iso27001:a.5.1", ())] {
            assert!(
                mapped_tos(&pack, from).is_empty(),
                "ISO-R-015: {from} must not keep a Related sliver onto incident.response-process"
            );
        }
    }

    for (from, prefix) in UNMAPPED_UNLESS_FAMILY_LANDS {
        let landed = catalog_has_prefix(&catalog, prefix)
            && *prefix != "control."
            && catalog
                .controls()
                .keys()
                .any(|id| id.starts_with(prefix) && id != "control.source.protected-branch");
        let tos = mapped_tos(&pack, from);
        if !landed && *prefix != "control.source." {
            assert!(
                tos.is_empty()
                    || tos
                        .iter()
                        .all(|t| t.starts_with("control.") && catalog.control(t).is_ok()),
                "ISO-R-015: {from} must not keep a pack sliver; unlanded families stay unmapped (have {tos:?})"
            );
            if !catalog_has_prefix(&catalog, "control.vulnerability.") && *from == "iso27001:a.8.8"
            {
                assert!(
                    tos.is_empty(),
                    "ISO-R-015: A.8.8 stays unmapped without vuln catalog"
                );
            }
        }
        for to in &tos {
            assert!(
                !RETIRED_SLIVERS.contains(&to.as_str()),
                "ISO-R-015: {from} still maps to retired sliver {to}"
            );
        }
    }

    let a825 = mapping_rows(&pack, "iso27001:a.8.25");
    if catalog
        .control("control.source.default-branch-protection")
        .is_ok()
    {
        assert!(
            a825.iter()
                .any(|m| m.to_control().as_str() == "control.source.default-branch-protection")
        );
    } else if catalog.control("control.source.protected-branch").is_ok() {
        for mapping in &a825 {
            if mapping.to_control().as_str() == "control.source.protected-branch" {
                assert!(
                    matches!(
                        mapping.relation(),
                        MappingRelation::PartiallySatisfies | MappingRelation::Supports
                    ),
                    "ISO-R-015: exists-only fixture must not fully satisfy A.8.25"
                );
                assert_ne!(mapping.completeness(), MappingCompleteness::Full);
            }
        }
        assert!(
            a825.iter()
                .all(|m| m.to_control().as_str() != "source.branch-protection"),
            "ISO-R-015: A.8.25 must not target the pack sliver source.branch-protection"
        );
    }

    let metadata = fs::read_to_string(iso_pack_dir().join("metadata.toml")).unwrap();
    assert!(
        !metadata.contains("id = \"test.iso27001.4.1\"")
            && !metadata.contains("id = \"test.access.mfa.privileged\""),
        "ISO-R-015: do not invent automated pack tests for governance clauses"
    );
}

#[test]
fn iso_r_016_collectors_and_control_test_stay_framework_neutral() {
    let collector = crate_sources_joined("weeping-angel-collector");
    assert!(
        !collector.contains("iso27001:") && !collector.to_ascii_lowercase().contains("iso27001:"),
        "ISO-R-016: collectors must contain no iso27001 requirement ids"
    );
    let control_test = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !control_test.contains("iso27001") && !control_test.contains("Iso27001"),
        "ISO-R-016: control-test runtime must have no ISO branches"
    );
}

#[test]
fn iso_r_017_iam_008_and_expected_canonical_controls_are_superseded() {
    let iam = fs::read_to_string(manifest_dir().join("tests/sdd/iam_catalog.target.rs")).unwrap();
    let iam_008 = iam
        .split("fn iam_008_iso_pack_is_unchanged_and_has_no_control_identity")
        .nth(1)
        .unwrap_or("");
    let iam_008_header = iam
        .split("fn iam_008_iso_pack_is_unchanged_and_has_no_control_identity")
        .next()
        .unwrap_or("");
    let ignored = iam_008_header
        .rsplit("#[test]")
        .next()
        .unwrap_or("")
        .contains("superseded by sdd_iso27001_remap_target")
        || iam.contains("#[ignore = \"superseded by sdd_iso27001_remap_target\"]")
            && iam.contains("fn iam_008_iso_pack_is_unchanged_and_has_no_control_identity");
    let rewritten = iam_008.contains("control.identity.privileged-mfa")
        && !iam_008.contains("ISO mappings must not retarget control.identity.*");
    assert!(
        ignored || rewritten,
        "ISO-R-017: IAM-008 must be skip-superseded or rewritten onto catalog identity remaps in this cut"
    );

    let iso_mvp =
        fs::read_to_string(manifest_dir().join("tests/sdd/iso27001_assurance.target.rs")).unwrap();
    let expected_still_sliver = iso_mvp.contains("const EXPECTED_CANONICAL_CONTROLS")
        && iso_mvp.contains("\"access.mfa.privileged\"");
    let prefixes_still_sliver =
        iso_mvp.contains("const CANONICAL_CONTROL_PREFIXES") && iso_mvp.contains("\"access.\"");
    let mvp_superseded = iso_mvp.contains("superseded by sdd_iso27001_remap_target")
        && (iso_mvp.contains("EXPECTED_CANONICAL_CONTROLS")
            || iso_mvp.contains("CANONICAL_CONTROL_PREFIXES"));
    let mvp_retargeted =
        iso_mvp.contains("control.identity.") && !iso_mvp.contains("\"access.mfa.privileged\"");
    assert!(
        mvp_superseded || mvp_retargeted || !(expected_still_sliver || prefixes_still_sliver),
        "ISO-R-017: EXPECTED_CANONICAL_CONTROLS / CANONICAL_CONTROL_PREFIXES must stop freezing pack slivers"
    );
    assert!(
        iso_mvp.contains("iso27001.") || iso_mvp.contains("ISO-004"),
        "ISO-R-017: ISO-004's no iso27001./.github. control-id rule stays in the MVP suite"
    );
}

#[test]
fn iso_r_018_catalog_and_framework_validate_succeed_on_catalog_targeted_pack() {
    let catalog = load_catalog();
    catalog
        .validate()
        .expect("ISO-R-018: catalog validate must succeed");
    let pack = validate_framework_pack(&iso_pack_dir())
        .expect("ISO-R-018: framework validate frameworks/iso-27001/2022 must succeed");
    for mapping in &pack.mappings {
        catalog
            .control(mapping.to_control().as_str())
            .unwrap_or_else(|_| {
                panic!(
                    "ISO-R-018: validated pack still maps to non-catalog id {}",
                    mapping.to_control()
                )
            });
    }

    let catalog_cli =
        weeping_angel::assurance_catalog::run(weeping_angel::cli::AssuranceCatalogArgs {
            command: weeping_angel::cli::AssuranceCatalogCommand::Validate { path: catalog_v1() },
        });
    assert_eq!(
        catalog_cli.expect("catalog CLI"),
        0,
        "ISO-R-018: weeping-angel assurance catalog validate must exit 0"
    );
}

#[test]
fn iso_r_019_structural_only_legal_boundary_holds() {
    let manifest = fs::read_to_string(iso_pack_dir().join("manifest.toml")).unwrap();
    assert!(
        manifest.contains("content_mode = \"StructuralOnly\""),
        "ISO-R-019: pack remains StructuralOnly"
    );
    let pack = load_iso_pack();
    assert_eq!(
        pack.content_provider,
        weeping_angel_framework::FrameworkContentProvider::StructuralOnly
    );
    let requirements = fs::read_to_string(iso_pack_dir().join("requirements.toml")).unwrap();
    let mappings = fs::read_to_string(iso_pack_dir().join("mappings.toml")).unwrap();
    let blob = format!("{requirements}\n{mappings}").to_ascii_lowercase();
    assert!(
        !blob.contains("the organization shall") && !blob.contains("iso/iec 27001"),
        "ISO-R-019: public pack must not redistribute ISO/IEC normative wording"
    );
    assert_eq!(
        requirements.matches("[[requirement]]").count(),
        pack.requirements.len()
    );
    assert_eq!(
        pack.requirements.len(),
        42,
        "ISO-R-019: do not inflate requirements.toml with extra Annex A clauses"
    );
}

#[test]
fn iso_r_020_neighbor_suites_stay_registered_after_sliver_supersession() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    for name in [
        "sdd_assurance_runtime_target",
        "sdd_iso27001_assurance_target",
        "sdd_canonical_assurance_catalog_target",
        "sdd_typed_evidence_target",
        "sdd_population_runtime_target",
        "sdd_iam_catalog_target",
        "sdd_iso27001_remap_target",
        "sdd_iso27001_remap_baseline",
    ] {
        assert!(
            toml.contains(name),
            "ISO-R-020: neighbor suite `{name}` must stay registered"
        );
    }
    let iam = fs::read_to_string(manifest_dir().join("tests/sdd/iam_catalog.target.rs")).unwrap();
    assert!(
        !iam.contains("ISO mappings must not retarget control.identity.*")
            || iam.contains("superseded by sdd_iso27001_remap_target"),
        "ISO-R-020: IAM target must not keep the sliver freeze after this cut"
    );
}
