//! Target suite for evidence validity and temporal assurance.
//!
//! Encodes DESIRED behavior in `docs/specs/temporal-assurance.md` §4 / §5.2
//! (TMP-001…012). Must stay RED on CURRENT HEAD for the **missing temporal
//! contract** (as-of selection, validity events, period results, no leakage) —
//! not compile/harness noise. Do not implement the feature in this file and
//! do not import types that do not exist on this HEAD.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde::Serialize;
use weeping_angel_assurance::AssessmentRun;
use weeping_angel_assurance::assessment_result_digest;
use weeping_angel_assurance_ir::{AssetId, ControlId, ControlTestId, canonical_digest};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, ControlTestResult, Effectiveness,
    EvidenceSelector, EvidenceSet, EvidenceValue, SubjectSelector, TestExpr, ValueExpr, evaluate,
};
use weeping_angel_evidence::{
    EVIDENCE_SCHEMA, EvidenceArtifactRef, EvidenceEnvelope, EvidenceLedger, EvidenceObservation,
    EvidenceProvenance, EvidenceType, EvidenceValidityEvent,
};

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

fn product_crates_joined() -> String {
    let mut files = Vec::new();
    walk_rs_files(&manifest_dir().join("crates"), &mut files);
    files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/support/mod.rs"
));

/// Mirrors private `DigestBody` in `weeping-angel-evidence`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DigestBody<'a> {
    observation: &'a EvidenceObservation,
    provenance: &'a EvidenceProvenance,
}

fn ts(y: i32, m: u32, d: u32, h: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
}

fn ctx_at(now: chrono::DateTime<Utc>, max_age: Duration) -> AssessmentContext {
    AssessmentContext { now, max_age }
}

fn generous() -> Duration {
    Duration::from_secs(365 * 24 * 3600)
}

fn day() -> Duration {
    Duration::from_secs(24 * 3600)
}

fn seal_at(
    evidence_type: &str,
    asset: &str,
    collected_at: chrono::DateTime<Utc>,
    fact: &str,
    value: &str,
) -> EvidenceEnvelope {
    let obs = EvidenceObservation::new(EvidenceType::new(evidence_type)).with_fact(fact, value);
    EvidenceEnvelope::seal(
        obs,
        EvidenceProvenance {
            collector_id: "fixture.temporal-target".into(),
            collected_at,
            scope: "target".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn observation_selector(asset: &str) -> EvidenceSelector {
    EvidenceSelector {
        evidence_type: EvidenceType::new("evidence.control.observation"),
        subject_selector: SubjectSelector {
            kind: None,
            id: Some(asset.into()),
        },
        field: Some("state".into()),
        freshness: None,
    }
}

fn exists_test(asset: &str) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new("test.temporal.target.exists"))
        .control_id(ControlId::new("canonical.temporal.target"))
        .kind(ControlTestKind::Automated)
        .expr(TestExpr::Exists(EvidenceSelector {
            evidence_type: EvidenceType::new("evidence.control.observation"),
            subject_selector: SubjectSelector {
                kind: None,
                id: Some(asset.into()),
            },
            field: None,
            freshness: None,
        }))
        .build()
}

fn eq_state_test(asset: &str, expected: &str) -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new("test.temporal.target.eq"))
        .control_id(ControlId::new("canonical.temporal.target"))
        .kind(ControlTestKind::Automated)
        .expr(TestExpr::Eq(
            ValueExpr::Field(observation_selector(asset)),
            EvidenceValue::String(expected.into()),
        ))
        .build()
}

fn period_label(result: &ControlTestResult) -> Option<String> {
    let json = serde_json::to_value(result).unwrap();
    if let Some(s) = json.get("periodEffectiveness").and_then(|v| v.as_str()) {
        return Some(s.to_string());
    }
    match json.get("period") {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(obj) => obj
            .get("effectiveness")
            .or_else(|| obj.get("periodEffectiveness"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        None => None,
    }
}

fn eval_exists(
    set: &EvidenceSet,
    asset: &str,
    at: chrono::DateTime<Utc>,
    max_age: Duration,
) -> ControlTestResult {
    evaluate(&exists_test(asset), set, &ctx_at(at, max_age))
}

fn eval_eq(
    set: &EvidenceSet,
    asset: &str,
    expected: &str,
    at: chrono::DateTime<Utc>,
    max_age: Duration,
) -> ControlTestResult {
    evaluate(&eq_state_test(asset, expected), set, &ctx_at(at, max_age))
}

#[test]
fn dual_suite_target_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        harness_src().contains("temporal_assurance.target.rs")
            && harness_src().contains("temporal_assurance.target.rs"),
        "target suite must be wired as a harness module"
    );
    assert!(
        !toml.contains("sdd_temporal_assurance_baseline")
            && !toml.contains("tests/contracts/temporal_assurance.baseline.rs"),
        "superseded baseline must be deleted from Cargo.toml"
    );
    assert!(
        !manifest_dir()
            .join("tests/contracts/temporal_assurance.baseline.rs")
            .exists(),
        "superseded baseline file must be deleted"
    );
}

#[test]
fn temporal_contract_types_exist() {
    let crates = product_crates_joined();
    require_needles(
        "TMP public temporal contract",
        &crates,
        &[
            "struct EvidenceValidityEvent",
            "EVIDENCE_VALIDITY_SCHEMA",
            "evidence-validity/v1",
            "fn record_validity_event",
            "fn select_latest_as_of",
            "struct TemporalQuery",
            "enum PeriodEffectiveness",
            "struct FreshnessPolicy",
            "struct TimeRange",
            "struct TemporalDiff",
            "fn project_timeline",
            "ContinuouslyEffective",
            "IntermittentRegression",
            "InsufficientObservationCoverage",
            "ContinuousUntilSuperseded",
        ],
    );
}

#[test]
fn formal_temporal_fields_exist() {
    let ev = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "TMP formal validity fields",
        &ev,
        &[
            "observed_at",
            "valid_from",
            "valid_until",
            "source_revision",
            "artifact_digest",
        ],
    );
    let ctx = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    let start = ctx
        .find("pub struct AssessmentContext")
        .expect("AssessmentContext");
    let block = &ctx[start..start + 500];
    require_needles(
        "TMP AssessmentContext clock seam",
        block,
        &["as_of", "period", "max_age"],
    );
}

/// TMP-001 overlapping evidence
#[test]
fn tmp_001_overlapping_evidence() {
    require_needles(
        "TMP-001 overlapping evidence",
        &product_crates_joined(),
        &["fn select_latest_as_of", "valid_from", "valid_until"],
    );

    let first = seal_at(
        "evidence.control.observation",
        "asset:overlap",
        ts(2026, 8, 1, 0),
        "state",
        "window-a",
    );
    let second = seal_at(
        "evidence.control.observation",
        "asset:overlap",
        ts(2026, 8, 10, 0),
        "state",
        "window-b",
    );
    let mut set = EvidenceSet::new();
    set.insert(first.clone());
    set.insert(second.clone());

    let mid = eval_exists(&set, "asset:overlap", ts(2026, 8, 5, 0), generous());
    assert_eq!(
        mid.effectiveness,
        Effectiveness::Effective,
        "TMP-001: as-of inside the first window must use the earlier envelope, not a future sibling (no temporal leakage)"
    );
    assert_eq!(
        mid.evidence_refs,
        vec![first.digest().to_string()],
        "TMP-001: evaluation never double-counts overlapping windows; as-of leaf is the earlier envelope"
    );

    let late = eval_exists(&set, "asset:overlap", ts(2026, 8, 12, 0), generous());
    assert_eq!(late.effectiveness, Effectiveness::Effective);
    assert_eq!(
        late.evidence_refs,
        vec![second.digest().to_string()],
        "TMP-001: after both observations, as-of leaf is deterministic (latest observed_at, then collected_at, then digest)"
    );
    assert_eq!(late.evidence_refs.len(), 1);
}

/// TMP-002 supersession
#[test]
fn tmp_002_supersession() {
    require_needles(
        "TMP-002 supersession",
        &product_crates_joined(),
        &["fn select_latest_as_of", "EvidenceValidityEvent"],
    );

    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let older = seal_at(
        "evidence.control.observation",
        "asset:super",
        ts(2026, 8, 1, 0),
        "state",
        "fail",
    );
    let prev_bytes = serde_json::to_vec(&older).unwrap();
    let prev_digest = older.digest().to_string();
    ledger.append(older.clone()).unwrap();
    let newer = seal_at(
        "evidence.control.observation",
        "asset:super",
        ts(2026, 8, 3, 0),
        "state",
        "pass",
    )
    .with_supersedes(&prev_digest);
    let stored = ledger.supersede(&prev_digest, newer.clone()).unwrap();
    assert_eq!(stored.supersedes(), Some(prev_digest.as_str()));
    let reloaded = ledger.get(&prev_digest).unwrap();
    assert_eq!(
        serde_json::to_vec(&reloaded).unwrap(),
        prev_bytes,
        "TMP-002: previous sealed row is unchanged"
    );

    let mut set = EvidenceSet::new();
    set.insert(older);
    set.insert(stored);

    let before = eval_eq(&set, "asset:super", "fail", ts(2026, 8, 2, 0), generous());
    assert_eq!(
        before.effectiveness,
        Effectiveness::Effective,
        "TMP-002: as-of before the replacement still uses the older fail observation"
    );
    assert_eq!(before.evidence_refs, vec![prev_digest.clone()]);

    let after = eval_eq(&set, "asset:super", "pass", ts(2026, 8, 4, 0), generous());
    assert_eq!(
        after.effectiveness,
        Effectiveness::Effective,
        "TMP-002: as-of after the new assertion uses only the supersession leaf"
    );
    assert_eq!(after.evidence_refs, vec![newer.digest().to_string()]);
}

/// TMP-003 revocation
#[test]
fn tmp_003_revocation() {
    let ev = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "TMP-003 revocation",
        &ev,
        &[
            "fn record_validity_event",
            "EvidenceValidityEvent",
            "evidence-validity/v1",
            "revoked",
        ],
    );

    let env = seal_at(
        "evidence.control.observation",
        "asset:revoke",
        ts(2026, 8, 1, 0),
        "state",
        "ok",
    );
    let digest = env.digest().to_string();
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    ledger.append(env.clone()).unwrap();

    let mut set = EvidenceSet::new();
    set.insert(env);
    let before = eval_exists(&set, "asset:revoke", ts(2026, 8, 2, 0), generous());
    assert_eq!(
        before.effectiveness,
        Effectiveness::Effective,
        "TMP-003: as-of T < T_r still uses the envelope"
    );

    let revoked_at = ts(2026, 8, 10, 0);
    let event =
        EvidenceValidityEvent::revoked(&digest, revoked_at, Some("withdrawn".into())).unwrap();
    ledger.record_validity_event(event.clone()).unwrap();
    set.record_validity_event(event);
    let after = eval_exists(&set, "asset:revoke", revoked_at, generous());
    assert_ne!(
        after.effectiveness,
        Effectiveness::Effective,
        "TMP-003: as-of T >= T_r must not use a revoked envelope (missing revoke contract currently leaves it Effective)"
    );
    let still = ledger.get(&digest).unwrap();
    assert_eq!(still.digest(), digest);
}

/// TMP-004 clock boundaries
#[test]
fn tmp_004_clock_boundaries() {
    require_needles(
        "TMP-004 clock boundaries",
        &product_crates_joined(),
        &["valid_from", "valid_until", "fn select_latest_as_of"],
    );

    let env = seal_at(
        "evidence.control.observation",
        "asset:bound",
        ts(2026, 8, 10, 0),
        "state",
        "ok",
    )
    .with_valid_from(ts(2026, 8, 10, 0))
    .with_valid_until(ts(2026, 8, 11, 0));
    let mut set = EvidenceSet::new();
    set.insert(env);

    let at_from = eval_exists(&set, "asset:bound", ts(2026, 8, 10, 0), generous());
    assert_eq!(
        at_from.effectiveness,
        Effectiveness::Effective,
        "TMP-004: T == valid_from (default observed/collected) is inside the half-open window"
    );
    let at_until = eval_exists(&set, "asset:bound", ts(2026, 8, 11, 0), generous());
    assert_ne!(
        at_until.effectiveness,
        Effectiveness::Effective,
        "TMP-004: T == valid_until is outside the half-open window"
    );

    // Exclusive valid_until is the missing contract: collected-at-only windows are inclusive
    // on both ends (ledger.within_window) and never expire by validity.
    let ledger_src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        ledger_src.contains("valid_during")
            || ledger_src.contains("fn valid_during")
            || ledger_src.contains("validity window"),
        "TMP-004: missing validity-window query (half-open valid_from inclusive, valid_until exclusive); within_window is still inclusive collected_at"
    );
}

/// TMP-005 stale evidence
#[test]
fn tmp_005_stale_evidence() {
    require_needles(
        "TMP-005 stale vs expired",
        &product_crates_joined(),
        &["valid_until", "FreshnessPolicy"],
    );

    let env = seal_at(
        "evidence.control.observation",
        "asset:stale",
        ts(2026, 8, 1, 0),
        "state",
        "ok",
    );
    let mut set = EvidenceSet::new();
    set.insert(env);
    let stale = eval_exists(&set, "asset:stale", ts(2026, 8, 3, 0), day());
    assert_eq!(
        stale.effectiveness,
        Effectiveness::StaleEvidence,
        "TMP-005: candidate inside valid_* but T - collected_at > max_age is StaleEvidence, not missing"
    );
    assert_ne!(stale.effectiveness, Effectiveness::InsufficientEvidence);

    let ct = crate_sources_joined("weeping-angel-control-test");
    assert!(
        ct.contains("expired") || ct.contains("Expired") || ct.contains("outside validity"),
        "TMP-005: stale must be disjoint from expired (outside valid_until); current code has no expired state"
    );
}

/// TMP-006 future observation
#[test]
fn tmp_006_future_observation() {
    require_needles(
        "TMP-006 future observation",
        &product_crates_joined(),
        &["fn select_latest_as_of", "observed_at"],
    );

    let past = seal_at(
        "evidence.control.observation",
        "asset:future",
        ts(2026, 8, 1, 0),
        "state",
        "past-ok",
    );
    let future = seal_at(
        "evidence.control.observation",
        "asset:future",
        ts(2026, 8, 30, 0),
        "state",
        "future-ok",
    );
    let as_of = ts(2026, 8, 10, 0);

    let mut only_future = EvidenceSet::new();
    only_future.insert(future.clone());
    let future_only = eval_exists(&only_future, "asset:future", as_of, generous());
    assert_eq!(
        future_only.effectiveness,
        Effectiveness::InsufficientEvidence,
        "TMP-006: envelope with collected_at/observed_at after as_of cannot satisfy a past assessment (must not become Effective or StaleEvidence of the future)"
    );
    assert_ne!(future_only.effectiveness, Effectiveness::Effective);
    assert_ne!(
        future_only.effectiveness,
        Effectiveness::StaleEvidence,
        "TMP-006: future is excluded from candidates; is_stale must not be the future mechanism"
    );

    let mut both = EvidenceSet::new();
    both.insert(past.clone());
    both.insert(future);
    let combined = eval_exists(&both, "asset:future", as_of, generous());
    assert_eq!(
        combined.effectiveness,
        Effectiveness::Effective,
        "TMP-006: future sibling must not shadow an older still-valid envelope"
    );
    assert_eq!(combined.evidence_refs, vec![past.digest().to_string()]);
}

/// TMP-007 intermittent control failure
#[test]
fn tmp_007_intermittent_control_failure() {
    require_needles(
        "TMP-007 intermittent control failure",
        &product_crates_joined(),
        &[
            "enum PeriodEffectiveness",
            "IntermittentRegression",
            "ContinuouslyEffective",
        ],
    );

    let pass = seal_at(
        "evidence.control.observation",
        "asset:flip",
        ts(2026, 8, 1, 0),
        "state",
        "pass",
    );
    let fail = seal_at(
        "evidence.control.observation",
        "asset:flip",
        ts(2026, 8, 10, 0),
        "state",
        "fail",
    )
    .with_supersedes(pass.digest());
    let mut set = EvidenceSet::new();
    set.insert(pass);
    set.insert(fail);

    let point = eval_eq(&set, "asset:flip", "pass", ts(2026, 8, 15, 0), generous());
    assert_ne!(
        period_label(&point).as_deref(),
        Some("continuouslyEffective"),
        "TMP-007: point-in-time Effectiveness must not be labeled continuously effective"
    );
    assert_eq!(
        period_label(&point).as_deref(),
        Some("intermittentRegression"),
        "TMP-007: period containing pass then fail is intermittentRegression, not continuouslyEffective (missing period projection today)"
    );
}

/// TMP-008 sparse observations
#[test]
fn tmp_008_sparse_observations() {
    require_needles(
        "TMP-008 sparse observations",
        &product_crates_joined(),
        &[
            "InsufficientObservationCoverage",
            "ContinuousUntilSuperseded",
            "enum PeriodEffectiveness",
        ],
    );

    let env = seal_at(
        "evidence.control.observation",
        "asset:sparse",
        ts(2026, 8, 15, 12),
        "state",
        "one-shot",
    );
    let mut set = EvidenceSet::new();
    set.insert(env);
    let result = eval_exists(&set, "asset:sparse", ts(2026, 8, 15, 13), day());
    assert_eq!(
        result.effectiveness,
        Effectiveness::Effective,
        "TMP-008: one fresh Exists remains point-in-time Effective"
    );
    assert_eq!(
        period_label(&result).as_deref(),
        Some("insufficientObservationCoverage"),
        "TMP-008: a single instant observation in a wide period is insufficientObservationCoverage unless continuousUntilSuperseded; must not infer continuouslyEffective from one Exists hit"
    );
    assert_ne!(
        period_label(&result).as_deref(),
        Some("continuouslyEffective")
    );
}

/// TMP-009 reproducible historical assessment
#[test]
fn tmp_009_reproducible_historical_assessment() {
    require_needles(
        "TMP-009 reproducible historical assessment",
        &product_crates_joined(),
        &["fn select_latest_as_of", "struct FreshnessPolicy"],
    );

    let run_src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    assert!(
        run_src.contains("as_of")
            || run_src.contains("asOf")
            || run_src.contains("evaluation_clock"),
        "TMP-009: AssessmentRun must pin the evaluation clock; missing as_of allows wall-clock leakage"
    );
    let json = serde_json::to_value(AssessmentRun::default()).unwrap();
    assert!(
        json.get("asOf").is_some()
            || json.get("period").is_some()
            || json.get("evaluationClock").is_some(),
        "TMP-009: pinned AssessmentRun JSON must carry asOf/period; keys={json}"
    );

    let monday = seal_at(
        "evidence.control.observation",
        "asset:hist",
        ts(2026, 8, 10, 0),
        "state",
        "monday",
    );
    let tuesday = seal_at(
        "evidence.control.observation",
        "asset:hist",
        ts(2026, 8, 11, 0),
        "state",
        "tuesday",
    );
    let clock = ts(2026, 8, 10, 12);

    let mut set = EvidenceSet::new();
    set.insert(monday.clone());
    let first = eval_exists(&set, "asset:hist", clock, generous());
    let d1 = assessment_result_digest(std::slice::from_ref(&first));

    set.insert(tuesday);
    let second = eval_exists(&set, "asset:hist", clock, generous());
    let d2 = assessment_result_digest(std::slice::from_ref(&second));
    assert_eq!(
        second.effectiveness,
        Effectiveness::Effective,
        "TMP-009: replay at pinned clock uses only evidence valid then"
    );
    assert_eq!(second.evidence_refs, vec![monday.digest().to_string()]);
    assert_eq!(
        d1, d2,
        "TMP-009: same pins + same as_of ⇒ same result digest; appending a later envelope must not change the historical result"
    );
}

/// TMP-010 expired evidence
#[test]
fn tmp_010_expired_evidence() {
    require_needles(
        "TMP-010 expired evidence",
        &product_crates_joined(),
        &["valid_until", "fn select_latest_as_of"],
    );

    let env = seal_at(
        "evidence.control.observation",
        "asset:exp",
        ts(2026, 1, 1, 0),
        "state",
        "ok",
    )
    .with_valid_until(ts(2026, 2, 1, 0));
    let mut set = EvidenceSet::new();
    set.insert(env);
    // Open-ended default window plus max_age large enough that age is not the issue:
    // without valid_until the envelope still satisfies Exists. Desired: T >= valid_until
    // cannot be Effective.
    let result = eval_exists(&set, "asset:exp", ts(2026, 8, 18, 12), generous());
    assert_ne!(
        result.effectiveness,
        Effectiveness::Effective,
        "TMP-010: expired evidence (valid_until <= as_of) cannot satisfy the control; current HEAD has no validity window so a generous max_age still yields Effective"
    );
    assert_ne!(
        result.effectiveness,
        Effectiveness::StaleEvidence,
        "TMP-010: expiry is not StaleEvidence; stale is policy freshness of a still-valid candidate"
    );
}

/// TMP-011 sealed envelope untouched
#[test]
fn tmp_011_sealed_envelope_untouched() {
    let ev = crate_sources_joined("weeping-angel-evidence");
    require_needles(
        "TMP-011 sealed envelope untouched",
        &ev,
        &[
            "struct EvidenceValidityEvent",
            "fn record_validity_event",
            "EVIDENCE_VALIDITY_SCHEMA",
        ],
    );

    let obs = EvidenceObservation::new(EvidenceType::new("evidence.control.observation"))
        .with_fact("state", "ok");
    let provenance = EvidenceProvenance {
        collector_id: "fixture.temporal-target".into(),
        collected_at: ts(2026, 8, 18, 12),
        scope: "target".into(),
        asset: AssetId::new("asset:seal"),
    };
    let expected = canonical_digest(&DigestBody {
        observation: &obs,
        provenance: &provenance,
    })
    .unwrap();
    let env = EvidenceEnvelope::seal(obs, provenance).unwrap();
    assert_eq!(env.digest(), expected);
    assert_eq!(env.content_digest(), expected);
    assert_eq!(
        serde_json::to_value(&env).unwrap()["schemaVersion"],
        EVIDENCE_SCHEMA
    );

    let mutated = env
        .clone()
        .with_supersedes("deadbeef")
        .with_collection_run("run:mutated")
        .with_artifact_ref(EvidenceArtifactRef {
            artifact_id: "art:1".into(),
            digest: "abc123".into(),
            media_type: "application/json".into(),
            size: 4,
            storage_locator: "mem:1".into(),
            redaction_state: "none".into(),
        });
    assert_eq!(
        mutated.digest(),
        expected,
        "TMP-011: DigestBody stays observation+provenance; supersedes / artifact digest sit outside the sealed digest"
    );

    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        !src.contains("UPDATE envelopes") && !src.contains("set_valid_until"),
        "TMP-011: validity changes are new records/events, never edits to sealed envelopes"
    );
}

/// TMP-012 timeline/diff primitives
#[test]
fn tmp_012_timeline_diff_primitives() {
    require_needles(
        "TMP-012 timeline/diff primitives",
        &product_crates_joined(),
        &[
            "fn project_timeline",
            "struct TemporalDiff",
            "observation_gaps",
            "intermittent_controls",
            "coverage_insufficient",
        ],
    );

    let snap_src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    assert!(
        !snap_src.contains("control_became_effective")
            || product_crates_joined().contains("struct TemporalDiff"),
        "TMP-012: do not reuse pairwise SnapshotDiff control_became_effective as period coverage; add TemporalDiff / project_timeline for readiness and audit library exports"
    );
}

#[test]
fn prompt_13_is_clock_seam_only() {
    require_needles(
        "Prompt 13 FreshnessPolicy seam",
        &product_crates_joined(),
        &["struct FreshnessPolicy"],
    );
    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        lib.contains("pub mod scheduler"),
        "Prompt 13 scheduler lives in weeping-angel-assurance (library runtime)"
    );
    assert!(
        !manifest_dir()
            .join("crates")
            .join("weeping-angel-scheduler")
            .exists(),
        "no dedicated scheduler crate"
    );
}

#[test]
fn period_results_are_distinct_from_point_in_time_effectiveness() {
    require_needles(
        "period projection variants",
        &product_crates_joined(),
        &[
            "ContinuouslyEffective",
            "IntermittentRegression",
            "InsufficientObservationCoverage",
        ],
    );
    let ct = crate_sources_joined("weeping-angel-control-test");
    assert!(
        ct.contains("enum PeriodEffectiveness"),
        "period results must be a distinct PeriodEffectiveness type, not aliases of Effectiveness::Effective"
    );
}
