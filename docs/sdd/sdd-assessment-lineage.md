# SDD run: Immutable Assessment Lineage, Explainability, and Report Cleanup

| Field | Value |
| --- | --- |
| Run id | `sdd-94837bf9b18c` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `94837bf9b18c3978` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 11: persistable immutable execution lineage, `ControlExplanation`, pure report serialization, generic framework loader, snapshot compare, ledger persist/load |
| Spec | [`docs/sdd/assessment-lineage.md`](assessment-lineage.md) |
| ADR | Accepted [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/11-assessment-lineage.md`](../prompts/canonical-assurance-v1/11-assessment-lineage.md) |
| Telemetry | [`sdd-assessment-lineage-telemetry.json`](sdd-assessment-lineage-telemetry.json) |
| Dual-suite | `tests/sdd/assessment_lineage.baseline.rs` (skip-retired) · `tests/sdd/assessment_lineage.target.rs` (active; LIN-001–015) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Collision fence | Prompt 09 GitHub collector files · Prompt 10 evaluator reimplementation · Prompt 12 ISO pack ID remap / catalog domain TOML |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

This report is the **finalize** artifact for telemetry run `sdd-94837bf9b18c` against characterization SHA `e430980c…`. Dual-suite was already registered before this run; baseline encoded current shortcuts; remaining LIN locks went RED before product feature code.

---

## Spec

- **Title:** Immutable Assessment Lineage, Explainability, and Report Cleanup
- **Problem:** Operators cannot reconstruct why an assessment said what it said. `AssessmentRun` exists as a type and empty SQLite tables, but `assess` builds one, copies the compile digest into every identity field, records no collector runs, and drops it (`let _run`). JSON reports reload `frameworks/iso-27001/2022` while serializing; non-ISO profiles compile a production stub; SoA rereads live `applicability.toml`; compare only notices effective/ineffective/stale; there is no explain command. Changing current catalog or pack files can silently change what a stored report appears to say.
- **Current behavior (SHA `e430980c…`):** Encoded by `sdd_assessment_lineage_baseline`. `AssuranceEngineBuilder::assess` constructs `AssessmentRun` as `let _run` with `collector_runs: Vec::new()`, status always `completed`, and `assessment_definition_digest` / `evidence_snapshot_digest` / `result_digest` all `compiled.digest.clone()`; pack digest is hardcoded `load_framework_pack("iso-27001", "2022")`. `AssessmentReport::serialize` loads that same ISO pack, invents `automationCoverage` / `evidenceCoverage` percent strings, and drops `collectionRunId` / `evidenceRefs`. `assessment_for_target` / `normalize` / `stub_catalog` special-case ISO 27001:2022; other profiles compile `canonical:stub-1` / `assess-runtime-1`. `compare` only fills effective/ineffective/stale. `project_soa` reads live `applicability.toml`. CLI `AssuranceCommand` has no `Explain`; non-catalog arms banner-and-exit-0. Ledger creates `assessment_runs` / `control_test_runs` / `framework_snapshots` with no persist/load APIs. No `ControlExplanation`, `CoverageMetrics`, or snapshot persist types. Dual-suite is registered; baseline encodes these shortcuts; target only asserted LIN-009/LIN-015 before this run's TargetRed phase.
- **Desired behavior:** An assessment is a reproducible immutable execution artifact. Persist `FrameworkPackSnapshot`, `CanonicalCatalogSnapshot`, `AssessmentDefinitionSnapshot`, `ApplicabilitySnapshot` (from `ApplicabilityRule` + `statically_applicable` + pack rows; unknown predicates stay unresolved, never false), `CollectionRun[]`, `EvidenceEnvelope[]` (append-only), `EvidenceSnapshot`, `ControlTestRun[]`, `AssessmentRun`, `FrameworkReadinessSnapshot`, and `StatementOfApplicabilitySnapshot`. Return and persist `AssessmentRun` with start/completion, scope, `completed`/`partial`/`failed`, collector ids/versions/run ids, and distinct pack/catalog/definition/evidence-snapshot/applicability/result pins. Replay from pins; consulting current files must detect `DigestMismatch`. `ControlExplanation` plus `weeping-angel assurance explain --assessment <id> --control <id>` cites exact evidence digests, population, missing evidence, failing/missing subjects, test id/version, exceptions, and mappings. `AssessmentReport` serialization is pure (no pack load, network, filesystem, or hidden current-state resolution). `CoverageMetrics` expose seven separate families with no single compliance percentage. One `(id, version)` loader path; production stub removed. `compare` identifies applicability, subjects, evidence add/remove/supersession, test results, exceptions, and digest changes. Digests are SHA-256 of canonical JSON, not reused compile digest, excluding `duration`/`evaluatedAt`.
- **ADR:** needed — accepted at [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md)

### Acceptance criteria (this slice)

1. Dual-suite registered; baseline GREEN on current shortcuts; LIN-001–008 and LIN-010–014 authored so target is RED on CURRENT before product feature code; after implement target GREEN and baseline skip-superseded.
2. `AssessmentRun` is returned/persisted (never `let _run`) with start/completion, scope, `completed`/`partial`/`failed`, collector runs, and distinct pack/catalog/definition/evidence-snapshot/applicability/result pins.
3. Immutable chain is persistable; historical assessment reconstructs from pinned snapshots (LIN-001).
4. Changing current catalog/pack files does not silently rewrite a stored result digest or explanation; digest mismatch is detected (LIN-002).
5. Historical evidence is append-only; partial/failed collection is distinguishable from a completed empty collection (LIN-005).
6. `ControlExplanation` exists; CLI `assurance explain --assessment <id> --control <id>` is parsed and dispatched and cites exact evidence digests, population, missing evidence, failing/missing subjects, test id/version, exceptions, and mappings (LIN-003, LIN-012).
7. `AssessmentReport` serialization is pure: no `load_framework_pack`, network, filesystem, or hidden current-state resolution (LIN-004).
8. Explicit `AssessmentSummary` / `FrameworkReadinessSnapshot` / `CoverageMetrics`; seven metric families stay separate; no single compliance percentage (LIN-013).
9. One registry/loader path for every framework; production stub `canonical:stub-1` / `assess-runtime-1` removed from production (LIN-010, LIN-011).
10. `compare` identifies applicability, subject population, evidence add/remove/supersession, test-result, exception, and framework/catalog digest changes (LIN-006, LIN-007).
11. Snapshot and result digests are deterministic SHA-256 of canonical JSON, domain-separated from compile-digest reuse, exclude `duration`/`evaluatedAt` (LIN-008).
12. Ledger persist/load for `AssessmentRun` and `ControlTestRun`; replacing a completed run payload with different bytes is rejected or ignored (LIN-014).
13. `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_canonical_assurance_catalog_target` and `cargo test --workspace --features demo` stay GREEN after implement (LIN-015).

### Out of scope

- Multi-tenant SaaS backend, authn/z, hosted control plane
- UI / dashboards / HTML report engine
- New frameworks (SOC 2, NIS2, DORA, GDPR, ISO 27701 production packs)
- Domain catalog redesign or `catalog/canonical/v1` TOML rewrite
- Prompt 09 GitHub collector files (`tests/sdd/github_collector.*` and `crates/weeping-angel-collector/src/github/**`)
- Prompt 10 `OrgContext` / `ManualDeterminationRequired` / `evaluate_org_context`
- Prompt 12 ISO pack ID remap or pack `to=` remapping
- Forking `assurance-ir/v1` or redesigning `AssessmentDefinition` / `Control` / `Requirement` / `Mapping`
- Teaching `compile_framework` to load `catalog/canonical/v1` as a pack substitute
- Certification claims or licensed ISO normative wording
- Collector discovery / scanner-bridge redesign
- Unrelated rustfmt/clippy cleanup

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Spine ACT tests depend on the production stub for non-ISO profiles | Fail-closed production plus explicit test fixtures; keep ACT-003 |
| ISO `iso_007` / serialize tests needle serialize-time `load_framework_pack` | Carry pack digest on the snapshot and load at assess time |
| Concurrent Prompt 10 may collide if this slice adds org-context types | Persist static rule + pack rows only; unknown predicates stay unresolved |
| `collection_runs` already uses `INSERT OR REPLACE` | Lineage persist is append-only or digest-keyed |
| Including `duration`/`evaluatedAt` in result digest | Exclude wall-clock fields from identity |
| Hashing raw TOML bytes breaks on Windows CRLF | Digest canonicalized structures |
| CLI explain remaining under the banner-exit-0 wildcard | Dispatch like Catalog |
| Removing the stub too early reddens neighbor SDD targets | Fixtures first; neighbor GREEN is a hard gate |
| Public JSON shape change surprises consumers | Draft then accept ADR; serde defaults; contract update on accept |
| Ledger in the evidence crate growing conclusion types | Store opaque JSON; evidence crate still does not compute effectiveness |
| Collapsing seven metrics into one renamed compliance percentage | LIN-013 forbids it |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/assessment-lineage.md`](assessment-lineage.md) — durable SSOT for Prompt 11; dual-suite already registered; no production feature code in Spec phase |
| Baseline | PASS on old | `cargo test --test sdd_assessment_lineage_baseline --test sdd_assessment_lineage_target -- --nocapture` → exit 0. No product implementation. Baseline encodes current shortcuts on HEAD; **14/14 GREEN**. Target still only LIN-009/LIN-015 (**2/2 PASS**) as specified for characterization. Excerpt: `running 14 tests` / `assessment_run_reuses_compile_digest_for_three_identities ... ok` / `applicability_rule_is_static_only_prompt_10_absent ... ok` / `test result: ok. 14 passed; 0 failed` then `running 2 tests` / `lin_015_neighbor_sdd_targets_remain_registered ... ok` / `lin_009_dual_suite_binaries_are_registered ... ok` / `test result: ok. 2 passed`. Suites: `tests/sdd/assessment_lineage.baseline.rs`, `tests/sdd/assessment_lineage.target.rs`. |
| Target pre | FAIL on old | Same dual command → exit 1. Compile-safe target locks for Prompt 11. Baseline remains GREEN on current shortcuts. LIN-009 and LIN-015 pass by design. No product feature code. Excerpt: `baseline: test result: ok. 14 passed; 0 failed` / `target: test result: FAILED. 2 passed; 13 failed (lin_001..lin_008, lin_010..lin_014)` / missing snapshot types (`FrameworkPackSnapshot`, `ApplicabilitySnapshot`, `ControlExplanation`, `CoverageMetrics`, …) / missing `DigestMismatch` / Serialize still `load_framework_pack("iso-27001", "2022")` / production stub `canonical:stub-1` / `assess-runtime-1` / CLI missing `Explain` / missing `persist_assessment_run` / `load_assessment_run`. Suites: `tests/sdd/assessment_lineage.target.rs`, `tests/sdd/assessment_lineage.baseline.rs`. |
| Implement | target PASS | Same dual command after shipping persistable `AssessmentRun` (returned, never `let _run`), snapshot/explain/metrics types, pure serialize (carried pins only; no pack load in the `AssessmentReport` Serialize window), generic framework load, append-only ledger persist/load, dispatched CLI `assurance explain`. Target **LIN-001–015 GREEN** (`ok. 15 passed; 0 failed; 0 ignored; finished in 0.22s`). Baseline re-run (not assumed additive): `ok. 0 passed; 0 failed; 14 ignored` (`#[ignore = "superseded by sdd_assessment_lineage_target"]`). Neighbors: `sdd_assurance_runtime_target` **21 passed**; `sdd_iso27001_assurance_target` **49 passed**. Files: `Cargo.toml`, `crates/weeping-angel-assurance/{Cargo.toml,src/lib.rs,src/lineage.rs,src/snapshot.rs,src/soa.rs}`, `crates/weeping-angel-evidence/src/ledger.rs`, `crates/weeping-angel-framework/src/lib.rs`, `src/{assurance_explain.rs,cli.rs,lib.rs,main.rs}`, dual-suite + neighbor test files, spec/protocol docs. |
| Baseline post | FAIL or retired | Skip-retired (`supersede_kind=skip`). Dual run: baseline **ok. 0 passed; 0 failed; 14 ignored** (`superseded by sdd_assessment_lineage_target`; e.g. `assess_builds_then_drops_run_with_empty_collector_runs`, `assessment_report_serialize_loads_iso_pack_and_formats_percentages`). Not additive. Characterization of dropped-`_run` / compile-digest reuse / serialize-time ISO pack load is no longer CI-required. `baseline_retired=true`, `baseline_not_green=true`. |
| Supersede | target still PASS | After skip-supersede: target **ok. 15 passed; 0 failed; 0 ignored** (LIN-001–015). Dual-suite files stay registered for LIN-009. `target_still_green=true`. |
| Docs/ADR | updated | [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`docs/sdd/assessment-lineage.md`](assessment-lineage.md), [`docs/sdd/sdd-assessment-lineage.md`](sdd-assessment-lineage.md), [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/sdd/applicability-engine.md`](applicability-engine.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), [`README.md`](../../README.md) |

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

Immutable assessment lineage (Prompt 11):

- `AssessmentRun` is returned and persisted with start/completion, scope, `completed`/`partial`/`failed`, collector ids/versions/run ids, and distinct pack / catalog / definition / evidence-snapshot / applicability / result pins (never `let _run`, never reused compile digest).
- Persistable snapshots: `FrameworkPackSnapshot`, `CanonicalCatalogSnapshot`, `AssessmentDefinitionSnapshot`, `ApplicabilitySnapshot`, `EvidenceSnapshot`, `FrameworkReadinessSnapshot`, `StatementOfApplicabilitySnapshot`, plus append-only `CollectionRun[]` / `EvidenceEnvelope[]` and `ControlTestRun[]`.
- Replay from pins; consulting current catalog/pack files detects `DigestMismatch`.
- `ControlExplanation` plus dispatched `weeping-angel assurance explain --assessment <id> --control <id>` cites evidence digests, population, missing evidence, failing/missing subjects, test id/version, exceptions, and mappings.
- `AssessmentReport` serialization is pure: pins come from the in-memory report/run; no `load_framework_pack`, network, filesystem, or hidden current-state resolution in the Serialize window.
- `CoverageMetrics` expose seven separate families; no single compliance percentage.
- One `(id, version)` framework loader path; production stub `canonical:stub-1` / `assess-runtime-1` removed from production (test fixtures only).
- `compare` identifies applicability, subject population, evidence add/remove/supersession, test results, exceptions, and framework/catalog digest changes.
- Snapshot/result digests are SHA-256 of canonical JSON, domain-separated from compile-digest reuse, excluding `duration` / `evaluatedAt`.
- Ledger persist/load for `AssessmentRun` and `ControlTestRun`; replacing a completed run payload with different bytes is rejected or ignored.

### Files changed (implement)

`Cargo.toml`, `crates/weeping-angel-assurance/Cargo.toml`, `crates/weeping-angel-assurance/src/lib.rs`, `crates/weeping-angel-assurance/src/lineage.rs`, `crates/weeping-angel-assurance/src/snapshot.rs`, `crates/weeping-angel-assurance/src/soa.rs`, `crates/weeping-angel-evidence/src/ledger.rs`, `crates/weeping-angel-framework/src/lib.rs`, `src/assurance_explain.rs`, `src/cli.rs`, `src/lib.rs`, `src/main.rs`, `tests/sdd/assessment_lineage.baseline.rs`, `tests/sdd/assessment_lineage.target.rs`, `tests/sdd/assurance_runtime.target.rs`, `tests/sdd/canonical_assurance_catalog.baseline.rs`, `tests/sdd/iso27001_assurance.baseline.rs`, `tests/sdd/iso27001_assurance.target.rs`, `tests/sdd/iso27001_remap.baseline.rs`, `tests/sdd/iso27001_remap.target.rs`, `docs/sdd/assessment-lineage.md`, `docs/sdd/sdd-assessment-lineage.md`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-94837bf9b18c` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 16 975 836 |
| `duration_ms_sum` | 6 061 460 (~101.0 min) |
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
| Scope | `sdd-scope` | ok | 151 233 | 312 614 |
| Spec | `sdd-spec` | ok | 852 744 | 1 070 174 |
| BaselineGreen | `sdd-baseline-green` | ok | 66 469 | 192 548 |
| TargetRed | `sdd-target-red` | ok | 726 485 | 1 869 814 |
| Implement | `sdd-implement` | ok | 2 899 303 | 10 740 723 |
| DocsAdr | `sdd-docs-adr` | ok | 538 402 | 1 472 234 |
| Iterate | `sdd-baseline-post-check` | ok | 725 987 | 1 005 102 |
| Supersede | `sdd-supersede` | ok | 100 837 | 312 627 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-assessment-lineage-telemetry.json`](sdd-assessment-lineage-telemetry.json).

---

## Remaining backlog (not this slice)

1. Multi-tenant SaaS backend, authn/z, hosted control plane
2. UI / dashboards / HTML report engine
3. New frameworks (SOC 2, NIS2, DORA, GDPR, ISO 27701 production packs)
4. Domain catalog redesign or `catalog/canonical/v1` TOML rewrite
5. Prompt 09 GitHub collector files (`tests/sdd/github_collector.*`, `crates/weeping-angel-collector/src/github/**`)
6. Prompt 10 `OrgContext` / `ManualDeterminationRequired` / `evaluate_org_context` (persist landed snapshot only)
7. Prompt 12 ISO pack ID remap or pack `to=` remapping
8. Forking `assurance-ir/v1` or redesigning `AssessmentDefinition` / `Control` / `Requirement` / `Mapping`
9. Teaching `compile_framework` to load `catalog/canonical/v1` as a pack substitute
10. Certification claims or licensed ISO normative wording
11. Collector discovery / scanner-bridge redesign
12. Unrelated rustfmt/clippy cleanup

---

## Summary

Prompt 11 assessment lineage landed under dual-suite SDD: spec + accepted ADR 0003, baseline GREEN on SHA `e430980c` (14 passed on dropped-`_run` / compile-digest reuse / serialize-time ISO pack load), target RED (13 failed LIN-001–008, LIN-010–014), then persistable `AssessmentRun`, snapshot/explain/metrics types, pure serialize, generic loader, append-only ledger, and dispatched `assurance explain` until target GREEN 15/15. Baseline shortcut characterization is skip-retired (14 ignored). Historical assessments reconstruct from pins; current-file consults detect `DigestMismatch`; seven metric families stay separate; production stub is gone. Neighbor ACT and ISO targets stayed GREEN.
