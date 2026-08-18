//! Offline control-test evaluation. Provider-blind. Zero network I/O.

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{ControlId, ControlTestId};
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceType};

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
    InsufficientEvidence,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ControlTestResult {
    pub test_id: ControlTestId,
    pub control_id: ControlId,
    pub effectiveness: Effectiveness,
    pub rationale: String,
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

    pub fn build(self) -> CompiledControlTest {
        CompiledControlTest {
            id: self.id.expect("CompiledControlTest.id"),
            control_id: self.control_id.expect("CompiledControlTest.control_id"),
            kind: self.kind.unwrap_or(ControlTestKind::Automated),
            required: self.required,
            break_on: self.break_on,
        }
    }
}

pub fn evaluate(
    test: &CompiledControlTest,
    evidence: &EvidenceSet,
    context: &AssessmentContext,
) -> ControlTestResult {
    let mut result = ControlTestResult {
        test_id: test.id.clone(),
        control_id: test.control_id.clone(),
        effectiveness: Effectiveness::InsufficientEvidence,
        rationale: String::new(),
    };

    let envelopes: Vec<&EvidenceEnvelope> = evidence.iter().collect();

    if let Some(broken) = first_matching(&envelopes, &test.break_on) {
        result.effectiveness = Effectiveness::Ineffective;
        result.rationale = format!(
            "breaking observation {} on {}",
            broken.observation().evidence_type(),
            broken.provenance().asset()
        );
        return result;
    }

    if test.kind == ControlTestKind::Manual {
        let attested = first_matching(&envelopes, &[EvidenceType::new("manual_attestation")]);
        match attested {
            None => {
                result.effectiveness = Effectiveness::InsufficientEvidence;
                result.rationale = "manual control cannot auto-pass without attestation".into();
                return result;
            }
            Some(env) if is_stale(env, context) => {
                result.effectiveness = Effectiveness::Inconclusive;
                result.rationale = "manual attestation is stale".into();
                return result;
            }
            Some(_) => {}
        }
    }

    if test.required.is_empty() {
        result.effectiveness = Effectiveness::InsufficientEvidence;
        result.rationale = "no required evidence types; absence is not effectiveness".into();
        return result;
    }

    for required in &test.required {
        match first_matching(&envelopes, std::slice::from_ref(required)) {
            None => {
                result.effectiveness = Effectiveness::InsufficientEvidence;
                result.rationale = format!("missing required evidence {required}");
                return result;
            }
            Some(env) if is_stale(env, context) => {
                result.effectiveness = Effectiveness::Inconclusive;
                result.rationale = format!("stale required evidence {required}");
                return result;
            }
            Some(_) => {}
        }
    }

    result.effectiveness = Effectiveness::Effective;
    result.rationale = "fresh matching observations satisfy the compiled test".into();
    result
}

fn first_matching<'a>(
    envelopes: &[&'a EvidenceEnvelope],
    types: &[EvidenceType],
) -> Option<&'a EvidenceEnvelope> {
    envelopes.iter().copied().find(|env| {
        types
            .iter()
            .any(|t| env.observation().evidence_type() == t)
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
