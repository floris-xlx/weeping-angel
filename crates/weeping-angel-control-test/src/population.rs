//! Subject population resolution and coverage arithmetic.
#![allow(clippy::collapsible_if, clippy::if_same_then_else)]
//!
//! Absence of evidence is never a pass unless the population is authoritative
//! and the observation covers it. Indexes are keyed by evidence type + subject
//! (`EvidenceIndex`) so evaluation is not `O(|subjects| × |envelopes|)`.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{Exception, ExceptionStatus, SelectorScope, SubjectKind};
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceType, EvidenceValue};

use crate::expr::{CountPredicate, EvidenceSelector, SubjectSelector};
use crate::{AssessmentContext, Effectiveness, EvidenceSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PopulationCompleteness {
    Authoritative,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Population {
    pub selector: weeping_angel_assurance_ir::SubjectSelector,
    pub subject_ids: Vec<String>,
    pub authoritative: bool,
    pub observed_at: DateTime<Utc>,
    pub completeness: PopulationCompleteness,
}

impl Population {
    fn new(
        selector: weeping_angel_assurance_ir::SubjectSelector,
        mut subject_ids: Vec<String>,
        completeness: PopulationCompleteness,
        observed_at: DateTime<Utc>,
    ) -> Self {
        subject_ids.sort();
        subject_ids.dedup();
        Self {
            selector,
            subject_ids,
            authoritative: completeness == PopulationCompleteness::Authoritative,
            observed_at,
            completeness,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulationEvaluation {
    pub population: u64,
    pub evaluated: u64,
    pub passing: u64,
    pub failing: u64,
    pub missing: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<f64>,
    #[serde(default)]
    pub failing_subjects: Vec<String>,
    #[serde(default)]
    pub missing_subjects: Vec<String>,
    #[serde(default)]
    pub stale_subjects: Vec<String>,
    #[serde(default)]
    pub excepted_subjects: Vec<String>,
    #[serde(default)]
    pub technical_subjects: Vec<String>,
}

pub struct EvidenceIndex<'a> {
    by_type: BTreeMap<&'a str, Vec<&'a EvidenceEnvelope>>,
    by_type_and_subject: BTreeMap<(&'a str, &'a str), Vec<&'a EvidenceEnvelope>>,
    latest: BTreeMap<(&'a str, &'a str), &'a EvidenceEnvelope>,
}

impl<'a> EvidenceIndex<'a> {
    pub fn build(evidence: &'a EvidenceSet) -> Self {
        build_index(evidence)
    }

    pub fn by_subject(
        &self,
        evidence_type: &str,
        subject_id: &str,
    ) -> Option<&'a EvidenceEnvelope> {
        self.latest.get(&(evidence_type, subject_id)).copied()
    }

    pub fn group(&self, evidence_type: &str, subject_id: &str) -> Vec<&'a EvidenceEnvelope> {
        self.by_type_and_subject
            .get(&(evidence_type, subject_id))
            .cloned()
            .unwrap_or_default()
    }

    fn of_type(&self, evidence_type: &str) -> &[&'a EvidenceEnvelope] {
        self.by_type
            .get(evidence_type)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

pub fn index_envelopes(evidence: &EvidenceSet) -> EvidenceIndex<'_> {
    build_index(evidence)
}

pub fn build_index(evidence: &EvidenceSet) -> EvidenceIndex<'_> {
    let mut by_type: BTreeMap<&str, Vec<&EvidenceEnvelope>> = BTreeMap::new();
    let mut by_type_and_subject: BTreeMap<(&str, &str), Vec<&EvidenceEnvelope>> = BTreeMap::new();
    for env in evidence.iter() {
        let ty = env.observation().evidence_type().as_str();
        let subject = env.provenance().asset().as_str();
        by_type.entry(ty).or_default().push(env);
        by_type_and_subject
            .entry((ty, subject))
            .or_default()
            .push(env);
    }
    let mut latest = BTreeMap::new();
    for (&key, group) in &by_type_and_subject {
        latest.insert(key, select_latest(group));
    }
    EvidenceIndex {
        by_type,
        by_type_and_subject,
        latest,
    }
}

fn select_latest<'a>(group: &[&'a EvidenceEnvelope]) -> &'a EvidenceEnvelope {
    let superseded: BTreeSet<&str> = group.iter().filter_map(|e| e.supersedes()).collect();
    let mut leaves: Vec<&EvidenceEnvelope> = group
        .iter()
        .copied()
        .filter(|e| !superseded.contains(e.digest()))
        .collect();
    if leaves.is_empty() {
        leaves = group.to_vec();
    }
    leaves.sort_by(|a, b| {
        a.provenance()
            .collected_at
            .cmp(&b.provenance().collected_at)
            .then_with(|| a.digest().cmp(b.digest()))
    });
    leaves.into_iter().next_back().expect("group is non-empty")
}

pub fn resolve_population(
    selector: &SubjectSelector,
    evidence: &EvidenceSet,
    index: &EvidenceIndex<'_>,
    observation_type: Option<&EvidenceType>,
    observed_at: DateTime<Utc>,
) -> Population {
    if let Some(explicit) = evidence.explicit_population() {
        return explicit.clone();
    }

    let ir = selector.to_ir();

    if !ir.ids.is_empty() && ir.scope != SelectorScope::NoneOf {
        let ids: Vec<String> = ir.ids.iter().cloned().collect();
        return Population::new(ir, ids, PopulationCompleteness::Authoritative, observed_at);
    }

    let kind_key = selector.kind.as_deref().unwrap_or("");
    if let Some(identity) = resolve_identity_inventory(kind_key, index, &ir, observed_at) {
        return identity;
    }

    let mut members: Vec<String> = Vec::new();
    for env in index.of_type("inventory.subject") {
        if kind_matches(env, kind_key) {
            let id = env
                .observation()
                .fact("id")
                .unwrap_or_else(|| env.provenance().asset().as_str())
                .to_string();
            members.push(id);
        }
    }
    apply_scope(&ir, &mut members);

    let mut complete = false;
    for env in index.of_type("inventory.complete") {
        if !kind_matches(env, kind_key) {
            continue;
        }
        if fact_truthy(env, "authoritative") {
            complete = true;
            break;
        }
    }

    if complete {
        return Population::new(
            ir,
            members,
            PopulationCompleteness::Authoritative,
            observed_at,
        );
    }
    if !members.is_empty() {
        return Population::new(ir, members, PopulationCompleteness::Partial, observed_at);
    }

    let mut inferred = Vec::new();
    if let Some(ty) = observation_type {
        for env in index.of_type(ty.as_str()) {
            inferred.push(env.provenance().asset().as_str().to_string());
        }
        inferred.sort();
        inferred.dedup();
        apply_scope(&ir, &mut inferred);
    }
    Population::new(ir, inferred, PopulationCompleteness::Unknown, observed_at)
}

fn resolve_identity_inventory(
    kind_key: &str,
    index: &EvidenceIndex<'_>,
    ir: &weeping_angel_assurance_ir::SubjectSelector,
    observed_at: DateTime<Utc>,
) -> Option<Population> {
    let rows = index.of_type("evidence.identity.inventory");
    if rows.is_empty() {
        return None;
    }

    let authoritative = rows.iter().any(|env| {
        fact_truthy(env, "authoritative")
            && (fact_eq(env, "account_kind", "organization")
                || env.observation().fact("population_id").is_some()
                || env.provenance().asset().as_str().starts_with("org:"))
    });

    let mut members: Vec<String> = Vec::new();
    let want = normalize(kind_key);
    let privileged = index.of_type("evidence.identity.privileged-membership");
    let services = index.of_type("evidence.identity.service-account");

    for env in rows {
        let id = env
            .observation()
            .fact("subject_id")
            .unwrap_or_else(|| env.provenance().asset().as_str())
            .to_string();
        let account_kind = env.observation().fact("account_kind").unwrap_or("user");
        if account_kind == "organization" {
            continue;
        }
        let include = match want.as_str() {
            "privilegedidentity" => privileged
                .iter()
                .any(|p| p.provenance().asset().as_str() == id && fact_truthy(p, "privileged")),
            "serviceaccount" | "service" => {
                account_kind == "service"
                    || services
                        .iter()
                        .any(|s| s.provenance().asset().as_str() == id)
            }
            "user" => matches!(account_kind, "user" | "guest" | "break-glass"),
            "identity" | "" => account_kind != "organization",
            _ => normalize(account_kind) == want || kind_matches(env, kind_key),
        };
        if include {
            members.push(id);
        }
    }

    if want == "privilegedidentity" {
        for env in privileged {
            if fact_truthy(env, "privileged") {
                members.push(env.provenance().asset().as_str().to_string());
            }
        }
    }
    if matches!(want.as_str(), "serviceaccount" | "service") {
        for env in services {
            members.push(env.provenance().asset().as_str().to_string());
        }
    }

    apply_scope(ir, &mut members);
    let completeness = if authoritative {
        PopulationCompleteness::Authoritative
    } else {
        PopulationCompleteness::Partial
    };
    Some(Population::new(
        ir.clone(),
        members,
        completeness,
        observed_at,
    ))
}

fn fact_eq(env: &EvidenceEnvelope, key: &str, expected: &str) -> bool {
    env.observation()
        .fact(key)
        .is_some_and(|have| have.eq_ignore_ascii_case(expected))
}

fn apply_scope(ir: &weeping_angel_assurance_ir::SubjectSelector, members: &mut Vec<String>) {
    match ir.scope {
        SelectorScope::NoneOf => {
            members.retain(|id| !ir.ids.contains(id));
        }
        SelectorScope::AnyOf if !ir.ids.is_empty() => {
            members.retain(|id| ir.ids.contains(id));
        }
        _ => {}
    }
}

fn kind_matches(env: &EvidenceEnvelope, want: &str) -> bool {
    if want.is_empty() {
        return true;
    }
    let Some(have) = env.observation().fact("kind") else {
        return false;
    };
    if normalize(have) == normalize(want) {
        return true;
    }
    match (SubjectKind::parse_name(have), SubjectKind::parse_name(want)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

fn fact_truthy(env: &EvidenceEnvelope, key: &str) -> bool {
    env.observation()
        .fact_value(key)
        .map(is_truthy)
        .unwrap_or(false)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Passing,
    Failing,
    #[allow(dead_code)]
    Missing,
    #[allow(dead_code)]
    Stale,
    #[allow(dead_code)]
    Excepted,
    Technical,
}

fn is_truthy(value: &EvidenceValue) -> bool {
    matches!(classify_value(value), Outcome::Passing)
}

fn classify_value(value: &EvidenceValue) -> Outcome {
    match value {
        EvidenceValue::Bool(true) => Outcome::Passing,
        EvidenceValue::Bool(false) => Outcome::Failing,
        EvidenceValue::Integer(1) => Outcome::Passing,
        EvidenceValue::Integer(0) => Outcome::Failing,
        EvidenceValue::String(s) => classify_str(s),
        _ => Outcome::Technical,
    }
}

fn classify_str(raw: &str) -> Outcome {
    let t = raw.trim();
    if t.eq_ignore_ascii_case("true")
        || t.eq_ignore_ascii_case("pass")
        || t.eq_ignore_ascii_case("protected")
        || t.eq_ignore_ascii_case("enabled")
        || t == "1"
    {
        Outcome::Passing
    } else if t.eq_ignore_ascii_case("false")
        || t.eq_ignore_ascii_case("fail")
        || t.eq_ignore_ascii_case("unprotected")
        || t.eq_ignore_ascii_case("disabled")
        || t == "0"
    {
        Outcome::Failing
    } else {
        Outcome::Technical
    }
}

struct Partitions {
    passing: Vec<String>,
    failing: Vec<String>,
    missing: Vec<String>,
    stale: Vec<String>,
    excepted: Vec<String>,
    technical: Vec<String>,
    refs: Vec<String>,
}

fn partition(
    pop: &Population,
    evidence_sel: &EvidenceSelector,
    index: &EvidenceIndex<'_>,
    evidence: &EvidenceSet,
    context: &AssessmentContext,
) -> Partitions {
    let mut parts = Partitions {
        passing: Vec::new(),
        failing: Vec::new(),
        missing: Vec::new(),
        stale: Vec::new(),
        excepted: Vec::new(),
        technical: Vec::new(),
        refs: Vec::new(),
    };
    let ty = evidence_sel.evidence_type.as_str();
    let field = evidence_sel.field.as_deref();
    let kind = pop.selector.kind;
    for id in &pop.subject_ids {
        if subject_is_excepted(evidence.exceptions(), id, kind, context.now) {
            parts.excepted.push(id.clone());
            continue;
        }
        let Some(env) = index.by_subject(ty, id) else {
            parts.missing.push(id.clone());
            continue;
        };
        parts.refs.push(env.digest().to_string());
        if envelope_stale(env, context, evidence_sel.freshness) {
            parts.stale.push(id.clone());
            continue;
        }
        if let Some(name) = field {
            if field_is_temporal(name) || value_is_timestamp(env.observation().fact_value(name)) {
                if temporal_field_stale(index, ty, id, name, context) {
                    parts.stale.push(id.clone());
                } else if env.observation().fact_value(name).is_none() {
                    parts.missing.push(id.clone());
                } else {
                    parts.passing.push(id.clone());
                }
                continue;
            }
        }
        match field {
            None => parts.passing.push(id.clone()),
            Some(name) => match env.observation().fact_value(name) {
                None => parts.missing.push(id.clone()),
                Some(value) => match classify_value(value) {
                    Outcome::Passing => parts.passing.push(id.clone()),
                    Outcome::Failing => parts.failing.push(id.clone()),
                    Outcome::Technical => parts.technical.push(id.clone()),
                    _ => parts.failing.push(id.clone()),
                },
            },
        }
    }
    parts
}

fn subject_is_excepted(
    exceptions: &[Exception],
    subject_id: &str,
    kind: SubjectKind,
    now: DateTime<Utc>,
) -> bool {
    exceptions.iter().any(|ex| {
        if ex.status != ExceptionStatus::Approved {
            return false;
        }
        if ex.expires_at.is_some_and(|exp| exp <= now) {
            return false;
        }
        if ex.subjects.is_empty() {
            return false;
        }
        ex.subjects.iter().any(|sel| {
            if sel.kind != kind {
                return false;
            }
            match sel.scope {
                SelectorScope::All if sel.ids.is_empty() => true,
                SelectorScope::NoneOf => !sel.ids.contains(subject_id),
                _ => sel.ids.contains(subject_id),
            }
        })
    })
}

fn field_is_temporal(name: &str) -> bool {
    name.ends_with("_at") || name.contains("reviewed")
}

fn value_is_timestamp(value: Option<&EvidenceValue>) -> bool {
    match value {
        Some(EvidenceValue::Timestamp(_)) => true,
        Some(EvidenceValue::String(s)) => chrono::DateTime::parse_from_rfc3339(s).is_ok(),
        _ => false,
    }
}

fn parse_timestamp(value: &EvidenceValue) -> Option<DateTime<Utc>> {
    match value {
        EvidenceValue::Timestamp(ts) => Some(*ts),
        EvidenceValue::String(s) => chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc)),
        _ => None,
    }
}

fn temporal_field_stale(
    index: &EvidenceIndex<'_>,
    evidence_type: &str,
    subject_id: &str,
    field: &str,
    context: &AssessmentContext,
) -> bool {
    index.group(evidence_type, subject_id).iter().any(|env| {
        env.observation()
            .fact_value(field)
            .and_then(parse_timestamp)
            .is_some_and(|ts| {
                context
                    .now
                    .signed_duration_since(ts)
                    .to_std()
                    .map(|age| age > context.max_age)
                    .unwrap_or(true)
            })
    })
}

fn is_break_glass(index: &EvidenceIndex<'_>, subject_id: &str) -> bool {
    index
        .by_subject("evidence.identity.inventory", subject_id)
        .is_some_and(|env| fact_eq(env, "account_kind", "break-glass"))
}

fn envelope_stale(
    env: &EvidenceEnvelope,
    context: &AssessmentContext,
    freshness: Option<Duration>,
) -> bool {
    let age = context
        .now
        .signed_duration_since(env.provenance().collected_at)
        .to_std()
        .unwrap_or(Duration::MAX);
    if age > context.max_age {
        return true;
    }
    if let Some(window) = freshness {
        if age > window {
            return true;
        }
    }
    false
}

fn evaluation_from(pop: &Population, parts: &Partitions) -> PopulationEvaluation {
    let excepted = parts.excepted.len() as u64;
    let population = (pop.subject_ids.len() as u64).saturating_sub(excepted);
    let passing = parts.passing.len() as u64;
    let failing = parts.failing.len() as u64;
    let missing = parts.missing.len() as u64;
    let evaluated = passing + failing;
    let coverage = if population > 0 && pop.completeness != PopulationCompleteness::Unknown {
        Some(evaluated as f64 / population as f64)
    } else {
        None
    };
    PopulationEvaluation {
        population,
        evaluated,
        passing,
        failing,
        missing,
        coverage,
        failing_subjects: parts.failing.clone(),
        missing_subjects: parts.missing.clone(),
        stale_subjects: parts.stale.clone(),
        excepted_subjects: parts.excepted.clone(),
        technical_subjects: parts.technical.clone(),
    }
}

#[derive(Clone, Copy)]
pub enum CoverageMode {
    AtLeast,
    Exactly,
    All,
    Any,
    None,
    Missing,
}

pub struct PopulationOutcome {
    pub effectiveness: Effectiveness,
    pub rationale: String,
    pub population: PopulationEvaluation,
    pub refs: Vec<String>,
}

pub fn evaluate_coverage(
    selector: &SubjectSelector,
    evidence_sel: &EvidenceSelector,
    percentage: Option<&str>,
    mode: CoverageMode,
    evidence: &EvidenceSet,
    index: &EvidenceIndex<'_>,
    context: &AssessmentContext,
) -> PopulationOutcome {
    let pop = resolve_population(
        selector,
        evidence,
        index,
        Some(&evidence_sel.evidence_type),
        context.now,
    );
    let parts = partition(&pop, evidence_sel, index, evidence, context);
    let eval = evaluation_from(&pop, &parts);
    let threshold = percentage.and_then(|p| parse_percent(p).ok());
    let (mut effectiveness, mut rationale) = conclude(&pop, &parts, &eval, threshold, mode);
    if effectiveness == Effectiveness::Ineffective
        && !parts.failing.is_empty()
        && parts.failing.iter().all(|id| is_break_glass(index, id))
        && parts.technical.is_empty()
    {
        effectiveness = Effectiveness::ExceptionApproved;
        rationale = "approved break-glass exception for failing privileged subjects".into();
    }
    PopulationOutcome {
        effectiveness,
        rationale,
        population: eval,
        refs: parts.refs,
    }
}

pub fn evaluate_count(
    selector: &EvidenceSelector,
    predicate: &CountPredicate,
    index: &EvidenceIndex<'_>,
    context: &AssessmentContext,
) -> (Effectiveness, String, Vec<String>) {
    let ty = selector.evidence_type.as_str();
    let mut n = 0u64;
    let mut refs = Vec::new();
    let mut seen = BTreeSet::new();
    for env in index.of_type(ty) {
        let id = env.provenance().asset().as_str();
        if !seen.insert(id) {
            continue;
        }
        if let Some(id_filter) = selector.subject_selector.id.as_deref() {
            if id != id_filter {
                continue;
            }
        }
        let Some(latest) = index.by_subject(ty, id) else {
            continue;
        };
        if envelope_stale(latest, context, selector.freshness) {
            continue;
        }
        n += 1;
        refs.push(latest.digest().to_string());
    }
    let ok = predicate_holds(n, predicate);
    let rationale = format!("count {n} against {predicate:?}");
    (
        if ok {
            Effectiveness::Effective
        } else {
            Effectiveness::Ineffective
        },
        rationale,
        refs,
    )
}

pub fn evaluate_count_where(
    selector: &SubjectSelector,
    evidence_sel: &EvidenceSelector,
    predicate: &CountPredicate,
    evidence: &EvidenceSet,
    index: &EvidenceIndex<'_>,
    context: &AssessmentContext,
) -> PopulationOutcome {
    let pop = resolve_population(
        selector,
        evidence,
        index,
        Some(&evidence_sel.evidence_type),
        context.now,
    );
    let parts = partition(&pop, evidence_sel, index, evidence, context);
    let eval = evaluation_from(&pop, &parts);
    let n = parts.passing.len() as u64;
    let ok = predicate_holds(n, predicate);
    PopulationOutcome {
        effectiveness: if ok {
            Effectiveness::Effective
        } else {
            Effectiveness::Ineffective
        },
        rationale: format!("countWhere {n} against {predicate:?}"),
        population: eval,
        refs: parts.refs,
    }
}

fn predicate_holds(n: u64, predicate: &CountPredicate) -> bool {
    match predicate {
        CountPredicate::Eq(x) => n == *x,
        CountPredicate::Gte(x) => n >= *x,
        CountPredicate::Lte(x) => n <= *x,
    }
}

pub fn parse_percent(raw: &str) -> Result<f64, String> {
    let trimmed = raw.trim();
    let number = trimmed.strip_suffix('%').unwrap_or(trimmed).trim();
    let value: f64 = number
        .parse()
        .map_err(|_| format!("invalid percentage {raw}"))?;
    if !(0.0..=100.0).contains(&value) {
        return Err(format!("percentage out of range: {raw}"));
    }
    Ok(value / 100.0)
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn conclude(
    pop: &Population,
    parts: &Partitions,
    eval: &PopulationEvaluation,
    threshold: Option<f64>,
    mode: CoverageMode,
) -> (Effectiveness, String) {
    let unknown = pop.completeness != PopulationCompleteness::Authoritative;
    let p = eval.population;
    let passing = eval.passing;
    let failing = eval.failing;
    let missing = eval.missing;
    let stale = parts.stale.len() as u64;
    let technical = parts.technical.len() as u64;
    let strong = matches!(
        mode,
        CoverageMode::All | CoverageMode::None | CoverageMode::Missing | CoverageMode::Exactly
    ) || threshold.is_some_and(|t| (t - 1.0).abs() < 1e-12);

    if matches!(mode, CoverageMode::Any) {
        if passing > 0 {
            return (
                Effectiveness::Effective,
                format!("any-subject: {passing} passing"),
            );
        }
        if unknown {
            return (
                Effectiveness::Inconclusive,
                "any-subject: population completeness unknown and no passing subject".into(),
            );
        }
        if p == 0 {
            return (
                Effectiveness::InsufficientEvidence,
                "any-subject: empty population".into(),
            );
        }
        return (
            Effectiveness::Ineffective,
            "any-subject: no passing subject".into(),
        );
    }

    if unknown && strong {
        if pop.completeness == PopulationCompleteness::Partial {
            return (
                Effectiveness::InsufficientEvidence,
                "population completeness is partial; refusing strong all-subject conclusion".into(),
            );
        }
        return (
            Effectiveness::Inconclusive,
            "population completeness is unknown; refusing strong all-subject conclusion".into(),
        );
    }
    if unknown {
        return (
            Effectiveness::Inconclusive,
            "population completeness is unknown; coverage threshold not proven".into(),
        );
    }

    if p == 0 {
        return (
            Effectiveness::InsufficientEvidence,
            "authoritative population is empty; never effective without applicability".into(),
        );
    }

    if matches!(mode, CoverageMode::Missing) {
        if missing == 0 {
            return (
                Effectiveness::Effective,
                "no subjects missing evidence".into(),
            );
        }
        return (
            Effectiveness::InsufficientEvidence,
            format!("missing evidence for {missing} subjects"),
        );
    }

    let denom = p as f64;
    let pessimistic = passing as f64 / denom;
    let optimistic = (passing + missing + stale) as f64 / denom;

    if matches!(mode, CoverageMode::None) {
        if failing > 0 || technical > 0 {
            return (
                Effectiveness::Ineffective,
                format!("none-subjects: {failing} failing"),
            );
        }
        if missing > 0 {
            return (
                Effectiveness::InsufficientEvidence,
                format!("none-subjects: {missing} missing"),
            );
        }
        if stale > 0 {
            return (
                Effectiveness::StaleEvidence,
                format!("none-subjects: {stale} stale"),
            );
        }
        return (
            Effectiveness::Effective,
            "none-subjects: no failing or missing members".into(),
        );
    }

    if matches!(mode, CoverageMode::Exactly) {
        let Some(t) = threshold else {
            return (
                Effectiveness::Inconclusive,
                "coverage exactly: missing percentage".into(),
            );
        };
        if round4(pessimistic) == round4(t) {
            return (
                Effectiveness::Effective,
                format!("coverage exactly {}", round4(pessimistic)),
            );
        }
        return (
            Effectiveness::Ineffective,
            format!(
                "coverage exactly: pessimistic {} != {}",
                round4(pessimistic),
                round4(t)
            ),
        );
    }

    let t = threshold.unwrap_or(1.0);

    if failing > 0 && t >= 1.0 - 1e-12 {
        return (
            Effectiveness::Ineffective,
            format!("coverage: {failing} failing of {p}"),
        );
    }
    if technical > 0 && t >= 1.0 - 1e-12 {
        return (
            Effectiveness::Ineffective,
            format!("coverage: {technical} technical failures of {p}"),
        );
    }
    if optimistic < t - 1e-12 {
        return (
            Effectiveness::Ineffective,
            format!(
                "coverage: optimistic {} below threshold {}",
                round4(optimistic),
                round4(t)
            ),
        );
    }
    // Stale is the deciding defect when no explicit fails/missing remain and
    // the threshold would pass if stale observations were fresh passes.
    if stale > 0 && failing == 0 && technical == 0 && missing == 0 && pessimistic + 1e-12 < t {
        return (
            Effectiveness::StaleEvidence,
            format!("coverage: {stale} stale subjects decide the threshold"),
        );
    }
    if pessimistic + 1e-12 < t && t <= optimistic + 1e-12 {
        return (
            Effectiveness::InsufficientEvidence,
            format!(
                "coverage: pessimistic {} < {} <= optimistic {}",
                round4(pessimistic),
                round4(t),
                round4(optimistic)
            ),
        );
    }
    if pessimistic + 1e-12 >= t {
        let effectiveness = if failing == 0 && technical == 0 && stale == 0 {
            Effectiveness::Effective
        } else if t < 1.0 - 1e-12 && failing + technical > 0 {
            Effectiveness::Effective
        } else if stale > 0 {
            Effectiveness::StaleEvidence
        } else {
            Effectiveness::PartiallyEffective
        };
        return (
            effectiveness,
            format!(
                "coverage: {}/{} passing (threshold {})",
                passing,
                p,
                round4(t)
            ),
        );
    }
    (
        Effectiveness::InsufficientEvidence,
        format!("coverage inconclusive for threshold {}", round4(t)),
    )
}
