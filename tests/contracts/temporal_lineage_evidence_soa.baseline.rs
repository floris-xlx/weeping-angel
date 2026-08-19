//! SUPERSEDED by `sdd_temporal_lineage_evidence_soa_target`.
//!
//! Historical characterization of SHA `0015f6395e7ead042e3cfd3066fefde3d39aa36b`
//! (`docs/specs/temporal-lineage-evidence-soa.md` §3): four-term aliases,
//! `asOf` serialized from `startedAt`, fail-open replay, collector-Err empty
//! bag, collection-run replace, untyped persist errors, live `project_soa` as
//! history. Target `sdd_temporal_lineage_evidence_soa_target` is the SSOT.
//! Tests are `#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]`.
//! Dual-suite registration remains. No product feature code lives here.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance::lineage::{
    ApplicabilitySnapshot, AssessmentDefinitionSnapshot, CanonicalCatalogSnapshot,
    EvidenceSnapshot, FrameworkPackSnapshot, LineageBundle, detect_digest_mismatch,
};
use weeping_angel_assurance::readiness::FrameworkReadinessSnapshot;
use weeping_angel_assurance::{
    AssessmentRun, AssessmentScope, AssuranceEngine, AssuranceError, StatementOfApplicability,
    StatementOfApplicabilitySnapshot, reconstruct, replay_assessment,
};
use weeping_angel_assurance_ir::{
    AssessmentId, AssetId, ControlId, ControlTestId, FrameworkVersion,
};
use weeping_angel_collector::{
    CollectorCapabilities, CollectorDescriptor, CollectorError, CollectorScope, EvidenceCollector,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, EvidenceSelector, EvidenceSet,
    FreshnessPolicy, PeriodEffectiveness, SubjectSelector, TestExpr, project_period_effectiveness,
};
use weeping_angel_evidence::{
    CollectionRun, EvidenceEnvelope, EvidenceLedger, EvidenceObservation, EvidenceProvenance,
    EvidenceType, EvidenceValidityEvent, LedgerError,
};
use weeping_angel_framework::{
    Assessment, FrameworkCapabilities, FrameworkContext, FrameworkProfile, FrameworkTarget,
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

fn ts(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
}

fn evidence_type() -> EvidenceType {
    EvidenceType::new("identity.privileged.mfa")
}

fn seal_at(collected_at: DateTime<Utc>, salt: &str) -> EvidenceEnvelope {
    let observation = EvidenceObservation::new(evidence_type())
        .with_fact("enabled", "true")
        .with_fact("salt", salt)
        .with_narrative("privileged MFA is enabled");
    EvidenceEnvelope::seal(
        observation,
        EvidenceProvenance {
            collector_id: "fixture.tle-baseline".into(),
            collected_at,
            scope: "repo:in-scope".into(),
            asset: AssetId::new("repo:in-scope"),
        },
    )
    .expect("seal baseline envelope")
}

fn iso_target() -> FrameworkTarget {
    FrameworkTarget {
        profile: FrameworkProfile::Iso27001,
        capabilities: FrameworkCapabilities::default(),
        version: FrameworkVersion::new("2022"),
        context: FrameworkContext::default(),
    }
}

fn fn_assess(src: &str) -> &str {
    let start = src
        .find("pub fn assess(self, scope: AssessmentScope)")
        .expect("AssuranceEngineBuilder::assess must exist");
    let rest = &src[start..];
    rest.split("\nfn evaluate_compiled").next().unwrap_or(rest)
}

fn fn_replay(src: &str) -> &str {
    let start = src
        .find("pub fn replay_assessment(")
        .expect("replay_assessment must exist");
    let rest = &src[start..];
    rest.split("\npub fn ").nth(0).unwrap_or(rest)
}

struct FailingCollector;

impl EvidenceCollector for FailingCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorDescriptor {
            id: "fixture.tle-fail".into(),
            version: "1".into(),
            evidence_types: BTreeSet::from([evidence_type()]),
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
            detail: "forced collector failure for TLE baseline".into(),
        })
    }
}

fn empty_readiness() -> FrameworkReadinessSnapshot {
    FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new("assess-unpinned"),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: "pack-live".into(),
        catalog_digest: String::new(),
        assessment_digest: "def-live".into(),
        evaluated_at: "2026-01-01T00:00:00Z".into(),
        requirements: Vec::new(),
        controls: Vec::new(),
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

fn mismatched_bundle() -> LineageBundle {
    let schema = "weeping-angel/assessment-lineage/v1";
    LineageBundle {
        pack: FrameworkPackSnapshot {
            schema: schema.into(),
            framework: "iso-27001".into(),
            version: "2022".into(),
            digest: "pack-live".into(),
            payload: serde_json::json!({}),
        },
        catalog: CanonicalCatalogSnapshot {
            schema: schema.into(),
            digest: "catalog-live".into(),
            payload: serde_json::json!({}),
        },
        definition: AssessmentDefinitionSnapshot {
            schema: schema.into(),
            assessment_id: AssessmentId::new("assess-unpinned"),
            digest: "def-live".into(),
            definition: Assessment::new(AssessmentId::new("assess-unpinned")),
        },
        applicability: ApplicabilitySnapshot {
            schema: schema.into(),
            assessment_id: AssessmentId::new("assess-unpinned"),
            scope: "repo:in-scope".into(),
            requirement_decisions: Vec::new(),
            control_decisions: Vec::new(),
            pack_entries: Vec::new(),
            digest: "app-live".into(),
        },
        evidence: EvidenceSnapshot {
            schema: schema.into(),
            envelope_digests: Vec::new(),
            collection_run_ids: Vec::new(),
            digest: "ev-live".into(),
        },
        tests: Vec::new(),
        run: AssessmentRun {
            id: AssessmentId::new("assess-unpinned"),
            framework: "iso-27001".into(),
            framework_pack_digest: String::new(),
            assessment_definition_digest: "def-historical".into(),
            started_at: "2026-01-01T00:00:00Z".into(),
            completed_at: "2026-01-01T00:00:01Z".into(),
            scope: "repo:in-scope".into(),
            collector_runs: Vec::new(),
            evidence_snapshot_digest: "ev-historical".into(),
            result_digest: "result-historical".into(),
            status: "completed".into(),
            canonical_catalog_pin: String::new(),
            applicability_snapshot_id: String::new(),
            as_of: String::new(),
        },
        readiness: empty_readiness(),
        soa: StatementOfApplicabilitySnapshot {
            schema: schema.into(),
            digest: "soa-live".into(),
            framework_pack_digest: "pack-live".into(),
            soa: StatementOfApplicability {
                framework: "iso-27001".into(),
                framework_version: "2022".into(),
                entries: Vec::new(),
                disclaimer: "This is a readiness assessment and is not certification.".into(),
            },
        },
        results: Vec::new(),
    }
}

fn ledger_error_kind(err: &LedgerError) -> &'static str {
    match err {
        LedgerError::Sqlite(_) => "sqlite",
        LedgerError::Serialize(_) => "serialize",
        LedgerError::NotFound(_) => "not_found",
        LedgerError::Path(_) => "path",
        LedgerError::Immutable(_) => "immutable",
    }
}

fn assurance_error_kind(err: &AssuranceError) -> &'static str {
    match err {
        AssuranceError::Collector(_) => "collector",
        AssuranceError::Compile(_) => "compile",
        AssuranceError::MissingCollector => "missing_collector",
        AssuranceError::MissingFramework => "missing_framework",
        AssuranceError::UnknownPack(_) => "unknown_pack",
        AssuranceError::DigestMismatch(_) => "digest_mismatch",
        AssuranceError::UnknownControl { .. } => "unknown_control",
    }
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b00_baseline_registered_against_spec() {
    let cargo = read_repo_file("Cargo.toml");
    assert!(
        cargo.contains("name = \"sdd_temporal_lineage_evidence_soa_baseline\""),
        "TLE-B00: register sdd_temporal_lineage_evidence_soa_baseline in root Cargo.toml"
    );
    assert!(
        cargo.contains("path = \"tests/contracts/temporal_lineage_evidence_soa.baseline.rs\""),
        "TLE-B00: baseline path must be tests/contracts/temporal_lineage_evidence_soa.baseline.rs"
    );
    let spec = read_repo_file("docs/specs/temporal-lineage-evidence-soa.md");
    assert!(
        spec.contains("sdd_temporal_lineage_evidence_soa_baseline"),
        "TLE-B00: spec must name this characterization suite"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b01_latest_is_collected_at_desc_without_validity() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let older = seal_at(ts(2026, 1, 1, 12), "older-valid");
    let newer = seal_at(ts(2026, 12, 1, 12), "newer-revoked")
        .with_valid_from(ts(2026, 12, 1, 12))
        .with_valid_until(ts(2026, 12, 2, 0));
    ledger.append(older.clone()).unwrap();
    ledger.append(newer.clone()).unwrap();
    ledger
        .record_validity_event(
            EvidenceValidityEvent::revoked(
                newer.digest(),
                ts(2026, 12, 2, 1),
                Some("revoked".into()),
            )
            .unwrap(),
        )
        .unwrap();

    let latest = ledger.latest(&evidence_type()).unwrap().expect("row");
    assert_eq!(
        latest.digest(),
        newer.digest(),
        "TLE-B01: latest is ORDER BY collected_at DESC; revoked/expired newest row still wins"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b02_latest_as_of_is_validity_filtered_leaf() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let older = seal_at(ts(2026, 1, 1, 12), "older-valid");
    let newer = seal_at(ts(2026, 12, 1, 12), "newer-revoked")
        .with_valid_from(ts(2026, 12, 1, 12))
        .with_valid_until(ts(2026, 12, 2, 0));
    ledger.append(older.clone()).unwrap();
    ledger.append(newer.clone()).unwrap();
    ledger
        .record_validity_event(
            EvidenceValidityEvent::revoked(
                newer.digest(),
                ts(2026, 12, 2, 1),
                Some("revoked".into()),
            )
            .unwrap(),
        )
        .unwrap();

    let as_of = ledger
        .latest_as_of(&evidence_type(), ts(2026, 12, 15, 0))
        .unwrap()
        .expect("validity-filtered leaf");
    assert_eq!(
        as_of.digest(),
        older.digest(),
        "TLE-B02: latest_as_of applies project_validity then leaf selection (as-of evaluation, named latest)"
    );

    let at_expiry = ledger
        .latest_as_of(&evidence_type(), ts(2026, 12, 2, 0))
        .unwrap();
    assert_eq!(
        at_expiry.as_ref().map(|e| e.digest()),
        Some(older.digest()),
        "TLE-B02: T == valid_until is excluded (half-open); latest_as_of falls back to the still-valid older row"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b03_current_valid_at_as_of_public_methods_absent() {
    let ledger = crate_sources_joined("weeping-angel-evidence");
    assert!(
        ledger.contains("pub fn latest("),
        "TLE-B03: latest remains the record-order API"
    );
    assert!(
        ledger.contains("pub fn latest_as_of("),
        "TLE-B03: latest_as_of exists as the validity-filtered leaf"
    );
    assert!(
        ledger.contains("pub fn valid_during("),
        "TLE-B03: valid_during exists; it is not valid_at"
    );
    assert!(
        !ledger.contains("pub fn current("),
        "TLE-B03 found-case: pub fn current( is absent from the evidence crate"
    );
    assert!(
        !ledger.contains("pub fn valid_at("),
        "TLE-B03 found-case: pub fn valid_at( is absent"
    );
    assert!(
        !ledger.contains("pub fn as_of("),
        "TLE-B03 found-case: pub fn as_of( is absent as a ledger method name"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b04_assessment_context_as_of_aliases_now() {
    let now = ts(2026, 3, 1, 8);
    let later = ts(2026, 9, 1, 8);
    let ctx = AssessmentContext {
        now,
        max_age: Duration::from_secs(3600),
    };
    assert_eq!(
        ctx.as_of(),
        now,
        "TLE-B04: AssessmentContext::as_of() returns self.now"
    );
    let ctx2 = FreshnessPolicy {
        max_age: Duration::from_secs(3600),
        as_of: later,
        period: None,
    }
    .into_context();
    assert_eq!(ctx2.now, later);
    assert_eq!(
        ctx2.as_of(),
        later,
        "TLE-B04: FreshnessPolicy.into_context copies as_of into now; the four terms remain aliases"
    );

    let control = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    assert!(
        control.contains("pub fn as_of(&self) -> DateTime<Utc> {\n        self.now"),
        "TLE-B04: as_of body is `self.now`"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b05_assessment_run_json_asof_is_started_at() {
    let run = AssessmentRun {
        id: AssessmentId::new("assess-tle-asof"),
        framework: "iso-27001".into(),
        framework_pack_digest: "pack".into(),
        assessment_definition_digest: "def".into(),
        started_at: "2026-08-01T12:00:00+00:00".into(),
        completed_at: "2026-08-01T12:00:01+00:00".into(),
        scope: "repo:in-scope".into(),
        collector_runs: Vec::new(),
        evidence_snapshot_digest: "ev".into(),
        result_digest: "result".into(),
        status: "completed".into(),
        canonical_catalog_pin: "catalog".into(),
        applicability_snapshot_id: "app".into(),
        as_of: "2025-01-01T00:00:00+00:00".into(),
    };
    let json = serde_json::to_value(&run).expect("serialize AssessmentRun");
    assert_eq!(
        json.get("asOf").and_then(Value::as_str),
        Some("2026-08-01T12:00:00+00:00"),
        "TLE-B05: custom Serialize writes asOf from started_at, ignoring the as_of field; got {json}"
    );
    assert_ne!(
        json.get("asOf").and_then(Value::as_str),
        Some("2025-01-01T00:00:00+00:00"),
        "TLE-B05: independently pinned as_of does not survive JSON"
    );

    let snapshot = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    assert!(
        snapshot.contains("state.serialize_field(\"asOf\", &self.started_at)"),
        "TLE-B05: serialize_field asOf uses started_at"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b06_replay_assessment_is_ok_reconstruct_without_pin_checks() {
    let lineage = read_repo_file("crates/weeping-angel-assurance/src/lineage.rs");
    let replay = fn_replay(&lineage);
    assert!(
        replay.contains("Ok(reconstruct(bundle))"),
        "TLE-B06: replay_assessment is Ok(reconstruct(bundle)); got {replay}"
    );
    assert!(
        !replay.contains("detect_digest_mismatch"),
        "TLE-B06: replay_assessment does not call detect_digest_mismatch"
    );

    let bundle = mismatched_bundle();
    assert!(
        detect_digest_mismatch(&bundle.run.framework_pack_digest, &bundle.pack.digest).is_err()
            || bundle.run.framework_pack_digest != bundle.pack.digest,
        "TLE-B06: fixture pins disagree with snapshot digests"
    );
    let report = replay_assessment(&bundle).expect("TLE-B06 found-case: replay is fail-open");
    assert_eq!(
        report.digest, bundle.run.result_digest,
        "TLE-B06: empty pins / mismatched snapshots still return Ok(reconstruct)"
    );
    let again = reconstruct(&bundle);
    assert_eq!(
        report.digest, again.digest,
        "TLE-B06: reconstruct is a clone helper; replay is equivalent"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b07_assurance_error_has_no_replay_or_pin_variants() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let start = src.find("pub enum AssuranceError").expect("AssuranceError");
    let rest = &src[start..];
    let enum_src = rest
        .split("pub struct AssessmentScope")
        .next()
        .unwrap_or(rest);
    for needle in [
        "MissingPinnedMaterial",
        "IncompleteLineage",
        "InconsistentLineage",
        "CorruptPersistence",
        "IncompatibleSchema",
    ] {
        assert!(
            !enum_src.contains(needle),
            "TLE-B07 found-case: AssuranceError has no {needle} variant"
        );
    }
    let _ = assurance_error_kind(&AssuranceError::MissingCollector);
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b08_assess_maps_collector_err_to_empty_evidence_bag() {
    let facade = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess = fn_assess(&facade);
    assert!(
        assess.contains("Err(_err)") && assess.contains("Vec::new()"),
        "TLE-B08: collector Err binds envelopes = Vec::new(); got {assess}"
    );
    assert!(
        !assess.contains("NoNewObservation")
            && !assess.contains("KnownAbsent")
            && !assess.contains("CollectionFailed")
            && !assess.contains("EvidenceNoLongerValid"),
        "TLE-B08: one-shot assess does not distinguish collection-failure kinds"
    );

    let report = AssuranceEngine::builder()
        .framework(iso_target())
        .collector(FailingCollector)
        .assess(AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .expect("collector Err still returns Ok(report)");
    assert_eq!(
        report.evidence_count, 0,
        "TLE-B08: failed collector evaluates an empty bag"
    );
    let run = report.run.expect("AssessmentRun is sealed");
    assert_eq!(
        run.status, "failed",
        "TLE-B08: status is failed, evidence snapshot is the empty bag"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b09_record_collection_run_is_insert_or_replace() {
    let ledger_src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        ledger_src.contains("INSERT OR REPLACE INTO collection_runs"),
        "TLE-B09: record_collection_run is INSERT OR REPLACE"
    );
    assert!(
        !ledger_src.contains("fn load_collection_run"),
        "TLE-B09: collection runs have no load API; overwrite is silent"
    );

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tle-collection.sqlite");
    let mut ledger = EvidenceLedger::open(&path).unwrap();
    let mut first = CollectionRun::new("fixture.tle", "1");
    first.run_id = "run:tle-replace".into();
    first.status = "completed".into();
    first.evidence_count = 3;
    ledger.record_collection_run(&first).unwrap();

    let mut second = first.clone();
    second.status = "failed".into();
    second.evidence_count = 0;
    second.error_count = 1;
    ledger.record_collection_run(&second).unwrap();

    let conn = rusqlite::Connection::open(&path).unwrap();
    let payload: String = conn
        .query_row(
            "SELECT payload FROM collection_runs WHERE run_id = ?1",
            ["run:tle-replace"],
            |row| row.get(0),
        )
        .unwrap();
    let stored: CollectionRun = serde_json::from_str(&payload).unwrap();
    assert_eq!(
        stored.status, "failed",
        "TLE-B09: a later write of the same run_id silently overwrites a completed payload"
    );
    assert_eq!(stored.evidence_count, 0);
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b10_ledger_error_has_no_corrupt_or_incompatible_schema() {
    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    let start = src.find("pub enum LedgerError").expect("LedgerError");
    let rest = &src[start..];
    let enum_src = rest
        .split("/// Append-only evidence store")
        .next()
        .unwrap_or(rest);
    assert!(
        !enum_src.contains("Corrupt") && !enum_src.contains("IncompatibleSchema"),
        "TLE-B10 found-case: LedgerError has no Corrupt / IncompatibleSchema; got {enum_src}"
    );
    let _ = ledger_error_kind(&LedgerError::NotFound("x".into()));
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b11_malformed_envelope_payload_surfaces_as_serialize() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tle-corrupt.sqlite");
    let mut ledger = EvidenceLedger::open(&path).unwrap();
    let env = seal_at(ts(2026, 4, 1, 0), "corrupt-me");
    let digest = env.digest().to_string();
    ledger.append(env).unwrap();
    drop(ledger);

    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "UPDATE evidence_envelopes SET payload = ?1 WHERE digest = ?2",
        ["{not-json", digest.as_str()],
    )
    .unwrap();
    drop(conn);

    let ledger = EvidenceLedger::open(&path).unwrap();
    let err = ledger.get(&digest).expect_err("malformed payload");
    assert_eq!(
        ledger_error_kind(&err),
        "serialize",
        "TLE-B11: get maps malformed JSON to LedgerError::Serialize, not Corrupt"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b12_persist_immutable_accepts_any_utf8_without_schema_check() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    ledger
        .persist_assessment_run("assess-garbage", "not-json-but-utf8")
        .unwrap();
    let loaded = ledger.load_assessment_run("assess-garbage").unwrap();
    assert_eq!(
        loaded, "not-json-but-utf8",
        "TLE-B12: persist_immutable stores any UTF-8 string; no schema/version check"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b13_cli_soa_empty_or_latest_calls_live_project_soa() {
    let cli = read_repo_file("src/assurance_soa.rs");
    assert!(
        cli.contains("assessment.is_empty() || assessment.eq_ignore_ascii_case(\"latest\")"),
        "TLE-B13: CLI latest/empty is the live path"
    );
    assert!(
        cli.contains("project_soa(\"iso-27001\", \"2022\")"),
        "TLE-B13: empty/latest calls live project_soa(iso-27001, 2022)"
    );

    let soa_src = read_repo_file("crates/weeping-angel-assurance/src/soa.rs");
    assert!(
        soa_src.contains("soa-live:{framework}:{version}"),
        "TLE-B13: project_soa stamps assessment soa-live:{{framework}}:{{version}}"
    );
    assert!(
        soa_src.contains("as_of: Utc::now()"),
        "TLE-B13: live project_soa uses Utc::now() as the clock"
    );

    let live = weeping_angel_assurance::project_soa("iso-27001", "2022");
    assert_eq!(live.framework, "iso-27001");
    assert_eq!(live.framework_version, "2022");
    assert!(
        !live
            .disclaimer
            .to_ascii_lowercase()
            .contains("certification")
            || live.disclaimer.contains("not certification")
            || live.disclaimer.contains("not a certification"),
        "TLE-B13: live SoA still carries the non-certification disclaimer; got {}",
        live.disclaimer
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b14_period_effectiveness_instant_stays_conservative() {
    let observed = seal_at(ts(2026, 6, 1, 12), "period-sample");
    let mut set = EvidenceSet::new();
    set.insert(observed);
    let test = CompiledControlTest::builder()
        .id(ControlTestId::new("test.tle.period.exists"))
        .control_id(ControlId::new("canonical.tle.period"))
        .kind(ControlTestKind::Automated)
        .expr(TestExpr::Exists(EvidenceSelector {
            evidence_type: evidence_type(),
            subject_selector: SubjectSelector {
                kind: None,
                id: Some("repo:in-scope".into()),
            },
            field: None,
            freshness: None,
        }))
        .build();
    let ctx = AssessmentContext {
        now: ts(2026, 6, 2, 12),
        max_age: Duration::from_secs(48 * 3600),
    };
    let outcome = project_period_effectiveness(&test, &set, &ctx);
    assert_eq!(
        outcome,
        PeriodEffectiveness::InsufficientObservationCoverage,
        "TLE-B14: Instant semantics: a single Effective sample is not ContinuouslyEffective"
    );

    let temporal = read_repo_file("crates/weeping-angel-control-test/src/temporal.rs");
    assert!(
        temporal.contains("let semantics = TemporalSemantics::Instant;"),
        "TLE-B14: period projection hard-codes Instant"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b15_envelope_and_validity_append_are_already_idempotent() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let env = seal_at(ts(2026, 5, 1, 0), "idempotent");
    assert!(ledger.append(env.clone()).unwrap());
    assert!(
        !ledger.append(env.clone()).unwrap(),
        "TLE-B15: append is INSERT OR IGNORE by digest"
    );
    let before = ledger.validity_events().unwrap().len();
    let event =
        EvidenceValidityEvent::revoked(env.digest(), ts(2026, 5, 2, 0), Some("duplicate".into()))
            .unwrap();
    assert!(ledger.record_validity_event(event.clone()).unwrap());
    assert!(
        !ledger.record_validity_event(event.clone()).unwrap(),
        "TLE-B15: identical validity-event bytes are a no-op"
    );
    let mut mutated = event.clone();
    mutated.reason = Some("different-bytes".into());
    let err = ledger
        .record_validity_event(mutated)
        .expect_err("same eventId, different bytes");
    assert_eq!(ledger_error_kind(&err), "immutable");
    assert_eq!(ledger.validity_events().unwrap().len(), before + 1);
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b16_collection_failure_kinds_are_absent_from_product() {
    let product = product_crates_joined();
    for needle in ["NoNewObservation", "KnownAbsent", "EvidenceNoLongerValid"] {
        assert!(
            !product.contains(needle),
            "TLE-B16 found-case: {needle} is not a product type"
        );
    }
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b17_scheduler_project_calls_live_project_soa() {
    let scheduler = read_repo_file("crates/weeping-angel-assurance/src/scheduler.rs");
    assert!(
        scheduler.contains("let _soa = project_soa(framework, version)"),
        "TLE-B17: scheduler run_project calls live project_soa"
    );
}

#[ignore = "superseded by sdd_temporal_lineage_evidence_soa_target"]
#[test]
fn tle_b18_latest_and_latest_as_of_disagree_on_future_collected_at() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let past = seal_at(ts(2026, 1, 1, 0), "past");
    let future = seal_at(ts(2027, 1, 1, 0), "future");
    ledger.append(past.clone()).unwrap();
    ledger.append(future.clone()).unwrap();

    let latest = ledger.latest(&evidence_type()).unwrap().unwrap();
    assert_eq!(
        latest.digest(),
        future.digest(),
        "TLE-B18: latest will select a future-dated collected_at"
    );
    let as_of = ledger
        .latest_as_of(&evidence_type(), ts(2026, 6, 1, 0))
        .unwrap()
        .unwrap();
    assert_eq!(
        as_of.digest(),
        past.digest(),
        "TLE-B18: latest_as_of excludes collected_at > t; latest and latest_as_of are not aliases"
    );
}
