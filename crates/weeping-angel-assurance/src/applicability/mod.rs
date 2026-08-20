//! Generic Kleene applicability engine over IR rules and assessment scope.
//!
//! Network-free. No framework or provider branches. IR stays declarative.

mod context;
mod evaluator;
mod snapshot;

pub use context::{
    ApplicabilityContext, ContextExtras, FactKey, FactValue, InventoryCompleteness,
    InventoryFamily, build_applicability_context,
};
pub use evaluator::{
    ApplicabilityDecision, ApplicabilityOutcome, ExcludedSubject, PredicateTrace, RationaleEntry,
    UnknownFact, evaluate_applicability,
};
pub use snapshot::{
    APPLICABILITY_SNAPSHOT_SCHEMA, ApplicabilityItemDecision, ApplicabilitySnapshot,
    PackApplicabilityEntry, evaluate_assessment_applicability, pin_compiled_applicability,
};
