//! Deterministic serialization and domain-separated digests.

use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::ASSURANCE_IR_SCHEMA;

#[derive(Debug, Error)]
pub enum CanonicalDigestError {
    #[error("canonical serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Independent of [`crate::ASSURANCE_IR_SCHEMA`].
pub struct CanonicalizationVersion(&'static str);

impl CanonicalizationVersion {
    pub const CURRENT: Self = Self("canon/v1");

    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

/// SHA-256 hex of deterministic serde JSON (struct field order + BTree maps).
pub fn canonical_digest<T: Serialize>(value: &T) -> Result<String, CanonicalDigestError> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

/// Domain-separated digest: `wa:assurance-ir:<schema>:<type>:` + canonical JSON.
pub fn typed_canonical_digest<T: Serialize>(
    type_name: &str,
    value: &T,
) -> Result<String, CanonicalDigestError> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(
        format!("wa:assurance-ir:{ASSURANCE_IR_SCHEMA}:{type_name}:").as_bytes(),
    );
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}
