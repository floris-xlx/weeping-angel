//! SUPERSEDED by `sdd_assurance_runtime_target` after Phases 0–8 landed.
//!
//! Historical characterization of the pre-assurance scanner-only tree
//! (`docs/specs/assurance-runtime-spine.md` §2 / §15.1). Kept for rollback
//! narrative. Do not delete. Tests are ignored because the workspace now
//! contains the assurance spine.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde_json::{Value, json};
use weeping_angel::cli::{Cli, Commands};
use weeping_angel::contract::{
    ArtifactRecord, Candidate, CoverageDocument, FindingsDocument, SemanticFinding,
    normalize_raw_candidate,
};
use weeping_angel::engines::EngineHit;
use weeping_angel::engines::web_adapt::web_finding_to_semantic;
use weeping_angel::finding::Finding;

const FORBIDDEN_COMPLIANCE_KEYS: &[&str] = &[
    "iso27001",
    "iso_27001",
    "iso27701",
    "iso_27701",
    "iso27007",
    "iso_27007",
    "gdpr",
    "soc2",
    "soc_2",
    "nis2",
    "dora",
    "controlresult",
    "control_result",
    "controltestresult",
    "frameworks",
];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_object_keys(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                out.insert(key.clone());
                collect_object_keys(child, out);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_object_keys(child, out);
            }
        }
        _ => {}
    }
}

fn assert_no_forbidden_keys(label: &str, value: &Value) {
    let mut keys = BTreeSet::new();
    collect_object_keys(value, &mut keys);
    for key in &keys {
        let folded = key.to_ascii_lowercase().replace('-', "_");
        assert!(
            !FORBIDDEN_COMPLIANCE_KEYS.contains(&folded.as_str()),
            "{label} serialized a forbidden compliance key `{key}` (keys: {keys:?})"
        );
    }
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn sample_hit() -> EngineHit {
    EngineHit {
        rule_id: "path-traversal.archive-extraction".into(),
        anchor: "archive-entry-write-without-containment".into(),
        instance: None,
        title: "Unsafe archive extraction".into(),
        summary: "An attacker-controlled path reaches a filesystem write.".into(),
        evidence: "open(join(dest, name))".into(),
        severity: "high",
        confidence: "high",
        confidence_rationale: "static pattern".into(),
        category: "path-traversal".into(),
        cwe: vec!["CWE-22".into()],
        remediation: "Contain extraction paths.".into(),
        path: "src/extract.py".into(),
        start_line: 41,
        end_line: Some(44),
        role: "sink",
        snippet: "extract(archive, dest)".into(),
        validation_json: None,
        attack_path_json: None,
    }
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn package_is_single_crate_not_workspace() {
    let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        cargo.contains("name = \"weeping-angel\""),
        "root package must remain weeping-angel"
    );
    assert!(
        !cargo.contains("[workspace]"),
        "current tree is a single package; found [workspace]"
    );
    assert!(
        !cargo.contains("weeping-angel-assurance"),
        "current tree must not declare weeping-angel-assurance* members"
    );
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn assurance_crates_are_absent() {
    let root = manifest_dir();
    let names = [
        "weeping-angel-assurance",
        "weeping-angel-assurance-ir",
        "weeping-angel-framework",
        "weeping-angel-evidence",
        "weeping-angel-collector",
        "weeping-angel-control-test",
    ];
    for name in names {
        assert!(
            !root.join(name).exists(),
            "unexpected crate directory at repo root: {name}"
        );
        assert!(
            !root.join("crates").join(name).exists(),
            "unexpected crate directory under crates/: {name}"
        );
    }
    assert!(
        !root.join("crates").is_dir(),
        "current tree has no crates/ workspace directory"
    );
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn lib_surface_is_scanner_only() {
    let lib = fs::read_to_string(manifest_dir().join("src/lib.rs")).unwrap();
    for needle in [
        "pub mod assurance",
        "pub mod framework",
        "pub mod collector",
        "pub mod control_test",
        "weeping_angel_assurance",
    ] {
        assert!(
            !lib.contains(needle),
            "src/lib.rs currently has no assurance surface; found `{needle}`"
        );
    }
    for module in [
        "pub mod engines;",
        "pub mod checks;",
        "pub mod contract;",
        "pub mod finding;",
        "pub mod cli;",
    ] {
        assert!(lib.contains(module), "expected scanner module `{module}`");
    }
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn commands_are_scan_finalize_code_diff_workbench_depcheck_version_completions() {
    let cmd = Cli::clap_command();
    let names: Vec<&str> = cmd.get_subcommands().map(|c| c.get_name()).collect();
    assert_eq!(
        names,
        vec![
            "scan",
            "finalize",
            "scan-code",
            "scan-diff",
            "workbench",
            "depcheck",
            "version",
            "completions",
        ]
    );
    assert!(
        !names
            .iter()
            .any(|n| n.contains("assurance") || n.contains("assess")),
        "Commands currently has no assurance surface: {names:?}"
    );
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn clap_rejects_assurance_subcommand() {
    let err = Cli::try_parse_from(["weeping-angel", "assurance"])
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("unrecognized subcommand") || err.contains("unexpected argument"),
        "expected clap to reject `assurance`, got: {err}"
    );
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn commands_match_is_exhaustive_without_assurance() {
    // Construct one of each current variant via clap; match must stay exhaustive.
    let samples = [
        vec![
            "weeping-angel",
            "scan",
            "example.com",
            "--i-own-this",
            "--allow-host",
            "example.com",
        ],
        vec!["weeping-angel", "finalize", "--scan-dir", "."],
        vec!["weeping-angel", "scan-code", ".", "-o", "out/code"],
        vec![
            "weeping-angel",
            "scan-diff",
            "--repo",
            ".",
            "-o",
            "out/diff",
        ],
        vec!["weeping-angel", "workbench", "list"],
        vec!["weeping-angel", "depcheck", "package.json"],
        vec!["weeping-angel", "version"],
        vec!["weeping-angel", "completions", "powershell"],
    ];
    let mut seen = BTreeSet::new();
    for argv in samples {
        let cli = Cli::try_parse_from(&argv).unwrap_or_else(|e| panic!("parse {argv:?}: {e}"));
        let tag = match cli.command {
            Commands::Scan(_) => "Scan",
            Commands::Finalize(_) => "Finalize",
            Commands::ScanCode(_) => "ScanCode",
            Commands::ScanDiff(_) => "ScanDiff",
            Commands::Workbench(_) => "Workbench",
            Commands::Depcheck(_) => "Depcheck",
            Commands::Version => "Version",
            Commands::Completions { .. } => "Completions",
            Commands::Assurance(_) => "Assurance",
        };
        seen.insert(tag);
    }
    assert_eq!(
        seen.len(),
        8,
        "expected all eight current Commands variants"
    );
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn semantic_finding_serde_is_security_only() {
    let finding = SemanticFinding::default();
    let value = serde_json::to_value(&finding).unwrap();
    let obj = value
        .as_object()
        .expect("SemanticFinding serializes as object");
    let keys: BTreeSet<&str> = obj.keys().map(String::as_str).collect();
    for required in [
        "findingId",
        "occurrenceId",
        "ruleId",
        "identity",
        "fingerprints",
        "title",
        "summary",
        "severity",
        "confidence",
        "taxonomy",
        "locations",
        "remediation",
        "provenance",
    ] {
        assert!(keys.contains(required), "missing security field {required}");
    }
    assert_no_forbidden_keys("SemanticFinding::default", &value);

    let fixture_path = manifest_dir().join("tests/fixtures/completed-scan/findings.json");
    let fixture: FindingsDocument =
        serde_json::from_str(&fs::read_to_string(fixture_path).unwrap()).unwrap();
    assert_eq!(fixture.document_type, "codex-security.findings");
    assert_eq!(fixture.findings.len(), 1);
    let fixture_value = serde_json::to_value(&fixture).unwrap();
    assert_no_forbidden_keys("fixture FindingsDocument", &fixture_value);
    assert!(
        fixture.findings[0].extensions.is_object(),
        "fixture extensions remain a JSON object, not a framework map"
    );
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn engine_hit_to_semantic_finding_extensions_are_engine_snippet_validation_method() {
    let semantic = sample_hit().to_semantic_finding();
    let value = serde_json::to_value(&semantic).unwrap();
    assert_no_forbidden_keys("EngineHit::to_semantic_finding", &value);

    let ext = semantic
        .extensions
        .as_object()
        .expect("extensions is an object");
    let ext_keys: BTreeSet<&str> = ext.keys().map(String::as_str).collect();
    assert_eq!(
        ext_keys,
        BTreeSet::from(["engine", "snippet", "validationMethod"])
    );
    assert_eq!(ext["engine"], json!("algorithmic"));
    assert_eq!(ext["snippet"], json!("extract(archive, dest)"));
    assert_eq!(ext["validationMethod"], json!("static-pattern"));
    assert_eq!(semantic.taxonomy.category, "path-traversal");
    assert_eq!(semantic.taxonomy.cwe, vec!["CWE-22".to_string()]);
    assert_eq!(semantic.provenance.source, "weeping-angel-engine");
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn web_finding_to_semantic_extensions_are_module_url_web_finding_id() {
    let finding: Finding = serde_json::from_value(json!({
        "id": "missing-csp",
        "title": "Missing CSP",
        "severity": "medium",
        "url": "https://example.com/",
        "module": "headers",
        "description": "No Content-Security-Policy",
        "remediation": "Add CSP",
        "cwe": "CWE-693",
        "evidence": [],
        "found_at": "2026-01-01T00:00:00Z"
    }))
    .unwrap();
    let finding_json = serde_json::to_value(&finding).unwrap();
    assert_no_forbidden_keys("finding::Finding", &finding_json);

    let semantic = web_finding_to_semantic(&finding);
    let value = serde_json::to_value(&semantic).unwrap();
    assert_no_forbidden_keys("web_finding_to_semantic", &value);

    let ext = semantic
        .extensions
        .as_object()
        .expect("extensions is an object");
    let ext_keys: BTreeSet<&str> = ext.keys().map(String::as_str).collect();
    assert_eq!(ext_keys, BTreeSet::from(["module", "url", "webFindingId"]));
    assert_eq!(ext["module"], json!("headers"));
    assert_eq!(ext["url"], json!("https://example.com/"));
    assert_eq!(ext["webFindingId"], json!("missing-csp"));
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn candidate_artifact_and_coverage_remain_codex_security_types() {
    let mut scope = BTreeSet::new();
    scope.insert("src/extract.py".into());
    let candidate = normalize_raw_candidate(
        &json!({
            "cwe_ids": ["CWE-22"],
            "locations": [{"path": "src/extract.py", "start_line": 41, "role": "sink"}],
            "summary": "path write",
            "evidence": "open(...)",
        }),
        &scope,
    )
    .unwrap();
    assert!(candidate.candidate_id.is_empty());
    assert_eq!(candidate.cwe_ids, vec!["CWE-22".to_string()]);
    let candidate_json = serde_json::to_value(&candidate).unwrap();
    assert_no_forbidden_keys("Candidate", &candidate_json);

    let rejected = normalize_raw_candidate(
        &json!({
            "cwe_ids": ["CWE-22"],
            "locations": [{"path": "src/extract.py", "start_line": 41}],
            "summary": "path write",
            "evidence": "open(...)",
            "iso_27001": "A.8.2",
        }),
        &scope,
    );
    assert!(
        rejected.is_err(),
        "raw Candidate currently fail-closes on unknown fields"
    );
    let err = rejected.unwrap_err().to_string();
    assert!(
        err.contains("unsupported field"),
        "expected unsupported-field reject, got: {err}"
    );

    let artifact = ArtifactRecord {
        path: "findings.json".into(),
        sha256: "0".repeat(64),
        media_type: "application/json".into(),
    };
    let artifact_json = serde_json::to_value(&artifact).unwrap();
    let artifact_keys: BTreeSet<&str> = artifact_json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        artifact_keys,
        BTreeSet::from(["path", "sha256", "mediaType"])
    );

    let coverage_path = manifest_dir().join("tests/fixtures/completed-scan/coverage.json");
    let coverage: CoverageDocument =
        serde_json::from_str(&fs::read_to_string(coverage_path).unwrap()).unwrap();
    assert_eq!(coverage.document_type, "codex-security.coverage");
    assert_eq!(coverage.completeness, "complete");
    assert_eq!(coverage.surfaces[0].disposition, "reported");
    let coverage_json = serde_json::to_value(&coverage).unwrap();
    assert_no_forbidden_keys("CoverageDocument", &coverage_json);
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn hit_to_candidate_stays_security_ledger_row() {
    let candidate = weeping_angel::engines::hit_to_candidate(&sample_hit(), "cand_1".into());
    assert_eq!(candidate.candidate_id, "cand_1");
    assert_eq!(candidate.cwe_ids, vec!["CWE-22".to_string()]);
    let json = serde_json::to_value(&candidate).unwrap();
    assert_no_forbidden_keys("hit_to_candidate", &json);
    let _typed: Candidate = serde_json::from_value(json).unwrap();
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn src_rust_has_no_framework_or_assurance_identifiers() {
    let src = manifest_dir().join("src");
    let mut files = Vec::new();
    walk_rs_files(&src, &mut files);
    assert!(!files.is_empty(), "expected src/**/*.rs");

    let needles = [
        "iso_27001",
        "iso27001",
        "iso_27701",
        "gdpr",
        "soc2",
        "EvidenceObservation",
        "EvidenceEnvelope",
        "ControlTestResult",
        "CompiledFramework",
        "FrameworkTarget",
        "AssuranceEngine",
    ];
    let mut hits = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).unwrap();
        let lower = text.to_ascii_lowercase();
        for needle in needles {
            if needle.chars().any(|c| c.is_ascii_uppercase()) {
                if text.contains(needle) {
                    hits.push(format!("{}: {needle}", path.display()));
                }
            } else if lower.contains(needle) {
                hits.push(format!("{}: {needle}", path.display()));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "src/**/*.rs currently has no assurance/framework identifiers; found {hits:?}"
    );
}

#[test]
#[ignore = "superseded by sdd_assurance_runtime_target"]
fn engines_and_checks_dirs_exist_without_assurance_bridge() {
    let root = manifest_dir();
    assert!(root.join("src/engines").is_dir());
    assert!(root.join("src/checks").is_dir());
    assert!(root.join("src/contract").is_dir());
    for missing in [
        "src/assurance",
        "src/collector",
        "src/control_test",
        "src/framework",
        "src/evidence",
    ] {
        assert!(
            !root.join(missing).exists(),
            "current tree has no {missing} module directory"
        );
    }
}
