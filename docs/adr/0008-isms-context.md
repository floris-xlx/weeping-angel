# ADR 0008 — Canonical ISMS context IR (operational root)

<!-- weeping-angel-adr-meta
id = "0008"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_isms_context_target` GREEN (CTX-T01–T14); `sdd_isms_context_baseline` skip-superseded |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine. Does **not** supercede `AssessmentDefinition` as compile input, `Identity` as IAM principal, `AssetKind::Organization` / `SubjectKind::Organization` as inventory/selector kinds, IR-019 dangling `RiskId` checks, or the obligation-registry types in `party.rs` / `obligation.rs`. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (IR as framework-neutral documents + `canonical_digest`), [ADR 0004](0004-documentation-architecture.md) (spec/ADR/contract paths) |
| Spec | [`docs/specs/isms-context.md`](../specs/isms-context.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) (ISMS context section) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_isms_context_target` GREEN (`tests/contracts/isms_context.target.rs`). `sdd_isms_context_baseline` `#[ignore = "superseded by sdd_isms_context_target"]`. Neighbor IsmsContext-absence found-cases skip-superseded (`scp_b09`, IPO baseline comment). |

> Filename **`0008-*`**. Cite **this file by path**. Do **not** add a `0003-isms-context.md` sibling. Concurrent Operational ISMS drafts also use `0008-*` ([interested parties](0008-interested-parties-obligations.md), [scope engine](0008-scope-engine.md), [security objectives](0008-security-objectives.md)).

## Context

On SHA `6e31bf1a…` the assurance IR could describe a **point-in-time assessment** (`AssessmentDefinition`) and compile it. It could not describe the **management system** that runs continuously.

There was no `IsmsContext`, no ISMS `Organization` record, and no context-graph `InterestedParty` / `Obligation` / `SecurityObjective` / lifecycle types. `Identity` is an IAM principal. `AssessmentScope.organizations` is `Vec<String>`. Neighbor baselines asserted that `IsmsContext` was absent.

Operational ISMS v1 needs a **single root** so later scope, risk, governance, audit, and readiness work do not invent a parallel GRC graph.

Questions this decision answers:

1. Is the ISMS root a new schema (`isms-ir/v1`) or an extension of `assurance-ir/v1`?
2. How is durable definition kept distinct from point-in-time assessment input and results?
3. What is “organization” if `Identity` and `AssetKind::Organization` already exist?
4. How much of parties, obligations, objectives, scope, and methodology belong here vs later slices?
5. How are identifiers, digests, and lifecycle enums kept consistent with the spine?

## Decision

This is what shipped. Field-level law is [`docs/specs/isms-context.md`](../specs/isms-context.md).

### 1. One IR schema; no parallel GRC

`IsmsContext` is a document in **`weeping-angel-assurance-ir`** ([`crates/weeping-angel-assurance-ir/src/isms.rs`](../../crates/weeping-angel-assurance-ir/src/isms.rs), re-exported from that crate’s `lib.rs`) with `ASSURANCE_IR_SCHEMA` (`assurance-ir/v1`). No `isms-ir/v1` / `grc-ir/v1`. No extra crate.

ISO 27001 can use the model now; other frameworks later. Generic types carry **no** ISO clause numbers, Annex A semantics, SoA fields, or cloud-provider objects.

### 2. Definition ≠ assessment

```text
IsmsContext          durable management-system definition (root)
AssessmentDefinition point-in-time compile/assessment input
```

`AssessmentDefinition::new` remains valid. Additive `isms_context_id: Option<IsmsContextId>` uses `#[serde(default, skip_serializing_if = "Option::is_none")]` and **snake_case** on the assessment document so `tests/fixtures/assurance-ir/v1/assessment.json` still decodes. `IsmsContext` itself is serde `camelCase` (`schemaVersion`).

Do **not** embed `Effectiveness`, residual scores, SoA rows, snapshots, or control-test results in `IsmsContext`. CTX-T12 locks this.

### 3. Organization is a legal/management-system entity

Shipped `Organization` + `BusinessUnit` in `isms.rs`. They are **not** `Identity`, **not** `AssetKind::Organization`, **not** `SubjectKind::Organization`.

v1: exactly **one** organization per context. Required identity field: non-empty `legalName` (trim-aware). Population links to assets/vendors/identities are existing `AssetId` / `VendorId` / `IdentityId` `BTreeSet`s.

### 4. Graph records here; engines later

Canonical relationships:

```text
ISMS → Organization → ManagementSystemScope     (named reference, not ScopeResolution)
ISMS → InterestedParty → Obligation             (id graph, not mapping engine)
ISMS → SecurityObjective                        (declaration, not OnTrack projection)
ISMS → RiskMethodologyId                        (typed reference; scoring is ADR 0005)
ISMS → Asset/Vendor/Identity ids                (existing IR)
```

Plus: internal/external `ContextIssue`, `GovernanceCadence` (`count ≥ 1` + unit), `IsmsLifecycleStatus`.

`RiskMethodologyId` was already present from the methodology slice and is **reused**. This slice does not implement `score_risk`.

Crate-root `InterestedParty` / `Obligation` / `InterestedPartyKind` are the **membership-graph** records on `IsmsContext`. The obligation registry lives in `party.rs` / `obligation.rs` (distinct types, shared `InterestedPartyId` / `ObligationId`). This ADR does not make the registry the context graph.

### 5. Reuse identity and digest law

New ids go through existing `typed_id!` / `validate_stable_id` in `id.rs`: `IsmsContextId`, `OrganizationId`, `BusinessUnitId`, `ScopeId`, `IssueId`, `InterestedPartyId`, `ObligationId`, `ObjectiveId`. Digests use `canonical_digest` / `typed_canonical_digest` (`canon/v1`). No UUID v4. No second hasher.

Lifecycle and issue/party/cadence enums are **exhaustive**, serde `camelCase`, unknown tags fail closed, validated in `ValidateIr for IsmsContext` (central), not ad-hoc in callers.

### 6. Fail closed

`IsmsContext::validate()` rejects duplicate ids, dangling internal references (scope id mismatch, `parentId`, party↔obligation both directions, self-successor), empty required identity/title fields, and impossible lifecycle combinations (`superseded` without `supersededBy`, `supersededBy` when status ≠ superseded, `active`/`underReview` without methodology id and cadence, zero-count cadence).

Standalone context validation does not require assessment inventories. `validate_assessment_against_context` checks the optional pointer and population ids against that assessment. `AssessmentDefinition::validate()` does **not** require a context.

`explain_isms_context` is a pure, deterministic definition string (not an assessment-result projection).

### 7. Dual-suite and neighbors

Registered `sdd_isms_context_{baseline,target}` in root `Cargo.toml` (`tests/contracts/` is not auto-discovered).

Golden fixture: [`tests/fixtures/assurance-ir/v1/isms-context.json`](../../tests/fixtures/assurance-ir/v1/isms-context.json) — one org (`org:acme`), two business units, one internal + one external issue, interested party + obligation, declared objective, `risk-method:acme-v1`, active cadence.

When target went GREEN, neighbor found-cases that asserted `IsmsContext` **absence** were skip-superseded. Those neighbor suites are not this slice.

Traces go to `.sdd/runs/` / `.sdd/artifacts/` (ADR 0004). Do not write them under `docs/sdd/`.

## Consequences

- Later scope, risk, governance, audit, and readiness slices hang off **one** root instead of inventing another graph.
- ISO packs remain projections onto canonical controls; they do not own org context types.
- Collectors remain fact emitters; they do not create `IsmsContext`.
- `weeping-angel-framework` stays network-free and unaware of providers.
- Methodology, scope-resolution, obligation-mapping, and objective-measurement slices stay separately specified; this ADR plants typed references and the context membership graph.

## Non-goals

UI, persistence, policy editor, workflow engine, ISO mapping, risk scoring, auditor portal, CLI, `ScopeResolution`, obligation mapping engine, objective measurement, scheduler jobs.

## Related

- Spec: [`docs/specs/isms-context.md`](../specs/isms-context.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Spine: [ADR 0001](0001-inwardly-extensible-assurance-runtime.md)
- Docs layout: [ADR 0004](0004-documentation-architecture.md)
- Risk methodology (scoring, not this root): [ADR 0005-risk-methodology](0005-risk-methodology.md)
- Typed evidence (facts ≠ conclusions): [ADR 0003 typed evidence](0003-typed-evidence-canonical-serialization.md)
- Obligation registry (later engine, shared ids): [ADR 0008 interested parties](0008-interested-parties-obligations.md)
