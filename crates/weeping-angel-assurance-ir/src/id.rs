//! Validated, deterministic typed identifiers. No random v4 identities.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum persisted identity length in UTF-8 bytes.
pub const MAX_ID_LEN: usize = 256;

/// Shared string view for persisted IR identities.
pub trait StableId {
    fn as_str(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IdError {
    #[error("identity is empty")]
    Empty,
    #[error("identity exceeds {MAX_ID_LEN} bytes")]
    TooLong,
    #[error("identity contains an invalid character")]
    InvalidCharacter,
    #[error("identity namespace is invalid")]
    InvalidNamespace,
}

pub fn validate_stable_id(raw: &str) -> Result<(), IdError> {
    if raw.is_empty() || raw.chars().all(char::is_whitespace) {
        return Err(IdError::Empty);
    }
    if raw.len() > MAX_ID_LEN {
        return Err(IdError::TooLong);
    }
    if raw.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(IdError::InvalidCharacter);
    }
    if !raw
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':' | '/'))
    {
        return Err(IdError::InvalidCharacter);
    }
    if looks_like_uuid_v4(raw) {
        return Err(IdError::InvalidCharacter);
    }
    Ok(())
}

fn looks_like_uuid_v4(raw: &str) -> bool {
    let parts: Vec<&str> = raw.split('-').collect();
    parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[2].starts_with('4')
        && parts[3].len() == 4
        && parts[4].len() == 12
        && raw.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn try_new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                validate_stable_id(&value)?;
                Ok(Self(value))
            }

            pub fn new(value: impl Into<String>) -> Self {
                Self::try_new(value).expect("invalid stable id")
            }

            #[allow(dead_code)]
            pub(crate) fn new_unchecked(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl StableId for $name {
            fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = String::deserialize(deserializer)?;
                Self::try_new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

typed_id!(FrameworkId);
typed_id!(FrameworkVersion);
typed_id!(RequirementId);
typed_id!(ControlId);
typed_id!(ControlImplementationId);
typed_id!(ControlTestId);
typed_id!(AssetId);
typed_id!(IdentityId);
typed_id!(VendorId);
typed_id!(ProcessingActivityId);
typed_id!(EvidenceRequirementId);
typed_id!(RiskId);
typed_id!(ExceptionId);
typed_id!(AssessmentId);
typed_id!(AuditProgramId);
typed_id!(MappingId);

/// Evidence kind advertised by collectors (not a framework name).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EvidenceType(String);

impl EvidenceType {
    pub fn try_new(value: impl Into<String>) -> Result<Self, IdError> {
        let value = value.into();
        validate_stable_id(&value)?;
        Ok(Self(value))
    }

    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("invalid evidence type")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl StableId for EvidenceType {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EvidenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for EvidenceType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}
