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
        other => panic!("expected Scan command, got {other:?}"),
    }
}

#[test]
fn scan_diff_parses_base_head() {
    let cli = parse(&[
        "scan-diff",
        "--repo",
        ".",
        "-o",
        "out-diff",
        "--base",
        "main",
        "--head",
        "HEAD",
        "--fail-on",
        "high",
    ]);
    match cli.command {
        Commands::ScanDiff(d) => {
            assert_eq!(d.base.as_deref(), Some("main"));
            assert_eq!(d.head, "HEAD");
            assert!(!d.working_tree);
            assert_eq!(d.fail_on, "high");
        }
        other => panic!("expected ScanDiff, got {other:?}"),
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
fn cookie_key_value_space_and_equals() {
    let s = scan(&[
        "scan",
        "z.com",
        "--i-own-this",
        "--allow-host",
        "z.com",
        "--cookie",
        "session",
        "admin",
        "--cookie",
        "role=ops",
    ]);
    assert_eq!(s.cookie_header().as_deref(), Some("session=admin; role=ops"));
}

#[test]
fn header_key_equals_value_and_two_args() {
    let s = scan(&[
        "scan",
        "z.com",
        "--i-own-this",
        "--allow-host",
        "z.com",
        "--header",
        "X-A=1",
        "--header",
        "X-B",
        "two",
        "--header",
        "X-C: three",
    ]);
    let parsed = weeping_angel::cli::parse_header_lines(&s.headers).unwrap();
    assert_eq!(
        parsed,
        vec![
            ("X-A".into(), "1".into()),
            ("X-B".into(), "two".into()),
            ("X-C".into(), "three".into()),
        ]
    );
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
    let cli = parse(&[
        "--verbose",
        "--verbose",
        "scan",
        "z.com",
        "--i-own-this",
        "--allow-host",
        "z.com",
    ]);
    assert_eq!(cli.verbose, 2);
}

#[test]
fn version_flags_display_package_version() {
    use clap::error::ErrorKind;
    for args in [vec!["-v"], vec!["-V"], vec!["--version"]] {
        let err = Cli::try_parse_from(std::iter::once("weeping-angel").chain(args.iter().copied()))
            .expect_err("version flags should print version");
        assert_eq!(err.kind(), ErrorKind::DisplayVersion, "args={args:?}");
        let rendered = err.to_string();
        assert!(
            rendered.contains(env!("CARGO_PKG_VERSION")),
            "args={args:?} rendered={rendered}"
        );
        assert!(
            rendered.contains("weeping-angel"),
            "args={args:?} rendered={rendered}"
        );
    }
}

#[test]
fn version_subcommand_parses() {
    let cli = parse(&["version"]);
    match cli.command {
        Commands::Version => {}
        other => panic!("expected Version, got {other:?}"),
    }
}

#[test]
fn argv_is_version_only_accepts_common_flags() {
    assert!(Cli::argv_is_version_only(["-v"]));
    assert!(Cli::argv_is_version_only(["-V"]));
    assert!(Cli::argv_is_version_only(["--version"]));
    assert!(Cli::argv_is_version_only(["version"]));
    assert!(Cli::argv_is_version_only(["--verbose", "-v"]));
    assert!(!Cli::argv_is_version_only(["scan", "-v"]));
    assert!(!Cli::argv_is_version_only(std::iter::empty::<&str>()));
}

#[test]
fn completions_subcommand_parses_powershell() {
    let cli = parse(&["completions", "powershell"]);
    match cli.command {
        Commands::Completions { shell } => {
            assert_eq!(shell.to_string(), "powershell");
        }
        other => panic!("expected Completions, got {other:?}"),
    }
}

#[test]
fn scan_alias_s_parses() {
    let s = scan(&["s", "example.com", "--i-own-this", "--allow-host", "example.com"]);
    assert_eq!(s.targets, vec!["example.com"]);
}

#[test]
fn no_args_requests_help() {
    use clap::error::ErrorKind;
    let err = Cli::try_parse_from(["weeping-angel"]).expect_err("empty argv should show help");
    assert_eq!(err.kind(), ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand);
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

#[test]
fn workbench_compare_parses() {
    let cli = parse(&[
        "workbench",
        "compare",
        "--before",
        "scan-a",
        "--after",
        "scan-b",
        "--out",
        "delta.json",
    ]);
    match cli.command {
        Commands::Workbench(w) => match w.command {
            weeping_angel::cli::WorkbenchCommand::Compare {
                before,
                after,
                out,
            } => {
                assert_eq!(before, std::path::PathBuf::from("scan-a"));
                assert_eq!(after, std::path::PathBuf::from("scan-b"));
                assert_eq!(out.unwrap(), std::path::PathBuf::from("delta.json"));
            }
            other => panic!("expected Compare, got {other:?}"),
        },
        other => panic!("expected Workbench, got {other:?}"),
    }
}

#[test]
fn workbench_generate_patches_parses() {
    let cli = parse(&[
        "workbench",
        "generate-patches",
        "--scan-dir",
        "out",
        "--source-root",
        ".",
    ]);
    match cli.command {
        Commands::Workbench(w) => match w.command {
            weeping_angel::cli::WorkbenchCommand::GeneratePatches {
                scan_dir,
                source_root,
            } => {
                assert_eq!(scan_dir, std::path::PathBuf::from("out"));
                assert_eq!(source_root, std::path::PathBuf::from("."));
            }
            other => panic!("expected GeneratePatches, got {other:?}"),
        },
        other => panic!("expected Workbench, got {other:?}"),
    }
}

#[test]
fn workbench_apply_and_verify_parse() {
    let cli = parse(&[
        "workbench",
        "apply-patch",
        "--source-root",
        "src",
        "--patch",
        "fix.patch",
    ]);
    match cli.command {
        Commands::Workbench(w) => match w.command {
            weeping_angel::cli::WorkbenchCommand::ApplyPatch {
                source_root,
                patch,
            } => {
                assert_eq!(source_root, std::path::PathBuf::from("src"));
                assert_eq!(patch, std::path::PathBuf::from("fix.patch"));
            }
            other => panic!("expected ApplyPatch, got {other:?}"),
        },
        other => panic!("expected Workbench, got {other:?}"),
    }

    let cli = parse(&[
        "workbench",
        "verify",
        "--source-root",
        "src",
        "--path",
        "a.py",
        "--rule-id",
        "command-injection.shell-true",
    ]);
    match cli.command {
        Commands::Workbench(w) => match w.command {
            weeping_angel::cli::WorkbenchCommand::Verify {
                source_root,
                path,
                rule_id,
            } => {
                assert_eq!(source_root, std::path::PathBuf::from("src"));
                assert_eq!(path, "a.py");
                assert_eq!(rule_id, "command-injection.shell-true");
            }
            other => panic!("expected Verify, got {other:?}"),
        },
        other => panic!("expected Workbench, got {other:?}"),
    }
}
