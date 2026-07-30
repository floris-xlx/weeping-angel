//! Clap / CLI surface parsing tests (no network).

use clap::Parser;
use weeping_angel::cli::{Cli, Commands};
use weeping_angel::parse::LogHttp;

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(
        std::iter::once("weeping-angel").chain(args.iter().copied()),
    )
    .unwrap_or_else(|e| panic!("parse failed for {args:?}: {e}"))
}

fn parse_err(args: &[&str]) -> String {
    Cli::try_parse_from(std::iter::once("weeping-angel").chain(args.iter().copied()))
        .unwrap_err()
        .to_string()
}

fn scan(args: &[&str]) -> weeping_angel::cli::ScanArgs {
    match parse(args).command {
        Commands::Scan(s) => s,
    }
}

#[test]
fn bare_i_own_this_sets_consent() {
    let s = scan(&["scan", "example.com", "--i-own-this", "--allow-host", "example.com"]);
    assert!(s.consent());
    assert_eq!(s.targets, vec!["example.com"]);
}

#[test]
fn i_own_this_equals_yes() {
    let s = scan(&["scan", "a.com", "--i-own-this=yes", "--allow-host", "a.com"]);
    assert!(s.consent());
}

#[test]
fn i_own_this_equals_true_one_on() {
    for v in ["true", "1", "on", "yes", "y", "TRUE", "Yes"] {
        let flag = format!("--i-own-this={v}");
        let s = scan(&["scan", "a.com", flag.as_str(), "--allow-host", "a.com"]);
        assert!(s.consent(), "expected consent for {v}");
    }
}

#[test]
fn i_own_this_equals_false_rejected() {
    let err = parse_err(&["scan", "a.com", "--i-own-this=false", "--allow-host", "a.com"]);
    assert!(
        err.contains("false") || err.contains("consent") || err.contains("i-own-this"),
        "err={err}"
    );
}

#[test]
fn i_own_this_space_value_not_accepted_as_flag_value() {
    // require_equals: `--i-own-this yes` treats `yes` as a target, not consent value
    let s = scan(&["scan", "--i-own-this", "yes", "--allow-host", "yes"]);
    assert!(s.consent());
    assert!(s.targets.iter().any(|t| t == "yes"));
}

#[test]
fn missing_consent_defaults_false() {
    let s = scan(&["scan", "example.com", "--allow-host", "example.com"]);
    assert!(!s.consent());
}

#[test]
fn enable_active_bare_and_equals() {
    let s = scan(&[
        "scan",
        "x.com",
        "--i-own-this",
        "--allow-host",
        "x.com",
        "--enable-active",
    ]);
    assert!(s.enable_active());
    let s = scan(&[
        "scan",
        "x.com",
        "--i-own-this",
        "--allow-host",
        "x.com",
        "--enable-active=true",
    ]);
    assert!(s.enable_active());
    let s = scan(&["scan", "x.com", "--i-own-this", "--allow-host", "x.com"]);
    assert!(!s.enable_active());
}

#[test]
fn allow_host_repeatable_and_csv_preserved_raw() {
    let s = scan(&[
        "scan",
        "a.com",
        "--i-own-this",
        "--allow-host",
        "a.com,b.com",
        "--allow-host",
        "c.com",
    ]);
    assert_eq!(s.allow_hosts.len(), 2);
    assert!(s.allow_hosts[0].contains("a.com"));
    assert_eq!(s.allow_hosts[1], "c.com");
}

#[test]
fn allow_host_from_target_flag() {
    let s = scan(&["scan", "z.com", "--i-own-this", "--allow-host-from-target"]);
    assert!(s.allow_host_from_target);
}

#[test]
fn prefer_http_and_fast_presets() {
    let s = scan(&[
        "scan",
        "z.com",
        "--i-own-this",
        "--allow-host",
        "z.com",
        "--prefer-http",
        "--fast",
    ]);
    assert!(s.prefer_http);
    assert!(s.fast);
    assert!(s.effective_rps() >= 40.0);
    assert!(s.effective_concurrency() >= 40);
    assert_eq!(s.log_http_mode(), LogHttp::Summary);
}

#[test]
fn log_http_modes() {
    for (mode, expected) in [
        ("full", LogHttp::Full),
        ("compact", LogHttp::Compact),
        ("summary", LogHttp::Summary),
        ("off", LogHttp::Off),
        ("quiet", LogHttp::Off),
    ] {
        let s = scan(&[
            "scan",
            "z.com",
            "--i-own-this",
            "--allow-host",
            "z.com",
            "--log-http",
            mode,
        ]);
        assert_eq!(s.log_http_mode(), expected, "mode={mode}");
    }
}

#[test]
fn multiple_cookies_merge() {
    let s = scan(&[
        "scan",
        "z.com",
        "--i-own-this",
        "--allow-host",
        "z.com",
        "--cookie",
        "a=1",
        "--cookie",
        "b=2",
    ]);
    assert_eq!(s.cookie_header().as_deref(), Some("a=1; b=2"));
}

#[test]
fn headers_repeatable() {
    let s = scan(&[
        "scan",
        "z.com",
        "--i-own-this",
        "--allow-host",
        "z.com",
        "--header",
        "X-A: 1",
        "--header",
        "X-B=2",
    ]);
    assert_eq!(s.headers.len(), 2);
    let parsed = weeping_angel::cli::parse_header_lines(&s.headers).unwrap();
    assert_eq!(parsed[0], ("X-A".into(), "1".into()));
    assert_eq!(parsed[1], ("X-B".into(), "2".into()));
}

#[test]
fn defaults_concurrency_and_rps_raised() {
    let s = scan(&["scan", "z.com", "--i-own-this", "--allow-host", "z.com"]);
    assert_eq!(s.concurrency, 20);
    assert!((s.rps - 15.0).abs() < f64::EPSILON);
}

#[test]
fn format_and_fail_on_and_profile() {
    let s = scan(&[
        "scan",
        "z.com",
        "--i-own-this",
        "--allow-host",
        "z.com",
        "--format",
        "terminal,json,html",
        "--fail-on",
        "high",
        "--profile",
        "deep",
        "--max-terminal-routes",
        "50",
        "--report-width",
        "120",
    ]);
    assert_eq!(s.format, "terminal,json,html");
    assert_eq!(s.fail_on, "high");
    assert_eq!(s.profile, "deep");
    assert_eq!(s.max_terminal_routes, 50);
    assert_eq!(s.report_width, 120);
}

#[test]
fn verbose_global_count() {
    let cli = parse(&["-vv", "scan", "z.com", "--i-own-this", "--allow-host", "z.com"]);
    assert_eq!(cli.verbose, 2);
}

#[test]
fn compare_auth_and_ignore_robots_optional_bools() {
    let s = scan(&[
        "scan",
        "z.com",
        "--i-own-this",
        "--allow-host",
        "z.com",
        "--compare-auth",
        "--ignore-robots=true",
        "--insecure=true",
    ]);
    assert!(s.compare_auth());
    assert!(s.ignore_robots());
    assert!(s.insecure());
}

#[test]
fn multi_target_positionals() {
    let s = scan(&[
        "scan",
        "a.com",
        "b.com/path",
        "--i-own-this",
        "--allow-host",
        "a.com",
        "--allow-host",
        "b.com",
    ]);
    assert_eq!(s.targets.len(), 2);
}
