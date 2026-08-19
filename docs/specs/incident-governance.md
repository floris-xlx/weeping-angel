# SDD: Incident Governance Engine (ISMS v1)

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_incident_governance_target` GREEN; `Incident` on `AssessmentDefinition.incidents`; baseline characterizes pre-incident HEAD (expected RED on this product) |
| Program | Operational ISMS v1 — incident governance |
| Slice | Canonical organizational information-security **incident record** on the existing assets/risks/controls/remediation graph; explicit promotion; first-class post-incident review; no SIEM |
| Dual-suite | `sdd_incident_governance_baseline` · `sdd_incident_governance_target` (`tests/contracts/incident_governance.{baseline,target}.rs`) — registered in root [`Cargo.toml`](../../Cargo.toml); directory is **not** auto-discovered (I3) |
| ADR | Accepted [`docs/adr/0023-incident-governance.md`](../adr/0023-incident-governance.md) — 0003-* sibling filename (cite by **path**) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) — incident-governance pointer; do not fork the spine |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) — this file is the human SSOT; `docs/sdd/` is a stub; traces go to `.sdd/runs` and `.sdd/artifacts` |
| Governance catalog (do **not** retarget) | [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) — `control.incident.{response-plan,exercise,postmortem}` + `evidence.incident.exercise` stay **capability/governance tests**, not this operational engine |
| Prompt | [`docs/prompts/operational-isms-v1/19-incident-governance.md`](../prompts/operational-isms-v1/19-incident-governance.md) |
| Consumes (seams) | assets; [`risk-register.md`](risk-register.md); [`control-implementation-registry.md`](control-implementation-registry.md); Prompt 15 `EventRef`; Prompt 16 `RemediationRef` |
| Neighbors (do not implement here) | Prompt 15 event bus / snapshot drift product; Prompt 16 ticket/remediation engine product; Prompt 21 internal audit; Prompt 22 CAPA (landed; consume, do not retarget `correctiveActionIds`); Prompt 23 management review |
| Collision fence | Catalog TOML, ISO packs, GitHub collector, `src/finding.rs`, existing `sdd_*` suites except this dual-suite / `documentation_layout.rs` registration |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `serde_json` struct field order + `BTreeMap` / `BTreeSet` (`canon/v1`) |
| Workspace verify | `cargo test --test sdd_incident_governance_target`; `cargo test --test sdd_documentation_layout`; keep `sdd_compliance_ir_target`, `sdd_governance_catalog_target`, `sdd_assurance_runtime_target` GREEN; `cargo test --workspace --features demo` when practical |

This document is the durable human SSOT for Operational ISMS v1 **incident governance**. It owns the **canonical `Incident` record**, **explicit declaration/promotion**, **timeline ordering**, **exercise vs real kind**, **post-incident review as first-class evidence**, **graph links** (assets, populations, control failures, risks, corrective actions), **external incident-system references**, and **immutable history**.

It does **not** own detection rules, log ingestion, paging, forensics, breach-notification legal advice, SIEM product behavior, the governance-catalog incident *capability* family, Prompt 15’s event/drift engine, or Prompt 16’s remediation/ticket engine.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

An incident is a **management-system record** over that graph. A scanner finding, imported alert, or Prompt 15 event is **not** an incident until a named principal **declares** it.

```text
Finding / imported alert / ControlRegressed (observation)
        ↓  explicit declare / promote (principal, time, rationale)
   Incident   (this slice)
        ├─ timeline, containment, recovery evidence
        ├─ linked ControlId failures + RiskId
        ├─ corrective-action refs (Prompt 16 seam)
        └─ PostIncidentReview  → may *propose* risk/control/remediation updates
```

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only. Do not write suites under `tests/sdd/`.

---

## 0. Collision fence (concurrent SDD)

This slice added IR `incident.rs` and `AssessmentDefinition.incidents`. It must **not** rewrite the governance catalog incident family, scanner findings, or unlanded Prompt 15/16 products.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps | Catalog / ISO remap |
| `control.incident.response-plan`, `control.incident.exercise`, `control.incident.postmortem`, `evidence.incident.exercise`, `test.incident.{plan-current,exercise-current,postmortem-recorded}` | Governance catalog — **keep GREEN**; do not retarget as the operational incident engine |
| `crates/weeping-angel-collector/src/github/**`, `tests/contracts/github_collector.*` | GitHub collector |
| `src/finding.rs` scanner `Finding` / `Severity` | Recon/scanner product; **not IR**; never auto-promote |
| Prompt 15 `ControlRegressed` event types, snapshot-diff product, generic event bus | ISMS events/drift — **consume event ids as opaque refs** if present; do not implement the bus |
| Prompt 16 `Remediation` type, Jira/Linear/GitHub ticket adapters, SLA engine | Remediation — **store typed/opaque corrective-action ids**; do not invent a ticket product |
| Operational `Risk` field expansion / status table | [`risk-register.md`](risk-register.md) — **consume `RiskId`**; do not fork `Risk` |
| `ControlImplementation` schema expansion | [`control-implementation-registry.md`](control-implementation-registry.md) |
| `tests/contracts/{compliance_ir,assurance_runtime,governance_catalog,documentation_layout}.*` rewrite | Existing suites — stay GREEN |
| ADR `0004-*`, `0005-*`, `0006-*` filenames | Documentation architecture and in-flight Operational ISMS drafts |

Landed product: IR module `incident.rs`; `IncidentId` / `FindingRef` / `AlertRef` / `EventRef` / `RemediationRef` via `typed_id!`; `lib.rs` re-exports; `AssessmentDefinition.incidents: Vec<Incident>` with `serde(default)`; `validation.rs` incident integrity; `weeping-angel-assurance::incident_query` (audit/management-review *preparation*); `ComplianceNodeRef::Incident(IncidentId)`.

Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** add `struct Finding` or `struct Alert` to the IR crate. Do **not** implement detection rules, log pipelines, pagers, forensic tooling, or legal notification engines.

---

## 1. Problem / user-visible goal

Weeping Angel can assess whether an organization *has* an incident-response plan, *ran* an exercise, and *attested* a postmortem (**governance catalog**). It cannot record that **this organization had (or rehearsed) a specific incident**, bind that record to assets/risks/controls, or feed audit and management-review preparation automatically.

On characterization SHA `6e31bf1a…`:

- there is no `Incident` / `IncidentId` in `weeping-angel-assurance-ir` or any product crate;
- `AssessmentDefinition` inventories stop at requirements, controls, mappings, evidence requirements, tests, implementations, scope, assets, identities, vendors, risks, exceptions, processing activities — **no incidents**;
- `Risk` is still a four-field stub (`id`, `title`, `description`, `status`);
- scanner `Finding` in `src/finding.rs` is recon output; nothing promotes it to ISMS state;
- Prompt 15 (events/drift) and Prompt 16 (remediation) exist as prompts/specs only;
- `ControlDomain::IncidentResponse` is a catalog domain tag, not an incident register;
- ISO pack still projects `iso27001:a.5.24` onto pack sliver `incident.response-process` (unrelated to this engine).

Without an operational record, a declared breach, a tabletop, a recovered service, and an un-reviewed ticket close are indistinguishable. Alerts look like incidents. Closing a ticket looks like lessons learned. Control regressions never attach to the event that revealed them.

**User-visible goal:** given an `AssessmentDefinition` that already holds assets, risks, controls, and remediations, an organization can **declare**, serialize, validate, and close an information-security incident such that:

```text
detection / source reference (finding, alert, event, external system, or manual)
  → explicit declare (principal + time)     // never automatic
  → IncidentId + kind (Real | Exercise)
  → classification, severity, status, response owner
  → affected assets / services / data / populations
  → ordered timeline + declared_at
  → containment + eradication/recovery evidence refs
  → communications/notification records (facts, not legal advice)
  → evidence/artifacts
  → root cause + PostIncidentReview (first-class; may propose graph updates)
  → linked control failures + linked risks + corrective-action refs
  → immutable history
```

Examples the engine must distinguish:

```text
Scanner Finding "unprotected-branch" exists
  → not an Incident

Imported alert / Prompt 15 ControlRegressed
  → not an Incident until declare()

declare(finding, principal, time)
  → one IncidentId; source retained; finding still a finding

timeline events out of chronological order
  → validate() error

Incident links ControlId that later regressed
  → control-failure ref resolves; incident is not the control test

Recovered/Closed Real incident without recovery evidence
  → fail closed

Closed Real incident without PostIncidentReview
  → missing-postmortem query / validation fail

IncidentKind::Exercise vs IncidentKind::Real
  → same record type; different kind; catalog exercise evidence is not this row

Incident status Closed + corrective action still Open
  → valid (incident close ≠ CAPA close)

mutate a past timeline event in place
  → forbidden; history is append-only
```

Definition of done (prompt): incidents feed the **same** risk/control/improvement graph and can be consumed automatically by audit and management-review **preparation** (Prompt 21/23 consume this inventory; they are not implemented here).

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| New `Incident` / `IncidentId` | `weeping-angel-assurance-ir` (`incident.rs` + `typed_id!(IncidentId)`) | **SSOT record.** Do not create a sidecar GRC crate or `IncidentV2`. |
| `AssessmentDefinition` | `assessment.rs` | Add `incidents: Vec<Incident>` with `#[serde(default)]`. `AssessmentDefinition::new` leaves it empty. Do not redesign other inventories. |
| `ValidateIr` | `validation.rs` | **Add** per-incident graph checks, duplicate `IncidentId`, timeline order, kind/postmortem rules. Keep IR-019 and existing duplicate-id checks. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for response owner, declarer, communicators. Do not invent `IncidentOwner`. |
| `Asset` / `AssetId` / `AssetKind::Service` | `asset.rs` | Affected assets and services. Services are assets, not a new type. |
| `ProcessingActivity` / `ProcessingActivityId` | `privacy.rs` | Affected data/processes. |
| `Identity` / `SubjectSelector` | identity / subject | Affected populations. Reuse selectors; do not invent a second population runtime. |
| `Control` / `ControlId` | `control.rs` | Linked control **failures** are `ControlId` (+ optional test/event ref). Canonical meaning stays on `Control`. |
| `Risk` / `RiskId` | `risk.rs` | Linked risks. Consume register fields if they have landed; do not expand `Risk` here. `RiskSource::Incident` (register spec) is the inverse tag — do not require it in this slice. |
| Scanner `Finding` | `src/finding.rs` | **Not IR.** `FindingRef` (or opaque stable id) on the incident source only. No `From<Finding> for Incident`. |
| Governance catalog incident family | `catalog/canonical/v1/{controls,evidence,tests}/governance.toml` | Unchanged. `evidence.incident.exercise` still means “a tabletop occurred in-window,” not “this `Incident` row.” |
| Prompt 15 events/drift | prompt / future module | Detection/source may cite an event id (`ControlRegressed`, …) as an **opaque ref**. Do not implement the event catalog, snapshot diff, or a message bus. |
| Prompt 16 remediation | [`remediation-engine.md`](remediation-engine.md) (landed) | Corrective actions are `RemediationRef` / `RemediationId`. Empty `remediations` skips resolve; non-empty inventory fail-closes dangling ids. This slice does not implement tickets, SLA, or kanban. |
| Evidence envelopes | `weeping-angel-evidence` | Recovery/containment/postmortem **refs** (requirement ids or opaque digests). Envelope crate stays conclusion-free. |
| `ComplianceNodeRef` | `crosswalk.rs` | Optional additive `Incident(IncidentId)`. Do not infer “control is effective” from a closed incident. |
| Golden IR fixtures | `tests/fixtures/assurance-ir/v1/**` | Existing fixtures have no `incidents` key; default empty must keep decoding. Do not require a golden incident fixture in this spec-first phase. |
| Neighbor suites | root `Cargo.toml` | `sdd_compliance_ir_target`, `sdd_governance_catalog_target`, `sdd_assurance_runtime_target`, `sdd_documentation_layout` stay GREEN. |
| Docs layout | ADR 0004 | Human SSOT is this file. Path is listed in `sdd_documentation_layout` `CANONICAL_SPECS`. |

Serde compatibility law:

- Existing assessment JSON **without** `incidents` deserializes (`#[serde(default)]`).
- New JSON is camelCase, matching IR.
- Empty vectors / `None` skip-serialize.
- Schema remains `assurance-ir/v1`.

Network-free. No ISO annex numbers as incident classification. No GRC product types (`ServiceNowIncident`, `PagerDutyIncident`) as the canonical record — only **external refs**.

---

## 3. Current behavior (baseline — characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` (pre-incident HEAD). `sdd_incident_governance_baseline` encodes this found case and is **expected RED** on the implemented product. `sdd_incident_governance_target` is GREEN.

This section remains the historical contract for that SHA. Product behavior is §4.

### 3.1 No incident IR

`crates/weeping-angel-assurance-ir/src/` has no `incident.rs`. `id.rs` `typed_id!` list has no `IncidentId`. `lib.rs` re-exports have no incident types. A workspace-wide search of product `crates/` and `src/` for an incident *engine* is empty (domain enum `ControlDomain::IncidentResponse` and catalog TOML only).

There is no `Incident`, `IncidentStatus`, `IncidentKind`, `PostIncidentReview`, `declare_incident`, or promotion API.

### 3.2 `AssessmentDefinition` has no incidents inventory

[`assessment.rs`](../../crates/weeping-angel-assurance-ir/src/assessment.rs) fields:

```text
requirements, controls, mappings, evidence_requirements, tests, requests,
implementations, scope, assets, identities, vendors, risks, exceptions,
processing_activities
```

`AssessmentDefinition::new` does not allocate an incidents vec. Assessment golden fixture has no `incidents` key.

`AssessmentRequests` flags include `risk_treatment`, `audit_program`, `nonconformities` — **not** incidents.

### 3.3 `Risk` remains a four-field stub

[`risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs): `id`, `title`, `description`, `status ∈ {Open, Accepted, Mitigated, Closed}`. Module docs: *“Minimal risk record. Not a risk engine.”* No incident linkage field on HEAD.

### 3.4 Validation never walks incidents

[`validation.rs`](../../crates/weeping-angel-assurance-ir/src/validation.rs) checks schema version, duplicate requirement/control/evidence ids, mappings, tests→controls, implementation→control/risk/exception (IR-019/020). Risk ids are collected as an **id bag** (duplicates silently collapse). There is no incident duplicate check, timeline check, or postmortem check.

### 3.5 Findings and alerts are not incidents

- `weeping-angel-assurance-ir` has **no** `Finding` / `Alert` type.
- `src/finding.rs` `Finding` is scanner output (`severity`, `url`, `module`, `cwe`, file evidence). Constructing or deserializing it does not create ISMS state.
- No imported-alert type exists in IR. Collectors that emit `security_finding` envelopes remain evidence, not incidents.

### 3.6 Governance catalog incident family is capability evidence, not a register

Already landed and tested by `sdd_governance_catalog_target`:

| Catalog id | Meaning on HEAD |
| --- | --- |
| `control.incident.response-plan` | Plan attested as a current artifact (`evidence.manual.attestation`) |
| `control.incident.exercise` | An exercise occurred in-window (`evidence.incident.exercise`) |
| `control.incident.postmortem` | Reviews recorded as attestations |
| `evidence.incident.exercise` | Fact: `exercised_at` + `exercise_kind` (tabletop / walkthrough / simulation) |

Those tests stay **governance-only**. They do not construct `Incident` records. Baseline must assert the catalog ids still exist **and** that they are not this slice’s engine (no retarget).

### 3.7 Prompt 15 / 16 are not product

No `ControlRegressed` event type, drift engine, or `Remediation` record in product crates at characterization. Specs/prompts may exist; baseline asserts absence of an incident-consuming event bus and of auto-closing tickets from incidents.

### 3.8 Crosswalk

`ComplianceNodeRef` is `Requirement | Control | Test | EvidenceRequirement | Risk | Exception`. No `Incident` variant.

### 3.9 Dual-suite not registered

Root `Cargo.toml` has no `sdd_incident_governance_{baseline,target}`. `tests/contracts/incident_governance.*.rs` do not exist on characterization HEAD.

---

## 4. Desired behavior (target)

### 4.1 Product home

The record lives in `weeping-angel-assurance-ir`. Landed layout:

```text
weeping-angel-assurance-ir
  incident.rs        # Incident, kind/status/severity/classification, timeline,
                     # post-incident review, declare/promote/transition/revise
  id.rs              # IncidentId, FindingRef, AlertRef, EventRef, RemediationRef
  assessment.rs      # incidents: Vec<Incident>
  validation.rs      # integrity + timeline + postmortem/recovery rules
  lib.rs             # re-exports
  crosswalk.rs       # ComplianceNodeRef::Incident

weeping-angel-assurance
  incident_query.rs  # incidents_in_period, incident_postmortem_missing,
                     # closed_incidents_with_open_corrective_actions,
                     # real_incidents, exercise_incidents
```

`Incident::declare` is the only constructor that creates a management-system incident. `Incident::promote` is an alias. Network-free. No provider SDK types. External systems appear only as `ExternalIncidentRef`.

### 4.2 Record kind vs catalog family

```text
IncidentKind = Real | Exercise     // JSON "real" | "exercise"
```

- **Same struct** for both. Kind is a field, not a second type and not a catalog control id.
- `IncidentKind::Exercise` is an operational **rehearsal record** (tabletop, walkthrough, simulation **instance**).
- `control.incident.exercise` / `evidence.incident.exercise` remain **capability** evidence (“an exercise occurred in the policy window”). Linking an exercise `Incident` to that evidence type is allowed; substituting the catalog row for this register is forbidden.
- Do not invent `control.incident.operational-register` in this slice. Do not rewrite governance TOML.

### 4.3 Canonical record (additive inventory)

JSON names are **camelCase**. All additive fields **default on deserialize** and **omit when empty** on serialize so old assessments stay valid.

| Field (Rust) | JSON | Required on declare | Semantics |
| --- | --- | --- | --- |
| `id` | `id` | yes | Stable `IncidentId` |
| `kind` | `kind` | yes | `Real` or `Exercise` |
| `title` | `title` | yes | Short name |
| `summary` | `summary` | no | Narrative |
| `classification` | `classification` | no | `IncidentClassification` (CIA / privacy / availability / other) — **not** an ISO clause |
| `severity` | `severity` | no | `IncidentSeverity` in **this** crate (do not import `weeping_angel::finding::Severity`) |
| `status` | `status` | default `declared` | §4.5 |
| `detection` | `detection` | yes (may be `Manual`) | Source reference §4.4 |
| `external_refs` | `externalRefs` | no | `Vec<ExternalIncidentRef>` §4.4 |
| `declared_at` | `declaredAt` | yes | Declaration timestamp (UTC) |
| `declared_by` | `declaredBy` | yes | `PrincipalRef` |
| `response_owner` | `responseOwner` | no | `PrincipalRef` |
| `asset_ids` | `assetIds` | no | Affected assets (includes services via `AssetKind::Service`) |
| `processing_activity_ids` | `processingActivityIds` | no | Affected data/processes |
| `population` | `population` | no | `Vec<SubjectSelector>` and/or identity ids — affected people/org units |
| `timeline` | `timeline` | seeded | Ordered `IncidentTimelineEvent` §4.6 |
| `containment` | `containment` | no | Containment record (when, who, summary, evidence refs) |
| `eradication_refs` | `eradicationRefs` | no | Evidence / action refs |
| `recovery_refs` | `recoveryRefs` | no | Evidence / action refs required for Real recovered/closed §4.7 |
| `communications` | `communications` | no | Notification **facts** §4.8 |
| `evidence_refs` | `evidenceRefs` | no | Artifacts: `EvidenceRequirementId` and/or opaque digests |
| `root_cause` | `rootCause` | no | Recorded cause (string or thin struct). Not an AI RCA engine |
| `lessons_learned` | `lessonsLearned` | no | May live on the PIR; duplicated top-level is optional |
| `post_incident_review` | `postIncidentReview` | no | First-class PIR §4.9 |
| `control_failure_refs` | `controlFailureRefs` | no | Linked control failures §4.10 |
| `risk_ids` | `riskIds` | no | Linked `RiskId`s |
| `corrective_action_ids` | `correctiveActionIds` | no | Prompt 16 refs §4.11 |
| `version` | `version` | default `1` | Monotonic revision of this `IncidentId` |
| `history` | `history` | seeded on declare | Append-only `IncidentEvent` §4.12 |
| `tags` | `tags` | no | `BTreeSet<String>` |

`Incident::declare(...)` is the **only** constructor that creates a management-system incident. `Incident::promote` is an alias of `declare`. There is no silent promotion path from findings.

### 4.4 Detection, alerts, and external systems — promotion is explicit

```text
DetectionSource =
    Manual
  | Finding(FindingRef)          // scanner / imported finding id; not src/finding.rs struct
  | Alert(AlertRef)              // imported alert id (opaque; no Alert document in IR)
  | AssuranceEvent(EventRef)     // Prompt 15 id, e.g. ControlRegressed — opaque if 15 absent
  | External(ExternalIncidentRef)

ExternalIncidentRef {
  system: String,                // "pagerduty" | "jira" | "servicenow" | ...  (label, not an adapter)
  external_id: String,           // their key
  url?: String
}

FindingRef / AlertRef / EventRef  // typed stable ids; validate_stable_id; not inventories on HEAD
```

Invariants:

1. Deserializing or constructing `src/finding.rs` `Finding` does **not** create an `Incident`.
2. There is no `From<Finding> for Incident`, no collector hook that inserts into `assessment.incidents`, and no “severity ≥ high ⇒ incident” rule.
3. An `AlertRef` / finding / Prompt 15 event may be cited by **zero or many** incidents only after `declare`.
4. `DetectionSource::Manual` with empty external refs is valid (phone call / human report).
5. External refs are **pointers**. Canonical `IncidentId` stays in Weeping Angel. This slice does not call PagerDuty/Jira/ServiceNow APIs.
6. Do **not** add `struct Finding` or `struct Alert` to `weeping-angel-assurance-ir`.

### 4.5 Status and transitions

```text
IncidentStatus =
    Declared              // JSON "declared"  — default after declare()
  | Investigating         // "investigating"
  | Contained             // "contained"
  | Eradicated            // "eradicated"
  | Recovered             // "recovered"
  | Closed                // "closed"
  | Cancelled             // "cancelled"  — false positive / withdrawn declaration
```

Fail-closed table (from → allowed to). Any other pair is an error.

| From | Allowed targets | Notes |
| --- | --- | --- |
| `Declared` | `Investigating`, `Contained`, `Cancelled` | May skip investigating into containment when containment is immediate |
| `Investigating` | `Contained`, `Cancelled` | Cannot jump to `Closed` |
| `Contained` | `Eradicated`, `Recovered`, `Cancelled` | Eradication may be N/A; skip to `Recovered` is allowed when recorded |
| `Eradicated` | `Recovered`, `Cancelled` | |
| `Recovered` | `Closed`, `Investigating` | Reopen to investigating if the issue returns |
| `Closed` | `Investigating` | Reopen only; not to `Declared` |
| `Cancelled` | ∅ | Terminal for this id. Replacement is a **new** `IncidentId` |

`fn IncidentStatus::can_transition(from, to) -> bool` and `Incident::transition(to, at, principal) -> Result<…>` are mandatory. Invalid transitions return a deterministic error (no panic on library paths). Recorded timeline/history status steps must satisfy the same table.

Exercise records use the **same** machine (a tabletop still has declared/closed). They do not skip PIR rules via a secret status.

### 4.6 Timeline ordering

```text
IncidentTimelineEvent {
  at: DateTime<Utc>,
  kind: TimelineKind,            // Detected | Declared | StatusTransition | Contained |
                                 // Eradicated | Recovered | Communicated | EvidenceAttached |
                                 // ReviewRecorded | Note
  principal?: PrincipalRef,
  detail?: String,
}
```

Rules:

1. `declare()` appends a `Declared` event with `at == declared_at`.
2. `validate()` requires `timeline` sorted non-decreasing by `at`. Out-of-order is an error naming the incident id.
3. Detection time, if present on `DetectionSource` metadata or a `Detected` event, must be `<= declared_at`.
4. Status-transition events must match §4.5 and the `status` field (last transition wins).
5. Events are **append-only**. There is no `timeline[i] = …` API. Corrections append a note or a compensating event; they do not rewrite bytes of prior events (§4.12).

### 4.7 Recovery evidence

For `IncidentKind::Real`:

| Status | Recovery evidence |
| --- | --- |
| `Recovered` or `Closed` | at least one `recovery_refs` entry **or** a containment/eradication evidence ref explicitly marked as recovered-in-place | **fail closed** if none |
| earlier statuses | recovery refs optional |

For `IncidentKind::Exercise`: recovery evidence is optional (rehearsal may not restore a real service). Target tests encode this split.

`recovery_refs` / `eradication_refs` / evidence artifacts are requirement ids or opaque digest strings (non-empty). IR does not open the ledger.

### 4.8 Communications / notifications (facts only)

```text
NotificationRecord {
  at: DateTime<Utc>,
  channel: String,               // "email" | "regulator" | "customer" | "internal" | ...
  audience: String,              // free text / principal / party id
  principal?: PrincipalRef,      // who sent it
  evidence_ref?: String,
}
```

This is an **audit fact**: a communication occurred. It is **not**:

- legal advice on whether a breach must be notified;
- a statutory clock engine;
- a mailer/pager.

Omitting `communications` is valid. Presence does not imply regulatory sufficiency.

### 4.9 Post-incident review is first-class evidence

```text
PostIncidentReview {
  recorded_at: DateTime<Utc>,
  recorded_by: PrincipalRef,
  root_cause?: String,           // may copy Incident.root_cause
  lessons_learned: String,       // required on the PIR document when PIR is present
  proposed_risk_ids: Vec<RiskId>,
  proposed_control_ids: Vec<ControlId>,
  proposed_corrective_action_ids: Vec</* RemediationRef or opaque */>,
  evidence_refs: Vec<…>,
}
```

Rules:

1. PIR is a structured field on `Incident`, not only a catalog attestation. Catalog `control.incident.postmortem` may still be assessed independently.
2. PIR **may propose** risk/control/remediation updates. Proposals do **not** mutate `assessment.risks` / controls / remediations. Apply/reject is the owning slice (register, 16, 22).
3. **Missing postmortem (target found-case):** `kind == Real` and `status == Closed` and `post_incident_review` is `None` → `validate()` or dedicated `incident_postmortem_missing(...)` **fails closed**. Prefer assessment validation so audit prep cannot silently omit it.
4. `kind == Exercise` and `status == Closed` **without** PIR is **valid** (exercise vs real). An after-action *may* still be stored as PIR; it is not required by this slice.
5. `Cancelled` does not require PIR.

### 4.10 Control-regression linkage

```text
ControlFailureRef {
  control_id: ControlId,
  test_id?: ControlTestId,
  event_ref?: EventRef,          // Prompt 15 ControlRegressed id when present
  snapshot_digest?: String,      // lineage/snapshot pin; opaque
}
```

Rules:

1. `control_id` must resolve in `assessment.controls`.
2. `test_id` if set must resolve in `assessment.tests`.
3. `event_ref` is a stable id string; **do not** require Prompt 15 types. If 15 later lands, the same string is the event id.
4. Linking a control failure does **not** change `Effectiveness` and does **not** rewrite control tests. The incident **cites** the regression.
5. An imported finding is still not a control failure until linked here (explicit).

### 4.11 Corrective actions and closed-with-open

`corrective_action_ids` are `RemediationRef` values pointing at Prompt 16 `RemediationId`. Prompt 22 CAPA `CorrectiveActionId` is a **different** type; this field was not retargeted. `AssessmentDefinition.remediations` is landed:

- store typed ids (`validate_stable_id`);
- when `assessment.remediations` is **empty**, dangling ids are stored, not fail-closed (old assessments / no remediations yet);
- when that collection is **non-empty**, dangling corrective-action ids fail closed.

**Closed incident with open corrective action is valid.** `IncidentStatus::Closed` does not require every linked remediation to be verified/closed. A query helper `closed_incidents_with_open_corrective_actions` (name flexible) must return those rows for audit/management-review prep. Auto-closing remediations from incident close is forbidden.

### 4.12 Immutable history

Chosen model: stable `IncidentId` + monotonic `version` + append-only `history: Vec<IncidentEvent>` + append-only `timeline`.

```text
IncidentEvent {
  version: u32,
  at: DateTime<Utc>,
  principal?: PrincipalRef,
  kind: Declared
      | FieldsRevised
      | StatusTransition { from, to }
      | ReviewRecorded
      | Cancelled,
}
```

Rules:

1. `declare` seeds history with `Declared` (version 1).
2. `revise` increments `version` and appends `FieldsRevised`. Prior title/status/severity remain recoverable from history payload or a retained prior snapshot. History is never cleared.
3. `transition` appends `StatusTransition` and fails if §4.5 forbids it.
4. In-place overwrite of a past `IncidentEvent` or `IncidentTimelineEvent` is not provided. Target tests may copy a record, mutate a historical timestamp, and show `validate()` failure **or** show that the public API cannot express the mutation without going through append.
5. Old assessments with no `history` key decode as empty **only** if they also have no incidents; newly declared incidents always have history.

This is in-memory IR + serde, not an event-sourcing database.

### 4.13 Reference integrity (fail closed)

On `AssessmentDefinition::validate()`, in addition to existing checks:

| Reference | Rule |
| --- | --- |
| Duplicate `Incident.id` | error |
| `asset_ids` | every id ∈ `assessment.assets` |
| `processing_activity_ids` | every id ∈ `assessment.processing_activities` |
| `population` identity ids | ∈ `assessment.identities` |
| `risk_ids` | ∈ `assessment.risks` |
| `control_failure_refs[].control_id` | ∈ `assessment.controls` |
| `declared_by` / `response_owner` = `Identity(id)` | ∈ `assessment.identities` |
| `Team` / `Role` | non-empty string |
| timeline order | §4.6 |
| history transitions | §4.5 |
| Real + Recovered/Closed | recovery evidence §4.7 |
| Real + Closed | PIR present §4.9 |
| `corrective_action_ids` | resolve when a remediation inventory exists |

Clockless `validate()` uses timestamps already stored on the record. It does not require a `Clock` trait (scheduler’s concern).

### 4.14 Consumption seams (audit / management review)

This slice does not implement Prompt 21/23. It **must** expose enough structure that those slices can later pull:

- incidents in a period (`declared_at` / timeline range);
- open vs closed vs cancelled;
- exercises vs real;
- missing PIR;
- closed-with-open corrective actions;
- linked risks and control failures.

Landed in `weeping-angel-assurance::incident_query`: `incidents_in_period`, `incident_postmortem_missing`, `closed_incidents_with_open_corrective_actions`, `real_incidents`, `exercise_incidents`. The query returns closed incidents that still cite corrective-action ids; it does not close remediations. Graph resolve of those ids is `validate_assessment_ir` when `remediations` is non-empty. Do not generate management-review minutes or audit conclusions here.

### 4.15 Serialization and digest

- Schema stays `assurance-ir/v1`.
- `canonical_digest` remains SHA-256 of `serde_json::to_vec` (struct field order + BTree maps).
- Maps/sets use `BTreeMap` / `BTreeSet`.
- `version` default 1.

---

## 5. Dual-suite protocol

Follow [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

| Suite | File | Cargo `[[test]]` name | On this product |
| --- | --- | --- | --- |
| Baseline | `tests/contracts/incident_governance.baseline.rs` | `sdd_incident_governance_baseline` | **RED** (characterizes pre-incident HEAD absence) |
| Target | `tests/contracts/incident_governance.target.rs` | `sdd_incident_governance_target` | **GREEN** (IG-001–IG-012) |

Protocol completed: spec + ADR, dual-suite registered, product landed, target GREEN, spec in `CANONICAL_SPECS`. Baseline remains the characterization of SHA `6e31bf1a…` and is not skip-superseded while it still encodes that found case.

One regression test per invariant titled from the spec ids below, encoding the **original found case** in baseline.

Do not add `tests/sdd/` or generated dumps under `docs/sdd/`.

---

## 6. Acceptance criteria (testable)

Target suite must encode at least:

- **IG-001 Alert not promoted.** Constructing/deserializing `src/finding.rs` `Finding` (or an imported `AlertRef` / evidence `security_finding`) does not insert into `assessment.incidents` and does not implement `From<Finding> for Incident`. Zero incidents until `declare`.
- **IG-002 Declared incident.** `declare` (principal + `declared_at` + source + kind) yields a stable `IncidentId`, `status == Declared`, detection retained, history seeded, serde round-trip, `canonical_digest` stable under `BTree` ordering.
- **IG-003 Timeline ordering.** Out-of-order timeline events fail `validate()`. Ordered events (detected ≤ declared ≤ contained ≤ recovered) pass. `declared_at` matches the Declared timeline event.
- **IG-004 Control-regression linkage.** `ControlFailureRef` with a known `ControlId` validates; dangling `ControlId` fails. Linking does not change control `Effectiveness`. Optional opaque `event_ref` (Prompt 15 seam) serializes without requiring the event type to exist.
- **IG-005 Recovery evidence.** Real + `Recovered`/`Closed` without recovery/eradication evidence fails. Real with `recovery_refs` passes. Exercise may close without recovery evidence.
- **IG-006 Missing postmortem.** Real + `Closed` without `postIncidentReview` fails. Real + `Closed` with PIR (lessons learned, recorded_by, recorded_at) passes. PIR proposals do not auto-insert risks/controls/remediations.
- **IG-007 Exercise vs real.** `IncidentKind::Exercise` vs `Real` is a record field on the same type. Catalog `control.incident.exercise` / `evidence.incident.exercise` still exist and are **not** this row. Exercise closed without PIR is valid; Real closed without PIR is not.
- **IG-008 Closed incident with open corrective action.** `status == Closed` with a linked corrective-action id that is still open (or unresolved because Prompt 16 is a seam) **validates**. Incident close does not close the action. Query lists the pair.
- **IG-009 Immutable history.** `revise` / `transition` append history and increment `version`; prior status/title remain represented. Public API does not rewrite past timeline events. Illegal transitions (`Declared → Closed`, `Cancelled → Declared`) fail.
- **IG-010 External incident-system refs.** `externalRefs` round-trip (`system` + `external_id`); canonical id remains `IncidentId`. No adapter calls.
- **IG-011 Graph integrity.** Duplicate `IncidentId` fails. Dangling `AssetId` / `RiskId` / identity owner fails. Existing assessments without `incidents` still decode. IR-019 still holds.
- **IG-012 Dual-suite registration.** `sdd_incident_governance_baseline` / `sdd_incident_governance_target` listed in root `Cargo.toml`; files live under `tests/contracts/`; this spec path is in `CANONICAL_SPECS` after implement.

Baseline suite must encode §3: no `Incident`/`IncidentId`; no `assessment.incidents`; four-field `Risk`; no auto-promotion from `Finding`; catalog incident tests remain governance-only; Prompt 15/16 not required as product.

---

## 7. Out of scope

- Detection rules, correlation engines, log ingestion pipelines, SIEM UX.
- Pager / on-call routing, Slack/Teams transport, email senders.
- Forensic imaging, malware analysis, packet capture tooling.
- Breach-notification **legal** advice, statutory deadline calculators, regulator form generation.
- Rewriting or retargeting `control.incident.*` / `evidence.incident.exercise` as this register.
- Implementing Prompt 15 event/drift product or a generic event bus.
- Implementing Prompt 16 remediation/ticket product, Jira/Linear/GitHub issue APIs.
- Implementing Prompt 21/22/23 (audit, CAPA, management review) beyond **consumable** fields/queries. CAPA is landed as `Nonconformity` / `CorrectiveAction`; incidents still only *propose* via Prompt 22 APIs.
- Moving scanner `Finding` into IR; auto-promotion by severity.
- Bumping `assurance-ir/v1`.
- UI, persistence service, GRC SaaS sync.
- Claiming ISO 27001 A.5.24 (or any clause) is satisfied because an `Incident` row exists.

---

## 8. Risks

- **Catalog collision.** Implementers may try to store operational incidents as `evidence.incident.exercise`. Mitigation: kind field on `Incident`; governance suite stays GREEN and unrewritten; tests assert catalog ids ≠ register API.
- **Silent promotion.** Collectors or Prompt 15 diffs could insert incidents. Mitigation: no `From<Finding>`; declare requires `PrincipalRef` + time; baseline/target IG-001.
- **Prompt 15/16 absence.** Over-fitting validation to types that do not exist will block implement. Mitigation: opaque refs; fail-closed resolution **only when** the neighbor inventory exists.
- **Closed-implies-done.** Operators may treat `Closed` as “all CAPA done.” Mitigation: IG-008; query for open corrective actions; docs forbid auto-close.
- **Exercise loophole.** Recording a real outage as `Exercise` to skip PIR/recovery. Mitigation: kind is explicit and auditable; this slice does not police honesty beyond the field; audit/management-review consume kind.
- **Legal overreach.** Notification records may be mistaken for legal sufficiency. Mitigation: spec §4.8; no statutory engine; out of scope list.
- **History vs digest.** Append-only vectors change canonical bytes. Tests pin fixtures; do not log unbounded debug events.
- **Exhaustive struct literals.** Adding `incidents` to `AssessmentDefinition` breaks in-tree literals. Mitigation: `new()` + serde default; fix literals in this slice; do not redesign the struct.
- **Severity type clash.** Reusing scanner `Severity` pulls the binary crate into IR. Mitigation: IR-local `IncidentSeverity`.

---

## 9. Landed files

Product:

- `crates/weeping-angel-assurance-ir/src/incident.rs`
- `crates/weeping-angel-assurance-ir/src/id.rs` (`IncidentId`, `FindingRef`, `AlertRef`, `EventRef`, `RemediationRef`)
- `crates/weeping-angel-assurance-ir/src/assessment.rs` (`incidents` inventory)
- `crates/weeping-angel-assurance-ir/src/validation.rs`
- `crates/weeping-angel-assurance-ir/src/lib.rs` re-exports
- `crates/weeping-angel-assurance-ir/src/crosswalk.rs` (`ComplianceNodeRef::Incident`)
- `crates/weeping-angel-assurance/src/incident_query.rs`

Tests/docs:

- `tests/contracts/incident_governance.baseline.rs`
- `tests/contracts/incident_governance.target.rs`
- root `Cargo.toml` `[[test]]` rows for both names
- `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` includes this file
- pointer in [`assurance-runtime.md`](assurance-runtime.md)
- Accepted ADR [`docs/adr/0023-incident-governance.md`](../adr/0023-incident-governance.md)

---

## 10. Definition of done

Weeping Angel has a canonical organizational incident record in `assurance-ir/v1` that:

- is created only by explicit declaration/promotion;
- distinguishes exercise vs real without hijacking the governance catalog family;
- carries timeline, owner, affected graph nodes, containment/recovery evidence, communication facts, PIR, control-failure links, risk links, and corrective-action refs;
- keeps history immutable;
- remains consumable by later audit and management-review preparation.

Dual-suite SDD protocol: spec + ADR, baseline characterizes pre-incident HEAD, target GREEN on this product, public-contract pointer landed. Baseline is expected RED here because the found case (no incident IR) no longer holds.
