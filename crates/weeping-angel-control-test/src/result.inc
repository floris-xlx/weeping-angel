#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlTestResult {
    pub test_id: ControlTestId,
    pub control_id: ControlId,
    pub effectiveness: Effectiveness,
    pub rationale: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub missing_evidence: Vec<String>,
    #[serde(rename = "evaluatedAt", default = "default_checked_at")]
    pub checked_at: DateTime<Utc>,
    #[serde(default = "default_test_version")]
    pub test_version: String,
    #[serde(default)]
    pub input_digest: String,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub status: Option<Effectiveness>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub population: Option<crate::PopulationEvaluation>,
}
