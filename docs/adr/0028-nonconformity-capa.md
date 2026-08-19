# ADR 0028 — Nonconformity and CAPA as operational IR records

<!-- weeping-angel-adr-meta
id = "0028"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_nonconformity_capa_target` GREEN; baseline skip-superseded; inventories on `AssessmentDefinition.nonconformities` / `corrective_actions` |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Does **not** supercede the governance-catalog corrective-action *attestation* (`control.governance.corrective-action`), IR schema `assurance-ir/v1`, ADR 0001 spine, Prompt 16 `Remediation`, Prompt 19 `Incident`, or Prompt 21 `AuditFinding`. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0003 remediation](0031-remediation-engine.md), [ADR 0003 incident governance](0023-incident-governance.md), [ADR 0003 internal audit](0025-internal-audit.md), [ADR 0003 ISMS events](0026-isms-events-drift.md), [ADR 0004](0004-documentation-architecture.md) |
| Spec | [`docs/specs/nonconformity-capa.md`](../specs/nonconformity-capa.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) — Nonconformity and CAPA |
| Characterization | Pre-product SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` + landed Prompts 15/16/19/21 (baseline found case) |
| Tests | `sdd_nonconformity_capa_target` GREEN (NC-001–NC-012); `sdd_nonconformity_capa_baseline` skip-superseded (`#[ignore = "superseded by sdd_nonconformity_capa_target"]`) |

> Filename `0003-*` is shared with catalog-program / IR-engine siblings. **0004** is documentation architecture. Cite this decision by **path**. Parallel Operational ISMS drafts occupy `0005-*` / `0006-*` / `0007-*` / `0008-*`; do not reuse those numbers for this decision.

## Context

Before this slice Weeping Angel could attest that an organization *has* a corrective-action process (governance catalog `control.governance.corrective-action` + `manual-review`). Internal audit stored an opaque `AuditFinding.nonconformityId` (`NonconformityRef = String`) and could label a finding `kind = nonconformity`. Incidents cited Prompt 16 `RemediationRef` as `correctiveActionIds`. Prompt 15 already *named* `IsmsEventKind::NonconformityOpened` and `CorrectiveActionOverdue`, but `IsmsSnapshot` bags for those inventories were empty no-ops.

There was no operational record that proved a detected nonconformity was contained, root-caused, corrected, verified for effectiveness over a declared period, and formally closed.

Pre-product facts (baseline SHA):

1. No `Nonconformity` / `CorrectiveAction` / `NonconformityId` / `CorrectiveActionId` in `weeping-angel-assurance-ir`.
2. `AssessmentDefinition` had no `nonconformities` / `corrective_actions` inventories.
3. `AssessmentRequests.nonconformities` and `FrameworkCapabilities.supports_nonconformities` were fail-closed compile flags (default `false`).
4. `AuditFinding.kind = nonconformity` did not start CAPA.
5. `Incident.corrective_action_ids` / PIR `proposed_corrective_action_ids` were `RemediationRef`.
6. Catalog `control.governance.corrective-action` was an attestation fact. Retargeting it as the operational register would collapse “we attested that we handle NCs” into “this CAPA closed.”

Operational ISMS v1 Prompt 22 required a canonical CAPA lifecycle **without** a generic issue tracker or an AI root-cause engine.

Questions this decision answers:

1. Where do nonconformity and corrective-action records live (IR vs catalog vs tickets vs Prompt 16 `Remediation`)?
2. When does an audit finding, incident, or control regression become a nonconformity, and who classifies major/minor?
3. How does CAPA `CorrectiveAction` relate to Prompt 16 `Remediation` and incident `correctiveActionIds`?
4. When may a green control test close CAPA?
5. How are cancel, supersede, reopen, and closure history preserved?
6. What remains out of product scope?

## Decision

Field names and tables are specified in [`docs/specs/nonconformity-capa.md`](../specs/nonconformity-capa.md). Landed in `weeping-angel-assurance-ir` (`capa.rs`) + `weeping-angel-assurance` (`capa`); schema stays `assurance-ir/v1`.

### 1. Same IR crate, two new records, same schema version

The register **is** `Nonconformity` + `CorrectiveAction` stored on `AssessmentDefinition.nonconformities` and `AssessmentDefinition.corrective_actions` (`serde(default)` empty). There is no GRC sidecar crate, `CapaV2`, or `assurance-ir/v2`. JSON is camelCase. Canonical digest stays serde field order + BTree maps.

Canonical identities are `NonconformityId` and `CorrectiveActionId` (`typed_id!`). `AuditFinding.nonconformity_id` remains the Prompt 21 seam (`NonconformityRef = String`); when the CAPA inventory is non-empty those refs must resolve.

`ComplianceNodeRef` was **not** extended with CAPA variants (spec-optional; not required for NC-001–NC-012). Closed CAPA does not imply a control is effective.

Incorrect: storing CAPA only as `evidence.manual.attestation`; using Jira keys as the canonical id; overloading `Remediation` as the nonconformity.

### 2. Proposal is explicit; classification is a decision boundary

Audit findings, incidents, and `ControlRegressed` events **may propose** nonconformities through `propose_from_audit_finding` / `propose_from_incident` / `propose_from_control_regression` / `Nonconformity::open` with a `PrincipalRef` and timestamps. They are not CAPA until that call.

There is no `From<AuditFinding> for Nonconformity`, no `From<Incident> for Nonconformity`, no severity auto-threshold, and no collector insert.

`AuditFinding.kind = nonconformity` still does **not** start CAPA (Prompt 21 law).

Major / minor / opportunity is **unset** until `classify(principal, rationale, classification)`. Copying `AuditFindingSeverity` or `IncidentSeverity` into `NonconformityClassification` is forbidden. `propose_from_incident` may copy Prompt 16 `RemediationRef`s onto `nonconformity.remediationRefs` as supporting work; that is not classification and not a CAPA `CorrectiveActionId`.

### 3. `CorrectiveAction` is not `Remediation` and not incident IR

Prompt 16 `Remediation` remains the general assurance **work** record. Incident `correctiveActionIds` **stay** `RemediationRef`. PIR `proposed_corrective_action_ids` stay `RemediationRef`.

This slice’s `CorrectiveAction` is the ISO 10.x CAPA action bound to a `NonconformityId`, with target date, implementation evidence, declared effectiveness criteria, review period, and reviewer.

They may **cite** each other (`remediationRefs` on CAPA). Closing one does not close the other. Incident fields were not retargeted.

### 4. State machine is fail-closed; one green test does not auto-close

```text
Open → Contained → RootCauseIdentified → CorrectiveActionPlanned
     → Implemented → EffectivenessReview → Closed
+ Cancelled | Superseded (accountable rationale)
+ Closed → Open (reopen with rationale)
```

IR methods: `open`, `contain`, `record_root_cause`, `classify`, `plan_corrective_action`, `mark_implemented`, `start_effectiveness_review`, `close`, `cancel`, `supersede`, `reopen`, `transition`. Library engine wraps propose/evaluate/close/query; it does not panic on illegal pairs (`CapaError`).

Missing RCA cannot leave `Contained`. Unclassified records cannot reach `CorrectiveActionPlanned`.

Effectiveness review **reads** `Effectiveness` / `ControlTestResult` and **reuses** Prompt 16 `VerificationMode` / window semantics. `EffectivenessCriteria` serde field is `window` (seconds), matching `VerificationPolicy` — not a distinct `windowSeconds` name. Default `SustainedWindow`, 14d, ≥2 greens, no intervening fail. A single green control test **must not** close CAPA unless declared criteria (`SingleGreenPermitted`) are satisfied **inside** the required review period — and `close` is still an explicit principal + time + rationale (`close_nonconformity` / `Nonconformity::close`).

Failed review forbids `Closed`. Repair is a legal transition back to `Implemented` or `CorrectiveActionPlanned` with a `ReviewFailed` history event.

### 5. Cancel, supersede, reopen, and immutable closure

Cancellation and supersession require non-empty accountable rationale (supersession also requires a successor id ≠ self). `Closed` / `Cancelled` / `Superseded` reject in-place mutation (`ImmutableClosure`). Reopen is a transition that **appends** history; it does not delete the prior closure. Reopen clears current `closure` / `effectiveness` fields; prior `Closed` remains in `history`. A new close still requires a fresh Satisfied review.

### 6. Consume neighbors; do not fork them

- Prompt 21: resolve `NonconformityRef` when inventory present; do not change finding creation.
- Prompt 19: consume `Incident`; do not invent a parallel incident type.
- Prompt 16: cite `RemediationRef`; do not replace the remediation engine.
- Prompt 15: consume existing `IsmsEventKind` names. **No** IR-inventory → `IsmsSnapshot` adapter landed; empty bags remain no-ops. `detect_isms_drift` was not rewritten.
- Compile flags: do **not** auto-enable `requests.nonconformities` or `supports_nonconformities`.
- Catalog: do **not** retarget `control.governance.corrective-action`.

Query helpers in `weeping-angel-assurance::capa`: `open_nonconformities`, `overdue_corrective_actions`, `failed_effectiveness_reviews`, `nonconformities_for_audit`, `nonconformities_for_incident`, `reopened_nonconformities`, `closed_nonconformities`. Overdue is a query fact, not an auto-transition.

### 7. Dual-suite law

Executable law is `tests/contracts/nonconformity_capa.{baseline,target}.rs` registered as `sdd_nonconformity_capa_{baseline,target}` in root `Cargo.toml`. Baseline characterizes pre-product absence and is skip-superseded after target GREEN. Neighbors listed in the spec header stay GREEN. Spec path is in `CANONICAL_SPECS`. No `tests/sdd/`. Traces go to `.sdd/runs` and `.sdd/artifacts`. `docs/sdd/` remains a stub.

## Non-goals

- Generic issue tracker, kanban, notifications.
- AI root-cause engine.
- Retargeting catalog TOML / ISO packs.
- Bumping `ASSURANCE_IR_SCHEMA`.
- Parallel incident/audit IRs.
- Auto-enabling compile flags.
- Ticket HTTP clients, SIEM, certification language.
- `ComplianceNodeRef` CAPA variants (optional, not landed).
- Automatic `IsmsSnapshot` population from IR inventories.

## Consequences

- Operators can prove containment → RCA → action → effectiveness window → closure on the same IR graph as audits, incidents, and remediations.
- Governance catalog continues to answer “do we attest a CAPA process?”; this ADR answers “what happened to *this* nonconformity?”
- Audit findings and incidents remain proposal sources, not silent classifiers.
- Prompt 16 remediations remain work records; CAPA actions remain management-system corrections.
- Public-contract pointer is live; dual-suite target is GREEN; baseline is skip-superseded.

## Related

- Spec: [`docs/specs/nonconformity-capa.md`](../specs/nonconformity-capa.md)
- Prompt: [`docs/prompts/operational-isms-v1/22-nonconformity-capa.md`](../prompts/operational-isms-v1/22-nonconformity-capa.md)
- Remediation: [`docs/specs/remediation-engine.md`](../specs/remediation-engine.md)
- Incident: [`docs/specs/incident-governance.md`](../specs/incident-governance.md)
- Internal audit: [`docs/specs/internal-audit.md`](../specs/internal-audit.md)
- Events/drift: [`docs/specs/isms-events-drift.md`](../specs/isms-events-drift.md)
- Governance catalog: [`docs/specs/governance-canonical-assurance-catalog.md`](../specs/governance-canonical-assurance-catalog.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
