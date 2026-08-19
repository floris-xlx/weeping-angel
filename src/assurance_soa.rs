//! Execution for `weeping-angel assurance soa`. Parser lives in `cli.rs`.

use anyhow::{Result, bail};
use weeping_angel_assurance::{
    LineageBundle, pin_soa_snapshot, project_soa_from_snapshot, replay_assessment,
};
use weeping_angel_evidence::EvidenceLedger;

use crate::cli::AssuranceSoaArgs;

const NOT_CERTIFICATION: &str = "This is a readiness assessment and is not certification.";

pub fn run(args: AssuranceSoaArgs) -> Result<i32> {
    println!("{NOT_CERTIFICATION}");
    let assessment = args.assessment.trim();
    if assessment.is_empty() || assessment.eq_ignore_ascii_case("latest") {
        let soa = historical_soa_latest()?;
        println!("{}", serde_json::to_string_pretty(&soa)?);
        return Ok(0);
    }
    let payload = load_pinned_soa(assessment)?;
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(0)
}

fn historical_soa_latest() -> Result<serde_json::Value> {
    let path = std::path::Path::new("assurance-ledger.sqlite");
    if !path.is_file() {
        bail!("historical SoA requires a pinned assessment; refusing live project_soa as history");
    }
    let ledger = EvidenceLedger::open(path)?;
    let runs = ledger.list_assessment_runs()?;
    let Some((_id, payload)) = runs.last() else {
        bail!("no pinned assessment in the ledger; replay_assessment cannot reconstruct latest");
    };
    historical_soa_from_payload(payload)
}

fn historical_soa_from_payload(payload: &str) -> Result<serde_json::Value> {
    if let Ok(bundle) = serde_json::from_str::<LineageBundle>(payload) {
        let _report = replay_assessment(&bundle).map_err(|e| anyhow::anyhow!("{e}"))?;
        return Ok(serde_json::to_value(project_soa_from_snapshot(
            &bundle.soa,
        ))?);
    }
    bail!(
        "selected historical assessment cannot be reconstructed exactly; replay_assessment would fail"
    )
}

fn load_pinned_soa(assessment_id: &str) -> Result<serde_json::Value> {
    let path = std::path::Path::new("assurance-ledger.sqlite");
    if path.is_file() {
        let ledger = EvidenceLedger::open(path)?;
        let payload = ledger.load_assessment_run(assessment_id)?;
        if let Ok(value) = historical_soa_from_payload(&payload) {
            return Ok(value);
        }
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
                if pack.is_empty() || pack.eq_ignore_ascii_case("unpinned") {
                    bail!("named assessment {assessment_id} is missing a pack pin");
                }
                let pinned = pin_soa_snapshot(soa, pack);
                return Ok(serde_json::to_value(project_soa_from_snapshot(&pinned))?);
            }
            return Ok(snapshot);
        }
        bail!("named assessment {assessment_id} cannot be reconstructed exactly");
    }
    bail!("unknown assessment {assessment_id}");
}
