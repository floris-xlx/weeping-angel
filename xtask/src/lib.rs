//! Repository health gate: `cargo xtask guard`.
//!
//! Evaluation plane: `run_guard` loads one [`RepositoryModel`] (Cargo
//! workspace, package graph, filesystem, architecture manifests,
//! `docs/debt/register.toml`, `docs/adr` metadata, `docs/specs` metadata,
//! framework packs, catalog sources, cached source index) and runs
//! [`ArchitectureCheck::check`]. Checks do not reread the Rust tree.
//!
//! Implemented: 01–04, 13–15.
//! Product-semantic stubs 05–12 skip only with a live `DEBT-GUARD-NN`
//! finding (fail closed otherwise). No silent skips.

pub mod architecture;
pub mod checks;
pub mod debt;
pub mod model;
pub mod report;

use std::path::{Path, PathBuf};
use std::time::Instant;

pub use architecture::{
    ADR_IDENTITY_SCHEMA, ARCH_SCHEMA, ArchitectureInvariant, ArchitectureManifest,
    ArchitecturePolicy, FORBIDDEN_SCHEMA, ForbiddenPattern, INVARIANTS_SCHEMA, InvariantResult,
    OwnershipRow, REQUIRED_OWNERSHIP, SPEC_LIFECYCLE_SCHEMA,
};
pub use checks::{
    ArchitectureCheck, check_01_architecture_manifest, check_02_ownership,
    check_03_forbidden_patterns, check_04_architecture_invariants, explain_invariant,
};
pub use debt::{DEBT_SCHEMA, validate_debt_register_file, validate_debt_register_str};
pub use model::RepositoryModel;
pub use report::{
    CheckResult, CheckStatus, GUARD_REPORT_SCHEMA, GuardCounts, GuardReport, GuardSkip,
    GuardViolation,
};

/// Workspace root: parent of the `xtask` crate.
pub fn repo_root_from_xtask_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask crate lives under the workspace root")
        .to_path_buf()
}

pub fn main_with_args<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    match args.first().map(String::as_str) {
        Some("guard") => {
            let mut json = false;
            let mut selected: Option<String> = None;
            let mut explain: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--json" => json = true,
                    "--check" => {
                        i += 1;
                        match args.get(i) {
                            Some(id) => selected = Some(id.clone()),
                            None => {
                                eprintln!(
                                    "usage: cargo xtask guard [--json] [--check NN] [--explain INV-…]"
                                );
                                return 2;
                            }
                        }
                    }
                    "--explain" => {
                        i += 1;
                        match args.get(i) {
                            Some(id) => explain = Some(id.clone()),
                            None => {
                                eprintln!(
                                    "usage: cargo xtask guard [--json] [--check NN] [--explain INV-…]"
                                );
                                return 2;
                            }
                        }
                    }
                    other => {
                        eprintln!("unrecognized argument: {other}");
                        eprintln!(
                            "usage: cargo xtask guard [--json] [--check NN] [--explain INV-…]"
                        );
                        return 2;
                    }
                }
                i += 1;
            }

            let root = repo_root_from_xtask_manifest();
            if let Some(inv_id) = explain {
                match explain_invariant(&root, &inv_id) {
                    Ok(text) => {
                        print!("{text}");
                        0
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        1
                    }
                }
            } else {
                let report = run_guard_with_options(&root, selected.as_deref());
                if json {
                    println!("{}", report.to_json());
                } else {
                    print!("{}", report.render());
                }
                if report.failed() { 1 } else { 0 }
            }
        }
        _ => {
            eprintln!("usage: cargo xtask guard [--json] [--check NN] [--explain INV-…]");
            2
        }
    }
}

pub fn run_guard(root: &Path) -> GuardReport {
    run_guard_with_options(root, None)
}

fn run_guard_with_options(root: &Path, selected: Option<&str>) -> GuardReport {
    let started = Instant::now();
    let repo = RepositoryModel::load(root);
    let mut checks = checks::run_all_checks(&repo);
    if let Some(id) = selected {
        // CLI --check NN runs the selected check (model + debt already loaded).
        checks.retain(|c| c.id == id);
        if checks.is_empty() {
            checks.push(CheckResult::fail(
                id,
                "unknown-check",
                format!("unknown check {id}"),
            ));
        }
    }
    GuardReport::from_checks(checks, started.elapsed())
}
