//! Parser for `docs/debt/structural-duplication.toml` (program backlog).
//!
//! Schema: `weeping-angel/structural-duplication/v2`. Close law blocks
//! `verified` / `removed` until canonical_symbol exists, consumers migrated,
//! old paths gone (or compatibility-only), and a regression guard is cited.

use std::collections::BTreeSet;
use std::path::Path;

use crate::model::{read_toml, require_schema};

pub const DUPLICATION_SCHEMA_V2: &str = "weeping-angel/structural-duplication/v2";
pub const STRUCTURAL_DUPLICATION_PATH: &str = "docs/debt/structural-duplication.toml";

pub const V2_STATUSES: [&str; 7] = [
    "candidate",
    "confirmed",
    "canonicalized",
    "consumers-migrating",
    "compatibility-only",
    "removed",
    "verified",
];

const V1_RETIRED_STATUSES: [&str; 3] = ["migrating", "resolved", "false-positive"];

const SEVERITIES: [&str; 4] = ["p0", "p1", "p2", "info"];

const PUBLIC_API_IMPACTS: [&str; 4] = ["none", "additive", "breaking", "unknown"];

const SERIALIZATION_IMPACTS: [&str; 3] = ["none", "format-change", "unknown"];

const REQUIRED_ROW_FIELDS: [&str; 13] = [
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicationRow {
    pub id: String,
    pub concept: String,
    pub severity: String,
    pub canonical_owner: String,
    pub canonical_symbol: String,
    pub duplicates: Vec<String>,
    pub migration_state: String,
    pub removal_blockers: Vec<String>,
    pub public_api_impact: String,
    pub serialization_impact: String,
    pub tests: Vec<String>,
    pub guard: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralDuplicationMap {
    pub schema: String,
    pub program: String,
    pub phase: i64,
    pub rows: Vec<DuplicationRow>,
}

pub fn load_structural_duplication(root: &Path) -> Result<StructuralDuplicationMap, String> {
    let path = root.join(STRUCTURAL_DUPLICATION_PATH);
    if !path.is_file() {
        return Err(format!("{STRUCTURAL_DUPLICATION_PATH} is not a file"));
    }
    let value = read_toml(&path)?;
    require_schema(&value, DUPLICATION_SCHEMA_V2, &path)?;
    let program = value
        .get("program")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if program != "architectural-consolidation" {
        return Err(format!(
            "{STRUCTURAL_DUPLICATION_PATH} program must be architectural-consolidation, got {program:?}"
        ));
    }
    let phase = value
        .get("phase")
        .and_then(|v| v.as_integer())
        .unwrap_or(-1);
    if phase != 0 {
        return Err(format!(
            "{STRUCTURAL_DUPLICATION_PATH} phase must be 0, got {phase}"
        ));
    }
    let rows_val = value
        .get("duplication")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{STRUCTURAL_DUPLICATION_PATH} missing [[duplication]] array"))?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for row in rows_val {
        let parsed = parse_row(row)?;
        if !seen.insert(parsed.id.clone()) {
            return Err(format!(
                "{STRUCTURAL_DUPLICATION_PATH} duplicate id {}",
                parsed.id
            ));
        }
        rows.push(parsed);
    }
    Ok(StructuralDuplicationMap {
        schema: DUPLICATION_SCHEMA_V2.to_string(),
        program,
        phase,
        rows,
    })
}

fn parse_row(row: &toml::Value) -> Result<DuplicationRow, String> {
    let table = row
        .as_table()
        .ok_or_else(|| format!("{STRUCTURAL_DUPLICATION_PATH} duplication row is not a table"))?;
    for field in REQUIRED_ROW_FIELDS {
        if !table.contains_key(field) {
            let id = table
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("<missing-id>");
            return Err(format!(
                "{STRUCTURAL_DUPLICATION_PATH} {id} missing required field {field}"
            ));
        }
    }
    let id = require_nonempty_str(row, "id")?;
    if !id.starts_with("DUP-") {
        return Err(format!(
            "{STRUCTURAL_DUPLICATION_PATH} id must be DUP-NNN, got {id}"
        ));
    }
    let concept = require_nonempty_str(row, "concept")?;
    let severity = require_closed(row, "severity", &SEVERITIES)?;
    let canonical_owner = require_nonempty_str(row, "canonical_owner")?;
    let canonical_symbol = require_nonempty_str(row, "canonical_symbol")?;
    let duplicates = require_string_array(row, "duplicates")?;
    let migration_state = require_nonempty_str(row, "migration_state")?;
    let removal_blockers = require_string_array(row, "removal_blockers")?;
    let public_api_impact = require_closed(row, "public_api_impact", &PUBLIC_API_IMPACTS)?;
    let serialization_impact = require_closed(row, "serialization_impact", &SERIALIZATION_IMPACTS)?;
    let tests = require_string_array(row, "tests")?;
    let guard = require_nonempty_str(row, "guard")?;
    let status = require_nonempty_str(row, "status")?;
    if V1_RETIRED_STATUSES.contains(&status.as_str()) {
        return Err(format!(
            "{id} uses retired v1 status {status}; map migrating→consumers-migrating; never auto-map resolved/false-positive to verified/removed"
        ));
    }
    if !V2_STATUSES.contains(&status.as_str()) {
        return Err(format!("{id} status {status} is not in the v2 closed set"));
    }
    if canonical_symbol == "unknown" && status != "candidate" {
        return Err(format!(
            "{id} canonical_symbol may be unknown only while status=candidate"
        ));
    }
    if tests.is_empty() && status != "candidate" {
        return Err(format!(
            "{id} tests may be empty only while status=candidate"
        ));
    }
    enforce_close_law(&id, &status, &canonical_symbol, &duplicates, &tests, &guard)?;
    Ok(DuplicationRow {
        id,
        concept,
        severity,
        canonical_owner,
        canonical_symbol,
        duplicates,
        migration_state,
        removal_blockers,
        public_api_impact,
        serialization_impact,
        tests,
        guard,
        status,
    })
}

fn enforce_close_law(
    id: &str,
    status: &str,
    canonical_symbol: &str,
    duplicates: &[String],
    tests: &[String],
    guard: &str,
) -> Result<(), String> {
    if status != "verified" && status != "removed" {
        return Ok(());
    }
    if canonical_symbol.is_empty() || canonical_symbol == "unknown" {
        return Err(format!(
            "{id} close law: verified/removed requires canonical_symbol (not unknown)"
        ));
    }
    if tests.is_empty() && guard.is_empty() {
        return Err(format!(
            "{id} close law: verified/removed requires tests or a live guard"
        ));
    }
    if status == "removed" && !duplicates.is_empty() {
        return Err(format!(
            "{id} close law: removed must not still list tracked duplicate paths"
        ));
    }
    if status == "verified" && tests.is_empty() {
        return Err(format!(
            "{id} close law: verified requires executable tests (xtask/contracts) or a cited repository_guard"
        ));
    }
    Ok(())
}

fn require_nonempty_str(row: &toml::Value, key: &str) -> Result<String, String> {
    let s = row
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("duplication row {key} must be a string"))?
        .trim()
        .to_string();
    if s.is_empty() {
        return Err(format!("duplication row {key} must be non-empty"));
    }
    Ok(s)
}

fn require_closed(row: &toml::Value, key: &str, allowed: &[&str]) -> Result<String, String> {
    let s = require_nonempty_str(row, key)?;
    if !allowed.contains(&s.as_str()) {
        return Err(format!(
            "duplication row {key}={s:?} is not one of {}",
            allowed.join("|")
        ));
    }
    Ok(s)
}

fn require_string_array(row: &toml::Value, key: &str) -> Result<Vec<String>, String> {
    let arr = row
        .get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("duplication row {key} must be an array"))?;
    Ok(arr
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect())
}
