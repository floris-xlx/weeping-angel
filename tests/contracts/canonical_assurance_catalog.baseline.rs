//! SUPERSEDED by `sdd_canonical_assurance_catalog_target`.
//!
//! Historical characterization of the pre-catalog tree on planning SHA
//! `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` (`docs/specs/canonical-assurance-catalog-v1.md`
//! §3 / §4.10) and increment-2 leaky seams
//! (`docs/specs/catalog-framework-readiness-trust-boundary.md` §3 / §7.1).
//! Absence assertions and retired leaky-seam tests are
//! `#[ignore = "superseded by target suite"]` (or a named successor suite)
//! so CI does not keep those absences as required green. Compatibility
//! checks (IR permissiveness, crate graph) stay active. The ISO thin-stub
//! pack-ID characterization is ignored after ISO remap
//! (`sdd_iso27001_remap_target`). Target
//! `sdd_canonical_assurance_catalog_target` is the source of truth.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use weeping_angel::cli::{AssuranceCommand, Cli, Commands};
use weeping_angel_assurance::readiness::FrameworkReadinessSnapshot;
use weeping_angel_assurance::snapshot::{AssessmentRun, catalog_digest};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentId, ControlId, ControlTestId, EvidenceRequirementId, IdError,
    MappingCompleteness, MappingDirection, MappingRelation, MappingSource,
};
use weeping_angel_canonical_catalog::{CanonicalCatalog, CatalogError};
use weeping_angel_framework::pack::FRAMEWORK_PACK_SCHEMA;
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget,
    assessment_from_pack, compile_framework, load_framework_pack, load_framework_pack_from,
    stub_catalog,
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

fn product_rust_sources() -> String {
    let mut files = Vec::new();
    walk_rs_files(&manifest_dir().join("crates"), &mut files);
    walk_rs_files(&manifest_dir().join("src"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_crate_toml(name: &str) -> String {
    fs::read_to_string(manifest_dir().join("crates").join(name).join("Cargo.toml"))
        .unwrap_or_else(|e| panic!("read {name} Cargo.toml: {e}"))
}

fn crate_src_file(name: &str, rel: &str) -> String {
    fs::read_to_string(
        manifest_dir()
            .join("crates")
            .join(name)
            .join("src")
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("read {name}/src/{rel}: {e}"))
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: weeping_angel_assurance_ir::FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), to).unwrap();
        }
    }
}

fn write_trust_boundary_pack(dir: &Path, mapping_block: &str, metadata: &str) {
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
    fs::write(dir.join("metadata.toml"), metadata).unwrap();
    fs::write(
        dir.join("mappings.toml"),
        format!(
            r#"schema = "weeping-angel/framework-pack/v1"

{mapping_block}
"#
        ),
    )
    .unwrap();
}

fn assert_current_assurance_command(command: &AssuranceCommand) {
    match command {
        AssuranceCommand::Framework(_)
        | AssuranceCommand::Collect(_)
        | AssuranceCommand::Evidence(_)
        | AssuranceCommand::Assess(_)
        | AssuranceCommand::Result(_)
        | AssuranceCommand::Compare(_)
        | AssuranceCommand::Soa(_)
        | AssuranceCommand::Catalog(_)
        | AssuranceCommand::Explain(_) => {}
    }
}

#[test]
fn dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        toml.contains("sdd_canonical_assurance_catalog_baseline")
            && toml.contains("sdd_canonical_assurance_catalog_target")
            && toml.contains("tests/contracts/canonical_assurance_catalog.baseline.rs")
            && toml.contains("tests/contracts/canonical_assurance_catalog.target.rs"),
        "dual-suite must be listed in root Cargo.toml (tests/contracts is not auto-discovered)"
    );
}

#[ignore = "superseded by sdd_canonical_assurance_catalog_target"]
#[test]
fn catalog_canonical_v1_tree_does_not_exist() {
    let root = manifest_dir().join("catalog");
    let v1 = manifest_dir().join("catalog/canonical/v1");
    assert!(
        !root.exists() && !v1.exists(),
        "current tree has no catalog/ directory (found {})",
        root.display()
    );
}

#[ignore = "superseded by sdd_canonical_assurance_catalog_target"]
#[test]
fn no_canonical_catalog_crate_or_workspace_member() {
    let crate_dir = manifest_dir().join("crates/weeping-angel-canonical-catalog");
    assert!(
        !crate_dir.exists(),
        "current tree has no weeping-angel-canonical-catalog crate"
    );
    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !cargo.contains("weeping-angel-canonical-catalog"),
        "workspace must not list weeping-angel-canonical-catalog yet"
    );
    let members: BTreeSet<&str> = [
        "crates/weeping-angel-assurance-ir",
        "crates/weeping-angel-framework",
        "crates/weeping-angel-evidence",
        "crates/weeping-angel-collector",
        "crates/weeping-angel-control-test",
        "crates/weeping-angel-assurance",
    ]
    .into_iter()
    .collect();
    for member in &members {
        assert!(
            cargo.contains(&format!("\"{member}\"")) || cargo.contains(&format!("'{member}'")),
            "workspace member {member} must remain"
        );
    }
}

#[ignore = "superseded by sdd_canonical_assurance_catalog_target"]
#[test]
fn no_canonical_catalog_api_in_product_rust() {
    let src = product_rust_sources();
    for needle in [
        "CanonicalCatalog",
        "CanonicalCatalog::load",
        "CanonicalCatalog::validate",
        "CanonicalCatalog::digest",
        "weeping-angel/canonical-catalog/v1",
        "wa:canonical-catalog:",
        "CatalogDigest",
        "CatalogError",
        "AssuranceCatalogCommand",
        "AssuranceCatalogArgs",
    ] {
        assert!(
            !src.contains(needle),
            "product Rust (crates/ + src/) currently has no `{needle}`"
        );
    }
}

#[ignore = "superseded by sdd_canonical_assurance_catalog_target"]
#[test]
fn assurance_command_lists_only_current_family() {
    let cmd = Cli::clap_command();
    let assurance = cmd
        .get_subcommands()
        .find(|c| c.get_name() == "assurance")
        .expect("assurance family exists today");
    let names: Vec<&str> = assurance.get_subcommands().map(|c| c.get_name()).collect();
    assert_eq!(
        names,
        [
            "framework",
            "collect",
            "evidence",
            "assess",
            "result",
            "compare",
            "soa"
        ],
        "AssuranceCommand is Framework/Collect/Evidence/Assess/Result/Compare/Soa only; have {names:?}"
    );
    assert!(
        !names.iter().any(|n| *n == "catalog"),
        "current CLI has no `assurance catalog` subcommand"
    );

    let parsed = Cli::try_parse_from(["weeping-angel", "assurance", "framework", "list"])
        .expect("framework list already parses");
    match parsed.command {
        Commands::Assurance(args) => assert_current_assurance_command(&args.command),
        other => panic!("expected Assurance, got {other:?}"),
    }
}

#[ignore = "superseded by sdd_canonical_assurance_catalog_target"]
#[test]
fn assurance_catalog_cli_does_not_parse() {
    for argv in [
        &["weeping-angel", "assurance", "catalog", "validate"][..],
        &["weeping-angel", "assurance", "catalog", "stats"][..],
        &[
            "weeping-angel",
            "assurance",
            "catalog",
            "inspect",
            "control.source.protected-branch",
        ][..],
    ] {
        let parsed = Cli::try_parse_from(argv);
        assert!(
            parsed.is_err(),
            "current clap parser rejects `{:?}` (got {parsed:?})",
            &argv[1..]
        );
    }
}

#[ignore = "superseded by sdd_canonical_assurance_catalog_target"]
#[test]
fn main_assurance_arm_is_not_certification_stub() {
    let main = fs::read_to_string(manifest_dir().join("src/main.rs")).unwrap();
    assert!(
        main.contains("Commands::Assurance(_)")
            && main.contains("This is a readiness assessment and is not certification."),
        "Assurance arm currently prints the not-certification banner and ignores the subcommand"
    );
    assert!(
        !main.contains("AssuranceCommand::Catalog")
            && !main.contains("assurance_catalog")
            && !main.contains("catalog validate"),
        "main.rs currently does not dispatch catalog execution"
    );
}

#[test]
fn ir_ids_remain_permissive_including_provider_and_framework_shapes() {
    assert!(ControlId::try_new("source.branch-protection").is_ok());
    assert!(ControlTestId::try_new("test.source.branch-protection").is_ok());
    assert!(ControlId::try_new("control.github.branch").is_ok());
    assert!(ControlId::try_new("control.iso27001.access").is_ok());
    assert!(EvidenceRequirementId::try_new("ev.source.branch.protection").is_ok());
    assert_eq!(ASSURANCE_IR_SCHEMA, "assurance-ir/v1");
}

#[test]
fn ir_invalid_namespace_is_never_returned() {
    let id_src =
        fs::read_to_string(manifest_dir().join("crates/weeping-angel-assurance-ir/src/id.rs"))
            .unwrap();
    assert!(
        id_src.contains("InvalidNamespace"),
        "IdError::InvalidNamespace is defined"
    );
    assert!(
        !id_src.contains("Err(IdError::InvalidNamespace)")
            && !id_src.contains("return Err(IdError::InvalidNamespace)"),
        "validate_stable_id currently never produces InvalidNamespace"
    );
    assert_eq!(
        ControlId::try_new(""),
        Err(IdError::Empty),
        "empty still fails as Empty, not InvalidNamespace"
    );
}

#[ignore = "superseded by sdd_iso27001_remap_target"]
#[test]
fn iso_metadata_owns_thin_canonical_stubs() {
    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        metadata.contains("id = \"source.branch-protection\"")
            && metadata.contains("id = \"test.source.branch-protection\""),
        "ISO pack still owns thin canonical stubs (no control.* remap)"
    );
    assert!(
        !metadata.contains("id = \"control.source.protected-branch\"")
            && !metadata.contains("id = \"control.iso27001."),
        "ISO metadata is not remapped onto catalog control.* ids"
    );
    let wa_meta = manifest_dir().join("frameworks/wa-baseline/1/metadata.toml");
    assert!(
        !wa_meta.is_file(),
        "wa-baseline/1 currently has no metadata.toml"
    );
    assert_eq!(FRAMEWORK_PACK_SCHEMA, "weeping-angel/framework-pack/v1");
}

#[test]
fn stub_catalog_loads_iso_pack_not_a_canonical_catalog() {
    let iso = stub_catalog(FrameworkProfile::Iso27001);
    assert!(
        !iso.is_empty(),
        "stub_catalog(Iso27001) currently loads the on-disk framework pack"
    );
    let other = stub_catalog(FrameworkProfile::Soc2);
    assert!(
        other.is_empty(),
        "non-ISO stub_catalog profiles currently return []"
    );
}

#[test]
fn framework_crate_has_no_collector_sdk_or_catalog() {
    let toml = read_crate_toml("weeping-angel-framework");
    for forbidden in [
        "weeping-angel-collector",
        "weeping-angel-canonical-catalog",
        "weeping-angel-control-test",
        "reqwest",
        "octocrab",
        "aws-sdk-",
        "cloudflare",
    ] {
        assert!(
            !toml.contains(forbidden),
            "framework Cargo.toml currently must not depend on `{forbidden}`"
        );
    }
    assert!(
        toml.contains("weeping-angel-assurance-ir"),
        "framework depends on IR only among assurance crates"
    );
}

#[test]
fn collector_crate_has_no_framework_or_catalog() {
    let toml = read_crate_toml("weeping-angel-collector");
    for forbidden in [
        "weeping-angel-framework",
        "weeping-angel-canonical-catalog",
        "weeping-angel-control-test",
        "iso27001",
        "soc2",
        "gdpr",
    ] {
        assert!(
            !toml.contains(forbidden),
            "collector Cargo.toml currently must not mention `{forbidden}`"
        );
    }
}

#[test]
fn ir_crate_has_no_toml_or_fs_loader() {
    let toml = read_crate_toml("weeping-angel-assurance-ir");
    assert!(
        !toml.contains("toml") && !toml.contains("std::fs"),
        "IR stays the identity crate without toml/fs catalog loading"
    );
    let lib =
        fs::read_to_string(manifest_dir().join("crates/weeping-angel-assurance-ir/src/lib.rs"))
            .unwrap();
    assert!(
        !lib.contains("std::fs") && !lib.contains("CanonicalCatalog"),
        "IR lib does not load catalog files"
    );
}

#[test]
fn cat_ssot_b00_canonical_catalog_load_rejects_duplicate_ids() {
    let src = manifest_dir().join("catalog/canonical/v1");
    let tmp = tempfile::tempdir().expect("temp catalog");
    let root = tmp.path().join("v1");
    copy_dir(&src, &root);
    let controls = root.join("controls/identity.toml");
    let mut text = fs::read_to_string(&controls).unwrap();
    text.push_str(
        r#"

[[control]]
id = "control.identity.privileged-mfa"
title = "duplicate privileged MFA"
domain = "identity"
narrative = "second row with the same id"
automation = "Automated"
criticality = "high"
evidence = ["evidence.identity.mfa-status"]
tests = ["test.identity.privileged-mfa-enabled"]
"#,
    );
    fs::write(&controls, text).unwrap();
    match CanonicalCatalog::load(&root) {
        Err(CatalogError::Duplicate { kind, id }) => {
            assert_eq!(kind, "control");
            assert_eq!(id, "control.identity.privileged-mfa");
        }
        other => panic!("CanonicalCatalog::load must fail closed on duplicate ids, got {other:?}"),
    }
}

#[ignore = "superseded by target suite"]
#[test]
fn cat_ssot_b01_pack_has_silent_catalog_parser() {
    let pack = crate_src_file("weeping-angel-framework", "pack.rs");
    assert!(
        pack.contains("fn discover_catalog_index() -> Option<CatalogIndex>"),
        "pack.rs currently re-parses catalog/canonical/v1 as Option"
    );
    assert!(
        pack.contains(
            "toml::from_str(&fs::read_to_string(root.join(\"manifest.toml\")).ok()?).ok()?"
        ),
        "catalog discover returns None on manifest IO/TOML failure"
    );
    assert!(
        pack.contains("let Ok(text) = fs::read_to_string(&path) else {")
            && pack.contains("let Ok(parsed) = toml::from_str::<toml::Value>(&text) else {"),
        "listed catalog files currently continue on IO/TOML failure"
    );
    assert!(
        pack.contains("let catalog = discover_catalog_index();"),
        "load_framework_pack_from currently injects the silent catalog index"
    );
    assert!(
        !pack.contains("struct IndexedTest")
            || !pack
                .split("struct IndexedTest")
                .nth(1)
                .unwrap()
                .split('}')
                .next()
                .unwrap()
                .contains("expression"),
        "IndexedTest currently has no expression field"
    );

    let loaded =
        load_framework_pack("iso-27001", "2022").expect("ISO pack loads via catalog index");
    assert!(
        loaded
            .controls
            .iter()
            .any(|c| c.id().as_str() == "control.identity.privileged-mfa"),
        "discover_catalog_index currently injects catalog controls into the pack"
    );
    assert!(
        loaded
            .tests
            .iter()
            .any(|t| t.id.as_str() == "test.identity.privileged-mfa-enabled"),
        "catalog tests are attached without going through CanonicalCatalog::load"
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn cat_ssot_b02_construct_test_plan_drops_expressions() {
    let lib = crate_src_file("weeping-angel-framework", "lib.rs");
    let plan = lib
        .split("fn construct_test_plan(")
        .nth(1)
        .expect("construct_test_plan");
    assert!(
        plan.contains("expr: None,"),
        "construct_test_plan currently sets CompiledTest.expr = None"
    );

    let catalog = CanonicalCatalog::load(manifest_dir().join("catalog/canonical/v1"))
        .expect("authoritative catalog load");
    let catalog_test = catalog
        .tests()
        .get("test.identity.privileged-mfa-enabled")
        .expect("catalog test");
    assert_eq!(
        catalog_test.expression.get("op").and_then(|v| v.as_str()),
        Some("coverage-at-least"),
        "catalog SSOT currently stores the coverage expression"
    );

    let pack = load_framework_pack("iso-27001", "2022").expect("ISO pack");
    let assessment = assessment_from_pack(&pack, &iso_target());
    let compiled = compile_framework(&assessment, &iso_target()).expect("compile");
    assert!(
        !compiled.tests.is_empty(),
        "compiled ISO plan must include catalog-injected tests"
    );
    assert!(
        compiled.tests.iter().all(|t| t.expr.is_none()),
        "compiled tests currently drop catalog [test.expression] trees"
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn cat_ssot_b03_metadata_is_latent_competing_library() {
    let metadata =
        fs::read_to_string(manifest_dir().join("frameworks/iso-27001/2022/metadata.toml")).unwrap();
    assert!(
        !metadata.contains("[[control]]") && !metadata.contains("[[test]]"),
        "ISO metadata.toml currently has no competing control/test library"
    );
    assert!(
        metadata.contains("library = \"catalog/canonical/v1\""),
        "ISO pack annotates catalog/canonical/v1 as the library"
    );

    let pack = crate_src_file("weeping-angel-framework", "pack.rs");
    assert!(
        pack.contains("struct MetadataFile")
            && pack.contains("control: Vec<ControlRow>")
            && pack.contains("test: Vec<TestRow>"),
        "pack loader still deserializes metadata [[control]] / [[test]]"
    );
    assert!(
        pack.contains("if row.id.starts_with(\"control.\")")
            && pack.contains("if !row.control.starts_with(\"control.\")"),
        "non-control.* metadata rows are still silently skipped"
    );

    let tmp = tempfile::tempdir().expect("temp pack");
    write_trust_boundary_pack(
        tmp.path(),
        r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "forward"
completeness = "partial"
relation = "PartiallySatisfies"
rationale = "catalog control plus competing sliver rows"
"#,
        r#"schema = "weeping-angel/framework-pack/v1"

[[control]]
id = "sliver.mfa.privileged"
title = "competing sliver"
description = "must be skipped today"

[[control]]
id = "control.identity.privileged-mfa"
title = "pack-local privileged MFA"
description = "competing catalog id"

[[test]]
id = "test.sliver.mfa"
control = "sliver.mfa.privileged"
kind = "automated"

[[test]]
id = "test.pack.privileged-mfa"
control = "control.identity.privileged-mfa"
kind = "automated"
"#,
    );
    let loaded = load_framework_pack_from(tmp.path()).expect("pack with sliver rows still loads");
    assert!(
        loaded
            .controls
            .iter()
            .all(|c| c.id().as_str() != "sliver.mfa.privileged"),
        "non-control.* metadata [[control]] is currently dropped"
    );
    assert!(
        loaded
            .tests
            .iter()
            .all(|t| t.id.as_str() != "test.sliver.mfa"),
        "metadata [[test]] whose control is not control.* is currently dropped"
    );
    assert!(
        loaded
            .tests
            .iter()
            .any(|t| t.id.as_str() == "test.pack.privileged-mfa"),
        "pack metadata tests for control.* currently merge beside catalog tests"
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn frw_b01_unknown_mapping_fields_default_best_effort() {
    let pack = crate_src_file("weeping-angel-framework", "pack.rs");
    assert!(
        pack.contains("_ => MappingCompleteness::Partial"),
        "unknown completeness currently defaults to Partial"
    );
    assert!(
        pack.contains("_ => MappingDirection::Forward"),
        "unknown / empty direction currently defaults to Forward"
    );
    assert!(
        pack.contains("_ => MappingSource::BuiltIn"),
        "unknown provenance source currently defaults to BuiltIn"
    );
    assert!(
        pack.contains("\"\" => MappingRelation::from_completeness(completeness)"),
        "empty relation currently derives from completeness"
    );
    assert!(
        pack.contains("content_provider: FrameworkContentProvider::StructuralOnly"),
        "content_provider is always StructuralOnly regardless of manifest content_mode"
    );

    let tmp = tempfile::tempdir().expect("temp pack");
    write_trust_boundary_pack(
        tmp.path(),
        r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "sideways"
completeness = "totally-bogus"
relation = ""
rationale = "unknown tokens default today"
provenance = { source = "mystery" }
"#,
        "schema = \"weeping-angel/framework-pack/v1\"\n",
    );
    let loaded = load_framework_pack_from(tmp.path()).expect("unknown mapping tokens still parse");
    let mapping = loaded.mappings.first().expect("one mapping");
    assert_eq!(mapping.completeness(), MappingCompleteness::Partial);
    assert_eq!(mapping.direction(), MappingDirection::Forward);
    assert_eq!(mapping.relation(), MappingRelation::PartiallySatisfies);
    assert_eq!(mapping.provenance().source, MappingSource::BuiltIn);
}

#[ignore = "superseded by target suite"]
#[test]
fn frw_b02_pack_digest_is_merged_id_lists() {
    let pack = crate_src_file("weeping-angel-framework", "pack.rs");
    assert!(
        pack.contains("\"requirements\": requirements.iter().map(|r| r.id().as_str())")
            && pack.contains("\"controls\": controls.iter().map(|c| c.id().as_str())")
            && pack.contains("\"tests\": tests.iter().map(|t| t.id.as_str())")
            && pack.contains("format!(\"{:?}\", m.relation())"),
        "pack digest currently hashes schema + id lists + relation Debug after merge"
    );
    assert!(
        !pack.contains("completeness")
            || !pack
                .split("let digest_body = serde_json::json!")
                .nth(1)
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .contains("completeness"),
        "digest body currently omits mapping completeness"
    );

    let mapping = r#"[[mapping]]
from = "iso27001:a.8.5"
to = "control.identity.privileged-mfa"
direction = "forward"
completeness = "COMPLETENESS"
relation = "PartiallySatisfies"
rationale = "RATIONALE"
"#;
    let meta = "schema = \"weeping-angel/framework-pack/v1\"\n";
    let a = tempfile::tempdir().expect("pack a");
    let b = tempfile::tempdir().expect("pack b");
    write_trust_boundary_pack(
        a.path(),
        &mapping
            .replace("COMPLETENESS", "partial")
            .replace("RATIONALE", "first rationale"),
        meta,
    );
    write_trust_boundary_pack(
        b.path(),
        &mapping.replace("COMPLETENESS", "full").replace(
            "RATIONALE",
            "second rationale  # comment-only change in meaning",
        ),
        meta,
    );
    let da = load_framework_pack_from(a.path()).unwrap();
    let db = load_framework_pack_from(b.path()).unwrap();
    assert_eq!(
        da.digest.0, db.digest.0,
        "completeness/rationale changes currently collide when relation + ids stay the same"
    );
    assert_ne!(
        da.mappings[0].completeness(),
        db.mappings[0].completeness(),
        "the two packs currently differ in completeness (partial vs full)"
    );

    let c = tempfile::tempdir().expect("pack c");
    write_trust_boundary_pack(
        c.path(),
        &mapping
            .replace("COMPLETENESS", "partial")
            .replace(
                "relation = \"PartiallySatisfies\"",
                "relation = \"Supports\"",
            )
            .replace("RATIONALE", "supports"),
        meta,
    );
    let dc = load_framework_pack_from(c.path()).unwrap();
    assert_ne!(
        da.digest.0, dc.digest.0,
        "relation Debug currently is hashed, so Supports changes the digest"
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn pin_b01_readiness_snapshot_serialize_reloads_live_catalog() {
    let readiness = crate_src_file("weeping-angel-assurance", "readiness.rs");
    assert!(
        readiness.contains("state.serialize_field(\"catalogDigest\", &catalog_digest())?"),
        "FrameworkReadinessSnapshot::serialize currently calls catalog_digest()"
    );

    let snap = FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new("assess-pin-b01"),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: "pack-pin-ignored".into(),
        catalog_digest: String::new(),
        assessment_digest: "assessment-pin-ignored".into(),
        evaluated_at: "2026-01-01T00:00:00Z".into(),
        requirements: Vec::new(),
        controls: Vec::new(),
        effective: 0,
        ineffective: 0,
        partial: 0,
        manual_review: 0,
        insufficient_evidence: 0,
        not_applicable: 0,
        automation_coverage: "stored-not-used".into(),
        evidence_coverage: "stored-not-used".into(),
    };
    let json = serde_json::to_value(&snap).expect("serialize snapshot");
    let live = catalog_digest();
    assert_eq!(
        json.get("catalogDigest").and_then(|v| v.as_str()),
        Some(live.as_str()),
        "snapshot JSON catalogDigest is the live catalog, not a stored pin"
    );
    assert_ne!(
        live, "pack-pin-ignored",
        "live catalog digest must not be the unused pack pin string"
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn pin_b02_catalog_digest_and_empty_run_pin_fallback() {
    let snapshot = crate_src_file("weeping-angel-assurance", "snapshot.rs");
    assert!(
        snapshot.contains("\"catalog-unavailable\".into()"),
        "snapshot::catalog_digest currently falls back to catalog-unavailable"
    );
    assert!(
        snapshot.contains("if self.canonical_catalog_pin.is_empty()")
            && snapshot.contains("catalog_digest()"),
        "empty AssessmentRun pin currently reloads live catalog at serialize"
    );
    let facade = crate_src_file("weeping-angel-assurance", "lib.rs");
    assert!(
        facade.contains("fn load_catalog_pin()")
            && facade.contains("\"catalog-unavailable\".into()"),
        "load_catalog_pin currently falls back to catalog-unavailable"
    );

    let live = catalog_digest();
    assert!(
        live.starts_with("wa:canonical-catalog:weeping-angel/canonical-catalog/v1:"),
        "workspace catalog is loadable today; live pin is {live}"
    );
    let run = AssessmentRun {
        canonical_catalog_pin: String::new(),
        ..AssessmentRun::default()
    };
    let json = serde_json::to_value(&run).expect("serialize run");
    assert_eq!(
        json.get("catalogDigest").and_then(|v| v.as_str()),
        Some(live.as_str()),
        "empty pin currently becomes the live catalog digest at serialize"
    );
    assert_eq!(
        json.get("canonicalCatalogDigest").and_then(|v| v.as_str()),
        Some(live.as_str())
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn pin_b03_scheduler_reloads_unpinned_pack() {
    let scheduler = crate_src_file("weeping-angel-assurance", "scheduler.rs");
    let project = scheduler
        .split("fn run_project(")
        .nth(1)
        .expect("run_project");
    assert!(
        project.contains("load_framework_pack(framework, version)")
            && project.contains("\"unpinned\""),
        "run_project currently reloads the pack and treats failure as unpinned"
    );
    let snap = scheduler
        .split("fn run_snapshot(")
        .nth(1)
        .expect("run_snapshot");
    assert!(
        snap.contains("load_framework_pack(framework, self.target.version.as_str())")
            && snap.contains("canonical_catalog_pin: String::new()"),
        "run_snapshot currently reloads the pack and leaves the catalog pin empty"
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn rdy_b01_overlay_privileged_mfa_and_empty_readiness_forks() {
    let scheduler = crate_src_file("weeping-angel-assurance", "scheduler.rs");
    assert!(
        scheduler.contains("fn overlay_privileged_mfa_presence(")
            && scheduler.contains("overlay_privileged_mfa_presence(&mut results, evidence, &ctx);"),
        "scheduler currently overlays privileged-MFA results after evaluate_compiled"
    );
    assert!(
        scheduler.contains(".require(EvidenceType::new(\"identity.privileged.mfa\"))")
            && scheduler.contains("*row = overlay;"),
        "overlay currently replaces control.identity.privileged-mfa with a presence test"
    );
    assert!(
        scheduler.contains("fn empty_readiness(")
            && scheduler.contains("automation_coverage: \"0%\".into()")
            && scheduler.contains("evidence_coverage: \"0%\".into()"),
        "empty_readiness currently invents 0% coverage strings"
    );
    let readiness = crate_src_file("weeping-angel-assurance", "readiness.rs");
    assert!(
        readiness.contains("fn coverage_metrics(snapshot: &FrameworkReadinessSnapshot)")
            && readiness.contains("let metrics = coverage_metrics(self);"),
        "snapshot serialize currently re-derives coverage instead of using project_readiness strings"
    );
}
