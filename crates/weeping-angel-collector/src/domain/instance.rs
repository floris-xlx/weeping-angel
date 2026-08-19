use std::collections::BTreeMap;

/// Opaque handle to a secret store. Never a token, PAT, or password.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CredentialRef {
    pub id: String,
}

impl CredentialRef {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// One deployed collector instance. Distinct from collector type (`collector_id`).
#[derive(Debug, Clone)]
pub struct CollectorInstance {
    pub id: String,
    pub collector_id: String,
    pub configuration: BTreeMap<String, String>,
    pub credential_ref: CredentialRef,
}

impl CollectorInstance {
    pub fn new(
        id: impl Into<String>,
        collector_id: impl Into<String>,
        credential_ref: CredentialRef,
    ) -> Self {
        Self {
            id: id.into(),
            collector_id: collector_id.into(),
            configuration: BTreeMap::new(),
            credential_ref,
        }
    }
}
