//! Parse and validate versioned `architecture/*.toml` policy files.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::model::{read_toml, require_schema};

pub const ARCH_SCHEMA: &str = "weeping-angel/architecture/v1";
pub const INVARIANTS_SCHEMA: &str = "weeping-angel/architecture-invariants/v1";
pub const FORBIDDEN_SCHEMA: &str = "weeping-angel/forbidden-patterns/v1";
pub const ADR_IDENTITY_SCHEMA: &str = "weeping-angel/adr-identity/v1";
pub const SPEC_LIFECYCLE_SCHEMA: &str = "weeping-angel/spec-lifecycle/v1";

/// Compatibility view of the seven increment-1 concepts (tests / ACP).
/// Policy SSOT is `architecture/architecture.toml` `[policy]` + `[ownership.*]`.
pub const REQUIRED_OWNERSHIP: [(&str, &str, &[&str]); 7] = [
    (
        "catalog",
        "weeping-angel-canonical-catalog",
        &["crates/weeping-angel-canonical-catalog"],
    ),
    (
        "framework_compilation",
        "weeping-angel-framework",
        &["crates/weeping-angel-framework"],
    ),
    (
        "readiness_projection",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/readiness.rs"],
    ),
    (
        "temporal_evidence_selection",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/temporal.rs"],
    ),
    (
        "assessment_lineage",
        "weeping-angel-assurance",
        &["crates/weeping-angel-assurance/src/lineage.rs"],
    ),
    (
        "evidence_persistence",
        "weeping-angel-evidence",
        &["crates/weeping-angel-evidence"],
    ),
    (
        "assurance_cli",
        "weeping-angel",
        &["src/main.rs", "src/cli.rs"],
    ),
];

pub const ADR_STATUSES: [&str; 6] = [
    "draft",
    "proposed",
    "accepted",
    "superseded",
    "rejected",
    "deprecated",
];

pub const SPEC_STATES: [&str; 4] = ["draft", "active", "superseded", "retired"];

/// Ownership row from `architecture/architecture.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipRow {
    pub crate_name: String,
    pub kind: Option<String>,
    pub paths: Vec<String>,
}

/// `[policy]` table: allowed kinds and required concept keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitecturePolicy {
    pub ownership_kinds: Vec<String>,
    pub required_concepts: Vec<String>,
}

/// Parsed `architecture/architecture.toml` including ownership kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureManifest {
    pub schema: String,
    pub policy: ArchitecturePolicy,
    pub ownership: BTreeMap<String, OwnershipRow>,
}

/// One `[[invariant]]` row from `architecture/invariants.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureInvariant {
    pub id: String,
    pub summary: String,
    pub guard_check: String,
}

/// Per-row outcome of Guard 04 evaluation against the repository model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantResult {
    pub id: String,
    pub summary: String,
    pub guard_check: String,
    pub passed: bool,
    pub evidence: String,
}

/// One `[[pattern]]` row from `architecture/forbidden-patterns.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForbiddenPattern {
    pub id: String,
    pub kind: Option<String>,
    pub value: String,
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrIdentityPolicy {
    pub grandfathered_debt: String,
    pub grandfathered_prefixes: BTreeSet<String>,
    pub grandfathered_files: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrMeta {
    pub filename: String,
    pub stem: String,
    pub prefix: String,
    pub id: String,
    pub status: String,
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecLifecycleRow {
    pub path: String,
    pub state: String,
    pub ownership: Vec<String>,
    pub depends_on: Vec<String>,
    pub supersedes: Vec<String>,
    pub successor: String,
}

pub fn load_architecture_manifest(root: &Path) -> Result<ArchitectureManifest, String> {
    let path = root.join("architecture/architecture.toml");
    if !path.is_file() {
        return Err("architecture/architecture.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, ARCH_SCHEMA, &path)?;
    let policy = parse_policy(&value)?;
    let ownership_table = value
        .get("ownership")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "architecture.toml missing [ownership] table".to_string())?;
    let mut ownership = BTreeMap::new();
    for (concept, entry) in ownership_table {
        let crate_name = entry
            .get("crate")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let kind = entry
            .get("kind")
            .and_then(|c| c.as_str())
            .map(str::to_string);
        let paths = entry
            .get("paths")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        ownership.insert(
            concept.clone(),
            OwnershipRow {
                crate_name,
                kind,
                paths,
            },
        );
    }
    Ok(ArchitectureManifest {
        schema: ARCH_SCHEMA.to_string(),
        policy,
        ownership,
    })
}

fn parse_policy(value: &toml::Value) -> Result<ArchitecturePolicy, String> {
    let policy = value
        .get("policy")
        .ok_or_else(|| "architecture.toml missing [policy] table".to_string())?;
    let ownership_kinds = string_array(policy, "ownership_kinds")?;
    let required_concepts = string_array(policy, "required_concepts")?;
    if ownership_kinds.is_empty() {
        return Err("architecture.toml [policy].ownership_kinds must be non-empty".into());
    }
    if required_concepts.is_empty() {
        return Err("architecture.toml [policy].required_concepts must be non-empty".into());
    }
    Ok(ArchitecturePolicy {
        ownership_kinds,
        required_concepts,
    })
}

fn string_array(value: &toml::Value, key: &str) -> Result<Vec<String>, String> {
    let arr = value
        .get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("architecture.toml [policy].{key} must be a non-empty array"))?;
    Ok(arr
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect())
}

pub fn load_invariants(root: &Path) -> Result<Vec<ArchitectureInvariant>, String> {
    let path = root.join("architecture/invariants.toml");
    if !path.is_file() {
        return Err("architecture/invariants.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, INVARIANTS_SCHEMA, &path)?;
    let rows = value
        .get("invariant")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "architecture/invariants.toml missing [[invariant]] array".to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(ArchitectureInvariant {
            id: row
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            summary: row
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            guard_check: row
                .get("guard_check")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    Ok(out)
}

pub fn load_forbidden_patterns(root: &Path) -> Result<Vec<ForbiddenPattern>, String> {
    let path = root.join("architecture/forbidden-patterns.toml");
    if !path.is_file() {
        return Err("architecture/forbidden-patterns.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, FORBIDDEN_SCHEMA, &path)?;
    let Some(rows) = value.get("pattern").and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for row in rows {
        let mut extra = BTreeMap::new();
        if let Some(table) = row.as_table() {
            for (k, v) in table {
                if matches!(k.as_str(), "id" | "kind" | "value" | "rationale") {
                    continue;
                }
                if let Some(s) = v.as_str() {
                    extra.insert(k.clone(), s.to_string());
                }
            }
        }
        out.push(ForbiddenPattern {
            id: row
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            kind: row.get("kind").and_then(|v| v.as_str()).map(str::to_string),
            value: row
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            extra,
        });
    }
    Ok(out)
}

pub fn load_adr_identity(root: &Path) -> Result<AdrIdentityPolicy, String> {
    let path = root.join("architecture/adr-identity.toml");
    if !path.is_file() {
        return Err("architecture/adr-identity.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, ADR_IDENTITY_SCHEMA, &path)?;
    let grandfathered_debt = value
        .get("grandfathered_debt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if value.get("grandfathered_debt").is_none() {
        return Err("architecture/adr-identity.toml missing grandfathered_debt".into());
    }
    let prefixes = value
        .get("grandfathered_prefixes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            "architecture/adr-identity.toml missing grandfathered_prefixes".to_string()
        })?;
    let grandfathered_prefixes = prefixes
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect::<BTreeSet<_>>();
    let grandfathered_files = value
        .get("grandfathered_files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    Ok(AdrIdentityPolicy {
        grandfathered_debt,
        grandfathered_prefixes,
        grandfathered_files,
    })
}

pub fn load_spec_lifecycle(root: &Path) -> Result<Vec<SpecLifecycleRow>, String> {
    let path = root.join("architecture/spec-lifecycle.toml");
    if !path.is_file() {
        return Err("architecture/spec-lifecycle.toml is not a file".into());
    }
    let value = read_toml(&path)?;
    require_schema(&value, SPEC_LIFECYCLE_SCHEMA, &path)?;
    let rows = value
        .get("spec")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            "architecture/spec-lifecycle.toml missing [[spec]] array (malformed)".to_string()
        })?;
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for row in rows {
        let spec_path = row
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if spec_path.is_empty() {
            return Err("architecture/spec-lifecycle.toml [[spec]] missing path".into());
        }
        if !seen.insert(spec_path.clone()) {
            return Err(format!(
                "architecture/spec-lifecycle.toml duplicate spec path {spec_path}"
            ));
        }
        let successor = row
            .get("successor")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        out.push(SpecLifecycleRow {
            path: spec_path,
            state: row
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ownership: string_list(row, "ownership"),
            depends_on: string_list(row, "depends_on"),
            supersedes: string_list(row, "supersedes"),
            successor,
        });
    }
    Ok(out)
}

fn string_list(row: &toml::Value, key: &str) -> Vec<String> {
    row.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_adr_meta(filename: &str, text: &str) -> Result<AdrMeta, String> {
    let prefix = adr_filename_prefix(filename)
        .ok_or_else(|| format!("{filename} must match ^(\\d{{4}})-.+\\.md$"))?;
    let stem = filename.strip_suffix(".md").unwrap_or(filename).to_string();
    let block = extract_adr_meta_block(text).ok_or_else(|| {
        format!("{filename} missing weeping-angel-adr-meta block (malformed or absent)")
    })?;
    let value: toml::Value = block
        .parse()
        .map_err(|e| format!("{filename} weeping-angel-adr-meta is not parseable TOML: {e}"))?;
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if id != prefix {
        return Err(format!(
            "{filename} weeping-angel-adr-meta id {id:?} must match filename prefix {prefix}"
        ));
    }
    let status = value
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if !ADR_STATUSES.contains(&status.as_str()) {
        return Err(format!(
            "{filename} weeping-angel-adr-meta has illegal status {status:?}"
        ));
    }
    Ok(AdrMeta {
        filename: filename.to_string(),
        stem,
        prefix,
        id,
        status,
        supersedes: string_list(&value, "supersedes"),
        superseded_by: string_list(&value, "superseded_by"),
        depends_on: string_list(&value, "depends_on"),
    })
}

pub fn adr_filename_prefix(filename: &str) -> Option<String> {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    if !name.ends_with(".md") {
        return None;
    }
    let rest = name.strip_suffix(".md")?;
    let (prefix, slug) = rest.split_once('-')?;
    if prefix.len() == 4 && prefix.chars().all(|c| c.is_ascii_digit()) && !slug.is_empty() {
        Some(prefix.to_string())
    } else {
        None
    }
}

fn extract_adr_meta_block(text: &str) -> Option<String> {
    const START: &str = "<!-- weeping-angel-adr-meta";
    let start = text.find(START)? + START.len();
    let rest = &text[start..];
    let end = rest.find("-->")?;
    Some(rest[..end].trim().to_string())
}
