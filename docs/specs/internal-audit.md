# SDD: Internal Audit Domain

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_internal_audit_target` GREEN; baseline skip-superseded |
| Program | Operational ISMS v1 — internal audit |
| Slice | First-class `AuditProgram` / child `Audit` domain: independence metadata, explicit reproducible sampling, immutable evidence snapshot pins, findings, human sign-off |
| Dual-suite (register at implement, same commit as `.rs`) | `sdd_internal_audit_baseline` · `sdd_internal_audit_target` (`tests/contracts/internal_audit.{baseline,target}.rs`) — **not** auto-discovered; add `[[test]]` in root `Cargo.toml`. `tests/sdd/` is forbidden ([ADR 0004](../adr/0004-documentation-architecture.md)) |
| ADR | Accepted [`docs/adr/0003-internal-audit.md`](../adr/0003-internal-audit.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) — Internal audit section matches landed APIs |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Consumes | Operational ISMS graph including incident inventory ([`incident-governance.md`](incident-governance.md) query helpers); temporal-assurance period / as-of / snapshot pins ([`temporal-assurance.md`](temporal-assurance.md)); lineage [`EvidenceSnapshot`](assessment-lineage.md) / `AssessmentRun`; existing `AssessmentRequests.audit_program` / `FrameworkCapabilities.supports_audit_program` fail-closed gates |
| Governance catalog (consume, do not rewrite) | [`governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) — `evidence.governance.internal-audit` / `control.governance.{internal-audit,audit-program}` remain freshness / attestation facts |
| Neighbors (must stay GREEN) | `sdd_assurance_runtime_target`, `sdd_governance_catalog_target`, `sdd_assessment_lineage_target`, `sdd_compliance_ir_target` |
| Collision fence | Prompt 22 CAPA; Prompt 24 certification pack; hosted auditor UX; catalog ISO remaps; ISO 27007 pack content |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Lineage snapshot schema (reuse) | `weeping-angel/assessment-lineage/v1` (`LINEAGE_SNAPSHOT_SCHEMA`) |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for Operational ISMS v1 internal audit. It owns the **audit program**, **child audits**, **auditor independence records**, **explicit reproducible sampling**, **evidence snapshot pinning**, **audit findings / local nonconformity refs**, **human conclusion and sign-off**, and **immutable audit history**.

It does **not** own external certification, an auditor marketplace, a generic document editor, CAPA lifecycle (Prompt 22), certification-readiness export (Prompt 24), hosted ISO 27007 UX, or catalog ISO remaps.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Internal audit is a **management-system process over that graph**, not a second control library and not a compliance conclusion engine. Machine output **prepares**; a human auditor **judges**.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

---

## 0. Collision fence (concurrent SDD)

This slice may edit only internal-audit IR types, validation of those types, and an assurance-crate prepare / sample / pin / sign-off module.

| Do not touch | Owner |
| --- | --- |
| Prompt 22 `Nonconformity` / CAPA state machine, effectiveness-review closure | Prompt 22 — store opaque `nonconformity_id` refs only |
| Prompt 24 readiness CLI, auditor evidence pack, forbidden “certified” language | Prompt 24 |
| Hosted auditor workflows / ISO 27007 program UX | Spine Phase 17; `Iso27007` remains a compile selector |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps, `tests/contracts/iso27001_remap.*` | Catalog / ISO remap |
| `frameworks/iso-27007/**` (must not be invented) | Out of scope — no 27007 pack |
| `src/finding.rs` scanner `Finding` | Recon product; not IR |
| Governance catalog test expressions (`test.governance.internal-audit-current` freshness) | Governance catalog — consume |
| Temporal-assurance envelope validity events, as-of index rewrite | Temporal assurance — consume `TimeRange` / pins when present |
| `tests/sdd/` | ADR 0004 forbids this path |
| Existing dual-suite bodies except additive `Cargo.toml` / `documentation_layout.rs` registration | Neighbors stay GREEN |

Suggested **product** modules stay in **existing crates** (no new crate, no new long-term DB):

| Concern | Home |
| --- | --- |
| `AuditProgram`, `Audit`, sample, independence, finding, sign-off types | `crates/weeping-angel-assurance-ir/src/audit.rs` |
| `AuditId`, `AuditFindingId` (keep existing `AuditProgramId`) | `crates/weeping-angel-assurance-ir/src/id.rs` |
| Inventories on `AssessmentDefinition` | `assessment.rs` — additive `#[serde(default)]` vecs |
| Dangling refs, duplicate ids, sign-off preconditions | `validation.rs` (`validate_assessment_ir`) |
| Prepare candidates, deterministic sample, pin, refuse auto-sign | `weeping-angel-assurance` (`audit` module) |
| Evidence pin bytes | Reuse `lineage::seal_evidence_snapshot` / `AssessmentRun` digests |

Tiny allowed adjustments: additive serde-default fields; new typed ids; new IR module; optional `evidence.governance.internal-audit` **fact** emission from a *signed* audit (`audited_at`, `auditor_id`) without a conclusion sentence. Do **not** redesign `AssessmentDefinition` core inventories, catalog schema, collectors, or ISO pack IDs.

`Iso27007` stays a compile **selector**. Production pack content remains `iso-27001` / `2022` only.

---

## 1. Problem / user-visible goal

Internal audit is required by the ISMS (ISO 27001 clause 9.2 intent; ISO 27007 as *guidance* for how to audit) but Weeping Angel does not treat it as an operational process.

On characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`:

- `AuditProgramId` is a typed id only (`typed_id!(AuditProgramId)`). Nothing constructs a program.
- `AssessmentRequests.audit_program` and `FrameworkCapabilities.supports_audit_program` are fail-closed booleans. Requesting a program without the capability is `CapabilityViolation`; enabling the capability still compiles **no program objects**.
- `supports_sampling` is the same pattern: a flag with no reproducible sample engine.
- `FrameworkProfile::Iso27007` parses (`iso27007` / `iso-27007`) and selects pack id `iso-27007`. There is **no** pack on disk; `load_framework_pack` is `UnknownPack` and compile continues with the in-memory assessment (empty unless the caller stuffed IR).
- `AssessmentDefinition` has no `audits` / `audit_programs` inventory.
- Governance catalog only attests freshness: `control.governance.internal-audit` + `test.governance.internal-audit-current` (`fresh-within` on `evidence.governance.internal-audit.audited_at`, 365d) and `control.governance.audit-program` + `test.governance.audit-program-attested` (`manual-review`). Envelope facts must not say “audit passed”.
- Lineage can pin an `EvidenceSnapshot` (`envelope_digests`, `collection_run_ids`, `digest`) and an `AssessmentRun` (`evidenceSnapshotDigest`, pack/catalog pins). Nothing binds those pins to an auditor’s review.
- There is no independence declaration, audit finding type, incomplete-vs-signed state machine, or human sign-off. Scanner `Finding` in `src/finding.rs` is recon, not an audit finding.

That means an “internal audit” is a disconnected folder or a dated attestation envelope. Later control tests, catalog edits, or live `assess()` can change what the organization *appears* to have reviewed. The machine can look like it concluded the audit.

**User-visible goal:** given an assessment graph, an organization can:

```text
plan an annual AuditProgram (period, scope, objectives, criteria, schedule, auditor/principal)
  → declare independence with evidence
  → open a scoped child Audit
  → accept an explicit, reproducible sample (machine lists are proposals)
  → pin the evidence snapshot/digests the auditor actually reviewed
  → record procedures, observations, findings, local nonconformity refs
  → refuse conclusion while incomplete
  → sign a human conclusion (never auto-signed)
  → replay the signed audit from pins after the live graph moves
```

Definition of done: *internal audit is a first-class operational process backed by the same evidence graph rather than a disconnected manual folder, while auditor independence and judgment remain human.*

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `AuditProgramId` | `weeping-angel-assurance-ir::id` | **Keep.** Do not replace with `String`. Add `AuditId` / `AuditFindingId` the same way. |
| `AssessmentRequests.audit_program` / `sampling` | `assessment.rs` | Keep fail-closed request bits. Do not remove. Compiling a program object requires `audit_program = true` **and** `supports_audit_program`. Sampling engine requires `sampling` **and** `supports_sampling`. |
| `FrameworkCapabilities.supports_audit_program` / `supports_sampling` | `weeping-angel-framework` | Default remains `false`. Request without support still `CapabilityViolation`. Flags do **not** imply ISO 27007 pack content. |
| `FrameworkProfile::Iso27007` | same | Compile selector only. No `frameworks/iso-27007/` pack. |
| `AssessmentDefinition` | `assessment.rs` | Additive `audit_programs` / `audits` vecs, `#[serde(default)]`, skip empty. Existing JSON without these fields deserializes. |
| `PrincipalRef` | `implementation.rs` | Reuse for auditor, principal, sign-off. Do not invent `AuditorRef`. |
| `AssessmentScope` / `SubjectSelector` | IR | Reuse for program/audit scope. Do not reimplement scope engine. |
| `EvidenceSnapshot` / `seal_evidence_snapshot` | `weeping-angel-assurance::lineage` | **Pin these.** Do not fork a second snapshot schema. |
| `AssessmentRun` | `snapshot.rs` | Pin `evidence_snapshot_digest`, `assessment_definition_digest`, pack/catalog pins, run id, JSON `asOf`. |
| Temporal `TimeRange` / as-of | [`temporal-assurance.md`](temporal-assurance.md) | Consume `TimeRange` / `select_latest_as_of` / period projection. Local `AuditPeriod { start, end }` may alias the same half-open JSON. Do not reimplement as-of selection. |
| Governance `evidence.governance.internal-audit` | catalog | Fact type. Signed audits **may** emit `audited_at` + `auditor_id`. Forbidden: “audit passed”, `Effective` on the envelope. |
| `Effectiveness` | control-test | Tests remain readiness. `Effective` controls **must not** auto-sign or auto-conclude an audit. |
| Scanner `Finding` | `src/finding.rs` | Untouched. Audit findings are a different type. |
| `Risk` | `risk.rs` | Hotspot **input** to prepare (ids + status). Do not expand the risk engine here. |
| Dual-suite neighbors | root `Cargo.toml` | Do not disturb green targets listed in the header. Register `sdd_internal_audit_*` only in the implement commit that adds the `.rs` files. |

Serde compatibility law:

- Existing assessments without audit inventories deserialize.
- Capability JSON (`supports_audit_program`, `audit_program`) keeps the same names and fail-closed meaning.
- New conclusion / status strings are **new** identifiers. Do not reuse `Effectiveness` serde names as audit conclusions.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Executable characterization lives in `sdd_internal_audit_baseline`. This section is the **historical** contract of SHA `6e31bf1`; after target GREEN the absence tests are skip-superseded (`#[ignore = "superseded by sdd_internal_audit_target"]`).

### 3.1 `AuditProgramId` is an id, not a document

[`crates/weeping-angel-assurance-ir/src/id.rs`](../../crates/weeping-angel-assurance-ir/src/id.rs):

```text
typed_id!(AuditProgramId);
```

Re-exported from `weeping-angel-assurance-ir`. `assurance_runtime.target` ACT-006 constructs `AuditProgramId::new("audit:2026")` among other newtypes. There is **no** `struct AuditProgram`, `struct Audit`, `AuditId`, independence type, sample type, or sign-off type in IR.

IR `lib.rs` modules: applicability, assessment, asset, control, crosswalk, digest, evidence, exception, extension, framework, id, identity, implementation, mapping, privacy, requirement, risk, subject, test, validation, vendor. **No** `audit` module.

### 3.2 Request / capability flags, fail-closed, no objects

[`crates/weeping-angel-assurance-ir/src/assessment.rs`](../../crates/weeping-angel-assurance-ir/src/assessment.rs):

```text
AssessmentRequests {
  statement_of_applicability, control_applicability, privacy_processing,
  risk_treatment, manual_attestation, sampling, audit_program, nonconformities
}
```

All default `false`. `AssessmentDefinition::new` does not initialize audit inventories; the struct has no such fields.

[`crates/weeping-angel-framework/src/lib.rs`](../../crates/weeping-angel-framework/src/lib.rs):

```text
FrameworkCapabilities { … supports_sampling, supports_audit_program, supports_nonconformities }
```

`Default` is all `false`. `validate_capabilities` pairs `req.audit_program` with `cap.supports_audit_program` (needle `supports_audit_program`) and `req.sampling` with `cap.supports_sampling`. Requested ∧ ¬supported → `FrameworkCompileError::CapabilityViolation`. Success does not construct a program.

### 3.3 `Iso27007` is a selector with no pack

```text
FrameworkProfile::Iso27007
as_selector() → "iso-27007"
TryFrom: "iso27007" | "iso-27007"
```

Canonical catalog forbids provider/framework segments including `iso27007` / `iso-27007` as **catalog** ids. Framework crate still parses the profile. `load_framework_pack("iso-27007", _)` has no `frameworks/iso-27007/` tree; unknown pack is skipped in `normalize`. Spine Phase 17: “Hosted auditor workflows, ISO 27007 program UX | `Iso27007` profile flag only”.

### 3.4 Assessment definition has no audits inventory

`AssessmentDefinition` inventories today: requirements, controls, mappings, evidence_requirements, tests, requests, implementations, scope, assets, identities, vendors, risks, exceptions, processing_activities.

Absent: `audit_programs`, `audits`, samples, findings, independence, sign-off.

`validate_assessment_ir` has **no** audit checks.

### 3.5 Governance catalog is freshness / attestation only

[`catalog/canonical/v1/controls/governance.toml`](../../catalog/canonical/v1/controls/governance.toml) / tests / evidence:

| Id | Meaning today |
| --- | --- |
| `control.governance.internal-audit` | Hybrid; “an internal-audit **record** exists inside the required window”; objective: freshness assessable, **independence not inferred from a file** |
| `test.governance.internal-audit-current` | `fresh-within` `evidence.governance.internal-audit` field `audited_at` duration `365d`, subject organization |
| `control.governance.audit-program` | Manual; program attested **in addition to** a current audit record; “a single audit file does not prove a program exists” |
| `test.governance.audit-program-attested` | `manual-review`; required evidence internal-audit + `evidence.manual.attestation` |
| `evidence.governance.internal-audit` | Facts `audited_at`, `auditor_id?`. Forbidden: “audit passed” |

This slice **must not** rewrite those TOML ids or turn freshness into “audit quality / passed”.

### 3.6 Sampling is a flag

`supports_sampling` / `requests.sampling` exist. There is no sample population type, seed, method, digest, or accept/reject of machine proposals. Temporal-assurance spec mentions period *evaluation* sampling; that is not audit sampling.

### 3.7 Snapshots exist; audits do not pin them

`EvidenceSnapshot { schema, envelope_digests, collection_run_ids, digest }` with sorted unique digests. `AssessmentRun` carries `evidence_snapshot_digest`, `assessment_definition_digest`, `framework_pack_digest`, `canonical_catalog_pin`. Facade `assess` can produce a run. No audit record stores those pins as “what the auditor reviewed”. Live graph mutation can change subsequent `assess` output; unsigned historical review is not an audit object.

### 3.8 No findings, independence, or human sign-off

- No independence declaration or conflict evidence.
- No audit finding / observation / nonconformity-on-audit types (Prompt 22 not landed; `supports_nonconformities` is a flag).
- No conclusion enum, sign-off principal, or “incomplete cannot conclude” gate.
- Control-test `Effectiveness::Effective` is a readiness result, not an audit conclusion.

### 3.9 Baseline suite obligations

`sdd_internal_audit_baseline` characterized **absence** on SHA `6e31bf1` and is skip-superseded on this HEAD:

- `AuditProgramId` exists; `struct AuditProgram` / `struct Audit` do not.
- `audit_program` / `supports_audit_program` are booleans; compile with both true still yields no program inventory.
- `Iso27007` selector; no pack directory.
- `AssessmentDefinition` source has no `audits` field.
- Governance TOML still has the freshness tests above.
- No sample engine / snapshot pin-on-audit / independence / sign-off APIs.

After target GREEN, prove those absence assertions **fail**, then:

```rust
#[ignore = "superseded by sdd_internal_audit_target"]
```

on each baseline test (same protocol as lineage / population).

---

## 4. Desired behavior (target)

### 4.1 Domain split

```text
AuditProgram     annual (or other) plan: period, scope, objectives, criteria, schedule, people, independence, child audits
Audit            one engagement under a program
AuditSample      explicit, reproducible selection the auditor accepted
AuditEvidencePin immutable snapshot/digests reviewed
AuditFinding     auditor-created finding (not scanner Finding, not CAPA)
AuditSignOff     human conclusion; never machine-written
```

Machine **prepare** returns candidates and proposals. Persisting a conclusion or sample-as-accepted is always an explicit auditor/principal action.

### 4.2 `AuditProgram`

```text
AuditProgram {
  schemaVersion: "assurance-ir/v1"
  id: AuditProgramId
  title: String
  period: AuditPeriod            // half-open [start, end)
  scope: AssessmentScope         // reuse IR
  objectives: Vec<String>        // non-empty when status ≥ Approved
  criteria: Vec<AuditCriterion>  // framework / requirement / control refs
  schedule: AuditSchedule        // planned windows for child audits
  principal: PrincipalRef        // program owner (CAE / ISMS owner)
  auditor: PrincipalRef          // default engagement auditor
  independence: IndependenceRecord
  childAuditIds: Vec<AuditId>
  status: AuditProgramStatus     // draft | approved | inProgress | closed
  preparedFrom?: PrepareDigest   // optional machine-prep identity
}
```

**Annual program:** `period.end = period.start + 1 year` is the canonical fixture (e.g. `[2026-01-01T00:00:00Z, 2027-01-01T00:00:00Z)`). Other period lengths are allowed; tests for “annual program” use the one-year fixture.

`AuditCriterion` names what the audits evaluate against (requirement ids, control ids, and/or a framework pin). Criteria are references into the existing graph, not a new control library.

`AuditSchedule` lists planned `{ auditId?, window: AuditPeriod, scopeNote? }`. Child audits must lie inside the program period (fail closed if `audit.period` is outside `program.period`).

Status:

| Status | Meaning |
| --- | --- |
| `draft` | editable plan; children may be prepared |
| `approved` | principal accepted plan; independence present |
| `inProgress` | ≥1 child audit in progress or prepared |
| `closed` | all children signed or withdrawn; program immutable except supersession |

A program does **not** auto-close because all in-scope controls are `Effective`.

### 4.3 Child `Audit`

```text
Audit {
  schemaVersion: "assurance-ir/v1"
  id: AuditId
  programId: AuditProgramId
  title: String
  period: AuditPeriod
  scope: AssessmentScope
  sample: Option<AuditSample>              // required to conclude
  selectedControls: Vec<ControlId>
  selectedRequirements: Vec<RequirementId>
  evidencePin: Option<AuditEvidencePin>    // required to conclude
  procedures: Vec<AuditProcedure>
  observations: Vec<AuditObservation>
  findings: Vec<AuditFindingId>            // ids; documents in assessment.auditFindings or nested
  nonconformityRefs: Vec<NonconformityRef> // opaque ids for Prompt 22
  conclusion: Option<AuditConclusion>      // only via sign-off
  signOff: Option<AuditSignOff>
  status: AuditStatus
  history: Vec<AuditHistoryEvent>          // append-only
  sampleProposal?: AuditSampleProposal     // machine suggestion; not the sample
}
```

`AuditStatus`: `draft` | `prepared` | `inProgress` | `concluded` | `signed` | `withdrawn`.

`AssessmentDefinition` additive inventories:

```text
audit_programs: Vec<AuditProgram>   // skip empty
audits: Vec<Audit>                  // skip empty
audit_findings: Vec<AuditFinding>   // skip empty; side inventory
```

Landed: findings live in `AssessmentDefinition.audit_findings`; `Audit.findings` is the id list. Validation is dangling-closed.

Child audit `programId` must exist. Selected controls/requirements must exist on the assessment (fail closed). Scope may be a subset of the program scope; it must not silently exceed program organizations/subjects (fail closed or require an explicit program-level inclusion).

### 4.4 Independence

```text
IndependenceRecord {
  auditor: PrincipalRef
  principal: PrincipalRef
  declaredAt: DateTime<Utc>
  statement: String              // non-empty
  evidenceRefs: Vec<String>      // envelope digests and/or artifact digests
  conflictFlags: Vec<IndependenceConflict>  // machine flags; never auto-cleared
  accepted: bool                 // principal/auditor accepted the declaration
}
```

Laws:

1. Program `approved` and any audit `signed` require an `IndependenceRecord` with `accepted = true`, non-empty `statement`, ≥1 `evidenceRefs` digest, and both auditor and principal set.
2. Machine **may** flag conflicts (`flag_independence_conflicts` emits `auditorOwnsControl` when the auditor equals a selected control implementation owner). Flags persist and do not block prepare. They never set `accepted`. Conclude / sign-off require `is_accepted_declaration()` (accepted + non-empty statement + ≥1 evidence ref). There is no separate override-rationale field; an accepted declaration is the human override. Absence of flags is not “independent”.
3. The engine **must not** set `accepted = true` or invent a statement.
4. Independence evidence is pinned like other audit evidence (digest list); live envelope mutation must not rewrite the reviewed declaration bytes.

Governance catalog remains honest: a file path is not independence.

### 4.5 Sampling — explicit and reproducible

```text
AuditSample {
  populationId: String           // stable identity of the population snapshot
  populationDigest: String       // digest of sorted member ids (+ kind)
  method: SampleMethod           // census | systematic | seededRandom | judgmental
  seed: Option<String>           // required for systematic / seededRandom
  size: u32
  selectedIds: Vec<String>       // sorted unique
  acceptedBy: PrincipalRef
  acceptedAt: DateTime<Utc>
  proposalDigest: Option<String> // if accepted from a proposal
  sampleDigest: String           // canonical digest of method, seed, populationDigest, selectedIds
}

SampleMethod = census | systematic | seededRandom | judgmental

AuditSampleProposal {
  populationId, populationDigest, method, seed?, size
  suggestedIds: Vec<String>
  rationale: String              // machine explanation (stale/failed/hotspot)
  generatedAt: DateTime<Utc>
  proposalDigest: String
  kind: "proposal"               // literal; never persisted as AuditSample
}
```

Laws:

1. Same `(method, seed, populationDigest, size)` ⇒ same `suggestedIds` / `selectedIds` (deterministic). Population members are sorted by id before selection. Digest uses IR `canonical_digest` / `canon/v1`.
2. `census` selects the full population (`size == population.len()`).
3. `seededRandom` / `systematic` require a seed. Seed is canonical bytes (hex or utf-8 string documented in tests), not `Utc::now()`.
4. `judgmental` **cannot** be produced by `propose_sample`. It requires the auditor to supply `selectedIds` explicitly.
5. Attaching `AuditSampleProposal` does **not** set `Audit.sample`. `accept_sample(audit, proposal | explicit, principal, clock)` copies ids and records `acceptedBy`.
6. Machine suggestions are **proposals only**. Tests must show: prepare can emit a proposal; `conclude` / `sign_off` fail if only a proposal exists; after accept, `sampleDigest` is stable.
7. `requests.sampling` without `supports_sampling` remains `CapabilityViolation`. When sampling is requested, a signed audit must have an accepted `AuditSample` (census is valid).

Do not use a non-deterministic RNG. Do not sample from a live unsorted `HashMap` iteration.

### 4.6 Evidence snapshot pinning

```text
AuditEvidencePin {
  evidenceSnapshotDigest: String     // EvidenceSnapshot.digest
  envelopeDigests: Vec<String>       // copy of snapshot.envelope_digests (sorted)
  collectionRunIds: Vec<String>
  assessmentRunId?: AssessmentId
  assessmentDefinitionDigest?: String
  frameworkPackDigest?: String
  canonicalCatalogDigest?: String
  asOf?: DateTime<Utc>               // consume temporal-assurance clock when present
  period?: AuditPeriod
  pinnedAt: DateTime<Utc>
  pinnedBy: PrincipalRef
}
```

Laws:

1. Landed: `pin_evidence(audit, snapshot, principal, clock)` writes the pin from an existing `EvidenceSnapshot` (digest + envelope digests + collection run ids) plus the audit period. It does not reseal envelopes. Optional `AssessmentRun` / pack / catalog / `asOf` fields exist on the pin type and stay unset on this engine path.
2. After pin, mutating the live evidence set or re-running `assess` **must not** change `AuditEvidencePin` fields.
3. Historical reproducibility: `replay_audit(audit)` / `reviewed_envelopes(audit, ledger)` returns the pinned digest set. A later envelope with a new digest is **not** in the reviewed set.
4. Sign-off requires a pin. An audit whose pin is missing or whose stored `evidenceSnapshotDigest` does not match a recomputed digest over the stored envelope list fails closed (tamper).
5. Prefer calling `seal_evidence_snapshot`. Do not invent a parallel schema id; reuse `weeping-angel/assessment-lineage/v1` for the snapshot document. The pin is an IR field pointing at that digest.

Copy `AssessmentRun.asOf` (and the audit’s `AuditPeriod`) into the pin. Do not pretend live `Utc::now()` was the review clock.

### 4.7 Procedures, observations, findings

```text
AuditProcedure { id, title, selectedControlIds, status: planned | performed | notPerformed, notes? }
AuditObservation { id, procedureId?, evidenceDigests: Vec<String>, text, recordedBy, recordedAt }

AuditFinding {
  id: AuditFindingId
  auditId: AuditId
  kind: observation | finding | nonconformity
  severity?: minor | major | opportunity   // required when kind = finding | nonconformity
  title: String
  description: String
  controlIds: Vec<ControlId>
  requirementIds: Vec<RequirementId>
  evidenceDigests: Vec<String>             // must be ⊆ pin.envelopeDigests when pin exists
  createdBy: PrincipalRef
  createdAt: DateTime<Utc>
  nonconformityId?: NonconformityRef       // opaque; Prompt 22 owns lifecycle
}
```

Laws:

1. Creating a finding is an explicit API (`record_finding`). Failed control tests **may** appear on the prepare bundle as *candidate observations*; they do not insert `AuditFinding` rows.
2. `kind = nonconformity` does **not** start CAPA. Optional `nonconformityId` is a stable string/newtype for Prompt 22 to consume later. This slice does not implement Open→Closed CAPA.
3. Scanner `Finding` is never auto-promoted. No `From<src::Finding> for AuditFinding`.
4. Finding evidence digests must be in the pin once pinned (fail closed).
5. `supports_nonconformities` remains a compile flag for Prompt 22; this slice may store refs without enabling that capability. Do not silently set `requests.nonconformities = true`.

### 4.8 Conclusion, incomplete, sign-off (never auto-sign)

```text
AuditConclusion = conformant | qualified | nonconformant | notConcluded

AuditSignOff {
  principal: PrincipalRef        // Identity preferred; Team/Role allowed
  signedAt: DateTime<Utc>
  conclusion: AuditConclusion    // not notConcluded
  statement: String              // non-empty
}

fn prepare_audit(...) -> Audit            // status = prepared; conclusion = None
fn conclude_audit(...) -> Result          // may set status = concluded only if complete; still unsigned
fn sign_off(audit, principal, conclusion, statement, clock) -> Result<Audit>
```

**Incomplete audit cannot conclude.** `conclude_audit` and `sign_off` fail closed unless all of:

- program exists and is `approved` or `inProgress`
- independence `accepted` with evidence
- accepted `AuditSample` present (`sampleDigest` set)
- `AuditEvidencePin` present and internally consistent
- every procedure is `performed` or `notPerformed` with notes (no leftover `planned` if the procedure is in scope)
- `conclusion` is supplied **only** by the caller of `sign_off`, never defaulted from control tests

**Never auto-sign:**

- No `Default` for `AuditSignOff`.
- No path from `Effectiveness::Effective` (or all-green readiness) to `signOff`.
- `prepare_*` must leave `signOff = None`, `conclusion = None`.
- Tests must call prepare + pin + sample + all-effective fixtures and assert the audit is **not** signed.

`sign_off` sets `status = signed`, appends a history event, and freezes the audit: further `record_finding` / sample changes fail closed. Correction is a **new** `AuditId` that `supersedes` the prior (history on the successor points at the predecessor id). The signed document remains queryable.

`notConcluded` is not a successful sign-off value. Withdrawal is `status = withdrawn` with rationale in history, not a fake conformant conclusion.

### 4.9 Immutable history

```text
AuditHistoryEvent {
  at: DateTime<Utc>
  principal?: PrincipalRef
  kind: String   // prepared | sampleAccepted | pinned | findingRecorded | signed | withdrawn | superseded
  payloadDigest: String
}
```

Append-only. Signed audits do not rewrite prior events. Replay uses the signed body + pins, not the latest live graph.

### 4.10 Automation (prepare only)

```text
AuditPrepareBundle {
  candidateScope: AssessmentScope
  staleOrFailedControls: Vec<ControlId>
  riskHotspots: Vec<RiskId>
  evidenceBundle: EvidenceSnapshot        // or digest + envelope list
  samplePopulations: Vec<PopulationRef>
  sampleProposal?: AuditSampleProposal
  priorFindings: Vec<PriorFindingRef>     // from earlier signed audits
  remediationStatus: Vec<RemediationRef>  // Prompt 16 RemediationId strings; empty when unused
}

fn prepare_audit_program(definition, period, lastRun?, clock) -> (AuditProgram, AuditPrepareBundle)
fn prepare_audit(program, scope, lastRun?, clock) -> (Audit, AuditPrepareBundle)
```

Inputs: assessment inventories, last `AssessmentRun` / `ControlTestResult`s when provided, `Risk` rows, prior signed `Audit`s, lineage snapshot when provided.

The bundle is **advisory**. Persisting it onto a draft program/audit does not approve, sample-accept, conclude, or sign.

Stale/failed controls: those with last known `Effectiveness` in `{Ineffective, StaleEvidence, InsufficientEvidence}` (and `Partial` if present). Missing last-run ⇒ empty stale list, not a fabricated fail.

Risk hotspots: open / unmitigated `Risk` ids when the operational register exists; else all `RiskStatus::Open` stubs.

Prior findings: from `definition.audits` that are `signed` and overlap scope/controls.

Remediation: this slice always returns `[]`. Do not implement Prompt 16 here. Prior findings are taken from `definition.audit_findings` whose `auditId` belongs to a signed audit.

### 4.11 Capability and profile

- Default capabilities stay fail-closed.
- Creating/validating program objects on an assessment that has `requests.audit_program = true` requires `supports_audit_program` at compile, same as today.
- IR `validate_assessment_ir` validates inventories whenever they are non-empty, even if the request bit is false (fail closed: cannot smuggle unsigned programs past validation).
- `Iso27007` still has no pack. Do not add `frameworks/iso-27007/`.
- Do not set `supports_audit_program = true` on the ISO 27001 default target as a side effect.

### 4.12 Optional fact emission (not a conclusion)

A **signed** audit MAY be projected to `evidence.governance.internal-audit` with facts `audited_at = signOff.signedAt`, `auditor_id = auditor identity string`. The envelope must not contain “passed” / “conformant” as a fact key. `test.governance.internal-audit-current` continues to mean “a record exists inside 365d”, not “the ISMS is fine”.

This projection is optional and **not landed** in this slice. Catalog freshness tests continue to mean “a record exists inside 365d”.

### 4.13 Crate API sketch (normative names for tests)

Landed in `weeping-angel-assurance::audit` (not crate-root re-exports):

```text
prepare_audit_program
prepare_audit
propose_sample
accept_sample
pin_evidence
record_finding
conclude_audit
sign_off
replay_audit
reviewed_envelopes
```

Semantics above are law.

---

## 5. Tests (target GREEN on this HEAD; baseline skip-superseded)

Register at implement, **same commit** as the `.rs` files:

```toml
[[test]]
name = "sdd_internal_audit_baseline"
path = "tests/contracts/internal_audit.baseline.rs"

[[test]]
name = "sdd_internal_audit_target"
path = "tests/contracts/internal_audit.target.rs"
```

Protocol: write tests first → **RED** (must fail on current main for missing program/audit/sample/pin/sign-off — not unrelated compile noise) → implement → **GREEN**. Baseline stays GREEN until superseded.

Each target test title: `IA-00N: <exact subject>` matching this table.

| Id | Scenario | Expected |
| --- | --- | --- |
| IA-001 | **annual program** | Construct `AuditProgram` with a one-year `period`, scope, objectives, criteria, schedule, auditor, principal; persists on `AssessmentDefinition.audit_programs`; `validate_assessment_ir` Ok when ids hang together |
| IA-002 | **scoped audit** | Child `Audit` under that program with narrower scope and selected controls/requirements; dangling control/program id fails closed; audit period outside program period fails closed |
| IA-003 | **auditor independence metadata** | `IndependenceRecord` carries auditor, principal, statement, evidence digest(s), `accepted`; sign-off without it fails; machine conflict flag does not auto-accept |
| IA-004 | **deterministic sample** | Same method+seed+sorted population ⇒ identical `selectedIds` and `sampleDigest`; `propose_sample` is a proposal; `accept_sample` required before conclude; judgmental method not emitted by propose |
| IA-005 | **evidence snapshot pinning** | `pin_evidence` stores lineage snapshot digest + envelope digests; after extra live envelopes are added, pin is unchanged; recomputed digest of stored lists matches `evidenceSnapshotDigest` |
| IA-006 | **finding creation** | `record_finding` adds an auditor finding bound to the audit and pinned evidence; failed tests / scanner `Finding` do not auto-insert; nonconformity ref is opaque |
| IA-007 | **incomplete audit** | Missing sample, missing pin, planned procedures, or missing independence ⇒ `conclude_audit` / `sign_off` err; status stays unsigned |
| IA-008 | **signed audit** (never auto-sign) | `sign_off` with human principal + conclusion + statement sets `status = signed`; prepare on an all-`Effective` fixture leaves `signOff = None`; no API sets sign-off without principal |
| IA-009 | **historical reproducibility** | Signed audit replay returns the same pin/sampleDigest/findings/conclusion after the live evidence set and control results change |
| IA-010 | Neighbor targets | `sdd_assurance_runtime_target`, `sdd_governance_catalog_target`, `sdd_assessment_lineage_target`, `sdd_compliance_ir_target` stay GREEN (verify / CI; do not rewrite those suites) |

IA-001–IA-009 are the Prompt 21 acceptance tests. IA-010 is workspace law.

Prefer tests that construct IR + call engine functions so RED is a missing type/function or assertion, not an unrelated crate error. Clock injection is mandatory (no wall-clock seeds).

---

## 6. Dual-suite / SDD protocol (abort rather than skip)

Protocol completed on this HEAD:

1. Spec (this file) + ADR + `CANONICAL_SPECS` registration.
2. Baseline characterized SHA `6e31bf1` absence, then skip-superseded.
3. Target RED for missing program/audit/sample/pin/sign-off, then GREEN (IA-001–IA-009).
4. Implement: IR `audit` module + validation + assurance `audit` engine.
5. ADR **Accepted**; public contract [`assurance-runtime.md`](assurance-runtime.md) documents landed APIs.

Traces only under `.sdd/runs/` and `.sdd/artifacts/`. Do not dump into `docs/sdd`.

---

## 7. Non-goals / out of scope

- External certification workflow, certificate objects, or “audit passed / certified / compliant” product claims
- Auditor marketplace, staffing, or multi-tenant hosted auditor UX (spine Phase 17)
- Generic document editor / rich-text working papers
- Prompt 22 CAPA lifecycle (containment, RCA, effectiveness review, closure)
- Prompt 24 certification-readiness pack and readiness CLI
- ISO 27007 framework pack content or licensed ISO normative text
- Rewriting catalog ISO remaps or governance TOML control/test ids
- Replacing `EvidenceSnapshot` / ledger with a new database
- Auto-signing or auto-concluding from control-test effectiveness
- Promoting scanner `Finding` / `SemanticFinding` into audit findings
- Sampling as a hidden cron inside temporal period evaluation (that belongs to temporal-assurance)
- New workspace crate

---

## 8. Crate homes and files (implement phase)

| Path | Role |
| --- | --- |
| `crates/weeping-angel-assurance-ir/src/audit.rs` | Program, audit, sample, pin, finding, sign-off types |
| `crates/weeping-angel-assurance-ir/src/id.rs` | `AuditId`, `AuditFindingId`; keep `AuditProgramId` |
| `crates/weeping-angel-assurance-ir/src/assessment.rs` | Additive inventories |
| `crates/weeping-angel-assurance-ir/src/validation.rs` | Integrity + incomplete/signed gates callable from engine |
| `crates/weeping-angel-assurance-ir/src/lib.rs` | Re-exports |
| `crates/weeping-angel-assurance/src/audit.rs` (new module ok) | prepare / propose_sample / accept_sample / pin_evidence / record_finding / sign_off |
| `tests/contracts/internal_audit.baseline.rs` | Absence characterization |
| `tests/contracts/internal_audit.target.rs` | Normative IA-001–IA-010 |
| `Cargo.toml` | `[[test]]` `sdd_internal_audit_{baseline,target}` **same commit as `.rs`** |
| `tests/contracts/documentation_layout.rs` | `CANONICAL_SPECS` includes this file (spec-first) |
| `docs/adr/0003-internal-audit.md` | Decision (**Accepted**) |

Do not add a crate. Do not edit collision-fenced paths in §0.

---

## 9. Acceptance criteria

- An annual `AuditProgram` can be recorded with period, scope, objectives, criteria, schedule, auditor/principal, independence, and child audits.
- A scoped child `Audit` selects controls/requirements, holds procedures/observations/findings, and cannot outlive its program period or dangle graph refs.
- Auditor independence is metadata + evidence, accepted by a human; the machine only flags conflicts.
- Sampling is explicit: method, seed, population digest, selected ids, sample digest; same inputs replay; machine output is a proposal until `accept_sample`.
- Audit evidence is pinned to immutable snapshot/digests (`EvidenceSnapshot` / `AssessmentRun` pins); later live changes do not rewrite the reviewed set.
- Findings are created by the auditor; tests and scanner findings do not auto-create them; CAPA is not started here.
- Incomplete audits cannot conclude or sign.
- Signed audits require a human principal, statement, and conclusion; nothing auto-signs, including all-`Effective` fixtures.
- A signed audit is historically reproducible from pins after the live graph moves.
- Dual-suite registered at implement; baseline GREEN then skip-superseded; target GREEN; neighbor targets remain GREEN.
- `Iso27007` remains a pack-less compile selector; governance catalog freshness tests are not rewritten.

---

## 10. Risks

- Readers treat governance `internal-audit-current` or all-green `Effectiveness` as the audit conclusion (mitigate: separate types; IA-008; envelope fact ban on “passed”).
- Machine sample silently becomes the sample (mitigate: proposal type + accept API + IA-004).
- Pin stores live `assess()` identity without envelope list, so later ledger contents leak into replay (mitigate: store envelope digests + snapshot digest; IA-005 / IA-009).
- Prompt 22 lands a different nonconformity id type (mitigate: opaque ref, no lifecycle).
- Temporal-assurance `TimeRange` not landed yet (mitigate: identical `AuditPeriod` shape).
- Independence auto-pass when auditor ≠ principal but auditor owns the controls (mitigate: conflict flags vs implementation owners; human override).
- `supports_audit_program = true` on ISO 27001 default target would change ACT-007 (mitigate: do not flip defaults).
- Dual-suite forgotten in `Cargo.toml` (mitigate: register in the same commit as the `.rs` files; ADR 0004).
- History mutated in place after sign-off (mitigate: freeze + superseding new `AuditId`).

---

## 11. Definition of done

Internal audit is a first-class operational process on the assurance graph: programs and audits are IR, samples are explicit and reproducible, evidence is pinned, findings are human-recorded, conclusions are human-signed, and yesterday’s review cannot be rewritten by today’s collect.

Prompt 21 mission complete when `sdd_internal_audit_target` is GREEN, baseline is skip-superseded, and neighbor suites listed in the header remain GREEN.

---

## 12. Landed surface (HEAD)

Product:

- `crates/weeping-angel-assurance-ir/src/audit.rs`
- `crates/weeping-angel-assurance-ir/src/id.rs` (`AuditId`, `AuditFindingId`; keep `AuditProgramId`)
- `crates/weeping-angel-assurance-ir/src/assessment.rs` (`audit_programs` / `audits` / `audit_findings`)
- `crates/weeping-angel-assurance-ir/src/validation.rs` (`validate_audit_inventories`)
- `crates/weeping-angel-assurance-ir/src/lib.rs` re-exports
- `crates/weeping-angel-assurance/src/audit.rs`
- `crates/weeping-angel-assurance/src/lib.rs` (`pub mod audit`)

Tests/docs:

- `tests/contracts/internal_audit.baseline.rs` (skip-superseded)
- `tests/contracts/internal_audit.target.rs` (IA-001–IA-009)
- root `Cargo.toml` `[[test]]` `sdd_internal_audit_{baseline,target}`
- `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` includes this file
- pointer in [`assurance-runtime.md`](assurance-runtime.md)
- Accepted ADR [`docs/adr/0003-internal-audit.md`](../adr/0003-internal-audit.md)

Not landed: `evidence.governance.internal-audit` fact projection from signed audits; ISO 27007 pack; crate-root re-export of the audit engine.
