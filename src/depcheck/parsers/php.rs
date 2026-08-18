use std::collections::{BTreeMap, HashSet};

use anyhow::{Context, Result};
use serde_json::Value;

use super::map_to_vec;
use crate::depcheck::filter::is_composer_platform;
use crate::depcheck::types::{Ecosystem, PackageRef};

pub fn parse_composer_json(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let data: Value = serde_json::from_str(content).context("parse composer.json")?;
    let mut local_repos = HashSet::new();
    if let Some(repos) = data.get("repositories").and_then(|v| v.as_array()) {
        for repo in repos {
            let Some(obj) = repo.as_object() else {
                continue;
            };
            let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if matches!(ty, "path" | "vcs" | "git") {
                if let Some(name) = obj
                    .get("package")
                    .and_then(|v| v.as_object())
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                {
                    local_repos.insert(name.to_string());
                }
            }
        }
    }

    let mut packages = BTreeMap::new();
    for section in ["require", "require-dev"] {
        let Some(deps) = data.get(section).and_then(|v| v.as_object()) else {
            continue;
        };
        for (name, version) in deps {
            if is_composer_platform(name) || local_repos.contains(name) {
                continue;
            }
            let ver = version.as_str().unwrap_or("*").to_string();
            packages.insert(name.clone(), ver);
        }
    }
    Ok((map_to_vec(packages), Ecosystem::Composer))
}

pub fn parse_composer_lock(content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let data: Value = serde_json::from_str(content).context("parse composer.lock")?;
    let mut packages = BTreeMap::new();
    for section in ["packages", "packages-dev"] {
        let Some(arr) = data.get(section).and_then(|v| v.as_array()) else {
            continue;
        };
        for pkg in arr {
            let Some(name) = pkg.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            if pkg
                .get("source")
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str())
                == Some("path")
            {
                continue;
            }
            let ver = pkg
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string();
            packages.insert(name.to_string(), ver);
        }
    }
    Ok((map_to_vec(packages), Ecosystem::Composer))
}
