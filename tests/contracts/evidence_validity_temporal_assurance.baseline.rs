//! SUPERSEDED by `sdd_temporal_assurance_target`.
//!
//! Historical characterization of SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`
//! (`docs/specs/evidence-validity-temporal-assurance.md` §3 / §5.1).
//! Target suite `sdd_temporal_assurance_target` is the SSOT. Characterization
//! tests are `#[ignore]` so CI does not require retired collected-at-only
//! behavior. Distinct from the sibling dual-suite `sdd_temporal_assurance_*`.
//!
//! Today: `EvidenceEnvelope` is immutable after seal; `DigestBody` hashes
//! observation+provenance only; provenance has `collected_at` and no
//! `observed_at`; envelope has `supersedes` plus optional artifact digest;
//! no `valid_from`/`valid_until`, revocation event, or `source_revision`.
//! Ledger is append-only SQLite (`INSERT OR IGNORE` by digest); supersede
//! appends a new row; `within_window` and `latest` filter/order
//! `collected_at` only. `AssessmentContext` is `{ now, max_age }`;
//! `is_stale` / `envelope_stale` use collected_at age (future collected_at
//! → `to_std` fail → stale). Population `select_latest` is supersedes-leaf
//! then latest collected_at+digest with no as-of filter; `first_selector`
//! is digest-order first hit. `Exists` can be `Effective` from one
//! non-stale envelope. `Effectiveness` has StaleEvidence / InsufficientEvidence
//! / ManualReviewRequired but no period outcomes. `SnapshotDiff`/`compare`
//! is pairwise readiness, not validity timelines. Prompt 13 scheduler is
//! not in product; `assess` stamps `Utc::now()` and 24h `max_age`. Public
//! assurance-runtime envelope list omits observedAt/validFrom/validUntil.
//!
//! Skip-superseded. Does not implement temporal validity.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde::Serialize;
use serde_json::Value;
use weeping_angel_assurance::readiness::ControlReadiness;
use weeping_angel_assurance::{
    AssessmentRun, FrameworkReadinessSnapshot, compare, compare_lineage, compare_runs,
};
use weeping_angel_assurance_ir::{
    AssessmentId, AssetId, ControlId, ControlTestId, RequirementId, canonical_digest,
};
use weeping_angel_control_test::{
    AssessmentContext, CompiledControlTest, ControlTestKind, Effectiveness, EvidenceIndex,
    EvidenceSelector, EvidenceSet, SubjectSelector, TestExpr, evaluate,
};
use weeping_angel_evidence::{
    EVIDENCE_SCHEMA, EvidenceArtifactRef, EvidenceEnvelope, EvidenceLedger, EvidenceObservation,
    EvidenceProvenance, EvidenceType,
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
            collector_id: "fixture.evt-baseline".into(),
            collected_at,
            scope: "evt-baseline".into(),
            asset: AssetId::new(asset),
        },
    )
    .unwrap()
}

fn exists_test() -> CompiledControlTest {
    CompiledControlTest::builder()
        .id(ControlTestId::new("test.evt.baseline.exists"))
        .control_id(ControlId::new("canonical.evt.baseline"))
        .kind(ControlTestKind::Automated)
        .expr(TestExpr::Exists(EvidenceSelector {
            evidence_type: EvidenceType::new("evidence.control.observation"),
            subject_selector: SubjectSelector::default(),
            field: None,
            freshness: None,
        }))
        .build()
}

fn object_keys(value: &Value) -> BTreeSet<String> {
    match value {
        Value::Object(map) => map.keys().cloned().collect(),
        _ => BTreeSet::new(),
    }
}

fn empty_readiness() -> FrameworkReadinessSnapshot {
    FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new("assess-evt-baseline"),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: "pack".into(),
        assessment_digest: "def".into(),
        evaluated_at: "2026-08-01T00:00:00Z".into(),
        requirements: Vec::new(),
        controls: Vec::new(),
        effective: 0,
        ineffective: 0,
        partial: 0,
        manual_review: 0,
        insufficient_evidence: 0,
        not_applicable: 0,
        automation_coverage: "0".into(),
        evidence_coverage: "0".into(),
    }
}

#[test]
#[ignore = "superseded by target suite"]
fn dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("sdd_evidence_validity_temporal_assurance_baseline")
            && toml.contains("tests/contracts/evidence_validity_temporal_assurance.baseline.rs")
            && toml.contains("sdd_evidence_validity_temporal_assurance_target")
            && toml.contains("tests/contracts/evidence_validity_temporal_assurance.target.rs"),
        "this run's dual-suite must be listed in root Cargo.toml"
    );
    assert!(
        toml.contains("sdd_temporal_assurance_baseline")
            && toml.contains("tests/contracts/temporal_assurance.baseline.rs"),
        "must not clobber the sibling temporal-assurance dual-suite registration"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn envelope_clock_is_collected_at_only() {
    let env = seal_at(
        "evidence.control.observation",
        "asset:clock",
        ts(2026, 8, 18, 12),
        "state",
        "ok",
    );
    assert_eq!(env.provenance().collected_at, ts(2026, 8, 18, 12));

    let json = serde_json::to_value(&env).unwrap();
    let root = object_keys(&json);
    for forbidden in [
        "observedAt",
        "observed_at",
        "validFrom",
        "valid_from",
        "validUntil",
        "valid_until",
        "sourceRevision",
        "source_revision",
        "revokedAt",
        "revoked_at",
        "invalidatedAt",
        "invalidated_at",
    ] {
        assert!(
            !root.contains(forbidden),
            "envelope JSON currently has no `{forbidden}` field; keys={root:?}"
        );
    }
    assert!(
        root.contains("provenance") && root.contains("supersedes") && root.contains("digest"),
        "envelope still exposes provenance/supersedes/digest; keys={root:?}"
    );

    let prov = object_keys(json.get("provenance").unwrap());
    assert!(
        prov.contains("collectedAt") && prov.contains("collectorId"),
        "provenance clock today is collectedAt; keys={prov:?}"
    );
    for forbidden in ["observedAt", "validFrom", "validUntil", "sourceRevision"] {
        assert!(
            !prov.contains(forbidden),
            "provenance JSON currently has no `{forbidden}`; keys={prov:?}"
        );
    }

    let ev_src = crate_sources_joined("weeping-angel-evidence");
    assert!(
        ev_src.contains("pub collected_at: DateTime<Utc>"),
        "EvidenceProvenance still owns collected_at"
    );
    assert!(
        ev_src.contains("struct DigestBody")
            && ev_src.contains("observation: &'a EvidenceObservation")
            && ev_src.contains("provenance: &'a EvidenceProvenance"),
        "DigestBody remains observation + provenance"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn public_assurance_runtime_envelope_list_omits_iso_phase7_clocks() {
    let contract = read_repo_file("docs/specs/assurance-runtime.md");
    let start = contract
        .find("Envelope (`evidence/v1`):")
        .expect("assurance-runtime Evidence envelope list");
    let block = &contract[start..start + 420];
    assert!(
        block.contains("evidenceId, schemaVersion, observation, provenance, digest,")
            && block.contains(
                "artifactRef?, collectionRunId, contentDigest, sensitivity, scope, supersedes?"
            ),
        "public envelope list is the live (already larger) shape: {block}"
    );
    for needle in ["observedAt", "validFrom", "validUntil"] {
        assert!(
            !block.contains(needle),
            "public envelope list currently omits ISO Phase 7 additive `{needle}`"
        );
    }
    assert!(
        block.contains("collectorId, collectedAt, scope, asset"),
        "provenance list still names collectedAt"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn digest_body_is_observation_plus_provenance() {
    let obs = EvidenceObservation::new(EvidenceType::new("evidence.control.observation"))
        .with_fact("state", "ok")
        .with_narrative("observed");
    let provenance = EvidenceProvenance {
        collector_id: "fixture.evt-baseline".into(),
        collected_at: ts(2026, 8, 18, 12),
        scope: "evt-baseline".into(),
        asset: AssetId::new("asset:digest"),
    };
    let expected = canonical_digest(&DigestBody {
        observation: &obs,
        provenance: &provenance,
    })
    .unwrap();
    let env = EvidenceEnvelope::seal(obs, provenance).unwrap();
    assert_eq!(env.digest(), expected);
    assert_eq!(env.content_digest(), expected);
    assert_eq!(env.evidence_id(), format!("ev:sha256:{expected}"));
    assert_eq!(
        env.observation().evidence_type().as_str(),
        "evidence.control.observation"
    );
    let json = serde_json::to_value(&env).unwrap();
    assert_eq!(json["schemaVersion"], EVIDENCE_SCHEMA);
}

#[test]
#[ignore = "superseded by target suite"]
fn supersedes_run_id_and_artifact_ref_sit_outside_digest() {
    let env = seal_at(
        "evidence.control.observation",
        "asset:outside",
        ts(2026, 8, 18, 12),
        "state",
        "ok",
    );
    let digest = env.digest().to_string();
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
        digest,
        "changing supersedes / collectionRunId / artifactRef must not rewrite digest"
    );
    assert_eq!(mutated.content_digest(), digest);
    assert_eq!(mutated.supersedes(), Some("deadbeef"));
    assert_eq!(mutated.collection_run_id(), "run:mutated");
    let json = serde_json::to_value(&mutated).unwrap();
    assert_eq!(json["artifactRef"]["digest"], "abc123");
}

#[test]
#[ignore = "superseded by target suite"]
fn ledger_append_is_insert_or_ignore_by_digest() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let env = seal_at(
        "evidence.control.observation",
        "asset:idemp",
        ts(2026, 8, 18, 12),
        "state",
        "ok",
    );
    assert!(ledger.append(env.clone()).unwrap());
    assert!(
        !ledger.append(env.clone()).unwrap(),
        "second append of the same digest is INSERT OR IGNORE (false), not an UPDATE"
    );
    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        src.contains("INSERT OR IGNORE INTO evidence_envelopes"),
        "append stays idempotent by digest"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn ledger_within_window_filters_collected_at_inclusive() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let early = seal_at(
        "evidence.control.observation",
        "asset:w",
        ts(2026, 8, 1, 0),
        "state",
        "early",
    );
    let start = seal_at(
        "evidence.control.observation",
        "asset:w",
        ts(2026, 8, 10, 0),
        "state",
        "start",
    );
    let mid = seal_at(
        "evidence.control.observation",
        "asset:w",
        ts(2026, 8, 15, 0),
        "state",
        "mid",
    );
    let end = seal_at(
        "evidence.control.observation",
        "asset:w",
        ts(2026, 8, 20, 0),
        "state",
        "end",
    );
    let late = seal_at(
        "evidence.control.observation",
        "asset:w",
        ts(2026, 8, 25, 0),
        "state",
        "late",
    );
    for env in [&early, &start, &mid, &end, &late] {
        assert!(ledger.append(env.clone()).unwrap());
    }

    let window = ledger
        .within_window(ts(2026, 8, 10, 0), ts(2026, 8, 20, 0))
        .unwrap();
    let states: Vec<&str> = window
        .iter()
        .map(|e| e.observation().fact("state").unwrap())
        .collect();
    assert_eq!(
        states,
        vec!["start", "mid", "end"],
        "within_window is inclusive on collected_at and ignores validity windows (none exist)"
    );

    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        src.contains("WHERE collected_at >= ?1 AND collected_at <= ?2"),
        "within_window SQL filters collected_at inclusively"
    );
    assert!(
        !src.contains("valid_from")
            && !src.contains("valid_until")
            && !src.contains("valid_during")
            && !src.contains("evidence_validity"),
        "ledger has no validity-event table or validity-window query today"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn ledger_latest_is_collected_at_desc_without_supersede_walk() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let older_leaf = seal_at(
        "evidence.control.observation",
        "asset:latest",
        ts(2026, 8, 10, 0),
        "state",
        "leaf-older",
    );
    let newer_superseded = seal_at(
        "evidence.control.observation",
        "asset:latest",
        ts(2026, 8, 20, 0),
        "state",
        "superseded-newer",
    );
    let newer_digest = newer_superseded.digest().to_string();
    ledger.append(older_leaf.clone()).unwrap();
    ledger.append(newer_superseded.clone()).unwrap();
    let earlier_leaf_that_supersedes = seal_at(
        "evidence.control.observation",
        "asset:latest",
        ts(2026, 8, 5, 0),
        "state",
        "leaf-earliest",
    )
    .with_supersedes(&newer_digest);
    ledger.append(earlier_leaf_that_supersedes.clone()).unwrap();

    let latest = ledger
        .latest(&EvidenceType::new("evidence.control.observation"))
        .unwrap()
        .expect("row");
    assert_eq!(
        latest.observation().fact("state"),
        Some("superseded-newer"),
        "ledger.latest is ORDER BY collected_at DESC and can return a superseded envelope"
    );
    assert_eq!(latest.digest(), newer_digest);

    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        src.contains("ORDER BY collected_at DESC LIMIT 1"),
        "latest SQL is collected_at DESC LIMIT 1"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn ledger_supersede_keeps_previous_payload_bytes() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let previous = seal_at(
        "evidence.control.observation",
        "asset:hist",
        ts(2026, 8, 1, 0),
        "state",
        "fail",
    );
    let next = seal_at(
        "evidence.control.observation",
        "asset:hist",
        ts(2026, 8, 2, 0),
        "state",
        "pass",
    );
    let prev_digest = previous.digest().to_string();
    let prev_bytes = serde_json::to_vec(&previous).unwrap();
    ledger.append(previous).unwrap();
    let stored = ledger.supersede(&prev_digest, next).unwrap();
    assert_eq!(stored.supersedes(), Some(prev_digest.as_str()));
    let reloaded = ledger.get(&prev_digest).unwrap();
    assert_eq!(serde_json::to_vec(&reloaded).unwrap(), prev_bytes);
    let src = read_repo_file("crates/weeping-angel-evidence/src/ledger.rs");
    assert!(
        !src.contains("record_validity_event") && !src.contains("EvidenceValidityEvent"),
        "supersede does not append a validity event today"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn select_latest_walks_supersedes_then_collected_at_digest() {
    let older = seal_at(
        "evidence.control.observation",
        "asset:sel",
        ts(2026, 8, 1, 0),
        "state",
        "older-fail",
    );
    let newer = seal_at(
        "evidence.control.observation",
        "asset:sel",
        ts(2026, 8, 3, 0),
        "state",
        "newer-pass",
    )
    .with_supersedes(older.digest());
    let sibling = seal_at(
        "evidence.control.observation",
        "asset:sel",
        ts(2026, 8, 2, 0),
        "state",
        "sibling-mid",
    );
    let mut set = EvidenceSet::new();
    set.insert(older);
    set.insert(newer.clone());
    set.insert(sibling);
    let index = EvidenceIndex::build(&set);
    let picked = index
        .by_subject("evidence.control.observation", "asset:sel")
        .expect("group");
    assert_eq!(
        picked.digest(),
        newer.digest(),
        "select_latest keeps supersession leaves and then latest collected_at"
    );

    let pop = read_repo_file("crates/weeping-angel-control-test/src/population.rs");
    assert!(
        pop.contains("fn select_latest")
            && pop.contains("e.supersedes()")
            && pop.contains("collected_at")
            && pop.contains("a.digest().cmp(b.digest())"),
        "select_latest walks supersedes then collected_at then digest"
    );
    assert!(
        !pop.contains("as_of")
            && !pop.contains("valid_until")
            && !pop.contains("select_latest_as_of"),
        "select_latest has no as-of / validity filter today"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn select_latest_future_leaf_wins_with_no_as_of_filter() {
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
    let mut set = EvidenceSet::new();
    set.insert(past);
    set.insert(future.clone());
    let index = EvidenceIndex::build(&set);
    let picked = index
        .by_subject("evidence.control.observation", "asset:future")
        .expect("group");
    assert_eq!(
        picked.digest(),
        future.digest(),
        "a leaf collected after the evaluation clock still wins select_latest"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn overlapping_evidence_has_no_validity_window_latest_collected_wins() {
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
    set.insert(first);
    set.insert(second.clone());
    let index = EvidenceIndex::build(&set);
    let picked = index
        .by_subject("evidence.control.observation", "asset:overlap")
        .unwrap();
    assert_eq!(picked.digest(), second.digest());
    assert_eq!(
        set.len(),
        2,
        "both overlapping observations remain in the bag"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn exists_uses_digest_order_first_selector_not_as_of() {
    let a = seal_at(
        "evidence.control.observation",
        "asset:exists",
        ts(2026, 8, 1, 0),
        "state",
        "alpha",
    );
    let b = seal_at(
        "evidence.control.observation",
        "asset:exists",
        ts(2026, 8, 20, 0),
        "state",
        "omega",
    );
    let mut set = EvidenceSet::new();
    set.insert(a.clone());
    set.insert(b.clone());
    let first_digest = set.iter().next().unwrap().digest().to_string();
    let result = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 10, 0), Duration::from_secs(365 * 24 * 3600)),
    );
    assert_eq!(
        result.evidence_refs,
        vec![first_digest.clone()],
        "Exists binds the digest-order first match, not latest/as-of"
    );
    let src = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    assert!(
        src.contains("fn first_selector") && src.contains("envelopes.iter().copied().find"),
        "first_selector walks the EvidenceSet iterator (BTreeMap digest order)"
    );
    let _ = (a, b);
}

#[test]
#[ignore = "superseded by target suite"]
fn exists_effective_from_one_non_stale_observation() {
    let env = seal_at(
        "evidence.control.observation",
        "asset:one",
        ts(2026, 8, 18, 11),
        "state",
        "ok",
    );
    let mut set = EvidenceSet::new();
    set.insert(env);
    let result = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 18, 12), Duration::from_secs(24 * 3600)),
    );
    assert_eq!(result.effectiveness, Effectiveness::Effective);
    assert!(
        result.rationale.contains("exists"),
        "one fresh in-set observation is enough: {}",
        result.rationale
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn stale_is_age_of_collected_at_against_max_age() {
    let env = seal_at(
        "evidence.control.observation",
        "asset:stale",
        ts(2026, 8, 1, 0),
        "state",
        "ok",
    );
    let mut set = EvidenceSet::new();
    set.insert(env);
    let stale = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 18, 12), Duration::from_secs(24 * 3600)),
    );
    assert_eq!(stale.effectiveness, Effectiveness::StaleEvidence);
    let fresh = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 1, 12), Duration::from_secs(24 * 3600)),
    );
    assert_eq!(fresh.effectiveness, Effectiveness::Effective);

    let lib = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    assert!(
        lib.contains("signed_duration_since(collected)")
            && lib.contains(".to_std()")
            && lib.contains(".unwrap_or(true)"),
        "is_stale treats conversion failure (including future collected_at) as stale"
    );
    let pop = read_repo_file("crates/weeping-angel-control-test/src/population.rs");
    assert!(
        pop.contains("fn envelope_stale")
            && pop.contains("unwrap_or(Duration::MAX)")
            && pop.contains("age > context.max_age"),
        "envelope_stale is collected_at age vs max_age / selector freshness"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn future_collected_at_is_stale_and_can_shadow_older_candidate() {
    let older = seal_at(
        "evidence.control.observation",
        "asset:shadow",
        ts(2026, 8, 1, 0),
        "state",
        "older-ok",
    );
    let future = seal_at(
        "evidence.control.observation",
        "asset:shadow",
        ts(2026, 9, 1, 0),
        "state",
        "future-ok",
    );
    let now = ts(2026, 8, 10, 0);
    let max_age = Duration::from_secs(40 * 24 * 3600);

    let mut only_future = EvidenceSet::new();
    only_future.insert(future.clone());
    let future_only = evaluate(&exists_test(), &only_future, &ctx_at(now, max_age));
    assert_eq!(
        future_only.effectiveness,
        Effectiveness::StaleEvidence,
        "future collected_at is StaleEvidence via to_std failure, not excluded from the set"
    );

    let mut both = EvidenceSet::new();
    both.insert(older.clone());
    both.insert(future.clone());
    let first = both.iter().next().unwrap();
    let combined = evaluate(&exists_test(), &both, &ctx_at(now, max_age));
    if first.digest() == future.digest() {
        assert_eq!(
            combined.effectiveness,
            Effectiveness::StaleEvidence,
            "digest-order first_selector shadows the older still-valid envelope with future StaleEvidence"
        );
        assert_eq!(combined.evidence_refs, vec![future.digest().to_string()]);
    } else {
        assert_eq!(
            combined.effectiveness,
            Effectiveness::Effective,
            "when the older digest sorts first, Exists uses it; selection is still not as-of"
        );
        assert_eq!(combined.evidence_refs, vec![older.digest().to_string()]);
    }

    let index = EvidenceIndex::build(&both);
    assert_eq!(
        index
            .by_subject("evidence.control.observation", "asset:shadow")
            .unwrap()
            .digest(),
        future.digest(),
        "index latest still prefers the future-dated leaf"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn past_now_still_sees_every_envelope_in_the_set() {
    let later = seal_at(
        "evidence.control.observation",
        "asset:hist",
        ts(2026, 8, 20, 0),
        "state",
        "tuesday",
    );
    let mut set = EvidenceSet::new();
    set.insert(later);
    let monday = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 10, 0), Duration::from_secs(24 * 3600)),
    );
    assert_ne!(
        monday.effectiveness,
        Effectiveness::InsufficientEvidence,
        "a past now still binds envelopes already in the set (temporal leakage)"
    );
    assert_eq!(monday.effectiveness, Effectiveness::StaleEvidence);
}

#[test]
#[ignore = "superseded by target suite"]
fn no_expired_state_distinct_from_stale() {
    let env = seal_at(
        "evidence.control.observation",
        "asset:exp",
        ts(2026, 1, 1, 0),
        "state",
        "ok",
    );
    let json = serde_json::to_value(&env).unwrap();
    assert!(json.get("validUntil").is_none());
    let mut set = EvidenceSet::new();
    set.insert(env);
    let result = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 18, 12), Duration::from_secs(24 * 3600)),
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::StaleEvidence,
        "without valid_until, age-from-collection is the only way to leave the usable set"
    );
    let ct = crate_sources_joined("weeping-angel-control-test");
    assert!(
        !ct.contains("ExpiredEvidence") && !ct.contains("outside validity"),
        "no expired-effectiveness distinct from StaleEvidence"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn assessment_context_is_now_plus_max_age_only() {
    let src = read_repo_file("crates/weeping-angel-control-test/src/lib.rs");
    let start = src
        .find("pub struct AssessmentContext")
        .expect("AssessmentContext");
    let block = &src[start..start + 180];
    assert!(
        block.contains("pub now: DateTime<Utc>") && block.contains("pub max_age: Duration"),
        "AssessmentContext today is {{ now, max_age }}: {block}"
    );
    for needle in ["as_of", "period", "TimeRange", "FreshnessPolicy"] {
        assert!(
            !block.contains(needle),
            "AssessmentContext currently has no `{needle}`"
        );
    }

    let _ctx = AssessmentContext {
        now: ts(2026, 8, 18, 12),
        max_age: Duration::from_secs(24 * 3600),
    };
}

#[test]
#[ignore = "superseded by target suite"]
fn facade_assess_stamps_utc_now_and_hardcoded_24h() {
    let src = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    let assess = fn_assess(&src);
    assert!(
        assess.contains("now: Utc::now()"),
        "assess fills AssessmentContext.now from Utc::now"
    );
    assert!(
        assess.contains("max_age: Duration::from_secs(24 * 3600)"),
        "assess hard-codes 24h max_age"
    );
    for needle in ["as_of", "period", "FreshnessPolicy", "TimeRange"] {
        assert!(
            !assess.contains(needle),
            "assess currently has no injected `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by target suite"]
fn assessment_run_does_not_pin_evaluation_clock() {
    let run = AssessmentRun::default();
    let json = serde_json::to_value(&run).unwrap();
    let keys = object_keys(&json);
    for forbidden in ["asOf", "as_of", "period", "validFrom", "validUntil"] {
        assert!(
            !keys.contains(forbidden),
            "AssessmentRun JSON has no `{forbidden}`; keys={keys:?}"
        );
    }
    let src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    assert!(
        src.contains("pub struct AssessmentRun") && !src.contains("as_of"),
        "AssessmentRun does not pin as_of today"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn snapshot_diff_is_pairwise_readiness_not_period_coverage() {
    let mut previous = empty_readiness();
    previous.controls.push(ControlReadiness {
        id: ControlId::new("ctl.a"),
        effectiveness: Effectiveness::Ineffective,
    });
    let mut next = empty_readiness();
    next.controls.push(ControlReadiness {
        id: ControlId::new("ctl.a"),
        effectiveness: Effectiveness::Effective,
    });
    next.requirements
        .push(weeping_angel_assurance::readiness::RequirementReadiness {
            id: RequirementId::new("req.a"),
            status: "applicable".into(),
            mapped_controls: vec![ControlId::new("ctl.a")],
        });
    let diff = compare(&previous, &next);
    assert!(
        diff.control_became_effective
            .iter()
            .any(|s| s.contains("ctl.a")),
        "compare reports pairwise effectiveness transitions: {diff:?}"
    );
    let json = serde_json::to_value(&diff).unwrap();
    let keys = object_keys(&json);
    for forbidden in [
        "observationGaps",
        "expiredAt",
        "revoked",
        "intermittentControls",
        "coverageInsufficient",
        "periodEffectiveness",
    ] {
        assert!(
            !keys.contains(forbidden),
            "SnapshotDiff has no period-coverage field `{forbidden}`; keys={keys:?}"
        );
    }

    let run_a = AssessmentRun::default();
    let mut run_b = AssessmentRun::default();
    run_b.framework_pack_digest = "other".into();
    let run_diff = compare_runs(&run_a, &run_b);
    assert!(run_diff.framework_pack_digest_changed);
    assert!(compare_lineage(&run_a, &run_b).framework_pack_digest_changed);

    let snap_src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    assert!(
        !snap_src.contains("struct TemporalDiff") && !snap_src.contains("fn project_timeline"),
        "no timeline/diff primitives beside pairwise SnapshotDiff"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn effectiveness_has_no_period_variants() {
    let names: BTreeSet<&str> = [
        "effective",
        "ineffective",
        "partiallyEffective",
        "notApplicable",
        "notTested",
        "insufficientEvidence",
        "staleEvidence",
        "manualReviewRequired",
        "exceptionApproved",
        "inconclusive",
    ]
    .into_iter()
    .collect();
    for variant in [
        Effectiveness::Effective,
        Effectiveness::Ineffective,
        Effectiveness::PartiallyEffective,
        Effectiveness::NotApplicable,
        Effectiveness::NotTested,
        Effectiveness::InsufficientEvidence,
        Effectiveness::StaleEvidence,
        Effectiveness::ManualReviewRequired,
        Effectiveness::ExceptionApproved,
        Effectiveness::Inconclusive,
    ] {
        let label = serde_json::to_value(variant).unwrap();
        let s = label.as_str().expect("camelCase unit variant");
        assert!(names.contains(s), "unexpected Effectiveness variant {s}");
    }
    for forbidden in [
        "continuouslyEffective",
        "intermittentRegression",
        "insufficientObservationCoverage",
    ] {
        assert!(
            !names.contains(forbidden),
            "Effectiveness currently has no period outcome `{forbidden}`"
        );
    }
}

#[test]
#[ignore = "superseded by target suite"]
fn no_public_as_of_period_or_validity_event_contract() {
    let crates = product_crates_joined();
    for needle in [
        "struct TemporalQuery",
        "enum PeriodEffectiveness",
        "fn select_latest_as_of",
        "fn project_timeline",
        "struct TemporalDiff",
        "struct EvidenceValidityEvent",
        "EVIDENCE_VALIDITY_SCHEMA",
        "fn record_validity_event",
        "continuouslyEffective",
        "intermittentRegression",
        "insufficientObservationCoverage",
        "continuousUntilSuperseded",
    ] {
        assert!(
            !crates.contains(needle),
            "product crates currently have no temporal-assurance contract `{needle}`"
        );
    }
}

#[test]
#[ignore = "superseded by target suite"]
fn revocation_is_absent_and_cannot_withdraw_usability() {
    let ev = crate_sources_joined("weeping-angel-evidence");
    for needle in [
        "fn revoke",
        "record_validity_event",
        "evidence-validity/v1",
        "EvidenceValidityEvent",
    ] {
        assert!(
            !ev.contains(needle),
            "evidence crate currently has no revocation/validity-event surface `{needle}`"
        );
    }
    let env = seal_at(
        "evidence.control.observation",
        "asset:rev",
        ts(2026, 8, 1, 0),
        "state",
        "ok",
    );
    let mut set = EvidenceSet::new();
    set.insert(env);
    let result = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 1, 12), Duration::from_secs(24 * 3600)),
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::Effective,
        "without a revoke event, a stored envelope remains usable"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn prompt_13_scheduler_is_absent() {
    let assurance_src = crate_src("weeping-angel-assurance");
    assert!(
        !assurance_src.join("scheduler").exists(),
        "today there is no scheduler/ module directory"
    );
    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !lib.contains("pub mod scheduler") && !lib.contains("mod scheduler"),
        "lib.rs currently does not declare a scheduler module"
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
#[ignore = "superseded by target suite"]
fn no_period_projection_from_sparse_exists() {
    let env = seal_at(
        "evidence.control.observation",
        "asset:sparse",
        ts(2026, 8, 15, 12),
        "state",
        "one-shot",
    );
    let mut set = EvidenceSet::new();
    set.insert(env);
    let result = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 15, 13), Duration::from_secs(24 * 3600)),
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::Effective,
        "Exists infers Effective from a single observation; no period coverage object"
    );
    let json = serde_json::to_value(&result).unwrap();
    assert!(
        json.get("period").is_none() && json.get("periodEffectiveness").is_none(),
        "ControlTestResult has no period projection today: {json}"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn intermittent_pass_fail_is_still_point_in_time_exists() {
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
    );
    let mut set = EvidenceSet::new();
    set.insert(pass);
    set.insert(fail);
    let result = evaluate(
        &exists_test(),
        &set,
        &ctx_at(ts(2026, 8, 15, 0), Duration::from_secs(30 * 24 * 3600)),
    );
    assert_eq!(
        result.effectiveness,
        Effectiveness::Effective,
        "Exists does not project intermittentRegression across the period; one in-set hit is Effective"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn clock_boundaries_are_inclusive_collected_at_not_half_open_validity() {
    let mut ledger = EvidenceLedger::open_in_memory().unwrap();
    let at_start = seal_at(
        "evidence.control.observation",
        "asset:bound",
        ts(2026, 8, 1, 0),
        "state",
        "start",
    );
    let at_end = seal_at(
        "evidence.control.observation",
        "asset:bound",
        ts(2026, 8, 2, 0),
        "state",
        "end",
    );
    ledger.append(at_start).unwrap();
    ledger.append(at_end).unwrap();
    let window = ledger
        .within_window(ts(2026, 8, 1, 0), ts(2026, 8, 2, 0))
        .unwrap();
    assert_eq!(
        window.len(),
        2,
        "both inclusive collected_at bounds are kept"
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn historical_replay_uses_bag_contents_not_as_of_prefix() {
    let earlier = seal_at(
        "evidence.control.observation",
        "asset:replay",
        ts(2026, 8, 1, 0),
        "state",
        "monday",
    );
    let later = seal_at(
        "evidence.control.observation",
        "asset:replay",
        ts(2026, 8, 20, 0),
        "state",
        "tuesday",
    );
    let mut prefix = EvidenceSet::new();
    prefix.insert(earlier.clone());
    let at_monday = ctx_at(ts(2026, 8, 10, 0), Duration::from_secs(40 * 24 * 3600));
    let first = evaluate(&exists_test(), &prefix, &at_monday);
    assert_eq!(first.effectiveness, Effectiveness::Effective);
    assert_eq!(first.evidence_refs, vec![earlier.digest().to_string()]);

    prefix.insert(later.clone());
    let second = evaluate(&exists_test(), &prefix, &at_monday);
    let first_digest = prefix.iter().next().unwrap().digest().to_string();
    assert_eq!(
        second.evidence_refs,
        vec![first_digest],
        "appending a later envelope can change which digest Exists binds even at a past now"
    );
    assert_eq!(
        second.effectiveness,
        Effectiveness::Effective,
        "historical now does not exclude later-collected envelopes already in the set"
    );
    let _ = later;
}
