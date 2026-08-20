//! Lane D DUP-007 — one validity-leaf algorithm; public clocks stay distinct.

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
fn dup007_shared_leaf_helper_without_moving_select_latest_as_of() {
    let control = read_live("crates/weeping-angel-control-test/src/temporal.rs");
    assert!(
        control.contains("pub fn select_latest_as_of"),
        "ACP-T07 / TLE-015: select_latest_as_of stays in control-test"
    );
    assert!(
        control.contains("select_valid_leaf_as_of"),
        "control-test leaf delegates to the shared helper"
    );

    let evidence_validity = read_live("crates/weeping-angel-evidence/src/validity.rs");
    assert!(
        evidence_validity.contains("pub fn select_valid_leaf_as_of"),
        "shared validity-leaf helper lives next to project_validity"
    );
    assert!(
        !evidence_validity.contains("pub fn select_latest_as_of"),
        "must not rename the helper to select_latest_as_of"
    );

    let ledger = read_live("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        ledger.contains("select_valid_leaf_as_of"),
        "ledger as_of leaf uses the shared helper"
    );
    assert!(
        ledger.contains("pub fn as_of(") && ledger.contains("pub fn current(") && ledger.contains("pub fn latest("),
        "public clocks remain distinct"
    );
}

#[test]
fn dup007_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-007")
        .unwrap_or_else(|| panic!("DUP-007 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "select_latest_as_of");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c09_dup007")),
        "regression pin is sdd_consolidation_c09_dup007, tests={:?}",
        row.tests
    );
}
