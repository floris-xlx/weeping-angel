# ADR 0003 — Subject population runtime and coverage semantics

| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-18 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “`CoverageAtLeast` is unfinished / placeholder `PartiallyEffective`” reading of [ADR 0002](0002-iso-27001-assurance-vertical.md) control-test semantics. Does **not** supercede envelope immutability, collector blindness, or catalog schema ownership. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md) |
| Spec | [`docs/sdd/population-runtime.md`](../sdd/population-runtime.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Prompt | [`docs/prompts/canonical-assurance-v1/03-population-runtime.md`](../prompts/canonical-assurance-v1/03-population-runtime.md) |
| Planning baseline | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |
| IR revision consumed | `assurance-ir/v1`. Catalog schema and typed `EvidenceValue` are consumed, not forked. |
| Tests | `sdd_population_runtime_target` GREEN (36). `sdd_population_runtime_baseline` characterization `#[ignore]` after proven FAIL (11). |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0001 delivered a provider-blind assurance spine. ADR 0002 delivered the first ISO 27001 vertical, including a bounded `TestExpr` AST with a `CoverageAtLeast` arm.

On SHA `5fa3a23…`, `CoverageAtLeast` discarded selector, evidence, and percentage and always returned `PartiallyEffective`. `Count` existed but evaluated to `NotTested`. Control-test owned a second, thinner `SubjectSelector { kind, id }` beside the IR SSOT `{ kind, ids, tags, scope }`. `Exception` could not bind subjects. Facade `AssessmentScope` is an `AssetId` allow-set, not IR scope. `evaluate_compiled` never attached a `TestExpr`.

Canonical Catalog v1 Prompt 03 requires first-class subject populations and real coverage so tests can evaluate an entire in-scope set. Absence of evidence must not become a pass unless the runtime knows the authoritative population.

Questions this decision answers:

1. Which `SubjectSelector` is canonical?
2. How does the evaluator know a population is complete?
3. What does `CoverageAtLeast` actually compute?
4. How are missing, failing, stale, excepted, and technical subjects kept distinct?
5. How do we stay efficient at inventory scale without a provider discovery layer?

## Decision

This is what shipped.

### 1. IR `SubjectSelector` is SSOT; control-test keeps a thin adapter

Do not create a third selector type. IR `weeping-angel-assurance-ir::SubjectSelector` remains the stored/inventory type.

Control-test keeps `weeping-angel-control-test::SubjectSelector { kind: Option<String>, id: Option<String> }` as a **serde adapter** for existing `{ kind, id }` JSON. `From` / `to_ir()` folds `id` into `ids` and sets `scope` to `AnyOf` when ids are present, else `All`. Provider-shaped selectors remain forbidden.

### 2. Narrow kind extensions

Added only kinds the IR could not already name:

| Type | Variants added |
| --- | --- |
| `SubjectKind` | `Branch`, `Application`, `Database`, `CloudAccount`, `CloudResource`, `ServiceAccount`, `Endpoint`, `DataStore`, `Network`, `Deployment` |
| `IdentityKind` | `ServiceAccount` |
| `AssetKind` | `Branch`, `Deployment` |

`DataStore` stays distinct from `Dataset`. No provider kinds.

### 3. Population completeness is first-class

```text
Population { selector, subject_ids, authoritative, observed_at, completeness }
completeness = Authoritative | Partial | Unknown
```

`authoritative == true` iff completeness is `Authoritative`. Subject ids are unique and sorted lexicographically.

**Resolution order** (`resolve_population`):

1. `EvidenceSet.explicit_population()` when the caller supplies a closed set.
2. Non-empty selector `ids` with scope other than `NoneOf` → **Authoritative** closed set.
3. Identity inventory (`evidence.identity.inventory`, plus `evidence.identity.privileged-membership` / `evidence.identity.service-account`) → **Authoritative** when an org-level envelope marks `authoritative=true`; otherwise **Partial**.
4. Fixture inventory: `inventory.subject` members + `inventory.complete` with `authoritative=true` for that kind → **Authoritative**. Members without complete → **Partial**.
5. Else infer subjects from observation envelopes of the test’s evidence type → **Unknown**. Never treat that inferred set as the full in-scope population.

Unknown completeness cannot yield strong all-subject `Effective` (`AllSubjects`, `NoneSubjects`, `CoverageExactly`, `CoverageAtLeast(100%)`, `MissingSubjects`). Partial completeness on those arms is `InsufficientEvidence`. Existential `AnySubject` may still succeed when at least one subject passes.

IR `AssessmentScope` / `AssessmentDefinition` inventories are **not** resolved inside the control-test crate. Facade `AssessmentScope` remains the collector allow-set. Callers who have an authoritative inventory inject it via `EvidenceSet::set_population`.

### 4. Coverage uses pessimistic / optimistic bounds

`coverage` is observation completeness (`evaluated / P`) when `P > 0` and completeness is not `Unknown`; otherwise it is omitted (never a fake `0.0` / `1.0`).

- `P` = `|subject_ids|` minus excepted subjects (default: exceptions leave the denominator).
- `evaluated` = `passing + failing` (fresh observations that rendered pass/fail).
- `pessimistic_pass_rate` = `passing / P`.
- `optimistic_pass_rate` = `(passing + missing + stale) / P`.

`percentage` is a decimal **percent** in `[0, 100]` (`"95"`, `"95%"`). Unit-interval `"0.95"` is 0.95% (fail-closed parse, not 95%).

`CoverageAtLeast(t)` (`t = percentage/100`):

| Condition | Effectiveness |
| --- | --- |
| completeness `Unknown` | `Inconclusive` |
| completeness `Partial` and `t == 1` (or other strong arm) | `InsufficientEvidence` |
| authoritative `P == 0` | `InsufficientEvidence` (**never** `Effective`) |
| `optimistic < t` | `Ineffective` |
| `pessimistic < t ≤ optimistic` | `InsufficientEvidence` |
| stale decides (`failing=missing=technical=0`, threshold would pass if stale were fresh-pass) | `StaleEvidence` |
| `pessimistic ≥ t` and no residual fail/stale/technical | `Effective` |
| `pessimistic ≥ t` and `t < 1` with residual failures | `Effective` (threshold explicitly permits failures) |

`CoverageExactly` succeeds iff `round4(pessimistic) == round4(t)`. `AllSubjects` is 100% coverage plus: any fail → `Ineffective`; missing without fail → `InsufficientEvidence`; stale without fail/missing → `StaleEvidence`.

### 5. Distinct partitions; exceptions bind subjects

Per-subject outcomes are disjoint: passing, failing, missing, stale, excepted, technical.

`Exception.subjects: Vec<IR SubjectSelector>` (default empty). Empty does **not** mean the entire inventory. Only `Approved` and unexpired exceptions apply. `scope: All` with empty ids is the explicit “all of this kind” carve-out.

Unclassified field values and type mismatches are **technical**, not missing. A privileged-identity fail whose subject is marked break-glass (`evidence.identity.inventory.account_kind = break-glass`) may surface as `ExceptionApproved` when every failing subject is break-glass.

When `conclude` would otherwise return `Effective` solely because approved unexpired bound IR exceptions removed subjects from the denominator (`excepted` non-empty; `failing` / `missing` / `stale` / `technical` empty), overall effectiveness is `ExceptionApproved`, not silent `Effective`. Same IR `Exception` type; no second engine. Shipped for Prompt 08 honesty and applies to every population family.

### 6. Existing `TestExpr`, dedicated evaluation detail

Completed/added arms on the existing enum (no script host): `Count`, `CountWhere`, `AllSubjects`, `AnySubject`, `NoneSubjects`, `CoverageAtLeast`, `CoverageExactly`, `MissingSubjects`.

`ControlTestResult.population: Option<PopulationEvaluation>` (`skip_serializing_if = None`):

```text
population, evaluated, passing, failing, missing, coverage?,
failingSubjects, missingSubjects, staleSubjects, exceptedSubjects, technicalSubjects
```

Subject id lists are sorted. `Count` does not attach this object; the other population arms do.

### 7. Index by evidence type and subject

`EvidenceIndex` is built once per `evaluate`: `by_type`, `by_type_and_subject`, `latest`. Latest selection: follow `supersedes` to a non-superseded leaf, then max `collectedAt`, then digest lexicographic order. Digest-keyed `EvidenceSet` already collapses identical duplicates.

Forbidden as the steady-state algorithm: `O(|subjects| × |all envelopes|)` nested scans.

### 8. Expression attachment on compile

`CompiledTest.expr` is an optional JSON `TestExpr` side-table (packs still do not own expression bodies). Facade `evaluate_compiled` deserializes it onto `CompiledControlTest`. Callers may also set `CompiledControlTestBuilder::expr` directly.

### 9. Non-goals remain non-goals

No provider discovery, no ISO-specific coverage rules, no organization graph, no catalog schema redesign.

## Consequences

- Domain catalogs can declare population tests (`all privileged identities have MFA`, `≥95% of endpoints encrypt`) without provider types in `TestExpr`.
- Thin `{kind,id}` JSON remains valid; new documents should prefer IR `{kind,ids,tags,scope}`.
- Missing subjects can no longer disappear into a pass rate: 47 passing / 2 failing / 1 missing reports all four numbers.
- Public contract and README must describe real coverage, not the placeholder.
- Downstream catalog slices consume this runtime; they must not reimplement `CoverageAtLeast`.

## Status

Accepted after target GREEN. Placeholder `CoverageAtLeast` is retired. Baseline characterization is ignored so CI does not require the stub.
