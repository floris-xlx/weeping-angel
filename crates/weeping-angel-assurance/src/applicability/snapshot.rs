//! In-memory applicability snapshot for later lineage persist.

use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{
    ApplicabilityRule, AssessmentDefinition, AssessmentId, AssessmentScope, canonical_digest,
};

use super::context::ApplicabilityContext;
use weeping_angel_framework::Assessment;

use super::evaluator::{
    ApplicabilityDecision, ApplicabilityOutcome, ExcludedSubject, PredicateTrace, RationaleEntry,
    UnknownFact, evaluate_applicability_for_subjects,
};

pub const APPLICABILITY_SNAPSHOT_SCHEMA: &str = "weeping-angel/applicability-snapshot/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackApplicabilityEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applicable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicabilityItemDecision {
    pub id: String,
    pub rule: ApplicabilityRule,
    pub rule_digest: String,
    pub decision: ApplicabilityDecision,
    pub rationale: Vec<RationaleEntry>,
    pub predicates: Vec<PredicateTrace>,
    pub unknown_facts: Vec<UnknownFact>,
    pub selected_subjects: Vec<String>,
    pub excluded_subjects: Vec<ExcludedSubject>,
}

impl ApplicabilityItemDecision {
    fn from_outcome(id: String, rule: ApplicabilityRule, outcome: ApplicabilityOutcome) -> Self {
        let rule_digest = canonical_digest(&rule).unwrap_or_default();
        Self {
            id,
            rule,
            rule_digest,
            decision: outcome.decision,
            rationale: outcome.rationale,
            predicates: outcome.predicates,
            unknown_facts: outcome.unknown_facts,
            selected_subjects: outcome.selected_subjects,
            excluded_subjects: outcome.excluded_subjects,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicabilitySnapshot {
    pub schema: String,
    pub assessment_id: AssessmentId,
    pub scope: AssessmentScope,
    pub requirement_decisions: Vec<ApplicabilityItemDecision>,
    pub control_decisions: Vec<ApplicabilityItemDecision>,
    pub pack_entries: Vec<PackApplicabilityEntry>,
    pub digest: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotDigestBody<'a> {
    schema: &'a str,
    assessment_id: &'a AssessmentId,
    scope: &'a AssessmentScope,
    requirement_decisions: &'a [ApplicabilityItemDecision],
    control_decisions: &'a [ApplicabilityItemDecision],
    pack_entries: &'a [PackApplicabilityEntry],
}

pub fn evaluate_assessment_applicability(
    definition: &AssessmentDefinition,
    context: &ApplicabilityContext,
) -> ApplicabilitySnapshot {
    let mut requirement_decisions: Vec<ApplicabilityItemDecision> = definition
        .requirements
        .iter()
        .map(|req| {
            let outcome =
                evaluate_applicability_for_subjects(req.applicability(), context, Some(&[]));
            ApplicabilityItemDecision::from_outcome(
                req.id().as_str().to_string(),
                req.applicability().clone(),
                outcome,
            )
        })
        .collect();
    requirement_decisions.sort_by(|a, b| a.id.cmp(&b.id));

    let mut control_decisions: Vec<ApplicabilityItemDecision> = definition
        .controls
        .iter()
        .map(|control| {
            let outcome = evaluate_applicability_for_subjects(
                control.applicability(),
                context,
                Some(control.subjects()),
            );
            ApplicabilityItemDecision::from_outcome(
                control.id().as_str().to_string(),
                control.applicability().clone(),
                outcome,
            )
        })
        .collect();
    control_decisions.sort_by(|a, b| a.id.cmp(&b.id));

    let mut snapshot = ApplicabilitySnapshot {
        schema: APPLICABILITY_SNAPSHOT_SCHEMA.into(),
        assessment_id: definition.id.clone(),
        scope: definition.scope.clone(),
        requirement_decisions,
        control_decisions,
        pack_entries: context.pack_entries.clone(),
        digest: String::new(),
    };
    snapshot.digest = snapshot_digest(&snapshot);
    snapshot
}

impl ApplicabilitySnapshot {
    pub fn recompute_digest(&mut self) {
        self.digest = snapshot_digest(self);
    }
}

/// Pin compiled-framework static applicability into the canonical snapshot type
/// (DUP-004). Full Kleene evaluation remains [`evaluate_assessment_applicability`].
pub fn pin_compiled_applicability(
    assessment: &Assessment,
    scope_label: &str,
) -> ApplicabilitySnapshot {
    let scope = AssessmentScope {
        organizations: vec![scope_label.to_string()],
        ..AssessmentScope::default()
    };
    let requirement_decisions = assessment
        .requirements
        .iter()
        .map(|req| {
            static_item_decision(
                req.id().to_string(),
                req.applicability().clone(),
                "static applicability from ApplicabilityRule; unresolved predicates stay included",
            )
        })
        .collect();
    let control_decisions = assessment
        .controls
        .iter()
        .map(|ctl| {
            static_item_decision(
                ctl.id().to_string(),
                ctl.applicability().clone(),
                "static applicability from ApplicabilityRule",
            )
        })
        .collect();
    let mut snapshot = ApplicabilitySnapshot {
        schema: APPLICABILITY_SNAPSHOT_SCHEMA.into(),
        assessment_id: assessment.id.clone(),
        scope,
        requirement_decisions,
        control_decisions,
        pack_entries: Vec::new(),
        digest: String::new(),
    };
    snapshot.recompute_digest();
    snapshot
}

fn static_item_decision(
    id: String,
    rule: ApplicabilityRule,
    rationale: &str,
) -> ApplicabilityItemDecision {
    let decision = match rule.statically_applicable() {
        Some(true) => ApplicabilityDecision::Applicable,
        Some(false) => ApplicabilityDecision::NotApplicable,
        None => ApplicabilityDecision::ManualDeterminationRequired,
    };
    ApplicabilityItemDecision {
        id,
        rule_digest: canonical_digest(&rule).unwrap_or_default(),
        rule,
        decision,
        rationale: vec![RationaleEntry {
            code: "static".into(),
            message: rationale.into(),
        }],
        predicates: Vec::new(),
        unknown_facts: Vec::new(),
        selected_subjects: Vec::new(),
        excluded_subjects: Vec::new(),
    }
}

fn snapshot_digest(snapshot: &ApplicabilitySnapshot) -> String {
    let body = SnapshotDigestBody {
        schema: &snapshot.schema,
        assessment_id: &snapshot.assessment_id,
        scope: &snapshot.scope,
        requirement_decisions: &snapshot.requirement_decisions,
        control_decisions: &snapshot.control_decisions,
        pack_entries: &snapshot.pack_entries,
    };
    canonical_digest(&body).unwrap_or_default()
}
