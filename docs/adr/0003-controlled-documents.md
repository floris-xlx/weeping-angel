# ADR 0003 — Controlled document and policy registry (immutable versions ≠ effectiveness)

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_controlled_documents_target` GREEN (CD-001–014); baseline absence cases CD-B001/B002 skip-superseded. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Adds a public IR document-control contract. Does **not** supercede envelope immutability, `canon/v1` digest, catalog ownership of `control.governance.document-control`, IR-009 (`ImplementationStatus` ≠ `Effectiveness`), or CIR opaque `DocumentRef`. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [governance catalog](0003-governance-canonical-assurance-catalog.md), [ADR 0004](0004-documentation-architecture.md) |
| Spec | [`docs/specs/controlled-documents.md`](../specs/controlled-documents.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Prompt | [`docs/prompts/operational-isms-v1/12-controlled-documents.md`](../prompts/operational-isms-v1/12-controlled-documents.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | Dual-suite `sdd_controlled_documents_baseline` / `sdd_controlled_documents_target` at `tests/contracts/controlled_documents.{baseline,target}.rs` (registered in root `Cargo.toml`). Neighbors `sdd_governance_catalog_target`, `sdd_typed_evidence_target`, `sdd_assessment_lineage_target` stay GREEN. |

> Filename `0003-*` is shared with catalog-program siblings. **0004** is documentation architecture. Cite this decision by **path**.

## Context

On SHA `6e31bf1a…`, Weeping Angel could store immutable evidence envelopes and attest (hybrid / manual-review) that a document-control *process* exists (`control.governance.document-control`). It could not record policies, standards, procedures, plans, runbooks, guidelines, or records as **governed, versioned artifacts**.

Missing capabilities at characterization:

1. No `ControlledDocument` type in `weeping-angel-assurance-ir`.
2. No way to name which artifact digest was **approved and effective at time T**.
3. No draft vs approved split; no supersession that keeps old versions addressable.
4. No fail-closed links from a document to `ControlId` / `ObligationId` / `RiskId` / `SubjectSelector`.
5. Risk that a stored policy file is later treated as `Effectiveness::Effective` for operational controls that require execution evidence.

Operational ISMS v1 Prompt 12 requires document-control **governance** without building an editor, Drive clone, e-sign product, ISO ingest pipeline, or general DMS.

Questions this decision answers:

1. Where does the document record live (IR vs evidence vs a new crate)?
2. How are bytes identified (new hash vs existing envelope digest)?
3. How do draft, approve, supersede, and digest-change interact?
4. How do Prompts 01/03 get referenced without those engines owning this slice?
5. May a current policy make a control `Effective`?

## Decision

This is what shipped. Field-level law is [`docs/specs/controlled-documents.md`](../specs/controlled-documents.md). Types live in `crates/weeping-angel-assurance-ir/src/document.rs` and are re-exported from `lib.rs`.

### 1. New IR type, same schema, no new crate

`ControlledDocument` is a standalone IR document. Schema remains `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`). `ControlledDocumentId` is a `typed_id!` alias (`StableId`). `ObligationId` is the **same** alias used by interested-parties/obligations — not a second newtype.

Do **not** add a crate. Do **not** encode document-control as a second `Control`. The registry is `DocumentControlRegistry`. This slice does **not** add `AssessmentDefinition.documents` (collision with in-flight IR growth). CIR keeps opaque `DocumentRef`; this registry does not replace that pointer type.

No `weeping-angel-assurance` engine module: queries are on the IR registry.

### 2. Artifact identity is the existing envelope digest

Approved bytes are `EvidenceEnvelope.content_digest` (IR `canonical_digest` of observation+provenance). `DocumentVersion.artifact_digest` stores that hex string; `artifact_ref` may hold an envelope `evidence_id`. Do not invent a second digest system. Do not copy observation bytes onto the IR struct. The evidence crate remains conclusion-free: no effectiveness field on envelopes.

`artifact_digest` is private. Callers read `artifact_digest()` and mutate only via `set_artifact_digest` (draft) or `append_version` (new version).

### 3. Versions are immutable after approval; edits are new versions

`ControlledDocument::new` starts with no versions and `current_version = None`. `append_version` always records `Draft` and clears `effective_from`. Draft artifact digest may be replaced.

`approve(version, approvers, approval_evidence_digests, effective_from, review_by)` requires a draft, non-empty `approvers`, and non-empty `approval_evidence_digests`. Missing either is `DocumentControlError::MissingApproval`. Success sets `Approved`, dates, and **moves `current_version` to that version**. Re-approving a non-draft is `NotDraft`.

Overwriting `artifact_digest` on an approved or retired version is `ImmutableApprovedArtifact`. A content change seals a new envelope and `append_version` with a new version string. `supersedes_version` names the prior version of **this** id. Superseded versions stay `registry.version(id, old)`.

`DocumentVersionStatus` is `{Draft, Approved, Retired}` — lifecycle of that version’s metadata, **not** `Effectiveness`. There is no stored `Effective` status.

### 4. Operational currency is derived at time T

Shipped split:

| Helper | True iff |
| --- | --- |
| `DocumentVersion::is_approved` | `status == Approved` |
| `DocumentVersion::is_effective_at(t)` | approved and `effective_from` is `Some(start)` with `start <= t` |
| `DocumentVersion::within_review_window(t)` | `review_by` is `Some(end)` with `t <= end` |
| `DocumentVersion::is_operational_current_at(t)` | `is_effective_at(t)` **and** `within_review_window(t)` |
| `ControlledDocument::is_operational_current_at(t)` | `current()` exists and that version is operational at `t` |
| `effective_version_at(id, t)` | among versions operational at `t`, drop those named by another candidate’s `supersedes_version`; if several remain, prefer `current_version` if it is among them, else latest `effective_from` |

Unscheduled (`review_by == None`) is not in-window and therefore not operational. Stale/review-overdue is document metadata, not `Effectiveness::StaleEvidence`. Do not store `Effectiveness` on the document.

### 5. Minimum refs; fail closed; no Prompt 01/03 engines here

Documents may list `ControlId`, `ObligationId`, `RiskId`, and `SubjectSelector` applicability. `DocumentControlRegistry::validate(&DocumentLinkUniverse)` (and `ControlledDocument::validate`) fail closed on dangling ids, duplicate document/version ids, empty `artifact_digest`, approved versions missing approvers/evidence, acknowledgement-required with an empty subject list, unknown `current_version` / `supersedes_version`, and supersession cycles.

The caller supplies the universe. Empty universe + any linked id ⇒ dangling. This ADR does not implement `IsmsContext` or the obligation mapping engine. A listed obligation id is a **ref**, not satisfaction.

### 6. Documents do not satisfy execution-required controls

Linking a current policy to a control does not yield `Effectiveness::Effective` when the control test requires execution evidence. Catalog `test.governance.document-control-attested` stays `manual-review` → `ManualReviewRequired`. Consume `control.governance.document-control`; do not rewrite `catalog/canonical/v1/**`.

### 7. Acknowledgements, classification, retention

`AcknowledgementRecord` is `{ subjectId, acknowledgedAt, evidenceDigest? }` against **this version**. If `acknowledgementRequired` is false, coverage is complete (`required = 0`). If true, `complete` iff every required subject has a matching acknowledgement on that version.

`InformationClassification` defaults to `Internal` (`public` / `internal` / `confidential` / `restricted` / `Other(String)`). `RetentionMetadata` is always present (`retainUntil`, `retentionPeriodSeconds`, `legalHold`, `disposition`); empty periods are valid.

### 8. Owner/approver/scope reuse

`PrincipalRef` and `SubjectSelector` are reused. Kleene applicability is not reimplemented.

## Non-goals

Rich text editor; Drive clone; e-signature product; ISO text ingestion; general DMS; residual-risk / implementation-registry / operational SoA product files; GitHub collector; catalog TOML rewrite; forking `assurance-ir/v1`; putting `Effectiveness` on envelopes; encoding document-control as a second `Control`.

## Consequences

- Reviewers can name the governed digest effective at T and its links (`controlIds` / `obligationIds` / `riskIds` / applicability).
- Prompt 03 hangs real `Obligation` records off the same `ObligationId`; documents only store ids.
- Prompt 10 may cite document ids via existing `DocumentRef` without owning `ControlledDocument`.
- Implement kept governance / typed-evidence / assessment-lineage target suites green.
- Collision fence: do not edit residual-risk, control-implementation-registry, operational SoA (`soa.rs` prefer untouched), collector GitHub mapping, ISO remap, or Kleene evaluator.

## Related

- Spec: [`docs/specs/controlled-documents.md`](../specs/controlled-documents.md)
- Tests: `tests/contracts/controlled_documents.{baseline,target}.rs`
- Layout: [ADR 0004](0004-documentation-architecture.md)
