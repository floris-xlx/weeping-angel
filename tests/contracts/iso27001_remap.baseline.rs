//! Baseline characterization of ISO 27001:2022 pack-local slivers and ISO
//! special-cases (`docs/specs/iso-27001-canonical-remap.md` §3 / §4.11).
//!
//! SUPERSEDED by `sdd_iso27001_remap_target` after ISO remap implement.
//! Encodes what HEAD does today: mappings target `access.*` / `source.*`
//! slivers, `metadata.toml` is the control library, the pack loader rejects
//! `EvidenceFor` / `SupersetOf` / `SubsetOf`, generic serialize/assess hard-load
//! `iso-27001/2022`, SoA copies `applicability.toml` booleans, and
//! `AssessmentRun` pins pack digest only. GREEN until the remap target lands.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use weeping_angel_assurance::readiness::project_readiness;
use weeping_angel_assurance::{AssessmentReport, AssessmentRun, project_soa};
use weeping_angel_assurance_ir::{
    AssessmentId, MappingCompleteness, MappingRelation, MappingSource,
};
use weeping_angel_framework::pack::PackError;
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget, compile_framework,
    load_framework_pack, load_framework_pack_from, stub_catalog,
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

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: weeping_angel_assurance_ir::FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

const PACK_SLIVERS: &[&str] = &[
    "access.mfa.privileged",
    "access.least-privilege",
    "access.periodic-review",
    "source.branch-protection",
    "vulnerability.remediation",
    "personnel.access-termination",
];

const REJECTED_RELATIONS: &[&str] = &["EvidenceFor", "SupersetOf", "SubsetOf"];

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_iso27001_remap_baseline")
            && toml.contains("sdd_iso27001_remap_target")
            && toml.contains("tests/contracts/iso27001_remap.baseline.rs")
            && toml.contains("tests/contracts/iso27001_remap.target.rs"),
        "ISO remap dual-suite must be listed in root Cargo.toml (tests/contracts is not auto-discovered)"
    );
    assert!(
        !toml.contains("tests/contracts/iso27001_assurance.baseline.rs")
            || toml.contains("sdd_iso27001_assurance_baseline"),
        "must not reuse the MVP dual-suite names for the remap suite"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/iso27001_remap.baseline.rs")
            .is_file()
            && manifest_dir()
                .join("tests/contracts/iso27001_remap.target.rs")
                .is_file(),
        "remap dual-suite files must exist and must not be iso27001_assurance.*"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn mappings_target_pack_slivers_not_catalog_identity() {
    let mappings =
        fs::read_to_string(iso_pack_dir().join("mappings.toml")).expect("read mappings.toml");
    for sliver in [
        "access.mfa.privileged",
        "source.branch-protection",
        "vulnerability.remediation",
    ] {
        assert!(
            mappings.contains(&format!("to = \"{sliver}\"")),
            "current mappings must target pack sliver `{sliver}`"
        );
    }
    assert!(
        !mappings.contains("control.identity.privileged-mfa"),
        "current mappings must not mention control.identity.privileged-mfa"
    );
    assert!(
        !mappings.contains("control.identity."),
        "current mappings must not retarget control.identity.*"
    );
    assert!(
        !mappings.contains("relation = \"Equivalent\""),
        "current pack uses no Equivalent rows"
    );
    for rel in REJECTED_RELATIONS {
        assert!(
            !mappings.contains(&format!("relation = \"{rel}\"")),
            "current pack does not emit relation {rel}"
        );
    }
    assert!(
        !mappings.contains("provenance") && !mappings.contains("valid_for"),
        "MappingRow files currently carry no provenance / valid_for fields"
    );

    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    assert_eq!(
        pack.mappings.len(),
        27,
        "current pack ships 27 mapping rows"
    );
    let relations: BTreeSet<String> = pack
        .mappings
        .iter()
        .map(|m| format!("{:?}", m.relation()))
        .collect();
    assert_eq!(
        relations,
        BTreeSet::from([
            format!("{:?}", MappingRelation::PartiallySatisfies),
            format!("{:?}", MappingRelation::Supports),
            format!("{:?}", MappingRelation::Related),
        ]),
        "current relations are PartiallySatisfies / Supports / Related only"
    );
    for mapping in &pack.mappings {
        assert!(
            mapping.valid_for().is_unconstrained(),
            "loader does not populate valid_for from the pack file"
        );
        assert_eq!(
            mapping.provenance().source,
            MappingSource::BuiltIn,
            "loader leaves IR default provenance (BuiltIn); pack has no provenance field"
        );
        assert_ne!(mapping.completeness(), MappingCompleteness::Full);
        let to = mapping.to_control().as_str();
        assert!(
            !to.starts_with("control."),
            "mapping to `{to}` is a pack sliver, not a catalog control id"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn metadata_toml_is_the_competing_control_library() {
    let metadata =
        fs::read_to_string(iso_pack_dir().join("metadata.toml")).expect("read metadata.toml");
    for sliver in PACK_SLIVERS {
        assert!(
            metadata.contains(&format!("id = \"{sliver}\"")),
            "metadata.toml currently declares sliver `{sliver}`"
        );
    }
    assert!(
        metadata.contains("id = \"test.access.mfa.privileged\""),
        "pack-local privileged MFA test lives in metadata.toml"
    );
    assert!(
        metadata.contains("source.admin.permissions"),
        "privileged MFA test currently requires some source.admin.permissions"
    );

    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let control_ids: BTreeSet<_> = pack.controls.iter().map(|c| c.id().as_str()).collect();
    assert_eq!(
        pack.controls.len(),
        22,
        "metadata.toml currently ships 22 pack-local controls"
    );
    assert_eq!(
        pack.tests.len(),
        22,
        "metadata.toml currently ships 22 pack-local tests"
    );
    for sliver in PACK_SLIVERS {
        assert!(
            control_ids.contains(sliver),
            "loaded pack must expose sliver `{sliver}` (have {control_ids:?})"
        );
    }
    assert!(
        !control_ids
            .iter()
            .any(|id| id.starts_with("control.identity.")),
        "ISO pack must not contain catalog identity control ids"
    );

    let compiled = compile_framework(
        &weeping_angel_framework::assessment_from_pack(&pack, &iso_target()),
        &iso_target(),
    )
    .expect("ISO pack compiles");
    let compiled_ids: BTreeSet<_> = compiled.controls.iter().map(|c| c.id().as_str()).collect();
    assert!(
        compiled_ids.contains("access.mfa.privileged"),
        "compiled ISO assessment still owns the privileged-MFA sliver"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn assessment_report_serialize_hard_loads_iso_pack() {
    let src = crate_sources_joined("weeping-angel-assurance");
    assert!(
        src.contains("impl Serialize for AssessmentReport"),
        "AssessmentReport custom Serialize must still exist"
    );
    let serialize_window = src
        .split("impl Serialize for AssessmentReport")
        .nth(1)
        .expect("serialize impl");
    assert!(
        serialize_window.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "AssessmentReport serialize currently hard-loads the ISO pack"
    );
    assert!(
        !serialize_window.contains("catalogDigest") && !serialize_window.contains("catalog_digest"),
        "AssessmentReport serialize currently emits no catalogDigest"
    );
    assert!(
        serialize_window.contains("automationCoverage")
            && serialize_window.contains("evidenceCoverage")
            && serialize_window.contains("\"{:.0}%\""),
        "serialize invents automationCoverage / evidenceCoverage percentage strings"
    );

    let report = AssessmentReport {
        assessment_id: AssessmentId::new("assess-remap-baseline"),
        profile: "iso-27001".into(),
        digest: "digest".into(),
        results: Vec::new(),
        evidence_count: 0,
        ..Default::default()
    };
    let value = serde_json::to_value(&report).expect("serialize report");
    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    assert_eq!(
        value.get("frameworkPackDigest").and_then(Value::as_str),
        Some(pack.digest.as_str()),
        "serialize pins the live ISO pack digest via filesystem load"
    );
    assert!(
        value.get("catalogDigest").is_none(),
        "serialized report has no catalogDigest field"
    );
    let automation = value
        .get("automationCoverage")
        .and_then(Value::as_str)
        .unwrap_or("");
    let evidence = value
        .get("evidenceCoverage")
        .and_then(Value::as_str)
        .unwrap_or("");
    assert!(
        automation.ends_with('%') && evidence.ends_with('%'),
        "coverage fields are invented NN% strings, not separate count metrics"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn generic_paths_special_case_iso27001_2022() {
    let framework = crate_sources_joined("weeping-angel-framework");
    assert!(
        framework.contains(
            "target.profile == FrameworkProfile::Iso27001 && target.version.as_str() == \"2022\""
        ) || framework
            .contains("FrameworkProfile::Iso27001 && target.version.as_str() == \"2022\""),
        "normalize currently special-cases Iso27001 + 2022"
    );
    assert!(
        framework.contains(
            "FrameworkProfile::Iso27001 => pack::load_framework_pack(\"iso-27001\", \"2022\")"
        ),
        "stub_catalog currently loads the ISO pack only for Iso27001"
    );

    let assurance =
        fs::read_to_string(crate_src("weeping-angel-assurance").join("lib.rs")).unwrap();
    assert!(
        assurance.contains("fn assessment_for_target")
            && assurance.contains("FrameworkProfile::Iso27001")
            && assurance.contains("target.version.as_str() == \"2022\"")
            && assurance.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "assessment_for_target currently branches on Iso27001 + 2022"
    );

    let iso_reqs = stub_catalog(FrameworkProfile::Iso27001);
    assert_eq!(
        iso_reqs.len(),
        42,
        "stub_catalog(Iso27001) currently returns the 42 structural pack requirements"
    );
    assert!(
        iso_reqs.iter().any(|r| r.id().as_str() == "iso27001:4.1"),
        "ISO stub catalog includes iso27001:4.1"
    );
    for profile in [
        FrameworkProfile::Soc2,
        FrameworkProfile::Nis2,
        FrameworkProfile::Dora,
        FrameworkProfile::Gdpr,
        FrameworkProfile::Iso27701,
        FrameworkProfile::Iso27007,
    ] {
        assert!(
            stub_catalog(profile).is_empty(),
            "stub_catalog({profile:?}) currently returns []"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn project_soa_rereads_applicability_toml_booleans() {
    let soa_src = fs::read_to_string(crate_src("weeping-angel-assurance").join("soa.rs")).unwrap();
    assert!(
        soa_src.contains("applicability.toml"),
        "project_soa currently rereads live applicability.toml"
    );
    assert!(
        soa_src.contains("as_bool") && soa_src.contains("applicable:"),
        "SoA entries currently copy a boolean applicable field"
    );
    assert!(
        !soa_src.contains("Unresolved") && !soa_src.contains("ManualDeterminationRequired"),
        "SoA projection has no three-state generic applicability result"
    );

    let applicability =
        fs::read_to_string(iso_pack_dir().join("applicability.toml")).expect("read applicability");
    assert_eq!(
        applicability.matches("[[entry]]").count(),
        10,
        "pack currently ships 10 SoA-oriented entries"
    );
    assert!(
        !applicability.contains("applicable = false")
            && !applicability.contains("unresolved")
            && !applicability.contains("manual"),
        "applicability.toml currently has only applicable = true booleans"
    );

    let soa = project_soa("iso-27001", "2022");
    assert_eq!(soa.entries.len(), 10);
    assert!(
        soa.entries.iter().all(|e| e.applicable),
        "every SoA entry is the pack boolean true; none are unresolved"
    );
    assert!(
        soa.entries
            .iter()
            .any(|e| e.reference == "A.8.5" && e.applicable),
        "A.8.5 is copied as applicable=true from the file"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn assessment_run_has_pack_digest_only() {
    let snapshot =
        fs::read_to_string(crate_src("weeping-angel-assurance").join("snapshot.rs")).unwrap();
    assert!(
        snapshot.contains("pub framework_pack_digest: String"),
        "AssessmentRun currently has framework_pack_digest"
    );
    assert!(
        !snapshot.contains("catalog_digest"),
        "AssessmentRun currently has no catalog_digest field"
    );

    let assess_src =
        fs::read_to_string(crate_src("weeping-angel-assurance").join("lib.rs")).unwrap();
    assert!(
        assess_src.contains("let _run = AssessmentRun")
            && assess_src.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "assess() constructs AssessmentRun from a hardcoded ISO pack load, then drops it"
    );

    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    let run = AssessmentRun {
        id: AssessmentId::new("run-remap-baseline"),
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
    let value = serde_json::to_value(&run).expect("serialize run");
    assert_eq!(
        value.get("frameworkPackDigest").and_then(Value::as_str),
        Some(pack.digest.as_str())
    );
    assert!(value.get("catalogDigest").is_none());

    let readiness_src =
        fs::read_to_string(crate_src("weeping-angel-assurance").join("readiness.rs")).unwrap();
    assert!(
        readiness_src.contains("let has_partial = true"),
        "project_readiness currently hard-codes has_partial = true"
    );
    assert!(
        readiness_src.contains("compiled.controls.iter().map(|c| c.id().clone()).collect()"),
        "project_readiness currently maps every compiled control onto every requirement"
    );
    assert!(
        !readiness_src.contains("catalog_digest") && !readiness_src.contains("catalogDigest"),
        "readiness snapshot pins pack digest only"
    );

    let compiled = compile_framework(
        &weeping_angel_framework::assessment_from_pack(&pack, &iso_target()),
        &iso_target(),
    )
    .expect("compile");
    let snapshot = project_readiness(
        &compiled,
        &[],
        "iso-27001",
        "2022",
        pack.digest.as_str(),
        AssessmentId::new("assess-readiness-baseline"),
    );
    assert_eq!(snapshot.framework_pack_digest, pack.digest.0);
    let snap_json = serde_json::to_value(&snapshot).unwrap();
    assert!(snap_json.get("catalogDigest").is_none());
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn catalog_has_identity_and_fixture_only_no_iso_slivers() {
    let manifest =
        fs::read_to_string(manifest_dir().join("catalog/canonical/v1/manifest.toml")).unwrap();
    assert!(
        manifest.contains("controls/fixture.example.toml")
            && manifest.contains("controls/identity.toml"),
        "catalog v1 currently lists fixture.example + identity only"
    );
    assert!(
        !manifest.contains("iso") && !manifest.contains("access.mfa"),
        "catalog manifest is not an ISO stub library"
    );

    let mut catalog_controls = String::new();
    for name in ["fixture.example.toml", "identity.toml"] {
        catalog_controls.push_str(
            &fs::read_to_string(
                manifest_dir()
                    .join("catalog/canonical/v1/controls")
                    .join(name),
            )
            .unwrap(),
        );
    }
    assert!(
        catalog_controls.contains("id = \"control.identity.privileged-mfa\""),
        "catalog already has the IAM privileged-MFA control"
    );
    for sliver in PACK_SLIVERS {
        assert!(
            !catalog_controls.contains(&format!("id = \"{sliver}\"")),
            "catalog must not contain competing ISO sliver `{sliver}`"
        );
    }

    let controls_dir = manifest_dir().join("catalog/canonical/v1/controls");
    let mut listed = BTreeSet::new();
    for entry in fs::read_dir(&controls_dir).unwrap() {
        let name = entry.unwrap().file_name();
        listed.insert(name.to_string_lossy().into_owned());
    }
    assert_eq!(
        listed,
        BTreeSet::from(["fixture.example.toml".into(), "identity.toml".into(),]),
        "catalog control files on HEAD are fixture + identity only"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn pack_loader_rejects_evidence_for_superset_of_subset_of() {
    let pack_src =
        fs::read_to_string(crate_src("weeping-angel-framework").join("pack.rs")).unwrap();
    let relation_match = pack_src
        .split("let relation = match row.relation.as_str()")
        .nth(1)
        .expect("relation match");
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
    ] {
        assert!(
            relation_match.contains(&format!("\"{accepted}\"")),
            "loader currently accepts {accepted}"
        );
    }
    for rejected in REJECTED_RELATIONS {
        assert!(
            !relation_match.contains(&format!("\"{rejected}\"")),
            "loader match currently omits {rejected}"
        );
    }
    assert!(
        pack_src.contains("struct MappingRow")
            && !pack_src
                .split("struct MappingRow")
                .nth(1)
                .unwrap()
                .split('}')
                .next()
                .unwrap()
                .contains("provenance"),
        "MappingRow has no provenance field"
    );
    assert!(
        pack_src.contains(
            "if !controls.iter().any(|c| c.id().as_str() == row.to) && !meta.control.is_empty()"
        ),
        "mapping `to` is validated against pack metadata, not the catalog"
    );

    let tmp = tempfile::tempdir().expect("temp pack");
    write_minimal_pack(tmp.path(), "EvidenceFor");
    match load_framework_pack_from(tmp.path()) {
        Err(PackError::UnsupportedRelation(rel)) => {
            assert_eq!(rel, "EvidenceFor");
        }
        other => panic!("EvidenceFor must be UnsupportedRelation, got {other:?}"),
    }

    write_minimal_pack(tmp.path(), "SupersetOf");
    assert!(matches!(
        load_framework_pack_from(tmp.path()),
        Err(PackError::UnsupportedRelation(rel)) if rel == "SupersetOf"
    ));
    write_minimal_pack(tmp.path(), "SubsetOf");
    assert!(matches!(
        load_framework_pack_from(tmp.path()),
        Err(PackError::UnsupportedRelation(rel)) if rel == "SubsetOf"
    ));

    write_dangling_catalog_target_pack(tmp.path());
    match load_framework_pack_from(tmp.path()) {
        Err(PackError::Dangling { to, .. }) => {
            assert_eq!(to, "control.identity.privileged-mfa");
        }
        other => panic!("catalog id without pack metadata must dangle, got {other:?}"),
    }
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn neighbor_tests_still_freeze_the_sliver() {
    let iam =
        fs::read_to_string(manifest_dir().join("tests/contracts/iam_catalog.target.rs")).unwrap();
    assert!(
        iam.contains("fn iam_008_iso_pack_is_unchanged_and_has_no_control_identity"),
        "IAM-008 still lives in the IAM target suite"
    );
    assert!(
        iam.contains("access.mfa.privileged")
            && iam.contains("ISO mappings must not retarget control.identity.*"),
        "IAM-008 still freezes pack slivers and forbids control.identity.* mappings"
    );

    let iso_mvp =
        fs::read_to_string(manifest_dir().join("tests/contracts/iso27001_assurance.target.rs"))
            .unwrap();
    assert!(
        iso_mvp.contains("const EXPECTED_CANONICAL_CONTROLS")
            && iso_mvp.contains("\"access.mfa.privileged\""),
        "ISO MVP target still freezes EXPECTED_CANONICAL_CONTROLS including access.mfa.privileged"
    );
    assert!(
        iso_mvp.contains("const CANONICAL_CONTROL_PREFIXES") && iso_mvp.contains("\"access.\""),
        "ISO MVP target still freezes pack-local prefixes such as access."
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn collectors_and_control_test_have_no_iso_requirement_ids() {
    let collector = crate_sources_joined("weeping-angel-collector");
    assert!(
        !collector.contains("iso27001:") && !collector.to_ascii_lowercase().contains("iso27001:"),
        "collectors currently contain no iso27001 requirement ids"
    );
    let control_test = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !control_test.contains("iso27001") && !control_test.contains("Iso27001"),
        "control-test runtime currently has no ISO branches"
    );
}

#[test]
#[ignore = "superseded by sdd_iso27001_remap_target"]
fn pack_is_structural_only_with_forty_two_requirements() {
    let manifest = fs::read_to_string(iso_pack_dir().join("manifest.toml")).unwrap();
    assert!(
        manifest.contains("content_mode = \"StructuralOnly\""),
        "pack remains StructuralOnly"
    );
    assert!(manifest.contains("id = \"iso-27001\"") && manifest.contains("version = \"2022\""));

    let requirements = fs::read_to_string(iso_pack_dir().join("requirements.toml")).unwrap();
    assert_eq!(requirements.matches("[[requirement]]").count(), 42);
    assert!(
        !requirements
            .to_ascii_lowercase()
            .contains("the organization shall")
            && !requirements.contains("ISO/IEC 27001"),
        "public pack must not carry ISO/IEC normative wording"
    );

    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack loads");
    assert_eq!(pack.requirements.len(), 42);
    assert_eq!(
        pack.content_provider,
        weeping_angel_framework::FrameworkContentProvider::StructuralOnly
    );
}

fn write_minimal_pack(dir: &Path, relation: &str) {
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

[[control]]
id = "access.mfa.privileged"
title = "Privileged MFA sliver"
description = "Pack-local sliver"
"#,
    )
    .unwrap();
    fs::write(
        dir.join("mappings.toml"),
        format!(
            r#"schema = "weeping-angel/framework-pack/v1"

[[mapping]]
from = "iso27001:a.8.5"
to = "access.mfa.privileged"
direction = "forward"
completeness = "partial"
relation = "{relation}"
rationale = "characterization of unsupported relation"
"#
        ),
    )
    .unwrap();
}

fn write_dangling_catalog_target_pack(dir: &Path) {
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

[[control]]
id = "access.mfa.privileged"
title = "Privileged MFA sliver"
description = "Pack-local sliver"
"#,
    )
    .unwrap();
    fs::write(
        dir.join("mappings.toml"),
        r#"schema = "weeping-angel/framework-pack/v1"

[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "forward"
completeness = "partial"
relation = "PartiallySatisfies"
rationale = "catalog id is dangling against pack metadata today"
"#,
    )
    .unwrap();
}
