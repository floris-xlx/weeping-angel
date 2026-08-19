# SDD: Risk Treatment Engine (ISMS v1)

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_risk_treatment_target` GREEN (P08-T01–T16); baseline skip-superseded (`#[ignore = "superseded by target suite"]`) |
| Program | Operational ISMS v1 — risk treatment |
| Slice | First-class `RiskTreatmentDecision` / `TreatmentPlan` / `TreatmentAction` / immutable `RiskAcceptance`; four strategies; fail-closed lifecycle |
| Dual-suite | `sdd_risk_treatment_baseline` · `sdd_risk_treatment_target` (`tests/contracts/risk_treatment.{baseline,target}.rs`) — **not auto-discovered**; listed in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0006-risk-treatment-engine.md`](../adr/0006-risk-treatment-engine.md). Numeric **0006** because methodology/register occupy `0005-*`. Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (Risk treatment section; do not fork the spine) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Consumes (types may land in parallel) | [`docs/specs/risk-methodology.md`](risk-methodology.md); [`docs/specs/risk-register.md`](risk-register.md); risk identification (spec if present) |
| Neighbors (do not implement here) | residual-risk effectiveness; control-implementation schema expansion; remediation records / tickets; operational SoA |
| Collision fence | Do not overwrite risk methodology, register, and identification specs/ADRs/contracts. Do not fork `Risk` or invent a 5×5 / global `RiskRating`. Catalog TOML, ISO packs, GitHub collector, existing `sdd_*` suites stay green. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `serde_json` struct field order + `BTreeMap` / `BTreeSet` (`canon/v1`) via `canonical_digest` |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; keep existing `sdd_*` GREEN |

This document is the durable human SSOT for risk treatment. It owns **treatment decisions**, **plans and actions**, **strategy-specific evidence (including immutable risk acceptance)**, **lifecycle state**, and **fail-closed reference / transition / expiry checks**. It does **not** own the operational `Risk` record (risk register), scoring scales/matrices (risk methodology), `RiskCandidate` promotion (risk identification), control-derived residual (residual risk), `ControlImplementation` schema expansion (control-implementation registry), or ticket/remediation documents (remediation engine).

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

A treatment path is a **management decision over that graph**. Selecting `Accept` / `Transfer` as an enum is not evidence that the organization accepted or transferred the risk. Collectors must not emit `RiskRating` or `RiskStatus::Accepted` as compliance evidence.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

---

## 0. Collision fence (concurrent SDD)

Parallel SDD runs may land risk methodology (methodology) as `spec-driven-development`, risk register (register) as `spec-driven-development-2`, and risk identification (identification) as `spec-driven-development-3` in this workspace. Those contracts are **consumed, not rewritten**.

| Do not touch | Owner |
| --- | --- |
| [`docs/specs/risk-methodology.md`](risk-methodology.md), [`docs/adr/0041-risk-methodology.md`](../adr/0041-risk-methodology.md), `risk_methodology.rs`, `score_risk`, scales/matrices | risk methodology |
| [`docs/specs/risk-register.md`](risk-register.md), [`docs/adr/0040-operational-risk-register.md`](../adr/0040-operational-risk-register.md), operational `Risk` fields / register status table | risk register — **consume `Risk` / `RiskId` / optional `treatment_id`** |
| risk identification `RiskCandidate` / promotion | risk identification — do not invent a second promotion engine |
| `catalog/canonical/v1/**`, ISO pack IDs / remaps | Catalog / ISO remap |
| `crates/weeping-angel-collector/src/github/**` | GitHub collector |
| `weeping-angel-assurance::applicability` Kleene engine | Already landed; treatment is not applicability |
| `src/contract/severity_policy.rs` | Scanner attack-path matrix |
| Existing `sdd_*` suite bodies except additive `Cargo.toml` / `documentation_layout.rs` registration | Stay GREEN |

Landed product adjustments: IR module `risk_treatment.rs`; typed ids in `id.rs`; `lib.rs` re-exports; `AssessmentDefinition.risk_treatments` (`serde(default)`, skip empty); `validation.rs` inventory integrity; consume register `Risk.treatment_id`.

Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** create a second `Risk` type. Do **not** expand `ControlImplementation` (control-implementation registry) or add `struct Remediation` (remediation engine).

---

## 1. Problem / user-visible goal

ISO-style ISMS operation requires an accountable **treatment path** for every open information-security risk: mitigate, accept, avoid, or transfer — with owner, principal, rationale, dates, target residual, linked controls, evidence, approval, and review. Today Weeping Angel has none of that.

On characterization SHA `6e31bf1a…`:

- `Risk` is four fields (`id`, `title`, `description`, `status`) with comment *“Minimal risk record. Not a risk engine.”*
- `RiskStatus::Accepted` is a freely assignable enum variant. No principal, no expiry, no immutable acceptance record. Setting `status = Accepted` is enough to *look* treated.
- There is no `RiskTreatmentDecision`, `TreatmentPlan`, `TreatmentAction`, or state machine.
- Dangling canonical `ControlId`s are validated only on implementations and mappings — never as treatment errors.
- `supports_risk_treatment` is a **compile capability flag**. ISO pack sets `risk_treatment = true`. `compile_framework` fail-closes if the assessment *requests* treatment and the target lacks the flag. The compiler does **not** evaluate, store, or audit a treatment path.
- Governance catalog attests `control.risk.treatment` / `evidence.risk.treatment` as **assurance catalog rows**, not a GRC engine.

An expired verbal “we accepted this” must not keep suppressing treatment. Selecting `Transfer` without a contract artifact must not complete. A half-finished mitigation must not become `completed`. A superseded plan must not remain the active path.

**User-visible goal:** every open risk *can* have an explicit, reproducible, auditable treatment path:

```text
RiskId
  → RiskTreatmentDecision { strategy, owner, principal, rationale, targetResidual, state }
       ├─ Mitigate → TreatmentPlan { actions, ControlId*, ControlImplementationId*, opaque remediation refs }
       ├─ Accept   → RiskAcceptance { principal, validFrom, expiresAt, immutable bytes }
       ├─ Avoid    → evidence that the organizational avoidance happened
       └─ Transfer → contract/instrument evidence (missing contract ⇒ fail closed)
```

Example the engine must distinguish:

```text
Risk status = Accepted, no principal, no expiry
  → not valid acceptance; treatment still required

Acceptance expires_at < as_of
  → must not suppress treatment_required

Mitigate plan: 2 of 3 required actions done
  → cannot complete

Transfer strategy, no contract evidence
  → cannot complete / cannot claim transferred

Decision A superseded by Decision B
  → A is not the active path

Approved target residual ≠ completion claim
  → fail closed

Plan cites ControlId absent from assessment.controls
  → treatment validation error (not a silent skip)
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Risk` / `RiskStatus` / `Risk::new` | `risk.rs` | **Do not fork.** If risk register has landed, consume expanded fields including optional `treatment_id: Option<RiskTreatmentId>`. If 06 is still the four-field stub, decisions still key by `RiskId`; do not invent `RiskV2`. |
| `RiskId` | `id.rs` | Keep. Add typed treatment/acceptance/plan/action ids (see §4.2). |
| `PrincipalRef` | `implementation.rs` | **Reuse** for owner, decision principal, approval, acceptance principal. Do not invent `TreatmentOwner`. |
| `ControlId` | `id.rs` / `control.rs` | Canonical control refs on mitigation. Must resolve in `assessment.controls`. |
| `ControlImplementationId` / `ControlImplementation` | existing stub | **Reference only.** Mitigation may list impl ids. Do **not** add control-implementation registry fields (populations, automation class, …). Resolve against `assessment.implementations` when an id is present. |
| `canonical_digest` / `typed_canonical_digest` | `digest.rs` | Reuse. No second digest. Acceptance immutability is byte-equality of the sealed acceptance body. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Do not fork. |
| `AssessmentDefinition` | `assessment.rs` | Additive `risk_treatments: Vec<…>` (name flexible) with `serde(default)`. Empty on old assessments. |
| `ValidateIr` | `validation.rs` | Keep IR-019 (implementation→risk). **Add** treatment integrity. |
| `FrameworkCapabilities.supports_risk_treatment` | `weeping-angel-framework` | Remains a compile **capability gate**. This slice does **not** make compile interpret treatment plans or SoA applicability. |
| `AssessmentRequests.risk_treatment` | `assessment.rs` | Unchanged request flag. |
| risk methodology scoring | spec [`risk-methodology.md`](risk-methodology.md); types exist in IR | Target residual is a frozen claim, not a `score_risk` call. v1 ships `VersionedPlaceholder` only (see §4.7). **No** crate-wide `enum RiskRating { High }`. |
| risk register register | spec [`risk-register.md`](risk-register.md) | `RiskStatus::Accepted` / `UnderTreatment` / `Mitigated` remain register statuses. This slice owns whether acceptance **evidence** is valid and whether it **suppresses** treatment requirements. Do not rewrite 06’s transition table. |
| risk identification | identification | Do not build `RiskCandidate` or promotion. |
| residual risk | residual risk | This slice **stores** a target residual claim. It does **not** calculate residual effectiveness from control tests. |
| control-implementation registry | control implementation registry | Absent as a schema expansion. Use existing `ControlImplementationId`. |
| remediation engine | [`remediation-engine.md`](remediation-engine.md) (landed) | This slice stores opaque `RemediationRef` only (`== RemediationId` string). It does **not** own `struct Remediation` or ticket adapters. |
| Golden `risk.json` | `tests/fixtures/assurance-ir/v1/risk.json` | Must keep decoding. |
| Exception state | `exception.rs` | Neighbor pattern (proposed/approved/expired/revoked) **without** a transition function. Treatment **does** have a transition function. Do not overload `Exception` as risk acceptance. |

Tiny allowed: new `typed_id!` aliases; serde defaults; validation messages; re-exports.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 Minimal `Risk` JSON still decodes

[`tests/fixtures/assurance-ir/v1/risk.json`](../../tests/fixtures/assurance-ir/v1/risk.json):

```json
{
  "id": "risk:source-tamper",
  "title": "Source tampering",
  "description": "Unauthorized change to the source of record.",
  "status": "open"
}
```

`sdd_compliance_ir_target` `ir_golden_fixtures_round_trip` decodes it. `Risk::new(id, title, description)` yields `status = Open`. Additive keys `owner`, `treatment` / `treatmentId`, `residualScore` are absent on constructor JSON (governance found-case, already ignored as a catalog invariant).

### 3.2 `RiskStatus::Accepted` is freely assignable

[`crates/weeping-angel-assurance-ir/src/risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs):

```text
RiskStatus = Open | Accepted | Mitigated | Closed   // camelCase JSON
#[default] Open

Risk { id, title, description, status }
```

There is **no** transition function. `status` is a public field. `RiskStatus::Accepted` requires no `PrincipalRef`, no rationale, no `expiresAt`, and no acceptance document. Nothing expires. Nothing un-suppresses treatment, because there is no treatment requirement query.

### 3.3 No treatment types or state machine

Product crate sources (`crates/**/src/**/*.rs`) contain none of:

- `RiskTreatmentDecision`, `TreatmentPlan`, `TreatmentAction`, `RiskAcceptance`
- `TreatmentStrategy` / `Mitigate` / `Avoid` / `Transfer` as IR treatment enums
- `TreatmentState` / `proposed` → `approved` → `executing` → `verification` → `completed`
- `fn` treating invalid treatment transitions as errors
- `RiskTreatmentId`, `RiskAcceptanceId`, `TreatmentPlanId`, `TreatmentActionId` in `id.rs`

`lib.rs` re-exports `risk::{Risk, RiskStatus}` only. There is no `mod risk_treatment`.

`Exception` has `Proposed | Approved | Expired | Revoked` and optional `approved_by` / `expires_at` but **no** `can_transition`, and it is a **control exception**, not risk acceptance.

### 3.4 Dangling control refs are not treatment errors

[`validation.rs`](../../crates/weeping-angel-assurance-ir/src/validation.rs):

- Implementation `control_id` must exist in `assessment.controls`.
- Implementation `risk_ids` must exist in `assessment.risks` (IR-019).
- Mapping endpoints must exist.
- **Does not** walk a treatment plan. There is no treatment inventory. A `ControlId` that would be cited by mitigation cannot fail as a *treatment* error.

### 3.5 `supports_risk_treatment` is compile-only

[`FrameworkCapabilities`](../../crates/weeping-angel-framework/src/lib.rs) includes `supports_risk_treatment: bool` (default `false`). [`AssessmentRequests.risk_treatment`](../../crates/weeping-angel-assurance-ir/src/assessment.rs) is the matching request. `validate_capabilities` emits `CapabilityViolation` named `supports_risk_treatment` when requested and unsupported.

ISO 27001 pack [`frameworks/iso-27001/2022/manifest.toml`](../../frameworks/iso-27001/2022/manifest.toml) sets `risk_treatment = true`.

`sdd_assurance_runtime_target` ACT-007 asserts default capabilities are all false, including `supports_risk_treatment`.

Compile does **not** construct `RiskTreatmentDecision` objects, evaluate expiry, or fail on missing contract evidence. Spine text: flag means “Risk/treatment objects in context” — the objects do not exist yet.

### 3.6 Catalog attestation ≠ engine

Governance catalog defines `control.risk.treatment`, `evidence.risk.treatment`, `test.risk.treatment-current`. Those are canonical assurance rows. They do not implement Mitigate/Accept/Avoid/Transfer workflows.

### 3.7 methodology / register / identification / implementation registry in this tree at characterization

| Surface | Product code | Human spec |
| --- | --- | --- |
| risk methodology methodology types / `score_risk` | Absent | [`docs/specs/risk-methodology.md`](risk-methodology.md) (Specified) |
| risk register operational `Risk` | Absent (four-field stub) | [`docs/specs/risk-register.md`](risk-register.md) (Specified) |
| risk identification `RiskCandidate` | Absent | spec file only |
| control-implementation registry implementation registry expansion | `ControlImplementation` stub only | spec file only |
| risk treatment treatment | **Absent** | this document |

Baseline tests must pin the **code** found-case, not the neighboring specs.

### 3.8 What current tests lock (must stay green)

- `sdd_compliance_ir_target` IR-019 dangling `RiskId`; golden `risk.json`.
- `sdd_assurance_runtime_target` default `supports_risk_treatment == false`; capability gate.
- `sdd_governance_catalog_target` catalog ids including `control.risk.treatment`.
- Neighbor dual-suites (applicability, lineage, catalogs, typed evidence, population, ISO).

---

## 4. Desired behavior (target)

### 4.1 Product home

Treatment lives in **`weeping-angel-assurance-ir`** (`src/risk_treatment.rs`). Network-free. No ISO annex numbers, no provider SDK types, no GRC product names (`Jira`, `ServiceNow`) in generic IR.

Suggested layout (names flexible; semantics not):

```text
weeping-angel-assurance-ir
  risk.rs              # risk register SSOT — consume, do not fork
  risk_treatment.rs    # this slice
  id.rs                # additive typed ids
  validation.rs        # treatment graph + transitions + expiry-at-time
  implementation.rs    # PrincipalRef (consumed)
  assessment.rs        # additive risk_treatments inventory
  (risk methodology modules if present — consumed for target residual)
```

Do **not** put the engine in `weeping-angel-framework` compile, `weeping-angel-assurance` Kleene applicability, collectors, or control-test `TestExpr`.

Scoring remains risk methodology. Register status remains risk register. This module answers: *what is the accountable treatment path, is it legal, is acceptance still in force, is evidence sufficient for this strategy?*

### 4.2 Typed identifiers

Add via existing `typed_id!` (same charset / length / no uuid-v4 rules as `RiskId`):

| Type | Typical prefix (documentary) | Identifies |
| --- | --- | --- |
| `RiskTreatmentId` | `rt:` | A `RiskTreatmentDecision` |
| `RiskAcceptanceId` | `ra:` | An immutable `RiskAcceptance` |
| `TreatmentPlanId` | `tp:` | A `TreatmentPlan` |
| `TreatmentActionId` | `ta:` | A `TreatmentAction` |

risk register already reserves `RiskTreatmentId` on `Risk.treatment_id`. If 06 lands first without the type, this slice **owns** adding `typed_id!(RiskTreatmentId)` once. If 06 already added it, **do not duplicate**.

Optional opaque ref (not a document):

```text
RemediationRef   // typed_id! or newtype over validated stable id
```

No `struct Remediation` **in this slice** (Prompt 16 owns it). No finding promotion types (risk identification).

### 4.3 Strategies

```text
TreatmentStrategy = Mitigate | Accept | Avoid | Transfer
```

Serde `camelCase`. Exhaustive. Unknown JSON tags fail closed.

Selecting the enum is **never** sufficient to complete the path. Each strategy has **mandatory evidence / structure** (§4.8–§4.11).

### 4.4 Core records

JSON names **camelCase**. Schema `assurance-ir/v1`. Optional/empty skip-serialize.

```text
TreatmentState =
  Proposed
  | Approved
  | Executing
  | Verification
  | Completed
  | Cancelled
  | Superseded

RiskTreatmentDecision {
  schemaVersion: "assurance-ir/v1"
  id: RiskTreatmentId
  riskId: RiskId
  strategy: TreatmentStrategy
  state: TreatmentState                 // default Proposed
  owner: PrincipalRef                   // accountable execution owner
  decisionPrincipal: PrincipalRef       // who decided the strategy
  rationale: String                     // non-empty when leaving Proposed (normative: required on Approved+)
  targetDate: DateTime<Utc>? 
  targetResidual: TargetResidualRisk    // §4.7
  canonicalControlIds: [ControlId]      // especially Mitigate; may be empty for Accept
  implementationIds: [ControlImplementationId]
  remediationRefs: [RemediationRef]     // opaque; remediation engine
  evidenceExpectations: [TreatmentEvidenceExpectation]
  approval: TreatmentApproval?          // required to enter Approved
  reviewAt: DateTime<Utc>?              // next governance review of this path
  expiresAt: DateTime<Utc>?             // path-level expiry (Accept uses acceptance.expiresAt as the suppression clock)
  plan: TreatmentPlan?                  // required for Mitigate; optional extra actions for Avoid/Transfer
  acceptance: RiskAcceptance?           // required for Accept
  avoidEvidence: TreatmentEvidenceRef?  // required for Avoid to leave Verification→Completed
  transferEvidence: TransferEvidence?   // required for Transfer
  supersedes: RiskTreatmentId?
  supersededBy: RiskTreatmentId?
  version: u32                          // default 1
  history: [TreatmentEvent]
  approvedTargetResidualDigest: String? // set on Proposed → Approved
  sealedAcceptanceDigest: String?       // Accept: freeze of RiskAcceptance bytes
}

TreatmentPlan {
  id: TreatmentPlanId
  owner: PrincipalRef
  actions: [TreatmentAction]            // Mitigate: ≥ 1 required action
  targetDate: DateTime<Utc>?
}

TreatmentAction {
  id: TreatmentActionId
  title: String                         // non-empty
  owner: PrincipalRef
  required: bool                        // default true
  state: ActionState                    // Proposed | InProgress | Done | Cancelled
  controlIds: [ControlId]
  implementationIds: [ControlImplementationId]
  remediationRefs: [RemediationRef]
  evidence: [TreatmentEvidenceRef]
  dueAt: DateTime<Utc>?
}

ActionState = Proposed | InProgress | Done | Cancelled

TreatmentApproval {
  principal: PrincipalRef
  at: DateTime<Utc>
  note: String?
}

TreatmentEvidenceExpectation {
  id: EvidenceRequirementId?            // if set, must resolve in assessment.evidence_requirements
  evidenceType: EvidenceType?           // documentary type, e.g. "risk.treatment"
  criticality: EvidenceCriticality      // reuse IR enum; Required vs Supporting
  description: String
}

TreatmentEvidenceRef {
  kind: EnvelopeDigest | EvidenceRequirement | NarrativeAttestation
  value: String                         // digest / id / attestation body id
  at: DateTime<Utc>?
  principal: PrincipalRef?
}

TreatmentEvent {
  version: u32
  at: DateTime<Utc>
  principal: PrincipalRef?
  kind: Created
      | FieldsRevised
      | StateTransition { from: TreatmentState, to: TreatmentState }
      | Superseded { successor: RiskTreatmentId }
}
```

`Risk::new` / old `risk.json` **must not** start requiring these objects. Assessments with empty `risk_treatments` remain valid.

Public construction: `RiskTreatmentDecision::propose(...)` (name flexible) starts in `Proposed`. State changes go through `transition` so history is appended. Direct field mutation that records an illegal `history` pair fails `validate()`.

### 4.5 State machine (fail closed)

Happy path:

```text
proposed → approved → executing → verification → completed
```

Also: `cancelled`, `superseded`.

Normative table (from → allowed to). Any other pair is an error (`TreatmentError::InvalidTransition`). Library paths must not panic.

| From | Allowed targets |
| --- | --- |
| `Proposed` | `Approved`, `Cancelled` |
| `Approved` | `Executing`, `Cancelled`, `Superseded` |
| `Executing` | `Verification`, `Cancelled`, `Superseded` |
| `Verification` | `Completed`, `Executing` (verification failed / more work), `Cancelled`, `Superseded` |
| `Completed` | `Superseded` |
| `Cancelled` | ∅ |
| `Superseded` | ∅ |

Guards (still fail closed even if the edge exists in the table):

1. `Proposed → Approved` requires `approval` (principal + time) **and** non-empty `rationale` **and** `decisionPrincipal`.
2. `Approved → Executing` is allowed for all four strategies. Accept/Avoid/Transfer still walk `executing` (recording strategy evidence is work, not a skip).
3. `Verification → Completed` requires strategy completion predicates (§4.8–§4.11) **and** target-residual identity with the value frozen at approval (§4.7).
4. `* → Superseded` requires `supersededBy` set to a different `RiskTreatmentId` that exists in the inventory and cites `supersedes = this.id`.
5. `Cancelled` and `Superseded` are terminal for that id.

```text
TreatmentState::can_transition(from, to) -> bool
RiskTreatmentDecision::transition(to, principal, at) -> Result<Self, TreatmentError>
```

All four strategies use this machine. There is no `Approved → Completed` shortcut.

### 4.6 Inventory, uniqueness, active path

`AssessmentDefinition` gains:

```text
risk_treatments: Vec<RiskTreatmentDecision>   // serde default empty
```

Laws:

1. Duplicate `RiskTreatmentId` fails `validate()`.
2. Nested plan/action/acceptance ids unique within the assessment.
3. `riskId` must resolve in `assessment.risks` (fail closed). If risk register is still the stub, the four-field `Risk` is enough.
4. At most **one active** decision per `RiskId`. Active = state ∈ {`Proposed`, `Approved`, `Executing`, `Verification`}. `Completed` / `Cancelled` / `Superseded` are inactive. A second active decision fails unless it is the in-progress successor that already points `supersedes` at the previous **and** the previous is already `Superseded` (ordering: supersede old first).
5. When risk register `Risk.treatment_id` is present and `Some`, it must equal the **active** decision id if one exists, or the latest `Completed` id if none is active. A `treatment_id` that does not resolve is already risk register RR-006; this slice supplies the resolving inventory so that check can pass.

Query APIs (names flexible):

```text
active_treatment(assessment, risk_id) -> Option<&RiskTreatmentDecision>

treatment_required(assessment, risk_id, as_of) -> bool
  // true when the risk is not Closed/Retired (if those statuses exist)
  // AND there is no currently suppressing path
  // Suppressing paths:
  //   - Completed Mitigate/Avoid/Transfer with strategy evidence still valid
  //   - Accept whose RiskAcceptance is in force at as_of (§4.9)
  // Expired acceptance ⇒ treatment_required == true even if Risk.status == Accepted
```

Open / draft / under-treatment risks without an active or valid completed path are `treatment_required`. This is a **query**, not a requirement that every fixture invent a plan.

### 4.7 Target residual (risk methodology shaped; not residual risk)

```text
TargetResidualRisk =
  VersionedPlaceholder {        // shipped v1 (even though methodology types exist)
      methodologyVersion: String,  // non-empty pin, not a rating enum
      inputNote: String?           // raw description; not "High"
    }
```

Shipped decision: this slice does **not** call `score_risk` or store `MethodologyScored`. Scoring remains risk methodology; residual effectiveness remains residual risk. `VersionedPlaceholder.methodologyVersion` is a pin (e.g. `rm:acme-default:2`). A later additive variant may store a methodology snapshot; it must not replace this variant or invent `enum RiskRating`.

Laws:

1. **Do not** add `enum RiskRating { Low, Medium, High }` in this crate for treatment.
2. Collectors must not construct target residual. Target tests grep `weeping-angel-collector` for `RiskTreatmentDecision` / `TargetResidualRisk` / `RiskAcceptance`.
3. `methodologyVersion` is required and non-empty. It is **not** a collector-emitted rating.
4. Target residual is **frozen at `Approved`** (`approved_target_residual_digest`). Completing with a different `canonical_digest` of `targetResidual` is `TargetResidualMismatch`.
5. This slice does **not** recompute residual from control-test `Effective` / `Ineffective` (residual risk). Tests must not assert that completed mitigation lowers a score.

### 4.8 Mitigate

Required to `Verification → Completed`:

1. `strategy = Mitigate`.
2. `plan` present with **≥ 1** `required: true` action.
3. Every required action is `Done` (partial completion **blocks** complete). Optional (`required: false`) actions may remain open.
4. Every `ControlId` on the decision, plan, and actions ∈ `assessment.controls` (dangling ⇒ `DanglingControlReference`).
5. Every `ControlImplementationId` ∈ `assessment.implementations` (dangling ⇒ same class of error). Unresolved impl ids are treatment errors even though control-implementation registry schema is not implemented.
6. `RemediationRef` values must be well-formed stable ids. There is **no** remediation inventory; absence of a remediation engine document is not an error.
7. `evidenceExpectations` with `criticality = Required` must have at least one `TreatmentEvidenceRef` attached on the decision or on the completing actions.

Mitigation **may** reference multiple implementations and actions. One canonical control with two `ControlImplementationId`s is valid.

Partially complete: 1 of 2 required actions `Done` ⇒ `transition(Completed)` fails. State may still be `Executing` or `Verification`.

### 4.9 Accept — immutable governance evidence

```text
RiskAcceptance {
  id: RiskAcceptanceId
  riskId: RiskId
  treatmentId: RiskTreatmentId
  principal: PrincipalRef          // accountable; required
  rationale: String                // non-empty
  approvedAt: DateTime<Utc>
  validFrom: DateTime<Utc>         // default approvedAt
  expiresAt: DateTime<Utc>         // required — no open-ended suppression
  reviewAt: DateTime<Utc>?         // must be ≤ expiresAt if both set
  evidence: [TreatmentEvidenceRef] // ≥ 1
  digest: String                   // canonical_digest of the sealed body
}
```

Laws:

1. `strategy = Accept` requires `acceptance` present before `Approved` (may be drafted in `Proposed` but incomplete).
2. After first successful `Proposed → Approved`, the acceptance **body** (all fields except nothing — the whole struct used for `digest`) is **immutable**. Further edits fail `TreatmentError::ImmutableAcceptance`. Evolution = new treatment decision that **supersedes** this one, plus a new `RiskAcceptance` id.
3. `principal` is mandatory. Team/Role/Identity allowed (`PrincipalRef`). Identity ids must resolve in `assessment.identities`.
4. `expiresAt` required. `validFrom < expiresAt`.
5. **In force** at `as_of` iff `state` of the parent decision is `Completed` (or at least `Approved` **and** acceptance sealed — normative: suppression begins only when the path is `Completed`, so an approved-but-unexecuted accept does not hide an open risk) **and** `validFrom ≤ as_of < expiresAt` **and** parent not `Cancelled`/`Superseded`.
6. **Expired acceptance must never continue suppressing treatment requirements.** `treatment_required(..., as_of ≥ expiresAt) == true` even if `Risk.status == Accepted`.
7. `RiskStatus::Accepted` (risk register) is **not** itself the evidence. This slice may provide `acceptance_in_force(...)` for 06/callers. It must **not** silently write `Risk.status = Accepted` without a sealed acceptance (if this slice offers a helper that updates register status, it fail-closes without principal/expiry). If 06 `transition(Accepted)` exists, this slice’s validation of an assessment **fails** when any risk has `status == Accepted` and `acceptance_in_force` is false at a provided `as_of`, via `validate_treatments_at(assessment, as_of)`. Clockless `AssessmentDefinition::validate()` does not require a clock; it still rejects `Accepted` with **missing** acceptance record. Expiry is the clocked API.
8. Reusing `Exception` as acceptance is forbidden.

### 4.10 Avoid

Selecting `Avoid` is not completion.

`Verification → Completed` requires `avoidEvidence` present with a non-empty `TreatmentEvidenceRef` (envelope digest, requirement id, or narrative attestation bound to a `PrincipalRef` + time) that demonstrates the **organizational action happened** (decommission, stop processing, remove from scope — documentary; this slice does not interpret applicability).

Empty evidence ⇒ fail closed (`MissingStrategyEvidence`).

### 4.11 Transfer

Selecting `Transfer` is not completion.

```text
TransferEvidence {
  contract: TreatmentEvidenceRef    // required — insurance, DPA, assignment, etc.
  transferee: String                // documentary counterparty name (non-empty); shipped as String
  effectiveAt: DateTime<Utc>?
}
```

Missing `contract` (absent, empty value) ⇒ `MissingContractEvidence`. Cannot complete. Cannot claim the risk is transferred.

This is **not** a ticket (`remediation engine`) and **not** a vendor SDK object.

### 4.12 Approval, review, expiry (path level)

- `TreatmentApproval` required on `Proposed → Approved`.
- `reviewAt` is documentary/schedule; overdue review does **not** by itself complete or cancel. Clocked helper `path_review_overdue(as_of)` is true when `reviewAt < as_of` and state is not terminal.
- Path `expiresAt` if set: after expiry, a `Completed` Mitigate/Avoid/Transfer **no longer suppresses** `treatment_required` (same spirit as acceptance). If unset, those completed paths keep suppressing until superseded/cancelled. Accept **always** uses `acceptance.expiresAt` (required).

### 4.13 Supersession

1. Successor `propose` with `supersedes = old.id`.
2. `old.transition(Superseded)` sets `supersededBy = successor.id`.
3. Old plan/acceptance remain in the inventory for audit (immutable bytes). They are not the active path.
4. `treatment_required` looks at the **active** path and in-force acceptance of non-superseded completed accepts only.

A superseded Accept, even if `as_of` is inside the old `expiresAt`, does **not** suppress treatment.

### 4.14 Dangling references (fail closed)

On `AssessmentDefinition::validate()` in addition to IR-019:

| Reference | Rule |
| --- | --- |
| `decision.riskId` | ∈ `assessment.risks` |
| `canonicalControlIds` / action `controlIds` | ∈ `assessment.controls` |
| `implementationIds` | ∈ `assessment.implementations` |
| `EvidenceRequirementId` on expectations | ∈ `assessment.evidence_requirements` |
| `owner` / principals `Identity(_)` | ∈ `assessment.identities` |
| `Team` / `Role` | non-empty string |
| `supersedes` / `supersededBy` | those treatment ids exist; no self-supersession |
| Duplicate treatment/plan/action/acceptance ids | error |
| `RemediationRef` | `validate_stable_id` only |

Envelope digests are well-formed non-empty strings; IR does not open the evidence ledger.

### 4.15 Serialization and digest

- `#[serde(rename_all = "camelCase")]`.
- `canonical_digest` / `typed_canonical_digest("risk-treatment", …)` / `typed_canonical_digest("risk-acceptance", …)`.
- Acceptance `digest` field is SHA-256 hex of `canonical_digest` of the body **excluding** the `digest` field itself (or a dedicated `AcceptanceSealBody`). Tests pin stability.
- `BTreeSet`/`BTreeMap` for unordered collections so insert order does not change bytes.
- Do not change `Risk::new` JSON keys.

### 4.16 Capability flag

`supports_risk_treatment` stays a **compile-time capability**. This slice does **not**:

- interpret framework applicability from treatment;
- generate SoA rows from plans (operational SoA);
- fail `compile_framework` because a plan is partial (that is IR `validate` / treatment API).

Optional (not required): if `requests.risk_treatment == true` and the assessment includes `risk_treatments`, compile still only checks the capability flag. Keep ACT-007 semantics.

### 4.17 Interaction with risk register status

| Register status (06) | Treatment engine |
| --- | --- |
| `Open` / `Draft` | `treatment_required` unless a suppressing completed path exists (unusual for draft) |
| `UnderTreatment` | expects an **active** decision; validation-at-time may warn/fail if none |
| `Accepted` | valid iff `acceptance_in_force`; else fail clocked validate / `treatment_required` |
| `Mitigated` | does not prove residual (residual risk); this slice only checks a completed Mitigate path exists if the caller asks |
| `Closed` / `Retired` | `treatment_required` false |

This slice **must not** replace 06’s `can_transition` table. Helpers may *recommend* `UnderTreatment` when a decision becomes `Executing`, but register writes stay 06’s API.

If 06 is not landed, do not add those statuses here. Engine still runs on `RiskId` + stub `Risk`.

---

## 5. Dual-suite protocol

Follow [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

| Suite | File | Cargo `[[test]]` name | Status |
| --- | --- | --- | --- |
| Baseline | `tests/contracts/risk_treatment.baseline.rs` | `sdd_risk_treatment_baseline` | skip-superseded (`#[ignore = "superseded by target suite"]`) |
| Target | `tests/contracts/risk_treatment.target.rs` | `sdd_risk_treatment_target` | **GREEN** (P08-T01–T16) |

```toml
[[test]]
name = "sdd_risk_treatment_baseline"
path = "tests/contracts/risk_treatment.baseline.rs"

[[test]]
name = "sdd_risk_treatment_target"
path = "tests/contracts/risk_treatment.target.rs"
```

Protocol (mandatory; **complete**): spec first, baseline GREEN on characterization, target RED then GREEN, ADR Accepted, baseline absence tests skip-superseded, `CANONICAL_SPECS` registered, target still GREEN.

One regression test per invariant. Titles: `P08: <exact subject>`.

Traces: `.sdd/runs/` and `.sdd/artifacts/` only.

---

## 6. Acceptance criteria (testable)

### 6.1 Baseline suite (additive GREEN; absence tests skip-superseded)

Encode **current** HEAD. Titles `P08: …` for the found case.

| ID | Assertion |
| --- | --- |
| P08-B01 | Minimal `tests/fixtures/assurance-ir/v1/risk.json` decodes; `id == "risk:source-tamper"`; `status == Open` |
| P08-B02 | `RiskStatus::Accepted` is assignable on a `Risk` value with no principal, rationale, or `expiresAt`; serialized JSON has no acceptance object |
| P08-B03 | Product crate sources have no `struct RiskTreatmentDecision`, `TreatmentPlan`, `TreatmentAction`, `RiskAcceptance` |
| P08-B04 | Product sources have no treatment state machine (`TreatmentState` / `proposed`→`completed` transition API) |
| P08-B05 | `id.rs` has `typed_id!(RiskId)` and does not define `RiskAcceptanceId` / `TreatmentPlanId` / `TreatmentActionId`. `RiskTreatmentId` is absent **unless** risk register already added it — if 06 added it, baseline asserts the type exists **and** no treatment module uses it |
| P08-B06 | `AssessmentDefinition::validate()` does **not** fail a dangling `ControlId` that is only imagined as a treatment ref (no treatment walk). IR-019 still fails `risk:missing` on implementations |
| P08-B07 | `supports_risk_treatment` default false; compile capability check still names the flag; no compile stage builds treatment decisions |
| P08-B08 | `lib.rs` has `mod risk` and no `mod risk_treatment` |
| P08-B09 | Collision fence: this suite does not rewrite risk methodology/register specs; collector sources have no treatment types |

### 6.2 Target suite (GREEN)

Executable contract for the shipped engine.

| ID | Title / assertion |
| --- | --- |
| P08-T01 | `P08: Mitigate strategy with plan and actions completes` — required actions `Done`, controls resolve, `Verification → Completed` succeeds; digest stable |
| P08-T02 | `P08: Accept strategy seals immutable acceptance with principal` — `Proposed → … → Completed`; principal + `expiresAt` required; post-approve mutation fails |
| P08-T03 | `P08: Avoid strategy requires organizational action evidence` — enum alone cannot complete; evidence ref allows complete |
| P08-T04 | `P08: Transfer strategy requires contract evidence` — missing contract fails; present contract + transferee completes |
| P08-T05 | `P08: expired risk acceptance does not suppress treatment` — `as_of ≥ expiresAt` ⇒ `treatment_required`; `RiskStatus::Accepted` without in-force acceptance does not suppress |
| P08-T06 | `P08: partially complete mitigation cannot complete` — 1 of 2 required actions done ⇒ `transition(Completed)` fails |
| P08-T07 | `P08: transferred risk with missing contract evidence` — fail closed (`MissingContractEvidence` or equivalent needle) |
| P08-T08 | `P08: superseded treatment is not the active path` — successor is active; old accept cannot suppress |
| P08-T09 | `P08: target residual mismatch fails closed` — completion digest ≠ approved target residual; if risk methodology present, stored rating must match `score_risk` |
| P08-T10 | `P08: dangling control references are treatment errors` — `ControlId` / `ControlImplementationId` absent from assessment fail `validate()` |
| P08-T11 | `P08: invalid transitions fail closed` — e.g. `Proposed → Completed`, `Proposed → Executing`, `Cancelled → Approved`, `Completed → Executing` error; no panic |
| P08-T12 | `P08: all four strategies share the state machine` — each can walk proposed→approved→executing→verification→completed |
| P08-T13 | `P08: Risk::new and risk.json remain compatible` |
| P08-T14 | `P08: risk register treatment_id resolves when present` — if `Risk` has `treatment_id`, `Some` must exist in `risk_treatments`; do not fork `Risk` |
| P08-T15 | `P08: collectors cannot emit treatment ratings or acceptance` — no global `RiskRating::High`; collector grep clean |
| P08-T16 | `P08: dual-suite registered` — root `Cargo.toml` lists `sdd_risk_treatment_baseline` and `sdd_risk_treatment_target` |

Workspace verify GREEN; neighbor targets stay GREEN (do not regress risk methodology, register, and identification contracts).

---

## 7. Out of scope

- Calculating residual effectiveness from control tests / `Effective` → lower score (residual risk).
- Creating external tickets or `Remediation` documents (remediation engine). Opaque refs only.
- Interpreting framework applicability or generating operational SoA from treatment (operational SoA / Kleene engine).
- Expanding `ControlImplementation` schema (control-implementation registry).
- Identifying risks, `RiskCandidate`, promotion (risk identification).
- Forking `Risk` or rewriting risk register status table.
- Hardcoding a 5×5 or global `RiskRating::{Low,Medium,High}`.
- Building `IsmsContext` (ISMS context IR) or scope engine (scope engine).
- Changing catalog TOML, ISO pack requirement IDs, GitHub collector mapping.
- Making `compile_framework` execute treatment plans.
- UI, persistence service, notifications, insurer/legal integrations.
- Bumping `assurance-ir/v1`.
- Using `Exception` as risk acceptance.
- Auto-advancing `reviewAt` / `targetDate`.

---

## 8. Risks

- Parallel risk methodology/register/07 landing: forking `Risk` or a second scoring model. Mitigation: consume specs/types; placeholder residual only if 05 types absent; single `RiskTreatmentId` in `id.rs`.
- ADR number collision: risk methodology and 06 both drafted `0005-*`. This decision is **0006** and is cited by path.
- Treating `RiskStatus::Accepted` as evidence. Mitigation: T02/T05; clocked `treatment_required`; immutable `RiskAcceptance`.
- Skipping state machine edges for Accept (`Approved → Completed`). Mitigation: T11/T12; all strategies walk the full happy path.
- Completing mitigation without controls (enum-only). Mitigation: T01/T06/T10.
- residual risk readers mistaking `targetResidual` for calculated residual. Mitigation: field docs; T09 does structural match only.
- Dangling `ControlImplementationId` ignored because control-implementation registry is absent. Mitigation: resolve against existing `assessment.implementations` inventory.
- Compile capability confused with engine. Mitigation: B07/T16; no compile-stage treatment evaluation.
- `skip_serializing_if` forgotten on new assessment field changing golden assessment JSON. Mitigation: `serde(default)` + skip empty vec so old assessments round-trip.
- Baseline absence tests blocking CI after implement. Mitigation: skip-supersede like other replacement suites; keep registration.

---

## 9. ADR

This is an architecture/contract decision (IR home, four strategies with mandatory evidence, immutable acceptance + expiry, fail-closed state machine, versioned-placeholder target residual, control-implementation registry / remediation engine as references only). Accepted: [`docs/adr/0006-risk-treatment-engine.md`](../adr/0006-risk-treatment-engine.md).

Do not add `0003-risk-treatment.md` or a third `0005-*`.

---

## 10. Implementation notes (landed)

Owned crate: `weeping-angel-assurance-ir` (`src/risk_treatment.rs` plus `id.rs` / `assessment.rs` / `validation.rs` / re-exports).

Shipped exports:

```text
RiskTreatmentId, RiskAcceptanceId, TreatmentPlanId, TreatmentActionId, RemediationRef
TreatmentStrategy, TreatmentState, ActionState
RiskTreatmentDecision, TreatmentPlan, TreatmentAction
RiskAcceptance, TransferEvidence, TargetResidualRisk::VersionedPlaceholder
TreatmentApproval, TreatmentEvidenceExpectation, TreatmentEvidenceRef, TreatmentEvent
TreatmentError
TreatmentState::can_transition
RiskTreatmentDecision::{propose, transition, validate}
acceptance_in_force, treatment_required, active_treatment
validate_treatment_inventory, validate_treatments_at
```

Freeze fields (not in the original record sketch): `approvedTargetResidualDigest` (set on approve), `sealedAcceptanceDigest` (Accept). `Risk::new` and `risk.json` remain valid. Register `Risk.treatment_id` resolves in this inventory.

`CANONICAL_SPECS` includes this path. ADR 0006 is **Accepted**. Traces: `.sdd/runs/` only.

---

## 11. Handoff contract (downstream slices)

```text
risk register  Risk.treatment_id → RiskTreatmentId (inventory in this slice)
residual risk  residual projection reads locked methodology + this plan version + control-test snapshot;
           does not treat Completed as residual zero
control-implementation registry  ControlImplementation may later list linked treatment ids; this slice only cites existing impl ids
operational SoA  operational SoA may read treatment-driven applicability later; this slice does not project SoA
remediation engine  Landed `Remediation` may point at TreatmentActionId; this slice stores opaque RemediationRef only (`RemediationId` string)
```

Downstream must **not** teach tests that `RiskStatus::Accepted` is evidence. Downstream must **not** hardcode 5×5 in residual engines.

---

## 12. Definition of done

Every open risk can have an explicit accountable treatment path whose state, evidence, and approvals are reproducible and auditable. Four strategies are first-class. Expired acceptance cannot suppress treatment. Invalid transitions, dangling controls, missing transfer contracts, partial mitigation, target-residual mismatch, and superseded paths fail closed or lose active status as specified.

Dual-suite SDD protocol is satisfied: spec first, baseline GREEN on characterization, target RED then GREEN after implement, docs+ADR accepted, baseline absence tests skip-superseded, target still GREEN.
