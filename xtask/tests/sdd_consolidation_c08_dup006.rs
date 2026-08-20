//! Lane B DUP-006 — explicit live vs pinned SoA projection paths.

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
fn dup006_live_and_pinned_soa_paths_are_named() {
    let soa = read_live("crates/weeping-angel-assurance/src/soa.rs");
    assert!(
        soa.contains("pub fn project_soa_from_snapshot("),
        "historical reconstruction is project_soa_from_snapshot"
    );
    assert!(
        soa.contains("pub fn project_soa_live("),
        "live convenience is project_soa_live"
    );
    assert!(
        !soa.contains("pub fn project_soa("),
        "ambiguous project_soa name must be gone"
    );

    let scheduler = read_live("crates/weeping-angel-assurance/src/scheduler.rs");
    assert!(
        scheduler.contains("project_soa_live("),
        "scheduler projection uses the live path"
    );
    assert!(
        !scheduler.contains("project_soa("),
        "scheduler must not call the removed ambiguous name"
    );

    let historical = read_live("apps/cli/src/assurance_soa.rs");
    assert!(
        historical.contains("project_soa_from_snapshot"),
        "CLI historical SoA uses the pinned path"
    );
    assert!(
        !historical.contains("project_soa_live("),
        "CLI historical SoA must not call the live projector"
    );

    let lib = read_live("crates/weeping-angel-assurance/src/lib.rs");
    assert!(lib.contains("project_soa_from_snapshot"));
    assert!(lib.contains("project_soa_live"));
}

#[test]
fn dup006_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-006")
        .unwrap_or_else(|| panic!("DUP-006 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "project_soa_from_snapshot");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c08_dup006")),
        "regression pin is sdd_consolidation_c08_dup006, tests={:?}",
        row.tests
    );
}
