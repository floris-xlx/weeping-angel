//! `cargo xtask inventory` — mechanical repository counts + debt snapshot.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const INVENTORY_SCHEMA: &str = "weeping-angel/inventory/v1";
pub const CONSOLIDATION_BASELINE_SCHEMA: &str = "weeping-angel/consolidation-baseline/v1";

const EXCLUSIONS: [&str; 3] = ["target/", "target-*", "node_modules/"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryCounts {
    pub root_test_binaries: u64,
    pub tests_rs_autodiscovered: u64,
    pub tests_contracts_rs: u64,
    pub ignored_test_attrs: u64,
    pub unwrap_calls: u64,
    pub expect_calls: u64,
    pub unwrap_plus_expect: u64,
    pub require_needles_fns: u64,
    pub require_needles_calls: u64,
    pub adr_markdown_files: u64,
    pub catalog_test_toml: u64,
    pub framework_packs: u64,
    pub schema_json_files: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryAbsences {
    pub inventory_module: bool,
    pub debt_current_md: bool,
    pub structural_reconciliation_spec: bool,
}

/// Phase 0 extra metrics projected from the same `walk_included` inventory walker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryExtended {
    pub workspace_crates: u64,
    pub rust_modules: u64,
    pub public_symbols: u64,
    pub pub_use_count: u64,
    pub public_structs: u64,
    pub public_enums: u64,
    pub duplicate_helper_definitions: u64,
    pub duplicate_type_names: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipSnapshot {
    pub concept: String,
    pub crate_name: String,
    pub kind: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryReport {
    pub schema: String,
    pub exclusions: Vec<String>,
    pub counts: InventoryCounts,
    pub absences: InventoryAbsences,
    pub extended: InventoryExtended,
    pub architecture_ownership: Vec<OwnershipSnapshot>,
    pub schema_locations: Vec<String>,
    pub spec_count: u64,
    pub debt_rows: u64,
    pub git_sha: Option<String>,
    pub generated_at: Option<String>,
}

impl InventoryReport {
    pub fn collect(root: &Path) -> Self {
        let mut files: Vec<PathBuf> = Vec::new();
        walk_included(root, root, &mut files);

        let mut ignored_test_attrs = 0u64;
        let mut unwrap_calls = 0u64;
        let mut expect_calls = 0u64;
        let mut require_needles_fns = 0u64;
        let mut require_needles_calls = 0u64;
        let mut schema_json_files = 0u64;
        let mut rust_modules = 0u64;
        let mut public_symbols = 0u64;
        let mut pub_use_count = 0u64;
        let mut public_structs = 0u64;
        let mut public_enums = 0u64;
        let mut schema_locations: Vec<String> = Vec::new();
        let mut type_name_files: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for path in &files {
            let rel = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if name.ends_with(".schema.json") || rel.ends_with(".schema.json") {
                schema_json_files += 1;
                schema_locations.push(rel.clone());
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            rust_modules += 1;
            let Ok(text) = fs::read_to_string(path) else {
                continue;
            };
            let mut defines_require_needles = false;
            for line in text.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("#[ignore") {
                    ignored_test_attrs += 1;
                }
                unwrap_calls += count_needle(line, ".unwrap()");
                expect_calls += count_needle(line, ".expect(");
                require_needles_calls += count_needle(line, "require_needles(");
                if trimmed.contains("fn require_needles") {
                    defines_require_needles = true;
                }
                if !trimmed.starts_with("//") {
                    public_symbols += count_needle(line, "pub fn ")
                        + count_needle(line, "pub async fn ")
                        + count_needle(line, "pub struct ")
                        + count_needle(line, "pub enum ")
                        + count_needle(line, "pub trait ")
                        + count_needle(line, "pub type ")
                        + count_needle(line, "pub const ");
                    pub_use_count += count_needle(line, "pub use ");
                    public_structs += count_needle(line, "pub struct ");
                    public_enums += count_needle(line, "pub enum ");
                    if let Some(ident) = extract_type_ident(trimmed) {
                        type_name_files
                            .entry(ident)
                            .or_default()
                            .insert(rel.clone());
                    }
                }
            }
            if defines_require_needles {
                require_needles_fns += 1;
            }
        }
        schema_locations.sort();

        let root_test_binaries = count_toml_tables(root.join("Cargo.toml"), "[[test]]");
        let tests_rs_autodiscovered = count_dir_ext(root.join("tests"), "rs", false);
        let tests_contracts_rs = count_dir_ext(root.join("tests/contracts"), "rs", false);
        let adr_markdown_files = count_dir_ext(root.join("docs/adr"), "md", false);
        let catalog_test_toml =
            count_dir_ext(root.join("catalog/canonical/v1/tests"), "toml", false);
        let framework_packs = count_framework_packs(root);
        let unwrap_plus_expect = unwrap_calls + expect_calls;

        let absences = InventoryAbsences {
            inventory_module: !root.join("xtask/src/inventory.rs").is_file(),
            debt_current_md: !root.join("docs/debt/current.md").is_file(),
            structural_reconciliation_spec: !root
                .join("docs/specs/structural-reconciliation.md")
                .is_file(),
        };

        let duplicate_type_names = type_name_files
            .values()
            .filter(|files| files.len() > 1)
            .count() as u64;
        let architecture_ownership = collect_ownership_snapshots(root);
        let spec_count = count_dir_ext(root.join("docs/specs"), "md", false);
        let debt_rows = count_toml_tables(root.join("docs/debt/register.toml"), "[[finding]]");
        let workspace_crates = count_workspace_crates(root);

        Self {
            schema: INVENTORY_SCHEMA.to_string(),
            exclusions: EXCLUSIONS.iter().map(|s| (*s).to_string()).collect(),
            counts: InventoryCounts {
                root_test_binaries,
                tests_rs_autodiscovered,
                tests_contracts_rs,
                ignored_test_attrs,
                unwrap_calls,
                expect_calls,
                unwrap_plus_expect,
                require_needles_fns,
                require_needles_calls,
                adr_markdown_files,
                catalog_test_toml,
                framework_packs,
                schema_json_files,
            },
            absences,
            extended: InventoryExtended {
                workspace_crates,
                rust_modules,
                public_symbols,
                pub_use_count,
                public_structs,
                public_enums,
                duplicate_helper_definitions: require_needles_fns,
                duplicate_type_names,
            },
            architecture_ownership,
            schema_locations,
            spec_count,
            debt_rows,
            git_sha: read_git_sha(root),
            generated_at: None,
        }
    }

    pub fn to_json(&self) -> String {
        let mut counts = BTreeMap::new();
        counts.insert("root_test_binaries", self.counts.root_test_binaries);
        counts.insert(
            "tests_rs_autodiscovered",
            self.counts.tests_rs_autodiscovered,
        );
        counts.insert("tests_contracts_rs", self.counts.tests_contracts_rs);
        counts.insert("ignored_test_attrs", self.counts.ignored_test_attrs);
        counts.insert("unwrap_calls", self.counts.unwrap_calls);
        counts.insert("expect_calls", self.counts.expect_calls);
        counts.insert("unwrap_plus_expect", self.counts.unwrap_plus_expect);
        counts.insert("require_needles_fns", self.counts.require_needles_fns);
        counts.insert("require_needles_calls", self.counts.require_needles_calls);
        counts.insert("adr_markdown_files", self.counts.adr_markdown_files);
        counts.insert("catalog_test_toml", self.counts.catalog_test_toml);
        counts.insert("framework_packs", self.counts.framework_packs);
        counts.insert("schema_json_files", self.counts.schema_json_files);

        let mut absences = BTreeMap::new();
        absences.insert("inventory_module", self.absences.inventory_module);
        absences.insert("debt_current_md", self.absences.debt_current_md);
        absences.insert(
            "structural_reconciliation_spec",
            self.absences.structural_reconciliation_spec,
        );

        let mut obj = serde_json::json!({
            "schema": self.schema,
            "exclusions": self.exclusions,
            "counts": counts,
            "absences": absences,
        });
        if let Some(sha) = &self.git_sha {
            obj["git_sha"] = serde_json::json!(sha);
        }
        if let Some(at) = &self.generated_at {
            obj["generated_at"] = serde_json::json!(at);
        }
        serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".into())
    }

    pub fn to_markdown(&self) -> String {
        let c = &self.counts;
        let mut out = String::new();
        out.push_str("# Current repository counts (mechanical)\n\n");
        out.push_str("Generated by `cargo xtask inventory --markdown`. ");
        out.push_str("This file is the **current** mechanical snapshot. ");
        out.push_str("`docs/debt/baseline-2026-08.md` is **Historical** evidence only.\n\n");
        out.push_str("Inclusion rule: all matching paths under the repo root **excluding** ");
        out.push_str("`target/`, `target-*`, and `node_modules/`.\n\n");
        out.push_str("## Counts\n\n");
        out.push_str("| Metric | Count |\n| --- | --- |\n");
        out.push_str(&format!(
            "| Root `[[test]]` binaries | {} |\n",
            c.root_test_binaries
        ));
        out.push_str(&format!(
            "| `tests/*.rs` (auto-discovered) | {} |\n",
            c.tests_rs_autodiscovered
        ));
        out.push_str(&format!(
            "| `tests/contracts/*.rs` | {} |\n",
            c.tests_contracts_rs
        ));
        out.push_str(&format!(
            "| ignored tests (`#[ignore`) | {} |\n",
            c.ignored_test_attrs
        ));
        out.push_str(&format!("| `.unwrap()` in `*.rs` | {} |\n", c.unwrap_calls));
        out.push_str(&format!("| `.expect(` in `*.rs` | {} |\n", c.expect_calls));
        out.push_str(&format!("| unwrap + expect | {} |\n", c.unwrap_plus_expect));
        out.push_str(&format!(
            "| Files defining `fn require_needles` | {} |\n",
            c.require_needles_fns
        ));
        out.push_str(&format!(
            "| `require_needles(` occurrences | {} |\n",
            c.require_needles_calls
        ));
        out.push_str(&format!(
            "| ADR markdown files | {} |\n",
            c.adr_markdown_files
        ));
        out.push_str(&format!(
            "| Catalog test TOML | {} |\n",
            c.catalog_test_toml
        ));
        out.push_str(&format!("| Framework packs | {} |\n", c.framework_packs));
        out.push_str(&format!(
            "| `*.schema.json` files | {} |\n",
            c.schema_json_files
        ));
        out.push_str("\n## Absences\n\n");
        out.push_str("| Key | Missing |\n| --- | --- |\n");
        out.push_str(&format!(
            "| `inventory_module` | {} |\n",
            self.absences.inventory_module
        ));
        out.push_str(&format!(
            "| `debt_current_md` | {} |\n",
            self.absences.debt_current_md
        ));
        out.push_str(&format!(
            "| `structural_reconciliation_spec` | {} |\n",
            self.absences.structural_reconciliation_spec
        ));
        out.push_str("\n## Stable marker\n\n");
        out.push_str("<!-- weeping-angel-inventory-stable -->\n");
        out
    }

    /// Compare recomputed markdown to committed `docs/debt/current.md` (ignore wall-clock).
    pub fn check_current_md(&self, root: &Path) -> Result<(), String> {
        let path = root.join("docs/debt/current.md");
        let committed =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let expected = self.to_markdown();
        let left = stable_markdown(&committed);
        let right = stable_markdown(&expected);
        if left == right {
            Ok(())
        } else {
            Err("docs/debt/current.md is out of sync with cargo xtask inventory --markdown".into())
        }
    }

    /// Frozen Phase 0 snapshot (`weeping-angel/consolidation-baseline/v1`).
    /// Line-based public-symbol scan; not a uniqueness ban.
    pub fn to_consolidation_baseline_json(&self) -> String {
        let mut counts = BTreeMap::new();
        counts.insert("root_test_binaries", self.counts.root_test_binaries);
        counts.insert(
            "tests_rs_autodiscovered",
            self.counts.tests_rs_autodiscovered,
        );
        counts.insert("tests_contracts_rs", self.counts.tests_contracts_rs);
        counts.insert("ignored_test_attrs", self.counts.ignored_test_attrs);
        counts.insert("unwrap_calls", self.counts.unwrap_calls);
        counts.insert("expect_calls", self.counts.expect_calls);
        counts.insert("unwrap_plus_expect", self.counts.unwrap_plus_expect);
        counts.insert("require_needles_fns", self.counts.require_needles_fns);
        counts.insert("require_needles_calls", self.counts.require_needles_calls);
        counts.insert("adr_markdown_files", self.counts.adr_markdown_files);
        counts.insert("catalog_test_toml", self.counts.catalog_test_toml);
        counts.insert("framework_packs", self.counts.framework_packs);
        counts.insert("schema_json_files", self.counts.schema_json_files);

        let e = &self.extended;
        let mut extended = BTreeMap::new();
        extended.insert("workspace_crates", e.workspace_crates);
        extended.insert("rust_modules", e.rust_modules);
        extended.insert("public_symbols", e.public_symbols);
        extended.insert("pub_use_count", e.pub_use_count);
        extended.insert("public_structs", e.public_structs);
        extended.insert("public_enums", e.public_enums);
        extended.insert(
            "duplicate_helper_definitions",
            e.duplicate_helper_definitions,
        );
        extended.insert("duplicate_type_names", e.duplicate_type_names);

        let ownership: Vec<serde_json::Value> = self
            .architecture_ownership
            .iter()
            .map(|row| {
                serde_json::json!({
                    "concept": row.concept,
                    "crate": row.crate_name,
                    "kind": row.kind,
                    "paths": row.paths,
                })
            })
            .collect();

        let obj = serde_json::json!({
            "schema": CONSOLIDATION_BASELINE_SCHEMA,
            "program": "architectural-consolidation",
            "phase": 0,
            "source": INVENTORY_SCHEMA,
            "exclusions": self.exclusions,
            "inventory_counts": counts,
            "extended": extended,
            "architecture_ownership": ownership,
            "schema_locations": self.schema_locations,
            "adr_count": self.counts.adr_markdown_files,
            "spec_count": self.spec_count,
            "debt_rows": self.debt_rows,
        });
        serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".into())
    }

    pub fn to_consolidation_baseline_markdown(&self) -> String {
        let c = &self.counts;
        let e = &self.extended;
        let mut out = String::new();
        out.push_str("# Frozen Phase 0 consolidation baseline\n\n");
        out.push_str("This file is the **frozen Phase 0** consolidation snapshot, **not** the live [`current.md`](current.md). ");
        out.push_str("`docs/debt/current.md` remains the live mechanical inventory projection (`weeping-angel/inventory/v1`). ");
        out.push_str(
            "Counts come from the same `xtask/src/inventory.rs` walker (`walk_included`). ",
        );
        out.push_str("Public-symbol / type-name metrics are a **line-based heuristic** (not a uniqueness ban).\n\n");
        out.push_str(&format!(
            "- schema: `{CONSOLIDATION_BASELINE_SCHEMA}`\n- program: `architectural-consolidation`\n- phase: `0`\n- source: `{INVENTORY_SCHEMA}`\n"
        ));
        out.push_str(&format!(
            "- exclusions: `{}`\n\n",
            self.exclusions.join("`, `")
        ));
        out.push_str("## Inventory counts\n\n");
        out.push_str("| Metric | Count |\n| --- | --- |\n");
        out.push_str(&format!(
            "| root_test_binaries | {} |\n",
            c.root_test_binaries
        ));
        out.push_str(&format!(
            "| tests_rs_autodiscovered | {} |\n",
            c.tests_rs_autodiscovered
        ));
        out.push_str(&format!(
            "| tests_contracts_rs | {} |\n",
            c.tests_contracts_rs
        ));
        out.push_str(&format!(
            "| ignored_test_attrs | {} |\n",
            c.ignored_test_attrs
        ));
        out.push_str(&format!("| unwrap_calls | {} |\n", c.unwrap_calls));
        out.push_str(&format!("| expect_calls | {} |\n", c.expect_calls));
        out.push_str(&format!(
            "| unwrap_plus_expect | {} |\n",
            c.unwrap_plus_expect
        ));
        out.push_str(&format!(
            "| require_needles_fns | {} |\n",
            c.require_needles_fns
        ));
        out.push_str(&format!(
            "| require_needles_calls | {} |\n",
            c.require_needles_calls
        ));
        out.push_str(&format!(
            "| adr_markdown_files | {} |\n",
            c.adr_markdown_files
        ));
        out.push_str(&format!(
            "| catalog_test_toml | {} |\n",
            c.catalog_test_toml
        ));
        out.push_str(&format!("| framework_packs | {} |\n", c.framework_packs));
        out.push_str(&format!(
            "| schema_json_files | {} |\n",
            c.schema_json_files
        ));
        out.push_str("\n## Extended\n\n");
        out.push_str("| Metric | Count |\n| --- | --- |\n");
        out.push_str(&format!("| workspace_crates | {} |\n", e.workspace_crates));
        out.push_str(&format!("| rust_modules | {} |\n", e.rust_modules));
        out.push_str(&format!("| public_symbols | {} |\n", e.public_symbols));
        out.push_str(&format!("| pub_use_count | {} |\n", e.pub_use_count));
        out.push_str(&format!("| public_structs | {} |\n", e.public_structs));
        out.push_str(&format!("| public_enums | {} |\n", e.public_enums));
        out.push_str(&format!(
            "| duplicate_helper_definitions | {} |\n",
            e.duplicate_helper_definitions
        ));
        out.push_str(&format!(
            "| duplicate_type_names | {} |\n",
            e.duplicate_type_names
        ));
        out.push_str(&format!("| adr_count | {} |\n", c.adr_markdown_files));
        out.push_str(&format!("| spec_count | {} |\n", self.spec_count));
        out.push_str(&format!("| debt_rows | {} |\n", self.debt_rows));
        out.push_str("\n## Schema locations\n\n");
        if self.schema_locations.is_empty() {
            out.push_str("(none)\n");
        } else {
            for loc in &self.schema_locations {
                out.push_str(&format!("- `{loc}`\n"));
            }
        }
        out.push_str("\n## Architecture ownership\n\n");
        out.push_str("| Concept | Crate | Kind | Paths |\n| --- | --- | --- | --- |\n");
        for row in &self.architecture_ownership {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                row.concept,
                row.crate_name,
                row.kind,
                row.paths.join(", ")
            ));
        }
        out.push_str("\n## Stable marker\n\n");
        out.push_str("<!-- weeping-angel-consolidation-baseline-stable -->\n");
        out
    }
}

pub fn json_u64(parent: Option<&serde_json::Value>, key: &str) -> u64 {
    parent
        .and_then(|v| v.get(key))
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_i64().and_then(|n| u64::try_from(n).ok()))
        })
        .unwrap_or(0)
}

fn stable_markdown(text: &str) -> String {
    text.lines()
        .filter(|l| !l.contains("generated_at") && !l.starts_with("Generated at"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn count_needle(line: &str, needle: &str) -> u64 {
    let mut count = 0u64;
    let mut rest = line;
    while let Some(idx) = rest.find(needle) {
        count += 1;
        rest = &rest[idx + needle.len()..];
    }
    count
}

/// Line-based `struct`/`enum` identifier (characterization; not a uniqueness ban).
fn extract_type_ident(trimmed: &str) -> Option<String> {
    let mut rest = trimmed;
    if let Some(r) = rest.strip_prefix("pub(crate) ") {
        rest = r;
    } else if let Some(r) = rest.strip_prefix("pub ") {
        rest = r;
    }
    for kw in ["struct ", "enum "] {
        if let Some(r) = rest.strip_prefix(kw) {
            let name: String = r
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn collect_ownership_snapshots(root: &Path) -> Vec<OwnershipSnapshot> {
    let path = root.join("architecture/architecture.toml");
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(value) = text.parse::<toml::Value>() else {
        return Vec::new();
    };
    let Some(table) = value.get("ownership").and_then(|v| v.as_table()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (concept, entry) in table {
        let crate_name = entry
            .get("crate")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let kind = entry
            .get("kind")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let paths = entry
            .get("paths")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        out.push(OwnershipSnapshot {
            concept: concept.clone(),
            crate_name,
            kind,
            paths,
        });
    }
    out.sort_by(|a, b| a.concept.cmp(&b.concept));
    out
}

fn count_workspace_crates(root: &Path) -> u64 {
    let Ok(text) = fs::read_to_string(root.join("Cargo.toml")) else {
        return 0;
    };
    let Ok(value) = text.parse::<toml::Value>() else {
        return 0;
    };
    let mut n = 0u64;
    if value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
        .is_some()
    {
        n += 1;
    }
    if let Some(members) = value
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
    {
        n += members.iter().filter(|v| v.as_str().is_some()).count() as u64;
    }
    n
}

fn count_toml_tables(path: PathBuf, marker: &str) -> u64 {
    let Ok(text) = fs::read_to_string(path) else {
        return 0;
    };
    text.lines().filter(|l| l.trim() == marker).count() as u64
}

fn count_dir_ext(dir: PathBuf, ext: &str, recursive: bool) -> u64 {
    let Ok(entries) = fs::read_dir(&dir) else {
        return 0;
    };
    let mut n = 0u64;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                n += count_dir_ext(path, ext, true);
            }
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            n += 1;
        }
    }
    n
}

fn count_framework_packs(root: &Path) -> u64 {
    let frameworks = root.join("frameworks");
    let Ok(entries) = fs::read_dir(&frameworks) else {
        return 0;
    };
    let mut n = 0u64;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // frameworks/<name>/<ver>/manifest.toml
        let Ok(versions) = fs::read_dir(&path) else {
            continue;
        };
        for ver in versions.flatten() {
            let ver_path = ver.path();
            if ver_path.join("manifest.toml").is_file() {
                n += 1;
            }
        }
    }
    n
}

fn read_git_sha(root: &Path) -> Option<String> {
    let head = root.join(".git/HEAD");
    let text = fs::read_to_string(head).ok()?;
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("ref: ") {
        let refer = root.join(".git").join(rest);
        fs::read_to_string(refer).ok().map(|s| s.trim().to_string())
    } else if text.len() >= 7 {
        Some(text.to_string())
    } else {
        None
    }
}

fn should_skip_dir(name: &str) -> bool {
    if name == "node_modules" || name == "target" || name == ".git" {
        return true;
    }
    if name.starts_with("target-") || name.starts_with("target_") {
        return true;
    }
    false
}

fn walk_included(_root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if path.is_dir() {
            if should_skip_dir(&name_str) {
                continue;
            }
            walk_included(_root, &path, out);
        } else {
            out.push(path);
        }
    }
}

/// CLI entry for `inventory` subcommand. Returns process exit code.
pub fn main_inventory(args: &[String]) -> i32 {
    let mut json = false;
    let mut markdown = false;
    let mut check = false;
    let mut consolidation_baseline = false;
    let mut consolidation_baseline_md = false;
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--json" => {
                if markdown || check || consolidation_baseline || consolidation_baseline_md {
                    eprintln!(
                        "usage: cargo xtask inventory [--json | --markdown | --check | --consolidation-baseline]"
                    );
                    return 2;
                }
                json = true;
            }
            "--markdown" => {
                if json || check || consolidation_baseline || consolidation_baseline_md {
                    eprintln!(
                        "usage: cargo xtask inventory [--json | --markdown | --check | --consolidation-baseline]"
                    );
                    return 2;
                }
                markdown = true;
            }
            "--check" => {
                if json || markdown || consolidation_baseline || consolidation_baseline_md {
                    eprintln!(
                        "usage: cargo xtask inventory [--json | --markdown | --check | --consolidation-baseline]"
                    );
                    return 2;
                }
                check = true;
            }
            "--consolidation-baseline" => {
                if json || markdown || check || consolidation_baseline_md {
                    eprintln!(
                        "usage: cargo xtask inventory [--json | --markdown | --check | --consolidation-baseline]"
                    );
                    return 2;
                }
                consolidation_baseline = true;
            }
            "--consolidation-baseline-markdown" => {
                if json || markdown || check || consolidation_baseline {
                    eprintln!(
                        "usage: cargo xtask inventory [--json | --markdown | --check | --consolidation-baseline]"
                    );
                    return 2;
                }
                consolidation_baseline_md = true;
            }
            other => {
                eprintln!("unrecognized argument: {other}");
                eprintln!(
                    "usage: cargo xtask inventory [--json | --markdown | --check | --consolidation-baseline]"
                );
                return 2;
            }
        }
    }

    let root = crate::repo_root_from_xtask_manifest();
    let report = InventoryReport::collect(&root);
    if json {
        println!("{}", report.to_json());
        0
    } else if consolidation_baseline {
        println!("{}", report.to_consolidation_baseline_json());
        0
    } else if consolidation_baseline_md {
        print!("{}", report.to_consolidation_baseline_markdown());
        0
    } else if markdown {
        print!("{}", report.to_markdown());
        0
    } else if check {
        match report.check_current_md(&root) {
            Ok(()) => {
                // Also enforce active-spec drift when checking inventory sync.
                if let Err(err) = crate::checks::check_active_spec_drift(&root) {
                    eprintln!("{err}");
                    return 1;
                }
                0
            }
            Err(err) => {
                eprintln!("{err}");
                1
            }
        }
    } else {
        let c = &report.counts;
        println!("weeping-angel inventory ({})", report.schema);
        println!("exclusions: {}", report.exclusions.join(", "));
        println!("root_test_binaries={}", c.root_test_binaries);
        println!("tests_rs_autodiscovered={}", c.tests_rs_autodiscovered);
        println!("tests_contracts_rs={}", c.tests_contracts_rs);
        println!("ignored_test_attrs={}", c.ignored_test_attrs);
        println!("unwrap_calls={}", c.unwrap_calls);
        println!("expect_calls={}", c.expect_calls);
        println!("unwrap_plus_expect={}", c.unwrap_plus_expect);
        println!("require_needles_fns={}", c.require_needles_fns);
        println!("require_needles_calls={}", c.require_needles_calls);
        println!("adr_markdown_files={}", c.adr_markdown_files);
        println!("catalog_test_toml={}", c.catalog_test_toml);
        println!("framework_packs={}", c.framework_packs);
        println!("schema_json_files={}", c.schema_json_files);
        0
    }
}
