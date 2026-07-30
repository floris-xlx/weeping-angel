use weeping_angel::authz::{Authorization, AuthzError};
use weeping_angel::parse::{normalize_allow_hosts, parse_consent};
use weeping_angel::target::{NormalizeOptions, normalize_one, normalize_targets};

#[test]
fn no_network_without_consent() {
    let authz = Authorization::new(false, ["localhost".into()], false, false);
    let err = authz
        .validate_targets(&["http://localhost/".into()])
        .unwrap_err();
    assert!(matches!(err, AuthzError::MissingConsent));
}

#[test]
fn allowlist_required() {
    let authz = Authorization::new(true, Vec::<String>::new(), false, false);
    let err = authz
        .validate_targets(&["http://localhost/".into()])
        .unwrap_err();
    assert!(matches!(err, AuthzError::EmptyAllowlist));
}

#[test]
fn bare_host_normalizes_and_validates() {
    let targets = normalize_targets(&["example.com".into()], NormalizeOptions::default()).unwrap();
    assert_eq!(targets[0], "https://example.com/");
    let authz = Authorization::new(true, ["example.com".into()], false, false);
    let urls = authz.validate_targets(&targets).unwrap();
    assert_eq!(urls.len(), 1);
}

#[test]
fn loopback_defaults_http() {
    let u = normalize_one("127.0.0.1:8787", NormalizeOptions::default()).unwrap();
    assert_eq!(u, "http://127.0.0.1:8787/");
}

#[test]
fn consent_parser_accepts_yes() {
    assert_eq!(parse_consent("yes").unwrap(), true);
    assert_eq!(parse_consent("true").unwrap(), true);
    assert!(parse_consent("false").is_err());
}

#[test]
fn allow_host_from_url_csv() {
    let h = normalize_allow_hosts(vec!["https://App.Example.com/x, other.test".into()]);
    assert!(h.iter().any(|x| x == "app.example.com"));
    assert!(h.iter().any(|x| x == "other.test"));
}
