//! C03 target — temporal ownership metadata matches physical seats (DUP-017).
//!
//! GREEN when architecture.toml lists both the assurance façade and the
//! control-test leaf primitive, domain-ownership stays split=divided, and
//! `select_latest_as_of` is not moved.

use std::fs;
use std::path::PathBuf;

use xtask::debt::KNOWN_CHECK_IDS;
use xtask::duplication::load_structural_duplication;
use xtask::repo_root_from_xtask_manifest;

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn temporal_ownership_block() -> String {
    let live = read_live("architecture/architecture.toml");
    let idx = live
        .find("[ownership.temporal_evidence_selection]")
        .unwrap_or_else(|| panic!("temporal_evidence_selection row missing"));
    live[idx..].split("\n[").next().unwrap_or("").to_string()
}

#[test]
fn dup017_architecture_toml_lists_both_physical_seats() {
    let block = temporal_ownership_block();
    assert!(
        block.contains("kind = \"exclusive\""),
        "ACP-T07: crate-level kind stays exclusive (façade), not a code move:\n{block}"
    );
    assert!(
        block.contains("crate = \"weeping-angel-assurance\""),
        "crate-level owner of the timeline façade stays assurance:\n{block}"
    );
    assert!(
        block.contains("crates/weeping-angel-assurance/src/temporal.rs"),
        "paths must keep the assurance timeline/diff façade:\n{block}"
    );
    assert!(
        block.contains("crates/weeping-angel-control-test/src/temporal.rs"),
        "paths must list the control-test leaf primitive (DUP-017):\n{block}"
    );
    assert!(
        block.contains("select_latest_as_of") || block.contains("leaf"),
        "row comments must not hide the split:\n{block}"
    );
}

#[test]
fn dup017_primitive_not_moved() {
    let control = read_live("crates/weeping-angel-control-test/src/temporal.rs");
    assert!(
        control.contains("pub fn select_latest_as_of"),
        "evaluation primitive remains in weeping-angel-control-test"
    );
    let assurance = read_live("crates/weeping-angel-assurance/src/temporal.rs");
    assert!(
        !assurance.contains("pub fn select_latest_as_of"),
        "must not move select_latest_as_of into assurance to make metadata true"
    );
    assert!(
        assurance.contains("pub fn project_timeline")
            || assurance.contains("Timeline")
            || assurance.contains("temporal"),
        "assurance temporal.rs stays the timeline/diff projection"
    );
}

#[test]
fn dup017_domain_ownership_stays_divided() {
    let blob = read_live("architecture/domain-ownership.toml");
    let start = blob
        .find("[concept.temporal_evaluation]")
        .unwrap_or_else(|| panic!("temporal_evaluation concept missing"));
    let table = blob[start..].split("\n[").next().unwrap_or("");
    assert!(
        table.contains("split = \"divided\""),
        "temporal_evaluation.split must stay divided:\n{table}"
    );
    assert!(
        table.contains("weeping-angel-control-test"),
        "semantic/evaluation primitive owner is control-test:\n{table}"
    );
    assert!(
        table.contains("select_latest_as_of"),
        "domain-ownership must cite select_latest_as_of:\n{table}"
    );
}

#[test]
fn dup017_close_law() {
    let root = live_root();
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    assert!(
        !KNOWN_CHECK_IDS.iter().any(|id| *id == "16"),
        "Guard 16 must not exist"
    );

    let map = load_structural_duplication(&root)
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-017")
        .unwrap_or_else(|| panic!("DUP-017 row missing"));
    assert!(
        row.duplicates.is_empty(),
        "close law: path-only lie must be gone, duplicates={:?}",
        row.duplicates
    );
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "select_latest_as_of");
    assert!(
        row.tests.iter().any(|t| t.contains("sdd_consolidation_c03_dup017")),
        "regression pin missing, tests={:?}",
        row.tests
    );
}
