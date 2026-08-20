//! Lane B DUP-004 — one ApplicabilitySnapshot, no parallel lineage domain type.

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

fn count_pub_struct(text: &str, name: &str) -> usize {
    let needle = format!("pub struct {name}");
    text.matches(&needle).count()
}

#[test]
fn dup004_single_applicability_snapshot_type() {
    let snapshot = read_live("crates/weeping-angel-assurance/src/applicability/snapshot.rs");
    assert_eq!(
        count_pub_struct(&snapshot, "ApplicabilitySnapshot"),
        1,
        "canonical ApplicabilitySnapshot lives in applicability/snapshot.rs"
    );
    let lineage = read_live("crates/weeping-angel-assurance/src/lineage.rs");
    assert_eq!(
        count_pub_struct(&lineage, "ApplicabilitySnapshot"),
        0,
        "lineage must not define ApplicabilitySnapshot"
    );
    assert!(
        !lineage.contains("struct LineageApplicabilitySnapshot"),
        "parallel LineageApplicabilitySnapshot must be gone"
    );
    assert!(
        !lineage.contains("struct LineagePackApplicabilityEntry"),
        "parallel LineagePackApplicabilityEntry must be gone"
    );
    assert!(
        lineage.contains("applicability: ApplicabilitySnapshot"),
        "LineageBundle stores the canonical snapshot"
    );
    assert!(
        snapshot.contains("fn pin_compiled_applicability"),
        "compiled-framework pins go through pin_compiled_applicability"
    );
}

#[test]
fn dup004_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let root = live_root();
    let map = load_structural_duplication(&root)
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-004")
        .unwrap_or_else(|| panic!("DUP-004 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "ApplicabilitySnapshot");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c04_dup004"))
    );
}
