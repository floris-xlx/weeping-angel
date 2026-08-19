//! Facade / collector allow-set adapters. Only `InScope` asset ids.

use std::collections::BTreeSet;

use weeping_angel_assurance_ir::{AssetId, SubjectKind};
use weeping_angel_collector::CollectorScope;

use super::snapshot::{ScopeDecision, ScopeResolution};

impl ScopeResolution {
    pub fn in_scope_asset_ids(&self) -> BTreeSet<AssetId> {
        self.subjects
            .iter()
            .filter(|row| row.decision == ScopeDecision::InScope && kind_maps_to_asset(row.kind))
            .map(|row| AssetId::new(row.id.as_str()))
            .collect()
    }

    pub fn to_facade_assessment_scope(&self) -> crate::AssessmentScope {
        let mut scope = crate::AssessmentScope::new();
        for id in self.in_scope_asset_ids() {
            scope = scope.allow_asset(id);
        }
        scope
    }

    pub fn to_collector_scope(&self) -> CollectorScope {
        self.to_facade_assessment_scope().to_collector_scope()
    }
}

fn kind_maps_to_asset(kind: SubjectKind) -> bool {
    !matches!(
        kind,
        SubjectKind::Identity
            | SubjectKind::User
            | SubjectKind::PrivilegedIdentity
            | SubjectKind::ServiceAccount
            | SubjectKind::Vendor
            | SubjectKind::ProcessingActivity
            | SubjectKind::BusinessUnit
            | SubjectKind::PersonnelPopulation
            | SubjectKind::Location
            | SubjectKind::DataDomain
    )
}
