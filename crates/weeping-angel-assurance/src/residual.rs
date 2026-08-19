//! Project control-test `Effectiveness` into an immutable residual-risk document.
//!
//! Consumes `weeping_angel_control_test::{ControlTestResult, Effectiveness}`.
//! Does not implement Prompt 05/06/08 engines. Evidence crate stays
//! conclusion-free.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use weeping_angel_assurance_ir::{
    CONTROL_EFFECTIVENESS_METHODOLOGY_ID, ControlId, ControlTestSnapshotRef, Exception,
    ExceptionId, InherentRiskSnapshot, MIN_RESIDUAL_FLOOR, ManualResidualAssessment,
    MethodologyRef, ResidualReductionStep, ResidualRiskError, ResidualRiskId, ResidualRiskMode,
    ResidualRiskProjection, TreatmentCompleteness, TreatmentPlanSnapshot, canonical_digest,
};
use weeping_angel_control_test::{ControlTestResult, Effectiveness};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidualRiskRequest {
    pub mode: ResidualRiskMode,
    pub inherent: InherentRiskSnapshot,
    pub treatment: TreatmentPlanSnapshot,
    pub methodology: MethodologyRef,
    pub control_tests: ControlTestSnapshotRef,
    #[serde(default)]
    pub control_test_results: Vec<ControlTestResult>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual: Option<ManualResidualAssessment>,
    pub projected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ResidualRiskStore {
    projections: BTreeMap<ResidualRiskId, ResidualRiskProjection>,
}

impl ResidualRiskStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, projection: ResidualRiskProjection) {
        self.projections
            .entry(projection.id.clone())
            .or_insert(projection);
    }

    pub fn get(&self, id: &ResidualRiskId) -> Option<ResidualRiskProjection> {
        self.projections.get(id).cloned()
    }
}

pub fn project_residual_risk(
    store: &mut ResidualRiskStore,
    request: ResidualRiskRequest,
) -> Result<ResidualRiskProjection, ResidualRiskError> {
    let projection = project_request(&request)?;
    store.insert(projection.clone());
    Ok(projection)
}

pub fn query_residual_risk(
    store: &ResidualRiskStore,
    id: &ResidualRiskId,
) -> Option<ResidualRiskProjection> {
    store.get(id)
}

fn project_request(
    request: &ResidualRiskRequest,
) -> Result<ResidualRiskProjection, ResidualRiskError> {
    validate_pins(request)?;
    let results_by_control = index_results(&request.control_test_results);
    let mut trace = Vec::new();
    let mut calculated = calculate_residual(request, &results_by_control, &mut trace)?;

    let (residual_ordinal, residual_rating_id, manual) = match request.mode {
        ResidualRiskMode::Calculated => (calculated.ordinal, calculated.rating_id, None),
        ResidualRiskMode::Assessed => {
            let manual = require_manual(request.manual.as_ref())?;
            (
                manual.residual_ordinal,
                manual.residual_rating_id.clone(),
                Some(manual.clone()),
            )
        }
        ResidualRiskMode::Hybrid => {
            let manual = require_management(request.manual.as_ref())?;
            if manual.residual_ordinal > calculated.ordinal {
                calculated.ordinal = manual.residual_ordinal;
                calculated.rating_id = manual.residual_rating_id.clone();
            }
            trace.push(ResidualReductionStep {
                control_id: None,
                effectiveness: None,
                step: 0,
                note: format!(
                    "hybrid combines deterministic signals with approved management assessment ({})",
                    manual.residual_rating_id
                ),
            });
            (
                calculated.ordinal,
                calculated.rating_id,
                Some(manual.clone()),
            )
        }
    };

    let exception_ids = collect_exception_ids(request, &results_by_control);
    let mut projection = ResidualRiskProjection {
        id: ResidualRiskId::new("residual:pending"),
        risk_id: request.inherent.pin.risk_id.clone(),
        mode: request.mode,
        inherent: request.inherent.pin.clone(),
        treatment: request.treatment.pin.clone(),
        methodology: request.methodology.clone(),
        relevant_control_ids: request.treatment.relevant_control_ids.clone(),
        control_tests: request.control_tests.clone(),
        projected_at: request.projected_at,
        residual_ordinal,
        residual_rating_id,
        reduction_trace: trace,
        manual,
        exception_ids,
    };
    projection.id = seal_projection_id(&projection);
    Ok(projection)
}

struct CalculatedResidual {
    ordinal: u32,
    rating_id: String,
}

fn validate_pins(request: &ResidualRiskRequest) -> Result<(), ResidualRiskError> {
    if request.inherent.pin.version.trim().is_empty() {
        return Err(ResidualRiskError::MissingInherentRiskVersion);
    }
    if request.treatment.pin.version.trim().is_empty() {
        return Err(ResidualRiskError::MissingTreatmentPlanVersion);
    }
    if request.methodology.methodology_id.trim().is_empty()
        || request.methodology.version.trim().is_empty()
    {
        return Err(ResidualRiskError::MissingMethodologyVersion);
    }
    if !request.methodology.is_known_v1() {
        return Err(ResidualRiskError::UnknownMethodology);
    }
    if request.control_tests.digest.trim().is_empty()
        || (request.control_tests.result_ids.is_empty() && request.control_test_results.is_empty())
    {
        return Err(ResidualRiskError::MissingControlTestSnapshot);
    }
    Ok(())
}

fn index_results(results: &[ControlTestResult]) -> BTreeMap<ControlId, Vec<&ControlTestResult>> {
    let mut map = BTreeMap::new();
    for result in results {
        map.entry(result.control_id.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }
    map
}

fn calculate_residual(
    request: &ResidualRiskRequest,
    results_by_control: &BTreeMap<ControlId, Vec<&ControlTestResult>>,
    trace: &mut Vec<ResidualReductionStep>,
) -> Result<CalculatedResidual, ResidualRiskError> {
    let no_reduction = request.methodology.is_no_reduction_v1();
    let calculated_credit = request.mode != ResidualRiskMode::Assessed && !no_reduction;
    let mut total_step = 0u32;
    let mut ordered = request.treatment.relevant_control_ids.clone();
    ordered.sort();
    ordered.dedup();

    for control_id in &ordered {
        let Some(results) = results_by_control.get(control_id) else {
            return Err(ResidualRiskError::DanglingControl);
        };
        let chosen = conservative_result(results);
        let effectiveness = chosen.effectiveness;
        fail_closed_effectiveness(effectiveness, request.mode)?;

        let step = if !calculated_credit {
            0
        } else {
            reduction_step(effectiveness, request.treatment.completeness)
        };
        total_step = total_step.saturating_add(step);

        let note = if no_reduction {
            "no reduction; effectiveness does not lower residual".into()
        } else if effectiveness == Effectiveness::ExceptionApproved {
            format!("exceptionApproved is governance evidence; step: {step}; residual is not low")
        } else if step == 0 {
            format!("{effectiveness:?} grants no reduction; step: 0")
        } else {
            format!(
                "{effectiveness:?} reduction step: {step} (methodology {CONTROL_EFFECTIVENESS_METHODOLOGY_ID})"
            )
        };
        trace.push(ResidualReductionStep {
            control_id: Some(control_id.clone()),
            effectiveness: Some(format!("{effectiveness:?}").to_ascii_lowercase()),
            step,
            note,
        });
    }

    if no_reduction {
        if ordered.is_empty() {
            trace.push(ResidualReductionStep {
                control_id: None,
                effectiveness: None,
                step: 0,
                note: "no-reduction methodology; residual equals inherent".into(),
            });
        }
        return Ok(CalculatedResidual {
            ordinal: request.inherent.ordinal,
            rating_id: request.inherent.rating_id.clone(),
        });
    }

    let reduced = request.inherent.ordinal.saturating_sub(total_step);
    let ordinal = reduced.max(MIN_RESIDUAL_FLOOR);
    Ok(CalculatedResidual {
        ordinal,
        rating_id: ResidualRiskProjection::rating_for_ordinal(ordinal, &request.inherent),
    })
}

fn conservative_result<'a>(results: &[&'a ControlTestResult]) -> &'a ControlTestResult {
    results
        .iter()
        .copied()
        .min_by_key(|r| effectiveness_rank(r.effectiveness))
        .expect("non-empty result group")
}

fn effectiveness_rank(effectiveness: Effectiveness) -> u8 {
    match effectiveness {
        Effectiveness::NotTested => 0,
        Effectiveness::InsufficientEvidence => 1,
        Effectiveness::StaleEvidence => 2,
        Effectiveness::ManualReviewRequired => 3,
        Effectiveness::Inconclusive => 4,
        Effectiveness::NotApplicable => 5,
        Effectiveness::Ineffective => 6,
        Effectiveness::ExceptionApproved => 7,
        Effectiveness::PartiallyEffective => 8,
        Effectiveness::Effective => 9,
    }
}

fn fail_closed_effectiveness(
    effectiveness: Effectiveness,
    mode: ResidualRiskMode,
) -> Result<(), ResidualRiskError> {
    match effectiveness {
        Effectiveness::NotTested => Err(ResidualRiskError::NotTested),
        Effectiveness::InsufficientEvidence => Err(ResidualRiskError::InsufficientEvidence),
        Effectiveness::StaleEvidence => Err(ResidualRiskError::StaleEvidence),
        Effectiveness::NotApplicable => Err(ResidualRiskError::NotApplicableContradiction),
        Effectiveness::ManualReviewRequired if mode == ResidualRiskMode::Calculated => {
            Err(ResidualRiskError::ManualReviewRequired)
        }
        Effectiveness::Inconclusive if mode == ResidualRiskMode::Calculated => {
            Err(ResidualRiskError::Inconclusive)
        }
        _ => Ok(()),
    }
}

fn reduction_step(effectiveness: Effectiveness, completeness: TreatmentCompleteness) -> u32 {
    match (effectiveness, completeness) {
        (Effectiveness::Effective, TreatmentCompleteness::Complete) => 2,
        (Effectiveness::Effective, TreatmentCompleteness::Partial)
        | (Effectiveness::PartiallyEffective, TreatmentCompleteness::Complete)
        | (Effectiveness::PartiallyEffective, TreatmentCompleteness::Partial) => 1,
        _ => 0,
    }
}

fn require_manual(
    manual: Option<&ManualResidualAssessment>,
) -> Result<&ManualResidualAssessment, ResidualRiskError> {
    let Some(manual) = manual else {
        return Err(ResidualRiskError::MissingManualAssessment);
    };
    if manual.principal.is_none()
        || manual.rationale.trim().is_empty()
        || manual.assessed_at.is_none()
    {
        return Err(ResidualRiskError::MissingManualAssessment);
    }
    Ok(manual)
}

fn require_management(
    manual: Option<&ManualResidualAssessment>,
) -> Result<&ManualResidualAssessment, ResidualRiskError> {
    let Some(manual) = manual else {
        return Err(ResidualRiskError::MissingManagementAssessment);
    };
    if manual.approved_by.is_none()
        || manual.principal.is_none()
        || manual.rationale.trim().is_empty()
        || manual.assessed_at.is_none()
    {
        return Err(ResidualRiskError::MissingManagementAssessment);
    }
    Ok(manual)
}

fn collect_exception_ids(
    request: &ResidualRiskRequest,
    results_by_control: &BTreeMap<ControlId, Vec<&ControlTestResult>>,
) -> Vec<ExceptionId> {
    let mut ids: Vec<ExceptionId> = request.exceptions.iter().map(|e| e.id.clone()).collect();
    ids.sort();
    ids.dedup();
    if ids.is_empty()
        && results_by_control.values().any(|group| {
            group
                .iter()
                .any(|r| r.effectiveness == Effectiveness::ExceptionApproved)
        })
    {
        // Effectiveness-only path still records the variant on the trace.
    }
    ids
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectionIdentity<'a> {
    risk_id: &'a weeping_angel_assurance_ir::RiskId,
    mode: ResidualRiskMode,
    inherent: &'a weeping_angel_assurance_ir::InherentRiskRef,
    treatment: &'a weeping_angel_assurance_ir::TreatmentPlanRef,
    methodology: &'a MethodologyRef,
    relevant_control_ids: &'a [ControlId],
    control_tests: &'a ControlTestSnapshotRef,
    projected_at: &'a chrono::DateTime<chrono::Utc>,
    residual_ordinal: u32,
    residual_rating_id: &'a str,
    reduction_trace: &'a [ResidualReductionStep],
    manual: &'a Option<ManualResidualAssessment>,
    exception_ids: &'a [ExceptionId],
}

fn seal_projection_id(projection: &ResidualRiskProjection) -> ResidualRiskId {
    let identity = ProjectionIdentity {
        risk_id: &projection.risk_id,
        mode: projection.mode,
        inherent: &projection.inherent,
        treatment: &projection.treatment,
        methodology: &projection.methodology,
        relevant_control_ids: &projection.relevant_control_ids,
        control_tests: &projection.control_tests,
        projected_at: &projection.projected_at,
        residual_ordinal: projection.residual_ordinal,
        residual_rating_id: &projection.residual_rating_id,
        reduction_trace: &projection.reduction_trace,
        manual: &projection.manual,
        exception_ids: &projection.exception_ids,
    };
    let digest = canonical_digest(&identity).unwrap_or_else(|_| "undigested".into());
    ResidualRiskId::new(format!("residual:{digest}"))
}
