use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "info" | "informational" => Some(Self::Info),
            "low" => Some(Self::Low),
            "medium" | "med" => Some(Self::Medium),
            "high" => Some(Self::High),
            "critical" | "crit" => Some(Self::Critical),
            _ => None,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub location: String,
    pub snippet: String,
}

impl Evidence {
    pub fn new(location: impl Into<String>, snippet: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            snippet: truncate(snippet.into(), 500),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub url: String,
    pub module: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwe: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    pub found_at: DateTime<Utc>,
}

impl Finding {
    pub fn builder(module: impl Into<String>, id: impl Into<String>) -> FindingBuilder {
        FindingBuilder {
            module: module.into(),
            id: id.into(),
            title: String::new(),
            severity: Severity::Info,
            url: String::new(),
            description: String::new(),
            remediation: None,
            cwe: None,
            evidence: Vec::new(),
        }
    }
}

pub struct FindingBuilder {
    module: String,
    id: String,
    title: String,
    severity: Severity,
    url: String,
    description: String,
    remediation: Option<String>,
    cwe: Option<String>,
    evidence: Vec<Evidence>,
}

impl FindingBuilder {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    pub fn cwe(mut self, cwe: impl Into<String>) -> Self {
        self.cwe = Some(cwe.into());
        self
    }

    pub fn evidence(mut self, evidence: Evidence) -> Self {
        self.evidence.push(evidence);
        self
    }

    pub fn build(self) -> Finding {
        Finding {
            id: self.id,
            title: self.title,
            severity: self.severity,
            url: self.url,
            module: self.module,
            description: self.description,
            remediation: self.remediation,
            cwe: self.cwe,
            evidence: self.evidence,
            found_at: Utc::now(),
        }
    }
}

fn truncate(s: String, max: usize) -> String {
    if s.chars().count() <= max {
        return s;
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub tool: String,
    pub version: String,
    pub target: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub profile: String,
    pub modules: Vec<String>,
    pub discovered_urls: Vec<String>,
    pub findings: Vec<Finding>,
    pub stats: ScanStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanStats {
    pub requests: u64,
    pub urls_discovered: usize,
    pub findings_total: usize,
    pub by_severity: SeverityCounts,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub info: usize,
    pub low: usize,
    pub medium: usize,
    pub high: usize,
    pub critical: usize,
}

impl ScanStats {
    pub fn from_findings(findings: &[Finding], requests: u64, urls: usize) -> Self {
        let mut by: SeverityCounts = SeverityCounts::default();
        for f in findings {
            match f.severity {
                Severity::Info => by.info += 1,
                Severity::Low => by.low += 1,
                Severity::Medium => by.medium += 1,
                Severity::High => by.high += 1,
                Severity::Critical => by.critical += 1,
            }
        }
        Self {
            requests,
            urls_discovered: urls,
            findings_total: findings.len(),
            by_severity: by,
        }
    }
}
