//! IR-shaped catalog projection consumed by pack compile.
//!
//! Not a second catalog parser. The canonical-catalog crate is the only TOML
//! loader; this type is the in-memory adapter the framework crate accepts.

use serde::{Deserialize, Serialize};

use crate::{Control, PlannedControlTest};

/// Controls and planned tests (including lossless expression JSON) projected
/// from `CanonicalCatalog`. Pack load resolves `control.*` mappings against this
/// view and must not walk catalog files itself.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProjection {
    pub digest: String,
    pub controls: Vec<Control>,
    pub tests: Vec<PlannedControlTest>,
}

impl CatalogProjection {
    pub fn control(&self, id: &str) -> Option<&Control> {
        self.controls.iter().find(|c| c.id().as_str() == id)
    }

    pub fn tests_for(&self, control_id: &str) -> impl Iterator<Item = &PlannedControlTest> {
        self.tests
            .iter()
            .filter(move |t| t.control_id.as_str() == control_id)
    }
}

/// Loader installed by `weeping-angel-canonical-catalog` when that crate is linked.
pub struct WorkspaceCatalogLoader(pub fn() -> Option<CatalogProjection>);

inventory::collect!(WorkspaceCatalogLoader);

/// Workspace catalog projection, if the catalog crate is linked and the tree exists.
pub fn workspace_catalog_projection() -> Option<CatalogProjection> {
    for loader in inventory::iter::<WorkspaceCatalogLoader> {
        if let Some(projection) = (loader.0)() {
            return Some(projection);
        }
    }
    None
}
