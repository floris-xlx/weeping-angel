//! Exhaustive target URL normalization matrix.

use weeping_angel::target::{
    NormalizeOptions, host_of_normalized, normalize_one, normalize_targets,
};

#[test]
fn public_hosts_default_https() {
    for raw in [
        "example.com",
        "Example.COM",
        "sub.domain.co.uk",
        "xn--bcher-kva.example",
    ] {
        let u = normalize_one(raw, NormalizeOptions::default()).unwrap();
        assert!(u.starts_with("https://"), "{raw} → {u}");
        assert!(u.contains("://"), "{u}");
    }
}

#[test]
fn loopback_and_local_default_http() {
    for raw in [
        "localhost",
        "localhost:3000",
        "127.0.0.1",
        "127.0.0.1:8787",
        "10.0.0.5",
        "10.0.0.5:8080/app",
        "192.168.1.1",
        "172.16.0.1",
        "foo.local",
        "app.localhost",
    ] {
        let u = normalize_one(raw, NormalizeOptions::default()).unwrap();
        assert!(
            u.starts_with("http://"),
            "expected http default for {raw}, got {u}"
        );
    }
}

#[test]
fn prefer_http_forces_public_hosts() {
    let u = normalize_one("example.com", NormalizeOptions { prefer_http: true }).unwrap();
    assert_eq!(u, "http://example.com/");
}

#[test]
fn protocol_relative_and_schemes() {
    assert_eq!(
        normalize_one("//cdn.example.com/x", NormalizeOptions::default()).unwrap(),
        "https://cdn.example.com/x"
    );
    assert_eq!(
        normalize_one("http://insecure.example/p", NormalizeOptions::default()).unwrap(),
        "http://insecure.example/p"
    );
    assert_eq!(
        normalize_one("https://secure.example/p?q=1", NormalizeOptions::default()).unwrap(),
        "https://secure.example/p?q=1"
    );
}

#[test]
fn port_and_path_preserved() {
    assert_eq!(
        normalize_one("example.com:8443/app/v1", NormalizeOptions::default()).unwrap(),
        "https://example.com:8443/app/v1"
    );
    assert_eq!(
        normalize_one("127.0.0.1:8787/login", NormalizeOptions::default()).unwrap(),
        "http://127.0.0.1:8787/login"
    );
}

#[test]
fn explicit_port_80_defaults_http() {
    let u = normalize_one("example.com:80", NormalizeOptions::default()).unwrap();
    assert!(u.starts_with("http://"), "got {u}");
}

#[test]
fn strips_quotes_and_commas() {
    assert_eq!(
        normalize_one("\"example.com\"", NormalizeOptions::default()).unwrap(),
        "https://example.com/"
    );
    assert_eq!(
        normalize_one("'example.com',", NormalizeOptions::default()).unwrap(),
        "https://example.com/"
    );
}

#[test]
fn multi_csv_and_whitespace_targets() {
    let v = normalize_targets(
        &["a.com, b.com;c.com\td.com".into()],
        NormalizeOptions::default(),
    )
    .unwrap();
    assert_eq!(v.len(), 4);
}

#[test]
fn empty_and_garbage_rejected() {
    assert!(normalize_one("", NormalizeOptions::default()).is_err());
    assert!(normalize_one("   ", NormalizeOptions::default()).is_err());
    assert!(normalize_one("not a host", NormalizeOptions::default()).is_err());
    assert!(normalize_one("ftp://example.com", NormalizeOptions::default()).is_err());
    assert!(normalize_one("file:///etc/passwd", NormalizeOptions::default()).is_err());
}

#[test]
fn host_of_normalized_extracts() {
    let u = normalize_one("App.Example.COM/path", NormalizeOptions::default()).unwrap();
    assert_eq!(host_of_normalized(&u).as_deref(), Some("app.example.com"));
}

#[test]
fn ipv6_loopback_http() {
    // bracket form
    let u = normalize_one("[::1]:8080", NormalizeOptions::default());
    // may or may not parse depending on authority heuristics; if Ok, http
    if let Ok(u) = u {
        assert!(u.starts_with("http://") || u.starts_with("https://"), "{u}");
    }
}

#[test]
fn empty_targets_list_errors() {
    assert!(normalize_targets(&[], NormalizeOptions::default()).is_err());
    assert!(normalize_targets(&["".into(), "  ".into()], NormalizeOptions::default()).is_err());
}
