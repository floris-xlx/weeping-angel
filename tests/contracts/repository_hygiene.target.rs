//! Target suite for repository hygiene (`docs/specs/repository-hygiene.md` §4).
//!
//! Fail-closed invariants. RED on the characterization SHA until implement.
//! Do not `#[ignore]` these tests.
//!
//! Prompt 1–3 skip (do not collapse or rewrite):
//! - Prompt 1: `repository_integrity.*`, `xtask/tests/sdd_architectural_cleanup_*`,
//!   `docs/debt/register.toml`, `architecture/**`
//! - Prompt 2: catalog/framework/readiness product + those `*.target.rs`
//! - Prompt 3: temporal/lineage/evidence/SoA product + those `*.target.rs`
//! - All 16 `require_needles` `*.target.rs` files (rewrite later, non-concurrent)

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rel(path: &str) -> PathBuf {
    repo_root().join(path)
}

fn read(path: &str) -> String {
    fs::read_to_string(rel(path)).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn git_ls_files() -> Vec<String> {
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(repo_root())
        .output()
        .expect("git ls-files");
    assert!(
        out.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.replace('\\', "/"))
        .collect()
}

/// Line-level gitignore match (optional leading `/`). Comments ignored.
/// source-structure invariant: exact admission patterns are the hygiene law.
fn gi_has_line(gi: &str, pat: &str) -> bool {
    let want = pat.trim_start_matches('/');
    gi.lines().any(|l| {
        let t = l.split('#').next().unwrap_or("").trim();
        if t.is_empty() {
            return false;
        }
        let t = t.trim_start_matches('/');
        t == want || t == want.trim_end_matches('/')
    })
}

const SCHEMA_NAMES: &[&str] = &[
    "coverage.schema.json",
    "findings.schema.json",
    "scan-manifest.schema.json",
];

const SCHEMA_SSOT_DIR: &str = "schemas/codex-security";
const SCHEMA_SECOND_DIR: &str = "codex-security/schemas";
const SCHEMA_GENERATED_STAMP: &str = "codex-security/schemas/GENERATED_FROM_SSOT";

const BUDGETED_PREFIXES: &[&str] = &[
    "src/parse.rs",
    "src/http/",
    "src/authz.rs",
    "src/report/",
    "src/workbench/",
    "src/cli.rs",
    "src/lib.rs",
    "src/main.rs",
    "src/contract/",
    "src/discovery/",
    "src/depcheck/parsers/",
];

/// Prompt 1–3 owned dual-suite stems. Hygiene must not delete these.
const SKIP_COLLAPSE_STEMS: &[&str] = &[
    "repository_integrity",
    "temporal_lineage_evidence_soa",
    "canonical_assurance_catalog",
    "typed_evidence",
    "population_runtime",
    "iam_catalog",
    "sdlc_catalog",
    "vulnerability_catalog",
    "infrastructure_catalog",
    "governance_catalog",
    "github_collector",
    "applicability_engine",
    "iso27001_remap",
    "iso27001_assurance",
    "temporal_assurance",
    "evidence_validity_temporal_assurance",
    "assessment_lineage",
    "operational_soa",
    "continuous_assurance_scheduler",
    "assurance_runtime",
];

fn is_budgeted(path: &str) -> bool {
    BUDGETED_PREFIXES.iter().any(|p| path.starts_with(p))
}

fn strip_cfg_test_tails(src: &str) -> String {
    let mut out = String::new();
    let mut skipping = 0i32;
    let mut drop_next_item = false;
    for line in src.lines() {
        let t = line.trim_start();
        if skipping > 0 {
            skipping += t.matches('{').count() as i32;
            skipping -= t.matches('}').count() as i32;
            continue;
        }
        if t.starts_with("#[cfg(test)]") {
            drop_next_item = true;
            continue;
        }
        if drop_next_item {
            if t.starts_with("#[") {
                continue;
            }
            let opens = t.matches('{').count() as i32;
            let closes = t.matches('}').count() as i32;
            skipping = opens - closes;
            if skipping <= 0 && t.ends_with('{') {
                skipping = 1;
            }
            drop_next_item = false;
            continue;
        }
        if t.contains("cfg(test)") && t.contains("mod ") {
            skipping = t.matches('{').count() as i32 - t.matches('}').count() as i32;
            if skipping == 0 && t.ends_with('{') {
                skipping = 1;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn source_grep_helper() -> &'static str {
    concat!("require_", "needles")
}

fn hygiene_owned_paths() -> Vec<String> {
    let mut out = vec!["tests/contracts/repository_hygiene.target.rs".to_string()];
    let support = rel("tests/support");
    if support.is_dir()
        && let Ok(entries) = fs::read_dir(&support)
    {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("rs")
                && let Ok(rel_path) = p.strip_prefix(repo_root())
            {
                out.push(rel_path.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out
}

#[test]
fn dual_suite_is_registered_and_tests_sdd_stays_absent() {
    let cargo = read("Cargo.toml");
    assert!(
        !cargo.contains("name = \"sdd_repository_hygiene_baseline\""),
        "superseded hygiene baseline must not stay registered"
    );
    assert!(
        cargo.contains("name = \"sdd_repository_hygiene_target\""),
        "register sdd_repository_hygiene_target in root Cargo.toml"
    );
    assert!(
        !cargo.contains("path = \"tests/contracts/repository_hygiene.baseline.rs\"")
            && cargo.contains("path = \"tests/contracts/repository_hygiene.target.rs\""),
        "hygiene target lives under tests/contracts/; baseline deleted"
    );
    assert!(!rel("tests/sdd").exists(), "do not create tests/sdd/");
    assert!(
        !cargo.contains("tests/sdd/"),
        "Cargo.toml must not point at tests/sdd/"
    );
}

#[test]
fn audit_txt_is_not_tracked_and_is_gitignored() {
    let files = git_ls_files();
    assert!(
        !files
            .iter()
            .any(|p| p == "audit.txt" || p.ends_with("/audit.txt")),
        "audit.txt must not be tracked source"
    );
    let gi = read(".gitignore");
    assert!(
        gi_has_line(&gi, "audit.txt"),
        ".gitignore must have a dedicated audit.txt line"
    );
}

#[test]
fn pycache_bytecode_is_not_tracked_and_is_gitignored() {
    let files = git_ls_files();
    let pyc = files
        .iter()
        .filter(|p| p.ends_with(".pyc") || p.contains("__pycache__/"))
        .cloned()
        .collect::<Vec<_>>();
    assert!(pyc.is_empty(), "tracked python cache: {pyc:?}");
    let gi = read(".gitignore");
    assert!(
        gi_has_line(&gi, "__pycache__/") || gi_has_line(&gi, "__pycache__"),
        "gitignore must ignore __pycache__/"
    );
    assert!(gi_has_line(&gi, "*.pyc"), "gitignore must ignore *.pyc");
}

#[test]
fn schema_ssot_is_schemas_codex_security_and_second_copy_is_generated() {
    for name in SCHEMA_NAMES {
        let ssot = rel(&format!("{SCHEMA_SSOT_DIR}/{name}"));
        assert!(ssot.is_file(), "SSOT missing {SCHEMA_SSOT_DIR}/{name}");
    }

    let second_dir = rel(SCHEMA_SECOND_DIR);
    if !second_dir.is_dir() {
        return;
    }

    let stamp = rel(SCHEMA_GENERATED_STAMP);
    assert!(
        stamp.is_file(),
        "second schema tree must be generated packaging; missing stamp {SCHEMA_GENERATED_STAMP} \
         (bytes-identical hand copies are not an SSOT)"
    );
    let stamp_text = fs::read_to_string(&stamp).unwrap_or_default();
    assert!(
        stamp_text.contains(SCHEMA_SSOT_DIR),
        "generation stamp must name the SSOT directory {SCHEMA_SSOT_DIR}"
    );

    for name in SCHEMA_NAMES {
        let ssot = rel(&format!("{SCHEMA_SSOT_DIR}/{name}"));
        let second = rel(&format!("{SCHEMA_SECOND_DIR}/{name}"));
        if second.is_file() {
            let a = fs::read(&ssot).unwrap();
            let b = fs::read(&second).unwrap();
            assert_eq!(
                a, b,
                "second schema path must be byte-identical (SHA-256 equivalent) to SSOT for {name}"
            );
        }
    }
}

#[test]
fn gitignore_contains_hardened_hygiene_patterns() {
    let gi = read(".gitignore");
    assert!(
        gi.contains(".sdd/runs/") && gi.contains(".sdd/artifacts/"),
        "ADR 0004: keep .sdd/runs/ and .sdd/artifacts/"
    );

    let required = [
        ".env",
        ".env.*",
        "node_modules/",
        "target-*/",
        "__pycache__/",
        "*.pyc",
        "*.pem",
        "*.key",
        "*.sqlite",
        ".idea/",
        "audit.txt",
    ];
    let mut missing = Vec::new();
    for pat in required {
        let ok = match pat {
            "node_modules/" => {
                gi_has_line(&gi, "node_modules/") || gi_has_line(&gi, "node_modules")
            }
            "__pycache__/" => gi_has_line(&gi, "__pycache__/") || gi_has_line(&gi, "__pycache__"),
            ".idea/" => gi_has_line(&gi, ".idea/") || gi_has_line(&gi, ".idea"),
            "target-*/" => gi_has_line(&gi, "target-*/") || gi_has_line(&gi, "target*/"),
            other => gi_has_line(&gi, other),
        };
        if !ok {
            missing.push(pat);
        }
    }
    assert!(
        missing.is_empty(),
        ".gitignore missing dedicated hygiene lines (not substring hits): {missing:?}"
    );
}

#[test]
fn gitignore_does_not_hide_fixtures_or_schemas() {
    let gi = read(".gitignore");
    for pat in [
        "tests/fixtures",
        "tests/fixtures/",
        "fixtures/",
        "schemas/",
        "schemas/codex-security",
        "codex-security/examples",
        "codex-security/examples/",
    ] {
        assert!(
            !gi.lines().any(|l| {
                let t = l.split('#').next().unwrap_or("").trim();
                t == pat || t == format!("/{pat}")
            }),
            "must not gitignore {pat}"
        );
    }
    assert!(rel("tests/fixtures/completed-scan/findings.json").is_file());
    assert!(rel("codex-security/examples/completed-scan/findings.json").is_file());
    assert!(rel(&format!("{SCHEMA_SSOT_DIR}/findings.schema.json")).is_file());
}

#[test]
fn hygiene_owned_tests_do_not_use_source_grep_helpers() {
    let helper = source_grep_helper();
    let def = format!("fn {helper}(");
    let call = format!("{helper}(");
    for name in hygiene_owned_paths() {
        let path = rel(&name);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(path).unwrap();
        assert!(!text.contains(&def), "{name} must not define {helper}");
        assert!(!text.contains(&call), "{name} must not call {helper}");
    }
}

#[test]
fn budgeted_production_modules_have_no_unmarked_panic_on_input() {
    let files = git_ls_files();
    let mut offenders = Vec::new();
    for path in files
        .iter()
        .filter(|p| is_budgeted(p) && p.ends_with(".rs"))
    {
        let raw = fs::read_to_string(rel(path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let src = strip_cfg_test_tails(&raw);
        for (idx, line) in src.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || t.starts_with("///") {
                continue;
            }
            if t.contains("panic-ok:") {
                continue;
            }
            let panic_call = t.contains(".unwrap(") || t.contains(".expect(");
            if !panic_call {
                continue;
            }
            if t.contains("Regex::new(") {
                continue;
            }
            offenders.push(format!("{path}:{}:{t}", idx + 1));
        }
    }
    assert!(
        offenders.is_empty(),
        "unmarked unwrap/expect in budgeted production modules: {offenders:?}"
    );
}

#[test]
fn docs_contracts_readme_is_a_pointer_not_a_dual_suite_inventory() {
    let docs = read("docs/contracts/README.md");
    let braces = docs.matches("{baseline,target}").count();
    assert!(
        braces < 3,
        "docs/contracts/README.md must not hand-list dual-suites; brace mentions={braces}"
    );
    assert!(
        docs.matches("sdd_").count() < 5,
        "docs/contracts/README.md must not be an sdd_* inventory"
    );
    assert!(
        docs.contains("docs/specs/") && docs.contains("Cargo.toml"),
        "index must point at docs/specs/ and Cargo.toml instead of listing suites"
    );
}

#[test]
fn root_readme_stays_capability_oriented() {
    let readme = read("README.md");
    assert!(
        !readme.contains("sdd_"),
        "root README must not list sdd_* contract suites"
    );
    assert!(
        readme.contains("weeping-angel scan") || readme.contains("## CLI"),
        "root README stays capability + CLI oriented"
    );
}

#[test]
fn canonical_specs_indexes_repository_hygiene() {
    let layout = read("tests/contracts/documentation_layout.rs");
    assert!(
        layout.contains("docs/specs/repository-hygiene.md"),
        "CANONICAL_SPECS must include docs/specs/repository-hygiene.md"
    );
}

#[test]
fn hygiene_counts_live_outside_the_debt_register() {
    let spec = read("docs/specs/repository-hygiene.md");
    assert!(
        spec.contains("## 12. Before / after counts"),
        "counts live in docs/specs/repository-hygiene.md §12"
    );
    let after_still_tbd = spec
        .lines()
        .filter(|l| l.contains("| *TBD") || l.contains("| *TBD "))
        .count();
    assert!(
        after_still_tbd == 0,
        "§12 After column must be filled at implement (TBD rows={after_still_tbd}); \
         copy a snapshot to .sdd/runs/ — not docs/debt/register.toml"
    );

    let register = read("docs/debt/register.toml");
    assert!(
        !register.contains("repository-hygiene-counts")
            && !register.contains("sdd_repository_hygiene"),
        "do not record hygiene before/after counts in docs/debt/register.toml"
    );
}

#[test]
fn prompt_1_3_owned_suites_are_skipped_not_deleted() {
    let mut missing = Vec::new();
    for stem in SKIP_COLLAPSE_STEMS {
        let baseline = format!("tests/contracts/{stem}.baseline.rs");
        let target = format!("tests/contracts/{stem}.target.rs");
        if !rel(&target).is_file() && !rel(&baseline).is_file() {
            missing.push(stem.to_string());
        }
    }
    assert!(
        missing.is_empty(),
        "Prompt 1–3 skip-list suites vanished (hygiene must not collapse them): {missing:?}"
    );
    assert!(
        rel("xtask/tests/sdd_architectural_cleanup_baseline.rs").is_file()
            || rel("xtask/tests/sdd_architectural_cleanup_target.rs").is_file(),
        "Prompt 1 xtask architectural-cleanup suite is skipped, not deleted"
    );
}

#[test]
fn no_new_ignore_shortcut_on_hygiene_owned_files() {
    for name in hygiene_owned_paths() {
        let path = rel(&name);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(path).unwrap();
        for line in text.lines() {
            let t = line.trim_start();
            if !t.starts_with("#[ignore") {
                continue;
            }
            let allowed = name.ends_with("repository_hygiene.baseline.rs")
                && t.contains("superseded by sdd_repository_hygiene_target");
            assert!(
                allowed,
                "hygiene-owned {name} must not use #[ignore] as a shortcut: {t}"
            );
        }
    }
}
