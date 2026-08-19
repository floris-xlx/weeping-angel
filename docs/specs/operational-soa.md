# SDD: Operational Statement of Applicability

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_operational_soa_target` is the SSOT; baseline skip-superseded |
| Program | Operational ISMS v1 — Prompt 11 |
| Source prompt | [`docs/prompts/operational-isms-v1/11-operational-soa.md`](../prompts/operational-isms-v1/11-operational-soa.md) |
| Slice | Upgrade SoA from a static pack-TOML assessment projection into a living, explainable operational-graph record (applicability, treatment, mappings, implementation, exceptions, evidence effectiveness) with immutable snapshots and cause-bearing diffs |
| Dual-suite | `sdd_operational_soa_baseline` · `sdd_operational_soa_target` (`tests/contracts/operational_soa.{baseline,target}.rs`) — **registered** in root `Cargo.toml` (tests/contracts is **not** auto-discovered) |
| ADR | Accepted [`docs/adr/0003-operational-soa.md`](../adr/0003-operational-soa.md) (`0003-*` program sibling; `0004` is documentation architecture). Cite by **path** |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) |
| Protocol report | [`.sdd/runs/sdd-operational-soa.md`](../../.sdd/runs/sdd-operational-soa.md) (generated; do not commit) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| ISO remap (three-state + justified NA; do not rewrite pack IDs) | [`docs/specs/iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) §4.6, ADR 0003 remap |
| Lineage (consume snapshots) | [`docs/specs/assessment-lineage.md`](assessment-lineage.md) — `StatementOfApplicabilitySnapshot` already exists |
| Applicability engine (consume; do not reimplement Kleene) | [`docs/specs/applicability-engine.md`](applicability-engine.md) — map SoA `Unresolved` ↔ Kleene `ManualDeterminationRequired` |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` (baseline asserts **current working-tree** `soa.rs` / CLI / lineage, not stale remap-baseline SHA text) |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Lineage schema | `weeping-angel/assessment-lineage/v1` (`LINEAGE_SNAPSHOT_SCHEMA`) |
| Kleene snapshot schema | `weeping-angel/applicability-snapshot/v1` |
| Operational SoA input schema (this slice) | `weeping-angel/operational-soa-input/v1` |
| Treatment ref schema (minimum; not the engine) | `weeping-angel/risk-treatment-ref/v1` |
| Risk-register ref schema (minimum; not the register) | `weeping-angel/risk-register-ref/v1` |
| Workspace verify | `cargo test --test sdd_operational_soa_baseline`; `cargo test --test sdd_operational_soa_target`; `cargo test --test sdd_documentation_layout`; keep `sdd_iso27001_assurance_target`, `sdd_iso27001_remap_target`, `sdd_assessment_lineage_target`, `sdd_applicability_engine_target` GREEN; `cargo test --workspace --features demo` when practical |

This document is the durable human SSOT for Operational ISMS v1 Prompt 11. It owns the **operational Statement of Applicability projection**, **explainable inclusion/exclusion/implementation/effectiveness rows**, **immutable SoA snapshots/digests**, and **snapshot diffs with causes**. It does **not** own Kleene evaluation, ISO pack remapping, residual-risk calculation, the control-implementation registry engine, risk-register/treatment engines, licensed ISO text, certification claims, or dashboards.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

The SoA is a **readiness projection over that graph**, never a certificate and never an ISO-normative document.

```text
what the requirement means     = pack requirement id + catalog mapping (remap)
whether it applies             = Kleene ApplicabilityDecision (engine) + NA governance
how this org implements it     = ControlImplementation (status ≠ effectiveness)
whether it is effective        = ControlTestResult.effectiveness / weeping_angel_control_test::Effectiveness
why it is in/out of the SoA    = this projection (rationale, treatment, exceptions, review/approval)
```

---

## 0. Collision fence (concurrent SDD)

This slice may edit only operational-SoA projection / snapshot-diff / optional CLI dispatch paths listed in crate homes. Do not implement Prompts 02 / 06 / 08 / 09 / 10 engines here.

| Do not touch | Owner |
| --- | --- |
| `docs/specs/residual-risk.md`, `tests/contracts/residual_risk.*`, `**/residual*.rs`, `docs/adr/*residual*` | Prompt 09 residual risk (landed) |
| `docs/specs/control-implementation-registry.md`, `tests/contracts/control_implementation_registry.*`, `docs/adr/*control-implementation*` | Prompt 10 control-implementation registry (in-flight) |
| `tests/contracts/github_collector.*`, `crates/weeping-angel-collector/src/github/**` | Canonical Assurance GitHub collector |
| `catalog/canonical/v1/**` domain TOML, ISO pack requirement/control IDs, pack `to =` remaps, `tests/contracts/iso27001_remap.*` | Remap / catalog owners — **consume mappings; do not remap** |
| Applicability Kleene evaluator modules (`weeping-angel-assurance::applicability`) | Canonical Assurance applicability engine — **consume** |
| Unrelated catalog SDD suites (`iam` / `sdlc` / `vuln` / `infra` / `governance`) | Those prompts |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| Operational projection, row explainability, NA governance, fail-closed input refs | `crates/weeping-angel-assurance/src/soa.rs` |
| `StatementOfApplicabilitySnapshot` persist type; `LineageBundle.soa` | `crates/weeping-angel-assurance/src/lineage.rs` (schema unchanged). Pin/digest is `pin_soa_snapshot` in `soa.rs` |
| Snapshot diff cause taxonomy | `crates/weeping-angel-assurance/src/snapshot.rs` `SoaDiffCause` + additive `SnapshotDiff.soaCauses`; `diff_soa_snapshots` in `soa.rs` |
| Crate-root re-exports (`project_soa_from_snapshot`, operational projector, pin, diff) | `crates/weeping-angel-assurance/src/lib.rs` |
| CLI | `src/assurance_soa.rs` (sibling of `src/assurance_explain.rs`); parser stays in `src/cli.rs`; dispatch in `src/main.rs` |
| IR types consumed as-is | `ControlImplementation`, `ImplementationStatus`, `Exception`, `Risk`, `AssessmentDefinition`, IR `AssessmentScope` — **no ISO Annex A fields** |

Tiny allowed adjustments: serde-default fields on `SoaEntry` / `StatementOfApplicability` / `SnapshotDiff` / `StatementOfApplicabilitySnapshot`; new projection input/error types in `soa.rs`; crate-root `pub use`. Do **not** put ISO Annex A fields on `weeping-angel-assurance-ir` `Control` / `ControlImplementation`. `weeping-angel-evidence` stays conclusion-free.

---

## 1. Problem / user-visible goal

Operators cannot treat the Statement of Applicability as an operational record. `project_soa(framework, version)` rereads today’s pack `applicability.toml` via `resolve_pack_dir`, copies three-state pack flags, hard-codes `implementation_state = "assessed"`, leaves `automated_effectiveness = None`, and emits empty evidence/exceptions. Missing implementation cannot be distinguished from not-applicable. Non-applicability has a pack rationale but no accountable approval/review lifecycle. Historical reconstruction exists as `project_soa_from_snapshot` (clone) but is **not** crate-root exported, and live CLI `assurance soa` still banner-and-exit-0. `SnapshotDiff` has no SoA cause taxonomy.

That means:

- an applicable control with no implementation can be misread as “assessed” or silently confused with NA;
- expired or unapproved NA does not surface as a readiness gap;
- a reviewer cannot name the risks, treatments, owners, evidence digests, or exceptions that produced a row;
- a later pack-file edit can be mistaken for the audit-period SoA if callers use live `project_soa` as history;
- diffs cannot say *why* a row changed (applicability vs implementation vs effectiveness vs exception expiry vs mapping vs treatment).

**User-visible goal:** upgrade SoA output from a static assessment projection into a living, explainable operational record generated from framework applicability, risk treatment, canonical mappings, implementation state, exceptions, and evidence effectiveness. The SoA must **explain every inclusion, exclusion, implementation, and effectiveness state**. It remains a **readiness projection** — never a certification claim. Do not store licensed ISO normative text.

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` and re-verified against **current** `soa.rs` / lineage / CLI (not the older remap-baseline lie that `Unresolved` is absent or that `applicability.toml` is only `applicable = true`).

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `project_soa(framework, version)` | `soa.rs` | Keep signature. Live convenience over pack **default rules / structural flags** + empty graph. Must not be the historical reconstruction path. After implement, missing implementations are first-class `notImplemented` rows, **not** NA. |
| `project_soa_from_snapshot` | `soa.rs` | Historical reconstruction. Crate-root re-export. Must not reread pack files. |
| `StatementOfApplicabilitySnapshot` | `lineage.rs` | Already `{schema, digest, frameworkPackDigest, soa}`. Add a pin/seal helper that **computes** digest. |
| `Applicability` (SoA) | `soa.rs` | Keep `Applicable / NotApplicable / Unresolved`. Map `Unresolved` ↔ Kleene `ManualDeterminationRequired`. |
| Kleene engine | `weeping-angel-assurance::applicability` | **Landed.** Consume `ApplicabilityDecision` / outcomes / snapshot. Do not reimplement. Pack TOML is not a second evaluator. |
| `ControlImplementation` | IR `implementation.rs` | Status ≠ effectiveness. Consume `id()`, `control_id()`, `status()`, `risk_ids()`, `exception_ids()`. `owner` is currently a **private** field with no getter (Prompt 10 unlanded) — project `owner: None` rather than implementing the registry. |
| `ImplementationStatus` | IR | `NotImplemented` is the first-class missing-implementation state. IR `NotApplicable` on an implementation must **not** flip SoA applicability by itself. |
| `Exception` | IR `exception.rs` | Public `rationale`, `status`, `approved_by`, `expires_at`. NA governance uses these (or projection-local approval metadata). `Expired` / past `expires_at` → readiness gap. |
| `Risk` | IR `risk.rs` | Operational register (Prompt 06 landed). Consume as linked-risk ids/titles. Do not re-expand `Risk` here. |
| `AssessmentDefinition` | IR | Already carries `implementations`, `risks`, `exceptions`, `mappings`, `scope`, `requirements`. Operational input wraps this plus results + versioned refs. |
| `AssessmentScope` | IR + assurance facade | Consume existing types. Do not implement Prompt 02 scope engine. |
| `ControlTestResult` / `Effectiveness` | `weeping-angel-control-test` | Effectiveness dimension. `InsufficientEvidence` is first-class, not NA. |
| Mappings | pack `mappings.toml` + IR `Mapping` | Consume. Partial (`PartiallySatisfies` / `Supports` / `Related` / `EvidenceFor` / `SubsetOf`) stays partial. Do not rewrite `to =`. |
| `SnapshotDiff` / `compare` | `snapshot.rs` | Extend with SoA cause taxonomy. Do not collapse to effective/ineffective/stale only. |
| Prompts 02 / 06 / 08 / 10 | 06 register landed; others neighbor | **MINIMUM versioned references** (§4.4). **FAIL CLOSED** when a required ref is absent. Do not implement those engines here. |
| CLI `AssuranceCommand::Soa` | `src/cli.rs` / `src/main.rs` | Today catch-all banner-and-exit-0. Optional wire in `assurance_soa.rs` without compiler topology. |
| Neighbor suites | root `Cargo.toml` | Keep `sdd_iso27001_assurance_target`, `sdd_iso27001_remap_target` (ISO-R-009 / g07 / g08), `sdd_assessment_lineage_target` (LIN-011 snapshot/digest needles), `sdd_applicability_engine_target` GREEN. |

Live ISO pack defaults (must remain representable so remap stays green):

| Reference | Pack `applicability` | Current `project_soa` |
| --- | --- | --- |
| `A.5.19` | `not-applicable` (`applicable = false`) | `Applicability::NotApplicable` — supplier context rationale (not missing evidence) |
| `A.8.13` | `unresolved` (`applicable = true`) | `Applicability::Unresolved` — incomplete org context |
| remaining SoA rows (`A.5.1`, `A.5.15`, …) | `applicable` | `Applicability::Applicable` |

`Applicability::from_pack` already accepts `applicable` / `not-applicable` / `unresolved` / `manual` / `manualdeterminationrequired` plus boolean fallback. Do not regress that parser. Baseline must **not** claim Unresolved is absent.

---

## 3. Current behavior (baseline — GREEN on CURRENT HEAD)

Characterized from current product sources (not SHA `e430980c…` remap-baseline comments).

### 3.1 Live `project_soa` rereads pack TOML

[`crates/weeping-angel-assurance/src/soa.rs`](../../crates/weeping-angel-assurance/src/soa.rs):

```text
load_framework_pack(framework, version) → mapped_controls from pack mappings
resolve_pack_dir(framework, version) → read applicability.toml → [[entry]]
Applicability::from_pack(raw state, applicable bool fallback)
implementation_state = "assessed"
automated_effectiveness = None
evidence = []
exceptions = []
manual_review_state = pending | not applicable | "manual determination required"
```

`SoaEntry` public fields today: `reference`, `applicability`, `applicable` (bool alias of `Applicable`), `applicability_rationale`, `implementation_state`, `automated_effectiveness`, `manual_review_state`, `evidence`, `exceptions`, `mapped_controls`, `notes`.

No linked risks, treatment rationale, implementation references, owner, review/approval metadata, evidence lineage (digests), or readiness gaps.

### 3.2 Snapshot clone exists; not a sealed historical API

`project_soa_from_snapshot` clones `snapshot.soa` and does not recompute. It is **not** crate-root re-exported (`lib.rs` exports `StatementOfApplicability` + `project_soa` only). `StatementOfApplicabilitySnapshot` is `{ schema, digest, frameworkPackDigest, soa }` with **caller-supplied** digest — no `pin_soa_snapshot` / `seal_soa_snapshot` helper.

Lineage already has the persist type (LIN-011). Live CLI/path still uses disk TOML.

### 3.3 CLI is still the catch-all

[`src/main.rs`](../../src/main.rs): `Catalog` and `Explain` are dispatched. `AssuranceCommand::Soa` falls through to print `"This is a readiness assessment and is not certification."` and exit **0**. There is no `src/assurance_soa.rs`.

### 3.4 Missing implementation is not a first-class row

Every live row is `implementation_state = "assessed"` regardless of `AssessmentDefinition.implementations` (unused). Applicable + not-implemented cannot be distinguished from NA. IR `ImplementationStatus::NotImplemented` is never projected.

### 3.5 `SnapshotDiff` has no SoA cause taxonomy

[`crates/weeping-angel-assurance/src/snapshot.rs`](../../crates/weeping-angel-assurance/src/snapshot.rs) `SnapshotDiff` has control effectiveness flips, stale evidence, subjects, requirement applicable/NA flips, exceptions, evidence add/remove/supersede, pack/catalog digest booleans. There is **no** cause enum for:

```text
applicability change
implementation change
effectiveness regression
exception expiry
mapping change
treatment change
```

### 3.6 Disclaimer is already non-certifying

Live disclaimer: `"This Statement of Applicability projection is a readiness aid and is not certification."` Preserve this class of language.

### 3.7 Kleene engine is landed but unused by SoA

`weeping-angel-assurance::applicability::ApplicabilityDecision` is `Applicable / NotApplicable / ManualDeterminationRequired`. SoA still copies pack TOML through `from_pack`. Pack `applicability.toml` is treated as the live boolean/three-state **copy**, not merely default rules / structural flags.

---

## 4. Desired behavior (target — RED on CURRENT, GREEN after implement)

### 4.1 Operational row (every framework control/requirement in the ISO projection)

For each SoA-oriented requirement/control, the projection MUST expose:

| Dimension | Meaning | Must not collapse into |
| --- | --- | --- |
| Applicability state | `Applicable` / `NotApplicable` / `Unresolved` (↔ Kleene `ManualDeterminationRequired`) | Implementation or evidence |
| Applicability rationale | Ordered explain text / engine rationale codes | Empty NA |
| Linked risks | Risk ids (and titles if present) from IR `Risk` / implementation `risk_ids` | Silent omission when required |
| Treatment rationale | From `RiskTreatmentRef` when inclusion/exclusion is treatment-driven | Invented engine |
| Canonical controls | Mapped catalog ids (consume pack/IR mappings) | Remapped pack ids |
| Implementation references | `ControlImplementation.id` values | Control definition ids only |
| Owner | Principal when readable; else `None` (Prompt 10 unlanded) | NA |
| Implementation status | IR `ImplementationStatus` (or stable camelCase string). Missing registry row ⇒ `notImplemented` | `assessed`, NA |
| Effectiveness status | `Effectiveness` from tests. None/untested/insufficient stay visible | NA |
| Evidence lineage | Evidence envelope digests / refs from `ControlTestResult.evidence_refs` + missing evidence | Empty-by-default forever |
| Exclusions / exceptions | Exception ids + status | Hidden NA |
| Review state | pending / approved / expired / manual determination required / readiness gap | Silent |
| Approval metadata | Principal + time/review identity when NA or exception requires it | Pack flag alone |

JSON camelCase field names (serde-default so existing decoders keep working):

```text
linkedRisks
treatmentRationale
treatmentRefs
canonicalControls          # alias of / same as mappedControls
implementationRefs
owner
implementationStatus       # preferred over hardcoded implementationState="assessed"
effectivenessStatus        # Option<Effectiveness>; may coexist with automatedEffectiveness
evidenceLineage
readinessGaps
reviewState
approval                   # { principal, approvedAt?, expiresAt?, reviewState }
inclusionReasons
exclusionReasons
```

Existing fields (`applicability`, `applicable`, `applicabilityRationale`, `mappedControls`, `disclaimer`) stay. `implementationState` may remain as a compatibility alias of `implementationStatus`.

### 4.2 Applicability ⊥ implementation ⊥ effectiveness

Hard invariants:

1. **Missing implementation MUST NOT become not applicable.** Applicable + `notImplemented` is a first-class row.
2. **Insufficient / missing evidence MUST NOT become not applicable.** Use `Effectiveness::InsufficientEvidence` (or empty evidence lineage + gap), applicability stays `Applicable`.
3. **IR `ImplementationStatus::NotApplicable` does not set SoA `NotApplicable`.** SoA NA is applicability + NA governance only.
4. **`Implemented` does not imply `Effective`.** Effectiveness comes only from `ControlTestResult` / `Effectiveness`.
5. Pack `applicable = false` / `applicability = "not-applicable"` is a **structural default**, not a silent live boolean.

### 4.3 Non-applicability governance

`NotApplicable` requires:

- explicit rationale (context — never “no evidence” / “missing evidence” / “insufficient evidence”);
- accountable approval/review semantics: principal + review state;
- validity: unexpired (`expires_at` in the future or absent-with-explicit-open-ended review).

Pack-declared NA (ISO `A.5.19`) **remains representable as `NotApplicable`** so `sdd_iso27001_remap_target` ISO-R-009 / golden-7 stay GREEN. If approval/review is missing or expired, the row **also** surfaces a readiness gap (e.g. `missingNaApproval` / `expiredNaApproval`). That is **not** silent NA and **not** a remap regression.

Expired rationale/approval ⇒ readiness gap (do not drop the row; do not coerce to Applicable without a new decision).

`Unresolved` / `ManualDeterminationRequired` remains first-class (ISO `A.8.13`). Incomplete context stays unresolved, never coerced to NA.

### 4.4 Operational input and fail-closed refs

New types live in `soa.rs` (projection layer, not IR schema fork):

```text
OperationalSoaInput {
  schema = weeping-angel/operational-soa-input/v1
  framework, version
  assessment: AssessmentDefinition          # implementations, risks, exceptions, mappings, scope
  kleene: Option<applicability::ApplicabilitySnapshot>   # prefer over pack copy
  results: Vec<ControlTestResult>
  treatments: Vec<RiskTreatmentRef>        # MINIMUM — not Prompt 08
  risk_register: Option<RiskRegisterRef>   # MINIMUM — not Prompt 06
  as_of: DateTime<Utc>
}

RiskTreatmentRef {
  schema = weeping-angel/risk-treatment-ref/v1
  id, risk_id
  strategy                    # mitigate | accept | avoid | transfer (string; no state machine)
  digest                      # required when the ref is present
  approved_by?, valid_until?
}

RiskRegisterRef {
  schema = weeping-angel/risk-register-ref/v1
  digest                      # required when the ref is present
  assessment_id?
}
```

`project_operational_soa(input) -> Result<StatementOfApplicability, OperationalSoaError>`

**FAIL CLOSED** (do not invent treatments/risks/implementations):

| Condition | Error / gap |
| --- | --- |
| Any inclusion/exclusion reason is treatment-driven and `treatments` is empty or a cited `id` is missing | `MissingRiskTreatment` |
| `treatments` non-empty and `risk_register` is `None` or `digest` empty | `MissingRiskRegister` |
| Present treatment/register ref with empty `digest` | `MissingInputDigest` |
| Kleene snapshot required by caller (replay) and absent, with no pack structural default | `MissingApplicabilitySnapshot` |

Live `project_soa(framework, version)` builds an input with pack structural flags, empty implementations/results/treatments, and **must not** fail closed for missing treatment engines. It projects applicable + `notImplemented` + no effectiveness. It is a convenience, not history.

When Kleene results are provided, use them. Map:

```text
Applicable                    ↔ Applicable
NotApplicable                 ↔ NotApplicable
ManualDeterminationRequired   ↔ Unresolved
```

Pack `applicability.toml` supplies default rules / `soa = true` structural flags / fallback rationale **only** when Kleene has no decision for that requirement.

### 4.5 Partial canonical mapping

Walk the actual mapping graph. A requirement whose mappings are only `PartiallySatisfies` / `Supports` / `Related` / `EvidenceFor` / `SubsetOf` is **partially mapped**. The SoA row stays applicable (unless Kleene/NA governance says otherwise), lists the mapped catalog ids, and records a gap or note `partialCanonicalMapping`. It must **not** become NA because the mapping is incomplete. Do not rewrite pack `to =` targets.

### 4.6 Risk-treatment-driven applicability

When a `RiskTreatmentRef` is the reason a requirement is included or excluded:

- cite `treatmentRefs` + `treatmentRationale`;
- require the versioned ref + register digest (§4.4);
- do not evaluate residual risk (Prompt 09 fence);
- do not implement the treatment state machine (Prompt 08 fence).

Absence of the treatment engine is **not** NA.

### 4.7 Immutable snapshots and diffs with causes

```text
pin_soa_snapshot(soa, framework_pack_digest) → StatementOfApplicabilitySnapshot
digest = typed_canonical_digest("soa-snapshot", body excluding digest)
```

Digest MUST be a function of the pinned `soa` payload + pack digest + schema, **not** of live pack file bytes after the pin. Reconstruct with `project_soa_from_snapshot` only.

`diff_soa_snapshots(previous, next) → SnapshotDiff` (or populate an extended `SnapshotDiff`) MUST classify each material row change with one or more of:

```text
SoaDiffCause =
  ApplicabilityChange
  | ImplementationChange
  | EffectivenessRegression
  | ExceptionExpiry
  | MappingChange
  | TreatmentChange
```

Effectiveness regression includes `Effective` → `Ineffective` / `PartiallyEffective` / `InsufficientEvidence` / `StaleEvidence`. Exception expiry includes `ExceptionStatus::Expired` or `expires_at <= as_of`.

Live `project_soa` must **not** be the sole historical reconstruction path. A pinned snapshot’s digest stays stable if someone later edits `frameworks/iso-27001/2022/applicability.toml` (tests must not mutate the repo pack: compare pin vs live re-read conceptually via source + `project_soa_from_snapshot` independence).

### 4.8 Public API / CLI

Crate-root exports after implement:

```text
project_soa
project_soa_from_snapshot
project_operational_soa
pin_soa_snapshot
diff_soa_snapshots          # or compare extension documented here
StatementOfApplicability
SoaEntry
Applicability
OperationalSoaInput
OperationalSoaError
RiskTreatmentRef
RiskRegisterRef
```

Optional CLI: `src/assurance_soa.rs` prints the not-certification banner then JSON of the operational (or pinned) projection. No `compilerTopology`. Unknown assessment id for a pinned path exits non-zero (same family as `explain`). Live convenience may still project `iso-27001` / `2022` defaults.

### 4.9 Language

Never emit: `ISO 27001 certified`, `ISO 27001 compliant`, `certification guaranteed`, `audit passed`.

Allowed: ready, applicable, not applicable, unresolved, not implemented, effective, ineffective, insufficient evidence, stale evidence, manual review required, readiness gap, partially mapped.

Do not store licensed ISO/IEC normative text. Titles/rationale stay structural / organization-context.

---

## 5. Acceptance criteria (testable)

1. Every ISO SoA-oriented requirement row exposes applicability, rationale, linked risks, treatment rationale/refs, canonical/mapped controls, implementation refs, owner, implementation status, effectiveness, evidence lineage, exceptions, review state, approval metadata, and readiness gaps (serde-default empty when genuinely absent).
2. Applicable + not implemented is representable and is **not** `NotApplicable`.
3. Applicable + insufficient evidence is representable and is **not** `NotApplicable`.
4. Applicable + effective is representable when implementations + `Effectiveness::Effective` results are supplied to `project_operational_soa`.
5. Approved NA requires explicit rationale + principal + review; remap live `A.5.19` remains `NotApplicable` with context rationale (not missing evidence).
6. Expired or missing NA approval surfaces a readiness gap (not silent NA).
7. Partial canonical mapping stays applicable (unless Kleene/NA says otherwise) and lists mapped catalog ids.
8. Treatment-driven applicability fail-closes on missing `RiskTreatmentRef` / register digest; it does not invent a treatment engine.
9. `pin_soa_snapshot` digest is deterministic and independent of later pack-file edits; `project_soa_from_snapshot` reconstructs history; live `project_soa` is not the historical path.
10. Snapshot diff emits the six-cause taxonomy when those dimensions change.
11. Live `project_soa` still uses generic three-state (`Unresolved` representable). Pack TOML is default/structural, not a second Kleene evaluator.
12. Disclaimer remains a readiness-not-certification statement. No licensed ISO body text. IR `Control` / `ControlImplementation` gain no Annex A fields.
13. Neighbor suites listed in the header stay GREEN.

---

## 6. Dual-suite contents

### 6.1 Baseline (`sdd_operational_soa_baseline`) — GREEN on CURRENT

Assert **today’s** projection (already true on current `soa.rs`):

| ID | Assertion |
| --- | --- |
| SOA-B01 | Dual-suite names registered in root `Cargo.toml`; this spec path listed in `documentation_layout.rs` `CANONICAL_SPECS` |
| SOA-B02 | `project_soa` source contains `resolve_pack_dir` and `applicability.toml` |
| SOA-B03 | `Applicability::from_pack` maps Applicable / NotApplicable / Unresolved (including `manualdeterminationrequired`) |
| SOA-B04 | Live ISO SoA: `A.5.19` `NotApplicable`, `A.8.13` `Unresolved`, `A.5.1` `Applicable` — **do not** assert Unresolved is absent; **do not** assert pack is only `applicable = true` |
| SOA-B05 | Every live row `implementation_state == "assessed"`; `automated_effectiveness` is `None`; `evidence` and `exceptions` empty |
| SOA-B06 | `mapped_controls` come from pack mappings (at least one ISO SoA row has a non-empty mapping list) |
| SOA-B07 | Disclaimer contains readiness language and “not certification” |
| SOA-B08 | `project_soa_from_snapshot` exists in `soa.rs` and clones `snapshot.soa`; **not** crate-root `pub use`d in `lib.rs` |
| SOA-B09 | `StatementOfApplicabilitySnapshot` fields `{schema, digest, framework_pack_digest, soa}` exist |
| SOA-B10 | `src/main.rs` Soa path is the assurance catch-all (banner + exit 0); no `assurance_soa.rs` required today |
| SOA-B11 | `SnapshotDiff` source has no SoA cause taxonomy needles (`ApplicabilityChange` / `ImplementationChange` / `EffectivenessRegression` / `ExceptionExpiry` / `MappingChange` / `TreatmentChange` as a SoA cause enum) |
| SOA-B12 | Missing implementation is not first-class: no live row is applicable + `notImplemented` |

After implement: skip-supersede (`#[ignore = "superseded by sdd_operational_soa_target"]`) or let them fail as documented.

### 6.2 Target (`sdd_operational_soa_target`) — RED on CURRENT for the right reason, GREEN after

Compile-safe (no import of symbols that do not exist yet). Fail because operational graph / pin / causes / first-class not-implemented are missing — not because of unrelated compile noise.

| ID | Assertion |
| --- | --- |
| SOA-T01 | Applicable + effective: source exports `project_operational_soa`; wires `Effectiveness::Effective` + `ImplementationStatus::Implemented`; operational field names present on `SoaEntry` / projector |
| SOA-T02 | Applicable + not implemented: live or operational projection emits `Applicable` + `notImplemented` and **not** `NotApplicable` for a requirement with no implementation (ISO `A.5.1` live convenience is the found case) |
| SOA-T03 | Applicable + insufficient evidence: projector consumes `Effectiveness::InsufficientEvidence`; must not map that to NA |
| SOA-T04 | NA approved: explicit rationale + principal + review needles / JSON `approval` |
| SOA-T05 | NA expired → readiness gap (`expiredNaApproval` / `readinessGaps` / `ExceptionStatus::Expired`) |
| SOA-T06 | Partial canonical mapping: ISO `A.8.5` (or any partial mapping) stays non-NA and lists catalog `control.*` ids |
| SOA-T07 | Risk-treatment-driven applicability: `RiskTreatmentRef` / `MissingRiskTreatment` fail-closed needles; no residual-risk / treatment-engine implementation |
| SOA-T08 | Snapshot diff with causes: `SoaDiffCause` (or equivalent) lists the six causes |
| SOA-T09 | Missing implementation ≠ NA (live ISO applicable rows must not flip to NA merely because implementations are empty) |
| SOA-T10 | Historical reconstruction: `pin_soa_snapshot` / computed digest; `project_soa_from_snapshot` crate-root export; projector clone path does not `resolve_pack_dir`; digest independent of later pack-file edit |
| SOA-T11 | Live `project_soa` is not the sole historical path (source + snapshot type) |

Do not `#[ignore]` these tests. Do not implement the projector inside the test file.

---

## 7. Out of scope

- Licensed ISO/IEC 27001 normative text in packs, SoA rows, or fixtures
- Certification / compliance / audit-passed claims
- Dashboards or UI
- Reimplementing Kleene applicability
- Remapping ISO pack IDs or catalog domain TOML
- Prompt 09 residual-risk engine
- Prompt 10 control-implementation registry (additive IR fields, overlap integrity, supersession)
- Prompt 02 scope engine, Prompt 06 risk-register engine, Prompt 08 treatment state machine
- Forking `assurance-ir/v1` or adding Annex A fields to generic IR objects
- Evidence crate conclusions
- New crates

---

## 8. Risks

- Live convenience vs remap: changing `A.5.19` / `A.8.13` representation would fail `sdd_iso27001_remap_target`. Mitigation: keep three-state + justified NA; add gaps beside NA, do not delete NA.
- Prompt 10 `owner` still private: projecting owner requires a getter this slice must not invent on IR. Mitigation: `owner: None` until registry lands.
- Fail-closed refs vs empty live path: over-strict fail-closed on live `project_soa` would empty the ISO SoA. Mitigation: fail-closed only on `project_operational_soa` when treatment-driven reasons are claimed.
- SnapshotDiff extension vs lineage compare tests: additive serde-default fields only.
- Dual Prompt 11 naming (canonical lineage vs operational SoA): cite **paths**; do not reuse `sdd_assessment_lineage_*`.
- Accidental pack-file mutation in digest tests: never write the repo `applicability.toml` from contracts.

---

## 9. SDD protocol (abort rather than skip)

```text
Spec (this file; no product feature code)
  → Baseline GREEN on CURRENT code
  → Target RED on CURRENT code for the right reason
  → Implement product code in crate homes
  → Docs + ADR finalize
  → Iterate until Target GREEN
  → Prove Baseline FAILS or skip-supersede with additive documentation
  → Target still GREEN
```

Generated traces only under `.sdd/runs/` and `.sdd/artifacts/` (ADR 0004).

`adr_needed = true`: SoA becomes an operational-graph projection with immutable snapshots/diffs and a public CLI/report contract.

---

## 10. Definition of done

The SoA is mostly generated from the operational ISMS graph and can explain every inclusion, exclusion, implementation, and effectiveness state. It remains a readiness projection, reconstructable from a pinned snapshot, and never a certification claim.

---

## 11. As implemented (2026-08-19)

Landed in `weeping-angel-assurance::soa` (crate-root re-exports) and `src/assurance_soa.rs`.

- `project_operational_soa` projects Kleene first, pack structural defaults second; empty implementations are `notImplemented`, never NA.
- `SoaEntry` exposes operational fields (`linkedRisks`, `treatmentRefs`, `implementationStatus`, `effectivenessStatus`, `evidenceLineage`, `readinessGaps`, `approval`, inclusion/exclusion reasons). `implementationState` aliases status. `owner` is `None` until Prompt 10.
- NA without unexpired approved exception → `missingNaApproval`; expired → `expiredNaApproval`. Pack `A.5.19` stays `NotApplicable`; `A.8.13` stays `Unresolved`.
- Fail-closed: `MissingRiskTreatment` / `MissingRiskRegister` / `MissingInputDigest` / `MissingApplicabilitySnapshot`. Live `project_soa` does not fail closed for empty treatments.
- `pin_soa_snapshot` digest = `typed_canonical_digest("soa-snapshot", {schema, frameworkPackDigest, soa})`. History = `project_soa_from_snapshot`. `diff_soa_snapshots` fills `SnapshotDiff.soaCauses`.
- CLI `assurance soa` prints the not-certification banner then JSON. Dual-suite target GREEN; baseline skip-superseded. ADR [`docs/adr/0003-operational-soa.md`](../adr/0003-operational-soa.md).
