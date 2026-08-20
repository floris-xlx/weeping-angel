//! DEBT-ENV P0 target — Cargo workspace SSOT close law.
//!
//! RED on CURRENT: virtual workspace, apps/cli package weeping-angel,
//! [workspace.package]/[workspace.dependencies], workspace=true internals,
//! rust-toolchain.toml + CI consumption, one [[test]] harness, and migrated
//! consumers are absent. GREEN after P0 implement. Keep this file as the pin.
//! After GREEN, delete sdd_consolidation_debt_env_baseline.rs.

use std::fs;
use std::path::{Path, PathBuf};

use xtask::CheckStatus;
use xtask::check_04_architecture_invariants;
use xtask::debt::KNOWN_CHECK_IDS;
use xtask::explain_invariant;
use xtask::inventory::InventoryReport;
use xtask::repo_root_from_xtask_manifest;

const INTERNAL_CRATES: [&str; 7] = [
    "weeping-angel-assurance",
    "weeping-angel-assurance-ir",
    "weeping-angel-canonical-catalog",
    "weeping-angel-collector",
    "weeping-angel-control-test",
    "weeping-angel-evidence",
    "weeping-angel-framework",
];

const SHARED_WORKSPACE_DEPS: [&str; 4] = ["chrono", "serde", "thiserror", "toml"];

const STABLE_TOOLCHAIN_WORKFLOWS: [&str; 4] = [
    ".github/workflows/ci.yml",
    ".github/workflows/compliance-regression.yml",
    ".github/workflows/security-diff.yml",
    ".github/workflows/release-provenance.yml",
];

const ROOT_CATALOG_NEEDLES: [&str; 7] = [
    "listed in root Cargo.toml",
    "registered in root Cargo.toml",
    "register sdd_",
    "root Cargo.toml must register",
    "stay registered in root Cargo.toml",
    "[[test]] names are listed in root",
    "[[test]] rows must be listed in root",
];

fn live_root() -> PathBuf {
    repo_root_from_xtask_manifest()
}

fn read_live(rel: &str) -> String {
    fs::read_to_string(live_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn parse_toml(rel: &str) -> toml::Value {
    read_live(rel)
        .parse()
        .unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

fn workspace_tables(root: &toml::Value) -> Option<&toml::Value> {
    root.get("workspace")
}

fn count_test_tables(path: &Path) -> u64 {
    let Ok(text) = fs::read_to_string(path) else {
        return 0;
    };
    text.lines().filter(|l| l.trim() == "[[test]]").count() as u64
}

fn weeping_angel_package_manifest() -> PathBuf {
    let root = live_root();
    let apps = root.join("apps/cli/Cargo.toml");
    if apps.is_file() {
        apps
    } else {
        root.join("Cargo.toml")
    }
}

fn package_inherits_workspace(pkg: &toml::Value, rel: &str) {
    for key in ["version", "edition", "license"] {
        let inherited = pkg
            .get(key)
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("workspace"))
            .and_then(|v| v.as_bool())
            == Some(true);
        assert!(inherited, "{rel} {key} must use workspace = true");
    }
}

#[test]
fn env_t01_virtual_workspace_apps_cli_named_weeping_angel() {
    let root = parse_toml("Cargo.toml");
    assert!(
        root.get("package").is_none(),
        "root Cargo.toml must be workspace-only (no [package])"
    );
    assert!(
        root.get("bin").is_none(),
        "virtual root must not declare [[bin]]"
    );
    let members = workspace_tables(&root)
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .unwrap_or_else(|| panic!("[workspace].members missing"));
    let members: Vec<&str> = members.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        members.iter().any(|m| *m == "apps/cli"),
        "workspace members must include apps/cli, got {members:?}"
    );
    assert!(
        members.iter().any(|m| *m == "xtask"),
        "xtask remains a workspace member"
    );
    for name in INTERNAL_CRATES {
        let want = format!("crates/{name}");
        assert!(members.iter().any(|m| *m == want), "missing member {want}");
    }
    assert_eq!(members.len(), 9, "nine members; do not add a 10th crate");
    assert!(
        !members
            .iter()
            .any(|m| *m == "." || m.contains("contract-test")),
        "no dummy root package and no contract-tests crate"
    );

    let cli = parse_toml("apps/cli/Cargo.toml");
    assert_eq!(
        cli.get("package")
            .and_then(|p| p.get("name"))
            .and_then(|v| v.as_str()),
        Some("weeping-angel"),
        "CLI package name remains weeping-angel (never weeping-angel-assurance-cli)"
    );
    assert!(
        live_root().join("apps/cli/src/main.rs").is_file(),
        "CLI source moves with the package"
    );
    assert!(
        live_root().join("apps/cli/src/cli.rs").is_file(),
        "CLI parser moves with the package"
    );
    assert!(
        !live_root().join("src/main.rs").is_file(),
        "repo-root src/main.rs must not remain the package entry"
    );
    assert!(
        !live_root()
            .join("crates/weeping-angel-assurance-cli/Cargo.toml")
            .is_file(),
        "FORBID-HYPOTHETICAL-ASSURANCE-CLI"
    );
}

#[test]
fn env_t02_workspace_package_and_dependencies_ssot() {
    let root = parse_toml("Cargo.toml");
    let ws = workspace_tables(&root).unwrap_or_else(|| panic!("[workspace] missing"));
    let pkg = ws
        .get("package")
        .unwrap_or_else(|| panic!("[workspace.package] missing"));
    for key in ["version", "edition", "license"] {
        assert!(pkg.get(key).is_some(), "[workspace.package] must own {key}");
    }
    let deps = ws
        .get("dependencies")
        .and_then(|d| d.as_table())
        .unwrap_or_else(|| panic!("[workspace.dependencies] missing"));
    for name in INTERNAL_CRATES {
        let entry = deps
            .get(name)
            .unwrap_or_else(|| panic!("[workspace.dependencies] missing {name}"));
        let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
        assert_eq!(
            path,
            format!("crates/{name}"),
            "{name} must be workspace-owned at crates/{name}, got {path}"
        );
    }
    for name in SHARED_WORKSPACE_DEPS {
        assert!(
            deps.get(name).is_some(),
            "[workspace.dependencies] must pin shared {name}"
        );
    }
}

#[test]
fn env_t03_internal_crates_inherit_and_workspace_true() {
    for name in INTERNAL_CRATES {
        let rel = format!("crates/{name}/Cargo.toml");
        let text = read_live(&rel);
        let manifest = parse_toml(&rel);
        let pkg = manifest
            .get("package")
            .unwrap_or_else(|| panic!("{rel} [package]"));
        package_inherits_workspace(pkg, &rel);
        assert!(
            !text.contains("path = \"../weeping-angel-"),
            "{rel} must not use relative internal paths"
        );
        if name != "weeping-angel-assurance-ir" {
            assert!(
                text.contains(".workspace = true"),
                "{rel} must consume workspace-owned internal deps"
            );
        }
    }
    let xtask_rel = "xtask/Cargo.toml";
    let xtask = parse_toml(xtask_rel);
    package_inherits_workspace(
        xtask
            .get("package")
            .unwrap_or_else(|| panic!("{xtask_rel} [package]")),
        xtask_rel,
    );
}

#[test]
fn env_t04_pinned_rust_toolchain_consumed_by_ci() {
    let text = read_live("rust-toolchain.toml");
    assert!(
        text.contains("[toolchain]"),
        "rust-toolchain.toml must declare [toolchain]"
    );
    assert!(
        text.contains("rustfmt") && text.contains("clippy"),
        "toolchain must include rustfmt and clippy"
    );
    let channel_is_stable_only = text.lines().any(|l| {
        let t = l.trim();
        t == "channel = \"stable\"" || t == "channel = 'stable'"
    });
    assert!(
        !channel_is_stable_only,
        "pin a versioned rustc channel, not floating stable"
    );
    let versioned = text.contains("channel = \"1.") || text.contains("channel = '1.");
    assert!(versioned, "channel must be a 1.x version pin");
    for rel in STABLE_TOOLCHAIN_WORKFLOWS {
        let wf = read_live(rel);
        assert!(
            !wf.contains("dtolnay/rust-toolchain@stable"),
            "{rel} must not use @stable as compiler SSOT"
        );
        assert!(
            wf.contains("rust-toolchain.toml"),
            "{rel} must consume rust-toolchain.toml"
        );
    }
}

#[test]
fn env_t05_one_harness_inventory_and_no_new_test_plane() {
    let report = InventoryReport::collect(&live_root());
    let harness = count_test_tables(&weeping_angel_package_manifest());
    assert_eq!(
        harness, 1,
        "weeping-angel package has one [[test]] table (45 -> 1)"
    );
    assert_eq!(
        report.counts.root_test_binaries, harness,
        "inventory root_test_binaries must count the weeping-angel package [[test]] tables"
    );
    assert_eq!(
        report.counts.root_test_binaries, 1,
        "collapse [[test]] catalog to one harness (45 -> 1)"
    );
    assert_eq!(
        report.extended.workspace_crates, 9,
        "workspace_crates remains 9 (7 libraries + xtask + apps/cli)"
    );
    let xtask_cargo = read_live("xtask/Cargo.toml");
    assert!(
        !xtask_cargo.contains("[[test]]"),
        "xtask tests stay auto-discovered"
    );
    let root_cargo = read_live("Cargo.toml");
    assert!(
        !root_cargo.contains("[[test]]"),
        "virtual root must not remain the [[test]] registry"
    );
    assert!(
        !root_cargo.contains("sdd_consolidation_debt_env"),
        "do not add a root [[test]] for DEBT-ENV"
    );
    assert!(!live_root().join("tests/sdd").exists());
    assert!(!live_root().join("test/sdd").exists());
}

#[test]
fn env_t06_architecture_consumers_point_at_apps_cli() {
    let arch = read_live("architecture/architecture.toml");
    assert!(
        arch.contains("apps/cli/src/main.rs") && arch.contains("apps/cli/src/cli.rs"),
        "ownership.assurance_cli paths must move with the package"
    );
    assert!(
        !arch.contains("paths = [\"src/main.rs\", \"src/cli.rs\"]"),
        "stale root CLI paths must not remain in architecture.toml"
    );
    let domain = read_live("architecture/domain-ownership.toml");
    assert!(
        domain.contains("apps/cli/src/main.rs")
            || domain.contains("facade = \"apps/cli/src/main.rs\""),
        "domain-ownership assurance_cli facade must follow apps/cli"
    );
    let forbidden = read_live("architecture/forbidden-patterns.toml");
    assert!(
        forbidden.contains("FORBID-HYPOTHETICAL-ASSURANCE-CLI"),
        "keep the hypothetical CLI forbid"
    );
    assert!(
        !forbidden.contains("root weeping-angel package (src/main.rs"),
        "FORBID rationale must not pin the fused-root CLI path"
    );
}

#[test]
fn env_t07_close_law_no_new_plane() {
    assert_eq!(KNOWN_CHECK_IDS.len(), 15);
    assert!(!KNOWN_CHECK_IDS.iter().any(|id| *id == "16"));
    let xtask_src = read_live("xtask/src/debt.rs");
    assert!(
        xtask_src.contains("KNOWN_CHECK_IDS: [&str; 15]"),
        "do not grow Guard ids"
    );
    let lib = read_live("xtask/src/lib.rs");
    assert!(
        !lib.contains("\"health\" =>"),
        "do not add cargo xtask health / a second health CLI"
    );
    let repo_files = [
        "Cargo.toml",
        "package.json",
        "docs/sdd/debt-env-p0-workspace-ssot-run/spec.md",
    ];
    for rel in repo_files {
        let text = read_live(rel);
        assert!(
            !text.to_ascii_lowercase().contains("turbo.json") && !text.contains("\"turbo\""),
            "{rel} must not introduce Turbo"
        );
    }
    assert!(!live_root().join("turbo.json").is_file());
}

#[test]
fn env_t08_cli_package_owns_bins_packager_and_workspace_deps() {
    let cli_text = read_live("apps/cli/Cargo.toml");
    let cli = parse_toml("apps/cli/Cargo.toml");
    package_inherits_workspace(
        cli.get("package")
            .unwrap_or_else(|| panic!("apps/cli/Cargo.toml [package]")),
        "apps/cli/Cargo.toml",
    );
    assert!(
        cli_text.contains("name = \"weeping-angel\"")
            && cli_text.contains("name = \"weeping-angel-docs-export\""),
        "both CLI bins move with apps/cli"
    );
    assert!(
        !cli_text.contains("path = \"crates/weeping-angel-")
            && !cli_text.contains("path = \"../weeping-angel-"),
        "CLI internal deps must be workspace-owned"
    );
    assert!(
        cli_text.contains(".workspace = true"),
        "apps/cli must consume [workspace.dependencies]"
    );
    assert!(
        cli_text.contains("[package.metadata.packager]")
            || cli_text.contains("before-packaging-command"),
        "packager metadata follows the publishable CLI package"
    );
    assert!(
        live_root()
            .join("apps/cli/src/bin/weeping-angel-docs-export.rs")
            .is_file(),
        "docs-export bin source moves with the CLI package"
    );
    let root = read_live("Cargo.toml");
    assert!(
        !root.contains("[package.metadata.packager]") && !root.contains("[package.metadata.wix]"),
        "root workspace must not remain the packager/WiX host"
    );
}

#[test]
fn env_t09_dist_ci_and_manifest_dir_consumers() {
    let dist = read_live("dist-workspace.toml");
    assert!(
        dist.contains("cargo:apps/cli"),
        "dist-workspace members must pin the CLI package, not cargo:."
    );
    assert!(
        !dist.contains("members = [\"cargo:.\"]"),
        "dist-workspace must not treat the virtual root as the cargo package"
    );

    let support = read_live("tests/support/mod.rs");
    let parent_hops = support.matches("parent()").count();
    assert!(
        parent_hops >= 2 || support.contains("apps/cli") || support.contains("repo_root"),
        "tests/support/mod.rs must resolve repo root from apps/cli CARGO_MANIFEST_DIR"
    );
    assert!(
        !support.contains("manifest_dir().join(\"crates\")"),
        "crate source walks must not treat CARGO_MANIFEST_DIR as the workspace root"
    );

    let layout = read_live("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("parent()") || layout.contains("apps/cli") || layout.contains("support"),
        "documentation_layout must not treat CARGO_MANIFEST_DIR as repo root after the CLI move"
    );
}

#[test]
fn env_t10_catalog_greps_and_adr_wording_migrated() {
    let contracts = live_root().join("tests/contracts");
    let entries = fs::read_dir(&contracts).unwrap_or_else(|e| panic!("read tests/contracts: {e}"));
    let mut offenders = Vec::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("dirent: {e}"));
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        if ROOT_CATALOG_NEEDLES.iter().any(|n| text.contains(n)) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                offenders.push(name.to_string());
            }
        }
    }
    offenders.sort();
    assert!(
        offenders.is_empty(),
        "contract tests still treat root Cargo.toml as the [[test]] catalog: {offenders:?}"
    );

    let adr4 = read_live("docs/adr/0004-documentation-architecture.md");
    assert!(
        !adr4.contains("remain explicitly listed in root"),
        "ADR 0004 rule 3 must drop the 45-row root catalog law"
    );
    assert!(
        adr4.contains("harness")
            || adr4.contains("one `[[test]]`")
            || adr4.contains("one [[test]]"),
        "ADR 0004 must state the one-harness law"
    );

    let adr12 = read_live("docs/adr/0012-repository-hygiene.md");
    assert!(
        !adr12.contains("remains explicitly listed in root `Cargo.toml`"),
        "ADR 0012 §4 must drop the per-suite root [[test]] listing"
    );

    let contracts_readme = read_live("docs/contracts/README.md");
    assert!(
        !contracts_readme.contains("rg \"^name = \\\"sdd_\\\" Cargo.toml\""),
        "docs/contracts/README.md must not discover suites via root Cargo.toml rg"
    );

    let adr51 = read_live("docs/adr/0051-repository-environment.md");
    assert!(
        adr51.contains("status = \"accepted\""),
        "ADR 0051 is Accepted only after P0 target GREEN"
    );
    assert!(
        !adr51.contains("status = \"proposed\""),
        "ADR 0051 must leave proposed once the pin is GREEN"
    );
}

#[test]
fn env_t11_expansion_restricted_stays_pass() {
    let explained = explain_invariant(&live_root(), "INV-CONSOLIDATION-EXPANSION-RESTRICTED");
    let text = explained.unwrap_or_else(|e| panic!("explain invariant: {e}"));
    assert!(
        text.contains("result: pass"),
        "INV-CONSOLIDATION-EXPANSION-RESTRICTED must stay pass (decrease allowed): {text}"
    );
    let c04 = check_04_architecture_invariants(&live_root());
    assert!(
        matches!(c04.status, CheckStatus::Pass),
        "check 04 must remain pass while P0 decreases root_test_binaries"
    );
}
