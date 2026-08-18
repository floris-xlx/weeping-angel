# SDD run: Subject Population Runtime and Coverage Semantics

| Field | Value |
| --- | --- |
| Run id | `sdd-a31ed4f6ab18` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `a31ed4f6ab181648` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Population resolution, subject-aware evaluation, coverage metrics, missing-subject semantics, complete `CoverageAtLeast` (Prompt 03). No provider discovery, ISO coverage packs, org-graph, or catalog schema redesign. |
| Spec | [`docs/sdd/population-runtime.md`](population-runtime.md) |
| ADR | [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/03-population-runtime.md`](../prompts/canonical-assurance-v1/03-population-runtime.md) |
| Telemetry | [`sdd-population-runtime-telemetry.json`](sdd-population-runtime-telemetry.json) |
| Dual-suite | `tests/sdd/population_runtime.baseline.rs` (placeholder characterization skip-superseded) · `tests/sdd/population_runtime.target.rs` (active; goldens + index/perf) |
| Base SHA (characterization) | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** Subject Population Runtime and Coverage Semantics
- **Problem:** Assurance tests can only ask whether one evidence envelope exists. `CoverageAtLeast` is a placeholder, so a 50-repo inventory with 47 passes, 2 fails, and 1 missing subject cannot be reported as four distinct numbers; absence can be hidden inside a pass rate.
- **Current behavior (pre-population, SHA 5fa3a23):** `TestExpr::CoverageAtLeast` ignores selector/evidence/percentage and always returns `Effectiveness::PartiallyEffective` with rationale `subject coverage remains partial unless the threshold is met`. `Count` exists in the AST but eval falls through to `NotTested`. `CountWhere` / `AllSubjects` / `AnySubject` / `NoneSubjects` / `CoverageExactly` / `MissingSubjects` do not exist. `ControlTestResult` has no `population` / `evaluated` / `passing` / `failing` / `missing` / `coverage` / `failingSubjects` / `missingSubjects`. `EvidenceSet` is `BTreeMap`-by-digest; `first_selector` is a linear scan. `evaluate_compiled` never attaches `TestExpr` (`CompiledTest` is required/break_on only). Two `SubjectSelector` types: IR `{kind,ids,tags,scope}` is SSOT; control-test owns a competing `{kind,id}`. `Exception` has no subject binding. Facade `AssessmentScope` is an `AssetId` allow-set, not IR `AssessmentScope`. `catalog/` is missing at the planning SHA (Prompts 01/02 later landed and were consumed, not redefined).
- **Desired behavior:** A deterministic `Population {selector, subject_ids, authoritative, observed_at, completeness}` is resolved from IR inventory/scope, a closed `selector.ids` set, or fixture inventory evidence; unknown completeness cannot yield strong all-subject `Effective`. IR `SubjectSelector` is the only selector (control-test aliases/adapts). Narrow `SubjectKind` extensions cover branch / application / database / cloud account / cloud resource / service account / endpoint / data store / network / deployment. `Count`, `CountWhere`, `AllSubjects`, `AnySubject`, `NoneSubjects`, `CoverageAtLeast`, `CoverageExactly`, and `MissingSubjects` evaluate for real. `CoverageAtLeast` uses pessimistic/optimistic pass-rate bounds and emits `PopulationEvaluation` (`population`, `evaluated`, `passing`, `failing`, `missing`, `coverage`, `failingSubjects`, `missingSubjects`). Missing, failing, stale, excepted, and technical failure stay distinct. Zero population is never `Effective` without explicit applicability. Evaluation indexes by evidence type and subject; latest/`supersedes` wins; subject lists are lexicographic.
- **ADR:** needed — accepted at [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md)

### Acceptance criteria (this slice)

1. Dual-suite `sdd_population_runtime_baseline` / `sdd_population_runtime_target` registered in root `Cargo.toml`.
2. Baseline GREEN on current placeholder/linear/no-population behavior before feature code.
3. Target RED on current code for the goldens, real `CoverageAtLeast`, authoritative vs unknown, zero-pop not `Effective`, latest/supersede, deterministic order, and index/perf fixtures.
4. After implement: target GREEN; baseline ignored/superseded so the placeholder is not CI-required.
5. IR `SubjectSelector` remains SSOT; control-test does not keep a competing long-term type.
6. Required conceptual subject kinds are expressible via narrow IR extensions only.
7. Population completeness is first-class; unknown completeness cannot yield strong all-subject `Effective`.
8. `Count` / `CountWhere` / `AllSubjects` / `AnySubject` / `NoneSubjects` / `CoverageAtLeast` / `CoverageExactly` / `MissingSubjects` evaluate (no `NotTested` fall-through).
9. `CoverageAtLeast` computes real coverage; placeholder rationale is gone.
10. Results expose `population` / `evaluated` / `passing` / `failing` / `missing` / `coverage` / `failingSubjects` / `missingSubjects` (nested object allowed).
11. Missing evidence, technical failure, stale evidence, and explicit fail remain distinct.
12. Approved subject-bound exceptions are represented; unbound `Exception` does not silently pass a population.
13. Indexes avoid `O(subjects × all_evidence)` as the evaluation algorithm; fixtures for 100 / 1k / 10k subjects and 100k envelopes.
14. Domain catalogs can declare the four handoff sentences without provider types in `TestExpr`.
15. No provider discovery, ISO-specific coverage, org-graph, or catalog schema redesign; control-test stays network-free.

### Out of scope

- Provider discovery / inventory collectors
- ISO-specific coverage rules or pack redesign
- Organization graph beyond population resolution
- Canonical catalog schema (Prompt 01)
- Typed evidence fact bags (Prompt 02; rebase when landed)
- Collapsing facade `AssessmentScope` and IR `AssessmentScope` into one type
- Script-host `TestExpr` (Rhai/Lua/JS)
- Certification or compliant language
- Redesign of `Control`, `Requirement`, or `Mapping`

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Second `SubjectSelector` becomes permanent unless IR is enforced as SSOT | IR selector is SSOT; control-test is a thin `{kind,id}` adapter (ADR 0003). |
| Prompt 01/02 landing mid-slice if this slice redefines catalog or typed facts | Slice consumed landed catalog + typed `EvidenceValue`; did not redefine those contracts. |
| Unknown population treated as the observed set, hiding missing in-scope subjects | Unknown completeness cannot yield strong all-subject `Effective` (target golden). |
| Missing subjects hidden behind a single pass-rate percentage | `PopulationEvaluation` exposes population/evaluated/passing/failing/missing/coverage plus subject lists. |
| Exception without subjects excepting the entire inventory | Subject-bound exceptions only; unbound `Exception` does not silently pass a population. |
| Zero population accidentally evaluating as `Effective` | Zero-pop is never `Effective` without explicit applicability. |
| Facade vs IR `AssessmentScope` confusion | Types remain distinct; not collapsed this slice. |
| Quadratic evaluation on 10k-subject inventories | `EvidenceIndex` by type and subject; 100 / 1k / 10k / 100k fixtures in target suite. |
| `evaluate_compiled` continuing to drop `TestExpr` | Facade attaches `CompiledTest.expr`; target asserts evaluation. |
| Serde break on thin `{kind,id}` selector JSON | Adapter preserves thin JSON; IR remains SSOT. |
| Workspace `cargo fmt --check` already red on main | Unrelated rustfmt must not ship as this feature; pre-existing hygiene stayed out of scope. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/population-runtime.md`](population-runtime.md) |
| Baseline | PASS on old | `cargo test --test sdd_population_runtime_baseline -- --nocapture` → exit 0. Characterization at SHA `5fa3a23`. Baseline: **16 passed; 0 failed**. Dual command also ran: baseline 16 passed; target FAILED 1 passed / 18 failed on the first target draft. Excerpt: `coverage_at_least_is_placeholder_partially_effective ... ok`; `count_exists_in_ast_but_evaluates_to_not_tested ... ok`; `test result: ok. 16 passed; 0 failed; 0 ignored`. |
| Target pre | FAIL on old | `cargo test --test sdd_population_runtime_baseline --test sdd_population_runtime_target -- --nocapture` → exit 1. Target suite rewritten to pin desired semantics. Baseline stays GREEN (**16 passed**). Target: **FAILED. 1 passed; 35 failed** (registration only). Right reasons: placeholder rationale still `"subject coverage remains partial unless the threshold is met"`; `golden_50_of_50_passing` left `PartiallyEffective` right `Effective`; `golden_47_pass_2_fail_1_missing` population metric `None` vs `Some(50)`; `population_arms_deserialize_and_evaluate` unknown variant `CountWhere`. |
| Implement | target PASS | Same dual command after IR kinds, `Exception.subjects`, `Population` / completeness, `EvidenceIndex`, real coverage arms, and `PopulationEvaluation`. Target: **36 passed; 0 failed**. Re-run confirmed. Baseline still holds as dual-suite registration because placeholder characterization is `#[ignore]` after supersede (**1 passed; 15 ignored**). Goldens 50/50, 47/3, 47/2/1, unknown, stale, exceptions, zero-pop, dupes, supersede, order; perf 100 / 1k / 10k all ok. |
| Baseline post | FAIL or retired | Behavior changed (real `CoverageAtLeast`, population arms, metrics). Pre-supersede: **FAILED. 5 passed; 11 failed** (`CoverageAtLeast` `Inconclusive` vs `PartiallyEffective`; `Count` `Effective` vs `NotTested`; population arms exist; result has population; `SubjectKind::Branch`; `Exception.subjects`; `evaluate_compiled` attaches expr). Skip-supersede (`supersede_kind=skip`): keep `tests/sdd/population_runtime.baseline.rs` registered; **1 passed; 0 failed; 15 ignored** (`#[ignore = "superseded by sdd_population_runtime_target"]`). Placeholder characterization is not CI-required. Not additive. |
| Supersede | target still PASS | After skip-supersede: target **36/36** still GREEN. Baseline is skip-retired (1 registration pass, 15 ignored). Dual-suite registration remains the only required-green baseline test. |
| Docs/ADR | updated | [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md), [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`docs/sdd/population-runtime.md`](population-runtime.md), [`docs/sdd/sdd-population-runtime.md`](sdd-population-runtime.md), [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md), [`README.md`](../../README.md) |

### Planning-SHA placeholder (superseded)

```text
TestExpr::CoverageAtLeast { selector, evidence, percentage }
  => discard all three
  => Effectiveness::PartiallyEffective
  => rationale = "subject coverage remains partial unless the threshold is met"

Count => NotTested ("unsupported expression arm")
CountWhere / AllSubjects / AnySubject / NoneSubjects / CoverageExactly / MissingSubjects => absent
ControlTestResult => no population/evaluated/passing/failing/missing/coverage/failingSubjects/missingSubjects
```

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | `skip` |
| `baseline_retired` | `true` |
| `additive_baseline` | `false` |
| `baseline_not_green` | `true` |
| `target_still_green` | `true` |

`verify_ok` = `target_still_green` ∧ (`baseline_retired` ∧ `baseline_not_green` ∨ `additive_baseline`) = **true**.

---

## What landed

Subject population runtime on `assurance-ir/v1` + network-free control-test (Prompt 03):

- IR `SubjectKind` extensions (branch / application / database / cloud account / cloud resource / service account / endpoint / data store / network / deployment).
- `Exception.subjects` — approved subject-bound exceptions; unbound exception does not silently pass a population.
- Deterministic `Population` with completeness; unknown completeness cannot yield strong all-subject `Effective`.
- IR `SubjectSelector` is SSOT; control-test thin `{kind,id}` adapter only.
- Real `Count` / `CountWhere` / `AllSubjects` / `AnySubject` / `NoneSubjects` / `CoverageAtLeast` / `CoverageExactly` / `MissingSubjects` (no `NotTested` fall-through).
- `CoverageAtLeast` uses pessimistic/optimistic pass-rate bounds; placeholder rationale is gone.
- `PopulationEvaluation` on results: `population`, `evaluated`, `passing`, `failing`, `missing`, `coverage`, `failingSubjects`, `missingSubjects`.
- Missing, failing, stale, excepted, and technical failure stay distinct; zero population is never `Effective` without explicit applicability.
- `EvidenceIndex` by evidence type and subject; latest/`supersedes` wins; lexicographic subject lists.
- `evaluate_compiled` attaches `CompiledTest.expr`.
- Domain catalogs can declare the four handoff sentences without provider types in `TestExpr`.
- Control-test stays network-free. No provider discovery, ISO-specific coverage, org-graph, or catalog schema redesign.

### Files changed (implement)

`crates/weeping-angel-assurance-ir/src/subject.rs`, `crates/weeping-angel-assurance-ir/src/identity.rs`, `crates/weeping-angel-assurance-ir/src/asset.rs`, `crates/weeping-angel-assurance-ir/src/exception.rs`, `crates/weeping-angel-control-test/src/expr.rs`, `crates/weeping-angel-control-test/src/lib.rs`, `crates/weeping-angel-control-test/src/population.rs`, `crates/weeping-angel-control-test/src/result.inc`, `crates/weeping-angel-control-test/src/run.inc`, `crates/weeping-angel-framework/src/lib.rs`, `crates/weeping-angel-assurance/src/lib.rs`, `tests/sdd/population_runtime.baseline.rs`, `docs/sdd/population-runtime.md`, `docs/sdd/sdd-population-runtime.md`, `docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`, `docs/contracts/assurance-runtime.md`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-a31ed4f6ab18` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 12 109 130 |
| `duration_ms_sum` | 7 257 517 (~121.0 min) |
| `budget.total` | 48 |
| `budget.spent` | 8 |
| `budget.remaining` | 40 |
| `event_count` | 29 |
| `max_iters` | 3 |
| `iters_used` | 0 |
| `dry_run` | false |
| `no_delta` | false |

### Gates (final snapshot)

| Gate | Value |
| --- | --- |
| `baseline_green` | true |
| `target_red` | true |
| `target_green` | true |
| `baseline_superseded` | true |
| `dry_run` | false |
| `no_delta` | false |

### Agents

| Phase | Label | Success | Duration (ms) | Tokens |
| --- | --- | --- | --- | --- |
| Scope | `sdd-scope` | ok | 788 394 | 362 505 |
| Spec | `sdd-spec` | ok | 1 530 449 | 1 735 180 |
| BaselineGreen | `sdd-baseline-green` | ok | 400 962 | 282 775 |
| TargetRed | `sdd-target-red` | ok | 927 933 | 1 050 687 |
| Implement | `sdd-implement` | ok | 3 048 348 | 7 297 632 |
| DocsAdr | `sdd-docs-adr` | ok | 385 756 | 1 057 620 |
| Iterate | `sdd-baseline-post-check` | ok | 64 827 | 96 125 |
| Supersede | `sdd-supersede` | ok | 110 848 | 226 606 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-population-runtime-telemetry.json`](sdd-population-runtime-telemetry.json).

---

## Remaining backlog (not this slice)

1. Provider discovery / inventory collectors
2. ISO-specific coverage rules or pack redesign
3. Organization graph beyond population resolution
4. Canonical catalog schema (Prompt 01; already a sibling slice — do not redefine here)
5. Typed evidence fact bags (Prompt 02; rebase when landed)
6. Collapsing facade `AssessmentScope` and IR `AssessmentScope` into one type
7. Script-host `TestExpr` (Rhai/Lua/JS)
8. Certification or compliant language
9. Redesign of `Control`, `Requirement`, or `Mapping`
10. Fixing pre-existing workspace rustfmt / clippy failures (already red on `main`; not this feature)

---

## Summary

Subject population runtime and coverage semantics landed under dual-suite SDD: spec + accepted ADR 0003, baseline GREEN on SHA `5fa3a23` (16 passed on the `CoverageAtLeast` placeholder and linear no-population evaluator), target RED (1 passed / 35 failed for the right reasons), then target GREEN 36/36. Placeholder characterization is skip-superseded (`#[ignore]`; default baseline 1 registration pass / 15 ignored) so the stub is not CI-required. IR `SubjectSelector` is SSOT; completeness, real `CoverageAtLeast` bounds, `EvidenceIndex`, subject-bound exceptions, and `PopulationEvaluation` are first-class. Control-test stays network-free. No provider discovery, ISO coverage packs, org-graph, or catalog schema redesign.
