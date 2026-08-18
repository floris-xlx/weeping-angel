//! Bounded TestExpr AST. Provider-blind. No script host.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use weeping_angel_assurance_ir::EvidenceType;

/// Typed evidence values. Avoid untyped JSON everywhere.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Decimal(String),
    String(String),
    Timestamp(String),
    Duration(String),
    StringSet(Vec<String>),
    Identifier(String),
}

impl EvidenceValue {
    pub fn parse_fact(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.eq_ignore_ascii_case("true") {
            return Self::Boolean(true);
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return Self::Boolean(false);
        }
        if let Ok(i) = trimmed.parse::<i64>() {
            return Self::Integer(i);
        }
        if trimmed.parse::<f64>().is_ok() {
            return Self::Decimal(trimmed.to_string());
        }
        Self::String(raw.to_string())
    }

    pub fn as_integer(&self) -> Result<i64, String> {
        match self {
            Self::Integer(v) => Ok(*v),
            Self::String(s) => s
                .parse()
                .map_err(|_| format!("type mismatch: expected Integer, got {s}")),
            other => Err(format!("type mismatch: expected Integer, got {other:?}")),
        }
    }
}

/// Provider-neutral subject selector. No collector id.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectSelector {
    pub kind: Option<String>,
    pub id: Option<String>,
}

/// Provider-neutral evidence selector. No collector id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSelector {
    pub evidence_type: EvidenceType,
    pub subject_selector: SubjectSelector,
    pub field: Option<String>,
    pub freshness: Option<Duration>,
}

impl EvidenceSelector {
    pub fn of_type(evidence_type: EvidenceType) -> Self {
        Self {
            evidence_type,
            subject_selector: SubjectSelector::default(),
            field: None,
            freshness: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CountPredicate {
    Eq(u64),
    Gte(u64),
    Lte(u64),
}

/// Bounded expression AST. Not a script host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestExpr {
    Exists(EvidenceSelector),
    Missing(EvidenceSelector),
    Eq(ValueExpr, EvidenceValue),
    Neq(ValueExpr, EvidenceValue),
    Gt(ValueExpr, EvidenceValue),
    Gte(ValueExpr, EvidenceValue),
    Lt(ValueExpr, EvidenceValue),
    Lte(ValueExpr, EvidenceValue),
    Contains(ValueExpr, EvidenceValue),
    NotContains(ValueExpr, EvidenceValue),
    In(ValueExpr, Vec<EvidenceValue>),
    Count {
        selector: EvidenceSelector,
        predicate: CountPredicate,
    },
    FreshWithin {
        selector: EvidenceSelector,
        duration: Duration,
    },
    CoverageAtLeast {
        selector: SubjectSelector,
        evidence: EvidenceSelector,
        percentage: String,
    },
    All(Vec<TestExpr>),
    Any(Vec<TestExpr>),
    None(Vec<TestExpr>),
    Not(Box<TestExpr>),
    ManualReview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValueExpr {
    Field(EvidenceSelector),
    Literal(EvidenceValue),
}
