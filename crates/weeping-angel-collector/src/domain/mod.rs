//! Collector domain types. Framework-blind facts, never control results.

pub mod batch;
pub mod capabilities;
pub mod collector;
pub mod coverage;
pub mod cursor;
pub mod descriptor;
pub mod diagnostic;
pub mod instance;
pub mod observation;
pub mod scope;

pub use batch::{CollectionBatch, CollectionRequest, ObservationBatch};
pub use capabilities::CollectorCapabilities;
pub use collector::{CollectorError, EvidenceCollector};
pub use coverage::CollectionCoverage;
pub use cursor::CollectionCursor;
pub use descriptor::CollectorDescriptor;
pub use diagnostic::CollectionDiagnostic;
pub use instance::{CollectorInstance, CredentialRef};
pub use observation::ObservationCandidate;
pub use scope::CollectorScope;
