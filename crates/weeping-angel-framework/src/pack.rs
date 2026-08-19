//! Versioned framework-pack loader. Network-free. Provider-independent.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;
use weeping_angel_assurance_ir::{
    Control, ControlId, ControlTestId, EvidenceRequirement, EvidenceRequirementId, EvidenceType,
    FrameworkId, FrameworkVersion, Mapping, MappingCompleteness, MappingDirection,
    MappingProvenance, MappingRelation, MappingSource, MappingVersionConstraint,
    PlannedControlTest, PlannedTestKind, Requirement, RequirementId, canonical_digest,
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
    #[error("empty rationale where required for {0}")]
    EmptyRationale(String),
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
            PackError::EmptyRationale(id) => FrameworkCompileError::MappingIntegrity {
                message: format!("empty rationale where required for {id}"),
            },
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
}

#[derive(Debug, Deserialize)]
struct ManifestFramework {
    id: String,
    version: String,
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
    id: String,
    title: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct TestRow {
    id: String,
    control: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    required: Vec<String>,
    #[serde(default)]
    break_on: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedPack {
    pub digest: FrameworkPackDigest,
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
    load_framework_pack_from(&dir)
}

pub fn load_framework_pack_from(dir: &Path) -> Result<LoadedPack, PackError> {
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

    let framework_id = FrameworkId::new(&manifest.framework.id);
    let framework_version = FrameworkVersion::new(&manifest.framework.version);

    let mut requirements = Vec::new();
    for row in reqs_file.requirement {
        requirements.push(Requirement::new(
            RequirementId::new(&row.id),
            framework_id.clone(),
            framework_version.clone(),
            row.title,
            row.kind,
        ));
    }

    let mut controls = Vec::new();
    for row in &meta.control {
        if row.id.starts_with("control.") {
            controls.push(Control::new(
                ControlId::new(&row.id),
                row.title.clone(),
                row.description.clone(),
            ));
        }
    }

    let catalog = discover_catalog_index();

    let mut mappings = Vec::new();
    for row in maps_file.mapping {
        if !requirements.iter().any(|r| r.id().as_str() == row.from) {
            return Err(PackError::Dangling {
                from: row.from,
                to: row.to,
            });
        }
        let in_pack = controls.iter().any(|c| c.id().as_str() == row.to);
        let in_catalog = catalog.as_ref().is_some_and(|c| c.has_control(&row.to));
        if !row.to.starts_with("control.") || !(in_pack || in_catalog) {
            return Err(PackError::Dangling {
                from: row.from,
                to: row.to,
            });
        }
        if let Some(index) = catalog.as_ref()
            && let Some(control) = index.control(&row.to)
            && !controls.iter().any(|c| c.id().as_str() == row.to)
        {
            controls.push(Control::new(
                ControlId::new(&control.id),
                control.title.clone(),
                control.description.clone(),
            ));
        }
        if row.rationale.trim().is_empty() && row.completeness != "full" {
            return Err(PackError::EmptyRationale(format!(
                "{}→{}",
                row.from, row.to
            )));
        }
        let completeness = match row.completeness.as_str() {
            "full" => MappingCompleteness::Full,
            "related" => MappingCompleteness::Related,
            _ => MappingCompleteness::Partial,
        };
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
            "reverse" => MappingDirection::Reverse,
            "bidirectional" => MappingDirection::Bidirectional,
            _ => MappingDirection::Forward,
        };
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
                source: parse_mapping_source(&prov.source),
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
        mappings.push(mapping);
    }

    let mut tests = Vec::new();
    let mut evidence_requirements = Vec::new();
    if let Some(index) = catalog.as_ref() {
        for control in &controls {
            for planned in index.tests_for(control.id().as_str()) {
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
                    tests.push(planned);
                }
            }
        }
    }
    for row in meta.test {
        if !row.control.starts_with("control.") {
            continue;
        }
        let kind = if row.kind.eq_ignore_ascii_case("manual") {
            PlannedTestKind::Manual
        } else {
            PlannedTestKind::Automated
        };
        let mut planned =
            PlannedControlTest::new(ControlTestId::new(&row.id), ControlId::new(&row.control));
        planned.kind = kind;
        planned.required_evidence = row
            .required
            .iter()
            .map(|t| EvidenceType::new(t.as_str()))
            .collect();
        planned.break_on = row
            .break_on
            .iter()
            .map(|t| EvidenceType::new(t.as_str()))
            .collect();
        for ty in &planned.required_evidence {
            evidence_requirements.push(EvidenceRequirement::new(
                EvidenceRequirementId::new(format!("ev.{}", ty.as_str())),
                ty.clone(),
            ));
        }
        tests.push(planned);
    }

    evidence_requirements.sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
    evidence_requirements.dedup_by(|a, b| a.id() == b.id());

    let digest_body = serde_json::json!({
        "schema": FRAMEWORK_PACK_SCHEMA,
        "framework": manifest.framework.id,
        "version": manifest.framework.version,
        "requirements": requirements.iter().map(|r| r.id().as_str()).collect::<Vec<_>>(),
        "controls": controls.iter().map(|c| c.id().as_str()).collect::<Vec<_>>(),
        "mappings": mappings.iter().map(|m| {
            (m.from_requirement().as_str(), m.to_control().as_str(), format!("{:?}", m.relation()))
        }).collect::<Vec<_>>(),
        "tests": tests.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
    });
    let digest = FrameworkPackDigest(
        canonical_digest(&digest_body).map_err(|e| PackError::Parse(e.to_string()))?,
    );

    Ok(LoadedPack {
        digest,
        profile: manifest.framework.id,
        version: manifest.framework.version,
        content_provider: FrameworkContentProvider::StructuralOnly,
        requirements,
        controls,
        mappings,
        tests,
        evidence_requirements,
    })
}

pub fn validate_framework_pack(dir: &Path) -> Result<LoadedPack, PackError> {
    let pack = load_framework_pack_from(dir)?;
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

fn read(path: PathBuf) -> Result<String, PackError> {
    fs::read_to_string(&path).map_err(|e| PackError::Io(format!("{}: {e}", path.display())))
}

fn parse_err(err: toml::de::Error) -> PackError {
    PackError::Parse(err.to_string())
}

fn parse_mapping_source(raw: &str) -> MappingSource {
    match raw {
        "UserDefined" | "userDefined" => MappingSource::UserDefined,
        "LicensedFrameworkContent" | "licensedFrameworkContent" => {
            MappingSource::LicensedFrameworkContent
        }
        "Imported" | "imported" => MappingSource::Imported,
        "AuditorApproved" | "auditorApproved" => MappingSource::AuditorApproved,
        "Generated" | "generated" => MappingSource::Generated,
        _ => MappingSource::BuiltIn,
    }
}

fn catalog_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let base = PathBuf::from(dir);
        roots.push(base.join("catalog/canonical/v1"));
        roots.push(base.join("..").join("catalog/canonical/v1"));
        roots.push(base.join("..").join("..").join("catalog/canonical/v1"));
    }
    roots.push(PathBuf::from("catalog/canonical/v1"));
    roots
}

#[derive(Debug, Default)]
struct CatalogIndex {
    controls: std::collections::BTreeMap<String, IndexedControl>,
    tests: Vec<IndexedTest>,
}

#[derive(Debug, Clone)]
struct IndexedControl {
    id: String,
    title: String,
    description: String,
}

#[derive(Debug, Clone)]
struct IndexedTest {
    id: String,
    control: String,
    kind: String,
    required: Vec<String>,
}

impl CatalogIndex {
    fn has_control(&self, id: &str) -> bool {
        self.controls.contains_key(id)
    }

    fn control(&self, id: &str) -> Option<&IndexedControl> {
        self.controls.get(id)
    }

    fn tests_for(&self, control_id: &str) -> Vec<PlannedControlTest> {
        self.tests
            .iter()
            .filter(|t| t.control == control_id)
            .map(|t| {
                let kind = if t.kind.eq_ignore_ascii_case("manual") {
                    PlannedTestKind::Manual
                } else if t.kind.eq_ignore_ascii_case("hybrid") {
                    PlannedTestKind::Hybrid
                } else {
                    PlannedTestKind::Automated
                };
                let mut planned =
                    PlannedControlTest::new(ControlTestId::new(&t.id), ControlId::new(&t.control));
                planned.kind = kind;
                planned.required_evidence = t
                    .required
                    .iter()
                    .map(|ty| EvidenceType::new(ty.as_str()))
                    .collect();
                planned
            })
            .collect()
    }
}

fn discover_catalog_index() -> Option<CatalogIndex> {
    let root = catalog_search_roots()
        .into_iter()
        .find(|p| p.join("manifest.toml").is_file())?;
    let manifest: toml::Value =
        toml::from_str(&fs::read_to_string(root.join("manifest.toml")).ok()?).ok()?;
    let files = manifest.get("files")?;
    let mut index = CatalogIndex::default();
    if let Some(controls) = files.get("controls").and_then(|v| v.as_array()) {
        for entry in controls {
            let Some(rel) = entry.as_str() else { continue };
            let path = root.join(rel);
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(parsed) = toml::from_str::<toml::Value>(&text) else {
                continue;
            };
            if let Some(rows) = parsed.get("control").and_then(|v| v.as_array()) {
                for row in rows {
                    let Some(id) = row.get("id").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    index.controls.insert(
                        id.to_string(),
                        IndexedControl {
                            id: id.to_string(),
                            title: row
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or(id)
                                .to_string(),
                            description: row
                                .get("description")
                                .or_else(|| row.get("narrative"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        },
                    );
                }
            }
        }
    }
    if let Some(tests) = files.get("tests").and_then(|v| v.as_array()) {
        for entry in tests {
            let Some(rel) = entry.as_str() else { continue };
            let path = root.join(rel);
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(parsed) = toml::from_str::<toml::Value>(&text) else {
                continue;
            };
            if let Some(rows) = parsed.get("test").and_then(|v| v.as_array()) {
                for row in rows {
                    let Some(id) = row.get("id").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(control) = row.get("control").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let required = row
                        .get("required_evidence")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                                .collect()
                        })
                        .unwrap_or_default();
                    index.tests.push(IndexedTest {
                        id: id.to_string(),
                        control: control.to_string(),
                        kind: row
                            .get("kind")
                            .and_then(|v| v.as_str())
                            .unwrap_or("automated")
                            .to_string(),
                        required,
                    });
                }
            }
        }
    }
    Some(index)
}
