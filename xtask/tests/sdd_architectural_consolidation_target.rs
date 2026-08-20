//! Target suite for Architectural Consolidation Program Phase 0.
//!
//! Encodes CON-T01–T10 (consolidation mode, frozen baseline, v2 backlog).
//! Must FAIL (RED) on CURRENT pre-implement code because those three
//! artifacts/schema/enforcement are absent. Do not implement Phase 0.1–0.3 here.
//!
//! Avoid unwrap/expect method-call needles so inventory counts stay CURRENT.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;
use xtask::architecture::{ARCH_SCHEMA, load_architecture_manifest};
use xtask::check_01_architecture_manifest;
use xtask::check_04_architecture_invariants;
use xtask::debt::KNOWN_CHECK_IDS;
use xtask::inventory::{INVENTORY_SCHEMA, InventoryReport};
use xtask::{CheckStatus, repo_root_from_xtask_manifest, run_guard};

const CONSOLIDATION_BASELINE_SCHEMA: &str = "weeping-angel/consolidation-baseline/v1";
const DUPLICATION_V2_SCHEMA: &str = "weeping-angel/structural-duplication/v2";

const ALLOWED_CHANGE_CLASSES: [&str; 5] = [
    "bug-fix",
    "security-fix",
    "consolidation",
    "non-semantic-collector",
    "consolidation-docs",
];

const FORBIDDEN_CHANGE_CLASSES: [&str; 7] = [
    "new-public-domain-type",
    "new-persistence-representation",
    "new-projection-path",
    "new-root-test-binary",
    "new-duplicated-helper",
    "new-compatibility-alias",
    "second-ssot",
];

const V2_ROW_FIELDS: [&str; 13] = [
    "id",
    "concept",
    "severity",
    "canonical_owner",
    "canonical_symbol",
    "duplicates",
    "migration_state",
    "removal_blockers",
    "public_api_impact",
    "serialization_impact",
    "tests",
    "guard",
    "status",
];

const V2_STATUSES: [&str; 7] = [
    "candidate",
    "confirmed",
    "canonicalized",
    "consumers-migrating",
    "compatibility-only",
    "removed",
    "verified",
];

const V1_RETIRED_STATUSES: [&str; 3] = ["migrating", "resolved", "false-positive"];

const CONSOLIDATION_INVARIANTS: [&str; 4] = [
    "INV-CONSOLIDATION-MODE-ACTIVE",
    "INV-CONSOLIDATION-BASELINE-PRESENT",
    "INV-CONSOLIDATION-EXPANSION-RESTRICTED",
    "INV-STRUCTURAL-DUPLICATION-BACKLOG",
];

const INVENTORY_COUNT_KEYS: [&str; 13] = [
    "root_test_binaries",
    "tests_rs_autodiscovered",
    "tests_contracts_rs",
    "ignored_test_attrs",
    "unwrap_calls",
    "expect_calls",
    "unwrap_plus_expect",
    "require_needles_fns",
    "require_needles_calls",
    "adr_markdown_files",
    "catalog_test_toml",
    "framework_packs",
    "schema_json_files",
];

const EXTENDED_KEYS: [&str; 8] = [
    "workspace_crates",
    "rust_modules",
    "public_symbols",
    "pub_use_count",
    "public_structs",
    "public_enums",
    "duplicate_helper_definitions",
    "duplicate_type_names",
];

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn architecture_src() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/architecture.rs"))
        .unwrap_or_else(|e| panic!("read xtask/src/architecture.rs: {e}"))
}

fn xtask_src_tree() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = String::new();
    fn walk(dir: &Path, out: &mut String) {
        let entries =
            fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push_str(
                    &fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
                );
                out.push('\n');
            }
        }
    }
    walk(&root, &mut out);
    out
}

fn toml_string_array(table: &toml::value::Table, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn program_table(text: &str) -> toml::value::Table {
    let parsed: toml::Value = text
        .parse()
        .unwrap_or_else(|e| panic!("architecture.toml parses: {e}"));
    let program = parsed
        .get("program")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("architecture.toml missing [program] table"));
    program
        .get("architectural_consolidation")
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_else(|| {
            panic!("architecture.toml missing [program.architectural_consolidation]")
        })
}

fn is_fail(status: &CheckStatus) -> bool {
    matches!(status, CheckStatus::Fail(_))
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("mkdir {}: {e}", parent.display()));
    }
    fs::write(path, contents).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

fn copy_live(rel: &str, dest_root: &Path) {
    write_file(&dest_root.join(rel), &read_live(rel));
}

fn production_rs_mentioning(needle: &str) -> Vec<String> {
    let root = live_root();
    let mut hits = Vec::new();
    fn walk(dir: &Path, root: &Path, needle: &str, hits: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if path.is_dir() {
                if matches!(
                    name,
                    "target" | "node_modules" | ".git" | "apps" | "__pycache__" | "tests"
                ) {
                    continue;
                }
                walk(&path, root, needle, hits);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            if text.contains(needle) {
                let rel = path
                    .strip_prefix(root)
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| path.display().to_string());
                hits.push(rel);
            }
        }
    }
    for rel in ["xtask/src", "crates", "src"] {
        walk(&root.join(rel), &root, needle, &mut hits);
    }
    hits.sort();
    hits
}

/// CON-T01: program table is parsed into ArchitectureManifest with restricted freeze.
#[test]
fn con_t01_program_table_parsed_active_restricted() {
    let text = read_live("architecture/architecture.toml");
    assert!(
        text.contains(&format!("schema = \"{ARCH_SCHEMA}\"")),
        "architecture.toml schema stays {ARCH_SCHEMA}"
    );
    assert!(
        text.contains("[program.architectural_consolidation]"),
        "architecture.toml must declare [program.architectural_consolidation]"
    );

    let table = program_table(&text);
    assert_eq!(
        table.get("status").and_then(|v| v.as_str()),
        Some("active"),
        "status must be active"
    );
    assert_eq!(
        table.get("feature_expansion").and_then(|v| v.as_str()),
        Some("restricted"),
        "feature_expansion must be restricted while status=active"
    );
    let allowed = toml_string_array(&table, "allowed_change_classes");
    let forbidden = toml_string_array(&table, "forbidden_change_classes");
    assert!(
        !allowed.is_empty(),
        "allowed_change_classes must be non-empty"
    );
    assert!(
        !forbidden.is_empty(),
        "forbidden_change_classes must be non-empty"
    );
    for class in ALLOWED_CHANGE_CLASSES {
        assert!(
            allowed.iter().any(|c| c == class),
            "allowed_change_classes must include {class}: {allowed:?}"
        );
    }
    for class in FORBIDDEN_CHANGE_CLASSES {
        assert!(
            forbidden.iter().any(|c| c == class),
            "forbidden_change_classes must include {class}: {forbidden:?}"
        );
    }

    let src = architecture_src();
    assert!(
        src.contains("architectural_consolidation")
            || src.contains("ConsolidationProgram")
            || src.contains("consolidation:"),
        "ArchitectureManifest must parse [program.architectural_consolidation]"
    );
    assert!(
        src.contains("get(\"program\")")
            || src.contains("get(\"architectural_consolidation\")")
            || src.contains("[program.architectural_consolidation]"),
        "load_architecture_manifest must read the program table, not ignore extra TOML"
    );

    let live = load_architecture_manifest(&live_root()).unwrap_or_else(|e| {
        panic!("live architecture.toml must load once the table is valid: {e}")
    });
    let debug = format!("{live:?}").to_ascii_lowercase();
    assert!(
        debug.contains("consolidat") && debug.contains("active") && debug.contains("restricted"),
        "ArchitectureManifest Debug must include parsed consolidation mode: {debug}"
    );

    let invariants = read_live("architecture/invariants.toml");
    assert!(
        invariants.contains("INV-CONSOLIDATION-MODE-ACTIVE"),
        "Guard 04 must list INV-CONSOLIDATION-MODE-ACTIVE"
    );
}

/// CON-T02: missing/malformed program table fails Guard 01 (and/or 04); paper TOML is not a pass.
#[test]
fn con_t02_missing_or_malformed_program_table_fails_closed() {
    let dir = tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let mut missing_toml = read_live("architecture/architecture.toml");
    if let Some(idx) = missing_toml.find("[program") {
        missing_toml = missing_toml[..idx].trim_end().to_string();
        missing_toml.push('\n');
    }
    write_file(
        &dir.path().join("architecture/architecture.toml"),
        &missing_toml,
    );
    let missing = load_architecture_manifest(dir.path());
    assert!(
        missing.is_err(),
        "missing [program.architectural_consolidation] must fail closed, got {missing:?}"
    );
    let c01 = check_01_architecture_manifest(dir.path());
    assert!(
        is_fail(&c01.status),
        "Guard 01 must fail when the program table is missing: {:?}",
        c01.status
    );

    let mut paper = read_live("architecture/architecture.toml");
    if !paper.contains("[program.architectural_consolidation]") {
        paper.push_str(
            r#"

[program.architectural_consolidation]
status = "bogus"
feature_expansion = "unrestricted"
allowed_change_classes = []
forbidden_change_classes = []
"#,
        );
    } else {
        paper = paper.replace("status = \"active\"", "status = \"bogus\"");
        paper = paper.replace(
            "feature_expansion = \"restricted\"",
            "feature_expansion = \"unrestricted\"",
        );
    }
    write_file(&dir.path().join("architecture/architecture.toml"), &paper);
    let malformed = load_architecture_manifest(dir.path());
    assert!(
        malformed.is_err(),
        "malformed consolidation enums/empty class arrays must fail closed, got {malformed:?}"
    );
    let c01_bad = check_01_architecture_manifest(dir.path());
    assert!(
        is_fail(&c01_bad.status),
        "Guard 01 must fail on malformed program table: {:?}",
        c01_bad.status
    );
}

/// CON-T03: frozen consolidation-baseline.json and .md exist with v1 schema + §5.2 keys.
#[test]
fn con_t03_consolidation_baseline_artifacts_exist() {
    let json_path = live_root().join("docs/debt/consolidation-baseline.json");
    let md_path = live_root().join("docs/debt/consolidation-baseline.md");
    assert!(
        json_path.is_file(),
        "docs/debt/consolidation-baseline.json must exist"
    );
    assert!(
        md_path.is_file(),
        "docs/debt/consolidation-baseline.md must exist"
    );

    let json_text = fs::read_to_string(&json_path)
        .unwrap_or_else(|e| panic!("read consolidation-baseline.json: {e}"));
    let json: serde_json::Value = serde_json::from_str(&json_text)
        .unwrap_or_else(|e| panic!("consolidation-baseline.json parses: {e}"));
    assert_eq!(
        json.get("schema").and_then(|v| v.as_str()),
        Some(CONSOLIDATION_BASELINE_SCHEMA)
    );
    assert_eq!(
        json.get("program").and_then(|v| v.as_str()),
        Some("architectural-consolidation")
    );
    assert_eq!(json.get("phase").and_then(|v| v.as_i64()), Some(0));
    assert_eq!(
        json.get("source").and_then(|v| v.as_str()),
        Some(INVENTORY_SCHEMA)
    );
    let exclusions = json
        .get("exclusions")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("exclusions array"));
    let exclusion_text: Vec<&str> = exclusions.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        exclusion_text.iter().any(|s| s.contains("target/")),
        "exclusions must include inventory target/: {exclusion_text:?}"
    );
    assert!(
        exclusion_text.iter().any(|s| s.contains("node_modules")),
        "exclusions must include node_modules/: {exclusion_text:?}"
    );

    let counts = json
        .get("inventory_counts")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("inventory_counts object"));
    for key in INVENTORY_COUNT_KEYS {
        assert!(counts.contains_key(key), "inventory_counts missing {key}");
    }
    let extended = json
        .get("extended")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("extended object"));
    for key in EXTENDED_KEYS {
        assert!(extended.contains_key(key), "extended missing {key}");
        assert!(
            extended[key].as_u64().is_some() || extended[key].as_i64().is_some(),
            "extended.{key} must be an integer"
        );
    }
    assert!(
        json.get("architecture_ownership")
            .and_then(|v| v.as_array())
            .is_some(),
        "architecture_ownership array"
    );
    assert!(
        json.get("schema_locations")
            .and_then(|v| v.as_array())
            .is_some(),
        "schema_locations array"
    );
    assert!(json.get("adr_count").and_then(|v| v.as_i64()).is_some());
    assert!(json.get("spec_count").and_then(|v| v.as_i64()).is_some());
    assert!(json.get("debt_rows").and_then(|v| v.as_i64()).is_some());

    let md = fs::read_to_string(&md_path)
        .unwrap_or_else(|e| panic!("read consolidation-baseline.md: {e}"));
    assert!(
        md.contains("weeping-angel-consolidation-baseline-stable"),
        "markdown must carry the frozen stable marker"
    );
    assert!(
        md.to_ascii_lowercase().contains("frozen") && md.contains("current.md"),
        "markdown must state it is the frozen Phase 0 snapshot, not live current.md"
    );
}

/// CON-T04: frozen inventory_counts share keys with InventoryReport; current.md stays live.
#[test]
fn con_t04_frozen_counts_share_inventory_keys_current_md_live() {
    let report = InventoryReport::collect(&live_root());
    report
        .check_current_md(&live_root())
        .unwrap_or_else(|e| panic!("docs/debt/current.md remains the live snapshot: {e}"));
    assert_eq!(report.schema, INVENTORY_SCHEMA);

    let json_path = live_root().join("docs/debt/consolidation-baseline.json");
    assert!(
        json_path.is_file(),
        "frozen consolidation-baseline.json is required to compare inventory_counts"
    );
    let json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&json_path)
            .unwrap_or_else(|e| panic!("read consolidation-baseline.json: {e}")),
    )
    .unwrap_or_else(|e| panic!("json: {e}"));
    let frozen = json
        .get("inventory_counts")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("inventory_counts"));
    let live_json: serde_json::Value =
        serde_json::from_str(&report.to_json()).unwrap_or_else(|e| panic!("inventory json: {e}"));
    let live_counts = live_json
        .get("counts")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("live counts"));
    for key in INVENTORY_COUNT_KEYS {
        assert!(frozen.contains_key(key), "frozen inventory_counts.{key}");
        assert!(live_counts.contains_key(key), "live counts.{key}");
        let frozen_n = frozen[key]
            .as_u64()
            .or_else(|| frozen[key].as_i64().and_then(|n| u64::try_from(n).ok()));
        let live_n = live_counts[key].as_u64();
        assert!(
            frozen_n.is_some() && live_n.is_some(),
            "shared count {key} must be numeric"
        );
    }
    let current = read_live("docs/debt/current.md");
    assert!(
        current.contains("weeping-angel-inventory-stable"),
        "current.md keeps the live inventory marker"
    );
    assert!(
        !current.contains("weeping-angel-consolidation-baseline-stable"),
        "current.md must not become the frozen consolidation snapshot"
    );
}

/// CON-T05: structural-duplication.toml is v2 with required fields and the new status set.
#[test]
fn con_t05_structural_duplication_v2_required_fields() {
    let text = read_live("docs/debt/structural-duplication.toml");
    let parsed: toml::Value = text
        .parse()
        .unwrap_or_else(|e| panic!("structural-duplication.toml parses: {e}"));
    assert_eq!(
        parsed.get("schema").and_then(|v| v.as_str()),
        Some(DUPLICATION_V2_SCHEMA)
    );
    assert_eq!(
        parsed.get("program").and_then(|v| v.as_str()),
        Some("architectural-consolidation")
    );
    assert_eq!(parsed.get("phase").and_then(|v| v.as_integer()), Some(0));

    let rows = parsed
        .get("duplication")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("[[duplication]] array"));
    assert_eq!(rows.len(), 17, "keep all 17 DUP-001..017 rows");

    let allowed: BTreeSet<&str> = V2_STATUSES.into_iter().collect();
    let mut ids = Vec::new();
    for row in rows {
        let table = row
            .as_table()
            .unwrap_or_else(|| panic!("duplication row is a table"));
        let id = table
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("row missing id"))
            .to_string();
        for field in V2_ROW_FIELDS {
            assert!(
                table.contains_key(field),
                "{id} missing required v2 field {field}"
            );
        }
        let status = table
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("{id} missing status"));
        assert!(
            allowed.contains(status),
            "{id} status {status} is not in the v2 closed set"
        );
        for retired in V1_RETIRED_STATUSES {
            assert_ne!(
                status, retired,
                "{id} still uses retired v1 status {retired}"
            );
        }
        ids.push(id);
    }
    let expected: Vec<String> = (1..=17).map(|n| format!("DUP-{n:03}")).collect();
    assert_eq!(ids, expected);

    let rust_hits = production_rs_mentioning("structural-duplication");
    assert!(
        !rust_hits.is_empty(),
        "xtask must parse structural-duplication.toml (found no production mention)"
    );
}

/// CON-T06: close law; v1 resolved/false-positive never auto-map to verified/removed.
#[test]
fn con_t06_close_law_blocks_verified_removed() {
    let text = read_live("docs/debt/structural-duplication.toml");
    let parsed: toml::Value = text
        .parse()
        .unwrap_or_else(|e| panic!("structural-duplication.toml parses: {e}"));
    let rows = parsed
        .get("duplication")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("[[duplication]] array"));

    let mut by_id = std::collections::BTreeMap::new();
    for row in rows {
        let table = row.as_table().unwrap_or_else(|| panic!("row table"));
        let id = table
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("id"))
            .to_string();
        let status = table
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("{id} status"))
            .to_string();
        let symbol = table
            .get("canonical_symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let tests_empty = table
            .get("tests")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true);
        let guard = table
            .get("guard")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let duplicates = table
            .get("duplicates")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if status == "verified" || status == "removed" {
            assert!(
                !symbol.is_empty() && symbol != "unknown",
                "{id} close law: canonical_symbol must exist (not unknown)"
            );
            assert!(
                !tests_empty || !guard.is_empty(),
                "{id} close law: tests or guard must pin a regression"
            );
            if status == "removed" {
                assert!(
                    duplicates.is_empty(),
                    "{id} removed must not still list tracked duplicate paths"
                );
            }
        }
        by_id.insert(id, (status, symbol));
    }

    for id in ["DUP-001", "DUP-008", "DUP-010", "DUP-013"] {
        let (status, _) = by_id.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_ne!(
            status.as_str(),
            "verified",
            "{id} v1 resolved must not silently become verified"
        );
        assert_ne!(
            status.as_str(),
            "removed",
            "{id} v1 resolved must not silently become removed"
        );
        assert!(
            matches!(
                status.as_str(),
                "canonicalized" | "consumers-migrating" | "compatibility-only"
            ),
            "{id} v1 resolved maps to canonicalized|consumers-migrating|compatibility-only, got {status}"
        );
    }
    for id in ["DUP-004", "DUP-005"] {
        let (status, _) = by_id.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(
            status.as_str(),
            "consumers-migrating",
            "{id} v1 migrating → consumers-migrating, got {status}"
        );
    }
    for id in ["DUP-009", "DUP-012"] {
        let (status, _) = by_id.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_ne!(status.as_str(), "verified");
        assert_ne!(status.as_str(), "removed");
    }

    let src = xtask_src_tree();
    assert!(
        src.contains("canonical_symbol")
            && (src.contains("\"verified\"") || src.contains("verified")),
        "close law must be enforced in the xtask parser"
    );
}

/// CON-T07: dual-suite under xtask/tests; no tests/sdd/; no new root [[test]].
#[test]
fn con_t07_suites_under_xtask_tests_no_root_harness() {
    let xtask_tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    assert!(
        xtask_tests
            .join("sdd_architectural_consolidation_target.rs")
            .is_file(),
        "target suite must live under xtask/tests/"
    );
    assert!(
        !xtask_tests
            .join("sdd_architectural_consolidation_baseline.rs")
            .exists(),
        "superseded xtask baseline suite must be deleted"
    );
    assert!(
        !live_root().join("tests/sdd").exists(),
        "tests/sdd/ is forbidden (ADR 0004 / FORBID-TESTS-SDD)"
    );
    assert!(
        !live_root().join("test/sdd").exists(),
        "test/sdd/*.ts is not a dual-suite home"
    );
    let root_cargo = read_live("Cargo.toml");
    assert!(
        !root_cargo.contains("sdd_architectural_consolidation"),
        "do not add a new root [[test]] for this program"
    );
    let xtask_cargo = read_live("xtask/Cargo.toml");
    assert!(
        !xtask_cargo.contains("[[test]]"),
        "xtask dual-suite is auto-discovered; do not add [[test]]"
    );
}

/// CON-T08: spec listed in CANONICAL_SPECS + spec-lifecycle; ADR 0049 meta id.
#[test]
fn con_t08_spec_lifecycle_and_adr_0049() {
    let layout = read_live("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/architectural-consolidation-program.md"),
        "spec must remain in CANONICAL_SPECS"
    );
    let life = read_live("architecture/spec-lifecycle.toml");
    assert!(
        life.contains("docs/specs/architectural-consolidation-program.md"),
        "Guard 15 lifecycle must list the consolidation spec"
    );
    assert!(
        life.contains("ownership = [\"repository_guard\"]") || life.contains("repository_guard"),
        "lifecycle ownership is repository_guard"
    );
    let adr_path = live_root().join("docs/adr/0049-architectural-consolidation-phase-0.md");
    assert!(adr_path.is_file(), "ADR 0049 file must exist");
    let adr = fs::read_to_string(&adr_path).unwrap_or_else(|e| panic!("read ADR 0049: {e}"));
    assert!(
        adr.contains("id = \"0049\""),
        "ADR 0049 weeping-angel-adr-meta must set id = \"0049\""
    );
}

/// CON-T09: increasing a frozen expansion metric fails Guard 04 (not Guard 16).
#[test]
fn con_t09_expansion_increase_fails_guard_04() {
    let invariants = read_live("architecture/invariants.toml");
    for id in CONSOLIDATION_INVARIANTS {
        assert!(
            invariants.contains(id),
            "architecture/invariants.toml must declare {id}"
        );
    }
    let src = xtask_src_tree();
    assert!(
        src.contains("INV-CONSOLIDATION-EXPANSION-RESTRICTED"),
        "evaluate_invariant must have a predicate for expansion restriction"
    );
    assert!(
        !src.contains("\"16\"") || src.contains("KNOWN_CHECK_IDS: [&str; 15]"),
        "do not add Guard 16"
    );
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    assert!(
        !src.contains("cargo xtask consolidation") && !src.contains("Some(\"consolidation\")"),
        "do not add a second health CLI"
    );

    let dir = tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    seed_expansion_fixture(dir.path());
    let c04 = check_04_architecture_invariants(dir.path());
    match &c04.status {
        CheckStatus::Fail(msg) => {
            assert!(
                !msg.contains("unknown invariant"),
                "expansion must be a real predicate, not unknown-id fail-closed: {msg}"
            );
            assert!(
                msg.contains("root_test_binaries")
                    || msg.contains("schema_json")
                    || msg.to_ascii_lowercase().contains("expansion"),
                "Guard 04 fail must name the expansion metric: {msg}"
            );
        }
        other => panic!("increasing a frozen expansion metric must fail Guard 04, got {other:?}"),
    }
}

/// CON-T10: neighbor suites and live guard 01–15 stay green; ADR remains Draft until GREEN.
#[test]
fn con_t10_neighbors_and_live_guard_stay_green() {
    let xtask_tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    assert!(
        xtask_tests
            .join("sdd_architectural_cleanup_target.rs")
            .is_file()
    );
    assert!(
        xtask_tests
            .join("sdd_structural_reconciliation_target.rs")
            .is_file()
    );

    let report = run_guard(&live_root());
    let rendered = report.render();
    assert!(!report.failed(), "{rendered}");
    for id in [
        "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    ] {
        let check = report
            .checks
            .iter()
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("missing check {id}: {rendered}"));
        assert_eq!(check.status, CheckStatus::Pass, "{rendered}");
    }
    assert!(
        !report.checks.iter().any(|c| c.id == "16"),
        "no Guard 16: {rendered}"
    );
}

fn seed_expansion_fixture(root: &Path) {
    copy_live("architecture/architecture.toml", root);
    let mut invariants = read_live("architecture/invariants.toml");
    if !invariants.contains("INV-CONSOLIDATION-EXPANSION-RESTRICTED") {
        invariants.push_str(
            r#"

[[invariant]]
id = "INV-CONSOLIDATION-EXPANSION-RESTRICTED"
summary = "Live expansion metrics must not increase versus the frozen consolidation baseline"
guard_check = "04"
"#,
        );
    }
    write_file(&root.join("architecture/invariants.toml"), &invariants);
    copy_live("architecture/forbidden-patterns.toml", root);
    copy_live("architecture/adr-identity.toml", root);
    copy_live("architecture/spec-lifecycle.toml", root);
    copy_live("docs/debt/register.toml", root);
    if live_root()
        .join("docs/debt/structural-duplication.toml")
        .is_file()
    {
        copy_live("docs/debt/structural-duplication.toml", root);
    }

    write_file(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = [
    "crates/weeping-angel-assurance-ir",
    "crates/weeping-angel-framework",
    "crates/weeping-angel-evidence",
    "crates/weeping-angel-collector",
    "crates/weeping-angel-control-test",
    "crates/weeping-angel-assurance",
    "crates/weeping-angel-canonical-catalog",
    ".",
    "xtask",
]

[package]
name = "weeping-angel"
version = "0.0.0"
edition = "2024"

[[test]]
name = "seeded"
path = "tests/seeded.rs"

[[test]]
name = "expansion_extra"
path = "tests/expansion_extra.rs"
"#,
    );
    write_file(&root.join("tests/seeded.rs"), "#[test] fn t() {}");
    write_file(&root.join("tests/expansion_extra.rs"), "#[test] fn t() {}");

    for pkg in [
        "weeping-angel-canonical-catalog",
        "weeping-angel-framework",
        "weeping-angel-assurance",
        "weeping-angel-evidence",
        "weeping-angel-collector",
        "weeping-angel-control-test",
        "weeping-angel-assurance-ir",
    ] {
        write_file(
            &root.join(format!("crates/{pkg}/Cargo.toml")),
            &format!("[package]\nname = \"{pkg}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"),
        );
    }
    write_file(
        &root.join("xtask/Cargo.toml"),
        "[package]\nname = \"xtask\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    );
    write_file(&root.join("src/main.rs"), "");
    write_file(&root.join("src/cli.rs"), "");
    write_file(
        &root.join("crates/weeping-angel-assurance/src/readiness.rs"),
        "",
    );
    write_file(
        &root.join("crates/weeping-angel-assurance/src/temporal.rs"),
        "",
    );
    write_file(
        &root.join("crates/weeping-angel-assurance/src/lineage.rs"),
        "",
    );

    let frozen = serde_json::json!({
        "schema": CONSOLIDATION_BASELINE_SCHEMA,
        "program": "architectural-consolidation",
        "phase": 0,
        "source": INVENTORY_SCHEMA,
        "exclusions": ["target/", "target-*", "node_modules/"],
        "inventory_counts": {
            "root_test_binaries": 1,
            "tests_rs_autodiscovered": 0,
            "tests_contracts_rs": 0,
            "ignored_test_attrs": 0,
            "unwrap_calls": 0,
            "expect_calls": 0,
            "unwrap_plus_expect": 0,
            "require_needles_fns": 0,
            "require_needles_calls": 0,
            "adr_markdown_files": 0,
            "catalog_test_toml": 0,
            "framework_packs": 0,
            "schema_json_files": 0
        },
        "extended": {
            "workspace_crates": 1,
            "rust_modules": 1,
            "public_symbols": 0,
            "pub_use_count": 0,
            "public_structs": 0,
            "public_enums": 0,
            "duplicate_helper_definitions": 0,
            "duplicate_type_names": 0
        },
        "architecture_ownership": [],
        "schema_locations": [],
        "adr_count": 0,
        "spec_count": 0,
        "debt_rows": 0
    });
    write_file(
        &root.join("docs/debt/consolidation-baseline.json"),
        &serde_json::to_string_pretty(&frozen)
            .unwrap_or_else(|e| panic!("serialize frozen baseline: {e}")),
    );
    write_file(
        &root.join("docs/debt/consolidation-baseline.md"),
        "frozen Phase 0 snapshot, not current.md\n<!-- weeping-angel-consolidation-baseline-stable -->\n",
    );
}
