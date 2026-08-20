//! Shared DepCheck types (detection-only).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Public package ecosystem used for registry lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    Npm,
    Pip,
    Composer,
    Rubygems,
    Nuget,
    Cargo,
    Go,
    Maven,
}

impl Ecosystem {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Pip => "pip",
            Self::Composer => "composer",
            Self::Rubygems => "rubygems",
            Self::Nuget => "nuget",
            Self::Cargo => "cargo",
            Self::Go => "go",
            Self::Maven => "maven",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "npm" | "yarn" | "pnpm" | "javascript" | "js" => Some(Self::Npm),
            "pip" | "pypi" | "python" => Some(Self::Pip),
            "composer" | "packagist" | "php" => Some(Self::Composer),
            "rubygems" | "gem" | "ruby" => Some(Self::Rubygems),
            "nuget" | "dotnet" | "csharp" => Some(Self::Nuget),
            "cargo" | "crates" | "crates.io" | "rust" => Some(Self::Cargo),
            "go" | "golang" => Some(Self::Go),
            "maven" | "mvn" | "gradle" | "java" => Some(Self::Maven),
            // DepFuzzer uses "pypi" as provider name
            _ => None,
        }
    }

    /// All ecosystems DepFuzzer `--provider all` iterates.
    pub fn all_providers() -> &'static [Ecosystem] {
        &[
            Self::Npm,
            Self::Pip,
            Self::Cargo,
            Self::Go,
            Self::Maven,
            Self::Rubygems,
            Self::Nuget,
            Self::Composer,
        ]
    }

    /// confused-compatible default manifest kind when only `-l` is given.
    pub fn default_file_kind(self) -> FileKind {
        match self {
            Self::Npm => FileKind::PackageJson,
            Self::Pip => FileKind::RequirementsTxt,
            Self::Composer => FileKind::ComposerJson,
            Self::Rubygems => FileKind::GemfileLock,
            Self::Maven => FileKind::PomXml,
            Self::Cargo => FileKind::CargoToml,
            Self::Go => FileKind::GoMod,
            Self::Nuget => FileKind::NugetPackagesConfig,
        }
    }
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Detected / overridden dependency file kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    PackageJson,
    PackageLockJson,
    YarnLock,
    PnpmLock,
    RequirementsTxt,
    Pipfile,
    PipfileLock,
    PyprojectToml,
    ComposerJson,
    ComposerLock,
    Gemfile,
    GemfileLock,
    PomXml,
    BuildGradle,
    GoMod,
    CargoToml,
    CargoLock,
    NugetPackagesConfig,
    Csproj,
    Unknown,
}

impl FileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PackageJson => "package_json",
            Self::PackageLockJson => "package_lock_json",
            Self::YarnLock => "yarn_lock",
            Self::PnpmLock => "pnpm_lock",
            Self::RequirementsTxt => "requirements_txt",
            Self::Pipfile => "pipfile",
            Self::PipfileLock => "pipfile_lock",
            Self::PyprojectToml => "pyproject_toml",
            Self::ComposerJson => "composer_json",
            Self::ComposerLock => "composer_lock",
            Self::Gemfile => "gemfile",
            Self::GemfileLock => "gemfile_lock",
            Self::PomXml => "pom_xml",
            Self::BuildGradle => "build_gradle",
            Self::GoMod => "go_mod",
            Self::CargoToml => "cargo_toml",
            Self::CargoLock => "cargo_lock",
            Self::NugetPackagesConfig => "nuget_packages_config",
            Self::Csproj => "csproj",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "package_json" | "package.json" => Some(Self::PackageJson),
            "package_lock_json" | "package_lock" | "package-lock.json" => {
                Some(Self::PackageLockJson)
            }
            "yarn_lock" | "yarn.lock" => Some(Self::YarnLock),
            "pnpm_lock" | "pnpm-lock.yaml" | "pnpm_lock_yaml" => Some(Self::PnpmLock),
            "requirements_txt" | "requirements.txt" => Some(Self::RequirementsTxt),
            "pipfile" => Some(Self::Pipfile),
            "pipfile_lock" | "pipfile.lock" => Some(Self::PipfileLock),
            "pyproject_toml" | "pyproject.toml" => Some(Self::PyprojectToml),
            "composer_json" | "composer.json" => Some(Self::ComposerJson),
            "composer_lock" | "composer.lock" => Some(Self::ComposerLock),
            "gemfile" => Some(Self::Gemfile),
            "gemfile_lock" | "gemfile.lock" => Some(Self::GemfileLock),
            "pom_xml" | "pom.xml" => Some(Self::PomXml),
            "build_gradle" | "build.gradle" | "build.gradle.kts" => Some(Self::BuildGradle),
            "go_mod" | "go.mod" | "go.sum" => Some(Self::GoMod),
            "cargo_toml" | "cargo.toml" => Some(Self::CargoToml),
            "cargo_lock" | "cargo.lock" => Some(Self::CargoLock),
            "nuget_packages_config" | "packages.config" => Some(Self::NugetPackagesConfig),
            "csproj" => Some(Self::Csproj),
            _ => None,
        }
    }

    pub fn ecosystem(self) -> Option<Ecosystem> {
        match self {
            Self::PackageJson | Self::PackageLockJson | Self::YarnLock | Self::PnpmLock => {
                Some(Ecosystem::Npm)
            }
            Self::RequirementsTxt | Self::Pipfile | Self::PipfileLock | Self::PyprojectToml => {
                Some(Ecosystem::Pip)
            }
            Self::ComposerJson | Self::ComposerLock => Some(Ecosystem::Composer),
            Self::Gemfile | Self::GemfileLock => Some(Ecosystem::Rubygems),
            Self::PomXml | Self::BuildGradle => Some(Ecosystem::Maven),
            Self::GoMod => Some(Ecosystem::Go),
            Self::CargoToml | Self::CargoLock => Some(Ecosystem::Cargo),
            Self::NugetPackagesConfig | Self::Csproj => Some(Ecosystem::Nuget),
            Self::Unknown => None,
        }
    }

    pub fn all_known() -> &'static [FileKind] {
        &[
            Self::PackageJson,
            Self::PackageLockJson,
            Self::YarnLock,
            Self::PnpmLock,
            Self::RequirementsTxt,
            Self::Pipfile,
            Self::PipfileLock,
            Self::PyprojectToml,
            Self::ComposerJson,
            Self::ComposerLock,
            Self::Gemfile,
            Self::GemfileLock,
            Self::PomXml,
            Self::BuildGradle,
            Self::GoMod,
            Self::CargoToml,
            Self::CargoLock,
            Self::NugetPackagesConfig,
            Self::Csproj,
        ]
    }
}

impl std::fmt::Display for FileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A package name + optional version constraint extracted from a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRef {
    pub name: String,
    pub version: String,
}

impl PackageRef {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Result of one registry existence check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Package exists on the public registry.
    Safe,
    /// Package was not found (dependency confusion candidate).
    Vulnerable,
    /// Network / non-404 failure — do not treat as safe or vulnerable.
    Error,
}

/// Per-package scan outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageResult {
    pub name: String,
    pub version: String,
    pub status: CheckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Full scan summary for one dependency file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub file: String,
    pub file_kind: FileKind,
    pub ecosystem: Ecosystem,
    pub packages: BTreeMap<String, String>,
    pub vulnerable: Vec<PackageResult>,
    pub safe: Vec<PackageResult>,
    pub errors: Vec<PackageResult>,
    /// Missing on public registry but matched `--secure-namespace` / `-s` (confused-compatible).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressed: Vec<PackageResult>,
    /// Loki inspector: git commits that introduced vulnerable package names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub introductions: Vec<crate::depcheck::inspect::IntroductionCommit>,
    /// Loki-style npm hardening recon findings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardening: Option<crate::depcheck::hardening::HardeningReport>,
    pub duration_secs: f64,
    pub tool_version: String,
}

impl ScanSummary {
    pub fn total(&self) -> usize {
        self.packages.len()
    }
}

/// Options for a depcheck run.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub threads: usize,
    pub timeout_secs: u64,
    pub quiet: bool,
    pub kind_override: Option<FileKind>,
    /// Known-secure namespaces (confused `-s`); supports `*` wildcards.
    pub secure_namespaces: Vec<String>,
    /// Extra per-package progress lines (confused `-v` style).
    pub verbose: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            threads: 20,
            timeout_secs: 10,
            quiet: false,
            kind_override: None,
            secure_namespaces: Vec::new(),
            verbose: false,
        }
    }
}

/// Input target after resolution (path content or fetched URL body).
#[derive(Debug, Clone)]
pub struct ManifestInput {
    pub display: String,
    pub path: Option<PathBuf>,
    pub content: String,
    pub kind: FileKind,
}
