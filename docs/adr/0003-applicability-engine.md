# ADR 0003 — Generic Kleene applicability engine over IR rules

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_applicability_engine_target` GREEN; public types frozen in `weeping-angel-assurance::applicability` |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “`resolve_applicability` is a static `Never` filter and SoA is pack-boolean” *operational* reading of [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) compile stage 2 **for org-context evaluation**. Does **not** supercede IR declarativeness, pack schema, collector blindness, or Prompt 11 persist/explain ownership. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [population](0003-subject-population-runtime-and-coverage-semantics.md), [catalog](0003-canonical-assurance-catalog-v1.md) |
| Spec | [`docs/sdd/applicability-engine.md`](../sdd/applicability-engine.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — update on accept |
| Prompt | [`docs/prompts/canonical-assurance-v1/10-applicability-engine.md`](../prompts/canonical-assurance-v1/10-applicability-engine.md) |
| Characterization | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests (planned) | `sdd_applicability_engine_baseline` GREEN on static-only behavior; `sdd_applicability_engine_target` RED then GREEN |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0001 put `resolve_applicability` in the compile pipeline. The IR grew `ApplicabilityRule` / `ApplicabilityPredicate` on every `Requirement` and `Control`. Prompt 03 shipped subject populations. Prompt 11 reserved `ApplicabilitySnapshot` persist shape and currently characterizes Prompt 10 as absent.

On SHA `e430980c…`:

1. IR applicability is **declarative** — `statically_applicable(): Option<bool>` only. Predicates are always `None`.
2. Compile keeps a requirement unless that fold is `Some(false)`.
3. SoA rereads ISO `applicability.toml` booleans — a second, framework-shaped path.
4. Inventories and `AssessmentScope` exist; nothing builds org context from them.
5. Unknown facts are never named. Zero subjects are not distinguished from non-applicability.

Canonical Catalog v1 Prompt 10 requires a **generic** evaluator that decides Applicable / NotApplicable / ManualDeterminationRequired for both canonical controls and framework requirements, without provider or framework special cases.

Questions this decision answers:

1. Where does evaluation live — IR, framework compile, assurance, or control-test?
2. What is the truth table when a fact is unknown?
3. How is organization context represented without a second inventory?
4. How do applicability and population compose, including the empty-set case?
5. What snapshot does lineage persist, and who writes vs stores it?

## Decision (draft — implement will freeze signatures)

### 1. IR stays declarative; the engine lives beside it

`weeping-angel-assurance-ir` does **not** evaluate platform facts. `statically_applicable` remains the no-context Kleene fold over `Always`/`Never`/combinators.

The product home is a new applicability module in `weeping-angel-assurance` (`applicability/{context,evaluator,snapshot}` or equivalent). It **consumes** IR types. Control-test may apply the selected subject set through existing `Population` / `EvidenceSet::set_population`. It must not grow `TestExpr` arms for predicates.

Incorrect: turning IR into a fact engine; a parallel org-graph crate; ISO-only `resolve_iso_applicability`.

### 2. Kleene three-state; unknown is not false

```text
FactValue              = True | False | Unknown
ApplicabilityDecision  = Applicable | NotApplicable | ManualDeterminationRequired
```

`All` / `Any` / `Not` follow Kleene K3. **`Not(Unknown)` remains `Unknown`.** A missing personal-data fact makes `ProcessesPersonalData(true)` `ManualDeterminationRequired`, never `NotApplicable`.

Empty `All` is `True`; empty `Any` is `False` (same as today’s static fold).

### 3. Context is a derived view of existing IR inventory

`ApplicabilityContext` is built from `AssessmentDefinition` (`assets`, `identities`, `vendors`, `processing_activities`, `risks`) + IR `AssessmentScope` / `ScopeExclusion` + optional explicit tri-state facts for attributes thin records cannot store (`Risk` has no level; `ProcessingActivity` has no category).

Use `Asset.tags` and explicit facts. Do not invent a competing inventory if those types can express the fact. Per-family completeness defaults to **Unknown** when a list is empty and unmarked — empty is not authoritative false.

No collector calls. No pack-TOML fact loading.

### 4. Selected scope constrains populations; cardinality is not the decision

A control may apply to a subset of subjects. Preserve reasons and the selected id list (lex-sorted). Hand the set to Prompt 03 via `set_population`.

**Zero selected subjects ≠ `NotApplicable`** unless a predicate/rule evaluates false. `Always` on an empty inventory is still `Applicable` with an empty selected set. Downstream tests stay fail-closed (never `Effective` on empty authoritative populations).

### 5. One engine; snapshot is the lineage handoff

The same `evaluate_applicability` runs on `Requirement.applicability` and `Control.applicability`. No `FrameworkProfile`, collector id, or annex branch.

`evaluate_assessment_applicability` fills Prompt 11’s reserved document:

```text
ApplicabilitySnapshot {
  schema, assessment_id, scope,
  requirement_decisions[], control_decisions[],
  pack_entries[], digest
}
```

This slice **produces** the snapshot. Prompt 11 **persists** it and projects explain. `pack_entries` may carry pack rows as artifacts; they are not Kleene inputs.

Compile, when given a context, drops only `NotApplicable` requirements. `ManualDeterminationRequired` stays in-scope for review. Without a context, today’s `statically_applicable != Some(false)` filter remains.

### 6. Rationale is deterministic

Outcomes carry ordered rationale, predicate traces, unknown facts, selected subjects, and exclusion reasons sufficient to answer why X applied, why Y did not, which fact was unknown, and which exclusion removed Z. Ordering is preorder + lexicographic ids. Digest uses existing IR canonical digest helpers.

## Consequences

- Reviewers can explain applicability without reading compile filters or pack TOML.
- Prompt 12 / SoA can consume a generic three-state instead of inventing an ISO evaluator.
- Prompt 11 baseline absence asserts (`struct ApplicabilitySnapshot`, `ManualDeterminationRequired`) will fail once this lands — the lineage run skip-supersedes those characterization tests and persists the snapshot this engine produces.
- Prompt 09 collector work stays isolated; this engine never calls providers.
- SoA boolean projection remains until Prompt 11/12 switch to the snapshot — two paths exist briefly; only the engine is normative for IR rules.

## Non-goals (this ADR)

Framework-specific branches; provider APIs; ontology engines; catalog redesign; explain CLI; ledger APIs; collapsing facade vs IR `AssessmentScope`.
