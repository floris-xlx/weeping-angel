//! Organizational ISMS scope engine. Not crawl URL membership (`src/engine/scope.rs`).

mod adapter;
mod engine;
mod snapshot;

pub use engine::{
    InScopePopulation, ScopeError, ScopeInputs, SubjectRef, in_scope_population,
    is_definitely_in_scope, resolve_scope, resolve_subject,
};
pub use snapshot::{
    InfluencingRule, InfluencingRuleClass, LineageHop, SCOPE_RESOLUTION_SCHEMA, ScopeDecision,
    ScopeResolution, SubjectScopeDecision,
};
