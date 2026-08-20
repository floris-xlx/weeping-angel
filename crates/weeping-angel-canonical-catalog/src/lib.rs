//! Offline canonical catalog loader, validator, and digest.
//!
//! Zero network I/O. Downstream domain files are listed in the manifest;
//! this crate does not hard-code fixture names.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    CatalogProjection, Control, ControlId, ControlTestId, EvidenceType, PlannedControlTest,
    PlannedTestKind, WorkspaceCatalogLoader, canonical_digest,
};

pub const CATALOG_SCHEMA: &str = "weeping-angel/canonical-catalog/v1";
pub const DIGEST_PREFIX: &str = "wa:canonical-catalog:weeping-angel/canonical-catalog/v1:";

const PROVIDER_SEGMENTS: &[&str] = &[
    "github",
    "gitlab",
    "bitbucket",
    "aws",
    "azure",
    "azure-ad",
    "gcp",
    "google",
    "google-workspace",
    "cloudflare",
    "vercel",
    "okta",
    "entra",
    "auth0",
    "workspace",
    "cognito",
];

const FRAMEWORK_SEGMENTS: &[&str] = &[
    "iso27001",
    "iso27701",
    "iso27007",
    "soc2",
    "nis2",
    "dora",
    "gdpr",
    "iso-27001",
    "iso-27701",
    "iso-27007",
    "soc-2",
    "nis-2",
];

const ALLOWED_OPS: &[&str] = &[
    "exists",
    "missing",
    "eq",
    "neq",
    "gt",
    "gte",
    "lt",
    "lte",
    "contains",
    "not-contains",
    "in",
    "count",
    "count-where",
    "fresh-within",
    "coverage-at-least",
    "coverage_at_least",
    "CoverageAtLeast",
    "coverage-exactly",
    "all-subjects",
    "all_subjects",
    "AllSubjects",
    "any-subject",
    "none-subjects",
    "none_subjects",
    "missing-subjects",
    "manual-review",
    "manual_review",
    "all",
    "any",
    "not",
];

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("unsupported catalog schema: {0}")]
    UnsupportedSchema(String),
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("toml parse error at {path}: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("duplicate {kind} id: {id}")]
    Duplicate { kind: String, id: String },
    #[error("dangling {kind} reference {id} from {from}")]
    Dangling {
        kind: String,
        id: String,
        from: String,
    },
    #[error("orphaned test {id} is not listed on its control")]
    Orphaned { id: String },
    #[error("reserved {class} segment `{segment}` in id {id}")]
    Reserved {
        class: String,
        segment: String,
        id: String,
    },
    #[error("malformed catalog id (namespace/invalid): {0}")]
    MalformedId(String),
    #[error("unknown expression operator `{op}` on {id}")]
    UnknownOperator { op: String, id: String },
    #[error("malformed expression on {id}: {reason}")]
    MalformedExpression { id: String, reason: String },
    #[error("unlisted extra section file not listed in the manifest: {0}")]
    Unlisted(String),
    #[error("listed path escapes catalog root: {0}")]
    PathEscape(String),
    #[error("listed file is missing: {0}")]
    MissingFile(String),
    #[error("unknown subject kind `{kind}` on {id}")]
    UnknownKind { kind: String, id: String },
    #[error("unknown control {0}")]
    UnknownControl(String),
    #[error("digest failed: {0}")]
    Digest(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogDigest(pub String);

impl fmt::Display for CatalogDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct CatalogStats {
    pub schema: String,
    pub catalog_id: String,
    pub catalog_version: String,
    pub control_count: usize,
    pub evidence_count: usize,
    pub test_count: usize,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogControl {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objective: String,
    pub domains: Vec<String>,
    pub evidence: Vec<String>,
    pub tests: Vec<String>,
    pub automation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogEvidence {
    pub id: String,
    pub title: String,
    pub evidence_type: String,
    pub collection: String,
    pub criticality: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogTest {
    pub id: String,
    pub control: String,
    pub kind: String,
    pub required_evidence: Vec<String>,
    pub break_on: Vec<String>,
    pub expression: BTreeMap<String, toml::Value>,
    pub subjects: Vec<BTreeMap<String, toml::Value>>,
}

#[derive(Debug, Clone)]
pub struct CanonicalCatalog {
    root: PathBuf,
    catalog_id: String,
    catalog_version: String,
    controls: BTreeMap<String, CatalogControl>,
    evidence: BTreeMap<String, CatalogEvidence>,
    tests: BTreeMap<String, CatalogTest>,
}

impl CanonicalCatalog {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let root = path.as_ref().to_path_buf();
        let manifest_path = root.join("manifest.toml");
        let manifest_raw = read_to_string(&manifest_path)?;
        let manifest: ManifestFile = parse_toml(&manifest_path, &manifest_raw)?;
        if manifest.schema != CATALOG_SCHEMA {
            return Err(CatalogError::UnsupportedSchema(manifest.schema));
        }

        let files = manifest.files.unwrap_or_default();
        let control_files = listed_paths(&root, &files.controls)?;
        let evidence_files = listed_paths(&root, &files.evidence)?;
        let test_files = listed_paths(&root, &files.tests)?;

        reject_unlisted(&root.join("controls"), &control_files)?;
        reject_unlisted(&root.join("evidence"), &evidence_files)?;
        reject_unlisted(&root.join("tests"), &test_files)?;

        let mut controls = BTreeMap::new();
        for path in &control_files {
            let raw = read_to_string(path)?;
            let file: ControlFile = parse_toml(path, &raw)?;
            if !file.schema.is_empty() && file.schema != CATALOG_SCHEMA {
                return Err(CatalogError::UnsupportedSchema(file.schema));
            }
            for item in file.control {
                let id = item.id.clone();
                if controls.insert(id.clone(), item.into_catalog()).is_some() {
                    return Err(CatalogError::Duplicate {
                        kind: "control".into(),
                        id,
                    });
                }
            }
        }

        let mut evidence = BTreeMap::new();
        for path in &evidence_files {
            let raw = read_to_string(path)?;
            let file: EvidenceFile = parse_toml(path, &raw)?;
            if !file.schema.is_empty() && file.schema != CATALOG_SCHEMA {
                return Err(CatalogError::UnsupportedSchema(file.schema));
            }
            for item in file.evidence {
                let id = item.id.clone();
                if evidence.insert(id.clone(), item.into_catalog()).is_some() {
                    return Err(CatalogError::Duplicate {
                        kind: "evidence".into(),
                        id,
                    });
                }
            }
        }

        let mut tests = BTreeMap::new();
        for path in &test_files {
            let raw = read_to_string(path)?;
            let file: TestFile = parse_toml(path, &raw)?;
            if !file.schema.is_empty() && file.schema != CATALOG_SCHEMA {
                return Err(CatalogError::UnsupportedSchema(file.schema));
            }
            for item in file.test {
                let id = item.id.clone();
                if tests.insert(id.clone(), item.into_catalog()).is_some() {
                    return Err(CatalogError::Duplicate {
                        kind: "test".into(),
                        id,
                    });
                }
            }
        }

        let catalog = Self {
            root,
            catalog_id: manifest.catalog.id,
            catalog_version: manifest.catalog.version,
            controls,
            evidence,
            tests,
        };
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn validate(&self) -> Result<(), CatalogError> {
        for id in self.controls.keys() {
            validate_id("control", id)?;
        }
        for id in self.evidence.keys() {
            validate_id("evidence", id)?;
        }
        for id in self.tests.keys() {
            validate_id("test", id)?;
        }

        for control in self.controls.values() {
            for ev in &control.evidence {
                if !self.evidence.contains_key(ev) {
                    return Err(CatalogError::Dangling {
                        kind: "evidence".into(),
                        id: ev.clone(),
                        from: control.id.clone(),
                    });
                }
            }
            for test_id in &control.tests {
                if !self.tests.contains_key(test_id) {
                    return Err(CatalogError::Dangling {
                        kind: "test".into(),
                        id: test_id.clone(),
                        from: control.id.clone(),
                    });
                }
            }
        }

        for test in self.tests.values() {
            if !self.controls.contains_key(&test.control) {
                return Err(CatalogError::Dangling {
                    kind: "control".into(),
                    id: test.control.clone(),
                    from: test.id.clone(),
                });
            }
            let listed = self
                .controls
                .get(&test.control)
                .map(|c| c.tests.contains(&test.id))
                .unwrap_or(false);
            if !listed {
                return Err(CatalogError::Orphaned {
                    id: test.id.clone(),
                });
            }
            for ev in &test.required_evidence {
                if !self.evidence.contains_key(ev) {
                    return Err(CatalogError::Dangling {
                        kind: "evidence".into(),
                        id: ev.clone(),
                        from: test.id.clone(),
                    });
                }
            }
            validate_expression(test)?;
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<CatalogDigest, CatalogError> {
        let body = DigestBody {
            schema: CATALOG_SCHEMA,
            catalog_id: &self.catalog_id,
            catalog_version: &self.catalog_version,
            controls: self.controls.values().collect(),
            evidence: self.evidence.values().collect(),
            tests: self.tests.values().collect(),
        };
        let hex = canonical_digest(&body).map_err(|e| CatalogError::Digest(e.to_string()))?;
        Ok(CatalogDigest(format!("{DIGEST_PREFIX}{hex}")))
    }

    pub fn stats(&self) -> Result<CatalogStats, CatalogError> {
        Ok(CatalogStats {
            schema: CATALOG_SCHEMA.into(),
            catalog_id: self.catalog_id.clone(),
            catalog_version: self.catalog_version.clone(),
            control_count: self.controls.len(),
            evidence_count: self.evidence.len(),
            test_count: self.tests.len(),
            digest: self.digest()?.to_string(),
        })
    }

    pub fn control(&self, id: &str) -> Result<&CatalogControl, CatalogError> {
        self.controls
            .get(id)
            .ok_or_else(|| CatalogError::UnknownControl(id.to_string()))
    }

    pub fn evidence(&self) -> &BTreeMap<String, CatalogEvidence> {
        &self.evidence
    }

    pub fn tests(&self) -> &BTreeMap<String, CatalogTest> {
        &self.tests
    }

    pub fn controls(&self) -> &BTreeMap<String, CatalogControl> {
        &self.controls
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// IR-shaped projection for pack compile. Not a second authoring SSOT.
    pub fn projection(&self) -> Result<CatalogProjection, CatalogError> {
        let mut controls = Vec::new();
        for control in self.controls.values() {
            controls.push(
                Control::new(
                    ControlId::new(&control.id),
                    control.title.clone(),
                    control.description.clone(),
                )
                .with_objective(control.objective.clone()),
            );
        }
        let mut tests = Vec::new();
        for test in self.tests.values() {
            tests.push(planned_test_from_catalog(test)?);
        }
        Ok(CatalogProjection {
            digest: self.digest()?.to_string(),
            controls,
            tests,
        })
    }
}

fn planned_test_from_catalog(test: &CatalogTest) -> Result<PlannedControlTest, CatalogError> {
    let kind = if test.kind.eq_ignore_ascii_case("manual") {
        PlannedTestKind::Manual
    } else if test.kind.eq_ignore_ascii_case("hybrid") {
        PlannedTestKind::Hybrid
    } else {
        PlannedTestKind::Automated
    };
    let mut planned =
        PlannedControlTest::new(ControlTestId::new(&test.id), ControlId::new(&test.control));
    planned.kind = kind;
    planned.required_evidence = test
        .required_evidence
        .iter()
        .map(|ty| EvidenceType::new(ty.as_str()))
        .collect();
    planned.break_on = test
        .break_on
        .iter()
        .map(|ty| EvidenceType::new(ty.as_str()))
        .collect();
    if !test.expression.is_empty() {
        planned.expr = Some(expression_to_json(
            &test.expression,
            &test.subjects,
            &test.id,
        )?);
    }
    Ok(planned)
}

fn expression_to_json(
    expression: &BTreeMap<String, toml::Value>,
    subjects: &[BTreeMap<String, toml::Value>],
    test_id: &str,
) -> Result<serde_json::Value, CatalogError> {
    let op = expression
        .get("op")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CatalogError::MalformedExpression {
            id: test_id.into(),
            reason: "missing op".into(),
        })?;
    match op {
        "all" => Ok(serde_json::json!({
            "All": child_exprs(expression, subjects, test_id)?
        })),
        "any" => Ok(serde_json::json!({
            "Any": child_exprs(expression, subjects, test_id)?
        })),
        "none" => Ok(serde_json::json!({
            "None": child_exprs(expression, subjects, test_id)?
        })),
        "not" => {
            let children = child_exprs(expression, subjects, test_id)?;
            let inner =
                children
                    .into_iter()
                    .next()
                    .ok_or_else(|| CatalogError::MalformedExpression {
                        id: test_id.into(),
                        reason: "not requires a child expression".into(),
                    })?;
            Ok(serde_json::json!({ "Not": inner }))
        }
        "manual-review" | "manual_review" | "ManualReview" => {
            Ok(serde_json::Value::String("ManualReview".into()))
        }
        other => population_or_leaf_json(other, expression, subjects, test_id),
    }
}

fn child_exprs(
    expression: &BTreeMap<String, toml::Value>,
    subjects: &[BTreeMap<String, toml::Value>],
    test_id: &str,
) -> Result<Vec<serde_json::Value>, CatalogError> {
    let mut out = Vec::new();
    for key in ["children", "of", "args", "expressions"] {
        if let Some(arr) = expression.get(key).and_then(|v| v.as_array()) {
            for item in arr {
                let table: BTreeMap<String, toml::Value> = item
                    .as_table()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                out.push(expression_to_json(&table, subjects, test_id)?);
            }
        }
    }
    if let Some(child) = expression.get("child").and_then(|v| v.as_table()) {
        let table: BTreeMap<String, toml::Value> = child.clone().into_iter().collect();
        out.push(expression_to_json(&table, subjects, test_id)?);
    }
    Ok(out)
}

fn population_or_leaf_json(
    op: &str,
    expression: &BTreeMap<String, toml::Value>,
    subjects: &[BTreeMap<String, toml::Value>],
    test_id: &str,
) -> Result<serde_json::Value, CatalogError> {
    let evidence = expression
        .get("evidence")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let field = expression
        .get("field")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    let kind = subjects
        .first()
        .and_then(|row| row.get("kind"))
        .and_then(|v| v.as_str())
        .unwrap_or("organization");
    let selector = serde_json::json!({
        "kind": kind,
        "id": serde_json::Value::Null,
    });
    let evidence_sel = serde_json::json!({
        "evidenceType": evidence,
        "subjectSelector": selector.clone(),
        "field": field,
        "freshness": serde_json::Value::Null,
    });
    let percentage = expression
        .get("percentage")
        .and_then(|v| {
            v.as_str()
                .map(ToOwned::to_owned)
                .or_else(|| v.as_integer().map(|n| n.to_string()))
        })
        .unwrap_or_else(|| "100".into());
    match op {
        "exists" => Ok(serde_json::json!({ "Exists": evidence_sel })),
        "missing" => Ok(serde_json::json!({ "Missing": evidence_sel })),
        "coverage-at-least" | "coverage_at_least" | "CoverageAtLeast" => Ok(serde_json::json!({
            "CoverageAtLeast": {
                "selector": selector,
                "evidence": evidence_sel,
                "percentage": percentage,
            }
        })),
        "coverage-exactly" | "coverage_exactly" | "CoverageExactly" => Ok(serde_json::json!({
            "CoverageExactly": {
                "selector": selector,
                "evidence": evidence_sel,
                "percentage": percentage,
            }
        })),
        "all-subjects" | "all_subjects" | "AllSubjects" => Ok(serde_json::json!({
            "AllSubjects": {
                "selector": selector,
                "evidence": evidence_sel,
            }
        })),
        "any-subject" | "any_subject" | "AnySubject" => Ok(serde_json::json!({
            "AnySubject": {
                "selector": selector,
                "evidence": evidence_sel,
            }
        })),
        "none-subjects" | "none_subjects" | "NoneSubjects" => Ok(serde_json::json!({
            "NoneSubjects": {
                "selector": selector,
                "evidence": evidence_sel,
            }
        })),
        "missing-subjects" | "missing_subjects" | "MissingSubjects" => Ok(serde_json::json!({
            "MissingSubjects": {
                "selector": selector,
                "evidence": evidence_sel,
            }
        })),
        "count" => Ok(serde_json::json!({
            "Count": {
                "selector": evidence_sel,
                "predicate": { "Gte": 1 },
            }
        })),
        "count-where" | "count_where" | "CountWhere" => Ok(serde_json::json!({
            "CountWhere": {
                "selector": selector,
                "evidence": evidence_sel,
                "predicate": { "Gte": 1 },
            }
        })),
        "fresh-within" | "fresh_within" | "FreshWithin" => Ok(serde_json::json!({
            "FreshWithin": {
                "selector": evidence_sel,
                "duration": { "secs": 0, "nanos": 0 },
            }
        })),
        other => Err(CatalogError::UnknownOperator {
            op: other.into(),
            id: test_id.into(),
        }),
    }
}

fn load_workspace_catalog() -> Option<CatalogProjection> {
    for root in canonical_catalog_search_roots() {
        if root.join("manifest.toml").is_file()
            && let Ok(catalog) = CanonicalCatalog::load(&root)
            && let Ok(projection) = catalog.projection()
        {
            return Some(projection);
        }
    }
    None
}

/// Candidate roots for `catalog/canonical/v1` relative to crate/workspace layouts.
///
/// Single search-path owner for assurance pins, CLI inspect, and the
/// `WorkspaceCatalogLoader` hook (DUP-008). Callers still use
/// [`CanonicalCatalog::load`] — this only consolidates path discovery.
pub fn canonical_catalog_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut base = PathBuf::from(dir);
        for _ in 0..5 {
            roots.push(base.join("catalog/canonical/v1"));
            match base.parent() {
                Some(parent) => base = parent.to_path_buf(),
                None => break,
            }
        }
    }
    roots.push(PathBuf::from("catalog/canonical/v1"));
    roots
}

inventory::submit! {
    WorkspaceCatalogLoader(load_workspace_catalog)
}

#[derive(Serialize)]
struct DigestBody<'a> {
    schema: &'a str,
    catalog_id: &'a str,
    catalog_version: &'a str,
    controls: Vec<&'a CatalogControl>,
    evidence: Vec<&'a CatalogEvidence>,
    tests: Vec<&'a CatalogTest>,
}

#[derive(Debug, Default, Deserialize)]
struct ManifestFile {
    #[serde(default)]
    schema: String,
    #[serde(default)]
    catalog: ManifestCatalog,
    #[serde(default)]
    files: Option<ManifestFiles>,
}

#[derive(Debug, Default, Deserialize)]
struct ManifestCatalog {
    #[serde(default)]
    id: String,
    #[serde(default)]
    version: String,
}

#[derive(Debug, Default, Deserialize)]
struct ManifestFiles {
    #[serde(default)]
    controls: Vec<String>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    tests: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ControlFile {
    #[serde(default)]
    schema: String,
    #[serde(default)]
    control: Vec<ControlRow>,
}

#[derive(Debug, Default, Deserialize)]
struct ControlRow {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    narrative: String,
    #[serde(default)]
    objective: String,
    #[serde(default)]
    domains: Vec<String>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    tests: Vec<String>,
    #[serde(default)]
    automation: String,
    #[serde(default)]
    class: String,
    #[serde(default)]
    kind: String,
}

impl ControlRow {
    fn into_catalog(self) -> CatalogControl {
        let automation = first_nonempty(&[&self.automation, &self.class, &self.kind])
            .unwrap_or("automated")
            .to_string();
        let description = if self.description.is_empty() {
            self.narrative
        } else {
            self.description
        };
        CatalogControl {
            id: self.id,
            title: self.title,
            description,
            objective: self.objective,
            domains: self.domains,
            evidence: self.evidence,
            tests: self.tests,
            automation,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct EvidenceFile {
    #[serde(default)]
    schema: String,
    #[serde(default)]
    evidence: Vec<EvidenceRow>,
}

#[derive(Debug, Default, Deserialize)]
struct EvidenceRow {
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    evidence_type: String,
    #[serde(default)]
    collection: String,
    #[serde(default)]
    criticality: String,
}

impl EvidenceRow {
    fn into_catalog(self) -> CatalogEvidence {
        CatalogEvidence {
            id: self.id,
            title: self.title,
            evidence_type: self.evidence_type,
            collection: self.collection,
            criticality: self.criticality,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct TestFile {
    #[serde(default)]
    schema: String,
    #[serde(default)]
    test: Vec<TestRow>,
}

#[derive(Debug, Default, Deserialize)]
struct TestRow {
    id: String,
    #[serde(default)]
    control: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    required_evidence: Vec<String>,
    #[serde(default)]
    break_on: Vec<String>,
    #[serde(default)]
    expression: BTreeMap<String, toml::Value>,
    #[serde(default)]
    subjects: Vec<BTreeMap<String, toml::Value>>,
}

impl TestRow {
    fn into_catalog(self) -> CatalogTest {
        CatalogTest {
            id: self.id,
            control: self.control,
            kind: self.kind,
            required_evidence: self.required_evidence,
            break_on: self.break_on,
            expression: self.expression,
            subjects: self.subjects,
        }
    }
}

fn first_nonempty<'a>(values: &[&'a str]) -> Option<&'a str> {
    values.iter().copied().find(|v| !v.trim().is_empty())
}

fn read_to_string(path: &Path) -> Result<String, CatalogError> {
    fs::read_to_string(path).map_err(|source| CatalogError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn parse_toml<T: for<'de> Deserialize<'de>>(path: &Path, raw: &str) -> Result<T, CatalogError> {
    toml::from_str(raw).map_err(|source| CatalogError::Toml {
        path: path.display().to_string(),
        source,
    })
}

fn listed_paths(root: &Path, listed: &[String]) -> Result<Vec<PathBuf>, CatalogError> {
    let mut out = Vec::new();
    for rel in listed {
        if rel.contains("..") || Path::new(rel).is_absolute() {
            return Err(CatalogError::PathEscape(rel.clone()));
        }
        let path = root.join(rel);
        if path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(CatalogError::PathEscape(rel.clone()));
        }
        let canon_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        if let Ok(canon) = path.canonicalize()
            && !canon.starts_with(&canon_root)
        {
            return Err(CatalogError::PathEscape(rel.clone()));
        }
        if !path.is_file() {
            return Err(CatalogError::MissingFile(rel.clone()));
        }
        out.push(path);
    }
    Ok(out)
}

fn reject_unlisted(dir: &Path, listed: &[PathBuf]) -> Result<(), CatalogError> {
    if !dir.is_dir() {
        return Ok(());
    }
    let listed: BTreeSet<PathBuf> = listed.iter().cloned().collect();
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|source| CatalogError::Io {
            path: dir.display().to_string(),
            source,
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("toml"))
        .collect();
    entries.sort();
    for path in entries {
        if !listed.contains(&path) {
            return Err(CatalogError::Unlisted(path.display().to_string()));
        }
    }
    Ok(())
}

fn validate_id(expected_kind: &str, id: &str) -> Result<(), CatalogError> {
    if id != id.to_ascii_lowercase() || id.contains('_') {
        return Err(CatalogError::MalformedId(id.to_string()));
    }
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() < 3 || parts[0] != expected_kind {
        return Err(CatalogError::MalformedId(id.to_string()));
    }
    for part in &parts {
        if part.is_empty()
            || !part.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            || !part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(CatalogError::MalformedId(id.to_string()));
        }
        if PROVIDER_SEGMENTS.contains(part) {
            return Err(CatalogError::Reserved {
                class: "provider".into(),
                segment: (*part).into(),
                id: id.to_string(),
            });
        }
        if FRAMEWORK_SEGMENTS.contains(part) {
            return Err(CatalogError::Reserved {
                class: "framework".into(),
                segment: (*part).into(),
                id: id.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_expression(test: &CatalogTest) -> Result<(), CatalogError> {
    if !test.expression.is_empty() {
        validate_expression_table(&test.expression, &test.id)?;
    }
    for subject in &test.subjects {
        if let Some(kind) = subject.get("kind").and_then(|v| v.as_str())
            && weeping_angel_assurance_ir::SubjectKind::parse_name(kind).is_none()
        {
            return Err(CatalogError::UnknownKind {
                kind: kind.to_string(),
                id: test.id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_expression_table(
    expression: &BTreeMap<String, toml::Value>,
    test_id: &str,
) -> Result<(), CatalogError> {
    if let Some(op) = expression.get("op").and_then(|v| v.as_str()) {
        if !ALLOWED_OPS.contains(&op) {
            return Err(CatalogError::UnknownOperator {
                op: op.to_string(),
                id: test_id.to_string(),
            });
        }
    } else if !expression.is_empty() {
        return Err(CatalogError::MalformedExpression {
            id: test_id.to_string(),
            reason: "missing op".into(),
        });
    }
    for key in ["children", "of", "args", "expressions"] {
        if let Some(arr) = expression.get(key).and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(table) = item.as_table() {
                    let map: BTreeMap<String, toml::Value> = table.clone().into_iter().collect();
                    validate_expression_table(&map, test_id)?;
                }
            }
        }
    }
    if let Some(child) = expression.get("child").and_then(|v| v.as_table()) {
        let map: BTreeMap<String, toml::Value> = child.clone().into_iter().collect();
        validate_expression_table(&map, test_id)?;
    }
    Ok(())
}
