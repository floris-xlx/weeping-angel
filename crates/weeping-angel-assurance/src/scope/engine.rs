//! Pure deterministic ISMS boundary resolution over canonical inventories.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentDefinition, AssessmentScope, Asset, AssetKind, Identity,
    IdentityKind, IsmsContext, PrincipalRef, ProcessingActivity, ScopeExclusion, SelectorScope,
    SubjectKind, SubjectSelector, Vendor, canonical_digest,
};
use weeping_angel_control_test::PopulationCompleteness;

use super::snapshot::{
    InfluencingRule, InfluencingRuleClass, LineageHop, ScopeDecision, ScopeResolution,
    SubjectScopeDecision,
};

const RANK_EXACT: u16 = 100;
const RANK_TAG: u16 = 80;
const RANK_KIND: u16 = 60;
const RANK_INHERIT: u16 = 40;
const RANK_ORG: u16 = 30;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScopeError {
    #[error("scope conflict")]
    Conflict,
    #[error("parent cycle")]
    Cycle,
    #[error("invalid exclusion")]
    InvalidExclusion,
    #[error("unresolved subject")]
    Unresolved,
    #[error("schema version mismatch")]
    Schema,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubjectRef {
    pub kind: SubjectKind,
    pub id: String,
}

impl SubjectRef {
    pub fn new(kind: SubjectKind, id: impl Into<String>) -> Self {
        Self {
            kind,
            id: id.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScopeInputs<'a> {
    definition: &'a AssessmentDefinition,
    context: Option<&'a IsmsContext>,
    candidates: Option<Vec<SubjectRef>>,
}

impl<'a> ScopeInputs<'a> {
    pub fn from_assessment(definition: &'a AssessmentDefinition) -> Self {
        Self {
            definition,
            context: None,
            candidates: None,
        }
    }

    pub fn with_context(mut self, context: &'a IsmsContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_candidates(mut self, candidates: Vec<SubjectRef>) -> Self {
        self.candidates = Some(candidates);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InScopePopulation {
    pub ids: Vec<String>,
    pub completeness: PopulationCompleteness,
}

pub fn is_definitely_in_scope(decision: ScopeDecision) -> bool {
    decision == ScopeDecision::InScope
}

pub fn resolve_scope(
    input: &ScopeInputs<'_>,
    as_of: DateTime<Utc>,
) -> Result<ScopeResolution, ScopeError> {
    if input.definition.schema_version != ASSURANCE_IR_SCHEMA {
        return Err(ScopeError::Schema);
    }
    let mut engine = Engine::new(input, as_of);
    let candidates = engine.candidate_list();
    let mut subjects = Vec::with_capacity(candidates.len());
    for cand in candidates {
        subjects.push(engine.resolve_one(&cand));
    }
    let scope_id = input.context.map(|ctx| ctx.scope.id.clone());
    Ok(ScopeResolution::seal(
        input.definition.id.clone(),
        as_of,
        scope_id,
        subjects,
    ))
}

pub fn resolve_subject(
    subject: &SubjectRef,
    input: &ScopeInputs<'_>,
    as_of: DateTime<Utc>,
) -> SubjectScopeDecision {
    let mut scoped = input.clone();
    scoped.candidates = Some(vec![subject.clone()]);
    match resolve_scope(&scoped, as_of) {
        Ok(resolution) => resolution
            .subjects
            .into_iter()
            .find(|row| row.kind == subject.kind && row.id == subject.id)
            .unwrap_or_else(|| unknown_row(subject, "unresolved subject")),
        Err(_) => unknown_row(subject, "unresolved subject"),
    }
}

/// Population selection consults IR `AssessmentScope` inventories, not envelopes.
pub fn in_scope_population(
    selector: &SubjectSelector,
    resolution: &ScopeResolution,
    definition: &AssessmentDefinition,
) -> InScopePopulation {
    let scope: &AssessmentScope = &definition.scope;
    let mut ids = Vec::new();
    let mut saw_unknown = false;
    for row in &resolution.subjects {
        if !row_matches_selector(row, selector) {
            continue;
        }
        match row.decision {
            ScopeDecision::InScope => ids.push(row.id.clone()),
            ScopeDecision::Unknown => saw_unknown = true,
            ScopeDecision::OutOfScope | ScopeDecision::Conditional => {}
        }
    }
    ids.sort();
    ids.dedup();
    let explicit_family = scope
        .subjects
        .iter()
        .any(|s| kinds_compatible(s.kind, selector.kind))
        || !scope.organizations.is_empty();
    let completeness = if saw_unknown {
        PopulationCompleteness::Unknown
    } else if explicit_family {
        PopulationCompleteness::Authoritative
    } else {
        PopulationCompleteness::Partial
    };
    InScopePopulation { ids, completeness }
}

fn unknown_row(subject: &SubjectRef, rationale: &str) -> SubjectScopeDecision {
    let lineage = vec![LineageHop {
        kind: subject.kind,
        id: subject.id.clone(),
    }];
    let explain = format!(
        "{} -> ISMS scope -> Unknown",
        hop_token(subject.kind, &subject.id)
    );
    SubjectScopeDecision {
        kind: subject.kind,
        id: subject.id.clone(),
        decision: ScopeDecision::Unknown,
        rationale: rationale.to_string(),
        lineage,
        explain,
        influencing_rules: Vec::new(),
    }
}

struct Engine<'a> {
    definition: &'a AssessmentDefinition,
    context: Option<&'a IsmsContext>,
    assets: BTreeMap<String, &'a Asset>,
    identities: BTreeMap<String, &'a Identity>,
    vendors: BTreeMap<String, &'a Vendor>,
    activities: BTreeMap<String, &'a ProcessingActivity>,
    bound_orgs: BTreeSet<String>,
    inclusions: Vec<IndexedRule>,
    exclusions: Vec<IndexedExclusion>,
    memo: BTreeMap<(SubjectKind, String), SubjectScopeDecision>,
    visiting: BTreeSet<(SubjectKind, String)>,
    explicit_only: Option<Vec<SubjectRef>>,
}

#[derive(Clone)]
struct IndexedRule {
    selector: SubjectSelector,
    digest: String,
    condition: Option<String>,
}

#[derive(Clone)]
struct IndexedExclusion {
    exclusion: ScopeExclusion,
    digest: String,
    index: u32,
    status: ExclusionStatus,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExclusionStatus {
    Active,
    Expired,
    Overdue,
    Invalid,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Polarity {
    Include,
    Exclude,
}

#[derive(Clone)]
struct Competing {
    polarity: Polarity,
    rank: u16,
    conditional: bool,
    rule: InfluencingRule,
}

impl<'a> Engine<'a> {
    fn new(input: &ScopeInputs<'a>, as_of: DateTime<Utc>) -> Self {
        let definition = input.definition;
        let assets = definition
            .assets
            .iter()
            .map(|a| (a.id.as_str().to_string(), a))
            .collect();
        let identities = definition
            .identities
            .iter()
            .map(|i| (i.id.as_str().to_string(), i))
            .collect();
        let vendors = definition
            .vendors
            .iter()
            .map(|v| (v.id.as_str().to_string(), v))
            .collect();
        let activities = definition
            .processing_activities
            .iter()
            .map(|p| (p.id.as_str().to_string(), p))
            .collect();
        let bound_orgs = bound_organizations(definition, input.context);
        let inclusions = unique_inclusions(&definition.scope.subjects);
        let exclusions = index_exclusions(&definition.scope.exclusions, as_of);
        Self {
            definition,
            context: input.context,
            assets,
            identities,
            vendors,
            activities,
            bound_orgs,
            inclusions,
            exclusions,
            memo: BTreeMap::new(),
            visiting: BTreeSet::new(),
            explicit_only: input.candidates.clone(),
        }
    }

    fn candidate_list(&self) -> Vec<SubjectRef> {
        if let Some(cands) = &self.explicit_only {
            let mut out = cands.clone();
            out.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.kind.cmp(&b.kind)));
            out.dedup();
            return out;
        }
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        if let Some(ctx) = self.context {
            let org = SubjectRef::new(
                SubjectKind::Organization,
                ctx.organization.id.as_str().to_string(),
            );
            if seen.insert((org.kind, org.id.clone())) {
                out.push(org);
            }
            for bu in &ctx.organization.business_units {
                let row = SubjectRef::new(SubjectKind::BusinessUnit, bu.id.as_str().to_string());
                if seen.insert((row.kind, row.id.clone())) {
                    out.push(row);
                }
            }
        }
        for asset in &self.definition.assets {
            let row = SubjectRef::new(
                asset_subject_kind(asset.kind),
                asset.id.as_str().to_string(),
            );
            if seen.insert((row.kind, row.id.clone())) {
                out.push(row);
            }
        }
        for identity in &self.definition.identities {
            let row = SubjectRef::new(
                identity_subject_kind(identity.kind),
                identity.id.as_str().to_string(),
            );
            if seen.insert((row.kind, row.id.clone())) {
                out.push(row);
            }
        }
        for vendor in &self.definition.vendors {
            let row = SubjectRef::new(SubjectKind::Vendor, vendor.id.as_str().to_string());
            if seen.insert((row.kind, row.id.clone())) {
                out.push(row);
            }
        }
        for activity in &self.definition.processing_activities {
            let row = SubjectRef::new(
                SubjectKind::ProcessingActivity,
                activity.id.as_str().to_string(),
            );
            if seen.insert((row.kind, row.id.clone())) {
                out.push(row);
            }
        }
        out.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.kind.cmp(&b.kind)));
        out
    }

    fn resolve_one(&mut self, subject: &SubjectRef) -> SubjectScopeDecision {
        let key = (subject.kind, subject.id.clone());
        if let Some(hit) = self.memo.get(&key) {
            return hit.clone();
        }
        if !self.visiting.insert(key.clone()) {
            return self.finish(
                subject,
                ScopeDecision::Unknown,
                "cycle in parent chain",
                Vec::new(),
                true,
            );
        }
        let row = self.evaluate(subject);
        self.visiting.remove(&key);
        self.memo.insert(key, row.clone());
        row
    }

    fn evaluate(&mut self, subject: &SubjectRef) -> SubjectScopeDecision {
        let known = self.is_known(subject);
        let mut competing = Vec::new();
        let mut trace = Vec::new();

        for inclusion in &self.inclusions.clone() {
            if let Some(rank) = self.selector_rank(&inclusion.selector, subject) {
                let polarity = if inclusion.selector.scope == SelectorScope::NoneOf {
                    Polarity::Exclude
                } else {
                    Polarity::Include
                };
                let rule = InfluencingRule {
                    class: if polarity == Polarity::Exclude {
                        InfluencingRuleClass::Exclusion
                    } else {
                        InfluencingRuleClass::Inclusion
                    },
                    rank,
                    selector_digest: inclusion.digest.clone(),
                    exclusion_index: None,
                    owner: None,
                    approval_ref: None,
                    approved_at: None,
                    expires_at: None,
                    review_by: None,
                    applied: true,
                };
                competing.push(Competing {
                    polarity,
                    rank,
                    conditional: inclusion.condition.is_some(),
                    rule,
                });
            }
        }

        for exclusion in &self.exclusions.clone() {
            let matches = exclusion
                .exclusion
                .subjects
                .iter()
                .filter_map(|sel| self.selector_rank(sel, subject).map(|rank| (sel, rank)))
                .max_by_key(|(_, rank)| *rank);
            let Some((_sel, rank)) = matches else {
                continue;
            };
            match exclusion.status {
                ExclusionStatus::Invalid => {
                    trace.push(InfluencingRule {
                        class: InfluencingRuleClass::InvalidExclusion,
                        rank,
                        selector_digest: exclusion.digest.clone(),
                        exclusion_index: Some(exclusion.index),
                        owner: exclusion.exclusion.owner.clone(),
                        approval_ref: exclusion.exclusion.approval_ref.clone(),
                        approved_at: exclusion.exclusion.approved_at,
                        expires_at: exclusion.exclusion.expires_at,
                        review_by: exclusion.exclusion.review_by,
                        applied: false,
                    });
                }
                ExclusionStatus::Expired | ExclusionStatus::Overdue => {
                    trace.push(InfluencingRule {
                        class: InfluencingRuleClass::ExpiredExclusion,
                        rank,
                        selector_digest: exclusion.digest.clone(),
                        exclusion_index: Some(exclusion.index),
                        owner: exclusion.exclusion.owner.clone(),
                        approval_ref: exclusion.exclusion.approval_ref.clone(),
                        approved_at: exclusion.exclusion.approved_at,
                        expires_at: exclusion.exclusion.expires_at,
                        review_by: exclusion.exclusion.review_by,
                        applied: false,
                    });
                }
                ExclusionStatus::Active => {
                    competing.push(Competing {
                        polarity: Polarity::Exclude,
                        rank,
                        conditional: false,
                        rule: InfluencingRule {
                            class: InfluencingRuleClass::Exclusion,
                            rank,
                            selector_digest: exclusion.digest.clone(),
                            exclusion_index: Some(exclusion.index),
                            owner: exclusion.exclusion.owner.clone(),
                            approval_ref: exclusion.exclusion.approval_ref.clone(),
                            approved_at: exclusion.exclusion.approved_at,
                            expires_at: exclusion.exclusion.expires_at,
                            review_by: exclusion.exclusion.review_by,
                            applied: true,
                        },
                    });
                }
            }
        }

        if known && self.belongs_to_bound_org(subject) {
            competing.push(Competing {
                polarity: Polarity::Include,
                rank: RANK_ORG,
                conditional: false,
                rule: InfluencingRule {
                    class: InfluencingRuleClass::Organization,
                    rank: RANK_ORG,
                    selector_digest: org_digest(&self.bound_orgs),
                    exclusion_index: None,
                    owner: None,
                    approval_ref: None,
                    approved_at: None,
                    expires_at: None,
                    review_by: None,
                    applied: true,
                },
            });
        }

        let parents = self.parents_of(subject);
        let mut inherited: Option<(u16, SubjectScopeDecision)> = None;
        for (distance, parent) in parents.iter().enumerate() {
            if parent.kind == subject.kind && parent.id == subject.id {
                continue;
            }
            let parent_row = self.resolve_one(parent);
            let parent_applied = parent_row
                .influencing_rules
                .iter()
                .any(|r| r.applied && r.class != InfluencingRuleClass::InvalidExclusion);
            if !parent_applied
                && !matches!(
                    parent_row.decision,
                    ScopeDecision::InScope | ScopeDecision::OutOfScope | ScopeDecision::Conditional
                )
            {
                continue;
            }
            let closer = u16::try_from(distance).unwrap_or(u16::MAX);
            let take = match &inherited {
                None => true,
                Some((prev, _)) => closer < *prev,
            };
            if take {
                inherited = Some((closer, parent_row));
            }
        }
        if let Some((_, parent_row)) = inherited
            && parent_row.decision != ScopeDecision::Unknown
        {
            competing.push(Competing {
                polarity: match parent_row.decision {
                    ScopeDecision::OutOfScope => Polarity::Exclude,
                    _ => Polarity::Include,
                },
                rank: RANK_INHERIT,
                conditional: parent_row.decision == ScopeDecision::Conditional,
                rule: InfluencingRule {
                    class: InfluencingRuleClass::Inheritance,
                    rank: RANK_INHERIT,
                    selector_digest: canonical_digest(&parent_row.id).unwrap_or_default(),
                    exclusion_index: None,
                    owner: None,
                    approval_ref: None,
                    approved_at: None,
                    expires_at: None,
                    review_by: None,
                    applied: true,
                },
            });
        }

        if !known {
            let mut influencing = trace;
            influencing.extend(competing.into_iter().map(|c| c.rule));
            return self.finish(
                subject,
                ScopeDecision::Unknown,
                "unresolved subject",
                influencing,
                false,
            );
        }

        let max_rank = competing.iter().map(|c| c.rank).max();
        let Some(max_rank) = max_rank else {
            return self.finish(
                subject,
                ScopeDecision::Unknown,
                "no matching inclusion or exclusion rule",
                trace,
                false,
            );
        };
        let winners: Vec<&Competing> = competing.iter().filter(|c| c.rank == max_rank).collect();
        let has_include = winners.iter().any(|c| c.polarity == Polarity::Include);
        let has_exclude = winners.iter().any(|c| c.polarity == Polarity::Exclude);
        let has_conditional = winners.iter().any(|c| c.conditional);

        let (decision, rationale) = if has_include && has_exclude {
            trace.push(InfluencingRule {
                class: InfluencingRuleClass::Conflict,
                rank: max_rank,
                selector_digest: winners
                    .iter()
                    .map(|c| c.rule.selector_digest.as_str())
                    .min()
                    .unwrap_or("")
                    .to_string(),
                exclusion_index: None,
                owner: None,
                approval_ref: None,
                approved_at: None,
                expires_at: None,
                review_by: None,
                applied: true,
            });
            (
                ScopeDecision::Unknown,
                "conflict: equal-rank include vs exclude".to_string(),
            )
        } else if has_exclude && has_conditional {
            trace.push(InfluencingRule {
                class: InfluencingRuleClass::Conflict,
                rank: max_rank,
                selector_digest: String::new(),
                exclusion_index: None,
                owner: None,
                approval_ref: None,
                approved_at: None,
                expires_at: None,
                review_by: None,
                applied: true,
            });
            (
                ScopeDecision::Unknown,
                "conflict: conditional vs active exclude".to_string(),
            )
        } else if has_exclude {
            let winner = winners
                .iter()
                .find(|c| c.polarity == Polarity::Exclude)
                .expect("exclude");
            (ScopeDecision::OutOfScope, exclude_rationale(&winner.rule))
        } else if has_conditional {
            (
                ScopeDecision::Conditional,
                "winning inclusion carries a documented condition".to_string(),
            )
        } else {
            (
                ScopeDecision::InScope,
                include_rationale(&winners, &self.bound_orgs),
            )
        };

        let mut influencing = trace;
        for c in competing {
            influencing.push(c.rule);
        }
        self.finish(subject, decision, &rationale, influencing, false)
    }

    fn finish(
        &self,
        subject: &SubjectRef,
        decision: ScopeDecision,
        rationale: &str,
        mut influencing: Vec<InfluencingRule>,
        cycle: bool,
    ) -> SubjectScopeDecision {
        dedup_rules(&mut influencing);
        let lineage = if cycle {
            vec![LineageHop {
                kind: subject.kind,
                id: subject.id.clone(),
            }]
        } else {
            self.lineage_hops(subject)
        };
        let explain = format_explain(&lineage, decision);
        SubjectScopeDecision {
            kind: subject.kind,
            id: subject.id.clone(),
            decision,
            rationale: rationale.to_string(),
            lineage,
            explain,
            influencing_rules: influencing,
        }
    }

    fn is_known(&self, subject: &SubjectRef) -> bool {
        if self.assets.contains_key(&subject.id) {
            return true;
        }
        if self.identities.contains_key(&subject.id) {
            return true;
        }
        if self.vendors.contains_key(&subject.id) {
            return true;
        }
        if self.activities.contains_key(&subject.id) {
            return true;
        }
        if let Some(ctx) = self.context {
            if ctx.organization.id.as_str() == subject.id {
                return true;
            }
            if ctx
                .organization
                .business_units
                .iter()
                .any(|bu| bu.id.as_str() == subject.id)
            {
                return true;
            }
            if ctx.identity_ids.iter().any(|id| id.as_str() == subject.id)
                || ctx.asset_ids.iter().any(|id| id.as_str() == subject.id)
                || ctx.vendor_ids.iter().any(|id| id.as_str() == subject.id)
            {
                return true;
            }
        }
        false
    }

    fn selector_rank(&self, selector: &SubjectSelector, subject: &SubjectRef) -> Option<u16> {
        if !self.selector_matches(selector, subject) {
            return None;
        }
        let tags = non_condition_tags(&selector.tags);
        match selector.scope {
            SelectorScope::AnyOf => {
                if selector.ids.contains(&subject.id) {
                    Some(RANK_EXACT)
                } else {
                    None
                }
            }
            SelectorScope::All => {
                if !tags.is_empty() {
                    Some(RANK_TAG)
                } else {
                    Some(RANK_KIND)
                }
            }
            SelectorScope::NoneOf => {
                if selector.ids.contains(&subject.id) {
                    None
                } else if !tags.is_empty() {
                    Some(RANK_TAG)
                } else {
                    Some(RANK_KIND)
                }
            }
        }
    }

    fn selector_matches(&self, selector: &SubjectSelector, subject: &SubjectRef) -> bool {
        if !kinds_compatible(selector.kind, subject.kind) {
            return false;
        }
        let have = subject_tags(subject, self);
        if !tags_match(&have, &non_condition_tags(&selector.tags)) {
            return false;
        }
        match selector.scope {
            SelectorScope::AnyOf => !selector.ids.is_empty() && selector.ids.contains(&subject.id),
            SelectorScope::All => selector.ids.is_empty() || selector.ids.contains(&subject.id),
            SelectorScope::NoneOf => !selector.ids.contains(&subject.id),
        }
    }

    fn belongs_to_bound_org(&self, subject: &SubjectRef) -> bool {
        if self.bound_orgs.is_empty() {
            return false;
        }
        if self.bound_orgs.contains(&subject.id) {
            return true;
        }
        if let Some(ctx) = self.context {
            if ctx.organization.id.as_str() == subject.id
                && self.bound_orgs.contains(ctx.organization.id.as_str())
            {
                return true;
            }
            if subject.kind == SubjectKind::BusinessUnit
                && ctx
                    .organization
                    .business_units
                    .iter()
                    .any(|bu| bu.id.as_str() == subject.id)
                && self.bound_orgs.contains(ctx.organization.id.as_str())
            {
                return true;
            }
            if matches!(
                subject.kind,
                SubjectKind::Identity | SubjectKind::User | SubjectKind::PrivilegedIdentity
            ) && ctx.identity_ids.iter().any(|id| id.as_str() == subject.id)
                && self.bound_orgs.contains(ctx.organization.id.as_str())
            {
                return true;
            }
            if subject.kind == SubjectKind::Vendor
                && ctx.vendor_ids.iter().any(|id| id.as_str() == subject.id)
                && self.bound_orgs.contains(ctx.organization.id.as_str())
            {
                return true;
            }
        }
        if let Some(asset) = self.assets.get(&subject.id) {
            if asset.kind == AssetKind::Organization && self.bound_orgs.contains(asset.id.as_str())
            {
                return true;
            }
            let tags = &asset.tags;
            if let Some(bu) = tags.get("businessUnit")
                && let Some(ctx) = self.context
                && ctx
                    .organization
                    .business_units
                    .iter()
                    .any(|unit| unit.id.as_str() == bu)
                && self.bound_orgs.contains(ctx.organization.id.as_str())
            {
                return true;
            }
            let mut current = asset.parent.clone();
            let mut guard = 0u32;
            let mut seen = BTreeSet::new();
            while let Some(pid) = current {
                if !seen.insert(pid.as_str().to_string()) || guard > 64 {
                    break;
                }
                guard += 1;
                if self.bound_orgs.contains(pid.as_str()) {
                    return true;
                }
                match self.assets.get(pid.as_str()) {
                    Some(parent) => {
                        if parent.kind == AssetKind::Organization
                            && self.bound_orgs.contains(parent.id.as_str())
                        {
                            return true;
                        }
                        current = parent.parent.clone();
                    }
                    None => break,
                }
            }
        }
        false
    }

    fn parents_of(&self, subject: &SubjectRef) -> Vec<SubjectRef> {
        let mut out = Vec::new();
        let tags = subject_tags(subject, self);
        if let Some(bu) = tags.get("businessUnit") {
            out.push(SubjectRef::new(SubjectKind::BusinessUnit, bu.clone()));
        }
        if let Some(asset) = self.assets.get(&subject.id)
            && let Some(parent) = &asset.parent
        {
            let kind = self
                .assets
                .get(parent.as_str())
                .map(|a| asset_subject_kind(a.kind))
                .unwrap_or(SubjectKind::Asset);
            out.push(SubjectRef::new(kind, parent.as_str().to_string()));
        }
        if subject.kind == SubjectKind::BusinessUnit
            && let Some(ctx) = self.context
            && let Some(bu) = ctx
                .organization
                .business_units
                .iter()
                .find(|b| b.id.as_str() == subject.id)
        {
            if let Some(parent) = &bu.parent_id {
                out.push(SubjectRef::new(
                    SubjectKind::BusinessUnit,
                    parent.as_str().to_string(),
                ));
            } else {
                out.push(SubjectRef::new(
                    SubjectKind::Organization,
                    ctx.organization.id.as_str().to_string(),
                ));
            }
        }
        out
    }

    fn lineage_hops(&self, subject: &SubjectRef) -> Vec<LineageHop> {
        let mut hops = vec![LineageHop {
            kind: subject.kind,
            id: subject.id.clone(),
        }];
        let tags = subject_tags(subject, self);
        if let Some(bu) = tags.get("businessUnit") {
            hops.push(LineageHop {
                kind: SubjectKind::BusinessUnit,
                id: bu.clone(),
            });
        }
        if let Some(asset) = self.assets.get(&subject.id) {
            let mut current = asset.parent.clone();
            let mut seen = BTreeSet::new();
            let mut guard = 0u32;
            while let Some(pid) = current {
                if !seen.insert(pid.as_str().to_string()) || guard > 64 {
                    break;
                }
                guard += 1;
                let Some(parent) = self.assets.get(pid.as_str()) else {
                    hops.push(LineageHop {
                        kind: SubjectKind::Asset,
                        id: pid.as_str().to_string(),
                    });
                    break;
                };
                if parent.kind == AssetKind::Organization
                    && self.bound_orgs.contains(parent.id.as_str())
                {
                    break;
                }
                hops.push(LineageHop {
                    kind: asset_subject_kind(parent.kind),
                    id: parent.id.as_str().to_string(),
                });
                current = parent.parent.clone();
            }
        }
        hops
    }
}

fn bound_organizations(
    definition: &AssessmentDefinition,
    context: Option<&IsmsContext>,
) -> BTreeSet<String> {
    let mut bound = BTreeSet::new();
    let listed: Vec<String> = definition.scope.organizations.clone();
    if let Some(ctx) = context {
        let org_id = ctx.organization.id.as_str().to_string();
        let legal = ctx.organization.legal_name.clone();
        if listed.is_empty() {
            bound.insert(org_id);
        } else {
            for entry in &listed {
                if entry == &org_id
                    || entry.eq_ignore_ascii_case(&legal)
                    || ctx
                        .organization
                        .display_name
                        .as_deref()
                        .is_some_and(|n| n.eq_ignore_ascii_case(entry))
                {
                    bound.insert(org_id.clone());
                } else if definition
                    .assets
                    .iter()
                    .any(|a| a.id.as_str() == entry && a.kind == AssetKind::Organization)
                {
                    bound.insert(entry.clone());
                }
            }
        }
    } else {
        for entry in listed {
            if definition
                .assets
                .iter()
                .any(|a| a.id.as_str() == entry && a.kind == AssetKind::Organization)
            {
                bound.insert(entry);
            }
        }
    }
    bound
}

fn unique_inclusions(subjects: &[SubjectSelector]) -> Vec<IndexedRule> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    let mut sorted = subjects.to_vec();
    sorted.sort_by(|a, b| {
        canonical_digest(a)
            .unwrap_or_default()
            .cmp(&canonical_digest(b).unwrap_or_default())
    });
    for selector in sorted {
        let digest = canonical_digest(&selector).unwrap_or_default();
        if !seen.insert(digest.clone()) {
            continue;
        }
        let condition = selector
            .tags
            .get("scopeCondition")
            .cloned()
            .filter(|s| !s.trim().is_empty());
        out.push(IndexedRule {
            selector,
            digest,
            condition,
        });
    }
    out
}

fn index_exclusions(exclusions: &[ScopeExclusion], as_of: DateTime<Utc>) -> Vec<IndexedExclusion> {
    let mut indexed: Vec<IndexedExclusion> = exclusions
        .iter()
        .enumerate()
        .map(|(i, exclusion)| {
            let digest = canonical_digest(exclusion).unwrap_or_default();
            let status = if !exclusion.governance_is_complete() {
                ExclusionStatus::Invalid
            } else if exclusion.is_expired_at(as_of) {
                ExclusionStatus::Expired
            } else if exclusion.is_review_overdue_at(as_of) {
                ExclusionStatus::Overdue
            } else {
                ExclusionStatus::Active
            };
            IndexedExclusion {
                exclusion: exclusion.clone(),
                digest,
                index: u32::try_from(i).unwrap_or(u32::MAX),
                status,
            }
        })
        .collect();
    indexed.sort_by(|a, b| a.digest.cmp(&b.digest).then_with(|| a.index.cmp(&b.index)));
    for (i, item) in indexed.iter_mut().enumerate() {
        item.index = u32::try_from(i).unwrap_or(u32::MAX);
    }
    indexed
}

fn asset_subject_kind(kind: AssetKind) -> SubjectKind {
    match kind {
        AssetKind::Organization => SubjectKind::Organization,
        AssetKind::Repository => SubjectKind::Repository,
        AssetKind::Application => SubjectKind::Application,
        AssetKind::Service => SubjectKind::Service,
        AssetKind::Database => SubjectKind::Database,
        AssetKind::CloudAccount => SubjectKind::CloudAccount,
        AssetKind::CloudResource => SubjectKind::CloudResource,
        AssetKind::Device => SubjectKind::Device,
        AssetKind::Network => SubjectKind::Network,
        AssetKind::Dataset => SubjectKind::Dataset,
        AssetKind::Endpoint => SubjectKind::Endpoint,
        AssetKind::Branch => SubjectKind::Branch,
        AssetKind::Deployment => SubjectKind::Deployment,
        AssetKind::Other => SubjectKind::Asset,
    }
}

fn identity_subject_kind(kind: IdentityKind) -> SubjectKind {
    match kind {
        IdentityKind::User => SubjectKind::Identity,
        IdentityKind::Service | IdentityKind::ServiceAccount => SubjectKind::ServiceAccount,
        IdentityKind::Team | IdentityKind::Role | IdentityKind::Other => SubjectKind::Identity,
    }
}

fn kinds_compatible(selector: SubjectKind, subject: SubjectKind) -> bool {
    selector == subject
        || matches!(
            (selector, subject),
            (
                SubjectKind::Asset,
                SubjectKind::Organization
                    | SubjectKind::Repository
                    | SubjectKind::Service
                    | SubjectKind::Application
                    | SubjectKind::Database
                    | SubjectKind::CloudAccount
                    | SubjectKind::CloudResource
                    | SubjectKind::Device
                    | SubjectKind::Network
                    | SubjectKind::Dataset
                    | SubjectKind::DataStore
                    | SubjectKind::Endpoint
                    | SubjectKind::Branch
                    | SubjectKind::Deployment,
            ) | (
                SubjectKind::Identity | SubjectKind::PersonnelPopulation,
                SubjectKind::Identity
                    | SubjectKind::User
                    | SubjectKind::PrivilegedIdentity
                    | SubjectKind::PersonnelPopulation,
            ) | (SubjectKind::User, SubjectKind::Identity)
                | (SubjectKind::Dataset, SubjectKind::DataStore)
                | (SubjectKind::DataStore, SubjectKind::Dataset)
        )
}

fn non_condition_tags(tags: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    tags.iter()
        .filter(|(k, _)| normalize_token(k) != "scopecondition")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn tags_match(have: &BTreeMap<String, String>, want: &BTreeMap<String, String>) -> bool {
    want.iter()
        .all(|(k, v)| have.get(k).is_some_and(|have_v| have_v == v))
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn subject_tags(subject: &SubjectRef, engine: &Engine<'_>) -> BTreeMap<String, String> {
    if let Some(asset) = engine.assets.get(&subject.id) {
        return asset.tags.clone();
    }
    BTreeMap::new()
}

fn kind_token(kind: SubjectKind) -> &'static str {
    match kind {
        SubjectKind::Organization => "org",
        SubjectKind::BusinessUnit => "business-unit",
        SubjectKind::Repository => "repo",
        SubjectKind::Service => "service",
        SubjectKind::Identity | SubjectKind::User | SubjectKind::PrivilegedIdentity => "identity",
        SubjectKind::Vendor => "vendor",
        SubjectKind::ProcessingActivity => "processing-activity",
        SubjectKind::Location => "location",
        SubjectKind::DataDomain => "data-domain",
        SubjectKind::PersonnelPopulation => "population",
        SubjectKind::Network => "network",
        SubjectKind::CloudAccount => "cloud-account",
        SubjectKind::Application => "application",
        SubjectKind::Asset => "asset",
        SubjectKind::Device => "device",
        SubjectKind::Dataset | SubjectKind::DataStore => "dataset",
        SubjectKind::Branch => "branch",
        SubjectKind::Database => "database",
        SubjectKind::CloudResource => "cloud-resource",
        SubjectKind::ServiceAccount => "service-account",
        SubjectKind::Endpoint => "endpoint",
        SubjectKind::Deployment => "deployment",
    }
}

fn id_suffix(id: &str) -> &str {
    id.rsplit(':').next().unwrap_or(id)
}

fn hop_token(kind: SubjectKind, id: &str) -> String {
    format!("{}:{}", kind_token(kind), id_suffix(id))
}

fn format_explain(lineage: &[LineageHop], decision: ScopeDecision) -> String {
    let mut parts: Vec<String> = lineage
        .iter()
        .map(|hop| hop_token(hop.kind, &hop.id))
        .collect();
    parts.push("ISMS scope".into());
    parts.push(decision_label(decision).into());
    parts.join(" -> ")
}

fn decision_label(decision: ScopeDecision) -> &'static str {
    match decision {
        ScopeDecision::InScope => "InScope",
        ScopeDecision::OutOfScope => "OutOfScope",
        ScopeDecision::Conditional => "Conditional",
        ScopeDecision::Unknown => "Unknown",
    }
}

fn org_digest(orgs: &BTreeSet<String>) -> String {
    canonical_digest(orgs).unwrap_or_default()
}

fn exclude_rationale(rule: &InfluencingRule) -> String {
    let owner = match &rule.owner {
        Some(PrincipalRef::Team(name)) => format!("team:{name}"),
        Some(PrincipalRef::Role(name)) => format!("role:{name}"),
        Some(PrincipalRef::Identity(id)) => format!("identity:{id}"),
        None => "unspecified".into(),
    };
    format!(
        "excluded by {} owned by {owner}",
        rule.approval_ref.as_deref().unwrap_or("exclusion")
    )
}

fn include_rationale(winners: &[&Competing], orgs: &BTreeSet<String>) -> String {
    if winners
        .iter()
        .any(|c| c.rule.class == InfluencingRuleClass::Organization)
    {
        let org = orgs.iter().next().cloned().unwrap_or_else(|| "org".into());
        return format!("included by organization-wide membership of {org}");
    }
    if winners
        .iter()
        .any(|c| c.rule.class == InfluencingRuleClass::Inheritance)
    {
        return "included by inherited parent membership".into();
    }
    "included by assessment scope selector".into()
}

fn dedup_rules(rules: &mut Vec<InfluencingRule>) {
    let mut seen = BTreeSet::new();
    rules.retain(|r| {
        seen.insert((
            format!("{:?}", r.class),
            r.selector_digest.clone(),
            r.applied,
            r.exclusion_index,
        ))
    });
    rules.sort_by(|a, b| {
        b.rank
            .cmp(&a.rank)
            .then_with(|| format!("{:?}", a.class).cmp(&format!("{:?}", b.class)))
            .then_with(|| a.selector_digest.cmp(&b.selector_digest))
    });
}

fn row_matches_selector(row: &SubjectScopeDecision, selector: &SubjectSelector) -> bool {
    if !kinds_compatible(selector.kind, row.kind) {
        return false;
    }
    match selector.scope {
        SelectorScope::AnyOf => selector.ids.contains(&row.id),
        SelectorScope::All => selector.ids.is_empty() || selector.ids.contains(&row.id),
        SelectorScope::NoneOf => !selector.ids.contains(&row.id),
    }
}
