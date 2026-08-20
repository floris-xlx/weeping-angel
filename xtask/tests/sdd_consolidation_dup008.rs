//! DUP-008 — one catalog root-candidate walk.

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

#[test]
fn dup008_one_catalog_root_walk() {
    let catalog = read_live("crates/weeping-angel-canonical-catalog/src/lib.rs");
    assert!(
        catalog.contains("pub fn canonical_catalog_search_roots"),
        "canonical owner is canonical_catalog_search_roots"
    );
    let pin = read_live("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        pin.contains("canonical_catalog_search_roots"),
        "load_catalog_pin consumes canonical roots"
    );
    let snapshot = read_live("crates/weeping-angel-assurance/src/snapshot.rs");
    assert!(
        snapshot.contains("canonical_catalog_search_roots"),
        "catalog_digest consumes canonical roots"
    );
}

#[test]
fn dup008_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-008")
        .unwrap_or_else(|| panic!("DUP-008 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "canonical_catalog_search_roots");
    assert!(
        row.tests.iter().any(|t| t.contains("sdd_consolidation_dup008")),
        "regression pin is sdd_consolidation_dup008, tests={:?}",
        row.tests
    );
}
