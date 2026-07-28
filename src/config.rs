use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub authorization: AuthConfig,
    #[serde(default)]
    pub scan: ScanConfigFile,
    #[serde(default)]
    pub headers: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub i_own_this: bool,
    #[serde(default)]
    pub allow_hosts: Vec<String>,
    #[serde(default)]
    pub enable_active: bool,
    #[serde(default)]
    pub allow_write_methods: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScanConfigFile {
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub depth: Option<u32>,
    #[serde(default)]
    pub max_urls: Option<usize>,
    #[serde(default)]
    pub concurrency: Option<usize>,
    #[serde(default)]
    pub rps: Option<f64>,
    #[serde(default)]
    pub ignore_robots: Option<bool>,
    #[serde(default)]
    pub fail_on: Option<String>,
}

impl FileConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&raw)?;
        Ok(cfg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Recon,
    Standard,
    Deep,
}

impl Profile {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "recon" => Some(Self::Recon),
            "standard" => Some(Self::Standard),
            "deep" => Some(Self::Deep),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Recon => "recon",
            Self::Standard => "standard",
            Self::Deep => "deep",
        }
    }

    pub fn default_modules(self) -> Vec<&'static str> {
        match self {
            Self::Recon => vec![
                "discovery",
                "headers",
                "tls",
                "cookies",
                "secrets",
                "exposures",
                "tech",
                "firebase",
            ],
            Self::Standard => vec![
                "discovery",
                "headers",
                "tls",
                "cookies",
                "secrets",
                "exposures",
                "tech",
                "firebase",
                "cors",
                "auth-surface",
                "rate-limits",
                "wordlist",
                "templates",
            ],
            Self::Deep => vec![
                "discovery",
                "headers",
                "tls",
                "cookies",
                "secrets",
                "exposures",
                "tech",
                "firebase",
                "cors",
                "auth-surface",
                "auth-compare",
                "rate-limits",
                "wordlist",
                "openapi",
                "templates",
            ],
        }
    }
}

pub fn merge_hosts(cli: Vec<String>, file: Vec<String>) -> HashSet<String> {
    cli.into_iter().chain(file).collect()
}
