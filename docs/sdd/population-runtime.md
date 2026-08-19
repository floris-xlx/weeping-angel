# SDD: Subject Population Runtime and Coverage Semantics

| Field | Value |
| --- | --- |
| Status | **Implemented** (target GREEN; baseline superseded) |
| Program | Canonical Assurance Catalog v1 — Prompt 03 |
| Source prompt | [`docs/prompts/canonical-assurance-v1/03-population-runtime.md`](../prompts/canonical-assurance-v1/03-population-runtime.md) |
| Dual-suite | `sdd_population_runtime_baseline` (superseded placeholder; `#[ignore]`) · `sdd_population_runtime_target` (normative coverage semantics) |
| ADR | Accepted [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Spine (still law) | [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (do not fork) | [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Planning baseline | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |
| IR revision consumed | `assurance-ir/v1` as shipped on that SHA |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for Prompt 03. Prompts 01 (catalog infrastructure) and 02 (typed evidence) have landed; this slice consumes those contracts and does not redefine them. Historical §3 is the planning-SHA characterization (`5fa3a23`); §12 records what shipped.

---

## 1. Problem / user-visible goal

Assurance tests can ask whether **one** evidence envelope exists. They cannot evaluate an **in-scope population**.

`TestExpr::CoverageAtLeast` is a placeholder: it ignores selector, evidence, and percentage and always returns `Effectiveness::PartiallyEffective`. `Count` is in the AST but evaluates to `NotTested`. There is no population resolver, no authoritative-vs-unknown completeness, no per-subject pass/fail/missing/stale split, and no way for a domain catalog to say “all privileged identities have MFA” without provider logic.

**Architectural law:** absence of evidence must never become positive evidence unless the runtime knows the authoritative population and can prove the observation covers it.

Example the runtime must distinguish — all four numbers, never “94% passing” while hiding the missing subject:

```text
50 in-scope repositories
47 branch-protection observations passing
2 observations failing
1 repository missing evidence
```

**User-visible goal:** domain catalog prompts can declare, without provider-specific logic:

```text
all privileged identities have MFA
100% of non-archived repositories protect default branch
no critical vulnerability exceeds SLA
at least 95% of endpoints report encryption enabled
```

and receive deterministic effectiveness plus population detail (`population`, `evaluated`, `passing`, `failing`, `missing`, `coverage`, `failingSubjects`, `missingSubjects`).

---

## 2. Compatibility / dependencies

Pinned at planning SHA `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `SubjectSelector` (IR) | `weeping-angel-assurance-ir::subject` | **SSOT.** `{ kind, ids, tags, scope }`. Do not grow a second selector type. |
| `SubjectSelector` (control-test) | `weeping-angel-control-test::expr` | Competing thin type `{ kind: Option<String>, id: Option<String> }`. Adapt, alias, or replace — do not fork a third. |
| `SubjectKind` | IR | `Organization`, `Asset`, `Repository`, `Service`, `Identity`, `User`, `PrivilegedIdentity`, `Device`, `Vendor`, `Dataset`, `ProcessingActivity` |
| `AssetKind` | IR | Already has `Application`, `Database`, `CloudAccount`, `CloudResource`, `Endpoint`, `Network`, `Repository`, … |
| `IdentityKind` | IR | `User`, `Service`, `Team`, `Role`, `Other` — no `ServiceAccount` |
| `AssessmentScope` (IR) | `assessment.rs` | `{ organizations, subjects, exclusions }` |
| `AssessmentScope` (facade) | `weeping-angel-assurance` | **Different type:** `BTreeSet<AssetId>` allow-set for collectors. Do not collapse names without an adapter. |
| `Exception` | IR | `{ id, controlId, rationale, status, approvedBy, expiresAt }` — **no subject binding** |
| `PlannedControlTest` | IR | `subjects: Vec<SubjectSelector>`; `evaluation: TestEvaluationRef { id }` only — no `TestExpr` payload |
| `CompiledTest` | framework | `{ id, controlId, kind, required, breakOn }` — no expression |
| `evaluate_compiled` | assurance facade | Builds `CompiledControlTest` from required/break_on only; **never attaches `expr`** |
| `EvidenceSet` | control-test | `BTreeMap<digest, EvidenceEnvelope>`; `first_selector` is a linear scan |
| `EvidenceEnvelope` | evidence | Has `supersedes`; `seal` leaves it `None`; ledger can `supersede` |
| `EvidenceObservation` | evidence | `BTreeMap<String, EvidenceValue>` facts (`evidence-value/v1`; Prompt 02 landed) |
| `catalog/` | repo root | **Missing.** Prompt 01 not landed. Do not invent catalog schema. |
| Typed evidence | Prompt 02 | Landed. Compare stored `EvidenceValue`; do not reintroduce `parse_fact`. |

Tiny allowed adjustments: type alias / `From` between the two `SubjectSelector`s; optional fields on `Exception` and `AssessmentContext` with serde defaults; extra `TestExpr` arms in the existing enum; a dedicated evaluation-detail object. Do not redesign catalog schema, IR `AssessmentDefinition`, or collector discovery.

---

## 3. Current behavior (baseline on planning SHA)

Characterized against `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b`. The baseline suite **must stay GREEN** on this behavior until the target suite is GREEN and the baseline is superseded.

### 3.1 `CoverageAtLeast` is a placeholder

```259:268:crates/weeping-angel-control-test/src/lib.rs
        TestExpr::CoverageAtLeast {
            selector,
            evidence,
            percentage,
        } => {
            let _ = (selector, evidence, percentage);
            NodeOut {
                effectiveness: Effectiveness::PartiallyEffective,
                rationale: "subject coverage remains partial unless the threshold is met".into(),
            }
        }
```

Any fixture — 50/50 pass, 0 subjects, unknown inventory — yields the same `PartiallyEffective` string.

### 3.2 Incomplete `TestExpr` evaluation

Present in the AST (`expr.rs`): `Exists`, `Missing`, comparisons, `Contains` / `NotContains`, `In`, `Count`, `FreshWithin`, `CoverageAtLeast`, `All` / `Any` / `None` / `Not`, `ManualReview`.

Evaluated today: `Exists`, `Missing`, numeric/eq comparisons (when `ValueExpr::Field`), `FreshWithin`, `CoverageAtLeast` (placeholder), `All` / `Any` / `None` / `Not`, `ManualReview`.

Fall through to `NotTested` + `unsupported expression arm:`: `Count`, `Contains`, `NotContains`, `In`, and `ValueExpr::Literal` comparison arms.

**Absent from the AST:** `CountWhere`, `AllSubjects`, `AnySubject`, `NoneSubjects`, `CoverageExactly`, `MissingSubjects`.

### 3.3 No population object

There is no `Population { selector, subject_ids, authoritative, observed_at }`. The evaluator never knows whether a subject list is complete.

`first_selector` matches `evidence_type` and, if the thin selector has `id`, `provenance.asset == id`. First hit in `BTreeMap` digest order wins. No latest-by-`(type, subject)`, no `supersedes` walk, no index.

### 3.4 Result shape has no coverage metrics

`ControlTestResult` = `{ testId, controlId, effectiveness, rationale, evidenceRefs, missingEvidence, evaluatedAt, testVersion, inputDigest, duration?, status?, reason? }`.

No `population`, `evaluated`, `passing`, `failing`, `missing`, `coverage`, `failingSubjects`, `missingSubjects`, `staleSubjects`.

`missingEvidence` is a list of **evidence type strings**, not subject ids.

### 3.5 Exceptions cannot bind subjects

`Exception` has no `subjects` / `appliesTo`. Approved exceptions cannot carve selected subjects out of a population.

`ControlImplementation.applies_to` exists but is unused by the evaluator.

### 3.6 Compiler / facade do not carry expressions or IR scope

`evaluate_compiled` never calls `CompiledControlTestBuilder::expr`. `CompiledTest` has no expression field. `PlannedControlTest.evaluation` is an id ref.

Facade `AssessmentScope` is an asset allow-set for collectors, not IR `AssessmentScope` (organizations / subject selectors / exclusions).

### 3.7 Subject kinds vs required conceptual kinds

| Conceptual kind (prompt) | Today |
| --- | --- |
| organization | `SubjectKind::Organization`, `AssetKind::Organization` |
| repository | `SubjectKind::Repository`, `AssetKind::Repository` |
| branch | **missing** |
| application | `AssetKind::Application` only (no `SubjectKind`) |
| service | `SubjectKind::Service`, `AssetKind::Service` |
| database | `AssetKind::Database` only |
| cloud account | `AssetKind::CloudAccount` only |
| cloud resource | `AssetKind::CloudResource` only |
| identity | `SubjectKind::Identity` |
| privileged identity | `SubjectKind::PrivilegedIdentity` |
| service account | **missing** (`IdentityKind::Service` is the nearest) |
| endpoint | `AssetKind::Endpoint` only |
| vendor | `SubjectKind::Vendor` |
| data store | nearest `SubjectKind::Dataset` / `AssetKind::Dataset` |
| processing activity | `SubjectKind::ProcessingActivity` |
| network | `AssetKind::Network` only |
| deployment | **missing** |

### 3.8 Performance

`evaluate` collects `evidence.iter()` into a `Vec` and every node scans it (`O(nodes × envelopes)`). There is no `(evidence_type, subject)` index. No fixtures for 100 / 1k / 10k subjects or 100k envelopes.

---

## 4. Desired behavior

### 4.1 Population resolution

Introduce a deterministic runtime value (name may vary; fields are required):

```text
Population {
    selector: IR SubjectSelector,
    subject_ids: ordered unique ids,
    authoritative: bool,
    observed_at: DateTime<Utc>,
    completeness: Authoritative | Partial | Unknown,
}
```

`authoritative == true` iff completeness is `Authoritative`. Unknown or partial completeness **must prevent** strong universal conclusions (`AllSubjects`, `NoneSubjects`, `CoverageAtLeast(100%)`, `CoverageExactly`).

**Resolution order** (first match that yields a set; completeness as noted):

1. Explicit `Population` supplied on `AssessmentContext` (or equivalent evaluation input) — completeness as marked by the caller.
2. IR `AssessmentScope.subjects` plus `AssessmentDefinition` inventories (`assets`, `identities`, `vendors`, `processing_activities`), minus `exclusions` — **Authoritative** when the definition is the in-scope inventory for that kind.
3. `PlannedControlTest.subjects` / `CoverageAtLeast.selector.ids` when `scope` is `AnyOf` and `ids` is non-empty — **Authoritative** for that closed set.
4. Inventory evidence (test/fixture path; not a provider): envelopes of type `inventory.subject` with facts `kind` + `id`, **and** a fresh `inventory.complete` envelope with fact `authoritative=true` for that kind — **Authoritative**. Inventory members without `inventory.complete` → **Partial**.
5. Otherwise: subjects inferred only from observation envelopes → **Unknown**. Never treat that inferred set as the full in-scope population.

Subject ids are unique, sorted lexicographically (deterministic). Tags on the IR selector filter inventory records that carry matching `tags`. `SelectorScope::NoneOf` excludes listed ids. `SelectorScope::All` with empty ids means “all inventory of this kind”.

Do not build an organization graph beyond this resolution.

### 4.2 One subject selector

IR `SubjectSelector` is the only selector type after this slice.

Control-test must **stop owning a competing struct**. Replace or type-alias `weeping-angel_control_test::SubjectSelector` to the IR type. Provide `From` / serde compatibility so a legacy `{ kind, id }` JSON object folds `id` into `ids`.

Do not introduce `GithubRepositorySelector` or any provider-shaped selector.

### 4.3 Narrow subject-kind extensions

Extend IR **only** where the conceptual list is not already expressible:

| Add to | Variants |
| --- | --- |
| `SubjectKind` | `Branch`, `Application`, `Database`, `CloudAccount`, `CloudResource`, `ServiceAccount`, `Endpoint`, `DataStore`, `Network`, `Deployment` |
| `IdentityKind` | `ServiceAccount` (map `SubjectKind::ServiceAccount` here) |
| `AssetKind` | `Branch`, `Deployment` if subjects of those kinds must live in `assets` |

`DataStore` may remain distinct from `Dataset`. Do not add provider kinds.

### 4.4 Exception subject binding

Narrow IR extension on `Exception`:

```text
subjects: Vec<SubjectSelector>   // default empty
```

An exception applies to a population member when status is `Approved`, `expires_at` is absent or `> now`, and the subject id matches `subjects` (empty `subjects` means control-wide, not population-wide: do not silently except the entire inventory unless the selector is explicit `scope: All` for that kind).

Excepted subjects are listed separately. They are **not** counted as passing observations and **not** counted as failures. They leave the effectiveness denominator unless catalog policy says otherwise (default: exclude from denominator).

### 4.5 Test expression semantics

Keep the bounded `TestExpr` AST. Add/complete arms; do **not** add a script host.

| Arm | Semantics |
| --- | --- |
| `Count { selector, predicate }` | Count **latest** envelopes matching the selector (one per subject after supersede/latest). Compare to `Eq` / `Gte` / `Lte`. Missing population does not invent counts. |
| `CountWhere { selector, evidence, predicate }` | Count subjects whose latest evidence satisfies the field/predicate. |
| `AllSubjects { selector, evidence }` | Universal: every member of an **authoritative** population has a fresh passing observation. |
| `AnySubject { selector, evidence }` | Existential: at least one passing subject. May succeed on a partial/unknown population. |
| `NoneSubjects { selector, evidence }` | Universal negative: no member has a failing observation and none are missing. Requires authoritative population. |
| `CoverageAtLeast { selector, evidence, percentage }` | Compute real coverage. `percentage` is a decimal **percent** string in `[0, 100]` (e.g. `"95"`). Accept `"95%"` as 95. Reject unit-interval `"0.95"` as 0.95% (fail-closed parse). |
| `CoverageExactly { selector, evidence, percentage }` | Same math; success iff pessimistic rate **equals** the threshold (exact, after rounding to 4 decimal places on the ratio). |
| `MissingSubjects { selector, evidence }` | Effective iff authoritative population and `missing == 0`. Always attach missing subject ids. |

Default per-subject outcome from `evidence.field`:

| Latest envelope | Outcome |
| --- | --- |
| none | `missing` |
| older than `AssessmentContext.max_age` or selector freshness | `stale` |
| field unset | `passing` if the arm is existence-only; for `CoverageAtLeast` / `AllSubjects` with a field set, missing field → `missing` |
| field truthy (`true`, `pass`, `protected`, `enabled`, `1`) | `passing` |
| field falsey (`false`, `fail`, `unprotected`, `disabled`, `0`) | `failing` |
| other field value | fail-closed `failing` (type mismatch is technical failure, not missing) |

Technical collector/eval errors (if represented) are **not** missing evidence. They must not be folded into `missingSubjects`.

### 4.6 Coverage arithmetic

Let:

- `P` = `|subject_ids|` (0 if unknown completeness and no explicit ids)
- `passing`, `failing`, `missing`, `stale`, `excepted` as disjoint partitions of `P` when authoritative
- `evaluated` = `passing + failing` (fresh observations that rendered a pass/fail)
- `coverage` = `evaluated / P` when `P > 0` and completeness is not `Unknown`; otherwise `coverage` is **absent** (do not emit `1.0` or `0.0` as a fake ratio)
- `pessimistic_pass_rate` = `passing / P` (excepted excluded from `P'` if policy excludes them)
- `optimistic_pass_rate` = `(passing + missing + stale) / P` for threshold arms (missing/stale could theoretically become passes)

`CoverageAtLeast(t)` where `t` is `percentage/100`:

| Condition | Effectiveness |
| --- | --- |
| completeness `Unknown` | `Inconclusive` (never `Effective`) |
| `P == 0` | see zero-population rule |
| any technical failure distinct from missing | do not classify as missing; do not auto-`Effective` |
| `pessimistic_pass_rate >= t` and `stale == 0` | `Effective` if `failing == 0`, else `PartiallyEffective` when `t < 1.0` and policy allows residual failures; `Effective` is allowed when the threshold explicitly permits failures |
| `optimistic_pass_rate < t` | `Ineffective` (missing cannot save the threshold) |
| `pessimistic < t <= optimistic` | `InsufficientEvidence` |
| `stale > 0` and stale affects the bound | `StaleEvidence` when that is the deciding defect (no fails, threshold would pass if stale were fresh-pass) |

`AllSubjects` is `CoverageAtLeast("100")` plus: any `failing` → `Ineffective`; any `missing` with no fails → `InsufficientEvidence`; any `stale` with no fails/missing → `StaleEvidence`.

**Zero population:** `P == 0` with authoritative empty inventory → `NotApplicable` only when an applicability rule (or explicit test kind) says the control does not apply; otherwise `InsufficientEvidence`. **Never `Effective`.** Unknown completeness with zero observed subjects is `Inconclusive`, not `Effective`.

### 4.7 Latest / superseding / duplicates

For each `(evidence_type, subject_id)`:

1. If a `supersedes` chain exists, follow it to the leaf that is not itself superseded.
2. Else take the envelope with the latest `provenance.collected_at`; break ties by digest lexicographic order (stable).
3. Duplicate identical digests (`EvidenceSet` already keyed by digest) count once.
4. Two envelopes with different digests for the same subject: latest/supersede wins; do not double-count passing.

### 4.8 Evaluation output

Do not force the example JSON onto `ControlTestResult` if a nested object is cleaner. Required: results **or** explanation metadata can produce:

```json
{
  "population": 50,
  "evaluated": 49,
  "passing": 47,
  "failing": 2,
  "missing": 1,
  "coverage": 0.98,
  "failingSubjects": ["repo:a", "repo:b"],
  "missingSubjects": ["repo:c"]
}
```

Recommended shape: `PopulationEvaluation` (or `CoverageBreakdown`) on `ControlTestResult` as `population: Option<PopulationEvaluation>` (serde `skip_serializing_if = None`). Subject id lists are sorted. `coverage` omitted when unknown.

Facade `evaluate_compiled` must attach `TestExpr` when the planned/compiled test has one (wire `evaluation` id → catalog/test body once Prompt 01 lands; until then, allow `CompiledControlTest.expr` to be set by callers and extend `CompiledTest` with an optional expr or a side table). This slice may add `expr` to `CompiledTest` without redesigning the pack schema.

### 4.9 Performance

Build an index keyed by `(evidence_type, subject_id)` (and by `evidence_type` alone) when evaluating population arms. Forbidden: `O(|subjects| × |all envelopes|)` nested scans as the steady-state algorithm.

Ship test fixtures (not live benches required in CI if too heavy) for:

- 100 subjects
- 1,000 subjects
- 10,000 subjects
- 100,000 evidence envelopes

CI target: 10k subjects + indexed lookup must complete in the integration-test budget without quadratic scans. 100k envelopes may be a fixture constructor + index-presence proof rather than a wall-clock bench in default CI.

### 4.10 Compiler / facade

- Consume IR `SubjectSelector` and IR `AssessmentScope` when resolving populations for an assess run.
- Keep facade `AssessmentScope` (asset allow-set) as the collector filter; map allowed assets into inventory when that is the only scope the caller provided (authoritative **only** for those asset ids, kind as given).
- Still never key decisions on `collector_id`.

---

## 5. Effectiveness rules (deterministic summary)

| Situation | Effectiveness | Notes |
| --- | --- | --- |
| Full authoritative population passes | `Effective` | `missing=0`, `failing=0`, `stale=0` |
| Threshold met despite allowed residual failures | `Effective` or `PartiallyEffective` | `CoverageAtLeast` with `t < 1`; still emit failing subjects |
| Population threshold cannot be met even if missing/stale passed | `Ineffective` | Distinct from missing |
| Missing evidence on one or more **known** subjects, and that gap can change the conclusion | `InsufficientEvidence` | Never count missing as pass |
| Population unknown / incomplete for a universal/threshold conclusion | `Inconclusive` | No fake coverage ratio |
| Stale evidence on a subset that decides the result | `StaleEvidence` | Distinct from missing |
| Approved, unexpired, subject-bound exceptions | excepted partition | Default: drop from denominator |
| Zero population | `NotApplicable` or `InsufficientEvidence` | **Never `Effective`** without explicit applicability |
| Technical failure | not `missing` | Keep distinct |

Rank already in `rank()` may be reused for `All`/`Any` composition; population detail is **always** attached when a population arm ran, including when effectiveness is `Ineffective`.

---

## 6. Golden tests (target)

1. **50/50 passing** — authoritative 50, 50 fresh passes → `Effective`, `population=50`, `passing=50`, `missing=0`, `coverage=1.0`.
2. **47/50 passing, 3 explicit failures** — `failing=3`, `missing=0`; `CoverageAtLeast("100")` → `Ineffective`; `CoverageAtLeast("90")` → threshold-pass path with failing subjects listed.
3. **47 passing, 2 failing, 1 missing** — all four numbers present; `coverage=0.98`; `CoverageAtLeast("95")` → `InsufficientEvidence` (optimistic 48/50, pessimistic 47/50); never “94% pass” without `missing=1`.
4. **Unknown / incomplete population** — observations only, no authoritative inventory → `Inconclusive`; no `Effective` all-subject claim; `coverage` omitted or explicitly unknown.
5. **Stale evidence on a subset** — `StaleEvidence` (or stale partition listed) when stale decides; not classified as missing.
6. **Exceptions on a subset** — approved subject-bound exception removed from failing/missing; unapproved/expired does not apply.
7. **Zero population** — not `Effective`.
8. **Duplicated evidence envelopes** — same digest / same subject+type duplicates do not inflate passing.
9. **Latest / superseding selection** — older fail + newer pass (or `supersedes`) → subject counts as passing once.
10. **Deterministic subject ordering** — `failingSubjects` / `missingSubjects` sorted lexicographically; repeat eval equal.

Plus: real `CoverageAtLeast` (placeholder rationale gone); index/perf fixtures 100 / 1k / 10k / 100k; `evaluate_compiled` can attach expressions.

---

## 7. Acceptance criteria

1. Dual-suite registered as `sdd_population_runtime_baseline` / `sdd_population_runtime_target` in root `Cargo.toml`.
2. Baseline GREEN on current placeholder/linear/no-population behavior **before** feature code.
3. Target RED on current code for the 10 goldens, real `CoverageAtLeast`, authoritative vs unknown, zero-pop not `Effective`, latest/supersede, deterministic order, and index/perf fixtures.
4. After implement: target GREEN; baseline ignored/superseded so the placeholder is not CI-required; target still GREEN.
5. IR `SubjectSelector` remains SSOT; control-test does not keep a competing long-term type.
6. Required conceptual subject kinds are expressible (narrow IR extensions only).
7. `Population` completeness is first-class; unknown completeness cannot yield strong all-subject `Effective`.
8. `Count`, `CountWhere`, `AllSubjects`, `AnySubject`, `NoneSubjects`, `CoverageAtLeast`, `CoverageExactly`, `MissingSubjects` evaluate (no `NotTested` fall-through for those arms).
9. `CoverageAtLeast` computes real coverage; placeholder rationale is gone.
10. Evaluation exposes population / evaluated / passing / failing / missing / coverage / failingSubjects / missingSubjects (nested object allowed).
11. Missing evidence, technical failure, stale evidence, and explicit fail remain distinct.
12. Approved subject-bound exceptions are represented; unbound `Exception` does not silently pass a population.
13. Indexes (or ledger query) avoid `O(subjects × all_evidence)` as the evaluation algorithm.
14. Domain catalogs can declare the four handoff sentences without provider types in `TestExpr`.
15. Provider-blind, network-free control-test crate; no ISO-specific coverage rules; no catalog schema redesign; no provider discovery.
16. Workspace verify command stays the three cargo invocations above. Public contracts/docs stay truthful after implement.

---

## 8. Out of scope

- Provider discovery (GitHub/AWS/… inventory collectors)
- ISO-specific coverage rules or pack redesign
- Organization graph beyond population resolution
- Canonical catalog schema (`catalog/canonical/v1`) — Prompt 01
- Typed evidence fact bags — Prompt 02 (rebase when it lands)
- Collapsing facade `AssessmentScope` and IR `AssessmentScope` into one type in this slice (adapter only)
- Script-host `TestExpr`, Rhai/Lua/JS
- Certification / compliant language
- Redesign of `Control`, `Requirement`, `Mapping`

---

## 9. Risks

| Risk | Mitigation |
| --- | --- |
| Second `SubjectSelector` becomes permanent | ADR: IR type is SSOT; alias/adapt the thin type |
| Prompt 01/02 land mid-slice | Rebase; do not redefine catalog or typed facts |
| Unknown population treated as observed set | Completeness enum; universal arms fail-closed |
| Missing subjects hidden behind a pass rate | Mandatory partitions + optimistic/pessimistic bounds |
| `Exception` without subjects excepts everyone | Empty subjects ≠ all inventory |
| Zero population auto-`Effective` | Explicit ban; golden 7 |
| Facade vs IR `AssessmentScope` confusion | Keep both; document adapter |
| Quadratic eval on 10k subjects | Required index + fixtures |
| `evaluate_compiled` still drops `TestExpr` | Extend `CompiledTest` / attach expr |
| Serde break on thin `{kind,id}` | Compatibility fold `id` → `ids` |
| Fmt gate already red on `main` | Record; do not mix unrelated rustfmt of IR into this feature |

---

## 10. Dual-suite protocol

```text
Spec first (this file; no product feature code)
  → Baseline GREEN on CURRENT code
  → Target RED on CURRENT code (right reasons)
  → Implement
  → Docs/ADR finalize if needed
  → Target GREEN
  → Prove baseline FAILS or additive-documented
  → Supersede baseline (prefer skip/ignore so CI does not require the placeholder)
  → Target still GREEN
```

Fail-closed if baseline cannot go green, target cannot go red for the right reason, or target never greens within max iters.

Prefer `#[ignore]` / skip-supersede on the baseline file (keep registration) rather than leaving placeholder assertions as required green.

---

## 11. Handoff / done

Domain catalog prompts (04+) declare population tests against this runtime. They do not implement coverage math.

**Done:** population-aware evaluation is real; `CoverageAtLeast` is not a placeholder; missing / failing / stale subjects are separate; evaluation is efficient enough for realistic inventories; architecture stays provider- and framework-neutral.

---

## 12. Implemented (what shipped)

Target `sdd_population_runtime_target` GREEN (36). Baseline characterization `#[ignore]` after 11 proven FAILs.

Deviations from §4 that are now law (see ADR):

| Spec intent | Shipped |
| --- | --- |
| Replace/alias control-test `SubjectSelector` with the IR type | Thin `{ kind, id }` adapter + `to_ir()` / `From` (no third type) |
| Resolve IR `AssessmentScope` / `AssessmentDefinition` inventories inside evaluate | Not in control-test. Caller injects `EvidenceSet::set_population` |
| `AssessmentContext` carries explicit `Population` | Lives on `EvidenceSet` (`explicit_population`) |
| Zero-pop `NotApplicable` when applicability says so | Authoritative empty → `InsufficientEvidence` only (still never `Effective`) |
| `PopulationEvaluation` required fields | Also emits `staleSubjects`, `exceptedSubjects`, `technicalSubjects` |
| `evaluate_compiled` attaches expr | `CompiledTest.expr` JSON side-table deserialized onto `CompiledControlTest` |

Identity populations additionally resolve from `evidence.identity.inventory` (+ privileged-membership / service-account). Break-glass failing privileged subjects may conclude `ExceptionApproved` when every failing subject is break-glass. Remaining-all-pass sets that are `Effective` only because approved unexpired bound IR exceptions emptied the remainder (`excepted` non-empty; no fail/missing/stale/technical) conclude `ExceptionApproved` (Prompt 08 honesty; not a second exception engine).
