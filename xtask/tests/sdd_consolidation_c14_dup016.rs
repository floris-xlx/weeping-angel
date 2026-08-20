//! Wave 3 DUP-016 — two workbenches, two scan modules, not one SSOT.

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
fn dup016_workbenches_are_two_products() {
    let rust_wb = read_live("apps/cli/src/workbench/mod.rs");
    assert!(
        rust_wb.contains("CREATE TABLE IF NOT EXISTS scans"),
        "weeping-angel workbench indexes sealed scan directories"
    );
    assert!(
        rust_wb.contains(".weeping-angel") && rust_wb.contains("workbench.sqlite3"),
        "Rust workbench default DB is ~/.weeping-angel/workbench.sqlite3"
    );
    assert!(
        !rust_wb.contains("CREATE TABLE workspaces"),
        "Rust workbench is not the Codex Security workspace/phase DB"
    );

    let py_schema = read_live("codex-security/scripts/workbench_schema.py");
    assert!(
        py_schema.contains("CREATE TABLE workspaces") && py_schema.contains("CREATE TABLE scans"),
        "Codex Security workbench owns workspace/scan/phase schema"
    );
    let py_db = read_live("codex-security/scripts/workbench_db.py");
    assert!(
        py_db.contains("workbench.sqlite3"),
        "Python workbench has its own sqlite file under Codex Security state"
    );
}

#[test]
fn dup016_engine_orchestrator_vs_detector_pack() {
    let orchestrator = read_live("apps/cli/src/engine/mod.rs");
    assert!(
        orchestrator.contains("pub async fn run_scan"),
        "src/engine is the web/HTTP scan orchestrator"
    );
    let detectors = read_live("apps/cli/src/engines/mod.rs");
    assert!(
        detectors.contains("pub struct EngineHit") || detectors.contains("struct EngineHit"),
        "src/engines is the code-SAST detector pack"
    );
    assert!(
        !orchestrator.contains("pub struct EngineHit"),
        "orchestrator must not own detector hit type"
    );
}

#[test]
fn dup016_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-016")
        .unwrap_or_else(|| panic!("DUP-016 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "src/workbench + src/engine");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c14_dup016")),
        "regression pin is sdd_consolidation_c14_dup016, tests={:?}",
        row.tests
    );
}
