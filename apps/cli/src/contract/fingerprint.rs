//! Stable finding fingerprints and derived ids (codex-security/v1).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{Result, bail};
use sha2::{Digest, Sha256};

use crate::contract::types::{FindingsDocument, Fingerprints, ManifestDocument, SemanticFinding};

pub const FINGERPRINT_ALGORITHM: &str = "codex-security/v1";

pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn sha256_text(value: &str) -> String {
    sha256_bytes(value.as_bytes())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 64];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// `codex-security/v1:sha256:<hex>` from targetId + ruleId + anchor + instance.
pub fn derive_fingerprint(
    target_id: &str,
    rule_id: &str,
    anchor: &str,
    instance: Option<&str>,
) -> Result<String> {
    validate_slug(rule_id, "ruleId")?;
    validate_slug(anchor, "identity.anchor")?;
    let instance = instance.unwrap_or("");
    if !instance.is_empty() {
        validate_slug(instance, "identity.instance")?;
    }
    let material = [FINGERPRINT_ALGORITHM, target_id, rule_id, anchor, instance].join("\0");
    Ok(format!(
        "{FINGERPRINT_ALGORITHM}:sha256:{}",
        sha256_text(&material)
    ))
}

pub fn stable_id(prefix: &str, parts: &[&str]) -> String {
    let material = parts.join("\0");
    let hex = sha256_text(&material);
    format!("{prefix}_{}", &hex[..24])
}

pub fn finding_id_from_fingerprint(fingerprint: &str) -> String {
    stable_id("csf", &[fingerprint])
}

pub fn occurrence_id_from(scan_id: &str, fingerprint: &str) -> String {
    stable_id("occ", &[scan_id, fingerprint])
}

fn validate_slug(value: &str, field: &str) -> Result<()> {
    let ok = !value.is_empty()
        && value.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '/' | '-')
        });
    let starts_ok = value
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    if !ok || !starts_ok {
        bail!("{field}: expected stable lowercase slug, got `{value}`");
    }
    Ok(())
}

/// Replace draft findingId / occurrenceId / fingerprints with derived values.
pub fn populate_finding_identities(
    manifest: &ManifestDocument,
    findings: &mut FindingsDocument,
) -> Result<()> {
    let scan_id = &manifest.scan.id;
    let target_id = &manifest.scan.target.target_id;
    if findings.scan_id != *scan_id {
        bail!(
            "findings.scanId `{}` must match manifest scan id `{}`",
            findings.scan_id,
            scan_id
        );
    }

    let mut seen_fid = std::collections::HashSet::new();
    let mut seen_oid = std::collections::HashSet::new();

    for finding in &mut findings.findings {
        apply_identity(target_id, scan_id, finding)?;
        if !seen_fid.insert(finding.finding_id.clone()) {
            bail!("duplicate findingId {}", finding.finding_id);
        }
        if !seen_oid.insert(finding.occurrence_id.clone()) {
            bail!("duplicate occurrenceId {}", finding.occurrence_id);
        }
    }
    Ok(())
}

fn apply_identity(target_id: &str, scan_id: &str, finding: &mut SemanticFinding) -> Result<()> {
    let fingerprint = derive_fingerprint(
        target_id,
        &finding.rule_id,
        &finding.identity.anchor,
        finding.identity.instance.as_deref(),
    )?;
    finding.finding_id = finding_id_from_fingerprint(&fingerprint);
    finding.occurrence_id = occurrence_id_from(scan_id, &fingerprint);
    finding.fingerprints = Fingerprints {
        algorithm: FINGERPRINT_ALGORITHM.into(),
        primary: fingerprint,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_matches_codex_algorithm() {
        // material = algo\0target\0rule\0anchor\0instance
        let fp = derive_fingerprint(
            "target_sha256_example",
            "path-traversal.archive-extraction",
            "archive-entry-write-without-containment",
            None,
        )
        .unwrap();
        assert!(fp.starts_with("codex-security/v1:sha256:"));
        assert_eq!(fp.len(), "codex-security/v1:sha256:".len() + 64);

        let csf = finding_id_from_fingerprint(&fp);
        assert!(csf.starts_with("csf_"));
        assert_eq!(csf.len(), 4 + 24);
    }

    #[test]
    fn example_finding_id_recomputes() {
        // From examples/completed-scan — recompute with same inputs as fixture.
        let fp = derive_fingerprint(
            "target_sha256_example",
            "path-traversal.archive-extraction",
            "archive-entry-write-without-containment",
            None,
        )
        .unwrap();
        assert_eq!(
            fp,
            "codex-security/v1:sha256:990a4a6a2ec18440dd47eac4d7256c0ee2c02db1b43104720cab3cbe9db706ca"
        );
        assert_eq!(
            finding_id_from_fingerprint(&fp),
            "csf_852f90d6e1177502ff113d4a"
        );
        assert_eq!(
            occurrence_id_from("scan_example_001", &fp),
            "occ_e79cb19591e696572a1c22be"
        );
    }
}
