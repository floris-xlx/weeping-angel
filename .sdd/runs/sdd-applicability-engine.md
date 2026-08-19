# SDD run: Organization Context and Applicability Engine

| Field | Value |
| --- | --- |
| Run id | `sdd-bbd1eb519611` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `bbd1eb519611920e` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 10: Kleene applicability engine over existing IR rules and inventories. Network-free evaluator in `weeping-angel-assurance`; fill Prompt 11 reserved `ApplicabilitySnapshot` without persist/explain. |
| Spec | [`docs/sdd/applicability-engine.md`](applicability-engine.md) |
| ADR | Accepted [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/10-applicability-engine.md`](../prompts/canonical-assurance-v1/10-applicability-engine.md) |
| Telemetry | [`sdd-applicability-engine-telemetry.json`](sdd-applicability-engine-telemetry.json) |
| Dual-suite | `tests/sdd/applicability_engine.baseline.rs` (skip-retired) · `tests/sdd/applicability_engine.target.rs` (active; P10-T01…T16) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Collision fence (Prompt 09) | Do not touch `crates/weeping-angel-collector/**`, `tests/sdd/github_collector.*`, `docs/sdd/github-collector.md`, `GITHUB_EVIDENCE_TYPES` |
| Collision fence (Prompt 11) | Do not implement explain/ledger. Own only engine / evaluator / snapshot paths. |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

Treat the prior Prompt 10 session (never started; 4-run cap) as abandoned. This report is the **finalize** artifact for telemetry run `sdd-bbd1eb519611` against characterization SHA `e430980c…`.

---

## Spec

- **Title:** Organization Context and Applicability Engine
- **Problem:** IR `ApplicabilityRule` / `ApplicabilityPredicate` trees exist on every control and requirement but nothing evaluates them against organization or assessment scope, so the runtime cannot decide Applicable vs NotApplicable vs ManualDeterminationRequired, cannot name unknown facts, and cannot explain selected-subject exclusions.
- **Current behavior (SHA `e430980c`):** The IR module is declarative and does not evaluate platform facts. `statically_applicable` is `Some(true/false)` for Always/Never and boolean combinators and `None` for every Predicate (`Not(None)` stays `None`). Framework `resolve_applicability` keeps a requirement unless `statically_applicable() == Some(false)`; controls are not filtered. `project_soa` rereads ISO pack `applicability.toml` booleans. There is no evaluator module, no `ApplicabilitySnapshot`, and no org-context builder. `AssessmentDefinition` inventories and IR `AssessmentScope` exist but are unused for applicability; population runtime injects subjects via `EvidenceSet::set_population` and does not walk those inventories. Facade `AssessmentScope` is a collector allow-set. Prompt 11 baseline still asserts Prompt 10 absent.
- **Desired behavior:** A network-free generic evaluator in `weeping-angel-assurance` builds `ApplicabilityContext` as a derived view over existing IR inventories, `AssessmentScope`, and explicit tri-state facts (not a second inventory). Kleene evaluation of Always/Never/All/Any/Not/Predicate yields Applicable | NotApplicable | ManualDeterminationRequired plus deterministic rationale, predicate traces, unknown facts, selected subjects, and exclusion reasons. Unknown is not false; `Not(Unknown)` stays unknown; zero selected subjects is not NotApplicable unless the rule is false. The same engine evaluates Requirement and Control rules. `evaluate_assessment_applicability` fills Prompt 11’s reserved `ApplicabilitySnapshot` shape (schema, assessment_id, scope, requirement/control decisions, pack_entries, digest) without persist/explain. Compile may drop only NotApplicable when a context is supplied. No framework/provider branches and no pack-TOML evaluator.
- **ADR:** needed — accepted at [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md)

### Acceptance criteria (this slice)

1. Dual-suite `sdd_applicability_engine_baseline` / `sdd_applicability_engine_target` registered at implement using the same `[[test]]` pattern as population/github/lineage.
2. Baseline GREEN on current static-only/no-evaluator/SoA-boolean behavior; target RED on current code then GREEN after implement.
3. Always→Applicable and Never→NotApplicable with no facts consulted.
4. Known-true predicates→Applicable; known-false→NotApplicable.
5. Unknown predicates including `ProcessesPersonalData(true)` with no personal-data fact→ManualDeterminationRequired, never NotApplicable.
6. Nested All/Any/Not follow Kleene K3; `Not(Unknown)` remains unknown.
7. Jurisdiction, authoritative no-cloud vs unknown cloud, personal-data known/unknown, vendor presence, and explicit exclusions match the spec tables.
8. Rationale, unknown facts, and subject lists are deterministically ordered; snapshot digest is stable.
9. Zero selected subjects + Always remains Applicable with an empty selected set.
10. Same engine for controls and requirements; no FrameworkProfile/collector/ISO branch in the evaluator.
11. Snapshot fields match the lineage persist shape; this slice does not persist, explain, or edit catalog/ISO `applicability.toml`/provider APIs.
12. IR stays declarative (`statically_applicable` unchanged in meaning); selected scope can be injected via existing `EvidenceSet::set_population`.
13. After implement, workspace verify (`cargo test --workspace --features demo`; `fmt --check`; `clippy -D warnings`) holds for files this slice touches; neighbor SDD targets stay GREEN.

### Out of scope

- Framework-specific applicability branches
- Using pack `applicability.toml` as a second Kleene evaluator
- Provider API calls and `crates/weeping-angel-collector/**`
- `tests/sdd/github_collector.*`, `docs/sdd/github-collector.md`, `GITHUB_EVIDENCE_TYPES`
- Generic ontology/description-logic engine
- Canonical catalog TOML redesign
- Prompt 11 explain CLI, ledger persist/load, `ControlExplanation`
- Collapsing facade `AssessmentScope` with IR `AssessmentScope`
- Growing IR Risk/ProcessingActivity/Vendor into full domain models
- Rewriting `statically_applicable` to consult inventories
- Certification/compliant/audit-passed language
- Prompt 12 ISO remap content

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Unknown facts treated as false or `Not(Unknown)` flipped to true | Kleene K3 in evaluator; T03/T13 pin unknown ≠ false and `Not(Unknown)` stays unknown. |
| Second inventory/org-graph instead of a derived IR context | `ApplicabilityContext` is a derived view over existing IR inventories + scope + explicit tri-state facts. |
| IR becoming a fact engine | Evaluator lives in `weeping-angel-assurance`; IR remains declarative. |
| Zero selected subjects auto-mapped to NotApplicable | T12: Always + empty selected set stays Applicable. |
| Prompt 09 collector file collision | Hard fence: no collector / `github_collector.*` / `GITHUB_EVIDENCE_TYPES` edits. |
| Prompt 11 baseline absence asserts vs required `ApplicabilitySnapshot` | Fill reserved persist shape only; no persist/explain. |
| SoA boolean path remaining a silent second evaluator | Unchanged this slice; pack TOML is not a Kleene evaluator. |
| Empty unmarked inventories treated as authoritative false | Completeness defaults to Unknown. |
| Facade vs IR `AssessmentScope` confusion | Types remain distinct; not collapsed. |
| Unrelated workspace fmt/clippy debt mixed into this slice | Only files this slice touches; Prompt 09 collector WIP stays fenced. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/applicability-engine.md`](applicability-engine.md) |
| Baseline | PASS on old | `cargo test --test sdd_applicability_engine_baseline --test sdd_applicability_engine_target -- --nocapture` → exit 0. Characterization SHA `e430980c`. Baseline **18 passed; 0 failed** (static-only IR fold, compile filter `!= Some(false)`, no evaluator/snapshot module). Target harness registered so the dual-suite invocation resolves; P10-T01..T16 desired-behavior tests are not in this baseline run. Excerpt: `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` / `dual_suite_is_registered ... ok` / `test result: ok. 1 passed`. Suites: `tests/sdd/applicability_engine.baseline.rs`, `tests/sdd/applicability_engine.target.rs`. |
| Target pre | FAIL on old | Same dual command → exit 1. Target authored P10-T01..T16 against [`docs/sdd/applicability-engine.md`](applicability-engine.md) §6.2. **RED for the right reason** (no applicability engine/module). Baseline unmodified and GREEN: 18 passed. No product feature code. Excerpt: `error[E0432]: unresolved import weeping_angel_assurance::applicability` → `tests\sdd\applicability_engine.target.rs:13` `could not find applicability in weeping_angel_assurance`; `could not compile weeping-angel (test "sdd_applicability_engine_target")`. Suite: `tests/sdd/applicability_engine.target.rs`. |
| Implement | target PASS | Same dual command after shipping the network-free Kleene engine (`applicability/{mod,context,evaluator,snapshot}.rs`), `Control::subjects` getter, and snapshot fill. Target **17 passed; 0 failed** (P10-T01…T16 + registration). Mid-implement baseline still GREEN after skip-superseding absence tests B06/B07/B09: **14 passed; 4 ignored**. Unknown is not false; snapshot fills Prompt 11 persist shape without persist/explain. Change is additive until supersede: remaining baseline characterization of IR declarativeness, `statically_applicable`, and compile filter still holds. Files: `crates/weeping-angel-assurance/src/applicability/{mod,context,evaluator,snapshot}.rs`, `crates/weeping-angel-assurance/src/lib.rs`, `crates/weeping-angel-assurance-ir/src/control.rs`, `tests/sdd/applicability_engine.baseline.rs`, spec/ADR/contract. |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). Default dual run: baseline **ok. 0 passed; 0 failed; 18 ignored** (message: `superseded by sdd_applicability_engine_target`). Forced `cargo test --test sdd_applicability_engine_baseline -- --ignored --nocapture` → **FAILED. 14 passed; 4 failed** (`p10_b05_project_soa_reads_pack_booleans`; `p10_b06_product_crates_lack_evaluator_and_snapshot`; `p10_b07_no_assurance_applicability_module`; `p10_b09_control_has_no_public_subjects_getter`). Not additive. Characterization of static-only/absence is no longer CI-required. |
| Supersede | target still PASS | After skip-supersede: target **ok. 17 passed; 0 failed; 0 ignored** (P10-T01…T16 + `dual_suite_is_registered`). Baseline file stays registered. `target_still_green=true`. |
| Docs/ADR | updated | [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md), [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`docs/sdd/applicability-engine.md`](applicability-engine.md), [`docs/sdd/sdd-applicability-engine.md`](sdd-applicability-engine.md), [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/sdd/iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md), [`docs/sdd/assessment-lineage.md`](assessment-lineage.md), [`docs/sdd/sdd-assessment-lineage.md`](sdd-assessment-lineage.md), [`README.md`](../../README.md) |

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

Network-free Kleene applicability engine in `weeping-angel-assurance` (Prompt 10):

- Derived `ApplicabilityContext` over existing IR inventories, IR `AssessmentScope`, and explicit tri-state facts (not a second inventory).
- Evaluator for Always / Never / All / Any / Not / Predicate → Applicable | NotApplicable | ManualDeterminationRequired.
- Unknown ≠ false; `Not(Unknown)` stays unknown; zero selected subjects is not NotApplicable unless the rule is false.
- Same engine for Control and Requirement rules; no FrameworkProfile / collector / ISO / pack-TOML branch.
- Deterministic rationale, predicate traces, unknown facts, selected subjects, and exclusion reasons (stable snapshot digest).
- `evaluate_assessment_applicability` fills Prompt 11 reserved `ApplicabilitySnapshot` (`schema`, `assessment_id`, `scope`, requirement/control decisions, `pack_entries`, `digest`) without persist or explain.
- IR stays declarative (`statically_applicable` meaning unchanged); thin `Control::subjects` getter only.
- Selected scope can still be injected via existing `EvidenceSet::set_population`.
- Compile may drop only NotApplicable when a context is supplied.

### Files changed (implement)

`crates/weeping-angel-assurance/src/applicability/mod.rs`, `crates/weeping-angel-assurance/src/applicability/context.rs`, `crates/weeping-angel-assurance/src/applicability/evaluator.rs`, `crates/weeping-angel-assurance/src/applicability/snapshot.rs`, `crates/weeping-angel-assurance/src/lib.rs`, `crates/weeping-angel-assurance-ir/src/control.rs`, `tests/sdd/applicability_engine.baseline.rs`, `docs/sdd/applicability-engine.md`, `docs/sdd/sdd-applicability-engine.md`, `docs/adr/0003-applicability-engine.md`, `docs/contracts/assurance-runtime.md`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-bbd1eb519611` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 10 017 800 |
| `duration_ms_sum` | 5 140 866 (~85.7 min) |
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
| Scope | `sdd-scope` | ok | 197 830 | 343 771 |
| Spec | `sdd-spec` | ok | 941 385 | 1 044 000 |
| BaselineGreen | `sdd-baseline-green` | ok | 444 890 | 2 221 222 |
| TargetRed | `sdd-target-red` | ok | 584 411 | 982 431 |
| Implement | `sdd-implement` | ok | 690 393 | 2 392 656 |
| DocsAdr | `sdd-docs-adr` | ok | 365 511 | 867 643 |
| Iterate | `sdd-baseline-post-check` | ok | 1 434 846 | 1 761 830 |
| Supersede | `sdd-supersede` | ok | 481 600 | 404 247 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-applicability-engine-telemetry.json`](sdd-applicability-engine-telemetry.json).

---

## Remaining backlog (not this slice)

1. Framework-specific applicability branches
2. Using pack `applicability.toml` as a second Kleene evaluator
3. Provider API calls and `crates/weeping-angel-collector/**` (Prompt 09 fence)
4. Generic ontology / description-logic engine
5. Canonical catalog TOML redesign
6. Prompt 11 explain CLI, ledger persist/load, `ControlExplanation`
7. Collapsing facade `AssessmentScope` with IR `AssessmentScope`
8. Growing IR Risk / ProcessingActivity / Vendor into full domain models
9. Rewriting `statically_applicable` to consult inventories
10. Certification / compliant / audit-passed language
11. Prompt 12 ISO remap content
12. SoA boolean path remaining a silent second evaluator (unchanged this slice)

---

## Summary

Prompt 10 applicability engine landed under dual-suite SDD: spec + accepted ADR 0003, baseline GREEN on SHA `e430980c` (18 passed on static-only IR fold / no evaluator), target RED (`unresolved import weeping_angel_assurance::applicability`), then Kleene evaluator + derived IR context + Prompt 11 snapshot shape until target GREEN 17/17. Baseline characterization is skip-retired (18 ignored; forced `--ignored` 4 FAIL on evaluator/module/subjects/SoA absence). Unknown ≠ false; `Not(Unknown)` stays unknown; zero selected subjects is not NotApplicable. Same engine for controls and requirements. No persist/explain, no pack-TOML evaluator, no collector/provider branches. IR stays declarative.
