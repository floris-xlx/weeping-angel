//! Execution for `weeping-angel assurance explain`. Parser lives in `cli.rs`.

use anyhow::{Result, bail};
use serde_json::{Value, json};
use weeping_angel_evidence::EvidenceLedger;

use crate::cli::AssuranceExplainArgs;

const NOT_CERTIFICATION: &str = "This is a readiness assessment and is not certification.";

pub fn run(args: AssuranceExplainArgs) -> Result<i32> {
    println!("{NOT_CERTIFICATION}");
    let payload = load_assessment_payload(&args.assessment)?;
    let explanation = project_explanation(&payload, &args.control)?;
    println!("{}", serde_json::to_string_pretty(&explanation)?);
    Ok(0)
}

pub fn explain(assessment: &str, control: &str) -> Result<i32> {
    run(AssuranceExplainArgs {
        assessment: assessment.into(),
        control: control.into(),
    })
}

fn load_assessment_payload(assessment_id: &str) -> Result<Value> {
    let path = std::path::Path::new("assurance-ledger.sqlite");
    if path.is_file() {
        let ledger = EvidenceLedger::open(path)?;
        let payload = ledger.load_assessment_run(assessment_id)?;
        return Ok(serde_json::from_str(&payload)?);
    }
    bail!("unknown assessment {assessment_id}");
}

fn project_explanation(payload: &Value, control_id: &str) -> Result<Value> {
    let results = payload
        .get("results")
        .or_else(|| payload.get("assessmentRun"))
        .cloned()
        .unwrap_or(Value::Null);
    let matching = results.as_array().and_then(|arr| {
        arr.iter().find(|r| {
            r.get("controlId")
                .and_then(Value::as_str)
                .is_some_and(|id| id == control_id)
        })
    });
    let Some(result) = matching else {
        bail!("unknown control {control_id}");
    };
    Ok(json!({
        "control": { "id": control_id },
        "applicability": result.get("applicability"),
        "implementation": result.get("implementation"),
        "population": result.get("population"),
        "tests": [{
            "id": result.get("testId"),
            "test_version": result.get("testVersion"),
        }],
        "evidence_requirements": result.get("evidenceRequirements"),
        "evidence": result.get("evidenceRefs"),
        "missing_evidence": result.get("missingEvidence"),
        "failing_subjects": result.pointer("/population/failingSubjects"),
        "missing_subjects": result.pointer("/population/missingSubjects"),
        "exceptions": result.get("exceptions"),
        "mappings": result.get("mappings"),
        "effectiveness": result.get("effectiveness"),
    }))
}
