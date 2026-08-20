//! Target suite for ISMS events and deterministic drift (Prompt 15).
//!
//! Encodes DESIRED behavior: `weeping-angel-assurance` `events` + `drift`
//! modules, `detect_events(previous, next) → Vec<IsmsEvent>` (schema
//! `weeping-angel/isms-event/v1`). Must stay RED on CURRENT HEAD for missing
//! `detect_events` / `IsmsEvent` / `ControlRegressed`, not a missing `[[test]]`
//! harness. Do not implement the engine here and do not `#[ignore]`.
//!
//! EVT-001 no-op permuted-equal snapshots
//! EVT-002 one ControlRegressed (Effective → Ineffective)
//! EVT-003 EvidenceExpired (validUntil crossed; not StaleEvidence)
//! EVT-004 RiskIncreased caused by ControlRegressed (`causeRefs`)
//! EVT-005 NewAssetDetected (AssetId, not a control id)
//! EVT-006 ExceptionExpired keyed by ExceptionId
//! EVT-007 detect_events(A, B) twice → same eventId set (no UUID v4)

use std::collections::BTreeSet;
use std::fs;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance::drift::{
    ControlPosture, EvidenceValidityView, IsmsSnapshot, RiskPosture, detect_events,
};
use weeping_angel_assurance::events::{EventId, ISMS_EVENT_SCHEMA, IsmsEvent, IsmsEventKind};
use weeping_angel_assurance::{SnapshotDiff, compare};
use weeping_angel_assurance_ir::{
    AssessmentId, Asset, AssetId, AssetKind, ControlId, Exception, ExceptionId, ExceptionStatus,
    RiskId, RiskStatus, validate_stable_id,
};
use weeping_angel_control_test::Effectiveness;

const SCHEMA: &str = "weeping-angel/isms-event/v1";
const MFA: &str = "control.identity.privileged-mfa";
const BRANCH: &str = "control.source.protected-branch";
const RISK: &str = "risk:privileged-access";
const ASSET_EXISTING: &str = "repo:in-scope";
const ASSET_NEW: &str = "repo:new-in-scope";
const EXCEPTION: &str = "exc.privileged-mfa.break-glass";
const ENVELOPE: &str = "sha256:envelope-mfa-window";
const SNAP_PREV: &str = "snap-prev";
const SNAP_NEXT: &str = "snap-next";

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/mod.rs"));

fn t0() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 18, 12, 0, 0).unwrap()
}

fn t1() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 19, 12, 0, 0).unwrap()
}

fn rfc3339(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn looks_like_uuid_v4(raw: &str) -> bool {
    let parts: Vec<&str> = raw.split('-').collect();
    parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[2].starts_with('4')
        && parts[3].len() == 4
        && parts[4].len() == 12
        && raw.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

fn control(id: &str, effectiveness: Effectiveness) -> ControlPosture {
    ControlPosture {
        id: ControlId::new(id),
        effectiveness,
        implementation_ids: Vec::new(),
        test_ids: Vec::new(),
    }
}

fn risk_posture(ordinal: i32, status: RiskStatus) -> RiskPosture {
    RiskPosture {
        id: RiskId::new(RISK),
        status,
        linked_control_ids: vec![ControlId::new(MFA)],
        residual_ordinal: Some(ordinal),
        inherent_ordinal: Some(ordinal),
        vendor_ids: Vec::new(),
    }
}

fn asset(id: &str, name: &str) -> Asset {
    Asset::new(AssetId::new(id), AssetKind::Repository, name)
}

fn validity_window() -> EvidenceValidityView {
    EvidenceValidityView {
        envelope_digest: ENVELOPE.into(),
        valid_from: Some(t0()),
        valid_until: Some(t1()),
        revoked_at: None,
        invalidated_at: None,
    }
}

fn approved_exception() -> Exception {
    Exception {
        id: ExceptionId::new(EXCEPTION),
        control_id: Some(ControlId::new(MFA)),
        rationale: "break-glass window".into(),
        status: ExceptionStatus::Approved,
        approved_by: None,
        expires_at: Some(t1()),
        subjects: Vec::new(),
    }
}

fn snapshot(id: &str, evaluated_at: DateTime<Utc>, controls: Vec<ControlPosture>) -> IsmsSnapshot {
    IsmsSnapshot {
        snapshot_id: id.into(),
        evaluated_at,
        assets: vec![asset(ASSET_EXISTING, "in-scope repository")],
        controls,
        risks: vec![risk_posture(2, RiskStatus::Mitigated)],
        exceptions: vec![approved_exception()],
        evidence: vec![validity_window()],
        ..Default::default()
    }
}

fn previous_ready() -> IsmsSnapshot {
    snapshot(
        SNAP_PREV,
        t0(),
        vec![
            control(MFA, Effectiveness::Effective),
            control(BRANCH, Effectiveness::Effective),
        ],
    )
}

fn json(event: &IsmsEvent) -> Value {
    serde_json::to_value(event).expect("IsmsEvent must serialize to camelCase JSON")
}

fn event_id_str(event: &IsmsEvent) -> String {
    let from_json = json(event)["eventId"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_default();
    if !from_json.is_empty() {
        from_json
    } else {
        event.event_id.as_str().to_string()
    }
}

fn event_ids(events: &[IsmsEvent]) -> Vec<String> {
    events.iter().map(event_id_str).collect()
}

fn event_id_set(events: &[IsmsEvent]) -> BTreeSet<String> {
    events.iter().map(event_id_str).collect()
}

fn kind_name(event: &IsmsEvent) -> String {
    match &json(event)["kind"] {
        Value::String(s) => s.clone(),
        Value::Object(map) => map.keys().next().cloned().unwrap_or_default(),
        other => panic!("kind must be a string or internally tagged object, got {other}"),
    }
}

fn of_kind(events: &[IsmsEvent], name: &str) -> Vec<IsmsEvent> {
    events
        .iter()
        .filter(|event| kind_name(event) == name)
        .cloned()
        .collect()
}

fn string_list(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| match item {
                Value::String(s) => Some(s.clone()),
                Value::Object(map) => map
                    .get("id")
                    .or_else(|| map.get("snapshotId"))
                    .or_else(|| map.get("digest"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
                _ => None,
            })
            .collect(),
        Value::String(s) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn subject_ids(event: &IsmsEvent) -> Vec<String> {
    json(event)["subjects"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_string))
        .collect()
}

fn cause_ids(event: &IsmsEvent) -> Vec<String> {
    let doc = json(event);
    let refs = doc
        .get("causeRefs")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    refs.as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_string))
        .collect()
}

fn assert_sorted(events: &[IsmsEvent]) {
    let ids = event_ids(events);
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(
        ids, sorted,
        "detect_events must return events sorted by eventId"
    );
}

fn assert_event_identity(event: &IsmsEvent, previous: &IsmsSnapshot, next: &IsmsSnapshot) {
    assert_eq!(ISMS_EVENT_SCHEMA, SCHEMA);
    let doc = json(event);
    assert_eq!(
        doc["schemaVersion"].as_str(),
        Some(SCHEMA),
        "schema must be {SCHEMA}, got {doc}"
    );

    let id = event_id_str(event);
    assert!(
        id.starts_with("event:sha256:"),
        "eventId must be event:sha256:{{hex}}, got {id}"
    );
    let hex = id.trim_start_matches("event:sha256:");
    assert_eq!(hex.len(), 64, "SHA-256 hex must be 64 chars, got {id}");
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "eventId digest must be hex, got {id}"
    );
    validate_stable_id(&id).unwrap_or_else(|e| panic!("eventId must be a StableId: {e}"));
    assert!(
        !looks_like_uuid_v4(&id),
        "persisted eventId must not be a random UUID v4: {id}"
    );
    let _typed = EventId::new(id.clone());
    assert_eq!(_typed.as_str(), id);

    let occurred = doc
        .get("occurredAt")
        .or_else(|| doc.get("evaluatedAt"))
        .and_then(Value::as_str)
        .unwrap_or("");
    assert_eq!(
        occurred,
        rfc3339(next.evaluated_at),
        "event time must come from next.evaluatedAt, got {doc}"
    );

    let sources = doc
        .get("sourceSnapshots")
        .map(string_list)
        .unwrap_or_default();
    assert!(
        sources
            .iter()
            .any(|s| s == &previous.snapshot_id || s.contains(&previous.snapshot_id)),
        "sourceSnapshots must cite previous {}, got {sources:?}",
        previous.snapshot_id
    );
    assert!(
        sources
            .iter()
            .any(|s| s == &next.snapshot_id || s.contains(&next.snapshot_id)),
        "sourceSnapshots must cite next {}, got {sources:?}",
        next.snapshot_id
    );

    let subjects = doc
        .get("subjects")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut sorted_subjects = subjects.clone();
    sorted_subjects.sort_by(|a, b| {
        let ak = a.get("kind").and_then(Value::as_str).unwrap_or("");
        let bk = b.get("kind").and_then(Value::as_str).unwrap_or("");
        let ai = a.get("id").and_then(Value::as_str).unwrap_or("");
        let bi = b.get("id").and_then(Value::as_str).unwrap_or("");
        (ak, ai).cmp(&(bk, bi))
    });
    assert_eq!(subjects, sorted_subjects, "subjects must be sorted");

    for ticket in ["assignee", "ack", "sla", "status", "ticketId"] {
        assert!(
            doc.get(ticket).is_none(),
            "events are immutable observations, not workflow tickets; found `{ticket}`"
        );
    }
}

fn assert_catalog_exists() {
    let _ = [
        IsmsEventKind::ControlRegressed,
        IsmsEventKind::ControlRecovered,
        IsmsEventKind::EvidenceExpired,
        IsmsEventKind::EvidenceRevoked,
        IsmsEventKind::RiskIncreased,
        IsmsEventKind::RiskDecreased,
        IsmsEventKind::RiskAccepted,
        IsmsEventKind::ExceptionExpired,
        IsmsEventKind::NewAssetDetected,
        IsmsEventKind::AssetRemoved,
        IsmsEventKind::VendorRiskChanged,
        IsmsEventKind::ObjectiveMissed,
        IsmsEventKind::PolicyExpired,
        IsmsEventKind::AuditFindingOpened,
        IsmsEventKind::NonconformityOpened,
        IsmsEventKind::CorrectiveActionOverdue,
    ];
    let _ = IsmsEventKind::Extensible {
        name: "custom.governance.kind".into(),
    };
}

fn readiness_pair(previous: Effectiveness, next: Effectiveness) -> SnapshotDiff {
    use weeping_angel_assurance::readiness::{ControlReadiness, FrameworkReadinessSnapshot};

    let snap = |effectiveness: Effectiveness, digest: &str| FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new("assess-events-target"),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: "pack-a".into(),
        catalog_digest: String::new(),
        assessment_digest: digest.into(),
        evaluated_at: rfc3339(t1()),
        requirements: Vec::new(),
        controls: vec![ControlReadiness {
            id: ControlId::new(MFA),
            effectiveness,
        }],
        effective: 0,
        ineffective: 0,
        partial: 0,
        manual_review: 0,
        insufficient_evidence: 0,
        not_applicable: 0,
        automation_coverage: "0".into(),
        evidence_coverage: "0".into(),
    };
    compare(&snap(previous, "def-a"), &snap(next, "def-a"))
}

#[test]
fn dual_suite_is_registered() {
    let toml = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    assert!(
        !toml.contains("name = \"sdd_isms_events_drift_baseline\"")
            && !toml.contains("path = \"tests/contracts/isms_events_drift.baseline.rs\"")
            && toml.contains("name = \"sdd_isms_events_drift_target\"")
            && toml.contains("path = \"tests/contracts/isms_events_drift.target.rs\""),
        "dual-suite must be listed in root Cargo.toml (tests/contracts is not auto-discovered)"
    );
}

/// EVT-001: semantically equal snapshots, including Vec permutation, emit zero events.
#[test]
fn evt_001_noop_permuted_equal_snapshots() {
    assert_catalog_exists();
    let previous = previous_ready();
    assert!(
        detect_events(&previous, &previous).is_empty(),
        "identical snapshots must emit zero events"
    );

    let mut permuted = previous.clone();
    permuted.snapshot_id = SNAP_NEXT.into();
    permuted.controls.reverse();
    permuted.assets.reverse();
    permuted.exceptions.reverse();
    permuted.evidence.reverse();
    permuted.risks.reverse();

    let events = detect_events(&previous, &permuted);
    assert!(
        events.is_empty(),
        "permuted-equal inventories must emit zero events, got {:?}",
        events.iter().map(kind_name).collect::<Vec<_>>()
    );
}

/// EVT-002: Effective → Ineffective on the same ControlId is one ControlRegressed.
#[test]
fn evt_002_one_control_regressed() {
    let previous = previous_ready();
    let mut next = snapshot(
        SNAP_NEXT,
        t1(),
        vec![
            control(MFA, Effectiveness::Ineffective),
            control(BRANCH, Effectiveness::Effective),
        ],
    );
    next.evidence = previous.evidence.clone();
    next.exceptions = previous.exceptions.clone();
    next.risks = previous.risks.clone();
    next.assets = previous.assets.clone();

    let events = detect_events(&previous, &next);
    assert_sorted(&events);
    let regressed = of_kind(&events, "ControlRegressed");
    assert_eq!(
        regressed.len(),
        1,
        "exactly one ControlRegressed, got {:?}",
        events.iter().map(kind_name).collect::<Vec<_>>()
    );
    assert!(matches!(regressed[0].kind, IsmsEventKind::ControlRegressed));
    let event = &regressed[0];
    assert_event_identity(event, &previous, &next);
    assert_eq!(subject_ids(event), vec![MFA.to_string()]);
    let payload = &json(event)["payload"];
    assert_eq!(payload["previousEffectiveness"], "effective");
    assert_eq!(payload["nextEffectiveness"], "ineffective");

    let diff = readiness_pair(Effectiveness::Effective, Effectiveness::Ineffective);
    assert_eq!(
        diff.control_became_ineffective,
        vec![format!("{MFA} became ineffective")]
    );
    assert!(json(event).get("controlBecameIneffective").is_none());
}

/// EVT-003: validUntil crossed by next.evaluatedAt emits EvidenceExpired, not StaleEvidence.
#[test]
fn evt_003_evidence_expired() {
    let previous = previous_ready();
    let mut next = previous.clone();
    next.snapshot_id = SNAP_NEXT.into();
    next.evaluated_at = t1();
    next.controls = previous.controls.clone();
    next.risks = previous.risks.clone();
    next.assets = previous.assets.clone();
    next.exceptions = previous.exceptions.clone();

    let events = detect_events(&previous, &next);
    let expired = of_kind(&events, "EvidenceExpired");
    assert_eq!(
        expired.len(),
        1,
        "validUntil crossed by next.evaluatedAt emits EvidenceExpired, got {:?}",
        events.iter().map(kind_name).collect::<Vec<_>>()
    );
    assert!(matches!(expired[0].kind, IsmsEventKind::EvidenceExpired));
    assert!(of_kind(&events, "StaleEvidence").is_empty());
    let event = &expired[0];
    assert_event_identity(event, &previous, &next);
    let payload = &json(event)["payload"];
    assert_eq!(payload["envelopeDigest"], ENVELOPE);
    assert_eq!(payload["validUntil"], rfc3339(t1()));
}

/// EVT-004: worse residual risk + linked ControlRegressed → RiskIncreased.causeRefs.
#[test]
fn evt_004_risk_increased_caused_by_control_regression() {
    let previous = previous_ready();
    let mut next = snapshot(
        SNAP_NEXT,
        t1(),
        vec![
            control(MFA, Effectiveness::Ineffective),
            control(BRANCH, Effectiveness::Effective),
        ],
    );
    next.risks = vec![risk_posture(4, RiskStatus::Open)];
    next.evidence = previous.evidence.clone();
    next.exceptions = previous.exceptions.clone();
    next.assets = previous.assets.clone();

    let events = detect_events(&previous, &next);
    assert_sorted(&events);
    let regressed = of_kind(&events, "ControlRegressed");
    let increased = of_kind(&events, "RiskIncreased");
    assert_eq!(
        regressed.len(),
        1,
        "need ControlRegressed, got {:?}",
        events.iter().map(kind_name).collect::<Vec<_>>()
    );
    assert_eq!(
        increased.len(),
        1,
        "need RiskIncreased, got {:?}",
        events.iter().map(kind_name).collect::<Vec<_>>()
    );
    assert!(matches!(increased[0].kind, IsmsEventKind::RiskIncreased));

    let regression_id = event_id_str(&regressed[0]);
    let causes = cause_ids(&increased[0]);
    assert!(
        causes.iter().any(|id| id == &regression_id),
        "RiskIncreased.causeRefs must contain ControlRegressed eventId {regression_id}, got {causes:?}"
    );
    assert_eq!(subject_ids(&increased[0]), vec![RISK.to_string()]);
    assert_event_identity(&increased[0], &previous, &next);
}

/// EVT-005: a new in-scope AssetId emits NewAssetDetected (never a control id).
#[test]
fn evt_005_new_asset_detected() {
    let previous = previous_ready();
    let mut next = previous.clone();
    next.snapshot_id = SNAP_NEXT.into();
    next.evaluated_at = t1();
    next.assets
        .push(asset(ASSET_NEW, "new in-scope repository"));
    next.controls = previous.controls.clone();
    next.risks = previous.risks.clone();
    next.evidence = previous.evidence.clone();
    next.exceptions = previous.exceptions.clone();

    let events = detect_events(&previous, &next);
    let detected = of_kind(&events, "NewAssetDetected");
    assert_eq!(
        detected.len(),
        1,
        "new in-scope asset emits NewAssetDetected, got {:?}",
        events.iter().map(kind_name).collect::<Vec<_>>()
    );
    assert!(matches!(detected[0].kind, IsmsEventKind::NewAssetDetected));
    assert_eq!(subject_ids(&detected[0]), vec![ASSET_NEW.to_string()]);
    assert!(
        !subject_ids(&detected[0])
            .iter()
            .any(|id| id.starts_with("control.")),
        "NewAssetDetected must be keyed by AssetId, not a readiness control id"
    );
    let payload = &json(&detected[0])["payload"];
    assert_eq!(payload["assetId"], ASSET_NEW);
    assert_event_identity(&detected[0], &previous, &next);

    let diff = {
        use weeping_angel_assurance::readiness::{ControlReadiness, FrameworkReadinessSnapshot};
        let prev = FrameworkReadinessSnapshot {
            assessment_id: AssessmentId::new("assess-prev"),
            framework: "iso-27001".into(),
            framework_version: "2022".into(),
            framework_pack_digest: "pack-a".into(),
            catalog_digest: String::new(),
            assessment_digest: "def-a".into(),
            evaluated_at: rfc3339(t0()),
            requirements: Vec::new(),
            controls: vec![ControlReadiness {
                id: ControlId::new(MFA),
                effectiveness: Effectiveness::Effective,
            }],
            effective: 0,
            ineffective: 0,
            partial: 0,
            manual_review: 0,
            insufficient_evidence: 0,
            not_applicable: 0,
            automation_coverage: "0".into(),
            evidence_coverage: "0".into(),
        };
        let mut nxt = prev.clone();
        nxt.controls.push(ControlReadiness {
            id: ControlId::new(BRANCH),
            effectiveness: Effectiveness::Effective,
        });
        compare(&prev, &nxt)
    };
    assert_eq!(diff.new_subjects, vec![BRANCH.to_string()]);
    assert!(
        !diff.new_subjects.iter().any(|s| s.contains("repo:")),
        "compare newSubjects remain control ids; NewAssetDetected is inventory membership"
    );
}

/// EVT-006: expiresAt / Expired status emits ExceptionExpired keyed by ExceptionId.
#[test]
fn evt_006_exception_expired() {
    let previous = previous_ready();
    let mut next = previous.clone();
    next.snapshot_id = SNAP_NEXT.into();
    next.evaluated_at = t1();
    next.exceptions = vec![Exception {
        status: ExceptionStatus::Expired,
        ..approved_exception()
    }];
    next.controls = previous.controls.clone();
    next.risks = previous.risks.clone();
    next.assets = previous.assets.clone();
    next.evidence = previous.evidence.clone();

    let events = detect_events(&previous, &next);
    let expired = of_kind(&events, "ExceptionExpired");
    assert_eq!(
        expired.len(),
        1,
        "expired exception emits ExceptionExpired keyed by ExceptionId, got {:?}",
        events.iter().map(kind_name).collect::<Vec<_>>()
    );
    assert!(matches!(expired[0].kind, IsmsEventKind::ExceptionExpired));
    assert_eq!(subject_ids(&expired[0]), vec![EXCEPTION.to_string()]);
    let payload = &json(&expired[0])["payload"];
    assert_eq!(payload["exceptionId"], EXCEPTION);
    assert_event_identity(&expired[0], &previous, &next);
}

/// EVT-007: detect_events is pure; repeated diff yields the same eventId set.
#[test]
fn evt_007_detect_events_dedupes() {
    let previous = previous_ready();
    let mut next = snapshot(
        SNAP_NEXT,
        t1(),
        vec![
            control(MFA, Effectiveness::Ineffective),
            control(BRANCH, Effectiveness::Effective),
        ],
    );
    next.evidence = previous.evidence.clone();
    next.exceptions = previous.exceptions.clone();
    next.risks = previous.risks.clone();
    next.assets = previous.assets.clone();

    let first = detect_events(&previous, &next);
    let second = detect_events(&previous, &next);
    assert!(!first.is_empty(), "regression pair must emit events");
    assert_eq!(
        event_id_set(&first),
        event_id_set(&second),
        "repeated detect_events on the same pair must reuse eventId"
    );
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap()
    );
    assert_sorted(&first);
    for id in event_ids(&first) {
        assert!(id.starts_with("event:sha256:"));
        assert!(!looks_like_uuid_v4(&id));
        validate_stable_id(&id).unwrap();
    }
}

/// Recovery is a new ControlRecovered observation, not a mutated ControlRegressed ticket.
#[test]
fn p15_control_recovered_is_a_new_event() {
    let previous = snapshot(
        SNAP_PREV,
        t0(),
        vec![control(MFA, Effectiveness::Ineffective)],
    );
    let next = snapshot(
        SNAP_NEXT,
        t1(),
        vec![control(MFA, Effectiveness::Effective)],
    );
    let events = detect_events(&previous, &next);
    let recovered = of_kind(&events, "ControlRecovered");
    assert_eq!(
        recovered.len(),
        1,
        "expected ControlRecovered, got {:?}",
        events.iter().map(kind_name).collect::<Vec<_>>()
    );
    assert!(matches!(recovered[0].kind, IsmsEventKind::ControlRecovered));
    assert!(of_kind(&events, "ControlRegressed").is_empty());
    let payload = &json(&recovered[0])["payload"];
    assert_eq!(payload["previousEffectiveness"], "ineffective");
    assert_eq!(payload["nextEffectiveness"], "effective");
    assert_event_identity(&recovered[0], &previous, &next);
}

/// compare / SnapshotDiff field meanings stay the readiness string bag.
#[test]
fn p15_compare_snapshot_diff_not_forked() {
    let previous = previous_ready();
    let mut next = snapshot(
        SNAP_NEXT,
        t1(),
        vec![control(MFA, Effectiveness::Ineffective)],
    );
    next.evidence = previous.evidence.clone();
    let _ = detect_events(&previous, &next);
    let diff = readiness_pair(Effectiveness::Effective, Effectiveness::Ineffective);
    let doc = serde_json::to_value(&diff).unwrap();
    assert!(doc["controlBecameIneffective"].is_array());
    assert!(doc.get("eventId").is_none());
    assert!(doc.get("kind").is_none());
    assert!(doc.get("causeRefs").is_none());
    let _ = SnapshotDiff::default();
}
