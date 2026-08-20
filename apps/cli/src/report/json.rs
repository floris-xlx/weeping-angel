use anyhow::Result;

use crate::finding::ScanReport;

pub fn to_string(report: &ScanReport) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}
