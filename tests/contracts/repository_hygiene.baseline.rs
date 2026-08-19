//! Baseline characterization of repository hygiene debt
//! (`docs/specs/repository-hygiene.md` §3).
//!
//! Encodes what the tree does TODAY: ignored dual-suite leftovers, source-grep
//! helpers in Prompt 2/3 targets, production unwrap/expect volume, duplicate
//! Codex Security schema trees, tracked generated artifacts, and `.gitignore`
//! / docs-index gaps. Does **not** implement hygiene cleanup.
//!
//! SUPERSEDED by `sdd_repository_hygiene_target` (found-case debt no longer holds).

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

fn excluded(rel_path: &str) -> bool {
    rel_path.split('/').any(|c| {
        c == "target"
            || c.starts_with("target-")
            || c == "node_modules"
            || c == ".sdd"
            || c == ".git"
    })
}

fn tracked_rs() -> Vec<String> {
    git_ls_files()
        .into_iter()
        .filter(|p| p.ends_with(".rs") && !excluded(p))
        .collect()
}

struct IgnoreHit {
    path: String,
    line: String,
}

fn line_starting_ignores() -> Vec<IgnoreHit> {
    let mut hits = Vec::new();
    for path in tracked_rs() {
        let text = fs::read_to_string(rel(&path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        for line in text.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("#[ignore") {
                hits.push(IgnoreHit {
                    path: path.clone(),
                    line: trimmed.to_string(),
                });
            }
        }
    }
    hits
}

fn count_substring(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// Split so this suite is not itself a `require_needles` hit.
fn source_grep_helper() -> &'static str {
    concat!("require_", "needles")
}

fn source_grep_call() -> String {
    format!("{}(", source_grep_helper())
}

fn source_grep_def() -> String {
    format!("fn {}", source_grep_call())
}

fn contract_files(suffix: &str) -> Vec<String> {
    let dir = rel("tests/contracts");
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(suffix) {
            names.push(name);
        }
    }
    names.sort();
    names
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn line_starting_ignore_count_is_the_found_superseded_debt() {
    let hits = line_starting_ignores();
    assert!(
        hits.len() >= 600,
        "found-case ignore debt is hundreds of superseded attrs; got {}",
        hits.len()
    );
    assert_eq!(
        hits.iter()
            .filter(|h| !h.line.contains("superseded by"))
            .count(),
        0,
        "every line-starting #[ignore] is a superseded-by leftover"
    );

    let mut files: Vec<&str> = hits.iter().map(|h| h.path.as_str()).collect();
    files.sort();
    files.dedup();
    assert!(
        files.len() >= 40,
        "ignore attrs are spread across dual-suite files; got {}",
        files.len()
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn substring_ignore_mentions_include_comments_and_assertions() {
    let mut n = 0usize;
    for path in tracked_rs() {
        let text = fs::read_to_string(rel(&path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        n += count_substring(&text, "#[ignore");
    }
    assert!(
        n >= 650,
        "substring #[ignore (attrs + comments/strings) is the larger found count; got {n}"
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn ignored_tests_are_obsolete_baselines_plus_five_target_leftovers() {
    let hits = line_starting_ignores();
    let in_baselines = hits
        .iter()
        .filter(|h| h.path.starts_with("tests/contracts/") && h.path.ends_with(".baseline.rs"))
        .count();
    let in_targets = hits
        .iter()
        .filter(|h| h.path.starts_with("tests/contracts/") && h.path.ends_with(".target.rs"))
        .count();
    let in_xtask = hits.iter().filter(|h| h.path.starts_with("xtask/")).count();

    assert!(
        in_baselines >= 600,
        "obsolete dual-suite baselines hold almost all ignores; got {in_baselines}"
    );
    assert_eq!(
        in_targets, 5,
        "Prompt 2/3 leftover ignores inside *.target.rs (skip those files)"
    );
    assert_eq!(
        in_xtask, 11,
        "Prompt 1 xtask architectural-cleanup baseline ignores stay (skip)"
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn dual_suite_files_and_explicit_test_rows_are_the_found_scaffold() {
    let baselines = contract_files(".baseline.rs");
    let targets = contract_files(".target.rs");
    assert!(
        baselines.len() >= 38,
        "found-case tests/contracts/*.baseline.rs count; got {}",
        baselines.len()
    );
    assert!(
        targets.len() >= 39,
        "found-case tests/contracts/*.target.rs count; got {}",
        targets.len()
    );
    assert!(
        !rel("tests/sdd").exists(),
        "tests/sdd/ is absent (ADR 0004) and must stay absent"
    );

    let cargo = read("Cargo.toml");
    let rows = cargo.lines().filter(|l| *l == "[[test]]").count();
    assert!(
        rows >= 80,
        "root Cargo.toml [[test]] rows are the explicit dual-suite registry; got {rows}"
    );

    let root_tests = fs::read_dir(rel("tests"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x == "rs")
        })
        .count();
    assert_eq!(
        root_tests, 16,
        "tests/*.rs auto-discovered integration tests"
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn e2e_rows_keep_required_demo_features() {
    let cargo = read("Cargo.toml");
    assert!(cargo.contains("name = \"e2e_demo\""));
    assert!(cargo.contains("name = \"e2e_recon\""));
    assert!(
        cargo.contains("path = \"tests/e2e_demo.rs\"")
            && cargo.contains("required-features = [\"demo\"]"),
        "e2e_demo stays an explicit [[test]] because of required-features"
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn require_needles_helpers_live_only_in_sixteen_prompt_targets() {
    let mut defs = 0usize;
    let mut calls = 0usize;
    let mut def_files = Vec::new();
    for path in tracked_rs() {
        let text = fs::read_to_string(rel(&path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let def = source_grep_def();
        let call = source_grep_call();
        for line in text.lines() {
            let t = line.trim_start();
            if t.starts_with(&def) || t.starts_with(&format!("pub {def}")) {
                defs += 1;
                def_files.push(path.clone());
            }
        }
        calls += count_substring(&text, &call);
    }
    assert_eq!(defs, 16, "source-grep helper definitions");
    assert_eq!(calls, 203, "source-grep helper matches (defs + call sites)");
    assert!(
        def_files
            .iter()
            .all(|p| p.starts_with("tests/contracts/") && p.ends_with(".target.rs")),
        "all require_needles defs are Prompt 2/3 *.target.rs (skip rewrite): {def_files:?}"
    );
    assert!(
        !rel("tests/support").exists(),
        "tests/support/ does not exist on this SHA"
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn hygiene_owned_files_do_not_introduce_source_grep_helpers() {
    for name in [
        "tests/contracts/repository_hygiene.baseline.rs",
        "tests/contracts/repository_hygiene.target.rs",
    ] {
        let path = rel(name);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains(&source_grep_def()),
            "{name} must not add a source-grep helper definition"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn production_src_unwrap_expect_volume_is_the_found_panic_debt() {
    let mut unwraps = 0usize;
    let mut expects = 0usize;
    let mut crate_unwraps = 0usize;
    let mut crate_expects = 0usize;
    for path in tracked_rs() {
        let text = fs::read_to_string(rel(&path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let u = count_substring(&text, ".unwrap(");
        let e = count_substring(&text, ".expect(");
        if path.starts_with("src/") {
            unwraps += u;
            expects += e;
        }
        if path.starts_with("crates/") && path.contains("/src/") {
            crate_unwraps += u;
            crate_expects += e;
        }
    }
    assert!(
        unwraps + expects >= 200,
        "src/ unwrap+expect found-case ≥ 200; got unwrap={unwraps} expect={expects}"
    );
    assert!(unwraps >= 150, "src/ .unwrap() found-case; got {unwraps}");
    assert!(expects >= 50, "src/ .expect( found-case; got {expects}");
    assert_eq!(
        crate_unwraps, 1,
        "crates/**/src .unwrap() (Prompt 2/3 skip)"
    );
    assert!(
        crate_expects >= 20,
        "crates/**/src .expect( found-case (Prompt 2/3 skip); got {crate_expects}"
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn clippy_unwrap_lints_are_not_configured() {
    let cargo = read("Cargo.toml");
    assert!(
        !cargo.contains("unwrap_used") && !cargo.contains("expect_used"),
        "no Clippy unwrap_used / expect_used in root Cargo.toml"
    );
    assert!(
        !rel("clippy.toml").is_file(),
        "no clippy.toml unwrap budget"
    );
    assert!(!cargo.contains("[lints"), "root [lints] table is absent");
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn codex_security_schema_trees_are_duplicate_and_byte_identical() {
    const NAMES: &[&str] = &[
        "coverage.schema.json",
        "findings.schema.json",
        "scan-manifest.schema.json",
    ];
    for name in NAMES {
        let ssot = rel(&format!("schemas/codex-security/{name}"));
        let copy = rel(&format!("codex-security/schemas/{name}"));
        assert!(ssot.is_file(), "missing SSOT schemas/codex-security/{name}");
        assert!(
            copy.is_file(),
            "missing second tree codex-security/schemas/{name}"
        );
        let a = fs::read(&ssot).unwrap();
        let b = fs::read(&copy).unwrap();
        assert_eq!(
            a, b,
            "{name} trees exist as two tracked copies and are byte-identical"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn generated_audit_and_pycache_are_tracked_source() {
    let files = git_ls_files();
    assert!(
        files.iter().any(|p| p == "audit.txt"),
        "audit.txt is tracked generated xbp output"
    );
    let pyc = files.iter().filter(|p| p.ends_with(".pyc")).count();
    assert_eq!(pyc, 21, "tracked __pycache__ bytecode files");
    assert!(
        files
            .iter()
            .any(|p| p.contains("__pycache__/") && p.ends_with(".pyc")),
        "tracked pyc files live under __pycache__/"
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn gitignore_lacks_hardened_hygiene_patterns() {
    let gi = read(".gitignore");
    assert!(
        gi.contains(".sdd/runs/") && gi.contains(".sdd/artifacts/"),
        "ADR 0004 generated-trace exclusions stay"
    );
    let missing = [".env*", "target-*/", "__pycache__", "*.pem", "*.sqlite"]
        .iter()
        .filter(|p| !gi.contains(*p))
        .count();
    assert!(
        missing >= 1,
        ".gitignore currently lacks at least one of {{.env*, target-*/, __pycache__, *.pem, *.sqlite}}"
    );
    for pat in [
        ".env*",
        "target-*/",
        "__pycache__",
        "*.pem",
        "*.key",
        "*.sqlite",
        ".idea",
        "audit.txt",
        "*.pyc",
    ] {
        assert!(!gi.contains(pat), ".gitignore found-case still omits {pat}");
    }
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn gitignore_does_not_hide_fixtures_or_schema_examples() {
    let gi = read(".gitignore");
    for pat in [
        "tests/fixtures",
        "codex-security/examples",
        "schemas/codex-security",
    ] {
        assert!(
            !gi.lines()
                .any(|l| l.trim() == pat || l.trim() == format!("{pat}/")),
            ".gitignore must not hide {pat}"
        );
    }
    assert!(rel("tests/fixtures/completed-scan/findings.json").is_file());
    assert!(rel("codex-security/examples/completed-scan/findings.json").is_file());
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn docs_contracts_readme_is_a_hand_maintained_dual_suite_inventory() {
    let docs = read("docs/contracts/README.md");
    let mentions = count_substring(&docs, "{baseline,target}.rs");
    assert!(
        mentions >= 8,
        "docs/contracts/README.md enumerates dual-suites via {{baseline,target}}; got {mentions}"
    );
}

#[test]
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn root_readme_is_capability_oriented_not_a_suite_inventory() {
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
#[ignore = "superseded by sdd_repository_hygiene_target"]
fn hygiene_spec_exists_but_is_not_yet_in_documentation_layout_index() {
    assert!(
        rel("docs/specs/repository-hygiene.md").is_file(),
        "human SSOT spec must exist"
    );
    let layout = read("tests/contracts/documentation_layout.rs");
    assert!(
        !layout.contains("docs/specs/repository-hygiene.md"),
        "CANONICAL_SPECS found-case does not yet index the hygiene spec"
    );
}
