# ADR 0003 — Evidence validity and temporal assurance

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_evidence_validity_temporal_assurance_target` GREEN; baseline skip-superseded. Product types are the same as [`0003-evidence-validity-temporal-assurance.md`](0003-evidence-validity-temporal-assurance.md). |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “time is `provenance.collected_at` plus `max_age`” *operational* reading of [ADR 0002](0002-iso-27001-assurance-vertical.md) Phase 7–8 **as implemented** (window = collection time; no as-of). Does **not** supercede envelope immutability, `DigestBody = observation + provenance`, ledger ownership of observations, or INV-1…5. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [population](0003-subject-population-runtime-and-coverage-semantics.md), [assessment lineage](0003-assessment-lineage.md) |
| Spec | [`docs/specs/evidence-validity-temporal-assurance.md`](../specs/evidence-validity-temporal-assurance.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Prompt | [`docs/prompts/operational-isms-v1/14-temporal-assurance.md`](../prompts/operational-isms-v1/14-temporal-assurance.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_evidence_validity_temporal_assurance_target` GREEN; baseline skip-superseded. |

> Filename `0003-*` is shared with catalog-program / operational siblings. Cite this decision by **path**. This file and [`0003-evidence-validity-temporal-assurance.md`](0003-evidence-validity-temporal-assurance.md) record the **same** product decision (one `evidence-validity/v1`, one as-of selector). Dual-suite slugs stay distinct.

## Context

ADR 0002 required immutable envelopes and a SQLite ledger, and named `observedAt` / `validFrom` / `validUntil` as *target* envelope fields. What shipped hashed `DigestBody { observation, provenance }` only. Operational ISMS v1 Prompt 14 requires historical operating effectiveness: deterministic as-of and period evaluation, no temporal leakage, and timeline/diff primitives — without mutating sealed evidence.

Prompt 13 (continuous scheduler) owns cadence. This decision leaves a `FreshnessPolicy` seam.

## Decision

Same seven clauses as [`0003-evidence-validity-temporal-assurance.md`](0003-evidence-validity-temporal-assurance.md):

1. `DigestBody` stays observation+provenance. Validity is append-only `evidence-validity/v1` (`asserted` | `superseded` | `revoked` | `invalidated`). Optional envelope clocks sit outside the digest.
2. Clocks: `collected_at`, `observed_at`, half-open `[valid_from, valid_until)`, `supersedes`, revocation events, optional `source_revision` / artifact digest.
3. Candidate filter at `T`; `within_window` stays collection-time; `valid_during` / `latest_as_of` / `select_latest_as_of` are the validity APIs.
4. `PeriodEffectiveness` is distinct from point-in-time `Effectiveness`. Default semantics `instant`.
5. `project_timeline` / `TemporalDiff`; pairwise `compare` unchanged.
6. `FreshnessPolicy` + injected `AssessmentContext.now` (`as_of()`). No daemon.
7. Reuse SQLite `EvidenceLedger`. No UI.

## Consequences

- Dual-suite names stay distinct; product types must not be duplicated.
- Catalog fact `valid_until` remains an observation fact.

## Non-goals

UI charts; new long-term database; Prompt 13 scheduler product; Prompt 15 event bus; catalog/GitHub/ISO remap rewrites; certification claims.

## Related

- Spec: [`docs/specs/evidence-validity-temporal-assurance.md`](../specs/evidence-validity-temporal-assurance.md)
- Sibling spec: [`docs/specs/temporal-assurance.md`](../specs/temporal-assurance.md)
- Canonical decision write-up (implemented surfaces): [`0003-evidence-validity-temporal-assurance.md`](0003-evidence-validity-temporal-assurance.md)
- Ledger / envelope: [`docs/specs/typed-evidence.md`](../specs/typed-evidence.md)
- Scheduler seam: [`docs/specs/continuous-assurance-scheduler.md`](../specs/continuous-assurance-scheduler.md)
