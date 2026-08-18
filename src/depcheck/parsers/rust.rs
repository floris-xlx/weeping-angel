use std::collections::BTreeMap;
use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;

use super::map_to_vec;
use crate::depcheck::types::{Ecosystem, PackageRef};

pub fn parse_cargo_toml(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    let mut in_deps = false;

    static DEPS_SEC: OnceLock<Regex> = OnceLock::new();
    let deps_sec =
        DEPS_SEC.get_or_init(|| Regex::new(r"^\[(.*dependencies.*)\]$").expect("cargo deps sec"));
    static PATH: OnceLock<Regex> = OnceLock::new();
    let path = PATH.get_or_init(|| Regex::new(r"\b(path|git)\s*=").expect("cargo path"));
    static QUOTED: OnceLock<Regex> = OnceLock::new();
    let quoted = QUOTED.get_or_init(|| Regex::new(r#""([^"]*)""#).expect("quoted"));

    for line in content.lines() {
        let s = line.trim();
        if deps_sec.is_match(s) {
            in_deps = true;
            continue;
        }
        if s.starts_with('[') && in_deps {
            in_deps = false;
            continue;
        }
        if in_deps && s.contains('=') && !s.starts_with('#') {
            if path.is_match(s) {
                continue;
            }
            let name = s.split('=').next().unwrap_or("").trim();
            if name.is_empty() {
                continue;
            }
            let version = quoted
                .captures(s)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "*".into());
            packages.insert(name.to_string(), version);
        }
    }
    Ok((map_to_vec(packages), Ecosystem::Cargo))
}

pub fn parse_cargo_lock(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    static NAME: OnceLock<Regex> = OnceLock::new();
    let name_re =
        NAME.get_or_init(|| Regex::new(r#"(?m)^name\s*=\s*"([^"]+)""#).expect("lock name"));
    static VER: OnceLock<Regex> = OnceLock::new();
    let ver_re =
        VER.get_or_init(|| Regex::new(r#"(?m)^version\s*=\s*"([^"]+)""#).expect("lock ver"));
    static SRC: OnceLock<Regex> = OnceLock::new();
    let src_re =
        SRC.get_or_init(|| Regex::new(r#"(?m)^source\s*=\s*"([^"]+)""#).expect("lock src"));

    for block in content.split("[[package]]") {
        let Some(nm) = name_re.captures(block) else {
            continue;
        };
        let Some(vm) = ver_re.captures(block) else {
            continue;
        };
        let Some(sm) = src_re.captures(block) else {
            continue;
        };
        if !sm[1].starts_with("registry+") {
            continue;
        }
        packages.insert(nm[1].to_string(), vm[1].to_string());
    }
    Ok((map_to_vec(packages), Ecosystem::Cargo))
}
