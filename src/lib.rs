//! weeping-angel — authorized web recon and security scanning library.

pub mod authz;
pub mod checks;
pub mod cli;
pub mod config;
pub mod discovery;
pub mod docs_export;
pub mod engine;
pub mod finding;
pub mod http;
pub mod parse;
pub mod report;
pub mod style;
pub mod target;
pub mod templates;

use std::collections::HashSet;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tracing::info;

use crate::authz::Authorization;
use crate::cli::ScanArgs;
use crate::config::{FileConfig, Profile, merge_hosts};
use crate::engine::{ScanOptions, run_scan};
use crate::finding::Severity;
use crate::http::ClientConfig;
use crate::parse::{expand_list_args, normalize_allow_hosts, split_list};
use crate::report::{Format, write_reports};
use crate::target::{NormalizeOptions, normalize_targets};

pub async fn run_scan_command(args: ScanArgs) -> Result<i32> {
    let file_cfg: FileConfig = if let Some(path) = &args.config {
        FileConfig::load(path).with_context(|| format!("load config {}", path.display()))?
    } else {
        FileConfig::default()
    };

    let i_own_this: bool = args.consent() || file_cfg.authorization.i_own_this;
    let enable_active: bool = args.enable_active() || file_cfg.authorization.enable_active;
    let allow_write: bool =
        args.allow_write_methods() || file_cfg.authorization.allow_write_methods;

    if args.targets.is_empty() {
        bail!(
            "provide at least one target (example.com, //host/path, http://…, or https://…)"
        );
    }

    let norm_opts = NormalizeOptions {
        prefer_http: args.prefer_http,
    };
    let normalized: Vec<String> = normalize_targets(&args.targets, norm_opts)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut host_list: Vec<String> = normalize_allow_hosts(
        args.allow_hosts
            .iter()
            .cloned()
            .chain(file_cfg.authorization.allow_hosts.iter().cloned()),
    );
    if args.allow_host_from_target {
        for t in &normalized {
            if let Some(h) = target::host_of_normalized(t) {
                host_list.push(h);
            }
        }
    }
    let hosts: HashSet<String> = merge_hosts(host_list, Vec::new());

    let authz: Authorization = Authorization::new(
        i_own_this,
        hosts.into_iter().collect::<Vec<_>>(),
        enable_active,
        allow_write,
    );

    let targets: Vec<url::Url> = authz.validate_targets(&normalized)?;

    let profile_name: String = file_cfg
        .scan
        .profile
        .clone()
        .unwrap_or_else(|| args.profile.clone());
    let profile: Profile = Profile::parse(&profile_name)
        .with_context(|| format!("unknown profile: {profile_name} (recon|standard|deep|quick|full)"))?;

    let modules: Vec<String> = if let Some(m) = &args.modules {
        split_list(m)
    } else if !file_cfg.scan.modules.is_empty() {
        expand_list_args(&file_cfg.scan.modules)
    } else {
        profile
            .default_modules()
            .into_iter()
            .map(str::to_string)
            .collect()
    };

    let probes: Vec<String> = args
        .probe
        .as_deref()
        .map(split_list)
        .unwrap_or_default();

    if (!probes.is_empty() || modules.iter().any(|m| m == "active")) && !enable_active {
        bail!("active probes requested but --enable-active was not set (second safety gate)");
    }

    let fail_on: Severity =
        Severity::from_str_loose(file_cfg.scan.fail_on.as_deref().unwrap_or(&args.fail_on))
            .with_context(|| format!("invalid --fail-on {}", args.fail_on))?;

    let mut extra_headers: Vec<(String, String)> = cli::parse_header_lines(&args.headers)?;
    for h in &file_cfg.headers {
        if let Ok(parsed) = parse::parse_header_lines(&[h.clone()]) {
            extra_headers.extend(parsed);
        }
    }

    let concurrency = file_cfg
        .scan
        .concurrency
        .unwrap_or_else(|| args.effective_concurrency());
    let rps = file_cfg.scan.rps.unwrap_or_else(|| args.effective_rps());
    let log_http = args.log_http_mode();

    let client_cfg: ClientConfig = ClientConfig {
        timeout: Duration::from_secs(args.timeout),
        max_redirects: 5,
        max_body_bytes: 2 * 1024 * 1024,
        concurrency,
        rps,
        extra_headers,
        cookie: args.cookie_header(),
        insecure_tls: args.insecure(),
        log_http,
    };

    // Auto-include auth-compare when --compare-auth and not already listed
    let mut modules: Vec<String> = modules;
    if args.compare_auth() && !modules.iter().any(|m| m == "auth-compare") {
        modules.push("auth-compare".into());
    }

    let opts: ScanOptions = ScanOptions {
        targets,
        profile,
        modules: modules.clone(),
        depth: file_cfg.scan.depth.unwrap_or(args.depth),
        max_urls: file_cfg.scan.max_urls.unwrap_or(args.max_urls),
        ignore_robots: file_cfg
            .scan
            .ignore_robots
            .unwrap_or_else(|| args.ignore_robots()),
        wordlist: args.wordlist.clone(),
        probes,
        fail_on: Some(fail_on),
        templates_dir: args.templates_dir.clone(),
        compare_auth: args.compare_auth(),
        skip_image_options: args.fast,
        max_terminal_routes: args.max_terminal_routes,
        report_width: args.report_width,
    };

    crate::style::init();
    crate::style::set_log_http(log_http);
    crate::style::eprint_line(&format!(
        "{} {} {}  {}={}  {}={}",
        crate::style::brand("weeping-angel"),
        crate::style::ok("authorized scan"),
        crate::style::bold(&normalized.join(", ")),
        crate::style::cyan("profile"),
        crate::style::bright_magenta(profile.as_str()),
        crate::style::cyan("modules"),
        crate::style::dim(&modules.join(",")),
    ));
    crate::style::eprint_line(&format!(
        "{} rate ~{} req/s · concurrency {} · log-http={}{}",
        crate::style::brand("weeping-angel"),
        client_cfg.rps,
        client_cfg.concurrency,
        log_http.as_str(),
        if args.fast {
            " · fast preset"
        } else {
            ""
        },
    ));
    info!("consent OK; starting scan");

    let report: finding::ScanReport = run_scan(authz, client_cfg, opts.clone()).await?;

    let formats: Vec<Format> = Format::parse_list(&args.format);
    let formats: Vec<Format> = if formats.is_empty() {
        vec![Format::Terminal]
    } else {
        formats
    };

    write_reports(
        &report,
        &formats,
        args.output.as_deref(),
        opts.max_terminal_routes,
        opts.report_width,
    )?;

    Ok(exit_code_for(&report, fail_on))
}

pub fn exit_code_for(report: &crate::finding::ScanReport, fail_on: Severity) -> i32 {
    if report
        .findings
        .iter()
        .any(|f| f.severity >= fail_on && f.severity >= Severity::Low)
    {
        // When fail_on is Info, still only fail on Low+ to avoid noise from route discovery
        if fail_on == Severity::Info {
            if report.findings.iter().any(|f| f.severity >= Severity::Low) {
                return 1;
            }
            return 0;
        }
        return 1;
    }
    0
}
