use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "weeping-angel",
    version,
    about = "Authorized web recon and security scanning CLI",
    long_about = "Scan only systems you own or have written permission to test.\nRequires --i-own-this and --allow-host."
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
    /// Target URL(s) (http/https)
    pub targets: Vec<String>,

    /// Explicit ownership / authorization consent (required)
    #[arg(long = "i-own-this")]
    pub i_own_this: bool,

    /// Allowlisted host (repeatable). Supports *.example.com
    #[arg(long = "allow-host", value_name = "HOST")]
    pub allow_hosts: Vec<String>,

    /// Optional TOML config file
    #[arg(long, short = 'c')]
    pub config: Option<PathBuf>,

    /// Scan profile: recon | standard | deep
    #[arg(long, default_value = "standard")]
    pub profile: String,

    /// Comma-separated modules (overrides profile defaults when set)
    #[arg(long)]
    pub modules: Option<String>,

    /// Enable active (intrusive) probes — second safety gate
    #[arg(long)]
    pub enable_active: bool,

    /// Active probes: xss,sqli,open-redirect,path-traversal (requires --enable-active)
    #[arg(long, value_name = "LIST")]
    pub probe: Option<String>,

    /// Allow POST/PUT/PATCH/DELETE (default: GET/HEAD only)
    #[arg(long)]
    pub allow_write_methods: bool,

    /// Crawl depth
    #[arg(long, default_value_t = 2)]
    pub depth: u32,

    /// Maximum URLs to retain/fetch
    #[arg(long, default_value_t = 300)]
    pub max_urls: usize,

    /// Concurrent in-flight requests
    #[arg(long, default_value_t = 10)]
    pub concurrency: usize,

    /// Requests per second (global)
    #[arg(long, default_value_t = 5.0)]
    pub rps: f64,

    /// Ignore robots.txt Disallow (authorized pentest mode)
    #[arg(long)]
    pub ignore_robots: bool,

    /// Path to path wordlist
    #[arg(long, default_value = "wordlists/common-paths.txt")]
    pub wordlist: PathBuf,

    /// Extra Cookie header
    #[arg(long)]
    pub cookie: Option<String>,

    /// Extra header Name: Value (repeatable)
    #[arg(long = "header", value_name = "Name: Value")]
    pub headers: Vec<String>,

    /// Request timeout seconds
    #[arg(long, default_value_t = 15)]
    pub timeout: u64,

    /// Accept invalid TLS certificates (lab only)
    #[arg(long)]
    pub insecure: bool,

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
    #[arg(long)]
    pub compare_auth: bool,
}

pub fn parse_header_lines(lines: &[String]) -> anyhow::Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for line in lines {
        let Some((k, v)) = line.split_once(':') else {
            anyhow::bail!("invalid --header (expected 'Name: Value'): {line}");
        };
        out.push((k.trim().to_string(), v.trim().to_string()));
    }
    Ok(out)
}
