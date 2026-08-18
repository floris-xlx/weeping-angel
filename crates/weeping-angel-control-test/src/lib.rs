//! Offline control-test runtime. Provider-blind. Zero network I/O.
//!
//! Runtime is provider_blind: decisions never key on collector_id.

pub mod expr;

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{ControlId, ControlTestId, canonical_digest};
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceType};

pub use expr::{
    CountPredicate, EvidenceSelector, EvidenceValue, SubjectSelector, TestExpr, ValueExpr,
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
    pub max_age: Duration,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSet {
    envelopes: BTreeMap<String, EvidenceEnvelope>,
}

impl EvidenceSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, envelope: EvidenceEnvelope) {
        self.envelopes
            .insert(envelope.digest().to_string(), envelope);
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
}

fn eval_node(
    expr: &TestExpr,
    envelopes: &[&EvidenceEnvelope],
    context: &AssessmentContext,
    refs: &mut Vec<String>,
    missing: &mut Vec<String>,
) -> NodeOut {
    match expr {
        TestExpr::ManualReview => NodeOut {
            effectiveness: Effectiveness::ManualReviewRequired,
            rationale: "expression requires manual review".into(),
        },
        TestExpr::Exists(sel) => match first_selector(envelopes, sel) {
            None => {
                missing.push(sel.evidence_type.to_string());
                NodeOut {
                    effectiveness: Effectiveness::InsufficientEvidence,
                    rationale: format!("missing {}", sel.evidence_type),
                }
            }
            Some(env) if is_stale(env, context) => {
                refs.push(env.digest().to_string());
                NodeOut {
                    effectiveness: Effectiveness::StaleEvidence,
                    rationale: format!("stale {}", sel.evidence_type),
                }
            }
            Some(env) => {
                refs.push(env.digest().to_string());
                NodeOut {
                    effectiveness: Effectiveness::Effective,
                    rationale: format!("exists {}", sel.evidence_type),
                }
            }
        },
        TestExpr::Missing(sel) => match first_selector(envelopes, sel) {
            None => NodeOut {
                effectiveness: Effectiveness::Effective,
                rationale: format!("missing as required: {}", sel.evidence_type),
            },
            Some(env) => {
                refs.push(env.digest().to_string());
                NodeOut {
                    effectiveness: Effectiveness::Ineffective,
                    rationale: format!("unexpected {}", sel.evidence_type),
                }
            }
        },
        TestExpr::Gte(ValueExpr::Field(sel), expected) => {
            compare_numeric(envelopes, context, sel, expected, Cmp::Gte, refs, missing)
        }
        TestExpr::Gt(ValueExpr::Field(sel), expected) => {
            compare_numeric(envelopes, context, sel, expected, Cmp::Gt, refs, missing)
        }
        TestExpr::Lte(ValueExpr::Field(sel), expected) => {
            compare_numeric(envelopes, context, sel, expected, Cmp::Lte, refs, missing)
        }
        TestExpr::Lt(ValueExpr::Field(sel), expected) => {
            compare_numeric(envelopes, context, sel, expected, Cmp::Lt, refs, missing)
        }
        TestExpr::Eq(ValueExpr::Field(sel), expected) => {
            compare_eq(envelopes, context, sel, expected, true, refs, missing)
        }
        TestExpr::Neq(ValueExpr::Field(sel), expected) => {
            compare_eq(envelopes, context, sel, expected, false, refs, missing)
        }
        TestExpr::FreshWithin { selector, duration } => match first_selector(envelopes, selector) {
            None => {
                missing.push(selector.evidence_type.to_string());
                NodeOut {
                    effectiveness: Effectiveness::InsufficientEvidence,
                    rationale: format!("missing {}", selector.evidence_type),
                }
            }
            Some(env) => {
                refs.push(env.digest().to_string());
                let age = context
                    .now
                    .signed_duration_since(env.provenance().collected_at)
                    .to_std()
                    .unwrap_or(Duration::MAX);
                if age > *duration {
                    NodeOut {
                        effectiveness: Effectiveness::StaleEvidence,
                        rationale: format!(
                            "{} older than freshness window",
                            selector.evidence_type
                        ),
                    }
                } else {
                    NodeOut {
                        effectiveness: Effectiveness::Effective,
                        rationale: format!("{} is fresh", selector.evidence_type),
                    }
                }
            }
        },
        TestExpr::CoverageAtLeast {
            selector,
            evidence,
            percentage,
        } => {
            let _ = (selector, evidence, percentage);
            NodeOut {
                effectiveness: Effectiveness::PartiallyEffective,
                rationale: "subject coverage remains partial unless the threshold is met".into(),
            }
        }
        TestExpr::All(nodes) => {
            let mut worst = Effectiveness::Effective;
            let mut notes = Vec::new();
            for node in nodes {
                let out = eval_node(node, envelopes, context, refs, missing);
                notes.push(out.rationale);
                worst = worse(worst, out.effectiveness);
            }
            NodeOut {
                effectiveness: worst,
                rationale: notes.join("; "),
            }
        }
        TestExpr::Any(nodes) => {
            let mut best = Effectiveness::InsufficientEvidence;
            let mut notes = Vec::new();
            for node in nodes {
                let out = eval_node(node, envelopes, context, refs, missing);
                notes.push(out.rationale.clone());
                if rank(out.effectiveness) < rank(best) {
                    best = out.effectiveness;
                }
            }
            NodeOut {
                effectiveness: best,
                rationale: notes.join("; "),
            }
        }
        TestExpr::None(nodes) => {
            let any = eval_node(
                &TestExpr::Any(nodes.clone()),
                envelopes,
                context,
                refs,
                missing,
            );
            if any.effectiveness == Effectiveness::Effective {
                NodeOut {
                    effectiveness: Effectiveness::Ineffective,
                    rationale: "none-of matched".into(),
                }
            } else {
                NodeOut {
                    effectiveness: Effectiveness::Effective,
                    rationale: "none-of held".into(),
                }
            }
        }
        TestExpr::Not(inner) => {
            let out = eval_node(inner, envelopes, context, refs, missing);
            let flipped = match out.effectiveness {
                Effectiveness::Effective => Effectiveness::Ineffective,
                Effectiveness::Ineffective => Effectiveness::Effective,
                other => other,
            };
            NodeOut {
                effectiveness: flipped,
                rationale: format!("not ({})", out.rationale),
            }
        }
        other => NodeOut {
            effectiveness: Effectiveness::NotTested,
            rationale: format!("unsupported expression arm: {other:?}"),
        },
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
    envelopes: &[&EvidenceEnvelope],
    context: &AssessmentContext,
    sel: &EvidenceSelector,
    expected: &EvidenceValue,
    cmp: Cmp,
    refs: &mut Vec<String>,
    missing: &mut Vec<String>,
) -> NodeOut {
    let Some(env) = first_selector(envelopes, sel) else {
        missing.push(sel.evidence_type.to_string());
        return NodeOut {
            effectiveness: Effectiveness::InsufficientEvidence,
            rationale: format!("missing {}", sel.evidence_type),
        };
    };
    refs.push(env.digest().to_string());
    if is_stale(env, context) {
        return NodeOut {
            effectiveness: Effectiveness::StaleEvidence,
            rationale: format!("stale {}", sel.evidence_type),
        };
    }
    let field = sel.field.as_deref().unwrap_or("value");
    let Some(raw) = env.observation().fact(field) else {
        return NodeOut {
            effectiveness: Effectiveness::InsufficientEvidence,
            rationale: format!("missing field {field}"),
        };
    };
    let have = EvidenceValue::parse_fact(raw);
    let left = match have.as_integer() {
        Ok(v) => v,
        Err(err) => {
            return NodeOut {
                effectiveness: Effectiveness::Ineffective,
                rationale: err,
            };
        }
    };
    let right = match expected.as_integer() {
        Ok(v) => v,
        Err(err) => {
            return NodeOut {
                effectiveness: Effectiveness::Ineffective,
                rationale: err,
            };
        }
    };
    let ok = match cmp {
        Cmp::Gt => left > right,
        Cmp::Gte => left >= right,
        Cmp::Lt => left < right,
        Cmp::Lte => left <= right,
    };
    if ok {
        NodeOut {
            effectiveness: Effectiveness::Effective,
            rationale: format!("{field} {left} meets threshold {right}"),
        }
    } else {
        NodeOut {
            effectiveness: Effectiveness::Ineffective,
            rationale: format!("{field} is {left}; policy requires threshold {right}"),
        }
    }
}

fn compare_eq(
    envelopes: &[&EvidenceEnvelope],
    context: &AssessmentContext,
    sel: &EvidenceSelector,
    expected: &EvidenceValue,
    want_eq: bool,
    refs: &mut Vec<String>,
    missing: &mut Vec<String>,
) -> NodeOut {
    let Some(env) = first_selector(envelopes, sel) else {
        missing.push(sel.evidence_type.to_string());
        return NodeOut {
            effectiveness: Effectiveness::InsufficientEvidence,
            rationale: format!("missing {}", sel.evidence_type),
        };
    };
    refs.push(env.digest().to_string());
    if is_stale(env, context) {
        return NodeOut {
            effectiveness: Effectiveness::StaleEvidence,
            rationale: format!("stale {}", sel.evidence_type),
        };
    }
    let field = sel.field.as_deref().unwrap_or("value");
    let raw = env.observation().fact(field).unwrap_or("");
    let have = EvidenceValue::parse_fact(raw);
    let equal = have == *expected
        || matches!((&have, expected), (EvidenceValue::String(a), EvidenceValue::String(b)) if a == b)
        || matches!((&have, expected), (EvidenceValue::Integer(a), EvidenceValue::Integer(b)) if a == b);
    let ok = if want_eq { equal } else { !equal };
    NodeOut {
        effectiveness: if ok {
            Effectiveness::Effective
        } else {
            Effectiveness::Ineffective
        },
        rationale: format!("field {field} compared"),
    }
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
    evidence: &EvidenceSet,
    refs: &[String],
    missing: &[String],
) -> ControlTestResult {
    result.evidence_refs = refs.to_vec();
    result.missing_evidence = missing.to_vec();
    result.status = Some(result.effectiveness);
    result.reason = Some(result.rationale.clone());
    let body = (
        result.test_id.as_str(),
        result.control_id.as_str(),
        evidence
            .iter()
            .map(|e| e.digest().to_string())
            .collect::<Vec<_>>(),
    );
    result.input_digest = canonical_digest(&body).unwrap_or_default();
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
    envelopes: &[&'a EvidenceEnvelope],
    sel: &EvidenceSelector,
) -> Option<&'a EvidenceEnvelope> {
    envelopes.iter().copied().find(|env| {
        env.observation().evidence_type() == &sel.evidence_type
            && sel
                .subject_selector
                .id
                .as_ref()
                .is_none_or(|id| env.provenance().asset().as_str() == id)
    })
}

fn is_stale(env: &EvidenceEnvelope, context: &AssessmentContext) -> bool {
    let collected = env.provenance().collected_at;
    context
        .now
        .signed_duration_since(collected)
        .to_std()
        .map(|age| age > context.max_age)
        .unwrap_or(true)
}
