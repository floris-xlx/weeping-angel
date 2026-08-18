//! GitHub evidence collector. Emits canonical facts; never framework status.

mod branches;
mod client;
mod collaborators;
mod descriptor;
mod error;
mod normalize;
mod protection;
mod repositories;
mod rulesets;
mod security;
mod workflows;

use std::thread;
use std::time::Duration;

use chrono::Utc;
use serde_json::Value;

use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::{
    EvidenceEnvelope, EvidenceObservation, EvidenceProvenance, EvidenceType, redact,
};

use crate::{
    CollectionBatch, CollectionRequest, CollectorDescriptor, CollectorError, CollectorScope,
    EvidenceCollector,
};

pub use client::GitHubClient;
pub use descriptor::GITHUB_EVIDENCE_TYPES;
pub use error::GitHubError;

/// First production collector. Provider identity lives in provenance, not evidence type.
pub struct GitHubCollector {
    client: GitHubClient,
    version: String,
}

impl GitHubCollector {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: GitHubClient::new(token),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    pub fn with_client(client: GitHubClient) -> Self {
        Self {
            client,
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    fn handle_status(&self, status: u16, body: &str) -> Result<Value, CollectorError> {
        match status {
            401 => Err(CollectorError::PermissionDenied {
                detail: redact("unauthorized; check Authorization header"),
            }),
            403 => Err(CollectorError::PermissionDenied {
                detail: format!(
                    "403 permission denied (not a false observation): {}",
                    redact(body)
                ),
            }),
            404 => Err(CollectorError::InsufficientEvidence {
                detail: "resource not visible; do not infer false".into(),
            }),
            429 => Err(CollectorError::InsufficientEvidence {
                detail: "rate limited; Retry-After honored by client".into(),
            }),
            200 | 201 => {
                serde_json::from_str(body).map_err(|e| CollectorError::InsufficientEvidence {
                    detail: format!("normalize failed: {e}"),
                })
            }
            other => Err(CollectorError::InsufficientEvidence {
                detail: format!("unexpected status {other}"),
            }),
        }
    }

    fn collect_repo(
        &self,
        owner: &str,
        name: &str,
        scope: &CollectorScope,
    ) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        let asset = AssetId::new(format!("repo:{owner}/{name}"));
        if !scope.allows(&asset) {
            return Err(CollectorError::OutOfScope {
                asset: asset.to_string(),
            });
        }
        let mut envelopes = Vec::new();
        let collected_at = Utc::now();
        let prov = |asset: AssetId| EvidenceProvenance {
            collector_id: "collector.github".into(),
            collected_at,
            scope: scope.as_label(),
            asset,
        };

        match self.client.get(&format!("/repos/{owner}/{name}")) {
            Ok((status, body, _retry_after)) => {
                if status == 403 {
                    return Err(CollectorError::PermissionDenied {
                        detail: "403 on repository; InsufficientEvidence, not false".into(),
                    });
                }
                let repo = self.handle_status(status, &body)?;
                envelopes.extend(normalize::repository_facts(
                    &repo,
                    &asset,
                    collected_at,
                    scope,
                )?);
            }
            Err(err) => {
                return Err(CollectorError::InsufficientEvidence {
                    detail: redact(&err.to_string()),
                });
            }
        }

        if let Ok((status, body, _)) = self.client.get(&format!(
            "/repos/{owner}/{name}/branches/{}/protection",
            "main"
        )) {
            match status {
                403 => {
                    // PermissionDenied: a 403 is not a boolean observation.
                    return Err(CollectorError::PermissionDenied {
                        detail: "403 reading branch protection; InsufficientEvidence".into(),
                    });
                }
                404 => {
                    let obs =
                        EvidenceObservation::new(EvidenceType::new("source.branch.protection"))
                            .with_fact("enabled", "false")
                            .with_narrative("default branch has no protection rule");
                    envelopes.push(EvidenceEnvelope::seal(obs, prov(asset.clone()))?);
                }
                200 => {
                    let json = self.handle_status(status, &body)?;
                    envelopes.extend(protection::from_protection_json(
                        &json,
                        &asset,
                        collected_at,
                        scope,
                    )?);
                }
                _ => {
                    let _ = self.handle_status(status, &body)?;
                }
            }
        }

        let _ = (
            branches::MODULE,
            collaborators::MODULE,
            repositories::MODULE,
            rulesets::MODULE,
            security::MODULE,
            workflows::MODULE,
        );

        Ok(envelopes)
    }
}

impl EvidenceCollector for GitHubCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        descriptor::descriptor(&self.version)
    }

    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        let mut out = Vec::new();
        for label in scope.as_label().split(',') {
            let label = label.trim();
            if label.is_empty() {
                continue;
            }
            let repo = label.strip_prefix("repo:").unwrap_or(label);
            let Some((owner, name)) = repo.split_once('/') else {
                return Err(CollectorError::OutOfScope {
                    asset: label.to_string(),
                });
            };
            match self.collect_repo(owner, name, scope) {
                Ok(mut envs) => out.append(&mut envs),
                Err(CollectorError::PermissionDenied { detail }) => {
                    return Err(CollectorError::PermissionDenied { detail });
                }
                Err(other) => return Err(other),
            }
        }
        Ok(out)
    }
}

impl GitHubCollector {
    pub fn collect_batch(
        &self,
        request: CollectionRequest,
    ) -> Result<CollectionBatch, CollectorError> {
        let envelopes = self.collect(&request.scope)?;
        Ok(CollectionBatch {
            run: weeping_angel_evidence::CollectionRun::new("collector.github", &self.version),
            envelopes,
            errors: Vec::new(),
        })
    }

    pub fn backoff(attempt: u32, retry_after: Option<Duration>) -> Duration {
        if let Some(after) = retry_after {
            return after;
        }
        let exp = 1u64.checked_shl(attempt.min(5)).unwrap_or(32);
        Duration::from_secs(exp.min(32))
    }

    pub fn sleep_retry_after(retry_after: Duration) {
        thread::sleep(retry_after.min(Duration::from_secs(32)));
    }
}
