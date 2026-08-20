//! DUP-001 — Codex Security JSON Schema SSOT is schemas/codex-security.

use std::path::PathBuf;

use xtask::debt::KNOWN_CHECK_IDS;
use xtask::duplication::load_structural_duplication;
use xtask::repo_root_from_xtask_manifest;

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

#[test]
fn dup001_duplicate_schema_tree_gone() {
    let ssot = live_root().join("schemas/codex-security");
    for name in [
        "coverage.schema.json",
        "findings.schema.json",
        "scan-manifest.schema.json",
    ] {
        assert!(
            ssot.join(name).is_file(),
            "SSOT must keep {name} under schemas/codex-security"
        );
        let dup = live_root().join("codex-security/schemas").join(name);
        assert!(
            !dup.is_file(),
            "tracked duplicate {name} must not exist under codex-security/schemas"
        );
    }
}

#[test]
fn dup001_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-001")
        .unwrap_or_else(|| panic!("DUP-001 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "schemas/codex-security");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_dup001")),
        "regression pin is sdd_consolidation_dup001, tests={:?}",
        row.tests
    );
}
