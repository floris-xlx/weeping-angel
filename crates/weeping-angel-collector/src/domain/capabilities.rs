use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectorCapabilities {
    pub incremental: bool,
    pub pagination: bool,
    pub historical: bool,
    pub point_in_time: bool,
    pub event_driven: bool,
    pub sensitive_artifacts: bool,
    pub offline: bool,
    pub worker_safe: bool,
}
