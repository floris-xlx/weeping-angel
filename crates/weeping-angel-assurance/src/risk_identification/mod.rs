//! Deterministic risk-candidate identification. Network-free, no framework profile.
//!
//! Clustering is identical subject set + normalized scenario key. Category
//! disagreement after clustering uses `SuggestedRiskCategory::Other("mixed")`.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssetId, CandidateConfidence, CandidateStatus, CorrelationKey,
    DismissalId, DismissalRecord, FindingRef, ObservationIdentity, PrincipalRef, PromotionId,
    PromotionRecord, Risk, RiskCandidate, RiskCandidateId, RiskId, RiskSource, ScenarioProposal,
    ScoreSuggestion, SourceRef, SubjectKind, SubjectRef, SuggestedRiskCategory, canonical_digest,
    validate_stable_id,
};
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceObservation, looks_like_compliance_claim};

const SECURITY_FINDING: &str = "security_finding";
const CONFIDENTIALITY_VULN: &str = "confidentiality exposure via known vulnerability";
const INTEGRITY_AVAIL_VULN: &str = "integrity or availability failure via known vulnerability";

const TEMPORAL_FACT_KEYS: &[&str] = &[
    "collected_at",
    "collectedat",
    "timestamp",
    "observed_at",
    "observedat",
    "run_id",
    "runid",
    "collection_run_id",
    "collectionrunid",
    "time",
];

const CREDENTIAL_FACT_KEYS: &[&str] = &[
    "authorization",
    "token",
    "cookie",
    "password",
    "api_key",
    "apikey",
    "secret",
    "access_token",
    "refresh_token",
    "private_key",
];

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IdentificationError {
    #[error("stale evidence cannot be promoted")]
    StaleEvidence,
    #[error("candidate is not promotable")]
    NotPromotable,
    #[error("principal is required")]
    PrincipalRequired,
    #[error("rationale is required")]
    RationaleRequired,
    #[error("promotion or dismissal rationale is a compliance claim")]
    ComplianceClaim,
    #[error("dangling identity principal")]
    DanglingPrincipal,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IdentificationPolicy {
    pub max_evidence_age_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct IdentificationContext<'a> {
    pub definition: &'a AssessmentDefinition,
    pub observations: &'a [EvidenceObservation],
    pub envelopes: &'a [EvidenceEnvelope],
    pub prior_candidates: &'a [RiskCandidate],
    pub dismissals: &'a [DismissalRecord],
    pub promotions: &'a [PromotionRecord],
    pub policy: IdentificationPolicy,
    pub as_of: DateTime<Utc>,
}

pub fn normalize_scenario(text: &str) -> String {
    let mut out = String::new();
    let mut last_space = false;
    for c in text.to_ascii_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            last_space = false;
        } else if c.is_whitespace() && !last_space && !out.is_empty() {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim().to_string()
}

pub fn observation_identity(observation: &EvidenceObservation) -> ObservationIdentity {
    let mut facts = BTreeMap::new();
    let finding = observation.evidence_type().as_str() == SECURITY_FINDING;
    for (key, value) in observation.facts() {
        let Some(text) = value.as_str() else {
            continue;
        };
        if skip_identity_fact(key) {
            continue;
        }
        if finding {
            let folded = key.to_ascii_lowercase();
            if !matches!(
                folded.as_str(),
                "rule_id" | "path" | "finding_id" | "category" | "canonical_type"
            ) {
                continue;
            }
        }
        facts.insert(key.clone(), text.to_string());
    }
    ObservationIdentity {
        evidence_type: observation.evidence_type().as_str().to_string(),
        facts,
        narrative_fingerprint: normalize_scenario(observation.narrative()),
    }
}

pub fn observation_identity_digest(identity: &ObservationIdentity) -> String {
    let digest = canonical_digest(identity).unwrap_or_else(|_| "0".repeat(16));
    format!("oi:sha256:{}", hex_prefix(&digest, 16))
}

pub fn subject_key(subjects: &[SubjectRef]) -> String {
    let mut rows: Vec<String> = subjects
        .iter()
        .map(|s| format!("{}:{}", subject_kind_token(s.kind), s.id))
        .collect();
    rows.sort();
    rows.dedup();
    rows.join("\n")
}

pub fn correlation_key(subjects: &[SubjectRef], scenario_key: &str) -> CorrelationKey {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body<'a> {
        subject_key: String,
        scenario_key: &'a str,
    }
    let digest = canonical_digest(&Body {
        subject_key: subject_key(subjects),
        scenario_key,
    })
    .unwrap_or_else(|_| "0".repeat(32));
    CorrelationKey::new(format!("ck:sha256:{}", hex_prefix(&digest, 32)))
}

pub fn correlate_candidates(proposals: Vec<RiskCandidate>) -> Vec<RiskCandidate> {
    let mut groups: BTreeMap<String, Vec<RiskCandidate>> = BTreeMap::new();
    for proposal in proposals {
        groups
            .entry(proposal.correlation_key.as_str().to_string())
            .or_default()
            .push(proposal);
    }

    let mut out = Vec::new();
    for (_, mut members) in groups {
        members.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        let mut survivor = members.remove(0);
        survivor.id = id_from_correlation(&survivor.correlation_key);
        survivor.status = CandidateStatus::Proposed;
        survivor.duplicate_candidate_ids.clear();

        for mut extra in members {
            extra.status = CandidateStatus::ClusteredDuplicate;
            extra.duplicate_candidate_ids.clear();
            extra.resulting_risk_id = None;
            extra.id = duplicate_id(&survivor.correlation_key, &extra.supporting_observations);
            union_candidate(&mut survivor, &extra);
            survivor.duplicate_candidate_ids.push(extra.id.clone());
            out.push(extra);
        }

        survivor.impacted_subjects.sort();
        survivor.impacted_subjects.dedup();
        survivor.supporting_observations.sort();
        survivor.supporting_observations.dedup();
        survivor.source_lineage.sort();
        survivor.source_lineage.dedup();
        survivor.duplicate_candidate_ids.sort();
        survivor.duplicate_candidate_ids.dedup();
        survivor.confidence = confidence(
            &survivor.supporting_observations,
            &survivor.impacted_subjects,
            // inventory resolution is applied later; treat current subjects as-is
            &survivor.impacted_subjects,
        );
        out.push(survivor);
    }
    out.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    out
}

pub fn identify_risk_candidates(ctx: &IdentificationContext<'_>) -> Vec<RiskCandidate> {
    let inventory = InventoryIndex::from_definition(ctx.definition);
    let mut proposals = Vec::new();
    for observation in ctx.observations {
        proposals.extend(map_observation(observation, ctx, &inventory));
    }
    let mut candidates = correlate_candidates(proposals);
    for candidate in &mut candidates {
        finish_candidate(candidate, ctx, &inventory);
    }
    candidates.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    candidates
}

pub fn should_resurface(cluster: &RiskCandidate, dismissal: &DismissalRecord) -> bool {
    let identities: BTreeSet<String> = cluster
        .supporting_observations
        .iter()
        .map(observation_identity_digest)
        .collect();
    if identities.is_empty() {
        return false;
    }
    if subject_key(&cluster.impacted_subjects) != dismissal.subject_key
        || cluster.scenario_proposal.scenario_key != dismissal.scenario_key
    {
        return false;
    }
    !identities.is_subset(&dismissal.observation_identities)
}

pub fn dismiss_candidate(
    candidate: &RiskCandidate,
    principal: PrincipalRef,
    at: DateTime<Utc>,
    rationale: impl Into<String>,
) -> Result<(RiskCandidate, DismissalRecord), IdentificationError> {
    if candidate.status == CandidateStatus::Promoted {
        return Err(IdentificationError::NotPromotable);
    }
    let rationale = require_rationale(rationale.into())?;
    require_principal_shape(&principal)?;
    let identities = candidate
        .supporting_observations
        .iter()
        .map(observation_identity_digest)
        .collect();
    let record = DismissalRecord {
        id: DismissalId::new(format!(
            "dismiss:{}",
            hex_prefix(
                &canonical_digest(&(candidate.id.as_str(), at.to_rfc3339(), rationale.as_str()))
                    .unwrap_or_else(|_| "0".repeat(16)),
                16
            )
        )),
        candidate_id: candidate.id.clone(),
        correlation_key: candidate.correlation_key.clone(),
        observation_identities: identities,
        subject_key: subject_key(&candidate.impacted_subjects),
        scenario_key: candidate.scenario_proposal.scenario_key.clone(),
        principal,
        at,
        rationale,
    };
    let mut dismissed = candidate.clone();
    dismissed.status = CandidateStatus::Dismissed;
    dismissed.resulting_risk_id = None;
    Ok((dismissed, record))
}

pub fn promote_candidate(
    definition: &mut AssessmentDefinition,
    candidate: &RiskCandidate,
    principal: PrincipalRef,
    at: DateTime<Utc>,
    rationale: impl Into<String>,
    methodology_inputs: Option<ScoreSuggestion>,
) -> Result<(RiskCandidate, Risk, PromotionRecord), IdentificationError> {
    if candidate.stale || candidate.status == CandidateStatus::Stale {
        return Err(IdentificationError::StaleEvidence);
    }
    if !matches!(
        candidate.status,
        CandidateStatus::Proposed | CandidateStatus::Resurfaced
    ) {
        return Err(IdentificationError::NotPromotable);
    }
    let rationale = require_rationale(rationale.into())?;
    validate_principal(definition, &principal)?;

    let risk_id = assign_risk_id(candidate, at);
    let title = candidate.scenario_proposal.title.clone();
    let narrative = candidate.scenario_proposal.narrative.clone();
    let mut risk = Risk::new(risk_id.clone(), title, narrative.clone());
    risk.scenario = Some(narrative);
    if !candidate.supporting_observations.is_empty() {
        risk.source = Some(RiskSource::Finding);
    }
    risk.finding_refs = candidate
        .supporting_observations
        .iter()
        .map(|oi| FindingRef::new(observation_identity_digest(oi)))
        .collect();
    risk.asset_ids = candidate
        .impacted_subjects
        .iter()
        .filter(|s| s.kind == SubjectKind::Asset)
        .filter_map(|s| AssetId::try_new(s.id.clone()).ok())
        .collect();
    risk.owner = Some(principal.clone());
    risk.discovered_at = Some(at);
    if let Some(suggestion) = &methodology_inputs
        && let Some(version) = &suggestion.methodology_version
    {
        risk.methodology_version = Some(version.clone());
    }

    definition.risks.push(risk.clone());

    let record = PromotionRecord {
        id: PromotionId::new(format!(
            "promo:{}",
            hex_prefix(
                &canonical_digest(&(candidate.id.as_str(), risk_id.as_str(), at.to_rfc3339()))
                    .unwrap_or_else(|_| "0".repeat(16)),
                16
            )
        )),
        candidate_id: candidate.id.clone(),
        correlation_key: candidate.correlation_key.clone(),
        risk_id: risk_id.clone(),
        principal,
        at,
        rationale,
        methodology_version: methodology_inputs
            .as_ref()
            .and_then(|s| s.methodology_version.clone()),
        methodology_inputs,
    };

    let mut promoted = candidate.clone();
    promoted.status = CandidateStatus::Promoted;
    promoted.resulting_risk_id = Some(risk_id);
    Ok((promoted, risk, record))
}

struct InventoryIndex<'a> {
    assets: BTreeMap<&'a str, &'a weeping_angel_assurance_ir::Asset>,
    identities: BTreeSet<&'a str>,
    vendors: BTreeSet<&'a str>,
    activities: BTreeSet<&'a str>,
}

impl<'a> InventoryIndex<'a> {
    fn from_definition(definition: &'a AssessmentDefinition) -> Self {
        Self {
            assets: definition
                .assets
                .iter()
                .map(|asset| (asset.id.as_str(), asset))
                .collect(),
            identities: definition
                .identities
                .iter()
                .map(|id| id.id.as_str())
                .collect(),
            vendors: definition.vendors.iter().map(|v| v.id.as_str()).collect(),
            activities: definition
                .processing_activities
                .iter()
                .map(|a| a.id.as_str())
                .collect(),
        }
    }

    fn contains_subject(&self, subject: &SubjectRef) -> bool {
        match subject.kind {
            SubjectKind::Asset
            | SubjectKind::Repository
            | SubjectKind::Service
            | SubjectKind::Application
            | SubjectKind::Database
            | SubjectKind::CloudAccount
            | SubjectKind::CloudResource
            | SubjectKind::Device
            | SubjectKind::Network
            | SubjectKind::Dataset
            | SubjectKind::Endpoint
            | SubjectKind::Branch
            | SubjectKind::Deployment => self.assets.contains_key(subject.id.as_str()),
            SubjectKind::Identity
            | SubjectKind::User
            | SubjectKind::PrivilegedIdentity
            | SubjectKind::ServiceAccount => self.identities.contains(subject.id.as_str()),
            SubjectKind::Vendor => self.vendors.contains(subject.id.as_str()),
            SubjectKind::ProcessingActivity => self.activities.contains(subject.id.as_str()),
            _ => false,
        }
    }
}

fn map_observation(
    observation: &EvidenceObservation,
    ctx: &IdentificationContext<'_>,
    inventory: &InventoryIndex<'_>,
) -> Vec<RiskCandidate> {
    if observation.evidence_type().as_str() != SECURITY_FINDING {
        return Vec::new();
    }
    if looks_like_compliance_claim(observation.narrative()) {
        return Vec::new();
    }
    let identity = observation_identity(observation);
    let envelope = matching_envelope(observation, ctx.envelopes);
    let subject = resolve_subject(observation, envelope, inventory);
    let mut subjects = Vec::new();
    if let Some(subject) = subject {
        subjects.push(subject);
    }
    let source = source_ref(observation, envelope);
    let mut out = Vec::new();
    for (scenario_key, title, category) in scenario_proposals(observation, envelope, ctx) {
        if looks_like_compliance_claim(&title) {
            continue;
        }
        let narrative = observation.narrative().to_string();
        if looks_like_compliance_claim(&narrative) {
            continue;
        }
        let key = correlation_key(&subjects, &scenario_key);
        let mut candidate = RiskCandidate::new(
            id_from_correlation(&key),
            key,
            ScenarioProposal {
                scenario_key,
                title,
                narrative,
            },
            category,
        );
        candidate.impacted_subjects = subjects.clone();
        candidate.supporting_observations = vec![identity.clone()];
        if let Some(source) = source.clone() {
            candidate.source_lineage = vec![source];
        }
        out.push(candidate);
    }
    out
}

fn scenario_proposals(
    observation: &EvidenceObservation,
    envelope: Option<&EvidenceEnvelope>,
    ctx: &IdentificationContext<'_>,
) -> Vec<(String, String, SuggestedRiskCategory)> {
    let canonical = observation.fact("canonical_type").unwrap_or("");
    if canonical == "security.vulnerability.present" && dual_scenario(envelope, ctx) {
        return vec![
            (
                CONFIDENTIALITY_VULN.to_string(),
                CONFIDENTIALITY_VULN.to_string(),
                SuggestedRiskCategory::Confidentiality,
            ),
            (
                INTEGRITY_AVAIL_VULN.to_string(),
                INTEGRITY_AVAIL_VULN.to_string(),
                SuggestedRiskCategory::Integrity,
            ),
        ];
    }
    let raw = if !canonical.is_empty() {
        canonical
    } else {
        observation.fact("category").unwrap_or("security.finding")
    };
    let scenario_key = normalize_scenario(raw);
    let title = if observation.narrative().trim().is_empty() {
        scenario_key.clone()
    } else {
        observation.narrative().to_string()
    };
    vec![(scenario_key, title, suggested_category(canonical))]
}

fn dual_scenario(envelope: Option<&EvidenceEnvelope>, ctx: &IdentificationContext<'_>) -> bool {
    let Some(envelope) = envelope else {
        return false;
    };
    let asset_id = envelope.provenance().asset.as_str();
    let Some(asset) = ctx
        .definition
        .assets
        .iter()
        .find(|asset| asset.id.as_str() == asset_id)
    else {
        return false;
    };
    if asset.kind != weeping_angel_assurance_ir::AssetKind::Service {
        return false;
    }
    ctx.definition.processing_activities.iter().any(|activity| {
        activity
            .systems
            .iter()
            .any(|system| system.as_str() == asset_id)
    })
}

fn suggested_category(canonical_type: &str) -> SuggestedRiskCategory {
    match canonical_type {
        "security.secret.exposure" => SuggestedRiskCategory::Confidentiality,
        "security.authz.weakness" => SuggestedRiskCategory::Identity,
        "security.vulnerability.present" => SuggestedRiskCategory::Vulnerability,
        other if other.starts_with("security.supplier") => SuggestedRiskCategory::Supplier,
        _ => SuggestedRiskCategory::Vulnerability,
    }
}

fn resolve_subject(
    observation: &EvidenceObservation,
    envelope: Option<&EvidenceEnvelope>,
    inventory: &InventoryIndex<'_>,
) -> Option<SubjectRef> {
    if let Some(envelope) = envelope {
        let id = envelope.provenance().asset.as_str();
        if inventory.assets.contains_key(id) {
            return Some(SubjectRef {
                kind: SubjectKind::Asset,
                id: id.to_string(),
            });
        }
        if validate_stable_id(id).is_ok() {
            return Some(SubjectRef {
                kind: SubjectKind::Asset,
                id: id.to_string(),
            });
        }
    }
    if let Some(path) = observation.fact("path") {
        if inventory.assets.contains_key(path) {
            return Some(SubjectRef {
                kind: SubjectKind::Asset,
                id: path.to_string(),
            });
        }
        if validate_stable_id(path).is_ok() {
            return Some(SubjectRef {
                kind: SubjectKind::Asset,
                id: path.to_string(),
            });
        }
    }
    None
}

fn matching_envelope<'a>(
    observation: &EvidenceObservation,
    envelopes: &'a [EvidenceEnvelope],
) -> Option<&'a EvidenceEnvelope> {
    let wanted = observation_identity(observation);
    envelopes
        .iter()
        .find(|env| observation_identity(env.observation()) == wanted)
}

fn source_ref(
    observation: &EvidenceObservation,
    envelope: Option<&EvidenceEnvelope>,
) -> Option<SourceRef> {
    Some(SourceRef {
        evidence_type: observation.evidence_type().as_str().to_string(),
        envelope_digest: envelope.map(|e| e.digest().to_string()),
        collection_run_id: envelope.map(|e| e.collection_run_id().to_string()),
        collector_id: envelope.map(|e| e.provenance().collector_id.clone()),
    })
}

fn finish_candidate(
    candidate: &mut RiskCandidate,
    ctx: &IdentificationContext<'_>,
    inventory: &InventoryIndex<'_>,
) {
    let resolved: Vec<SubjectRef> = candidate
        .impacted_subjects
        .iter()
        .filter(|s| inventory.contains_subject(s))
        .cloned()
        .collect();
    candidate.confidence = confidence(
        &candidate.supporting_observations,
        &candidate.impacted_subjects,
        &resolved,
    );
    let ages = observation_ages(candidate, ctx);
    candidate.stale = cluster_is_stale(&ages, ctx);
    if let Some((first, last)) = seen_range(&ages) {
        candidate.first_seen_at = Some(first);
        candidate.last_seen_at = Some(last);
    }
    candidate.matches_existing_risk_ids = overlap_existing(candidate, ctx.definition);

    if candidate.status == CandidateStatus::ClusteredDuplicate {
        return;
    }

    if let Some(promo) = ctx
        .promotions
        .iter()
        .find(|p| p.correlation_key.as_str() == candidate.correlation_key.as_str())
    {
        candidate.status = CandidateStatus::Promoted;
        candidate.resulting_risk_id = Some(promo.risk_id.clone());
        candidate.id = promo.candidate_id.clone();
        return;
    }

    if let Some(prior) = ctx
        .prior_candidates
        .iter()
        .find(|p| p.correlation_key.as_str() == candidate.correlation_key.as_str())
    {
        candidate.id = prior.id.clone();
        if prior.status == CandidateStatus::Promoted {
            candidate.status = CandidateStatus::Promoted;
            candidate.resulting_risk_id = prior.resulting_risk_id.clone();
            return;
        }
    }

    if let Some(dismissal) = ctx
        .dismissals
        .iter()
        .find(|d| d.correlation_key.as_str() == candidate.correlation_key.as_str())
    {
        candidate.id = dismissal.candidate_id.clone();
        if should_resurface(candidate, dismissal) {
            candidate.status = CandidateStatus::Resurfaced;
        } else {
            candidate.status = CandidateStatus::Dismissed;
        }
        return;
    }

    if candidate.stale {
        candidate.status = CandidateStatus::Stale;
    }
}

fn observation_ages(
    candidate: &RiskCandidate,
    ctx: &IdentificationContext<'_>,
) -> Vec<(ObservationIdentity, Option<DateTime<Utc>>)> {
    candidate
        .supporting_observations
        .iter()
        .map(|identity| {
            let collected = ctx.envelopes.iter().find_map(|env| {
                if observation_identity(env.observation()) == *identity {
                    Some(env.provenance().collected_at)
                } else {
                    None
                }
            });
            (identity.clone(), collected)
        })
        .collect()
}

fn freshness_limit(ctx: &IdentificationContext<'_>) -> Option<u64> {
    ctx.definition
        .evidence_requirements
        .iter()
        .find(|req| req.evidence_type().as_str() == SECURITY_FINDING)
        .and_then(|req| req.freshness().map(|f| f.max_age_seconds))
        .or(ctx.policy.max_evidence_age_seconds)
}

fn observation_is_stale(
    collected_at: Option<DateTime<Utc>>,
    ctx: &IdentificationContext<'_>,
) -> bool {
    let Some(limit) = freshness_limit(ctx) else {
        return false;
    };
    match collected_at {
        None => true,
        Some(at) => {
            let age = ctx.as_of.signed_duration_since(at);
            age.num_seconds() > limit as i64
        }
    }
}

fn cluster_is_stale(
    ages: &[(ObservationIdentity, Option<DateTime<Utc>>)],
    ctx: &IdentificationContext<'_>,
) -> bool {
    !ages.is_empty() && ages.iter().all(|(_, at)| observation_is_stale(*at, ctx))
}

fn seen_range(
    ages: &[(ObservationIdentity, Option<DateTime<Utc>>)],
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let times: Vec<DateTime<Utc>> = ages.iter().filter_map(|(_, at)| *at).collect();
    let first = times.iter().min().copied()?;
    let last = times.iter().max().copied()?;
    Some((first, last))
}

fn overlap_existing(candidate: &RiskCandidate, definition: &AssessmentDefinition) -> Vec<RiskId> {
    let scenario = candidate.scenario_proposal.scenario_key.as_str();
    let cluster_assets: BTreeSet<&str> = candidate
        .impacted_subjects
        .iter()
        .filter(|s| s.kind == SubjectKind::Asset)
        .map(|s| s.id.as_str())
        .collect();
    let contributor_ids: BTreeSet<String> = candidate
        .supporting_observations
        .iter()
        .map(observation_identity_digest)
        .collect();
    let mut matches = Vec::new();
    for risk in &definition.risks {
        let finding_hit = risk
            .finding_refs
            .iter()
            .any(|fr| contributor_ids.contains(fr.as_str()));
        let risk_text = risk
            .scenario
            .as_deref()
            .unwrap_or(risk.description.as_str());
        let scenario_hit = normalize_scenario(risk_text) == scenario;
        let risk_assets: BTreeSet<&str> = risk.asset_ids.iter().map(|id| id.as_str()).collect();
        let assets_hit = !risk.asset_ids.is_empty() && risk_assets == cluster_assets;
        let stub_hit = risk.asset_ids.is_empty()
            && risk.finding_refs.is_empty()
            && normalize_scenario(&risk.description) == scenario;
        if finding_hit || (scenario_hit && assets_hit) || stub_hit {
            matches.push(risk.id.clone());
        }
    }
    matches
}

fn confidence(
    observations: &[ObservationIdentity],
    subjects: &[SubjectRef],
    resolved: &[SubjectRef],
) -> CandidateConfidence {
    let distinct: BTreeSet<String> = observations
        .iter()
        .map(observation_identity_digest)
        .collect();
    let all_resolved = !subjects.is_empty() && resolved.len() == subjects.len();
    if distinct.len() >= 2 && all_resolved {
        CandidateConfidence::High
    } else if distinct.len() == 1 && !resolved.is_empty() {
        CandidateConfidence::Medium
    } else {
        CandidateConfidence::Low
    }
}

fn union_candidate(survivor: &mut RiskCandidate, extra: &RiskCandidate) {
    for obs in &extra.supporting_observations {
        if !survivor.supporting_observations.contains(obs) {
            survivor.supporting_observations.push(obs.clone());
        }
    }
    for src in &extra.source_lineage {
        if !survivor.source_lineage.contains(src) {
            survivor.source_lineage.push(src.clone());
        }
    }
    for subject in &extra.impacted_subjects {
        if !survivor.impacted_subjects.contains(subject) {
            survivor.impacted_subjects.push(subject.clone());
        }
    }
    if survivor.suggested_risk_category != extra.suggested_risk_category {
        survivor.suggested_risk_category = SuggestedRiskCategory::mixed();
    }
}

fn id_from_correlation(key: &CorrelationKey) -> RiskCandidateId {
    let hex = key.as_str().rsplit(':').next().unwrap_or("0");
    RiskCandidateId::new(format!("rc:{hex}"))
}

fn duplicate_id(key: &CorrelationKey, observations: &[ObservationIdentity]) -> RiskCandidateId {
    let ois: Vec<String> = observations
        .iter()
        .map(observation_identity_digest)
        .collect();
    let digest = canonical_digest(&(key.as_str(), ois)).unwrap_or_else(|_| "0".repeat(16));
    RiskCandidateId::new(format!("rc:dup:{}", hex_prefix(&digest, 16)))
}

fn assign_risk_id(candidate: &RiskCandidate, at: DateTime<Utc>) -> RiskId {
    let digest = canonical_digest(&(
        candidate.id.as_str(),
        candidate.correlation_key.as_str(),
        at.to_rfc3339(),
    ))
    .unwrap_or_else(|_| "0".repeat(16));
    RiskId::new(format!("risk:{}", hex_prefix(&digest, 20)))
}

fn require_rationale(rationale: String) -> Result<String, IdentificationError> {
    if rationale.trim().is_empty() {
        return Err(IdentificationError::RationaleRequired);
    }
    if looks_like_compliance_claim(&rationale) {
        return Err(IdentificationError::ComplianceClaim);
    }
    Ok(rationale)
}

fn require_principal_shape(principal: &PrincipalRef) -> Result<(), IdentificationError> {
    match principal {
        PrincipalRef::Identity(_) => Ok(()),
        PrincipalRef::Team(name) | PrincipalRef::Role(name) => {
            if name.trim().is_empty() {
                Err(IdentificationError::PrincipalRequired)
            } else {
                Ok(())
            }
        }
    }
}

fn validate_principal(
    definition: &AssessmentDefinition,
    principal: &PrincipalRef,
) -> Result<(), IdentificationError> {
    require_principal_shape(principal)?;
    match principal {
        PrincipalRef::Identity(id) => {
            if definition
                .identities
                .iter()
                .any(|known| known.id.as_str() == id.as_str())
            {
                Ok(())
            } else {
                Err(IdentificationError::DanglingPrincipal)
            }
        }
        PrincipalRef::Team(_) | PrincipalRef::Role(_) => Ok(()),
    }
}

fn skip_identity_fact(key: &str) -> bool {
    let folded = key.to_ascii_lowercase().replace('-', "_");
    TEMPORAL_FACT_KEYS.contains(&folded.as_str()) || CREDENTIAL_FACT_KEYS.contains(&folded.as_str())
}

fn subject_kind_token(kind: SubjectKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| format!("{kind:?}").to_ascii_lowercase())
}

fn hex_prefix(digest: &str, n: usize) -> &str {
    let end = n.min(digest.len());
    &digest[..end]
}
