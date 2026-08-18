use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::parse::{self, LogHttp};

const CLAP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

const AFTER_HELP: &str = "\
Examples:
  weeping-angel -v
  weeping-angel --version
  weeping-angel version
  weeping-angel completions powershell
  weeping-angel scan example.com --i-own-this --allow-host example.com
  weeping-angel scan-code . -o out/code --fail-on high
  weeping-angel scan-diff --repo . -o out/diff --base main --head HEAD
  weeping-angel workbench list
  weeping-angel depcheck package.json
  weeping-angel depcheck -l npm package.json
  weeping-angel depcheck -l npm -s '@mycompany/*' package.json
  weeping-angel depcheck --provider pypi --path ./project
  weeping-angel depcheck --provider npm --dependency left-pad:1.3.0 --check-email
  weeping-angel depcheck -d ./app -i --entrypoint index.js
  cat hosts.txt | weeping-angel depcheck --stdin --i-own-this --threads 20
  weeping-angel depcheck --hosts-file hosts.txt --i-own-this --link --silent
  weeping-angel depcheck --list Cargo.lock
  weeping-angel depcheck --web --port 8443

Version flags work without a subcommand: -v, -V, --version.
Web scans require --i-own-this and --allow-host (or --allow-host-from-target).
Increase engine logs with --verbose (repeat: --verbose --verbose).
";

/// Multi-line version string used by `-v` / `--version` / `version`.
pub const VERSION_LINE: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    env!("CARGO_PKG_DESCRIPTION")
);

#[derive(Debug, Parser)]
#[command(
    name = "weeping-angel",
    version = VERSION_LINE,
    about = "Authorized security toolchain (web recon, code SAST, depcheck)",
    long_about = "Authorized security toolchain: live web recon/DAST, algorithmic code scans, and dependency-confusion detection (depcheck).\n\n\
Web scans require --i-own-this and --allow-host (or --allow-host-from-target).\n\
Targets accept bare hosts (example.com), //host, http://, or https://.\n\
Consent: --i-own-this or --i-own-this=true|yes|1 (value requires =).\n\n\
Code/diff scans produce Codex Security–compatible sealed bundles.\n\
depcheck is detection-only (no auto-publish / exploit payloads).",
    after_help = AFTER_HELP,
    arg_required_else_help = true,
    propagate_version = true,
    disable_version_flag = true,
    styles = CLAP_STYLES,
    help_template = "\
{before-help}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}",
    next_line_help = false,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Print version (-v, -V, --version). Works without a subcommand.
    #[arg(
        short = 'v',
        short_alias = 'V',
        long = "version",
        action = clap::ArgAction::Version,
        global = true,
        help_heading = "Meta"
    )]
    version: (),

    /// Increase logging verbosity (repeatable)
    #[arg(long, action = clap::ArgAction::Count, global = true, help_heading = "Meta")]
    pub verbose: u8,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run an authorized live web security scan
    #[command(visible_alias = "s")]
    Scan(ScanArgs),

    /// Validate, fingerprint, seal, and project report.md for a scan bundle
    #[command(visible_alias = "seal")]
    Finalize(FinalizeArgs),

    /// Algorithmic code SAST (full tree or scoped path) → sealed contract
    #[command(name = "scan-code", visible_alias = "code")]
    ScanCode(ScanCodeArgs),

    /// Algorithmic code SAST on a Git change-set (PR/commit/working-tree)
    #[command(name = "scan-diff", visible_alias = "diff")]
    ScanDiff(ScanDiffArgs),

    /// Local SQLite workbench: register / list sealed scans
    #[command(visible_alias = "wb")]
    Workbench(WorkbenchArgs),

    /// Dependency confusion scanner (multi-format, detection only)
    #[command(visible_alias = "dc")]
    Depcheck(DepcheckArgs),

    /// Automated readiness / assurance (not certification)
    Assurance(AssuranceArgs),

    /// Print version and package description
    Version,

    /// Write shell completions to stdout
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Debug, Clone, clap::Args)]
pub struct AssuranceArgs {
    #[command(subcommand)]
    pub command: AssuranceCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AssuranceCommand {
    /// Framework pack list / validate / show
    Framework(AssuranceFrameworkArgs),
    /// Collect evidence
    Collect(AssuranceCollectArgs),
    /// List, show, or add evidence
    Evidence(AssuranceEvidenceArgs),
    /// Run a readiness assessment
    Assess(AssuranceAssessArgs),
    /// Show a stored result
    Result(AssuranceResultArgs),
    /// Compare two assessment snapshots
    Compare(AssuranceCompareArgs),
    /// Statement of Applicability projection
    Soa(AssuranceSoaArgs),
}

#[derive(Debug, Clone, clap::Args)]
pub struct AssuranceFrameworkArgs {
    #[command(subcommand)]
    pub command: AssuranceFrameworkCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AssuranceFrameworkCommand {
    List,
    Validate {
        #[arg(default_value = "frameworks/iso-27001/2022")]
        path: PathBuf,
    },
    Show {
        framework: String,
    },
}

#[derive(Debug, Clone, clap::Args)]
pub struct AssuranceCollectArgs {
    #[arg(long)]
    pub collector: Option<String>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct AssuranceEvidenceArgs {
    #[command(subcommand)]
    pub command: AssuranceEvidenceCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AssuranceEvidenceCommand {
    List,
    Show {
        #[arg(default_value = "latest")]
        id: String,
    },
    Add {
        #[arg(long = "type")]
        evidence_type: String,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long = "attested-by")]
        attested_by: String,
    },
}

#[derive(Debug, Clone, clap::Args)]
pub struct AssuranceAssessArgs {
    #[arg(long)]
    pub framework: String,
    #[arg(long, default_value = ".")]
    pub scope: PathBuf,
    #[arg(long)]
    pub github_repo: Option<String>,
    #[arg(long)]
    pub github_token_env: Option<String>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct AssuranceResultArgs {
    #[command(subcommand)]
    pub command: AssuranceResultCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AssuranceResultCommand {
    Show {
        #[arg(default_value = "latest")]
        id: String,
    },
}

#[derive(Debug, Clone, clap::Args)]
pub struct AssuranceCompareArgs {
    #[arg(default_value = "previous")]
    pub before: String,
    #[arg(default_value = "latest")]
    pub after: String,
}

#[derive(Debug, Clone, clap::Args)]
pub struct AssuranceSoaArgs {
    #[arg(default_value = "latest")]
    pub assessment: String,
}

impl Cli {
    /// Build the clap `Command` used by the binary and completion generator.
    pub fn clap_command() -> clap::Command {
        Self::command()
    }

    /// True when argv is only a version request (`-v` / `-V` / `--version` / `version`).
    pub fn argv_is_version_only(args: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
        let mut saw_version = false;
        for arg in args {
            let a = arg.as_ref();
            if a == "--" {
                break;
            }
            if matches!(a, "-v" | "-V" | "--version" | "version") {
                saw_version = true;
                continue;
            }
            if a.starts_with('-') {
                continue;
            }
            return false;
        }
        saw_version
    }
}

#[derive(Debug, Clone, Parser)]
pub struct WorkbenchArgs {
    #[command(subcommand)]
    pub command: WorkbenchCommand,

    /// Override workbench SQLite path (default: ~/.weeping-angel/workbench.sqlite3)
    #[arg(long, global = true)]
    pub db: Option<PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum WorkbenchCommand {
    /// Index a sealed scan directory
    Register {
        /// Path to sealed scan_dir (has findings.json + scan-manifest.json)
        #[arg(long = "scan-dir")]
        scan_dir: PathBuf,
    },
    /// List recent registered scans
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show one registered scan by id
    Show { scan_id: String },
    /// Compare two sealed scans by primary fingerprint (introduced / resolved / persistent)
    Compare {
        /// Earlier scan directory
        #[arg(long)]
        before: PathBuf,
        /// Later scan directory
        #[arg(long)]
        after: PathBuf,
        /// Optional path to write compare JSON (default: after/compare.json)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Generate algorithmic remediation patches for findings in a sealed scan
    #[command(name = "generate-patches")]
    GeneratePatches {
        /// Sealed scan directory (reads findings.json; writes remediation/)
        #[arg(long = "scan-dir")]
        scan_dir: PathBuf,
        /// Source tree root (paths in findings are relative to this)
        #[arg(long = "source-root")]
        source_root: PathBuf,
    },
    /// Apply a generated unified-diff patch under source-root
    #[command(name = "apply-patch")]
    ApplyPatch {
        /// Source tree root
        #[arg(long = "source-root")]
        source_root: PathBuf,
        /// Path to fix.patch (from generate-patches)
        #[arg(long)]
        patch: PathBuf,
    },
    /// Re-scan one file and confirm a rule_id no longer matches
    Verify {
        /// Source tree root
        #[arg(long = "source-root")]
        source_root: PathBuf,
        /// Path relative to source-root
        #[arg(long)]
        path: String,
        /// Engine rule id (e.g. command-injection.shell-true)
        #[arg(long = "rule-id")]
        rule_id: String,
    },
}

#[derive(Debug, Clone, Parser)]
pub struct FinalizeArgs {
    /// Scan directory containing scan-manifest.json, findings.json, coverage.json
    #[arg(long = "scan-dir", value_name = "DIR")]
    pub scan_dir: PathBuf,
}

#[derive(Debug, Clone, Parser)]
pub struct DepcheckArgs {
    /// Dependency file or project directory (omit with --url / --web / --dependency)
    #[arg(conflicts_with = "dependency")]
    pub target: Option<PathBuf>,

    /// Path to folder(s) to analyze (DepFuzzer / Loki alias for positional target)
    #[arg(long, short = 'd', visible_alias = "directory", conflicts_with_all = ["target", "dependency", "url"])]
    pub path: Option<PathBuf>,

    /// Fetch dependency file from URL (requires --i-own-this)
    #[arg(long, short = 'u')]
    pub url: Option<String>,

    /// Check one dependency: NAME or NAME:VERSION (DepFuzzer-compatible)
    #[arg(long, value_name = "NAME[:VERSION]")]
    pub dependency: Option<String>,

    /// Package repository system. Values: npm, pip/pypi, composer, mvn/maven, gradle, rubygems, cargo, go, nuget, all
    #[arg(long, short = 'l', visible_alias = "provider", value_name = "LANG")]
    pub language: Option<String>,

    /// List packages only (no registry checks)
    #[arg(long)]
    pub list: bool,

    /// Known-secure namespaces (confused `-s`). Comma-separated; `*` wildcards. Repeatable.
    #[arg(long = "secure-namespace", short = 's', value_name = "PATTERN", action = clap::ArgAction::Append)]
    pub secure_namespaces: Vec<String>,

    /// Stream takeover candidates as they are found (DepFuzzer `--print-takeover`)
    #[arg(long = "print-takeover")]
    pub print_takeover: bool,

    /// Write takeover candidates to a text file (NAME:VERSION per line)
    #[arg(long = "output-file", value_name = "PATH")]
    pub output_file: Option<PathBuf>,

    /// For packages that exist: check maintainer emails for disposable / purchasable domains (DepFuzzer `--check-email`)
    #[arg(long = "check-email")]
    pub check_email: bool,

    /// Also walk transitive deps via deps.dev (DepFuzzer-style; slower)
    #[arg(long)]
    pub transitive: bool,

    /// Convert extracted packages to package.json for other tools
    #[arg(long)]
    pub convert: bool,

    /// Export scan results JSON
    #[arg(long, short = 'e', value_name = "PATH")]
    pub export: Option<PathBuf>,

    /// Concurrent registry checks (default: 20)
    #[arg(long, short = 't', default_value_t = 20)]
    pub threads: usize,

    /// Registry request timeout seconds (default: 10)
    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    /// Override file type detection (e.g. package_lock_json, requirements_txt)
    #[arg(long = "type", value_name = "KIND")]
    pub file_type: Option<String>,

    /// Only print vulnerable package names
    #[arg(long, short = 'q')]
    pub quiet: bool,

    /// Verbose scan progress (confused `-v` style)
    #[arg(long = "scan-verbose")]
    pub scan_verbose: bool,

    /// Loki inspector: show git commit that introduced each free-namespace dependency
    #[arg(long = "inspect", short = 'i')]
    pub inspect: bool,

    /// Application entry file for impact context (Loki `--entrypoint`; defaults to package.json main / index.js)
    #[arg(long = "entrypoint", value_name = "FILE")]
    pub entrypoint: Option<String>,

    /// Skip npm hardening recon (.npmrc / floating ranges / scopes)
    #[arg(long = "no-hardening")]
    pub no_hardening: bool,

    /// DepenFusion-style: read hosts/URLs from stdin and probe package.json + package-lock.json
    #[arg(long)]
    pub stdin: bool,

    /// File of hosts/URLs to probe (one per line; DepenFusion-style remote hunt)
    #[arg(long = "hosts-file", value_name = "PATH")]
    pub hosts_file: Option<PathBuf>,

    /// Append string to each probe URL (DepenFusion `-a`, e.g. `?token=foo`)
    #[arg(long = "append", value_name = "SUFFIX", default_value = "")]
    pub append: String,

    /// Ignore path from input URLs; probe host root only (DepenFusion `-p`)
    #[arg(long = "strip-path")]
    pub strip_path: bool,

    /// Print full https://registry.npmjs.org/… links for missing packages (DepenFusion `-link`)
    #[arg(long = "link")]
    pub link: bool,

    /// Silent remote-hunt output: only missing package names/links (DepenFusion `-s` silent)
    #[arg(long = "silent")]
    pub silent: bool,

    /// Start scan-only local Web UI
    #[arg(long)]
    pub web: bool,

    /// Web UI port (default: 8443)
    #[arg(long, default_value_t = 8443)]
    pub port: u16,

    /// Web UI bind address (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: String,

    /// Ownership consent required when using --url
    #[arg(
        long = "i-own-this",
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_consent,
        value_name = "BOOL"
    )]
    pub i_own_this: Option<bool>,
}

impl DepcheckArgs {
    pub fn consent(&self) -> bool {
        self.i_own_this == Some(true)
    }
}

#[derive(Debug, Clone, Parser)]
pub struct ScanCodeArgs {
    /// Repository or directory root to inventory
    pub path: PathBuf,

    /// Output scan directory (created if missing)
    #[arg(long = "scan-dir", short = 'o', value_name = "DIR")]
    pub scan_dir: PathBuf,

    /// Optional path scope under the root (default: entire tree)
    #[arg(long)]
    pub scope: Option<String>,

    /// Exit 1 when findings at or above this severity: low|medium|high|critical (default: low)
    #[arg(long, default_value = "low")]
    pub fail_on: String,
}

#[derive(Debug, Clone, Parser)]
pub struct ScanDiffArgs {
    /// Git repository root (defaults to cwd)
    #[arg(long, default_value = ".")]
    pub repo: PathBuf,

    /// Output scan directory (created if missing)
    #[arg(long = "scan-dir", short = 'o', value_name = "DIR")]
    pub scan_dir: PathBuf,

    /// Base revision (e.g. main, origin/main, commit SHA). Omit for working-tree mode.
    #[arg(long)]
    pub base: Option<String>,

    /// Head revision (default HEAD). Used with --base for PR/branch/commit ranges.
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// Scan staged+unstaged+untracked changes vs base (default HEAD) instead of a revision range
    #[arg(long)]
    pub working_tree: bool,

    /// Exit 1 when findings at or above this severity: low|medium|high|critical (default: medium)
    #[arg(long, default_value = "medium")]
    pub fail_on: String,
}

#[derive(Debug, Clone, Parser)]
pub struct ScanArgs {
    /// Target host(s) or URL(s): example.com, //host/path, http(s)://…
    pub targets: Vec<String>,

    /// Ownership / authorization consent (required). Use bare flag or =true|yes|1
    #[arg(
        help_heading = "Safety",
        long = "i-own-this",
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_consent,
        value_name = "BOOL"
    )]
    pub i_own_this: Option<bool>,

    /// Allowlisted host (repeatable; CSV ok). Supports *.example.com and full URLs
    #[arg(
        help_heading = "Safety",
        long = "allow-host",
        value_name = "HOST",
        action = clap::ArgAction::Append
    )]
    pub allow_hosts: Vec<String>,

    /// Add each target's host to the allowlist (still requires --i-own-this)
    #[arg(help_heading = "Safety", long = "allow-host-from-target")]
    pub allow_host_from_target: bool,

    /// When scheme is omitted, prefer http instead of smart https/http default
    #[arg(help_heading = "Request", long = "prefer-http")]
    pub prefer_http: bool,

    /// Optional TOML config file
    #[arg(help_heading = "Scan", long, short = 'c')]
    pub config: Option<PathBuf>,

    /// Scan profile: recon | standard | deep (aliases: quick, full)
    #[arg(help_heading = "Scan", long, default_value = "standard")]
    pub profile: String,

    /// Comma/space-separated modules (overrides profile defaults when set)
    #[arg(help_heading = "Scan", long)]
    pub modules: Option<String>,

    /// Enable active (intrusive) probes — second safety gate
    #[arg(
        help_heading = "Safety",
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub enable_active: Option<bool>,

    /// Active probes: xss,sqli,open-redirect,path-traversal (requires --enable-active)
    #[arg(help_heading = "Safety", long, value_name = "LIST")]
    pub probe: Option<String>,

    /// Allow POST/PUT/PATCH/DELETE (default: GET/HEAD only)
    #[arg(
        help_heading = "Safety",
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub allow_write_methods: Option<bool>,

    /// Crawl depth
    #[arg(help_heading = "Scan", long, default_value_t = 2)]
    pub depth: u32,

    /// Maximum URLs to retain/fetch
    #[arg(help_heading = "Scan", long, default_value_t = 300)]
    pub max_urls: usize,

    /// Concurrent in-flight requests
    #[arg(help_heading = "Request", long, default_value_t = 20)]
    pub concurrency: usize,

    /// Requests per second (global)
    #[arg(help_heading = "Request", long, default_value_t = 15.0)]
    pub rps: f64,

    /// Speed preset: higher rps/concurrency, summary HTTP log, skip image OPTIONS
    #[arg(help_heading = "Request", long)]
    pub fast: bool,

    /// Ignore robots.txt Disallow (authorized pentest mode)
    #[arg(
        help_heading = "Safety",
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub ignore_robots: Option<bool>,

    /// Path to path wordlist
    #[arg(
        help_heading = "Scan",
        long,
        default_value = "wordlists/common-paths.txt"
    )]
    pub wordlist: PathBuf,

    /// Extra Cookie (`name=value` or `name value`; repeatable; values merged)
    #[arg(
        help_heading = "Request",
        long = "cookie",
        value_name = "COOKIE",
        num_args = 1..=2,
        action = clap::ArgAction::Append
    )]
    pub cookies: Vec<String>,

    /// Extra header: `Name: Value`, `Name=Value`, or `Name Value` (repeatable)
    #[arg(
        help_heading = "Request",
        long = "header",
        value_name = "HEADER",
        num_args = 1..=2,
        action = clap::ArgAction::Append
    )]
    pub headers: Vec<String>,

    /// Request timeout seconds
    #[arg(help_heading = "Request", long, default_value_t = 15)]
    pub timeout: u64,

    /// Accept invalid TLS certificates (lab only)
    #[arg(
        help_heading = "Request",
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub insecure: Option<bool>,

    /// Output path prefix/file for json/sarif/html
    #[arg(help_heading = "Output", long, short = 'o')]
    pub output: Option<PathBuf>,

    /// Formats: terminal,json,sarif,html,manifest,openapi,images (comma-separated)
    #[arg(help_heading = "Output", long, default_value = "terminal")]
    pub format: String,

    /// Exit 1 when findings at or above this severity: low|medium|high|critical
    #[arg(help_heading = "Output", long, default_value = "medium")]
    pub fail_on: String,

    /// Directory of YAML path templates (Nuclei-lite). Default: templates/
    #[arg(help_heading = "Scan", long, default_value = "templates")]
    pub templates_dir: PathBuf,

    /// Compare authenticated (--cookie/--header) vs anonymous requests
    #[arg(
        help_heading = "Scan",
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        value_parser = parse::parse_optional_bool,
        value_name = "BOOL"
    )]
    pub compare_auth: Option<bool>,

    /// Live HTTP log density: full | compact | summary | off
    #[arg(help_heading = "Output", long = "log-http", value_name = "MODE")]
    pub log_http: Option<String>,

    /// Max discovered routes printed in terminal report
    #[arg(help_heading = "Output", long, default_value_t = 120)]
    pub max_terminal_routes: usize,

    /// Terminal report width (0 = auto-detect)
    #[arg(help_heading = "Output", long, default_value_t = 0)]
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
        parse::cookie_header_from_args(&self.cookies)
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
