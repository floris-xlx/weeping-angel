//! Executable invariants for ADR 0004 documentation architecture.
//! Specs, decisions, and contracts live in git. Generated SDD traces do not.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("apps/cli CARGO_MANIFEST_DIR")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn exists(rel: &str) -> bool {
    repo_root().join(rel).is_file()
}

const CANONICAL_SPECS: &[&str] = &[
    "docs/specs/assurance-runtime.md",
    "docs/specs/assurance-runtime-spine.md",
    "docs/specs/iso-27001-automated-assurance-mvp.md",
    "docs/specs/iso-27001-canonical-remap.md",
    "docs/specs/canonical-assurance-catalog-v1.md",
    "docs/specs/typed-evidence.md",
    "docs/specs/population-runtime.md",
    "docs/specs/iam-canonical-assurance-catalog.md",
    "docs/specs/sdlc-canonical-assurance-catalog.md",
    "docs/specs/vulnerability-canonical-assurance-catalog.md",
    "docs/specs/infrastructure-canonical-assurance-catalog.md",
    "docs/specs/governance-canonical-assurance-catalog.md",
    "docs/specs/github-collector.md",
    "docs/specs/applicability-engine.md",
    "docs/specs/assessment-lineage.md",
    "docs/specs/operational-soa.md",
    "docs/specs/risk-register.md",
    "docs/specs/risk-treatment.md",
    "docs/specs/risk-identification.md",
    "docs/specs/control-implementation-registry.md",
    "docs/specs/incident-governance.md",
    "docs/specs/supplier-risk.md",
    "docs/specs/residual-risk.md",
    "docs/specs/isms-events-drift.md",
    "docs/specs/remediation-engine.md",
    "docs/specs/internal-audit.md",
    "docs/specs/nonconformity-capa.md",
    "docs/specs/personnel-security.md",
    "docs/specs/controlled-documents.md",
    "docs/specs/continuity-resilience.md",
    "docs/specs/scope-engine.md",
    "docs/specs/security-objectives.md",
    "docs/specs/risk-methodology.md",
    "docs/specs/continuous-assurance-scheduler.md",
    "docs/specs/isms-context.md",
    "docs/specs/interested-parties-obligations.md",
    "docs/specs/temporal-assurance.md",
    "docs/specs/evidence-validity-temporal-assurance.md",
    "docs/specs/repository-integrity.md",
    "docs/specs/architectural-cleanup-program.md",
    "docs/specs/architectural-consolidation-program.md",
    "docs/specs/repository-hygiene.md",
    "docs/specs/structural-reconciliation.md",
    "docs/specs/collector-hexagonal.md",
];

#[test]
fn canonical_specs_live_under_docs_specs() {
    for rel in CANONICAL_SPECS {
        assert!(exists(rel), "missing human spec {rel}");
    }
}

#[test]
fn decisions_live_under_docs_adr() {
    assert!(exists(
        "docs/adr/0001-inwardly-extensible-assurance-runtime.md"
    ));
    assert!(exists("docs/adr/0004-documentation-architecture.md"));
    assert!(exists("docs/adr/0017-controlled-documents.md"));
    assert!(exists("docs/adr/0026-isms-events-drift.md"));
    assert!(exists("docs/adr/0025-internal-audit.md"));
    assert!(exists("docs/adr/0028-nonconformity-capa.md"));
    assert!(exists("docs/adr/0041-risk-methodology.md"));
    assert!(exists("docs/adr/0040-operational-risk-register.md"));
    assert!(exists("docs/adr/0038-continuity-resilience.md"));
    assert!(exists("docs/adr/0008-isms-context.md"));
    assert!(exists("docs/adr/0044-scope-engine.md"));
    assert!(exists("docs/adr/0043-interested-parties-obligations.md"));
    assert!(exists("docs/adr/0045-security-objectives.md"));
    assert!(exists("docs/adr/0006-risk-treatment-engine.md"));
    assert!(exists("docs/adr/0007-supplier-risk.md"));
    assert!(exists(
        "docs/adr/0042-risk-identification-candidate-correlation.md"
    ));
    assert!(exists("docs/adr/0009-repository-health-gate.md"));
    assert!(exists("docs/adr/0010-architecture-as-law.md"));
    assert!(exists("docs/adr/0011-repository-guard-governance.md"));
    assert!(exists("docs/adr/0012-repository-hygiene.md"));
    assert!(exists("docs/adr/0051-repository-environment.md"));
    assert!(exists(
        "docs/adr/0013-collector-hexagonal-modular-monolith.md"
    ));
}

#[test]
fn adr_prefixes_are_unique() {
    let dir = repo_root().join("docs/adr");
    let mut seen = std::collections::BTreeMap::<String, Vec<String>>::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if let Some(prefix) = name.get(..4)
            && prefix.bytes().all(|b| b.is_ascii_digit())
        {
            seen.entry(prefix.to_string()).or_default().push(name);
        }
    }
    let dups: Vec<_> = seen.into_iter().filter(|(_, v)| v.len() > 1).collect();
    assert!(dups.is_empty(), "duplicate ADR prefixes: {dups:?}");
}

#[test]
fn executable_invariants_live_under_tests_contracts() {
    let harness = read("apps/cli/tests/harness.rs");
    assert!(
        harness.contains("tests/contracts/assurance_runtime.target.rs"),
        "dual-suite must remain a harness module under tests/contracts/"
    );
    assert!(
        !harness.contains("tests/sdd/"),
        "harness must not still point at tests/sdd/"
    );
    assert!(exists("tests/contracts/assurance_runtime.target.rs"));
    assert!(
        !repo_root().join("tests/sdd").exists(),
        "tests/sdd/ must not remain as a parallel tree"
    );
}

#[test]
fn gitignore_excludes_generated_sdd_traces() {
    let gi = read(".gitignore");
    assert!(
        gi.contains(".sdd/runs/"),
        ".gitignore must exclude .sdd/runs/"
    );
    assert!(
        gi.contains(".sdd/artifacts/"),
        ".gitignore must exclude .sdd/artifacts/"
    );
}

#[test]
fn docs_sdd_is_not_an_execution_dump() {
    let sdd = repo_root().join("docs/sdd");
    if !sdd.exists() {
        return;
    }
    let forbidden = collect_generated(&sdd);
    assert!(
        forbidden.is_empty(),
        "docs/sdd/ must not hold generated traces; found {forbidden:?} (use .sdd/runs/ or .sdd/artifacts/)"
    );
}

fn collect_generated(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "README.md" {
            continue;
        }
        if name.starts_with("sdd-")
            || name.ends_with("-telemetry.json")
            || name == "xylex"
            || name.contains("xylex-sdd")
        {
            out.push(name.into_owned());
        }
    }
    out
}
