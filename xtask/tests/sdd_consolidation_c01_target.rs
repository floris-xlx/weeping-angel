//! C01 target — one canonical contract-test helper (DUP-002 close law).
//!
//! RED on CURRENT: owner missing, 17 per-file copies, inventory matcher still
//! `contains` (live count 18). GREEN after extract+migrate+delete+matcher
//! tighten: `require_needles_fns == 1`. Keep this file as the uniqueness pin.
//! After GREEN, delete `sdd_consolidation_c01_baseline.rs` (do not ignore it).
//!
//! Split helper-name fragments so this suite does not inflate frozen
//! definition/call counters. Avoid unwrap/expect method-call needles.

use std::fs;
use std::path::{Path, PathBuf};

use xtask::debt::KNOWN_CHECK_IDS;
use xtask::duplication::load_structural_duplication;
use xtask::inventory::InventoryReport;
use xtask::repo_root_from_xtask_manifest;

const HELPER_STEM: &str = "needles";

const CONTRACT_COPIES: [&str; 17] = [
    "tests/contracts/assessment_lineage.target.rs",
    "tests/contracts/control_implementation_registry.target.rs",
    "tests/contracts/controlled_documents.target.rs",
    "tests/contracts/continuity_resilience.target.rs",
    "tests/contracts/iso27001_assurance.target.rs",
    "tests/contracts/interested_parties_obligations.target.rs",
    "tests/contracts/incident_governance.target.rs",
    "tests/contracts/nonconformity_capa.target.rs",
    "tests/contracts/internal_audit.target.rs",
    "tests/contracts/operational_soa.target.rs",
    "tests/contracts/population_runtime.target.rs",
    "tests/contracts/remediation_engine.target.rs",
    "tests/contracts/risk_register.target.rs",
    "tests/contracts/supplier_risk.target.rs",
    "tests/contracts/temporal_lineage_evidence_soa.target.rs",
    "tests/contracts/temporal_assurance.target.rs",
    "tests/contracts/typed_evidence.target.rs",
];

const CONSUMER_BINARIES: [&str; 17] = [
    "sdd_assessment_lineage_target",
    "sdd_control_implementation_registry_target",
    "sdd_controlled_documents_target",
    "sdd_continuity_resilience_target",
    "sdd_iso27001_assurance_target",
    "sdd_interested_parties_obligations_target",
    "sdd_incident_governance_target",
    "sdd_nonconformity_capa_target",
    "sdd_internal_audit_target",
    "sdd_operational_soa_target",
    "sdd_population_runtime_target",
    "sdd_remediation_engine_target",
    "sdd_risk_register_target",
    "sdd_supplier_risk_target",
    "sdd_temporal_lineage_evidence_soa_target",
    "sdd_temporal_assurance_target",
    "sdd_typed_evidence_target",
];

const ISO27001_COPY: &str = "tests/contracts/iso27001_assurance.target.rs";

const OWNER_CANDIDATES: [&str; 2] = ["tests/support/mod.rs", "tests/support/require_needles.rs"];

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn helper_def_needle() -> String {
    format!("fn require_{HELPER_STEM}")
}

fn helper_call_needle() -> String {
    format!("require_{HELPER_STEM}(")
}

fn src_signature() -> String {
    format!("fn require_{HELPER_STEM}(label: &str, src: &str, needles: &[&str])")
}

fn contains_matcher() -> String {
    format!("trimmed.contains(\"{}\")", helper_def_needle())
}

fn starts_with_matcher() -> String {
    format!("trimmed.starts_with(\"{}\")", helper_def_needle())
}

fn line_defines_helper(line: &str) -> bool {
    line.trim_start().starts_with(&helper_def_needle())
}

fn text_defines_helper(text: &str) -> bool {
    text.lines().any(line_defines_helper)
}

fn contract_files_defining_helper(root: &Path) -> Vec<String> {
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
        if text_defines_helper(&text) {
            found.push(format!("tests/contracts/{name}"));
        }
    }
    found.sort();
    found
}

fn owner_definition_files(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for rel in OWNER_CANDIDATES {
        let path = root.join(rel);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        if text_defines_helper(&text) {
            found.push(rel.to_string());
        }
    }
    found
}

fn dup002_table() -> String {
    let debt = read_live("docs/debt/structural-duplication.toml");
    let start = debt
        .find("id = \"DUP-002\"")
        .unwrap_or_else(|| panic!("DUP-002 row missing from structural-duplication.toml"));
    let rest = &debt[start..];
    let end = rest[1..]
        .find("[[duplication]]")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

fn includes_canonical_owner(text: &str) -> bool {
    let mentions_support = text.contains("tests/support")
        || text.contains("../support/")
        || text.contains("support/mod.rs")
        || text.contains("support/require_needles.rs");
    let include_or_path = text.contains("include!") || text.contains("#[path");
    mentions_support && include_or_path
}

fn alias_needles() -> [String; 2] {
    [
        format!(" as require_{HELPER_STEM}"),
        format!("use super::require_{HELPER_STEM}"),
    ]
}

/// C01-T01: live inventory uniqueness after starts_with matcher + extract.
#[test]
fn c01_t01_inventory_reports_one_helper_definition() {
    let report = InventoryReport::collect(&live_root());
    assert_eq!(
        report.counts.require_needles_fns, 1,
        "desired: one definition file after starts_with matcher and copy delete"
    );
    assert_eq!(
        report.extended.duplicate_helper_definitions, 1,
        "duplicate_helper_definitions must alias require_needles_fns at 1"
    );
    assert!(
        report.counts.require_needles_calls <= 222,
        "calls must stay 222 or drop, got {}",
        report.counts.require_needles_calls
    );
}

/// C01-T02: canonical owner is tests/support/ (directory, crate-private signature).
#[test]
fn c01_t02_canonical_owner_is_tests_support_directory() {
    let root = live_root();
    let support_dir = root.join("tests/support");
    assert!(
        support_dir.is_dir(),
        "canonical owner home is tests/support/ directory"
    );
    assert!(
        !root.join("tests/support.rs").is_file(),
        "tests/support.rs would raise tests_rs_autodiscovered"
    );
    assert!(
        !root.join("tests/support/main.rs").is_file(),
        "tests/support/main.rs would become a root test binary"
    );
    assert!(!root.join("tests/sdd").exists(), "tests/sdd/ is forbidden");

    let owners = owner_definition_files(&root);
    assert_eq!(
        owners.len(),
        1,
        "exactly one owner file may define the helper, found {owners:?}"
    );
    let owner_rel = &owners[0];
    let owner_text = read_live(owner_rel);
    assert!(
        owner_text.contains(&src_signature()),
        "{owner_rel} must define {}",
        src_signature()
    );
    assert!(
        owner_text.lines().any(|line| {
            let t = line.trim_start();
            t.starts_with(&src_signature())
        }),
        "{owner_rel} definition must be crate-private (line starts with the signature)"
    );
}

/// C01-T03: all 17 contract binaries include the owner; per-file copies gone.
#[test]
fn c01_t03_seventeen_consumers_migrated_copies_gone() {
    let root = live_root();
    let leftover = contract_files_defining_helper(&root);
    assert!(
        leftover.is_empty(),
        "per-file copies and aliases must be gone, still in {leftover:?}"
    );

    let call = helper_call_needle();
    let aliases = alias_needles();
    for rel in CONTRACT_COPIES {
        let text = read_live(rel);
        assert!(
            includes_canonical_owner(&text),
            "{rel} must include! or #[path] the tests/support helper"
        );
        assert!(
            text.contains(&call),
            "{rel} must keep calling the canonical helper"
        );
        assert!(
            !text_defines_helper(&text),
            "{rel} must not define a local helper"
        );
        for alias in &aliases {
            assert!(
                !text.contains(alias.as_str()),
                "{rel} must not alias the helper ({alias})"
            );
        }
        if rel == ISO27001_COPY {
            assert!(
                !text.contains(&format!(
                    "fn require_{HELPER_STEM}(label: &str, haystack: &str, needles: &[&str])"
                )),
                "iso27001 must use the canonical src signature types, not a haystack copy"
            );
        }
    }

    let owners = owner_definition_files(&root);
    assert_eq!(owners.len(), 1, "owner must exist for consumers to include");
}

/// C01-T04: inventory matcher is starts_with so inventory.rs is not a 2nd def.
#[test]
fn c01_t04_inventory_matcher_is_starts_with() {
    let inventory_src = read_live("xtask/src/inventory.rs");
    assert!(
        inventory_src.contains(&starts_with_matcher()),
        "matcher must be trimmed.starts_with on the helper definition substring"
    );
    assert!(
        !inventory_src.contains(&contains_matcher()),
        "contains matcher would keep counting inventory.rs itself"
    );

    let frozen = read_live("docs/debt/consolidation-baseline.json");
    assert!(
        frozen.contains("\"require_needles_fns\": 18"),
        "do not rebase Phase 0 freeze; live 1 is a legal decrease"
    );
}

/// C01-T05: expansion-adjacent counts must not increase; no Guard 16.
#[test]
fn c01_t05_expansion_counts_do_not_increase() {
    let report = InventoryReport::collect(&live_root());
    assert_eq!(report.counts.root_test_binaries, 1);
    assert_eq!(report.counts.tests_rs_autodiscovered, 16);
    assert_eq!(report.counts.tests_contracts_rs, 43);
    assert!(
        report.extended.public_symbols <= 2043,
        "public_symbols must not increase (live ceiling 2043 after DUP-007 shared leaf; freeze 2022), got {}",
        report.extended.public_symbols
    );
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
        !root_cargo.contains("sdd_consolidation_c01"),
        "do not add a root [[test]] for C01"
    );
    let harness = read_live("apps/cli/tests/harness.rs");
    for path in CONTRACT_COPIES {
        let file = path.rsplit('/').next().unwrap_or(path);
        assert!(
            harness.contains(file),
            "consumer {file} must remain a harness module"
        );
    }

    let baseline = live_root().join("xtask/tests/sdd_consolidation_c01_baseline.rs");
    if baseline.is_file() {
        let text =
            fs::read_to_string(&baseline).unwrap_or_else(|e| panic!("read C01 baseline: {e}"));
        assert!(
            !text.contains("#[ignore"),
            "do not #[ignore] the C01 baseline; delete it after target GREEN"
        );
    }
}

/// C01-T06: DUP-002 verified only when owner exists, consumers migrated, copies gone, guard exists.
#[test]
fn c01_t06_dup002_close_law() {
    let root = live_root();
    let leftover = contract_files_defining_helper(&root);
    let owners = owner_definition_files(&root);
    assert!(
        leftover.is_empty(),
        "close law: old copies gone, still {leftover:?}"
    );
    assert_eq!(
        owners.len(),
        1,
        "close law: canonical owner exists under tests/support/, found {owners:?}"
    );

    let map = load_structural_duplication(&root)
        .unwrap_or_else(|e| panic!("load structural-duplication.toml: {e}"));
    let row = map
        .rows
        .iter()
        .find(|r| r.id == "DUP-002")
        .unwrap_or_else(|| panic!("DUP-002 row missing"));
    assert_eq!(row.canonical_symbol, format!("require_{HELPER_STEM}"));
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
    let table = dup002_table();
    for name in CONSUMER_BINARIES {
        assert!(table.contains(name), "DUP-002 consumers must list {name}");
    }
    assert!(
        row.tests
            .iter()
            .any(|t| t.contains("sdd_consolidation_c01_target")),
        "regression pin is the staying C01 target, tests={:?}",
        row.tests
    );
    assert!(
        row.guard.contains("04") || row.guard.to_ascii_lowercase().contains("guard 04"),
        "guard must cite Guard 04 expansion freeze, got {}",
        row.guard
    );
    assert!(
        row.guard.to_ascii_lowercase().contains("inventory") || row.guard.contains("starts_with"),
        "guard must cite inventory uniqueness, got {}",
        row.guard
    );
    assert!(
        !row.guard.contains("Guard 16") && !row.guard.contains("guard 16"),
        "uniqueness must not invent Guard 16"
    );
    assert_eq!(
        row.status, "verified",
        "verified only when close law holds (owner, consumers, copies gone, guard)"
    );
}
