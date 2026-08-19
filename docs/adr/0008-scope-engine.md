# ADR 0008 — Organizational scope engine (executable ISMS boundary)

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_scope_engine_target` GREEN (SCP-T01–T15); `sdd_scope_engine_baseline` skip-superseded |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The operational reading “IR `AssessmentScope` is descriptive text and silent `ScopeExclusion` rows may drop inventory members.” Does **not** supercede IR schema `assurance-ir/v1`, facade `AssessmentScope` as an `AssetId` collector allow-set, collector `CollectorScope`, crawl URL `src/engine/scope.rs`, Kleene applicability, or ADR 0001 compile stages. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md); consumes [0008-isms-context](0008-isms-context.md) (`IsmsContext` / `Organization` / `BusinessUnit` / `ManagementSystemScope`) |
| Spec | [`docs/specs/scope-engine.md`](../specs/scope-engine.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) (Organizational scope engine) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_scope_engine_target` GREEN (`tests/contracts/scope_engine.target.rs`). `sdd_scope_engine_baseline` `#[ignore = "superseded by sdd_scope_engine_target"]` (ISMS-absence row `scp_b09` skip-superseded by ISMS context IR). |

> Filename **`0008-*`**. Cite **this file by path**. Do **not** add a `0003-scope-engine.md` sibling. Concurrent Operational ISMS siblings also use `0008-*` ([ISMS context](0008-isms-context.md), [interested parties](0008-interested-parties-obligations.md), [security objectives](0008-security-objectives.md)).

## Context

On SHA `6e31bf1a…` the ISMS boundary was a bag of selectors plus optional exclusion text:

1. IR `AssessmentScope` is `{ organizations: Vec<String>, subjects, exclusions }`.
2. `ScopeExclusion` was `{ subjects, optional rationale }` — no owner, approval reference, approved-at, review/expiry, or evidence refs.
3. Applicability `build_applicability_context` synthesized `excluded by assessment scope[{index}]`, so silent exclusions **worked**.
4. There was no `ScopeResolution` and no `InScope | OutOfScope | Conditional | Unknown` quad; no explain traces.
5. Facade `weeping_angel_assurance::AssessmentScope` and `CollectorScope` are `AssetId` allow-sets. Crawl `src/engine/scope.rs` is URL membership. Those names stay.
6. `Exception` expiry already refused to suppress when expired; exclusions ignored time.
7. `resolve_population` did not consult IR `AssessmentScope` inventories.
8. `ManagementSystemScope` (ISMS context IR) is a named handle, not a resolver.

Operational ISMS v1 needs the boundary to be **executable and accountable**: what is inside, why, who approved the carve-out, until when — fail closed on ambiguity.

Questions this decision answers:

1. Where does resolution live (IR vs assurance engine vs collectors)?
2. What is the decision set if not a boolean?
3. How are exclusions governed so silence and expiry cannot shrink the ISMS invisibly?
4. What is precedence, if not vec order?
5. How do facade/collector allow-sets and crawl scope coexist?
6. How does this reuse ISMS context IR without a second GRC graph?

## Decision

This is what shipped. Field-level law is [`docs/specs/scope-engine.md`](../specs/scope-engine.md).

### 1. Engine in assurance; IR stays data; collectors stay blind

`resolve_scope` is a **pure** function in `weeping-angel-assurance::scope` ([`engine.rs`](../../crates/weeping-angel-assurance/src/scope/engine.rs), [`snapshot.rs`](../../crates/weeping-angel-assurance/src/scope/snapshot.rs), [`adapter.rs`](../../crates/weeping-angel-assurance/src/scope/adapter.rs)).

```text
ScopeInputs::from_assessment(&AssessmentDefinition)
    .with_context(&IsmsContext)          // optional
    .with_candidates(Vec<SubjectRef>)    // optional; else all inventory + context members
resolve_scope(&ScopeInputs, as_of) -> Result<ScopeResolution, ScopeError>
resolve_subject(&SubjectRef, &ScopeInputs, as_of) -> SubjectScopeDecision
```

IR `AssessmentScope` / `ScopeExclusion` remain the canonical **document** (`assurance-ir/v1`). Collectors consume `ScopeResolution::to_collector_scope()` (`AssetId` allow-set) and **must not** mutate scope state. Schema mismatch (`definition.schema_version != ASSURANCE_IR_SCHEMA`) is the only `Err(ScopeError::Schema)`; recoverable fail-closed cases are per-subject `Unknown`.

Incorrect: evaluating scope inside `weeping-angel-assurance-ir`; teaching GitHub/AWS collectors ISO; writing a `grc-ir` crate; reusing `src/engine/scope.rs`.

Snapshot schema: `weeping-angel/scope-resolution/v1` (`SCOPE_RESOLUTION_SCHEMA`). Digest is `canonical_digest` of the body excluding `digest`.

### 2. Four-state result; never a silent boolean

```text
ScopeDecision = InScope | OutOfScope | Conditional | Unknown
# serde camelCase: inScope, outOfScope, conditional, unknown
```

Every candidate gets a non-empty rationale, leaf→root lineage, an explain string (`repo:payments -> business-unit:finance -> service:payments -> ISMS scope -> InScope`), and `influencingRules` for every competing or skipped rule. `is_definitely_in_scope` is `(d == InScope)` only.

`Unknown` and `Conditional` are **not** `InScope`. Unknown/contradictory data **never** becomes positive in-scope evidence. Out-of-scope subjects **must not** contribute passing/`Effective` evidence to an in-scope assessment (SCP-T09: population is the `InScope` id set; extra envelopes do not count).

Allow-set adapters include **only** `InScope` asset ids (`to_facade_assessment_scope` / `to_collector_scope`). Identity, vendor, processing-activity, business-unit, location, data-domain, and personnel-population rows are omitted from the asset allow-set.

### 3. Exclusions are accountable records; expired does not suppress

Shipped additive `ScopeExclusion` fields (`#[serde(default)]`, old `{ subjects, rationale? }` still **deserializes**):

```text
rationale, owner: PrincipalRef, approvalRef, approvedAt,
reviewBy and/or expiresAt, evidenceRefs: Vec<stable id>
```

`ScopeExclusion::governance_is_complete` / `is_active_at(as_of)` encode the clock. `AssessmentDefinition::validate()` rejects silent or incomplete exclusions (`validate_scope_exclusions`). Empty default scope still validates.

Only **active** exclusions compete. Invalid / expired / review-overdue rows stay in the trace (`invalidExclusion` / `expiredExclusion`, `applied=false`) and **do not** suppress. Renewal is a later `expiresAt` / `reviewBy` on the canonical record.

Applicability `build_applicability_context` **no longer** synthesizes `excluded by assessment scope[{index}]`; rows with empty rationale are skipped.

### 4. Precedence is a table, not iteration order

Shipped ranks (include and exclude **share** the class value; equal-rank include vs exclude is `Unknown`, independent of vec order):

| Rank | Rule class |
| --- | --- |
| 100 | Exact-id match (`AnyOf` / ids contains id) |
| 80 | Tag match (`SelectorScope::All` with non-condition tags) |
| 60 | Kind-only (`SelectorScope::All` / kind-wide `NoneOf`) |
| 40 | Inherited decision from the nearest ancestor (`Asset.parent` / `BusinessUnit.parentId` / `businessUnit` tag) |
| 30 | Organization-wide inclusion (bound org membership) |
| 0 | No matching rule |

`SelectorScope::NoneOf` on `subjects` competes as an exclude-shaped rule. Duplicate inclusions are idempotent (`canonical_digest` of the selector). Cycles in `parent` / `parentId` → `Unknown` (`cycle in parent chain`). Unresolved ids → `Unknown` (`unresolved subject`). Empty `subjects` **and** empty bound orgs → no implicit “everything in inventory is in scope.”

Conditional: selector tag `scopeCondition` (not used as a membership tag). Winning conditional inclusion → `Conditional`; conditional vs active exclude at the same rank → `Unknown`.

### 5. Reuse inventories and ISMS context IR; do not fork a GRC graph

Scopeable entities are existing `SubjectKind`s plus **generic** additive kinds: `BusinessUnit`, `Location`, `DataDomain`, `PersonnelPopulation` (`parse_name`: `businessunit`, `location`, `datadomain`, `personnelpopulation` / `population`). **No** AWS/GitHub/Entra types.

Nested inclusion walks `Asset.parent`. When `&IsmsContext` is supplied, bind `Organization` / `BusinessUnit` / `ManagementSystemScope.id` (`scopeId` on the snapshot); do not copy those structs into a second module. Unbound `AssessmentScope.organizations` strings do not guess membership.

`ManagementSystemScope` remains a named handle; this engine **resolves** subjects.

`in_scope_population(selector, resolution, definition)` returns only `InScope` member ids (lex-sorted). Completeness is `Unknown` if any matching member is `Unknown`; `Authoritative` when the family was an explicit inclusion or org-wide include; else `Partial`.

### 6. Two other “scope” types remain

| Type | Role after this decision |
| --- | --- |
| IR `AssessmentScope` | Canonical inclusion/exclusion document |
| Facade `AssessmentScope` / `CollectorScope` | `AssetId` allow-set, filled from `InScope` ids |
| `src/engine/scope.rs` | Crawl URLs only |

Do not collapse these types.

## Consequences

- Reviewers can explain `repo:payments -> … -> InScope` with owner and validity window.
- Applicability no longer invents silent exclusion reasons; Kleene logic is unchanged.
- Population receives in-scope ids from this engine (`in_scope_population`) instead of inventing membership from envelopes.
- Neighbor suites stay GREEN; baseline characterizes silent/descriptive scope and is skip-superseded.
- Security-objectives / obligation / register slices may consume a pinned `ScopeResolution`; they do not reimplement precedence.

## Non-goals (this ADR)

Discovery; AWS/GitHub/Entra schemas; Statement of Applicability; implementing `IsmsContext` (owned by [0008-isms-context](0008-isms-context.md)); crawl URL scope; collapsing facade vs IR `AssessmentScope`.

## Related

- Spec: [`docs/specs/scope-engine.md`](../specs/scope-engine.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- ISMS context IR: [ADR 0008-isms-context](0008-isms-context.md)
- Applicability (Kleene unchanged): [ADR 0003-applicability-engine](0003-applicability-engine.md)
- Population: [ADR 0003-subject-population-runtime-and-coverage-semantics](0003-subject-population-runtime-and-coverage-semantics.md)
- Docs layout: [ADR 0004](0004-documentation-architecture.md)
