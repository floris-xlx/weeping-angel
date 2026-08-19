//! Baseline suite for ISMS events and deterministic drift (Prompt 15).
//!
//! Characterization of CURRENT behavior on SHA
//! `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` (`docs/specs/isms-events-drift.md`
//! §3). `weeping-angel-assurance` exposes `compare` / `compare_runs` /
//! `compare_lineage` → `SnapshotDiff` string bags. A newly appearing control is
//! pushed into `controlBecameEffective` regardless of effectiveness. There is
//! no events/drift module, no `ControlRegressed` / `IsmsEvent` /
//! `detect_isms_drift`, no SHA-256 event id, no `causeRefs`. IR `Risk` is
//! `{id,title,description,status}`; `EvidenceSnapshot` is envelope digest lists
//! without `validUntil`; Prompt 13/14 scheduler and temporal product are
//! absent.
//!
//! Target suite is the SSOT. After implement, `compare` / `SnapshotDiff`
//! characterizations remain GREEN (additive: detect_events sits beside compare).
//! Absence-of-events / pre-temporal found cases that no longer hold are
//! `#[ignore = "superseded by target suite"]`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{TimeZone, Utc};
use serde_json::Value;
use weeping_angel_assurance::lineage::seal_evidence_snapshot;
use weeping_angel_assurance::readiness::ControlReadiness;
use weeping_angel_assurance::{
    AssessmentRun, FrameworkReadinessSnapshot, SnapshotDiff, compare, compare_lineage, compare_runs,
};
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssessmentId, Asset, AssetId, AssetKind, ControlId,
    ControlImplementation, ControlImplementationId, Exception, ExceptionId, ExceptionStatus,
    RequirementId, Risk, RiskId, RiskStatus,
};
use weeping_angel_control_test::Effectiveness;

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

fn object_keys(value: &Value) -> BTreeSet<String> {
    match value {
        Value::Object(map) => map.keys().cloned().collect(),
        _ => BTreeSet::new(),
    }
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

fn readiness_with(
    id: &str,
    pack: &str,
    assessment: &str,
    controls: Vec<(&str, Effectiveness)>,
) -> FrameworkReadinessSnapshot {
    FrameworkReadinessSnapshot {
        assessment_id: AssessmentId::new(id),
        framework: "iso-27001".into(),
        framework_version: "2022".into(),
        framework_pack_digest: pack.into(),
        assessment_digest: assessment.into(),
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
        automation_coverage: "0".into(),
        evidence_coverage: "0".into(),
    }
}

fn sample_run(pack: &str, catalog: &str) -> AssessmentRun {
    AssessmentRun {
        id: AssessmentId::new("assess-events-baseline"),
        framework: "iso-27001".into(),
        framework_pack_digest: pack.into(),
        assessment_definition_digest: "def-a".into(),
        started_at: "2026-08-18T12:00:00Z".into(),
        completed_at: "2026-08-18T12:00:01Z".into(),
        scope: "repo:in-scope".into(),
        collector_runs: vec!["run:fixture".into()],
        evidence_snapshot_digest: "ev-a".into(),
        result_digest: "res-a".into(),
        status: "completed".into(),
        canonical_catalog_pin: catalog.into(),
        applicability_snapshot_id: "app-a".into(),
        as_of: "2026-08-18T12:00:00Z".into(),
    }
}

fn assert_empty_bags(diff: &SnapshotDiff) {
    assert!(diff.control_became_effective.is_empty());
    assert!(diff.control_became_ineffective.is_empty());
    assert!(diff.evidence_became_stale.is_empty());
    assert!(diff.new_subjects.is_empty());
    assert!(diff.disappeared_subjects.is_empty());
    assert!(diff.requirement_became_applicable.is_empty());
    assert!(diff.requirement_became_not_applicable.is_empty());
    assert!(diff.manual_review_resolved.is_empty());
    assert!(diff.new_exceptions.is_empty());
    assert!(diff.expired_exceptions.is_empty());
    assert!(diff.evidence_added.is_empty());
    assert!(diff.evidence_removed.is_empty());
    assert!(diff.evidence_superseded.is_empty());
    assert!(!diff.framework_pack_digest_changed);
    assert!(!diff.canonical_catalog_digest_changed);
}

const SNAPSHOT_DIFF_KEYS: &[&str] = &[
    "controlBecameEffective",
    "controlBecameIneffective",
    "evidenceBecameStale",
    "newSubjects",
    "disappearedSubjects",
    "requirementBecameApplicable",
    "requirementBecameNotApplicable",
    "manualReviewResolved",
    "newExceptions",
    "expiredExceptions",
    "evidenceAdded",
    "evidenceRemoved",
    "evidenceSuperseded",
    "frameworkPackDigestChanged",
    "canonicalCatalogDigestChanged",
];

const ABSENT_EVENT_NEEDLES: &[&str] = &[
    "struct IsmsEvent",
    "enum IsmsEventKind",
    "struct EventSubjectRef",
    "struct EventCauseRef",
    "enum EventSeverity",
    "struct IsmsSnapshot",
    "struct IsmsDrift",
    "fn detect_isms_drift",
    "fn detect_events",
    "fn append_isms_events",
    "ISMS_EVENT_SCHEMA",
    "isms-event/v1",
    "ControlRegressed",
    "ControlRecovered",
    "EvidenceExpired",
    "EvidenceRevoked",
    "RiskIncreased",
    "NewAssetDetected",
    "typed_id!(EventId)",
];

#[test]
fn dual_suite_is_registered() {
    let toml = read_repo_file("Cargo.toml");
    assert!(
        toml.contains("name = \"sdd_isms_events_drift_baseline\"")
            && toml.contains("path = \"tests/contracts/isms_events_drift.baseline.rs\"")
            && toml.contains("name = \"sdd_isms_events_drift_target\"")
            && toml.contains("path = \"tests/contracts/isms_events_drift.target.rs\""),
        "dual-suite must be listed in root Cargo.toml (tests/contracts is not auto-discovered)"
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/isms_events_drift.baseline.rs")
            .is_file()
    );
    assert!(
        manifest_dir()
            .join("tests/contracts/isms_events_drift.target.rs")
            .is_file()
    );
}

#[test]
#[ignore = "superseded by target suite"]
fn no_event_types_or_drift_module() {
    let assurance_src = crate_src("weeping-angel-assurance");
    assert!(
        !assurance_src.join("events.rs").exists() && !assurance_src.join("events").exists(),
        "today there is no events module under weeping-angel-assurance"
    );
    assert!(
        !assurance_src.join("drift.rs").exists() && !assurance_src.join("drift").exists(),
        "today there is no drift module under weeping-angel-assurance"
    );
    assert!(
        !crate_src("weeping-angel-assurance-ir")
            .join("event.rs")
            .exists(),
        "today IR has no event.rs"
    );

    let lib = read_repo_file("crates/weeping-angel-assurance/src/lib.rs");
    assert!(
        !lib.contains("pub mod events")
            && !lib.contains("mod events")
            && !lib.contains("pub mod drift")
            && !lib.contains("mod drift"),
        "assurance lib.rs currently does not declare events/drift"
    );

    let ir_lib = read_repo_file("crates/weeping-angel-assurance-ir/src/lib.rs");
    assert!(
        !ir_lib.contains("pub mod event") && !ir_lib.contains("EventId"),
        "IR lib.rs currently does not export an event module or EventId"
    );

    let id_src = read_repo_file("crates/weeping-angel-assurance-ir/src/id.rs");
    assert!(
        !id_src.contains("typed_id!(EventId)"),
        "id.rs currently has no EventId typed identity"
    );

    let crates = product_crates_joined();
    for needle in ABSENT_EVENT_NEEDLES {
        assert!(
            !crates.contains(needle),
            "product crates currently have no `{needle}`"
        );
    }
}

#[test]
fn snapshot_diff_is_camelcase_string_bag() {
    let diff = SnapshotDiff::default();
    let json = serde_json::to_value(&diff).unwrap();
    let keys = object_keys(&json);
    let expected: BTreeSet<String> = SNAPSHOT_DIFF_KEYS
        .iter()
        .map(|k| (*k).to_string())
        .collect();
    assert_eq!(
        keys, expected,
        "SnapshotDiff camelCase field set; got {json}"
    );

    for absent in [
        "eventId",
        "events",
        "causeRefs",
        "causes",
        "subjects",
        "severity",
        "occurredAt",
        "previousSnapshotDigest",
        "nextSnapshotDigest",
        "schemaVersion",
        "kind",
        "payload",
    ] {
        assert!(
            json.get(absent).is_none(),
            "found-case SnapshotDiff JSON must not contain `{absent}`"
        );
    }

    assert!(json["controlBecameIneffective"].is_array());
    assert!(json["newSubjects"].is_array());
    assert_eq!(json["frameworkPackDigestChanged"], false);
}

#[test]
fn p15_noop_snapshots() {
    let previous = readiness_with(
        "assess-noop",
        "pack-a",
        "def-a",
        vec![
            ("control.identity.mfa", Effectiveness::Effective),
            ("control.source.protected-branch", Effectiveness::Effective),
        ],
    );
    let mut reordered = previous.clone();
    reordered.controls.reverse();

    let same = compare(&previous, &previous);
    assert_empty_bags(&same);

    let shuffled = compare(&previous, &reordered);
    assert_empty_bags(&shuffled);

    let json = serde_json::to_value(&same).unwrap();
    assert_eq!(json, serde_json::to_value(&shuffled).unwrap());
    assert_eq!(json["controlBecameIneffective"], serde_json::json!([]));
    assert!(
        json.get("events").is_none(),
        "no-op compare currently yields a SnapshotDiff, not an event list"
    );
}

#[test]
fn p15_one_control_regression() {
    let previous = readiness_with(
        "assess-prev",
        "pack-a",
        "def-a",
        vec![("control.identity.privileged-mfa", Effectiveness::Effective)],
    );
    let next = readiness_with(
        "assess-next",
        "pack-a",
        "def-a",
        vec![(
            "control.identity.privileged-mfa",
            Effectiveness::Ineffective,
        )],
    );
    let diff = compare(&previous, &next);
    assert_eq!(
        diff.control_became_ineffective,
        vec!["control.identity.privileged-mfa became ineffective".to_string()]
    );
    assert!(diff.control_became_effective.is_empty());
    assert!(diff.new_subjects.is_empty());
    assert!(diff.disappeared_subjects.is_empty());

    let json = serde_json::to_value(&diff).unwrap();
    assert_eq!(
        json["controlBecameIneffective"],
        serde_json::json!(["control.identity.privileged-mfa became ineffective"])
    );
    assert!(json.get("kind").is_none());
    assert!(json.get("eventId").is_none());
    assert!(
        json.get("ControlRegressed").is_none(),
        "compare does not return a typed ControlRegressed event"
    );

    let src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    let body = fn_compare_body(&src);
    assert!(
        body.contains("became ineffective"),
        "compare still formats prose strings, not ControlRegressed"
    );
    assert!(
        !body.contains("ControlRegressed") && !body.contains("detect_isms_drift"),
        "compare body currently has no event detector"
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn p15_evidence_expiry_is_stale_control_id() {
    let previous = readiness_with(
        "assess-prev",
        "pack-a",
        "def-a",
        vec![("control.source.protected-branch", Effectiveness::Effective)],
    );
    let next = readiness_with(
        "assess-next",
        "pack-a",
        "def-a",
        vec![(
            "control.source.protected-branch",
            Effectiveness::StaleEvidence,
        )],
    );
    let diff = compare(&previous, &next);
    assert_eq!(
        diff.evidence_became_stale,
        vec!["control.source.protected-branch".to_string()]
    );
    assert!(
        diff.control_became_ineffective.is_empty(),
        "StaleEvidence is attributed as a control id, not a regression string"
    );
    assert!(diff.evidence_added.is_empty());
    assert!(diff.evidence_removed.is_empty());
    assert!(diff.evidence_superseded.is_empty());

    let snapshot = seal_evidence_snapshot(
        ["sha256:envelope-without-window".to_string()],
        ["run:fixture".to_string()],
    );
    let json = serde_json::to_value(&snapshot).unwrap();
    let keys = object_keys(&json);
    assert!(
        keys.contains("envelopeDigests")
            && keys.contains("collectionRunIds")
            && keys.contains("digest")
            && keys.contains("schema")
    );
    for absent in [
        "validUntil",
        "validFrom",
        "revokedAt",
        "invalidatedAt",
        "validity",
    ] {
        assert!(
            json.get(absent).is_none(),
            "EvidenceSnapshot currently has no `{absent}`; keys={keys:?}"
        );
    }

    let ev_src = crate_sources_joined("weeping-angel-evidence");
    assert!(
        !ev_src.contains("valid_until") && !ev_src.contains("validUntil"),
        "evidence crate currently has no validity window product"
    );
}

#[test]
fn p15_risk_increase_caused_by_a_control_regression() {
    let previous = readiness_with(
        "assess-prev",
        "pack-a",
        "def-a",
        vec![("control.identity.privileged-mfa", Effectiveness::Effective)],
    );
    let next = readiness_with(
        "assess-next",
        "pack-a",
        "def-a",
        vec![(
            "control.identity.privileged-mfa",
            Effectiveness::Ineffective,
        )],
    );
    let diff = compare(&previous, &next);
    assert_eq!(
        diff.control_became_ineffective,
        vec!["control.identity.privileged-mfa became ineffective".to_string()]
    );

    let mut risk = Risk::new(
        RiskId::new("risk:privileged-access"),
        "Privileged access without MFA",
        "Control regression would increase residual exposure.",
    );
    risk.status = RiskStatus::Open;
    let json = serde_json::to_value(&risk).unwrap();
    let mut keys: Vec<_> = object_keys(&json).into_iter().collect();
    keys.sort();
    assert_eq!(keys, vec!["description", "id", "status", "title"]);
    for absent in [
        "residualOrdinal",
        "inherentOrdinal",
        "linkedControlIds",
        "controlIds",
        "causes",
        "causeRefs",
        "vendorIds",
    ] {
        assert!(json.get(absent).is_none());
    }

    let impl_rec = ControlImplementation::new(
        ControlImplementationId::new("impl.privileged-mfa"),
        ControlId::new("control.identity.privileged-mfa"),
    )
    .with_risk(RiskId::new("risk:privileged-access"));
    assert_eq!(
        impl_rec.risk_ids(),
        &[RiskId::new("risk:privileged-access")]
    );

    let snapshot_src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    let src = fn_compare_body(&snapshot_src);
    assert!(
        !src.contains("risk") && !src.contains("cause"),
        "compare currently does not walk risks or emit cause references"
    );
    let diff_json = serde_json::to_value(&diff).unwrap();
    assert!(diff_json.get("causes").is_none());
    assert!(diff_json.get("causeRefs").is_none());
}

#[test]
fn p15_new_asset_in_scope() {
    let previous = readiness_with(
        "assess-prev",
        "pack-a",
        "def-a",
        vec![("control.identity.mfa", Effectiveness::Effective)],
    );
    let next = previous.clone();
    let diff = compare(&previous, &next);
    assert!(diff.new_subjects.is_empty());

    let mut definition = AssessmentDefinition::new(AssessmentId::new("assess-events-baseline"));
    definition.assets.push(Asset::new(
        AssetId::new("repo:new-in-scope"),
        AssetKind::Repository,
        "new in-scope repository",
    ));
    assert_eq!(definition.assets.len(), 1);
    assert_eq!(definition.assets[0].id.as_str(), "repo:new-in-scope");

    let with_new_control = readiness_with(
        "assess-next",
        "pack-a",
        "def-a",
        vec![
            ("control.identity.mfa", Effectiveness::Effective),
            (
                "control.identity.privileged-mfa",
                Effectiveness::Ineffective,
            ),
        ],
    );
    let control_diff = compare(&previous, &with_new_control);
    assert_eq!(
        control_diff.new_subjects,
        vec!["control.identity.privileged-mfa".to_string()],
        "newSubjects today are control ids, not Asset inventory membership"
    );
    assert!(
        !control_diff
            .new_subjects
            .iter()
            .any(|s| s.contains("repo:")),
        "compare does not emit NewAssetDetected / asset ids"
    );
    assert_eq!(
        control_diff.control_became_effective,
        vec!["control.identity.privileged-mfa became effective".to_string()],
        "a newly appearing control is pushed into controlBecameEffective regardless of effectiveness"
    );

    let snapshot_src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    let src = fn_compare_body(&snapshot_src);
    assert!(
        src.contains("fn subject_ids(snapshot: &FrameworkReadinessSnapshot)")
            || src.contains("snapshot.controls.iter().map(|c| c.id"),
        "subject_ids currently walks readiness controls"
    );
    assert!(!src.contains("assets"));
}

#[test]
fn p15_expired_exception() {
    let previous = readiness_with(
        "assess-prev",
        "pack-a",
        "def-a",
        vec![(
            "control.identity.privileged-mfa",
            Effectiveness::ExceptionApproved,
        )],
    );
    let next = readiness_with(
        "assess-next",
        "pack-a",
        "def-a",
        vec![(
            "control.identity.privileged-mfa",
            Effectiveness::Ineffective,
        )],
    );
    let diff = compare(&previous, &next);
    assert_eq!(
        diff.expired_exceptions,
        vec!["control.identity.privileged-mfa exception expired".to_string()]
    );
    assert!(
        diff.control_became_ineffective.is_empty(),
        "ExceptionApproved → Ineffective fills expiredExceptions, not became-ineffective"
    );

    let exception = Exception {
        id: ExceptionId::new("exc.privileged-mfa.break-glass"),
        control_id: Some(ControlId::new("control.identity.privileged-mfa")),
        rationale: "break-glass window".into(),
        status: ExceptionStatus::Expired,
        approved_by: None,
        expires_at: Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
        subjects: Vec::new(),
    };
    assert_eq!(exception.status, ExceptionStatus::Expired);
    assert!(exception.expires_at.is_some());

    let mut definition = AssessmentDefinition::new(AssessmentId::new("assess-events-baseline"));
    definition.exceptions.push(exception);
    let unchanged = compare(&previous, &previous);
    assert!(
        unchanged.expired_exceptions.is_empty(),
        "compare never reads AssessmentDefinition.exceptions / expires_at"
    );

    let snapshot_src = read_repo_file("crates/weeping-angel-assurance/src/snapshot.rs");
    let src = fn_compare_body(&snapshot_src);
    assert!(src.contains("ExceptionApproved"));
    assert!(!src.contains("expires_at"));
    assert!(!src.contains("ExceptionStatus"));
}

#[test]
fn p15_event_deduplication_on_repeated_diff() {
    let previous = readiness_with(
        "assess-prev",
        "pack-a",
        "def-a",
        vec![("control.identity.privileged-mfa", Effectiveness::Effective)],
    );
    let next = readiness_with(
        "assess-next",
        "pack-a",
        "def-a",
        vec![(
            "control.identity.privileged-mfa",
            Effectiveness::Ineffective,
        )],
    );
    let first = compare(&previous, &next);
    let second = compare(&previous, &next);
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap(),
        "identical compare inputs currently yield the same SnapshotDiff values"
    );
    let json = serde_json::to_value(&first).unwrap();
    assert!(
        json.get("eventId").is_none(),
        "there is no event identity to dedupe into a stream"
    );

    let crates = product_crates_joined();
    for needle in [
        "append_isms_events",
        "fn persist_events",
        "struct EventStore",
    ] {
        assert!(
            !crates.contains(needle),
            "product crates currently have no event persist `{needle}`"
        );
    }
}

#[test]
fn compare_runs_only_flips_digest_booleans() {
    let previous = sample_run("pack-a", "catalog-a");
    let mut next = sample_run("pack-b", "catalog-b");
    next.scope = "repo:other".into();
    next.evidence_snapshot_digest = "ev-b".into();

    let diff = compare_runs(&previous, &next);
    assert_empty_control_and_inventory_bags(&diff);
    assert!(diff.framework_pack_digest_changed);
    assert!(diff.canonical_catalog_digest_changed);

    let lineage = compare_lineage(&previous, &next);
    assert_eq!(
        serde_json::to_value(&diff).unwrap(),
        serde_json::to_value(&lineage).unwrap()
    );

    let only_pack = compare_runs(&previous, &sample_run("pack-b", "catalog-a"));
    assert!(only_pack.framework_pack_digest_changed);
    assert!(!only_pack.canonical_catalog_digest_changed);
}

fn assert_empty_control_and_inventory_bags(diff: &SnapshotDiff) {
    assert!(diff.control_became_effective.is_empty());
    assert!(diff.control_became_ineffective.is_empty());
    assert!(diff.evidence_became_stale.is_empty());
    assert!(diff.new_subjects.is_empty());
    assert!(diff.disappeared_subjects.is_empty());
    assert!(diff.new_exceptions.is_empty());
    assert!(diff.expired_exceptions.is_empty());
}

#[test]
fn newly_appearing_control_is_became_effective_regardless_of_effectiveness() {
    let previous = readiness_with("assess-prev", "pack-a", "def-a", Vec::new());
    let next = readiness_with(
        "assess-next",
        "pack-a",
        "def-a",
        vec![(
            "control.identity.privileged-mfa",
            Effectiveness::Ineffective,
        )],
    );
    let diff = compare(&previous, &next);
    assert_eq!(
        diff.control_became_effective,
        vec!["control.identity.privileged-mfa became effective".to_string()]
    );
    assert!(diff.control_became_ineffective.is_empty());
    assert_eq!(
        diff.new_subjects,
        vec!["control.identity.privileged-mfa".to_string()]
    );
}

#[ignore = "superseded by target suite"]
#[test]
fn no_notification_transport_or_bus() {
    let crates = product_crates_joined();
    for needle in [
        "slack",
        "Slack",
        "webhook",
        "Kafka",
        "NATS",
        "notification transport",
        "EventBus",
        "notification bus",
    ] {
        assert!(
            !crates.contains(needle),
            "product crates currently have no `{needle}` (Prompt 15 non-goal still holds)"
        );
    }
}

#[test]
fn compare_does_not_fill_evidence_add_remove_on_requirement_walk() {
    use weeping_angel_assurance::readiness::RequirementReadiness;

    let mut previous = readiness_with("assess-prev", "pack-a", "def-a", Vec::new());
    previous.requirements.push(RequirementReadiness {
        id: RequirementId::new("iso.5.1"),
        status: "not applicable".into(),
        mapped_controls: Vec::new(),
    });
    let mut next = previous.clone();
    next.requirements[0].status = "applicable".into();
    next.assessment_digest = "def-b".into();

    let diff = compare(&previous, &next);
    assert_eq!(
        diff.requirement_became_applicable,
        vec!["iso.5.1".to_string()]
    );
    assert!(diff.evidence_added.is_empty());
    assert!(diff.evidence_removed.is_empty());
    assert!(diff.framework_pack_digest_changed);
    assert!(
        diff.canonical_catalog_digest_changed,
        "assessment_digest inequality currently sets both digest-changed flags"
    );
}
