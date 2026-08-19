use serde_json::Value;
use weeping_angel_evidence::EvidenceValue;

use super::normalize::{EmitCtx, emit, repo_subject_id};
use crate::{CollectorError, EvidenceEnvelope};

pub fn from_protection_json(
    json: &Value,
    owner: &str,
    name: &str,
    ctx: &EmitCtx<'_>,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let subject = repo_subject_id(owner, name);
    let mut out = Vec::new();

    let force_push_allowed = json
        .pointer("/allow_force_pushes/enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let deletion_allowed = json
        .pointer("/allow_deletions/enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let enforce_admins = json
        .pointer("/enforce_admins/enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let admin_bypass_allowed = !enforce_admins;
    let reviews = json
        .pointer("/required_pull_request_reviews/required_approving_review_count")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let checks = json.pointer("/required_status_checks").is_some();
    let code_owner_reviews = json
        .pointer("/required_pull_request_reviews/require_code_owner_reviews")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let signing_required = json
        .pointer("/required_signatures/enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    out.push(emit(
        "evidence.repository.branch-protection",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("protected", EvidenceValue::from_bool(true)),
            (
                "force_push_allowed",
                EvidenceValue::from_bool(force_push_allowed),
            ),
            (
                "deletion_allowed",
                EvidenceValue::from_bool(deletion_allowed),
            ),
            (
                "admin_bypass_allowed",
                EvidenceValue::from_bool(admin_bypass_allowed),
            ),
        ],
        "default branch protection observation",
        ctx,
    )?);
    out.push(emit(
        "evidence.repository.review-policy",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("reviews_required", EvidenceValue::from_bool(reviews > 0)),
            ("required_reviewer_count", EvidenceValue::integer(reviews)),
        ],
        "required review observation",
        ctx,
    )?);
    out.push(emit(
        "evidence.cicd.status-checks",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("status_checks_required", EvidenceValue::from_bool(checks)),
        ],
        "required status checks observation",
        ctx,
    )?);
    if signing_required {
        out.push(emit(
            "evidence.repository.commit-signing",
            vec![
                ("subject_id", EvidenceValue::string(&subject)),
                ("signing_required", EvidenceValue::from_bool(true)),
            ],
            "commit signing observation",
            ctx,
        )?);
    }
    if code_owner_reviews {
        out.push(emit(
            "evidence.repository.review-ownership",
            vec![
                ("subject_id", EvidenceValue::string(&subject)),
                ("ownership_defined", EvidenceValue::from_bool(true)),
            ],
            "review ownership observation",
            ctx,
        )?);
    }
    Ok(out)
}

pub fn unprotected(
    owner: &str,
    name: &str,
    ctx: &EmitCtx<'_>,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let subject = repo_subject_id(owner, name);
    let mut out = Vec::new();
    out.push(emit(
        "evidence.repository.branch-protection",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("protected", EvidenceValue::from_bool(false)),
            ("force_push_allowed", EvidenceValue::from_bool(true)),
            ("deletion_allowed", EvidenceValue::from_bool(true)),
            ("admin_bypass_allowed", EvidenceValue::from_bool(true)),
        ],
        "default branch has no protection rule",
        ctx,
    )?);
    out.push(emit(
        "evidence.repository.review-policy",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("reviews_required", EvidenceValue::from_bool(false)),
            ("required_reviewer_count", EvidenceValue::integer(0)),
        ],
        "no required reviews observed",
        ctx,
    )?);
    out.push(emit(
        "evidence.cicd.status-checks",
        vec![
            ("subject_id", EvidenceValue::string(&subject)),
            ("status_checks_required", EvidenceValue::from_bool(false)),
        ],
        "no required status checks observed",
        ctx,
    )?);
    Ok(out)
}
