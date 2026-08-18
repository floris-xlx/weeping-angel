//! Compliance graph. Direction is preserved. Partial never becomes equivalent.

use std::collections::BTreeMap;

use crate::{MappingCompleteness, MappingDirection, RequirementId};

#[derive(Debug, Clone, Default)]
pub struct ComplianceGraph {
    edges: BTreeMap<(RequirementId, RequirementId), MappingCompleteness>,
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
            (Some(MappingCompleteness::Full), Some(MappingCompleteness::Full))
        )
    }
}
