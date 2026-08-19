//! SUPERSEDED by `sdd_canonical_assurance_catalog_target` after catalog
//! infrastructure landed (Prompt 01).
//!
//! Historical characterization of the pre-catalog tree on planning SHA
//! `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` (`docs/sdd/canonical-assurance-catalog-v1.md`
//! §3 / §4.10). Absence assertions are ignored so CI does not keep
//! “there is no catalog” as required green. Compatibility checks (IR
//! permissiveness, ISO pack IDs, crate graph) stay active.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use weeping_angel::cli::{AssuranceCommand, Cli, Commands};
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, ControlId, ControlTestId, EvidenceRequirementId, IdError,
};
use weeping_angel_framework::pack::FRAMEWORK_PACK_SCHEMA;
use weeping_angel_framework::{FrameworkProfile, stub_catalog};

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
            && toml.contains("tests/sdd/canonical_assurance_catalog.baseline.rs")
            && toml.contains("tests/sdd/canonical_assurance_catalog.target.rs"),
        "dual-suite must be listed in root Cargo.toml (tests/sdd is not auto-discovered)"
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
