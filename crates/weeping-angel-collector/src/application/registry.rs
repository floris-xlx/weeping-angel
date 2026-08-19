use std::collections::BTreeMap;
use std::sync::Arc;

use crate::CollectorError;
use crate::domain::CollectorInstance;
use crate::ports::CollectorAdapter;

/// Resolves collector type / instance to an adapter.
#[derive(Clone, Default)]
pub struct CollectorRegistry {
    adapters: BTreeMap<String, Arc<dyn CollectorAdapter>>,
}

impl CollectorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        collector_type: impl Into<String>,
        adapter: Arc<dyn CollectorAdapter>,
    ) {
        self.adapters.insert(collector_type.into(), adapter);
    }

    pub fn resolve(
        &self,
        instance: &CollectorInstance,
    ) -> Result<&dyn CollectorAdapter, CollectorError> {
        self.adapters
            .get(&instance.collector_id)
            .map(|a| a.as_ref())
            .ok_or_else(|| CollectorError::InsufficientEvidence {
                detail: format!(
                    "no adapter registered for collector type {}",
                    instance.collector_id
                ),
            })
    }
}
