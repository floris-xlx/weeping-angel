//! Bounded TestExpr AST. Provider-blind. No script host.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use weeping_angel_assurance_ir::EvidenceType;
use weeping_angel_assurance_ir::{SelectorScope, SubjectKind};

/// Re-export of the single stored value model. Control-test does not define a second enum.
pub use weeping_angel_evidence::EvidenceValue;

/// Adapter over `weeping_angel_assurance_ir::SubjectSelector` (IR SSOT).
/// Legacy `{ kind, id }` JSON folds `id` into `ids`. Not a third selector type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectSelector {
    pub kind: Option<String>,
    pub id: Option<String>,
}

impl SubjectSelector {
    pub fn to_ir(&self) -> weeping_angel_assurance_ir::SubjectSelector {
        self.clone().into()
    }
}

impl From<SubjectSelector> for weeping_angel_assurance_ir::SubjectSelector {
    fn from(thin: SubjectSelector) -> Self {
        let kind = thin
            .kind
            .as_deref()
            .and_then(SubjectKind::parse_name)
            .unwrap_or_default();
        let mut ids = std::collections::BTreeSet::new();
        if let Some(id) = thin.id {
            ids.insert(id);
        }
        let scope = if ids.is_empty() {
            SelectorScope::All
        } else {
            SelectorScope::AnyOf
        };
        weeping_angel_assurance_ir::SubjectSelector {
            kind,
            ids,
            tags: Default::default(),
            scope,
        }
    }
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
    CoverageExactly {
        selector: SubjectSelector,
        evidence: EvidenceSelector,
        percentage: String,
    },
    CountWhere {
        selector: SubjectSelector,
        evidence: EvidenceSelector,
        predicate: CountPredicate,
    },
    AllSubjects {
        selector: SubjectSelector,
        evidence: EvidenceSelector,
    },
    AnySubject {
        selector: SubjectSelector,
        evidence: EvidenceSelector,
    },
    NoneSubjects {
        selector: SubjectSelector,
        evidence: EvidenceSelector,
    },
    MissingSubjects {
        selector: SubjectSelector,
        evidence: EvidenceSelector,
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
