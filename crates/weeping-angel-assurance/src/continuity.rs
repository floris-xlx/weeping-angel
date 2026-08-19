//! Continuity / resilience capability evaluation.
//!
//! `demonstrated_recovery` is derived and excludes plan existence. A plan
//! document, `procedure_present`, or current BCP never proves recovery.
//!
//! Product surface (needles for the dual-suite):
//! `fn evaluate_continuity_resilience`, `struct ContinuityResilienceVerdict`,
//! `plan_existence`, `backup_configuration`, `successful_restore`,
//! `exercise_cadence`, `rto_achievement`, `rpo_achievement`,
//! `unresolved_exercise_findings`, `dependency_coverage`.

use chrono::{DateTime, Utc};
use weeping_angel_assurance_ir::{
    AssessmentDefinition, AssetId, BackupExpectation, CadenceStatus, ContinuityDimension,
    ContinuityExercise, ContinuityGap, ContinuityRemediationRef, ContinuityResilienceError,
    ContinuityResilienceProfile, ContinuityResilienceVerdict, CoverageStatus, DimensionStatus,
    ExerciseOutcome, ExerciseResult, FindingStatus, ObjectiveStatus, RecoveryObjective,
    RestoreStatus, RiskRef, ServiceDependency, ValidateIr,
};
use weeping_angel_control_test::EvidenceSet;
use weeping_angel_evidence::{EvidenceEnvelope, EvidenceValue};

const RECOVERY_PLAN_TYPE: &str = "evidence.resilience.recovery-plan";
const CONTINUITY_PLAN_TYPE: &str = "evidence.resilience.continuity-plan";
const RESTORE_TEST_TYPE: &str = "evidence.backup.restore-test";
const BACKUP_CONFIG_TYPE: &str = "evidence.backup.configuration";

pub fn evaluate_continuity_resilience(
    assessment: &AssessmentDefinition,
    profile: &ContinuityResilienceProfile,
    evidence: &EvidenceSet,
    as_of: DateTime<Utc>,
) -> Result<ContinuityResilienceVerdict, ContinuityResilienceError> {
    assessment
        .validate()
        .map_err(|err| ContinuityResilienceError::Message(err.to_string()))?;

    let plan_existence = evaluate_plan_existence(profile, evidence, as_of);
    let backup_configuration = evaluate_backup_configuration(profile, evidence);
    let exercise_cadence = evaluate_cadence(profile, as_of);
    let technical = latest_technical(profile);
    let coverage_exercise = latest_exercise(profile);
    let successful_restore = evaluate_restore(profile, evidence, technical, exercise_cadence);
    let (rto_achievement, rpo_achievement) = evaluate_objectives(profile, technical);
    let (unresolved_exercise_findings, finding_rems) = evaluate_findings(profile)?;
    let dependency_coverage = evaluate_dependency_coverage(profile, coverage_exercise);

    let backup_ok = matches!(
        backup_configuration,
        DimensionStatus::Satisfied | DimensionStatus::NotApplicable
    );
    let demonstrated_recovery = successful_restore == RestoreStatus::Demonstrated
        && rto_achievement == ObjectiveStatus::Met
        && rpo_achievement == ObjectiveStatus::Met
        && unresolved_exercise_findings == FindingStatus::None
        && dependency_coverage == CoverageStatus::Covered
        && backup_ok
        && exercise_cadence == CadenceStatus::Current
        && technical.is_some_and(|(ex, _)| ex.kind.is_technical());

    let result_refs = result_refs(profile, technical);
    let mut gaps = Vec::new();
    emit_gap(
        &mut gaps,
        plan_existence != DimensionStatus::Satisfied
            && plan_existence != DimensionStatus::NotApplicable,
        ContinuityDimension::PlanExistence,
        "continuity plan is missing or stale",
        &result_refs.0,
        &[],
    );
    emit_gap(
        &mut gaps,
        matches!(
            backup_configuration,
            DimensionStatus::Missing | DimensionStatus::Insufficient | DimensionStatus::Stale
        ),
        ContinuityDimension::BackupConfiguration,
        "required backup configuration evidence is missing",
        &result_refs.0,
        &[],
    );
    emit_gap(
        &mut gaps,
        successful_restore != RestoreStatus::Demonstrated,
        ContinuityDimension::SuccessfulRestore,
        "successful restore is not demonstrated",
        &result_refs.0,
        &[],
    );
    emit_gap(
        &mut gaps,
        exercise_cadence != CadenceStatus::Current,
        ContinuityDimension::ExerciseCadence,
        "exercise cadence is missing or stale",
        &result_refs.0,
        &[],
    );
    emit_gap(
        &mut gaps,
        rto_achievement != ObjectiveStatus::Met,
        ContinuityDimension::RtoAchievement,
        "RTO is not met",
        &result_refs.0,
        &[],
    );
    emit_gap(
        &mut gaps,
        rpo_achievement != ObjectiveStatus::Met,
        ContinuityDimension::RpoAchievement,
        "RPO is not met",
        &result_refs.0,
        &[],
    );
    emit_gap(
        &mut gaps,
        unresolved_exercise_findings == FindingStatus::Open,
        ContinuityDimension::UnresolvedExerciseFindings,
        "exercise findings remain open",
        &result_refs.0,
        &finding_rems,
    );
    emit_gap(
        &mut gaps,
        dependency_coverage == CoverageStatus::Gap,
        ContinuityDimension::DependencyCoverage,
        "critical dependency is not in exercise scope",
        &result_refs.0,
        &[],
    );

    Ok(ContinuityResilienceVerdict {
        profile_id: profile.id.clone(),
        service: profile.service.clone(),
        as_of,
        plan_existence,
        backup_configuration,
        successful_restore,
        exercise_cadence,
        rto_achievement,
        rpo_achievement,
        unresolved_exercise_findings,
        dependency_coverage,
        demonstrated_recovery,
        gaps,
    })
}

fn emit_gap(
    gaps: &mut Vec<ContinuityGap>,
    failing: bool,
    dimension: ContinuityDimension,
    summary: &str,
    risk_refs: &[RiskRef],
    remediation_refs: &[ContinuityRemediationRef],
) {
    if !failing {
        return;
    }
    gaps.push(ContinuityGap {
        dimension,
        summary: summary.into(),
        risk_refs: risk_refs.to_vec(),
        remediation_refs: remediation_refs.to_vec(),
    });
}

fn result_refs(
    profile: &ContinuityResilienceProfile,
    technical: Option<(&ContinuityExercise, &ExerciseResult)>,
) -> (Vec<RiskRef>, Vec<ContinuityRemediationRef>) {
    if let Some((_, result)) = technical {
        return (result.risk_refs.clone(), result.remediation_refs.clone());
    }
    if let Some(result) = profile.results.first() {
        return (result.risk_refs.clone(), result.remediation_refs.clone());
    }
    (Vec::new(), Vec::new())
}

fn evaluate_plan_existence(
    profile: &ContinuityResilienceProfile,
    evidence: &EvidenceSet,
    as_of: DateTime<Utc>,
) -> DimensionStatus {
    if !profile.procedures.is_empty() {
        return DimensionStatus::Satisfied;
    }
    let mut saw_stale = false;
    for env in evidence.iter() {
        if env.provenance().asset().as_str() != profile.service.as_str() {
            continue;
        }
        let ev_type = env.observation().evidence_type().as_str();
        if ev_type == RECOVERY_PLAN_TYPE
            && matches!(
                env.observation().fact_value("procedure_present"),
                Some(EvidenceValue::Bool(true))
            )
        {
            return DimensionStatus::Satisfied;
        }
        if ev_type == CONTINUITY_PLAN_TYPE {
            match reviewed_age_seconds(env, as_of) {
                Some(age)
                    if age <= weeping_angel_assurance_ir::continuity::plan_freshness_seconds() =>
                {
                    return DimensionStatus::Satisfied;
                }
                Some(_) => saw_stale = true,
                None => {}
            }
        }
    }
    if saw_stale {
        DimensionStatus::Stale
    } else {
        DimensionStatus::Missing
    }
}

fn reviewed_age_seconds(env: &EvidenceEnvelope, as_of: DateTime<Utc>) -> Option<i64> {
    match env.observation().fact_value("reviewed_at") {
        Some(EvidenceValue::Timestamp(at)) => Some((as_of - *at).num_seconds()),
        _ => None,
    }
}

fn evaluate_backup_configuration(
    profile: &ContinuityResilienceProfile,
    evidence: &EvidenceSet,
) -> DimensionStatus {
    let required: Vec<&BackupExpectation> = profile
        .backup_expectations
        .iter()
        .filter(|e| e.required)
        .collect();
    if required.is_empty() {
        return DimensionStatus::NotApplicable;
    }
    let mut any_insufficient = false;
    for expectation in required {
        let ev_type = if expectation.evidence_type.is_empty() {
            BACKUP_CONFIG_TYPE
        } else {
            expectation.evidence_type.as_str()
        };
        match find_subject_evidence(evidence, ev_type, &expectation.subject) {
            None => return DimensionStatus::Missing,
            Some(env) => {
                if matches!(
                    env.observation().fact_value("enabled"),
                    Some(EvidenceValue::Bool(false))
                ) {
                    any_insufficient = true;
                }
            }
        }
    }
    if any_insufficient {
        DimensionStatus::Insufficient
    } else {
        DimensionStatus::Satisfied
    }
}

fn find_subject_evidence<'a>(
    evidence: &'a EvidenceSet,
    evidence_type: &str,
    subject: &AssetId,
) -> Option<&'a EvidenceEnvelope> {
    evidence.iter().find(|env| {
        env.observation().evidence_type().as_str() == evidence_type
            && env.provenance().asset().as_str() == subject.as_str()
    })
}

fn evaluate_cadence(profile: &ContinuityResilienceProfile, as_of: DateTime<Utc>) -> CadenceStatus {
    let Some(latest) = latest_exercise(profile) else {
        return CadenceStatus::Missing;
    };
    let Some(cadence) = profile.exercise_cadence_seconds else {
        return CadenceStatus::Current;
    };
    let age = (as_of - latest.conducted_at).num_seconds();
    if age > cadence as i64 {
        CadenceStatus::Stale
    } else {
        CadenceStatus::Current
    }
}

fn latest_exercise(profile: &ContinuityResilienceProfile) -> Option<&ContinuityExercise> {
    profile.exercises.iter().max_by_key(|ex| ex.conducted_at)
}

fn latest_technical(
    profile: &ContinuityResilienceProfile,
) -> Option<(&ContinuityExercise, &ExerciseResult)> {
    profile
        .exercises
        .iter()
        .filter(|ex| ex.kind.is_technical())
        .filter_map(|ex| {
            profile
                .results
                .iter()
                .find(|r| r.exercise_id == ex.id)
                .map(|r| (ex, r))
        })
        .max_by_key(|(ex, _)| ex.conducted_at)
}

fn evaluate_restore(
    profile: &ContinuityResilienceProfile,
    evidence: &EvidenceSet,
    technical: Option<(&ContinuityExercise, &ExerciseResult)>,
    cadence: CadenceStatus,
) -> RestoreStatus {
    if restore_evidence_failed(profile, evidence) {
        return RestoreStatus::Failed;
    }
    match technical {
        Some((_, result)) if result.outcome == ExerciseOutcome::Failed => RestoreStatus::Failed,
        Some((ex, result))
            if result.outcome == ExerciseOutcome::Passed && ex.kind.is_technical() =>
        {
            if cadence == CadenceStatus::Stale {
                RestoreStatus::Stale
            } else {
                RestoreStatus::Demonstrated
            }
        }
        Some(_) => RestoreStatus::Missing,
        None => {
            let only_tabletop = !profile.exercises.is_empty()
                && profile.exercises.iter().all(|ex| ex.kind.is_tabletop());
            if only_tabletop {
                RestoreStatus::NotApplicable
            } else {
                RestoreStatus::Missing
            }
        }
    }
}

fn restore_evidence_failed(profile: &ContinuityResilienceProfile, evidence: &EvidenceSet) -> bool {
    profile.backup_expectations.iter().any(|expectation| {
        find_subject_evidence(evidence, RESTORE_TEST_TYPE, &expectation.subject).is_some_and(
            |env| {
                matches!(
                    env.observation().fact_value("success"),
                    Some(EvidenceValue::Bool(false))
                )
            },
        )
    }) || find_subject_evidence(evidence, RESTORE_TEST_TYPE, &profile.service).is_some_and(|env| {
        matches!(
            env.observation().fact_value("success"),
            Some(EvidenceValue::Bool(false))
        )
    })
}

fn evaluate_objectives(
    profile: &ContinuityResilienceProfile,
    technical: Option<(&ContinuityExercise, &ExerciseResult)>,
) -> (ObjectiveStatus, ObjectiveStatus) {
    let Some((exercise, result)) = technical else {
        return (ObjectiveStatus::NotMeasured, ObjectiveStatus::NotMeasured);
    };
    if !exercise.kind.is_technical() {
        return (ObjectiveStatus::NotMeasured, ObjectiveStatus::NotMeasured);
    }
    let objectives = service_objectives(profile);
    (
        compare_objective(
            result.observed_recovery_duration_seconds,
            objectives.iter().map(|o| o.rto_seconds),
        ),
        compare_objective(
            result.observed_data_loss_window_seconds,
            objectives.iter().map(|o| o.rpo_seconds),
        ),
    )
}

fn service_objectives(profile: &ContinuityResilienceProfile) -> Vec<&RecoveryObjective> {
    let for_service: Vec<_> = profile
        .objectives
        .iter()
        .filter(|o| o.subject.as_str() == profile.service.as_str())
        .collect();
    if for_service.is_empty() {
        profile.objectives.iter().collect()
    } else {
        for_service
    }
}

fn compare_objective(
    observed: Option<u64>,
    limits: impl IntoIterator<Item = u64>,
) -> ObjectiveStatus {
    let Some(observed) = observed else {
        return ObjectiveStatus::NotMeasured;
    };
    let limits: Vec<u64> = limits.into_iter().collect();
    if limits.is_empty() {
        return ObjectiveStatus::NotMeasured;
    }
    if limits.iter().all(|limit| observed <= *limit) {
        ObjectiveStatus::Met
    } else {
        ObjectiveStatus::Missed
    }
}

fn evaluate_findings(
    profile: &ContinuityResilienceProfile,
) -> Result<(FindingStatus, Vec<ContinuityRemediationRef>), ContinuityResilienceError> {
    let mut remediations = Vec::new();
    let mut open = false;
    for result in &profile.results {
        for issue in &result.issues {
            if !issue.open {
                continue;
            }
            open = true;
            if issue.remediation_refs.is_empty() {
                return Err(ContinuityResilienceError::UntrackedExerciseFinding {
                    id: issue.id.clone(),
                });
            }
            remediations.extend(issue.remediation_refs.clone());
        }
    }
    remediations.sort_by(|a, b| a.id.cmp(&b.id));
    remediations.dedup_by(|a, b| a.id == b.id);
    Ok((
        if open {
            FindingStatus::Open
        } else {
            FindingStatus::None
        },
        remediations,
    ))
}

fn evaluate_dependency_coverage(
    profile: &ContinuityResilienceProfile,
    exercise: Option<&ContinuityExercise>,
) -> CoverageStatus {
    let critical: Vec<&ServiceDependency> =
        profile.dependencies.iter().filter(|d| d.critical).collect();
    if critical.is_empty() {
        return CoverageStatus::Covered;
    }
    let Some(exercise) = exercise else {
        return CoverageStatus::Gap;
    };
    let scoped: std::collections::BTreeSet<String> = exercise
        .in_scope_dependencies
        .iter()
        .map(weeping_angel_assurance_ir::AssetRef::as_key)
        .collect();
    if critical.iter().all(|dep| scoped.contains(&dep.to.as_key())) {
        CoverageStatus::Covered
    } else {
        CoverageStatus::Gap
    }
}
