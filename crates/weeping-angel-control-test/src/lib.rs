//! Offline control-test runtime. Provider-blind. Zero network I/O.
//!
//! Runtime is provider_blind: decisions never key on collector_id.
//! Comparisons consume enum EvidenceValue (Integer, Bool, String, …) from
//! weeping-angel-evidence. Incompatible types fail closed with a
//! `type mismatch` rationale.

pub mod expr;
pub mod population;
pub mod temporal;

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{ControlId, ControlTestId, Exception, canonical_digest};
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceType, EvidenceValidityEvent};

pub use expr::{
    CountPredicate, EvidenceSelector, EvidenceValue, SubjectSelector, TestExpr, ValueExpr,
};
pub use population::{
    CoverageMode, EvidenceIndex, Population, PopulationCompleteness, PopulationEvaluation,
    build_index, build_index_as_of, index_envelopes,
};
pub use temporal::{
    FreshnessPolicy, PeriodEffectiveness, TemporalQuery, TemporalSemantics, TimeRange,
    project_period_effectiveness, select_evidence, select_latest_as_of,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlTestKind {
    Automated,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Effectiveness {
    Effective,
    Ineffective,
    PartiallyEffective,
    NotApplicable,
    NotTested,
    InsufficientEvidence,
    StaleEvidence,
    ManualReviewRequired,
    ExceptionApproved,
    Inconclusive,
}

include!("result.inc");

fn default_checked_at() -> DateTime<Utc> {
    Utc::now()
}

fn default_test_version() -> String {
    "1".into()
}

#[derive(Debug, Clone)]
pub struct AssessmentContext {
    pub now: DateTime<Utc>,
    pub max_age: Duration, // as_of defaults to `now`; period via FreshnessPolicy / TimeRange
}

impl AssessmentContext {
    /// Injected assessment clock (pinned `asOf` for this evaluation).
    /// Distinct from live ledger [`weeping_angel_evidence::EvidenceLedger::current`].
    pub fn as_of(&self) -> DateTime<Utc> {
        pinned_assessment_clock(self)
    }

    pub fn period(&self) -> Option<TimeRange> {
        None
    }

    pub fn freshness_policy(&self) -> FreshnessPolicy {
        FreshnessPolicy {
            max_age: self.max_age,
            as_of: self.as_of(),
            period: self.period(),
        }
    }
}

fn pinned_assessment_clock(ctx: &AssessmentContext) -> DateTime<Utc> {
    // `now` is the injected assessment clock for this context (pinned asOf).
    // Live ledger current() uses Utc::now(), not this helper.
    ctx.now
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSet {
    envelopes: BTreeMap<String, EvidenceEnvelope>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    exceptions: Vec<Exception>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    explicit_population: Option<Population>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    validity_events: Vec<EvidenceValidityEvent>,
}

impl EvidenceSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, envelope: EvidenceEnvelope) {
        self.envelopes
            .insert(envelope.digest().to_string(), envelope);
    }

    pub fn record_validity_event(&mut self, event: EvidenceValidityEvent) {
        if self
            .validity_events
            .iter()
            .any(|existing| existing.event_id == event.event_id)
        {
            return;
        }
        self.validity_events.push(event);
    }

    pub fn validity_events(&self) -> &[EvidenceValidityEvent] {
        &self.validity_events
    }

    pub fn insert_exception(&mut self, exception: Exception) {
        self.exceptions.push(exception);
    }

    pub fn set_population(&mut self, population: Population) {
        self.explicit_population = Some(population);
    }

    pub fn exceptions(&self) -> &[Exception] {
        &self.exceptions
    }

    pub fn explicit_population(&self) -> Option<&Population> {
        self.explicit_population.as_ref()
    }

    pub fn len(&self) -> usize {
        self.envelopes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.envelopes.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &EvidenceEnvelope> {
        self.envelopes.values()
    }
}

#[derive(Debug, Clone)]
pub struct CompiledControlTest {
    pub id: ControlTestId,
    pub control_id: ControlId,
    pub kind: ControlTestKind,
    pub required: Vec<EvidenceType>,
    pub break_on: Vec<EvidenceType>,
    pub expr: Option<TestExpr>,
}

impl CompiledControlTest {
    pub fn builder() -> CompiledControlTestBuilder {
        CompiledControlTestBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct CompiledControlTestBuilder {
    id: Option<ControlTestId>,
    control_id: Option<ControlId>,
    kind: Option<ControlTestKind>,
    required: Vec<EvidenceType>,
    break_on: Vec<EvidenceType>,
    expr: Option<TestExpr>,
}

impl CompiledControlTestBuilder {
    pub fn id(mut self, id: ControlTestId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn control_id(mut self, control_id: ControlId) -> Self {
        self.control_id = Some(control_id);
        self
    }

    pub fn kind(mut self, kind: ControlTestKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn require(mut self, evidence_type: EvidenceType) -> Self {
        self.required.push(evidence_type);
        self
    }

    pub fn break_on(mut self, evidence_type: EvidenceType) -> Self {
        self.break_on.push(evidence_type);
        self
    }

    pub fn expr(mut self, expr: TestExpr) -> Self {
        self.expr = Some(expr);
        self
    }

    pub fn build(self) -> CompiledControlTest {
        CompiledControlTest {
            id: self.id.expect("CompiledControlTest.id"),
            control_id: self.control_id.expect("CompiledControlTest.control_id"),
            kind: self.kind.unwrap_or(ControlTestKind::Automated),
            required: self.required,
            break_on: self.break_on,
            expr: self.expr,
        }
    }
}

include!("run.inc");

struct NodeOut {
    effectiveness: Effectiveness,
    rationale: String,
    population: Option<PopulationEvaluation>,
}

fn node(effectiveness: Effectiveness, rationale: impl Into<String>) -> NodeOut {
    NodeOut {
        effectiveness,
        rationale: rationale.into(),
        population: None,
    }
}

fn attach_population(out: population::PopulationOutcome, refs: &mut Vec<String>) -> NodeOut {
    refs.extend(out.refs.iter().cloned());
    NodeOut {
        effectiveness: out.effectiveness,
        rationale: out.rationale,
        population: Some(out.population),
    }
}

fn eval_node(
    expr: &TestExpr,
    envelopes: &[&EvidenceEnvelope],
    evidence: &EvidenceSet,
    index: &EvidenceIndex<'_>,
    context: &AssessmentContext,
    refs: &mut Vec<String>,
    missing: &mut Vec<String>,
) -> NodeOut {
    match expr {
        TestExpr::ManualReview => node(
            Effectiveness::ManualReviewRequired,
            "expression requires manual review",
        ),
        TestExpr::Exists(sel) => match first_selector(evidence, sel, context) {
            None => {
                missing.push(sel.evidence_type.to_string());
                node(
                    Effectiveness::InsufficientEvidence,
                    format!("missing {}", sel.evidence_type),
                )
            }
            Some(env) if is_stale(env, context) => {
                refs.push(env.digest().to_string());
                node(
                    Effectiveness::StaleEvidence,
                    format!("stale {}", sel.evidence_type),
                )
            }
            Some(env) => {
                refs.push(env.digest().to_string());
                node(
                    Effectiveness::Effective,
                    format!("exists {}", sel.evidence_type),
                )
            }
        },
        TestExpr::Missing(sel) => match first_selector(evidence, sel, context) {
            None => node(
                Effectiveness::Effective,
                format!("missing as required: {}", sel.evidence_type),
            ),
            Some(env) => {
                refs.push(env.digest().to_string());
                node(
                    Effectiveness::Ineffective,
                    format!("unexpected {}", sel.evidence_type),
                )
            }
        },
        TestExpr::Gte(ValueExpr::Field(sel), expected) => {
            compare_numeric(evidence, context, sel, expected, Cmp::Gte, refs, missing)
        }
        TestExpr::Gt(ValueExpr::Field(sel), expected) => {
            compare_numeric(evidence, context, sel, expected, Cmp::Gt, refs, missing)
        }
        TestExpr::Lte(ValueExpr::Field(sel), expected) => {
            compare_numeric(evidence, context, sel, expected, Cmp::Lte, refs, missing)
        }
        TestExpr::Lt(ValueExpr::Field(sel), expected) => {
            compare_numeric(evidence, context, sel, expected, Cmp::Lt, refs, missing)
        }
        TestExpr::Eq(ValueExpr::Field(sel), expected) => {
            compare_eq(evidence, context, sel, expected, true, refs, missing)
        }
        TestExpr::Neq(ValueExpr::Field(sel), expected) => {
            compare_eq(evidence, context, sel, expected, false, refs, missing)
        }
        TestExpr::Contains(ValueExpr::Field(sel), expected) => {
            compare_contains(evidence, context, sel, expected, true, refs, missing)
        }
        TestExpr::NotContains(ValueExpr::Field(sel), expected) => {
            compare_contains(evidence, context, sel, expected, false, refs, missing)
        }
        TestExpr::In(ValueExpr::Field(sel), expected) => {
            compare_in(evidence, context, sel, expected, refs, missing)
        }
        TestExpr::FreshWithin { selector, duration } => {
            match first_selector(evidence, selector, context) {
                None => {
                    missing.push(selector.evidence_type.to_string());
                    node(
                        Effectiveness::InsufficientEvidence,
                        format!("missing {}", selector.evidence_type),
                    )
                }
                Some(env) => {
                    refs.push(env.digest().to_string());
                    let age = context
                        .now
                        .signed_duration_since(env.provenance().collected_at)
                        .to_std()
                        .unwrap_or(Duration::MAX);
                    if age > *duration {
                        node(
                            Effectiveness::StaleEvidence,
                            format!("{} older than freshness window", selector.evidence_type),
                        )
                    } else {
                        node(
                            Effectiveness::Effective,
                            format!("{} is fresh", selector.evidence_type),
                        )
                    }
                }
            }
        }
        TestExpr::Count {
            selector,
            predicate,
        } => {
            let (effectiveness, rationale, counted_refs) =
                population::evaluate_count(selector, predicate, index, context);
            refs.extend(counted_refs);
            node(effectiveness, rationale)
        }
        TestExpr::CountWhere {
            selector,
            evidence: evidence_sel,
            predicate,
        } => attach_population(
            population::evaluate_count_where(
                selector,
                evidence_sel,
                predicate,
                evidence,
                index,
                context,
            ),
            refs,
        ),
        TestExpr::CoverageAtLeast {
            selector,
            evidence: evidence_sel,
            percentage,
        } => attach_population(
            population::evaluate_coverage(
                selector,
                evidence_sel,
                Some(percentage),
                CoverageMode::AtLeast,
                evidence,
                index,
                context,
            ),
            refs,
        ),
        TestExpr::CoverageExactly {
            selector,
            evidence: evidence_sel,
            percentage,
        } => attach_population(
            population::evaluate_coverage(
                selector,
                evidence_sel,
                Some(percentage),
                CoverageMode::Exactly,
                evidence,
                index,
                context,
            ),
            refs,
        ),
        TestExpr::AllSubjects {
            selector,
            evidence: evidence_sel,
        } => attach_population(
            population::evaluate_coverage(
                selector,
                evidence_sel,
                Some("100"),
                CoverageMode::All,
                evidence,
                index,
                context,
            ),
            refs,
        ),
        TestExpr::AnySubject {
            selector,
            evidence: evidence_sel,
        } => attach_population(
            population::evaluate_coverage(
                selector,
                evidence_sel,
                None,
                CoverageMode::Any,
                evidence,
                index,
                context,
            ),
            refs,
        ),
        TestExpr::NoneSubjects {
            selector,
            evidence: evidence_sel,
        } => attach_population(
            population::evaluate_coverage(
                selector,
                evidence_sel,
                None,
                CoverageMode::None,
                evidence,
                index,
                context,
            ),
            refs,
        ),
        TestExpr::MissingSubjects {
            selector,
            evidence: evidence_sel,
        } => attach_population(
            population::evaluate_coverage(
                selector,
                evidence_sel,
                None,
                CoverageMode::Missing,
                evidence,
                index,
                context,
            ),
            refs,
        ),
        TestExpr::All(nodes) => {
            let mut worst = Effectiveness::Effective;
            let mut notes = Vec::new();
            for child in nodes {
                let out = eval_node(child, envelopes, evidence, index, context, refs, missing);
                notes.push(out.rationale);
                worst = worse(worst, out.effectiveness);
            }
            node(worst, notes.join("; "))
        }
        TestExpr::Any(nodes) => {
            let mut best = Effectiveness::InsufficientEvidence;
            let mut notes = Vec::new();
            for child in nodes {
                let out = eval_node(child, envelopes, evidence, index, context, refs, missing);
                notes.push(out.rationale.clone());
                if rank(out.effectiveness) < rank(best) {
                    best = out.effectiveness;
                }
            }
            node(best, notes.join("; "))
        }
        TestExpr::None(nodes) => {
            let any = eval_node(
                &TestExpr::Any(nodes.clone()),
                envelopes,
                evidence,
                index,
                context,
                refs,
                missing,
            );
            if any.effectiveness == Effectiveness::Effective {
                node(Effectiveness::Ineffective, "none-of matched")
            } else {
                node(Effectiveness::Effective, "none-of held")
            }
        }
        TestExpr::Not(inner) => {
            let out = eval_node(inner, envelopes, evidence, index, context, refs, missing);
            let flipped = match out.effectiveness {
                Effectiveness::Effective => Effectiveness::Ineffective,
                Effectiveness::Ineffective => Effectiveness::Effective,
                other => other,
            };
            node(flipped, format!("not ({})", out.rationale))
        }
        other => node(
            Effectiveness::NotTested,
            format!("unsupported expression arm: {other:?}"),
        ),
    }
}

#[derive(Clone, Copy)]
enum Cmp {
    Gt,
    Gte,
    Lt,
    Lte,
}

fn compare_numeric(
    evidence: &EvidenceSet,
    context: &AssessmentContext,
    sel: &EvidenceSelector,
    expected: &EvidenceValue,
    cmp: Cmp,
    refs: &mut Vec<String>,
    missing: &mut Vec<String>,
) -> NodeOut {
    let Some(env) = first_selector(evidence, sel, context) else {
        missing.push(sel.evidence_type.to_string());
        return node(
            Effectiveness::InsufficientEvidence,
            format!("missing {}", sel.evidence_type),
        );
    };
    refs.push(env.digest().to_string());
    if is_stale(env, context) {
        return node(
            Effectiveness::StaleEvidence,
            format!("stale {}", sel.evidence_type),
        );
    }
    let field = sel.field.as_deref().unwrap_or("value");
    let Some(have) = env.observation().fact_value(field) else {
        return node(
            Effectiveness::InsufficientEvidence,
            format!("missing field {field}"),
        );
    };
    let order = match have.cmp_numeric(expected) {
        Ok(order) => order,
        Err(err) => return node(Effectiveness::Ineffective, err),
    };
    let ok = match cmp {
        Cmp::Gt => order.is_gt(),
        Cmp::Gte => order.is_ge(),
        Cmp::Lt => order.is_lt(),
        Cmp::Lte => order.is_le(),
    };
    if ok {
        node(Effectiveness::Effective, format!("{field} meets threshold"))
    } else {
        node(
            Effectiveness::Ineffective,
            format!("{field} does not meet threshold"),
        )
    }
}

fn compare_eq(
    evidence: &EvidenceSet,
    context: &AssessmentContext,
    sel: &EvidenceSelector,
    expected: &EvidenceValue,
    want_eq: bool,
    refs: &mut Vec<String>,
    missing: &mut Vec<String>,
) -> NodeOut {
    let Some(env) = first_selector(evidence, sel, context) else {
        missing.push(sel.evidence_type.to_string());
        return node(
            Effectiveness::InsufficientEvidence,
            format!("missing {}", sel.evidence_type),
        );
    };
    refs.push(env.digest().to_string());
    if is_stale(env, context) {
        return node(
            Effectiveness::StaleEvidence,
            format!("stale {}", sel.evidence_type),
        );
    }
    let field = sel.field.as_deref().unwrap_or("value");
    let Some(have) = env.observation().fact_value(field) else {
        return node(
            Effectiveness::InsufficientEvidence,
            format!("missing field {field}"),
        );
    };
    let equal = match have.typed_eq(expected) {
        Ok(equal) => equal,
        Err(err) => return node(Effectiveness::Ineffective, err),
    };
    let ok = if want_eq { equal } else { !equal };
    node(
        if ok {
            Effectiveness::Effective
        } else {
            Effectiveness::Ineffective
        },
        format!("field {field} compared"),
    )
}

fn compare_contains(
    evidence: &EvidenceSet,
    context: &AssessmentContext,
    sel: &EvidenceSelector,
    expected: &EvidenceValue,
    want_contains: bool,
    refs: &mut Vec<String>,
    missing: &mut Vec<String>,
) -> NodeOut {
    let Some(env) = first_selector(evidence, sel, context) else {
        missing.push(sel.evidence_type.to_string());
        return node(
            Effectiveness::InsufficientEvidence,
            format!("missing {}", sel.evidence_type),
        );
    };
    refs.push(env.digest().to_string());
    if is_stale(env, context) {
        return node(
            Effectiveness::StaleEvidence,
            format!("stale {}", sel.evidence_type),
        );
    }
    let field = sel.field.as_deref().unwrap_or("value");
    let Some(have) = env.observation().fact_value(field) else {
        return node(
            Effectiveness::InsufficientEvidence,
            format!("missing field {field}"),
        );
    };
    let contains = match have.list_contains(expected) {
        Ok(contains) => contains,
        Err(err) => return node(Effectiveness::Ineffective, err),
    };
    let ok = if want_contains { contains } else { !contains };
    node(
        if ok {
            Effectiveness::Effective
        } else {
            Effectiveness::Ineffective
        },
        format!("field {field} compared"),
    )
}

fn compare_in(
    evidence: &EvidenceSet,
    context: &AssessmentContext,
    sel: &EvidenceSelector,
    expected: &[EvidenceValue],
    refs: &mut Vec<String>,
    missing: &mut Vec<String>,
) -> NodeOut {
    let Some(env) = first_selector(evidence, sel, context) else {
        missing.push(sel.evidence_type.to_string());
        return node(
            Effectiveness::InsufficientEvidence,
            format!("missing {}", sel.evidence_type),
        );
    };
    refs.push(env.digest().to_string());
    if is_stale(env, context) {
        return node(
            Effectiveness::StaleEvidence,
            format!("stale {}", sel.evidence_type),
        );
    }
    let field = sel.field.as_deref().unwrap_or("value");
    let Some(have) = env.observation().fact_value(field) else {
        return node(
            Effectiveness::InsufficientEvidence,
            format!("missing field {field}"),
        );
    };
    let mut matched = false;
    for candidate in expected {
        match have.typed_eq(candidate) {
            Ok(true) => {
                matched = true;
                break;
            }
            Ok(false) => {}
            Err(err) => return node(Effectiveness::Ineffective, err),
        }
    }
    node(
        if matched {
            Effectiveness::Effective
        } else {
            Effectiveness::Ineffective
        },
        format!("field {field} compared"),
    )
}

fn worse(a: Effectiveness, b: Effectiveness) -> Effectiveness {
    if rank(b) > rank(a) { b } else { a }
}

fn rank(e: Effectiveness) -> u8 {
    match e {
        Effectiveness::Effective => 0,
        Effectiveness::ExceptionApproved => 1,
        Effectiveness::PartiallyEffective => 2,
        Effectiveness::NotApplicable | Effectiveness::NotTested => 3,
        Effectiveness::InsufficientEvidence | Effectiveness::StaleEvidence => 4,
        Effectiveness::ManualReviewRequired => 5,
        Effectiveness::Inconclusive => 6,
        Effectiveness::Ineffective => 7,
    }
}

fn finalize(
    mut result: ControlTestResult,
    test: &CompiledControlTest,
    evidence: &EvidenceSet,
    context: &AssessmentContext,
    refs: &[String],
    missing: &[String],
) -> ControlTestResult {
    result.evidence_refs = refs.to_vec();
    result.missing_evidence = missing.to_vec();
    result.status = Some(result.effectiveness);
    result.reason = Some(result.rationale.clone());
    let as_of = context.as_of();
    let mut candidate_digests: Vec<String> = evidence
        .iter()
        .filter(|e| {
            weeping_angel_evidence::project_validity(e, evidence.validity_events(), as_of).is_some()
        })
        .map(|e| e.digest().to_string())
        .collect();
    candidate_digests.sort();
    let body = (
        result.test_id.as_str(),
        result.control_id.as_str(),
        candidate_digests,
    );
    result.input_digest = canonical_digest(&body).unwrap_or_default();
    if result.period.is_none() {
        result.period = Some(crate::temporal::project_period_effectiveness(
            test, evidence, context,
        ));
    }
    result
}

fn first_matching<'a>(
    envelopes: &[&'a EvidenceEnvelope],
    types: &[EvidenceType],
) -> Option<&'a EvidenceEnvelope> {
    envelopes
        .iter()
        .copied()
        .find(|env| types.iter().any(|t| env.observation().evidence_type() == t))
}

fn first_selector<'a>(
    evidence: &'a EvidenceSet,
    sel: &EvidenceSelector,
    context: &AssessmentContext,
) -> Option<&'a EvidenceEnvelope> {
    crate::temporal::selector_as_of(evidence, sel, context.as_of())
}

fn is_stale(env: &EvidenceEnvelope, context: &AssessmentContext) -> bool {
    is_stale_candidate(env, context)
}

/// Age vs `max_age` for a still-valid candidate. Expired (outside validity) is not stale.
pub(crate) fn is_stale_candidate(env: &EvidenceEnvelope, context: &AssessmentContext) -> bool {
    let age = context
        .now
        .signed_duration_since(env.provenance().collected_at);
    age.to_std().map(|d| d > context.max_age).unwrap_or(false)
}

/// Expired: outside the half-open validity window. Distinct from stale freshness.
pub fn expired_outside_validity(env: &EvidenceEnvelope, at: DateTime<Utc>) -> bool {
    env.valid_until().is_some_and(|until| at >= until)
}
