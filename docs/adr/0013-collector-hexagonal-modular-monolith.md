# ADR 0013 — Collector hexagonal modular monolith (observations in, envelopes out)

<!-- weeping-angel-adr-meta
id = "0013"
status = "draft"
supersedes = []
superseded_by = []
depends_on = ["0001-inwardly-extensible-assurance-runtime", "0002-iso-27001-assurance-vertical", "0003-github-collector-canonical-evidence-mapping", "0004-documentation-architecture", "0005-continuous-assurance-scheduler", "0010-architecture-as-law"]
-->

| Field | Value |
| --- | --- |
| Status | **Draft** — specified with increment 1; accept when `sdd_collector_hexagonal_target` is GREEN |
| Date | 2026-08-20 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in GitHub evidence mapping, envelope digest law, or scheduler job semantics. **Amends operational practice** that provider adapters construct `EvidenceProvenance` and call `EvidenceEnvelope::seal`. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [ADR 0003 GitHub mapping](0003-github-collector-canonical-evidence-mapping.md), [ADR 0004](0004-documentation-architecture.md), [ADR 0005 scheduler](0005-continuous-assurance-scheduler.md), [ADR 0010](0010-architecture-as-law.md) |
| Spec | [`docs/specs/collector-hexagonal.md`](../specs/collector-hexagonal.md) |
| Evidence-contract SSOT (unchanged) | [`docs/specs/github-collector.md`](../specs/github-collector.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `0015f6395e7ead042e3cfd3066fefde3d39aa36b` plus collector crate sources before hexagonal implement |
| Tests (at implement) | `sdd_collector_hexagonal_target` GREEN; `sdd_collector_hexagonal_baseline` skip-superseded. Neighbor `sdd_github_collector_target` stays GREEN. |

> Filename **`0013-*`**. Cite **this file by path**. Do **not** mint `0011-*` (already used by concurrent cleanup ADRs). Do **not** add a `0003-collector-hexagonal.md` sibling. Duplicate prefixes remain `DEBT-DUP-ADR`.

## Context

The collector crate is one Cargo package that already emits canonical GitHub evidence ([ADR 0003 GitHub mapping](0003-github-collector-canonical-evidence-mapping.md)). On characterization HEAD, that package is a **monolith**:

1. Domain types (`CollectorCapabilities` eight fields, `CollectorDescriptor`, `CollectorScope`, `CollectionRequest`, `CollectionBatch { errors: Vec<String> }`, `EvidenceCollector`) live in `src/lib.rs`.
2. `github/normalize.rs`, `LocalCollector`, `FixtureCollector`, and `ManualEvidence` construct `EvidenceProvenance` and seal `EvidenceEnvelope`. Adapters invent provenance.
3. Collector **type** (`collector.github`) and **instance** (e.g. `github:xylex-group`) are the same object. `GitHubCollector::new(token)` and `GitHubClient.token` hold secrets.
4. There is no `CollectionEngine`, `CollectorRegistry`, `ObservationGate`, or `EnvelopeFactory`. Scheduler (`EvidenceCollector::collect`) and GitHub `collect_batch` both get envelopes directly from adapters.
5. Splitting into multiple collector crates would multiply workspace members and violate “inwardly extensible” (ADR 0001) plus architecture-as-law (no hypothetical packages).

Questions this decision answers:

1. Is the collector a crate-per-adapter workspace or a **hexagonal modular monolith**?
2. Who is allowed to construct `EvidenceProvenance` and call `EvidenceEnvelope::seal`?
3. What is the adapter output type (`ObservationCandidate` vs envelope)?
4. How is instance distinct from type, and where do credentials live?
5. Does increment 1 rewrite `AssuranceScheduler`?

## Decision (to accept at implement)

### 1. One crate, hexagonal modules

`weeping-angel-collector` remains **one** Cargo package. Layout:

```text
src/lib.rs           facade
src/domain/          capabilities, descriptor, scope, collector, observation,
                     coverage, diagnostic, batch, cursor, instance
src/application/     CollectionEngine, CollectorRegistry, ObservationGate, EnvelopeFactory
src/ports/           CollectorAdapter
src/adapters/        re-exports; GitHub implementation stays under src/github/ this increment
src/github/          KEEP on disk (`sdd_github_collector_target` `github_src()`)
src/local/
```

Do not create `weeping-angel-collector-github` or other collector subcrates.

### 2. Frozen pipeline

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Never `GitHub → ISO check`. Collectors are framework-blind facts engines. Scheduler owns time. Ledger owns persistence.

Increment 1 flow:

```text
CollectionRequest → Registry → CollectorAdapter → ObservationBatch
  → ObservationGate → EnvelopeFactory → CollectionBatch
```

### 3. Adapters emit observations; only EnvelopeFactory seals

Adapters emit `ObservationCandidate` / `ObservationBatch`. They MUST NOT construct `EvidenceEnvelope` or `EvidenceProvenance`.

`EnvelopeFactory` is the **only** collector-crate site that builds `EvidenceProvenance` and calls `EvidenceEnvelope::seal`. It consumes Prompt 3 seal **as-is**. Provenance `collector_id` remains the **type** id (`collector.github`) so evidence IDs do not change.

`ObservationGate` validates adapter output (declared types, no compliance claims, scope, 403≠false, no silent negative facts). Seal remains a second rejector (credentials, claims).

### 4. Instance ≠ type; credentials are refs

```text
CollectorInstance { id, collector_id, configuration, credential_ref }
```

`collector.github` is a type. `github:xylex-group` is an instance. Tokens are not fields of `CollectorInstance`. `GitHubCollector::new(token)` may remain a compatibility constructor; secrets stay off the instance object.

Do not force `CollectorId` newtypes in increment 1.

### 5. Compatibility facade; scheduler later

Public `use weeping_angel_collector::{EvidenceCollector, GitHubCollector, LocalCollector}` and `GitHubCollector::collect_batch` stay compile-stable. The facade is `CollectorAdapter` → `CollectionEngine` → `EvidenceCollector`. **Do not** rewrite `AssuranceScheduler` in increment 1 (Phase 27).

### 6. Preserve GitHub evidence contract

No change to mappings, evidence IDs, `GITHUB_EVIDENCE_TYPES`, 403/404 semantics, or `sdd_github_collector_target`. [`docs/specs/github-collector.md`](../specs/github-collector.md) remains that SSOT.

## Collectors MAY / MUST NOT

**MAY:** connect, discover, retrieve provider state, interpret responses, normalize to canonical observations, report incompleteness, report permission/transport failures.

**MUST NOT:** decide ISO compliance, evaluate canonical tests, emit `ControlTestResult` / Effectiveness, decide readiness, construct SoA, accept risk, schedule themselves, persist assurance state, own evidence history, invent provenance, convert unavailable evidence into negative facts.

403 ≠ false. Missing coverage is not success.

## Rejected alternatives

- **Crate per adapter** — workspace explosion; scheduler would take a crate graph; forbidden hypothetical packages.
- **Keep adapters sealing envelopes** — provenance law forks per provider.
- **Put instance id in `EvidenceProvenance.collector_id` this increment** — changes evidence IDs.
- **Rewrite scheduler now** — concurrent Prompt 3/5 surface; Phase 27.

## Consequences

- Hexagonal dual-suite `sdd_collector_hexagonal_{baseline,target}` under `tests/contracts/` (never `tests/sdd/`).
- Architecture IDs COL-001…015 are **declared** in the spec; xtask guards are Prompt 1 Phases 24–25.
- Adding `docs/specs/collector-hexagonal.md` fails Guard 15 until Prompt 1 lists it in `architecture/spec-lifecycle.toml`.
- Remaining backlog (typed diagnostics, public coverage, SubjectSelector, live transport split, API shrink, scheduler engine, ledger/hosted) stays out of increment 1.

## Related

- Program spec: [`docs/specs/collector-hexagonal.md`](../specs/collector-hexagonal.md)
- GitHub mapping: [ADR 0003](0003-github-collector-canonical-evidence-mapping.md)
- Envelope seal: [ADR 0011 temporal/lineage/evidence/SoA](0011-temporal-lineage-evidence-soa-integrity.md) (Prompt 3; consume, do not edit)
