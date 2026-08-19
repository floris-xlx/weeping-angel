# SDD: Collector hexagonal architecture PROGRAM — Increment 1 (Phases 1–6 + compatibility facade)

| Field | Value |
| --- | --- |
| Status | **Specified** — increment 1 not implemented. Baseline `sdd_collector_hexagonal_baseline` characterizes CURRENT monolith (must stay GREEN). Target suite not registered in this characterization slice. This file is the human SSOT. |
| Program | Collector hexagonal architecture (30 phases). One coordinated modular-monolith refactor of `weeping-angel-collector`. **Not** a crate split. |
| Slice | **Increment 1** = Phases **1–6** plus a compile-stable public facade. **Not** phases 7–30. |
| Characterization | Workspace HEAD `0015f6395e7ead042e3cfd3066fefde3d39aa36b` (`main`, 2026-08-20) plus current collector crate sources (inspected before any hexagonal product change). |
| Dual-suite | `sdd_collector_hexagonal_baseline` at `tests/contracts/collector_hexagonal.baseline.rs` (registered; GREEN on current monolith). Register `sdd_collector_hexagonal_target` at **implement** (`tests/contracts/collector_hexagonal.target.rs`). **Do not** create `tests/sdd/`. |
| Neighbor (must stay GREEN throughout) | `sdd_github_collector_target` (`ghc_000`–`ghc_024`). Do **not** weaken `ghc_*`. Baseline `sdd_github_collector_baseline` stays superseded/`#[ignore]`. |
| ADR | **Draft** [`docs/adr/0013-collector-hexagonal-modular-monolith.md`](../adr/0013-collector-hexagonal-modular-monolith.md). Unique number **0013** (0011/0012 already used). Do **not** mint `0011-*`. Do **not** add another `0003-*`. Accept at implement when target is GREEN. |
| Evidence-contract SSOT (do **not** overwrite) | [`github-collector.md`](github-collector.md) + [ADR 0003 GitHub mapping](../adr/0003-github-collector-canonical-evidence-mapping.md) |
| Public collector trait contract | [`assurance-runtime.md`](assurance-runtime.md) (COL-001…006 remain law). This spec does not rewrite that table. |
| Spine / ISO law | [`assurance-runtime-spine.md`](assurance-runtime-spine.md), [`iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Scheduler (consume, do not rewrite) | [`continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md), [ADR 0005](../adr/0005-continuous-assurance-scheduler.md) |
| Envelope seal (consume as-is, Prompt 3) | `EvidenceEnvelope::seal` in `weeping-angel-evidence`. Do not change digest law / `EvidenceProvenance` fields. |
| Documentation architecture | [ADR 0004](../adr/0004-documentation-architecture.md) — this file under `docs/specs/` is SSOT. `docs/sdd/` is a stub. Traces only under `.sdd/runs/` and `.sdd/artifacts/`. |
| Architecture-as-law (Prompt 1 owns execution) | [`architectural-cleanup-program.md`](architectural-cleanup-program.md), [ADR 0010](../adr/0010-architecture-as-law.md). **COL-001…015 below are declared here only.** Do not edit `xtask/**`, `architecture/**`, or `docs/debt/register.toml`. |
| Guard 15 note | Adding this path will fail spec-lifecycle until Prompt 1 lists it in `architecture/spec-lifecycle.toml` (and optionally `documentation_layout.rs` `CANONICAL_SPECS`, Prompt 4). That listing is **not** this increment. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Crate | one Cargo package: `crates/weeping-angel-collector` (`publish = false`) |
| `adr_needed` | **true** — observation vs envelope ownership, instance vs type, hexagonal module law inside one crate |
| Workspace verify (after implement) | `cargo test -p weeping-angel-collector`; `cargo test --test sdd_github_collector_target`; `cargo test --test sdd_collector_hexagonal_target`; `cargo fmt --all -- --check`; `cargo check -p weeping-angel-collector --all-targets`; `cargo check --workspace` |

This document is the durable human SSOT for the **full hexagonal collector program** and for **increment 1 acceptance**. It owns:

- the frozen architecture law for collectors
- collector MAY / MUST NOT hard invariants
- the 30-phase catalog and remaining backlog
- increment 1 (Phases 1–6 + facade) module layout, types, and flow
- dual-suite protocol (`sdd_collector_hexagonal_{baseline,target}`)
- architecture invariant IDs **COL-001…015** (spec-declared; xtask guards are Prompt 1 later)

It does **not** own GitHub evidence IDs, mappings, 403/404 semantics, `GITHUB_EVIDENCE_TYPES`, goldens, or provenance digest law. Those remain [`github-collector.md`](github-collector.md).

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Never `GitHub → ISO check`. Collectors remain **framework-blind** and only produce **facts**. Scheduler owns orchestration over time. Evidence ledger owns persistence.

Definition of done for increment 1: *the collector crate is a strict hexagonal modular monolith (`domain` / `application` / `ports` / `adapters` + facade `lib.rs`) without splitting Cargo packages, without changing GitHub evidence behavior, and with `EvidenceCollector` / `GitHubCollector` / `LocalCollector` still compiling for scheduler and assurance consumers.*

---

## 0. Collision fence (concurrent Prompts 1–4)

Prompts 1–4 run concurrently. This increment may change **only** the trees below.

| Allowed this increment | Home |
| --- | --- |
| Collector crate internals | `crates/weeping-angel-collector/**` |
| This program spec | `docs/specs/collector-hexagonal.md` (**new**; do not overwrite `github-collector.md`) |
| Draft → accepted ADR 0013 | `docs/adr/0013-collector-hexagonal-modular-monolith.md` |
| Hexagonal dual-suite (new files) | `tests/contracts/collector_hexagonal.{baseline,target}.rs` + root `Cargo.toml` `[[test]]` rows `sdd_collector_hexagonal_{baseline,target}` |
| Tiny re-exports | only if scheduler/assurance need **compile-stable public names** (`EvidenceCollector`, `GitHubCollector`, `LocalCollector`, `CollectorScope`, `CollectionRequest`, `CollectionBatch`, `CollectorError`, `CollectorDescriptor`, `CollectorCapabilities`) |

| Forbidden | Owner |
| --- | --- |
| `xtask/**`, `architecture/**`, `docs/debt/register.toml`, repository-integrity / architectural-cleanup suites | Prompt 1 |
| Catalog / framework / readiness product code and their active semantic suites | Prompt 2 |
| `weeping-angel-evidence` persistence, assurance `temporal` / `lineage` / `soa`, `EvidenceEnvelope::seal` / `EvidenceProvenance` field law | Prompt 3 — **consume seal as-is** |
| Broad hygiene, README, schemas, mass-reformat of unrelated tests, `tests/contracts/documentation_layout.rs` | Prompt 4 |
| `tests/sdd/` | Forbidden (ADR 0004 / `FORBID-TESTS-SDD`) |
| Rewriting `AssuranceScheduler` / `EvidenceCollector` call sites to `CollectionEngine` | Phase 27 |
| New Cargo crates (`weeping-angel-collector-github`, etc.) | Never |
| Athena-specific collectors; framework-specific GitHub evidence; `evidence.github.*` | Never |
| Weakening `ghc_*` assertions or moving `github_src()` off `crates/weeping-angel-collector/src/github` | Forbidden |

`sdd_github_collector_target` `github_src()` is **hard-wired** to `crates/weeping-angel-collector/src/github`. Increment 1 **must keep that directory on disk**. Preferred layout: keep GitHub adapter implementation physically under `src/github/` and re-export it from `src/adapters/` so `github_sources_joined()` still sees ISO GH-012 needles, `GITHUB_EVIDENCE_TYPES`, `SOURCE_TO_CANONICAL`, and 403≠false law. A later physical move under `src/adapters/github/` is allowed **only** if `src/github/` remains a compiling facade whose walked `.rs` files still satisfy every `ghc_*` source-scan. Do not change `github_src()`.

---

## 1. Problem / user-visible goal

`weeping-angel-collector` is already a **reference-grade GitHub facts emitter** ([`github-collector.md`](github-collector.md) target GREEN). It is **not** a hexagonal collector: domain types live in a `lib.rs` monolith, GitHub/local/fixture adapters **invent provenance and seal envelopes**, there is no collector **instance** distinct from collector **type**, tokens sit on `GitHubCollector` / `GitHubClient`, and there is no application engine that can gate observations before seal.

That means:

1. **Adapters own the evidence contract.** `github/normalize.rs::emit`, `LocalCollector::collect`, `FixtureCollector::collect`, and `ManualEvidence::seal` all construct `EvidenceProvenance` and call `EvidenceEnvelope::seal`. Provenance identity is invented at the provider edge. A future GitLab adapter would copy that law (or get it wrong).
2. **Type and instance are collapsed.** Descriptor id is always `collector.github`. There is no `github:xylex-group` instance, no `CredentialRef`, and `GitHubCollector::new(token)` plus `GitHubClient.token` hold secrets on the collector object.
3. **Scheduler talks to a god trait.** `AssuranceScheduler` / `AssuranceEngineBuilder` call `EvidenceCollector::collect(scope) → Vec<EvidenceEnvelope>`. There is no registry, no observation gate, no envelope factory, no `CollectionEngine`. Moving orchestration into collectors later would violate spine law.
4. **Module law is informal.** Capabilities, descriptor, scope, batch, and GitHub HTTP live as one crate soup. Hexagonal ports cannot be tested without GitHub. Architecture-as-law cannot name `COL-*` module edges until the directories exist.

**User-visible goal (program):** one collector crate that is a **strict hexagonal modular monolith**:

```text
Provider adapter
  → ObservationCandidate / ObservationBatch   (facts only)
    → ObservationGate                         (declared types, no claims, no 403→false)
      → EnvelopeFactory                       (only place that builds EvidenceProvenance + seal)
        → CollectionBatch                     (envelopes for ledger / scheduler facade)
```

Collectors connect, discover, retrieve, interpret provider responses, normalize to canonical **observations**, and report incompleteness / permission / transport failure. They **never** decide ISO compliance, evaluate tests, emit effectiveness, construct SoA, persist history, or schedule themselves.

**User-visible goal (this increment):** Phases 1–6 land structurally. Public `use weeping_angel_collector::{EvidenceCollector, GitHubCollector, LocalCollector}` and `GitHubCollector::collect_batch` stay compile-stable. GitHub evidence IDs, mappings, 403≠false, and `GITHUB_EVIDENCE_TYPES` stay GREEN. No crate split.

---

## 2. Compatibility / dependencies

Pinned to the tree characterized in [§3](#3-current-behavior-baseline--green-on-current-code).

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| Workspace member | `weeping-angel-collector` in root `Cargo.toml` | Keep **one** package. Do not add collector subcrates. |
| Public names | `lib.rs` re-exports | `EvidenceCollector`, `GitHubCollector`, `LocalCollector`, `ManualEvidence`, `FixtureCollector`, `CollectorDescriptor`, `CollectorCapabilities`, `CollectorScope`, `CollectionRequest`, `CollectionBatch`, `CollectorError` remain importable from the crate root. |
| `EvidenceCollector` | `fn descriptor` + `fn collect(&self, scope) -> Result<Vec<EvidenceEnvelope>, CollectorError>` | **Compile-stable** for scheduler (`Arc<dyn EvidenceCollector + Send + Sync>`) and `AssuranceEngineBuilder<C: EvidenceCollector>`. Facade may delegate adapter → engine internally. Do **not** add mandatory new trait methods this increment. |
| `GitHubCollector::new(token: Option<String>)` / `with_client` / `collect_batch` | `src/github/mod.rs` today | Keep callable. Token may still live on the **transport client** this increment (live transport split is Phase 13+). It must **not** live on `CollectorInstance`. |
| `CollectionBatch` | `{ run, envelopes, errors: Vec<String> }` | Public shape stays compatible. Typed diagnostics / first-class `CollectionCoverage` that would change this struct are Phase 7–8. |
| `CollectorCapabilities` | eight bools | Preserve fields: `incremental`, `pagination`, `historical`, `point_in_time`, `event_driven`, `sensitive_artifacts`, `offline`, `worker_safe`. |
| `CollectorDescriptor` | `{ id, version, evidence_types, provider_family, subject_types, capabilities, required_permissions }` | Preserve fields. Do not add `failure_behavior` (GitHub-owned const remains). Do **not** force `CollectorId` newtypes this increment. |
| `CollectorScope` | allow-list of `AssetId` + `as_label()` | Keep existing API. Structured `SubjectSelector` is Phase 9–10. |
| GitHub mapping SSOT | [`github-collector.md`](github-collector.md) | No evidence mapping / ID / 403/404 / golden behavior change. |
| `GITHUB_EVIDENCE_TYPES` | `src/github/descriptor.rs` | Unchanged historical `source.*` list (ISO GH-012 / IAM-015). |
| Envelope seal | `weeping_angel_evidence::EvidenceEnvelope::seal` | Only `EnvelopeFactory` (plus existing Prompt 3 law inside seal) constructs provenance + digest. Factory **calls** seal; it does not fork digest. |
| Scheduler | `weeping-angel-assurance::scheduler` | **Do not rewrite.** Facade keeps `EvidenceCollector`. Phase 27 adopts `CollectionEngine`. |
| ADR numbering | `docs/adr/` | Unique file **0013**. Concurrent `0011-*` files stay grandfathered debt. |
| Dual-suite discovery | root `[[test]]` | Register hexagonal suites at implement. Keep `sdd_github_collector_target`. |
| `ASSURANCE_IR_SCHEMA` / `EVIDENCE_SCHEMA` | unchanged | Untouched. |

Assurance-runtime executable tests **COL-001…006** (`sdd_assurance_runtime_target`) remain law and **keep those IDs**. Hexagonal architecture IDs in [§8](#8-architecture-invariants-col-001015-spec-only) reuse the `COL-` prefix as **xtask invariant ids** for a later Prompt 1 guard. They are **not** a second copy of ACT/COL test functions. Spec and future `architecture/invariants.toml` must title them so they cannot be confused (e.g. `COL-001` *Collectors are framework-blind module edges*).

---

## 3. Current behavior (baseline — GREEN on CURRENT code)

§3 is characterization of HEAD `0015f63…` **before hexagonal product changes**. Executable characterization must live in `sdd_collector_hexagonal_baseline` and **PASS on current sources**. Target suite is **RED** on the same tree.

### 3.1 Package and layout

- One crate: [`crates/weeping-angel-collector`](../../crates/weeping-angel-collector).
- Modules on disk: `src/lib.rs`, `src/github/**` (14 files), `src/local/mod.rs`.
- **No** directories named `domain/`, `application/`, `ports/`, or `adapters/`.
- `lib.rs` is the domain **and** the facade: `CollectorCapabilities`, `CollectorDescriptor`, `CollectorScope`, `CollectionRequest`, `CollectionBatch`, `CollectorError`, `EvidenceCollector`, `FixtureCollector` are defined there (not extracted).

### 3.2 Shared types (fields to preserve)

`CollectorCapabilities` (serde `camelCase`): `incremental`, `pagination`, `historical`, `point_in_time`, `event_driven`, `sensitive_artifacts`, `offline`, `worker_safe` — all `bool`, `Default` false except as set by constructors.

`CollectorDescriptor`: `id: String`, `version: String`, `evidence_types: BTreeSet<EvidenceType>`, `provider_family: String`, `subject_types: BTreeSet<String>`, `capabilities: CollectorCapabilities`, `required_permissions: Vec<String>`. Ids are strings (`"collector.github"`), not newtypes.

`CollectorScope`: private `allowed: BTreeSet<AssetId>`; `new` / `allow_asset` / `allows` / `as_label` (comma-joined).

`CollectionRequest { scope: CollectorScope }`.

`CollectionBatch { run: CollectionRun, envelopes: Vec<EvidenceEnvelope>, errors: Vec<String> }`.

`EvidenceCollector`: `descriptor(&self) -> CollectorDescriptor`; `collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError>`.

### 3.3 Adapters construct envelopes and provenance today

| Site | What it does |
| --- | --- |
| `github/normalize.rs::emit` | Builds `EvidenceObservation`, then `EvidenceProvenance { collector_id: "collector.github", collected_at, scope: scope.as_label(), asset }` and `EvidenceEnvelope::seal`. |
| `github/{protection,security,workflows,collaborators,rulesets}.rs` | Return `Vec<EvidenceEnvelope>` via `normalize::emit`. |
| `GitHubCollector::collect_inner` | Accumulates envelopes; `collect` returns them; `collect_batch` wraps a filled `CollectionRun` + `errors: Vec<String>`. |
| `LocalCollector::collect` | Constructs `EvidenceProvenance { collector_id: "collector.local", … }` and seals. |
| `ManualEvidence::seal` | Constructs `EvidenceProvenance { collector_id: "collector.manual", … }` and seals. |
| `FixtureCollector::collect` | Constructs `EvidenceProvenance { collector_id: self.id, … }` and seals. Fixed `collected_at` `(2026, 8, 18, 12, 0, 0)` for COL-004. |

There is **no** `ObservationCandidate`, `ObservationBatch`, `ObservationGate`, `EnvelopeFactory`, `CollectionEngine`, `CollectorRegistry`, `CollectorAdapter` trait, `CollectorInstance`, or `CredentialRef`.

### 3.4 Instance vs type vs secrets

- GitHub type id is `collector.github` (descriptor + `CollectionRun::new("collector.github", …)` + provenance `collector_id`).
- `GitHubCollector { client: GitHubClient, version }`. `GitHubCollector::new(token: Option<String>)` stores the token on `GitHubClient.token`.
- No instance id such as `github:xylex-group`. No configuration object on a hexagonal instance. Credentials are **material**, not refs.

### 3.5 Application layer absent

Collection flow today:

```text
GitHubCollector::collect_batch(CollectionRequest)
  → collect_inner(scope)
    → fetch + normalize::emit → Vec<EvidenceEnvelope>
  → CollectionRun + CollectionBatch
```

Scheduler / assurance:

```text
EvidenceCollector::collect(&scope) → Vec<EvidenceEnvelope>
```

No registry lookup, no observation gate, no exclusive factory.

### 3.6 GitHub behavior that must remain (neighbor SSOT)

Do **not** re-specify mappings here. Neighbor `sdd_github_collector_target` already encodes:

- Canonical emitted types (`GITHUB_CANONICAL_EVIDENCE_TYPES`); historical `GITHUB_EVIDENCE_TYPES` `source.*` needles; no `evidence.identity.*` on that const; no `evidence.github.*`.
- 403 / 401 → diagnostic / insufficient, **never** `protected=false` / `enabled=false`. 404 on protection → observed absent. Partial run continues other subjects.
- `collect_batch` records a real `CollectionRun`. Tokens never in envelopes, diagnostics, fixtures, or `configuration_digest`.
- GitHub sources contain no ISO/SOC2/NIS2/DORA / Effective / Ineffective.

Increment 1 is a **structural** move of *who seals*. It is not a GitHub feature increment.

### 3.7 Baseline suite must encode (characterize TODAY)

`sdd_collector_hexagonal_baseline` (suggested ids `chx_b001`…):

1. Types live in `src/lib.rs`: `CollectorCapabilities` has the eight fields; `CollectorDescriptor`, `CollectorScope`, `CollectionRequest`, `CollectionBatch` (with `errors: Vec<String>`), `EvidenceCollector` are defined or re-exported from `lib.rs` without `src/domain/`.
2. No `src/domain/`, `src/application/`, `src/ports/`, `src/adapters/` directories.
3. `github/normalize.rs` (and local/fixture) construct `EvidenceProvenance` and call `EvidenceEnvelope::seal`.
4. `GitHubCollector::new` takes a token; `GitHubClient` has `token`; no `CollectorInstance` / `CredentialRef` symbols.
5. No `CollectionEngine`, `CollectorRegistry`, `ObservationGate`, `EnvelopeFactory` symbols.
6. Public `use weeping_angel_collector::{EvidenceCollector, GitHubCollector, LocalCollector}` compiles.
7. 403≠false still holds (may source-scan or call existing GitHub helper; do not duplicate `ghc_*` goldens).

Baseline **must stay GREEN** until target is GREEN and baseline is explicitly `#[ignore]`-superseded.

---

## 4. Desired behavior (increment 1 — Phases 1–6 + facade)

### 4.1 Phase 1 — Extract collector domain

Move types from `src/lib.rs` into:

```text
src/domain/mod.rs
src/domain/capabilities.rs
src/domain/descriptor.rs
src/domain/scope.rs
src/domain/collector.rs          # EvidenceCollector trait + CollectorError (or split)
src/domain/observation.rs        # ObservationCandidate
src/domain/coverage.rs           # internal coverage notes; do not change public CollectionBatch
src/domain/diagnostic.rs         # may introduce types internally; CollectionBatch.errors stays Vec<String>
src/domain/batch.rs              # CollectionRequest / CollectionBatch / ObservationBatch
src/domain/cursor.rs             # stub/module only; no new incremental GitHub cursor behavior
src/domain/instance.rs           # CollectorInstance, CredentialRef
```

Preserve `CollectorCapabilities` field set and `CollectorDescriptor` field set. **Do not** introduce `CollectorId` / `EvidenceTypeId` newtypes until this structural refactor is GREEN.

`lib.rs` becomes a **facade**: `pub mod domain;` (or private modules + re-exports), `pub use` of public names.

### 4.2 Phase 2 — Collector instance ≠ collector type

```text
CollectorInstance {
  id,              // e.g. "github:xylex-group"
  collector_id,    // e.g. "collector.github"  (type)
  configuration,   // non-secret instance config (org, API host, selectors already expressible)
  credential_ref,  // CredentialRef — NOT a token
}
```

- Type: `collector.github`. Instance: `github:xylex-group`.
- Tokens / PATs / installation secrets are **not** fields of `CollectorInstance`.
- `CredentialRef` is an opaque handle (string id is enough this increment). Resolution to `GitHubClient` remains adapter/constructor concern.
- `GitHubCollector::new(token)` may remain as a **compatibility constructor** that wires a client. Hexagonal code paths take `CollectorInstance` + resolved client, never a secret on the instance struct.

### 4.3 Phase 3 — Adapters emit observations, not envelopes

Adapters (GitHub, local, fixture, manual) emit:

```text
ObservationCandidate {
  asset,
  evidence_type,
  facts,
  narrative,
  observed_at,
  valid_from,
  valid_until,
  source_revision,
}
ObservationBatch { candidates, diagnostics, /* coverage/hole flags as needed internally */ }
```

They **MUST NOT** construct `EvidenceEnvelope` or `EvidenceProvenance`.

`github/normalize.rs::emit` becomes observation construction (same evidence types, facts, narratives as today). Protection/security/workflows/collaborators/rulesets return candidates. `GitHubCollector` internally returns an `ObservationBatch`; the public `EvidenceCollector` / `collect_batch` facade still returns envelopes via the engine.

GitHub **behavior** (which candidates, which diagnostics, 403/404, pagination honesty) is unchanged. Only the **type** of the adapter output changes.

### 4.4 Phase 4 — Application layer

```text
src/application/mod.rs
src/application/engine.rs      # CollectionEngine
src/application/registry.rs    # CollectorRegistry
src/application/gate.rs        # ObservationGate
src/application/envelope.rs    # EnvelopeFactory
```

```text
src/ports/mod.rs
src/ports/adapter.rs           # CollectorAdapter: collect observations for an instance + request
```

Flow (normative):

```text
CollectionRequest
  → CollectorRegistry (resolve type / instance → CollectorAdapter)
    → CollectorAdapter::collect → ObservationBatch
      → ObservationGate::validate
        → EnvelopeFactory::seal_batch
          → CollectionBatch
```

`CollectionEngine` owns that pipeline. Adapters do not call the factory.

### 4.5 Phase 5 — Only EnvelopeFactory seals

- **Only** `EnvelopeFactory` constructs `EvidenceProvenance` and calls `EvidenceEnvelope::seal`.
- GitHub normalizer **loses all knowledge** of `EvidenceProvenance` / `EvidenceEnvelope`.
- Local, fixture, and manual paths go through the same factory (manual may remain a thin helper that builds a candidate then factory-seals).
- Factory consumes `EvidenceEnvelope::seal` **as-is** (Prompt 3). Provenance fields stay `{ collector_id, collected_at, scope, asset }`. Digest law unchanged → **evidence IDs unchanged** for the same observation+provenance bytes.
- `collector_id` on provenance remains the **type** id (`collector.github`), not the instance id, unless a later phase documents a provenance-schema change (out of increment 1 — changing it would change digests). Instance id may live on `CollectionRun` / configuration digest / future extensions, not by silently rewriting `EvidenceProvenance.collector_id`.

### 4.6 Phase 6 — Compatibility facade (do not rewrite scheduler)

Keep compiling:

```rust
use weeping_angel_collector::{EvidenceCollector, GitHubCollector, LocalCollector};
```

`GitHubCollector` implements `EvidenceCollector` by `CollectorAdapter` → `CollectionEngine` → envelopes. `collect_batch` stays on `GitHubCollector` (or a facade inherent method) with the same signature and `CollectionBatch` shape.

Do **not** rewrite `AssuranceScheduler` this increment.

### 4.7 ObservationGate (defense in depth)

Gate validates adapter output **before** seal:

- candidate evidence type is declared on the descriptor (COL-001 / assurance-runtime);
- narrative is not a compliance claim; no `control_test_result` / effectiveness;
- asset in scope;
- no credential-shaped fact keys (seal also rejects — defense in depth);
- permission/transport holes are diagnostics, not negative facts (403≠false);
- missing coverage is not rewritten as success.

Failures become `CollectorError` and/or `CollectionBatch.errors` strings **compatible with today** (GitHub already pushes diagnostic strings and continues). Do not change 403/404 GitHub semantics.

### 4.8 Target suite must encode (RED on current code, GREEN after implement)

`sdd_collector_hexagonal_target` (suggested ids `chx_t001`…):

1. `crates/weeping-angel-collector` is still a **single** Cargo package (parse crate `Cargo.toml`; no workspace member `weeping-angel-collector-*`).
2. Directories exist: `src/domain/`, `src/application/`, `src/ports/`, `src/adapters/` (adapters dir may re-export `github` / `local`). `lib.rs` is a facade (does not define `CollectorCapabilities` / `CollectorDescriptor` inline).
3. Domain files exist: `capabilities.rs`, `descriptor.rs`, `scope.rs`, `collector.rs`, `observation.rs`, `coverage.rs`, `diagnostic.rs`, `batch.rs`, `cursor.rs`, `instance.rs` under `src/domain/`.
4. `CollectorCapabilities` still has the eight fields; `CollectorDescriptor` field set preserved; no required `CollectorId` newtype.
5. `CollectorInstance` has `id`, `collector_id`, `configuration`, `credential_ref`. Instance id is distinct from type id in a unit assertion (`github:xylex-group` vs `collector.github`). `CollectorInstance` source does not contain token/PAT fields.
6. GitHub adapter/normalizer sources do **not** mention `EvidenceEnvelope` or `EvidenceProvenance` (except possibly comments forbidding them). `normalize.rs` does not call `seal`.
7. `CollectionEngine`, `CollectorRegistry`, `ObservationGate`, `EnvelopeFactory` types exist. Source-scan: `EvidenceProvenance {` and `EvidenceEnvelope::seal` appear in application envelope factory, not in `src/github/normalize.rs` / local collect body.
8. Public `use weeping_angel_collector::{EvidenceCollector, GitHubCollector, LocalCollector}` still type-checks; `GitHubCollector::collect_batch` still exists.
9. GitHub collector remains framework-blind (reuse `ghc_024` needles on `github_src()`).
10. Neighbor: this file does **not** replace `sdd_github_collector_target`. Optionally `cargo` the github target from docs; do not copy goldens.

Protocol: spec (this file, no product feature code) → baseline GREEN on current code → target RED → implement → ADR 0013 accept → target GREEN → supersede baseline (`#[ignore]`) → target still GREEN. `sdd_github_collector_target` GREEN **throughout**.

---

## 5. Hard invariants (collectors MAY / MUST NOT)

### Collectors MAY

- connect to a provider
- discover subjects in scope
- retrieve provider state
- interpret provider responses
- normalize to canonical **observations** (not framework results)
- report incompleteness / partial coverage
- report permission and transport failures

### Collectors MUST NOT

- decide ISO (or SOC2/NIS2/DORA) compliance
- evaluate canonical tests
- emit `ControlTestResult` / Effectiveness
- decide readiness
- construct SoA
- accept risk
- schedule themselves
- persist assurance state
- own evidence history (ledger owns persistence)
- invent provenance (factory + `EvidenceEnvelope::seal` own it)
- silently convert unavailable evidence into negative facts

**403 ≠ false. Missing coverage is not success.**

---

## 6. Preserve (non-negotiable)

| Must not change this increment |
| --- |
| Evidence mappings (GitHub API → canonical type/facts) |
| GitHub collection behavior (pagination honesty, default-branch protection, goldens) |
| Evidence IDs (digest of observation+provenance) |
| Provenance **semantics** (`EvidenceProvenance` fields; type id in `collector_id`) |
| Control results (collectors still do not compute them) |
| `GITHUB_EVIDENCE_TYPES` contents |
| 403/404 semantics from [`github-collector.md`](github-collector.md) |
| `sdd_github_collector_target` GREEN; no weakened `ghc_*` |

---

## 7. Suggested on-disk layout after increment 1

```text
crates/weeping-angel-collector/
  Cargo.toml                          # still one package
  src/lib.rs                          # facade re-exports
  src/domain/                         # Phase 1
  src/application/                    # Phase 4–5
  src/ports/
  src/adapters/mod.rs                 # re-export github + local
  src/github/                         # KEEP on disk (ghc github_src)
  src/local/
```

`src/adapters/github` as a **physical** tree is optional in increment 1; a re-export is enough. Cursor/coverage modules may be thin placeholders.

---

## 8. Architecture invariants COL-001…015 (spec only)

Prompt 1 owns `architecture/invariants.toml` and `cargo xtask guard`. **Do not implement guards this increment.** Declare:

| ID | Invariant |
| --- | --- |
| COL-001 | Collectors are framework-blind: no ISO/SOC2/NIS2/DORA requirement IDs, no effectiveness, no `ControlTestResult` in adapter output. |
| COL-002 | Collectors emit observations/facts only. Never GitHub → ISO check. Framework mapping lives outside this crate. |
| COL-003 | Hexagonal modular monolith: `domain/`, `application/`, `ports/`, `adapters/` exist; `lib.rs` is a facade. Still one Cargo package. |
| COL-004 | Only `EnvelopeFactory` constructs `EvidenceProvenance` and calls `EvidenceEnvelope::seal`. |
| COL-005 | Adapters emit `ObservationCandidate` / `ObservationBatch`, never envelopes. |
| COL-006 | Collector **instance** (`id`) is distinct from collector **type** (`collector_id`). |
| COL-007 | `CollectorInstance` holds `CredentialRef` only — no secret material. |
| COL-008 | Unavailable evidence is a diagnostic. **403 ≠ false.** |
| COL-009 | Missing / partial coverage is not success (`inventory.complete` honesty unchanged). |
| COL-010 | Collectors do not persist assurance state or own evidence history (ledger does). |
| COL-011 | Collectors do not schedule themselves (scheduler does). |
| COL-012 | Collectors do not construct SoA, decide readiness, or accept risk. |
| COL-013 | Collectors do not invent provenance. |
| COL-014 | ObservationGate validates adapter output before seal (defense in depth with `seal`). |
| COL-015 | Public facade remains `EvidenceCollector` / `GitHubCollector` / `LocalCollector` until Phase 26. |

These IDs are for later `architecture.toml` + xtask (Phases 24–25). They must not be confused with assurance-runtime test functions COL-001…006.

---

## 9. Dual-suite protocol

1. **Spec first** (this file). No product feature code in the spec-only slice.
2. Write `tests/contracts/collector_hexagonal.baseline.rs` + register `sdd_collector_hexagonal_baseline`. **GREEN on current monolith.**
3. Write `tests/contracts/collector_hexagonal.target.rs` + register `sdd_collector_hexagonal_target`. **RED on current code.**
4. Implement Phases 1–6 + facade in `crates/weeping-angel-collector` only.
5. Accept ADR 0013.
6. Target GREEN. Supersede baseline with `#[ignore = "superseded by sdd_collector_hexagonal_target"]`.
7. Re-run target; still GREEN. `sdd_github_collector_target` GREEN throughout.

Do not rewrite `github_collector.target.rs` semantics. New hexagonal assertions belong in the new files.

---

## 10. Remaining backlog (do **not** implement this increment)

| Phase | Backlog |
| --- | --- |
| 7 | Typed diagnostics replacing `Vec<String>`. Types may exist internally for `ObservationBatch`; **public** `CollectionBatch.errors` stays `Vec<String>`. |
| 8 | First-class public `CollectionCoverage` if it would change `CollectionBatch`. |
| 9–10 | Structured `SubjectSelector`. Keep existing `CollectorScope` API this increment. |
| 13–21 | Live GitHub transport / DTO / normalize split / cursors / artifacts. Structural moves of existing `github/` under `adapters/github/` only if `src/github/` and `ghc_*` stay GREEN. |
| 22–23 | New conformance suites beyond the hexagonal dual-suite needed here. |
| 24–25 | `architecture.toml` + `cargo xtask guard` for COL-001…015 (Prompt 1). Spec-only now. |
| 26 | Public API shrink after scheduler migrates. |
| 27 | Scheduler `CollectionEngine` adoption. **Do not rewrite `AssuranceScheduler` now.** |
| 28 | Ledger integration changes. |
| 29–30 | Hosted expansion. |
| never | Athena-specific collectors; framework-specific GitHub evidence; crate split. |

---

## 11. Out of scope (increment 1)

- Changing GitHub evidence contracts, goldens, or `GITHUB_EVIDENCE_TYPES`
- Rewriting `AssuranceScheduler` or assurance `assess()` collection edge
- Editing `EvidenceEnvelope::seal`, ledger, temporal, lineage, SoA
- Catalog / framework / readiness product changes
- xtask / architecture manifests / debt register
- Mass-reformat, README, schema hygiene
- `CollectorId` newtypes as a prerequisite
- Changing public `CollectionBatch.errors` away from `Vec<String>`
- Live HTTP (`reqwest` / `octocrab`)
- New workspace crates
- `tests/sdd/`
- Overwriting [`github-collector.md`](github-collector.md)

---

## 12. Risks

- **Digest / evidence-id drift** if factory fills provenance differently than today’s GitHub `emit` (`collector_id`, `collected_at`, `scope` label, `asset`). Mitigate: same provenance bytes; type id not instance id on `EvidenceProvenance`.
- **`github_src()` breakage** if implementation leaves `src/github/` empty or moves needles. Mitigate: keep `src/github/` as the GitHub adapter sources.
- **Scheduler compile break** if `EvidenceCollector` methods or `Send + Sync` change. Mitigate: facade trait identical.
- **403≠false regression** while changing `emit` to candidates. Mitigate: keep GitHub fetch/diagnostic strings; gate must not fabricate negatives; neighbor suite GREEN.
- **Guard 15 red** until Prompt 1 lists this spec. Expected; do not edit `architecture/spec-lifecycle.toml` here.
- **COL- ID collision** with assurance-runtime COL-001…006. Mitigate: [§8](#8-architecture-invariants-col-001015-spec-only) titles + later toml descriptions.
- **Token still on `GitHubClient`** this increment vs COL-007. Mitigate: COL-007 applies to `CollectorInstance`; client/constructor compatibility is explicit until Phase 13+.
- **Concurrent Prompt 3 seal changes.** Consume current `seal`; do not fork.

---

## 13. Acceptance (increment 1)

Testable:

- [ ] Crate is still one Cargo package `weeping-angel-collector`.
- [ ] `domain/`, `application/`, `ports/`, `adapters/` exist; `lib.rs` is a facade.
- [ ] Domain modules listed in Phase 1 exist; capabilities 8 fields and descriptor fields preserved; no forced `CollectorId` newtypes.
- [ ] GitHubCollector / normalizer does not construct `EvidenceEnvelope` or `EvidenceProvenance`.
- [ ] GitHubCollector does not know ISO 27001 (existing `ghc_024` source law).
- [ ] GitHubCollector returns observation candidates internally; `CollectionEngine` / `EnvelopeFactory` seal envelopes.
- [ ] `ObservationGate` validates adapter output (defense in depth with `EvidenceEnvelope::seal`).
- [ ] Collector instance distinct from collector type; credentials on the instance are refs, not secrets.
- [ ] `use weeping_angel_collector::{EvidenceCollector, GitHubCollector, LocalCollector}` still works; `GitHubCollector::collect_batch` compile-stable.
- [ ] `sdd_github_collector_target` GREEN; no evidence ID / mapping / behavior change.
- [ ] `sdd_collector_hexagonal_target` GREEN; baseline superseded after target GREEN.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test -p weeping-angel-collector`
- [ ] `cargo test --test sdd_github_collector_target`
- [ ] `cargo test --test sdd_collector_hexagonal_target`
- [ ] `cargo check --workspace`

---

## 14. Related

- Evidence contract: [`github-collector.md`](github-collector.md)
- Draft decision: [`docs/adr/0013-collector-hexagonal-modular-monolith.md`](../adr/0013-collector-hexagonal-modular-monolith.md)
- Public trait: [`assurance-runtime.md`](assurance-runtime.md)
- Scheduler: [`continuous-assurance-scheduler.md`](continuous-assurance-scheduler.md)
- Docs layout: [ADR 0004](../adr/0004-documentation-architecture.md)
- Architecture-as-law: [ADR 0010](../adr/0010-architecture-as-law.md)
- Envelope seal / clocks: [`temporal-lineage-evidence-soa.md`](temporal-lineage-evidence-soa.md) (Prompt 3)
