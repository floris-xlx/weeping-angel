use std::collections::BTreeSet;

use weeping_angel_assurance_ir::EvidenceType;

use crate::{CollectorCapabilities, CollectorDescriptor};

/// ADR 0002 historical `source.*` names. ISO GH-012 needles and IAM-015
/// (`evidence.identity.*` must stay off this list). Not emitted types.
pub const GITHUB_EVIDENCE_TYPES: &[&str] = &[
    "source.repository.exists",
    "source.repository.visibility",
    "source.default_branch",
    "source.branch.protection",
    "source.branch.required_reviews",
    "source.branch.required_status_checks",
    "source.branch.force_push_protection",
    "source.branch.deletion_protection",
    "source.codeowners.present",
    "source.admin.permissions",
    "source.collaborator.permission",
    "source.security.dependabot.enabled",
    "source.security.secret_scanning.enabled",
    "source.security.code_scanning.configured",
    "source.workflow.permissions",
    "source.workflow.review_requirement",
    "source.ruleset.present",
    "source.repository.archived",
    "source.commit.signing",
];

/// Canonical types this build actually emits. Unioned onto `CollectorDescriptor`
/// (identity types live here, never on `GITHUB_EVIDENCE_TYPES`).
pub const GITHUB_CANONICAL_EVIDENCE_TYPES: &[&str] = &[
    "evidence.repository.inventory",
    "evidence.repository.visibility",
    "evidence.repository.default-branch",
    "evidence.repository.branch-protection",
    "evidence.repository.review-policy",
    "evidence.repository.review-ownership",
    "evidence.repository.security-scanning",
    "evidence.repository.dependency-scanning",
    "evidence.repository.commit-signing",
    "evidence.cicd.status-checks",
    "evidence.cicd.workflow-permissions",
    "evidence.deployment.environment-protection",
    "evidence.identity.privileged-membership",
    "evidence.identity.external-access",
    "inventory.subject",
    "inventory.complete",
];

/// Historical source.* → canonical mapping table (compatibility surface).
pub const SOURCE_TO_CANONICAL: &[(&str, &str)] = &[
    ("source.repository.exists", "evidence.repository.inventory"),
    (
        "source.repository.visibility",
        "evidence.repository.visibility",
    ),
    (
        "source.default_branch",
        "evidence.repository.default-branch",
    ),
    (
        "source.branch.protection",
        "evidence.repository.branch-protection",
    ),
    (
        "source.branch.required_reviews",
        "evidence.repository.review-policy",
    ),
    (
        "source.branch.required_status_checks",
        "evidence.cicd.status-checks",
    ),
    (
        "source.branch.force_push_protection",
        "evidence.repository.branch-protection",
    ),
    (
        "source.branch.deletion_protection",
        "evidence.repository.branch-protection",
    ),
    (
        "source.codeowners.present",
        "evidence.repository.review-ownership",
    ),
    (
        "source.admin.permissions",
        "evidence.identity.privileged-membership",
    ),
    (
        "source.collaborator.permission",
        "evidence.identity.external-access",
    ),
    (
        "source.security.dependabot.enabled",
        "evidence.repository.dependency-scanning",
    ),
    (
        "source.security.secret_scanning.enabled",
        "evidence.repository.security-scanning",
    ),
    (
        "source.security.code_scanning.configured",
        "evidence.repository.security-scanning",
    ),
    (
        "source.workflow.permissions",
        "evidence.cicd.workflow-permissions",
    ),
    (
        "source.workflow.review_requirement",
        "evidence.repository.review-policy",
    ),
    (
        "source.ruleset.present",
        "evidence.repository.branch-protection",
    ),
    (
        "source.repository.archived",
        "evidence.repository.inventory",
    ),
    (
        "source.commit.signing",
        "evidence.repository.commit-signing",
    ),
];

/// Failure behavior (GitHub-owned; not a shared `CollectorDescriptor` field).
///
/// 401/403 → PermissionDenied diagnostic (downstream insufficient evidence);
/// never a fabricated negative boolean. 404 on a protection resource → observed
/// absent. 404 on a repo → insufficient / not visible. 429 → retry then partial
/// if still limited; never a boolean observation. Partial subject failure does
/// not abort the rest of the batch.
pub const GITHUB_FAILURE_BEHAVIOR: &str = "\
401 unauthorized → PermissionDenied (insufficient evidence, not a negative fact). \
403 permission denied → PermissionDenied / insufficient-evidence diagnostic; \
continue other subjects; never protected=false or enabled=false. \
404 on branch protection / ruleset → observed absent (protected=false). \
404 on repository → insufficient evidence, not exists=false. \
429 rate limit → Retry-After / backoff retry, else partial; never a boolean. \
5xx / transport → partial run, keep prior envelopes. \
Pagination hole or list 403 → inventory.complete not authoritative.";

pub fn descriptor(version: &str) -> CollectorDescriptor {
    let _ = (SOURCE_TO_CANONICAL, GITHUB_FAILURE_BEHAVIOR);
    CollectorDescriptor {
        id: "collector.github".into(),
        version: version.into(),
        evidence_types: GITHUB_CANONICAL_EVIDENCE_TYPES
            .iter()
            .map(|t| EvidenceType::new(*t))
            .collect(),
        provider_family: "source-control".into(),
        subject_types: BTreeSet::from([
            "repository".into(),
            "branch".into(),
            "organization".into(),
            "identity".into(),
            "deployment".into(),
        ]),
        capabilities: CollectorCapabilities {
            pagination: true,
            incremental: false,
            point_in_time: true,
            worker_safe: true,
            ..CollectorCapabilities::default()
        },
        required_permissions: vec![
            "contents:read".into(),
            "metadata:read".into(),
            "administration:read".into(),
            "actions:read".into(),
            "members:read".into(),
            "security_events:read".into(),
        ],
    }
}
