# SDD: Controlled Document and Policy Registry

| Field | Value |
| --- | --- |
| Status | **Implemented** — target CD-001–014 GREEN; baseline absence cases skip-superseded |
| Program | Operational ISMS v1 — Prompt 12 |
| Source prompt | [`docs/prompts/operational-isms-v1/12-controlled-documents.md`](../prompts/operational-isms-v1/12-controlled-documents.md) |
| Slice | Immutable versioned `ControlledDocument` registry with governance metadata and evaluation-at-time-T helpers. No document editor. |
| Dual-suite | `sdd_controlled_documents_baseline` · `sdd_controlled_documents_target` (`tests/contracts/controlled_documents.{baseline,target}.rs`) — **not auto-discovered**; listed `[[test]]` in root [`Cargo.toml`](../../Cargo.toml). Target CD-001–014 GREEN; baseline CD-B001/B002 skip-superseded. |
| ADR | Accepted [`docs/adr/0017-controlled-documents.md`](../adr/0017-controlled-documents.md) (`0003-*` sibling; **0004** is documentation architecture). Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (pointer only; do not fork the spine) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Consumes (do not rewrite) | Governance catalog `control.governance.document-control` + `test.governance.document-control-attested` (hybrid / `manual-review`); typed evidence envelopes [`typed-evidence.md`](typed-evidence.md); IR `canonical_digest` / `content_digest` |
| Neighbors (keep GREEN) | `sdd_governance_catalog_target`, `sdd_typed_evidence_target`, `sdd_assessment_lineage_target` |
| Collision fence | Prompt 09 residual-risk, Prompt 10 control-implementation-registry, Prompt 11 operational SoA, GitHub collector, catalog domain TOML, ISO remap, Kleene evaluator |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Evidence schema | `evidence/v1` (`EVIDENCE_SCHEMA`) — observations, never conclusions |
| Canonical digest | `canon/v1` (`serde_json` struct field order + `BTreeMap` / `BTreeSet`) |
| Workspace verify (after implement) | `cargo test --test sdd_controlled_documents_baseline`; `cargo test --test sdd_controlled_documents_target`; `cargo test --test sdd_documentation_layout`; `cargo test --test sdd_governance_catalog_target`; `cargo test --test sdd_typed_evidence_target`; `cargo test --workspace --features demo` when practical |

This document is the durable human SSOT for Operational ISMS v1 Prompt 12. It owns **controlled-document identity**, **immutable versioned artifacts**, **governance metadata**, **evaluation at time T**, and **fail-closed link integrity** to obligations, controls, and risks. It does **not** own a document editor, Drive clone, e-signature product, ISO text ingestion, general DMS, the full `IsmsContext` / obligation engines (Prompts 01 / 03), residual-risk math, control-implementation registry rows, or operational SoA projection.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

A governed document is an **immutable versioned artifact with metadata**. It is **not** a control, **not** an effectiveness conclusion, and **not** a second identity/digest system.

```text
what the control means              = Control (canonical catalog)
how this org implements it          = ControlImplementation (Prompt 10; not this slice)
whether the control is effective    = ControlTestResult.effectiveness (tests + evidence)
which policy version was in force   = ControlledDocument evaluation at T (this slice)
```

Document existence alone **cannot** make an operational control `Effectiveness::Effective` when that control’s test requires execution evidence.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

---

## 0. Collision fence (concurrent SDD)

This slice may add IR document-control types and dual-suite tests. It must not rewrite in-flight neighbors.

| Do not touch | Owner |
| --- | --- |
| `docs/specs/residual-risk.md`, `tests/contracts/residual_risk.*`, `**/residual*.rs`, `docs/adr/*residual*` | Prompt 09 residual risk |
| `docs/specs/control-implementation-registry.md`, `tests/contracts/control_implementation_registry.*`, `docs/adr/*control-implementation*` | Prompt 10 implementation registry |
| `docs/specs/operational-soa.md`, `tests/contracts/operational_soa.*`, `crates/weeping-angel-assurance/src/soa.rs` (except tiny additive re-exports if unavoidable — **prefer none**), `docs/adr/*operational-soa*` | Prompt 11 operational SoA |
| `tests/contracts/github_collector.*`, `crates/weeping-angel-collector/src/github/**` | Canonical Assurance GitHub collector |
| `catalog/canonical/v1/**` domain TOML (including `governance.toml`), ISO pack IDs / `to =` remaps, `tests/contracts/iso27001_remap.*` | Catalog / ISO remap |
| Applicability Kleene evaluator (`weeping-angel-assurance::applicability`) | Canonical Assurance Prompt 10 |
| Unrelated catalog SDD suites (`iam` / `sdlc` / `vuln` / `infra` / `governance` product files) | Those prompts |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| `ControlledDocument`, version/status enums, evaluation helpers, registry, link universe | `crates/weeping-angel-assurance-ir/src/document.rs` (new) |
| `ControlledDocumentId`, `ObligationId` (minimum versioned ref — **not** the Prompt 03 engine) | `crates/weeping-angel-assurance-ir/src/id.rs` (`typed_id!`) |
| Re-exports | `crates/weeping-angel-assurance-ir/src/lib.rs` |
| Owner / approver principals | Reuse `PrincipalRef` from `implementation.rs` |
| Applicability scope | Reuse `SubjectSelector` from `subject.rs` |
| Artifact bytes / digests | Reference `weeping-angel-evidence::{EvidenceEnvelope, EvidenceLedger}` `content_digest` / `digest` / `canonical_digest`. Evidence crate stays **conclusion-free**. |
| Optional queries that need engine context | `weeping-angel-assurance` only if required; **prefer none** |

Tiny allowed adjustments at implement: new `document.rs`; additive `typed_id!`; `lib.rs` re-exports; serde camelCase. Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** add a second `Control` type. Do **not** put `Effectiveness` on a document. Do **not** edit `soa.rs` unless a one-line re-export is unavoidable (prefer none). Do **not** add document fields to `AssessmentDefinition` in this slice (collision with in-flight IR growth); the registry is a standalone IR document.

Prompts 01 (`IsmsContext`) and 03 (`Obligation` / `InterestedParty`) may not be fully landed. This slice defines **minimum versioned references** (`ObligationId`, `ControlId`, `RiskId`, `SubjectSelector`) and **fails closed** on dangling IDs. It does **not** implement those engines.

---

## 1. Problem / user-visible goal

Operators cannot prove which governed document version was in force at a point in time, or connect that version to obligations, controls, and evidence.

On characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`:

- `weeping-angel-assurance-ir` has **no** `ControlledDocument` type, no `document.rs`, and no document version registry.
- Governance catalog **does** already publish `control.governance.document-control` (hybrid) tested by `test.governance.document-control-attested` (`op = "manual-review"`, `required_evidence = ["evidence.manual.attestation"]`). That control is an **attestation that a document-control process exists**. It is not a document registry.
- Immutable artifacts already exist as `EvidenceEnvelope` + `EvidenceLedger` (facts, never conclusions) with `canonical_digest` / `content_digest`. There is no governed overlay that says “this digest is policy `doc.policy.information-security` version `1.0`, approved, effective from …”.
- `Effectiveness` is produced only by `weeping-angel-control-test::evaluate`. Nothing in IR treats “a policy file exists” as `Effectiveness::Effective`.

That means:

- a reviewer cannot name the **approved bytes** that were effective on a given date;
- a draft Word file and an approved policy are indistinguishable as governance records because neither record exists;
- supersession cannot keep the old version addressable;
- editing an approved PDF in place would be undetectable;
- linking a control to “the IS policy” cannot be fail-closed because there is no document id graph.

**User-visible goal:** given a stable document id and a time `T`, Weeping Angel can answer:

```text
which version was effective at T?
was it approved?
was it inside the review window, or stale/overdue?
is this identity only a draft (not operational)?
who approved it, and which evidence envelopes prove approval?
which artifact digest (immutable bytes) is that version?
which controls / obligations / risks does it bind?
did required acknowledgements cover the scoped subjects?
what is the retention metadata?
```

and can prove the negatives:

```text
draft-only → not operational
missing approval → cannot become approved/effective
superseded version → still get(id, version); current() moved
digest change after approval → new version, never in-place edit
document present → does not imply Effectiveness::Effective
  when the control test requires execution evidence
dangling control / obligation / risk / scope id → fail closed
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | **Do not fork.** Documents carry this schema version string. |
| `canonical_digest` / `typed_canonical_digest` | `digest.rs` | Reuse. Do not invent a second digest algorithm. |
| `EvidenceEnvelope.content_digest` / `digest` / `evidence_id` | `weeping-angel-evidence` | Artifact identity. Store the digest string on the document version; do not copy observation bytes into IR. |
| `EvidenceLedger` | evidence `ledger.rs` | Append-only observations. Do not store effectiveness/conclusions on envelopes or ledger rows. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for owner and approvers. Do not invent `DocumentOwner`. |
| `SubjectSelector` | `subject.rs` | Applicability scope. Do not reimplement Kleene / Prompt 02 `ScopeResolution`. |
| `ControlId` / `RiskId` | `id.rs` | Link targets. Fail closed when the provided universe does not contain them. |
| `ObligationId` | **new** `typed_id!` | Minimum versioned reference for Prompt 03. Not `struct Obligation`. |
| `Effectiveness` | `weeping-angel-control-test` | **Different type.** Never a field on `ControlledDocument`. |
| `Control` | `control.rs` | Canonical meaning. Do not encode document-control as a second Control type. |
| `control.governance.document-control` | `catalog/canonical/v1/controls/governance.toml` | **Consume.** Hybrid, `evidence.manual.attestation`. Do not edit TOML. |
| `test.governance.document-control-attested` | `catalog/canonical/v1/tests/governance.toml` | **Consume.** `kind = "hybrid"`, `op = "manual-review"`. |
| `AssessmentDefinition` | `assessment.rs` | Do **not** add a `documents` vec in this slice (collision fence). Registry is standalone. |
| Prompts 01 / 03 | prompt docs only at characterization | Minimum ids only. No `IsmsContext` / `InterestedParty` / obligation mapping engine. |

JSON names are **camelCase**, matching IR. Additive fields (when this type is new, all fields are authored together) still use `serde(default)` / `skip_serializing_if` where empty collections/options should omit.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. Encoded by `tests/contracts/controlled_documents.baseline.rs`. After implement, CD-B001/B002 (IR absence) are `#[ignore = "superseded by sdd_controlled_documents_target"]`. Remaining baseline tests still lock catalog consume, envelope identity, and “policy observation ≠ execution `Effective`”.

### 3.1 No controlled-document IR

[`crates/weeping-angel-assurance-ir/src/lib.rs`](../../crates/weeping-angel-assurance-ir/src/lib.rs) modules: applicability, assessment, asset, control, crosswalk, digest, evidence, exception, extension, framework, id, identity, implementation, mapping, privacy, requirement, risk, subject, test, validation, vendor.

There is **no** `mod document`, no `struct ControlledDocument`, no `DocumentVersion`, no `DocumentControlRegistry`, no `DocumentType`.

[`id.rs`](../../crates/weeping-angel-assurance-ir/src/id.rs) `typed_id!` aliases stop at `MappingId`. There is no `ControlledDocumentId`. (`ObligationId` is also absent; Prompt 03 may add it later — baseline characterizes **document** absence, not a freeze on Prompt 03 ids.)

### 3.2 No evaluation-at-T for governed versions

There is no helper that, given a document id and `DateTime<Utc>`, returns the approved/effective version, review-overdue flag, acknowledgement coverage, or supersession pointer. `Risk::` review helpers (Prompt 06, specified in parallel) are a different type and are not a document registry.

### 3.3 Catalog document-control is attestation, not a registry

[`catalog/canonical/v1/controls/governance.toml`](../../catalog/canonical/v1/controls/governance.toml):

```text
id = "control.governance.document-control"
title = "Document-control governance"
automation = "hybrid"
evidence = ["evidence.manual.attestation"]
tests = ["test.governance.document-control-attested"]
```

[`catalog/canonical/v1/tests/governance.toml`](../../catalog/canonical/v1/tests/governance.toml):

```text
id = "test.governance.document-control-attested"
kind = "hybrid"
required_evidence = ["evidence.manual.attestation"]
op = "manual-review"
```

`TestExpr::ManualReview` evaluates to `Effectiveness::ManualReviewRequired` ([`crates/weeping-angel-control-test/src/lib.rs`](../../crates/weeping-angel-control-test/src/lib.rs)). A policy PDF path is not that test.

This slice **must not** rewrite those TOML files. It **must not** make `ManualReview` auto-pass because a `ControlledDocument` exists.

### 3.4 Evidence envelopes are already immutable observations

`EvidenceEnvelope::seal` hashes observation+provenance via IR `canonical_digest`, stores `content_digest` and `digest`, and rejects compliance-claim narratives. `EvidenceLedger` is append-only. This slice reuses those digests; it does not wrap a second hash.

### 3.5 Effectiveness is test-only

`weeping-angel-assurance-ir` does not define `Effectiveness`. Control tests that `require` an execution evidence type (for example `source.branch.protection` + `TestExpr::Exists`) return `InsufficientEvidence` when that type is missing, even if an unrelated `evidence.governance.policy` envelope is in the `EvidenceSet`. Baseline locks this found case so implement cannot later promote “policy envelope present” to `Effective`.

### 3.6 IR validation has no document graph

[`validation.rs`](../../crates/weeping-angel-assurance-ir/src/validation.rs) checks requirements, controls, evidence requirements, mappings, tests, and implementation → control/risk/exception ids. It does not walk document → control/obligation/risk/scope links because those fields do not exist.

### 3.7 Schema

`ASSURANCE_IR_SCHEMA == "assurance-ir/v1"`. No ISO clause numbers, Annex A identifiers, or licensed ISO text appear on generic IR types.

---

## 4. Desired behavior (target)

### 4.1 Product home

```text
weeping-angel-assurance-ir
  document.rs     # ControlledDocument, DocumentVersion, registry, eval helpers
  id.rs           # ControlledDocumentId; ObligationId (min ref)
  lib.rs          # mod document + re-exports
  implementation.rs  # PrincipalRef consumed
  subject.rs      # SubjectSelector consumed
  digest.rs       # unchanged algorithm
```

Network-free. No ISO annex numbers, no provider SDK types, no GRC product vocabulary (`Confluence`, `SharePoint`, `Vanta`, `Drata`) in generic IR.

### 4.2 Types

```text
DocumentType =
  Policy | Standard | Procedure | Plan | Runbook | Guideline | Record | Other(String)

DocumentVersionStatus =
  Draft | Approved | Retired

InformationClassification =
  Public | Internal | Confidential | Restricted | Other(String)

ControlledDocumentId, ObligationId   // typed_id! + StableId
```

JSON: camelCase (`"policy"`, `"approved"`, `"confidential"`). `Other(String)` is the extensibility hatch for document types and classification (for example a later `"playbook"`) without forking the enum in every caller.

`DocumentVersionStatus` is **lifecycle of that version’s metadata**, not `Effectiveness`. Do not add `Effective` as a stored status; operational currency is **derived** at time `T` (§4.6).

### 4.3 `ControlledDocument` and `DocumentVersion`

One stable identity, many immutable versions:

| Field (Rust) | JSON | Semantics |
| --- | --- | --- |
| `schema_version` | `schemaVersion` | `assurance-ir/v1` |
| `id` | `id` | Stable `ControlledDocumentId` (survives supersession of versions) |
| `document_type` | `documentType` | §4.2 |
| `title` | `title` | Human title |
| `owner` | `owner` | `PrincipalRef` |
| `versions` | `versions` | Append-only list of `DocumentVersion` |
| `current_version` | `currentVersion` | Version string of the current pointer, or `None` if none approved |

`DocumentVersion`:

| Field (Rust) | JSON | Semantics |
| --- | --- | --- |
| `version` | `version` | Stable version label on this identity (`"1.0"`, `"2026-01"`). Unique per document. |
| `artifact_digest` | `artifactDigest` | `EvidenceEnvelope.content_digest` (or equal `digest` at seal time). **Identity of bytes.** |
| `artifact_ref` | `artifactRef` | Optional envelope `evidence_id` / artifact id string. Not a filesystem editor path as authority. |
| `status` | `status` | `Draft` \| `Approved` \| `Retired` |
| `effective_from` | `effectiveFrom` | Required for operational currency; `None` on drafts |
| `review_by` | `reviewBy` | Review-window end; `None` means not scheduled (not overdue, but **not** “within review window” for operational-current) |
| `approvers` | `approvers` | `Vec<PrincipalRef>`; required non-empty to approve |
| `approval_evidence_digests` | `approvalEvidenceDigests` | Envelope digests proving approval. Required non-empty to approve. **Evidence, not a signature product.** |
| `supersedes_version` | `supersedesVersion` | Prior version string of **this** id this version replaces |
| `applicability` | `applicability` | `Vec<SubjectSelector>` |
| `control_ids` | `controlIds` | Linked canonical controls |
| `obligation_ids` | `obligationIds` | Linked `ObligationId` (min refs) |
| `risk_ids` | `riskIds` | Linked `RiskId` |
| `acknowledgement_required` | `acknowledgementRequired` | If true, coverage is evaluated |
| `required_acknowledgement_subjects` | `requiredAcknowledgementSubjects` | Stable subject/identity ids that must acknowledge |
| `acknowledgements` | `acknowledgements` | Recorded acknowledgements |
| `classification` | `classification` | Confidentiality / classification |
| `retention` | `retention` | Retention metadata (must be present and queryable even if periods are `None`) |

`AcknowledgementRecord`: `{ subjectId, acknowledgedAt, evidenceDigest? }` — recorded against **this version**. Acknowledging version `1.0` does not cover `1.1`.

`RetentionMetadata`: `{ retainUntil?, retentionPeriodSeconds?, legalHold, disposition? }`. Presence of the object is required on every version; empty periods are valid (unknown schedule ≠ missing struct).

Constructor `ControlledDocument::new(id, document_type, title, owner)` starts with **no** versions and `current_version = None`. Adding a version starts as `Draft`.

### 4.4 Draft vs approved vs artifact immutability

1. Draft metadata may be edited **until approval** (title on the parent identity is identity metadata; **artifact digest on a draft version** may be replaced because the version is not yet approved).
2. `approve(...)` requires non-empty `approvers` **and** non-empty `approval_evidence_digests`, and sets `status = Approved` with `effective_from` (and typically `review_by`). Missing either is `DocumentControlError::MissingApproval` (name flexible; must be matchable).
3. An **approved** (or retired) version’s `artifact_digest` is immutable. Any API that would overwrite it returns an error. Callers must `append_version` with a **new** version string and the new digest.
4. Evidence behind an approved document is never mutated: the ledger envelope for that digest stays; a content edit seals a **new** envelope (new `content_digest`) which becomes a **new** document version.
5. `Retired` versions remain addressable. They are not current and not operational at `T` unless `effective_from..=retired` still covers `T` **and** they were not superseded as current — default: retired is not operational-current. Target tests treat `Retired` as not current.

Do not mutate `EvidenceEnvelope` fields to “fix” a policy. Do not store raw policy bytes on the IR struct.

### 4.5 Registry

```text
DocumentLinkUniverse {
  control_ids: BTreeSet<ControlId>,
  obligation_ids: BTreeSet<ObligationId>,
  risk_ids: BTreeSet<RiskId>,
  subject_ids: BTreeSet<String>,  // ids appearing in SubjectSelector.ids / acknowledgement subjects
}

DocumentControlRegistry { documents: Vec<ControlledDocument> }
```

Queries (names flexible if tests can call them; spec ids below pin behavior):

| Helper | Behavior |
| --- | --- |
| `registry.get(id)` | Document by stable id |
| `registry.version(id, version)` | **Always** addressable if recorded, including superseded / retired / draft |
| `registry.current(id)` | Version named by `current_version` |
| `registry.effective_version_at(id, t)` | Version that was operational-current at `T`, if any |
| `document.current()` / `document.version(v)` / `document.effective_at(t)` | Same, per identity |
| `registry.validate(&universe)` | Fail closed on integrity (§4.8) |

`current_version` moves when a newly approved version is published as current. The previous version remains in `versions` and `registry.version(id, old)`.

Supersession consistency: if version `B.supersedes_version == A`, then `A` exists on the same document; cycles fail validate; two current pointers fail; `current_version` must name an existing version.

### 4.6 Evaluation at time `T` (deterministic)

```text
DocumentVersion::is_approved() -> bool
DocumentVersion::is_effective_at(t) -> bool
DocumentVersion::within_review_window(t) -> bool
DocumentVersion::is_operational_current_at(t) -> bool
DocumentVersion::acknowledgement_coverage() -> AcknowledgementCoverage
ControlledDocument::is_operational_current_at(t) -> bool
ControlledDocument::effective_version_at(t) / DocumentControlRegistry::effective_version_at(id, t)
```

| Predicate | True iff |
| --- | --- |
| `DocumentVersion::is_approved` | `status == Approved` |
| `DocumentVersion::is_effective_at(t)` | approved **and** `effective_from` is `Some(start)` with `start <= t` |
| `DocumentVersion::within_review_window(t)` | `review_by` is `Some(end)` with `t <= end` |
| `DocumentVersion::is_operational_current_at(t)` | `is_effective_at(t)` **and** `within_review_window(t)` (version-level; does **not** require the current pointer) |
| `ControlledDocument::is_operational_current_at(t)` | `current()` exists **and** that version is operational at `t` |
| `effective_version_at(id, t)` | among versions with `DocumentVersion::is_operational_current_at(t)`, drop those named by another candidate’s `supersedes_version`; if several remain, prefer `current_version` if it is among them, else latest `effective_from` |

Shipped: `approve` always moves `current_version` to the approved version. `artifact_digest` is private (`artifact_digest()` / draft-only `set_artifact_digest`). No `weeping-angel-assurance` document engine module.

**Current policy (CD-001):** approved + `is_effective_at(T)` + within review window + current pointer. `effective_version_at(id, T)` returns that version; document-level `is_operational_current_at(T)` is true.

**Stale / review overdue (CD-002):** approved and `effective_from <= T`, but `review_by < T`. `is_operational_current_at(T)` is **false**. The version remains queryable. Stale is **not** `Effectiveness::StaleEvidence`; do not reuse that enum on documents.

**Draft-only (CD-003):** only `Draft` versions exist (or current pointer is `None`). `effective_version_at` returns `None`. Drafts are never operational.

**Not yet effective:** `effective_from > T` → not effective at `T`, even if approved.

`review_by == None`: not `within_review_window`; therefore not operational-current. Unscheduled ≠ overdue-as-review-miss (overdue requires a past `review_by`). Target stale case uses an explicit past `review_by`.

### 4.7 Acknowledgements and retention

```text
AcknowledgementCoverage { required: usize, recorded: usize, complete: bool }
```

If `acknowledgement_required` is false, coverage is complete (`required = 0`). If true, `complete` iff every `required_acknowledgement_subjects` id has a matching `acknowledgements[].subject_id` for **this version**. Gaps (CD-007) → `complete == false`. Missing required list while `acknowledgement_required` is true fails validate.

Retention (CD-008): `version.retention()` (or public field) returns the struct; registry/document query must expose it. Tests assert `legal_hold` / `retain_until` round-trip.

### 4.8 Fail closed links (CD-010)

`DocumentControlRegistry::validate(&DocumentLinkUniverse)` (or `ControlledDocument::validate`) errors on:

| Reference | Rule |
| --- | --- |
| Duplicate `ControlledDocument.id` | error |
| Duplicate `version` string on one document | error |
| `control_ids` | every id ∈ `universe.control_ids` |
| `obligation_ids` | every id ∈ `universe.obligation_ids` |
| `risk_ids` | every id ∈ `universe.risk_ids` |
| `applicability` selector `ids` | every non-empty id ∈ `universe.subject_ids` |
| `required_acknowledgement_subjects` | every id ∈ `universe.subject_ids` |
| `current_version` | names an existing version on that document |
| `supersedes_version` | names an existing earlier version; no cycles |
| Approved version | non-empty approvers **and** non-empty approval evidence digests |
| `artifact_digest` | non-empty on every version |

There is **no** obligation/control/risk inventory inside this slice. The caller supplies the universe (assessment inventories, Prompt 03 set, or a test fixture). Empty universe + any linked id ⇒ dangling ⇒ error. Empty link lists are valid.

Do not implement Prompt 03 mapping semantics (`ObligationMapping`, equivalence vs supports). A document may list obligation ids; that is a **ref**, not satisfaction.

### 4.9 Document presence ≠ control effectiveness (CD-009)

Invariant (must hold after implement):

```text
A ControlTest whose expression requires execution evidence
(e.g. TestExpr::Exists(source.branch.protection))
MUST NOT become Effectiveness::Effective
because a ControlledDocument is linked to that control
or because an evidence.governance.policy envelope is in the EvidenceSet.
```

`test.governance.document-control-attested` remains `ManualReview` → `ManualReviewRequired` even when the registry holds a current approved policy.

This slice may **link** `control_ids`; it may not write `Effectiveness` onto documents, envelopes, or catalog tests.

### 4.10 Catalog consume (not rewrite)

After implement, `catalog/canonical/v1/controls/governance.toml` still contains `control.governance.document-control` with `automation = "hybrid"` and `tests = ["test.governance.document-control-attested"]`. The test file still has `op = "manual-review"`. Target tests may **read** those files; they must not require TOML edits.

### 4.11 Evidence crate stays conclusion-free

No new envelope field `effective`, `approvedPolicy`, or similar. Approval evidence is a **digest reference** on the IR version. `looks_like_compliance_claim` continues to reject “ISO 27001 compliant” narratives.

### 4.12 Target suite contents (GREEN on this HEAD)

Stable titles. Author **before** product feature code. Prefer compile-safe source needles plus existing `evaluate` so RED is a named assertion (missing `struct ControlledDocument` / helpers), not an unrelated type error. When types land, keep the same ids and assert behavior through the public helpers.

| ID | Assertion |
| --- | --- |
| CD-001 | Current policy: approved + effective at `T` + in review window + current pointer. `effective_version_at` returns that version; `is_operational_current_at(T)` is true. |
| CD-002 | Stale policy: approved and dated-effective, `review_by < T`. Not operational-current; version still queryable. |
| CD-003 | Draft-only: `effective_version_at` is `None`; draft is not treated as effective. |
| CD-004 | Missing approval: `approve` / validate rejects empty approvers or empty approval evidence; cannot become approved/operational. |
| CD-005 | Supersession: old version remains `version(id, old)`; `current()` / current pointer moves to the successor. |
| CD-006 | Changed digest: replacing `artifact_digest` on an **approved** version errors; `append_version` with new digest creates a new version; approved bytes stay the old digest. |
| CD-007 | Required acknowledgement gaps: `acknowledgement_required` + missing subject → coverage incomplete. |
| CD-008 | Retention metadata present and queryable (round-trip `retainUntil` / `legalHold`). |
| CD-009 | Document presence does **not** imply `Effectiveness::Effective` when the control test requires execution evidence. Manual-review document-control test stays `ManualReviewRequired`. |
| CD-010 | Dangling control / obligation / risk / scope (`SubjectSelector` ids) fail closed. |
| CD-011 | Dual-suite names registered in root `Cargo.toml`. |
| CD-012 | Catalog `control.governance.document-control` / `test.governance.document-control-attested` still present as hybrid/`manual-review` (consumed, not rewritten). |
| CD-013 | `schemaVersion` is `assurance-ir/v1`; no ISO clause / Annex A strings on the document type. |
| CD-014 | Evidence crate source has no document-effectiveness conclusion type; envelopes remain claim-rejected observations. |

One regression test per later review comment must be titled `P?: <exact subject>` and encode the original found case (test first → RED → fix → GREEN).

---

## 5. Dual-suite protocol

Follow [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

| Suite | File | Cargo `[[test]]` name | On this HEAD |
| --- | --- | --- | --- |
| Baseline | `tests/contracts/controlled_documents.baseline.rs` | `sdd_controlled_documents_baseline` | Absence cases skip-superseded; catalog/envelope/execution-evidence locks remain GREEN |
| Target | `tests/contracts/controlled_documents.target.rs` | `sdd_controlled_documents_target` | **GREEN** (CD-001–014) |

Protocol completed (mandatory; do not re-open skipped gates):

1. Spec (this file) + ADR.
2. Baseline GREEN on characterization; target RED for missing registry / helpers / eval-at-T.
3. Product code in `weeping-angel-assurance-ir` (`document.rs` + ids + re-exports).
4. ADR Accepted. Target GREEN. Neighbors `sdd_governance_catalog_target`, `sdd_typed_evidence_target`, `sdd_assessment_lineage_target` GREEN.
5. Baseline CD-B001/B002 `#[ignore = "superseded by sdd_controlled_documents_target"]`. Target still GREEN.

Traces only under `.sdd/runs/` and `.sdd/artifacts/`.

---

## 6. Acceptance criteria (testable)

- **CD-001** An approved, dated-effective, in-window current policy is selected at `T`.
- **CD-002** Review-overdue approved policy is stale (not operational-current) and still addressable.
- **CD-003** Draft-only identity is not treated as effective/operational.
- **CD-004** Missing approvers or approval evidence cannot approve.
- **CD-005** Superseded version remains `get`/`version`-addressable; current pointer moves.
- **CD-006** Digest change after approval is a new version; approved artifact digest is immutable.
- **CD-007** Required acknowledgement gaps are reported (`complete == false`).
- **CD-008** Retention metadata is present and queryable.
- **CD-009** Linked/present document does not yield `Effectiveness::Effective` for execution-evidence tests; catalog document-control attestation stays manual-review.
- **CD-010** Dangling control, obligation, risk, or scope ids fail closed.
- Dual-suite registered; spec listed in `CANONICAL_SPECS`; schema remains `assurance-ir/v1`.
- Neighbor targets listed in the header stay GREEN after implement.

---

## 7. Out of scope

- Rich text editor, Google Drive / SharePoint clone, or general document management system.
- E-signature product (DocuSign/Adobe Sign). Approval **evidence digests** are enough.
- ISO text ingestion, clause numbers, Annex A, or licensed normative wording on generic IR.
- Implementing full `IsmsContext` (Prompt 01) or `Obligation` / `InterestedParty` engines (Prompt 03).
- Residual-risk engine, control-implementation registry expansion, operational SoA projection.
- Rewriting `catalog/canonical/v1/**`, ISO pack remaps, GitHub collector, Kleene evaluator.
- Putting `Effectiveness` or compliance conclusions on `EvidenceEnvelope`.
- Encoding document-control as a second `Control` type.
- UI, persistence service beyond reusing `EvidenceLedger` observations, workflow inbox, DMS ACL model.
- Forking `assurance-ir/v1`.

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Prompt 03 lands `ObligationId` with different semantics | This slice’s `ObligationId` is a `typed_id!` stable id only. Share the alias; do not add `struct Obligation` here. |
| Prompt 10 document **refs** on `ControlImplementation` collide | This slice owns the document **record**. Implementation registry may store ids; do not fork a second document struct. Do not edit Prompt 10 files. |
| Treating catalog `document-control` as auto-effective | CD-009 + consume-only TOML. `ManualReview` stays. |
| Second digest scheme for PDFs | Always `canonical_digest` / envelope `content_digest`. |
| Mutating approved bytes “for a typo” | CD-006: new version only. |
| `AssessmentDefinition.documents` conflicts with in-flight IR edits | Standalone `DocumentControlRegistry`; no assessment field in this slice. |
| Stale document confused with `Effectiveness::StaleEvidence` | Different types; document stale is review-window metadata. |
| Empty `review_by` treated as in-window | Spec: unscheduled is not within review window and not overdue. |
| Scope engine (Prompt 02) not landed | `SubjectSelector` ids checked against a caller-supplied universe, not Kleene. |
| Neighbor suites break | Do not edit their files; verify listed targets stay GREEN. |

---

## 9. ADR

**Accepted:** [`docs/adr/0017-controlled-documents.md`](../adr/0017-controlled-documents.md). Filename `0003-*` is shared with catalog-program siblings; **0004** remains documentation architecture. Cite by path.

---

## 10. Landed files

Product:

- `crates/weeping-angel-assurance-ir/src/document.rs`
- `crates/weeping-angel-assurance-ir/src/id.rs` (`typed_id!(ControlledDocumentId)`; shared `ObligationId`)
- `crates/weeping-angel-assurance-ir/src/lib.rs` (`mod document` + re-exports)

Tests/docs:

- `tests/contracts/controlled_documents.baseline.rs`
- `tests/contracts/controlled_documents.target.rs`
- root `Cargo.toml` `[[test]]` rows
- `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS`
- [`docs/adr/0017-controlled-documents.md`](../adr/0017-controlled-documents.md)
- public-contract pointer in [`docs/specs/assurance-runtime.md`](assurance-runtime.md)

Do **not** edit catalog TOML, `soa.rs`, residual-risk, control-implementation-registry, GitHub collector, or Kleene evaluator.

---

## 11. Definition of done

Weeping Angel can prove which governed document version was effective at any time and connect it to obligations, controls, and evidence — without a document editor — while document presence never substitutes for execution evidence on operational controls.

Dual-suite SDD protocol is complete on this HEAD: spec first, baseline characterized, target RED then GREEN (CD-001–014), ADR Accepted, baseline absence cases skip-superseded, target still GREEN. Neighbor `sdd_governance_catalog_target`, `sdd_typed_evidence_target`, and `sdd_assessment_lineage_target` stay GREEN.
