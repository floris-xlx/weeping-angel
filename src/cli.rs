use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::parse::{self, LogHttp};

#[derive(Debug, Parser)]
#[command(
    name = "weeping-angel",
    version,
    about = "Authorized web recon and security scanning CLI",
    long_about = "Scan only systems you own or have written permission to test.\n\
Requires --i-own-this and --allow-host (or --allow-host-from-target).\n\n\
Targets accept bare hosts (example.com), //host, http://, or https://.\n\
Consent: --i-own-this or --i-own-this=true|yes|1 (value requires =)."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Increase logging verbosity (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run an authorized security scan
    Scan(ScanArgs),
}

#[derive(Debug, Clone, Parser)]
pub struct ScanArgs {
    /// Target host(s) or URL(s): example.com, //host/path, http(s)://…
    pub targets: Vec<String>,

    /// Ownership / authorization consent (required). Use bare flag or =true|yes|1
    #[arg(
        long = "i-own-this",
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_consent,
        value_name = "BOOL"
    )]
    pub i_own_this: Option<bool>,

    /// Allowlisted host (repeatable; CSV ok). Supports *.example.com and full URLs
    #[arg(long = "allow-host", value_name = "HOST", action = clap::ArgAction::Append)]
    pub allow_hosts: Vec<String>,

    /// Add each target's host to the allowlist (still requires --i-own-this)
    #[arg(long = "allow-host-from-target")]
    pub allow_host_from_target: bool,

    /// When scheme is omitted, prefer http instead of smart https/http default
    #[arg(long = "prefer-http")]
    pub prefer_http: bool,

    /// Optional TOML config file
    #[arg(long, short = 'c')]
    pub config: Option<PathBuf>,

    /// Scan profile: recon | standard | deep (aliases: quick, full)
    #[arg(long, default_value = "standard")]
    pub profile: String,

    /// Comma/space-separated modules (overrides profile defaults when set)
    #[arg(long)]
    pub modules: Option<String>,

    /// Enable active (intrusive) probes — second safety gate
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub enable_active: Option<bool>,

    /// Active probes: xss,sqli,open-redirect,path-traversal (requires --enable-active)
    #[arg(long, value_name = "LIST")]
    pub probe: Option<String>,

    /// Allow POST/PUT/PATCH/DELETE (default: GET/HEAD only)
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub allow_write_methods: Option<bool>,

    /// Crawl depth
    #[arg(long, default_value_t = 2)]
    pub depth: u32,

    /// Maximum URLs to retain/fetch
    #[arg(long, default_value_t = 300)]
    pub max_urls: usize,

    /// Concurrent in-flight requests
    #[arg(long, default_value_t = 20)]
    pub concurrency: usize,

    /// Requests per second (global)
    #[arg(long, default_value_t = 15.0)]
    pub rps: f64,

    /// Speed preset: higher rps/concurrency, summary HTTP log, skip image OPTIONS
    #[arg(long)]
    pub fast: bool,

    /// Ignore robots.txt Disallow (authorized pentest mode)
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub ignore_robots: Option<bool>,

    /// Path to path wordlist
    #[arg(long, default_value = "wordlists/common-paths.txt")]
    pub wordlist: PathBuf,

    /// Extra Cookie header (repeatable; values merged)
    #[arg(long = "cookie", value_name = "COOKIE", action = clap::ArgAction::Append)]
    pub cookies: Vec<String>,

    /// Extra header Name: Value or Name=Value (repeatable)
    #[arg(long = "header", value_name = "Name: Value", action = clap::ArgAction::Append)]
    pub headers: Vec<String>,

    /// Request timeout seconds
    #[arg(long, default_value_t = 15)]
    pub timeout: u64,

    /// Accept invalid TLS certificates (lab only)
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub insecure: Option<bool>,

    /// Output path prefix/file for json/sarif/html
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,

    /// Formats: terminal,json,sarif,html,manifest,openapi,images (comma-separated)
    #[arg(long, default_value = "terminal")]
    pub format: String,

    /// Exit 1 when findings at or above this severity: low|medium|high|critical
    #[arg(long, default_value = "medium")]
    pub fail_on: String,

    /// Directory of YAML path templates (Nuclei-lite). Default: templates/
    #[arg(long, default_value = "templates")]
    pub templates_dir: PathBuf,

    /// Compare authenticated (--cookie/--header) vs anonymous requests
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub compare_auth: Option<bool>,

    /// Live HTTP log density: full | compact | summary | off
    #[arg(long = "log-http", value_name = "MODE")]
    pub log_http: Option<String>,

    /// Max discovered routes printed in terminal report
    #[arg(long, default_value_t = 120)]
    pub max_terminal_routes: usize,

    /// Terminal report width (0 = auto-detect)
    #[arg(long, default_value_t = 0)]
    pub report_width: usize,
}

impl ScanArgs {
    pub fn consent(&self) -> bool {
        self.i_own_this.unwrap_or(false)
    }

    pub fn enable_active(&self) -> bool {
        self.enable_active.unwrap_or(false)
    }

    pub fn allow_write_methods(&self) -> bool {
        self.allow_write_methods.unwrap_or(false)
    }

    pub fn ignore_robots(&self) -> bool {
        self.ignore_robots.unwrap_or(false)
    }

    pub fn compare_auth(&self) -> bool {
        self.compare_auth.unwrap_or(false)
    }

    pub fn insecure(&self) -> bool {
        self.insecure.unwrap_or(false)
    }

    pub fn cookie_header(&self) -> Option<String> {
        if self.cookies.is_empty() {
            None
        } else {
            Some(self.cookies.join("; "))
        }
    }

    pub fn log_http_mode(&self) -> LogHttp {
        if let Some(s) = &self.log_http {
            return LogHttp::parse(s).unwrap_or(LogHttp::Compact);
        }
        if self.fast {
            return LogHttp::Summary;
        }
        let rps = if self.fast { 40.0 } else { self.rps };
        let conc = if self.fast { 40 } else { self.concurrency };
        if rps > 10.0 || conc > 8 {
            LogHttp::Compact
        } else {
            LogHttp::Full
        }
    }

    pub fn effective_rps(&self) -> f64 {
        if self.fast {
            self.rps.max(40.0)
        } else {
            self.rps
        }
    }

    pub fn effective_concurrency(&self) -> usize {
        if self.fast {
            self.concurrency.max(40)
        } else {
            self.concurrency
        }
    }
}

pub fn parse_header_lines(lines: &[String]) -> anyhow::Result<Vec<(String, String)>> {
    parse::parse_header_lines(lines)
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatCli {
    Terminal,
    Json,
    Sarif,
    Html,
    Manifest,
    Openapi,
    Images,
}
