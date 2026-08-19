# ADR 0003 — Control implementation registry (organizational state ≠ effectiveness)

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_control_implementation_registry_target` GREEN (CIR-001–015); baseline skip-superseded after it failed on the new contract. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Extends the existing IR `ControlImplementation` contract. Does **not** supercede IR-008/009, envelope immutability, catalog ownership, or Kleene applicability. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [population](0003-subject-population-runtime-and-coverage-semantics.md), [assessment lineage](0003-assessment-lineage.md) |
| Spec | [`docs/specs/control-implementation-registry.md`](../specs/control-implementation-registry.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | Dual-suite `sdd_control_implementation_registry_baseline` (skip-superseded) / `sdd_control_implementation_registry_target` GREEN at `tests/contracts/control_implementation_registry.{baseline,target}.rs` (registered in root `Cargo.toml`). Neighbor `sdd_compliance_ir_target` IR-008/009/019/020 stays GREEN. |

> Filename `0003-*` is shared with catalog-program siblings. **0004** is documentation architecture. Cite this decision by **path**.

## Context

The IR already separates canonical `Control` from organizational `ControlImplementation` (IR-008) and `ImplementationStatus` from `Effectiveness` (IR-009). On SHA `6e31bf1` that split is incomplete:

1. `ImplementationStatus` is `{NotImplemented, Planned, PartiallyImplemented, Implemented, NotApplicable, Retired}` — no disabled/ineffective-as-state, no unknown.
2. Fields stop at owner, description, `implemented_at`, unused `applies_to`, compensating controls, exception ids, risk ids. No assets, review, evidence *expectations*, documents, treatments, automation, or supersession.
3. `validate_assessment_ir` rejects dangling `control_id` / `risk_ids` / `exception_ids` only. Overlapping selectors and dangling subjects/assets are silent, so coverage can double-count.
4. `explain_control` first-matches one implementation by `control_id`.
5. Readers can treat `status: implemented` as “the control is effective.”

Operational ISMS v1 control-implementation registry requires an explicit, operational registry of **how this organization implements** a control, without encoding Annex A, provider APIs, or evidence conclusions on that record.

Questions this decision answers:

1. Do we create a new type or extend `ControlImplementation`?
2. May `Implemented` imply `Effective`?
3. How are multiple implementations of one control scoped without double-counting?
4. How do material changes keep history?
5. Which crate validates integrity, and what stays out of `weeping-angel-evidence`?

## Decision

This is what shipped. Field-level law is [`docs/specs/control-implementation-registry.md`](../specs/control-implementation-registry.md).

### 1. Extend the existing type

`ControlImplementation` in `crates/weeping-angel-assurance-ir/src/implementation.rs` is the SSOT registry row. There is no competing type. Schema remains `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`).

### 2. Organizational state is not effectiveness

`ImplementationStatus` stays a different type from `Effectiveness`. Additive serde only:

- keep `notImplemented`, `planned`, `partiallyImplemented`, `implemented`, `notApplicable`, `retired`
- add `ineffective` (accept-alias `disabled`) and `unknown`
- never add `effective` to this enum
- never put an `effectiveness` field on `ControlImplementation`

Effectiveness is produced only by control tests. A row with `status = Implemented` plus `Effectiveness::Ineffective` remains ineffective (CIR-008).

Coverage-active statuses: `Planned`, `PartiallyImplemented`, `Implemented`, `Ineffective`, `Unknown`. Non-covering: `NotImplemented`, `NotApplicable`, `Retired`. A row with `superseded_by` set is not coverage-active regardless of status.

### 3. One control, many implementations — two-dimensional overlap

Several rows may share a `control_id` when populations (`SubjectSelector` / `applies_to`) or systems (`asset_ids`) are disjoint.

Empty `applies_to` is a **universal population**. Empty `asset_ids` is a **universal asset set**. `overlap_report` / `validate_assessment_ir` fail closed only when **both** axes collide (or are universal). That is what allows CIR-001 (split populations, empty assets) and CIR-006 (split systems, empty selectors) while still rejecting CIR-007 (intersecting selectors with universal assets, intersecting assets with universal selectors, or two fully universal rows).

Reuse IR `AssessmentScope`, `ScopeExclusion`, `SubjectSelector`, `Asset`, `Identity`, `Vendor`. Do not reimplement scope engine `ScopeResolution`.

### 4. Additive operational fields

The row supports: stable id, canonical control id, owner, description, state, scoped subjects, systems/assets, `implemented_at` plus `effective_from`, `implementation::ReviewCadence` (`intervalDays`) and `next_review`, evidence expectation **refs** (`EvidenceRequirementId`), opaque `DocumentRef` (not `ControlledDocument`), risk ids, treatment ids (`Vec<String>` until treatment identity is unified), exception ids, `ImplementationAutomation` (`manual` / `automated` / `hybrid`), compensating controls, supersession (`supersedes` / `superseded_by` / `superseded_at`).

Crate-root `ReviewCadence` remains the **risk-register** type (`interval_seconds`). Implementation review uses `weeping-angel-assurance-ir::implementation::ReviewCadence`.

`owner` and `description` stay `Option` and are **not** fail-closed when missing, so `tests/fixtures/assurance-ir/v1/control-implementation.json` still deserializes and validates as a single-row fixture.

Material changes create a new id that `supersedes` the prior snapshot. Authors write both sides (`superseding(prior)` on the successor; `superseded_by` / `superseded_at` on the prior). The validator does **not** auto-complete `superseded_by`. The prior row stays queryable via `implementation_by_id`.

### 5. Integrity lives in `validate_assessment_ir`

Fail closed on:

- dangling control / compensating control / subject / asset / identity-owner / risk / exception / evidence-expectation / supersession refs
- dangling `treatment_ids` **only when** `assessment.risk_treatments` is non-empty
- duplicate implementation ids
- missing `evidence_expectations` on `Implemented` / `PartiallyImplemented`
- missing **Required** control `EvidenceRequirement` refs on those same statuses
- missing review cadence **and** `next_review` on `Implemented` / `PartiallyImplemented`; cadence present ⇒ `intervalDays ≥ 1` and `next_review` present
- two-dimensional coverage overlap (error names both ids, `control_id`, reason, selector/asset summary)
- supersession cycles

Queries live in `weeping-angel-assurance-ir::registry` (re-exported): `implementations_for`, `current_implementations_for`, `implementation_by_id`, `overlap_report`. `weeping-angel-evidence` remains conclusion-free.

### 6. Lineage pin stays

`ControlExplanation.implementation: Option<ControlImplementation>` and first-match-by-`control_id` keep compiling. This ADR does not change lineage snapshot schemas.

### 7. Golden JSON stays loadable

`tests/fixtures/assurance-ir/v1/control-implementation.json` (`status: "implemented"`) deserializes via `#[serde(default)]` on new fields.

## Consequences

- Registry authors can split populations and systems without a second IR document type.
- Overlap errors are explicit; coverage math must not silently union intersecting selectors **on the same asset set**.
- `Implemented` + test `Ineffective` remains ineffective.
- Risk-treatment types are not forked here; treatment and document refs stay ids / opaque `DocumentRef`.
- Collision fence: residual-risk, GitHub collector, catalog/ISO remap, Kleene evaluator, catalog SDD suites, UI.

## Non-goals

ISO Annex A on the record; provider APIs; evidence conclusions; residual-risk engine; full scope engine; SoA; dashboards.

## Related

- Spec: [`docs/specs/control-implementation-registry.md`](../specs/control-implementation-registry.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
- Neighbor IR: `tests/contracts/compliance_ir.target.rs`
