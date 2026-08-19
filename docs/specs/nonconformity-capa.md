# SDD: Nonconformity and CAPA Engine (ISMS v1)

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_nonconformity_capa_target` GREEN; baseline skip-superseded |
| Program | Operational ISMS v1 — Prompt 22 nonconformity / CAPA |
| Slice | Canonical `Nonconformity` + `CorrectiveAction` lifecycle with containment, RCA, planned actions, implementation evidence, effectiveness review, and immutable closure |
| Dual-suite | `sdd_nonconformity_capa_baseline` (skip-superseded) · `sdd_nonconformity_capa_target` GREEN (`tests/contracts/nonconformity_capa.{baseline,target}.rs`) — **not** auto-discovered (I3); listed in root [`Cargo.toml`](../../Cargo.toml). `tests/sdd/` is forbidden ([ADR 0004](../adr/0004-documentation-architecture.md)) |
| ADR | Accepted [`docs/adr/0028-nonconformity-capa.md`](../adr/0028-nonconformity-capa.md) — 0003-* sibling filename (cite by **path**) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) — Nonconformity and CAPA pointer; do not fork the spine |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) — this file is the human SSOT; `docs/sdd/` is a stub; traces go to `.sdd/runs` and `.sdd/artifacts` |
| Governance catalog (do **not** retarget) | [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) — `control.governance.corrective-action` + `test.governance.corrective-action-recorded` stay **attestation facts**, not this operational engine |
| Prompt | [`docs/prompts/operational-isms-v1/22-nonconformity-capa.md`](../prompts/operational-isms-v1/22-nonconformity-capa.md) |
| Consumes (seams) | Prompt 16 [`remediation-engine.md`](remediation-engine.md) (`Remediation`, `RemediationRef`, `VerificationPolicy`); Prompt 19 [`incident-governance.md`](incident-governance.md) (`Incident`, PIR `proposed_corrective_action_ids`); Prompt 21 [`internal-audit.md`](internal-audit.md) (`AuditFinding`, opaque `NonconformityRef`); Prompt 15 [`isms-events-drift.md`](isms-events-drift.md) (`IsmsEventKind::{NonconformityOpened,CorrectiveActionOverdue,ControlRegressed}`); `Effectiveness` from `weeping-angel-control-test` |
| Neighbors (must stay GREEN) | `sdd_internal_audit_target`, `sdd_incident_governance_target`, `sdd_remediation_engine_target`, `sdd_isms_events_drift_target`, `sdd_governance_catalog_target`, `sdd_assurance_runtime_target`, `sdd_compliance_ir_target`, `sdd_documentation_layout` |
| Collision fence | Catalog TOML, ISO packs, GitHub collector, scanner `Finding`, existing `sdd_*` suite bodies except additive `Cargo.toml` / `documentation_layout.rs` registration |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization | Current workspace HEAD (git `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` + landed Prompts 15/16/19/21 product). Baseline encodes **this** found case |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `serde_json` struct field order + `BTreeMap` / `BTreeSet` (`canon/v1`) |
| JSON | `#[serde(rename_all = "camelCase")]` |
| Identity | `typed_id!(NonconformityId)` + `typed_id!(CorrectiveActionId)` + `validate_stable_id`; **no** random v4 |
| Workspace verify | `cargo test --test sdd_nonconformity_capa_target`; `cargo test --test sdd_documentation_layout`; keep header neighbors GREEN; `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable human SSOT for Operational ISMS v1 **nonconformity and CAPA**. It owns the **canonical `Nonconformity` record**, the **canonical `CorrectiveAction` record**, **explicit proposal/open (never silent major/minor)**, **containment**, **root-cause analysis**, **planned/implemented actions**, **effectiveness criteria and review period**, **evidence-backed verification**, **closure decision**, **cancellation/supersession with rationale**, **reopen**, and **immutable history**.

It does **not** own a generic issue tracker, an AI root-cause engine, Prompt 16 ticket/remediation product, Prompt 19 incident IR, Prompt 21 audit process, Prompt 15 event-bus/drift product, or the governance-catalog corrective-action *attestation*.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

A nonconformity is a **management-system record over that graph**. A failed control test, an audit finding, or an incident is **not** a CAPA until a named principal **proposes/opens** it and a **classification decision** is recorded.

```text
ControlRegressed / AuditFinding / Incident / Manual
        ↓  explicit propose / open (principal, time)
   Nonconformity   (this slice)
        ├─ containment actions
        ├─ root-cause analysis
        ├─ CorrectiveAction[]  (target dates, evidence, criteria, review period)
        ├─ effectiveness review (Effectiveness + declared window)
        └─ closure decision + immutable history
```

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only. Do not write suites under `tests/sdd/`.

---

## 0. Collision fence (concurrent SDD)

This slice may add CAPA IR + a library engine in **existing crates**. It must not fork neighboring ISMS slices or invent a second incident/audit IR.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps | Catalog / ISO remap |
| `control.governance.corrective-action`, `test.governance.corrective-action-recorded`, `control.governance.continual-improvement` | Governance catalog — **keep GREEN**; do not retarget as the operational CAPA engine |
| `crates/weeping-angel-collector/src/github/**`, `tests/contracts/github_collector.*` | GitHub collector |
| `src/finding.rs` scanner `Finding` | Recon/scanner product; **not IR**; never auto-promote |
| Prompt 15 `detect_isms_drift` / event catalog rewrite | [`isms-events-drift.md`](isms-events-drift.md) — **consume** `IsmsEventKind::{NonconformityOpened,CorrectiveActionOverdue,ControlRegressed}`; do not reimplement drift; do not populate snapshot inventories by mutating the detector |
| Prompt 16 `Remediation` type, ticket adapters, SLA engine | [`remediation-engine.md`](remediation-engine.md) — **cite** `RemediationRef`; do not replace `Remediation` with CAPA |
| `Incident` schema / `corrective_action_ids: Vec<RemediationRef>` / PIR `proposed_corrective_action_ids` | [`incident-governance.md`](incident-governance.md) — **consume**; do not retarget those fields to `CorrectiveActionId` |
| `AuditProgram` / `Audit` / `AuditFinding` field expansion beyond resolving `nonconformity_id` | [`internal-audit.md`](internal-audit.md) — keep `NonconformityRef`; `kind = nonconformity` still does not start CAPA |
| `ASSURANCE_IR_SCHEMA` bump | Forbidden |
| Auto-enable `AssessmentRequests.nonconformities` or `FrameworkCapabilities.supports_nonconformities` | Fail-closed compile flags stay `false` |
| `tests/sdd/` | ADR 0004 forbids this path |
| Existing `sdd_*` suite bodies except additive `Cargo.toml` / `documentation_layout.rs` | Neighbors stay GREEN |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| `Nonconformity`, `CorrectiveAction`, states, source, classification, containment, RCA, effectiveness criteria, closure, history | `crates/weeping-angel-assurance-ir/src/capa.rs` (name flexible; `nonconformity.rs` also acceptable) |
| `NonconformityId`, `CorrectiveActionId` | `crates/weeping-angel-assurance-ir/src/id.rs` `typed_id!` |
| `AssessmentDefinition.nonconformities` / `.corrective_actions` | `assessment.rs` — additive `Vec` + `serde(default)` + skip empty |
| Graph integrity (dangling refs, duplicate ids, illegal transitions, missing RCA, immutable closure) | `validation.rs` (`validate_assessment_ir` + clocked helpers) |
| Re-exports | `lib.rs` |
| Propose/open, contain, record RCA, plan/implement, evaluate effectiveness, close, reopen, overdue query | `weeping-angel-assurance` module `capa` (library; no HTTP) |
| `Effectiveness` / `ControlTestResult` | **consume**; never mutate; never treat one `Effective` as auto-close |
| `AuditFinding.nonconformity_id` | Keep `NonconformityRef`; resolve against inventory when present |
| `Incident.corrective_action_ids` | Stay `RemediationRef` |
| Drift `IsmsSnapshot.nonconformities` / `.corrective_actions` | Stay `GovernanceRecord` adapters; empty ⇒ no-op |

Landed adjustments: new `typed_id!` aliases; serde defaults / `skip_serializing_if`; transition APIs; validation messages; re-exports. `NonconformityRef` remains `String` so Prompt 21 opaque fixtures (`nc:opaque-prompt-22`) still deserialize. `ASSURANCE_IR_SCHEMA` was not bumped. `Incident`, `AuditFinding`, and `Remediation` schemas were not expanded except resolving refs when CAPA inventory is non-empty.

---

## 1. Problem / user-visible goal

Weeping Angel can attest that an organization *has* a corrective-action process (`control.governance.corrective-action` + manual review). It can store opaque `AuditFinding.nonconformityId` strings, link incidents to Prompt 16 remediations, and *name* `IsmsEventKind::NonconformityOpened` / `CorrectiveActionOverdue`. It **cannot** prove that a detected nonconformity was contained, root-caused, corrected, verified for effectiveness over a declared period, and formally closed.

On current HEAD:

- there is no `Nonconformity` / `CorrectiveAction` product type in `weeping-angel-assurance-ir` or any product crate;
- `AssessmentDefinition` inventories include incidents, remediations, audit programs/audits/findings — **no** `nonconformities` / `corrective_actions` registers;
- `AssessmentRequests.nonconformities` and `FrameworkCapabilities.supports_nonconformities` are fail-closed compile flags (default `false`); enabling them compiles **no** CAPA objects;
- `AuditFinding.nonconformity_id` is `NonconformityRef = String`; `AuditFindingKind::Nonconformity` does **not** start CAPA;
- `Incident.corrective_action_ids` and PIR `proposed_corrective_action_ids` are `RemediationRef` (Prompt 16);
- drift snapshot inventories for `nonconformities` / `corrective_actions` are empty `GovernanceRecord` bags; detectors are no-ops until a caller stuffs them;
- catalog `control.governance.corrective-action` remains a hybrid attestation fact.

Without an operational CAPA record, a single green control retest looks like closure, an audit “nonconformity” finding looks like a registered CAPA, an incident close looks like effectiveness review, and a cancelled gap has no accountable rationale.

**User-visible goal (mission):** the system must prove how a detected nonconformity was contained, corrected, verified for effectiveness, and formally closed.

```text
source (audit finding / incident / ControlRegressed / manual)
  → explicit propose/open (principal + time)     // never automatic
  → NonconformityId + classification decision (major|minor|opportunity)
  → containment actions
  → root-cause analysis
  → CorrectiveAction (target dates, implementation evidence, criteria, review period, reviewer)
  → implemented
  → effectiveness review over the declared window
  → closure decision (principal + time + rationale)
  → immutable history
```

Examples the engine **must** distinguish:

```text
AuditFinding { kind: nonconformity, severity: major }
  → not a Nonconformity; does not set CAPA classification

propose_from_finding(finding, principal, time)
  → Nonconformity Open; classification unset until classify()

classify(major) without principal/rationale
  → fail closed

Open → RootCauseIdentified without RCA
  → fail closed (missing root cause)

CorrectiveAction.dueAt < as_of, not implemented
  → overdue query true; does not auto-close or auto-reclassify

One ControlTestResult Effective
  → does NOT close CAPA; does NOT skip EffectivenessReview

Sustained window 14d, two greens 3 days apart
  → not satisfied; cannot close

Effectiveness review Failed
  → cannot close; return to plan/implement with history

Closed record rewrite
  → ImmutableClosure

Closed → Open (reopen)
  → allowed with principal + rationale; history append-only

Incident closed with open RemediationRef
  → still valid (Prompt 19); does not close this CAPA

Incident / PIR propose nonconformity
  → explicit API; does not silently create major/minor
```

Definition of done (prompt): *the system can prove how a detected nonconformity was contained, corrected, verified for effectiveness and formally closed.*

---

## 2. Compatibility / dependencies

Pinned to **current HEAD** (landed Prompts 15, 16, 19, 21).

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| New `Nonconformity` / `CorrectiveAction` | `weeping-angel-assurance-ir` | **SSOT records.** Do not create a sidecar GRC crate or `CapaV2`. |
| `AssessmentDefinition` | `assessment.rs` | Additive `nonconformities: Vec<Nonconformity>` and `corrective_actions: Vec<CorrectiveAction>` with `#[serde(default)]` + skip empty. `AssessmentDefinition::new` leaves them empty. Do not redesign other inventories. |
| `ValidateIr` | `validation.rs` | **Add** CAPA graph checks. Keep IR-019 and existing incident/audit/remediation checks. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for owner, reviewer, closer, classifier. Do not invent `CapaOwner`. |
| `AuditFinding` / `NonconformityRef` | `audit.rs` | **Consume.** Optional `nonconformity_id` must resolve when CAPA inventory is non-empty. `kind = nonconformity` still does not start CAPA. |
| `Incident` / `PostIncidentReview` | `incident.rs` | **Consume.** `corrective_action_ids` stay `RemediationRef`. Incidents may *propose* a nonconformity via this slice’s API; they must not mint classification. |
| `Remediation` / `RemediationRef` | `remediation.rs` | **Cite**, do not fork. CAPA may link remediations as supporting work. Closing CAPA does not close remediations and vice versa. |
| `IsmsEvent` / `IsmsEventKind` | `event.rs` | Consume `NonconformityOpened`, `CorrectiveActionOverdue`, `ControlRegressed`. Do not add new event kinds in this slice unless a hole is proven; do not call `detect_isms_drift` from CAPA writers. |
| `IsmsSnapshot.nonconformities` / `.corrective_actions` | `weeping-angel-assurance::drift` | Remain `GovernanceRecord` views. Implement **may** add a documented adapter from IR inventories → snapshot bags. Empty stays a no-op. Do not rewrite detector rules. |
| `Effectiveness` / `ControlTestResult` | `weeping-angel-control-test` | Immutable observations. Effectiveness review **reads** them. |
| `VerificationPolicy` / `VerificationMode` | `remediation.rs` | **Reuse the type** (or an identical CAPA-local struct with the same serde names) for declared effectiveness criteria. Default `SustainedWindow` 14d / `minEffectiveResults = 2`. |
| `AssessmentRequests.nonconformities` | `assessment.rs` | Keep fail-closed request bit. **Do not auto-set `true`.** Compiling a projection that *requests* nonconformities still requires `supports_nonconformities`. Non-empty inventories validate even when the flag is false (same pattern as audits/remediations). |
| `FrameworkCapabilities.supports_nonconformities` | `weeping-angel-framework` | Default remains `false`. Requested ∧ ¬supported → `CapabilityViolation`. Flag does **not** construct CAPA objects. |
| `ComplianceNodeRef` | `crosswalk.rs` | **Not extended.** Spec-optional CAPA variants were not required for NC-001–NC-012. Do not infer “control is effective” from a closed CAPA. |
| Golden IR fixtures | `tests/fixtures/assurance-ir/v1/**` | Existing fixtures have no CAPA keys; default empty must keep decoding. |
| Neighbor suites | root `Cargo.toml` | Listed header targets stay GREEN. |
| Docs layout | ADR 0004 | Human SSOT is this file. Path is listed in `sdd_documentation_layout` `CANONICAL_SPECS`. |

Serde compatibility law:

- Existing assessment JSON **without** `nonconformities` / `correctiveActions` deserializes (`#[serde(default)]`).
- New JSON is camelCase, matching IR.
- Empty vectors / `None` skip-serialize.
- Schema remains `assurance-ir/v1`.

Network-free. No ISO annex numbers as CAPA classification. No Jira/ServiceNow objects as the canonical record.

---

## 3. Current behavior (baseline — characterization of pre-product HEAD)

Executable characterization lives in `sdd_nonconformity_capa_baseline` and is skip-superseded (`#[ignore = "superseded by sdd_nonconformity_capa_target"]`). This section is the **found case** of SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` + landed Prompts 15/16/19/21 before this engine.

### 3.1 No `Nonconformity` / `CorrectiveAction` product type

`crates/weeping-angel-assurance-ir/src/` has modules including `audit`, `incident`, `remediation`, `event` — **no** `capa` / `nonconformity` module. `id.rs` `typed_id!` list has **no** `NonconformityId` / `CorrectiveActionId`. `lib.rs` re-exports have no CAPA types.

There is no `Nonconformity`, `NonconformityStatus`, `CorrectiveAction`, `contain_nonconformity`, `record_root_cause`, `evaluate_capa_effectiveness`, or `close_nonconformity` API.

Workspace product `crates/` and `src/` have no CAPA engine (catalog TOML and compile flags only).

### 3.2 `AssessmentDefinition` has no CAPA inventories

[`assessment.rs`](../../crates/weeping-angel-assurance-ir/src/assessment.rs) fields today:

```text
requirements, controls, mappings, evidence_requirements, tests, requests,
implementations, scope, assets, identities, vendors, risks, exceptions,
processing_activities, incidents, risk_treatments, isms_context_id,
remediations, continuity_profiles, audit_programs, audits, audit_findings
```

There is no `nonconformities` or `corrective_actions` field. `AssessmentDefinition::new` does not initialize them.

### 3.3 Request / capability flags exist and stay fail-closed

[`assessment.rs`](../../crates/weeping-angel-assurance-ir/src/assessment.rs):

```text
AssessmentRequests {
  statement_of_applicability, control_applicability, privacy_processing,
  risk_treatment, manual_attestation, sampling, audit_program, nonconformities
}
```

All default `false`.

[`crates/weeping-angel-framework/src/lib.rs`](../../crates/weeping-angel-framework/src/lib.rs):

```text
FrameworkCapabilities { … supports_audit_program, supports_nonconformities }
```

`Default` is all `false`. `validate_capabilities` pairs `req.nonconformities` with `cap.supports_nonconformities` (needle `supports_nonconformities`). Requested ∧ ¬supported → `FrameworkCompileError::CapabilityViolation`. Success does not construct CAPA objects.

`sdd_assurance_runtime_target` ACT-006 / ACT-007 assert default `supports_nonconformities == false`. Do not flip that.

### 3.4 Audit seam is opaque and does not start CAPA

[`audit.rs`](../../crates/weeping-angel-assurance-ir/src/audit.rs):

```text
pub type NonconformityRef = String;

AuditFinding {
  …
  kind: observation | finding | nonconformity
  severity?: minor | major | opportunity
  nonconformityId?: NonconformityRef
}

Audit { nonconformityRefs: Vec<NonconformityRef>, … }
```

`record_finding` copies a present `nonconformity_id` onto `audit.nonconformity_refs`. It does **not** construct a `Nonconformity`, does not require `requests.nonconformities`, and does not copy `AuditFindingSeverity` into a CAPA classification.

`kind = nonconformity` is an auditor label, not a lifecycle start.

### 3.5 Incident “corrective actions” are Prompt 16 remediations

[`incident.rs`](../../crates/weeping-angel-assurance-ir/src/incident.rs):

```text
Incident.corrective_action_ids: Vec<RemediationRef>
PostIncidentReview.proposed_corrective_action_ids: Vec<RemediationRef>
```

Incident close does not close remediations. `closed_incidents_with_open_corrective_actions` lists closed incidents that still cite remediation refs. There is no `propose_nonconformity` on `Incident`.

### 3.6 Drift names the events; inventories are empty bags

[`event.rs`](../../crates/weeping-angel-assurance-ir/src/event.rs) already enumerates `IsmsEventKind::NonconformityOpened` and `CorrectiveActionOverdue`. `EventSubjectKind::Nonconformity` exists.

[`drift.rs`](../../crates/weeping-angel-assurance/src/drift.rs) `IsmsSnapshot` has:

```text
nonconformities: Vec<GovernanceRecord>     // default empty
corrective_actions: Vec<GovernanceRecord>  // default empty
```

`detect_isms_drift` emits `NonconformityOpened` when a new `GovernanceRecord` id appears in `next.nonconformities`, and `CorrectiveActionOverdue` when a corrective-action record is due or status hits `"overdue"`. Empty lists are no-ops. Nothing on HEAD fills those bags from IR.

`RemediationSourceKind` maps those event names via `From<&IsmsEventKind>` — still not a CAPA type.

### 3.7 Catalog corrective-action is attestation

[`catalog/canonical/v1/controls/governance.toml`](../../catalog/canonical/v1/controls/governance.toml):

| Id | Meaning today |
| --- | --- |
| `control.governance.corrective-action` | Hybrid; “nonconformities have recorded corrective-action **attestations**”; objective: do **not** treat a ticket id as proof of correction |
| `test.governance.corrective-action-recorded` | `manual-review` on `evidence.manual.attestation` |

Those tests stay **governance-only**. Baseline must assert the catalog ids still exist **and** that they are not this slice’s engine (no retarget).

### 3.8 Validation never walks CAPA

[`validation.rs`](../../crates/weeping-angel-assurance-ir/src/validation.rs) has no nonconformity / corrective-action integrity checks. A string `nonconformityId` on an audit finding is not resolved.

### 3.9 Crosswalk has no CAPA node

`ComplianceNodeRef` is `Requirement | Control | Test | EvidenceRequirement | Risk | Exception | Incident`. No `Nonconformity` / `CorrectiveAction` variant.

### 3.10 Dual-suite not registered (pre-product)

On the characterization SHA, root `Cargo.toml` had no `sdd_nonconformity_capa_{baseline,target}` and `tests/contracts/nonconformity_capa.*.rs` did not exist. This HEAD registers both suites; baseline is skip-superseded; this spec path is in `CANONICAL_SPECS`.

---

## 4. Landed behavior (target GREEN)

### 4.1 Product home

```text
weeping-angel-assurance-ir
  capa.rs            # Nonconformity, CorrectiveAction, states, source,
                     # classification, containment, RCA, criteria, close/reopen
  id.rs              # NonconformityId, CorrectiveActionId
  assessment.rs      # nonconformities + corrective_actions inventories
  validation.rs      # integrity + lifecycle guards
  lib.rs             # re-exports
  audit.rs           # NonconformityRef remains the finding seam (String)

weeping-angel-assurance
  capa.rs            # propose/open, contain, rca, plan, implement,
                     # evaluate_effectiveness, close, reopen, overdue queries
```

Network-free. No provider SDK types. External ticket keys never become `NonconformityId`.

### 4.2 Records

```text
Nonconformity {
  id: NonconformityId
  schemaVersion?: "assurance-ir/v1"     # if present, must equal ASSURANCE_IR_SCHEMA
  title: String                         # non-empty
  description: String                   # non-empty
  source: NonconformitySource
  classification?: NonconformityClassification   # unset until classify()
  classificationRationale?: String
  classifiedBy?: PrincipalRef
  classifiedAt?: DateTime<Utc>
  severity?: NonconformitySeverity      # operational impact; not a silent major/minor
  status: NonconformityStatus
  owner: PrincipalRef
  detectedAt: DateTime<Utc>
  openedAt: DateTime<Utc>
  openedBy: PrincipalRef
  affected: NonconformityScope          # controls, requirements, assets, subjects, processes
  containment: Vec<ContainmentAction>
  rootCause?: RootCauseAnalysis
  correctiveActionIds: Vec<CorrectiveActionId>
  remediationRefs: Vec<RemediationRef>  # Prompt 16 supporting work; optional
  effectiveness?: EffectivenessReview
  closure?: ClosureDecision
  supersededBy?: NonconformityId
  supersedes?: NonconformityId
  version: u32                          # default 1
  history: Vec<NonconformityEvent>
}

NonconformitySource {
  kind: AuditFinding | Incident | ControlRegression | Manual
  auditFindingId?: AuditFindingId
  auditId?: AuditId
  incidentId?: IncidentId
  eventRef?: EventRef                   # typically ControlRegressed
  controlIds: Vec<ControlId>
}

NonconformityClassification = Major | Minor | Opportunity
NonconformitySeverity = Informational | Notable | Material | Critical
  # or reuse a local enum; do NOT import scanner Severity

NonconformityStatus =
    Open
  | Contained
  | RootCauseIdentified
  | CorrectiveActionPlanned
  | Implemented
  | EffectivenessReview
  | Closed
  | Cancelled
  | Superseded

NonconformityScope {
  controlIds: Vec<ControlId>
  requirementIds: Vec<RequirementId>
  assetIds: Vec<AssetId>
  processingActivityIds: Vec<ProcessingActivityId>
  population: Vec<SubjectSelector>
}

ContainmentAction {
  id: String                            # stable
  description: String                   # non-empty
  performedAt: DateTime<Utc>
  performedBy: PrincipalRef
  evidenceRefs: Vec<String>             # envelope digests or requirement ids
}

RootCauseAnalysis {
  method: String                        # e.g. "5-why" | "fishbone" | "other" — documentary
  statement: String                     # non-empty
  recordedAt: DateTime<Utc>
  recordedBy: PrincipalRef
  evidenceRefs: Vec<String>
}

CorrectiveAction {
  id: CorrectiveActionId
  nonconformityId: NonconformityId      # required
  kind: Corrective | Preventive
  title: String
  description: String
  owner: PrincipalRef
  targetDate: DateTime<Utc>             # due / planned complete
  implementedAt?: DateTime<Utc>
  implementationEvidence: Vec<String>
  effectivenessCriteria: EffectivenessCriteria
  reviewPeriod: ReviewPeriod            # required window
  reviewer: PrincipalRef                # ≠ owner when independentReview required
  status: CorrectiveActionStatus
  verificationState: VerificationState  # reuse remediation semantics
  remediationRefs: Vec<RemediationRef>
  history: Vec<CorrectiveActionEvent>
}

CorrectiveActionStatus =
    Planned | InProgress | Implemented | EffectivenessReview
  | Verified | FailedReview | Cancelled | Superseded

EffectivenessCriteria {
  # Reuses VerificationPolicy serde names (VerificationMode + window seconds):
  mode: SingleGreenPermitted | SustainedWindow | IndependentReviewRequired
  window?: u64                          # seconds; required when SustainedWindow (default 14d)
  minEffectiveResults: u32              # default 2 for SustainedWindow
  independentVerifier: bool
  statement: String                     # declared criteria in prose (required, non-empty)
  controlIds: Vec<ControlId>            # tests read for these ids
}

ReviewPeriod {
  start: DateTime<Utc>
  end: DateTime<Utc>                    # half-open preferred; end > start
}

EffectivenessReview {
  period: ReviewPeriod
  reviewer: PrincipalRef
  status: NotStarted | InWindow | Satisfied | Failed | Rejected
  resultDigests: Vec<String>
  note?: String
}

ClosureDecision {
  closedBy: PrincipalRef
  closedAt: DateTime<Utc>
  rationale: String                     # non-empty
  outcome: ClosedEffective | Cancelled | Superseded
}

NonconformityEvent {
  version: u32
  at: DateTime<Utc>
  principal: PrincipalRef?
  kind: Opened | Classified | Contained | RootCauseRecorded
      | ActionPlanned | Implemented | ReviewStarted | ReviewFailed
      | Closed | Cancelled | Superseded | Reopened
      | FieldsRevised
}
```

Public constructors:

```text
Nonconformity::open(id, title, description, source, owner, detected_at, opened_at, opened_by)
  → status = Open; classification = None; history seeded Opened

propose_from_audit_finding(...)
propose_from_incident(...)
propose_from_control_regression(...)   # requires IsmsEventKind::ControlRegressed
  → same as open; source filled; NEVER copies AuditFindingSeverity / IncidentSeverity
     into NonconformityClassification
```

`propose_*` may be aliases of `open` with source helpers. There is no `From<AuditFinding> for Nonconformity`, no `From<Incident> for Nonconformity`, and no collector insert.

### 4.3 Classification decision boundary

Major / minor / opportunity is a **human decision**:

```text
classify(nc, classification, rationale, principal, at)
```

Rules:

1. Classification starts **unset**. Unclassified records may stay `Open` / `Contained` for triage but **cannot** reach `CorrectiveActionPlanned` or later (fail closed).
2. `AuditFinding.severity`, `Incident.severity`, event severity, and scanner `Severity` **must not** be copied into `NonconformityClassification`.
3. `kind = nonconformity` on an audit finding does **not** classify and does **not** open CAPA.
4. Rationale must be non-empty. Principal required.
5. Re-classification appends history; it does not rewrite the original `Classified` event.

### 4.4 State machine (fail closed)

```text
Open → Contained → RootCauseIdentified → CorrectiveActionPlanned
     → Implemented → EffectivenessReview → Closed

Open / Contained / RootCauseIdentified / CorrectiveActionPlanned / Implemented
     / EffectivenessReview
        → Cancelled          (accountable rationale)
        → Superseded         (rationale + successor NonconformityId)

Closed → Open                (reopen; principal + rationale)

Cancelled / Superseded → ∅   (terminal)
```

Normative table. Any other pair is `CapaError::InvalidTransition`. Library paths must not panic.

| From | Allowed targets |
| --- | --- |
| `Open` | `Contained`, `Cancelled`, `Superseded` |
| `Contained` | `RootCauseIdentified`, `Cancelled`, `Superseded` |
| `RootCauseIdentified` | `CorrectiveActionPlanned`, `Cancelled`, `Superseded` |
| `CorrectiveActionPlanned` | `Implemented`, `Cancelled`, `Superseded` |
| `Implemented` | `EffectivenessReview`, `CorrectiveActionPlanned` (failed/incomplete evidence), `Cancelled`, `Superseded` |
| `EffectivenessReview` | `Closed`, `Implemented` / `CorrectiveActionPlanned` (failed review), `Cancelled`, `Superseded` |
| `Closed` | `Open` (reopen only) |
| `Cancelled` | ∅ |
| `Superseded` | ∅ |

Guards:

1. `Open → Contained` requires ≥1 `ContainmentAction` with non-empty description, principal, time, and either evidence refs **or** an explicit containment statement recorded on the action.
2. `Contained → RootCauseIdentified` requires `rootCause.statement` non-empty + `recordedBy` + `recordedAt`. **Missing RCA fails.** There is no AI-generated RCA API.
3. `RootCauseIdentified → CorrectiveActionPlanned` requires classification set **and** ≥1 `CorrectiveAction` with `targetDate`, non-empty `effectivenessCriteria.statement`, `reviewPeriod` (`end > start`), and `reviewer`.
4. `CorrectiveActionPlanned → Implemented` requires every non-cancelled action to have `implementedAt` + implementation evidence meeting declared cardinality (default ≥1 digest/ref).
5. `Implemented → EffectivenessReview` starts/records the review period clock (injected `as_of`, never `Utc::now()` inside the library).
6. `EffectivenessReview → Closed` requires `effectiveness.status == Satisfied` **and** an explicit `ClosureDecision` (principal + time + non-empty rationale). **A single green `Effectiveness::Effective` does not satisfy `SustainedWindow`.** `SingleGreenPermitted` may set `Satisfied` on one green **inside the declared review period**; `close` is still explicit.
7. Failed review (`verificationState.status == Failed` or review `Failed`) **forbids** `Closed`. Legal repair is return to `Implemented` or `CorrectiveActionPlanned` with a `ReviewFailed` history event.
8. `* → Cancelled` / `* → Superseded` require non-empty rationale. Supersession requires `supersededBy` ≠ self and a successor that cites `supersedes`.
9. `Closed → Open` (reopen) requires principal + rationale; increments `version`; appends `Reopened`; clears `closure` **by recording a new state**, not by deleting history. Prior closure remains in history.
10. Terminal `Cancelled` / `Superseded` reject field mutation (`CapaError::ImmutableClosure`). `Closed` rejects mutation other than `reopen`.

```text
NonconformityStatus::can_transition(from, to) -> bool
Nonconformity::transition(to, principal, at) -> Result<(), CapaError>
```

### 4.5 Effectiveness review — one green is not closure

Reuse remediation verification semantics ([`remediation-engine.md`](remediation-engine.md) §4.8) against `EffectivenessCriteria.controlIds`:

- Default `SustainedWindow`: need ≥ `minEffectiveResults` (default 2) `Effectiveness::Effective` results whose `checked_at` span is ≥ `window` (seconds), with **no** intervening `Ineffective` / `InsufficientEvidence` / `StaleEvidence`, **and** the span must lie within `reviewPeriod` (or start at `Implemented` and cover `window`).
- Two greens 3 days apart on a 14-day window ⇒ **not** Satisfied.
- `IndependentReviewRequired` (or `independentVerifier`) requires reviewer `PrincipalRef` ≠ action owner / NC owner.
- `ControlRecovered` / a single recovered control test **never** auto-closes CAPA.
- `evaluate_capa_effectiveness(nc, actions, results, as_of, reviewer) → EffectivenessReview`.
- Closing without `Satisfied` is `CapaError::EffectivenessNotSatisfied`.

Do not put `Effectiveness` on the nonconformity as a synonym of “CAPA done.”

### 4.6 Overdue actions

```text
corrective_actions_overdue(assessment, as_of) -> Vec<CorrectiveActionId>
```

An action is overdue when `targetDate <= as_of` and status is not `Implemented` / `Verified` / `Cancelled` / `Superseded` / parent NC is not `Closed`/`Cancelled`/`Superseded`.

Overdue is a **query fact**. It does not auto-transition, auto-reclassify, or auto-cancel.

Clocked validate **may** emit a diagnostic but must not mutate records. Drift `CorrectiveActionOverdue` fires only when a caller assembles `IsmsSnapshot.corrective_actions` (adapter allowed; detector rules stay).

### 4.7 Linkage

#### Audit

- `propose_from_audit_finding` copies `AuditFindingId` / `AuditId` / cited control and requirement ids into source + scope.
- After open, `AuditFinding.nonconformity_id` **may** be set by the caller to `NonconformityId.as_str()` (or typed alias). This slice **must not** rewrite `record_finding` to auto-open CAPA.
- When `assessment.nonconformities` is non-empty, dangling `AuditFinding.nonconformity_id` / `Audit.nonconformity_refs` fail `validate_assessment_ir`.
- When the inventory is empty, existing opaque strings remain valid (Prompt 21 fixtures stay GREEN).

#### Incident

- `propose_from_incident` copies `IncidentId` and may copy affected assets/processes/populations/control-failure control ids into scope. It does **not** copy incident severity into classification.
- PIR `proposed_corrective_action_ids` remain `RemediationRef`. This slice may read them as **candidates to link** (`remediationRefs`) after a human accepts; it must not treat them as `CorrectiveActionId`.
- Incident close still does not close CAPA. CAPA close does not close the incident or its remediations.

#### Control regression

- `propose_from_control_regression` requires an `IsmsEvent` (or kind+id) with `kind = ControlRegressed`. Source stores `EventRef`. Linking does not change control `Effectiveness`.

### 4.8 Inventory and validation

`AssessmentDefinition` gains:

```text
nonconformities: Vec<Nonconformity>       // serde default empty
corrective_actions: Vec<CorrectiveAction> // serde default empty
```

Clockless `validate_assessment_ir` (when either inventory is non-empty):

| Check | Rule |
| --- | --- |
| Duplicate `NonconformityId` / `CorrectiveActionId` | error |
| `corrective_actions[].nonconformityId` | ∈ nonconformities |
| `nonconformities[].correctiveActionIds` | ∈ corrective_actions and point back |
| Source `auditFindingId` / `auditId` | resolve when audit inventories non-empty |
| Source `incidentId` | resolve when `incidents` non-empty |
| `remediationRefs` | resolve when `remediations` non-empty |
| Scope control/requirement/asset/process ids | resolve when those inventories non-empty |
| Illegal history/status pairs | error |
| `RootCauseIdentified` and later (except Cancelled/Superseded) | RCA present |
| `CorrectiveActionPlanned` and later | classification set + ≥1 action (unless Cancelled/Superseded) |
| `Closed` | closure decision + effectiveness Satisfied (outcome `ClosedEffective`) |
| `Cancelled` / `Superseded` | rationale present |
| Owner / opener / reviewer | `PrincipalRef` non-empty |

Clocked helpers:

```text
validate_capa_at(assessment, as_of)
  — overdue is queryable; Closed+unsatisfied is still an error
  — review period end < start is an error
```

Do **not** require `requests.nonconformities == true` to validate a present inventory.

### 4.9 Immutable closure and history

- `close` records `ClosureDecision` and appends `Closed`.
- `revise` / `transition` / `classify` / `contain` on `Cancelled`/`Superseded` return `CapaError::ImmutableClosure`.
- `Closed` allows only `reopen`.
- Past history events are not rewritten in place.
- Frozen closed records have a stable `canonical_digest` until reopen appends.

### 4.10 Query helpers (assurance crate)

Land in `weeping-angel-assurance::capa` (names flexible):

```text
open_nonconformities(assessment)
overdue_corrective_actions(assessment, as_of)
failed_effectiveness_reviews(assessment)
nonconformities_for_audit(assessment, audit_id)
nonconformities_for_incident(assessment, incident_id)
reopened_nonconformities(assessment)
closed_nonconformities(assessment)
```

These prepare audit / management-review consumption (Prompts 21/23). They do not generate minutes or audit conclusions.

### 4.11 Compile flags stay fail-closed

- Default `supports_nonconformities = false`, `requests.nonconformities = false`.
- Do **not** auto-enable either because a `Nonconformity` exists.
- Requested ∧ ¬supported remains `CapabilityViolation`.
- Framework projection of CAPA is out of scope; inventories are IR regardless of the flag (same as remediations/incidents).

### 4.12 Serialization and digest

- Schema stays `assurance-ir/v1`.
- `canonical_digest` / `typed_canonical_digest("Nonconformity" | "CorrectiveAction", …)` SHA-256.
- Maps/sets use `BTreeMap` / `BTreeSet`.
- `version` default 1.

### 4.13 Public-contract pointer

[`assurance-runtime.md`](assurance-runtime.md) carries a short CAPA section matching landed APIs. Do not duplicate this spec. Do not claim ISO 10.2 is satisfied because a row exists.

Not landed (spec-optional): `ComplianceNodeRef::{Nonconformity,CorrectiveAction}`; automatic `IsmsSnapshot` fill from IR inventories.

---

## 5. Dual-suite / SDD protocol

Follow [ADR 0004](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

Registered in root `Cargo.toml` (I3), **same commit** as the `.rs` files:

```toml
[[test]]
name = "sdd_nonconformity_capa_baseline"
path = "tests/contracts/nonconformity_capa.baseline.rs"

[[test]]
name = "sdd_nonconformity_capa_target"
path = "tests/contracts/nonconformity_capa.target.rs"
```

| Suite | Pre-product HEAD | This HEAD |
| --- | --- | --- |
| Baseline | **PASS** (characterizes absence + seams) | skip-superseded `#[ignore = "superseded by sdd_nonconformity_capa_target"]` |
| Target | **FAIL** (missing types / lifecycle) | **GREEN** (NC-001–NC-012) |

Protocol: write tests first → **RED** target (must fail on the found case, not unrelated compile noise) → implement → **GREEN**. Baseline stays GREEN until superseded.

Each target test title: `NC-00N: <exact subject>` matching §6.

Clock injection is mandatory (no wall-clock seeds). Prefer tests that construct IR + call engine functions.

Do not add `tests/sdd/` or generated dumps under `docs/sdd/`. `docs/sdd/nonconformity-capa.md` is a stub pointer only.

---

## 6. Acceptance criteria (testable)

Target suite must encode at least the Prompt 22 found cases:

- **NC-001 Complete CAPA.** Open → contain → RCA → classify → plan action (criteria + review period + reviewer) → implement with evidence → effectiveness Satisfied over the declared window → explicit close. Record persists on `AssessmentDefinition`; `validate_assessment_ir` Ok; history contains each step; `canonical_digest` stable.
- **NC-002 Missing root cause.** `Contained → RootCauseIdentified` (or any later state) without `rootCause.statement` fails. Record stays `Contained`.
- **NC-003 Overdue action.** `targetDate < as_of`, action not implemented/verified/cancelled ⇒ `overdue_corrective_actions` includes it. Status does not auto-jump. Classification unchanged.
- **NC-004 Failed effectiveness review.** After implement, a fail-closed `Effectiveness` (`Ineffective` / `InsufficientEvidence` / `StaleEvidence`) during the review window sets review `Failed` and **forbids** `Closed`. Return to `Implemented` or `CorrectiveActionPlanned` is legal.
- **NC-005 Re-opened nonconformity.** `Closed → Open` with principal + rationale succeeds; history retains the prior `Closed` event; `version` increments; a new close still requires a fresh Satisfied review.
- **NC-006 Sustained verification window.** Default `SustainedWindow` 14d / min 2 greens: one `Effective` is not Satisfied; two `Effective` results 3 days apart are not Satisfied; two `Effective` results whose `checked_at` span ≥ 14d with no intervening fail **and** explicit `close` succeeds.
- **NC-007 Audit linkage.** `propose_from_audit_finding` binds `AuditFindingId` / `AuditId`; does not start from `kind = nonconformity` alone; does not copy `AuditFindingSeverity` into classification. When CAPA inventory is non-empty, dangling `nonconformityId` fails; empty inventory keeps opaque Prompt 21 strings valid.
- **NC-008 Incident linkage.** `propose_from_incident` binds `IncidentId`; does not retarget `Incident.corrective_action_ids` away from `RemediationRef`; incident close does not close CAPA; PIR proposed remediations are not `CorrectiveActionId`.
- **NC-009 Immutable closure.** Mutating a `Closed`/`Cancelled`/`Superseded` record (title, RCA, actions, closure fields) returns `ImmutableClosure`. History is append-only. `Cancelled`/`Superseded` require rationale; `Superseded` requires a successor id.
- **NC-010 No silent classification.** Constructing from finding/incident/`ControlRegressed` leaves `classification = None`. `classify` requires principal + non-empty rationale. Unclassified records cannot reach `CorrectiveActionPlanned`.
- **NC-011 Compile flags / catalog fence.** Default `requests.nonconformities` and `supports_nonconformities` remain `false`. Presence of CAPA inventories does not flip them. `control.governance.corrective-action` / `test.governance.corrective-action-recorded` still exist and are not this engine. `ASSURANCE_IR_SCHEMA` unchanged.
- **NC-012 Dual-suite registration.** `sdd_nonconformity_capa_{baseline,target}` listed in root `Cargo.toml`; files live under `tests/contracts/`; this spec path is in `CANONICAL_SPECS`.

Baseline suite must encode §3: no `Nonconformity`/`CorrectiveAction` types; no assessment CAPA inventories; flags default false; `NonconformityRef = String`; audit `kind = nonconformity` does not start CAPA; incident corrective actions are `RemediationRef`; drift bags empty; catalog attestation ids unchanged.

Neighbor targets listed in the header stay GREEN.

---

## 7. Non-goals / out of scope

- Generic issue tracker, kanban UI, assignment inbox, or notification transport
- AI / LLM root-cause engine or auto-written RCA
- Retargeting `control.governance.corrective-action` or other catalog/ISO pack IDs
- Auto-enabling `requests.nonconformities` / `supports_nonconformities`
- Replacing Prompt 16 `Remediation` or retargeting `Incident.corrective_action_ids` to CAPA ids
- Inventing a parallel `Incident` / `AuditFinding` IR
- Bumping `ASSURANCE_IR_SCHEMA` or forking `assurance-ir/v2`
- Rewriting `detect_isms_drift` rules or requiring non-empty drift inventories
- Promoting scanner `Finding` into CAPA
- Ticket-system HTTP clients (Jira/Linear/GitHub)
- Management-review minutes (Prompt 23) or certification packs (Prompt 24)
- Claiming ISO 27001 10.1/10.2 (or any clause) is satisfied because a CAPA row exists
- New workspace crate; `tests/sdd/`

---

## 8. Risks

- **Silent major/minor.** Copying `AuditFindingSeverity` into CAPA classification collapses the decision boundary. Mitigation: NC-010; `classify` is the only writer.
- **One-green close.** Operators treat `Effectiveness::Effective` as CAPA done. Mitigation: default `SustainedWindow`; NC-006; explicit `close`.
- **Remediation / CAPA identity collision.** Readers treat `Incident.correctiveActionIds` as this slice. Mitigation: keep `RemediationRef`; NC-008; ADR decision 3.
- **Audit fixture breakage.** Tightening `NonconformityRef` or requiring inventory resolve would fail `sdd_internal_audit_target`. Mitigation: resolve only when CAPA inventory is non-empty; keep opaque strings.
- **Flag flip.** Default `supports_nonconformities = true` breaks assurance-runtime ACT-007. Mitigation: NC-011; do not auto-enable.
- **Catalog hijack.** Storing operational CAPA only as `evidence.manual.attestation`. Mitigation: catalog suite stays GREEN and unrewritten.
- **Drift double-meaning.** Populating `IsmsSnapshot.nonconformities` incorrectly emits `NonconformityOpened` for every serialize. Mitigation: adapter is explicit; detector unchanged; empty remains no-op.
- **History vs digest.** Append-only vectors change canonical bytes after reopen. Tests pin pre-reopen digests; do not log unbounded events.
- **Exhaustive struct literals.** Adding inventories to `AssessmentDefinition` breaks in-tree literals. Mitigation: `new()` + serde default; fix literals in this slice only.
- **Prompt 21 still in flight.** If audit types regress, consume whatever `AuditFinding` / `NonconformityRef` exists; do not fork.

---

## 9. Crate homes and files

| Path | Role |
| --- | --- |
| `crates/weeping-angel-assurance-ir/src/capa.rs` | Types + transition guards |
| `crates/weeping-angel-assurance-ir/src/id.rs` | `NonconformityId`, `CorrectiveActionId` |
| `crates/weeping-angel-assurance-ir/src/assessment.rs` | Additive inventories |
| `crates/weeping-angel-assurance-ir/src/validation.rs` | Integrity + lifecycle |
| `crates/weeping-angel-assurance-ir/src/lib.rs` | Re-exports |
| `crates/weeping-angel-assurance/src/capa.rs` | Engine + queries |
| `tests/contracts/nonconformity_capa.baseline.rs` | Absence characterization (skip-superseded) |
| `tests/contracts/nonconformity_capa.target.rs` | Normative NC-001–NC-012 (GREEN) |
| `Cargo.toml` | `[[test]]` `sdd_nonconformity_capa_{baseline,target}` |
| `tests/contracts/documentation_layout.rs` | `CANONICAL_SPECS` includes this file |
| `docs/specs/assurance-runtime.md` | Pointer section |
| `docs/adr/0028-nonconformity-capa.md` | Decision (Accepted) |
| `docs/sdd/nonconformity-capa.md` | Stub pointer only |

Do not add a crate. Do not edit collision-fenced paths in §0.

---

## 10. Definition of done

Weeping Angel has a canonical nonconformity/CAPA record in `assurance-ir/v1` that:

- is created only by explicit propose/open;
- refuses silent major/minor classification;
- requires containment and RCA before planning actions;
- tracks corrective/preventive actions with target dates and implementation evidence;
- verifies effectiveness over a declared period (one green test is not enough);
- closes only with an accountable decision;
- allows cancel/supersede/reopen with rationale and append-only history;
- links to audit findings and incidents without forking those IRs;
- leaves compile flags, catalog TOML, and `ASSURANCE_IR_SCHEMA` unchanged.

Prompt 22 mission is complete: `sdd_nonconformity_capa_target` is GREEN, baseline is skip-superseded, neighbor suites listed in the header remain GREEN. Workspace verify: `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
