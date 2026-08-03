//! Shared scan artifact path conventions (Codex Security compatible).

pub const MANIFEST_FILE: &str = "scan-manifest.json";
pub const FINDINGS_FILE: &str = "findings.json";
pub const COVERAGE_FILE: &str = "coverage.json";
pub const REPORT_MD: &str = "report.md";

pub const ARTIFACTS_DIR: &str = "artifacts";
pub const CONTEXT_DIR: &str = "artifacts/01_context";
pub const DISCOVERY_DIR: &str = "artifacts/02_discovery";
pub const COVERAGE_DIR: &str = "artifacts/03_coverage";
pub const RECONCILIATION_DIR: &str = "artifacts/04_reconciliation";
pub const FINDINGS_DIR: &str = "artifacts/05_findings";

pub const IN_SCOPE_FILES: &str = "artifacts/02_discovery/in_scope_files.txt";
pub const CANDIDATE_LEDGER: &str = "artifacts/02_discovery/candidate_ledger.jsonl";
pub const THREAT_MODEL_MD: &str = "artifacts/01_context/threat_model.md";
pub const SECURITY_GUIDANCE_MD: &str = "artifacts/01_context/security_guidance.md";
