//! Compliance graph. Direction is preserved. Partial never becomes equivalent.

use std::collections::BTreeMap;

use crate::{
    ControlId, ControlTestId, EvidenceRequirementId, ExceptionId, IncidentId, MappingCompleteness,
    MappingDirection, MappingProvenance, MappingRelation, RequirementId, RiskId,
};

/// Explicit graph edge kinds. Never infer A satisfies C from A supports B supports C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphEdgeKind {
    Contains,
    MapsTo,
    Satisfies,
    PartiallySatisfies,
    Supports,
    TestedBy,
    RequiresEvidence,
    AppliesTo,
    Supersedes,
    DerivedFrom,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComplianceNodeRef {
    Requirement(RequirementId),
    Control(ControlId),
    Test(ControlTestId),
    EvidenceRequirement(EvidenceRequirementId),
    Risk(RiskId),
    Exception(ExceptionId),
    Incident(IncidentId),
}

#[derive(Debug, Clone)]
pub struct ComplianceEdge {
    pub from: ComplianceNodeRef,
    pub to: ComplianceNodeRef,
    pub relation: MappingRelation,
    pub completeness: MappingCompleteness,
    pub provenance: MappingProvenance,
}

#[derive(Debug, Clone, Default)]
pub struct ComplianceGraph {
    edges: BTreeMap<(RequirementId, RequirementId), MappingCompleteness>,
    typed: Vec<ComplianceEdge>,
}

impl ComplianceGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn link(
        &mut self,
        from: RequirementId,
        to: RequirementId,
        direction: MappingDirection,
        completeness: MappingCompleteness,
    ) {
        self.edges.insert((from.clone(), to.clone()), completeness);
        if matches!(direction, MappingDirection::Bidirectional) {
            self.edges.insert((to, from), completeness);
        }
    }

    pub fn link_edge(&mut self, edge: ComplianceEdge) {
        self.typed.push(edge);
    }

    pub fn maps(&self, from: &RequirementId, to: &RequirementId) -> Option<MappingCompleteness> {
        self.edges.get(&(from.clone(), to.clone())).copied()
    }

    /// Equivalent only when an explicit full bidirectional mapping exists.
    /// Partial paths and reverse-only edges never upgrade to equivalence.
    pub fn equivalent(&self, left: &RequirementId, right: &RequirementId) -> bool {
        if left == right {
            return false;
        }
        matches!(
            (self.maps(left, right), self.maps(right, left)),
            (
                Some(MappingCompleteness::Full),
                Some(MappingCompleteness::Full)
            )
        )
    }
}
