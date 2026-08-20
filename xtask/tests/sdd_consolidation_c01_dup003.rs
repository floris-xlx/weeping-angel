//! DUP-003 pin — one filesystem/discovery helper module (same owner as DUP-002).
//!
//! GREEN when tests/support/mod.rs owns manifest_dir / read_repo_file /
//! crate_sources_joined / text_has and tests/contracts copies are gone.

use std::fs;
use std::path::{Path, PathBuf};

use xtask::debt::KNOWN_CHECK_IDS;
use xtask::duplication::load_structural_duplication;
use xtask::inventory::InventoryReport;
use xtask::repo_root_from_xtask_manifest;

const OWNER: &str = "tests/support/mod.rs";
const INCLUDE_NEEDLE: &str = "tests/support/mod.rs";
const HELPER_STEM: &str = "needles";

const FS_HELPERS: [&str; 4] = [
    "fn manifest_dir",
    "fn read_repo_file",
    "fn crate_sources_joined",
    "fn text_has",
];

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn line_defines(line: &str, helper: &str) -> bool {
    line.trim_start().starts_with(helper)
}

fn text_defines(text: &str, helper: &str) -> bool {
    text.lines().any(|line| line_defines(line, helper))
}

fn contract_files_defining(root: &Path, helper: &str) -> Vec<String> {
    let dir = root.join("tests/contracts");
    let mut found = Vec::new();
    let entries = fs::read_dir(&dir).unwrap_or_else(|e| panic!("read tests/contracts: {e}"));
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !name.ends_with(".target.rs") {
            continue;
        }
        let text =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        if text_defines(&text, helper) {
            found.push(format!("tests/contracts/{name}"));
        }
    }
    found.sort();
    found
}

fn dup003_table() -> String {
    let debt = read_live("docs/debt/structural-duplication.toml");
    let start = debt
        .find("id = \"DUP-003\"")
        .unwrap_or_else(|| panic!("DUP-003 row missing from structural-duplication.toml"));
    let rest = &debt[start..];
    let end = rest[1..]
        .find("[[duplication]]")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn dup003_owner_defines_filesystem_helpers() {
    let root = live_root();
    assert!(
        root.join("tests/support").is_dir(),
        "same DUP-002 owner directory"
    );
    assert!(!root.join("tests/support.rs").is_file());
    assert!(!root.join("tests/sdd").exists());

    let owner = read_live(OWNER);
    for helper in FS_HELPERS {
        assert!(text_defines(&owner, helper), "{OWNER} must define {helper}");
    }
    assert!(
        owner.contains(&format!(
            "fn require_{HELPER_STEM}(label: &str, src: &str, needles: &[&str])"
        )),
        "DUP-002 close law: needle helper stays in the same module"
    );
    let vis = "pub";
    for name in [
        "manifest_dir",
        "read_repo_file",
        "crate_sources_joined",
        "text_has",
    ] {
        assert!(
            !owner.contains(&format!("{vis} fn {name}")),
            "filesystem helpers must stay crate-private ({name})"
        );
    }
}

#[test]
fn dup003_contract_copies_gone() {
    let root = live_root();
    for helper in FS_HELPERS {
        let leftover = contract_files_defining(&root, helper);
        assert!(
            leftover.is_empty(),
            "per-file {helper} copies must be gone, still in {leftover:?}"
        );
    }

    let dir = root.join("tests/contracts");
    let entries = fs::read_dir(&dir).unwrap_or_else(|e| panic!("read tests/contracts: {e}"));
    let mut consumers = 0u32;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !name.ends_with(".target.rs") {
            continue;
        }
        let text =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let uses_fs = [
            "manifest_dir(",
            "read_repo_file(",
            "crate_sources_joined(",
            "text_has(",
        ]
        .iter()
        .any(|call| text.contains(call));
        if !uses_fs {
            continue;
        }
        consumers += 1;
        assert!(
            text.contains("include!") && text.contains(INCLUDE_NEEDLE),
            "tests/contracts/{name} must include! {INCLUDE_NEEDLE}"
        );
        for helper in FS_HELPERS {
            assert!(
                !text_defines(&text, helper),
                "tests/contracts/{name} must not define {helper}"
            );
            let alias = format!(" as {}", helper.trim_start_matches("fn "));
            assert!(
                !text.contains(&alias),
                "tests/contracts/{name} must not alias {helper}"
            );
        }
    }
    assert!(
        consumers >= 17,
        "expected the historical 17+ contract consumers, got {consumers}"
    );
}

#[test]
fn dup003_close_law_and_dup002_still_holds() {
    let root = live_root();
    let report = InventoryReport::collect(&root);
    assert_eq!(
        report.counts.require_needles_fns, 1,
        "DUP-002 close law: one require_needles definition"
    );
    assert_eq!(report.counts.root_test_binaries, 1);
    assert_eq!(report.counts.tests_rs_autodiscovered, 16);
    assert_eq!(report.counts.tests_contracts_rs, 43);
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    assert!(
        !KNOWN_CHECK_IDS.iter().any(|id| *id == "16"),
        "Guard 16 must not exist"
    );

    let xtask_cargo = read_live("xtask/Cargo.toml");
    assert!(
        !xtask_cargo.contains("[[test]]"),
        "xtask tests stay auto-discovered; no [[test]] harness"
    );
    let root_cargo = read_live("Cargo.toml");
    assert!(
        !root_cargo.contains("sdd_consolidation_c01_dup003"),
        "do not add a root [[test]] for DUP-003"
    );

    let map = load_structural_duplication(&root)
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-003")
        .unwrap_or_else(|| panic!("DUP-003 row missing"));
    assert_eq!(row.canonical_symbol, "tests/support::filesystem_helpers");
    assert!(
        row.canonical_owner.contains("tests/support"),
        "canonical_owner must be tests/support, got {}",
        row.canonical_owner
    );
    assert!(
        !row.canonical_owner.contains("proposed"),
        "owner must exist, not remain proposed"
    );
    assert!(
        row.duplicates.is_empty(),
        "close law: duplicates list must be empty once copies are gone, got {:?}",
        row.duplicates
    );
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c01_dup003")),
        "regression pin is sdd_consolidation_c01_dup003, tests={:?}",
        row.tests
    );
    assert_eq!(row.status, "verified");

    let table = dup003_table();
    assert!(table.contains("tests/support"));
    assert!(
        !row.guard.contains("Guard 16") && !row.guard.contains("guard 16"),
        "uniqueness must not invent a sixteenth guard"
    );
}
