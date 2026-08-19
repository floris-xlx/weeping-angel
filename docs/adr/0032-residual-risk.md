# ADR 0032 — Residual risk is an explainable projection, not a hidden formula

<!-- weeping-angel-adr-meta
id = "0032"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_residual_risk_target` GREEN (P09-T01–T20); baseline absence claims skip-superseded. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The draft register claim that “GitHub collector owns control-derived residual” in [`0040-operational-risk-register.md`](0040-operational-risk-register.md) §5. Does **not** supercede IR schema `assurance-ir/v1`, assessment-lineage immutability, control-test `Effectiveness`, collector blindness, Kleene applicability, risk methodology scoring, or treatment state machines. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md), [assessment lineage](0015-assessment-lineage.md) |
| Spec | [`docs/specs/residual-risk.md`](../specs/residual-risk.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_residual_risk_baseline` skip-superseded; `sdd_residual_risk_target` GREEN (`tests/contracts/residual_risk.{baseline,target}.rs`) |

> Filename `0003-*` is shared with catalog-program / operational-slice siblings. **0004** is documentation architecture. **0005-*** remains risk methodology and the operational register. Cite this file by **path**.

## Context

On SHA `6e31bf1a…`, `weeping-angel-assurance-ir::Risk` is `{ id, title, description, status }` with module comment *“Minimal risk record. Not a risk engine.”* There was no residual-risk projection, mode, methodology version, treatment-plan version, or control-effectiveness reduction.

Canonical control-test results already exist (`ControlTestResult`, `Effectiveness` including `Effective`, `Ineffective`, `PartiallyEffective`, `NotApplicable`, `NotTested`, `InsufficientEvidence`, `StaleEvidence`, `ManualReviewRequired`, `ExceptionApproved`, `Inconclusive`). Assessment lineage already requires immutable snapshots and new records on change.

Without a decision, implementers will map `Effective` to zero residual, treat `ExceptionApproved` as Low, invent a 5×5 reduction inside collectors or Kleene, overwrite historical residuals when a control regresses, or implement full 05/06/08 engines “because residual risk needs them.”

Questions this decision answers:

1. Is residual a field on `Risk`, a collector fact, or a projection document?
2. Which modes exist, and what fails closed?
3. How do methodology / register / treatment show up without this slice owning those engines?
4. How does control effectiveness change residual without a secret formula?
5. What do exceptions mean for residual?
6. How does history work under assessment lineage?
7. Which crates own which types?

## Decision

This is what shipped. Field-level law is [`docs/specs/residual-risk.md`](../specs/residual-risk.md).

### 1. Residual is a projection document, not `Risk.status`

`ResidualRiskProjection` lives in `weeping-angel-assurance-ir::residual` and is re-exported from that crate’s `lib.rs`. `Risk` stays an inventory record. Schema remains `assurance-ir/v1`. Evidence envelopes and collectors do not carry residual ratings.

Incorrect: `Risk.residual = 0` because tests passed; `EvidenceValue::ResidualLow`; GitHub collector emitting residual.

### 2. Three modes; Hybrid is not optional Calculated

```text
ResidualRiskMode = Calculated | Assessed | Hybrid
```

- **Calculated** — deterministic, explicitly versioned methodology. Same pins → same residual identity.
- **Assessed** — `ManualResidualAssessment` with `PrincipalRef` + non-empty rationale + `assessedAt`. Missing any → `missing manual assessment`.
- **Hybrid** — deterministic signals **and** an approved management assessment (`approvedBy` required). Missing either side → `missing management assessment`. Management may **raise** residual above the calculated ordinal; it cannot silently ignore Calculated fail-closed conditions or lower residual below the calculated floor.

### 3. Pin lineage; do not implement neighbor engines

Every projection identifies inherent-risk version, treatment-plan version, methodology id+version, relevant controls, control-test snapshot (digest + result ids), `projectedAt`, residual ordinal/rating, reduction trace, and any manual assessment/approval plus exception ids.

Minimum versioned refs (caller-constructed snapshots; not `score_risk` / `TreatmentPlan` / register expansion):

```text
InherentRiskRef { riskId, version, digest? } + InherentRiskSnapshot { pin, ratingId, ordinal }
TreatmentPlanRef { planId, version, digest? } + TreatmentPlanSnapshot { pin, relevantControlIds, completeness }
MethodologyRef { methodologyId, version }
ControlTestSnapshotRef { digest, resultIds[] }
```

Missing or empty required pins fail closed (`missing inherent-risk version`, `missing treatment-plan version`, `missing methodology version`, `unknown methodology`, `missing control-test snapshot`).

### 4. Methodology-specific reduction; `Effective` is never zero residual

Reuse `weeping_angel_control_test::{ControlTestResult, Effectiveness}`. Shipped calculated methodologies:

| Id | Version | Behavior |
| --- | --- | --- |
| `residual-methodology:no-reduction` | `v1` | Effectiveness never lowers residual; residual equals inherent |
| `residual-methodology:control-effectiveness` | `v1` | Steps in sorted `ControlId` order; `Effective`+complete = 2; partial = 1; `Ineffective` / `ExceptionApproved` = 0; floor `MIN_RESIDUAL_FLOOR = 1` |

`Effective` never maps to residual ordinal 0. Partial treatment is not full credit. Multiple results for one control take the **worst** effectiveness. `NotApplicable` on a **relevant** control is `not applicable` (contradiction). `NotTested` / `InsufficientEvidence` / `StaleEvidence` fail closed. `ManualReviewRequired` / `Inconclusive` fail Calculated; they never grant reduction.

### 5. Exceptions are governance evidence, not a Low floor

`Effectiveness::ExceptionApproved` and approved `Exception` records grant **no** silent reduction to Low. Exception ids are copied onto the projection when present; the trace names the variant.

### 6. New projection on regression; history stays queryable

`project_residual_risk(store, request)` seals identity as `residual:{sha256}` over semantic fields (including caller-supplied `projectedAt`). `ResidualRiskStore::insert` is first-write-wins. Control regression or pin change produces a **new** id; `query_residual_risk` still returns the old document.

### 7. Crate homes (no new crate)

| Concern | Home |
| --- | --- |
| Domain types, refs, errors, methodology constants | `weeping-angel-assurance-ir` (`residual.rs`, `ResidualRiskId` in `id.rs`) |
| `ResidualRiskRequest`, `ResidualRiskStore`, `project_residual_risk`, `query_residual_risk` | `weeping-angel-assurance::residual` (module path; not crate-root re-export) |
| Control-test enum | `weeping_angel_control_test::Effectiveness` (unchanged) |
| Evidence | observations only |

Collision fence: GitHub collector, catalog TOML / ISO remaps, Kleene evaluator, unrelated catalog suites, dashboards / acceptance workflow.

## Non-goals

- Dashboards and risk-acceptance workflows.
- Full methodology / register / treatment engines (those slices own their types; residual pins snapshots).
- Forking `assurance-ir/v1` or adding residual variants to `Effectiveness`.

## Consequences

- Operators can reproduce residual risk for any finalized assessment snapshot from pinned treatment/control state.
- Calculated math is named and versioned; Assessed/Hybrid remain first-class because not all risk is a formula.
- Register rows may still hold a residual *placeholder*; the authoritative projection is `ResidualRiskProjection`.
- Dual-suite `sdd_residual_risk_baseline` / `sdd_residual_risk_target` is the executable contract.

## Related

- Spec: [`docs/specs/residual-risk.md`](../specs/residual-risk.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
- Register placeholder (superseded ownership): [ADR 0005 operational risk register](0040-operational-risk-register.md)
