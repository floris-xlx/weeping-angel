# ADR 0006 — Risk treatment engine in assurance IR

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_risk_treatment_target` GREEN (P08-T01–T16); baseline skip-superseded (`#[ignore = "superseded by target suite"]`) |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The operational reading “`supports_risk_treatment` is only a compile capability and `RiskStatus::Accepted` is a free enum with no evidence.” Does **not** supercede IR schema `assurance-ir/v1`, canonical digest `canon/v1`, ADR 0001 spine, Kleene applicability, [risk methodology scoring](0005-risk-methodology.md), [risk register](0005-operational-risk-register.md) ownership of `Risk`, or residual-risk projection math. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md) |
| Spec | [`docs/specs/risk-treatment.md`](../specs/risk-treatment.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_risk_treatment_baseline` skip-superseded; `sdd_risk_treatment_target` GREEN (`tests/contracts/risk_treatment.{baseline,target}.rs`) |

> Filename **`0006-*`**. Methodology and register occupy sibling `0005-*` ADRs. Do **not** add `0003-risk-treatment.md` or a third `0005-*`. Cite this file by **path**.

## Context

On SHA `6e31bf1a…`:

1. `Risk` was `{ id, title, description, status ∈ {Open, Accepted, Mitigated, Closed} }` with module comment *“Not a risk engine.”*
2. `RiskStatus::Accepted` was freely assignable. No principal, expiry, or immutable acceptance record.
3. There was no `RiskTreatmentDecision`, `TreatmentPlan`, `TreatmentAction`, or fail-closed treatment state machine.
4. Dangling `ControlId`s were not treatment errors (only implementation/mapping integrity).
5. `FrameworkCapabilities.supports_risk_treatment` was a compile gate. ISO 27001 pack sets the flag. Compile did not evaluate treatment paths.
6. Governance catalog attested `control.risk.treatment`; that is not a treatment engine.

Operational ISMS v1 risk treatment requires first-class Mitigate / Accept / Avoid / Transfer paths whose state, evidence, and approvals are reproducible. An expired acceptance must never keep suppressing treatment. Selecting an enum is not evidence.

Questions this decision answers:

1. Where does the engine live — IR, framework compile, Kleene applicability, or a GRC sidecar?
2. How do the four strategies share one state machine without shortcuts that skip evidence?
3. What makes risk acceptance governance evidence, and when does it expire?
4. How is target residual stored without residual-risk calculation or a collector `RiskRating` enum?
5. How do we cite controls and remediations without owning control-implementation registry / remediation-engine schemas?
6. How do we avoid forking `Risk` or risk methodology scoring?

## Decision

This is what shipped. Field-level law is [`docs/specs/risk-treatment.md`](../specs/risk-treatment.md). Product home: `weeping-angel-assurance-ir::risk_treatment` (`crates/weeping-angel-assurance-ir/src/risk_treatment.rs`), re-exported from that crate’s `lib.rs`.

### 1. Treatment is an IR engine, not a compile step

`AssessmentDefinition` has an additive `risk_treatments: Vec<RiskTreatmentDecision>` (`serde(default)`, skip empty). Schema stays `assurance-ir/v1`. Empty inventory remains valid; `Risk::new` and golden `tests/fixtures/assurance-ir/v1/risk.json` stay four-key compatible.

Incorrect: evaluating plans inside `compile_framework`; a parallel GRC crate; overloading `Exception` as acceptance; putting strategy enums on collectors.

`supports_risk_treatment` remains a **capability gate**. This slice does not interpret framework applicability or project SoA.

### 2. One state machine; evidence is strategy-specific

```text
proposed → approved → executing → verification → completed
+ cancelled | superseded
```

`TreatmentState::can_transition` is the table. `RiskTreatmentDecision::transition` is the only legal writer; illegal pairs return `TreatmentError::InvalidTransition` (no panic). All four strategies walk the happy path. There is no `Approved → Completed` shortcut.

Active states: `Proposed` | `Approved` | `Executing` | `Verification`. Terminal for that id: `Cancelled` | `Superseded`. `Completed` is inactive for “active path” queries but may still transition to `Superseded`.

| Strategy | Completion predicate |
| --- | --- |
| Mitigate | `plan` with ≥ 1 `required` action, all required actions `Done`; cited `ControlId` / `ControlImplementationId` resolve |
| Accept | Sealed `RiskAcceptance` (principal + non-empty rationale + `expiresAt` + ≥ 1 evidence) |
| Avoid | `avoidEvidence` demonstrating the organizational action happened |
| Transfer | `transferEvidence.contract` non-empty + non-empty `transferee` (`String`) |

Partial mitigation cannot complete (`IncompleteActions`). Missing transfer contract cannot complete (`MissingContractEvidence`). Enum assignment is never sufficient.

### 3. Acceptance is immutable and time-bounded

`RiskAcceptance` is sealed: `digest` is `canonical_digest` of the body with `digest` cleared. After `Proposed → Approved`, `sealed_acceptance_digest` freezes the whole record. Further body edits fail `TreatmentError::ImmutableAcceptance`. Evolution is a new decision that supersedes this one, plus a new `RiskAcceptance` id.

`expiresAt` is required (`validFrom < expiresAt`). **In-force acceptance is the only Accept suppressor:** parent state `Completed`, not `Cancelled`/`Superseded`, and `validFrom ≤ as_of < expiresAt`. `RiskStatus::Accepted` is a register label, not evidence.

Clockless `AssessmentDefinition::validate()` rejects `Accepted` with **no** acceptance record. Clocked `validate_treatments_at(assessment, as_of)` rejects `Accepted` when `acceptance_in_force` is false (expired or missing). `treatment_required(..., as_of ≥ expiresAt) == true` even if `Risk.status == Accepted`.

### 4. Target residual is a frozen claim, not calculated here

v1 stores `TargetResidualRisk::VersionedPlaceholder { methodologyVersion, inputNote? }`. `methodologyVersion` is a non-empty pin string, **not** a rating enum. `score_risk` remains risk methodology; this slice does **not** store `MethodologyScored` and does **not** recompute residual from control tests.

At `Proposed → Approved` the engine records `approved_target_residual_digest`. Completing with a different `canonical_digest` of `targetResidual` fails `TargetResidualMismatch`. Completed ≠ residual zero ([ADR 0003 residual risk](0003-residual-risk.md)).

No crate-wide `enum RiskRating { High }`. Collectors must not emit treatment types.

### 5. Consume `Risk` / scoring; reference impls and remediations

- **One `Risk` record** (risk register). Optional `Risk.treatment_id` must resolve in this inventory: equal the **active** decision if one exists, else the latest `Completed` id. Do not invent `RiskV2`.
- **One scoring model** (risk methodology). Do not hardcode 5×5 here.
- Mitigation may list existing `ControlId` / `ControlImplementationId` plus opaque `RemediationRef` (`validate_stable_id` only; no `Remediation` document required).
- Dangling canonical control or implementation ids are **treatment** validation errors (`dangling control reference` / `dangling implementation`).
- Reuse `PrincipalRef`. Typed ids: `RiskTreatmentId`, `RiskAcceptanceId`, `TreatmentPlanId`, `TreatmentActionId` (shared `RiskTreatmentId` with the register).
- At most one active decision per `RiskId`.

Query / validate APIs:

```text
TreatmentState::can_transition
RiskTreatmentDecision::{propose, transition, validate}
active_treatment(assessment, risk_id)
acceptance_in_force(assessment, risk_id, as_of)
treatment_required(assessment, risk_id, as_of)
validate_treatment_inventory(assessment)          # clockless; hooked from IR validate
validate_treatments_at(assessment, as_of)         # expiry vs Accepted
```

### 6. Dual-suite is explicit, not auto-discovered

Root `Cargo.toml` lists `sdd_risk_treatment_baseline` / `sdd_risk_treatment_target` (`tests/contracts/risk_treatment.{baseline,target}.rs`). Human SSOT is the spec; traces go to `.sdd/`.

## Non-goals

- Residual-risk math; control-implementation registry schema expansion; remediation tickets; operational SoA; risk identification candidates.
- Compile-time treatment execution; catalog TOML / ISO id rewrites; UI; `assurance-ir/v2`.
- Wiring `TargetResidualRisk` to `score_risk` in this slice.

## Consequences

- Open risks can carry an auditable treatment path without pretending enum assignment is governance.
- The register can point `treatment_id` at a real inventory; fail-closed `Some(treatment_id)` is satisfiable.
- Residual risk can pin a plan version without this slice claiming effectiveness.
- Neighbor `0005-*` drafts stay owners of scoring and the register; this file owns treatment only.

## Related

- Spec: [`docs/specs/risk-treatment.md`](../specs/risk-treatment.md)
- Methodology: [`docs/adr/0005-risk-methodology.md`](0005-risk-methodology.md)
- Register: [`docs/adr/0005-operational-risk-register.md`](0005-operational-risk-register.md)
- Residual: [`docs/adr/0003-residual-risk.md`](0003-residual-risk.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
