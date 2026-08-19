# SDD: Residual Risk and Control Effectiveness

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_residual_risk_target` GREEN (P09-T01–T20); baseline absence claims skip-superseded |
| Program | Operational ISMS v1 — residual risk |
| Slice | Project control effectiveness into an explainable, reproducible residual-risk result pinned to inherent-risk, treatment-plan, methodology, and control-test snapshots. Do **not** implement risk methodology and register / 08 engines. |
| Dual-suite | `sdd_residual_risk_baseline` · `sdd_residual_risk_target` (`tests/contracts/residual_risk.{baseline,target}.rs`) — registered in root [`Cargo.toml`](../../Cargo.toml) (directory is **not** auto-discovered) |
| ADR | Accepted [`docs/adr/0003-residual-risk.md`](../adr/0003-residual-risk.md) (`0003-*` program sibling numbering; 0004 is documentation architecture; 0005-* is methodology/register). Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (landed Residual risk section; do not fork the spine) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| Lineage (must fit) | [`docs/specs/assessment-lineage.md`](assessment-lineage.md), [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md) |
| Neighbors (consume / pin; do not implement here) | [`docs/specs/risk-methodology.md`](risk-methodology.md), [`docs/specs/risk-register.md`](risk-register.md), [`docs/specs/risk-treatment.md`](risk-treatment.md), [`docs/specs/control-implementation-registry.md`](control-implementation-registry.md) |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Collision fence | GitHub collector, catalog TOML / ISO remaps, Kleene applicability, unrelated catalog SDD suites, dashboards / risk-acceptance workflow |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Control-test contract (reuse) | `weeping_angel_control_test::{ControlTestResult, Effectiveness}` |
| Workspace verify (after implement) | `cargo test --test sdd_residual_risk_baseline`; `cargo test --test sdd_residual_risk_target`; `cargo test --test sdd_documentation_layout`; keep existing `sdd_*_target` GREEN; `cargo test --workspace --features demo` when practical |

This document is the durable human SSOT for Operational ISMS v1 residual risk. It owns **residual-risk modes**, **versioned projection lineage**, **methodology-specific reduction semantics**, and **fail-closed grounding in actual treatment/control-test state**. It does **not** own risk methodology scales/matrices (risk methodology), the operational register expansion (risk register), treatment-plan state machines (risk treatment), dashboards, or risk-acceptance workflows.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Residual risk is a **projection over pinned snapshots**, not a live current-state score and not a collector fact. Control-test `Effectiveness` is an input signal. It is **not** a residual-risk rating.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a pointer README only.

---

## 0. Collision fence (concurrent SDD)

This slice may edit only residual-risk types, projection APIs, dual-suite contracts, this spec, its ADR, `documentation_layout.rs` registration, and additive root `Cargo.toml` `[[test]]` entries.

| Do not touch | Owner |
| --- | --- |
| `tests/contracts/github_collector.*`, `crates/weeping-angel-collector/src/github/**` | GitHub collector |
| `catalog/canonical/v1/**` domain TOML, ISO pack IDs / `to =` remaps, `tests/contracts/iso27001_remap.*` | Catalog / ISO remap |
| Applicability Kleene evaluator (`weeping-angel-assurance::applicability`, `OrgContext`, `evaluate_org_context`) | control-implementation registry already landed |
| Unrelated catalog SDD suites (`iam` / `sdlc` / `vuln` / `infra` / `governance` contracts) | Those slices |
| Dashboards / UI / risk-acceptance workflow | residual risk non-goals |
| risk methodology scoring engine, risk register register expansion, risk treatment treatment state machine | Neighbor slices — **pin versions only** |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| Domain types, versioned refs, projection document, errors | `crates/weeping-angel-assurance-ir` — [`residual.rs`](../../crates/weeping-angel-assurance-ir/src/residual.rs); re-export from [`lib.rs`](../../crates/weeping-angel-assurance-ir/src/lib.rs). Do not fold residual into [`risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs). |
| Projection that consumes `ControlTestResult` | `crates/weeping-angel-assurance` [`residual.rs`](../../crates/weeping-angel-assurance/src/residual.rs) (`::residual`) |
| Control-test types | **Reuse** `weeping-angel-control-test`. Touch that crate only if a thin adapter is unavoidable. Do not add residual ratings to `Effectiveness`. |
| Evidence crate | **Conclusion-free.** Observations only. No residual score on envelopes. |

Tiny allowed adjustments at implement: additive IR types + `typed_id!` aliases; `lib.rs` re-exports; serde camelCase + `serde(default)` on new structs; assurance projection entry; dual-suite registration. Do **not** bump `ASSURANCE_IR_SCHEMA`. Do **not** add residual fields that mutate historical `Risk` rows in place.

control-implementation registry already fences this file (`docs/specs/residual-risk.md`, `tests/contracts/residual_risk.*`, `crates/**/residual*.rs`, `docs/adr/*residual*`). Reciprocate: do not implement the control-implementation registry here.

---

## 1. Problem / user-visible goal

Operators cannot project residual risk from actual treatment and control-test state. [`risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs) is a four-field inventory stub whose module comment is *“Minimal risk record. Not a risk engine.”* There is no residual-risk mode, no methodology version, no treatment-plan version, no inherent-risk snapshot pin, and no reduction from `ControlTestResult`.

That means:

- “the control is Effective” can be misread as “residual risk is zero”;
- an approved `Exception` / `Effectiveness::ExceptionApproved` can be misread as “risk is low”;
- a later control regression would have nowhere to write a **new** projection without overwriting history;
- missing evidence, stale tests, dangling controls, or an unversioned methodology would silently invent a number.

risk methodology (methodology), 06 (register), and 08 (treatment) are **specified but not landed as first-class engines**. This slice must not implement those engines. It must define the **minimum versioned references** residual risk needs so a projection can pin lineage and **fail closed** when those pins are missing.

**User-visible goal:** given a finalized assessment snapshot, project residual risk as an explainable, reproducible document grounded in actual treatment/control state — without pretending all risk reduces to a hidden formula.

```text
inherent-risk snapshot/version
  + treatment-plan snapshot/version
  + methodology id+version
  + relevant controls
  + pinned ControlTestResult snapshot
  + mode (Calculated | Assessed | Hybrid)
  + projection time
  + optional accountable manual assessment
        → ResidualRiskProjection (immutable)
```

A reviewer must be able to answer:

```text
which inherent-risk version was projected from?
which treatment-plan version was in force?
which controls and control-test results were used?
which methodology id+version defined reduction?
when was this projected?
who assessed/approved any manual residual, and why?
why did Effective not become zero?
why did ExceptionApproved not become low?
can I still load the previous projection after a control regression?
```

Example distinctions this slice must keep:

```text
Effectiveness::Effective
  → may reduce residual per methodology; NEVER maps to zero residual by itself

Effectiveness::Ineffective or missing relevant control
  → no reduction from that control; fail closed if the control is required and absent

partial treatment / PartiallyEffective
  → partial reduction only; never full credit

Assessed mode without principal + rationale + time
  → fail closed

Hybrid without deterministic signals or without approved management assessment
  → fail closed

StaleEvidence / InsufficientEvidence / NotTested / stale snapshot
  → fail closed (no invented residual)

approved Exception / ExceptionApproved
  → recorded as governance evidence; residual is NOT silently Low

no-reduction methodology
  → effectiveness never lowers residual (residual stays at inherent)

control regression after a completed projection
  → NEW projection; historical projection remains queryable and byte-stable
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Risk` / `RiskStatus` / `Risk::new` | `weeping-angel-assurance-ir::risk` | **Keep.** Do not replace the inventory record with a risk engine. Residual is a **separate projection document**, not a silent overwrite of `Risk.status`. risk register may later hang a residual *ref* on the register; this slice does not expand the register. |
| `RiskId` | `id.rs` `typed_id!` | Unchanged. Optional new typed ids: `ResidualRiskId`, `RiskMethodologyId` (if 05 still absent), `RiskTreatmentId` / snapshot ids. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Do **not** fork. Projection documents inherit this schema (or omit a nested schema id and inherit). |
| Golden `risk.json` | `tests/fixtures/assurance-ir/v1/risk.json` | Must keep decoding. Do not add required residual fields to `Risk::new` JSON. |
| IR-019 | `validation.rs` | Still: dangling `RiskId` on implementations fails closed. Do not retarget IR-019 at residual projections. |
| `ControlTestResult` / `Effectiveness` | `weeping-angel-control-test` | **Reuse the existing enum.** Variants in play: `Effective`, `Ineffective`, `PartiallyEffective`, `NotApplicable`, `NotTested`, `InsufficientEvidence`, `StaleEvidence`, `ManualReviewRequired`, `ExceptionApproved`, `Inconclusive`. Never add a residual rating to this enum. |
| `Exception` / `ExceptionStatus` | `exception.rs` | Approved exception is **governance evidence**. Never a residual floor of Low. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for Assessed / Hybrid accountable principal. Do not invent `ResidualAssessor`. |
| Assessment lineage | `weeping-angel-assurance::lineage` | Residual results must fit the immutable-snapshot model. New projection on control regression; never mutate historical values. Prefer attaching residual docs to a lineage bundle / ledger payload rather than rewriting `AssessmentRun` identity. |
| risk methodology methodology | spec-only / not landed | Do **not** implement scales, matrices, or `score_risk`. Define `MethodologyRef { id, version }` (and accept `RiskMethodologyId` if 05 lands first). Fail closed if the ref is missing or the version is unknown. |
| risk register register | spec-only / not landed | Do **not** expand `Risk` into the operational register. Pin `InherentRiskRef` (risk id + snapshot version/digest). Fail closed if missing. |
| risk treatment treatment | landed — [`risk-treatment.md`](risk-treatment.md) / [ADR 0006](../adr/0006-risk-treatment-engine.md) | Do **not** implement `TreatmentPlan` state machines here. Pin `TreatmentPlanRef` (plan id + snapshot version/digest). Fail closed if missing. Completed treatment ≠ residual zero. |
| control-implementation registry implementations | spec-only neighbor | Residual consumes **control-test** effectiveness, not `ImplementationStatus`. `Implemented` ≠ `Effective` ≠ low residual. |
| Collectors / evidence | collector + evidence crates | Collectors emit facts. Evidence crate stays observation-only. No `RiskRating` / residual on envelopes. |
| Applicability engine | `weeping-angel-assurance::applicability` | Collision fence. Residual is not Kleene evaluation. |
| Dual-suite neighbors | root `Cargo.toml` | Register residual suites next to existing `sdd_*`. Directory is **not** auto-discovered. Keep existing `sdd_*_target` GREEN. |

Tiny allowed: new IR module; typed snapshot refs; projection + query APIs; serde defaults; re-exports.

Do **not** redesign `AssessmentDefinition` inventories, catalog TOML, collectors, Kleene evaluation, or `Effectiveness`.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 `Risk` is a four-field inventory stub

[`crates/weeping-angel-assurance-ir/src/risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs):

```text
//! Minimal risk record. Not a risk engine.

RiskStatus = Open | Accepted | Mitigated | Closed   // camelCase, default Open

Risk { id: RiskId, title: String, description: String, status: RiskStatus }

Risk::new(id, title, description) → status = Open
```

There is **no** residual field, mode, methodology version, treatment-plan version, inherent-risk snapshot version, reduction trace, or projection time.

[`crates/weeping-angel-assurance-ir/src/lib.rs`](../../crates/weeping-angel-assurance-ir/src/lib.rs) re-exports `risk::{Risk, RiskStatus}` only. There is no `residual` module.

Golden fixture [`tests/fixtures/assurance-ir/v1/risk.json`](../../tests/fixtures/assurance-ir/v1/risk.json):

```json
{
  "id": "risk:source-tamper",
  "title": "Source tampering",
  "description": "Unauthorized change to the source of record.",
  "status": "open"
}
```

### 3.2 No residual-risk types or projection API

Product crate sources (`crates/**/src/**/*.rs`) contain none of:

- `ResidualRisk`, `ResidualRiskProjection`, `ResidualRiskMode`
- `Calculated` / `Assessed` / `Hybrid` as residual modes
- `InherentRiskRef` / `InherentRiskSnapshot`
- `TreatmentPlanRef` / `TreatmentPlanSnapshot`
- `MethodologyRef` used for residual reduction
- `project_residual_risk` / `query_residual_risk` (or equivalent)
- a no-reduction residual methodology id

`RiskMethodology`, `RiskMethodologyId`, `TreatmentPlan`, and operational-register residual placeholders are **also** absent (risk methodology and register / 08 not landed). Baseline must characterize that absence; target must **not** fill it by implementing those full engines.

### 3.3 Control-test effectiveness exists and is unused by risk

`weeping_angel_control_test::Effectiveness` already includes:

```text
Effective
Ineffective
PartiallyEffective
NotApplicable
NotTested
InsufficientEvidence
StaleEvidence
ManualReviewRequired
ExceptionApproved
Inconclusive
```

`ControlTestResult` already carries `test_id`, `control_id`, `effectiveness`, `rationale`, `evidence_refs`, `missing_evidence`, `evaluatedAt` (`checked_at`), `test_version`, `input_digest`, `population`.

Nothing in `risk.rs` or assurance orchestration projects these values into residual risk. There is no mapping, and therefore also no explicit **non**-mapping of `Effective` → zero.

### 3.4 Exceptions are governance records, not residual ratings

[`exception.rs`](../../crates/weeping-angel-assurance-ir/src/exception.rs): `Exception` + `ExceptionStatus::{Proposed, Approved, Expired, Revoked}`. Approved exceptions can produce `Effectiveness::ExceptionApproved` in control-test. No residual-risk code consults them.

### 3.5 Assessment lineage is immutable; residual is not on the chain

Assessment lineage (already landed) persists pinned snapshots and forbids silent rewrite of historical assessments. Residual-risk results are **not** part of that chain today. There is no historical residual query.

### 3.6 Neighbor register placeholders are not this projection

Accepted [`docs/adr/0005-operational-risk-register.md`](../adr/0005-operational-risk-register.md) places `residualScore` / `residualRating` on the register row as **placeholders only**. **This residual risk spec owns control-derived residual.** Projection lives in assurance-IR + `weeping-angel-assurance::residual`. The GitHub collector remains a **collision fence** (facts only).

### 3.7 Dual-suite registration

Root [`Cargo.toml`](../../Cargo.toml) lists:

```text
sdd_residual_risk_baseline → tests/contracts/residual_risk.baseline.rs
sdd_residual_risk_target   → tests/contracts/residual_risk.target.rs
```

Those binaries are **not auto-discovered**. The baseline suite encodes §3 absence and must stay GREEN on CURRENT until the target suite is GREEN and skip-superseded. The target binary currently locks spec IDs P09-T01–T20; implement replaces it with executable projection tests. This spec file **is** registered in `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS`.

---

## 4. Desired behavior

### 4.1 Residual risk is a projection, not a formula pretence

A residual-risk result is an immutable **projection document**. Calculation, when used, is a **named, versioned methodology** with an explainable reduction trace. The product must also support fully manual assessment and a hybrid of both.

Never:

- map `Effectiveness::Effective` directly to zero residual;
- treat `ExceptionApproved` or an approved `Exception` as residual = Low;
- invent a residual when required pins or required evidence are missing;
- mutate a historical projection because a later test regressed.

### 4.2 Modes

```text
ResidualRiskMode = Calculated | Assessed | Hybrid
```

| Mode | Required inputs | Result |
| --- | --- | --- |
| `Calculated` | All lineage pins in §4.3 + a **deterministic, explicitly versioned** calculated methodology | Residual derived only from those pins. Same pins → same projection identity (excluding wall-clock `projectedAt` from digest if a separate identity digest is sealed). |
| `Assessed` | All lineage pins in §4.3 + **accountable manual evidence** (`principal`, `rationale`, `assessedAt`; approval when the methodology requires it) | Residual is the manual assessment. Pins still recorded. Missing principal, empty rationale, or missing time → **fail closed**. |
| `Hybrid` | Calculated side **and** an approved management assessment (`approvedBy` plus Assessed accountability) | Combines deterministic signals with the approved assessment. Management may **raise** residual above the calculated ordinal; it may not lower residual below calculated and may not skip Calculated fail-closed pins. Missing either side → **fail closed**. |

`Calculated` methodologies must be deterministic and explicitly versioned (`methodologyId` + `methodologyVersion`). Two processes given the same pins and methodology must produce the same residual identity and the same reduction trace (sorted collections; no map-insert-order drift; no `f64` in digest-critical bytes).

### 4.3 Lineage on every result

Every `ResidualRiskProjection` MUST identify:

```text
risk id
inherent-risk snapshot/version          # InherentRiskRef
treatment-plan snapshot/version         # TreatmentPlanRef
methodology id + version                # MethodologyRef
relevant control ids                    # declared by treatment pin and/or caller
control-test results / evidence snapshot
  - snapshot digest and/or ordered result identities
  - each ControlTestResult used (id, control, effectiveness, test_version, input_digest)
projection time                         # projectedAt (RFC3339)
mode
manual assessment / approval            # required for Assessed; required approved side for Hybrid
reduction trace                         # explainable steps; empty only for fail-closed (no document)
```

Historical changes remain queryable by projection id and/or `(risk id, projectedAt)` / snapshot digest. A control regression, stale-evidence refresh, treatment-plan supersession, or methodology supersession produces a **new** projection. Loading the old id returns the old document bytes (semantic fields). Ledger/persist, if used, is append-only for completed projections (same law as assessment lineage: do not `INSERT OR REPLACE` a completed residual payload with different bytes).

Fit lineage: residual may be stored as an opaque ledger payload or a typed snapshot in `weeping-angel-assurance`. Evidence crate still does not compute residual.

### 4.4 Minimum versioned reference types (risk methodology and register / 08 not landed)

Do **not** implement full methodology, register, or treatment engines. Land only the pins residual-risk needs:

```text
InherentRiskRef {
  riskId: RiskId
  snapshotId | version     # monotonic version or snapshot id
  digest?                  # canonical digest of the inherent snapshot body
}

TreatmentPlanRef {
  planId                   # typed id or stable string; not a full TreatmentPlan engine
  snapshotId | version
  digest?
}

MethodologyRef {
  methodologyId            # e.g. "residual-methodology:no-reduction"
  version                  # e.g. "v1" — required, never implicit latest
}

ControlTestSnapshotRef {
  digest                   # SHA-256 hex of canonical JSON over sorted result identities
  resultIds[]              # test id + input_digest + control id
}
```

Callers (tests, later 05/06/08, assess orchestration) **construct** these refs. If any required ref is missing, empty, or dangling (unknown methodology version; inherent/treatment version not supplied; control-test snapshot absent), projection **fails closed** with a deterministic error that names the missing pin.

When risk methodology later lands `RiskMethodologyId`, this `MethodologyRef.methodologyId` should accept that id. Residual **reduction** methodologies (how effectiveness changes residual) are a residual risk concern and may be a small built-in set (§4.6) even if risk methodology scoring methodologies exist.

When risk treatment later lands `TreatmentPlan`, `TreatmentPlanRef` should point at that snapshot. Until then, tests supply a **minimum snapshot**: id, version, relevant control ids, treatment completeness (`none` / `partial` / `complete` or equivalent), and digest.

When risk register later lands inherent score/rating on `Risk`, `InherentRiskRef` should point at that versioned snapshot. Until then, tests supply a **minimum inherent snapshot**: risk id, version, inherent rating/ordinal (methodology-declared id or ordinal — **not** a global `RiskRating::{Low,Medium,High}` crate law), and digest.

### 4.5 Fail-closed evidence and control state

Projection fails closed (no residual document) when any of the following hold for a **required** relevant control or for the snapshot as a whole:

| Condition | Why |
| --- | --- |
| Missing methodology id or version | Unversioned math is forbidden |
| Unknown methodology version | Cannot reproduce |
| Missing inherent-risk version/snapshot | Nothing to project from |
| Missing treatment-plan version/snapshot | Treatment state unknown |
| Missing control-test snapshot | No grounding in tests |
| Relevant control dangling (id with no test result and no explicit NotApplicable pin) | Missing control |
| `Effectiveness::NotTested` | No evidence the control works |
| `Effectiveness::InsufficientEvidence` | Required evidence missing |
| `Effectiveness::StaleEvidence` or snapshot marked stale vs projection time / freshness pin | Stale evidence |
| Hybrid missing approved management assessment | Both sides required |
| Assessed missing principal, rationale, or time | Not accountable |
| Calculated methodology that requires a test result that is not in the snapshot | Incomplete pins |

`ManualReviewRequired` and `Inconclusive` are fail-closed for **Calculated** reduction credit. They may appear on an Assessed/Hybrid document only as **signals** alongside a complete manual/approved assessment; they never themselves grant reduction.

`NotApplicable` grants **no reduction**. It does not fail the projection if the treatment snapshot marks that control out of scope for this risk. If the control is still listed as required/relevant, `NotApplicable` is fail-closed (contradiction).

### 4.6 Methodology-specific reduction semantics

Reduction is **data + version**, not a hidden crate constant. Ship at least:

1. **No-reduction methodology** (required by tests)

   ```text
   methodologyId = residual-methodology:no-reduction
   version       = v1
   ```

   Effectiveness **never** lowers residual. Residual identity equals the inherent snapshot’s rating/ordinal (copy + explain “no reduction”). `Effective` still does not become zero; if inherent was non-zero, residual stays non-zero.

2. **Control-effectiveness reduction methodology** (versioned calculated default)

   ```text
   methodologyId = residual-methodology:control-effectiveness
   version       = v1
   ```

   Rules (normative for v1):

   - Start at inherent ordinal / declared rating.
   - Each **relevant** control may contribute at most one reduction step, recorded in the trace.
   Landed v1 steps (`MIN_RESIDUAL_FLOOR = 1`):

   | Effectiveness × completeness | Step |
   | --- | --- |
   | `Effective` × `complete` | 2 |
   | `Effective` × `partial`, or `PartiallyEffective` × `complete`/`partial` | 1 |
   | `Ineffective`, `ExceptionApproved`, `none` completeness, other variants | 0 |

   - `Effective` → apply that control’s declared reduction **step**. **Never** set residual to zero solely because one or all controls are `Effective`. Floor is mandatory and greater than zero (`MIN_RESIDUAL_FLOOR = 1`).
   - `PartiallyEffective` or treatment completeness `partial` → strictly smaller step than `Effective`+`complete`. Never full credit.
   - `Ineffective` → zero step from that control (residual stays at least inherent minus other valid steps, and never below floor).
   - `ExceptionApproved` → zero step from that control; trace MUST name the exception / effectiveness variant; residual MUST NOT become the methodology’s lowest band because of the exception.
   - Multiple controls: compose **conservatively** (deterministic): apply steps in sorted `ControlId` order; never go below `minResidual`; do not treat N Effective controls as zero. Multiple results for one control take the worst effectiveness.
   - Missing / fail-closed variants (§4.5) abort the projection. `NotApplicable` on a control listed in `relevantControlIds` is contradiction (`not applicable`). Out-of-scope controls are omitted from that list, not scored.

Organizations may later register additional calculated methodologies (risk methodology family). Unknown ids fail closed. A later methodology version is a **new** `MethodologyRef`; old projections keep the old version.

**There is no API that accepts a residual rating as a collector/evidence value.**

### 4.7 Partial treatment, multiple controls, exceptions

- **Partial treatment:** treatment snapshot completeness `partial` and/or `PartiallyEffective` tests. Residual moves, but not to the full mitigated band.
- **Multiple controls:** all relevant ids appear on the projection; each has a trace line (step or explicit zero-step / fail).
- **Approved exception:** include exception id(s) on the projection when present on the test/evidence set. Governance evidence only. Residual remains explainable and **not** silently Low.
- **Missing controls:** required relevant control without a result → fail closed (not “assume ineffective and continue”) unless the methodology is explicitly documented to treat missing as ineffective **and** tests cover that — default v1 **fails closed**.

### 4.8 Projection API (landed)

IR types in `weeping_angel_assurance_ir::{ResidualRiskProjection, ResidualRiskMode, …}`. Projection in `weeping_angel_assurance::residual`:

```text
project_residual_risk(store, ResidualRiskRequest) -> Result<ResidualRiskProjection, ResidualRiskError>
query_residual_risk(store, ResidualRiskId) -> Option<ResidualRiskProjection>

ResidualRiskRequest {
  mode: ResidualRiskMode
  inherent: InherentRiskSnapshot
  treatment: TreatmentPlanSnapshot
  methodology: MethodologyRef
  control_tests: ControlTestSnapshotRef
  control_test_results: [ControlTestResult]
  exceptions?: [Exception]
  manual?: ManualResidualAssessment { principal, rationale, assessedAt, approvedBy?, residualOrdinal, residualRatingId }
  projected_at: DateTime<Utc>
}
```

`ResidualRiskError` is deterministic `Display` (stable needles):

```text
missing inherent-risk version
missing treatment-plan version
missing methodology version
unknown methodology
missing control-test snapshot
dangling control
insufficient evidence
not tested
stale evidence
missing manual assessment
missing management assessment
effective is not zero residual   # invariant tests may assert on output, not only errors
```

Projection identity is `residual:{sha256}` of camelCase canonical JSON over semantic fields **including** caller-supplied `projectedAt` / manual `assessedAt`. `ResidualRiskStore::insert` is first-write-wins (historical id is not replaced).

### 4.9 Dual-suite protocol

Register at implement (directory is **not** auto-discovered):

```toml
[[test]]
name = "sdd_residual_risk_baseline"
path = "tests/contracts/residual_risk.baseline.rs"

[[test]]
name = "sdd_residual_risk_target"
path = "tests/contracts/residual_risk.target.rs"
```

| Gate | Suite | Expected |
| --- | --- | --- |
| Spec | this file | written **before** product feature code |
| Baseline on CURRENT | `sdd_residual_risk_baseline` | **GREEN** — characterizes §3 absence |
| Target on CURRENT | `sdd_residual_risk_target` | **RED for the right reason** (missing types/API, not half-written stubs) |
| Implement | IR refs + projection | — |
| Docs + ADR | this file + `docs/adr/0003-residual-risk.md` | ADR Draft → Accepted when target GREEN |
| Target after | same target suite | **GREEN** |
| Baseline after | baseline | skip-supersede (`#[ignore = "superseded by sdd_residual_risk_target"]`) **or** prove it FAILS because CURRENT no longer matches §3 |
| Neighbors | existing `sdd_*_target` | stay GREEN |
| Workspace | `cargo test --workspace --features demo` | GREEN after implement when practical |

SDD order is **mandatory**. Abort rather than skip gates.

Write tests **before** product projection (RED → fix → GREEN). One regression test per later review comment titled `P?: <exact subject>` encoding the original found case.

### 4.10 Baseline suite contents (GREEN on CURRENT)

Encode **today’s** shortcuts. Suggested titles `P09: …` for the found case:

| ID | Assertion |
| --- | --- |
| P09-B01 | `risk.rs` module docs still contain `Minimal risk record. Not a risk engine.` |
| P09-B02 | `Risk::new` JSON has no residual / mode / methodology / treatment / projection fields |
| P09-B03 | Golden `risk.json` still decodes; `id == "risk:source-tamper"` |
| P09-B04 | Product crate sources have no `ResidualRiskProjection`, `ResidualRiskMode`, `project_residual_risk` |
| P09-B05 | `lib.rs` re-exports `Risk` / `RiskStatus` only (no residual types) |
| P09-B06 | `Effectiveness` still declares `Effective` … `ExceptionApproved` … `Inconclusive` in control-test (reuse lock) |
| P09-B07 | No residual reduction mapping in `risk.rs` / assurance residual module (module absent) |
| P09-B08 | Collision fence: this suite does not edit GitHub collector paths or ISO remaps |

After implement they should fail or be `#[ignore = "superseded by sdd_residual_risk_target"]`.

### 4.11 Target suite contents (RED on CURRENT, GREEN after)

Stable titles. Author **before** product feature code (compile-safe; fail because projection types/APIs are missing):

| ID | Title / assertion |
| --- | --- |
| P09-T01 | `P09: effective control does not map to zero residual` — `Effectiveness::Effective` may reduce; residual ≠ zero / absent; floor held |
| P09-T02 | `P09: ineffective control grants no reduction` |
| P09-T03 | `P09: missing required control fails closed` |
| P09-T04 | `P09: partial treatment is not full credit` |
| P09-T05 | `P09: assessed residual requires principal rationale and time` — complete Assessed succeeds; missing any of the three fails closed |
| P09-T06 | `P09: historical projection remains queryable after new projection` — control regression writes a **new** id; old id unchanged |
| P09-T07 | `P09: stale evidence fails closed` — `StaleEvidence` or stale snapshot |
| P09-T08 | `P09: multiple controls compose conservatively and remain explainable` |
| P09-T09 | `P09: no-reduction methodology ignores effectiveness` — Effective does not lower residual |
| P09-T10 | `P09: approved exception does not silently mean residual is low` — `ExceptionApproved` and/or approved `Exception` |
| P09-T11 | `P09: Calculated vs Assessed vs Hybrid` — all three modes exist and differ |
| P09-T12 | `P09: Hybrid fails closed when management assessment is missing` |
| P09-T13 | `P09: fail closed missing methodology version` |
| P09-T14 | `P09: fail closed missing treatment-plan version` |
| P09-T15 | `P09: fail closed missing inherent-risk version` |
| P09-T16 | `P09: fail closed missing control-test snapshot` |

Additional locks (may share tests):

| ID | Assertion |
| --- | --- |
| P09-T17 | Dual-suite binaries registered in root `Cargo.toml` |
| P09-T18 | IR schema remains `assurance-ir/v1`; `Risk::new` / `risk.json` still decode |
| P09-T19 | Projection reuses `ControlTestResult` / `Effectiveness` (no parallel enum) |
| P09-T20 | Collectors / evidence crate sources still have no residual rating types |

---

## 5. Acceptance criteria (testable)

1. Dual-suite `sdd_residual_risk_baseline` / `sdd_residual_risk_target` is registered in root `Cargo.toml`; this spec is in `CANONICAL_SPECS`; baseline GREEN on current shortcuts; P09-T01–T16 authored so target is RED on current tree **before** product feature code; after implement, target GREEN and baseline skip-superseded (or proven failed).
2. Residual risk is an immutable projection document with mode `Calculated` | `Assessed` | `Hybrid`.
3. Every projection pins inherent-risk version, treatment-plan version, methodology id+version, relevant controls, control-test snapshot (digest/ids), projection time, and any manual assessment/approval.
4. `Calculated` is deterministic and versioned; `Assessed` requires principal + rationale + time; `Hybrid` requires both deterministic signals and an approved management assessment; missing either Hybrid side fails closed.
5. `Effectiveness::Effective` never maps directly to zero residual.
6. Required missing evidence (`NotTested`, `InsufficientEvidence`, stale snapshot / `StaleEvidence`, dangling/missing required control, missing methodology/treatment/inherent version, missing control-test snapshot) fails closed.
7. Partial treatment is first-class and is not full credit; multiple controls compose conservatively and appear in the reduction trace.
8. No-reduction methodology does not lower residual when controls are Effective.
9. Approved `Exception` / `Effectiveness::ExceptionApproved` is recorded and does **not** silently yield Low residual.
10. Control regression (or any pin change) creates a **new** projection; the previous projection remains queryable and semantically unchanged.
11. risk methodology and register / 08 full engines are **not** implemented in this slice; only minimum versioned refs exist.
12. `assurance-ir/v1` is not forked; `weeping-angel-evidence` stays conclusion-free; existing `sdd_*_target` suites stay GREEN; collision-fence paths are untouched.

---

## 6. Out of scope

- Dashboards, UI, HTML/PDF risk reports
- Risk-acceptance **workflow** (acceptance as a process, tickets, reminders). Assessed/Hybrid records may *cite* an approval principal; they do not implement acceptance lifecycle
- Full risk methodology methodology engine (scales, matrices, `score_risk`, appetite)
- Full risk register operational register expansion (scenario/threat/status machine)
- Full risk treatment treatment engine (`Mitigate`/`Accept`/`Avoid`/`Transfer` state machine, expiry of acceptance)
- control-implementation registry control-implementation registry
- Kleene applicability evaluator changes
- GitHub collector mapping; catalog domain TOML; ISO pack IDs / `to =` remaps
- New crate; forking `assurance-ir/v1`
- Mapping `Effective` → zero; treating exception as a Low residual floor
- Collectors emitting residual ratings as evidence
- Certification claims or a single `compliancePercent`
- Multi-tenant SaaS, authn/z, hosted control plane

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Implementers encode `Effective → residual = 0` | T01 + floor on calculated v1; no-reduction methodology T09 |
| `ExceptionApproved` treated as Low | T10; zero-step rule; exception ids on the trace |
| risk methodology and register/08 scope creep | Minimum refs only; fail closed when pins missing; neighbor specs stay owners |
| Historical mutation on regression | T06; append-only store; new projection id |
| Half-written stubs make target “pass” for the wrong reason | Target must import real types and call `project_residual_risk`; fail on CURRENT because they do not exist |
| Hybrid silently degrades to Calculated | T12 fail-closed when management assessment missing |
| Stale tests still scored | T07 fail-closed on `StaleEvidence` / stale snapshot |
| Evidence crate grows conclusions | Residual types stay in IR + assurance; T20 greps evidence/collector |
| ADR 0005-register “GitHub owns residual” confusion | This spec + ADR 0003-residual-risk own residual; GitHub stays fenced |
| Schema fork temptation | Keep `assurance-ir/v1`; additive types only |
| Dual-suite not registered (directory not auto-discovered) | T17 + implement checklist `[[test]]` in root `Cargo.toml` |
| Neighbor SDD targets go red | Do not edit fenced paths; keep `Risk::new` / golden fixture |

---

## 8. ADR

**Accepted.** [`docs/adr/0003-residual-risk.md`](../adr/0003-residual-risk.md). Filename `0003-*` is shared with catalog-program / operational-slice siblings. **0004** is documentation architecture. **0005-*** remains risk methodology and operational register. Cite the ADR, this spec, and the dual-suite by **path**.

---

## 9. Landed signatures

| Item | Home |
| --- | --- |
| `ResidualRiskId` | `weeping-angel-assurance-ir` `id.rs` (`typed_id!`) |
| `ResidualRiskMode`, `TreatmentCompleteness`, refs/snapshots, `ManualResidualAssessment`, `ResidualReductionStep`, `ResidualRiskProjection`, `ResidualRiskError` | `crates/weeping-angel-assurance-ir/src/residual.rs` |
| Constants | `NO_REDUCTION_METHODOLOGY_ID`, `CONTROL_EFFECTIVENESS_METHODOLOGY_ID`, `RESIDUAL_METHODOLOGY_V1`, `MIN_RESIDUAL_FLOOR` |
| Re-exports | `weeping_angel_assurance_ir::{ResidualRiskProjection, MethodologyRef, …}` |
| `ResidualRiskRequest`, `ResidualRiskStore`, `project_residual_risk`, `query_residual_risk` | `crates/weeping-angel-assurance/src/residual.rs` (`weeping_angel_assurance::residual`) |
| Control-test enum | `weeping_angel_control_test::{ControlTestResult, Effectiveness}` (unchanged) |
| Persist | in-memory `ResidualRiskStore` (first-write-wins). Ledger may store opaque JSON; evidence crate does not score. |

Neighbor engines (`score_risk`, `TreatmentPlan` state machine, register expansion) are **not** called. Callers construct the minimum snapshots in §4.4.

---

## 10. Landed notes

Product code is in `residual.rs` (IR + assurance). ADR [`docs/adr/0003-residual-risk.md`](../adr/0003-residual-risk.md) is **Accepted**. Baseline absence claims are skip-superseded. Target P09-T01–T20 is GREEN.

Do **not**: edit GitHub collector files; reimplement Kleene evaluation; rewrite catalog domain TOML or ISO pack IDs; fold residual into `Risk.status`; build dashboards or acceptance workflows.

---

## 11. Definition of done

Residual risk is an explainable projection grounded in actual treatment/control state and can be reproduced for any finalized assessment snapshot. Modes Calculated / Assessed / Hybrid behave as specified. Lineage pins are complete. Fail-closed conditions do not invent a number. `Effective` is never zero residual. Approved exceptions are never silently Low. History survives regression. Dual-suite protocol is satisfied (baseline skip-superseded, target GREEN). Neighbor SDD targets stay green.
