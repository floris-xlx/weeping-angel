//! Incremental cursors. GitHub cursor behavior is a later phase.

#[derive(Debug, Clone, Default)]
pub struct CollectionCursor {
    pub opaque: Option<String>,
}
