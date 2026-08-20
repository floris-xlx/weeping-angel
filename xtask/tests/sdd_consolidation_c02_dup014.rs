//! C02 target ΓÇö one filesystem walker (DUP-014 close law).
//!
//! GREEN when inventory reuses `xtask/src/model.rs` `walk_tree` /
//! `should_skip_dir` and the parallel inventory walk is gone.
//! Avoid unwrap/expect method-call needles.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;
use xtask::debt::KNOWN_CHECK_IDS;
use xtask::duplication::load_structural_duplication;
use xtask::inventory::InventoryReport;
use xtask::repo_root_from_xtask_manifest;

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn text_defines(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with("//") && trimmed.contains(name)
    })
}

fn dup014_table() -> String {
    let debt = read_live("docs/debt/structural-duplication.toml");
    let start = debt
        .find("id = \"DUP-014\"")
        .unwrap_or_else(|| panic!("DUP-014 row missing from structural-duplication.toml"));
    let rest = &debt[start..];
    let end = rest[1..]
        .find("[[duplication]]")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

fn write_rs(dir: &Path, rel: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("mkdir {}: {e}", parent.display()));
    }
    fs::write(&path, "fn marker() {}\n")
        .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

#[test]
fn dup014_model_owns_single_walker() {
    let model = read_live("xtask/src/model.rs");
    assert!(
        model.contains("SKIP_DIR_NAMES"),
        "canonical skip set lives in model.rs"
    );
    assert!(
        text_defines(&model, "fn should_skip_dir"),
        "model.rs must define should_skip_dir"
    );
    assert!(
        text_defines(&model, "fn walk_tree"),
        "model.rs must define walk_tree"
    );
    assert!(
        model.contains("__pycache__") && model.contains("apps"),
        "aligned skip set includes __pycache__ and apps"
    );
}

#[test]
fn dup014_inventory_has_no_parallel_walk() {
    let inventory = read_live("xtask/src/inventory.rs");
    assert!(
        !text_defines(&inventory, "fn should_skip_dir"),
        "inventory.rs must not define should_skip_dir"
    );
    assert!(
        !text_defines(&inventory, "fn walk_included"),
        "inventory.rs must not keep walk_included as a parallel walker"
    );
    assert!(
        !text_defines(&inventory, "fn walk_tree"),
        "inventory.rs must not define a second walk_tree"
    );
    assert!(
        inventory.contains("walk_tree"),
        "inventory must call the model walker"
    );
}

#[test]
fn dup014_skip_set_shared_on_temp_tree() {
    let dir = tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let root = dir.path();
    write_rs(root, "kept.rs");
    write_rs(root, "src/kept.rs");
    write_rs(root, "target/skip.rs");
    write_rs(root, "node_modules/skip.rs");
    write_rs(root, ".git/skip.rs");
    write_rs(root, "__pycache__/skip.rs");
    write_rs(root, "apps/skip.rs");
    write_rs(root, "target-extra/skip.rs");
    write_rs(root, "target_extra/skip.rs");

    let report = InventoryReport::collect(root);
    assert_eq!(
        report.extended.rust_modules, 2,
        "only non-skipped .rs files are counted, got {}",
        report.extended.rust_modules
    );
    for needle in [
        "target/",
        "target-*",
        "node_modules/",
        ".git/",
        "__pycache__/",
        "apps/",
    ] {
        assert!(
            report.exclusions.iter().any(|e| e == needle),
            "exclusions JSON must document {needle}, got {:?}",
            report.exclusions
        );
    }
}

#[test]
fn dup014_close_law() {
    let root = live_root();
    let report = InventoryReport::collect(&root);
    assert_eq!(report.counts.require_needles_fns, 1);
    assert_eq!(report.counts.root_test_binaries, 45);
    assert_eq!(report.counts.tests_rs_autodiscovered, 16);
    assert_eq!(report.counts.tests_contracts_rs, 43);
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    assert!(
        !KNOWN_CHECK_IDS.iter().any(|id| *id == "16"),
        "Guard 16 must not exist"
    );

    let xtask_cargo = read_live("xtask/Cargo.toml");
    assert!(
        !xtask_cargo.contains("[[test]]"),
        "xtask tests stay auto-discovered; no [[test]] harness"
    );
    let root_cargo = read_live("Cargo.toml");
    assert!(
        !root_cargo.contains("sdd_consolidation_c02_dup014"),
        "do not add a root [[test]] for DUP-014"
    );

    let map = load_structural_duplication(&root)
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-014")
        .unwrap_or_else(|| panic!("DUP-014 row missing"));
    assert!(
        row.canonical_owner.contains("model.rs"),
        "canonical_owner must be model.rs, got {}",
        row.canonical_owner
    );
    assert!(
        row.duplicates.is_empty(),
        "close law: duplicates list must be empty once the parallel walk is gone, got {:?}",
        row.duplicates
    );
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c02_dup014")),
        "regression pin is sdd_consolidation_c02_dup014, tests={:?}",
        row.tests
    );
    assert_eq!(row.status, "verified");

    let table = dup014_table();
    assert!(table.contains("walk_tree") || table.contains("SKIP_DIR_NAMES"));
    assert!(
        !row.guard.contains("Guard 16") && !row.guard.contains("guard 16"),
        "uniqueness must not invent a sixteenth guard"
    );
}
