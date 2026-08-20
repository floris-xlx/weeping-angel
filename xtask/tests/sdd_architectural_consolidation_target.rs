//! Target suite for Architectural Consolidation Program Phases 0 and 1.
//!
//! CON-T01–T10 stay GREEN (Phase 0 freeze/baseline/backlog). CON-T11–T20 encode
//! domain-ownership law and stay GREEN after Phase 1 implement. CON-T07
//! reasserts that `sdd_architectural_consolidation_baseline.rs` is deleted
//! (`INV-NO-SUPERSEDED-BASELINES`); do not `#[ignore]` it.
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

const DOMAIN_OWNERSHIP_SCHEMA: &str = "weeping-angel/domain-ownership/v1";

const DOMAIN_OWNERSHIP_ROLES: [&str; 5] = [
    "semantic_owner",
    "storage_owner",
    "projection_owner",
    "evaluation_primitive_owner",
    "adapter_owner",
];

const SEEDED_CONCEPTS: [&str; 15] = [
    "applicability",
    "readiness",
    "catalog",
    "framework",
    "evidence",
    "temporal_evaluation",
    "assessment_replay",
    "soa",
    "control_status",
    "control_test_kernel",
    "evidence_validity",
    "catalog_loading",
    "framework_compilation",
    "assurance_cli",
    "collectors",
];

const DOMAIN_OWNERSHIP_INVARIANTS: [&str; 2] =
    ["INV-DOMAIN-OWNERSHIP-PRESENT", "INV-DOMAIN-OWNERSHIP-ROLES"];

const HYPOTHETICAL_CRATES: [&str; 2] = ["weeping-angel-catalog", "weeping-angel-assurance-cli"];

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

    for id in ["DUP-010"] {
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
    let dup001 = by_id.get("DUP-001").unwrap_or_else(|| panic!("missing DUP-001"));
    assert_eq!(
        dup001.0.as_str(),
        "verified",
        "DUP-001 close law after duplicate schema tree deleted, got {}",
        dup001.0
    );
    let dup008 = by_id.get("DUP-008").unwrap_or_else(|| panic!("missing DUP-008"));
    assert_eq!(
        dup008.0.as_str(),
        "verified",
        "DUP-008 close law after catalog root walk SSOT, got {}",
        dup008.0
    );
    let dup013 = by_id.get("DUP-013").unwrap_or_else(|| panic!("missing DUP-013"));
    assert_eq!(
        dup013.0.as_str(),
        "verified",
        "DUP-013 close law after Lane C pack-parse SSOT, got {}",
        dup013.0
    );
    let dup004 = by_id.get("DUP-004").unwrap_or_else(|| panic!("missing DUP-004"));
    assert_eq!(
        dup004.0.as_str(),
        "verified",
        "DUP-004 close law after Lane B snapshot SSOT, got {}",
        dup004.0
    );
    let dup011 = by_id.get("DUP-011").unwrap_or_else(|| panic!("missing DUP-011"));
    assert_eq!(
        dup011.0.as_str(),
        "verified",
        "DUP-011 close law after Lane B readiness SSOT, got {}",
        dup011.0
    );
    let dup005 = by_id.get("DUP-005").unwrap_or_else(|| panic!("missing DUP-005"));
    assert_eq!(
        dup005.0.as_str(),
        "verified",
        "DUP-005 close law after Lane B replay boundary, got {}",
        dup005.0
    );
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
/// After Phase 1 GREEN the characterization file is deleted, not ignored.
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

fn domain_ownership_path() -> PathBuf {
    live_root().join("architecture/domain-ownership.toml")
}

fn read_domain_ownership() -> String {
    fs::read_to_string(domain_ownership_path()).unwrap_or_else(|e| {
        panic!("architecture/domain-ownership.toml must exist and be readable: {e}")
    })
}

fn parse_domain_ownership() -> toml::Value {
    read_domain_ownership()
        .parse()
        .unwrap_or_else(|e| panic!("architecture/domain-ownership.toml must parse: {e}"))
}

fn concept_table<'a>(parsed: &'a toml::Value, id: &str) -> &'a toml::value::Table {
    parsed
        .get("concept")
        .and_then(|v| v.as_table())
        .and_then(|c| c.get(id))
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("missing [concept.{id}]"))
}

fn loader_mentions_domain_ownership() -> bool {
    let src = architecture_src();
    src.contains("domain-ownership.toml")
        || src.contains("load_domain_ownership")
        || src.contains("domain_ownership")
}

fn owner_seat(table: &toml::value::Table, role: &str) -> String {
    table
        .get(role)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("[concept] missing required role {role}"))
        .to_string()
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

/// CON-T11: architecture/domain-ownership.toml is parsed as the concept SSOT.
#[test]
fn con_t11_domain_ownership_toml_parsed_schema_roles_seeds() {
    let path = domain_ownership_path();
    assert!(
        path.is_file(),
        "architecture/domain-ownership.toml must exist as the concept-level SSOT"
    );
    let parsed = parse_domain_ownership();
    assert_eq!(
        parsed.get("schema").and_then(|v| v.as_str()),
        Some(DOMAIN_OWNERSHIP_SCHEMA),
        "schema must be {DOMAIN_OWNERSHIP_SCHEMA}"
    );
    let roles = parsed
        .get("required_roles")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("required_roles must be a non-empty array"));
    let role_names: Vec<&str> = roles.iter().filter_map(|v| v.as_str()).collect();
    assert!(!role_names.is_empty(), "required_roles must be non-empty");
    for role in DOMAIN_OWNERSHIP_ROLES {
        assert!(
            role_names.iter().any(|r| *r == role),
            "required_roles must include {role}: {role_names:?}"
        );
    }
    assert!(
        !role_names.iter().any(|r| *r == "persistence_owner"),
        "persistence_owner maps to storage_owner; it is not a sixth role: {role_names:?}"
    );
    let concepts = parsed
        .get("concept")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("domain-ownership.toml missing [concept.*] tables"));
    for id in SEEDED_CONCEPTS {
        assert!(
            concepts.contains_key(id),
            "seeded [concept.{id}] must be present"
        );
        let table = concept_table(&parsed, id);
        for role in DOMAIN_OWNERSHIP_ROLES {
            assert!(
                table.get(role).and_then(|v| v.as_str()).is_some(),
                "[concept.{id}] must declare string role {role}"
            );
        }
    }
    assert!(
        loader_mentions_domain_ownership(),
        "xtask/src/architecture.rs must parse architecture/domain-ownership.toml"
    );
    let live = load_architecture_manifest(&live_root())
        .unwrap_or_else(|e| panic!("live manifests must load once domain-ownership is wired: {e}"));
    let debug = format!("{live:?}");
    assert!(
        debug.contains("semantic_owner")
            || debug.contains("domain_ownership")
            || debug.contains("DomainOwnership"),
        "ArchitectureManifest must carry parsed domain-ownership (paper file is not a pass): {debug}"
    );
}

/// CON-T12: missing/malformed domain-ownership.toml fails Guard 01 and/or 04.
/// A sibling paper file that is not parsed is not a pass.
#[test]
fn con_t12_missing_or_malformed_domain_ownership_fails_closed() {
    assert!(
        loader_mentions_domain_ownership(),
        "paper architecture/domain-ownership.toml without a parser is not a pass"
    );

    let dir = tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    write_file(
        &dir.path().join("architecture/architecture.toml"),
        &read_live("architecture/architecture.toml"),
    );
    let missing_load = load_architecture_manifest(dir.path());
    let missing_c01 = check_01_architecture_manifest(dir.path());
    assert!(
        missing_load.is_err() || is_fail(&missing_c01.status),
        "missing architecture/domain-ownership.toml must fail load_architecture_manifest or Guard 01, got load={missing_load:?} guard={:?}",
        missing_c01.status
    );

    write_file(
        &dir.path().join("architecture/domain-ownership.toml"),
        "this is not valid TOML {{{ and must fail closed\n",
    );
    let paper_load = load_architecture_manifest(dir.path());
    let paper_c01 = check_01_architecture_manifest(dir.path());
    assert!(
        paper_load.is_err() || is_fail(&paper_c01.status),
        "malformed architecture/domain-ownership.toml must fail closed, got load={paper_load:?} guard={:?}",
        paper_c01.status
    );

    let mut bad_schema = String::from("schema = \"not-a-real-schema\"\nrequired_roles = [");
    for (i, role) in DOMAIN_OWNERSHIP_ROLES.iter().enumerate() {
        if i > 0 {
            bad_schema.push_str(", ");
        }
        bad_schema.push('"');
        bad_schema.push_str(role);
        bad_schema.push('"');
    }
    bad_schema.push_str("]\n");
    write_file(
        &dir.path().join("architecture/domain-ownership.toml"),
        &bad_schema,
    );
    let schema_load = load_architecture_manifest(dir.path());
    let schema_c01 = check_01_architecture_manifest(dir.path());
    assert!(
        schema_load.is_err() || is_fail(&schema_c01.status),
        "wrong domain-ownership schema must fail closed, got load={schema_load:?} guard={:?}",
        schema_c01.status
    );
}

/// CON-T13: five roles are not collapsed into crate kind=exclusive;
/// persistence_owner is not a sixth role.
#[test]
fn con_t13_roles_not_collapsed_no_sixth_persistence_owner() {
    let parsed = parse_domain_ownership();
    let required = parsed
        .get("required_roles")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("required_roles array"));
    let role_names: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(
        role_names.len(),
        5,
        "exactly five required_roles, got {role_names:?}"
    );
    assert!(
        !role_names.iter().any(|r| *r == "persistence_owner"),
        "persistence_owner is not a required role"
    );

    let concepts = parsed
        .get("concept")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("[concept] table"));
    for (id, value) in concepts {
        let table = value
            .as_table()
            .unwrap_or_else(|| panic!("[concept.{id}] must be a table"));
        for role in DOMAIN_OWNERSHIP_ROLES {
            assert!(
                table.get(role).and_then(|v| v.as_str()).is_some(),
                "[concept.{id}] missing role {role}; do not copy architecture.toml kind=exclusive as fake exclusivity"
            );
        }
        assert!(
            !table.contains_key("persistence_owner"),
            "[concept.{id}] must not declare persistence_owner as a role key (map it to storage_owner)"
        );
        if table.get("kind").and_then(|v| v.as_str()) == Some("exclusive")
            && DOMAIN_OWNERSHIP_ROLES
                .iter()
                .all(|role| !table.contains_key(*role))
        {
            panic!("[concept.{id}] collapsed into kind=exclusive without the five roles");
        }
    }

    let dir = tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
    write_file(
        &dir.path().join("architecture/architecture.toml"),
        &read_live("architecture/architecture.toml"),
    );
    write_file(
        &dir.path().join("architecture/domain-ownership.toml"),
        r#"schema = "weeping-angel/domain-ownership/v1"
required_roles = [
  "semantic_owner",
  "storage_owner",
  "projection_owner",
  "evaluation_primitive_owner",
  "adapter_owner",
  "persistence_owner",
]

[concept.catalog]
crate = "weeping-angel-canonical-catalog"
kind = "exclusive"
paths = ["crates/weeping-angel-canonical-catalog"]
"#,
    );
    let collapsed = load_architecture_manifest(dir.path());
    let collapsed_c01 = check_01_architecture_manifest(dir.path());
    assert!(
        collapsed.is_err() || is_fail(&collapsed_c01.status),
        "sixth role persistence_owner and kind=exclusive without five keys must fail closed, got load={collapsed:?} guard={:?}",
        collapsed_c01.status
    );
}

/// CON-T14: seeded concepts cite live symbols in live workspace crates only.
#[test]
fn con_t14_seeded_concepts_cite_live_symbols_no_hypothetical_crates() {
    let parsed = parse_domain_ownership();
    let blob = read_domain_ownership();
    for forbidden in HYPOTHETICAL_CRATES {
        assert!(
            !blob.contains(forbidden),
            "domain-ownership.toml must not name hypothetical crate {forbidden}"
        );
    }

    let applicability = concept_table(&parsed, "applicability");
    assert_eq!(
        owner_seat(applicability, "semantic_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert_eq!(
        owner_seat(applicability, "projection_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert_eq!(
        owner_seat(applicability, "storage_owner").as_str(),
        "weeping-angel-assurance"
    );
    let appl_blob = format!("{applicability:?}");
    assert!(
        appl_blob.contains("ApplicabilitySnapshot") || blob.contains("ApplicabilitySnapshot"),
        "applicability must cite representation ApplicabilitySnapshot"
    );
    assert!(
        appl_blob.contains("lineage") || blob.contains("ApplicabilitySnapshot"),
        "applicability storage maps persistence_owner=lineage to the canonical ApplicabilitySnapshot"
    );
    let snapshot = read_live("crates/weeping-angel-assurance/src/applicability/snapshot.rs");
    assert!(
        snapshot.contains("struct ApplicabilitySnapshot"),
        "live ApplicabilitySnapshot must remain in weeping-angel-assurance"
    );
    let lineage = read_live("crates/weeping-angel-assurance/src/lineage.rs");
    assert!(
        !lineage.contains("struct LineageApplicabilitySnapshot"),
        "DUP-004: lineage must not keep a parallel ApplicabilitySnapshot domain type"
    );
    assert!(
        lineage.contains("applicability: ApplicabilitySnapshot"),
        "LineageBundle must pin canonical ApplicabilitySnapshot"
    );

    let readiness = concept_table(&parsed, "readiness");
    assert_eq!(
        owner_seat(readiness, "semantic_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert!(
        blob.contains("project_readiness"),
        "readiness must cite project_readiness"
    );
    let readiness_src = read_live("crates/weeping-angel-assurance/src/readiness.rs");
    assert!(
        readiness_src.contains("pub fn project_readiness"),
        "live project_readiness must remain"
    );

    let catalog = concept_table(&parsed, "catalog");
    assert_eq!(
        owner_seat(catalog, "semantic_owner").as_str(),
        "weeping-angel-canonical-catalog"
    );
    assert!(
        blob.contains("CanonicalCatalog") && blob.contains("load"),
        "catalog must cite CanonicalCatalog::load"
    );
    let catalog_src = read_live("crates/weeping-angel-canonical-catalog/src/lib.rs");
    assert!(
        catalog_src.contains("pub fn load("),
        "live CanonicalCatalog::load must remain"
    );

    let framework = concept_table(&parsed, "framework");
    assert_eq!(
        owner_seat(framework, "semantic_owner").as_str(),
        "weeping-angel-framework"
    );
    assert!(
        blob.contains("compile_framework"),
        "framework must cite compile_framework"
    );
    let framework_src = read_live("crates/weeping-angel-framework/src/lib.rs");
    assert!(
        framework_src.contains("pub fn compile_framework"),
        "live compile_framework must remain"
    );

    let evidence = concept_table(&parsed, "evidence");
    assert_eq!(
        owner_seat(evidence, "semantic_owner").as_str(),
        "weeping-angel-evidence"
    );
    assert_eq!(
        owner_seat(evidence, "storage_owner").as_str(),
        "weeping-angel-evidence"
    );
    assert!(
        blob.contains("current") && blob.contains("as_of") && blob.contains("latest"),
        "evidence must cite ledger current/as_of/latest"
    );
    let ledger = read_live("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        ledger.contains("pub fn current(")
            && ledger.contains("pub fn as_of(")
            && ledger.contains("pub fn latest("),
        "live evidence ledger current/as_of/latest must remain"
    );

    let replay = concept_table(&parsed, "assessment_replay");
    assert_eq!(
        owner_seat(replay, "semantic_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert!(
        blob.contains("replay_assessment"),
        "assessment_replay must cite replay_assessment"
    );
    assert!(
        lineage.contains("pub fn replay_assessment"),
        "live replay_assessment must remain"
    );

    let soa = concept_table(&parsed, "soa");
    assert_eq!(
        owner_seat(soa, "semantic_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert_eq!(
        owner_seat(soa, "projection_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert!(
        blob.contains("project_soa_from_snapshot"),
        "soa must cite project_soa_from_snapshot"
    );
    let soa_src = read_live("crates/weeping-angel-assurance/src/soa.rs");
    assert!(
        soa_src.contains("pub fn project_soa_from_snapshot"),
        "live project_soa_from_snapshot must remain"
    );

    let kernel = concept_table(&parsed, "control_test_kernel");
    assert_eq!(
        owner_seat(kernel, "semantic_owner").as_str(),
        "weeping-angel-control-test"
    );
    assert_eq!(
        owner_seat(kernel, "evaluation_primitive_owner").as_str(),
        "weeping-angel-control-test"
    );
    assert!(
        blob.contains("evaluate") && blob.contains("run.inc"),
        "control_test_kernel must cite evaluate in run.inc"
    );
    let run_inc = read_live("crates/weeping-angel-control-test/src/run.inc");
    assert!(
        run_inc.contains("pub fn evaluate("),
        "live evaluate in run.inc must remain"
    );

    let validity = concept_table(&parsed, "evidence_validity");
    assert_eq!(
        owner_seat(validity, "semantic_owner").as_str(),
        "weeping-angel-evidence"
    );
    assert!(
        blob.contains("project_validity"),
        "evidence_validity must cite project_validity"
    );
    let validity_src = read_live("crates/weeping-angel-evidence/src/validity.rs");
    assert!(
        validity_src.contains("pub fn project_validity"),
        "live project_validity must remain"
    );

    let catalog_loading = concept_table(&parsed, "catalog_loading");
    assert_eq!(
        owner_seat(catalog_loading, "semantic_owner").as_str(),
        "weeping-angel-canonical-catalog"
    );

    let framework_compilation = concept_table(&parsed, "framework_compilation");
    assert_eq!(
        owner_seat(framework_compilation, "semantic_owner").as_str(),
        "weeping-angel-framework"
    );

    let cli = concept_table(&parsed, "assurance_cli");
    assert_eq!(owner_seat(cli, "semantic_owner").as_str(), "weeping-angel");
    assert!(
        blob.contains("src/main.rs") && blob.contains("src/cli.rs"),
        "assurance_cli facade must cite src/main.rs and src/cli.rs"
    );
    assert!(
        live_root().join("src/main.rs").is_file() && live_root().join("src/cli.rs").is_file(),
        "CLI facade files must remain at src/main.rs and src/cli.rs"
    );

    let collectors = concept_table(&parsed, "collectors");
    assert_eq!(
        owner_seat(collectors, "semantic_owner").as_str(),
        "weeping-angel-collector"
    );
    assert_eq!(
        owner_seat(collectors, "adapter_owner").as_str(),
        "weeping-angel-collector"
    );
    assert!(
        blob.contains("CollectorAdapter"),
        "collectors must cite CollectorAdapter"
    );
    let adapter = read_live("crates/weeping-angel-collector/src/ports/adapter.rs");
    assert!(
        adapter.contains("CollectorAdapter"),
        "live CollectorAdapter must remain"
    );

    let ir = read_live("crates/weeping-angel-assurance-ir/src/implementation.rs");
    assert!(
        ir.contains("enum ImplementationStatus"),
        "live ImplementationStatus must remain in weeping-angel-assurance-ir"
    );
    let ctl = read_live("crates/weeping-angel-control-test/src/lib.rs");
    assert!(
        ctl.contains("enum Effectiveness"),
        "live Effectiveness must remain in weeping-angel-control-test"
    );
    assert!(
        blob.contains("ImplementationStatus") && blob.contains("Effectiveness"),
        "control_status seed must cite ImplementationStatus and Effectiveness"
    );
}

/// CON-T15: temporal_evaluation and control_status are split=divided, not fake exclusive.
#[test]
fn con_t15_temporal_evaluation_and_control_status_are_divided() {
    let parsed = parse_domain_ownership();
    let blob = read_domain_ownership();

    let temporal = concept_table(&parsed, "temporal_evaluation");
    assert_eq!(
        temporal.get("split").and_then(|v| v.as_str()),
        Some("divided"),
        "temporal_evaluation.split must be divided (do not copy architecture.toml kind=exclusive)"
    );
    assert_eq!(
        owner_seat(temporal, "semantic_owner").as_str(),
        "weeping-angel-control-test"
    );
    assert_eq!(
        owner_seat(temporal, "evaluation_primitive_owner").as_str(),
        "weeping-angel-control-test"
    );
    assert_eq!(
        owner_seat(temporal, "projection_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert_eq!(
        owner_seat(temporal, "storage_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert!(
        blob.contains("select_latest_as_of"),
        "temporal_evaluation must cite select_latest_as_of"
    );
    assert_ne!(
        temporal.get("kind").and_then(|v| v.as_str()),
        Some("exclusive"),
        "domain-ownership must not claim fake exclusivity on temporal_evaluation"
    );
    let control_temporal = read_live("crates/weeping-angel-control-test/src/temporal.rs");
    assert!(
        control_temporal.contains("pub fn select_latest_as_of"),
        "evaluation primitive remains in weeping-angel-control-test"
    );
    let assurance_temporal = read_live("crates/weeping-angel-assurance/src/temporal.rs");
    assert!(
        assurance_temporal.contains("pub fn project_timeline")
            && !assurance_temporal.contains("pub fn select_latest_as_of"),
        "assurance temporal.rs stays the timeline/diff projection"
    );

    let status = concept_table(&parsed, "control_status");
    assert_eq!(
        status.get("split").and_then(|v| v.as_str()),
        Some("divided"),
        "control_status.split must be divided"
    );
    assert_eq!(
        owner_seat(status, "semantic_owner").as_str(),
        "divided",
        "control_status semantic_owner is divided, not one exclusive crate"
    );
    assert_eq!(
        owner_seat(status, "evaluation_primitive_owner").as_str(),
        "weeping-angel-control-test"
    );
    assert_eq!(
        owner_seat(status, "projection_owner").as_str(),
        "weeping-angel-assurance"
    );
    assert_ne!(
        status.get("kind").and_then(|v| v.as_str()),
        Some("exclusive"),
        "control_status must not copy kind=exclusive as fake exclusivity"
    );
}

/// CON-T16: domain-ownership invariants fold into Guard 01/04; no Guard 16.
#[test]
fn con_t16_domain_ownership_invariants_no_guard_16() {
    let invariants = read_live("architecture/invariants.toml");
    for id in DOMAIN_OWNERSHIP_INVARIANTS {
        assert!(
            invariants.contains(id),
            "architecture/invariants.toml must declare {id}"
        );
    }
    let src = xtask_src_tree();
    for id in DOMAIN_OWNERSHIP_INVARIANTS {
        assert!(
            src.contains(id),
            "evaluate_invariant must have a predicate for {id} (unknown-id fail-closed is not enough)"
        );
    }
    assert!(
        src.contains("eval_domain_ownership")
            || src.contains("INV-DOMAIN-OWNERSHIP-PRESENT")
                && src.contains("INV-DOMAIN-OWNERSHIP-ROLES"),
        "Guard 04 must evaluate domain-ownership invariants"
    );
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    assert!(
        src.contains("KNOWN_CHECK_IDS: [&str; 15]"),
        "do not add Guard 16"
    );
    assert!(
        !src.contains("cargo xtask consolidation") && !src.contains("Some(\"consolidation\")"),
        "do not add a second health CLI"
    );
    let live = check_04_architecture_invariants(&live_root());
    match &live.status {
        CheckStatus::Fail(msg) => {
            panic!(
                "Guard 04 must pass on the live tree once INV-DOMAIN-OWNERSHIP* predicates exist, got {msg}"
            );
        }
        CheckStatus::Pass => {}
        other => panic!("Guard 04 unexpected status: {other:?}"),
    }
}

/// CON-T17: dual-suite stays under xtask/tests; Phase 0 ids remain; ownership law is wired.
#[test]
fn con_t17_dual_suite_under_xtask_tests_phase0_ids_remain() {
    let xtask_tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let target = xtask_tests.join("sdd_architectural_consolidation_target.rs");
    let baseline = xtask_tests.join("sdd_architectural_consolidation_baseline.rs");
    assert!(
        target.is_file(),
        "target suite must live under xtask/tests/"
    );
    let target_text =
        fs::read_to_string(&target).unwrap_or_else(|e| panic!("read consolidation target: {e}"));
    for id in [
        "con_t01", "con_t02", "con_t03", "con_t04", "con_t05", "con_t06", "con_t08", "con_t09",
        "con_t11", "con_t12", "con_t13", "con_t14", "con_t15", "con_t16", "con_t17", "con_t18",
        "con_t19", "con_t20",
    ] {
        assert!(
            target_text.contains(id),
            "target suite must keep Phase 0 and Phase 1 id {id}"
        );
    }
    assert!(
        !baseline.exists(),
        "superseded xtask baseline suite must be deleted"
    );
    assert!(!live_root().join("tests/sdd").exists());
    assert!(!live_root().join("test/sdd").exists());
    assert!(!read_live("Cargo.toml").contains("sdd_architectural_consolidation"));
    assert!(!read_live("xtask/Cargo.toml").contains("[[test]]"));
    assert!(
        domain_ownership_path().is_file() && loader_mentions_domain_ownership(),
        "dual-suite Phase 1 is ownership law: domain-ownership.toml must exist and be parsed"
    );
}

/// CON-T18: one program spec; ADR 0050 exists; no colliding 0003/0011 filenames.
#[test]
fn con_t18_single_program_spec_and_adr_0050() {
    let layout = read_live("tests/contracts/documentation_layout.rs");
    let spec_hits = layout
        .matches("docs/specs/architectural-consolidation-program.md")
        .count();
    assert_eq!(
        spec_hits, 1,
        "CANONICAL_SPECS must list the consolidation spec once, not a forked SSOT"
    );
    assert!(
        !layout.contains("docs/specs/domain-ownership"),
        "do not fork a second program spec for domain-ownership"
    );
    let life = read_live("architecture/spec-lifecycle.toml");
    assert!(
        life.contains("docs/specs/architectural-consolidation-program.md"),
        "Guard 15 existing consolidation spec row must remain"
    );
    let adr_path = live_root().join("docs/adr/0050-domain-ownership-model.md");
    assert!(adr_path.is_file(), "ADR 0050 file must exist");
    let adr = fs::read_to_string(&adr_path).unwrap_or_else(|e| panic!("read ADR 0050: {e}"));
    assert!(
        adr.contains("id = \"0050\""),
        "ADR 0050 weeping-angel-adr-meta must set id = \"0050\""
    );
    assert!(
        adr.contains("status = \"draft\"") || adr.contains("status = \"accepted\""),
        "ADR 0050 meta status is draft until GREEN, then accepted"
    );
    let adr_dir = live_root().join("docs/adr");
    let entries = fs::read_dir(&adr_dir).unwrap_or_else(|e| panic!("read docs/adr: {e}"));
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        assert!(
            !name.starts_with("0003-domain-ownership"),
            "do not mint 0003-domain-ownership*: {name}"
        );
        if name.starts_with("0011-") {
            assert_eq!(
                name.as_ref(),
                "0011-repository-guard-governance.md",
                "do not mint a colliding 0011-* ADR: {name}"
            );
        }
    }
    assert!(
        domain_ownership_path().is_file(),
        "ADR 0050 cites architecture/domain-ownership.toml as the machine SSOT; the file must exist"
    );
    let parsed = parse_domain_ownership();
    assert_eq!(
        parsed.get("schema").and_then(|v| v.as_str()),
        Some(DOMAIN_OWNERSHIP_SCHEMA)
    );
}

/// CON-T19: INV-NO-SUPERSEDED-BASELINES means leftover after GREEN / ignored / tests/sdd,
/// not a live xtask dual-suite window file.
#[test]
fn con_t19_no_superseded_baselines_honesty_window() {
    let invariants = read_live("architecture/invariants.toml");
    assert!(
        invariants.contains("INV-NO-SUPERSEDED-BASELINES"),
        "honesty-amend the existing invariant; do not invent a new leftover id"
    );
    let src = xtask_src_tree();
    assert!(
        src.contains("INV-NO-SUPERSEDED-BASELINES"),
        "evaluate_invariant must keep INV-NO-SUPERSEDED-BASELINES"
    );
    let start = src
        .find("fn eval_no_superseded_baselines")
        .unwrap_or_else(|| panic!("eval_no_superseded_baselines predicate must exist"));
    let rest = &src[start..];
    let end = rest.find("\nfn ").unwrap_or(rest.len().min(2500));
    let predicate = &rest[..end];
    assert!(
        predicate.contains("tests/sdd") || predicate.contains("test/sdd"),
        "honesty-amended leftover rule must still fail-close tests/sdd baselines"
    );
    assert!(
        predicate.contains("ignore"),
        "honesty-amended leftover rule must fail-close #[ignore] baselines"
    );
    assert!(
        predicate.contains("xtask/tests")
            || predicate.contains("dual-suite")
            || predicate.contains("sdd_architectural_consolidation_baseline"),
        "honesty-amended leftover rule must allow a live non-ignored xtask/tests/sdd_*_baseline.rs during the window"
    );

    let baseline = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sdd_architectural_consolidation_baseline.rs");
    if baseline.is_file() {
        let text =
            fs::read_to_string(&baseline).unwrap_or_else(|e| panic!("read window baseline: {e}"));
        assert!(
            !text.contains("#[ignore"),
            "window characterization must not be #[ignore]-superseded"
        );
        assert!(
            text.contains("con_b11") && text.contains("con_b16"),
            "window baseline encodes CON-B11–B16"
        );
    }
    assert!(
        !live_root().join("tests/sdd").exists(),
        "tests/sdd leftover baselines remain forbidden"
    );
}

/// CON-T20: neighbors stay green; product needles are not rewritten; ownership law is live.
#[test]
fn con_t20_neighbors_green_product_needles_unchanged() {
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

    let freeze = program_table(&read_live("architecture/architecture.toml"));
    assert_eq!(
        freeze.get("status").and_then(|v| v.as_str()),
        Some("active")
    );
    assert_eq!(
        freeze.get("feature_expansion").and_then(|v| v.as_str()),
        Some("restricted"),
        "Phase 0 freeze stays active"
    );

    let applicability = read_live("crates/weeping-angel-assurance/src/applicability/snapshot.rs");
    assert!(applicability.contains("struct ApplicabilitySnapshot"));
    let readiness = read_live("crates/weeping-angel-assurance/src/readiness.rs");
    assert!(readiness.contains("fn project_readiness"));
    let lineage = read_live("crates/weeping-angel-assurance/src/lineage.rs");
    assert!(lineage.contains("fn replay_assessment"));
    assert!(
        !lineage.contains("struct LineageApplicabilitySnapshot"),
        "DUP-004: no parallel lineage applicability domain type"
    );

    let blob = read_domain_ownership();
    assert!(
        blob.contains("ApplicabilitySnapshot")
            && blob.contains("project_readiness")
            && blob.contains("replay_assessment"),
        "Phase 1 names owners of live applicability/readiness/lineage symbols; it must not rewrite them"
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
