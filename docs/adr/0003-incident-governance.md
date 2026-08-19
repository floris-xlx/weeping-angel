# ADR 0003 — Incident governance in assurance IR

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_incident_governance_target` GREEN; incidents live on `AssessmentDefinition.incidents`. |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Does **not** supercede the governance-catalog incident *capability* family (`control.incident.*`), IR schema `assurance-ir/v1`, ADR 0001 spine, or scanner `Finding`. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md) |
| Spec | [`docs/specs/incident-governance.md`](../specs/incident-governance.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_incident_governance_target` GREEN; `sdd_incident_governance_baseline` characterizes pre-incident HEAD and is expected RED on this product |

> Filename `0003-*` is shared with catalog-program siblings (cite this file by **path**). **0004** is documentation architecture. Parallel Operational ISMS drafts occupy `0005-*` / `0006-*` / `0007-*`; do not reuse those numbers for this decision.

## Context

On SHA `6e31bf1a…` Weeping Angel could test whether an organization *has* an IR plan, *ran* a tabletop, and *attested* a postmortem (governance catalog). It could not record a **specific** information-security incident against the same assets/risks/controls graph used by assurance.

HEAD facts at characterization:

1. No `Incident` / `IncidentId` in `weeping-angel-assurance-ir` or `src/`.
2. `AssessmentDefinition` had no `incidents` inventory.
3. Scanner `Finding` lives in `src/finding.rs` and is not IR.
4. `control.incident.{response-plan,exercise,postmortem}` and `evidence.incident.exercise` are **capability** catalog rows. Retargeting them as the operational register would collapse “we exercised this year” into “this outage.”

Operational ISMS v1 Prompt 19 requires a canonical incident record (stable id, detection source, classification, severity, status, affected graph, timeline, declared time, owner, containment, recovery refs, communications facts, evidence, root cause, lessons learned, control-failure links, risk links, corrective actions) **without** building a SIEM or replacing IR tooling.

Questions this decision answers:

1. Where does the operational incident live (IR type vs catalog control vs external ticketing)?
2. When does a finding/alert/event become an incident?
3. How do exercises differ from real incidents without forking the catalog family?
4. Is post-incident review a ticket comment or first-class evidence?
5. May closing an incident close corrective actions?
6. How is history preserved, and what remains out of product scope?

## Decision

Field names and tables are specified in [`docs/specs/incident-governance.md`](../specs/incident-governance.md). Landed in `weeping-angel-assurance-ir` + `weeping-angel-assurance`; schema stays `assurance-ir/v1`.

### 1. Same IR crate, new record, same schema version

The register **is** `weeping-angel-assurance-ir::Incident` stored on `AssessmentDefinition.incidents` (`serde(default)` empty). There is no GRC sidecar crate, `IncidentV2`, or `assurance-ir/v2`. JSON is camelCase. Canonical digest stays serde field order + BTree maps.

Canonical identity is `IncidentId` (`typed_id!`). External incident-system keys are `ExternalIncidentRef` adapters/pointers (`system`, `external_id`, optional `url`).

Incorrect: storing incidents only as `evidence.incident.exercise`; using ServiceNow/PagerDuty objects as the canonical id.

### 2. Promotion is explicit; observations are not incidents

Imported alerts, scanner `Finding`s, and Prompt 15 events (e.g. `ControlRegressed`) are **detection sources**. They become incidents only through `Incident::declare` / `Incident::promote` with a `PrincipalRef` and `declared_at`. `promote` is an alias of `declare`.

There is no `From<Finding> for Incident`, no severity auto-threshold, and no collector insert into `incidents`. `FindingRef` / `AlertRef` / `EventRef` are typed stable ids, not IR documents. `src/finding.rs` stays in the scanner crate.

`DetectionSource` is `Manual | Finding | Alert | AssuranceEvent | External`.

### 3. Exercise vs real is a kind on one type

```text
IncidentKind = Real | Exercise
```

Same status machine, timeline, and graph links. Kind is **not** a second struct and **not** a rewrite of `control.incident.exercise`. Governance-catalog tests remain capability/governance-only and must stay GREEN.

Closed **Real** incidents require post-incident review and recovery evidence. Closed **Exercise** incidents do not (PIR optional).

### 4. Post-incident review is first-class and may only propose

`PostIncidentReview` is a structured field (recorder, time, lessons learned, optional root cause, proposed risk/control/remediation ids, evidence refs). It does **not** mutate the risk register, control catalog, or remediation engine. Neighbor slices apply or reject proposals.

Missing PIR on `Real + Closed` fails closed (`validate_assessment_ir` and `incident_postmortem_missing`). Catalog `control.incident.postmortem` may still be assessed independently.

### 5. Incident close ≠ corrective-action close

Linked `corrective_action_ids` are `RemediationRef` (Prompt 16 id seam). Closing an incident with open corrective actions is **valid** and queryable via `closed_incidents_with_open_corrective_actions`. Auto-closing remediations from incident close is forbidden.

`AssessmentDefinition.remediations` is landed. When that inventory is **empty**, dangling corrective-action ids are stored, not fail-closed. When it is **non-empty**, `IncidentGraph.remediation_ids` must resolve them.

Control-failure links cite `ControlId` (+ optional test id / opaque `EventRef`). They do not set `Effectiveness`.

### 6. Timeline is ordered; history is append-only

`declare` seeds timeline (`TimelineKind::Declared` at `declared_at`) and history (`IncidentEventKind::Declared`, version 1). `validate()` requires non-decreasing timeline timestamps, Declared event matching `declared_at`, Detected ≤ declared, and the status transition table.

`revise` / `transition` append events and increment `version`. Past timeline/history bytes are not rewritten in place. Illegal transitions return `IncidentError::IllegalTransition` (no panic on library paths).

Communications/notification records are **facts** (`NotificationRecord`: who/when/channel/audience). This ADR does not authorize legal notification advice, statutory clocks, pagers, log pipelines, detection rules, or forensic tooling.

Status machine:

```text
Declared → Investigating | Contained | Cancelled
Investigating → Contained | Cancelled
Contained → Eradicated | Recovered | Cancelled
Eradicated → Recovered | Cancelled
Recovered → Closed | Investigating
Closed → Investigating
Cancelled → ∅
```

### 7. Graph integrity lives in `validate_assessment_ir`

Fail closed on duplicate `IncidentId`, dangling assets/processes/identities/risks/controls/tests, illegal transitions, unordered timelines, Real recovered/closed without recovery evidence, Real closed without PIR. Existing IR-019 and golden assessments without `incidents` keep working.

`ComplianceNodeRef::Incident(IncidentId)` is additive. Do not infer “control is effective” from a closed incident.

Audit/management-review **preparation** helpers live in `weeping-angel-assurance::incident_query`:

- `incidents_in_period`
- `incident_postmortem_missing`
- `closed_incidents_with_open_corrective_actions`
- `real_incidents` / `exercise_incidents`

Do not implement Prompt 21/23 here.

## Non-goals

- SIEM, detection rules, log ingestion, pager, forensics, breach-notification legal engines.
- Retargeting or rewriting `control.incident.*` / `evidence.incident.exercise`.
- Implementing Prompt 15 event-bus/drift product or Prompt 16 ticket product.
- UI, persistence service, GRC SaaS sync, ISO clause claims from an incident row.

## Consequences

- Incidents are first-class nodes on the same IR graph as assets, risks, and controls, consumable later by audit and management review.
- Governance catalog continues to answer “do we have a plan / did we exercise / did we attest reviews?”; this ADR answers “what happened in *this* incident?”
- Collectors and scanners stay observation-only.
- Dual-suite `sdd_incident_governance_{baseline,target}` is registered under `tests/contracts/`; spec path is in `CANONICAL_SPECS`. Neighbor suites `sdd_compliance_ir_target`, `sdd_governance_catalog_target`, `sdd_assurance_runtime_target`, and `sdd_documentation_layout` stay green.
- Traces go to `.sdd/runs` and `.sdd/artifacts`. `docs/sdd/` remains a stub. No `tests/sdd/`.

## Related

- Spec: [`docs/specs/incident-governance.md`](../specs/incident-governance.md)
- Prompt: [`docs/prompts/operational-isms-v1/19-incident-governance.md`](../prompts/operational-isms-v1/19-incident-governance.md)
- Governance catalog: [`docs/specs/governance-canonical-assurance-catalog.md`](../specs/governance-canonical-assurance-catalog.md)
- Risk register (consume `RiskId`): [`docs/specs/risk-register.md`](../specs/risk-register.md)
- Prompt 15 / 16: seams only (`EventRef`, `RemediationRef`)
- Layout: [ADR 0004](0004-documentation-architecture.md)
