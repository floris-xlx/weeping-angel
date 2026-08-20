//! Lane C DUP-013 — framework pack applicability.toml is parsed once.

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
fn dup013_one_applicability_toml_parser() {
    let pack = read_live("crates/weeping-angel-framework/src/pack.rs");
    assert!(
        pack.contains("struct PackApplicabilityRow"),
        "canonical typed rows live on LoadedPack"
    );
    assert!(
        pack.contains("fn load_applicability"),
        "framework pack owns the applicability.toml parse"
    );

    let soa = read_live("crates/weeping-angel-assurance/src/soa.rs");
    assert!(
        !soa.contains("toml::from_str") && !soa.contains("applicability.toml"),
        "soa must not re-parse applicability.toml"
    );
    assert!(
        !soa.contains("fn load_pack_soa_rows"),
        "second pack-load helper must be gone"
    );
    assert!(
        soa.contains("fn pack_soa_rows") && soa.contains("PackError::UnknownPack"),
        "soa maps LoadedPack.applicability; UnknownPack is empty rows; other PackError fail-closed"
    );
    assert!(
        soa.contains("OperationalSoaError::PackLoad"),
        "parse/schema/dangling pack errors must not become Vec::new"
    );

    let lib = read_live("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !lib.contains("toml::from_str") && !lib.contains("applicability.toml"),
        "assurance facade must not re-parse applicability.toml"
    );
    assert!(
        lib.contains("pack.applicability") || lib.contains(".applicability"),
        "assess maps LoadedPack.applicability into the lineage pin"
    );
}

#[test]
fn dup013_close_law() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    let map = load_structural_duplication(&live_root())
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-013")
        .unwrap_or_else(|| panic!("DUP-013 row missing"));
    assert!(row.duplicates.is_empty(), "duplicates={:?}", row.duplicates);
    assert_eq!(row.status, "verified");
    assert_eq!(row.canonical_symbol, "LoadedPack.applicability");
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c06_dup013")),
        "regression pin is sdd_consolidation_c06_dup013, tests={:?}",
        row.tests
    );
}
