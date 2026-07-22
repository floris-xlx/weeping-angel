//! weeping-angel — authorized web recon and security scanning library.

pub mod authz;
pub mod checks;
pub mod cli;
pub mod config;
pub mod discovery;
pub mod engine;
pub mod finding;
pub mod http;
pub mod report;
pub mod templates;

use std::time::Duration;

use anyhow::{bail, Context, Result};
use tracing::info;

use crate::authz::Authorization;
use crate::cli::ScanArgs;
use crate::config::{merge_hosts, FileConfig, Profile};
use crate::engine::{run_scan, ScanOptions};
use crate::finding::Severity;
use crate::http::ClientConfig;
use crate::report::{write_reports, Format};

pub async fn run_scan_command(args: ScanArgs) -> Result<i32> {
    let file_cfg = if let Some(path) = &args.config {
        FileConfig::load(path).with_context(|| format!("load config {}", path.display()))?
    } else {
        FileConfig::default()
    };

    let i_own_this = args.i_own_this || file_cfg.authorization.i_own_this;
    let enable_active = args.enable_active || file_cfg.authorization.enable_active;
    let allow_write = args.allow_write_methods || file_cfg.authorization.allow_write_methods;

    let mut hosts = merge_hosts(
        args.allow_hosts.clone(),
        file_cfg.authorization.allow_hosts.clone(),
    );

    // Convenience: if user passed targets but forgot allow-host, do NOT auto-add —
    // security default. They must be explicit.

    let authz = Authorization::new(
        i_own_this,
        hosts.drain().collect::<Vec<_>>(),
        enable_active,
        allow_write,
    );

    if args.targets.is_empty() {
        bail!("provide at least one target URL");
    }

    let targets = authz.validate_targets(&args.targets)?;

    // Auto-suggest: if allowlist empty we already error; good.

    let profile_name = file_cfg
        .scan
        .profile
        .clone()
        .unwrap_or_else(|| args.profile.clone());
    let profile = Profile::parse(&profile_name)
        .with_context(|| format!("unknown profile: {profile_name}"))?;

    let modules: Vec<String> = if let Some(m) = &args.modules {
        m.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    } else if !file_cfg.scan.modules.is_empty() {
        file_cfg.scan.modules.clone()
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
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if (!probes.is_empty() || modules.iter().any(|m| m == "active")) && !enable_active {
        bail!(
            "active probes requested but --enable-active was not set (second safety gate)"
        );
    }

    let fail_on = Severity::from_str_loose(
        file_cfg
            .scan
            .fail_on
            .as_deref()
            .unwrap_or(&args.fail_on),
    )
    .with_context(|| format!("invalid --fail-on {}", args.fail_on))?;

    let mut extra_headers = cli::parse_header_lines(&args.headers)?;
    for h in &file_cfg.headers {
        if let Some((k, v)) = h.split_once(':') {
            extra_headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }

    let client_cfg = ClientConfig {
        timeout: Duration::from_secs(args.timeout),
        max_redirects: 5,
        max_body_bytes: 2 * 1024 * 1024,
        concurrency: file_cfg.scan.concurrency.unwrap_or(args.concurrency),
        rps: file_cfg.scan.rps.unwrap_or(args.rps),
        extra_headers,
        cookie: args.cookie.clone(),
        insecure_tls: args.insecure,
    };

    // Auto-include auth-compare when --compare-auth and not already listed
    let mut modules = modules;
    if args.compare_auth && !modules.iter().any(|m| m == "auth-compare") {
        modules.push("auth-compare".into());
    }

    let opts = ScanOptions {
        targets,
        profile,
        modules: modules.clone(),
        depth: file_cfg.scan.depth.unwrap_or(args.depth),
        max_urls: file_cfg.scan.max_urls.unwrap_or(args.max_urls),
        ignore_robots: file_cfg
            .scan
            .ignore_robots
            .unwrap_or(args.ignore_robots),
        wordlist: args.wordlist.clone(),
        probes,
        fail_on: Some(fail_on),
        templates_dir: args.templates_dir.clone(),
        compare_auth: args.compare_auth,
    };

    eprintln!(
        "weeping-angel: authorized scan of {} (profile={}, modules={})",
        args.targets.join(", "),
        profile.as_str(),
        modules.join(",")
    );
    info!("consent OK; starting scan");

    let report = run_scan(authz, client_cfg, opts).await?;

    let formats = Format::parse_list(&args.format);
    let formats = if formats.is_empty() {
        vec![Format::Terminal]
    } else {
        formats
    };

    write_reports(&report, &formats, args.output.as_deref())?;

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
