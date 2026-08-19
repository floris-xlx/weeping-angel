use serde_json::Value;
use weeping_angel_assurance_ir::AssetId;
use weeping_angel_evidence::EvidenceValue;

use super::normalize::{EmitCtx, emit};
use crate::{CollectorError, EvidenceEnvelope};

pub const MODULE: &str = "collaborators";

pub fn from_collaborators(
    people: &[Value],
    ctx: &EmitCtx<'_>,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let mut out = Vec::new();
    for person in people {
        let login = match person.get("login").and_then(Value::as_str) {
            Some(l) => l,
            None => continue,
        };
        let admin = person
            .pointer("/permissions/admin")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || person.get("role").and_then(Value::as_str) == Some("admin")
            || person
                .get("site_admin")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        if !admin {
            continue;
        }
        let subject = format!("user:{login}");
        let ident_ctx = EmitCtx {
            collected_at: ctx.collected_at,
            scope: ctx.scope,
            asset: AssetId::new(&subject),
        };
        out.push(emit(
            "evidence.identity.privileged-membership",
            vec![
                ("subject_id", EvidenceValue::string(&subject)),
                ("privileged", EvidenceValue::from_bool(true)),
                ("roles", EvidenceValue::StringList(vec!["admin".into()])),
            ],
            "privileged membership observation",
            &ident_ctx,
        )?);
        out.push(emit(
            "inventory.subject",
            vec![
                ("kind", EvidenceValue::string("identity")),
                ("id", EvidenceValue::string(&subject)),
            ],
            "privileged identity subject",
            &ident_ctx,
        )?);
    }
    Ok(out)
}

pub fn from_outside_collaborators(
    people: &[Value],
    ctx: &EmitCtx<'_>,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let mut out = Vec::new();
    for person in people {
        let login = match person.get("login").and_then(Value::as_str) {
            Some(l) => l,
            None => continue,
        };
        let subject = format!("user:{login}");
        let ident_ctx = EmitCtx {
            collected_at: ctx.collected_at,
            scope: ctx.scope,
            asset: AssetId::new(&subject),
        };
        out.push(emit(
            "evidence.identity.external-access",
            vec![
                ("subject_id", EvidenceValue::string(&subject)),
                ("external", EvidenceValue::from_bool(true)),
            ],
            "outside collaborator observation",
            &ident_ctx,
        )?);
        out.push(emit(
            "inventory.subject",
            vec![
                ("kind", EvidenceValue::string("user")),
                ("id", EvidenceValue::string(&subject)),
            ],
            "external identity subject",
            &ident_ctx,
        )?);
    }
    Ok(out)
}

pub fn from_deploy_keys(
    keys: &[Value],
    ctx: &EmitCtx<'_>,
) -> Result<Vec<EvidenceEnvelope>, CollectorError> {
    let mut out = Vec::new();
    for key in keys {
        let id = key
            .get("id")
            .and_then(Value::as_i64)
            .map(|n| n.to_string())
            .or_else(|| key.get("title").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "unknown".into());
        let read_only = key
            .get("read_only")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let subject = format!("deploy-key:{id}");
        let ident_ctx = EmitCtx {
            collected_at: ctx.collected_at,
            scope: ctx.scope,
            asset: AssetId::new(&subject),
        };
        out.push(emit(
            "evidence.identity.privileged-membership",
            vec![
                ("subject_id", EvidenceValue::string(&subject)),
                ("privileged", EvidenceValue::from_bool(true)),
                (
                    "roles",
                    EvidenceValue::StringList(vec!["deploy-key".into()]),
                ),
            ],
            "deploy key membership observation (no key material)",
            &ident_ctx,
        )?);
        out.push(emit(
            "inventory.subject",
            vec![
                ("kind", EvidenceValue::string("identity")),
                ("id", EvidenceValue::string(&subject)),
            ],
            "deploy key identity subject",
            &ident_ctx,
        )?);
        if !read_only {
            out.push(emit(
                "evidence.identity.external-access",
                vec![
                    ("subject_id", EvidenceValue::string(&subject)),
                    ("external", EvidenceValue::from_bool(true)),
                ],
                "write-capable deploy key observation",
                &ident_ctx,
            )?);
        }
    }
    Ok(out)
}
