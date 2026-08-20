//! Single-load repository evaluation plane with a cached source index.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::architecture::{
    AdrIdentityPolicy, AdrMeta, ArchitectureInvariant, ArchitectureManifest, ForbiddenPattern,
    SpecLifecycleRow, load_adr_identity, load_architecture_manifest, load_forbidden_patterns,
    load_invariants, load_spec_lifecycle, parse_adr_meta,
};
use crate::debt::load_and_validate_debt_register;

const SKIP_DIR_NAMES: [&str; 5] = ["target", "node_modules", ".git", "__pycache__", "apps"];

/// Snapshot loaded once per `run_guard`: workspace, package graph, filesystem,
/// architecture manifests, debt register, ADR/spec metadata, framework packs,
/// catalog sources, and cached Rust source text.
#[derive(Debug, Clone)]
pub struct RepositoryModel {
    pub root: PathBuf,
    pub workspace_members: Vec<String>,
    /// package graph: crate name → direct dependency names
    pub package_graph: BTreeMap<String, BTreeSet<String>>,
    pub package_names: BTreeSet<String>,
    pub filesystem: BTreeSet<String>,
    pub architecture: Option<ArchitectureManifest>,
    pub architecture_error: Option<String>,
    pub invariants: Vec<ArchitectureInvariant>,
    pub invariants_error: Option<String>,
    pub forbidden: Vec<ForbiddenPattern>,
    pub forbidden_error: Option<String>,
    pub debt_ids: BTreeSet<String>,
    pub debt_error: Option<String>,
    pub adr_files: Vec<String>,
    pub spec_files: Vec<String>,
    pub framework_packs: Vec<String>,
    pub catalog_sources: Vec<String>,
    pub source_files: Vec<String>,
    /// Normalized UTF-8 source text keyed by repo-relative path (filled at load).
    pub source_cache: BTreeMap<String, String>,
    pub adr_identity: Option<AdrIdentityPolicy>,
    pub adr_identity_error: Option<String>,
    pub adr_docs: BTreeMap<String, AdrMeta>,
    pub adr_docs_error: Option<String>,
    pub spec_lifecycle: Vec<SpecLifecycleRow>,
    pub spec_lifecycle_error: Option<String>,
}

impl RepositoryModel {
    pub fn load(root: &Path) -> Self {
        let root = root.to_path_buf();
        let (workspace_members, package_names, package_graph) = load_workspace(&root);
        let mut filesystem = BTreeSet::new();
        for rel in [
            "architecture",
            "docs/adr",
            "docs/specs",
            "docs/debt",
            "frameworks",
            "catalog",
            "crates",
            "src",
            "tests",
        ] {
            index_tree(&root, &root.join(rel), &mut filesystem);
        }

        let (architecture, architecture_error) = match load_architecture_manifest(&root) {
            Ok(m) => (Some(m), None),
            Err(e) => (None, Some(e)),
        };
        let (invariants, invariants_error) = match load_invariants(&root) {
            Ok(rows) => (rows, None),
            Err(e) => (Vec::new(), Some(e)),
        };
        let (forbidden, forbidden_error) = match load_forbidden_patterns(&root) {
            Ok(rows) => (rows, None),
            Err(e) => (Vec::new(), Some(e)),
        };
        let (debt_ids, debt_error) = match load_and_validate_debt_register(&root) {
            Ok(ids) => (ids, None),
            Err(e) => (BTreeSet::new(), Some(e)),
        };

        let adr_files = list_dir_files(&root.join("docs/adr"), "md");
        let spec_files = list_dir_files(&root.join("docs/specs"), "md");
        let mut framework_packs = Vec::new();
        collect_files(&root, &root.join("frameworks"), &mut framework_packs);
        let mut catalog_sources = Vec::new();
        collect_files(&root, &root.join("catalog"), &mut catalog_sources);
        if filesystem
            .iter()
            .any(|p| p.starts_with("crates/weeping-angel-canonical-catalog"))
        {
            catalog_sources.push("crates/weeping-angel-canonical-catalog".into());
        }
        catalog_sources.sort();
        catalog_sources.dedup();

        let source_files: Vec<String> = filesystem
            .iter()
            .filter(|p| p.ends_with(".rs") && (p.starts_with("src/") || p.starts_with("crates/")))
            .cloned()
            .collect();

        let mut source_cache = BTreeMap::new();
        for rel in &source_files {
            if let Ok(text) = fs::read_to_string(root.join(rel)) {
                source_cache.insert(rel.clone(), text);
            }
        }

        let (adr_identity, adr_identity_error) = match load_adr_identity(&root) {
            Ok(p) => (Some(p), None),
            Err(e) => (None, Some(e)),
        };
        let (adr_docs, adr_docs_error) = load_adr_docs(&root, &adr_files);
        let (spec_lifecycle, spec_lifecycle_error) = match load_spec_lifecycle(&root) {
            Ok(rows) => (rows, None),
            Err(e) => (Vec::new(), Some(e)),
        };

        Self {
            root,
            workspace_members,
            package_graph,
            package_names,
            filesystem,
            architecture,
            architecture_error,
            invariants,
            invariants_error,
            forbidden,
            forbidden_error,
            debt_ids,
            debt_error,
            adr_files,
            spec_files,
            framework_packs,
            catalog_sources,
            source_files,
            source_cache,
            adr_identity,
            adr_identity_error,
            adr_docs,
            adr_docs_error,
            spec_lifecycle,
            spec_lifecycle_error,
        }
    }

    pub(crate) fn rel_exists(&self, rel: &str) -> bool {
        let trimmed = rel.trim_end_matches(['/', '\\']);
        self.root.join(trimmed).exists()
            || self.filesystem.contains(rel)
            || self.filesystem.contains(trimmed)
    }

    pub(crate) fn source_contains(&self, needle: &str) -> bool {
        self.source_cache.values().any(|text| text.contains(needle))
    }

    pub(crate) fn crate_source_contains(&self, crate_name: &str, needle: &str) -> bool {
        let prefix = format!("crates/{crate_name}/");
        self.source_cache
            .iter()
            .any(|(rel, text)| rel.starts_with(&prefix) && text.contains(needle))
    }

    pub(crate) fn forbidden_package_names(&self) -> Vec<String> {
        self.forbidden
            .iter()
            .filter(|p| p.kind.as_deref() == Some("package"))
            .map(|p| p.value.clone())
            .collect()
    }
}

fn load_adr_docs(root: &Path, adr_files: &[String]) -> (BTreeMap<String, AdrMeta>, Option<String>) {
    let mut docs = BTreeMap::new();
    let mut errors = Vec::new();
    for name in adr_files {
        let path = root.join("docs/adr").join(name);
        match fs::read_to_string(&path) {
            Ok(text) => match parse_adr_meta(name, &text) {
                Ok(meta) => {
                    docs.insert(meta.stem.clone(), meta);
                }
                Err(e) => errors.push(e),
            },
            Err(e) => errors.push(format!("read {}: {e}", path.display())),
        }
    }
    let err = if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    };
    (docs, err)
}

pub(crate) fn read_toml(path: &Path) -> Result<toml::Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    text.parse::<toml::Value>()
        .map_err(|e| format!("parse {}: {e}", path.display()))
}

pub(crate) fn require_schema(
    value: &toml::Value,
    expected: &str,
    path: &Path,
) -> Result<(), String> {
    let got = value.get("schema").and_then(|s| s.as_str());
    if got == Some(expected) {
        Ok(())
    } else {
        Err(format!(
            "{} schema must be {expected}, got {got:?}",
            path.display()
        ))
    }
}

fn load_workspace(
    root: &Path,
) -> (
    Vec<String>,
    BTreeSet<String>,
    BTreeMap<String, BTreeSet<String>>,
) {
    let mut members = Vec::new();
    let mut names = BTreeSet::new();
    let mut graph = BTreeMap::new();
    let cargo_path = root.join("Cargo.toml");
    let Ok(value) = read_toml(&cargo_path) else {
        return (members, names, graph);
    };
    if let Some(name) = value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
    {
        names.insert(name.to_string());
        graph.insert(name.to_string(), collect_dep_names(&value));
        members.push(".".to_string());
    }
    if let Some(listed) = value
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
    {
        for item in listed {
            let Some(rel) = item.as_str() else { continue };
            members.push(rel.to_string());
            let member_cargo = root.join(rel).join("Cargo.toml");
            if let Ok(member) = read_toml(&member_cargo)
                && let Some(name) = member
                    .get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
            {
                names.insert(name.to_string());
                graph.insert(name.to_string(), collect_dep_names(&member));
            }
        }
    }
    (members, names, graph)
}

fn collect_dep_names(value: &toml::Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = value.get(key).and_then(|v| v.as_table()) {
            out.extend(table.keys().cloned());
        }
    }
    out
}

fn index_tree(root: &Path, dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if SKIP_DIR_NAMES.iter().any(|s| *s == name_str) || name_str.starts_with("target") {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(root) {
            out.insert(rel.to_string_lossy().replace('\\', "/"));
        }
        if path.is_dir() {
            index_tree(root, &path, out);
        }
    }
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if SKIP_DIR_NAMES.iter().any(|s| *s == name) {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, out);
        } else if let Ok(rel) = path.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn list_dir_files(dir: &Path, ext: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some(ext)
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            out.push(name.to_string());
        }
    }
    out.sort();
    out
}
