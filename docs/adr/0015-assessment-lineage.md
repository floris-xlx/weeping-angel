# ADR 0015 — Immutable assessment lineage and pure report serialization

<!-- weeping-angel-adr-meta
id = "0015"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “`AssessmentRun` is an immutable snapshot record” *intent* of [ADR 0002](0002-iso-27001-assurance-vertical.md) Phases 35–36 **as implemented**: dropped `_run`, serialize-time ISO pack load, production stub assessment, compare-only-effectiveness. Does **not** supercede pack schema, envelope immutability, collector blindness, or catalog ownership. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [catalog](0003-canonical-assurance-catalog-v1.md), [typed evidence](0036-typed-evidence-canonical-serialization.md), [population](0034-subject-population-runtime-and-coverage-semantics.md), [applicability engine](0014-applicability-engine.md) |
| Spec | [`docs/specs/assessment-lineage.md`](../specs/assessment-lineage.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | `sdd_assessment_lineage_target` GREEN (LIN-001–015). `sdd_assessment_lineage_baseline` skip-superseded (14 ignored). Neighbor ACT / ISO / catalog targets remain registered. |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0001 delivered the spine. ADR 0002 delivered the ISO 27001 vertical, including an `AssessmentRun` type, ledger table names (`assessment_runs`, `control_test_runs`, `framework_snapshots`), readiness/SoA projections, and `compare`. ADR 0003 applicability shipped an in-memory Kleene snapshot; it did not persist or explain it.

On SHA `e430980c…` those seams were MVP shortcuts:

1. Facade `assess` built `AssessmentRun` and **dropped** it (`let _run`) with empty `collector_runs` and the compile digest reused as definition, evidence-snapshot, and result identity.
2. `AssessmentReport::serialize` called `load_framework_pack("iso-27001", "2022")` and invented coverage percentages.
3. `assessment_for_target` / `normalize` / `stub_catalog` hard-coded ISO 27001:2022; other profiles compiled a **production** stub (`canonical:stub-1`).
4. `compare` only detected effective/ineffective/stale.
5. `project_soa` reread current `applicability.toml` from disk with no snapshot identity.
6. CLI had no `explain`; non-catalog assurance commands banner-and-exit-0.
7. Ledger had lineage tables and **no** persist/load APIs for them.

Canonical Catalog v1 assessment lineage requires assessments to be reproducible immutable execution artifacts, explainable to evidence digest / test version / exception / mapping, with pure serialization and one framework loader path.

Questions this decision answers:

1. What is the persisted lineage chain and which crate owns storage?
2. May serialization resolve current framework/catalog files?
3. How are non-ISO frameworks assessed without a production stub?
4. How does lineage relate to the applicability engine `ApplicabilitySnapshot`?
5. What is the public explain contract?
6. How are coverage metrics exposed without a fake compliance score?

## Decision

This is what shipped.

### 1. Assessment is a pinned execution record

`AssessmentRun` is the root of an immutable chain. Types live in `weeping-angel-assurance::lineage` (schema `weeping-angel/assessment-lineage/v1`, `LINEAGE_SNAPSHOT_SCHEMA`) except the run record itself (`weeping-angel-assurance::snapshot`).

```text
FrameworkPackSnapshot
CanonicalCatalogSnapshot
AssessmentDefinitionSnapshot
ApplicabilitySnapshot          # lineage persist document (see §5)
CollectionRun[]
EvidenceEnvelope[]             # already append-only
EvidenceSnapshot
ControlTestRun[]
AssessmentRun
FrameworkReadinessSnapshot
StatementOfApplicabilitySnapshot
```

`assess` **returns** the run on `AssessmentReport.run`. `let _run` is forbidden.

Pins on `AssessmentRun` (camelCase JSON):

```text
id, framework,
frameworkPackDigest, canonicalCatalogDigest,
assessmentDefinitionDigest, applicabilitySnapshotId,
collectorRuns[], evidenceSnapshotDigest, resultDigest,
startedAt, completedAt, scope,
status = completed | partial | failed
```

`collectorRuns` are collection-run ids. Collector id/version live on `CollectionRun`. Definition / evidence-snapshot / result identities are **distinct** digests (not the compile digest copied three times).

Collector failure no longer aborts `assess`. A failed or partial `CollectionRun` yields `AssessmentRun.status` `failed` or `partial` and an explainable (possibly empty) result set. History is a new row, never a rewrite of a completed run.

Replay: `replay_assessment` verifies pins then reconstructs; `reconstruct` / `load_lineage` clone an already-trusted bundle. They do not consult current pack/catalog files to fill gaps. Missing / mismatched / incomplete / inconsistent lineage is `ReplayFailure` (mapped to `AssuranceError::UnknownPack`). `verify_current_against_pins` / `detect_digest_mismatch` compare pins to current digests; mismatch is `DigestMismatch`, never a silent rewrite. JSON `asOf` is the run `as_of` field ([ADR 0011](0047-temporal-lineage-evidence-soa-integrity.md)).

### 2. Ledger is the storage seam; evidence crate does not conclude

Reuse existing SQLite tables. `EvidenceLedger` persist/load APIs:

```text
persist_assessment_run / load_assessment_run
persist_control_test_run / load_control_test_run
persist_framework_snapshot / load_framework_snapshot
```

Payloads are opaque JSON. `weeping-angel-evidence` still **owns observations, never conclusions**. Effectiveness is computed in control-test / assurance and stored as document bytes.

Envelope `append` remains `INSERT OR IGNORE`. Lineage persist is `INSERT OR IGNORE` after an existing-row check: a second write of **different** bytes for the same key returns `LedgerError::Immutable`. Identical bytes are idempotent (`Ok(false)`). `framework_snapshots` is digest-keyed and holds pack / catalog / definition / applicability / evidence / readiness / SoA payloads as needed.

`assess` does **not** open a ledger. Callers persist the returned run (CLI explain reads `assurance-ledger.sqlite`).

`record_collection_run` is idempotent on identical bytes. A **completed** collection-run payload with different bytes is `LedgerError::Immutable` ([ADR 0011](0047-temporal-lineage-evidence-soa-integrity.md)). In-flight (non-completed) identity may still update. Assessment and control-test rows are not silently replaced.

### 3. Serialization is pure

Generic `AssessmentReport` **must not** call `load_framework_pack`, perform network I/O, look up files, or resolve “current” catalog/pack state inside `Serialize`.

Rust fields:

```text
assessmentId, profile, digest, results, evidenceCount,
run?, summary?, coverageMetrics?,
frameworkPackDigest, canonicalCatalogDigest
```

`Serialize` writes those plus `disclaimer` / `banner`, `resultDigest`, `assessmentRun`, `status`, `collectionRuns`, carried `CoverageMetrics` / `AssessmentSummary`, and derived lists from **in-memory** results. Digests and metrics are computed at assess/project time and **carried**. If `summary` / `coverageMetrics` are unset, they are folded from the report’s own `results` — still no filesystem.

### 4. One registry / loader path

```text
(framework id, version) → resolve_pack_dir / load_framework_pack / load_framework_pack_from
```

is the only production resolution path. `assessment_for_target`, `normalize`, `stub_catalog`, and assess-time pack pin all pass the **target** identity. There is no `load_framework_pack("iso-27001", "2022")` literal on the generic serialize/orchestrate path.

The production stub assessment (`canonical:stub-1` / `assess-runtime-1`) is removed. Missing pack on `assessment_for_target` is `AssuranceError::UnknownPack` (fail closed). `normalize` still skips merge on `UnknownPack` when the caller already supplied an in-memory assessment. `stub_catalog` tries the same loader for the profile selector and common versions, then returns `[]`. Test fixtures may construct in-memory assessments explicitly.

### 5. Two applicability documents; lineage persist is the pin

applicability engine evaluation stays in `weeping-angel-assurance::applicability` (`weeping-angel/applicability-snapshot/v1`). That engine **produces** the Kleene snapshot; this slice does **not** reimplement it.

Crate-root `ApplicabilitySnapshot` is the **lineage persist document** (`::lineage`, schema `weeping-angel/assessment-lineage/v1`): static IR fold (`Always`/`Never`/combinators → applicable / not applicable / unresolved) plus optional `pack_entries` artifacts copied from pack `applicability.toml`. `assess` sets `applicabilitySnapshotId` to that document’s digest.

Unknown predicates stay unresolved (not false). Pack TOML rows are artifacts, not Kleene truth. The applicability engine snapshot remains addressable as `applicability::ApplicabilitySnapshot` and may be stored as opaque JSON; `assess` does not call `evaluate_assessment_applicability`.

Historical SoA: `project_soa_from_snapshot`. Live `project_soa(framework, version)` remains a convenience that may read today’s pack; it must not be used to rewrite a pinned `StatementOfApplicabilitySnapshot`.

### 6. Explain projection and CLI

Public `ControlExplanation`:

```text
control, applicability, implementation, population,
tests[{id, testVersion, inputDigest, exprIdentity}],
evidenceRequirements, evidence, missingEvidence,
failingSubjects, missingSubjects, exceptions, mappings,
effectiveness
```

`explain_control(report, control_id, assessment?, applicability?)` cites **exact** evidence envelope digests from the result, not “latest from current ledger”.

CLI:

```text
weeping-angel assurance explain --assessment <id> --control <id>
```

Parser in `src/cli.rs`; execution in `src/assurance_explain.rs`. Prints the not-certification banner, then JSON. Unknown assessment or control exits non-zero. Does not load a framework pack to answer.

### 7. Metrics stay separate

`CoverageMetrics` exposes seven `MetricFamily { covered, total }` fields:

```text
controlEffectiveness, evidence, automation, subject,
frameworkRequirement, freshEvidence, manualReviewBurden
```

No single compliance / certified percentage. Not-certification banner remains.

### 8. Compare is a lineage diff

`compare(previous, next)` on `FrameworkReadinessSnapshot` fills effectiveness, stale evidence, applicability status, subjects, exceptions, and pack/catalog digest-change flags.

`compare_runs` / `compare_lineage` on two `AssessmentRun`s detect `frameworkPackDigest` / `canonicalCatalogDigest` changes.

`SnapshotDiff` also carries `evidenceAdded` / `evidenceRemoved` / `evidenceSuperseded` for callers that have envelope identity.

### 9. Digests

Result and snapshot digests use IR `typed_canonical_digest` / `canonical_digest` (SHA-256 hex of canonical JSON), domain-separated by type name / schema. They are **not** the compile digest copied into three identity fields.

`assessment_result_digest` hashes test id, control id, effectiveness, evidence refs, missing evidence, test version, input digest, and population. Wall-clock `duration` / `evaluatedAt` (`checked_at`) are excluded. Pack/catalog hashes follow existing digest law over canonicalized structures, not raw TOML bytes.

## Consequences

- Public report JSON carries explicit summary / metrics / dual digests; serialize-time pack load is gone.
- Facade assess no longer succeeds on arbitrary profiles via a hidden stub; missing pack fails closed.
- CLI `AssuranceCommand` includes `Explain`.
- Contract documents lineage types, ledger APIs, explain CLI, and pure serialization.
- Neighbor suites stay green; ISO pack IDs are not remapped (ISO remap).
- Dual-suite `sdd_assessment_lineage_target` is the CI gate.

## Non-goals

Multi-tenant SaaS, UI, new frameworks, domain catalog redesign, applicability engine evaluator, ISO remap ISO remap, IR schema fork, certification claims, automatic ledger write inside `assess`.

## Status

Accepted after target GREEN. Public types are frozen in `weeping-angel-assurance::{lineage,snapshot}` and `EvidenceLedger` persist/load APIs. Baseline absence characterization is ignored so CI does not require the pre-lineage HEAD. Fail-closed replay, independent `asOf`, and collection-run immutability are [ADR 0011](0047-temporal-lineage-evidence-soa-integrity.md).
