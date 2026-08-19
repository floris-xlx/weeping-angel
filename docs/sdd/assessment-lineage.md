# SDD: Immutable Assessment Lineage, Explainability, and Report Cleanup

| Field | Value |
| --- | --- |
| Status | **Specified** — dual-suite registered; baseline GREEN on current shortcuts; target LIN-001–008 and LIN-010–014 authored so the suite is **RED on CURRENT** (LIN-009 / LIN-015 still PASS). No product feature code yet. |
| Program | Canonical Assurance Catalog v1 — Prompt 11 |
| Source prompt | [`docs/prompts/canonical-assurance-v1/11-assessment-lineage.md`](../prompts/canonical-assurance-v1/11-assessment-lineage.md) |
| Slice | Persistable execution lineage, explanation projection, pure report serialization, generic framework facade, snapshot compare |
| Dual-suite | `sdd_assessment_lineage_baseline` · `sdd_assessment_lineage_target` (`tests/sdd/assessment_lineage.{baseline,target}.rs`) — **already registered** in root `Cargo.toml` |
| ADR | Draft [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md) — finalize when public contracts / storage seams land |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — update in implement, not this spec-only phase |
| Protocol report | [`docs/sdd/sdd-assessment-lineage.md`](sdd-assessment-lineage.md) |
| Spine (still law) | [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Catalog infra | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |
| Typed evidence | [`docs/sdd/typed-evidence.md`](typed-evidence.md) |
| Population | [`docs/sdd/population-runtime.md`](population-runtime.md) |
| Applicability engine (Prompt 10) | **Out of this slice.** Concurrent; do **not** add `OrgContext`, `ManualDeterminationRequired`, or `evaluate_org_context`. Persist `ApplicabilitySnapshot` from existing `ApplicabilityRule` + `statically_applicable` + pack rows. Unknown predicates stay unresolved, never false. |
| GitHub collector (Prompt 09) | **Collision fence.** Do not edit `tests/sdd/github_collector.*` or `crates/weeping-angel-collector/src/github/**`. |
| ISO remap (Prompt 12) | **Collision fence.** Do not rewrite `catalog/canonical/v1` domain TOML or ISO pack IDs / `to =` mappings. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`). `Assessment` is already `type Assessment = AssessmentDefinition`. |
| Catalog schema | `weeping-angel/canonical-catalog/v1` |
| Pack schema | `weeping-angel/framework-pack/v1` |
| Workspace verify | `cargo test --workspace --features demo`; keep `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_canonical_assurance_catalog_target` GREEN |

This document is the durable SSOT for Prompt 11. It owns **persistence/orchestration lineage**, **explanation projections**, **generic report cleanup**, and **framework-generic facade fixes**. It does **not** own domain catalog content, Prompt 09 collector mapping, Prompt 10 evaluation semantics, Prompt 12 ISO remapping, UI, or multi-tenant SaaS.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

An assessment is a **reproducible immutable execution artifact**, not a current-state report computed from whatever files sit on disk at serialize time.

---

## 0. Collision fence (concurrent SDD)

This slice may edit only lineage / explain / report-serialization / snapshot / ledger persist / generic-facade paths.

| Do not touch | Owner |
| --- | --- |
| `tests/sdd/github_collector.*`, `crates/weeping-angel-collector/src/github/**` | Prompt 09 |
| `OrgContext`, `ManualDeterminationRequired`, `evaluate_org_context`, Prompt 10 evaluator modules | Prompt 10 |
| `catalog/canonical/v1/**` domain TOML, ISO pack requirement/control IDs, pack `to =` remaps | Prompt 12 / catalog owners |
| `tests/sdd/iso27001_remap.*` | Prompt 12 |

Suggested **new** product modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| `ControlExplanation`, `AssessmentSummary`, `CoverageMetrics`, run assembly, compare | `weeping-angel-assurance` |
| CLI execution for `assurance explain` | `src/assurance_explain.rs` (or sibling of `assurance_catalog.rs`) |
| Persist/load of lineage rows | `weeping-angel-evidence::ledger` (opaque JSON payloads; crate still owns observations, never conclusions) |
| Generic pack resolve | `weeping-angel-framework::pack` — callers pass `(id, version)`; no ISO-only branches in generic code |

Tiny allowed adjustments: optional serde-default fields on `AssessmentRun` / `AssessmentReport` / `SnapshotDiff`; new snapshot structs; ledger methods; CLI `Explain` arm. Do **not** redesign catalog TOML, IR `AssessmentDefinition` core fields, collector discovery, or ISO pack IDs.

---

## 1. Problem / user-visible goal

Operators cannot reconstruct *why* an assessment said what it said. `AssessmentRun` exists as a type and as empty SQLite tables, but the facade builds one, copies the compile digest into every identity field, records no collector runs, and drops it (`let _run`). JSON reports re-load `frameworks/iso-27001/2022` while serializing. Non-ISO profiles compile a **production stub** assessment. SoA projection rereads today’s pack `applicability.toml`. Compare only notices effective/ineffective/stale. There is no `explain` command.

That means:

- changing the current catalog or ISO pack can silently change what a stored report *appears* to say;
- failed or partial collection cannot be distinguished from a completed empty collection;
- a reviewer cannot name the evidence digest, test version, applicability decision, or exception that produced a control result;
- coverage is collapsed into on-the-fly `"NN%"` strings computed at serialize/project time.

**User-visible goal:** given an assessment id and a control id, reconstruct the pinned lineage and answer:

```text
why was this control evaluated?
what population was in scope?
which evidence digests were used or missing?
which subjects failed or were missing?
which expression / test version ran?
which exceptions influenced the result?
which framework requirements map to the control?
```

Replay of a historical assessment must use **pinned snapshots**. If current pack/catalog files no longer match the pinned digest, the runtime must detect the mismatch rather than silently recompute against current files.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `e430980c0d27a8138a153d49b62ddf3c57827891`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `AssessmentRun` | `weeping-angel-assurance::snapshot` | Grow into a real persisted record. Keep existing camelCase field names. Add pins this prompt requires (catalog digest, collector ids/versions, applicability snapshot id, status that can be partial/failed). |
| `SnapshotDiff` | same | Fields already exist for subjects / applicability / exceptions. **Populate them.** Prefer extending this struct for digest-change fields. |
| `compare` | same | Today only effectiveness + stale. Must identify the change classes in §4.8. |
| `AssessmentReport` | `weeping-angel-assurance::lib` | Rust fields today are `{assessmentId, profile, digest, results, evidenceCount}`. Serialization must become **pure**. Carry explicit summary / metrics / pack+catalog digests on the value; do not load packs in `Serialize`. |
| `assessment_for_target` | facade | Remove production stub + ISO-only branch. One registry/loader path. |
| `stub_catalog` | `weeping-angel-framework` | ISO-only pack fallback today. Must not remain a hidden production catalog. Test fixtures only. |
| `normalize` | framework compile | ISO-only `load_framework_pack("iso-27001","2022")`. Must use target identity, not a hardcoded pack. |
| `project_soa` | `soa.rs` | Must project from a **pinned** pack/applicability snapshot, not live disk. Persist `StatementOfApplicabilitySnapshot`. |
| `project_readiness` | `readiness.rs` | Keep as a projection. Persist `FrameworkReadinessSnapshot`. Do not collapse metrics into one compliance %. |
| Ledger tables | `weeping-angel-evidence::ledger` | `assessment_runs`, `control_test_runs`, `framework_snapshots` already exist with `(id/digest, payload)` and **no** persist/load APIs. Add APIs. Do not redesign envelope append-only semantics. |
| `CollectionRun` | evidence crate | Already has `status`, `error_count`, `collector_id`, `collector_version`. Record them on the run and in the ledger. |
| `CanonicalCatalog::{load,validate,digest}` | catalog crate | Consume digest string for `CanonicalCatalogSnapshot`. Do not fork the loader. |
| `FrameworkPackDigest` / `load_framework_pack` | framework pack | One loader. `load_framework_pack(id, version)` and `load_framework_pack_from` stay the only resolution path. `wa-baseline/1` uses the same path. |
| `ApplicabilityRule` | IR | Prompt 10 evaluator is **absent / concurrent**. Snapshot the rule tree + static outcome + pack rows. Do not add org-context evaluation here. |
| `Exception` | IR | Already has `subjects`, `status`, `expires_at`. Lineage and compare must surface them. Evaluator already can hold exceptions on `EvidenceSet`. |
| `AssessmentDefinition` | IR (`type Assessment = AssessmentDefinition`) | Snapshot it. Do not fork a second definition type. |
| `ControlTestResult` | control-test `result.inc` | Already carries `evidence_refs`, `missing_evidence`, `test_version`, `input_digest`, `population`, `evaluatedAt` (`checked_at`), `duration`. Result identity **excludes** `duration` and `evaluatedAt`. |
| CLI `AssuranceCommand` | `src/cli.rs` | Add `Explain`. Catalog stays dispatched. `explain` must not stay a silent success stub. |
| Dual-suite neighbors | root `Cargo.toml` | Lineage suites already sit next to existing `sdd_*`. Do not disturb green targets listed in the header. |

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `e430980c0d27a8138a153d49b62ddf3c57827891`. Encoded by `tests/sdd/assessment_lineage.baseline.rs`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 `AssessmentRun` is built then dropped

[`crates/weeping-angel-assurance/src/lib.rs`](../../crates/weeping-angel-assurance/src/lib.rs) `AssuranceEngineBuilder::assess`:

```text
let _run = AssessmentRun {
    id: assessment.id.clone(),
    framework: target.profile.as_selector().into(),
    framework_pack_digest: load_framework_pack("iso-27001", "2022")
        .map(|p| p.digest.0)
        .unwrap_or_default(),
    assessment_definition_digest: compiled.digest.clone(),
    started_at: Utc::now().to_rfc3339(),
    completed_at: Utc::now().to_rfc3339(),
    scope: "assess".into(),
    collector_runs: Vec::new(),
    evidence_snapshot_digest: compiled.digest.clone(),
    result_digest: compiled.digest.clone(),
    status: "completed".into(),
};
```

Facts:

- `_run` is never returned, stored, or compared.
- `collector_runs` is always empty even after `collector.collect`.
- `assessment_definition_digest`, `evidence_snapshot_digest`, and `result_digest` are the **same** compile digest.
- Pack digest is hardcoded ISO 27001:2022 (empty string if that pack is missing).
- Status is always `"completed"`; there is no `partial` / `failed` / `started` path on the run.
- `started_at` and `completed_at` are both `Utc::now()` after evaluate; collection is not timed.

`AssessmentRun` fields (already camelCase): `id`, `framework`, `frameworkPackDigest`, `assessmentDefinitionDigest`, `startedAt`, `completedAt`, `scope`, `collectorRuns`, `evidenceSnapshotDigest`, `resultDigest`, `status`. No `canonicalCatalogDigest`, no `applicabilitySnapshotId`.

### 3.2 `AssessmentReport::serialize` loads a pack and invents percentages

Custom `Serialize` for `AssessmentReport`:

- calls `load_framework_pack("iso-27001", "2022")` (filesystem lookup);
- counts effectiveness buckets;
- formats `automationCoverage` and `evidenceCoverage` as `"{:.0}%"`;
- writes `disclaimer` / `banner`, `frameworkPackDigest`, derived `requirements` / `controls` lists;
- drops `partial` via `let _ = (partial, "collectionRunId", "evidenceRefs")`.

The in-memory struct has **no** stored summary, pack digest, catalog digest, collection-run ids, or coverage metrics. Serializing the same report after the on-disk ISO pack changes can change `frameworkPackDigest` without re-running assess.

This is **not** pure: pack loading + current-state resolution happen inside serialization.

### 3.3 Production stub assessment + ISO-only fallbacks

`assessment_for_target`:

- if profile is `Iso27001` **and** version is `"2022"` **and** `load_framework_pack("iso-27001", "2022")` succeeds → `assessment_from_pack`;
- else builds a **production** stub: `canonical:stub-1` / `canonical.source-control` / partial mapping / `ev.branch_protection` / id `assess-runtime-1`.

`stub_catalog(profile)`:

- `Iso27001` → pack requirements (or `[]` on load failure);
- every other `FrameworkProfile` (`Iso27701`, `Gdpr`, `Soc2`, `Nis2`, `Dora`, `Iso27007`) → `[]`.

`normalize` (compile stage):

- if target is ISO 27001:2022, merge `load_framework_pack("iso-27001", "2022")`;
- other profiles (including on-disk `wa-baseline/1`) do not merge a pack through this branch.

There is no registry that maps *any* `(framework, version)` the same way for assess, serialize, SoA, and stub catalog. ISO is special-cased in three crates/modules.

### 3.4 `compare` only checks effectiveness / stale

[`crates/weeping-angel-assurance/src/snapshot.rs`](../../crates/weeping-angel-assurance/src/snapshot.rs):

`SnapshotDiff` already has:

```text
controlBecameEffective / Ineffective
evidenceBecameStale
newSubjects / disappearedSubjects
requirementBecameApplicable / NotApplicable
manualReviewResolved
newExceptions / expiredExceptions
```

`compare(previous, next)` only walks `controls` and fills the first three (effective, ineffective, stale). Subject, applicability, exception, catalog/pack digest, and evidence add/remove/supersession are never computed. Inputs are two `FrameworkReadinessSnapshot`s, not two `AssessmentRun`s.

### 3.5 `project_soa` reads current pack files from disk

[`crates/weeping-angel-assurance/src/soa.rs`](../../crates/weeping-angel-assurance/src/soa.rs):

```text
resolve_pack_dir(framework, version) → read applicability.toml → parse [entry]
```

No snapshot identity, no digest pin, no evidence, no exceptions, `implementation_state = "assessed"`, `manual_review_state = "pending"`. Historical SoA cannot be distinguished from a later pack edit.

`project_readiness` does compute counts and `"NN%"` coverage strings into `FrameworkReadinessSnapshot`, but that snapshot is not persisted as part of an assessment run, and `has_partial = true` forces every fully-effective requirement to `"partially covered"`. `evaluated_at` is wall-clock.

### 3.6 CLI: no Explain; non-catalog arms banner-and-exit-0

[`src/cli.rs`](../../src/cli.rs) `AssuranceCommand`:

```text
Framework | Collect | Evidence | Assess | Result | Compare | Soa | Catalog
```

No `Explain` variant. No `--assessment` / `--control` explain args.

[`src/main.rs`](../../src/main.rs): `Catalog` is dispatched to `assurance_catalog::run`. Every other assurance subcommand prints `"This is a readiness assessment and is not certification."` and returns **exit 0**.

### 3.7 Ledger tables without lineage APIs

[`crates/weeping-angel-evidence/src/ledger.rs`](../../crates/weeping-angel-evidence/src/ledger.rs) `init` creates:

```text
evidence_envelopes          — append / get / query (real APIs; INSERT OR IGNORE)
evidence_artifacts          — schema only
collection_runs             — record_collection_run (INSERT OR REPLACE)
assessment_runs             — CREATE TABLE only (id, payload)
control_test_runs           — CREATE TABLE only (id, payload)
framework_snapshots         — CREATE TABLE only (digest, payload)
```

There is no `persist_assessment_run` / `load_assessment_run` / `persist_control_test_run` / `persist_framework_snapshot`. No tables for catalog snapshots, assessment-definition snapshots, applicability snapshots, or evidence snapshots as first-class rows (evidence envelopes themselves are append-only).

`CollectionRun` already models `status` (constructor starts as `"started"`), `error_count`, `evidence_count`, collector id/version, but `assess` never constructs or records one. `collector.collect` returning `Err` aborts `assess`; there is no partial assessment path.

### 3.8 Prompt 10 applicability engine is absent

IR `ApplicabilityRule` / `ApplicabilityPredicate` exist. `statically_applicable()` returns `Some(true|false)` only for `Always`/`Never` (and boolean combinations); **predicates are `None`**.

Compile `resolve_applicability` keeps a requirement unless `statically_applicable() == Some(false)`. Predicates therefore stay in. There is no `ApplicabilitySnapshot`, no three-state org-context evaluator, no persisted rationale graph.

Pack `applicability.toml` is only consumed by `project_soa` (live disk).

### 3.9 Missing explanation and explicit metric types

No `ControlExplanation`. No `AssessmentSummary` / `CoverageMetrics` types. No `EvidenceSnapshot` / `FrameworkPackSnapshot` / `CanonicalCatalogSnapshot` / `AssessmentDefinitionSnapshot` / `ApplicabilitySnapshot` / `StatementOfApplicabilitySnapshot` persist types. No `ControlTestRun` type (only an empty ledger table name).

Coverage is two formatted percent strings on serialize / readiness, not separate first-class metrics for:

```text
control effectiveness
evidence
automation
subject
framework requirement
fresh-evidence
manual-review burden
```

### 3.10 Dual-suite registration (HEAD of this spec phase)

Root [`Cargo.toml`](../../Cargo.toml) registers lineage binaries:

```text
sdd_assessment_lineage_baseline → tests/sdd/assessment_lineage.baseline.rs
sdd_assessment_lineage_target   → tests/sdd/assessment_lineage.target.rs
```

Baseline encodes §3 shortcuts and must stay GREEN. Target currently asserts only:

- LIN-009: dual-suite names present in `Cargo.toml`
- LIN-015: neighbor `sdd_assurance_runtime_target` / `sdd_iso27001_assurance_target` / `sdd_canonical_assurance_catalog_target` remain registered

Those two tests **PASS** on CURRENT. LIN-001–008 and LIN-010–014 are authored in `tests/sdd/assessment_lineage.target.rs` and must stay **RED** on CURRENT (compile-safe, fail for the right reason) **before** product feature code.

---

## 4. Desired behavior

### 4.1 Immutable chain

Persist (ledger payload rows and/or typed snapshot documents with deterministic digests):

```text
FrameworkPackSnapshot
CanonicalCatalogSnapshot
AssessmentDefinitionSnapshot
ApplicabilitySnapshot
CollectionRun[]
EvidenceEnvelope[]          — already append-only
EvidenceSnapshot
ControlTestRun[]
AssessmentRun
FrameworkReadinessSnapshot
StatementOfApplicabilitySnapshot
```

An `AssessmentRun` must pin at least:

```text
framework pack digest
canonical catalog digest
assessment definition digest
collector IDs and versions
collection run IDs
evidence snapshot digest (and/or evidence envelope digests)
test IDs and versions
applicability snapshot identity / decisions
result digest
```

Replay **must not** depend on mutable current catalog/framework files. Loading current files is allowed only to **compare** their digest to the pin; mismatch is a detected error (`DigestMismatch` or equivalent), never a silent rewrite.

Historical evidence remains append-only (`INSERT OR IGNORE` / supersede keeps prior rows). Failed or partial collection is a **new** `CollectionRun` status (and an `AssessmentRun` status of `partial` or `failed`), never an overwrite of a completed run.

### 4.2 `AssessmentRun` is a real execution record

`assess` (or an explicit persist step the facade always performs when a ledger is present) must:

1. record `startedAt` before collection/evaluate;
2. record scope (facade allow-set **and** IR `AssessmentScope` when present);
3. record each `CollectionRun` (id, collector id/version, status, evidence_count, error_count);
4. seal an `EvidenceSnapshot` over the envelopes actually used (digest of sorted envelope digests + collection run ids);
5. persist `ControlTestRun` rows (test id, version, input digest, effectiveness, evidence refs, missing evidence, population summary, exception ids);
6. persist `AssessmentRun` with distinct definition / evidence-snapshot / result digests;
7. persist `FrameworkReadinessSnapshot` and `StatementOfApplicabilitySnapshot` derived from **pinned** inputs;
8. set `status` to `completed` | `partial` | `failed` without mutating prior runs.

Without a ledger, the run object must still be **returned** (or attached to `AssessmentReport`) so callers can persist later. Dropping `let _run` is forbidden.

Result digest is SHA-256 hex of canonical JSON over the semantic result document (test ids, effectiveness, evidence refs, missing evidence, population, exception ids) — **not** the compile digest and **not** wall-clock `duration` / `evaluatedAt` (`checked_at`). Domain-separate snapshot/result hashes from raw compile-digest reuse (schema field inside the hashed body, or `typed_canonical_digest`).

### 4.3 ApplicabilitySnapshot without Prompt 10

Do **not** implement the org-context three-state engine. Persist what already exists:

```text
ApplicabilitySnapshot {
  schema,                    # lineage snapshot schema id, not a new IR schema
  assessment_id,
  scope,                     # IR AssessmentScope if present
  requirement_decisions[],   # id, rule tree (or digest), static outcome, rationale
  control_decisions[],       # same
  pack_entries[],            # applicability.toml rows used, if any
  digest
}
```

Static outcome mapping until Prompt 10:

| Rule | Decision recorded |
| --- | --- |
| `Always` | applicable |
| `Never` | not applicable |
| predicate / unknown combo | included / unresolved (must not be treated as false) |

Prompt 10 may later replace the evaluator that *fills* this snapshot. The persist shape should already be loadable by explain/replay.

### 4.4 ControlExplanation + CLI

Generic projection (names may be camelCase in JSON; fit existing IR newtypes):

```text
ControlExplanation {
  control,
  applicability,
  implementation,            # ControlImplementation when present
  population,
  tests,                     # id + version + expr identity
  evidence_requirements,
  evidence,                  # envelope digests actually used
  missing_evidence,
  failing_subjects,
  missing_subjects,
  exceptions,
  mappings,
  effectiveness,
}
```

Must answer the user-visible questions in §1. Evidence entries **are digests**, not “latest from current ledger”.

CLI (parser in `src/cli.rs`, execution **not** inlined in the clap enum):

```text
weeping-angel assurance explain --assessment <id> --control <id>
```

Example:

```bash
weeping-angel assurance explain \
  --assessment assess-runtime-1 \
  --control control.identity.privileged-mfa
```

Exit non-zero if the assessment or control is unknown. Print the not-certification banner. Do not resolve a framework pack from disk to answer.

### 4.5 Pure serialization and explicit metrics

`AssessmentReport` serialization (and any JSON/Markdown/CSV report built from a completed assessment) is **pure**:

- no `load_framework_pack`;
- no network I/O;
- no filesystem lookup;
- no hidden current-state resolution.

Replace computed-on-serialize summaries with explicit structures carried on the report or a sibling projection:

```text
AssessmentSummary
FrameworkReadinessSnapshot
CoverageMetrics
```

`CoverageMetrics` exposes **separate** numbers (counts or ratios, not one compliance percentage):

```text
control effectiveness coverage
evidence coverage
automation coverage
subject coverage
framework requirement coverage
fresh-evidence coverage
manual-review burden
```

Do **not** emit a single `compliancePercent` / `isoCompliant` field. Keep the readiness-not-certification banner.

### 4.6 Framework-generic facade

Resolve every framework through **one** registry/loader path:

```text
(framework id, version) → resolve_pack_dir / load_framework_pack / load_framework_pack_from
```

Remove:

- hardcoded `load_framework_pack("iso-27001", "2022")` from generic serialize/orchestrate;
- ISO-only branches in `assessment_for_target` / `normalize` / `stub_catalog` that skip the same path;
- the production stub assessment (`canonical:stub-1` … `assess-runtime-1`).

Missing pack → fail closed (`UnknownPack` / `AssuranceError`), not a silent stub. Fixture/stub assessments may remain **`#[cfg(test)]`** or under `tests/` / explicit test helpers — never on the production assess path.

Spine ACT tests that relied on the stub for non-ISO profiles must be updated in the **target** suite / those tests’ own fixtures, without weakening ACT-003 collector-blindness or inventing new frameworks.

### 4.7 Collection failure is representable

`CollectionRun.status` must distinguish at least `started` | `completed` | `partial` | `failed`.

`AssessmentRun.status`:

- `completed` — every required collector finished without error;
- `partial` — at least one collector produced evidence and at least one failed/incomplete; results remain explainable for collected evidence;
- `failed` — collection or evaluate aborted; no silent rewrite of a previous completed run.

Partial is **visible** in lineage and explain (`missing_evidence`, collection run error counts). History is append-only. A completed empty collection is **not** the same record as a failed/partial run.

### 4.8 Snapshot comparison

`compare` (or a sibling that accepts two `AssessmentRun`s / lineage bundles) must identify:

| Change class | How |
| --- | --- |
| Applicability | requirement/control entered or left applicable set |
| Subject population | `newSubjects` / `disappearedSubjects` |
| Evidence add / remove / supersession | digest appeared, disappeared, or `supersedes` chain advanced |
| Test result | effectiveness or test version / input digest changed |
| Exceptions introduced / expired | `newExceptions` / `expiredExceptions` |
| Framework / catalog digest | pack digest or catalog digest changed |

Existing `SnapshotDiff` fields should be filled. Digest-change fields may be added (serde defaults) if reserved fields are insufficient.

### 4.9 Deterministic digests

Snapshot and result digests:

- SHA-256 hex of canonical `serde_json` (same law as IR `canonical_digest`);
- **not** a reused compile digest for definition / evidence-snapshot / result identities;
- domain-separated where they could collide with catalog/IR/pack hashes (prefix or schema field inside the hashed body);
- independent of map insert order (`BTreeMap` / sorted vecs);
- independent of wall-clock `duration` / `evaluatedAt` for **result** identity;
- identical across two processes given the same pinned inputs;
- pack/catalog hashes use existing digest law over **canonicalized structures**, not raw TOML bytes (Windows CRLF).

### 4.10 Dual-suite protocol

Already registered in root `Cargo.toml`:

```toml
[[test]]
name = "sdd_assessment_lineage_baseline"
path = "tests/sdd/assessment_lineage.baseline.rs"

[[test]]
name = "sdd_assessment_lineage_target"
path = "tests/sdd/assessment_lineage.target.rs"
```

| Gate | Suite | Expected |
| --- | --- | --- |
| Spec | this file | written **before** product feature code |
| Baseline on CURRENT | `sdd_assessment_lineage_baseline` | **GREEN** — characterizes §3 shortcuts |
| Target on CURRENT (today) | `sdd_assessment_lineage_target` | LIN-009 / LIN-015 PASS; LIN-001–008 and LIN-010–014 **RED** |
| Target-characterization | same target file | LIN-001–008 and LIN-010–014 authored — suite is **RED for the right reason** on CURRENT — **before** product feature code |
| Implement | lineage + explain + report cleanup | — |
| Target after | same target suite | **GREEN** |
| Baseline after | baseline | skip-supersede (`#[ignore = "superseded by sdd_assessment_lineage_target"]`); do not leave “run is dropped / serialize loads ISO” as required CI green |
| Neighbors | `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_canonical_assurance_catalog_target` | stay GREEN |
| Workspace | `cargo test --workspace --features demo` | GREEN after implement |

### 4.11 Baseline suite contents (GREEN on CURRENT)

Assert **today’s** shortcuts (already authored in `assessment_lineage.baseline.rs`):

1. Dual-suite names are registered in root `Cargo.toml`.
2. `assess` source contains `let _run` and `collector_runs: Vec::new()`.
3. `AssessmentRun` construction reuses `compiled.digest` for definition, evidence snapshot, and result identity.
4. `AssessmentReport` `Serialize` impl contains `load_framework_pack("iso-27001", "2022")` and formats `automationCoverage` / `evidenceCoverage` percentages.
5. `assessment_for_target` contains the production stub ids `canonical:stub-1` / `assess-runtime-1` and an ISO-only pack branch.
6. `stub_catalog` / `normalize` special-case `Iso27001` + `"iso-27001"` / `"2022"`.
7. `compare` body does not write `new_subjects`, `new_exceptions`, or catalog/pack digest fields.
8. `project_soa` calls `resolve_pack_dir` and reads `applicability.toml`.
9. `AssuranceCommand` debug/source has no `Explain` variant.
10. `src/main.rs` non-catalog assurance arm prints the banner and returns `0`.
11. Ledger `init` creates `assessment_runs` / `control_test_runs` / `framework_snapshots` but the ledger impl has no `persist_assessment_run` / `load_assessment_run` (or equivalent) methods.
12. No `ControlExplanation` / `ApplicabilitySnapshot` / `EvidenceSnapshot` type in product crates (string scan of `crates/`).
13. IR still exposes `ApplicabilityRule` and `statically_applicable`; there is no Prompt 10 org-context evaluator module to confuse this slice.

After implement they should fail or be `#[ignore = "superseded by sdd_assessment_lineage_target"]`.

### 4.12 Target suite contents (RED on CURRENT, GREEN after)

Stable titles. Prompt 11 proof list — **author these before product feature code** (compile-safe; fail because persist/explain/pure-serialize/generic facade are missing):

| ID | Assertion |
| --- | --- |
| LIN-001 | Historical assessment reconstructs from pinned snapshots (pack + catalog + definition + applicability + evidence + tests). Replay does not need current catalog/pack files when snapshots are present. |
| LIN-002 | Changing current catalog/pack files does **not** silently rewrite a stored assessment’s result digest or explanation; digest mismatch is detected if current files are consulted. |
| LIN-003 | `ControlExplanation` / `assurance explain` references **exact** evidence envelope digests used at evaluate time. |
| LIN-004 | Serializing `AssessmentReport` performs no framework resolution (no `load_framework_pack`, no pack path I/O). A test can wrap or source-scan the serialize impl and/or serialize in an environment where the ISO pack path is absent without changing JSON identity fields. |
| LIN-005 | Partial collector runs remain distinguishable (`CollectionRun.status` / `AssessmentRun.status` / error_count); a completed empty collection is not the same as a failed/partial run. |
| LIN-006 | Assessment diff identifies changed subjects and test results (and does not only flip effective/ineffective/stale). |
| LIN-007 | Exceptions approved/expired are visible on the lineage and in `ControlExplanation`. |
| LIN-008 | Snapshot and result digests are deterministic (two seals, insert-order independent, domain-separated from raw compile digest reuse). Wall-clock `duration` / `evaluatedAt` are excluded from result identity. |

Additional required locks (may share tests):

| ID | Assertion |
| --- | --- |
| LIN-009 | Dual-suite binaries registered in root `Cargo.toml`. (**already PASS**) |
| LIN-010 | Production `assessment_for_target` / assess path has no `canonical:stub-1` stub; fixtures remain test-only. |
| LIN-011 | Every framework id/version uses one loader path; no hardcoded `"iso-27001","2022"` in generic serialize/orchestrate. |
| LIN-012 | CLI `assurance explain --assessment <id> --control <id>` parses in `src/cli.rs` and is dispatched (not banner-exit-0). |
| LIN-013 | `CoverageMetrics` (or equivalent explicit fields) expose the seven metric families; no single compliance percentage field. |
| LIN-014 | Ledger can persist and load `AssessmentRun` and `ControlTestRun`; append-only — replacing a completed run’s payload with different bytes is rejected or ignored. |
| LIN-015 | `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_canonical_assurance_catalog_target` remain registered and this slice does not remap ISO pack IDs or add frameworks. (**already PASS**) |

One regression test per later review comment must be titled `P?: <exact subject>` and encode the original found case (test first → RED → fix → GREEN).

---

## 5. Acceptance criteria (testable)

1. Dual-suite `sdd_assessment_lineage_baseline` / `sdd_assessment_lineage_target` is registered in root `Cargo.toml`; baseline GREEN on current shortcuts; LIN-001–008 and LIN-010–014 authored so target is RED on current tree **before** product feature code; after implement, target GREEN and baseline skip-superseded.
2. `AssessmentRun` is returned/persisted (never `let _run`); records start/completion, scope, status (`completed`/`partial`/`failed`), collector run ids, evidence snapshot digest, framework pack digest, canonical catalog digest, assessment definition digest, applicability snapshot identity, result digest.
3. Immutable chain in §4.1 is persistable; replay reconstructs from pins (LIN-001); current file edits do not silently rewrite old results (LIN-002).
4. Historical evidence is append-only; partial/failed collection is representable without rewriting a completed run (LIN-005).
5. `ControlExplanation` exists; CLI `weeping-angel assurance explain --assessment <id> --control <id>` is parsed and dispatched; output cites exact evidence digests, population, missing evidence, failing/missing subjects, test id/version, exceptions, and framework mappings (LIN-003, LIN-012).
6. `AssessmentReport` serialization is pure: no pack load, network, filesystem, or hidden current-state resolution (LIN-004).
7. Explicit `AssessmentSummary` / `FrameworkReadinessSnapshot` / `CoverageMetrics` replace serialize-time percentage invention; seven metric families stay separate; no single compliance percentage (LIN-013).
8. One registry/loader path for every framework; ISO-only fallbacks and the production stub assessment are removed from production (LIN-010, LIN-011).
9. `compare` identifies applicability, subject population, evidence add/remove/supersession, test-result, exception, and framework/catalog digest changes (LIN-006, LIN-007).
10. Snapshot and result digests are deterministic SHA-256 of canonical JSON, not a reused compile digest for all three identities, and exclude wall-clock `duration` / `evaluatedAt` (LIN-008).
11. Ledger grows persist/load APIs for the new/empty lineage tables; `INSERT OR REPLACE` must not silently mutate a completed assessment’s semantic payload (LIN-014).
12. `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_canonical_assurance_catalog_target`, and `cargo test --workspace --features demo` stay GREEN after implement. No new frameworks. No domain catalog redesign. `assurance-ir/v1` is not forked.

---

## 6. Out of scope

- Multi-tenant SaaS backend, authn/z, hosted control plane
- UI / dashboards / HTML report engine
- New frameworks (SOC 2, NIS2, DORA, GDPR, ISO 27701 production packs)
- Domain catalog redesign or new IAM/SDLC/vuln/infra/governance content (`catalog/canonical/v1` TOML rewrite)
- Prompt 09 GitHub collector mapping (`crates/weeping-angel-collector/src/github/**`, `tests/sdd/github_collector.*`)
- Prompt 10 organization-context applicability evaluator (`OrgContext`, `ManualDeterminationRequired`, `evaluate_org_context`)
- Prompt 12 ISO remapping of pack `to =` onto `control.*` or ISO pack ID changes
- Forking `assurance-ir/v1` or redesigning `Control` / `Requirement` / `Mapping` / `EvidenceRequirement` / `PlannedControlTest` / `AssessmentDefinition`
- Teaching `compile_framework` to load `catalog/canonical/v1` as a pack substitute
- Certification claims or licensed ISO normative wording
- Redesign of collector discovery / scanner bridge
- Fixing pre-existing workspace rustfmt / clippy failures unrelated to this slice

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Spine ACT tests depend on the production stub assessment for non-ISO profiles | Target suite + ACT updates use explicit **test fixtures**; production path fail-closes on missing pack. Keep ACT-003 / collector-blindness. |
| ISO target `iso_007` / serialize tests require `frameworkPackDigest` and may needle `load_framework_pack` | Keep pack digest **on the snapshot**; move load to assess/load-pack time, not `Serialize`. Update ISO tests only if they assert serialize-time loading. |
| Prompt 10 absence (or concurrent landing) leaves applicability shallow or collides | Persist rule + static outcome + pack rows; unknown predicates stay unresolved, never false. Do not add Prompt 10 types. |
| `INSERT OR REPLACE` on `collection_runs` already overwrites | Assessment/control-test/framework snapshot APIs must be append-only or digest-keyed; document collection-run replace as “same run_id update while started” only. |
| Result digest including `evaluatedAt` / `duration` becomes non-deterministic | Exclude wall-clock fields from result identity (same as population contract). |
| Hashing live TOML bytes breaks Windows CRLF | Digest canonicalized structures (existing pack/catalog digest law), not raw files. |
| Generic serialize still mentioning ISO in tests | LIN-004 source-scan + functional test with pack path unavailable. |
| CLI explain accidentally stays under the banner-exit-0 `_` arm | Dispatch like catalog; baseline/target assert the arm. |
| Neighbor suites go red if stub is removed too early | Implement fixtures first; keep neighbor targets green as a hard gate. |
| Public JSON shape change surprises consumers | ADR 0003-assessment-lineage; contract update in implement; serde defaults on new fields. |
| Ledger in evidence crate growing conclusion types | Payloads are opaque JSON documents; evidence crate still does not compute effectiveness. |
| Collapsing metrics into one % under a new name | LIN-013 forbids a single compliance percentage field. |

---

## 8. ADR

**Required.** This slice changes public report JSON, facade assess semantics (no production stub), CLI family, and ledger storage/lineage seams.

Draft: [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md) (`0003-*` catalog-program numbering; cite by path). Finalize in the implement phase when signatures and table APIs are frozen.

---

## 9. Implement notes (not this spec-only phase)

This spec-only change must **not** edit production Rust or `docs/contracts/assurance-runtime.md`. Dual-suite registration in root `Cargo.toml` is **already done**. Baseline GREEN is already authored.

Implement later, in order:

1. ~~Author LIN-001–008 and LIN-010–014~~ done — target is **RED on CURRENT** (right reason: missing persist/explain/pure-serialize/generic facade). Do not implement product feature code until that RED proof is recorded.
2. Persist lineage; purify serialize; generic loader; explain CLI; compare; metrics — only lineage/explain/report-serialization/snapshot/ledger persist/generic-facade paths.
3. Prove target GREEN; skip-supersede baseline; keep neighbor + workspace green (`cargo test --workspace --features demo`).
4. Accept the ADR; update the public contract (facade, CLI, ledger APIs, report fields).

Do **not**: edit GitHub collector files; add Prompt 10 evaluator types; rewrite catalog domain TOML or ISO pack IDs.

---

## 10. Definition of done

An assessment is a reproducible immutable execution artifact rather than only a current-state report; every result is explainable down to pinned snapshots and evidence digests; report serialization is pure; framework resolution is generic; failed/partial collection is first-class; snapshot replay/diff is deterministic; dual-suite protocol is satisfied (baseline skip-superseded, target GREEN); neighbor SDD targets stay green.
