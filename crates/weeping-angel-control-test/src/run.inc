pub fn evaluate(
    test: &CompiledControlTest,
    evidence: &EvidenceSet,
    context: &AssessmentContext,
) -> ControlTestResult {
    let envelopes: Vec<&EvidenceEnvelope> = evidence.iter().collect();
    let mut evidence_refs: Vec<String> = Vec::new();
    let mut missing_evidence: Vec<String> = Vec::new();

    let mut result = ControlTestResult {
        test_id: test.id.clone(),
        control_id: test.control_id.clone(),
        effectiveness: Effectiveness::InsufficientEvidence,
        rationale: String::new(),
        evidence_refs: Vec::new(),
        missing_evidence: Vec::new(),
        checked_at: context.now,
        test_version: "1".into(),
        input_digest: String::new(),
        duration: None,
        status: None,
        reason: None,
    };

    if let Some(broken) = first_matching(&envelopes, &test.break_on) {
        result.effectiveness = Effectiveness::Ineffective;
        result.rationale = format!(
            "breaking observation {} on {}",
            broken.observation().evidence_type(),
            broken.provenance().asset()
        );
        evidence_refs.push(broken.digest().to_string());
        return finalize(result, evidence, &evidence_refs, &missing_evidence);
    }

    if let Some(expr) = &test.expr {
        return run_expr(test, expr, &envelopes, evidence, context);
    }

    if test.kind == ControlTestKind::Manual {
        let attested = first_matching(&envelopes, &[EvidenceType::new("manual_attestation")]);
        match attested {
            None => {
                result.effectiveness = Effectiveness::ManualReviewRequired;
                result.rationale = "manual control cannot auto-pass without attestation".into();
                missing_evidence.push("manual_attestation".into());
                return finalize(result, evidence, &evidence_refs, &missing_evidence);
            }
            Some(env) if is_stale(env, context) => {
                result.effectiveness = Effectiveness::StaleEvidence;
                result.rationale = "manual attestation is stale".into();
                evidence_refs.push(env.digest().to_string());
                return finalize(result, evidence, &evidence_refs, &missing_evidence);
            }
            Some(env) => evidence_refs.push(env.digest().to_string()),
        }
    }

    if test.required.is_empty() {
        result.effectiveness = Effectiveness::InsufficientEvidence;
        result.rationale = "no required evidence types; absence is not effectiveness".into();
        return finalize(result, evidence, &evidence_refs, &missing_evidence);
    }

    for required in &test.required {
        match first_matching(&envelopes, std::slice::from_ref(required)) {
            None => {
                result.effectiveness = Effectiveness::InsufficientEvidence;
                result.rationale = format!("missing required evidence {required}");
                missing_evidence.push(required.to_string());
                return finalize(result, evidence, &evidence_refs, &missing_evidence);
            }
            Some(env) if is_stale(env, context) => {
                result.effectiveness = Effectiveness::StaleEvidence;
                result.rationale = format!("stale required evidence {required}");
                evidence_refs.push(env.digest().to_string());
                return finalize(result, evidence, &evidence_refs, &missing_evidence);
            }
            Some(env) => evidence_refs.push(env.digest().to_string()),
        }
    }

    result.effectiveness = Effectiveness::Effective;
    result.rationale = "fresh matching observations satisfy the compiled test".into();
    finalize(result, evidence, &evidence_refs, &missing_evidence)
}

fn run_expr(
    test: &CompiledControlTest,
    expr: &TestExpr,
    envelopes: &[&EvidenceEnvelope],
    evidence: &EvidenceSet,
    context: &AssessmentContext,
) -> ControlTestResult {
    let mut refs = Vec::new();
    let mut missing = Vec::new();
    let outcome = eval_node(expr, envelopes, context, &mut refs, &mut missing);
    let result = ControlTestResult {
        test_id: test.id.clone(),
        control_id: test.control_id.clone(),
        effectiveness: outcome.effectiveness,
        rationale: outcome.rationale,
        evidence_refs: Vec::new(),
        missing_evidence: Vec::new(),
        checked_at: context.now,
        test_version: "1".into(),
        input_digest: String::new(),
        duration: None,
        status: None,
        reason: None,
    };
    finalize(result, evidence, &refs, &missing)
}
