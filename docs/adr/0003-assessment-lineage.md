# ADR 0003 — Immutable assessment lineage and pure report serialization

| Field | Value |
| --- | --- |
| Status | **Draft** — specify-only; accept when implement freezes public contracts and ledger APIs |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “`AssessmentRun` is an immutable snapshot record” *intent* of [ADR 0002](0002-iso-27001-assurance-vertical.md) Phases 35–36 **as implemented**: dropped `_run`, serialize-time ISO pack load, production stub assessment, compare-only-effectiveness. Does **not** supercede pack schema, envelope immutability, collector blindness, or catalog ownership. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [ADR 0003 catalog infra](0003-canonical-assurance-catalog-v1.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [population](0003-subject-population-runtime-and-coverage-semantics.md) |
| Spec | [`docs/sdd/assessment-lineage.md`](../sdd/assessment-lineage.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — update on accept |
| Prompt | [`docs/prompts/canonical-assurance-v1/11-assessment-lineage.md`](../prompts/canonical-assurance-v1/11-assessment-lineage.md) |
| Characterization | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | Dual-suite **registered**. Baseline GREEN on current shortcuts. Target LIN-001–008 and LIN-010–014 authored so suite is RED on CURRENT **before** product feature code (LIN-009 / LIN-015 still PASS); then GREEN after implement. |
| Collision fence | Do not edit Prompt 09 GitHub collector files; do not add Prompt 10 `OrgContext` / `ManualDeterminationRequired` / `evaluate_org_context`; do not rewrite catalog domain TOML or ISO pack IDs (Prompt 12). |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0001 delivered the spine. ADR 0002 delivered the ISO 27001 vertical, including an `AssessmentRun` type, ledger table names (`assessment_runs`, `control_test_runs`, `framework_snapshots`), readiness/SoA projections, and `compare`.

On SHA `e430980…` those seams are MVP shortcuts:

1. Facade `assess` builds `AssessmentRun` and **drops** it (`let _run`) with empty `collector_runs` and the compile digest reused as definition, evidence-snapshot, and result identity.
2. `AssessmentReport::serialize` calls `load_framework_pack("iso-27001", "2022")` and invents coverage percentages.
3. `assessment_for_target` / `normalize` / `stub_catalog` hard-code ISO 27001:2022; other profiles compile a **production** stub (`canonical:stub-1`).
4. `compare` only detects effective/ineffective/stale.
5. `project_soa` rereads current `applicability.toml` from disk.
6. CLI has no `explain`; non-catalog assurance commands banner-and-exit-0.
7. Ledger has lineage tables and **no** persist/load APIs for them.
8. Prompt 10 applicability engine is not landed — only IR `ApplicabilityRule` + `statically_applicable`.

Canonical Catalog v1 Prompt 11 requires assessments to be reproducible immutable execution artifacts, explainable to evidence digest / test version / exception / mapping, with pure serialization and one framework loader path.

IR already has `type Assessment = AssessmentDefinition` and `Exception.subjects`. This ADR snapshots those types; it does not fork them. New product modules stay in existing crates (`weeping-angel-assurance` for explain/metrics/run assembly; `weeping-angel-evidence::ledger` for persist/load of opaque JSON).

Questions this decision answers:

1. What is the persisted lineage chain and which crate owns storage?
2. May serialization resolve current framework/catalog files?
3. How are non-ISO frameworks assessed without a production stub?
4. How do we persist applicability before Prompt 10?
5. What is the public explain contract?
6. How are coverage metrics exposed without a fake compliance score?

## Decision (draft — implement will freeze signatures)

### 1. Assessment is a pinned execution record

`AssessmentRun` is the root of an immutable chain:

```text
FrameworkPackSnapshot
CanonicalCatalogSnapshot
AssessmentDefinitionSnapshot
ApplicabilitySnapshot
CollectionRun[]
EvidenceEnvelope[]          # already append-only
EvidenceSnapshot
ControlTestRun[]
AssessmentRun
FrameworkReadinessSnapshot
StatementOfApplicabilitySnapshot
```

Pins required on the run: framework pack digest, canonical catalog digest, assessment definition digest, collector ids/versions, collection run ids, evidence snapshot digest, test ids/versions, applicability snapshot identity, result digest, start/completion, scope, status.

`let _run` is forbidden. The run is returned and, when a ledger is supplied, persisted.

Status distinguishes `completed` | `partial` | `failed`. Partial/failed collection is a new row, never a rewrite of a completed run.

### 2. Ledger is the storage seam; evidence crate does not conclude

Reuse existing SQLite tables. Add persist/load APIs on `EvidenceLedger` for assessment runs, control-test runs, and framework snapshots, plus rows or payloads for catalog / definition / applicability / evidence snapshots as needed.

Payloads are opaque JSON. `weeping-angel-evidence` still **owns observations, never conclusions**. Effectiveness is computed in control-test / assurance and stored as document bytes.

Envelope `append` remains `INSERT OR IGNORE`. Completed assessment payloads must not be silently `INSERT OR REPLACE`d with different semantic bytes.

### 3. Serialization is pure

Generic `AssessmentReport` (and explain/report projections) **must not**:

- call `load_framework_pack`;
- perform network I/O;
- look up files;
- resolve “current” catalog or pack state.

Digests and `CoverageMetrics` / `AssessmentSummary` / `FrameworkReadinessSnapshot` are computed at assess/project time and **carried** on the value.

### 4. One registry / loader path

`(framework id, version) → resolve_pack_dir / load_framework_pack / load_framework_pack_from` is the only production resolution path.

Remove hardcoded `load_framework_pack("iso-27001", "2022")` from generic serialize and orchestration. Remove ISO-only fallbacks in `assessment_for_target`, `normalize`, and `stub_catalog` that skip this path.

Remove the production stub assessment. Test fixtures may construct in-memory assessments explicitly.

Missing pack fails closed.

### 5. ApplicabilitySnapshot from existing rule/scope data

Prompt 10 is **out of this ADR**. Persist:

- IR `AssessmentScope` when present;
- each requirement/control `ApplicabilityRule` (or its digest) plus `statically_applicable` outcome;
- pack `applicability.toml` rows actually used.

Unknown predicates stay unresolved (not false). Prompt 10 may later fill the same snapshot shape with three-state org-context results.

### 6. Explain projection and CLI

Public projection `ControlExplanation` (control, applicability, implementation, population, tests, evidence requirements, evidence digests, missing evidence, failing/missing subjects, exceptions, mappings, effectiveness).

CLI family grows:

```text
weeping-angel assurance explain --assessment <id> --control <id>
```

Parser in `src/cli.rs`; execution outside the clap enum. Not banner-and-exit-0.

### 7. Metrics stay separate

`CoverageMetrics` exposes distinct families: control effectiveness, evidence, automation, subject, framework requirement, fresh-evidence, manual-review burden.

No single compliance / certified percentage. Not-certification banner remains.

### 8. Compare is a lineage diff

`compare` (on readiness snapshots and/or full runs) identifies applicability changes, subject population changes, evidence add/remove/supersession, test-result changes, exception introduce/expire, and framework/catalog digest changes.

### 9. Digests

Result and snapshot digests use IR `canonical_digest` law (SHA-256 hex of canonical JSON), domain-separated from raw compile-digest reuse. They are **not** the compile digest copied into three identity fields. Wall-clock `duration` / `evaluatedAt` (`checked_at`) are not part of result identity. Pack/catalog hashes follow existing digest law over canonicalized structures, not raw TOML bytes.

## Consequences

- Public report JSON grows explicit summary/metrics/digest fields; serialize-time computed fields go away.
- Facade assess no longer succeeds on arbitrary profiles via a hidden stub.
- CLI `AssuranceCommand` gains `Explain`.
- Contract file must document lineage types, ledger APIs, explain CLI, and pure serialization.
- Neighbor suites (`sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_canonical_assurance_catalog_target`) stay green; ISO pack IDs are not remapped (Prompt 12).
- Dual-suite `sdd_assessment_lineage_*` is the CI gate for this decision once implement completes.

## Non-goals

Multi-tenant SaaS, UI, new frameworks, domain catalog redesign, Prompt 10 evaluator, Prompt 12 ISO remap, IR schema fork, certification claims.

## Status of this file

**Draft.** Do not treat signatures as frozen until implement accepts this ADR and updates [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md).
