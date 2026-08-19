# ADR 0003 — Remediation workflow engine (canonical ISMS work records)

| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_remediation_engine_target` GREEN; `sdd_remediation_engine_baseline` skip-superseded. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The informal ISO MVP Phase 40 reading that a failed control “may create/link a `RemediationItem`” **without** an IR type. Does **not** supercede IR schema `assurance-ir/v1`, canonical digest `canon/v1`, ADR 0001 spine, control-test immutability, Prompt 15 event observations, Prompt 08 treatment machines, or scanner workbench `RemediationRequest`. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [ADR 0003 ISMS events](0003-isms-events-drift.md), [ADR 0004](0004-documentation-architecture.md), [ADR 0006 risk treatment](0006-risk-treatment-engine.md) |
| Spec | [`docs/specs/remediation-engine.md`](../specs/remediation-engine.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) — Remediation engine |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_remediation_engine_baseline` skip-superseded; `sdd_remediation_engine_target` GREEN. Dual-suite at `tests/contracts/remediation_engine.{baseline,target}.rs` registered in root `Cargo.toml` (I3). |

> Filename `0003-*` is shared with catalog-program / IR-engine siblings. **0004** is documentation architecture. Cite this decision by **path**.

## Context

On characterization SHA `6e31bf1a…`:

1. `weeping-angel-assurance-ir` had no `Remediation` / `RemediationId`; `AssessmentDefinition` had no `remediations` inventory.
2. Prompt 15 human spec existed; product had no `IsmsEvent` (those types later landed and this engine consumes them).
3. `Risk` was still `{id, title, description, status}`. Prompt 08 now stores opaque `RemediationRef`; this slice owns `struct Remediation`.
4. Scanner workbench `RemediationRequest` generates unified diffs from findings — a different type.
5. No Jira/Linear/GitHub Issues client lives in the assurance crates.
6. `ControlTestResult` is an immutable observation. Nothing tracked cause → fix evidence → independent verification → closure.

Operational ISMS v1 Prompt 16 requires canonical remediation records that connect assurance failures and risk treatment to accountable work **without** becoming a generic project-management system: no kanban UI, no assignment notifications, no ticket-system implementation.

Questions this decision answers:

1. Where does the canonical record live — IR, workbench, a new crate, or an external tracker?
2. How do remediations bind to Prompt 15 events?
3. When may a green control test close a remediation?
4. How do external tickets relate to identity?
5. How do expired exceptions/acceptances interact with `AcceptedWaived`?
6. How is closure history preserved?

## Decision

This is what shipped. Field-level law is [`docs/specs/remediation-engine.md`](../specs/remediation-engine.md).

Product home: `weeping-angel-assurance-ir::remediation` (`crates/weeping-angel-assurance-ir/src/remediation.rs`) plus lifecycle queries in `weeping-angel-assurance::remediation` (crate-root re-exports). Schema stays `assurance-ir/v1`.

### 1. Canonical record is IR `Remediation` in existing crates

`weeping-angel-assurance-ir::Remediation` is the SSOT. `AssessmentDefinition.remediations: Vec<Remediation>` is additive (`serde(default)`). Empty inventory keeps old assessments valid. **No new crate.**

Incorrect: overloading `src/workbench/remediation.rs`; using Jira as system of record; a GRC sidecar; bumping to `assurance-ir/v2`.

ISO Phase 40 “`RemediationItem`” is this type. Scanner `RemediationRequest` stays a code-patch generator.

### 2. Identity is `typed_id!` + `validate_stable_id`; JSON camelCase; SHA-256 digests

Landed aliases: `RemediationId`, `RemediationActionId`, `SlaPolicyId`. `RemediationRef` (Prompt 08 / incident corrective actions) is the same charset; canonical work identity is `RemediationId`. Random v4 fails `IdError::InvalidCharacter`. External ticket keys never become the IR id.

JSON is `#[serde(rename_all = "camelCase")]`. Digests reuse `canonical_digest` / `typed_canonical_digest("Remediation", …)`. Default verification policy is `SustainedWindow` with `window = 14d` (`u64` seconds) and `minEffectiveResults = 2`.

### 3. Sources bind to Prompt 15 events, not a new bus

`RemediationSource.kind` uses Prompt 15 `IsmsEventKind` names (`ControlRegressed`, `EvidenceExpired`, `ExceptionExpired`, …) plus `RiskTreatmentAction` and `Manual`. `eventId` is the same `EventId` (`event:sha256:…`) that `detect_events` assigns.

Landed conversions: `From<&IsmsEventKind>` and `From<&IsmsEvent>` copy `eventId` / snapshot / cause refs / payload digest. They do **not** mint a second identity and do **not** call `detect_isms_drift`. This crate does not emit events or implement notification transport.

`create_from_control_regression` requires `kind = ControlRegressed` and copies the control id plus source subjects. `ControlRecovered` is a cause/source, never an auto-close.

### 4. State machine is fail-closed and not a kanban product

```text
Proposed → Open → InProgress → AwaitingVerification → Verified → Closed
+ AcceptedWaived (only with a waiver binding; in-force checked on the clocked path)
+ Cancelled | Superseded
```

`RemediationState::can_transition` / `Remediation::transition` are the writers. Invalid pairs return `RemediationError::InvalidTransition` (no panic). `Closed` / `Cancelled` / `Superseded` are terminal for that id. `Verified → Closed` is the only close path (`close` sets `closedBy` / `closedAt` / `closureRationale` first).

Construction: `Remediation::propose` (engine `create_from_source`) starts in `Proposed` with a `Created` history event.

### 5. One green test does not auto-close

`ControlTestResult` stays immutable. `evaluate_verification` reads results for the remediation’s `controlIds` (identity digest excludes wall-clock `duration`; clock is `checked_at`).

- Default `SustainedWindow`: one `Effective` is **not** `Satisfied`. Need ≥ `minEffectiveResults` (default 2) greens whose `checked_at` span is ≥ `window`, with no intervening `Ineffective` / `InsufficientEvidence` / `StaleEvidence`.
- `SingleGreenPermitted` may set `Satisfied` on one green; **still** requires explicit `close`.
- `IndependentReviewRequired` (or `independentVerifier`) requires a verifier `PrincipalRef` ≠ owner.
- While `AwaitingVerification`, a fail-closed effectiveness sets `verificationState.status = Failed` and forbids `Verified` / `Closed`; `InProgress` remains legal.

### 6. External tickets are adapter references only

`ExternalTicketRef { system: Jira | Linear | GitHubIssues | Other, key, url?, remoteState? }` may be attached (`attach_external_ticket`). Duplicate `(system, key)` fails. No HTTP clients. Remote “done” does not close the IR record.

### 7. Expired waiver cannot remain `AcceptedWaived`

`WaiverBinding` is `Exception` (must resolve to `Approved` + `as_of < expiresAt`; missing expiry is **not** in force) or `RiskAcceptance` (stable id + `waiver.expiresAt` in force; Prompt 08 `acceptance_in_force` when `risk_treatments` is non-empty).

`RiskStatus::Accepted` is not a waiver. `Effectiveness::ExceptionApproved` does not waive the remediation.

Clockless `validate_assessment_ir` walks graph integrity (`validate_remediation_inventory`). Clocked `validate_remediation_waivers_at` rejects any record still in `AcceptedWaived` whose waiver is expired/revoked. Repair is `reopen_expired_waiver` → `Open`. `validate_remediation_slas_at` does not fail waived overdue items; `sla_overdue` still reports them as a data fact.

### 8. Closure history is immutable

`close` records `closedBy` / `closedAt` / rationale and appends `RemediationEventKind::Closed` (principal lives on the event wrapper). `revise`, `transition`, `attach_ticket`, and `link_action` on `Closed` return `RemediationError::ImmutableClosure`. Frozen closed records have a stable `canonical_digest`.

### 9. Neighbors are consumed, not forked

- Prompt 15: consume `IsmsEvent`; do not reimplement drift.
- Prompt 08: `treatmentActionIds` resolve against `assessment.risk_treatments` (or an injected action-id set). Prompt 08 `RemediationRef` **is** this `RemediationId` string.
- Incidents: `correctiveActionIds` are `RemediationRef`. When `assessment.remediations` is non-empty, dangling corrective-action ids fail closed. Incident close does not close remediations.
- Scanner workbench types stay scanner types.

## Non-goals

Kanban UI; assignment notifications; ticket-system clients; Prompt 15 event emission; CAPA/nonconformity (Prompt 22); new crate; `tests/sdd/`.

## Consequences

- Operators can trace a control regression or treatment action to accountable work without a PM tool.
- ISO Phase 40 linkage is this `Remediation` type, not `RemediationItem` as a second schema.
- Prompt 15 `From<&IsmsEvent>` feeds `RemediationSource` without a bus.
- Neighbor dual-suites (`sdd_compliance_ir_target`, `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_assessment_lineage_target`) stay GREEN; empty `remediations` keeps old assessments valid.

## Related

- Spec: [`docs/specs/remediation-engine.md`](../specs/remediation-engine.md)
- Prompt: [`docs/prompts/operational-isms-v1/16-remediation-engine.md`](../prompts/operational-isms-v1/16-remediation-engine.md)
- Prompt 15: [`docs/prompts/operational-isms-v1/15-isms-events-drift.md`](../prompts/operational-isms-v1/15-isms-events-drift.md), [ADR 0003 ISMS events](0003-isms-events-drift.md)
- Risk treatment (opaque `RemediationRef`): [`docs/specs/risk-treatment.md`](../specs/risk-treatment.md), [ADR 0006](0006-risk-treatment-engine.md)
- Incidents (`correctiveActionIds`): [`docs/specs/incident-governance.md`](../specs/incident-governance.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
