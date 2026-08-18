//! Framework compile: profile + capabilities → CompiledFramework.
//!
//! Pure. No network I/O. ISO 27001:2022 loads a versioned framework pack.

pub mod pack;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use weeping_angel_assurance_ir::{
    ASSURANCE_IR_SCHEMA, AssessmentId, Control, ControlId, ControlTestId, EvidenceRequirement,
    EvidenceType, FrameworkVersion, PlannedTestKind, Requirement, ValidateIr, canonical_digest,
};

pub use weeping_angel_assurance_ir::{Assessment, AssessmentDefinition, AssessmentRequests};

pub use pack::{
    FrameworkContentProvider, FrameworkPackDigest, LoadedPack, assessment_from_pack,
    load_framework_pack, load_framework_pack_from, validate_framework_pack,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FrameworkProfile {
    Iso27001,
    Iso27701,
    Gdpr,
    Soc2,
    Nis2,
    Dora,
    Iso27007,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown framework profile: {value}")]
pub struct UnknownProfileError {
    pub value: String,
}

impl TryFrom<&str> for FrameworkProfile {
    type Error = UnknownProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "iso27001" | "iso-27001" => Ok(Self::Iso27001),
            "iso27701" | "iso-27701" => Ok(Self::Iso27701),
            "iso27007" | "iso-27007" => Ok(Self::Iso27007),
            "gdpr" => Ok(Self::Gdpr),
            "soc2" | "soc-2" => Ok(Self::Soc2),
            "nis2" | "nis-2" => Ok(Self::Nis2),
            "dora" => Ok(Self::Dora),
            _ => Err(UnknownProfileError {
                value: value.to_string(),
            }),
        }
    }
}

impl FrameworkProfile {
    pub fn as_selector(self) -> &'static str {
        match self {
            Self::Iso27001 => "iso-27001",
            Self::Iso27701 => "iso-27701",
            Self::Gdpr => "gdpr",
            Self::Soc2 => "soc-2",
            Self::Nis2 => "nis-2",
            Self::Dora => "dora",
            Self::Iso27007 => "iso-27007",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameworkCapabilities {
    pub supports_control_applicability: bool,
    pub supports_statement_of_applicability: bool,
    pub supports_privacy_processing: bool,
    pub supports_risk_treatment: bool,
    pub supports_manual_attestation: bool,
    pub supports_sampling: bool,
    pub supports_audit_program: bool,
    pub supports_nonconformities: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameworkContext {
    pub notes: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkTarget {
    pub profile: FrameworkProfile,
    pub capabilities: FrameworkCapabilities,
    pub version: FrameworkVersion,
    pub context: FrameworkContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileValidation {
    pub stages: Vec<String>,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompiledTest {
    pub id: ControlTestId,
    pub control_id: ControlId,
    pub kind: PlannedTestKind,
    pub required: Vec<EvidenceType>,
    pub break_on: Vec<EvidenceType>,
    /// Optional bounded test expression (JSON `TestExpr`). Side-table until packs ship bodies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expr: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompiledFramework {
    pub applicable_requirements: Vec<Requirement>,
    pub controls: Vec<Control>,
    pub tests: Vec<CompiledTest>,
    pub evidence_requirements: Vec<EvidenceRequirement>,
    pub validation: CompileValidation,
    pub digest: String,
}

#[derive(Debug, Error)]
pub enum FrameworkCompileError {
    #[error("capability violation: {capability}: {message}")]
    CapabilityViolation { capability: String, message: String },
    #[error("unknown profile: {profile}")]
    UnknownProfile { profile: String },
    #[error("identity error: {message}")]
    Identity { message: String },
    #[error("mapping integrity: {message}")]
    MappingIntegrity { message: String },
    #[error("schema error: {message}")]
    Schema { message: String },
    #[error("digest mismatch: expected {expected}, got {actual}")]
    DigestMismatch { expected: String, actual: String },
    #[error("unknown pack: {message}")]
    UnknownPack { message: String },
    #[error("unknown requirement: {id}")]
    UnknownRequirement { id: String },
}

const PIPELINE_STAGES: &[&str] = &[
    "normalize",
    "resolve_applicability",
    "validate_capabilities",
    "resolve_control_mappings",
    "resolve_evidence_requirements",
    "construct_test_plan",
    "construct_framework_projection",
    "integrity_validation",
];

pub fn compile_framework(
    assessment: &Assessment,
    target: &FrameworkTarget,
) -> Result<CompiledFramework, FrameworkCompileError> {
    let mut stages = Vec::new();

    let normalized = normalize(assessment, target)?;
    normalized
        .validate()
        .map_err(|e| FrameworkCompileError::Schema {
            message: e.to_string(),
        })?;
    stages.push(PIPELINE_STAGES[0].to_string());

    let applicable_requirements = resolve_applicability(&normalized, target)?;
    stages.push(PIPELINE_STAGES[1].to_string());

    validate_capabilities(&normalized, target)?;
    stages.push(PIPELINE_STAGES[2].to_string());

    let controls = resolve_control_mappings(&normalized)?;
    stages.push(PIPELINE_STAGES[3].to_string());

    let evidence_requirements = resolve_evidence_requirements(&normalized)?;
    stages.push(PIPELINE_STAGES[4].to_string());

    let tests = construct_test_plan(&normalized, &controls, &evidence_requirements)?;
    stages.push(PIPELINE_STAGES[5].to_string());

    let projection = construct_framework_projection(target, &applicable_requirements, &controls);
    let _ = projection;
    stages.push(PIPELINE_STAGES[6].to_string());

    let digest_body = DigestBody {
        schema_version: ASSURANCE_IR_SCHEMA,
        assessment_id: &normalized.id,
        profile: target.profile.as_selector(),
        version: target.version.as_str(),
        applicable_requirements: &applicable_requirements,
        controls: &controls,
        tests: &tests,
        evidence_requirements: &evidence_requirements,
    };
    let digest = canonical_digest(&digest_body).map_err(|e| FrameworkCompileError::Schema {
        message: e.to_string(),
    })?;
    let again = canonical_digest(&digest_body).map_err(|e| FrameworkCompileError::Schema {
        message: e.to_string(),
    })?;
    if digest != again {
        return Err(FrameworkCompileError::DigestMismatch {
            expected: digest,
            actual: again,
        });
    }
    stages.push(PIPELINE_STAGES[7].to_string());

    Ok(CompiledFramework {
        applicable_requirements,
        controls,
        tests,
        evidence_requirements,
        validation: CompileValidation { stages, ok: true },
        digest,
    })
}

fn normalize(
    assessment: &Assessment,
    target: &FrameworkTarget,
) -> Result<Assessment, FrameworkCompileError> {
    if assessment.schema_version != ASSURANCE_IR_SCHEMA {
        return Err(FrameworkCompileError::Schema {
            message: format!(
                "expected {}, got {}",
                ASSURANCE_IR_SCHEMA, assessment.schema_version
            ),
        });
    }
    if assessment.id.as_str().is_empty() {
        return Err(FrameworkCompileError::Identity {
            message: "assessment id is empty".into(),
        });
    }
    let _ = target;
    let mut out = assessment.clone();
    if target.profile == FrameworkProfile::Iso27001 && target.version.as_str() == "2022" {
        match pack::load_framework_pack("iso-27001", "2022") {
            Ok(loaded) => pack::merge_pack(&mut out, &loaded),
            Err(pack::PackError::UnknownPack(_)) => {}
            Err(err) => return Err(err.into()),
        }
    }
    out.requirements
        .sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
    out.controls
        .sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
    out.mappings.sort_by(|a, b| {
        (a.from_requirement().as_str(), a.to_control().as_str())
            .cmp(&(b.from_requirement().as_str(), b.to_control().as_str()))
    });
    out.evidence_requirements
        .sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
    out.tests.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(out)
}

fn resolve_applicability(
    assessment: &Assessment,
    target: &FrameworkTarget,
) -> Result<Vec<Requirement>, FrameworkCompileError> {
    let _ = target;
    Ok(assessment
        .requirements
        .iter()
        .filter(|req| req.applicability().statically_applicable() != Some(false))
        .cloned()
        .collect())
}

fn validate_capabilities(
    assessment: &Assessment,
    target: &FrameworkTarget,
) -> Result<(), FrameworkCompileError> {
    let req = &assessment.requests;
    let cap = &target.capabilities;
    let checks = [
        (
            req.statement_of_applicability,
            cap.supports_statement_of_applicability,
            "supports_statement_of_applicability",
        ),
        (
            req.control_applicability,
            cap.supports_control_applicability,
            "supports_control_applicability",
        ),
        (
            req.privacy_processing,
            cap.supports_privacy_processing,
            "supports_privacy_processing",
        ),
        (
            req.risk_treatment,
            cap.supports_risk_treatment,
            "supports_risk_treatment",
        ),
        (
            req.manual_attestation,
            cap.supports_manual_attestation,
            "supports_manual_attestation",
        ),
        (req.sampling, cap.supports_sampling, "supports_sampling"),
        (
            req.audit_program,
            cap.supports_audit_program,
            "supports_audit_program",
        ),
        (
            req.nonconformities,
            cap.supports_nonconformities,
            "supports_nonconformities",
        ),
    ];
    for (requested, supported, name) in checks {
        if requested && !supported {
            return Err(FrameworkCompileError::CapabilityViolation {
                capability: name.to_string(),
                message: format!("requested {name} but the target does not enable it"),
            });
        }
    }
    Ok(())
}

fn resolve_control_mappings(
    assessment: &Assessment,
) -> Result<Vec<Control>, FrameworkCompileError> {
    for mapping in &assessment.mappings {
        let req_ok = assessment
            .requirements
            .iter()
            .any(|r| r.id() == mapping.from_requirement());
        let ctl_ok = assessment
            .controls
            .iter()
            .any(|c| c.id() == mapping.to_control());
        if !req_ok || !ctl_ok {
            return Err(FrameworkCompileError::MappingIntegrity {
                message: format!(
                    "mapping {} → {} is not grounded in the assessment",
                    mapping.from_requirement(),
                    mapping.to_control()
                ),
            });
        }
    }
    Ok(assessment.controls.clone())
}

fn resolve_evidence_requirements(
    assessment: &Assessment,
) -> Result<Vec<EvidenceRequirement>, FrameworkCompileError> {
    Ok(assessment.evidence_requirements.clone())
}

fn construct_test_plan(
    assessment: &Assessment,
    controls: &[Control],
    evidence_requirements: &[EvidenceRequirement],
) -> Result<Vec<CompiledTest>, FrameworkCompileError> {
    if !assessment.tests.is_empty() {
        return Ok(assessment
            .tests
            .iter()
            .map(|t| CompiledTest {
                id: t.id.clone(),
                control_id: t.control_id.clone(),
                kind: t.kind,
                required: t.required_evidence.clone(),
                break_on: t.break_on.clone(),
                expr: None,
            })
            .collect());
    }
    let Some(control) = controls.first() else {
        return Ok(Vec::new());
    };
    let tests = evidence_requirements
        .iter()
        .map(|ev| CompiledTest {
            id: ControlTestId::new(format!("test.{}", ev.id())),
            control_id: control.id().clone(),
            kind: PlannedTestKind::Automated,
            required: vec![ev.evidence_type().clone()],
            break_on: vec![EvidenceType::new("exposed_without_auth")],
            expr: None,
        })
        .collect();
    Ok(tests)
}

fn construct_framework_projection(
    target: &FrameworkTarget,
    requirements: &[Requirement],
    controls: &[Control],
) -> Projection {
    // Phase 9–17 catalogs are not shipped. Projection is identity over the compiled IR.
    let _ = stub_catalog(target.profile);
    Projection {
        _requirement_count: requirements.len(),
        _control_count: controls.len(),
        _selector: target.profile.as_selector().to_string(),
    }
}

struct Projection {
    _requirement_count: usize,
    _control_count: usize,
    _selector: String,
}

/// Profile catalog. ISO 27001:2022 is loaded from the versioned pack.
pub fn stub_catalog(profile: FrameworkProfile) -> Vec<Requirement> {
    match profile {
        FrameworkProfile::Iso27001 => pack::load_framework_pack("iso-27001", "2022")
            .map(|p| p.requirements)
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DigestBody<'a> {
    schema_version: &'a str,
    assessment_id: &'a AssessmentId,
    profile: &'a str,
    version: &'a str,
    applicable_requirements: &'a [Requirement],
    controls: &'a [Control],
    tests: &'a [CompiledTest],
    evidence_requirements: &'a [EvidenceRequirement],
}
