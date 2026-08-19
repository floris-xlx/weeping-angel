# ADR 0008 — Security objectives IR and pure evidence-backed evaluator

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_security_objectives_target` GREEN (SO-T01–T20); `sdd_security_objectives_baseline` skip-superseded |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine. Does **not** supercede `Control.objective` as prose, governance catalog `control.governance.security-objectives` as an existence attestation, crate-root `isms::SecurityObjective` as a context declaration, crate-root `continuity::ObjectiveStatus`, Kleene applicability, or collector blindness. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (IR as framework-neutral documents + `canonical_digest`; crate graph), [ADR 0004](0004-documentation-architecture.md) (spec/ADR/contract paths), [ADR 0003 typed evidence](0003-typed-evidence-canonical-serialization.md) (`EvidenceValue`), [ADR 0003 assessment lineage](0003-assessment-lineage.md) (pinned snapshots), [ISMS context](0008-isms-context.md), [scope engine](0008-scope-engine.md) |
| Spec | [`docs/specs/security-objectives.md`](../specs/security-objectives.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) (security objectives section) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_security_objectives_target` GREEN (`tests/contracts/security_objectives.target.rs`). `sdd_security_objectives_baseline` `#[ignore = "superseded by sdd_security_objectives_target"]`. Dual-suite registered in root `Cargo.toml`. |

> Filename **`0008-*`**. Cite **this file by path**. Do **not** add a `0003-security-objectives.md` sibling. Catalog-program decisions share `0003-*`; documentation architecture is `0004`. Concurrent Operational ISMS siblings also use `0008-*` ([ISMS context](0008-isms-context.md), [scope engine](0008-scope-engine.md), [interested parties](0008-interested-parties-obligations.md)).

## Context

On SHA `6e31bf1a…`, information-security objectives were not records:

1. `Control.objective` is a `String`. Catalog TOML copies the same prose.
2. `control.governance.security-objectives` is a **manual** `manual-review` attestation that an objective *set exists*.
3. There was no measurable `SecurityObjective`, metric, target, measurement, or status projection in `crates/**`.
4. `EvidenceValue` already supplies typed percentage/count/duration/boolean/decimal/object values and deterministic `typed_eq` / `cmp_numeric`.
5. `EvidenceSnapshot` / `LineageBundle` already pin control-test evidence.
6. IR `AssessmentScope` + `SubjectSelector` exist. Concurrent slices have since landed `ScopeResolution` and `IsmsContext`.
7. `weeping-angel-evidence` depends on IR. IR must not depend on evidence (ADR 0001: IR ↛ any upper crate).

Operational ISMS v1 security objectives must make objectives **measurable first-class records** evaluated from evidence, so management review can reconstruct a status at time T. Without a decision, later slices will:

- treat missing measurements as OnTrack,
- invent a second metric-value enum in IR,
- emit `Achieved` from a collector,
- overload `Effectiveness` with `OnTrack`,
- or embed a formula VM.

Questions this decision answers:

1. Where do objective **records** live versus the **evaluator** and **value type**?
2. How do we reuse `EvidenceValue` without reversing the crate graph or forking a metric type?
3. What is the status algebra, and when is missing/stale/partial evidence allowed to be success?
4. How is every measurement scoped, including a pinned `ScopeResolution`?
5. How does lineage reconstruct a status without live systems?
6. How do crate-root `isms::SecurityObjective` and continuity `ObjectiveStatus` coexist?
7. What is explicitly not this seam (CLI, dashboards, Kleene, collectors, management review)?

## Decision

This is what shipped. Field-level law is [`docs/specs/security-objectives.md`](../specs/security-objectives.md).

### 1. Two-crate public seam; no new crate; no crate-root name collision

| Piece | Home |
| --- | --- |
| `SecurityObjectiveId`, `ObjectiveMetricId`, `ObjectiveTargetId`, `ObjectiveMeasurementId` | `weeping-angel-assurance-ir` `typed_id!` |
| Governance record `objectives::SecurityObjective` (owner, IR `AssessmentScope`, metric/target ids, cadence, dates, lifecycle, evidence-requirement ids) plus `MetricKind`, `ComparisonOp`, `ObjectiveLifecycle`, `PopulationCompleteness`, `ObjectiveMeasurementSource` | [`crates/weeping-angel-assurance-ir/src/objectives.rs`](../../crates/weeping-angel-assurance-ir/src/objectives.rs) |
| Value-bearing `ObjectiveMetric` / `ObjectiveTarget` / `ObjectiveMeasurement` + `evaluate_objective` / `evaluate_objective_with_resolution` | [`crates/weeping-angel-assurance/src/objectives.rs`](../../crates/weeping-angel-assurance/src/objectives.rs) (pure; sibling of `lineage`, **not** of Kleene `applicability`) |
| Lineage document `ObjectiveEvaluationSnapshot` | `weeping-angel-assurance`; schema `weeping-angel/objective-evaluation/v1` (`OBJECTIVE_EVALUATION_SCHEMA`) |

IR crate-root `pub use`s only the enums/source types from `objectives`. It does **not** crate-root `pub use objectives::SecurityObjective`: crate-root `SecurityObjective` remains the **ISMS context declaration** (`isms.rs`, `ObjectiveId`, title/description/owner only). Callers name `weeping_angel_assurance_ir::objectives::SecurityObjective` or `weeping_angel_assurance::objectives::SecurityObjective`.

Assurance `objectives::ObjectiveStatus` is **not** crate-root `pub use`: crate-root `ObjectiveStatus` remains the continuity-resilience verdict. Callers name `weeping_angel_assurance::objectives::ObjectiveStatus`.

Schema for governance records remains `assurance-ir/v1`. No parallel GRC schema. No new crate.

Incorrect: scoring inside Kleene; `Effectiveness::OnTrack`; collector-produced status; moving `EvidenceValue` into IR; collapsing context declarations into the measurable record.

### 2. One value type: `EvidenceValue`

Metric payloads are `EvidenceValue` (percentage, count, duration, boolean, ratio-as-object, bounded numeric). Comparison is `typed_eq` / `cmp_numeric` only. **No `f64`.** **No formula AST.**

IR does **not** define `enum EvidenceValue`, `MetricValue`, or `ObjectiveValue`. On-disk target/measurement bytes are **evidence-value/v1** JSON produced by serializing `EvidenceValue` in the assurance crate. The evaluator deserializes through `EvidenceValue` before compare. IR `SecurityObjective.baseline` is optional JSON (`serde_json::Value`) decoded at evaluation time.

`weeping-angel-assurance-ir` must not gain a dependency on `weeping-angel-evidence`.

### 3. Status algebra; missing is never success

```text
ObjectiveStatus = OnTrack | AtRisk | Missed | Achieved | InsufficientEvidence
```

JSON: `onTrack` | `atRisk` | `missed` | `achieved` | `insufficientEvidence`.

Degradation (missing, stale, partial, unscoped, type/domain error, manual without sealed attestation) **always** yields `InsufficientEvidence` (or `ObjectiveError::NotActive` / `Invalid` for non-Active / invalid records). It **never** yields `OnTrack` or `Achieved`.

After a valid candidate measurement:

- meeting target before/without deadline → `OnTrack`
- meeting target after deadline → `Achieved`
- missing target before/without deadline → `AtRisk`
- missing target after deadline → `Missed`

Ongoing objectives (no deadline) never become `Achieved`/`Missed` from the clock alone.

`as_of` is an argument. `evaluate_objective` is side-effect free: no clock, no I/O, no collector calls.

### 4. Scope is explicit; adapter for the landed scope engine

Every measurement carries IR `AssessmentScope` / `SubjectSelector`. Unscoped or out-of-scope mix → `InsufficientEvidence` (`scopeMismatch`). Facade collector `AssessmentScope` is not this type.

`evaluate_objective_with_resolution` accepts a pinned `ScopeResolution`. Measurement subjects must resolve `InScope`; `Unknown` / `Conditional` / `OutOfScope` never count as in-scope success. This slice does not implement precedence (`resolve_scope` stays in `::scope`).

### 5. Lineage is a snapshot, not a live query

`ObjectiveEvaluationSnapshot` pins objective/metric/target/measurement digests, evidence envelope digests, scope digest (`ScopeResolution.digest` when a resolution is passed, else IR `AssessmentScope`), `asOf`, status, reason codes, and comparison operands. Replay over the pin is byte-stable. Management-review consumers reconstruct without collectors.

Historical evaluations are immutable. A new measurement or clock produces a **new** snapshot.

### 6. Neighbors stay in their lanes

- `Control.objective` prose stays.
- Governance catalog attestation control stays; it is not the SLA engine.
- Collectors remain fact-only.
- `isms::SecurityObjective` remains a **declaration** on `IsmsContext` (`ObjectiveId`). The engine record is `objectives::SecurityObjective` (`SecurityObjectiveId`).
- Kleene applicability is not invoked.
- No dashboards, notifications, formula VM, CLI, or management-review workflow.

## Consequences

- Dual-suite `sdd_security_objectives_baseline` / `sdd_security_objectives_target` is registered under `tests/contracts/` in root `Cargo.toml`. Baseline is skip-superseded; target is law.
- Public contract pointer lives in [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md).
- Management review and continual improvement consume `ObjectiveEvaluationSnapshot`; they do not re-query collectors or treat catalog attestation as an SLA.
- Concurrent ISMS context / scope engine / obligations slices consume ids and the evaluator; they do not reimplement comparison.
- Fixture law: `tests/fixtures/assurance-ir/v1/security-objective-vuln-sla.json` (critical-vuln 7-day SLA, target ≥ 98%). 98% is fixture data, not an evaluator constant.

## Related

- Spec: [`docs/specs/security-objectives.md`](../specs/security-objectives.md)
- Tests: `tests/contracts/security_objectives.{baseline,target}.rs`
- Typed evidence: [`docs/specs/typed-evidence.md`](../specs/typed-evidence.md)
- Lineage: [`docs/specs/assessment-lineage.md`](../specs/assessment-lineage.md)
- Scope engine: [`docs/specs/scope-engine.md`](../specs/scope-engine.md)
- ISMS context: [`docs/specs/isms-context.md`](../specs/isms-context.md)
- Governance catalog neighbor: [`docs/specs/governance-canonical-assurance-catalog.md`](../specs/governance-canonical-assurance-catalog.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
