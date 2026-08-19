//! Adapter re-exports. GitHub sources stay under `src/github/` (github_src).

mod fixture;

pub use crate::github::GitHubCollector;
pub use crate::local::{LocalCollector, ManualEvidence};
pub use fixture::FixtureCollector;
