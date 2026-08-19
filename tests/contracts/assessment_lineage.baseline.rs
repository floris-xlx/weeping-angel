//! SUPERSEDED by `sdd_assessment_lineage_target`.
//!
//! Historical shortcut characterization on SHA
//! `e430980c0d27a8138a153d49b62ddf3c57827891` (`docs/specs/assessment-lineage.md`
//! §3 / §4.11). Persist, explain, pure serialize, generic facade, and compare
//! are now the SSOT in the target suite. Tests are
//! `#[ignore = "superseded by sdd_assessment_lineage_target"]` so dropped-run /
//! serialize-time ISO / stub-assessment is not required CI green. Dual-suite
//! registration remains (LIN-009).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use clap::Parser;
use serde_json::Value;
use weeping_angel::cli::{AssuranceCommand, Cli, Commands};
use weeping_angel_assurance::readiness::ControlReadiness;
use weeping_angel_assurance::{
    AssessmentReport, AssessmentRun, FrameworkReadinessSnapshot, compare, project_soa,
};
use weeping_angel_assurance_ir::{
    ApplicabilityPredicate, ApplicabilityRule, AssessmentId, ControlId, ControlTestId,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};
use weeping_angel_framework::{FrameworkProfile, stub_catalog};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn read_repo_file(rel: &str) -> String {
    fs::read_to_string(manifest_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn impl_serialize_assessment_report(src: &str) -> &str {
    let start = src
        .find("impl Serialize for AssessmentReport")
        .expect("AssessmentReport must have a custom Serialize impl today");
    let rest = &src[start..];
    let end = rest
        .find("\nimpl ")
        .or_else(|| rest.find("\npub struct AssuranceEngine"))
        .unwrap_or(rest.len());
    &rest[..end]
}

fn fn_compare_body(src: &str) -> &str {
    let start = src
        .find("pub fn compare(")
        .expect("snapshot.rs must expose compare");
    let rest = &src[start..];
    let end = rest
        .find("\npub fn ")
        .or_else(|| rest.find("\n#[derive"))
        .unwrap_or(rest.len());
    &rest[..end]
}

fn fn_project_soa(src: &str) -> &str {
    let start = src
        .find("pub fn project_soa(")
        .expect("soa.rs must expose project_soa");
    &src[start..]
}

fn fn_assessment_for_target(src: &str) -> &str {
    let start = src
        .find("fn assessment_for_target(")
        .expect("assessment_for_target must exist");
    let rest = &src[start..];
    let end = rest.find("\n/// ").unwrap_or(rest.len());
    &rest[..end]
}

fn fn_assess(src: &str) -> &str {
    let start = src
        .find("pub fn assess(self, scope: AssessmentScope)")
        .expect("AssuranceEngineBuilder::assess must exist");
    let rest = &src[start..];
    let end = rest.find("\nfn evaluate_compiled").unwrap_or(rest.len());
    &rest[..end]
}

fn fn_normalize(src: &str) -> &str {
    let start = src
        .find("fn normalize(")
        .expect("framework compile normalize must exist");
    let rest = &src[start..];
    let end = rest
        .find("\nfn resolve_applicability")
        .unwrap_or(rest.len());
    &rest[..end]
}

fn fn_stub_catalog(src: &str) -> &str {
    let start = src
        .find("pub fn stub_catalog(")
        .expect("stub_catalog must exist");
    &src[start..]
}

fn sample_result(effectiveness: Effectiveness) -> ControlTestResult {
    ControlTestResult {
        test_id: ControlTestId::new("test.baseline.lineage"),
        control_id: ControlId::new("canonical.source-control"),
        effectiveness,
        rationale: "baseline characterization".into(),
        evidence_refs: Vec::new(),
        missing_evidence: Vec::new(),
        checked_at: Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap(),
        test_version: "1".into(),
        input_digest: String::new(),
        duration: None,
        status: None,
        reason: None,
        population: None,
        period: None,
    }
}

fn readiness_with(
    id: &str,
    digest: &str,
    controls: Vec<(&str, Effectiveness)>,
) -> FrameworkReadinessSnapshot {
    FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new(id),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: digest.into(),
        assessment_digest: digest.into(),
        evaluated_at: "2026-08-18T12:00:00Z".into(),
        requirements: Vec::new(),
        controls: controls
            .into_iter()
            .map(|(cid, effectiveness)| ControlReadiness {
                id: ControlId::new(cid),
                effectiveness,
            })
            .collect(),
        effective: 0,
        ineffective: 0,
        partial: 0,
        manual_review: 0,
        insufficient_evidence: 0,
        not_applicable: 0,
        automation_coverage: "0%".into(),
        evidence_coverage: "0%".into(),
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

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_assessment_lineage_baseline")
            && toml.contains("sdd_assessment_lineage_target")
            && toml.contains("tests/contracts/assessment_lineage.baseline.rs")
            && toml.contains("tests/contracts/assessment_lineage.target.rs"),
        "dual-suite must be listed in root Cargo.toml (tests/contracts is not auto-discovered)"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn assess_builds_then_drops_run_with_empty_collector_runs() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess = fn_assess(&src);
    assert!(
        assess.contains("let _run = AssessmentRun"),
        "today assess constructs AssessmentRun as let _run and discards it"
    );
    assert!(
        assess.contains("collector_runs: Vec::new()"),
        "today collector_runs is always empty even after collect"
    );
    assert!(
        !assess.contains("collector_runs:")
            || assess
                .lines()
                .any(|l| l.contains("collector_runs: Vec::new()")),
        "collector_runs must be the empty-vec shortcut"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn assessment_run_reuses_compile_digest_for_three_identities() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess = fn_assess(&src);
    for field in [
        "assessment_definition_digest: compiled.digest.clone()",
        "evidence_snapshot_digest: compiled.digest.clone()",
        "result_digest: compiled.digest.clone()",
    ] {
        assert!(
            assess.contains(field),
            "today {field} reuses the compile digest"
        );
    }
    assert!(
        assess.contains("status: \"completed\".into()"),
        "today AssessmentRun.status is always completed"
    );
    assert!(
        assess.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "today assess hardcodes the ISO 27001:2022 pack for framework_pack_digest"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn assessment_report_serialize_loads_iso_pack_and_formats_percentages() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let ser = impl_serialize_assessment_report(&src);
    assert!(
        ser.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "Serialize is not pure: it loads the ISO pack from disk"
    );
    assert!(
        ser.contains("automationCoverage") && ser.contains("{:.0}%"),
        "serialize invents automationCoverage as a formatted percent string"
    );
    assert!(
        ser.contains("evidenceCoverage") && ser.contains("{:.0}%"),
        "serialize invents evidenceCoverage as a formatted percent string"
    );
    assert!(
        ser.contains("let _ = (partial, \"collectionRunId\", \"evidenceRefs\")"),
        "serialize drops partial / collectionRunId / evidenceRefs"
    );

    let report = AssessmentReport {
        assessment_id: AssessmentId::new("assess-runtime-1"),
        profile: "soc-2".into(),
        digest: "compile-digest".into(),
        results: vec![sample_result(Effectiveness::Effective)],
        evidence_count: 0,
        ..Default::default()
    };
    let json = serde_json::to_value(&report).expect("serialize current AssessmentReport");
    let automation = json
        .get("automationCoverage")
        .and_then(Value::as_str)
        .expect("automationCoverage string");
    let evidence = json
        .get("evidenceCoverage")
        .and_then(Value::as_str)
        .expect("evidenceCoverage string");
    assert!(
        automation.ends_with('%'),
        "automationCoverage is a percent string today, got {automation}"
    );
    assert!(
        evidence.ends_with('%'),
        "evidenceCoverage is a percent string today, got {evidence}"
    );
    assert!(
        json.get("frameworkPackDigest").is_some(),
        "serialize emits frameworkPackDigest from the live ISO pack"
    );
    assert!(
        json.get("collectionRunId").is_none(),
        "collectionRunId is dropped at serialize time"
    );
    assert!(
        json.get("canonicalCatalogDigest").is_none(),
        "in-memory report has no stored catalog digest"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn assessment_for_target_uses_production_stub_and_iso_only_branch() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let body = fn_assessment_for_target(&src);
    assert!(
        body.contains("FrameworkProfile::Iso27001")
            && body.contains("\"2022\"")
            && body.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "assessment_for_target special-cases ISO 27001:2022"
    );
    assert!(
        body.contains("canonical:stub-1"),
        "non-ISO profiles compile a production stub requirement"
    );
    assert!(
        body.contains("assess-runtime-1"),
        "non-ISO profiles use production stub assessment id assess-runtime-1"
    );
    assert!(
        body.contains("canonical.source-control") && body.contains("ev.branch_protection"),
        "production stub wires canonical.source-control / ev.branch_protection"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn stub_catalog_and_normalize_special_case_iso_27001() {
    let framework = crate_sources_joined("weeping-angel-framework");
    let stub = fn_stub_catalog(&framework);
    assert!(
        stub.contains("FrameworkProfile::Iso27001")
            && stub.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "stub_catalog loads ISO 27001:2022 and returns [] for every other profile"
    );
    assert!(
        stub.contains("_ => Vec::new()"),
        "non-ISO stub_catalog is an empty vec"
    );

    let normalize = fn_normalize(&framework);
    assert!(
        normalize.contains("FrameworkProfile::Iso27001")
            && normalize.contains("\"2022\"")
            && normalize.contains("load_framework_pack(\"iso-27001\", \"2022\")"),
        "normalize merges only the hardcoded ISO 27001:2022 pack"
    );

    assert!(
        stub_catalog(FrameworkProfile::Gdpr).is_empty(),
        "non-ISO stub_catalog is empty today"
    );
    assert!(
        !stub_catalog(FrameworkProfile::Iso27001).is_empty(),
        "ISO stub_catalog currently loads the on-disk 2022 pack"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn compare_only_fills_effective_ineffective_stale() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    let body = fn_compare_body(&src);
    assert!(
        !body.contains("new_subjects")
            && !body.contains("new_exceptions")
            && !body.contains("catalog")
            && !body.contains("framework_pack_digest")
            && !body.contains("pack_digest"),
        "compare body does not write new_subjects, new_exceptions, or catalog/pack digest fields"
    );
    assert!(
        body.contains("control_became_effective")
            && body.contains("control_became_ineffective")
            && body.contains("evidence_became_stale"),
        "compare only walks effectiveness / stale today"
    );

    let previous = readiness_with(
        "assess-prev",
        "digest-a",
        vec![
            (
                "control.identity.privileged-mfa",
                Effectiveness::Ineffective,
            ),
            ("control.source.protected-branch", Effectiveness::Effective),
        ],
    );
    let next = readiness_with(
        "assess-next",
        "digest-b",
        vec![
            ("control.identity.privileged-mfa", Effectiveness::Effective),
            (
                "control.source.protected-branch",
                Effectiveness::StaleEvidence,
            ),
            ("control.identity.mfa", Effectiveness::Effective),
        ],
    );
    let diff = compare(&previous, &next);
    assert!(
        !diff.control_became_effective.is_empty(),
        "effectiveness flip is detected"
    );
    assert!(
        !diff.evidence_became_stale.is_empty(),
        "stale evidence is detected"
    );
    assert!(
        diff.new_subjects.is_empty()
            && diff.disappeared_subjects.is_empty()
            && diff.requirement_became_applicable.is_empty()
            && diff.requirement_became_not_applicable.is_empty()
            && diff.new_exceptions.is_empty()
            && diff.expired_exceptions.is_empty(),
        "subject / applicability / exception buckets stay empty today: {diff:?}"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn project_soa_reads_live_pack_applicability_toml() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/soa.rs");
    let body = fn_project_soa(&src);
    assert!(
        body.contains("resolve_pack_dir(framework, version)"),
        "project_soa resolves the current pack directory"
    );
    assert!(
        body.contains("applicability.toml"),
        "project_soa reads live applicability.toml"
    );
    assert!(
        body.contains("implementation_state: \"assessed\".into()")
            && body.contains("manual_review_state: \"pending\".into()"),
        "SoA entries are hardcoded assessed/pending with no evidence or exceptions"
    );

    let soa = project_soa("iso-27001", "2022");
    assert!(
        !soa.entries.is_empty(),
        "live ISO pack applicability.toml is readable today"
    );
    assert!(
        soa.entries
            .iter()
            .all(|e| e.implementation_state == "assessed" && e.manual_review_state == "pending"),
        "historical SoA cannot be distinguished from a later pack edit"
    );
    assert!(
        soa.entries
            .iter()
            .all(|e| e.evidence.is_empty() && e.exceptions.is_empty()),
        "project_soa does not pin evidence or exceptions"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn assurance_command_has_no_explain_variant() {
    let cli_src = read_repo_file("src/cli.rs");
    assert!(
        !cli_src.contains("Explain") && !cli_src.contains("explain"),
        "AssuranceCommand source currently has no Explain variant"
    );

    let cmd = Cli::clap_command();
    let assurance = cmd
        .get_subcommands()
        .find(|c| c.get_name() == "assurance")
        .expect("assurance family exists today");
    let names: Vec<&str> = assurance.get_subcommands().map(|c| c.get_name()).collect();
    assert_eq!(
        names,
        [
            "framework",
            "collect",
            "evidence",
            "assess",
            "result",
            "compare",
            "soa",
            "catalog"
        ],
        "AssuranceCommand is Framework/Collect/Evidence/Assess/Result/Compare/Soa/Catalog; have {names:?}"
    );
    assert!(
        !names.iter().any(|n| *n == "explain"),
        "current CLI has no `assurance explain` subcommand"
    );

    let parsed = Cli::try_parse_from([
        "weeping-angel",
        "assurance",
        "explain",
        "--assessment",
        "assess-runtime-1",
        "--control",
        "control.identity.privileged-mfa",
    ]);
    assert!(
        parsed.is_err(),
        "current clap parser rejects `assurance explain` (got {parsed:?})"
    );

    let listed = Cli::try_parse_from(["weeping-angel", "assurance", "framework", "list"])
        .expect("framework list already parses");
    match listed.command {
        Commands::Assurance(args) => assert_current_assurance_command(&args.command),
        other => panic!("expected Assurance, got {other:?}"),
    }
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn non_catalog_assurance_arms_print_banner_and_return_zero() {
    let main = read_repo_file("src/main.rs");
    assert!(
        main.contains("This is a readiness assessment and is not certification."),
        "non-catalog assurance arm prints the not-certification banner"
    );
    assert!(
        main.contains("AssuranceCommand::Catalog(catalog)"),
        "only Catalog is dispatched"
    );
    let after_catalog = main
        .split("AssuranceCommand::Catalog")
        .nth(1)
        .expect("Catalog arm");
    assert!(
        after_catalog.contains("0") && after_catalog.contains("_ =>"),
        "every other assurance subcommand is the wildcard arm that returns 0"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn ledger_creates_lineage_tables_without_persist_load_apis() {
    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    for table in [
        "CREATE TABLE IF NOT EXISTS assessment_runs",
        "CREATE TABLE IF NOT EXISTS control_test_runs",
        "CREATE TABLE IF NOT EXISTS framework_snapshots",
    ] {
        assert!(
            src.contains(table),
            "init creates {table} but there is no persist/load API yet"
        );
    }
    for needle in [
        "fn persist_assessment_run",
        "fn load_assessment_run",
        "fn persist_control_test_run",
        "fn load_control_test_run",
        "fn persist_framework_snapshot",
        "fn load_framework_snapshot",
    ] {
        assert!(
            !src.contains(needle),
            "ledger impl currently has no `{needle}`"
        );
    }
    assert!(
        src.contains("fn record_collection_run"),
        "collection_runs already have record_collection_run (INSERT OR REPLACE)"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn product_crates_lack_explanation_and_snapshot_types() {
    let crates = product_crates_joined();
    for needle in [
        "struct ControlExplanation",
        "struct ApplicabilitySnapshot",
        "struct EvidenceSnapshot",
        "struct AssessmentSummary",
        "struct CoverageMetrics",
        "struct ControlTestRun",
        "struct FrameworkPackSnapshot",
        "struct CanonicalCatalogSnapshot",
        "struct AssessmentDefinitionSnapshot",
        "struct StatementOfApplicabilitySnapshot",
    ] {
        assert!(
            !crates.contains(needle),
            "product crates currently have no `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn applicability_rule_is_static_only_engine_absent() {
    let ir = crate_sources_joined("weeping-angel-assurance-ir");
    assert!(
        ir.contains("pub enum ApplicabilityRule") && ir.contains("fn statically_applicable"),
        "IR still exposes ApplicabilityRule and statically_applicable"
    );
    let crates = product_crates_joined();
    for needle in [
        "OrgContext",
        "org_context",
        "ManualDeterminationRequired",
        "fn evaluate_org_context",
    ] {
        assert!(
            !crates.contains(needle),
            "applicability engine org-context evaluator must be absent; found `{needle}`"
        );
    }

    assert_eq!(
        ApplicabilityRule::Always.statically_applicable(),
        Some(true)
    );
    assert_eq!(
        ApplicabilityRule::Never.statically_applicable(),
        Some(false)
    );
    assert_eq!(
        ApplicabilityRule::Predicate(ApplicabilityPredicate::Jurisdiction("NL".into()))
            .statically_applicable(),
        None,
        "predicates stay unknown (None), never false"
    );
}

#[test]
#[ignore = "superseded by sdd_assessment_lineage_target"]
fn assessment_run_serde_shape_is_the_current_camel_case_record() {
    let run = AssessmentRun {
        id: AssessmentId::new("assess-runtime-1"),
        framework: "iso-27001".into(),
        framework_pack_digest: "pack".into(),
        assessment_definition_digest: "same".into(),
        started_at: "2026-08-18T12:00:00Z".into(),
        completed_at: "2026-08-18T12:00:01Z".into(),
        scope: "assess".into(),
        collector_runs: Vec::new(),
        evidence_snapshot_digest: "same".into(),
        result_digest: "same".into(),
        status: "completed".into(),
        ..Default::default()
    };
    let json = serde_json::to_value(&run).unwrap();
    for key in [
        "id",
        "framework",
        "frameworkPackDigest",
        "assessmentDefinitionDigest",
        "startedAt",
        "completedAt",
        "scope",
        "collectorRuns",
        "evidenceSnapshotDigest",
        "resultDigest",
        "status",
    ] {
        assert!(json.get(key).is_some(), "AssessmentRun JSON missing {key}");
    }
    assert!(
        json.get("canonicalCatalogDigest").is_none()
            && json.get("applicabilitySnapshotId").is_none(),
        "current AssessmentRun does not pin catalog digest or applicability snapshot id"
    );
}
