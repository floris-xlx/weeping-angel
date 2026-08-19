use std::collections::BTreeMap;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;

use super::map_to_vec;
use crate::depcheck::filter::{is_remote_or_path_spec, resolve_npm_alias};
use crate::depcheck::types::{Ecosystem, PackageRef};

pub fn parse_package_json(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let data: Value = serde_json::from_str(content).context("parse package.json")?;
    let mut packages = BTreeMap::new();

    for section in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        let Some(deps) = data.get(section).and_then(|v| v.as_object()) else {
            continue;
        };
        for (name, version) in deps {
            match version {
                Value::String(v) => {
                    if let Some((n, ver)) = resolve_npm_alias(v) {
                        packages.insert(n, ver);
                    } else if !is_remote_or_path_spec(v) {
                        packages.insert(name.clone(), v.clone());
                    }
                }
                Value::Object(obj) => {
                    if obj.contains_key("git")
                        || obj.contains_key("url")
                        || obj.contains_key("path")
                        || obj.contains_key("file")
                    {
                        continue;
                    }
                    let ver = obj
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .to_string();
                    packages.insert(name.clone(), ver);
                }
                _ => {}
            }
        }
    }

    for key in ["bundledDependencies", "bundleDependencies"] {
        if let Some(arr) = data.get(key).and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(name) = item.as_str() {
                    packages
                        .entry(name.to_string())
                        .or_insert_with(|| "*".into());
                }
            }
        }
    }

    Ok((map_to_vec(packages), Ecosystem::Npm))
}

pub fn parse_package_lock_json(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let data: Value = serde_json::from_str(content).context("parse package-lock.json")?;
    let mut packages = BTreeMap::new();

    if let Some(pkgs) = data.get("packages").and_then(|v| v.as_object()) {
        for (path, info) in pkgs {
            if path.is_empty() {
                continue;
            }
            let Some(obj) = info.as_object() else {
                continue;
            };
            if obj.get("link").and_then(|v| v.as_bool()) == Some(true) {
                continue;
            }
            if let Some(resolved) = obj.get("resolved").and_then(|v| v.as_str()) {
                if resolved.starts_with("file:")
                    || resolved.starts_with("git+")
                    || resolved.starts_with("git://")
                {
                    continue;
                }
            }
            let name = if let Some(n) = obj.get("name").and_then(|v| v.as_str()) {
                n.to_string()
            } else if let Some(idx) = path.rfind("node_modules/") {
                path[idx + "node_modules/".len()..].to_string()
            } else {
                path.clone()
            };
            if name.is_empty() || name.starts_with('.') {
                continue;
            }
            let ver = obj
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();
            packages.insert(name, ver);
        }
    }

    if packages.is_empty() {
        if let Some(deps) = data.get("dependencies") {
            extract_lock_v1(deps, &mut packages);
        }
    }

    Ok((map_to_vec(packages), Ecosystem::Npm))
}

fn extract_lock_v1(deps: &Value, packages: &mut BTreeMap<String, String>) {
    let Some(obj) = deps.as_object() else {
        return;
    };
    for (name, info) in obj {
        match info {
            Value::Object(o) => {
                if let Some(resolved) = o.get("resolved").and_then(|v| v.as_str()) {
                    if resolved.starts_with("file:")
                        || resolved.starts_with("git+")
                        || resolved.starts_with("git://")
                    {
                        continue;
                    }
                }
                let mut pkg_name = name.clone();
                if let Some(from) = o.get("from").and_then(|v| v.as_str()) {
                    if from.contains("npm:") {
                        static RE: OnceLock<Regex> = OnceLock::new();
                        let re = RE
                            .get_or_init(|| Regex::new(r"npm:(@?[^@]+)").expect("npm from regex"));
                        if let Some(c) = re.captures(from) {
                            pkg_name = c[1].to_string();
                        }
                    }
                }
                let ver = o
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0")
                    .to_string();
                packages.insert(pkg_name, ver);
                if let Some(nested) = o.get("dependencies") {
                    extract_lock_v1(nested, packages);
                }
            }
            Value::String(v) => {
                packages.insert(name.clone(), v.clone());
            }
            _ => {}
        }
    }
}

pub fn parse_yarn_lock(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();

    static V1: OnceLock<Regex> = OnceLock::new();
    let v1 = V1.get_or_init(|| {
        Regex::new(r#"(?m)^"?(@?[^@"\s][^@"]*?)@(?:npm:)?[^:]+:\s*$\n\s+version\s+"?([^"\n]+)"?"#)
            .expect("yarn v1 regex") // panic-ok: regex literal
    });
    for caps in v1.captures_iter(content) {
        let name = caps[1].trim_matches('"').to_string();
        if !name.is_empty() {
            packages.insert(name, caps[2].trim().to_string());
        }
    }

    if packages.is_empty() {
        let mut current: Option<String> = None;
        static KEY: OnceLock<Regex> = OnceLock::new();
        let key = KEY
            .get_or_init(|| Regex::new(r#"^"?(@?[^@"\s][^@"]*?)@[^"]*"?:\s*$"#).expect("yarn key"));
        static VER: OnceLock<Regex> = OnceLock::new();
        let ver =
            VER.get_or_init(|| Regex::new(r#"\s+version:?\s+"?([^"\n]+)"#).expect("yarn ver"));
        for line in content.lines() {
            if let Some(c) = key.captures(line) {
                current = Some(c[1].trim_matches('"').to_string());
            } else if let Some(cur) = current.take() {
                if let Some(c) = ver.captures(line) {
                    packages.insert(cur, c[1].trim_matches('"').to_string());
                } else {
                    current = Some(cur);
                }
            }
        }
    }

    Ok((map_to_vec(packages), Ecosystem::Npm))
}

pub fn parse_pnpm_lock(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let mut packages = BTreeMap::new();

    static V9: OnceLock<Regex> = OnceLock::new();
    let v9 =
        V9.get_or_init(|| Regex::new(r"(?m)^'?(@?[^@'\s]+)@(\d[^':\s]*)'?:").expect("pnpm v9"));
    for caps in v9.captures_iter(content) {
        packages.insert(caps[1].to_string(), caps[2].to_string());
    }

    if packages.is_empty() {
        static V6: OnceLock<Regex> = OnceLock::new();
        let v6 = V6.get_or_init(|| {
            Regex::new(r"(?m)^\s*/(@?[^/]+(?:/[^/]+)?)/(\d[^:]*?):").expect("pnpm v6")
        });
        for caps in v6.captures_iter(content) {
            packages.insert(caps[1].to_string(), caps[2].to_string());
        }
    }

    Ok((map_to_vec(packages), Ecosystem::Npm))
}
