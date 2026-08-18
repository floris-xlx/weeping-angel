//! Framework-neutral compliance IR (Athena `Statement` analogue).
//!
//! Requirement → Mapping → Canonical Control → Control Test → Evidence Requirement.
//! No provider/SDK types. Control has no ISO-specific fields.

pub mod applicability;
pub mod assessment;
pub mod asset;
pub mod control;
pub mod crosswalk;
pub mod digest;
pub mod evidence;
pub mod exception;
pub mod extension;
pub mod framework;
pub mod id;
pub mod identity;
pub mod implementation;
pub mod mapping;
pub mod privacy;
pub mod requirement;
pub mod risk;
pub mod subject;
pub mod test;
pub mod validation;
pub mod vendor;

pub use applicability::{ApplicabilityPredicate, ApplicabilityRule};
pub use assessment::{
    Assessment, AssessmentDefinition, AssessmentRequests, AssessmentScope, ScopeExclusion,
};
pub use asset::{Asset, AssetKind};
pub use control::{Control, ControlDomain, ControlExpectation};
pub use digest::{
    canonical_digest, typed_canonical_digest, CanonicalDigestError, CanonicalizationVersion,
};
pub use evidence::{
    EvidenceCardinality, EvidenceCollectionKind, EvidenceCriticality, EvidenceRequirement,
    FreshnessRequirement,
};
pub use exception::{Exception, ExceptionStatus};
pub use extension::ExtensionMap;
pub use framework::{ExternalRequirementRef, FrameworkRef};
pub use id::{
    validate_stable_id, AssessmentId, AssetId, AuditProgramId, ControlId, ControlImplementationId,
    ControlTestId, EvidenceRequirementId, EvidenceType, ExceptionId, FrameworkId, FrameworkVersion,
    IdError, IdentityId, MappingId, ProcessingActivityId, RequirementId, RiskId, StableId, VendorId,
    MAX_ID_LEN,
};
pub use identity::{Identity, IdentityKind};
pub use implementation::{ControlImplementation, ImplementationStatus, PrincipalRef};
pub use mapping::{
    Mapping, MappingCompleteness, MappingConfidence, MappingDirection, MappingProvenance,
    MappingRelation, MappingSource, MappingVersionConstraint,
};
pub use privacy::ProcessingActivity;
pub use requirement::{Requirement, RequirementKind};
pub use risk::{Risk, RiskStatus};
pub use subject::{SelectorScope, SubjectKind, SubjectSelector};
pub use test::{PlannedControlTest, PlannedTestKind, TestEvaluationRef, TestFailureSeverity};
pub use validation::{validate_assessment_ir, IrValidationError, ValidateIr};
pub use vendor::Vendor;

/// Explicit schema version on every serialized IR document.
pub const ASSURANCE_IR_SCHEMA: &str = "assurance-ir/v1";
