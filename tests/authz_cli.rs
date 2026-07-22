use weeping_angel::authz::{Authorization, AuthzError};

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
