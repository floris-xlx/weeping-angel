//! weeping-angel — authorized dual-domain security toolchain (web + code).

pub mod assurance_catalog;
pub mod assurance_explain;
pub mod assurance_soa;
pub mod authz;
pub mod checks;
pub mod cli;
pub mod config;
pub mod contract;
pub mod depcheck;
pub mod discovery;
pub mod docs_export;
pub mod engine;
pub mod engines;
pub mod finding;
pub mod http;
pub mod parse;
pub mod report;
pub mod style;
pub mod target;
pub mod templates;
pub mod workbench;

/// Local vulnerable lab (axum). Enable with `--features demo`.
#[cfg(feature = "demo")]
pub mod lab;

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

pub fn run_workbench_command(args: crate::cli::WorkbenchArgs) -> Result<i32> {
    use crate::cli::WorkbenchCommand;
    use crate::workbench::{get_scan, list_scans, open_db, register_scan};

    crate::style::init();
    match args.command {
        WorkbenchCommand::Register { scan_dir } => {
            let conn = open_db(args.db.as_deref())?;
            let row = register_scan(&conn, &scan_dir)?;
            crate::style::eprint_line(&format!(
                "{} registered {} findings={} max={} dir={}",
                crate::style::brand("weeping-angel"),
                crate::style::ok(&row.scan_id),
                row.finding_count,
                row.max_severity,
                row.scan_dir,
            ));
            println!("{}", serde_json::to_string_pretty(&row)?);
            Ok(0)
        }
        WorkbenchCommand::List { limit } => {
            let conn = open_db(args.db.as_deref())?;
            let rows = list_scans(&conn, limit)?;
            println!("{}", serde_json::to_string_pretty(&rows)?);
            Ok(0)
        }
        WorkbenchCommand::Show { scan_id } => {
            let conn = open_db(args.db.as_deref())?;
            match get_scan(&conn, &scan_id)? {
                Some(row) => {
                    println!("{}", serde_json::to_string_pretty(&row)?);
                    Ok(0)
                }
                None => {
                    crate::style::eprint_line(&format!(
                        "{} scan not found: {scan_id}",
                        crate::style::err("error:")
                    ));
                    Ok(2)
                }
            }
        }
        WorkbenchCommand::Compare { before, after, out } => {
            let cmp = crate::workbench::compare::compare_scan_dirs(&before, &after)?;
            let out_path = out.unwrap_or_else(|| after.join("compare.json"));
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out_path, serde_json::to_string_pretty(&cmp)?)?;
            crate::style::eprint_line(&format!(
                "{} compare {} → {} introduced={} resolved={} persistent={} → {}",
                crate::style::brand("weeping-angel"),
                cmp.before_scan_id,
                cmp.after_scan_id,
                cmp.introduced.len(),
                cmp.resolved.len(),
                cmp.persistent.len(),
                out_path.display(),
            ));
            println!("{}", serde_json::to_string_pretty(&cmp)?);
            Ok(0)
        }
        WorkbenchCommand::GeneratePatches {
            scan_dir,
            source_root,
        } => {
            let results = crate::workbench::remediation::generate_all(&source_root, &scan_dir)?;
            let generated = results.iter().filter(|r| r.state == "generated").count();
            let failed = results.iter().filter(|r| r.state == "failed").count();
            crate::style::eprint_line(&format!(
                "{} generate-patches total={} generated={} failed={} index={}",
                crate::style::brand("weeping-angel"),
                results.len(),
                generated,
                failed,
                scan_dir.join("remediation").join("index.json").display(),
            ));
            println!("{}", serde_json::to_string_pretty(&results)?);
            Ok(if failed > 0 && generated == 0 { 1 } else { 0 })
        }
        WorkbenchCommand::ApplyPatch { source_root, patch } => {
            let r = crate::workbench::remediation::apply_patch(&source_root, &patch)?;
            crate::style::eprint_line(&format!(
                "{} apply-patch {} — {}",
                crate::style::brand("weeping-angel"),
                crate::style::ok(&r.state),
                r.summary,
            ));
            println!("{}", serde_json::to_string_pretty(&r)?);
            Ok(0)
        }
        WorkbenchCommand::Verify {
            source_root,
            path,
            rule_id,
        } => {
            let r =
                crate::workbench::remediation::verify_file_clean(&source_root, &path, &rule_id)?;
            crate::style::eprint_line(&format!(
                "{} verify {} — {}",
                crate::style::brand("weeping-angel"),
                if r.state == "verified" {
                    crate::style::ok(&r.state)
                } else {
                    crate::style::err(&r.state)
                },
                r.summary,
            ));
            println!("{}", serde_json::to_string_pretty(&r)?);
            Ok(if r.state == "verified" { 0 } else { 1 })
        }
    }
}

pub fn run_finalize_command(args: crate::cli::FinalizeArgs) -> Result<i32> {
    crate::style::init();
    let report = crate::contract::finalize_scan(&args.scan_dir, env!("CARGO_PKG_VERSION"))?;
    crate::style::eprint_line(&format!(
        "{} sealed bundle → {}",
        crate::style::brand("weeping-angel"),
        crate::style::ok(&report.display().to_string()),
    ));
    Ok(0)
}

fn resolve_depcheck_kind(
    args: &crate::cli::DepcheckArgs,
) -> Result<Option<crate::depcheck::types::FileKind>> {
    use crate::depcheck::types::{Ecosystem, FileKind};

    if let Some(s) = args.file_type.as_deref() {
        return Ok(Some(
            FileKind::from_str_loose(s).with_context(|| format!("unknown --type {s}"))?,
        ));
    }
    if let Some(lang) = args.language.as_deref() {
        if lang.eq_ignore_ascii_case("all") {
            return Ok(None);
        }
        let eco = Ecosystem::from_str_loose(lang).with_context(|| {
            format!(
                "unknown -l/--language/--provider {lang} (expected npm|pip|pypi|composer|mvn|maven|gradle|rubygems|cargo|go|nuget|all)"
            )
        })?;
        let path = args.path.as_ref().or(args.target.as_ref());
        // Only force a default kind for a *single unknown file*. Directories must auto-detect
        // per file, then optionally filter by provider ecosystem.
        if let Some(target) = path {
            if target.is_file() {
                let detected = crate::depcheck::detect::detect_file_type(target, None);
                if detected == FileKind::Unknown {
                    return Ok(Some(eco.default_file_kind()));
                }
            }
        }
        return Ok(None);
    }
    Ok(None)
}

fn parse_dependency_arg(raw: &str) -> (String, String) {
    if let Some((name, ver)) = raw.split_once(':') {
        (name.trim().to_string(), ver.trim().to_string())
    } else {
        (raw.trim().to_string(), String::new())
    }
}

/// Dependency confusion scanner (detection only — no auto-exploit).
pub async fn run_depcheck_command(args: crate::cli::DepcheckArgs) -> Result<i32> {
    use std::collections::{HashSet, VecDeque};
    use std::sync::Arc;

    use crate::depcheck::depsdev::{self, is_nuget_reserved};
    use crate::depcheck::email_check::{self, EmailFindingKind};
    use crate::depcheck::registry::{HttpRegistry, RegistryClient};
    use crate::depcheck::types::{CheckStatus, Ecosystem, PackageRef, ScanOptions, ScanSummary};
    use crate::depcheck::{
        convert, exit_code, list_only, load_path, load_url_body, report, resolve_targets,
        scan_manifest,
    };

    crate::style::init();

    if args.web {
        crate::depcheck::web::start_web(&args.bind, args.port).await?;
        return Ok(0);
    }

    let provider_all = args
        .language
        .as_deref()
        .is_some_and(|s| s.eq_ignore_ascii_case("all"));

    let kind_override = resolve_depcheck_kind(&args)?;
    let opts = ScanOptions {
        threads: args.threads,
        timeout_secs: args.timeout,
        quiet: args.quiet || args.silent,
        kind_override,
        secure_namespaces: crate::depcheck::filter::parse_secure_namespace_list(
            &args.secure_namespaces,
        ),
        verbose: args.scan_verbose,
    };

    let registry: Arc<dyn RegistryClient> = Arc::new(HttpRegistry::new(opts.timeout_secs)?);
    let mut worst_exit = 0i32;
    let mut takeover_lines: Vec<String> = Vec::new();

    // ── DepenFusion-style remote host hunt (stdin / hosts-file) ───────
    if args.stdin || args.hosts_file.is_some() {
        if !args.consent() {
            bail!("remote host hunting requires --i-own-this (authorized testing only)");
        }
        let lines = if let Some(path) = &args.hosts_file {
            crate::depcheck::remote_hunt::read_hosts_file(path)?
        } else {
            crate::depcheck::remote_hunt::read_stdin_lines()?
        };
        let hunt = crate::depcheck::remote_hunt::HuntOptions {
            threads: args.threads.max(1),
            timeout_secs: args.timeout,
            append: args.append.clone(),
            strip_path: args.strip_path,
            verbose: if args.scan_verbose { 2 } else { 0 },
            silent: args.silent || args.quiet,
            show_link: args.link,
            extra_paths: Vec::new(),
        };
        let report =
            crate::depcheck::remote_hunt::hunt_remote(&lines, &hunt, &opts, Arc::clone(&registry))
                .await?;
        crate::depcheck::remote_hunt::print_hunt_report(&report, &hunt);
        if let Some(path) = &args.output_file {
            let body = report
                .vulnerable
                .iter()
                .map(|h| format!("{}:{}", h.package, h.version))
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(path, body + "\n")?;
            if !hunt.silent {
                crate::style::eprint_line(&format!(
                    "{} results saved to {}",
                    crate::style::brand("weeping-angel"),
                    path.display()
                ));
            }
        }
        if let Some(export) = &args.export {
            let json = serde_json::to_string_pretty(&report)?;
            std::fs::write(export, json)?;
        }
        return Ok(if report.vulnerable.is_empty() { 0 } else { 1 });
    }

    // ── Single dependency mode (DepFuzzer --dependency) ──────────────
    if let Some(dep) = &args.dependency {
        let lang = args.language.as_deref().unwrap_or("npm");
        if lang.eq_ignore_ascii_case("all") {
            bail!("--dependency requires a concrete --provider (not all)");
        }
        let eco = Ecosystem::from_str_loose(lang)
            .with_context(|| format!("unknown --provider {lang}"))?;
        let (root_name, root_ver) = parse_dependency_arg(dep);
        if root_name.is_empty() {
            bail!("empty --dependency");
        }

        let mut queue: VecDeque<PackageRef> = VecDeque::new();
        queue.push_back(PackageRef::new(
            &root_name,
            if root_ver.is_empty() { "*" } else { &root_ver },
        ));
        let mut seen: HashSet<String> = HashSet::new();
        let mut vulnerable = Vec::new();
        let mut safe = Vec::new();
        let mut errors = Vec::new();
        let email_client = if args.check_email {
            Some(email_check::http_client(opts.timeout_secs)?)
        } else {
            None
        };
        let deps_client = if args.transitive {
            Some(depsdev::client(opts.timeout_secs)?)
        } else {
            None
        };

        while let Some(pkg) = queue.pop_front() {
            if !seen.insert(pkg.name.clone()) {
                continue;
            }
            if eco == Ecosystem::Nuget && is_nuget_reserved(&pkg.name) {
                continue;
            }

            let mut result = registry.check(eco, &pkg.name).await;
            result.version = pkg.version.clone();
            match result.status {
                CheckStatus::Vulnerable => {
                    if args.print_takeover && !args.quiet {
                        if pkg.name.contains('@') {
                            eprintln!(
                                "[DEBUG] {} is missing on the public registry but is scoped — verify the org/scope is claimed.",
                                pkg.name
                            );
                        } else {
                            eprintln!("[DEBUG] {}:{} might be taken over !", pkg.name, pkg.version);
                        }
                    }
                    takeover_lines.push(format!("{}:{}", pkg.name, pkg.version));
                    vulnerable.push(result);
                }
                CheckStatus::Safe => {
                    if let Some(http) = &email_client {
                        match email_check::check_package_emails(http, eco, &pkg.name).await {
                            Ok(findings) => {
                                for f in findings {
                                    match f.kind {
                                        EmailFindingKind::DomainPossiblyPurchasable => {
                                            eprintln!("[+] {}", f.detail);
                                            worst_exit = worst_exit.max(1);
                                        }
                                        EmailFindingKind::DisposableEmail => {
                                            eprintln!(
                                                "[+] Dependency {} uses a disposable email provider: {}",
                                                f.package, f.email
                                            );
                                            worst_exit = worst_exit.max(1);
                                        }
                                    }
                                }
                            }
                            Err(e) if opts.verbose => {
                                eprintln!("  [?] email check {}: {e}", pkg.name);
                            }
                            Err(_) => {}
                        }
                    }
                    if let Some(http) = &deps_client {
                        if let Ok(subs) =
                            depsdev::fetch_transitive(http, eco, &pkg.name, &pkg.version).await
                        {
                            for sub in subs {
                                if !seen.contains(&sub.name) {
                                    queue.push_back(sub);
                                }
                            }
                        }
                    }
                    safe.push(result);
                }
                CheckStatus::Error => errors.push(result),
            }
        }

        let summary = ScanSummary {
            file: format!("dependency:{root_name}"),
            file_kind: eco.default_file_kind(),
            ecosystem: eco,
            packages: [(root_name.clone(), root_ver.clone())]
                .into_iter()
                .collect(),
            vulnerable,
            safe,
            errors,
            suppressed: Vec::new(),
            introductions: Vec::new(),
            hardening: None,
            duration_secs: 0.0,
            tool_version: env!("CARGO_PKG_VERSION").into(),
        };
        if !args.print_takeover || args.quiet {
            report::print_summary(&summary, args.quiet);
        } else if summary.vulnerable.is_empty() && !args.quiet {
            eprintln!("[+] No package can be taken over !");
        }
        if let Some(path) = &args.output_file {
            std::fs::write(path, takeover_lines.join("\n") + "\n")?;
            crate::style::eprint_line(&format!(
                "{} results saved to {}",
                crate::style::brand("weeping-angel"),
                path.display()
            ));
        }
        if let Some(export) = &args.export {
            report::export_json(export, &summary)?;
        }
        return Ok(worst_exit.max(exit_code(&summary)));
    }

    // ── Path / URL / target modes ────────────────────────────────────
    let root = args.path.clone().or_else(|| args.target.clone());
    let mut inputs = Vec::new();
    if let Some(url) = &args.url {
        if !args.consent() {
            bail!("--url requires --i-own-this (authorized remote fetch)");
        }
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(opts.timeout_secs.max(5)))
            .user_agent(concat!(
                "weeping-angel-depcheck/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;
        let body = client
            .get(url)
            .send()
            .await
            .with_context(|| format!("fetch {url}"))?
            .error_for_status()
            .with_context(|| format!("HTTP error for {url}"))?
            .text()
            .await?;
        inputs.push(load_url_body(url, body, opts.kind_override)?);
    } else if let Some(target) = &root {
        for path in resolve_targets(target)? {
            let input = load_path(&path, opts.kind_override)?;
            if provider_all {
                inputs.push(input);
            } else if let Some(lang) = args.language.as_deref() {
                if let Some(eco) = Ecosystem::from_str_loose(lang) {
                    if input.kind.ecosystem() == Some(eco) {
                        inputs.push(input);
                    }
                } else {
                    inputs.push(input);
                }
            } else {
                inputs.push(input);
            }
        }
        if inputs.is_empty() {
            bail!(
                "no dependency files matched provider {:?} under {}",
                args.language,
                target.display()
            );
        }
    } else {
        bail!("provide a target/--path, --dependency, --url, or --web");
    }

    let email_client = if args.check_email {
        Some(email_check::http_client(opts.timeout_secs)?)
    } else {
        None
    };

    for input in &inputs {
        if args.list {
            let (pkgs, eco, kind) = list_only(input)?;
            if !args.quiet {
                crate::style::eprint_line(&format!(
                    "{} list {} ecosystem={} kind={} packages={}",
                    crate::style::brand("weeping-angel"),
                    input.display,
                    eco,
                    kind,
                    pkgs.len()
                ));
            }
            for (name, version) in &pkgs {
                println!("{name} @ {version}");
            }
            continue;
        }

        if args.convert {
            let (packages, _, _) = list_only(input)?;
            let pkgs: Vec<_> = packages
                .into_iter()
                .map(|(n, v)| PackageRef::new(n, v))
                .collect();
            let out = if let Some(path) = &input.path {
                convert::write_converted(path, &pkgs)?
            } else {
                let out = std::path::PathBuf::from("depcheck-converted.json");
                convert::write_converted_to(&out, &pkgs)?;
                out
            };
            crate::style::eprint_line(&format!(
                "{} converted {} packages → {}",
                crate::style::brand("weeping-angel"),
                pkgs.len(),
                crate::style::ok(&out.display().to_string())
            ));
            continue;
        }

        if !args.quiet {
            crate::style::eprint_line(&format!(
                "{} starting analysis for {} ({})…",
                crate::style::brand("weeping-angel"),
                input.display,
                input.kind
            ));
        }

        let mut summary = scan_manifest(input, &opts, Arc::clone(&registry)).await?;

        // NuGet reserved-prefix false positives
        if summary.ecosystem == Ecosystem::Nuget {
            summary.vulnerable.retain(|p| !is_nuget_reserved(&p.name));
        }

        for v in &summary.vulnerable {
            if args.print_takeover && !args.quiet {
                if v.name.contains('@') {
                    eprintln!(
                        "[DEBUG] {} is missing but scoped — verify org/scope ownership.",
                        v.name
                    );
                } else {
                    eprintln!("[DEBUG] {}:{} might be taken over !", v.name, v.version);
                }
            }
            takeover_lines.push(format!("{}:{}", v.name, v.version));
        }

        // Loki inspector: git introduction commits for free-namespace packages
        if args.inspect && !summary.vulnerable.is_empty() {
            let start = input
                .path
                .as_deref()
                .or(root.as_deref())
                .unwrap_or_else(|| std::path::Path::new("."));
            let names: Vec<String> = summary.vulnerable.iter().map(|v| v.name.clone()).collect();
            match crate::depcheck::inspect::inspect_introductions(
                start,
                &names,
                input.path.as_deref(),
            ) {
                Ok(intros) => summary.introductions = intros,
                Err(e) if !args.quiet => {
                    crate::style::eprint_line(&format!(
                        "{} inspector skipped: {e:#}",
                        crate::style::warn("[i]")
                    ));
                }
                Err(_) => {}
            }
        }

        // Loki hardening recon (npm / Node projects)
        if !args.no_hardening && summary.ecosystem == Ecosystem::Npm {
            let project_root = input
                .path
                .as_ref()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .or_else(|| root.clone())
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            summary.hardening = Some(crate::depcheck::hardening::analyze_npm_project(
                &project_root,
                input.path.as_deref(),
                &summary.vulnerable,
                args.entrypoint.as_deref(),
            ));
        }

        // Email checks on packages that exist
        if let Some(http) = &email_client {
            for s in &summary.safe {
                match email_check::check_package_emails(http, summary.ecosystem, &s.name).await {
                    Ok(findings) => {
                        for f in findings {
                            eprintln!("[+] {}", f.detail);
                            worst_exit = worst_exit.max(1);
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        if !args.print_takeover || args.quiet {
            report::print_summary(&summary, args.quiet);
        }

        if let Some(export) = &args.export {
            let path = if inputs.len() > 1 {
                let stem = std::path::Path::new(&summary.file)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("scan");
                export.with_file_name(format!(
                    "{}-{}",
                    export
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("results"),
                    stem
                ))
            } else {
                export.clone()
            };
            report::export_json(&path, &summary)?;
            if !args.quiet {
                crate::style::eprint_line(&format!(
                    "{} exported {}",
                    crate::style::brand("weeping-angel"),
                    path.display()
                ));
            }
        }
        worst_exit = worst_exit.max(exit_code(&summary));
    }

    if let Some(path) = &args.output_file {
        std::fs::write(path, takeover_lines.join("\n") + "\n")?;
        crate::style::eprint_line(&format!(
            "{} results saved to {}",
            crate::style::brand("weeping-angel"),
            path.display()
        ));
    }

    Ok(worst_exit)
}

pub fn run_scan_code_command(args: crate::cli::ScanCodeArgs) -> Result<i32> {
    use std::fs;

    crate::style::init();
    let root = args
        .path
        .canonicalize()
        .with_context(|| format!("resolve {}", args.path.display()))?;
    fs::create_dir_all(&args.scan_dir)?;

    let scope_prefix = args
        .scope
        .as_deref()
        .map(|s| s.trim_start_matches("./").replace('\\', "/"))
        .filter(|s| !s.is_empty());

    let result = crate::engines::code_scan::run_code_scan(
        &root,
        &args.scan_dir,
        scope_prefix.as_deref(),
        env!("CARGO_PKG_VERSION"),
    )?;

    crate::style::eprint_line(&format!(
        "{} code scan mode={} files={} hits={} findings={} max={} fail_on={} scan_id={}",
        crate::style::brand("weeping-angel"),
        result.mode,
        result.files_scanned,
        result.hit_count,
        result.finding_count,
        result.max_severity,
        args.fail_on,
        result.scan_id,
    ));
    crate::style::eprint_line(&format!(
        "{} sealed → {}",
        crate::style::brand("weeping-angel"),
        crate::style::ok(&result.report_path.display().to_string()),
    ));

    if crate::engines::findings_meet_fail_on(&result.max_severity, &args.fail_on) {
        Ok(1)
    } else {
        Ok(0)
    }
}

pub fn run_scan_diff_command(args: crate::cli::ScanDiffArgs) -> Result<i32> {
    use std::fs;

    use crate::engines::code_scan::{CodeScanOpts, run_code_scan_with_opts};
    use crate::engines::git_diff::{DiffTarget, find_git_root, inventory_diff};

    crate::style::init();
    let start = args
        .repo
        .canonicalize()
        .with_context(|| format!("resolve {}", args.repo.display()))?;
    let root = find_git_root(&start).unwrap_or(start);
    fs::create_dir_all(&args.scan_dir)?;

    let target = if args.working_tree || args.base.is_none() {
        DiffTarget::WorkingTree {
            base: args.base.clone(),
        }
    } else {
        let base = args
            .base
            .clone()
            .ok_or_else(|| anyhow::anyhow!("scan-diff --base is required unless --working-tree"))?;
        DiffTarget::Revisions {
            base,
            head: args.head.clone(),
        }
    };

    let inv = inventory_diff(&root, &target)?;
    if inv.files.is_empty() {
        crate::style::eprint_line(&format!(
            "{} diff scan: no changed source-like files",
            crate::style::brand("weeping-angel"),
        ));
    }

    let opts = CodeScanOpts {
        scope_prefix: None,
        file_list: Some(inv.files),
        mode: if args.working_tree || args.base.is_none() {
            "working_tree".into()
        } else {
            "branch_diff".into()
        },
        inventory_strategy: "diff".into(),
        target_kind: "git_diff".into(),
        base_revision: inv.base_revision,
        head_revision: inv.head_revision,
        summary_prefix: Some(format!(
            "Algorithmic diff scan ({})",
            inv.content_digest_hint
        )),
    };

    let result = run_code_scan_with_opts(&root, &args.scan_dir, opts, env!("CARGO_PKG_VERSION"))?;

    crate::style::eprint_line(&format!(
        "{} diff scan mode={} files={} hits={} findings={} max={} fail_on={} scan_id={}",
        crate::style::brand("weeping-angel"),
        result.mode,
        result.files_scanned,
        result.hit_count,
        result.finding_count,
        result.max_severity,
        args.fail_on,
        result.scan_id,
    ));
    crate::style::eprint_line(&format!(
        "{} sealed → {}",
        crate::style::brand("weeping-angel"),
        crate::style::ok(&result.report_path.display().to_string()),
    ));

    if crate::engines::findings_meet_fail_on(&result.max_severity, &args.fail_on) {
        Ok(1)
    } else {
        Ok(0)
    }
}

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
        bail!("provide at least one target (example.com, //host/path, http://…, or https://…)");
    }

    let norm_opts = NormalizeOptions {
        prefer_http: args.prefer_http,
    };
    let normalized: Vec<String> =
        normalize_targets(&args.targets, norm_opts).map_err(|e| anyhow::anyhow!("{e}"))?;

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
    let profile: Profile = Profile::parse(&profile_name).with_context(|| {
        format!("unknown profile: {profile_name} (recon|standard|deep|quick|full)")
    })?;

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

    let probes: Vec<String> = args.probe.as_deref().map(split_list).unwrap_or_default();

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
        if args.fast { " · fast preset" } else { "" },
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

    // When -o is set, also emit a Codex-compatible sealed scan bundle beside reports.
    if let Some(out) = args.output.as_ref() {
        if let Err(e) = write_web_sealed_bundle(&report, out, env!("CARGO_PKG_VERSION")) {
            crate::style::eprint_line(&format!(
                "{} sealed contract: {e:#}",
                crate::style::err("warning:")
            ));
        }
    }

    Ok(exit_code_for(&report, fail_on))
}

/// Project web DAST findings into sealed codex-security bundle under `<output>.codex-scan/`.
fn write_web_sealed_bundle(
    report: &finding::ScanReport,
    output: &std::path::Path,
    producer_version: &str,
) -> Result<()> {
    use crate::contract::{
        CoverageDocument, CoverageSurface, FindingsDocument, ManifestDocument, Producer, ScanBody,
        ScanScope, ScanTarget, finalize_scan, target_id_from_display, write_scan_bundle,
    };
    use crate::engines::web_adapt::web_finding_to_semantic;

    let scan_dir = {
        let mut p = output.to_path_buf();
        // if output is a prefix like "report-lab", use report-lab.codex-scan
        let name = p
            .file_name()
            .map(|s| format!("{}.codex-scan", s.to_string_lossy()))
            .unwrap_or_else(|| "scan.codex-scan".into());
        if let Some(parent) = p.parent() {
            p = parent.join(name);
        } else {
            p = std::path::PathBuf::from(name);
        }
        p
    };
    std::fs::create_dir_all(&scan_dir)?;

    let scan_id = format!(
        "wa_web_{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    );
    let display = if report.target.is_empty() {
        "web-target".into()
    } else {
        report.target.clone()
    };
    let findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| !finding::is_inventory_finding(f))
        .map(web_finding_to_semantic)
        .collect();

    let mut surfaces = vec![CoverageSurface {
        id: "surface_web_recon".into(),
        label: "Web recon / DAST modules".into(),
        disposition: if findings.is_empty() {
            "no_issue_found".into()
        } else {
            "reported".into()
        },
        receipt_refs: vec![],
        risk_area: Some("web".into()),
        notes: None,
    }];
    for m in &report.module_results {
        surfaces.push(CoverageSurface {
            id: format!("surface_web_{}", m.id),
            label: format!("Module {}", m.id),
            disposition: if m.findings > 0 {
                "reported".into()
            } else {
                "no_issue_found".into()
            },
            receipt_refs: vec![],
            risk_area: Some(m.id.clone()),
            notes: None,
        });
    }

    let digest_hex = crate::contract::fingerprint::sha256_text(&report.target);
    let manifest = ManifestDocument {
        document_type: "codex-security.scan-manifest".into(),
        schema_version: "1.0".into(),
        scan: ScanBody {
            id: scan_id.clone(),
            producer: Producer {
                name: "weeping-angel".into(),
                version: producer_version.into(),
            },
            status: "completed".into(),
            started_at: String::new(),
            completed_at: String::new(),
            sealed_at: String::new(),
            target: ScanTarget {
                kind: "directory_snapshot".into(),
                target_id: target_id_from_display(&display),
                display_name: display,
                remote: Some(report.target.clone()),
                revision: None,
                base_revision: None,
                head_revision: None,
                snapshot_digest: Some(format!("codex-security-snapshot/v1:sha256:{digest_hex}")),
            },
            scope: ScanScope {
                include_paths: vec!["/".into()],
                exclude_paths: vec![],
                summary: Some(format!(
                    "Live authorized web scan of {} (profile {}).",
                    report.target, report.profile
                )),
                artifacts_reviewed: None,
                runtime_status: Some("live-http".into()),
                validation_mode: Some("http-probe".into()),
                context: None,
                limitations: Some(vec![
                    "Findings produced by live HTTP recon/DAST modules.".into(),
                ]),
            },
            threat_model: None,
            hardening: None,
            coverage_ref: "coverage.json".into(),
            findings_ref: "findings.json".into(),
            artifacts: vec![],
        },
    };

    let findings_doc = FindingsDocument {
        document_type: "codex-security.findings".into(),
        schema_version: "1.0".into(),
        scan_id: scan_id.clone(),
        findings,
    };
    let coverage = CoverageDocument {
        document_type: "codex-security.coverage".into(),
        schema_version: "1.0".into(),
        scan_id,
        mode: "repository".into(),
        completeness: "complete".into(),
        inventory_strategy: "custom".into(),
        include_paths: vec!["/".into()],
        exclude_paths: vec![],
        surfaces,
        explicit_exclusions: vec![],
        deferred: vec![],
        open_questions: vec![],
    };

    write_scan_bundle(&scan_dir, &manifest, &findings_doc, &coverage)?;
    let report_md = finalize_scan(&scan_dir, producer_version)?;
    crate::style::eprint_line(&format!(
        "{} web sealed contract → {}",
        crate::style::brand("weeping-angel"),
        crate::style::ok(&report_md.display().to_string()),
    ));
    Ok(())
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
