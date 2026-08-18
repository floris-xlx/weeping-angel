use std::collections::BTreeMap;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;

use super::map_to_vec;
use crate::depcheck::filter::is_remote_or_path_spec;
use crate::depcheck::types::{Ecosystem, PackageRef};

pub fn parse_requirements_txt(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    static NAME: OnceLock<Regex> = OnceLock::new();
    let name_re =
        NAME.get_or_init(|| Regex::new(r"^([a-zA-Z0-9][\w.-]*)").expect("req name"));
    static VER: OnceLock<Regex> = OnceLock::new();
    let ver_re =
        VER.get_or_init(|| Regex::new(r"[=<>!~]+\s*(.+)").expect("req ver"));
    static URL_AT: OnceLock<Regex> = OnceLock::new();
    let url_re = URL_AT.get_or_init(|| {
        Regex::new(r"\s+@\s+(https?://|git\+|file://)").expect("req url")
    });

    for line in content.lines() {
        let mut line = line.trim().to_string();
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }
        if is_remote_or_path_spec(&line) {
            continue;
        }
        if let Some(idx) = line.find('#') {
            line.truncate(idx);
            line = line.trim().to_string();
        }
        if let Some(idx) = line.find(';') {
            line.truncate(idx);
            line = line.trim().to_string();
        }
        if url_re.is_match(&line) {
            continue;
        }
        if let Some(c) = name_re.captures(&line) {
            let name = c[1].to_string();
            let version = ver_re
                .captures(&line)
                .map(|m| m[1].trim().to_string())
                .unwrap_or_else(|| "*".into());
            packages.insert(name, version);
        }
    }
    Ok((map_to_vec(packages), Ecosystem::Pip))
}

pub fn parse_pipfile(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    let mut in_pkgs = false;
    static GIT: OnceLock<Regex> = OnceLock::new();
    let git = GIT.get_or_init(|| {
        Regex::new(r"\{\s*(git|path|url|file)\s*=").expect("pipfile git")
    });

    for line in content.lines() {
        let s = line.trim();
        if s == "[packages]" || s == "[dev-packages]" {
            in_pkgs = true;
            continue;
        }
        if s.starts_with('[') && in_pkgs {
            in_pkgs = false;
            continue;
        }
        if in_pkgs && s.contains('=') && !s.starts_with('#') {
            let (name, value) = s.split_once('=').unwrap();
            let name = name.trim();
            let value = value.trim();
            if git.is_match(value) {
                continue;
            }
            if !name.is_empty() {
                packages.insert(
                    name.to_string(),
                    value.trim_matches(|c| c == '"' || c == '\'').to_string(),
                );
            }
        }
    }
    Ok((map_to_vec(packages), Ecosystem::Pip))
}

pub fn parse_pipfile_lock(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let data: Value = serde_json::from_str(content).context("parse Pipfile.lock")?;
    let mut packages = BTreeMap::new();
    for section in ["default", "develop"] {
        let Some(obj) = data.get(section).and_then(|v| v.as_object()) else {
            continue;
        };
        for (name, info) in obj {
            match info {
                Value::Object(o) => {
                    if ["git", "path", "file", "directory"]
                        .iter()
                        .any(|k| o.contains_key(*k))
                    {
                        continue;
                    }
                    let ver = o
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .trim_start_matches('=')
                        .to_string();
                    packages.insert(name.clone(), ver);
                }
                _ => {
                    packages.insert(name.clone(), "*".into());
                }
            }
        }
    }
    Ok((map_to_vec(packages), Ecosystem::Pip))
}

pub fn parse_pyproject_toml(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();
    let mut in_deps = false;

    static POETRY: OnceLock<Regex> = OnceLock::new();
    let poetry = POETRY.get_or_init(|| {
        Regex::new(r"^\[tool\.poetry\.(dev-)?dependencies\]|^\[tool\.poetry\.group\.\w+\.dependencies\]")
            .expect("poetry section")
    });
    static PATH: OnceLock<Regex> = OnceLock::new();
    let path = PATH.get_or_init(|| Regex::new(r"\b(git|path|url)\s*=").expect("poetry path"));
    static QUOTED: OnceLock<Regex> = OnceLock::new();
    let quoted = QUOTED.get_or_init(|| Regex::new(r#""([^"]*)""#).expect("quoted"));

    for line in content.lines() {
        let s = line.trim();
        if poetry.is_match(s) {
            in_deps = true;
            continue;
        }
        if s.starts_with('[') && in_deps {
            in_deps = false;
            continue;
        }
        if in_deps && s.contains('=') && !s.starts_with('#') {
            let name = s.split('=').next().unwrap_or("").trim();
            if name.is_empty() || name == "python" {
                continue;
            }
            if path.is_match(s) {
                continue;
            }
            let version = quoted
                .captures(s)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| "*".into());
            packages.insert(name.to_string(), version);
        }
    }

    // PEP 621 dependencies = [ ... ]
    static PEP: OnceLock<Regex> = OnceLock::new();
    let pep = PEP.get_or_init(|| {
        Regex::new(r"(?ms)^\s*dependencies\s*=\s*\[(.*?)\]").expect("pep621 deps")
    });
    static NAME: OnceLock<Regex> = OnceLock::new();
    let name_re =
        NAME.get_or_init(|| Regex::new(r"^([a-zA-Z0-9][\w.-]*)").expect("pep name"));
    if let Some(c) = pep.captures(content) {
        for m in quoted.captures_iter(&c[1]) {
            let dep = m[1].trim();
            if let Some(n) = name_re.captures(dep) {
                let name = &n[1];
                if name != "python" {
                    let rest = &dep[name.len()..];
                    packages.insert(
                        name.to_string(),
                        if rest.is_empty() {
                            "*".into()
                        } else {
                            rest.to_string()
                        },
                    );
                }
            }
        }
    }

    static OPT: OnceLock<Regex> = OnceLock::new();
    let opt = OPT.get_or_init(|| {
        Regex::new(r"(?ms)^\[project\.optional-dependencies\.\w+\]\s*$\n(.*?)(?=^\[|\z)")
            .expect("optional deps")
    });
    for c in opt.captures_iter(content) {
        for m in quoted.captures_iter(&c[1]) {
            if let Some(n) = name_re.captures(m[1].trim()) {
                packages.insert(n[1].to_string(), "*".into());
            }
        }
    }

    Ok((map_to_vec(packages), Ecosystem::Pip))
}
