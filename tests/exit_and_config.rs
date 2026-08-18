//! exit_code_for + profile/config helpers.

use chrono::Utc;
use weeping_angel::config::{FileConfig, Profile, merge_hosts};
use weeping_angel::exit_code_for;
use weeping_angel::finding::{Finding, ScanReport, ScanStats, Severity};

fn report_with(severities: &[Severity]) -> ScanReport {
    let findings: Vec<_> = severities
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Finding::builder("test", format!("id-{i}"))
                .title("t")
                .severity(*s)
                .url("https://x/")
                .description("d")
                .build()
        })
        .collect();
    ScanReport {
        tool: "weeping-angel".into(),
        version: "0.0.0".into(),
        target: "https://x/".into(),
        started_at: Utc::now(),
        finished_at: Utc::now(),
        profile: "recon".into(),
        modules: vec![],
        discovered_urls: vec![],
        routes: vec![],
        findings,
        stats: ScanStats::default(),
        image_harvest: None,
        phases: vec![],
        module_results: vec![],
        surface: Default::default(),
        tech_stack: vec![],
        timing: Default::default(),
    }
}

#[test]
fn exit_zero_when_below_threshold() {
    let r = report_with(&[Severity::Info, Severity::Low]);
    assert_eq!(exit_code_for(&r, Severity::Medium), 0);
    assert_eq!(exit_code_for(&r, Severity::High), 0);
}

#[test]
fn exit_one_when_at_or_above_threshold() {
    let r = report_with(&[Severity::Medium]);
    assert_eq!(exit_code_for(&r, Severity::Medium), 1);
    assert_eq!(exit_code_for(&r, Severity::Low), 1);

    let r = report_with(&[Severity::Critical]);
    assert_eq!(exit_code_for(&r, Severity::High), 1);
}

#[test]
fn exit_info_threshold_ignores_pure_info() {
    let r = report_with(&[Severity::Info]);
    // fail_on Info still only fails on Low+
    assert_eq!(exit_code_for(&r, Severity::Info), 0);
    let r = report_with(&[Severity::Info, Severity::Low]);
    assert_eq!(exit_code_for(&r, Severity::Info), 1);
}

#[test]
fn profile_aliases() {
    assert_eq!(Profile::parse("quick"), Some(Profile::Recon));
    assert_eq!(Profile::parse("light"), Some(Profile::Recon));
    assert_eq!(Profile::parse("default"), Some(Profile::Standard));
    assert_eq!(Profile::parse("normal"), Some(Profile::Standard));
    assert_eq!(Profile::parse("full"), Some(Profile::Deep));
    assert_eq!(Profile::parse("aggressive"), Some(Profile::Deep));
    assert_eq!(Profile::parse("nope"), None);
    assert_eq!(Profile::Deep.as_str(), "deep");
}

#[test]
fn profile_module_sets_nested() {
    let recon = Profile::Recon.default_modules();
    let standard = Profile::Standard.default_modules();
    let deep = Profile::Deep.default_modules();
    assert!(recon.contains(&"discovery"));
    assert!(recon.contains(&"firebase"));
    assert!(!recon.contains(&"wordlist"));
    assert!(standard.contains(&"wordlist"));
    assert!(standard.contains(&"cors"));
    assert!(deep.contains(&"openapi"));
    assert!(deep.contains(&"auth-compare"));
    assert!(deep.len() >= standard.len());
}

#[test]
fn merge_hosts_unions() {
    let h = merge_hosts(
        vec!["a.com".into(), "b.com".into()],
        vec!["b.com".into(), "c.com".into()],
    );
    assert_eq!(h.len(), 3);
    assert!(h.contains("a.com"));
    assert!(h.contains("c.com"));
}

#[test]
fn file_config_default_and_toml_roundtrip() {
    let d = FileConfig::default();
    assert!(!d.authorization.i_own_this);
    assert!(d.authorization.allow_hosts.is_empty());

    let raw = r#"
[authorization]
i_own_this = true
allow_hosts = ["127.0.0.1", "lab.test"]
enable_active = true

[scan]
profile = "deep"
fail_on = "high"
concurrency = 30
rps = 25.0
modules = ["discovery", "headers"]
"#;
    let cfg: FileConfig = toml::from_str(raw).unwrap();
    assert!(cfg.authorization.i_own_this);
    assert_eq!(cfg.authorization.allow_hosts.len(), 2);
    assert!(cfg.authorization.enable_active);
    assert_eq!(cfg.scan.profile.as_deref(), Some("deep"));
    assert_eq!(cfg.scan.concurrency, Some(30));
    assert_eq!(cfg.scan.modules, vec!["discovery", "headers"]);
}

#[test]
fn file_config_load_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wa.toml");
    std::fs::write(
        &path,
        r#"
[authorization]
i_own_this = true
allow_hosts = ["example.com"]
"#,
    )
    .unwrap();
    let cfg = FileConfig::load(&path).unwrap();
    assert!(cfg.authorization.i_own_this);
    assert_eq!(cfg.authorization.allow_hosts, vec!["example.com"]);
}
