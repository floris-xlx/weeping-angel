use std::collections::BTreeSet;

use weeping_angel_assurance_ir::AssetId;

#[derive(Debug, Clone, Default)]
pub struct CollectorScope {
    allowed: BTreeSet<AssetId>,
}

impl CollectorScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_asset(mut self, asset: AssetId) -> Self {
        self.allowed.insert(asset);
        self
    }

    pub fn allows(&self, asset: &AssetId) -> bool {
        self.allowed.contains(asset)
    }

    pub fn as_label(&self) -> String {
        self.allowed
            .iter()
            .map(|a| a.as_str().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}
