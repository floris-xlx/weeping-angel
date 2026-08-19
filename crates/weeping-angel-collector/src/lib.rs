//! Collectors emit observations of declared evidence types. They cannot declare compliance.
//!
//! Hexagonal facade: domain / application / ports / adapters. One Cargo package.

pub mod adapters;
pub mod application;
pub mod domain;
pub mod github;
pub mod local;
pub mod ports;

pub use adapters::FixtureCollector;
pub use application::{CollectionEngine, CollectorRegistry, EnvelopeFactory, ObservationGate};
pub use domain::{
    CollectionBatch, CollectionCoverage, CollectionCursor, CollectionDiagnostic, CollectionRequest,
    CollectorCapabilities, CollectorDescriptor, CollectorError, CollectorInstance, CollectorScope,
    CredentialRef, EvidenceCollector, ObservationBatch, ObservationCandidate,
};
pub use github::GitHubCollector;
pub use local::{LocalCollector, ManualEvidence};
pub use ports::CollectorAdapter;
