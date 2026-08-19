//! Minimal risk record. Not a risk engine.
//!
//! Additive operational register fields live on the same `assurance-ir/v1` type.
//! Inherent scoring is delegated to Prompt 05 or [`crate::risk_scoring`].

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::id::{FindingRef, RiskTreatmentId};
use crate::risk_scoring::MethodologyValue;
use crate::{
    AssetId, ControlId, PrincipalRef, ProcessingActivityId, RiskId, VendorId, validate_stable_id,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RiskStatus {
    Draft,
    #[default]
    Open,
    UnderTreatment,
    Accepted,
    Mitigated,
    Closed,
    Retired,
}

impl RiskStatus {
    pub fn can_transition(from: Self, to: Self) -> bool {
        use RiskStatus::*;
        matches!(
            (from, to),
            (Draft, Open | Retired)
                | (Open, UnderTreatment | Accepted | Retired)
                | (UnderTreatment, Open | Accepted | Mitigated | Retired)
                | (Accepted, Open | UnderTreatment | Closed | Retired)
                | (Mitigated, Open | UnderTreatment | Closed | Retired)
                | (Closed, Open | Retired)
        )
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Closed | Self::Retired)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RiskSource {
    Manual,
    Finding,
    Incident,
    Assessment,
    Supplier,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCadence {
    pub interval_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CiaImpactInputs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidentiality: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub availability: Option<u32>,
}

impl CiaImpactInputs {
    pub fn is_empty(&self) -> bool {
        self.confidentiality.is_none() && self.integrity.is_none() && self.availability.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RiskEventKind {
    Created,
    FieldsRevised,
    StatusTransition { from: RiskStatus, to: RiskStatus },
    Superseded { successor: RiskId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskEvent {
    pub version: u32,
    pub at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<PrincipalRef>,
    pub kind: RiskEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RiskTransitionError {
    #[error("illegal risk status transition from {from:?} to {to:?}")]
    Illegal { from: RiskStatus, to: RiskStatus },
}

fn omit_unpinned_version(pin: &VersionPin) -> bool {
    pin.n == 1 && !pin.explicit
}

/// Monotonic risk revision. Default 1 is omitted unless the document pinned it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionPin {
    n: u32,
    explicit: bool,
}

impl VersionPin {
    pub fn new(n: u32) -> Self {
        Self {
            n: n.max(1),
            explicit: n != 1,
        }
    }

    pub fn pinned(n: u32) -> Self {
        Self {
            n: n.max(1),
            explicit: true,
        }
    }

    pub fn get(self) -> u32 {
        self.n
    }
}

impl Default for VersionPin {
    fn default() -> Self {
        Self {
            n: 1,
            explicit: false,
        }
    }
}

impl Serialize for VersionPin {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.n)
    }
}

impl<'de> Deserialize<'de> for VersionPin {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let n = u32::deserialize(deserializer)?;
        Ok(Self::pinned(n))
    }
}

impl From<u32> for VersionPin {
    fn from(n: u32) -> Self {
        Self::pinned(n)
    }
}

impl PartialEq<u32> for VersionPin {
    fn eq(&self, other: &u32) -> bool {
        self.n == *other
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Risk {
    pub id: RiskId,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub status: RiskStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threat: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weakness_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asset_ids: Vec<AssetId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub processing_activity_ids: Vec<ProcessingActivityId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vendor_ids: Vec<VendorId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cia: Option<CiaImpactInputs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub likelihood: Option<MethodologyValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact: Option<MethodologyValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherent_score: Option<MethodologyValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherent_rating: Option<MethodologyValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residual_score: Option<MethodologyValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residual_rating: Option<MethodologyValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methodology_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PrincipalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<RiskSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discovered_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_cadence: Option<ReviewCadence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_review: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treatment_id: Option<RiskTreatmentId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub control_ids: Vec<ControlId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finding_refs: Vec<FindingRef>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub tags: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<String>,
    #[serde(default, skip_serializing_if = "omit_unpinned_version")]
    pub version: VersionPin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RiskId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<RiskId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<RiskEvent>,
}

impl Risk {
    pub fn new(id: RiskId, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            status: RiskStatus::Open,
            scenario: None,
            threat: None,
            weakness_refs: Vec::new(),
            asset_ids: Vec::new(),
            processing_activity_ids: Vec::new(),
            vendor_ids: Vec::new(),
            cia: None,
            likelihood: None,
            impact: None,
            inherent_score: None,
            inherent_rating: None,
            residual_score: None,
            residual_rating: None,
            methodology_version: None,
            owner: None,
            source: None,
            discovered_at: None,
            review_cadence: None,
            next_review: None,
            treatment_id: None,
            control_ids: Vec::new(),
            evidence_refs: Vec::new(),
            finding_refs: Vec::new(),
            tags: BTreeSet::new(),
            classification: None,
            version: VersionPin::default(),
            supersedes: None,
            superseded_by: None,
            history: Vec::new(),
        }
    }

    pub fn with_scenario(mut self, scenario: impl Into<String>) -> Self {
        self.scenario = Some(scenario.into());
        self
    }

    pub fn with_owner(mut self, owner: PrincipalRef) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_assets(mut self, asset_ids: Vec<AssetId>) -> Self {
        self.asset_ids = asset_ids;
        self
    }

    pub fn with_controls(mut self, control_ids: Vec<ControlId>) -> Self {
        self.control_ids = control_ids;
        self
    }

    pub fn with_findings(mut self, finding_refs: Vec<FindingRef>) -> Self {
        self.finding_refs = finding_refs;
        self
    }

    pub fn with_methodology_inputs(
        mut self,
        methodology_version: impl Into<String>,
        likelihood: MethodologyValue,
        impact: MethodologyValue,
    ) -> Self {
        self.methodology_version = Some(methodology_version.into());
        self.likelihood = Some(likelihood);
        self.impact = Some(impact);
        self
    }

    pub fn with_review(mut self, cadence: ReviewCadence, next_review: DateTime<Utc>) -> Self {
        self.review_cadence = Some(cadence);
        self.next_review = Some(next_review);
        self
    }

    pub fn with_treatment(mut self, treatment_id: RiskTreatmentId) -> Self {
        self.treatment_id = Some(treatment_id);
        self
    }

    pub fn review_overdue(&self, as_of: DateTime<Utc>) -> bool {
        self.next_review.is_some_and(|next| next < as_of)
    }

    pub fn transition(&mut self, to: RiskStatus) -> Result<&mut Self, RiskTransitionError> {
        if !RiskStatus::can_transition(self.status, to) {
            return Err(RiskTransitionError::Illegal {
                from: self.status,
                to,
            });
        }
        let from = self.status;
        self.status = to;
        self.bump_version();
        self.history.push(RiskEvent {
            version: self.version.get(),
            at: Utc::now(),
            principal: None,
            kind: RiskEventKind::StatusTransition { from, to },
            previous: None,
        });
        Ok(self)
    }

    pub fn revise(&mut self, title: impl Into<String>) -> &mut Self {
        let previous = serde_json::json!({
            "title": self.title,
            "status": self.status,
            "inherentScore": self.inherent_score,
        });
        self.title = title.into();
        self.bump_version();
        self.history.push(RiskEvent {
            version: self.version.get(),
            at: Utc::now(),
            principal: None,
            kind: RiskEventKind::FieldsRevised,
            previous: Some(previous),
        });
        self
    }

    fn bump_version(&mut self) {
        self.version = VersionPin::pinned(self.version.get().saturating_add(1).max(2));
    }
}

pub(crate) fn evidence_ref_is_requirement(raw: &str) -> bool {
    raw.starts_with("evidence.")
}

pub(crate) fn evidence_ref_is_well_formed_digest(raw: &str) -> bool {
    !raw.trim().is_empty() && validate_stable_id(raw).is_ok()
}
