//! Shared parse helper coverage.

use weeping_angel::parse::{
    LogHttp, expand_list_args, normalize_allow_hosts, parse_bool_loose, parse_consent,
    parse_header_lines, parse_optional_bool, split_list,
};

#[test]
fn split_list_delimiters() {
    assert_eq!(split_list("a, b;c\td  e,,"), vec!["a", "b", "c", "d", "e"]);
    assert!(split_list("").is_empty());
    assert_eq!(split_list("  single  "), vec!["single"]);
}

#[test]
fn expand_list_args_flattens() {
    let v = expand_list_args(&["a,b".into(), "c".into(), "d e".into()]);
    assert_eq!(v, vec!["a", "b", "c", "d", "e"]);
}

#[test]
fn bool_loose_matrix() {
    for t in ["true", "TRUE", "yes", "Y", "1", "on", " On "] {
        assert_eq!(parse_bool_loose(t).unwrap(), true, "{t}");
    }
    for f in ["false", "no", "N", "0", "off"] {
        assert_eq!(parse_bool_loose(f).unwrap(), false, "{f}");
    }
    assert!(parse_bool_loose("maybe").is_err());
}

#[test]
fn consent_matrix() {
    for t in [
        "",
        "true",
        "yes",
        "y",
        "1",
        "on",
        "i-own-this",
        "owned",
        "authorized",
    ] {
        assert_eq!(parse_consent(t).unwrap(), true, "consent {t:?}");
    }
    for f in ["false", "no", "0", "off"] {
        assert!(parse_consent(f).is_err(), "should reject {f}");
    }
    assert!(parse_consent("banana").is_err());
}

#[test]
fn optional_bool_empty_is_true() {
    assert_eq!(parse_optional_bool("").unwrap(), true);
    assert_eq!(parse_optional_bool("false").unwrap(), false);
}

#[test]
fn headers_colon_eq_and_errors() {
    let h = parse_header_lines(&[
        "Authorization: Bearer tok".into(),
        "X-Custom=val".into(),
        "  X-Trim :  spaced  ".into(),
        "X-Space secret".into(),
    ])
    .unwrap();
    assert_eq!(h.len(), 4);
    assert_eq!(h[0].0, "Authorization");
    assert_eq!(h[0].1, "Bearer tok");
    assert_eq!(h[1].0, "X-Custom");
    assert_eq!(h[1].1, "val");
    assert_eq!(h[2].0, "X-Trim");
    assert_eq!(h[2].1, "spaced");
    assert_eq!(h[3].0, "X-Space");
    assert_eq!(h[3].1, "secret");

    assert!(parse_header_lines(&["nocolonoreq".into()]).is_err());
    assert!(parse_header_lines(&[": no-name".into()]).is_err());
    assert!(parse_header_lines(&["=no-name".into()]).is_err());
}

#[test]
fn allow_hosts_url_wildcard_csv_port() {
    let h = normalize_allow_hosts(vec![
        "https://App.Example.com:443/path".into(),
        "*.cdn.example.com, .example.org".into(),
        "127.0.0.1:8787".into(),
        "//proto.example".into(),
    ]);
    assert!(h.iter().any(|x| x == "app.example.com"), "{h:?}");
    assert!(h.iter().any(|x| x == "*.cdn.example.com"), "{h:?}");
    assert!(
        h.iter().any(|x| x == ".example.org" || x == "example.org"),
        "{h:?}"
    );
    assert!(h.iter().any(|x| x == "127.0.0.1"), "{h:?}");
    assert!(h.iter().any(|x| x == "proto.example"), "{h:?}");
}

#[test]
fn log_http_parse_and_str() {
    assert_eq!(LogHttp::parse("FULL"), Some(LogHttp::Full));
    assert_eq!(LogHttp::parse("short"), Some(LogHttp::Compact));
    assert_eq!(LogHttp::parse("sum"), Some(LogHttp::Summary));
    assert_eq!(LogHttp::parse("none"), Some(LogHttp::Off));
    assert_eq!(LogHttp::parse("nope"), None);
    assert_eq!(LogHttp::Full.as_str(), "full");
    assert_eq!(LogHttp::default(), LogHttp::Compact);
}
