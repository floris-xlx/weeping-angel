//! Offline DepCheck parser / detect / filter / convert tests.

use std::fs;
use std::path::PathBuf;

use weeping_angel::depcheck::convert::packages_to_map;
use weeping_angel::depcheck::detect::{detect_file_type, detect_from_content};
use weeping_angel::depcheck::filter::{
    filter_packages, is_composer_platform, is_remote_or_path_spec, resolve_npm_alias,
};
use weeping_angel::depcheck::parsers::parse_manifest;
use weeping_angel::depcheck::types::{Ecosystem, FileKind, PackageRef};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/depcheck")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    fs::read_to_string(fixture(name)).unwrap_or_else(|e| panic!("read {name}: {e}"))
}

#[test]
fn detects_standard_filenames() {
    assert_eq!(
        detect_file_type(&fixture("package.json"), None),
        FileKind::PackageJson
    );
    assert_eq!(
        detect_file_type(&fixture("requirements.txt"), None),
        FileKind::RequirementsTxt
    );
    assert_eq!(
        detect_file_type(&fixture("Cargo.toml"), None),
        FileKind::CargoToml
    );
    assert_eq!(detect_file_type(&fixture("go.mod"), None), FileKind::GoMod);
    assert_eq!(
        detect_file_type(&fixture("composer.json"), None),
        FileKind::ComposerJson
    );
    assert_eq!(
        detect_file_type(&fixture("pom.xml"), None),
        FileKind::PomXml
    );
    assert_eq!(
        detect_file_type(&fixture("packages.config"), None),
        FileKind::NugetPackagesConfig
    );
}

#[test]
fn parse_package_json_skips_git_and_file() {
    let content = read_fixture("package.json");
    let (pkgs, eco) = parse_manifest(FileKind::PackageJson, &content).unwrap();
    assert_eq!(eco, Ecosystem::Npm);
    let names: Vec<_> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"react"));
    assert!(names.contains(&"acme-billing-sdk-not-real-xyz"));
    assert!(names.contains(&"typescript"));
    assert!(!names.contains(&"left-pad"));
    assert!(!names.contains(&"local-helper"));
}

#[test]
fn parse_requirements_skips_vcs_and_flags() {
    let content = read_fixture("requirements.txt");
    let (pkgs, eco) = parse_manifest(FileKind::RequirementsTxt, &content).unwrap();
    assert_eq!(eco, Ecosystem::Pip);
    let names: Vec<_> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"requests"));
    assert!(names.contains(&"acme-internal-logger-xyz"));
    assert!(names.contains(&"flask"));
    assert_eq!(names.len(), 3);
}

#[test]
fn parse_cargo_toml_skips_path_and_git() {
    let content = read_fixture("Cargo.toml");
    let (pkgs, eco) = parse_manifest(FileKind::CargoToml, &content).unwrap();
    assert_eq!(eco, Ecosystem::Cargo);
    let names: Vec<_> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"serde"));
    assert!(names.contains(&"acme-private-crate-xyz"));
    assert!(!names.contains(&"local-util"));
    assert!(!names.contains(&"git-dep"));
}

#[test]
fn parse_go_mod_drops_replaced_modules() {
    let content = read_fixture("go.mod");
    let (pkgs, eco) = parse_manifest(FileKind::GoMod, &content).unwrap();
    assert_eq!(eco, Ecosystem::Go);
    let names: Vec<_> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"github.com/stretchr/testify"));
    assert!(!names.contains(&"example.com/internal/acme-go-sdk-xyz"));
}

#[test]
fn parse_composer_skips_platform() {
    let content = read_fixture("composer.json");
    let (pkgs, eco) = parse_manifest(FileKind::ComposerJson, &content).unwrap();
    assert_eq!(eco, Ecosystem::Composer);
    let names: Vec<_> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"monolog/monolog"));
    assert!(names.contains(&"acme/internal-billing-xyz"));
    assert!(names.contains(&"phpunit/phpunit"));
    assert!(!names.contains(&"php"));
    assert!(!names.contains(&"ext-json"));
}

#[test]
fn parse_pom_xml_group_artifact() {
    let content = read_fixture("pom.xml");
    let (pkgs, eco) = parse_manifest(FileKind::PomXml, &content).unwrap();
    assert_eq!(eco, Ecosystem::Maven);
    let names: Vec<_> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"com.google.guava:guava"));
    assert!(names.contains(&"com.acme.internal:billing-sdk-xyz"));
}

#[test]
fn parse_packages_config() {
    let content = read_fixture("packages.config");
    let (pkgs, eco) = parse_manifest(FileKind::NugetPackagesConfig, &content).unwrap();
    assert_eq!(eco, Ecosystem::Nuget);
    let names: Vec<_> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"Newtonsoft.Json"));
    assert!(names.contains(&"Acme.Internal.Auth.Xyz"));
}

#[test]
fn resolve_npm_alias_scoped() {
    let (n, v) = resolve_npm_alias("npm:@scope/pkg@^1.0.0").unwrap();
    assert_eq!(n, "@scope/pkg");
    assert_eq!(v, "^1.0.0");
}

#[test]
fn filter_skips_remote_specs() {
    assert!(is_remote_or_path_spec("git+https://example.com/x.git"));
    assert!(is_remote_or_path_spec("file:../x"));
    assert!(!is_remote_or_path_spec("^1.2.3"));
    assert!(is_composer_platform("ext-mbstring"));
}

#[test]
fn convert_map_preserves_names() {
    let pkgs = vec![
        PackageRef::new("react", "^18"),
        PackageRef::new("lodash", "4.17.21"),
    ];
    let map = packages_to_map(&pkgs);
    assert_eq!(map.get("react").map(String::as_str), Some("^18"));
    assert_eq!(map.len(), 2);
}

#[test]
fn detect_go_mod_content() {
    let c = "module example.com/foo\n\ngo 1.22\n";
    assert_eq!(detect_from_content(c), FileKind::GoMod);
}

#[test]
fn filter_packages_drops_empty() {
    let filtered = filter_packages(
        Ecosystem::Npm,
        vec![
            PackageRef::new("react", "^18"),
            PackageRef::new("", "*"),
            PackageRef::new(".hidden", "*"),
        ],
    );
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "react");
}
