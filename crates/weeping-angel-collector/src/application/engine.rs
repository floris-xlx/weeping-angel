use crate::CollectorError;
use crate::application::{CollectorRegistry, EnvelopeFactory, ObservationGate};
use crate::domain::{CollectionBatch, CollectionRequest, CollectorInstance};
use crate::ports::CollectorAdapter;

/// Owns CollectionRequest → Registry → Adapter → ObservationGate → EnvelopeFactory.
pub struct CollectionEngine {
    registry: CollectorRegistry,
    gate: ObservationGate,
    factory: EnvelopeFactory,
}

impl CollectionEngine {
    pub fn new() -> Self {
        Self {
            registry: CollectorRegistry::new(),
            gate: ObservationGate::new(),
            factory: EnvelopeFactory::new(),
        }
    }

    pub fn with_registry(mut self, registry: CollectorRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn collect_registered(
        &self,
        instance: &CollectorInstance,
        request: CollectionRequest,
    ) -> Result<CollectionBatch, CollectorError> {
        let adapter = self.registry.resolve(instance)?;
        self.collect(adapter, instance, request)
    }

    pub fn collect(
        &self,
        adapter: &dyn CollectorAdapter,
        instance: &CollectorInstance,
        request: CollectionRequest,
    ) -> Result<CollectionBatch, CollectorError> {
        let observations = adapter.collect_observations(instance, &request)?;
        self.gate
            .validate(&adapter.descriptor(), &request.scope, &observations)?;
        self.factory
            .seal_batch(instance, &request.scope, adapter, &observations)
    }
}

impl Default for CollectionEngine {
    fn default() -> Self {
        Self::new()
    }
}
