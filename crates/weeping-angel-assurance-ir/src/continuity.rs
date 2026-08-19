//! Continuity / disaster-recovery capability IR.
//!
//! Plan documents and `procedure_present` are intention evidence only. They
//! never prove recovery. `demonstrated_recovery` is derived and excludes
//! plan existence. Tabletop / walkthrough cannot satisfy RTO/RPO or restore.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AssessmentDefinition, AssetId, AssetKind, ContinuityExerciseId, ContinuityProfileId,
    DocumentRef, RecoveryObjectiveId, RiskId, VendorId,
};

const BACKUP_CONFIGURATION_EVIDENCE: &str = "evidence.backup.configuration";
const PLAN_FRESHNESS_SECONDS: i64 = 365 * 24 * 3600;

/// Fail-closed continuity evaluation / graph errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContinuityResilienceError {
    #[error("{0}")]
    Message(String),
    #[error("untracked exercise finding {id}")]
    UntrackedExerciseFinding { id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceCriticality {
    MissionCritical,
    High,
    Medium,
    Low,
}

impl ServiceCriticality {
    pub fn requires_exercise_cadence(self) -> bool {
        matches!(self, Self::MissionCritical | Self::High)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyKind {
    Runtime,
    Data,
    Identity,
    Network,
    Supplier,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetRef {
    Asset(AssetId),
    Vendor(VendorId),
}

impl AssetRef {
    pub fn as_key(&self) -> String {
        match self {
            Self::Asset(id) => format!("asset:{}", id.as_str()),
            Self::Vendor(id) => format!("vendor:{}", id.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDependency {
    pub from: AssetId,
    pub to: AssetRef,
    pub kind: DependencyKind,
    pub critical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryObjective {
    pub id: RecoveryObjectiveId,
    pub subject: AssetId,
    pub rto_seconds: u64,
    pub rpo_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupExpectation {
    pub subject: AssetId,
    pub required: bool,
    #[serde(default = "backup_configuration_type")]
    pub evidence_type: String,
}

fn backup_configuration_type() -> String {
    BACKUP_CONFIGURATION_EVIDENCE.into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecoveryProcedureRole {
    BusinessContinuityPlan,
    DisasterRecoveryPlan,
    RecoveryRunbook,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryProcedureRef {
    pub document: DocumentRef,
    pub role: RecoveryProcedureRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExerciseKind {
    Tabletop,
    Walkthrough,
    TechnicalRecovery,
    RestoreTest,
    Other,
}

impl ExerciseKind {
    pub fn is_technical(self) -> bool {
        matches!(self, Self::TechnicalRecovery | Self::RestoreTest)
    }

    pub fn is_tabletop(self) -> bool {
        matches!(self, Self::Tabletop | Self::Walkthrough)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuityExercise {
    pub id: ContinuityExerciseId,
    pub subject: AssetId,
    pub kind: ExerciseKind,
    pub conducted_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub procedure: Option<RecoveryProcedureRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub in_scope_dependencies: Vec<AssetRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExerciseOutcome {
    Passed,
    Failed,
    Partial,
    NotExecuted,
}

/// Opaque Prompt 16 identity. Distinct from the typed-id `crate::RemediationRef`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuityRemediationRef {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskRef {
    pub id: RiskId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseIssue {
    pub id: String,
    pub summary: String,
    pub open: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_refs: Vec<ContinuityRemediationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseResult {
    pub exercise_id: ContinuityExerciseId,
    pub outcome: ExerciseOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_recovery_duration_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_data_loss_window_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<ExerciseIssue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_refs: Vec<ContinuityRemediationRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_refs: Vec<RiskRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuityResilienceProfile {
    pub id: ContinuityProfileId,
    pub service: AssetId,
    pub criticality: ServiceCriticality,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<ServiceDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objectives: Vec<RecoveryObjective>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub backup_expectations: Vec<BackupExpectation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub procedures: Vec<RecoveryProcedureRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exercise_cadence_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exercises: Vec<ContinuityExercise>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<ExerciseResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DimensionStatus {
    Satisfied,
    Missing,
    Stale,
    Insufficient,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreStatus {
    Demonstrated,
    Failed,
    Missing,
    Stale,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CadenceStatus {
    Current,
    Stale,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjectiveStatus {
    Met,
    Missed,
    NotMeasured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FindingStatus {
    None,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CoverageStatus {
    Covered,
    Gap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContinuityDimension {
    PlanExistence,
    BackupConfiguration,
    SuccessfulRestore,
    ExerciseCadence,
    RtoAchievement,
    RpoAchievement,
    UnresolvedExerciseFindings,
    DependencyCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuityGap {
    pub dimension: ContinuityDimension,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_refs: Vec<RiskRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_refs: Vec<ContinuityRemediationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuityResilienceVerdict {
    pub profile_id: ContinuityProfileId,
    pub service: AssetId,
    pub as_of: DateTime<Utc>,
    pub plan_existence: DimensionStatus,
    pub backup_configuration: DimensionStatus,
    pub successful_restore: RestoreStatus,
    pub exercise_cadence: CadenceStatus,
    pub rto_achievement: ObjectiveStatus,
    pub rpo_achievement: ObjectiveStatus,
    pub unresolved_exercise_findings: FindingStatus,
    pub dependency_coverage: CoverageStatus,
    pub demonstrated_recovery: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<ContinuityGap>,
}

pub fn validate_continuity_profiles(assessment: &AssessmentDefinition) -> Result<(), String> {
    let mut profile_ids = std::collections::BTreeSet::new();
    let asset_by_id: std::collections::BTreeMap<_, _> = assessment
        .assets
        .iter()
        .map(|a| (a.id.as_str(), a))
        .collect();
    let vendor_ids: std::collections::BTreeSet<_> = assessment
        .vendors
        .iter()
        .map(|v| v.id.as_str().to_string())
        .collect();
    let risk_ids: std::collections::BTreeSet<_> = assessment
        .risks
        .iter()
        .map(|r| r.id.as_str().to_string())
        .collect();
    let remediation_ids: std::collections::BTreeSet<_> = assessment
        .remediations
        .iter()
        .map(|r| r.id.as_str().to_string())
        .collect();
    let document_ids = document_registry_ids(assessment);

    for profile in &assessment.continuity_profiles {
        if !profile_ids.insert(profile.id.as_str().to_string()) {
            return Err(format!("duplicate continuity profile id {}", profile.id));
        }

        match asset_by_id.get(profile.service.as_str()) {
            None => {
                return Err(format!(
                    "dangling continuity service {} on profile {}",
                    profile.service, profile.id
                ));
            }
            Some(asset) if asset.kind != AssetKind::Service => {
                return Err(format!(
                    "continuity service {} on profile {} is not AssetKind::Service",
                    profile.service, profile.id
                ));
            }
            Some(_) => {}
        }

        if profile.criticality.requires_exercise_cadence()
            && profile
                .exercise_cadence_seconds
                .is_none_or(|seconds| seconds == 0)
        {
            return Err(format!(
                "exercise cadence required for {} profile {}",
                match profile.criticality {
                    ServiceCriticality::MissionCritical => "mission-critical",
                    ServiceCriticality::High => "high",
                    _ => "critical",
                },
                profile.id
            ));
        }

        let mut objective_ids = std::collections::BTreeSet::new();
        for objective in &profile.objectives {
            if !objective_ids.insert(objective.id.as_str().to_string()) {
                return Err(format!(
                    "duplicate continuity objective id {}",
                    objective.id
                ));
            }
            if objective.rto_seconds == 0 {
                return Err(format!(
                    "rto must be > 0 seconds on objective {}",
                    objective.id
                ));
            }
            if !asset_by_id.contains_key(objective.subject.as_str()) {
                return Err(format!(
                    "dangling recovery objective subject {} on {}",
                    objective.subject, objective.id
                ));
            }
        }

        for dep in &profile.dependencies {
            if dep.from.as_str() != profile.service.as_str() {
                return Err(format!(
                    "dependency from {} must equal continuity profile service {}",
                    dep.from, profile.service
                ));
            }
            match &dep.to {
                AssetRef::Asset(id) if !asset_by_id.contains_key(id.as_str()) => {
                    return Err(format!("dangling dependency asset {id}"));
                }
                AssetRef::Vendor(id) if !vendor_ids.contains(id.as_str()) => {
                    return Err(format!("dangling dependency vendor {id}"));
                }
                _ => {}
            }
        }

        for expectation in &profile.backup_expectations {
            if !asset_by_id.contains_key(expectation.subject.as_str()) {
                return Err(format!(
                    "dangling backup expectation subject {}",
                    expectation.subject
                ));
            }
        }

        let mut exercise_ids = std::collections::BTreeSet::new();
        for exercise in &profile.exercises {
            if !exercise_ids.insert(exercise.id.as_str().to_string()) {
                return Err(format!("duplicate continuity exercise id {}", exercise.id));
            }
            if !asset_by_id.contains_key(exercise.subject.as_str()) {
                return Err(format!(
                    "dangling exercise subject {} on {}",
                    exercise.subject, exercise.id
                ));
            }
            if let Some(procedure) = &exercise.procedure {
                validate_document_ref(&procedure.document, &document_ids)?;
            }
            for target in &exercise.in_scope_dependencies {
                match target {
                    AssetRef::Asset(id) if !asset_by_id.contains_key(id.as_str()) => {
                        return Err(format!("dangling in-scope asset {id}"));
                    }
                    AssetRef::Vendor(id) if !vendor_ids.contains(id.as_str()) => {
                        return Err(format!("dangling in-scope vendor {id}"));
                    }
                    _ => {}
                }
            }
        }

        for procedure in &profile.procedures {
            validate_document_ref(&procedure.document, &document_ids)?;
        }

        for result in &profile.results {
            if !exercise_ids.contains(result.exercise_id.as_str()) {
                return Err(format!(
                    "dangling exercise {} on result",
                    result.exercise_id
                ));
            }
            for risk in &result.risk_refs {
                if !risk_ids.contains(risk.id.as_str()) {
                    return Err(format!("dangling risk {}", risk.id));
                }
            }
            if !remediation_ids.is_empty() {
                for rem in &result.remediation_refs {
                    if !remediation_ids.contains(rem.id.as_str()) {
                        return Err(format!("dangling remediation {}", rem.id));
                    }
                }
                for issue in &result.issues {
                    for rem in &issue.remediation_refs {
                        if !remediation_ids.contains(rem.id.as_str()) {
                            return Err(format!("dangling remediation {}", rem.id));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn document_registry_ids(
    assessment: &AssessmentDefinition,
) -> Option<std::collections::BTreeSet<String>> {
    let _ = assessment;
    None
}

fn validate_document_ref(
    document: &DocumentRef,
    registry: &Option<std::collections::BTreeSet<String>>,
) -> Result<(), String> {
    if let Some(ids) = registry
        && !ids.contains(document.id.as_str())
    {
        return Err(format!("dangling document {}", document.id));
    }
    Ok(())
}

pub fn plan_freshness_seconds() -> i64 {
    PLAN_FRESHNESS_SECONDS
}

pub fn backup_configuration_evidence_type() -> &'static str {
    BACKUP_CONFIGURATION_EVIDENCE
}
