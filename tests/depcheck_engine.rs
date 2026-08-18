//! Offline DepCheck engine hit shaping.

use weeping_angel::engines::depcheck_engine::{hits_for_missing, parse_packages_offline};

#[test]
fn offline_parse_package_json() {
    let content = r#"{
      "dependencies": {
        "react": "^18.2.0",
        "acme-private": "1.0.0",
        "local": "file:../local"
      }
    }"#;
    let pkgs = parse_packages_offline("package.json", content);
    let names: Vec<_> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"react"));
    assert!(names.contains(&"acme-private"));
    assert!(!names.iter().any(|n| *n == "local"));
}

#[test]
fn hits_for_missing_packages_shape() {
    let hits = hits_for_missing(
        "Cargo.toml",
        "cargo",
        &[("acme-private-crate".into(), "0.1.0".into())],
    );
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits[0].rule_id,
        "depcheck.confusion.public-registry-missing"
    );
    assert_eq!(hits[0].severity, "high");
    assert!(hits[0].cwe.iter().any(|c| c == "CWE-427"));
    assert!(hits[0].evidence.contains("registry_checked=true"));
    assert_eq!(hits[0].path, "Cargo.toml");
}
