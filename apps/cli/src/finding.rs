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

/// Structured route discovered during recon (canonical source for reports).
///
/// Prefer this over re-parsing free-text `route-discovered` findings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteRecord {
    pub url: String,
    pub path: String,
    #[serde(default = "default_get")]
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// Discovery source: crawl, wordlist, robots, sitemap, js, spa, image-*, …
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

fn default_get() -> String {
    "GET".into()
}

impl RouteRecord {
    pub fn from_asset(asset: &crate::discovery::DiscoveredAsset) -> Self {
        Self {
            url: asset.url.as_str().to_string(),
            path: asset.url.path().to_string(),
            method: "GET".into(),
            status: Some(asset.status),
            source: asset.source.clone(),
            content_type: asset.content_type.clone(),
            tags: Vec::new(),
        }
    }
}

/// Whether a finding is inventory noise (routes/images) rather than a security issue.
pub fn is_inventory_finding(f: &Finding) -> bool {
    if f.id == "route-discovered"
        || f.id == "image-head-ok"
        || f.id == "image-asset"
        || f.id == "image-hosting-pattern"
        || f.id.starts_with("image-")
    {
        return true;
    }
    f.module == "discovery" && f.severity == Severity::Info && f.id.contains("route")
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
    /// Structured route inventory (prefer over parsing discovery findings).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<RouteRecord>,
    pub findings: Vec<Finding>,
    pub stats: ScanStats,
    /// Full image path harvest (HEAD + OPTIONS preflight + sources).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_harvest: Option<crate::discovery::image_harvest::ImageHarvestManifest>,
    /// Per-phase wall timings (seconds).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phases: Vec<PhaseTiming>,
    /// Module run summaries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_results: Vec<ModuleSummary>,
    /// Aggregated attack surface inventory.
    #[serde(default)]
    pub surface: SurfaceInventory,
    /// Fingerprinted tech tokens.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tech_stack: Vec<String>,
    /// Wall-clock + request accounting.
    #[serde(default)]
    pub timing: TimingSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhaseTiming {
    pub name: String,
    pub seconds: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleSummary {
    pub id: String,
    pub ran: bool,
    pub findings: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SurfaceInventory {
    #[serde(default)]
    pub routes_by_source: Vec<SourceCount>,
    #[serde(default)]
    pub status_histogram: Vec<StatusCount>,
    #[serde(default)]
    pub content_types: Vec<SourceCount>,
    #[serde(default)]
    pub total_routes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceCount {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatusCount {
    pub status: u16,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimingSummary {
    pub wall_seconds: f64,
    pub requests: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_rps: Option<f64>,
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
