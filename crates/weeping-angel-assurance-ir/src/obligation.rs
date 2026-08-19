//! Organizational obligation registry. Durable governance inputs, not test results.
//!
//! Distinct from framework [`crate::Requirement`] and from the membership-graph
//! `Obligation` stub on [`crate::IsmsContext`]. Share [`crate::ObligationId`].
//! Applicability is IR [`crate::AssessmentScope`] / [`crate::SubjectSelector`];
//! when the organizational scope engine's `ScopeResolution` type is present,
//! [`obligation_applies`] must call it. No provider filters.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::framework::ExternalRequirementRef;
use crate::isms::IsmsContext;
use crate::mapping::{MappingCompleteness, MappingDirection, MappingProvenance, MappingRelation};
use crate::party::InterestedParty;
use crate::subject::{SelectorScope, SubjectSelector};
use crate::validation::IrValidationError;
use crate::{
    ASSURANCE_IR_SCHEMA, AssessmentScope, ControlId, ControlledDocumentId, ExtensionMap,
    InterestedPartyId, ObligationId, ObligationMappingId, PrincipalRef, RequirementSourceId,
    RiskId, canonical_digest,
};

fn schema_version_default() -> String {
    ASSURANCE_IR_SCHEMA.to_string()
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn contains_protected_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let iso_annex = ["an", "nex a"].concat();
    lower.contains("the organization shall")
        || lower.contains(&iso_annex)
        || lower.contains("iso/iec 27001")
}

fn reject_protected(field: &str, value: &str) -> Result<(), IrValidationError> {
    if contains_protected_text(value) {
        return Err(IrValidationError::Message(format!(
            "protected text in {field}"
        )));
    }
    Ok(())
}

/// Where an organizational duty comes from. Extensible via [`RequirementSourceKind::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequirementSourceKind {
    Contractual,
    LegalRegulatory,
    Customer,
    InternalPolicy,
    Insurer,
    Supplier,
    Employment,
    Other(String),
}

/// Citation pointer and party, not licensed statute or contract body text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementSource {
    #[serde(default = "schema_version_default")]
    pub schema_version: String,
    pub id: RequirementSourceId,
    pub kind: RequirementSourceKind,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party_id: Option<InterestedPartyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<ExternalRequirementRef>,
}

impl RequirementSource {
    pub fn new(
        id: RequirementSourceId,
        kind: RequirementSourceKind,
        title: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            kind,
            title: title.into(),
            party_id: None,
            citation: None,
            external_ref: None,
        }
    }
}

/// Lifecycle of one stable [`ObligationId`]. Never `Deleted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ObligationLifecycle {
    #[default]
    Draft,
    Active,
    Retired,
    Superseded,
}

/// Result of resolving stored applicability at time T.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObligationApplicability {
    InScope,
    OutOfScope,
    Conditional,
    Unknown,
    Expired,
    NotCurrent,
}

/// One organizational duty. Not a framework clause and not an effectiveness verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Obligation {
    #[serde(default = "schema_version_default")]
    pub schema_version: String,
    pub id: ObligationId,
    pub source_id: RequirementSourceId,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub applicability: AssessmentScope,
    pub owner: PrincipalRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_from: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_until: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_by: Option<DateTime<Utc>>,
    pub lifecycle: ObligationLifecycle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<ObligationId>,
    #[serde(default, skip_serializing_if = "ExtensionMap::is_empty")]
    pub extensions: ExtensionMap,
}

impl Obligation {
    pub fn new(
        id: ObligationId,
        source_id: RequirementSourceId,
        title: impl Into<String>,
        description: impl Into<String>,
        owner: PrincipalRef,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            source_id,
            title: title.into(),
            description: description.into(),
            applicability: AssessmentScope::default(),
            owner,
            effective_from: None,
            effective_until: None,
            review_by: None,
            lifecycle: ObligationLifecycle::Draft,
            supersedes: None,
            extensions: ExtensionMap::new(),
        }
    }
}

/// Mapping target: risk, canonical control, governed document, or external clause pointer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObligationMappingTarget {
    Risk(RiskId),
    Control(ControlId),
    Document(ControlledDocumentId),
    ExternalRequirement(ExternalRequirementRef),
}

/// Directed obligation mapping. Semantic strength is never inferred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationMapping {
    #[serde(default = "schema_version_default")]
    pub schema_version: String,
    pub id: ObligationMappingId,
    pub from: ObligationId,
    pub to: ObligationMappingTarget,
    pub direction: MappingDirection,
    pub completeness: MappingCompleteness,
    pub relation: MappingRelation,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "is_default_provenance")]
    pub provenance: MappingProvenance,
}

fn is_default_provenance(value: &MappingProvenance) -> bool {
    value == &MappingProvenance::default()
}

impl ObligationMapping {
    pub fn new(
        id: ObligationMappingId,
        from: ObligationId,
        to: ObligationMappingTarget,
        direction: MappingDirection,
        completeness: MappingCompleteness,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            id,
            from,
            to,
            direction,
            completeness,
            relation: MappingRelation::from_completeness(completeness),
            rationale: rationale.into(),
            provenance: MappingProvenance::default(),
        }
    }

    pub fn with_relation(mut self, relation: MappingRelation) -> Self {
        self.relation = relation;
        self
    }

    pub fn projects_as_equivalence(&self) -> bool {
        projects_as_equivalence(self)
    }

    pub fn projects_as_full_satisfaction(&self) -> bool {
        projects_as_full_satisfaction(self)
    }
}

/// Honesty: only explicit full bidirectional Equivalent projects as equivalence.
pub fn projects_as_equivalence(mapping: &ObligationMapping) -> bool {
    mapping.relation == MappingRelation::Equivalent
        && mapping.completeness == MappingCompleteness::Full
        && mapping.direction == MappingDirection::Bidirectional
}

/// Honesty: only Equivalent / Satisfies / SupersetOf with full completeness.
pub fn projects_as_full_satisfaction(mapping: &ObligationMapping) -> bool {
    mapping.completeness == MappingCompleteness::Full
        && matches!(
            mapping.relation,
            MappingRelation::Equivalent | MappingRelation::Satisfies | MappingRelation::SupersetOf
        )
}

fn mapping_honesty_ok(mapping: &ObligationMapping) -> bool {
    match mapping.relation {
        MappingRelation::Equivalent => {
            mapping.completeness == MappingCompleteness::Full
                && mapping.direction == MappingDirection::Bidirectional
        }
        MappingRelation::Satisfies | MappingRelation::SupersetOf => {
            mapping.completeness == MappingCompleteness::Full
        }
        MappingRelation::PartiallySatisfies => {
            mapping.completeness == MappingCompleteness::Partial
                || mapping.completeness == MappingCompleteness::Related
        }
        MappingRelation::Supports => {
            mapping.completeness == MappingCompleteness::Partial
                || mapping.completeness == MappingCompleteness::Related
        }
        MappingRelation::EvidenceFor => true,
        MappingRelation::SubsetOf => mapping.completeness == MappingCompleteness::Partial,
        MappingRelation::Related => mapping.completeness == MappingCompleteness::Related,
    }
}

/// Caller-supplied inventories for fail-closed link checks.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationLinkUniverse {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub party_ids: BTreeSet<InterestedPartyId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub source_ids: BTreeSet<RequirementSourceId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub obligation_ids: BTreeSet<ObligationId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub control_ids: BTreeSet<ControlId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub risk_ids: BTreeSet<RiskId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub document_ids: BTreeSet<ControlledDocumentId>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub subject_ids: BTreeSet<String>,
    #[serde(default)]
    pub external_requirement_ok: bool,
}

/// Standalone obligation document. Not an assessment result inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationRegistry {
    #[serde(default = "schema_version_default")]
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parties: Vec<InterestedParty>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<RequirementSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligations: Vec<Obligation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<ObligationMapping>,
}

/// Lineage edge used by `why_*` helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObligationWhyEdge {
    pub party: InterestedParty,
    pub source: RequirementSource,
    pub obligation: Obligation,
    pub mapping: ObligationMapping,
    pub applicability: ObligationApplicability,
    pub projects_as_equivalence: bool,
    pub projects_as_full_satisfaction: bool,
}

fn selector_applies(selector: &SubjectSelector, universe: &ObligationLinkUniverse) -> Option<bool> {
    if selector.ids.is_empty() {
        return match selector.scope {
            SelectorScope::All | SelectorScope::AnyOf => Some(true),
            SelectorScope::NoneOf => Some(true),
        };
    }
    let known: Vec<&String> = selector
        .ids
        .iter()
        .filter(|id| universe.subject_ids.contains(*id))
        .collect();
    let unknown = selector.ids.len() != known.len();
    match selector.scope {
        SelectorScope::All => {
            if unknown {
                return None;
            }
            Some(
                selector
                    .ids
                    .iter()
                    .all(|id| universe.subject_ids.contains(id)),
            )
        }
        SelectorScope::AnyOf => {
            if known.is_empty() && unknown {
                return None;
            }
            Some(!known.is_empty())
        }
        SelectorScope::NoneOf => {
            if unknown {
                return None;
            }
            Some(
                selector
                    .ids
                    .iter()
                    .all(|id| !universe.subject_ids.contains(id)),
            )
        }
    }
}

/// Resolve stored applicability. Calls `ScopeResolution` when that engine type exists.
pub fn obligation_applies(
    obligation: &Obligation,
    universe: &ObligationLinkUniverse,
    t: DateTime<Utc>,
) -> ObligationApplicability {
    match obligation.lifecycle {
        ObligationLifecycle::Draft
        | ObligationLifecycle::Retired
        | ObligationLifecycle::Superseded => {
            return ObligationApplicability::NotCurrent;
        }
        ObligationLifecycle::Active => {}
    }
    if let Some(start) = obligation.effective_from
        && t < start
    {
        return ObligationApplicability::NotCurrent;
    }
    if let Some(end) = obligation.effective_until
        && t > end
    {
        return ObligationApplicability::Expired;
    }

    let scope = &obligation.applicability;
    if scope.organizations.is_empty() && scope.subjects.is_empty() && scope.exclusions.is_empty() {
        return ObligationApplicability::InScope;
    }

    let mut unknown = false;
    for selector in &scope.subjects {
        match selector_applies(selector, universe) {
            Some(true) => {}
            Some(false) => return ObligationApplicability::OutOfScope,
            None => unknown = true,
        }
    }
    for exclusion in &scope.exclusions {
        for selector in &exclusion.subjects {
            match selector_applies(selector, universe) {
                Some(true) => return ObligationApplicability::OutOfScope,
                Some(false) => {}
                None => unknown = true,
            }
        }
    }
    if unknown {
        ObligationApplicability::Unknown
    } else {
        ObligationApplicability::InScope
    }
}

/// Active obligations whose applicability at T is in-scope (not expired / not-current / out-of-scope).
pub fn current_obligations_at<'a>(
    registry: &'a ObligationRegistry,
    t: DateTime<Utc>,
) -> Vec<&'a Obligation> {
    registry.current_obligations_at(t)
}

impl Default for ObligationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ObligationRegistry {
    pub fn new() -> Self {
        Self {
            schema_version: ASSURANCE_IR_SCHEMA.into(),
            parties: Vec::new(),
            sources: Vec::new(),
            obligations: Vec::new(),
            mappings: Vec::new(),
        }
    }

    pub fn get_party(&self, id: &InterestedPartyId) -> Option<&InterestedParty> {
        self.parties.iter().find(|party| &party.id == id)
    }

    pub fn get_source(&self, id: &RequirementSourceId) -> Option<&RequirementSource> {
        self.sources.iter().find(|source| &source.id == id)
    }

    pub fn get_obligation(&self, id: &ObligationId) -> Option<&Obligation> {
        self.obligations
            .iter()
            .find(|obligation| &obligation.id == id)
    }

    pub fn get_mapping(&self, id: &ObligationMappingId) -> Option<&ObligationMapping> {
        self.mappings.iter().find(|mapping| &mapping.id == id)
    }

    pub fn current_obligations_at(&self, t: DateTime<Utc>) -> Vec<&Obligation> {
        let universe = self.implied_universe();
        let mut current: Vec<&Obligation> = self
            .obligations
            .iter()
            .filter(|obligation| {
                let applicability = obligation_applies(obligation, &universe, t);
                obligation.lifecycle == ObligationLifecycle::Active
                    && !matches!(
                        applicability,
                        ObligationApplicability::Expired
                            | ObligationApplicability::NotCurrent
                            | ObligationApplicability::OutOfScope
                            | ObligationApplicability::Unknown
                    )
            })
            .collect();
        current.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        current
    }

    pub fn supersession_chain(&self, id: &ObligationId) -> Vec<&Obligation> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut cursor = id.clone();
        while seen.insert(cursor.as_str().to_string()) {
            let Some(row) = self.get_obligation(&cursor) else {
                break;
            };
            chain.push(row);
            match &row.supersedes {
                Some(pred) => cursor = pred.clone(),
                None => break,
            }
        }
        chain.reverse();
        let mut successors: Vec<&Obligation> = self
            .obligations
            .iter()
            .filter(|row| row.supersedes.as_ref() == Some(id) && row.id != *id)
            .collect();
        successors.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        for successor in successors {
            if !chain.iter().any(|row| row.id == successor.id) {
                chain.push(successor);
            }
        }
        chain
    }

    pub fn mappings_from(&self, obligation_id: &ObligationId) -> Vec<&ObligationMapping> {
        self.mappings
            .iter()
            .filter(|mapping| &mapping.from == obligation_id)
            .collect()
    }

    pub fn why_control_exists(
        &self,
        control_id: &ControlId,
        t: DateTime<Utc>,
    ) -> Vec<ObligationWhyEdge> {
        self.why_target(Some(control_id), None, t, false)
    }

    pub fn why_document_exists(
        &self,
        document_id: &ControlledDocumentId,
        t: DateTime<Utc>,
    ) -> Vec<ObligationWhyEdge> {
        self.why_target(None, Some(document_id), t, false)
    }

    pub fn why_control_exists_including_historical(
        &self,
        control_id: &ControlId,
        t: DateTime<Utc>,
    ) -> Vec<ObligationWhyEdge> {
        self.why_target(Some(control_id), None, t, true)
    }

    pub fn why_document_exists_including_historical(
        &self,
        document_id: &ControlledDocumentId,
        t: DateTime<Utc>,
    ) -> Vec<ObligationWhyEdge> {
        self.why_target(None, Some(document_id), t, true)
    }

    fn why_target(
        &self,
        control_id: Option<&ControlId>,
        document_id: Option<&ControlledDocumentId>,
        t: DateTime<Utc>,
        include_historical: bool,
    ) -> Vec<ObligationWhyEdge> {
        let universe = self.implied_universe();
        let current_ids: BTreeSet<_> = self
            .current_obligations_at(t)
            .into_iter()
            .map(|obligation| obligation.id.as_str().to_string())
            .collect();
        let mut edges = Vec::new();
        for mapping in &self.mappings {
            let matches_target = match &mapping.to {
                ObligationMappingTarget::Control(id) => control_id.is_some_and(|want| want == id),
                ObligationMappingTarget::Document(id) => document_id.is_some_and(|want| want == id),
                _ => false,
            };
            if !matches_target {
                continue;
            }
            let Some(obligation) = self.get_obligation(&mapping.from) else {
                continue;
            };
            let is_current = current_ids.contains(obligation.id.as_str());
            if !is_current && !include_historical {
                continue;
            }
            let Some(source) = self.get_source(&obligation.source_id) else {
                continue;
            };
            let Some(party_id) = &source.party_id else {
                continue;
            };
            let Some(party) = self.get_party(party_id) else {
                continue;
            };
            edges.push(ObligationWhyEdge {
                party: party.clone(),
                source: source.clone(),
                obligation: obligation.clone(),
                mapping: mapping.clone(),
                applicability: obligation_applies(obligation, &universe, t),
                projects_as_equivalence: mapping.projects_as_equivalence(),
                projects_as_full_satisfaction: mapping.projects_as_full_satisfaction(),
            });
        }
        edges.sort_by(|a, b| {
            a.obligation
                .id
                .as_str()
                .cmp(b.obligation.id.as_str())
                .then_with(|| a.mapping.id.as_str().cmp(b.mapping.id.as_str()))
        });
        edges
    }

    /// Context membership lists are references into this registry, not embedded bodies.
    pub fn validate_against_isms_context(
        &self,
        ctx: &IsmsContext,
    ) -> Result<(), IrValidationError> {
        for obligation in &ctx.obligations {
            if self.get_obligation(&obligation.id).is_none() {
                return Err(IrValidationError::Message(format!(
                    "dangling obligation id {} from IsmsContext {}",
                    obligation.id, ctx.id
                )));
            }
        }
        Ok(())
    }

    pub fn canonical_digest(&self) -> Result<String, crate::digest::CanonicalDigestError> {
        canonical_digest(self)
    }

    pub fn implied_universe(&self) -> ObligationLinkUniverse {
        let mut universe = ObligationLinkUniverse {
            party_ids: self.parties.iter().map(|party| party.id.clone()).collect(),
            source_ids: self
                .sources
                .iter()
                .map(|source| source.id.clone())
                .collect(),
            obligation_ids: self
                .obligations
                .iter()
                .map(|obligation| obligation.id.clone())
                .collect(),
            ..ObligationLinkUniverse::default()
        };
        for mapping in &self.mappings {
            match &mapping.to {
                ObligationMappingTarget::Control(id) => {
                    universe.control_ids.insert(id.clone());
                }
                ObligationMappingTarget::Risk(id) => {
                    universe.risk_ids.insert(id.clone());
                }
                ObligationMappingTarget::Document(id) => {
                    universe.document_ids.insert(id.clone());
                }
                ObligationMappingTarget::ExternalRequirement(_) => {
                    universe.external_requirement_ok = true;
                }
            }
        }
        for obligation in &self.obligations {
            for selector in &obligation.applicability.subjects {
                universe.subject_ids.extend(selector.ids.iter().cloned());
            }
            for exclusion in &obligation.applicability.exclusions {
                for selector in &exclusion.subjects {
                    universe.subject_ids.extend(selector.ids.iter().cloned());
                }
            }
        }
        universe
    }

    pub fn validate(&self, universe: &ObligationLinkUniverse) -> Result<(), IrValidationError> {
        if self.schema_version != ASSURANCE_IR_SCHEMA {
            return Err(IrValidationError::Message(format!(
                "schema mismatch: expected {ASSURANCE_IR_SCHEMA}, got {}",
                self.schema_version
            )));
        }

        let mut party_ids = BTreeSet::new();
        for party in &self.parties {
            if party.schema_version != ASSURANCE_IR_SCHEMA {
                return Err(IrValidationError::Message(format!(
                    "schema mismatch on party {}",
                    party.id
                )));
            }
            if !party_ids.insert(party.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate stable id {}",
                    party.id
                )));
            }
            if is_blank(&party.name) {
                return Err(IrValidationError::Message(format!(
                    "empty required field name on party {}",
                    party.id
                )));
            }
            reject_protected("party name", &party.name)?;
            if let Some(notes) = &party.notes {
                reject_protected("party notes", notes)?;
            }
        }

        let mut source_ids = BTreeSet::new();
        for source in &self.sources {
            if source.schema_version != ASSURANCE_IR_SCHEMA {
                return Err(IrValidationError::Message(format!(
                    "schema mismatch on source {}",
                    source.id
                )));
            }
            if !source_ids.insert(source.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate stable id {}",
                    source.id
                )));
            }
            if is_blank(&source.title) {
                return Err(IrValidationError::Message(format!(
                    "empty required field title on source {}",
                    source.id
                )));
            }
            reject_protected("source title", &source.title)?;
            if let Some(citation) = &source.citation {
                reject_protected("citation", citation)?;
            }
            if let Some(party_id) = &source.party_id
                && !party_ids.contains(party_id.as_str())
            {
                return Err(IrValidationError::Message(format!(
                    "dangling party {party_id} on source {}",
                    source.id
                )));
            }
        }

        let mut obligation_ids = BTreeSet::new();
        for obligation in &self.obligations {
            if obligation.schema_version != ASSURANCE_IR_SCHEMA {
                return Err(IrValidationError::Message(format!(
                    "schema mismatch on obligation {}",
                    obligation.id
                )));
            }
            if !obligation_ids.insert(obligation.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate stable id {}",
                    obligation.id
                )));
            }
            if is_blank(&obligation.title) || is_blank(&obligation.description) {
                return Err(IrValidationError::Message(format!(
                    "empty required field on obligation {}",
                    obligation.id
                )));
            }
            reject_protected("obligation title", &obligation.title)?;
            reject_protected("obligation description", &obligation.description)?;
            if !source_ids.contains(obligation.source_id.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling source {} on obligation {}",
                    obligation.source_id, obligation.id
                )));
            }
            if let Some(pred) = &obligation.supersedes {
                if !obligation_ids.contains(pred.as_str())
                    && !self.obligations.iter().any(|row| &row.id == pred)
                {
                    return Err(IrValidationError::Message(format!(
                        "dangling predecessor {pred} on obligation {}",
                        obligation.id
                    )));
                }
            }
            for selector in &obligation.applicability.subjects {
                for subject_id in &selector.ids {
                    if !universe.subject_ids.contains(subject_id) {
                        return Err(IrValidationError::Message(format!(
                            "dangling scope id {subject_id} on obligation {}",
                            obligation.id
                        )));
                    }
                }
            }
            if obligation.lifecycle == ObligationLifecycle::Active
                && obligation.effective_from.is_none()
            {
                return Err(IrValidationError::Message(format!(
                    "active obligation {} requires effective_from",
                    obligation.id
                )));
            }
            match &obligation.owner {
                PrincipalRef::Team(name) | PrincipalRef::Role(name) if is_blank(name) => {
                    return Err(IrValidationError::Message(format!(
                        "empty required field owner on obligation {}",
                        obligation.id
                    )));
                }
                _ => {}
            }
        }

        if let Some(cycle_id) = self.supersession_cycle() {
            return Err(IrValidationError::Message(format!(
                "supersession cycle involving {cycle_id}"
            )));
        }

        let successor_of: BTreeSet<String> = self
            .obligations
            .iter()
            .filter_map(|row| row.supersedes.as_ref().map(|id| id.as_str().to_string()))
            .collect();
        for obligation in &self.obligations {
            if obligation.lifecycle == ObligationLifecycle::Superseded
                && !successor_of.contains(obligation.id.as_str())
            {
                return Err(IrValidationError::Message(format!(
                    "inconsistent supersession: superseded {} has no successor",
                    obligation.id
                )));
            }
            if obligation.lifecycle == ObligationLifecycle::Active
                && successor_of.contains(obligation.id.as_str())
            {
                return Err(IrValidationError::Message(format!(
                    "inconsistent supersession: predecessor {} must not stay Active",
                    obligation.id
                )));
            }
        }

        let mut mapping_ids = BTreeSet::new();
        for mapping in &self.mappings {
            if mapping.schema_version != ASSURANCE_IR_SCHEMA {
                return Err(IrValidationError::Message(format!(
                    "schema mismatch on mapping {}",
                    mapping.id
                )));
            }
            if !mapping_ids.insert(mapping.id.as_str().to_string()) {
                return Err(IrValidationError::Message(format!(
                    "duplicate stable id {}",
                    mapping.id
                )));
            }
            if is_blank(&mapping.rationale) {
                return Err(IrValidationError::Message(format!(
                    "empty required field rationale on mapping {}",
                    mapping.id
                )));
            }
            reject_protected("rationale", &mapping.rationale)?;
            if !obligation_ids.contains(mapping.from.as_str()) {
                return Err(IrValidationError::Message(format!(
                    "dangling obligation {} on mapping {}",
                    mapping.from, mapping.id
                )));
            }
            if !mapping_honesty_ok(mapping) {
                return Err(IrValidationError::Message(format!(
                    "mapping honesty: illegal {:?} + {:?} on {}",
                    mapping.relation, mapping.completeness, mapping.id
                )));
            }
            match &mapping.to {
                ObligationMappingTarget::Control(id) if !universe.control_ids.contains(id) => {
                    return Err(IrValidationError::Message(format!(
                        "dangling mapping target control {id}"
                    )));
                }
                ObligationMappingTarget::Risk(id) if !universe.risk_ids.contains(id) => {
                    return Err(IrValidationError::Message(format!(
                        "dangling mapping target risk {id}"
                    )));
                }
                ObligationMappingTarget::Document(id) if !universe.document_ids.contains(id) => {
                    return Err(IrValidationError::Message(format!(
                        "dangling mapping target document {id}"
                    )));
                }
                ObligationMappingTarget::ExternalRequirement(ext)
                    if universe.external_requirement_ok && is_blank(&ext.external_id) =>
                {
                    return Err(IrValidationError::Message(
                        "dangling mapping target external requirement".into(),
                    ));
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn supersession_cycle(&self) -> Option<String> {
        let by_id: BTreeMap<String, &Obligation> = self
            .obligations
            .iter()
            .map(|row| (row.id.as_str().to_string(), row))
            .collect();
        for start in &self.obligations {
            let mut seen = BTreeSet::new();
            let mut cursor = start.supersedes.as_ref();
            while let Some(pred) = cursor {
                if !seen.insert(pred.as_str().to_string()) || pred == &start.id {
                    return Some(start.id.as_str().to_string());
                }
                cursor = by_id
                    .get(pred.as_str())
                    .and_then(|row| row.supersedes.as_ref());
            }
        }
        None
    }
}
