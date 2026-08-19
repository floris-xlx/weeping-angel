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

use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use serde_json::Value;
use weeping_angel_assurance_ir::{AssetId, canonical_digest};
use weeping_angel_evidence::{CollectionRun, EvidenceEnvelope, EvidenceValue};

use crate::{
    CollectionBatch, CollectionRequest, CollectorDescriptor, CollectorError, CollectorScope,
    EvidenceCollector,
};

use branches::protection_path;
use client::ClientError;
use error::sanitize_diagnostic;
use normalize::{EmitCtx, default_branch_of, emit, repo_subject_id};

pub use client::GitHubClient;
pub use descriptor::GITHUB_EVIDENCE_TYPES;
pub use error::GitHubError;

enum Fetch {
    Json(Value),
    Empty,
    Absent,
    Denied { status: u16, body: String },
    Failed { status: u16, body: String },
    RateLimited,
    Missing,
}

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

    fn fetch(&self, path: &str) -> Fetch {
        match self.client.get_response(path) {
            Ok(resp) => match resp.status {
                200 | 201 => match serde_json::from_str(&resp.body) {
                    Ok(v) => Fetch::Json(v),
                    Err(e) => Fetch::Failed {
                        status: resp.status,
                        body: format!("normalize failed: {e}"),
                    },
                },
                204 => Fetch::Empty,
                401 | 403 => Fetch::Denied {
                    status: resp.status,
                    body: resp.body,
                },
                404 => Fetch::Absent,
                429 => Fetch::RateLimited,
                other => Fetch::Failed {
                    status: other,
                    body: resp.body,
                },
            },
            Err(ClientError::Transport(_)) => Fetch::Missing,
        }
    }

    fn collect_inner(&self, scope: &CollectorScope) -> (Vec<EvidenceEnvelope>, Vec<String>, bool) {
        let parsed = parse_scope(scope);
        let mut envelopes = Vec::new();
        let mut errors = Vec::new();
        let mut hole = false;
        let collected_at = Utc::now();

        if parsed.orgs.is_empty() && parsed.repos.is_empty() {
            if parsed.unknown.is_empty() {
                errors.push("OutOfScope: empty collection scope".into());
            } else {
                for u in &parsed.unknown {
                    errors.push(format!("OutOfScope: {u}"));
                }
            }
            return (envelopes, errors, true);
        }
        for u in &parsed.unknown {
            errors.push(format!("OutOfScope: {u}"));
            hole = true;
        }

        let mut listed: BTreeMap<String, Value> = BTreeMap::new();
        let mut inventory_authoritative = !parsed.orgs.is_empty();

        for org in &parsed.orgs {
            let walk = repositories::list_org_repos(&self.client, org);
            if let Some(err) = walk.error {
                errors.push(sanitize_diagnostic(&err));
                inventory_authoritative = false;
                hole = true;
            }
            if !walk.complete {
                inventory_authoritative = false;
                hole = true;
            }
            for item in walk.items {
                if let Some(repo) = repositories::parse_listed_repo(&item, org) {
                    listed.insert(format!("{}/{}", repo.owner, repo.name), repo.json);
                }
            }
        }

        for (owner, name) in &parsed.repos {
            listed
                .entry(format!("{owner}/{name}"))
                .or_insert(Value::Null);
        }

        let only_explicit = parsed.orgs.is_empty();
        if only_explicit {
            inventory_authoritative = true;
        }

        for (full, listed_json) in &listed {
            let Some((owner, name)) = full.split_once('/') else {
                continue;
            };
            let archived_hint = listed_json
                .get("archived")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if parsed.exclude_archived && archived_hint {
                continue;
            }

            let asset = AssetId::new(repo_subject_id(owner, name));
            let ctx = EmitCtx {
                collected_at,
                scope,
                asset: asset.clone(),
            };

            let repo_json = match self.fetch(&format!("/repos/{owner}/{name}")) {
                Fetch::Json(v) => v,
                Fetch::Denied { status, body } => {
                    hole = true;
                    errors.push(denied_msg(status, "repository", owner, name, &body));
                    if listed_json.is_null() {
                        continue;
                    }
                    listed_json.clone()
                }
                Fetch::Failed { status, body } => {
                    hole = true;
                    errors.push(failed_msg(status, "repository", owner, name, &body));
                    if listed_json.is_null() {
                        continue;
                    }
                    listed_json.clone()
                }
                Fetch::RateLimited => {
                    hole = true;
                    errors.push(format!("429 rate limited on repository {owner}/{name}"));
                    if listed_json.is_null() {
                        continue;
                    }
                    listed_json.clone()
                }
                Fetch::Absent => {
                    hole = true;
                    errors.push(format!(
                        "insufficient evidence: repository {owner}/{name} not visible"
                    ));
                    continue;
                }
                Fetch::Missing | Fetch::Empty => {
                    if listed_json.is_null() {
                        hole = true;
                        errors.push(format!(
                            "insufficient evidence: no repository payload for {owner}/{name}"
                        ));
                        continue;
                    }
                    listed_json.clone()
                }
            };

            let archived = repo_json
                .get("archived")
                .and_then(Value::as_bool)
                .unwrap_or(archived_hint);
            if parsed.exclude_archived && archived {
                continue;
            }

            match normalize::repository_envelopes(&repo_json, owner, name, true, &ctx) {
                Ok(mut envs) => envelopes.append(&mut envs),
                Err(e) => {
                    hole = true;
                    errors.push(sanitize_diagnostic(&e.to_string()));
                }
            }

            match security::from_repo_security(&repo_json, owner, name, &ctx) {
                Ok(mut envs) => envelopes.append(&mut envs),
                Err(e) => errors.push(sanitize_diagnostic(&e.to_string())),
            }

            if let Some(branch) = default_branch_of(&repo_json) {
                match self.collect_protection(owner, name, branch, &ctx, &mut errors) {
                    Ok(mut envs) => envelopes.append(&mut envs),
                    Err(()) => hole = true,
                }
            } else {
                hole = true;
                errors.push(format!(
                    "insufficient evidence: default_branch missing on {owner}/{name}"
                ));
            }

            self.collect_optional_repo(owner, name, &ctx, &mut envelopes, &mut errors, &mut hole);
        }

        if !parsed.orgs.is_empty() {
            if inventory_authoritative && !errors.iter().any(|e| e.contains("listing")) {
                if let Ok(env) = emit(
                    "inventory.complete",
                    vec![
                        ("authoritative", EvidenceValue::from_bool(true)),
                        ("kind", EvidenceValue::string("repository")),
                    ],
                    "org repository inventory pagination finished",
                    &EmitCtx {
                        collected_at,
                        scope,
                        asset: AssetId::new(format!("org:{}", parsed.orgs[0])),
                    },
                ) {
                    envelopes.push(env);
                }
            } else if let Ok(env) = emit(
                "inventory.complete",
                vec![
                    ("authoritative", EvidenceValue::from_bool(false)),
                    ("kind", EvidenceValue::string("repository")),
                ],
                "org repository inventory is partial",
                &EmitCtx {
                    collected_at,
                    scope,
                    asset: AssetId::new(format!("org:{}", parsed.orgs[0])),
                },
            ) {
                envelopes.push(env);
            }
        } else if only_explicit && !hole {
            if let Ok(env) = emit(
                "inventory.complete",
                vec![
                    ("authoritative", EvidenceValue::from_bool(true)),
                    ("kind", EvidenceValue::string("repository")),
                ],
                "explicit repository allow-list collected",
                &EmitCtx {
                    collected_at,
                    scope,
                    asset: AssetId::new("scope:explicit"),
                },
            ) {
                envelopes.push(env);
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

        (envelopes, errors, hole)
    }

    fn collect_protection(
        &self,
        owner: &str,
        name: &str,
        default_branch: &str,
        ctx: &EmitCtx<'_>,
        errors: &mut Vec<String>,
    ) -> Result<Vec<EvidenceEnvelope>, ()> {
        let path = protection_path(owner, name, default_branch);
        match self.fetch(&path) {
            Fetch::Json(json) if looks_like_protection(&json) => {
                protection::from_protection_json(&json, owner, name, ctx).map_err(|e| {
                    errors.push(sanitize_diagnostic(&e.to_string()));
                })
            }
            Fetch::Json(_) => Ok(Vec::new()),
            Fetch::Absent => {
                // 404 on the protection resource is observed absence.
                match self.fetch(&format!("/repos/{owner}/{name}/rulesets")) {
                    Fetch::Json(rules) => {
                        let extra = rulesets::from_rulesets_json(&rules, owner, name, ctx)
                            .unwrap_or_default();
                        if extra.iter().any(|e| {
                            e.observation().evidence_type().as_str()
                                == "evidence.repository.branch-protection"
                        }) {
                            return Ok(extra);
                        }
                        let mut out = protection::unprotected(owner, name, ctx).map_err(|e| {
                            errors.push(sanitize_diagnostic(&e.to_string()));
                        })?;
                        out.extend(extra);
                        Ok(out)
                    }
                    Fetch::Denied { status, body } => {
                        errors.push(denied_msg(status, "rulesets", owner, name, &body));
                        protection::unprotected(owner, name, ctx).map_err(|e| {
                            errors.push(sanitize_diagnostic(&e.to_string()));
                        })
                    }
                    Fetch::Missing | Fetch::Empty | Fetch::Absent => {
                        protection::unprotected(owner, name, ctx).map_err(|e| {
                            errors.push(sanitize_diagnostic(&e.to_string()));
                        })
                    }
                    Fetch::Failed { status, body } => {
                        errors.push(failed_msg(status, "rulesets", owner, name, &body));
                        protection::unprotected(owner, name, ctx).map_err(|e| {
                            errors.push(sanitize_diagnostic(&e.to_string()));
                        })
                    }
                    Fetch::RateLimited => {
                        errors.push(format!("429 rate limited on rulesets {owner}/{name}"));
                        Err(())
                    }
                }
            }
            Fetch::Denied { status, body } => {
                errors.push(denied_msg(status, "branch protection", owner, name, &body));
                Err(())
            }
            Fetch::RateLimited => {
                errors.push(format!(
                    "429 rate limited on branch protection {owner}/{name}"
                ));
                Err(())
            }
            Fetch::Failed { status, body } => {
                errors.push(failed_msg(status, "branch protection", owner, name, &body));
                Err(())
            }
            Fetch::Missing | Fetch::Empty => Ok(Vec::new()),
        }
    }

    fn collect_optional_repo(
        &self,
        owner: &str,
        name: &str,
        ctx: &EmitCtx<'_>,
        envelopes: &mut Vec<EvidenceEnvelope>,
        errors: &mut Vec<String>,
        hole: &mut bool,
    ) {
        let subject = repo_subject_id(owner, name);
        match self.fetch(&format!(
            "/repos/{owner}/{name}/contents/.github/CODEOWNERS"
        )) {
            Fetch::Json(json) if looks_like_contents(&json, "CODEOWNERS") => {
                if let Ok(env) = emit(
                    "evidence.repository.review-ownership",
                    vec![
                        ("subject_id", EvidenceValue::string(&subject)),
                        ("ownership_defined", EvidenceValue::from_bool(true)),
                    ],
                    "CODEOWNERS presence observation",
                    ctx,
                ) {
                    if !envelopes.iter().any(|e| {
                        e.observation().evidence_type().as_str()
                            == "evidence.repository.review-ownership"
                            && e.observation().fact("subject_id") == Some(subject.as_str())
                    }) {
                        envelopes.push(env);
                    }
                }
            }
            Fetch::Absent => match self.fetch(&format!("/repos/{owner}/{name}/contents/CODEOWNERS"))
            {
                Fetch::Json(json) if looks_like_contents(&json, "CODEOWNERS") => {
                    if let Ok(env) = emit(
                        "evidence.repository.review-ownership",
                        vec![
                            ("subject_id", EvidenceValue::string(&subject)),
                            ("ownership_defined", EvidenceValue::from_bool(true)),
                        ],
                        "CODEOWNERS presence observation",
                        ctx,
                    ) {
                        envelopes.push(env);
                    }
                }
                Fetch::Absent => {
                    if let Ok(env) = emit(
                        "evidence.repository.review-ownership",
                        vec![
                            ("subject_id", EvidenceValue::string(&subject)),
                            ("ownership_defined", EvidenceValue::from_bool(false)),
                        ],
                        "CODEOWNERS absent observation",
                        ctx,
                    ) {
                        if !envelopes.iter().any(|e| {
                            e.observation().evidence_type().as_str()
                                == "evidence.repository.review-ownership"
                        }) {
                            envelopes.push(env);
                        }
                    }
                }
                _ => {}
            },
            Fetch::Json(_) | Fetch::Empty => {}
            Fetch::Denied { status, body } => {
                *hole = true;
                errors.push(denied_msg(status, "CODEOWNERS", owner, name, &body));
            }
            Fetch::Failed { status, body } => {
                *hole = true;
                errors.push(failed_msg(status, "CODEOWNERS", owner, name, &body));
            }
            Fetch::RateLimited => {
                *hole = true;
                errors.push(format!("429 rate limited on CODEOWNERS {owner}/{name}"));
            }
            Fetch::Missing => {}
        }

        match self.fetch(&format!(
            "/repos/{owner}/{name}/actions/permissions/workflow"
        )) {
            Fetch::Json(json) if json.get("default_workflow_permissions").is_some() => {
                match workflows::from_workflow_permissions(&json, owner, name, ctx) {
                    Ok(mut envs) => envelopes.append(&mut envs),
                    Err(e) => errors.push(sanitize_diagnostic(&e.to_string())),
                }
            }
            Fetch::Json(_) => {}
            Fetch::Denied { status, body } => {
                *hole = true;
                errors.push(denied_msg(
                    status,
                    "workflow permissions",
                    owner,
                    name,
                    &body,
                ));
            }
            Fetch::Failed { status, body } => {
                *hole = true;
                errors.push(failed_msg(
                    status,
                    "workflow permissions",
                    owner,
                    name,
                    &body,
                ));
            }
            Fetch::RateLimited => {
                *hole = true;
                errors.push(format!(
                    "429 rate limited on workflow permissions {owner}/{name}"
                ));
            }
            Fetch::Absent | Fetch::Empty | Fetch::Missing => {}
        }

        match self.fetch(&format!("/repos/{owner}/{name}/environments")) {
            Fetch::Json(json)
                if json.get("environments").is_some() || json.as_array().is_some() =>
            {
                match workflows::from_environments(&json, owner, name, ctx) {
                    Ok(mut envs) => envelopes.append(&mut envs),
                    Err(e) => errors.push(sanitize_diagnostic(&e.to_string())),
                }
            }
            Fetch::Json(_) => {}
            Fetch::Denied { status, body } => {
                *hole = true;
                errors.push(denied_msg(status, "environments", owner, name, &body));
            }
            Fetch::Failed { status, body } => {
                *hole = true;
                errors.push(failed_msg(status, "environments", owner, name, &body));
            }
            Fetch::RateLimited => {
                *hole = true;
                errors.push(format!("429 rate limited on environments {owner}/{name}"));
            }
            Fetch::Absent | Fetch::Empty | Fetch::Missing | Fetch::Json(_) => {}
        }

        match self.fetch(&format!("/repos/{owner}/{name}/collaborators")) {
            Fetch::Json(json) if json.as_array().is_some() => {
                let people = json.as_array().cloned().unwrap_or_default();
                match collaborators::from_collaborators(&people, ctx) {
                    Ok(mut envs) => envelopes.append(&mut envs),
                    Err(e) => errors.push(sanitize_diagnostic(&e.to_string())),
                }
            }
            Fetch::Json(_) => {}
            Fetch::Denied { status, body } => {
                *hole = true;
                errors.push(denied_msg(status, "collaborators", owner, name, &body));
            }
            Fetch::Failed { status, body } => {
                *hole = true;
                errors.push(failed_msg(status, "collaborators", owner, name, &body));
            }
            Fetch::RateLimited => {
                *hole = true;
                errors.push(format!("429 rate limited on collaborators {owner}/{name}"));
            }
            Fetch::Absent | Fetch::Empty | Fetch::Missing => {}
        }

        match self.fetch(&format!(
            "/repos/{owner}/{name}/collaborators?affiliation=outside"
        )) {
            Fetch::Json(json) if json.as_array().is_some() => {
                let people = json.as_array().cloned().unwrap_or_default();
                match collaborators::from_outside_collaborators(&people, ctx) {
                    Ok(mut envs) => envelopes.append(&mut envs),
                    Err(e) => errors.push(sanitize_diagnostic(&e.to_string())),
                }
            }
            Fetch::Json(_) => {}
            Fetch::Denied { status, body } => {
                *hole = true;
                errors.push(denied_msg(
                    status,
                    "outside collaborators",
                    owner,
                    name,
                    &body,
                ));
            }
            Fetch::Failed { .. } | Fetch::RateLimited => {
                *hole = true;
                errors.push(format!(
                    "insufficient evidence: outside collaborators {owner}/{name}"
                ));
            }
            Fetch::Absent | Fetch::Empty | Fetch::Missing => {}
        }

        match self.fetch(&format!("/repos/{owner}/{name}/keys")) {
            Fetch::Json(json) if json.as_array().is_some() => {
                let keys = json.as_array().cloned().unwrap_or_default();
                match collaborators::from_deploy_keys(&keys, ctx) {
                    Ok(mut envs) => envelopes.append(&mut envs),
                    Err(e) => errors.push(sanitize_diagnostic(&e.to_string())),
                }
            }
            Fetch::Json(_) => {}
            Fetch::Denied { status, body } => {
                *hole = true;
                errors.push(denied_msg(status, "deploy keys", owner, name, &body));
            }
            Fetch::Failed { .. } | Fetch::RateLimited => {
                *hole = true;
                errors.push(format!("insufficient evidence: deploy keys {owner}/{name}"));
            }
            Fetch::Absent | Fetch::Empty | Fetch::Missing => {}
        }
    }

    fn configuration_digest(&self, scope: &CollectorScope) -> String {
        let body = (
            "collector.github",
            self.version.as_str(),
            scope.as_label(),
            descriptor::GITHUB_CANONICAL_EVIDENCE_TYPES,
            self.client.transport_mode(),
        );
        canonical_digest(&body).unwrap_or_else(|_| "undigested".into())
    }
}

impl EvidenceCollector for GitHubCollector {
    fn descriptor(&self) -> CollectorDescriptor {
        descriptor::descriptor(&self.version)
    }

    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
        let parsed = parse_scope(scope);
        if parsed.orgs.is_empty() && parsed.repos.is_empty() {
            let asset = parsed
                .unknown
                .first()
                .cloned()
                .unwrap_or_else(|| scope.as_label());
            return Err(CollectorError::OutOfScope { asset });
        }
        let (envelopes, _errors, _hole) = self.collect_inner(scope);
        Ok(envelopes)
    }
}

impl GitHubCollector {
    pub fn collect_batch(
        &self,
        request: CollectionRequest,
    ) -> Result<CollectionBatch, CollectorError> {
        let mut run = CollectionRun::new("collector.github", &self.version);
        run.scope = request.scope.as_label();
        run.configuration_digest = self.configuration_digest(&request.scope);
        let (envelopes, errors, hole) = self.collect_inner(&request.scope);
        run.completed_at = Some(Utc::now());
        run.evidence_count = envelopes.len() as u32;
        run.error_count = errors.len() as u32;
        run.status = if envelopes.is_empty() && !errors.is_empty() {
            "failed".into()
        } else if hole || !errors.is_empty() {
            "partial".into()
        } else {
            "complete".into()
        };
        Ok(CollectionBatch {
            run,
            envelopes,
            errors,
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

struct ParsedScope {
    orgs: Vec<String>,
    repos: Vec<(String, String)>,
    exclude_archived: bool,
    unknown: Vec<String>,
}

fn parse_scope(scope: &CollectorScope) -> ParsedScope {
    let mut orgs = Vec::new();
    let mut repos = Vec::new();
    let mut exclude_archived = false;
    let mut unknown = Vec::new();
    for label in scope.as_label().split(',') {
        let label = label.trim();
        if label.is_empty() {
            continue;
        }
        if label == "exclude_archived" {
            exclude_archived = true;
            continue;
        }
        if let Some(org) = label.strip_prefix("org:") {
            orgs.push(org.to_string());
            continue;
        }
        if let Some(repo) = label.strip_prefix("repo:") {
            if let Some((owner, name)) = repo.split_once('/') {
                repos.push((owner.to_string(), name.to_string()));
                continue;
            }
        }
        if let Some((owner, name)) = label.split_once('/') {
            if !owner.contains(':') {
                repos.push((owner.to_string(), name.to_string()));
                continue;
            }
        }
        unknown.push(label.to_string());
    }
    ParsedScope {
        orgs,
        repos,
        exclude_archived,
        unknown,
    }
}

fn looks_like_protection(v: &Value) -> bool {
    v.get("required_pull_request_reviews").is_some()
        || v.get("allow_force_pushes").is_some()
        || v.get("enforce_admins").is_some()
        || v.get("required_status_checks").is_some()
        || v.get("url")
            .and_then(Value::as_str)
            .is_some_and(|u| u.contains("/protection"))
}

fn looks_like_contents(v: &Value, name: &str) -> bool {
    v.get("name")
        .and_then(Value::as_str)
        .is_some_and(|n| n.eq_ignore_ascii_case(name))
        || v.get("path")
            .and_then(Value::as_str)
            .is_some_and(|p| p.ends_with(name))
}

fn denied_msg(status: u16, what: &str, owner: &str, name: &str, body: &str) -> String {
    format!(
        "PermissionDenied: {status} on {what} {owner}/{name}: {}",
        sanitize_diagnostic(body)
    )
}

fn failed_msg(status: u16, what: &str, owner: &str, name: &str, body: &str) -> String {
    format!(
        "insufficient evidence: {status} on {what} {owner}/{name}: {}",
        sanitize_diagnostic(body)
    )
}
