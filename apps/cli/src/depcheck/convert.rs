//! Convert extracted packages to confused-compatible package.json.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::json;

use super::types::PackageRef;

/// Write `{source}-converted.json` with a flat `dependencies` map.
pub fn write_converted(source: &Path, packages: &[PackageRef]) -> Result<std::path::PathBuf> {
    let out = {
        let mut s = source.as_os_str().to_os_string();
        s.push("-converted.json");
        std::path::PathBuf::from(s)
    };
    write_converted_to(&out, packages)?;
    Ok(out)
}

pub fn write_converted_to(out: &Path, packages: &[PackageRef]) -> Result<()> {
    let mut deps = BTreeMap::new();
    for p in packages {
        deps.insert(p.name.clone(), p.version.clone());
    }
    let doc = json!({
        "name": "depcheck-converted",
        "private": true,
        "dependencies": deps,
    });
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(out, serde_json::to_string_pretty(&doc)?)
        .with_context(|| format!("write {}", out.display()))?;
    Ok(())
}

pub fn packages_to_map(packages: &[PackageRef]) -> BTreeMap<String, String> {
    packages
        .iter()
        .map(|p| (p.name.clone(), p.version.clone()))
        .collect()
}
