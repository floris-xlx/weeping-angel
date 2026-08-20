//! Lane B DUP-011 — one readiness assembly path.

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

fn independently_constructs(text: &str) -> bool {
    text.contains("= FrameworkReadinessSnapshot {")
}

fn rust_src() -> String {
    [
        "crates/weeping-angel-assurance/src/readiness.rs",
        "crates/weeping-angel-assurance/src/drift.rs",
        "crates/weeping-angel-assurance/src/lineage.rs",
        "crates/weeping-angel-assurance/src/scheduler.rs",
    ]
    .into_iter()
    .map(read_live)
    .collect::<Vec<_>>()
    .join("\n")
}

#[test]
fn dup011_only_readiness_rs_assembles_snapshots() {
    let readiness = read_live("crates/weeping-angel-assurance/src/readiness.rs");
    assert!(readiness.contains("fn project_readiness"));
    assert!(readiness.contains("fn empty(") || readiness.contains("fn empty "));
    assert!(readiness.contains("fn from_projected_controls"));

    let drift = read_live("crates/weeping-angel-assurance/src/drift.rs");
    assert!(
        !independently_constructs(&drift),
        "drift must not independently construct FrameworkReadinessSnapshot"
    );
    assert!(drift.contains("from_projected_controls"));

    let lineage = read_live("crates/weeping-angel-assurance/src/lineage.rs");
    assert!(
        !independently_constructs(&lineage),
        "lineage must not independently construct FrameworkReadinessSnapshot"
    );

    let scheduler = read_live("crates/weeping-angel-assurance/src/scheduler.rs");
    assert!(
        !independently_constructs(&scheduler),
        "scheduler must not independently construct FrameworkReadinessSnapshot"
    );
    assert!(
        scheduler.contains("FrameworkReadinessSnapshot::empty"),
        "scheduler empty_readiness must call FrameworkReadinessSnapshot::empty"
    );

    let _ = rust_src();
}

#[test]
fn dup011_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-011")
        .unwrap_or_else(|| panic!("DUP-011 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "project_readiness");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c04_dup011")),
        "regression pin is sdd_consolidation_c04_dup011, tests={:?}",
        row.tests
    );
}
