//! Target suite for temporal evidence, lineage, persistence, and SoA integrity
//! (`docs/specs/temporal-lineage-evidence-soa.md` §4 / §5).
//!
//! Encodes DESIRED behavior on CURRENT HEAD. Must stay RED for the remaining
//! trust-boundary shortcuts (four-term aliases, `asOf` serialize from
//! `startedAt`, fail-open replay, collector-Err empty bag, collection-run
//! replace, untyped persist errors, live `project_soa` as history) — not
//! compile/harness noise. Do not implement the feature in this file and do
//! not import symbols that do not exist on this HEAD.

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
    AssessmentRun, AssessmentScope, AssuranceEngine, reconstruct, replay_assessment,
};
use weeping_angel_assurance_ir::{
    AssessmentId, AssetId, ControlId, ControlTestId, FrameworkVersion,
};
use weeping_angel_collector::{
    CollectorCapabilities, CollectorDescriptor, CollectorError, CollectorScope, EvidenceCollector,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, EvidenceSelector, EvidenceSet,
    PeriodEffectiveness, SubjectSelector, TestExpr, project_period_effectiveness,
};
use weeping_angel_evidence::{
    CollectionRun, EvidenceEnvelope, EvidenceLedger, EvidenceObservation, EvidenceProvenance,
    EvidenceType, EvidenceValidityEvent, LedgerError, project_validity,
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
    walk_rs_files(&manifest_dir().join("src"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn require_needles(label: &str, src: &str, needles: &[&str]) {
    let missing: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| !src.contains(n))
        .collect();
    assert!(
        missing.is_empty(),
        "{label}: missing required surface {missing:?}"
    );
}

fn forbid_needles(label: &str, src: &str, needles: &[&str]) {
    let present: Vec<&str> = needles
        .iter()
        .copied()
        .filter(|n| src.contains(*n))
        .collect();
    assert!(
        present.is_empty(),
        "{label}: forbidden leftover surface {present:?}"
    );
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
            collector_id: "fixture.tle-target".into(),
            collected_at,
            scope: "repo:in-scope".into(),
            asset: AssetId::new("repo:in-scope"),
        },
    )
    .expect("seal target envelope")
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
    rest.split("\npub fn ").next().unwrap_or(rest)
}

fn fn_append(src: &str) -> &str {
    let start = src
        .find("pub fn append(&mut self, envelope: EvidenceEnvelope)")
        .expect("EvidenceLedger::append must exist");
    let rest = &src[start..];
    rest.split("\n    pub fn ").next().unwrap_or(rest)
}

fn fn_record_collection_run(src: &str) -> &str {
    let start = src
        .find("pub fn record_collection_run(")
        .expect("record_collection_run must exist");
    let rest = &src[start..];
    rest.split("\n    pub fn ").next().unwrap_or(rest)
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

fn lineage_schema() -> &'static str {
    "weeping-angel/assessment-lineage/v1"
}

fn mismatched_bundle() -> LineageBundle {
    LineageBundle {
        pack: FrameworkPackSnapshot {
            schema: lineage_schema().into(),
            framework: "iso-27001".into(),
            version: "2022".into(),
            digest: "pack-live".into(),
            payload: serde_json::json!({}),
        },
        catalog: CanonicalCatalogSnapshot {
            schema: lineage_schema().into(),
            digest: "catalog-live".into(),
            payload: serde_json::json!({}),
        },
        definition: AssessmentDefinitionSnapshot {
            schema: lineage_schema().into(),
            assessment_id: AssessmentId::new("assess-unpinned"),
            digest: "def-live".into(),
            definition: Assessment::new(AssessmentId::new("assess-unpinned")),
        },
        applicability: ApplicabilitySnapshot {
            schema: lineage_schema().into(),
            assessment_id: AssessmentId::new("assess-unpinned"),
            scope: "repo:in-scope".into(),
            requirement_decisions: Vec::new(),
            control_decisions: Vec::new(),
            pack_entries: Vec::new(),
            digest: "app-live".into(),
        },
        evidence: EvidenceSnapshot {
            schema: lineage_schema().into(),
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
        soa: weeping_angel_assurance::StatementOfApplicabilitySnapshot {
            schema: lineage_schema().into(),
            digest: "soa-live".into(),
            framework_pack_digest: "pack-live".into(),
            soa: weeping_angel_assurance::StatementOfApplicability {
                framework: "iso-27001".into(),
                framework_version: "2022".into(),
                entries: Vec::new(),
                disclaimer: "This is a readiness assessment and is not certification.".into(),
            },
        },
        results: Vec::new(),
    }
}

fn verified_bundle() -> LineageBundle {
    let mut bundle = mismatched_bundle();
    bundle.run.framework_pack_digest = bundle.pack.digest.clone();
    bundle.run.canonical_catalog_pin = bundle.catalog.digest.clone();
    bundle.run.assessment_definition_digest = bundle.definition.digest.clone();
    bundle.run.evidence_snapshot_digest = bundle.evidence.digest.clone();
    bundle.run.applicability_snapshot_id = bundle.applicability.digest.clone();
    bundle.run.as_of = "2026-01-01T00:00:00Z".into();
    bundle.run.result_digest = weeping_angel_assurance::assessment_result_digest(&bundle.results);
    bundle
}

fn incompatible_schema_bundle() -> LineageBundle {
    let mut bundle = verified_bundle();
    bundle.pack.schema = "weeping-angel/assessment-lineage/v0-incompatible".into();
    bundle
}

fn digest_mismatch_bundle() -> LineageBundle {
    let mut bundle = verified_bundle();
    bundle.run.framework_pack_digest = "pack-historical".into();
    bundle.pack.digest = "pack-after-repo-change".into();
    bundle
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
            detail: "forced collector failure for TLE target".into(),
        })
    }
}

struct EmptyOkCollector;

impl EvidenceCollector for EmptyOkCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        CollectorDescriptor {
            id: "fixture.tle-empty".into(),
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
        Ok(Vec::new())
    }
}

fn expired_then_valid_fixture(ledger: &mut EvidenceLedger) -> (EvidenceEnvelope, EvidenceEnvelope) {
    let older = seal_at(ts(2026, 1, 1, 12), "older-valid");
    let newer = seal_at(ts(2026, 12, 1, 12), "newer-expired")
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
    (older, newer)
}

#[test]
fn tle_000_target_registered_against_spec() {
    let cargo = read_repo_file("Cargo.toml");
    require_needles(
        "TLE-000",
        &cargo,
        &[
            "name = \"sdd_temporal_lineage_evidence_soa_target\"",
            "path = \"tests/contracts/temporal_lineage_evidence_soa.target.rs\"",
        ],
    );
    let spec = read_repo_file("docs/specs/temporal-lineage-evidence-soa.md");
    require_needles(
        "TLE-000",
        &spec,
        &[
            "sdd_temporal_lineage_evidence_soa_target",
            "`latest`, `current`, `valid_at`, and `as_of`",
            "replay_assessment",
        ],
    );
    forbid_needles(
        "TLE-000",
        &cargo,
        &["weeping-angel-catalog", "weeping-angel-assurance-cli"],
    );
}

#[test]
fn tle_001_latest_current_valid_at_as_of_are_distinct_public_apis() {
    let ledger = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "TLE-001",
        &ledger,
        &[
            "pub fn latest(",
            "pub fn current(",
            "pub fn valid_at(",
            "pub fn as_of(",
        ],
    );
    assert!(
        ledger.contains("pub fn latest_as_of("),
        "TLE-001: latest_as_of may remain only as a documented alias of as_of"
    );

    let mut store = EvidenceLedger::open_in_memory().unwrap();
    let (older, newer) = expired_then_valid_fixture(&mut store);
    let latest = store.latest(&evidence_type()).unwrap().expect("row");
    assert_eq!(
        latest.digest(),
        newer.digest(),
        "TLE-001: latest stays record-order and may return an expired/revoked row"
    );
    let desired_current = store
        .latest_as_of(&evidence_type(), ts(2026, 12, 15, 0))
        .unwrap()
        .expect("validity-filtered leaf");
    assert_eq!(
        desired_current.digest(),
        older.digest(),
        "TLE-001: current/as_of at live-now after expiry must not equal latest"
    );
    assert_ne!(
        latest.digest(),
        desired_current.digest(),
        "TLE-001: latest and current disagree when the newest row is expired or revoked"
    );
}

#[test]
fn tle_002_as_of_never_leaks_future_or_invalid_evidence() {
    let ledger_src = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "TLE-002",
        &ledger_src,
        &["pub fn as_of(", "pub fn valid_at("],
    );

    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let past = seal_at(ts(2026, 1, 1, 0), "past");
    let future = seal_at(ts(2027, 1, 1, 0), "future");
    let expired = seal_at(ts(2026, 3, 1, 0), "expired")
        .with_valid_from(ts(2026, 3, 1, 0))
        .with_valid_until(ts(2026, 4, 1, 0));
    ledger.append(past.clone()).unwrap();
    ledger.append(future.clone()).unwrap();
    ledger.append(expired.clone()).unwrap();
    ledger
        .record_validity_event(
            EvidenceValidityEvent::revoked(
                expired.digest(),
                ts(2026, 3, 15, 0),
                Some("rev".into()),
            )
            .unwrap(),
        )
        .unwrap();

    let t = ts(2026, 6, 1, 0);
    let events = ledger.validity_events().unwrap();
    let members: Vec<String> = ledger
        .for_type(&evidence_type())
        .unwrap()
        .into_iter()
        .filter(|env| project_validity(env, &events, t).is_some())
        .map(|env| env.digest().to_string())
        .collect();
    assert!(
        members.iter().any(|d| d == past.digest()),
        "TLE-002: past envelope remains a valid-at member at t"
    );
    assert!(
        !members.iter().any(|d| d == future.digest()),
        "TLE-002: collected_at > t must never be a valid-at / as-of candidate"
    );
    assert!(
        !members.iter().any(|d| d == expired.digest()),
        "TLE-002: expired-before or revoked-before t must never leak"
    );
    let leaf = ledger
        .latest_as_of(&evidence_type(), t)
        .unwrap()
        .expect("as-of leaf");
    assert_eq!(leaf.digest(), past.digest());
    assert_ne!(
        leaf.digest(),
        ledger.latest(&evidence_type()).unwrap().unwrap().digest(),
        "TLE-002: as-of leaf is not latest-record when latest is in the future"
    );
}

#[test]
fn tle_003_expiry_at_exact_instant_is_not_valid() {
    let ledger_src = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "TLE-003",
        &ledger_src,
        &["pub fn as_of(", "pub fn valid_at("],
    );

    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let older = seal_at(ts(2026, 1, 1, 12), "still-valid");
    let windowed = seal_at(ts(2026, 12, 1, 12), "expires-at-t")
        .with_valid_from(ts(2026, 12, 1, 12))
        .with_valid_until(ts(2026, 12, 2, 0));
    ledger.append(older.clone()).unwrap();
    ledger.append(windowed.clone()).unwrap();

    let at_until = ts(2026, 12, 2, 0);
    let events = ledger.validity_events().unwrap();
    assert!(
        project_validity(&windowed, &events, at_until).is_none(),
        "TLE-003: half-open window: T == valid_until is not valid"
    );
    let leaf = ledger.latest_as_of(&evidence_type(), at_until).unwrap();
    assert_eq!(
        leaf.as_ref().map(|e| e.digest()),
        Some(older.digest()),
        "TLE-003: as-of at the expiry instant selects the still-valid older row"
    );
}

#[test]
fn tle_004_validity_history_is_append_only() {
    let ledger_src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    forbid_needles(
        "TLE-004",
        &ledger_src,
        &["UPDATE evidence_envelopes SET payload"],
    );
    assert!(
        fn_append(&ledger_src).contains("INSERT OR IGNORE"),
        "TLE-004: envelope append stays INSERT OR IGNORE by digest"
    );
    require_needles("TLE-004", fn_append(&ledger_src), &["transaction", "BEGIN"]);

    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let env = seal_at(ts(2026, 5, 1, 0), "append-only");
    let original = serde_json::to_string(&env).unwrap();
    assert!(ledger.append(env.clone()).unwrap());
    assert!(
        !ledger.append(env.clone()).unwrap(),
        "TLE-004: identical envelope digest is idempotent"
    );
    let stored = ledger.get(env.digest()).unwrap();
    assert_eq!(
        serde_json::to_string(&stored).unwrap(),
        original,
        "TLE-004: sealed envelope payload is never rewritten"
    );

    let before = ledger.validity_events().unwrap().len();
    let revoked =
        EvidenceValidityEvent::revoked(env.digest(), ts(2026, 5, 2, 0), Some("revoked".into()))
            .unwrap();
    assert!(ledger.record_validity_event(revoked.clone()).unwrap());
    let superseded = EvidenceValidityEvent::superseded(env.digest(), ts(2026, 5, 3, 0)).unwrap();
    assert!(ledger.record_validity_event(superseded).unwrap());
    assert_eq!(
        ledger.validity_events().unwrap().len(),
        before + 2,
        "TLE-004: revoke/supersede append events; they do not mutate the envelope"
    );
    assert_eq!(
        serde_json::to_string(&ledger.get(env.digest()).unwrap()).unwrap(),
        original
    );
}

#[test]
fn tle_005_assessment_run_json_asof_is_the_as_of_field() {
    let snapshot = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    require_needles(
        "TLE-005",
        &snapshot,
        &["state.serialize_field(\"asOf\", &self.as_of)"],
    );
    forbid_needles(
        "TLE-005",
        &snapshot,
        &["state.serialize_field(\"asOf\", &self.started_at)"],
    );

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
        Some("2025-01-01T00:00:00+00:00"),
        "TLE-005: JSON asOf must serialize the as_of field; got {json}"
    );
    assert_eq!(
        json.get("startedAt").and_then(Value::as_str),
        Some("2026-08-01T12:00:00+00:00"),
        "TLE-005: startedAt remains the wall-clock start and can differ from asOf"
    );
}

#[test]
fn tle_006_replay_missing_pins_is_typed_err() {
    let lineage = read_repo_file("crates/weeping-angel-assurance/src/lineage.rs");
    let replay = fn_replay(&lineage);
    forbid_needles("TLE-006", replay, &["Ok(reconstruct(bundle))"]);
    require_needles(
        "TLE-006",
        &read_repo_file("crates/weeping-angel-assurance/src/lib.rs"),
        &[
            "MissingPinnedMaterial",
            "IncompleteLineage",
            "InconsistentLineage",
            "CorruptPersistence",
            "IncompatibleSchema",
        ],
    );

    let bundle = mismatched_bundle();
    assert!(
        bundle.run.framework_pack_digest.is_empty()
            && bundle.run.as_of.is_empty()
            && bundle.run.canonical_catalog_pin.is_empty(),
        "TLE-006: fixture has empty required pins"
    );
    replay_assessment(&bundle).expect_err(
        "TLE-006: missing pins / unpinned asOf must fail closed; never Ok(reconstruct)",
    );
}

#[test]
fn tle_007_replay_digest_mismatch_is_typed_err_and_does_not_load_current_files() {
    let lineage = read_repo_file("crates/weeping-angel-assurance/src/lineage.rs");
    let replay = fn_replay(&lineage);
    require_needles("TLE-007", replay, &["detect_digest_mismatch"]);
    forbid_needles(
        "TLE-007",
        replay,
        &[
            "load_framework_pack",
            "CanonicalCatalog::",
            "project_soa(",
            "std::fs::read",
        ],
    );

    let bundle = digest_mismatch_bundle();
    assert!(
        detect_digest_mismatch(&bundle.run.framework_pack_digest, &bundle.pack.digest).is_err(),
        "TLE-007: run pack pin disagrees with snapshot digest (replay after pack/catalog change)"
    );
    replay_assessment(&bundle)
        .expect_err("TLE-007: digest mismatch must be typed Err, not Ok plus current files");
}

#[test]
fn tle_008_replay_incomplete_inconsistent_or_incompatible_schema_fail_closed() {
    let product = product_crates_joined();
    require_needles(
        "TLE-008",
        &product,
        &[
            "IncompleteLineage",
            "InconsistentLineage",
            "IncompatibleSchema",
        ],
    );

    let mut incomplete = verified_bundle();
    incomplete.evidence.envelope_digests = vec!["ghost-envelope".into()];
    replay_assessment(&incomplete).expect_err(
        "TLE-008: envelope list vs snapshot identity contradiction is InconsistentLineage",
    );

    replay_assessment(&incompatible_schema_bundle())
        .expect_err("TLE-008: incompatible snapshot schema must fail closed");

    let twice = verified_bundle();
    let first = replay_assessment(&twice);
    let second = replay_assessment(&twice);
    match (first, second) {
        (Ok(a), Ok(b)) => assert_eq!(
            a.digest, b.digest,
            "TLE-008: replaying a verified bundle twice is byte-stable"
        ),
        (Err(_), _) | (_, Err(_)) => {
            // Fail-closed on a verified bundle is not the desired outcome; pins match.
            let reconstructed = reconstruct(&twice);
            assert_eq!(
                reconstructed.digest, twice.run.result_digest,
                "TLE-008: verified pins must be replayable to the same result identity"
            );
            panic!("TLE-008: verified bundle must replay Ok twice with identical result identity");
        }
    }
}

#[test]
fn tle_009_collection_failure_does_not_erase_or_imply_empty_world() {
    let facade = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess = fn_assess(&facade);
    forbid_needles("TLE-009", assess, &["Vec::new()"]);
    let product = product_crates_joined();
    require_needles(
        "TLE-009",
        &product,
        &[
            "NoNewObservation",
            "KnownAbsent",
            "CollectionFailed",
            "EvidenceNoLongerValid",
        ],
    );

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tle-collect.sqlite");
    let mut ledger = EvidenceLedger::open(&path).unwrap();
    let prior = seal_at(ts(2026, 2, 1, 0), "prior-valid");
    ledger.append(prior.clone()).unwrap();
    let before = ledger.query().unwrap().len();

    let mut failed = CollectionRun::new("fixture.tle-fail", "1");
    failed.run_id = "run:tle-collect-fail".into();
    failed.status = "failed".into();
    failed.error_count = 1;
    ledger.record_collection_run(&failed).unwrap();
    assert_eq!(
        ledger.query().unwrap().len(),
        before,
        "TLE-009: recording a failed collection run must not delete ledger envelopes"
    );
    assert_eq!(ledger.get(prior.digest()).unwrap().digest(), prior.digest());

    let report = AssuranceEngine::builder()
        .framework(iso_target())
        .collector(FailingCollector)
        .assess(AssessmentScope::new().allow_asset(AssetId::new("repo:in-scope")))
        .expect("collector Err is a CollectionFailed outcome, not a panic");
    assert_ne!(
        report.evidence_count, 0,
        "TLE-009: collector Err must not evaluate an implicit empty world when prior valid evidence exists; got evidence_count={}",
        report.evidence_count
    );
    let _empty_ok = EmptyOkCollector;
}

#[test]
fn tle_010_record_collection_run_is_idempotent_and_immutable_when_completed() {
    let ledger_src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    forbid_needles(
        "TLE-010",
        fn_record_collection_run(&ledger_src),
        &["INSERT OR REPLACE INTO collection_runs"],
    );

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tle-collection.sqlite");
    let mut ledger = EvidenceLedger::open(&path).unwrap();
    let mut first = CollectionRun::new("fixture.tle", "1");
    first.run_id = "run:tle-immutable".into();
    first.status = "completed".into();
    first.evidence_count = 3;
    ledger.record_collection_run(&first).unwrap();
    ledger
        .record_collection_run(&first)
        .expect("TLE-010: identical completed payload is idempotent");

    let mut second = first.clone();
    second.status = "failed".into();
    second.evidence_count = 0;
    second.error_count = 1;
    let err = ledger
        .record_collection_run(&second)
        .expect_err("TLE-010: a different completed payload for the same run_id is Immutable");
    assert_eq!(
        ledger_error_kind(&err),
        "immutable",
        "TLE-010: expected LedgerError::Immutable, got {err}"
    );

    let conn = rusqlite::Connection::open(&path).unwrap();
    let payload: String = conn
        .query_row(
            "SELECT payload FROM collection_runs WHERE run_id = ?1",
            ["run:tle-immutable"],
            |row| row.get(0),
        )
        .unwrap();
    let stored: CollectionRun = serde_json::from_str(&payload).unwrap();
    assert_eq!(
        stored.status, "completed",
        "TLE-010: completed collection-run payload must not be replaced"
    );
    assert_eq!(stored.evidence_count, 3);
}

#[test]
fn tle_011_corrupt_and_incompatible_schema_are_typed_fail_closed_errors() {
    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    require_needles("TLE-011", &src, &["Corrupt", "IncompatibleSchema"]);

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
    assert_ne!(
        ledger_error_kind(&err),
        "serialize",
        "TLE-011: malformed payload must be LedgerError::Corrupt, not Serialize; got {err}"
    );

    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    ledger
        .persist_assessment_run("assess-garbage", "not-json-but-utf8")
        .expect_err(
            "TLE-011: persist of incompatible/non-schema payload must fail closed (IncompatibleSchema)",
        );
}

#[test]
fn tle_012_historical_soa_does_not_call_live_project_soa() {
    let cli = read_repo_file("src/assurance_soa.rs");
    let latest_arm = {
        let start = cli
            .find("assessment.is_empty() || assessment.eq_ignore_ascii_case(\"latest\")")
            .expect("TLE-012: CLI still has a latest/empty branch");
        &cli[start..]
    };
    forbid_needles(
        "TLE-012",
        latest_arm,
        &["project_soa(\"iso-27001\", \"2022\")"],
    );
    require_needles(
        "TLE-012",
        latest_arm,
        &["project_soa_from_snapshot", "replay_assessment"],
    );

    let soa_src = read_repo_file("crates/weeping-angel-assurance/src/soa.rs");
    assert!(
        soa_src.contains("soa-live:{framework}:{version}"),
        "TLE-012: live project_soa may remain as a current-pack convenience"
    );
    forbid_needles("TLE-012", latest_arm, &["as_of: Utc::now()"]);

    let live = weeping_angel_assurance::project_soa("iso-27001", "2022");
    assert!(
        live.disclaimer
            .to_ascii_lowercase()
            .contains("not certification")
            || live
                .disclaimer
                .to_ascii_lowercase()
                .contains("not a certification"),
        "TLE-012: SoA must never infer certification from readiness; got {}",
        live.disclaimer
    );
}

#[test]
fn tle_013_period_effectiveness_stays_conservative_and_clock_is_not_now_alias() {
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
        "TLE-013: Instant semantics: a single Effective sample is not ContinuouslyEffective"
    );
    assert_ne!(
        outcome,
        PeriodEffectiveness::ContinuouslyEffective,
        "TLE-013: do not promote Instant coverage to ContinuouslyEffective"
    );

    let control = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    forbid_needles(
        "TLE-013",
        &control,
        &["pub fn as_of(&self) -> DateTime<Utc> {\n        self.now"],
    );
    let ledger = crate_sources_joined("weeping-angel-evidence");
    require_needles("TLE-013", &ledger, &["pub fn current(", "pub fn as_of("]);
}

#[test]
fn tle_014_duplicate_validity_events_stay_idempotent_or_immutable() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let env = seal_at(ts(2026, 5, 1, 0), "dup-event");
    ledger.append(env.clone()).unwrap();
    let event =
        EvidenceValidityEvent::revoked(env.digest(), ts(2026, 5, 2, 0), Some("duplicate".into()))
            .unwrap();
    assert!(ledger.record_validity_event(event.clone()).unwrap());
    assert!(
        !ledger.record_validity_event(event.clone()).unwrap(),
        "TLE-014: identical validity-event bytes are a no-op"
    );
    let mut mutated = event.clone();
    mutated.reason = Some("different-bytes".into());
    let err = ledger
        .record_validity_event(mutated)
        .expect_err("same eventId, different bytes");
    assert_eq!(ledger_error_kind(&err), "immutable");

    let product = product_crates_joined();
    require_needles(
        "TLE-014",
        &product,
        &["NoNewObservation", "KnownAbsent", "EvidenceNoLongerValid"],
    );
}

#[test]
fn tle_015_select_latest_as_of_stays_and_replay_does_not_substitute_current_state() {
    let temporal = read_repo_file("crates/weeping-angel-control-test/src/temporal.rs");
    require_needles("TLE-015", &temporal, &["pub fn select_latest_as_of"]);
    let evidence = crate_sources_joined("weeping-angel-evidence");
    assert!(
        !evidence.contains("pub fn select_latest_as_of"),
        "TLE-015: do not move select_latest_as_of into weeping-angel-evidence"
    );

    let members = read_repo_file("Cargo.toml");
    forbid_needles(
        "TLE-015",
        &members,
        &[
            "crates/weeping-angel-catalog",
            "crates/weeping-angel-assurance-cli",
        ],
    );

    let lineage = read_repo_file("crates/weeping-angel-assurance/src/lineage.rs");
    let replay = fn_replay(&lineage);
    forbid_needles("TLE-015", replay, &["Ok(reconstruct(bundle))"]);
    let changed = digest_mismatch_bundle();
    replay_assessment(&changed).expect_err(
        "TLE-015: after pack/catalog file change, replay must not fill gaps from current files",
    );
}
