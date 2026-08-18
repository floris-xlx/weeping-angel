//! Namespaced extensions. Canonical semantics must not depend on these alone.

use std::collections::BTreeMap;

use serde_json::Value;

pub type ExtensionMap = BTreeMap<String, Value>;

const ALLOWED_PREFIXES: &[&str] = &["wa", "iso27001", "gdpr", "soc2", "nis2", "dora", "user"];

pub fn extension_key_is_well_formed(key: &str) -> bool {
    let Some((prefix, rest)) = key.split_once('.') else {
        return false;
    };
    !prefix.is_empty() && !rest.is_empty()
}

pub fn extension_prefix_is_reserved(key: &str) -> bool {
    key.split_once('.')
        .is_some_and(|(prefix, _)| ALLOWED_PREFIXES.contains(&prefix))
}

pub fn extensions_override_canonical(key: &str) -> bool {
    matches!(
        key,
        "id" | "schemaVersion"
            | "title"
            | "description"
            | "frameworkId"
            | "frameworkVersion"
            | "owner"
            | "implemented"
    )
}
