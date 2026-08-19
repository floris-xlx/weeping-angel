/// Internal coverage notes. Public `CollectionBatch` is unchanged this increment.

#[derive(Debug, Clone, Default)]
pub struct CollectionCoverage {
    pub hole: bool,
    pub strict_scope: bool,
}
