use std::collections::BTreeMap;
use std::sync::OnceLock;

use anyhow::Result;
use regex::Regex;

use super::map_to_vec;
use crate::depcheck::types::{Ecosystem, PackageRef};

pub fn parse_gemfile(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    static PATH: OnceLock<Regex> = OnceLock::new();
    let path = PATH.get_or_init(|| Regex::new(r"\b(path|git)\s*:").expect("gem path"));
    static GEM: OnceLock<Regex> = OnceLock::new();
    let gem = GEM.get_or_init(|| {
        Regex::new(r#"gem\s+['"]([^'"]+)['"](?:\s*,\s*['"]([^'"]*)['"])?"#).expect("gem line")
    });

    for line in content.lines() {
        let s = line.trim();
        if s.starts_with('#') || path.is_match(s) {
            continue;
        }
        if let Some(c) = gem.captures(s) {
            let name = c[1].to_string();
            let version = c
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "*".into());
            packages.insert(name, version);
        }
    }
    Ok((map_to_vec(packages), Ecosystem::Rubygems))
}

pub fn parse_gemfile_lock(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    let mut in_gem = false;
    let mut in_specs = false;
    static SPEC: OnceLock<Regex> = OnceLock::new();
    let spec =
        SPEC.get_or_init(|| Regex::new(r"^\s{4}(\S+)\s+\(([^)]+)\)").expect("gemfile.lock spec"));

    for line in content.lines() {
        let s = line.trim();
        match s {
            "GEM" => {
                in_gem = true;
                in_specs = false;
                continue;
            }
            "GIT" | "PATH" | "PLUGIN SOURCE" => {
                in_gem = false;
                in_specs = false;
                continue;
            }
            "specs:" => {
                in_specs = true;
                continue;
            }
            _ => {}
        }
        if in_gem && in_specs {
            if let Some(c) = spec.captures(line) {
                packages.insert(c[1].to_string(), c[2].to_string());
            } else if !line.starts_with(' ') && !s.is_empty() {
                in_specs = false;
                in_gem = false;
            }
        }
    }
    Ok((map_to_vec(packages), Ecosystem::Rubygems))
}
