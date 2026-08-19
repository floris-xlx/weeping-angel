//! Versioned framework-pack loader. Network-free. Provider-independent.
//!
//! Packs are projections onto canonical controls. Catalog TOML is never parsed
//! here; callers supply an IR-shaped [`CatalogProjection`] or named load uses
//! the workspace adapter registered by the catalog crate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;
use weeping_angel_assurance_ir::{
    CatalogProjection, Control, ControlId, EvidenceRequirement, EvidenceRequirementId, FrameworkId,
    FrameworkVersion, Mapping, MappingCompleteness, MappingDirection, MappingProvenance,
    MappingRelation, MappingSource, MappingVersionConstraint, PlannedControlTest, Requirement,
    RequirementId, canonical_digest, workspace_catalog_projection,
};

use crate::{
    Assessment, AssessmentRequests, FrameworkCompileError, FrameworkProfile, FrameworkTarget,
};

pub const FRAMEWORK_PACK_SCHEMA: &str = "weeping-angel/framework-pack/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameworkPackDigest(pub String);

impl FrameworkPackDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameworkContentProvider {
    StructuralOnly,
    LicensedContent,
    UserSuppliedContent,
}

#[derive(Debug, Error)]
pub enum PackError {
    #[error("unknown pack: {0}")]
    UnknownPack(String),
    #[error("unknown requirement: {0}")]
    UnknownRequirement(String),
    #[error("dangling mapping {from} → {to}")]
    Dangling { from: String, to: String },
    #[error("unsupported relation: {0}")]
    UnsupportedRelation(String),
    #[error("unknown completeness: {0}")]
    UnknownCompleteness(String),
    #[error("unknown direction: {0}")]
    UnknownDirection(String),
    #[error("unknown provenance source: {0}")]
    UnknownSource(String),
    #[error("empty rationale where required for {0}")]
    EmptyRationale(String),
    #[error("competing metadata library row {id}")]
    CompetingLibrary { id: String },
    #[error("malformed expression: {0}")]
    MalformedExpression(String),
    #[error("duplicate requirement id: {0}")]
    DuplicateRequirement(String),
    #[error("duplicate mapping {from} → {to} ({relation})")]
    DuplicateMapping {
        from: String,
        to: String,
        relation: String,
    },
    #[error("digest mismatch: expected {expected}, got {actual}")]
    DigestMismatch { expected: String, actual: String },
    #[error("io: {0}")]
    Io(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("schema: {0}")]
    Schema(String),
}

impl From<PackError> for FrameworkCompileError {
    fn from(value: PackError) -> Self {
        match value {
            PackError::UnknownPack(message) => FrameworkCompileError::UnknownPack { message },
            PackError::UnknownRequirement(id) => FrameworkCompileError::UnknownRequirement { id },
            PackError::Dangling { from, to } => FrameworkCompileError::MappingIntegrity {
                message: format!("dangling mapping {from} → {to}"),
            },
            PackError::UnsupportedRelation(rel) => FrameworkCompileError::MappingIntegrity {
                message: format!("unsupported relation {rel}"),
            },
            PackError::UnknownCompleteness(value) => FrameworkCompileError::MappingIntegrity {
                message: format!("unknown completeness {value}"),
            },
            PackError::UnknownDirection(value) => FrameworkCompileError::MappingIntegrity {
                message: format!("unknown direction {value}"),
            },
            PackError::UnknownSource(value) => FrameworkCompileError::MappingIntegrity {
                message: format!("unknown provenance source {value}"),
            },
            PackError::EmptyRationale(id) => FrameworkCompileError::MappingIntegrity {
                message: format!("empty rationale where required for {id}"),
            },
            PackError::DigestMismatch { expected, actual } => {
                FrameworkCompileError::DigestMismatch { expected, actual }
            }
            other => FrameworkCompileError::Schema {
                message: other.to_string(),
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct ManifestFile {
    schema: String,
    framework: ManifestFramework,
    #[serde(default)]
    capabilities: BTreeMap<String, bool>,
    #[serde(default)]
    digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestFramework {
    id: String,
    version: String,
    #[serde(default)]
    content_mode: String,
}

#[derive(Debug, Deserialize)]
struct RequirementsFile {
    #[serde(default)]
    requirement: Vec<RequirementRow>,
}

#[derive(Debug, Deserialize)]
struct RequirementRow {
    id: String,
    title: String,
    #[serde(default)]
    kind: String,
}

#[derive(Debug, Deserialize)]
struct MappingsFile {
    #[serde(default)]
    mapping: Vec<MappingRow>,
}

#[derive(Debug, Deserialize)]
struct MappingRow {
    from: String,
    to: String,
    #[serde(default)]
    direction: String,
    #[serde(default)]
    completeness: String,
    #[serde(default)]
    relation: String,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    provenance: Option<MappingProvenanceRow>,
    #[serde(default)]
    valid_for: Option<MappingValidForRow>,
}

#[derive(Debug, Deserialize)]
struct MappingProvenanceRow {
    #[serde(default)]
    source: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    reference: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MappingValidForRow {
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataFile {
    #[serde(default)]
    control: Vec<ControlRow>,
    #[serde(default)]
    test: Vec<TestRow>,
}

#[derive(Debug, Deserialize)]
struct ControlRow {
    #[serde(default)]
    id: String,
}

#[derive(Debug, Deserialize)]
struct TestRow {
    #[serde(default)]
    id: String,
}

#[derive(Debug, Deserialize)]
struct ApplicabilityFile {
    #[serde(default)]
    entry: Vec<ApplicabilityRow>,
}

#[derive(Debug, Deserialize)]
struct ApplicabilityRow {
    #[serde(default)]
    reference: String,
    #[serde(default)]
    requirement: String,
    #[serde(default)]
    applicability: String,
    #[serde(default)]
    applicable: Option<bool>,
    #[serde(default)]
    applicability_rationale: String,
}

#[derive(Debug, Clone)]
pub struct LoadedPack {
    pub digest: FrameworkPackDigest,
    pub catalog_digest: String,
    pub profile: String,
    pub version: String,
    pub content_provider: FrameworkContentProvider,
    pub requirements: Vec<Requirement>,
    pub controls: Vec<Control>,
    pub mappings: Vec<Mapping>,
    pub tests: Vec<PlannedControlTest>,
    pub evidence_requirements: Vec<EvidenceRequirement>,
}

pub fn pack_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = PathBuf::from(dir);
        roots.push(base.join("frameworks"));
        roots.push(base.join("..").join("..").join("frameworks"));
        roots.push(base.join("..").join("frameworks"));
    }
    roots.push(PathBuf::from("frameworks"));
    roots
}

pub fn resolve_pack_dir(framework: &str, version: &str) -> Result<PathBuf, PackError> {
    for root in pack_search_roots() {
        let candidate = root.join(framework).join(version);
        if candidate.join("manifest.toml").is_file() {
            return Ok(candidate);
        }
    }
    Err(PackError::UnknownPack(format!(
        "no pack for {framework}/{version}"
    )))
}

pub fn load_framework_pack(framework: &str, version: &str) -> Result<LoadedPack, PackError> {
    let dir = resolve_pack_dir(framework, version)?;
    let catalog = workspace_catalog_projection();
    load_framework_pack_from_with(&dir, catalog.as_ref())
}

pub fn load_framework_pack_from(dir: &Path) -> Result<LoadedPack, PackError> {
    load_framework_pack_from_with(dir, None)
}

pub fn load_framework_pack_from_with(
    dir: &Path,
    catalog: Option<&CatalogProjection>,
) -> Result<LoadedPack, PackError> {
    let manifest_text = read(dir.join("manifest.toml"))?;
    let manifest: ManifestFile = toml::from_str(&manifest_text).map_err(parse_err)?;
    if manifest.schema != FRAMEWORK_PACK_SCHEMA {
        return Err(PackError::Schema(format!(
            "expected {FRAMEWORK_PACK_SCHEMA}, got {}",
            manifest.schema
        )));
    }

    let reqs_file: RequirementsFile =
        toml::from_str(&read(dir.join("requirements.toml"))?).map_err(parse_err)?;
    let maps_file: MappingsFile =
        toml::from_str(&read(dir.join("mappings.toml"))?).map_err(parse_err)?;
    let meta_path = dir.join("metadata.toml");
    let meta: MetadataFile = if meta_path.is_file() {
        toml::from_str(&read(meta_path)?).map_err(parse_err)?
    } else {
        MetadataFile {
            control: Vec::new(),
            test: Vec::new(),
        }
    };
    if let Some(row) = meta.control.first() {
        return Err(PackError::CompetingLibrary { id: row.id.clone() });
    }
    if let Some(row) = meta.test.first() {
        return Err(PackError::CompetingLibrary { id: row.id.clone() });
    }

    let applicability = load_applicability(dir)?;
    let content_provider = parse_content_mode(&manifest.framework.content_mode)?;

    let framework_id = FrameworkId::new(&manifest.framework.id);
    let framework_version = FrameworkVersion::new(&manifest.framework.version);

    let mut seen_requirements = BTreeSet::new();
    let mut requirements = Vec::new();
    for row in reqs_file.requirement {
        if !seen_requirements.insert(row.id.clone()) {
            return Err(PackError::DuplicateRequirement(row.id));
        }
        requirements.push(Requirement::new(
            RequirementId::new(&row.id),
            framework_id.clone(),
            framework_version.clone(),
            row.title,
            row.kind,
        ));
    }

    let mut mappings = Vec::new();
    let mut seen_mappings = BTreeSet::new();
    let mut used_control_ids = BTreeSet::new();
    for row in maps_file.mapping {
        let completeness = parse_completeness(&row.completeness)?;
        let relation = match row.relation.as_str() {
            "Equivalent" => MappingRelation::Equivalent,
            "Satisfies" => MappingRelation::Satisfies,
            "PartiallySatisfies" => MappingRelation::PartiallySatisfies,
            "Supports" => MappingRelation::Supports,
            "Related" => MappingRelation::Related,
            "EvidenceFor" => MappingRelation::EvidenceFor,
            "SupersetOf" => MappingRelation::SupersetOf,
            "SubsetOf" => MappingRelation::SubsetOf,
            "" => MappingRelation::from_completeness(completeness),
            other => return Err(PackError::UnsupportedRelation(other.into())),
        };
        let direction = match row.direction.as_str() {
            "forward" => MappingDirection::Forward,
            "reverse" => MappingDirection::Reverse,
            "bidirectional" => MappingDirection::Bidirectional,
            other => {
                return Err(PackError::UnknownDirection(if other.is_empty() {
                    "empty".into()
                } else {
                    other.into()
                }));
            }
        };
        if let Some(prov) = &row.provenance {
            let _ = parse_mapping_source(&prov.source)?;
        }
        if !requirements.iter().any(|r| r.id().as_str() == row.from) {
            return Err(PackError::Dangling {
                from: row.from,
                to: row.to,
            });
        }
        let in_catalog = catalog.and_then(|c| c.control(&row.to)).is_some();
        if !row.to.starts_with("control.") || !in_catalog {
            return Err(PackError::Dangling {
                from: row.from,
                to: row.to,
            });
        }
        if row.rationale.trim().is_empty() && row.completeness != "full" {
            return Err(PackError::EmptyRationale(format!(
                "{}→{}",
                row.from, row.to
            )));
        }
        let key = (row.from.clone(), row.to.clone(), format!("{relation:?}"));
        if !seen_mappings.insert(key) {
            return Err(PackError::DuplicateMapping {
                from: row.from,
                to: row.to,
                relation: format!("{relation:?}"),
            });
        }
        let mut mapping = Mapping::new(
            RequirementId::new(&row.from),
            ControlId::new(&row.to),
            direction,
            completeness,
        )
        .with_relation(relation)
        .with_rationale(row.rationale);
        if let Some(prov) = row.provenance {
            mapping = mapping.with_provenance(MappingProvenance {
                source: parse_mapping_source(&prov.source)?,
                author: prov.author,
                reference: prov.reference,
                reviewed_at: None,
            });
        }
        if let Some(constraint) = row.valid_for {
            mapping = mapping.with_valid_for(MappingVersionConstraint {
                from: constraint.from.map(FrameworkVersion::new),
                to: constraint.to.map(FrameworkVersion::new),
            });
        }
        used_control_ids.insert(row.to);
        mappings.push(mapping);
    }

    let mut controls = Vec::new();
    let mut tests = Vec::new();
    let mut evidence_requirements = Vec::new();
    if let Some(index) = catalog {
        for id in &used_control_ids {
            if let Some(control) = index.control(id) {
                if !controls.iter().any(|c: &Control| c.id() == control.id()) {
                    controls.push(control.clone());
                }
            }
            for planned in index.tests_for(id) {
                for ty in &planned.required_evidence {
                    evidence_requirements.push(EvidenceRequirement::new(
                        EvidenceRequirementId::new(format!("ev.{}", ty.as_str())),
                        ty.clone(),
                    ));
                }
                if !tests
                    .iter()
                    .any(|t: &PlannedControlTest| t.id == planned.id)
                {
                    tests.push(planned.clone());
                }
            }
        }
    }

    evidence_requirements.sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
    evidence_requirements.dedup_by(|a, b| a.id() == b.id());

    let digest_body = serde_json::json!({
        "schema": FRAMEWORK_PACK_SCHEMA,
        "framework": manifest.framework.id,
        "version": manifest.framework.version,
        "contentMode": format!("{content_provider:?}"),
        "capabilities": manifest.capabilities,
        "requirements": requirements.iter().map(|r| {
            serde_json::json!({
                "id": r.id().as_str(),
                "title": r.title(),
                "kind": r.description(),
            })
        }).collect::<Vec<_>>(),
        "mappings": mappings.iter().map(|m| {
            serde_json::json!({
                "from": m.from_requirement().as_str(),
                "to": m.to_control().as_str(),
                "relation": format!("{:?}", m.relation()),
                "completeness": format!("{:?}", m.completeness()),
                "direction": format!("{:?}", m.direction()),
                "rationale": m.rationale(),
                "provenance": {
                    "source": format!("{:?}", m.provenance().source),
                    "author": m.provenance().author.clone(),
                    "reference": m.provenance().reference.clone(),
                },
                "validFor": {
                    "from": m.valid_for().from.as_ref().map(|v| v.as_str().to_string()),
                    "to": m.valid_for().to.as_ref().map(|v| v.as_str().to_string()),
                },
            })
        }).collect::<Vec<_>>(),
        "applicability": applicability.iter().map(|e| {
            serde_json::json!({
                "reference": e.reference,
                "requirement": e.requirement,
                "applicability": e.applicability,
                "applicable": e.applicable,
                "rationale": e.applicability_rationale,
            })
        }).collect::<Vec<_>>(),
    });
    let digest = FrameworkPackDigest(
        canonical_digest(&digest_body).map_err(|e| PackError::Parse(e.to_string()))?,
    );
    if let Some(declared) = manifest.digest.as_deref()
        && !declared.is_empty()
        && declared != digest.as_str()
    {
        return Err(PackError::DigestMismatch {
            expected: declared.to_string(),
            actual: digest.0.clone(),
        });
    }

    Ok(LoadedPack {
        digest,
        catalog_digest: catalog.map(|c| c.digest.clone()).unwrap_or_default(),
        profile: manifest.framework.id,
        version: manifest.framework.version,
        content_provider,
        requirements,
        controls,
        mappings,
        tests,
        evidence_requirements,
    })
}

pub fn validate_framework_pack(dir: &Path) -> Result<LoadedPack, PackError> {
    let catalog = workspace_catalog_projection();
    let pack = load_framework_pack_from_with(dir, catalog.as_ref())?;
    if pack.requirements.is_empty() {
        return Err(PackError::Schema("pack has no requirements".into()));
    }
    Ok(pack)
}

pub fn validate_framework_pack_with(
    dir: &Path,
    catalog: Option<&CatalogProjection>,
) -> Result<LoadedPack, PackError> {
    let pack = load_framework_pack_from_with(dir, catalog)?;
    if pack.requirements.is_empty() {
        return Err(PackError::Schema("pack has no requirements".into()));
    }
    Ok(pack)
}

pub fn assessment_from_pack(pack: &LoadedPack, target: &FrameworkTarget) -> Assessment {
    let _ = target;
    let mut assessment = Assessment::new(weeping_angel_assurance_ir::AssessmentId::new(format!(
        "assess-{}-{}",
        pack.profile, pack.version
    )));
    assessment.requirements = pack.requirements.clone();
    assessment.controls = pack.controls.clone();
    assessment.mappings = pack.mappings.clone();
    assessment.evidence_requirements = pack.evidence_requirements.clone();
    assessment.tests = pack.tests.clone();
    assessment.requests = AssessmentRequests::default();
    assessment
}

pub fn merge_pack(assessment: &mut Assessment, pack: &LoadedPack) {
    for req in &pack.requirements {
        if !assessment.requirements.iter().any(|r| r.id() == req.id()) {
            assessment.requirements.push(req.clone());
        }
    }
    for ctl in &pack.controls {
        if !assessment.controls.iter().any(|c| c.id() == ctl.id()) {
            assessment.controls.push(ctl.clone());
        }
    }
    for mapping in &pack.mappings {
        let exists = assessment.mappings.iter().any(|m| {
            m.from_requirement() == mapping.from_requirement()
                && m.to_control() == mapping.to_control()
        });
        if !exists {
            assessment.mappings.push(mapping.clone());
        }
    }
    for ev in &pack.evidence_requirements {
        if !assessment
            .evidence_requirements
            .iter()
            .any(|e| e.id() == ev.id())
        {
            assessment.evidence_requirements.push(ev.clone());
        }
    }
    if assessment.tests.is_empty() {
        assessment.tests = pack.tests.clone();
    } else {
        for test in &pack.tests {
            if !assessment.tests.iter().any(|t| t.id == test.id) {
                assessment.tests.push(test.clone());
            }
        }
    }
}

pub fn profile_to_pack_id(profile: FrameworkProfile) -> &'static str {
    profile.as_selector()
}

fn load_applicability(dir: &Path) -> Result<Vec<ApplicabilityRow>, PackError> {
    let path = dir.join("applicability.toml");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let file: ApplicabilityFile = toml::from_str(&read(path)?).map_err(parse_err)?;
    Ok(file.entry)
}

fn parse_content_mode(raw: &str) -> Result<FrameworkContentProvider, PackError> {
    match raw {
        "" | "StructuralOnly" | "structural-only" | "structuralOnly" => {
            Ok(FrameworkContentProvider::StructuralOnly)
        }
        "LicensedContent" | "licensed" | "licensedContent" => {
            Ok(FrameworkContentProvider::LicensedContent)
        }
        "UserSuppliedContent" | "user-supplied" | "userSuppliedContent" => {
            Ok(FrameworkContentProvider::UserSuppliedContent)
        }
        other => Err(PackError::Schema(format!("unknown content_mode `{other}`"))),
    }
}

fn parse_completeness(raw: &str) -> Result<MappingCompleteness, PackError> {
    match raw {
        "full" => Ok(MappingCompleteness::Full),
        "partial" => Ok(MappingCompleteness::Partial),
        "related" => Ok(MappingCompleteness::Related),
        other => Err(PackError::UnknownCompleteness(if other.is_empty() {
            "empty".into()
        } else {
            other.into()
        })),
    }
}

fn parse_mapping_source(raw: &str) -> Result<MappingSource, PackError> {
    match raw {
        "BuiltIn" | "builtIn" | "builtin" => Ok(MappingSource::BuiltIn),
        "UserDefined" | "userDefined" => Ok(MappingSource::UserDefined),
        "LicensedFrameworkContent" | "licensedFrameworkContent" => {
            Ok(MappingSource::LicensedFrameworkContent)
        }
        "Imported" | "imported" => Ok(MappingSource::Imported),
        "AuditorApproved" | "auditorApproved" => Ok(MappingSource::AuditorApproved),
        "Generated" | "generated" => Ok(MappingSource::Generated),
        other => Err(PackError::UnknownSource(if other.is_empty() {
            "empty".into()
        } else {
            other.into()
        })),
    }
}

fn read(path: PathBuf) -> Result<String, PackError> {
    fs::read_to_string(&path).map_err(|e| PackError::Io(format!("{}: {e}", path.display())))
}

fn parse_err(err: toml::de::Error) -> PackError {
    PackError::Parse(err.to_string())
}
