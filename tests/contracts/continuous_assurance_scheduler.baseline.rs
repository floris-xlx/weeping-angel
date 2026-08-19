//! Baseline suite for continuous-assurance scheduler.
//!
//! Characterization of CURRENT behavior on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` (`docs/specs/continuous-assurance-scheduler.md`
//! §3). There is no `weeping-angel-assurance::scheduler` module, no `Clock`
//! trait, and no cadence/retry/backoff/jitter/next-run contracts. `assess` is a
//! one-shot library call: collector `Err` evaluates `Vec::new()` without
//! loading `EvidenceLedger` envelopes (`max_age` hardcoded 24h). CLI has
//! `assurance` but no `isms`; collect/assess are banner stubs. Framework and
//! control-test remain network-free. Prompts 01–12 `IsmsContext` IR is not in
//! tree.
//!
//! Skip-superseded by `sdd_continuous_assurance_scheduler_target` (CAS-001…016 GREEN).
//! `#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]` — not a CI gate.
//! Does not implement the scheduler. Target CAS-001 still requires this file to exist.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use clap::Parser;
use weeping_angel::cli::{AssuranceCommand, Cli, Commands};
use weeping_angel_assurance::readiness::ControlReadiness;
use weeping_angel_assurance::{
    AssessmentRun, AssessmentScope, AssuranceEngine, FrameworkReadinessSnapshot, compare,
    compare_lineage, compare_runs, project_soa,
};
use weeping_angel_assurance_ir::{
    AssessmentId, AssetId, ControlId, ControlTestId, FrameworkVersion,
};
use weeping_angel_collector::{
    CollectorCapabilities, CollectorDescriptor, CollectorError, CollectorScope, EvidenceCollector,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};
use weeping_angel_evidence::{
    CollectionRun, EvidenceEnvelope, EvidenceLedger, EvidenceObservation, EvidenceProvenance,
    EvidenceType,
};
use weeping_angel_framework::{
    FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn crate_src(name: &str) -> PathBuf {
    let path = manifest_dir().join("crates").join(name).join("src");
    assert!(
        path.is_dir(),
        "expected crate sources at {}",
        path.display()
    );
    path
}

fn crate_sources_joined(name: &str) -> String {
    let mut files = Vec::new();
    walk_rs_files(&crate_src(name), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn product_crates_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&manifest_dir().join("crates"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn fn_assess(src: &str) -> &str {
    let start = src
        .find("pub fn assess(self, scope: AssessmentScope)")
        .expect("AssuranceEngineBuilder::assess must exist");
    let rest = &src[start..];
    let end = rest.find("\nfn evaluate_compiled").unwrap_or(rest.len());
    &rest[..end]
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

struct FailingCollector;

impl EvidenceCollector for FailingCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorDescriptor {
            id: "fixture.cas-failing".into(),
            version: "1".into(),
            evidence_types: BTreeSet::new(),
            provider_family: "fixture".into(),
            subject_types: BTreeSet::from(["repository".into()]),
            capabilities: CollectorCapabilities {
                offline: true,
                worker_safe: true,
                ..CollectorCapabilities::default()
            },
            required_permissions: Vec::new(),
        }
    }

    fn collect(&self, _scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        Err(CollectorError::InsufficientEvidence {
            detail: "forced collector failure for CAS baseline".into(),
        })
    }
}

fn assert_current_commands(command: &Commands) {
    match command {
        Commands::Scan(_)
        | Commands::Finalize(_)
        | Commands::ScanCode(_)
        | Commands::ScanDiff(_)
        | Commands::Workbench(_)
        | Commands::Depcheck(_)
        | Commands::Assurance(_)
        | Commands::Version
        | Commands::Completions { .. } => {}
    }
}

fn assert_current_assurance_command(command: &AssuranceCommand) {
    match command {
        AssuranceCommand::Framework(_)
        | AssuranceCommand::Collect(_)
        | AssuranceCommand::Evidence(_)
        | AssuranceCommand::Assess(_)
        | AssuranceCommand::Result(_)
        | AssuranceCommand::Compare(_)
        | AssuranceCommand::Soa(_)
        | AssuranceCommand::Catalog(_)
        | AssuranceCommand::Explain(_) => {}
    }
}

fn network_sdk_needles() -> &'static [&'static str] {
    &[
        "reqwest",
        "octocrab",
        "hyper",
        "aws-sdk",
        "tokio-tungstenite",
        "ureq",
    ]
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b001_dual_suite_baseline_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("name = \"sdd_continuous_assurance_scheduler_baseline\"")
            && toml
                .contains("path = \"tests/contracts/continuous_assurance_scheduler.baseline.rs\""),
        "baseline suite must be listed in root Cargo.toml (tests/contracts is not auto-discovered)"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/continuous_assurance_scheduler.baseline.rs")
            .is_file(),
        "baseline file must exist"
    );
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b002_no_scheduler_module_or_clock_trait() {
    let assurance_src = crate_src("weeping-angel-assurance");
    assert!(
        !assurance_src.join("scheduler.rs").exists(),
        "today there is no scheduler.rs under weeping-angel-assurance"
    );
    assert!(
        !assurance_src.join("scheduler").exists(),
        "today there is no scheduler/ module directory"
    );

    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !lib.contains("pub mod scheduler") && !lib.contains("mod scheduler"),
        "lib.rs currently does not declare a scheduler module"
    );

    let assurance = crate_sources_joined("weeping-angel-assurance");
    for needle in [
        "trait Clock",
        "pub trait Clock",
        "struct JobSpec",
        "enum JobKind",
        "struct JobState",
        "struct Schedule",
        "struct BackoffPolicy",
        "struct AssuranceScheduler",
        "fn tick(",
        "fn run_due(",
        "next_run",
        "last_successful_run",
        "failure_state",
    ] {
        assert!(
            !assurance.contains(needle),
            "assurance crate currently has no scheduler contract `{needle}`"
        );
    }

    let crates = product_crates_joined();
    assert!(
        !crates.contains("trait Clock"),
        "workspace crates currently declare no Clock trait"
    );
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b003_assess_is_one_shot_wall_clock_collect() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess = fn_assess(&src);
    assert!(
        assess.contains("CollectionRun::new(&descriptor.id, &descriptor.version)"),
        "today assess constructs CollectionRun::new from the collector descriptor"
    );
    assert!(
        assess.contains("collector.collect("),
        "today assess calls EvidenceCollector::collect once"
    );
    assert!(
        assess.contains("now: Utc::now()"),
        "today AssessmentContext.now is filled with Utc::now inside assess"
    );
    assert!(
        assess.contains("max_age: Duration::from_secs(24 * 3600)"),
        "today freshness is hardcoded to 24h"
    );
    assert!(
        !assess.contains("loop "),
        "today assess is not a scheduling loop"
    );
    for absent in [
        "EvidenceLedger",
        "load_lineage",
        "project_soa",
        "project_readiness",
        "compare(",
        "compare_runs",
        "timeout",
        "backoff",
        "jitter",
        "depends_on",
        "next_run",
    ] {
        assert!(
            !assess.contains(absent),
            "one-shot assess currently does not contain `{absent}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b004_collector_err_evaluates_empty_vec_without_ledger() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess = fn_assess(&src);
    assert!(
        assess.contains("Err(_err) =>"),
        "today collect Err is swallowed inside assess"
    );
    assert!(
        assess.contains("Vec::new()"),
        "today collect Err replaces envelopes with Vec::new()"
    );

    let report = AssuranceEngine::builder()
        .collector(FailingCollector)
        .framework(iso_target())
        .assess(AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .expect("assess currently returns Ok even when collect fails");

    assert_eq!(
        report.evidence_count, 0,
        "failed collect evaluates an empty EvidenceSet (no ledger reattach)"
    );
    let run = report
        .run
        .as_ref()
        .expect("current assess still builds AssessmentRun");
    assert_eq!(run.status, "failed");
    assert_eq!(run.collector_runs.len(), 1);
    assert!(
        run.collector_runs[0].starts_with("run:"),
        "CollectionRun.run_id currently uses the run: prefix"
    );
    assert!(
        !report.results.is_empty(),
        "ISO pack tests still evaluate against the empty set"
    );
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b005_collection_run_identity_is_wall_clock_not_idempotent() {
    let src = read_repo_file("crates/weeping-angel-evidence/src/lib.rs");
    let start = src
        .find("impl CollectionRun")
        .expect("CollectionRun impl must exist");
    let ctor = &src[start..];
    assert!(
        ctor.contains("let started_at = Utc::now();"),
        "CollectionRun::new currently stamps started_at from Utc::now"
    );
    assert!(
        ctor.contains("canonical_digest(&(collector_id.as_str(), started_at.to_rfc3339()))"),
        "run_id is digest(collector_id, started_at) rather than a scheduler slot identity"
    );

    let first = CollectionRun::new("fixture.cas", "1");
    std::thread::sleep(Duration::from_millis(15));
    let second = CollectionRun::new("fixture.cas", "1");
    assert_ne!(
        first.run_id, second.run_id,
        "two attempts of the same collector currently mint distinct wall-clock run ids"
    );
    assert!(first.run_id.starts_with("run:"));
    assert!(second.run_id.starts_with("run:"));
    let now = Utc::now();
    assert!(
        now.signed_duration_since(first.started_at) < chrono::Duration::seconds(5),
        "started_at is wall-clock now, not a fake-clock slot"
    );
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b006_ledger_survives_independently_but_assess_does_not_query_it() {
    let mut ledger = EvidenceLedger::open_in_memory().expect("open in-memory ledger");
    let observation = EvidenceObservation::new(EvidenceType::new("identity.privileged.mfa"))
        .with_fact("enabled", "true")
        .with_narrative("prior envelope for CAS baseline");
    let provenance = EvidenceProvenance {
        collector_id: "fixture.cas-failing".into(),
        collected_at: Utc::now(),
        scope: "repo:in-scope".into(),
        asset: AssetId::new("repo:in-scope"),
    };
    let envelope = EvidenceEnvelope::seal(observation, provenance).expect("seal prior envelope");
    let digest = envelope.digest().to_string();
    ledger.append(envelope).expect("append prior evidence");
    assert_eq!(ledger.query().expect("query").len(), 1);

    let report = AssuranceEngine::builder()
        .collector(FailingCollector)
        .framework(iso_target())
        .assess(AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .expect("assess Ok on collect Err");
    assert_eq!(
        report.evidence_count, 0,
        "one-shot assess does not reattach prior ledger envelopes"
    );
    assert_eq!(
        ledger.query().expect("query after failed assess").len(),
        1,
        "failed collect does not erase ledger rows (assess never talks to the ledger)"
    );
    assert!(
        ledger.query().unwrap().iter().any(|e| e.digest() == digest),
        "prior digest remains"
    );

    let ledger_src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        ledger_src.contains("INSERT OR IGNORE INTO evidence_envelopes"),
        "envelope append is INSERT OR IGNORE"
    );
    assert!(
        ledger_src.contains("INSERT OR REPLACE INTO collection_runs"),
        "record_collection_run is INSERT OR REPLACE"
    );
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b007_single_collector_on_builder_no_fan_out() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        src.contains("collector: Option<C>"),
        "builder currently holds a single collector"
    );
    assert!(
        !src.contains("collectors:"),
        "today there is no multi-collector fan-out field"
    );
    let collector_src = crate_sources_joined("weeping-angel-collector");
    assert!(
        collector_src.contains("fn collect(&self, scope: &CollectorScope)"),
        "EvidenceCollector::collect is the collector seam"
    );
    assert!(
        !collector_src.contains("use weeping_angel_assurance::")
            && !collector_src.contains("weeping_angel_assurance::scheduler"),
        "collectors must not depend on the assurance facade / scheduler"
    );
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b008_cli_has_assurance_but_no_isms() {
    let names: Vec<String> = Cli::clap_command()
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n == "assurance"),
        "CLI currently exposes assurance"
    );
    assert!(
        !names.iter().any(|n| n == "isms"),
        "CLI currently has no isms family"
    );

    let parsed = Cli::try_parse_from([
        "weeping-angel",
        "assurance",
        "assess",
        "--framework",
        "iso-27001",
    ])
    .expect("assurance assess currently parses");
    match parsed.command {
        Commands::Assurance(args) => assert_current_assurance_command(&args.command),
        other => panic!("expected Assurance, got {other:?}"),
    }

    let err = Cli::try_parse_from(["weeping-angel", "isms", "run"])
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("unrecognized subcommand")
            || err.contains("unexpected")
            || err.contains("isms"),
        "isms run must not parse today; clap error was: {err}"
    );

    let sample = Commands::Version;
    assert_current_commands(&sample);

    let cli_src = read_repo_file("src/cli.rs");
    assert!(
        !cli_src.contains("Isms") && !cli_src.to_ascii_lowercase().contains("isms"),
        "src/cli.rs currently has no isms command"
    );
    for flag in ["--cadence", "--backoff", "--jitter", "--timeout"] {
        assert!(
            !cli_src.contains(flag),
            "scheduling flags are not clap-defined today; found {flag}"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b009_assurance_assess_and_collect_are_banner_stubs() {
    let main = read_repo_file("src/main.rs");
    assert!(
        main.contains("This is a readiness assessment and is not certification."),
        "non-catalog/explain assurance arms currently print the not-certification banner"
    );
    assert!(
        main.contains("AssuranceCommand::Catalog") && main.contains("AssuranceCommand::Explain"),
        "Catalog and Explain are the only dispatched assurance arms"
    );
    assert!(
        !main.contains("AssuranceEngine") && !main.contains(".assess("),
        "main.rs currently does not invoke AssuranceEngine::assess"
    );
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b010_compare_exists_but_is_not_scheduled() {
    let previous = FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new("assess-cas-prev"),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: "pack-a".into(),
        assessment_digest: "digest-a".into(),
        evaluated_at: "2026-08-18T12:00:00Z".into(),
        requirements: Vec::new(),
        controls: vec![ControlReadiness {
            id: ControlId::new("canonical.source-control"),
            effectiveness: Effectiveness::Ineffective,
        }],
        effective: 0,
        ineffective: 1,
        partial: 0,
        manual_review: 0,
        insufficient_evidence: 0,
        not_applicable: 0,
        automation_coverage: "0%".into(),
        evidence_coverage: "0%".into(),
    };
    let next = FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new("assess-cas-next"),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: "pack-a".into(),
        assessment_digest: "digest-b".into(),
        evaluated_at: "2026-08-18T13:00:00Z".into(),
        requirements: Vec::new(),
        controls: vec![ControlReadiness {
            id: ControlId::new("canonical.source-control"),
            effectiveness: Effectiveness::Effective,
        }],
        effective: 1,
        ineffective: 0,
        partial: 0,
        manual_review: 0,
        insufficient_evidence: 0,
        not_applicable: 0,
        automation_coverage: "0%".into(),
        evidence_coverage: "0%".into(),
    };
    let diff = compare(&previous, &next);
    assert!(
        !diff.control_became_effective.is_empty(),
        "on-demand compare exists and reports effectiveness changes"
    );

    let run_a = AssessmentRun {
        id: AssessmentId::new("assess-cas-prev"),
        framework: "iso-27001".into(),
        framework_pack_digest: "pack-a".into(),
        ..Default::default()
    };
    let run_b = AssessmentRun {
        id: AssessmentId::new("assess-cas-next"),
        framework: "iso-27001".into(),
        framework_pack_digest: "pack-b".into(),
        ..Default::default()
    };
    let run_diff = compare_runs(&run_a, &run_b);
    assert!(run_diff.framework_pack_digest_changed);
    let lineage_diff = compare_lineage(&run_a, &run_b);
    assert!(lineage_diff.framework_pack_digest_changed);

    let assess_src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess = fn_assess(&assess_src);
    assert!(
        !assess.contains("compare"),
        "assess currently does not schedule Drift/compare"
    );
    let _ = project_soa;
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b011_framework_and_control_test_remain_network_free() {
    let framework_toml = read_repo_file("crates/weeping-angel-framework/Cargo.toml");
    let control_toml = read_repo_file("crates/weeping-angel-control-test/Cargo.toml");
    for needle in network_sdk_needles() {
        assert!(
            !framework_toml.contains(needle),
            "framework Cargo.toml must stay network-free; found `{needle}`"
        );
        assert!(
            !control_toml.contains(needle),
            "control-test Cargo.toml must stay network-free; found `{needle}`"
        );
    }
    assert!(
        !framework_toml.contains("weeping-angel-collector"),
        "framework must not depend on collectors"
    );
    assert!(
        !control_toml.contains("weeping-angel-collector"),
        "control-test must not depend on collectors"
    );
    for toml in [&framework_toml, &control_toml] {
        assert!(
            !toml.lines().any(|line| {
                let t = line.trim();
                t.contains("weeping-angel-assurance") && !t.contains("weeping-angel-assurance-ir")
            }),
            "framework and control-test must not depend on weeping-angel-assurance"
        );
    }

    let framework_src = crate_sources_joined("weeping-angel-framework");
    let control_src = crate_sources_joined("weeping-angel-control-test");
    for needle in ["pub mod scheduler", "trait Clock", "AssuranceScheduler"] {
        assert!(
            !framework_src.contains(needle) && !control_src.contains(needle),
            "network-free crates currently have no scheduler seam `{needle}`"
        );
    }
    assert!(
        control_src.contains("fn default_checked_at() -> DateTime<Utc>")
            && control_src.contains("Utc::now()"),
        "control-test timestamps currently default checked_at to Utc::now"
    );
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b012_no_isms_context_ir_and_collectors_do_not_set_effectiveness() {
    let crates = product_crates_joined();
    assert!(
        !crates.contains("struct IsmsContext") && !crates.contains("IsmsContext"),
        "Prompts 01–12 IsmsContext IR is not in tree"
    );

    let collector = crate_sources_joined("weeping-angel-collector");
    assert!(
        !collector.contains("Effectiveness::") && !collector.contains("set_compliant"),
        "collectors currently do not write compliance Effectiveness"
    );

    let _unused_result = ControlTestResult {
        test_id: ControlTestId::new("test.cas.baseline"),
        control_id: ControlId::new("canonical.source-control"),
        effectiveness: Effectiveness::InsufficientEvidence,
        rationale: "baseline".into(),
        evidence_refs: Vec::new(),
        missing_evidence: Vec::new(),
        checked_at: Utc::now(),
        test_version: "1".into(),
        input_digest: String::new(),
        duration: None,
        status: None,
        reason: None,
        population: None,
        period: None,
    };
}

#[test]
#[ignore = "superseded by sdd_continuous_assurance_scheduler_target"]
fn cas_b013_github_http_backoff_is_collector_internal_not_a_job_contract() {
    let github = read_repo_file("crates/weeping-angel-collector/src/github/mod.rs");
    assert!(
        github.contains("pub fn backoff("),
        "GitHub collector currently has HTTP backoff helpers"
    );
    let assurance = crate_sources_joined("weeping-angel-assurance");
    assert!(
        !assurance.contains("GitHubCollector::backoff") && !assurance.contains("Retry-After"),
        "assurance facade currently does not orchestrate GitHub HTTP backoff as job retry"
    );
}
