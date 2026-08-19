//! Canonical organizational information-security incident record.
//!
//! Created only by explicit [`Incident::declare`]. Scanner findings, imported
//! alerts, and Prompt 15 events are detection sources, not incidents.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::validation::IrValidationError;
use crate::{
    AlertRef, AssetId, ControlId, ControlTestId, EventRef, FindingRef, IdentityId, IncidentId,
    PrincipalRef, ProcessingActivityId, RemediationRef, RiskId, SubjectKind, SubjectSelector,
};

fn incident_version_default() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IncidentKind {
    Real,
    Exercise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum IncidentStatus {
    #[default]
    Declared,
    Investigating,
    Contained,
    Eradicated,
    Recovered,
    Closed,
    Cancelled,
}

impl IncidentStatus {
    pub fn can_transition(from: Self, to: Self) -> bool {
        use IncidentStatus::*;
        matches!(
            (from, to),
            (Declared, Investigating)
                | (Declared, Contained)
                | (Declared, Cancelled)
                | (Investigating, Contained)
                | (Investigating, Cancelled)
                | (Contained, Eradicated)
                | (Contained, Recovered)
                | (Contained, Cancelled)
                | (Eradicated, Recovered)
                | (Eradicated, Cancelled)
                | (Recovered, Closed)
                | (Recovered, Investigating)
                | (Closed, Investigating)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IncidentClassification {
    Confidentiality,
    Integrity,
    Availability,
    Privacy,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IncidentSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DetectionSource {
    Manual,
    Finding(FindingRef),
    Alert(AlertRef),
    AssuranceEvent(EventRef),
    External(ExternalIncidentRef),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalIncidentRef {
    pub system: String,
    pub external_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimelineKind {
    Detected,
    Declared,
    StatusTransition,
    Contained,
    Eradicated,
    Recovered,
    Communicated,
    EvidenceAttached,
    ReviewRecorded,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentTimelineEvent {
    pub at: DateTime<Utc>,
    pub kind: TimelineKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IncidentEventKind {
    Declared,
    FieldsRevised,
    StatusTransition {
        from: IncidentStatus,
        to: IncidentStatus,
    },
    ReviewRecorded,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentEvent {
    pub version: u32,
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub kind: IncidentEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentContainment {
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub recovered_in_place: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRecord {
    pub at: DateTime<Utc>,
    pub channel: String,
    pub audience: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlFailureRef {
    pub control_id: ControlId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<ControlTestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_ref: Option<EventRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostIncidentReview {
    pub recorded_at: DateTime<Utc>,
    pub recorded_by: PrincipalRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<String>,
    pub lessons_learned: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_risk_ids: Vec<RiskId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed_corrective_action_ids: Vec<RemediationRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IncidentError {
    #[error("illegal incident status transition {from:?} → {to:?} on {id}")]
    IllegalTransition {
        id: IncidentId,
        from: IncidentStatus,
        to: IncidentStatus,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Incident {
    pub id: IncidentId,
    pub kind: IncidentKind,
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<IncidentClassification>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<IncidentSeverity>,
    #[serde(default)]
    pub status: IncidentStatus,
    pub detection: DetectionSource,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_refs: Vec<ExternalIncidentRef>,
    pub declared_at: DateTime<Utc>,
    pub declared_by: PrincipalRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_owner: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asset_ids: Vec<AssetId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub processing_activity_ids: Vec<ProcessingActivityId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub population: Vec<SubjectSelector>,
    #[serde(default)]
    pub timeline: Vec<IncidentTimelineEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub containment: Option<IncidentContainment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub eradication_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recovery_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub communications: Vec<NotificationRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lessons_learned: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_incident_review: Option<PostIncidentReview>,
    #[serde(default)]
    pub control_failure_refs: Vec<ControlFailureRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_ids: Vec<RiskId>,
    #[serde(default, rename = "correctiveActionIds")]
    pub corrective_action_ids: Vec<RemediationRef>,
    #[serde(default = "incident_version_default")]
    pub version: u32,
    #[serde(default)]
    pub history: Vec<IncidentEvent>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub tags: BTreeSet<String>,
}

impl Incident {
    /// Only constructor that creates a management-system incident.
    pub fn declare(
        id: IncidentId,
        kind: IncidentKind,
        title: impl Into<String>,
        detection: DetectionSource,
        declared_at: DateTime<Utc>,
        declared_by: PrincipalRef,
    ) -> Self {
        let title = title.into();
        Self {
            id,
            kind,
            title,
            summary: String::new(),
            classification: None,
            severity: None,
            status: IncidentStatus::Declared,
            detection,
            external_refs: Vec::new(),
            declared_at,
            declared_by: declared_by.clone(),
            response_owner: None,
            asset_ids: Vec::new(),
            processing_activity_ids: Vec::new(),
            population: Vec::new(),
            timeline: vec![IncidentTimelineEvent {
                at: declared_at,
                kind: TimelineKind::Declared,
                principal: Some(declared_by.clone()),
                detail: None,
            }],
            containment: None,
            eradication_refs: Vec::new(),
            recovery_refs: Vec::new(),
            communications: Vec::new(),
            evidence_refs: Vec::new(),
            root_cause: None,
            lessons_learned: None,
            post_incident_review: None,
            control_failure_refs: Vec::new(),
            risk_ids: Vec::new(),
            corrective_action_ids: Vec::new(),
            version: 1,
            history: vec![IncidentEvent {
                version: 1,
                at: declared_at,
                principal: Some(declared_by),
                kind: IncidentEventKind::Declared,
            }],
            tags: BTreeSet::new(),
        }
    }

    /// Explicit promotion alias for [`Incident::declare`].
    pub fn promote(
        id: IncidentId,
        kind: IncidentKind,
        title: impl Into<String>,
        detection: DetectionSource,
        declared_at: DateTime<Utc>,
        declared_by: PrincipalRef,
    ) -> Self {
        Self::declare(id, kind, title, detection, declared_at, declared_by)
    }

    pub fn transition(
        &mut self,
        to: IncidentStatus,
        at: DateTime<Utc>,
        principal: PrincipalRef,
    ) -> Result<(), IncidentError> {
        if !IncidentStatus::can_transition(self.status, to) {
            return Err(IncidentError::IllegalTransition {
                id: self.id.clone(),
                from: self.status,
                to,
            });
        }
        let from = self.status;
        self.status = to;
        self.version = self.version.saturating_add(1);
        let timeline_kind = match to {
            IncidentStatus::Contained => TimelineKind::Contained,
            IncidentStatus::Eradicated => TimelineKind::Eradicated,
            IncidentStatus::Recovered => TimelineKind::Recovered,
            IncidentStatus::Cancelled => TimelineKind::StatusTransition,
            _ => TimelineKind::StatusTransition,
        };
        self.timeline.push(IncidentTimelineEvent {
            at,
            kind: timeline_kind,
            principal: Some(principal.clone()),
            detail: None,
        });
        let kind = if to == IncidentStatus::Cancelled {
            IncidentEventKind::Cancelled
        } else {
            IncidentEventKind::StatusTransition { from, to }
        };
        self.history.push(IncidentEvent {
            version: self.version,
            at,
            principal: Some(principal),
            kind,
        });
        Ok(())
    }

    pub fn revise(
        &mut self,
        at: DateTime<Utc>,
        principal: PrincipalRef,
    ) -> Result<(), IncidentError> {
        self.version = self.version.saturating_add(1);
        self.history.push(IncidentEvent {
            version: self.version,
            at,
            principal: Some(principal),
            kind: IncidentEventKind::FieldsRevised,
        });
        Ok(())
    }

    pub fn has_recovery_evidence(&self) -> bool {
        self.recovery_refs.iter().any(|r| !r.trim().is_empty())
            || self.containment.as_ref().is_some_and(|c| {
                c.recovered_in_place && c.evidence_refs.iter().any(|r| !r.trim().is_empty())
            })
            || (self
                .containment
                .as_ref()
                .is_some_and(|c| c.recovered_in_place)
                && self.eradication_refs.iter().any(|r| !r.trim().is_empty()))
    }

    pub(crate) fn validate_record(&self) -> Result<(), IrValidationError> {
        if self.history.is_empty() {
            return Err(msg(format!(
                "incident {} history is empty; declare must seed history",
                self.id
            )));
        }
        for window in self.timeline.windows(2) {
            if window[0].at > window[1].at {
                return Err(msg(format!("incident {} timeline is not ordered", self.id)));
            }
        }
        let declared_events: Vec<_> = self
            .timeline
            .iter()
            .filter(|e| e.kind == TimelineKind::Declared)
            .collect();
        if declared_events.is_empty() {
            return Err(msg(format!(
                "incident {} timeline is missing a Declared event",
                self.id
            )));
        }
        if declared_events.iter().any(|e| e.at != self.declared_at) {
            return Err(msg(format!(
                "incident {} declaredAt does not match Declared timeline event",
                self.id
            )));
        }
        for event in &self.timeline {
            if event.kind == TimelineKind::Detected && event.at > self.declared_at {
                return Err(msg(format!(
                    "incident {} timeline detected time is after declaredAt",
                    self.id
                )));
            }
        }
        let mut status = IncidentStatus::Declared;
        let mut saw_transition = false;
        for event in &self.history {
            match event.kind {
                IncidentEventKind::Declared => {
                    status = IncidentStatus::Declared;
                }
                IncidentEventKind::StatusTransition { from, to } => {
                    if !IncidentStatus::can_transition(from, to) {
                        return Err(msg(format!(
                            "incident {} illegal history transition {from:?} → {to:?}",
                            self.id
                        )));
                    }
                    if from != status {
                        return Err(msg(format!(
                            "incident {} history transition from {from:?} does not follow {status:?}",
                            self.id
                        )));
                    }
                    status = to;
                    saw_transition = true;
                }
                IncidentEventKind::Cancelled => {
                    if !IncidentStatus::can_transition(status, IncidentStatus::Cancelled) {
                        return Err(msg(format!(
                            "incident {} illegal history transition {status:?} → Cancelled",
                            self.id
                        )));
                    }
                    status = IncidentStatus::Cancelled;
                    saw_transition = true;
                }
                IncidentEventKind::FieldsRevised | IncidentEventKind::ReviewRecorded => {}
            }
        }
        if saw_transition && status != self.status {
            return Err(msg(format!(
                "incident {} status {:?} does not match last history transition {status:?}",
                self.id, self.status
            )));
        }
        validate_principal_shape(&self.declared_by, &self.id)?;
        if let Some(owner) = &self.response_owner {
            validate_principal_shape(owner, &self.id)?;
        }
        for event in &self.timeline {
            if let Some(principal) = &event.principal {
                validate_principal_shape(principal, &self.id)?;
            }
        }
        for event in &self.history {
            if let Some(principal) = &event.principal {
                validate_principal_shape(principal, &self.id)?;
            }
        }
        if let Some(containment) = &self.containment
            && let Some(principal) = &containment.principal
        {
            validate_principal_shape(principal, &self.id)?;
        }
        for note in &self.communications {
            if let Some(principal) = &note.principal {
                validate_principal_shape(principal, &self.id)?;
            }
        }
        if matches!(
            self.status,
            IncidentStatus::Recovered | IncidentStatus::Closed
        ) && self.kind == IncidentKind::Real
            && !self.has_recovery_evidence()
        {
            return Err(msg(format!(
                "incident {} real recovered/closed requires recovery evidence",
                self.id
            )));
        }
        if self.kind == IncidentKind::Real
            && self.status == IncidentStatus::Closed
            && self.post_incident_review.is_none()
        {
            return Err(msg(format!(
                "incident {} real closed is missing post-incident review",
                self.id
            )));
        }
        if let Some(pir) = &self.post_incident_review {
            if pir.lessons_learned.trim().is_empty() {
                return Err(msg(format!(
                    "incident {} post-incident review requires lessonsLearned",
                    self.id
                )));
            }
            validate_principal_shape(&pir.recorded_by, &self.id)?;
        }
        for reference in self
            .recovery_refs
            .iter()
            .chain(self.eradication_refs.iter())
            .chain(self.evidence_refs.iter())
        {
            if reference.trim().is_empty() {
                return Err(msg(format!(
                    "incident {} has an empty evidence reference",
                    self.id
                )));
            }
        }
        for ext in &self.external_refs {
            if ext.system.trim().is_empty() || ext.external_id.trim().is_empty() {
                return Err(msg(format!(
                    "incident {} externalRefs require system and externalId",
                    self.id
                )));
            }
        }
        if let DetectionSource::External(ext) = &self.detection
            && (ext.system.trim().is_empty() || ext.external_id.trim().is_empty())
        {
            return Err(msg(format!(
                "incident {} external detection requires system and externalId",
                self.id
            )));
        }
        Ok(())
    }
}

pub(crate) struct IncidentGraph<'a> {
    pub asset_ids: &'a BTreeSet<String>,
    pub identity_ids: &'a BTreeSet<String>,
    pub processing_activity_ids: &'a BTreeSet<String>,
    pub risk_ids: &'a BTreeSet<String>,
    pub control_ids: &'a BTreeSet<String>,
    pub test_ids: &'a BTreeSet<String>,
    pub remediation_ids: Option<&'a BTreeSet<String>>,
}

impl Incident {
    pub(crate) fn validate_graph(
        &self,
        graph: &IncidentGraph<'_>,
    ) -> Result<(), IrValidationError> {
        self.validate_record()?;
        validate_principal_identity(&self.declared_by, graph.identity_ids, &self.id)?;
        if let Some(owner) = &self.response_owner {
            validate_principal_identity(owner, graph.identity_ids, &self.id)?;
        }
        for event in &self.timeline {
            if let Some(principal) = &event.principal {
                validate_principal_identity(principal, graph.identity_ids, &self.id)?;
            }
        }
        for event in &self.history {
            if let Some(principal) = &event.principal {
                validate_principal_identity(principal, graph.identity_ids, &self.id)?;
            }
        }
        if let Some(containment) = &self.containment
            && let Some(principal) = &containment.principal
        {
            validate_principal_identity(principal, graph.identity_ids, &self.id)?;
        }
        for note in &self.communications {
            if let Some(principal) = &note.principal {
                validate_principal_identity(principal, graph.identity_ids, &self.id)?;
            }
        }
        if let Some(pir) = &self.post_incident_review {
            validate_principal_identity(&pir.recorded_by, graph.identity_ids, &self.id)?;
        }
        for asset in &self.asset_ids {
            if !graph.asset_ids.contains(asset.as_str()) {
                return Err(msg(format!(
                    "dangling asset reference {asset} on incident {}",
                    self.id
                )));
            }
        }
        for activity in &self.processing_activity_ids {
            if !graph.processing_activity_ids.contains(activity.as_str()) {
                return Err(msg(format!(
                    "dangling processing activity {activity} on incident {}",
                    self.id
                )));
            }
        }
        for selector in &self.population {
            if matches!(
                selector.kind,
                SubjectKind::Identity | SubjectKind::User | SubjectKind::PrivilegedIdentity
            ) {
                for id in &selector.ids {
                    if !graph.identity_ids.contains(id.as_str()) {
                        return Err(msg(format!(
                            "dangling identity owner {id} on incident {}",
                            self.id
                        )));
                    }
                    IdentityId::try_new(id.as_str()).map_err(|e| {
                        msg(format!(
                            "invalid population identity {id} on incident {}: {e}",
                            self.id
                        ))
                    })?;
                }
            }
        }
        for risk in &self.risk_ids {
            if !graph.risk_ids.contains(risk.as_str()) {
                return Err(msg(format!(
                    "dangling risk reference {risk} on incident {}",
                    self.id
                )));
            }
        }
        for failure in &self.control_failure_refs {
            if !graph.control_ids.contains(failure.control_id.as_str()) {
                return Err(msg(format!(
                    "dangling control reference {} on incident {}",
                    failure.control_id, self.id
                )));
            }
            if let Some(test_id) = &failure.test_id
                && !graph.test_ids.contains(test_id.as_str())
            {
                return Err(msg(format!(
                    "dangling test reference {test_id} on incident {}",
                    self.id
                )));
            }
        }
        if let Some(remediation_ids) = graph.remediation_ids {
            for action in &self.corrective_action_ids {
                if !remediation_ids.contains(action.as_str()) {
                    return Err(msg(format!(
                        "dangling corrective action {action} on incident {}",
                        self.id
                    )));
                }
            }
        }
        Ok(())
    }
}

fn validate_principal_shape(
    principal: &PrincipalRef,
    incident_id: &IncidentId,
) -> Result<(), IrValidationError> {
    match principal {
        PrincipalRef::Identity(_) => Ok(()),
        PrincipalRef::Team(name) | PrincipalRef::Role(name) => {
            if name.trim().is_empty() {
                Err(msg(format!("empty principal on incident {incident_id}")))
            } else {
                Ok(())
            }
        }
    }
}

fn validate_principal_identity(
    principal: &PrincipalRef,
    identity_ids: &BTreeSet<String>,
    incident_id: &IncidentId,
) -> Result<(), IrValidationError> {
    if let PrincipalRef::Identity(id) = principal
        && !identity_ids.contains(id.as_str())
    {
        return Err(msg(format!(
            "dangling identity owner {id} on incident {incident_id}"
        )));
    }
    Ok(())
}

fn msg(text: String) -> IrValidationError {
    IrValidationError::Message(text)
}
