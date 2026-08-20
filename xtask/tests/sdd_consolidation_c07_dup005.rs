//! Lane B DUP-005 — public lineage rebuild is replay_assessment only.

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
fn dup005_replay_is_the_only_public_rebuild() {
    let lineage = read_live("crates/weeping-angel-assurance/src/lineage.rs");
    assert!(
        lineage.contains("pub fn replay_assessment("),
        "canonical public rebuild remains replay_assessment"
    );
    assert!(
        !lineage.contains("pub fn reconstruct("),
        "reconstruct must not be public"
    );
    assert!(
        lineage.contains("fn reconstruct("),
        "reconstruct stays a private helper of replay_assessment"
    );
    assert!(
        !lineage.contains("fn load_lineage(") && !lineage.contains("pub fn load_lineage("),
        "load_lineage alias must be deleted"
    );
    assert!(
        !lineage.contains("Ok(reconstruct(bundle))"),
        "replay_assessment must not skip verify_replay_bundle"
    );

    let lib = read_live("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !lib.contains("reconstruct,"),
        "crate root must not re-export reconstruct"
    );
    assert!(
        lib.contains("replay_assessment"),
        "crate root re-exports replay_assessment"
    );
}

#[test]
fn dup005_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-005")
        .unwrap_or_else(|| panic!("DUP-005 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "replay_assessment");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c07_dup005")),
        "regression pin is sdd_consolidation_c07_dup005, tests={:?}",
        row.tests
    );
}
