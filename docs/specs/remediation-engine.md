# SDD: Remediation Workflow Engine (ISMS v1)

| Field | Value |
| --- | --- |
| Status | **Implemented** — IR `Remediation` + assurance engine landed; `sdd_remediation_engine_target` GREEN; baseline skip-superseded; ADR Accepted |
| Program | Operational ISMS v1 — Prompt 16 remediation engine |
| Prompt | [`docs/prompts/operational-isms-v1/16-remediation-engine.md`](../prompts/operational-isms-v1/16-remediation-engine.md) |
| Slice | Canonical `Remediation` records that connect assurance failures and risk treatment to accountable work, with an auditable lifecycle, verification policy, SLA clock, and adapter-only external ticket refs |
| Dual-suite | `sdd_remediation_engine_baseline` · `sdd_remediation_engine_target` (`tests/contracts/remediation_engine.{baseline,target}.rs`) — **not auto-discovered** (I3); listed in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0031-remediation-engine.md`](../adr/0031-remediation-engine.md) (0003-* sibling; 0004 is documentation architecture). Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) — Remediation engine |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Prompt 15 event/drift | Landed [`docs/specs/isms-events-drift.md`](isms-events-drift.md). Consume `IsmsEvent` / `EventId` / `IsmsEventKind` via `From`; do not invent a parallel event bus |
| Neighbors (consume, do not fork) | risk register [`risk-register.md`](risk-register.md); risk treatment [`risk-treatment.md`](risk-treatment.md) (`treatmentActionIds` resolve against `risk_treatments`); control-implementation registry [`control-implementation-registry.md`](control-implementation-registry.md); temporal assurance [`temporal-assurance.md`](temporal-assurance.md); incident governance [`incident-governance.md`](incident-governance.md) (`correctiveActionIds` = `RemediationRef`; incident close does **not** close remediations) |
| Collision fence | Catalog TOML, ISO packs, GitHub collector, scanner workbench, existing `sdd_*` suites except additive `Cargo.toml` / `documentation_layout.rs` registration |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `canonical_digest` / `typed_canonical_digest` SHA-256 of serde JSON (struct field order + `BTreeMap` / `BTreeSet`, `canon/v1`) |
| JSON | `#[serde(rename_all = "camelCase")]` |
| Identity | `typed_id!(RemediationId)` + `validate_stable_id`; **no** random v4 in persisted identity |
| Workspace verify | `cargo test --test sdd_remediation_engine_target`; `cargo test --test sdd_documentation_layout`; keep `sdd_compliance_ir_target`, `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_assessment_lineage_target` GREEN; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` when practical |

This document is the durable human SSOT for Operational ISMS v1 Prompt 16. It owns the **canonical remediation record**, **fail-closed state machine**, **source binding to the Prompt 15 event contract**, **verification policy (including sustained-effectiveness windows)**, **SLA overdue query**, **adapter-only external ticket references**, **waiver/acceptance gating**, and **immutable closure history**.

It does **not** own ISMS event emission or snapshot drift (Prompt 15), risk-register field expansion (Prompt 06), treatment strategy/state machines (Prompt 08), control-implementation registry expansion (Prompt 10), evidence validity windows (Prompt 14), CAPA/nonconformity (Prompt 22), kanban UI, assignment notifications, or Jira/Linear/GitHub ticket-system **clients**.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

A remediation is a **management-system work record over that graph**. A control-test result is an **immutable observation**. A scanner workbench patch request is **not** an ISMS remediation. An external ticket is an **adapter reference**, never the canonical identity.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

---

## 0. Collision fence (concurrent SDD)

This slice may add remediation IR + a library engine in **existing crates**. It must not fork neighboring ISMS slices or invent a second event bus.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps | Catalog / ISO remap |
| `crates/weeping-angel-collector/src/github/**`, `tests/contracts/github_collector.*` | GitHub collector |
| `src/workbench/remediation.rs` `RemediationRequest` / `RemediationResult` | Scanner workbench (code-patch generator; **different type**) |
| `src/finding.rs` scanner `Finding` | Recon/scanner product; not IR |
| Prompt 15 event emission, snapshot diff, `detect_isms_drift`, `IsmsEvent` module | [`isms-events-drift.md`](isms-events-drift.md) — **consume** `EventId` / `IsmsEventKind::ControlRegressed`; do not reimplement drift |
| [`docs/specs/risk-register.md`](risk-register.md) / operational `Risk` expansion | Prompt 06 — consume `Risk` / `RiskId` as they exist |
| [`docs/specs/risk-treatment.md`](risk-treatment.md) `RiskTreatmentDecision` / `TreatmentAction` | Prompt 08 — **link** via `TreatmentActionId` / opaque `RemediationRef` identity; do not implement Mitigate/Accept/Avoid/Transfer here |
| [`docs/specs/control-implementation-registry.md`](control-implementation-registry.md) additive `ControlImplementation` fields | Prompt 10 — consume existing `ControlImplementation` / `ControlImplementationId` |
| [`docs/specs/temporal-assurance.md`](temporal-assurance.md) `valid_from` / revocation | Prompt 14 — consume `ControlTestResult` clocks as they exist |
| Prompt 22 `Nonconformity` / CAPA | Landed neighbor; remediations may be *cited* (`RemediationRef`), not owned here |
| Notification transport, Slack, email, assignment inbox | Non-goal |
| Jira / Linear / GitHub Issues HTTP clients, webhooks, OAuth | Non-goal — **refs only** |
| `tests/sdd/` | ADR 0004 forbids this path |
| Existing `sdd_*` suite bodies except additive `Cargo.toml` / `documentation_layout.rs` | Stay GREEN |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| `Remediation`, states, source, SLA, verification policy, planned actions, waiver binding, history | `crates/weeping-angel-assurance-ir/src/remediation.rs` (name flexible) |
| `RemediationId`, `RemediationActionId`, `SlaPolicyId` (and reuse `TreatmentActionId` if Prompt 08 already added it) | `crates/weeping-angel-assurance-ir/src/id.rs` `typed_id!` |
| `AssessmentDefinition.remediations` | `assessment.rs` — additive `Vec` + `serde(default)` |
| Graph integrity (dangling refs, duplicate ids, illegal history pairs, closed-record mutation) | `validation.rs` (`validate_assessment_ir` + clocked helper) |
| Re-exports | `lib.rs` |
| Create-from-source, verification evaluation, SLA overdue, waiver clock, close | `weeping-angel-assurance` module `remediation` (library; no HTTP) |
| `ControlTestResult` / `Effectiveness` | **consume** `weeping-angel-control-test`; do not put effectiveness on `Remediation` as a synonym of test green |
| Evidence envelopes | conclusion-free; store **refs** (requirement ids / envelope digests), not pass/fail on the envelope |

Tiny allowed adjustments at **implement**: new `typed_id!` aliases; serde defaults / `skip_serializing_if`; transition APIs; validation messages; re-exports; optional golden fixture `tests/fixtures/assurance-ir/v1/remediation.json`. Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** redesign `AssessmentDefinition` core inventories. Do **not** expand `Risk`, `Exception`, or `ControlImplementation` in this slice.

Risk treatment stores opaque `RemediationRef` on `TreatmentAction.remediationRefs`. This slice **owns** `struct Remediation`. `treatmentActionIds` resolve against `assessment.risk_treatments` (or an injected action-id set in `validate_remediation_inventory`). A present treatment-action id that is not in that inventory is a dangling ref (fail closed).

---

## 1. Problem / user-visible goal

Operators can observe that a control test failed or a risk needs treatment, but Weeping Angel has **no canonical, auditable work record** that tracks the gap from cause through planned actions, evidence-of-fix, independent verification, and closure.

On characterization SHA `6e31bf1a…`:

- there is no `Remediation` / `RemediationId` in `weeping-angel-assurance-ir`;
- `AssessmentDefinition` has no `remediations` inventory;
- Prompt 15 (`ControlRegressed` and related events) is **not on disk** — there is no event module to consume, and this slice must not invent a second bus;
- `Risk` is still `{id, title, description, status}` (*“Not a risk engine.”*);
- `Exception` exists (control exception, not a remediation waiver workflow);
- scanner `src/workbench/remediation.rs` `RemediationRequest` generates unified diffs for findings — a **different type**;
- there is no Jira/Linear/GitHub Issues client;
- ISO MVP Phase 40 named a future `RemediationItem` linked from failed tests; it is not an IR type.

That means a single green retest can be mistaken for closure, an expired exception can keep looking like a waiver, an external ticket can be mistaken for the system of record, and closure has no immutable history.

**User-visible goal (mission):** every material assurance/risk gap can be tracked from cause through corrective evidence and independent verification with an auditable lifecycle — **without** turning Weeping Angel into a generic project-management system.

```text
Prompt 15 event (ControlRegressed / …)  or  TreatmentAction
  → Remediation { source, affected risks/controls/subjects, owner, priority, severity,
                  sla, dueAt, state, externalTickets[], plannedActions[],
                  evidenceOfFix, verificationPolicy, verificationState,
                  waiver?, closedBy/closedAt, history }
       → work (InProgress) → AwaitingVerification
            → Verified (policy satisfied) → Closed (principal + time)
            → or AcceptedWaived only while Exception / risk acceptance is in force
```

Examples the engine **must** distinguish:

```text
ControlRegressed event observed
  → create Remediation sourced to that event id; ControlTestResult stays immutable

TreatmentAction ta:patch-branch-protection cites Remediation rem:bp-1
  → bidirectional link; canonical id remains rem:bp-1

dueAt < as_of, state = InProgress
  → sla_overdue == true

External ticket JIRA ENG-441 attached
  → adapter ref only; RemediationId is still rem:bp-1; no HTTP client

One ControlTestResult Effective
  → does NOT close unless verificationPolicy.mode explicitly permits single-green

Sustained window 14d, two greens 3 days apart
  → not verified; window not satisfied

Exception Approved, expiresAt < as_of, state was AcceptedWaived
  → cannot remain AcceptedWaived; treatment/remediation required again

Closed record
  → closure principal/time retained; rewrite of closed fields fails; history append-only
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| Prompt 15 events | Spec [`isms-events-drift.md`](isms-events-drift.md); **landed** (`weeping-angel-assurance-ir::event`) | Bind `RemediationSource.kind` to `IsmsEventKind` names. Store `eventId` as the same stable id `EventId` uses. Do **not** emit events or call `detect_isms_drift`. Landed `From<&IsmsEvent>` / `From<&IsmsEventKind>` must not mint a second identity. |
| `Risk` / `RiskStatus` | `risk.rs` | **Do not fork.** Cite `RiskId`. Four-field stub is enough for linkage. |
| `Exception` / `ExceptionStatus` | `exception.rs` | **Reuse** for waiver gating. Do not add transition functions here. `AcceptedWaived` requires `Approved` + unexpired `expires_at` (or explicit documented non-expiring only if `expires_at` is absent **and** a separate `RiskAcceptance` is in force — default fail-closed: missing expiry on an exception-backed waiver is **not** in force unless Prompt 08 acceptance is bound). |
| `ControlImplementation` | `implementation.rs` | **Reference** `ControlImplementationId` / `ControlId` only. Do not add Prompt 10 fields. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for owner, verifier, closer. Do not invent `RemediationOwner`. |
| `SubjectSelector` | `subject.rs` | Affected subjects. |
| `ControlTestResult` / `Effectiveness` | `weeping-angel-control-test` | Immutable. Verification **reads** results; it never mutates them and never treats one `Effective` as auto-close unless policy says so. |
| `PlannedControlTest` / `TestFailureSeverity` | `test.rs` | Optional severity default from the failed test; still stored on the remediation. |
| `canonical_digest` / `typed_canonical_digest` | `digest.rs` | Reuse SHA-256. No second hasher. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Do not fork. |
| `AssessmentDefinition` | `assessment.rs` | Additive `remediations: Vec<Remediation>` with `serde(default)`. `AssessmentDefinition::new` stays valid (empty vec). |
| `ValidateIr` | `validation.rs` | Keep IR-019. **Add** remediation integrity. |
| `typed_id!` / `validate_stable_id` | `id.rs` | New ids. UUID-v4 shaped strings remain `IdError::InvalidCharacter`. |
| Prompt 08 `TreatmentActionId` / `RemediationRef` | landed | Linkage: remediation stores `treatmentActionIds: [TreatmentActionId]` resolved against `assessment.risk_treatments` (or an injected set). Prompt 08 stores opaque `RemediationRef` equal to `RemediationId.as_str()`. Do not duplicate `TreatmentActionId`. |
| Prompt 06/10/14 product | consume existing IR seams | Leave those slices unforked. |
| Workbench `RemediationRequest` | `src/workbench/remediation.rs` | **Different type.** Do not rename, wrap, or serialize ISMS remediations through it. |
| ISO Phase 40 `RemediationItem` | [`iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md) | Informal name. Canonical type is `Remediation`. Failed tests **may** create/link a remediation; `ControlTestResult` remains immutable. |
| `sdd_compliance_ir_target` | golden fixtures, IR-019 | Must stay GREEN. Old assessments without `remediations` decode. |
| Neighbor targets | `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_assessment_lineage_target`, `sdd_documentation_layout` | Must stay GREEN. |

Tiny allowed: new `typed_id!` aliases; serde defaults; validation messages; re-exports.

---

## 3. Current behavior (baseline — historical characterization)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. After implement, `sdd_remediation_engine_baseline` is skip-superseded (`#[ignore = "superseded by sdd_remediation_engine_target"]`). This section remains the found-case record; it is **not** current product behavior.

### 3.1 No ISMS `Remediation` type

`weeping-angel-assurance-ir` modules are: `applicability`, `assessment`, `asset`, `control`, `crosswalk`, `digest`, `evidence`, `exception`, `extension`, `framework`, `id`, `identity`, `implementation`, `mapping`, `privacy`, `requirement`, `risk`, `subject`, `test`, `validation`, `vendor`.

There is **no** `mod remediation`. `lib.rs` does not export `Remediation`, `RemediationId`, `RemediationState`, or `VerificationPolicy`.

`id.rs` `typed_id!` aliases do **not** include `RemediationId`.

### 3.2 `AssessmentDefinition` has no remediations inventory

[`assessment.rs`](../../crates/weeping-angel-assurance-ir/src/assessment.rs) inventories: requirements, controls, mappings, evidence_requirements, tests, requests, implementations, scope, assets, identities, vendors, risks, exceptions, processing_activities.

There is no `remediations` field. `AssessmentDefinition::new` does not initialize one. Serde of a current assessment JSON has no `remediations` key.

`AssessmentRequests` has `risk_treatment` and `nonconformities` booleans — **capability/request flags**, not remediation records.

### 3.3 Prompt 15 event/drift types were absent at characterization

Human spec [`docs/specs/isms-events-drift.md`](isms-events-drift.md) existed (Specified). On the characterization SHA, product crates had **no** `IsmsEvent` / `EventId` / `detect_isms_drift`. Baseline RE-B04 records that absence.

Those types **have since landed**. Target binds `RemediationSource` to that event contract (`eventId`, `occurredAt`, snapshot refs, subjects, causes, optional severity, deterministic camelCase payload) via `From<&IsmsEvent>` / `From<&IsmsEventKind>` — not a second bus. Tests may still construct `RemediationSource` directly. Kinds match `IsmsEventKind`.

### 3.4 `Risk` is still the four-field stub

```text
Risk { id, title, description, status ∈ {Open, Accepted, Mitigated, Closed} }
```

No treatment actions, no remediation refs. Prompt 06/08 specs exist; product types do not.

### 3.5 `Exception` exists and is not a remediation waiver engine

```text
ExceptionStatus = Proposed | Approved | Expired | Revoked
Exception { id, controlId?, rationale, status, approvedBy?, expiresAt?, subjects }
```

No `can_transition`. Approved exceptions can produce `Effectiveness::ExceptionApproved` in control-test. Nothing creates a remediation when an exception expires.

### 3.6 Control tests are immutable observations, not tickets

`ControlTestResult` = `{ testId, controlId, effectiveness, rationale, evidenceRefs, missingEvidence, evaluatedAt, testVersion, inputDigest, duration?, status?, reason?, population? }`.

`Effectiveness` includes `Effective`, `Ineffective`, `ExceptionApproved`, etc. There is no API from evaluate → work item. A later `Effective` result does not close anything because nothing exists to close.

### 3.7 Scanner workbench is a different “remediation”

[`src/workbench/remediation.rs`](../../src/workbench/remediation.rs):

```text
RemediationRequest { finding_id, rule_id, path, start_line, title }
RemediationResult { finding_id, rule_id, strategy, state, summary, patch_path?, … }
```

This generates unified diffs from scan findings. It is **not** `assurance-ir` and must remain so.

### 3.8 No ticket-system clients

Product sources do not implement Jira, Linear, or GitHub Issues adapters for ISMS remediations. Scanner/codex-security skills may mention Jira/Linear for **finding intake**; that is out of this crate’s ISMS IR.

### 3.9 Validation does not walk remediations

[`validation.rs`](../../crates/weeping-angel-assurance-ir/src/validation.rs) checks schema, duplicate requirement/control/evidence ids, mappings, dangling tests/implementations, IR-019 risk refs, dangling exception refs on implementations. It does not know remediations.

### 3.10 Baseline suite obligations (must PASS on current code)

| Id | Characterization |
| --- | --- |
| RE-B01 | `weeping-angel-assurance-ir` has no `Remediation` / `RemediationId` / `RemediationState` symbols (compile-time `cfg` not required: source grep + unresolved import is the found case; baseline tests **must not** depend on those types existing) |
| RE-B02 | `AssessmentDefinition` serialized JSON from `::new` has no `remediations` key |
| RE-B03 | `id.rs` source does not contain `typed_id!(RemediationId)` |
| RE-B04 | crate sources contain no `ControlRegressed` type/module (Prompt 15 absence) |
| RE-B05 | `Risk` public fields remain `id`, `title`, `description`, `status` |
| RE-B06 | `Exception` type exists; `ExceptionStatus` includes `Approved` and `Expired` |
| RE-B07 | `src/workbench/remediation.rs` still defines `RemediationRequest` with `finding_id` (scanner type, not IR) |
| RE-B08 | no Jira/Linear/GitHub Issues client module under `crates/weeping-angel-*` for ticket create/transition |
| RE-B09 | `ControlTestResult` exists; evaluating `Effective` does not write any remediation inventory (none exists) |
| RE-B10 | `validate_assessment_ir` still enforces IR-019; empty assessments validate |

Baseline is skip-superseded with `#[ignore = "superseded by sdd_remediation_engine_target"]`. Target stays GREEN.

Target tests that would fail because of **compile/harness noise** (missing `[[test]]`, wrong crate, unwrap of unrelated APIs) are **not** a valid RED. RED must be the missing ISMS remediation **contract**.

---

## 4. Desired behavior (target)

### 4.1 Product home

Keep the record in **`weeping-angel-assurance-ir`**. Keep lifecycle/verification **queries** in **`weeping-angel-assurance`**. Network-free. No new crate. No kanban UI.

```text
weeping-angel-assurance-ir
  remediation.rs     # this slice’s record + transition table
  id.rs              # RemediationId (+ action / sla policy ids)
  assessment.rs      # remediations: Vec<Remediation>
  validation.rs      # graph + history + closed immutability
  exception.rs       # consumed
  risk.rs            # consumed, not forked
  implementation.rs  # PrincipalRef consumed

weeping-angel-assurance
  remediation.rs     # create_from_source, verify, sla_overdue, waive, close
```

ISO Phase 40 “failed assurance control may create/link a `RemediationItem`” is satisfied by `create_from_control_regression` / `create_from_source` on a `ControlRegressed` (or equivalent failed-test source). `ControlTestResult` remains immutable; remediation state changes independently.

Landed crate-root engine API (`weeping-angel-assurance`): `create_from_source`, `create_from_control_regression`, `link_treatment_action`, `attach_external_ticket`, `evaluate_verification`, `sla_overdue`, `reopen_expired_waiver`, `close`. IR helpers: `Remediation::propose` / `transition` / `validate`, `waiver_in_force`, `validate_remediation_inventory`, `validate_remediation_waivers_at`, `validate_remediation_slas_at`. Clockless `validate_assessment_ir` calls inventory only; waiver expiry and SLA fail-closed are clocked.

### 4.2 Typed identifiers

Add via existing `typed_id!` (same charset / length / **no uuid-v4**):

| Type | Typical prefix (documentary) | Identifies |
| --- | --- | --- |
| `RemediationId` | `rem:` | Canonical remediation |
| `RemediationActionId` | `ra:` (or `rema:`) | A planned action **on** a remediation (not Prompt 08 `RiskAcceptanceId`) |
| `SlaPolicyId` | `sla:` | Named SLA policy reference |

If `ra:` collides with Prompt 08 `RiskAcceptanceId` prefix in tests, use `rema:` for actions. Prefixes are documentary; `validate_stable_id` does not require them.

Reuse landed `TreatmentActionId`. `RemediationId` **is** the value Prompt 08 `RemediationRef` points at.

### 4.3 Source binding (Prompt 15 contract, not a new bus)

```text
RemediationSourceKind =
  ControlRegressed
  | ControlRecovered          # allowed as a cause ref, not a create-happy-path
  | EvidenceExpired
  | EvidenceRevoked
  | RiskIncreased
  | RiskDecreased
  | RiskAccepted
  | ExceptionExpired
  | NewAssetDetected
  | AssetRemoved
  | VendorRiskChanged
  | ObjectiveMissed
  | PolicyExpired
  | AuditFindingOpened
  | NonconformityOpened
  | CorrectiveActionOverdue
  | RiskTreatmentAction
  | Manual

RemediationSource {
  kind: RemediationSourceKind
  eventId: String                 # Prompt 15 EventId as stable string; validate_stable_id
  occurredAt: DateTime<Utc>?      # event time when known
  snapshotRefs: [String]          # source snapshot(s); Prompt 15 field
  causeRefs: [String]             # cause references
  subjectSelectors: [SubjectSelector]
  severityHint: TestFailureSeverity?  # classification when the event carries it
  payloadDigest: String?          # canonical_digest of deterministic payload
}
```

Serde: camelCase enum tags (`controlRegressed`, `riskTreatmentAction`, `manual`, …). Unknown tags fail closed.

Laws:

1. Creating from a control regression **requires** `kind = ControlRegressed` and a non-empty `eventId`.
2. The engine does **not** subscribe to a message bus. Callers (Prompt 15, tests, later scheduler) pass the event-shaped source in.
3. `ControlRecovered` must **not** auto-close a remediation (see §4.8). Recovery is evidence toward verification, not closure.
4. Do not define a competing event vocabulary (`ControlFailed`, `TestWentRed`, …).

Tests may construct `RemediationSource` directly. Prompt 15 types are on disk: `From<&IsmsEvent>` / `From<&IsmsEventKind>` copy `eventId` without changing identity. `IsmsEventKind::Extensible { name: "RiskTreatmentAction" }` maps to `RiskTreatmentAction`; other extensible names map to `Manual`.

### 4.4 Core record

JSON camelCase. Schema remains `assurance-ir/v1`. Empty/None skip-serialize so old assessments do not grow required keys.

```text
RemediationState =
  Proposed
  | Open
  | InProgress
  | AwaitingVerification
  | Verified
  | Closed
  | AcceptedWaived          # serde: "acceptedWaived" (alias "waived" allowed)
  | Cancelled
  | Superseded

RemediationPriority = P1 | P2 | P3 | P4     # serde p1…p4; default P3
  # or Critical | High | Medium | Low — pick one enum; tests pin serde names.
  # Normative: four-level camelCase; recommended serde: "p1"|"p2"|"p3"|"p4"

Remediation {
  schemaVersion: "assurance-ir/v1"          # skip-serialize if implementers pin via typed digest instead; prefer explicit field defaulting to ASSURANCE_IR_SCHEMA
  id: RemediationId
  title: String                             # non-empty
  description: String? 
  source: RemediationSource                 # required
  riskIds: [RiskId]
  controlIds: [ControlId]
  implementationIds: [ControlImplementationId]
  subjectSelectors: [SubjectSelector]
  treatmentActionIds: [TreatmentActionId]   # Prompt 08 linkage; empty ok
  owner: PrincipalRef                       # required once Open+
  priority: RemediationPriority
  severity: TestFailureSeverity             # reuse IR test severity
  slaPolicyId: SlaPolicyId?
  dueAt: DateTime<Utc>?                     # SLA instant; required if slaPolicyId is Some
  state: RemediationState                   # default Proposed
  externalTickets: [ExternalTicketRef]
  plannedActions: [RemediationAction]
  evidenceOfFix: [EvidenceOfFixRequirement]
  verificationPolicy: VerificationPolicy
  verificationState: VerificationState
  waiver: WaiverBinding?
  closedBy: PrincipalRef?
  closedAt: DateTime<Utc>?
  closureRationale: String?
  supersedes: RemediationId?
  supersededBy: RemediationId?
  version: u32                              # default 1
  history: [RemediationEvent]
}

RemediationAction {
  id: RemediationActionId
  title: String                             # non-empty
  owner: PrincipalRef?
  dueAt: DateTime<Utc>?
  state: Planned | InProgress | Done | Cancelled
  evidenceRefs: [String]                    # envelope digests or requirement ids
}

ExternalTicketRef {
  system: Jira | Linear | GitHubIssues | Other
  key: String                               # non-empty; adapter identity, NOT RemediationId
  url: String?
  remoteState: String?                      # documentary; never drives local state machine
}

EvidenceOfFixRequirement {
  evidenceRequirementId: EvidenceRequirementId?
  evidenceType: EvidenceType?
  description: String
  minCardinality: u32                       # default 1
}

VerificationPolicy {
  mode: SingleGreenPermitted | SustainedWindow | IndependentReviewRequired
  window: Duration?                         # required when mode = SustainedWindow (serde: seconds or ISO-8601; pick Duration secs u64)
  minEffectiveResults: u32                  # default 1; SustainedWindow typically ≥ 2
  independentVerifier: bool                 # if true, verifier principal ≠ owner
}

VerificationState {
  status: NotStarted | Failed | InWindow | Satisfied | Rejected
  lastResultDigest: String?                 # canonical_digest of last considered ControlTestResult (identity excludes duration)
  windowStart: DateTime<Utc>?
  satisfiedAt: DateTime<Utc>?
  note: String?
}

WaiverBinding {
  kind: Exception | RiskAcceptance
  exceptionId: ExceptionId?
  riskAcceptanceId: String?                 # opaque until Prompt 08; validate_stable_id
  expiresAt: DateTime<Utc>?                 # denormalized clock; must not outlive the governing record
}

RemediationEvent {
  version: u32
  at: DateTime<Utc>
  principal: PrincipalRef?
  kind: Created
      | FieldsRevised
      | StateTransition { from: RemediationState, to: RemediationState }
      | VerificationRecorded { status: VerificationState.status, resultDigest: String? }
      | ExternalTicketAttached { system, key }
      | WaiverBound { kind }
      | Closed                           # principal lives on RemediationEvent
      | Superseded { successor: RemediationId }
}
```

`schemaVersion` may be omitted on the struct if the typed digest already domain-separates `assurance-ir/v1`; if present it must equal `ASSURANCE_IR_SCHEMA`.

Public construction: `Remediation::propose(id, title, source, owner)` (name flexible) starts in `Proposed` (or `Open` if tests prefer a single start state — **normative default: `Proposed`**). State changes go through `transition` so history is appended. Direct field writes that record an illegal `history` pair fail `validate()`.

### 4.5 State machine (fail closed)

```text
Proposed → Open → InProgress → AwaitingVerification → Verified → Closed
                              ↘ AcceptedWaived (only with in-force waiver)
         ↘ Cancelled
Verified / Open / InProgress / AwaitingVerification / AcceptedWaived → Superseded
Closed / Cancelled / Superseded are terminal for that id
```

Normative table (from → allowed to). Any other pair is `RemediationError::InvalidTransition`. Library paths must not panic.

| From | Allowed targets |
| --- | --- |
| `Proposed` | `Open`, `Cancelled` |
| `Open` | `InProgress`, `AcceptedWaived`, `Cancelled`, `Superseded` |
| `InProgress` | `AwaitingVerification`, `Open` (return to queue), `AcceptedWaived`, `Cancelled`, `Superseded` |
| `AwaitingVerification` | `Verified`, `InProgress` (verification failed), `AcceptedWaived`, `Cancelled`, `Superseded` |
| `Verified` | `Closed`, `InProgress` (regression after verify, before close), `Superseded` |
| `Closed` | ∅ |
| `AcceptedWaived` | `Open` (waiver expired / revoked — **reopen**), `Cancelled`, `Superseded` |
| `Cancelled` | ∅ |
| `Superseded` | ∅ |

Guards:

1. `Proposed → Open` requires non-empty `title`, `source.eventId`, and `owner`.
2. `Open → InProgress` requires `owner`. Planned actions may still be empty (work can be recorded later) but `evidenceOfFix` should be non-empty before `AwaitingVerification`.
3. `InProgress → AwaitingVerification` requires every `EvidenceOfFixRequirement` with `minCardinality ≥ 1` to have enough attached evidence refs on planned actions or the parent **or** an explicit `verificationState.note` that independent review will collect them — **fail closed**: missing required evidence blocks the transition.
4. `AwaitingVerification → Verified` requires `verificationState.status == Satisfied` under §4.8. **A single green `ControlTestResult` does not satisfy this unless `verificationPolicy.mode == SingleGreenPermitted`.**
5. `Verified → Closed` requires `closedBy` + `closedAt` + non-empty `closureRationale`. Closing from any other state is illegal (no skip from `InProgress` to `Closed`).
6. `* → AcceptedWaived` requires `waiver` present **and** in force at `at` (§4.9). Invalid/expired waiver ⇒ error, not a stored waived state.
7. `AcceptedWaived → Open` is **required** when a clocked validate detects an expired/revoked waiver; callers may use `reopen_expired_waiver(as_of)`.
8. `* → Superseded` requires `supersededBy` pointing at a different existing `RemediationId` whose `supersedes` is this id.
9. Terminal records (`Closed`, `Cancelled`, `Superseded`) reject field mutation other than no-op deserialize. `revise` on `Closed` fails.

```text
RemediationState::can_transition(from, to) -> bool
Remediation::transition(to, principal, at) -> Result<Self, RemediationError>
```

### 4.6 Inventory and references (fail closed)

`AssessmentDefinition` gains:

```text
remediations: Vec<Remediation>    // serde default empty
```

On `validate()` (clockless):

| Check | Rule |
| --- | --- |
| Duplicate `RemediationId` | error |
| Duplicate nested `RemediationActionId` | error |
| `riskIds` | each ∈ `assessment.risks` |
| `controlIds` | each ∈ `assessment.controls` |
| `implementationIds` | each ∈ `assessment.implementations` |
| `treatmentActionIds` | each must resolve in `assessment.risk_treatments` **or** in the optional injected action-id set passed to `validate_remediation_inventory`. Empty `treatmentActionIds` is valid. A present id that is not in that inventory is dangling (fail closed). |
| `waiver.exceptionId` | if `Some`, ∈ `assessment.exceptions` |
| `owner` / `closedBy` `Identity` | ∈ `assessment.identities` |
| `evidenceOfFix.evidenceRequirementId` | if `Some`, ∈ `assessment.evidence_requirements` |
| `supersedes` / `supersededBy` | ids exist in `assessment.remediations` |
| `history` | consecutive `StateTransition` pairs obey §4.5 |
| `Closed` | `closedBy` and `closedAt` present; last history event is `Closed` or a `StateTransition` to `Closed` |
| UUID-v4 ids | rejected by `typed_id!` |

IR-019 (implementation → risk) remains.

### 4.7 SLA overdue

```text
Remediation::sla_overdue(as_of: DateTime<Utc>) -> bool
```

| Condition | Result |
| --- | --- |
| `dueAt` is `None` | not overdue |
| `dueAt >= as_of` | not overdue |
| `dueAt < as_of` and state ∈ `{Closed, Cancelled, Superseded}` | **not** overdue (clock stopped) |
| `dueAt < as_of` and state ∈ `{Proposed, Open, InProgress, AwaitingVerification, Verified, AcceptedWaived}` | **overdue** |

`AcceptedWaived` still reports SLA overdue as a **data fact** (the gap was not fixed; it was waived). Clocked `validate_remediation_slas_at(assessment, as_of)` **does not** fail the assessment solely for overdue waived items; it **does** fail (or the query API returns them separately) for overdue non-waived non-terminal items. Target test “SLA overdue” asserts the boolean on an `InProgress` record with `dueAt` in the past.

`slaPolicyId` is a reference, not an embedded policy engine. If `Some`, `dueAt` is required (fail closed). This slice does not auto-compute due dates from severity.

### 4.8 Verification policy (single green is not auto-close)

`ControlTestResult` is immutable. The engine **considers** a sequence of results for the remediation’s `controlIds` / `source` test:

```text
evaluate_verification(remediation, results: &[ControlTestResult], as_of, verifier: Option<PrincipalRef>)
  -> Result<VerificationState, RemediationError>
```

Laws:

1. **Default / `SustainedWindow`:** one `Effectiveness::Effective` result **does not** set `Satisfied` and **does not** transition to `Verified` or `Closed`.
2. **`SingleGreenPermitted`:** a single `Effective` **may** set `Satisfied` (and then the caller may `transition(Verified)`). It still does **not** auto-`Closed`. Close is always an explicit transition with principal + rationale.
3. **`SustainedWindow`:** `Satisfied` iff there exist ≥ `minEffectiveResults` (default **2**) `Effective` results whose `checked_at` (`evaluatedAt`) span is `≥ window` (default **14d** as `u64` seconds) **and** no intervening `Ineffective` / `InsufficientEvidence` / `StaleEvidence` for the same control after `windowStart`. A lone green inside a 14-day window fails. Default `VerificationPolicy` is this mode.
4. **`IndependentReviewRequired`:** `verifier` must be present and `verifier != owner` (compare `PrincipalRef` equality). A green test from the owner’s automation still needs the independent principal recorded.
5. **Verification failure:** an `Ineffective` (or equivalent fail-closed effectiveness) while `AwaitingVerification` sets `verificationState.status = Failed` and **only** allows `transition(InProgress)` — not `Verified`/`Closed`.
6. Result identity for `lastResultDigest` uses the same rule as control-test: **exclude** `duration` and prefer semantic fields (`testId`, `controlId`, `effectiveness`, `inputDigest`, `testVersion`, evidence refs). Reuse `canonical_digest` on a projection if the full struct includes wall-clock `evaluatedAt`.
7. Prompt 15 `ControlRecovered` is an input **source/cause**, not a close signal.

### 4.9 Waiver / accepted (expired waiver cannot remain waived)

`AcceptedWaived` is **not** a generic “won’t fix” ticket resolution. It is allowed **only** when governed by a valid exception or risk acceptance.

In force at `as_of` iff:

| `waiver.kind` | In force |
| --- | --- |
| `Exception` | `exception.status == Approved` **and** (`expires_at` is `Some` and `as_of < expires_at`) **and** not `Revoked`/`Expired`. Missing `expires_at` ⇒ **not** in force (fail closed; no silent perpetual waiver). |
| `RiskAcceptance` | Prompt 08 `acceptance_in_force` if types exist; otherwise `riskAcceptanceId` well-formed **and** `waiver.expiresAt` present **and** `as_of < expiresAt`. Missing expiry ⇒ not in force. |

Laws:

1. Transition into `AcceptedWaived` fail-closes if `waiver` is missing. In-force at `at` is `waiver_in_force`; clocked `validate_remediation_waivers_at` **rejects** any record still in `AcceptedWaived` whose waiver is expired/revoked.
2. The legal repair is `reopen_expired_waiver` (`AcceptedWaived → Open`).
3. `ExceptionStatus::Expired` / `Revoked` cannot keep `AcceptedWaived`.
4. Do not treat `RiskStatus::Accepted` as a waiver. Do not overload scanner finding `accepted-risk` as this state.
5. `Effectiveness::ExceptionApproved` on a test does not close or waive the remediation by itself.

### 4.10 External tickets are adapters only

```text
remediation.id          = rem:control-regressed-mfa-2026-08
externalTickets[0]      = { system: jira, key: "SEC-441", url: "https://…" }
```

Laws:

1. Canonical identity is `RemediationId`. Attaching/updating `ExternalTicketRef` does not change `id`.
2. `remoteState` is documentary. A remote “Done” **must not** `transition(Closed)`.
3. No HTTP client, webhook, or API token handling in this slice.
4. Multiple refs allowed (Jira + GitHub). Duplicate `(system, key)` pairs collapse or fail duplicate — **fail duplicate** on the same remediation.
5. `system` serde: `jira` \| `linear` \| `githubIssues` \| `other`.
6. Target test “external-ticket reference” asserts round-trip of the ref **and** that `canonical_digest` / id stay on `RemediationId`.

### 4.11 Create from control regression

```text
create_from_control_regression(source: RemediationSource, control_id, owner, …) -> Remediation
```

Requires `source.kind == ControlRegressed`. Copies affected `controlIds` / subjects from the source. Sets `state = Proposed` (caller may `transition(Open)`). Does **not** mutate `ControlTestResult`. Does **not** emit Prompt 15 events.

### 4.12 Risk-treatment-action linkage

A mitigation `TreatmentAction` and a `Remediation` may point at each other:

- Remediation: `treatmentActionIds` contains `ta:…`
- Treatment action (Prompt 08): `remediationRefs` contains `rem:…`

This slice implements the remediation side. Tests construct a treatment-action id and assert it round-trips on the record and fails validate when the injected inventory does not contain it (when an inventory is provided). Do **not** implement Prompt 08 state machines.

### 4.13 Immutable closure history

When `state == Closed`:

1. `closedBy`, `closedAt`, `closureRationale` are required and frozen.
2. `history` contains a `Closed` or `StateTransition { to: Closed }` event; subsequent events other than no-ops are illegal.
3. `transition` from `Closed` always errors.
4. `revise` / field builders that would change title, source, waiver, or tickets after close error (`RemediationError::ImmutableClosure`).
5. Deserializing a closed record and serializing again yields the same `canonical_digest` if no illegal mutation occurred.
6. Target test “immutable closure history” mutates a closed clone and expects failure; the original history still contains the closure event.

### 4.14 Serialization and digest

- Keep `assurance-ir/v1`.
- `#[serde(rename_all = "camelCase")]` on enums and structs.
- `canonical_digest` = SHA-256 hex of `serde_json::to_vec` (struct field order + BTree maps).
- `typed_canonical_digest("Remediation", &value)` domain-separates with `wa:assurance-ir:assurance-ir/v1:Remediation:`.
- Maps/sets use `BTreeMap`/`BTreeSet`.
- UUID-v4 in `RemediationId` fails deserialize.

### 4.15 Engine API (library, names flexible)

```text
create_from_source(id, title, source, owner) -> Result<Remediation>
create_from_control_regression(id, title, source, control_id, owner) -> Result<Remediation>
link_treatment_action(remediation, TreatmentActionId) -> Result<Remediation>
attach_external_ticket(remediation, ExternalTicketRef) -> Result<Remediation>
evaluate_verification(remediation, results, as_of, verifier) -> Result<VerificationState>
sla_overdue(remediation, as_of) -> bool
waiver_in_force(assessment, remediation, as_of) -> bool
reopen_expired_waiver(remediation, as_of, principal, assessment) -> Result<Remediation>
close(remediation, principal, at, rationale) -> Result<Remediation>
```

No `notify_assignee`. No `create_jira_issue`.

---

## 5. Dual-suite protocol

Follow [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

| Suite | File | Cargo `[[test]]` name | On characterization HEAD |
| --- | --- | --- | --- |
| Baseline | `tests/contracts/remediation_engine.baseline.rs` | `sdd_remediation_engine_baseline` | **GREEN** (found case in §3) |
| Target | `tests/contracts/remediation_engine.target.rs` | `sdd_remediation_engine_target` | **RED** until implement (missing contract, **not** harness noise) |

Protocol (mandatory):

1. Spec (this file) + draft ADR + `CANONICAL_SPECS` — **this phase**. **No product feature code.**
2. Write baseline — GREEN on current code.
3. Write target — RED on current code (do not `#[ignore]` target tests).
4. Implement product in existing crates.
5. Target GREEN; neighbor suites listed in the header stay GREEN.
6. Skip-supersede baseline: `#[ignore = "superseded by sdd_remediation_engine_target"]`.
7. Re-prove target GREEN.

Abort if baseline cannot go green or target cannot go red for the **right** reason.

Register dual-suite `[[test]]` rows in the **same implement commit** as the `.rs` files.

---

## 6. Acceptance criteria (testable)

Target suite must encode at least:

- **RE-001 Create from control regression.** Given a `RemediationSource` with `kind = controlRegressed` and a stable `eventId`, `create_from_control_regression` yields a `Remediation` whose `source.eventId` matches, `controlIds` contain the regressed control, `state` is `Proposed` (or `Open` after an explicit transition), and the originating `ControlTestResult` is unchanged.
- **RE-002 Risk-treatment-action linkage.** A remediation stores `treatmentActionIds: ["ta:mitigate-branch-protection"]`; serde round-trips camelCase `treatmentActionIds`; validate fails when an injected treatment inventory does not contain that id; canonical `id` remains a `RemediationId`.
- **RE-003 SLA overdue.** `dueAt` in the past + `InProgress` ⇒ `sla_overdue(as_of) == true`; `dueAt` in the future ⇒ false; `Closed` with past `dueAt` ⇒ false.
- **RE-004 External ticket reference.** Attaching `{ system: "jira", key: "SEC-441" }` round-trips; `RemediationId` does not become `SEC-441`; no ticket client is invoked; duplicate `(system, key)` fails; UUID-v4 as `RemediationId` is rejected.
- **RE-005 Verification failure.** While `AwaitingVerification`, an `Ineffective` `ControlTestResult` sets `verificationState.status = Failed` and forbids `Verified`/`Closed`; `InProgress` remains legal.
- **RE-006 Sustained success / single green.** Default or `SustainedWindow` policy: one `Effective` result does **not** `Satisfied` and does **not** close. Two (or `minEffectiveResults`) `Effective` results spanning `window` with no intervening fail ⇒ `Satisfied`; explicit `transition(Verified)` then `close` succeeds. `SingleGreenPermitted` may satisfy on one green but still requires explicit close.
- **RE-007 Expired waiver.** `AcceptedWaived` bound to `Exception` `Approved` with `expiresAt < as_of` (or `Expired`/`Revoked`) cannot remain `AcceptedWaived`; `waiver_in_force == false`; reopen to `Open` succeeds; transition **into** waived with expired exception fails.
- **RE-008 Immutable closure history.** After `close`, `closedBy`/`closedAt`/`history` retain the closure event; `transition`/`revise` fail; `canonical_digest` of the frozen record is stable.
- **RE-009 Prompt 15 seam.** Source kinds include `controlRegressed` (and the Prompt 15 set); crate still does **not** implement snapshot-diff event emission; no parallel event-bus module is required for the test to pass.
- **RE-010 Identity, JSON, digest.** `typed_id!(RemediationId)` + `validate_stable_id`; JSON camelCase (`inProgress`, `awaitingVerification`, `acceptedWaived`); `canonical_digest` / `typed_canonical_digest` SHA-256.
- **RE-011 Additive assessment.** `AssessmentDefinition` with empty `remediations` still validates; old assessments without the key deserialize.
- **RE-012 Dual-suite registration.** Root `Cargo.toml` lists `sdd_remediation_engine_baseline` and `sdd_remediation_engine_target` at `tests/contracts/remediation_engine.{baseline,target}.rs` (implement commit).
- **RE-013 Neighbor GREEN.** `sdd_compliance_ir_target`, `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_assessment_lineage_target`, `sdd_documentation_layout` remain GREEN.
- **RE-014 Workbench isolation.** `src/workbench/remediation.rs` `RemediationRequest` still exists and is not the IR type.

Baseline suite must encode §3 found-case (RE-B01…B10).

---

## 7. Out of scope

- Kanban UI, boards, drag-and-drop, dashboards, assignment inboxes.
- Assignment notifications, Slack/email/webhooks, notification transport.
- Jira / Linear / GitHub Issues **clients**, OAuth, webhooks, remote state sync.
- Implementing Prompt 15 event emission, snapshot drift, or a generic event bus.
- Forking Prompt 06 `Risk`, Prompt 08 treatment strategy machines, Prompt 10 `ControlImplementation` expansion, Prompt 14 validity events.
- Prompt 22 nonconformity/CAPA records (landed neighbor; may cite remediations via `RemediationRef`).
- Auto-closing from one green test unless `SingleGreenPermitted` is explicit.
- Scanner workbench patch generation (`RemediationRequest`).
- New crate, new long-term database, GRC SaaS sync.
- Bumping `assurance-ir/v1`.
- Claiming ISO 27001 clause satisfaction from a remediation record.
- `tests/sdd/` dual-suite paths.

---

## 8. Risks

- Prompt 15 lands with different field names than this seam. Mitigation: copy Prompt 15 event **names** exactly; keep `eventId` as a stable string; add `From` impls at rebase rather than a second catalog.
- Prompt 08 `RemediationRef` vs this `RemediationId` diverge. Mitigation: same `typed_id!` rules; `RemediationRef` **is** `RemediationId`’s string.
- `RemediationActionId` prefix `ra:` collides with `RiskAcceptanceId`. Mitigation: use `rema:` in fixtures; types remain distinct newtypes.
- Scanner `RemediationRequest` confuses reviewers and greps. Mitigation: baseline RE-B07 / target RE-014 pin the split; do not alias the types.
- Auto-close from `ControlRecovered` or one green test would violate the mission. Mitigation: §4.8 default fail-closed; tests RE-005/RE-006.
- Expired waivers left in `AcceptedWaived` would hide open gaps. Mitigation: clocked validate + RE-007.
- Ticket `key` used as IR id would import UUID-v4 / vendor identity. Mitigation: `typed_id!` + RE-004.
- Expanding `AssessmentDefinition` can break exhaustive struct literals. Mitigation: `serde(default)` + update `::new` only; search literals in this slice.
- Dual-suite forgotten in `Cargo.toml` yields false RED. Mitigation: I3 — register in the implement commit; target RED must be missing types/behavior, not “test binary not found”.
- Neighbor ISMS specs landing in parallel rewrite `Risk`/`Exception`. Mitigation: consume public fields; do not edit those modules except re-exports.

---

## 9. Implement-time file budget (not this phase)

Product (later):

- `crates/weeping-angel-assurance-ir/src/remediation.rs` (new)
- `crates/weeping-angel-assurance-ir/src/id.rs` (additive typed ids)
- `crates/weeping-angel-assurance-ir/src/assessment.rs` (`remediations` vec)
- `crates/weeping-angel-assurance-ir/src/validation.rs`
- `crates/weeping-angel-assurance-ir/src/lib.rs` re-exports
- `crates/weeping-angel-assurance/src/remediation.rs` (engine) + `lib.rs` module
- Optional `tests/fixtures/assurance-ir/v1/remediation.json`

Tests/docs (later, **same commit as product** for dual-suite):

- `tests/contracts/remediation_engine.baseline.rs`
- `tests/contracts/remediation_engine.target.rs`
- root `Cargo.toml` `[[test]]` rows
- pointer in [`docs/specs/assurance-runtime.md`](assurance-runtime.md) when APIs exist
- ADR status → **Accepted** when target GREEN

Implement landed: IR + engine modules, dual-suite `[[test]]` rows, public-contract section, ADR **Accepted**. `CANONICAL_SPECS` already lists this path.

---

## 10. Definition of done

Every material assurance/risk gap can be tracked from a Prompt 15-shaped cause (or treatment action) through planned corrective work, evidence-of-fix, independent/sustained verification, optional governed waiver, and immutable closure — with canonical identity inside Weeping Angel, camelCase IR, SHA-256 digests, and external tickets as adapters only.

Dual-suite SDD protocol is satisfied: spec first (this document), baseline GREEN on characterization, target RED then GREEN after implement, docs+ADR accepted, baseline skip-superseded, target still GREEN.
