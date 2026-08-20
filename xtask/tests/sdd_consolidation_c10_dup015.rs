//! Wave 3 DUP-015 — CollectionEngine is the only envelope collect path.

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
fn dup015_facade_collect_goes_through_engine() {
    let engine = read_live("crates/weeping-angel-collector/src/application/engine.rs");
    assert!(
        engine.contains("pub struct CollectionEngine"),
        "canonical owner is CollectionEngine"
    );
    assert!(
        engine.contains("fn collect_envelopes"),
        "one crate-private envelope collect helper"
    );

    for rel in [
        "crates/weeping-angel-collector/src/local/mod.rs",
        "crates/weeping-angel-collector/src/adapters/fixture.rs",
        "crates/weeping-angel-collector/src/github/mod.rs",
    ] {
        let src = read_live(rel);
        assert!(
            src.contains("collect_envelopes("),
            "{rel} EvidenceCollector::collect must call collect_envelopes"
        );
        assert!(
            !src.contains("EvidenceEnvelope::seal"),
            "{rel} façade must not seal envelopes"
        );
    }

    let scheduler = read_live("crates/weeping-angel-assurance/src/scheduler.rs");
    assert!(
        scheduler.contains("EvidenceCollector"),
        "scheduler keeps the EvidenceCollector façade (no scheduler rewrite)"
    );
}

#[test]
fn dup015_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-015")
        .unwrap_or_else(|| panic!("DUP-015 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "CollectionEngine");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c10_dup015")),
        "regression pin is sdd_consolidation_c10_dup015, tests={:?}",
        row.tests
    );
}
