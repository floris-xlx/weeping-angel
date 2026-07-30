//! Authorization / scope unit coverage beyond authz_cli.

use weeping_angel::authz::{Authorization, AuthzError};
use weeping_angel::parse::normalize_allow_hosts;
use weeping_angel::target::{NormalizeOptions, normalize_targets};
use url::Url;

#[test]
fn accepts_multiple_targets_same_allowlist() {
    let authz = Authorization::new(
        true,
        ["example.com".into(), "lab.test".into()],
        false,
        false,
    );
    let urls = authz
        .validate_targets(&[
            "https://example.com/a".into(),
            "https://lab.test/".into(),
        ])
        .unwrap();
    assert_eq!(urls.len(), 2);
}

#[test]
fn rejects_mixed_out_of_scope() {
    let authz = Authorization::new(true, ["example.com".into()], false, false);
    let err = authz
        .validate_targets(&[
            "https://example.com/".into(),
            "https://evil.com/".into(),
        ])
        .unwrap_err();
    assert!(matches!(err, AuthzError::HostNotAllowed { .. }));
}

#[test]
fn wildcard_suffix_and_dot_prefix() {
    let authz = Authorization::new(true, ["*.example.com".into()], false, false);
    assert!(authz.url_in_scope(&Url::parse("https://a.example.com").unwrap()));
    assert!(authz.url_in_scope(&Url::parse("https://example.com").unwrap()));
    assert!(!authz.url_in_scope(&Url::parse("https://example.org").unwrap()));

    let authz2 = Authorization::new(true, [".example.com".into()], false, false);
    assert!(authz2.url_in_scope(&Url::parse("https://www.example.com").unwrap()));
}

#[test]
fn host_normalization_case_and_trailing_dot() {
    let authz = Authorization::new(true, ["Example.COM.".into()], false, false);
    assert!(authz
        .validate_targets(&["https://example.com/".into()])
        .is_ok());
}

#[test]
fn invalid_scheme_and_url() {
    let authz = Authorization::new(true, ["example.com".into()], false, false);
    let err = authz
        .validate_targets(&["ftp://example.com/".into()])
        .unwrap_err();
    assert!(matches!(err, AuthzError::InvalidUrl(_)));

    let err = authz
        .validate_targets(&["not a url at all !!!".into()])
        .unwrap_err();
    assert!(matches!(err, AuthzError::InvalidUrl(_)));
}

#[test]
fn active_and_write_gates() {
    let passive = Authorization::new(true, ["x.com".into()], false, false);
    assert!(matches!(
        passive.require_active().unwrap_err(),
        AuthzError::ActiveNotEnabled
    ));
    assert!(matches!(
        passive.require_write().unwrap_err(),
        AuthzError::WriteNotAllowed
    ));

    let active = Authorization::new(true, ["x.com".into()], true, true);
    assert!(active.require_active().is_ok());
    assert!(active.require_write().is_ok());
}

#[test]
fn bare_host_pipeline_normalize_then_authz() {
    let raw = normalize_targets(
        &["Example.COM/app".into(), "127.0.0.1:9".into()],
        NormalizeOptions::default(),
    )
    .unwrap();
    let hosts = normalize_allow_hosts(raw.iter().map(|u| u.clone()).collect::<Vec<_>>());
    // hosts extracted from URLs
    assert!(hosts.iter().any(|h| h == "example.com"));
    assert!(hosts.iter().any(|h| h == "127.0.0.1"));

    let authz = Authorization::new(true, hosts, false, false);
    assert!(authz.validate_targets(&raw).is_ok());
}

#[test]
fn empty_allow_host_entries_filtered() {
    let authz = Authorization::new(true, ["".into(), "  ".into(), "ok.com".into()], false, false);
    assert!(authz.validate_targets(&["https://ok.com/".into()]).is_ok());
    assert!(authz
        .validate_targets(&["https://no.com/".into()])
        .is_err());
}

#[test]
fn url_in_scope_missing_host() {
    let authz = Authorization::new(true, ["example.com".into()], false, false);
    // data: has no host
    let u = Url::parse("data:text/plain,hi").unwrap();
    assert!(!authz.url_in_scope(&u));
}
