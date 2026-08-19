# ADR 0003 — Generic Kleene applicability engine over IR rules

| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “`resolve_applicability` is a static `Never` filter and SoA is pack-boolean” *operational* reading of [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) compile stage 2 **for org-context evaluation**. Does **not** supercede IR declarativeness, pack schema, collector blindness, or assessment lineage persist/explain ownership. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [population](0003-subject-population-runtime-and-coverage-semantics.md), [catalog](0003-canonical-assurance-catalog-v1.md) |
| Spec | [`docs/specs/applicability-engine.md`](../specs/applicability-engine.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | `sdd_applicability_engine_target` GREEN (P10-T01–T16, 17 passed). `sdd_applicability_engine_baseline` GREEN after skip-superseding absence tests B06/B07/B09 (14 passed, 4 ignored). |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0001 put `resolve_applicability` in the compile pipeline. The IR grew `ApplicabilityRule` / `ApplicabilityPredicate` on every `Requirement` and `Control`. population runtime shipped subject populations. assessment lineage reserved `ApplicabilitySnapshot` persist shape and characterized applicability engine as absent.

On SHA `e430980c…`:

1. IR applicability is **declarative** — `statically_applicable(): Option<bool>` only. Predicates are always `None`.
2. Compile keeps a requirement unless that fold is `Some(false)`.
3. SoA rereads ISO `applicability.toml` booleans — a second, framework-shaped path.
4. Inventories and `AssessmentScope` exist; nothing builds org context from them.
5. Unknown facts are never named. Zero subjects are not distinguished from non-applicability.

Canonical Catalog v1 applicability engine requires a **generic** evaluator that decides Applicable / NotApplicable / ManualDeterminationRequired for both canonical controls and framework requirements, without provider or framework special cases.

Questions this decision answers:

1. Where does evaluation live — IR, framework compile, assurance, or control-test?
2. What is the truth table when a fact is unknown?
3. How is organization context represented without a second inventory?
4. How do applicability and population compose, including the empty-set case?
5. What snapshot does lineage persist, and who writes vs stores it?

## Decision

This is what shipped.

### 1. IR stays declarative; the engine lives in assurance

`weeping-angel-assurance-ir` does **not** evaluate platform facts. `statically_applicable` remains the no-context Kleene fold over `Always`/`Never`/combinators. Tiny IR change: public `Control::subjects() -> &[SubjectSelector]`.

Product home: `weeping-angel-assurance::applicability` (`context.rs`, `evaluator.rs`, `snapshot.rs`). Network-free. No `FrameworkProfile`, collector id, or annex branch. Control-test does not grow `TestExpr` arms; selected ids are handed through existing `Population` / `EvidenceSet::set_population`.

Incorrect: turning IR into a fact engine; a parallel org-graph crate; ISO-only `resolve_iso_applicability`.

### 2. Kleene three-state; unknown is not false

```text
FactValue              = True | False | Unknown
ApplicabilityDecision  = Applicable | NotApplicable | ManualDeterminationRequired
```

`Not(Unknown)` remains `Unknown`. Empty `All` is `True`; empty `Any` is `False`.

| Node | Result |
| --- | --- |
| `Always` | `True` → `Applicable` |
| `Never` | `False` → `NotApplicable` |
| `Predicate` | inferred or explicit fact; missing fact is `Unknown`, never coerced to `False` |
| `All` | `False` if any child is `False`; `True` iff every child is `True`; else `Unknown` |
| `Any` | `True` if any child is `True`; `False` iff every child is `False`; else `Unknown` |
| `Not` | swap True/False; **Unknown stays Unknown** |

`ProcessesPersonalData(true)` with no personal-data fact is `ManualDeterminationRequired`, never `NotApplicable`.

`ApplicabilityDecision::remains_in_compiled_set` is true unless the decision is `NotApplicable`. SoA’s `Applicability::Unresolved` is the projection alias of `ManualDeterminationRequired`. This slice did not change `project_soa`; operational SoA later consumes Kleene snapshots ([`0003-operational-soa.md`](0003-operational-soa.md)).

### 3. Context is a derived view of existing IR inventory

```text
build_applicability_context(definition, extras) -> ApplicabilityContext
```

`ApplicabilityContext` is sliced from `AssessmentDefinition` inventories (`assets`, `identities`, `vendors`, `processing_activities`, `risks`) plus IR `AssessmentScope` / `ScopeExclusion`. `ContextExtras` supplies explicit `FactValue`s, per-family `InventoryCompleteness`, and optional pack-entry artifacts.

Empty family + unmarked completeness is **Unknown**, not authoritative false. Explicit facts win over inferred presence. No collector calls. No pack-TOML fact loading.

### 4. Selected scope constrains populations; cardinality is not the decision

`evaluate_applicability(rule, context) -> ApplicabilityOutcome` returns decision, ordered rationale, predicate traces, named unknown facts, lex-sorted selected subject ids, and exclusion reasons (`id`, `reason`, `exclusion_index`).

Control evaluation intersects `Control.subjects()`. **Zero selected subjects ≠ `NotApplicable`.** `Always` on an empty inventory is `Applicable` with `selected_subjects = []`. Downstream tests stay fail-closed (empty authoritative population is never `Effective`).

### 5. One engine; snapshot is the lineage handoff

The same evaluator runs on `Requirement.applicability` and `Control.applicability`.

```text
evaluate_assessment_applicability(definition, context) -> ApplicabilitySnapshot
```

Schema `weeping-angel/applicability-snapshot/v1` (`APPLICABILITY_SNAPSHOT_SCHEMA`):

```text
ApplicabilitySnapshot {
  schema, assessmentId, scope,
  requirementDecisions[], controlDecisions[],
  packEntries[], digest
}
```

Walks requirements and controls in id lexicographic order. Digest is IR `canonical_digest` over the body excluding the digest field. `pack_entries` are artifacts, not Kleene inputs.

This slice **produces** the snapshot. assessment lineage **persists** it and projects explain. Without a context, compile keeps `statically_applicable != Some(false)`. With a context, callers drop only `NotApplicable`.

### 6. Rationale is deterministic

Preorder walk of the rule tree, then lexicographic unknown-fact keys and excluded subject ids. Same `(rule, context)` yields the same JSON (ignore wall-clock).

## Consequences

- Reviewers can explain why control X applied, why Y did not, which fact was unknown, and which exclusion removed Z.
- ISO remap / SoA consume a generic three-state instead of inventing an ISO evaluator. Pack boolean `project_soa` remains until those slices switch to the snapshot.
- assessment lineage persist/explain consumes this snapshot; lineage absence asserts for `ApplicabilitySnapshot` / `ManualDeterminationRequired` are skip-superseded by the lineage run, not avoided by renaming types.
- GitHub collector collector work stays isolated; this engine never calls providers.

## Non-goals (this ADR)

Framework-specific branches; provider APIs; ontology engines; catalog redesign; explain CLI; ledger APIs; collapsing facade vs IR `AssessmentScope`.

## Status

Accepted after target GREEN. Public types are frozen in `weeping-angel-assurance::applicability`. Baseline absence characterization for B06/B07/B09 is ignored so CI does not require the pre-engine HEAD.
