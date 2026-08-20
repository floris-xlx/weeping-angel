//! Deterministic, order-insensitive ISMS snapshot drift.
//!
//! `detect_events` / `detect_isms_drift` emit immutable observations. Existing
//! `compare` / `SnapshotDiff` remain readiness string bags.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use weeping_angel_assurance_ir::{
    Asset, ControlId, ControlImplementation, ControlTestId, EventCauseKind, EventCauseRef,
    EventSeverity, EventSubjectKind, EventSubjectRef, Exception, ExceptionStatus, IsmsEvent,
    IsmsEventKind, RiskId, RiskStatus, Vendor, rfc3339_z,
};
use weeping_angel_control_test::Effectiveness;

use crate::readiness::{ControlReadiness, FrameworkReadinessSnapshot};
use crate::snapshot::{SnapshotDiff, compare};
use crate::soa::StatementOfApplicability;
use weeping_angel_assurance_ir::AssessmentId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlPosture {
    pub id: ControlId,
    pub effectiveness: Effectiveness,
    #[serde(default)]
    pub implementation_ids: Vec<String>,
    #[serde(default)]
    pub test_ids: Vec<ControlTestId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskPosture {
    pub id: RiskId,
    pub status: RiskStatus,
    #[serde(default)]
    pub linked_control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residual_ordinal: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherent_ordinal: Option<i32>,
    #[serde(default)]
    pub vendor_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceValidityView {
    pub envelope_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceRecord {
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<DateTime<Utc>>,
}

/// Normalized management-system view assembled by callers / tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsmsSnapshot {
    pub snapshot_id: String,
    pub evaluated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub vendors: Vec<Vendor>,
    #[serde(default)]
    pub risks: Vec<RiskPosture>,
    #[serde(default)]
    pub controls: Vec<ControlPosture>,
    #[serde(default)]
    pub implementations: Vec<ControlImplementation>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
    #[serde(default)]
    pub evidence: Vec<EvidenceValidityView>,
    #[serde(default)]
    pub tests: Vec<ControlPosture>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soa: Option<StatementOfApplicability>,
    #[serde(default)]
    pub objectives: Vec<GovernanceRecord>,
    #[serde(default)]
    pub policies: Vec<GovernanceRecord>,
    #[serde(default)]
    pub findings: Vec<GovernanceRecord>,
    #[serde(default)]
    pub nonconformities: Vec<GovernanceRecord>,
    #[serde(default)]
    pub corrective_actions: Vec<GovernanceRecord>,
}

impl Default for IsmsSnapshot {
    fn default() -> Self {
        Self {
            snapshot_id: String::new(),
            evaluated_at: DateTime::<Utc>::from_timestamp(0, 0).expect("unix epoch"),
            run_id: None,
            assets: Vec::new(),
            vendors: Vec::new(),
            risks: Vec::new(),
            controls: Vec::new(),
            implementations: Vec::new(),
            exceptions: Vec::new(),
            evidence: Vec::new(),
            tests: Vec::new(),
            soa: None,
            objectives: Vec::new(),
            policies: Vec::new(),
            findings: Vec::new(),
            nonconformities: Vec::new(),
            corrective_actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsmsDrift {
    pub readiness: SnapshotDiff,
    pub events: Vec<IsmsEvent>,
}

/// Semantic drift detector. Output `events` are sorted by `eventId`.
pub fn detect_events(previous: &IsmsSnapshot, next: &IsmsSnapshot) -> Vec<IsmsEvent> {
    detect_isms_drift(previous, next).events
}

/// Scheduler Drift seam: readiness `compare` plus the event catalog.
pub fn detect_isms_drift(previous: &IsmsSnapshot, next: &IsmsSnapshot) -> IsmsDrift {
    let readiness = compare(&readiness_view(previous), &readiness_view(next));
    let events = detect_semantic_events(previous, next);
    IsmsDrift { readiness, events }
}

fn readiness_view(snapshot: &IsmsSnapshot) -> FrameworkReadinessSnapshot {
    let controls = snapshot
        .controls
        .iter()
        .map(|c| ControlReadiness {
            id: c.id.clone(),
            effectiveness: c.effectiveness,
        })
        .collect();
    FrameworkReadinessSnapshot::from_projected_controls(
        AssessmentId::new(
            snapshot
                .run_id
                .as_deref()
                .filter(|id| !id.is_empty())
                .unwrap_or("assess-isms-drift"),
        ),
        "iso-27001",
        "2022",
        "pack-isms-drift",
        String::new(),
        snapshot.snapshot_id.clone(),
        rfc3339_z(snapshot.evaluated_at),
        controls,
        Vec::new(),
        "0",
        "0",
    )
}

fn detect_semantic_events(previous: &IsmsSnapshot, next: &IsmsSnapshot) -> Vec<IsmsEvent> {
    let clock = next.evaluated_at;
    let prev_id = previous.snapshot_id.as_str();
    let next_id = next.snapshot_id.as_str();

    let prev_controls = index_controls(previous);
    let next_controls = index_controls(next);
    let mut control_events: BTreeMap<String, IsmsEvent> = BTreeMap::new();
    let mut events: Vec<IsmsEvent> = Vec::new();

    for (id, next_ctrl) in &next_controls {
        let Some(prev_ctrl) = prev_controls.get(id) else {
            continue;
        };
        if is_regression(prev_ctrl.effectiveness, next_ctrl.effectiveness) {
            let event = emit(
                IsmsEventKind::ControlRegressed,
                clock,
                prev_id,
                next_id,
                vec![subject(EventSubjectKind::Control, id)],
                Vec::new(),
                Some(regression_severity(next_ctrl.effectiveness)),
                json!({
                    "controlId": id,
                    "fromEffectiveness": prev_ctrl.effectiveness,
                    "toEffectiveness": next_ctrl.effectiveness,
                    "previousEffectiveness": prev_ctrl.effectiveness,
                    "nextEffectiveness": next_ctrl.effectiveness,
                    "testIds": next_ctrl.test_ids,
                }),
            );
            control_events.insert(id.clone(), event.clone());
            events.push(event);
        } else if is_recovery(prev_ctrl.effectiveness, next_ctrl.effectiveness) {
            events.push(emit(
                IsmsEventKind::ControlRecovered,
                clock,
                prev_id,
                next_id,
                vec![subject(EventSubjectKind::Control, id)],
                Vec::new(),
                Some(EventSeverity::Informational),
                json!({
                    "controlId": id,
                    "fromEffectiveness": prev_ctrl.effectiveness,
                    "toEffectiveness": next_ctrl.effectiveness,
                    "previousEffectiveness": prev_ctrl.effectiveness,
                    "nextEffectiveness": next_ctrl.effectiveness,
                }),
            ));
        }
    }

    let prev_evidence = index_evidence(previous);
    let next_evidence = index_evidence(next);
    let mut envelopes: BTreeSet<&str> = BTreeSet::new();
    envelopes.extend(prev_evidence.keys().copied());
    envelopes.extend(next_evidence.keys().copied());
    for digest in envelopes {
        let prev = prev_evidence.get(digest);
        let next_view = next_evidence.get(digest).copied().or(prev.copied());
        let Some(view) = next_view.or(prev.copied()) else {
            continue;
        };
        if evidence_expired(previous, next, prev.copied(), view) {
            events.push(emit(
                IsmsEventKind::EvidenceExpired,
                clock,
                prev_id,
                next_id,
                vec![subject(EventSubjectKind::Evidence, digest)],
                Vec::new(),
                Some(EventSeverity::Notable),
                json!({
                    "envelopeDigest": digest,
                    "validUntil": view.valid_until.map(rfc3339_z),
                }),
            ));
        }
        if let Some(at) = view.revoked_at.or(view.invalidated_at)
            && at > previous.evaluated_at
            && at <= clock
        {
            events.push(emit(
                IsmsEventKind::EvidenceRevoked,
                clock,
                prev_id,
                next_id,
                vec![subject(EventSubjectKind::Evidence, digest)],
                Vec::new(),
                Some(EventSeverity::Material),
                json!({
                    "envelopeDigest": digest,
                    "revokedAt": rfc3339_z(at),
                }),
            ));
        }
    }

    let prev_exceptions = index_exceptions(previous);
    let next_exceptions = index_exceptions(next);
    let mut exception_ids: BTreeSet<&str> = BTreeSet::new();
    exception_ids.extend(prev_exceptions.keys().copied());
    exception_ids.extend(next_exceptions.keys().copied());
    for id in exception_ids {
        let prev = prev_exceptions.get(id).copied();
        let nxt = next_exceptions.get(id).copied();
        if exception_expired(previous, next, prev, nxt) {
            let rec = nxt.or(prev).expect("exception exists");
            events.push(emit(
                IsmsEventKind::ExceptionExpired,
                clock,
                prev_id,
                next_id,
                vec![subject(EventSubjectKind::Exception, id)],
                Vec::new(),
                Some(EventSeverity::Notable),
                json!({
                    "exceptionId": id,
                    "controlId": rec.control_id,
                    "expiresAt": rec.expires_at.map(rfc3339_z),
                }),
            ));
        }
    }

    let prev_assets = index_assets(previous);
    let next_assets = index_assets(next);
    for (id, asset) in &next_assets {
        if prev_assets.contains_key(id) {
            continue;
        }
        events.push(emit(
            IsmsEventKind::NewAssetDetected,
            clock,
            prev_id,
            next_id,
            vec![subject(EventSubjectKind::Asset, id)],
            Vec::new(),
            Some(EventSeverity::Informational),
            json!({
                "assetId": id,
                "kind": asset.kind,
                "inScope": true,
            }),
        ));
    }
    for id in prev_assets.keys() {
        if next_assets.contains_key(id) {
            continue;
        }
        events.push(emit(
            IsmsEventKind::AssetRemoved,
            clock,
            prev_id,
            next_id,
            vec![subject(EventSubjectKind::Asset, id)],
            Vec::new(),
            Some(EventSeverity::Informational),
            json!({
                "assetId": id,
                "inScope": false,
            }),
        ));
    }

    let prev_risks = index_risks(previous);
    let next_risks = index_risks(next);
    for (id, nxt) in &next_risks {
        let Some(prev) = prev_risks.get(id) else {
            continue;
        };
        if risk_increased(prev, nxt) {
            let mut causes = Vec::new();
            for control_id in &nxt.linked_control_ids {
                if let Some(regressed) = control_events.get(control_id.as_str()) {
                    causes.push(EventCauseRef {
                        kind: EventCauseKind::Event,
                        id: regressed.event_id.as_str().to_string(),
                    });
                    causes.push(EventCauseRef {
                        kind: EventCauseKind::Control,
                        id: control_id.as_str().to_string(),
                    });
                }
            }
            let severity = if ordinal_jump(prev.residual_ordinal, nxt.residual_ordinal) > 1
                || (matches!(
                    prev.status,
                    RiskStatus::Mitigated | RiskStatus::Accepted | RiskStatus::Closed
                ) && nxt.status == RiskStatus::Open)
            {
                EventSeverity::Material
            } else {
                EventSeverity::Notable
            };
            events.push(emit(
                IsmsEventKind::RiskIncreased,
                clock,
                prev_id,
                next_id,
                vec![subject(EventSubjectKind::Risk, id)],
                causes,
                Some(severity),
                json!({
                    "riskId": id,
                    "fromStatus": prev.status,
                    "toStatus": nxt.status,
                    "fromOrdinal": prev.residual_ordinal,
                    "toOrdinal": nxt.residual_ordinal,
                }),
            ));
            for vendor_id in &nxt.vendor_ids {
                events.push(emit(
                    IsmsEventKind::VendorRiskChanged,
                    clock,
                    prev_id,
                    next_id,
                    vec![subject(EventSubjectKind::Vendor, vendor_id)],
                    Vec::new(),
                    Some(EventSeverity::Notable),
                    json!({
                        "vendorId": vendor_id,
                        "riskId": id,
                        "fromOrdinal": prev.residual_ordinal,
                        "toOrdinal": nxt.residual_ordinal,
                    }),
                ));
            }
        } else if risk_decreased(prev, nxt) {
            events.push(emit(
                IsmsEventKind::RiskDecreased,
                clock,
                prev_id,
                next_id,
                vec![subject(EventSubjectKind::Risk, id)],
                Vec::new(),
                Some(EventSeverity::Informational),
                json!({
                    "riskId": id,
                    "fromStatus": prev.status,
                    "toStatus": nxt.status,
                    "fromOrdinal": prev.residual_ordinal,
                    "toOrdinal": nxt.residual_ordinal,
                }),
            ));
        }
        if prev.status != RiskStatus::Accepted && nxt.status == RiskStatus::Accepted {
            events.push(emit(
                IsmsEventKind::RiskAccepted,
                clock,
                prev_id,
                next_id,
                vec![subject(EventSubjectKind::Risk, id)],
                Vec::new(),
                Some(EventSeverity::Notable),
                json!({
                    "riskId": id,
                }),
            ));
        }
    }

    emit_governance(
        &mut events,
        previous,
        next,
        &previous.objectives,
        &next.objectives,
        IsmsEventKind::ObjectiveMissed,
        EventSubjectKind::Objective,
        "missed",
    );
    emit_governance(
        &mut events,
        previous,
        next,
        &previous.policies,
        &next.policies,
        IsmsEventKind::PolicyExpired,
        EventSubjectKind::Policy,
        "expired",
    );
    emit_opened(
        &mut events,
        previous,
        next,
        &previous.findings,
        &next.findings,
        IsmsEventKind::AuditFindingOpened,
        EventSubjectKind::Finding,
    );
    emit_opened(
        &mut events,
        previous,
        next,
        &previous.nonconformities,
        &next.nonconformities,
        IsmsEventKind::NonconformityOpened,
        EventSubjectKind::Nonconformity,
    );
    emit_governance(
        &mut events,
        previous,
        next,
        &previous.corrective_actions,
        &next.corrective_actions,
        IsmsEventKind::CorrectiveActionOverdue,
        EventSubjectKind::Other,
        "overdue",
    );

    events.sort_by(|a, b| {
        a.event_id.as_str().cmp(b.event_id.as_str()).then_with(|| {
            a.kind
                .as_label()
                .cmp(&b.kind.as_label())
                .then_with(|| first_subject(a).cmp(&first_subject(b)))
        })
    });
    events
}

#[allow(clippy::too_many_arguments)]
fn emit(
    kind: IsmsEventKind,
    occurred_at: DateTime<Utc>,
    previous_snapshot: &str,
    next_snapshot: &str,
    subjects: Vec<EventSubjectRef>,
    cause_refs: Vec<EventCauseRef>,
    severity: Option<EventSeverity>,
    payload: Value,
) -> IsmsEvent {
    IsmsEvent::new(
        kind,
        occurred_at,
        previous_snapshot,
        next_snapshot,
        subjects,
        cause_refs,
        severity,
        payload,
    )
}

fn subject(kind: EventSubjectKind, id: impl Into<String>) -> EventSubjectRef {
    EventSubjectRef {
        kind,
        id: id.into(),
    }
}

fn first_subject(event: &IsmsEvent) -> String {
    event
        .subjects
        .first()
        .map(|s| s.id.clone())
        .unwrap_or_default()
}

fn index_controls(snapshot: &IsmsSnapshot) -> BTreeMap<String, &ControlPosture> {
    snapshot
        .controls
        .iter()
        .map(|c| (c.id.as_str().to_string(), c))
        .collect()
}

fn index_risks(snapshot: &IsmsSnapshot) -> BTreeMap<String, &RiskPosture> {
    snapshot
        .risks
        .iter()
        .map(|r| (r.id.as_str().to_string(), r))
        .collect()
}

fn index_assets(snapshot: &IsmsSnapshot) -> BTreeMap<String, &Asset> {
    snapshot
        .assets
        .iter()
        .map(|a| (a.id.as_str().to_string(), a))
        .collect()
}

fn index_exceptions(snapshot: &IsmsSnapshot) -> BTreeMap<&str, &Exception> {
    snapshot
        .exceptions
        .iter()
        .map(|e| (e.id.as_str(), e))
        .collect()
}

fn index_evidence(snapshot: &IsmsSnapshot) -> BTreeMap<&str, &EvidenceValidityView> {
    snapshot
        .evidence
        .iter()
        .map(|e| (e.envelope_digest.as_str(), e))
        .collect()
}

fn regression_severity(to: Effectiveness) -> EventSeverity {
    match to {
        Effectiveness::Ineffective => EventSeverity::Material,
        Effectiveness::PartiallyEffective => EventSeverity::Notable,
        _ => EventSeverity::Material,
    }
}

fn is_regression(from: Effectiveness, to: Effectiveness) -> bool {
    matches!(
        from,
        Effectiveness::Effective | Effectiveness::ExceptionApproved
    ) && matches!(
        to,
        Effectiveness::Ineffective | Effectiveness::PartiallyEffective
    )
}

fn is_recovery(from: Effectiveness, to: Effectiveness) -> bool {
    matches!(
        from,
        Effectiveness::Ineffective | Effectiveness::PartiallyEffective
    ) && to == Effectiveness::Effective
}

fn evidence_expired(
    previous: &IsmsSnapshot,
    next: &IsmsSnapshot,
    prev_view: Option<&EvidenceValidityView>,
    view: &EvidenceValidityView,
) -> bool {
    let Some(valid_until) = view.valid_until.or(prev_view.and_then(|v| v.valid_until)) else {
        return false;
    };
    if valid_until > next.evaluated_at {
        return false;
    }
    let inside_previous = match prev_view.and_then(|v| v.valid_until) {
        None => true,
        Some(prev_until) => previous.evaluated_at < prev_until,
    };
    inside_previous && valid_until <= next.evaluated_at
}

fn exception_expired(
    previous: &IsmsSnapshot,
    next: &IsmsSnapshot,
    prev: Option<&Exception>,
    nxt: Option<&Exception>,
) -> bool {
    let Some(current) = nxt.or(prev) else {
        return false;
    };
    let already = prev.is_some_and(|p| p.status == ExceptionStatus::Expired);
    if already {
        return false;
    }
    if nxt.is_some_and(|n| n.status == ExceptionStatus::Expired)
        && prev.is_none_or(|p| p.status != ExceptionStatus::Expired)
    {
        return true;
    }
    if let Some(expires_at) = current.expires_at {
        let was_in_force = prev.is_some_and(|p| {
            matches!(
                p.status,
                ExceptionStatus::Approved | ExceptionStatus::Proposed
            )
        });
        return was_in_force
            && expires_at <= next.evaluated_at
            && previous.evaluated_at < expires_at;
    }
    false
}

fn status_rank(status: RiskStatus) -> i32 {
    match status {
        RiskStatus::Closed | RiskStatus::Retired => 0,
        RiskStatus::Mitigated | RiskStatus::Accepted => 1,
        RiskStatus::Draft | RiskStatus::UnderTreatment => 2,
        RiskStatus::Open => 3,
    }
}

fn risk_increased(prev: &RiskPosture, next: &RiskPosture) -> bool {
    ordinal_increased(prev.residual_ordinal, next.residual_ordinal)
        || ordinal_increased(prev.inherent_ordinal, next.inherent_ordinal)
        || status_rank(next.status) > status_rank(prev.status)
}

fn risk_decreased(prev: &RiskPosture, next: &RiskPosture) -> bool {
    ordinal_increased(next.residual_ordinal, prev.residual_ordinal)
        || ordinal_increased(next.inherent_ordinal, prev.inherent_ordinal)
        || status_rank(next.status) < status_rank(prev.status)
}

fn ordinal_increased(prev: Option<i32>, next: Option<i32>) -> bool {
    match (prev, next) {
        (Some(a), Some(b)) => b > a,
        _ => false,
    }
}

fn ordinal_jump(prev: Option<i32>, next: Option<i32>) -> i32 {
    match (prev, next) {
        (Some(a), Some(b)) => b.saturating_sub(a),
        _ => 0,
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_governance(
    events: &mut Vec<IsmsEvent>,
    previous: &IsmsSnapshot,
    next: &IsmsSnapshot,
    prev_list: &[GovernanceRecord],
    next_list: &[GovernanceRecord],
    kind: IsmsEventKind,
    subject_kind: EventSubjectKind,
    terminal: &str,
) {
    if prev_list.is_empty() && next_list.is_empty() {
        return;
    }
    let prev = index_gov(prev_list);
    let nxt = index_gov(next_list);
    for (id, rec) in nxt {
        let before = prev.get(id);
        let due = rec.due_at.is_some_and(|d| d <= next.evaluated_at)
            && before.is_none_or(|b| b.due_at.is_none_or(|d| previous.evaluated_at < d));
        let status_hit = rec.status.eq_ignore_ascii_case(terminal)
            && before.is_none_or(|b| !b.status.eq_ignore_ascii_case(terminal));
        if due || status_hit {
            events.push(emit(
                kind.clone(),
                next.evaluated_at,
                &previous.snapshot_id,
                &next.snapshot_id,
                vec![subject(subject_kind, id)],
                Vec::new(),
                Some(EventSeverity::Notable),
                json!({ "id": id, "status": rec.status }),
            ));
        }
    }
}

fn emit_opened(
    events: &mut Vec<IsmsEvent>,
    previous: &IsmsSnapshot,
    next: &IsmsSnapshot,
    prev_list: &[GovernanceRecord],
    next_list: &[GovernanceRecord],
    kind: IsmsEventKind,
    subject_kind: EventSubjectKind,
) {
    if prev_list.is_empty() && next_list.is_empty() {
        return;
    }
    let prev = index_gov(prev_list);
    for rec in next_list {
        if prev.contains_key(rec.id.as_str()) {
            continue;
        }
        events.push(emit(
            kind.clone(),
            next.evaluated_at,
            &previous.snapshot_id,
            &next.snapshot_id,
            vec![subject(subject_kind, rec.id.as_str())],
            Vec::new(),
            Some(EventSeverity::Notable),
            json!({ "id": rec.id }),
        ));
    }
}

fn index_gov(list: &[GovernanceRecord]) -> BTreeMap<&str, &GovernanceRecord> {
    list.iter().map(|r| (r.id.as_str(), r)).collect()
}
