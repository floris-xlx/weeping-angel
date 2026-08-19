//! Guard check results and machine-readable report rendering.

use std::time::Duration;

pub const GUARD_REPORT_SCHEMA: &str = "weeping-angel/guard-report/v1";
pub const GUARD_REPORT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    Skip { debt_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub id: String,
    pub name: String,
    pub status: CheckStatus,
}

impl CheckResult {
    pub(crate) fn pass(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Pass,
        }
    }

    pub(crate) fn fail(id: &str, name: &str, message: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Fail(message.into()),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn skip(id: &str, name: &str, debt_id: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: CheckStatus::Skip {
                debt_id: debt_id.into(),
            },
        }
    }

    pub fn report_line(&self) -> String {
        match &self.status {
            CheckStatus::Pass => format!("{}  {}  pass", self.id, self.name),
            CheckStatus::Fail(msg) => format!("{}  {}  fail  {msg}", self.id, self.name),
            CheckStatus::Skip { debt_id } => {
                format!("{}  {}  skip({debt_id})", self.id, self.name)
            }
        }
    }

    pub fn is_fail(&self) -> bool {
        matches!(self.status, CheckStatus::Fail(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardViolation {
    pub check_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardSkip {
    pub check_id: String,
    pub debt_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardCounts {
    pub total: usize,
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardReport {
    pub checks: Vec<CheckResult>,
    pub violations: Vec<GuardViolation>,
    pub skipped: Vec<GuardSkip>,
    pub debt_exemptions: Vec<String>,
    pub duration: Duration,
}

impl GuardReport {
    pub(crate) fn from_checks(checks: Vec<CheckResult>, duration: Duration) -> Self {
        let duration = if duration.is_zero() {
            Duration::from_nanos(1)
        } else {
            duration
        };
        let mut violations = Vec::new();
        let mut skipped = Vec::new();
        let mut debt_exemptions = Vec::new();
        for check in &checks {
            match &check.status {
                CheckStatus::Fail(message) => violations.push(GuardViolation {
                    check_id: check.id.clone(),
                    message: message.clone(),
                }),
                CheckStatus::Skip { debt_id } => {
                    skipped.push(GuardSkip {
                        check_id: check.id.clone(),
                        debt_id: debt_id.clone(),
                    });
                    if !debt_exemptions.iter().any(|id| id == debt_id) {
                        debt_exemptions.push(debt_id.clone());
                    }
                }
                CheckStatus::Pass => {}
            }
        }
        Self {
            checks,
            violations,
            skipped,
            debt_exemptions,
            duration,
        }
    }

    pub fn counts(&self) -> GuardCounts {
        let mut counts = GuardCounts {
            total: self.checks.len(),
            pass: 0,
            fail: 0,
            skip: 0,
        };
        for check in &self.checks {
            match check.status {
                CheckStatus::Pass => counts.pass += 1,
                CheckStatus::Fail(_) => counts.fail += 1,
                CheckStatus::Skip { .. } => counts.skip += 1,
            }
        }
        counts
    }

    pub fn render(&self) -> String {
        let mut out = String::from("cargo xtask guard\n");
        for check in &self.checks {
            out.push_str(&check.report_line());
            out.push('\n');
        }
        out
    }

    pub fn failed(&self) -> bool {
        self.checks.iter().any(CheckResult::is_fail)
    }

    pub fn to_json(&self) -> String {
        let checks: Vec<serde_json::Value> = self
            .checks
            .iter()
            .map(|c| {
                let status = match &c.status {
                    CheckStatus::Pass => serde_json::json!({"kind": "pass"}),
                    CheckStatus::Fail(msg) => serde_json::json!({"kind": "fail", "message": msg}),
                    CheckStatus::Skip { debt_id } => {
                        serde_json::json!({"kind": "skip", "debt_id": debt_id})
                    }
                };
                serde_json::json!({
                    "id": c.id,
                    "name": c.name,
                    "status": status,
                })
            })
            .collect();
        let violations: Vec<serde_json::Value> = self
            .violations
            .iter()
            .map(|v| serde_json::json!({"check_id": v.check_id, "message": v.message}))
            .collect();
        let skipped: Vec<serde_json::Value> = self
            .skipped
            .iter()
            .map(|s| serde_json::json!({"check_id": s.check_id, "debt_id": s.debt_id}))
            .collect();
        let counts = self.counts();
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": GUARD_REPORT_SCHEMA,
            "version": GUARD_REPORT_VERSION,
            "checks": checks,
            "violations": violations,
            "failed": violations,
            "skipped": skipped,
            "debt_exemptions": self.debt_exemptions,
            "counts": {
                "total": counts.total,
                "pass": counts.pass,
                "fail": counts.fail,
                "skip": counts.skip,
            },
            "duration": {
                "secs": self.duration.as_secs(),
                "nanos": self.duration.subsec_nanos(),
                "as_secs_f64": self.duration.as_secs_f64(),
            },
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }
}
