use crate::CollectorError;
use crate::domain::{
    CollectionRequest, CollectorDescriptor, CollectorInstance, CollectorScope, ObservationBatch,
};

/// Provider adapter: observations only. Never seals envelopes.
pub trait CollectorAdapter: Send + Sync {
    fn descriptor(&self) -> CollectorDescriptor;

    fn collect_observations(
        &self,
        instance: &CollectorInstance,
        request: &CollectionRequest,
    ) -> Result<ObservationBatch, CollectorError>;

    fn configuration_digest(&self, _scope: &CollectorScope) -> String {
        String::new()
    }
}
