use std::collections::BTreeSet;

use weeping_angel_assurance_ir::EvidenceType;

use crate::{CollectorCapabilities, CollectorDescriptor};

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

pub fn descriptor(version: &str) -> CollectorDescriptor {
    CollectorDescriptor {
        id: "collector.github".into(),
        version: version.into(),
        evidence_types: GITHUB_EVIDENCE_TYPES
            .iter()
            .map(|t| EvidenceType::new(*t))
            .collect(),
        provider_family: "source-control".into(),
        subject_types: BTreeSet::from(["repository".into(), "branch".into()]),
        capabilities: CollectorCapabilities {
            pagination: true,
            point_in_time: true,
            worker_safe: true,
            ..CollectorCapabilities::default()
        },
        required_permissions: vec![
            "contents:read".into(),
            "administration:read".into(),
            "metadata:read".into(),
        ],
    }
}
