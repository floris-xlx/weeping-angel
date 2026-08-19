//! Execution for `weeping-angel assurance soa`. Parser lives in `cli.rs`.

use anyhow::{Result, bail};
use weeping_angel_assurance::{pin_soa_snapshot, project_soa, project_soa_from_snapshot};
use weeping_angel_evidence::EvidenceLedger;

use crate::cli::AssuranceSoaArgs;

const NOT_CERTIFICATION: &str = "This is a readiness assessment and is not certification.";

pub fn run(args: AssuranceSoaArgs) -> Result<i32> {
    println!("{NOT_CERTIFICATION}");
    let assessment = args.assessment.trim();
    if assessment.is_empty() || assessment.eq_ignore_ascii_case("latest") {
        let soa = project_soa("iso-27001", "2022");
        println!("{}", serde_json::to_string_pretty(&soa)?);
        return Ok(0);
    }
    let payload = load_pinned_soa(assessment)?;
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(0)
}

fn load_pinned_soa(assessment_id: &str) -> Result<serde_json::Value> {
    let path = std::path::Path::new("assurance-ledger.sqlite");
    if path.is_file() {
        let ledger = EvidenceLedger::open(path)?;
        let payload = ledger.load_assessment_run(assessment_id)?;
        let value: serde_json::Value = serde_json::from_str(&payload)?;
        if let Some(snapshot) = value
            .get("soa")
            .cloned()
            .or_else(|| value.get("statementOfApplicability").cloned())
        {
            if let Ok(soa) = serde_json::from_value::<
                weeping_angel_assurance::StatementOfApplicability,
            >(snapshot.clone())
            {
                let pack = value
                    .get("frameworkPackDigest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unpinned");
                let pinned = pin_soa_snapshot(soa, pack);
                return Ok(serde_json::to_value(project_soa_from_snapshot(&pinned))?);
            }
            return Ok(snapshot);
        }
        return Ok(value);
    }
    bail!("unknown assessment {assessment_id}");
}
