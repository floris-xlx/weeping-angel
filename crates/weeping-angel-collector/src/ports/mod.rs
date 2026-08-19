//! Inbound ports. Adapters implement these; application depends on them.

pub mod adapter;

pub use adapter::CollectorAdapter;
