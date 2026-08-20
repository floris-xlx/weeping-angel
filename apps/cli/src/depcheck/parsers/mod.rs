//! Dependency manifest parsers.

mod go;
mod java;
mod npm;
mod nuget;
mod php;
mod python;
mod ruby;
mod rust;

use anyhow::{Result, bail};

use super::types::{Ecosystem, FileKind, PackageRef};

/// Parse content for a known file kind → packages + ecosystem.
pub fn parse_manifest(kind: FileKind, content: &str) -> Result<(Vec<PackageRef>, Ecosystem)> {
    let (map, eco) = match kind {
        FileKind::PackageJson => npm::parse_package_json(content)?,
        FileKind::PackageLockJson => npm::parse_package_lock_json(content)?,
        FileKind::YarnLock => npm::parse_yarn_lock(content)?,
        FileKind::PnpmLock => npm::parse_pnpm_lock(content)?,
        FileKind::RequirementsTxt => python::parse_requirements_txt(content)?,
        FileKind::Pipfile => python::parse_pipfile(content)?,
        FileKind::PipfileLock => python::parse_pipfile_lock(content)?,
        FileKind::PyprojectToml => python::parse_pyproject_toml(content)?,
        FileKind::ComposerJson => php::parse_composer_json(content)?,
        FileKind::ComposerLock => php::parse_composer_lock(content)?,
        FileKind::Gemfile => ruby::parse_gemfile(content)?,
        FileKind::GemfileLock => ruby::parse_gemfile_lock(content)?,
        FileKind::PomXml => java::parse_pom_xml(content)?,
        FileKind::BuildGradle => java::parse_build_gradle(content)?,
        FileKind::GoMod => go::parse_go_mod(content)?,
        FileKind::CargoToml => rust::parse_cargo_toml(content)?,
        FileKind::CargoLock => rust::parse_cargo_lock(content)?,
        FileKind::NugetPackagesConfig => nuget::parse_packages_config(content)?,
        FileKind::Csproj => nuget::parse_csproj(content)?,
        FileKind::Unknown => bail!("unknown dependency file type"),
    };
    Ok((map, eco))
}

pub(crate) fn map_to_vec(map: std::collections::BTreeMap<String, String>) -> Vec<PackageRef> {
    map.into_iter()
        .map(|(name, version)| PackageRef { name, version })
        .collect()
}
