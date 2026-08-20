//! CollectionEngine: ObservationGate validates, then EnvelopeFactory seals.

use crate::CollectorError;
use crate::application::{CollectorRegistry, EnvelopeFactory, ObservationGate};
use crate::domain::{CollectionBatch, CollectionRequest, CollectorInstance, CollectorScope};
use crate::ports::CollectorAdapter;
use weeping_angel_evidence::EvidenceEnvelope;

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

/// Scheduler/façade envelope collection. Only CollectionEngine seals (DUP-015).
pub(crate) fn collect_envelopes(
    adapter: &dyn CollectorAdapter,
    instance: &CollectorInstance,
    scope: &CollectorScope,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    Ok(CollectionEngine::new()
        .collect(
            adapter,
            instance,
            CollectionRequest {
                scope: scope.clone(),
            },
        )?
        .envelopes)
}

impl Default for CollectionEngine {
    fn default() -> Self {
        Self::new()
    }
}
