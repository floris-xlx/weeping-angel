//! Framework-neutral compliance IR (Athena `Statement` analogue).
//!
//! Requirement → Mapping → Canonical Control → Control Test → Evidence Requirement.
//! No provider/SDK types. Control has no ISO-specific fields.

pub mod applicability;
pub mod assessment;
pub mod asset;
pub mod audit;
pub mod continuity;
pub mod control;
pub mod crosswalk;
pub mod decimal;
pub mod digest;
pub mod document;
pub mod event;
pub mod evidence;
pub mod exception;
pub mod extension;
pub mod framework;
pub mod id;
pub mod identity;
pub mod implementation;
pub mod incident;
pub mod isms;
pub mod mapping;
pub mod objectives;
pub mod obligation;
pub mod party;
pub mod privacy;
pub mod registry;
pub mod remediation;
pub mod requirement;
pub mod residual;
pub mod risk;
pub mod risk_candidate;
pub mod risk_methodology;
pub mod risk_promotion;
pub mod risk_scoring;
pub mod risk_treatment;
pub mod subject;
pub mod test;
pub mod validation;
pub mod vendor;

pub use applicability::{ApplicabilityPredicate, ApplicabilityRule};
pub use assessment::{
    Assessment, AssessmentDefinition, AssessmentRequests, AssessmentScope, ScopeExclusion,
};
pub use asset::{Asset, AssetKind};
pub use audit::{
    Audit, AuditConclusion, AuditCriterion, AuditEvidencePin, AuditFinding, AuditFindingKind,
    AuditFindingSeverity, AuditHistoryEvent, AuditObservation, AuditPeriod, AuditProcedure,
    AuditProcedureStatus, AuditProgram, AuditProgramStatus, AuditSample, AuditSampleProposal,
    AuditScheduleEntry, AuditSignOff, AuditStatus, IndependenceConflict, IndependenceRecord,
    NonconformityRef, SampleMethod, compute_proposal_digest, compute_sample_digest,
    flag_independence_conflicts, population_digest, validate_audit_inventories,
};
pub use continuity::{
    AssetRef, BackupExpectation, CadenceStatus, ContinuityDimension, ContinuityExercise,
    ContinuityGap, ContinuityRemediationRef, ContinuityResilienceError,
    ContinuityResilienceProfile, ContinuityResilienceVerdict, CoverageStatus, DependencyKind,
    DimensionStatus, ExerciseIssue, ExerciseKind, ExerciseOutcome, ExerciseResult, FindingStatus,
    ObjectiveStatus, RecoveryObjective, RecoveryProcedureRef, RecoveryProcedureRole, RestoreStatus,
    RiskRef, ServiceCriticality, ServiceDependency, validate_continuity_profiles,
};
pub use control::{Control, ControlDomain, ControlExpectation};
pub use decimal::CanonicalDecimal;
pub use digest::{
    CanonicalDigestError, CanonicalizationVersion, canonical_digest, typed_canonical_digest,
};
pub use document::{
    AcknowledgementCoverage, AcknowledgementRecord, ControlledDocument, DocumentControlError,
    DocumentControlRegistry, DocumentLinkUniverse, DocumentType, DocumentVersion,
    DocumentVersionStatus, InformationClassification, RetentionMetadata,
};
pub use event::{
    EventCauseKind, EventCauseRef, EventSeverity, EventSubjectKind, EventSubjectRef,
    ISMS_EVENT_SCHEMA, IsmsEvent, IsmsEventKind, rfc3339_z,
};
pub use evidence::{
    EvidenceCardinality, EvidenceCollectionKind, EvidenceCriticality, EvidenceRequirement,
    FreshnessRequirement,
};
pub use exception::{Exception, ExceptionStatus};
pub use extension::ExtensionMap;
pub use framework::{ExternalRequirementRef, FrameworkRef};
pub use id::{
    AlertRef, AssessmentId, AssetId, AuditFindingId, AuditId, AuditProgramId, BusinessUnitId,
    ContinuityExerciseId, ContinuityProfileId, ControlId, ControlImplementationId, ControlTestId,
    ControlledDocumentId, DismissalId, EventId, EventRef, EvidenceRequirementId, EvidenceType,
    ExceptionId, FindingRef, FrameworkId, FrameworkVersion, IdError, IdentityId, IncidentId,
    InterestedPartyId, IsmsContextId, IssueId, MAX_ID_LEN, MappingId, ObjectiveId,
    ObjectiveMeasurementId, ObjectiveMetricId, ObjectiveTargetId, ObligationId,
    ObligationMappingId, OrganizationId, ProcessingActivityId, PromotionId, RecoveryObjectiveId,
    RemediationActionId, RemediationId, RemediationRef, RequirementId, RequirementSourceId,
    ResidualRiskId, RiskAcceptanceId, RiskCandidateId, RiskId, RiskMethodologyId, RiskTreatmentId,
    ScopeId, SecurityObjectiveId, SlaPolicyId, StableId, SupplierIssueId, SupplierRequirementId,
    SupplierReviewId, TreatmentActionId, TreatmentPlanId, VendorId, validate_stable_id,
};
pub use identity::{Identity, IdentityKind};
pub use implementation::{
    ControlImplementation, DocumentKind, DocumentRef, ImplementationAutomation,
    ImplementationStatus, PrincipalRef,
};
pub use incident::{
    ControlFailureRef, DetectionSource, ExternalIncidentRef, Incident, IncidentClassification,
    IncidentContainment, IncidentError, IncidentEvent, IncidentEventKind, IncidentKind,
    IncidentSeverity, IncidentStatus, IncidentTimelineEvent, NotificationRecord,
    PostIncidentReview, TimelineKind,
};
pub use isms::{
    BusinessUnit, CadenceInterval, CadenceUnit, ContextIssue, GovernanceCadence, InterestedParty,
    InterestedPartyKind, IsmsContext, IsmsLifecycleStatus, IssueKind, ManagementSystemScope,
    Obligation, Organization, SecurityObjective, explain_isms_context,
    validate_assessment_against_context,
};
pub use mapping::{
    Mapping, MappingCompleteness, MappingConfidence, MappingDirection, MappingProvenance,
    MappingRelation, MappingSource, MappingVersionConstraint,
};
pub use objectives::{
    ComparisonOp, MetricKind, ObjectiveLifecycle, ObjectiveMeasurementSource,
    PopulationCompleteness,
};
pub use privacy::ProcessingActivity;
pub use registry::{
    ImplementationOverlap, current_implementations_for, implementation_by_id, implementations_for,
    overlap_report,
};
pub use remediation::{
    EvidenceOfFixRequirement, ExternalTicketRef, Remediation, RemediationAction,
    RemediationActionState, RemediationError, RemediationEvent, RemediationEventKind,
    RemediationPriority, RemediationSource, RemediationSourceKind, RemediationState, TicketSystem,
    VerificationMode, VerificationPolicy, VerificationState, VerificationStatus, WaiverBinding,
    WaiverKind, treatment_action_inventory, validate_remediation_inventory,
    validate_remediation_slas_at, validate_remediation_waivers_at, validate_remediations_at,
    waiver_in_force,
};
pub use requirement::{Requirement, RequirementKind};
pub use residual::{
    CONTROL_EFFECTIVENESS_METHODOLOGY_ID, ControlTestSnapshotRef, InherentRiskRef,
    InherentRiskSnapshot, MIN_RESIDUAL_FLOOR, ManualResidualAssessment, MethodologyRef,
    NO_REDUCTION_METHODOLOGY_ID, RESIDUAL_METHODOLOGY_V1, ResidualReductionStep, ResidualRiskError,
    ResidualRiskMode, ResidualRiskProjection, TreatmentCompleteness, TreatmentPlanRef,
    TreatmentPlanSnapshot,
};
pub use risk::{
    CiaImpactInputs, ReviewCadence, RiskEvent, RiskEventKind, RiskSource, RiskTransitionError,
};
pub use risk::{Risk, RiskStatus};
pub use risk_candidate::{
    CandidateConfidence, CandidateStatus, CorrelationKey, ObservationIdentity, RiskCandidate,
    ScenarioProposal, ScoreSuggestion, SourceRef, SubjectRef, SuggestedRiskCategory,
};
pub use risk_methodology::{
    AcceptanceThreshold, Combination, DerivedRating, ImpactScale, LikelihoodScale, MatrixCell,
    NumericDomain, RatingBand, RatingLevel, RatingScale, RiskAppetite, RiskMatrix, RiskMethodology,
    RiskMethodologyError, RiskScore, RiskScoreInput, RiskTolerance, ScaleLevel, ScoredRisk,
    ScoringMode, score_risk, validate_risk_methodology,
};
pub use risk_promotion::{DismissalRecord, PromotionRecord};
pub use risk_scoring::{MethodologyValue, RiskScoringError, score_inherent};
pub use risk_treatment::{
    ActionState, RiskAcceptance, RiskTreatmentDecision, TargetResidualRisk, TransferEvidence,
    TreatmentAction, TreatmentApproval, TreatmentError, TreatmentEvent, TreatmentEventKind,
    TreatmentEvidenceExpectation, TreatmentEvidenceKind, TreatmentEvidenceRef, TreatmentPlan,
    TreatmentState, TreatmentStrategy, acceptance_in_force, active_treatment, treatment_required,
    validate_treatment_inventory, validate_treatments_at,
};
pub use subject::{SelectorScope, SubjectKind, SubjectSelector};
pub use test::{PlannedControlTest, PlannedTestKind, TestEvaluationRef, TestFailureSeverity};
pub use validation::{
    IrValidationError, ValidateIr, critical_suppliers, validate_assessment_ir,
    validate_risk_reviews_at, validate_supplier_reviews_at,
};
pub use vendor::{
    SupplierAccess, SupplierAccessGrant, SupplierAccessGrantStatus, SupplierApproval,
    SupplierApprovalDecision, SupplierClassification, SupplierCriticality, SupplierIssue,
    SupplierIssueStatus, SupplierLifecycleStatus, SupplierMonitoringStatus,
    SupplierReassessmentCadence, SupplierRequirementSource, SupplierReview, SupplierReviewKind,
    SupplierReviewSource, SupplierRiskAssessment, SupplierSecurityRequirement, Vendor, VendorEvent,
    VendorEventKind, VendorTransitionError,
};

/// Explicit schema version on every serialized IR document.
pub const ASSURANCE_IR_SCHEMA: &str = "assurance-ir/v1";
