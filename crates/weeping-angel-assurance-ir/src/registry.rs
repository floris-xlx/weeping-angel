//! Pure registry queries over `AssessmentDefinition` implementations.
//!
//! Overlap is fail-closed so coverage math cannot silently double-count.
//! These helpers do not call collectors or write evidence conclusions.

use std::collections::BTreeSet;

use crate::{
    AssessmentDefinition, ControlId, ControlImplementation, ControlImplementationId, SelectorScope,
    SubjectKind, SubjectSelector,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationOverlap {
    pub control_id: ControlId,
    pub left_id: ControlImplementationId,
    pub right_id: ControlImplementationId,
    pub reason: String,
    pub selectors_or_assets: String,
}

pub fn implementations_for<'a>(
    assessment: &'a AssessmentDefinition,
    control_id: &ControlId,
) -> Vec<&'a ControlImplementation> {
    assessment
        .implementations
        .iter()
        .filter(|row| row.control_id() == control_id)
        .collect()
}

pub fn current_implementations_for<'a>(
    assessment: &'a AssessmentDefinition,
    control_id: &ControlId,
) -> Vec<&'a ControlImplementation> {
    implementations_for(assessment, control_id)
        .into_iter()
        .filter(|row| row.is_coverage_active())
        .collect()
}

pub fn implementation_by_id<'a>(
    assessment: &'a AssessmentDefinition,
    id: &ControlImplementationId,
) -> Option<&'a ControlImplementation> {
    assessment.implementations.iter().find(|row| row.id() == id)
}

pub fn overlap_report(assessment: &AssessmentDefinition) -> Vec<ImplementationOverlap> {
    let mut out = Vec::new();
    let mut seen_controls = BTreeSet::new();
    for row in &assessment.implementations {
        if !seen_controls.insert(row.control_id().as_str().to_string()) {
            continue;
        }
        let active = current_implementations_for(assessment, row.control_id());
        for (i, left) in active.iter().enumerate() {
            for right in active.iter().skip(i + 1) {
                if let Some(hit) = pair_overlap(left, right) {
                    out.push(hit);
                }
            }
        }
    }
    out
}

fn pair_overlap(
    left: &ControlImplementation,
    right: &ControlImplementation,
) -> Option<ImplementationOverlap> {
    let pop = population_overlap(left.applies_to(), right.applies_to());
    let assets = asset_overlap(left, right);
    match (pop, assets) {
        (Some(pop_reason), Some(asset_reason)) => Some(ImplementationOverlap {
            control_id: left.control_id().clone(),
            left_id: left.id().clone(),
            right_id: right.id().clone(),
            reason: format!("{pop_reason}; {asset_reason}"),
            selectors_or_assets: format!(
                "left applies_to={} asset_ids={:?}; right applies_to={} asset_ids={:?}",
                summarize_selectors(left.applies_to()),
                left.asset_ids()
                    .iter()
                    .map(|a| a.as_str())
                    .collect::<Vec<_>>(),
                summarize_selectors(right.applies_to()),
                right
                    .asset_ids()
                    .iter()
                    .map(|a| a.as_str())
                    .collect::<Vec<_>>(),
            ),
        }),
        _ => None,
    }
}

fn population_overlap(left: &[SubjectSelector], right: &[SubjectSelector]) -> Option<String> {
    if left.is_empty() && right.is_empty() {
        return Some("universal selector (empty applies_to)".into());
    }
    if left.is_empty() {
        return Some(format!(
            "universal applies_to overlaps selector {}",
            summarize_selectors(right)
        ));
    }
    if right.is_empty() {
        return Some(format!(
            "universal applies_to overlaps selector {}",
            summarize_selectors(left)
        ));
    }
    for a in left {
        for b in right {
            if let Some(reason) = selector_pair_overlaps(a, b) {
                return Some(reason);
            }
        }
    }
    None
}

fn selector_pair_overlaps(a: &SubjectSelector, b: &SubjectSelector) -> Option<String> {
    if a.kind != b.kind {
        return None;
    }
    if tags_conflict(&a.tags, &b.tags) {
        return None;
    }
    match (a.scope, b.scope) {
        (SelectorScope::All, _) | (_, SelectorScope::All) => Some(format!(
            "selector scope All overlaps kind={:?} ids={:?}/{:?}",
            a.kind, a.ids, b.ids
        )),
        (SelectorScope::AnyOf, SelectorScope::AnyOf) => {
            if a.ids.is_empty() || b.ids.is_empty() {
                return Some(format!(
                    "empty anyOf is all-of-kind and overlaps kind={:?} selector",
                    a.kind
                ));
            }
            let shared: Vec<&String> = a.ids.intersection(&b.ids).collect();
            if shared.is_empty() {
                None
            } else {
                Some(format!(
                    "anyOf ids overlap {} (kind={:?} scope=anyOf)",
                    shared
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    a.kind
                ))
            }
        }
        (SelectorScope::NoneOf, _) | (_, SelectorScope::NoneOf) => Some(format!(
            "NoneOf remainder cannot be proven disjoint (kind={:?} ids={:?}/{:?} scope)",
            a.kind, a.ids, b.ids
        )),
    }
}

fn tags_conflict(
    left: &std::collections::BTreeMap<String, String>,
    right: &std::collections::BTreeMap<String, String>,
) -> bool {
    if left.is_empty() || right.is_empty() {
        return false;
    }
    for (key, value) in left {
        if let Some(other) = right.get(key)
            && other != value
        {
            return true;
        }
    }
    false
}

fn asset_overlap(left: &ControlImplementation, right: &ControlImplementation) -> Option<String> {
    if left.asset_ids().is_empty() && right.asset_ids().is_empty() {
        return Some("universal assets (empty asset_ids)".into());
    }
    if left.asset_ids().is_empty() {
        return Some(format!(
            "universal asset_ids overlap {}",
            join_assets(right)
        ));
    }
    if right.asset_ids().is_empty() {
        return Some(format!("universal asset_ids overlap {}", join_assets(left)));
    }
    let right_set: BTreeSet<&str> = right.asset_ids().iter().map(|a| a.as_str()).collect();
    let shared: Vec<&str> = left
        .asset_ids()
        .iter()
        .map(|a| a.as_str())
        .filter(|id| right_set.contains(id))
        .collect();
    if shared.is_empty() {
        None
    } else {
        Some(format!("asset_ids overlap {}", shared.join(",")))
    }
}

fn join_assets(row: &ControlImplementation) -> String {
    row.asset_ids()
        .iter()
        .map(|a| a.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn summarize_selectors(selectors: &[SubjectSelector]) -> String {
    if selectors.is_empty() {
        return "[] (universal)".into();
    }
    selectors
        .iter()
        .map(summarize_selector)
        .collect::<Vec<_>>()
        .join("; ")
}

fn summarize_selector(selector: &SubjectSelector) -> String {
    let kind = match selector.kind {
        SubjectKind::Identity => "identity",
        SubjectKind::Asset => "asset",
        SubjectKind::Vendor => "vendor",
        SubjectKind::Organization => "organization",
        other => {
            return format!(
                "{other:?} ids={:?} scope={:?}",
                selector.ids, selector.scope
            );
        }
    };
    let scope = match selector.scope {
        SelectorScope::All => "all",
        SelectorScope::AnyOf => "anyOf",
        SelectorScope::NoneOf => "noneOf",
    };
    format!(
        "kind={kind} ids={:?} tags={:?} scope={scope}",
        selector.ids, selector.tags
    )
}
