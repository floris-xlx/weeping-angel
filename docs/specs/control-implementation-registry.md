# SDD: Control Implementation Registry

| Field | Value |
| --- | --- |
| Status | **Implemented** — target CIR-001–015 GREEN; baseline skip-superseded |
| Program | Operational ISMS v1 — control-implementation registry |
| Slice | Extend existing IR `ControlImplementation` + `ImplementationStatus` + `validate_assessment_ir` into an operational registry: populations/assets, review, evidence *expectations*, document refs, treatments, automation, overlap integrity, supersession/history. Effectiveness stays on control tests. |
| Dual-suite | `sdd_control_implementation_registry_baseline` · `sdd_control_implementation_registry_target` (`tests/contracts/control_implementation_registry.{baseline,target}.rs`) — registered in root [`Cargo.toml`](../../Cargo.toml) (directory is **not** auto-discovered). Baseline skip-superseded after it failed on the new contract. |
| ADR | Accepted [`docs/adr/0003-control-implementation-registry.md`](../adr/0003-control-implementation-registry.md) (0003-* program sibling; 0004 is documentation architecture). Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Population (`applies_to` for registry integrity; unused by evaluator) | [`docs/specs/population-runtime.md`](population-runtime.md) §3.5 |
| Lineage (must keep compiling) | [`docs/specs/assessment-lineage.md`](assessment-lineage.md) — `ControlExplanation.implementation: Option<ControlImplementation>` |
| Neighbor IR (keep green) | `sdd_compliance_ir_target` (IR-008/009/019/020 + golden fixture `tests/fixtures/assurance-ir/v1/control-implementation.json`) |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Workspace verify | `cargo test --test sdd_control_implementation_registry_baseline`; `cargo test --test sdd_control_implementation_registry_target`; `cargo test --test sdd_documentation_layout`; keep `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_assessment_lineage_target` GREEN; `cargo test --workspace --features demo` when practical |

This document is the durable SSOT for Operational ISMS v1 control-implementation registry. It owns **how this organization implements a canonical control**, **registry integrity** (dangling refs, overlap, supersession), and the **additive** `ControlImplementation` / `ImplementationStatus` contract. It does **not** own ISO Annex A fields, provider APIs, evidence conclusions, residual-risk projection (residual risk), the full scope engine (scope engine), Kleene applicability, or catalog domain TOML.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

The registry inserts an organizational layer **beside** the canonical control, never **as** the control and never **as** effectiveness:

```text
what the control means     = Control (canonical)
how this org implements it = ControlImplementation (this slice)
whether it is effective    = ControlTestResult.effectiveness (tests only)
```

`Implemented` MUST NOT imply `Effective`. There is no `effectiveness` field on `ControlImplementation`.

---

## 0. Collision fence (concurrent SDD)

This slice may edit only implementation-registry paths listed in §8. Do not reimplement scope engine, and do not fork IR schema.

| Do not touch | Owner |
| --- | --- |
| `docs/specs/residual-risk.md`, `tests/contracts/residual_risk.*`, `crates/**/residual*.rs`, `docs/adr/*residual*` | residual risk (landed; [`residual-risk.md`](residual-risk.md)) |
| `tests/contracts/github_collector.*`, `crates/weeping-angel-collector/src/github/**` | Canonical Assurance residual risk collector |
| `catalog/canonical/v1/**` domain TOML, ISO pack requirement/control IDs, pack `to =` remaps, `tests/contracts/iso27001_remap.*` | controlled documents / catalog owners |
| Applicability Kleene evaluator modules (`weeping-angel-assurance::applicability`) | Canonical Assurance control-implementation registry |
| Unrelated catalog SDD suites (`iam` / `sdlc` / `vuln` / `infra` / `governance`) | Those slices |
| Dashboards / UI / provider adapters | Out of scope |

Suggested **product** modules stay in **existing crates** (no new crate):

| Concern | Home |
| --- | --- |
| `ControlImplementation`, `ImplementationStatus`, additive fields, serde | `crates/weeping-angel-assurance-ir/src/implementation.rs` |
| Dangling refs, overlap fail-closed, evidence-expectation presence, duplicate ids, supersession graph | `crates/weeping-angel-assurance-ir/src/validation.rs` (`validate_assessment_ir`) |
| Re-exports | `crates/weeping-angel-assurance-ir/src/lib.rs` |
| Registry queries / overlap reports | `crates/weeping-angel-assurance-ir/src/registry.rs` (pure over IR; re-exported from `lib.rs`). No engine/collector context. |
| Evidence crate | **conclusion-free**. Do not put implementation effectiveness or evidence pass/fail on envelopes |

Tiny allowed adjustments: additive `#[serde(default)]` fields and new `ImplementationStatus` variants with **new** camelCase names; new getters/builders; IR validation messages. Do **not** redesign `AssessmentDefinition` core fields, collector discovery, ISO pack IDs, or lineage snapshot schemas.

Lineage constraint: `ControlExplanation.implementation: Option<ControlImplementation>` and `explain_control` first-match-by-`control_id` must keep compiling and deserializing. This slice does not change that pin. Multiple implementations of one control are a registry fact; lineage may still first-match until a later explain slice pins by implementation id.

scope engine (scope engine) may not be fully landed. **Reuse** `AssessmentScope`, `ScopeExclusion`, `SubjectSelector`, `Asset`, `Identity`, `Vendor` as they exist. Do **not** invent `ScopeResolution` here.

---

## 1. Problem / user-visible goal

The IR already has a `Control` (canonical meaning) and a thin `ControlImplementation` (organizational state). Operators cannot yet record, query, or validate **how this organization actually implements** a control:

- One control rolled out to employees but not contractors looks like a single `Implemented` row with unused `applies_to`.
- Partial rollout, retired rows, and “turned off” implementations share a status enum that cannot say **disabled/ineffective-as-state** or **unknown**.
- Overlapping subject selectors can be stored with no error, so coverage math can double-count the same population.
- There is no review cadence, next review, evidence *expectation* list, policy/document pointer, treatment link, automation class, or supersession chain.
- `validate_assessment_ir` catches dangling `control_id` / `risk_ids` / `exception_ids` only. Dangling subjects and assets are silent.
- `explain_control` pins at most one implementation by first `control_id` match.
- Readers can confuse `ImplementationStatus::Implemented` with `Effectiveness::Effective`.

**User-visible goal:** given a canonical control id, the engine can answer:

```text
what does this control mean?                  → Control
how does this organization implement it?      → ControlImplementation[] (per population / system)
over which subjects / assets?                 → applies_to + asset_ids (explainable selectors)
in what organizational state?                 → ImplementationStatus (not effectiveness)
when is it effective, and when is it reviewed?→ effective_from / implemented_at + cadence + next_review
what evidence is expected (not concluded)?    → EvidenceRequirementId refs
which policies / risks / treatments / exceptions apply?
if we replaced it, can we still query the prior snapshot?
is that implementation actually effective?    → control tests only
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `ControlImplementation` | `weeping-angel-assurance-ir::implementation` | **Extend this type.** Do not create `OrgControlImplementation`, `ControlDeployment`, or a second registry record. |
| `ImplementationStatus` | same | Additive variants only. Existing camelCase values stay: `notImplemented`, `planned`, `partiallyImplemented`, `implemented`, `notApplicable`, `retired`. |
| `PrincipalRef` | same | Reuse for `owner`. |
| `SubjectSelector` | `subject.rs` | SSOT for scoped populations. `{ kind, ids, tags, scope }`. |
| `AssessmentScope` / `ScopeExclusion` | `assessment.rs` | Reuse. Implementations do not replace assessment scope. |
| `Asset` / `Identity` / `Vendor` | IR | Inventory for dangling-id checks. |
| `AssessmentDefinition.implementations` | `assessment.rs` | Registry storage. One assessment may list several implementations per `control_id`. |
| `validate_assessment_ir` | `validation.rs` | Grow here. Keep IR-019/020 messages meaningful. |
| `Control` | `control.rs` | Canonical meaning. `implementation_expectation` stays an expectation summary, not org state. |
| `EvidenceRequirement` | `evidence.rs` | Expectation target. Implementation stores **ids**, never pass/fail. |
| `Risk` | `risk.rs` | Existing `{ id, title, description, status }`. No residual-risk type here. |
| Risk treatment | risk treatment — **not landed** | Store `treatment_ids` as stable ids. Fail closed **only when** a treatments collection exists on the assessment. Do not invent `TreatmentPlan` in this slice. |
| Controlled documents | controlled documents — **not landed** | Store opaque `DocumentRef`s. No document registry validation yet. |
| `Effectiveness` | `weeping-angel-control-test` | Different type. IR-009 remains law. |
| `ControlExplanation` | `weeping-angel-assurance::lineage` | Keep `implementation: Option<ControlImplementation>` and serde. Do not break first-match. |
| Golden fixture | `tests/fixtures/assurance-ir/v1/control-implementation.json` | `{ schemaVersion, id, controlId, status: "implemented" }` must still deserialize. New fields default. |
| Dual-suite neighbors | root `Cargo.toml` | Do not disturb green targets listed in the header. |

Serde compatibility law:

- Existing JSON **without** new fields deserializes (`#[serde(default)]`, skip empty).
- Existing status strings keep the same meaning. Do **not** remap `implemented` → effective, `retired` → disabled, or `notImplemented` → unknown.
- New status strings are new identifiers (`ineffective`, `unknown`). Optional alias `disabled` → `Ineffective` is allowed **only** as an extra accept-name for the new variant, never as a reinterpretation of an old value.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Executable characterization lives in `sdd_control_implementation_registry_baseline` once that suite is written. Until then this section **is** the baseline contract: tests must assert these facts against **current** `implementation.rs` + `validation.rs`, and they must pass on SHA `6e31bf1` before any product edit.

### 3.1 Type and comment

`crates/weeping-angel-assurance-ir/src/implementation.rs` module docs:

```text
Organizational implementation state. Not control effectiveness.
```

`Control` and `ControlImplementation` are distinct types (IR-008). `ImplementationStatus` and `Effectiveness` are distinct types (IR-009). `ControlImplementation` has **no** `effectiveness` field.

Re-exports today (`lib.rs`): `ControlImplementation`, `ImplementationStatus`, `PrincipalRef`.

### 3.2 `ImplementationStatus`

```text
#[serde(rename_all = "camelCase")]
enum ImplementationStatus {
    #[default] NotImplemented,   // notImplemented
    Planned,                     // planned
    PartiallyImplemented,        // partiallyImplemented
    Implemented,                 // implemented
    NotApplicable,               // notApplicable
    Retired,                     // retired
}
```

Absent: ineffective/disabled, unknown. No `Effective` variant (correct — keep it that way).

### 3.3 `ControlImplementation` fields

Private fields, camelCase JSON:

| Field | Type | Notes |
| --- | --- | --- |
| `schema_version` | `String` | `ASSURANCE_IR_SCHEMA` = `assurance-ir/v1` |
| `id` | `ControlImplementationId` | stable id |
| `control_id` | `ControlId` | canonical control |
| `status` | `ImplementationStatus` | default `NotImplemented` in `new` |
| `owner` | `Option<PrincipalRef>` | skip if none |
| `description` | `Option<String>` | skip if none |
| `implemented_at` | `Option<DateTime<Utc>>` | skip if none |
| `applies_to` | `Vec<SubjectSelector>` | skip if empty; **unused by evaluator** |
| `compensating_controls` | `Vec<ControlId>` | skip if empty |
| `exception_ids` | `Vec<ExceptionId>` | skip if empty |
| `risk_ids` | `Vec<RiskId>` | skip if empty |

**Missing vs control-implementation registry:** systems/assets, distinct effective date, review cadence, next review, evidence expectations, policy/document refs, treatment ids, automation classification, supersession/history.

### 3.4 Builders and getters

Builders: `new(id, control_id)`, `with_status`, `with_risk`, `with_exception`.

Public getters **only**: `id`, `control_id`, `status`, `risk_ids`, `exception_ids`.

No getters for `applies_to`, `owner`, `description`, `implemented_at`, `compensating_controls`, `schema_version`.

### 3.5 Validation (`validate_assessment_ir`)

For each `assessment.implementations` row:

- `control_id` must exist in `assessment.controls` — else `dangling implementation control {id}`.
- each `risk_ids` entry must exist in `assessment.risks` — else `dangling risk reference {id}` (IR-019).
- each `exception_ids` entry must exist in `assessment.exceptions` — else `dangling exception reference {id}` (IR-020).

Does **not** validate:

- duplicate `ControlImplementationId`
- dangling `applies_to` subject / identity / vendor ids
- dangling asset ids (no asset field)
- dangling `compensating_controls`
- overlapping population / asset coverage
- review cadence / next review
- supersession graph (`supersedes` / `superseded_by`)
- evidence expectations or required evidence refs
- `Implemented` vs test effectiveness (no coupling)

### 3.6 Population runtime

[`population-runtime.md`](population-runtime.md) §3.5: `ControlImplementation.applies_to` remains unused by the control-test evaluator. Population coverage stays test-side. The registry uses `applies_to` for overlap integrity and query.

Facade `AssessmentScope` (`BTreeSet<AssetId>` allow-set) remains a different type from IR `AssessmentScope`. Do not collapse names.

### 3.7 Lineage pin

`ControlExplanation.implementation: Option<ControlImplementation>`. `explain_control` sets it to the **first** `assessment.implementations` row whose `control_id` matches. Multiple rows for one control are already representable in the vec; only the first is explained.

### 3.8 Golden fixture

`tests/fixtures/assurance-ir/v1/control-implementation.json`:

```json
{
  "schemaVersion": "assurance-ir/v1",
  "id": "impl.access.mfa.org",
  "controlId": "control.access.mfa",
  "status": "implemented"
}
```

`sdd_compliance_ir_target` `ir_golden_fixtures_round_trip` asserts `status() == Implemented`.

### 3.9 Baseline suite obligations (must PASS on current code)

| Id | Characterization |
| --- | --- |
| CIR-B01 | `ImplementationStatus` variants are exactly `{NotImplemented, Planned, PartiallyImplemented, Implemented, NotApplicable, Retired}` |
| CIR-B02 | serde of those six camelCase strings round-trips; unknown strings (`ineffective`, `unknown`) fail to deserialize **today** |
| CIR-B03 | Fields listed in §3.3 exist; source has no `effectiveness`, `asset_ids`, `review_cadence`, `next_review`, `evidence_expectations`, `document_refs`, `treatment_ids`, `automation`, `supersedes` |
| CIR-B04 | Builders `new` / `with_status` / `with_risk` / `with_exception` exist; getters are only `id` / `control_id` / `status` / `risk_ids` / `exception_ids` |
| CIR-B05 | `validate_assessment_ir` rejects dangling implementation `control_id`, `risk_ids`, `exception_ids` |
| CIR-B06 | Two implementations of the same control with overlapping `applies_to` **validate Ok** today |
| CIR-B07 | Implementation whose `applies_to` ids are not in `assets`/`identities` **validates Ok** today |
| CIR-B08 | Module comment contains `Not control effectiveness` |
| CIR-B09 | Golden fixture deserializes to `Implemented` |
| CIR-B10 | Control-test / evaluator sources do not read `ControlImplementation.applies_to` (population-runtime §3.5 still true) |

After the target is GREEN, this baseline is **superseded**: `#[ignore]` + `SUPERSEDED` banner (same pattern as `population_runtime.baseline.rs` / catalog suites). Additive-document if a characterization must remain. Target stays GREEN.

---

## 4. Desired behavior (target)

### 4.1 Three-way distinction (law)

| Question | Record | Forbidden shortcut |
| --- | --- | --- |
| What does the control mean? | `Control` | Copying Annex A / pack text onto the implementation |
| How does this org implement it? | `ControlImplementation` | Treating status as a test result |
| Is that implementation effective? | `ControlTestResult.effectiveness` | Adding `effectiveness` to the implementation record |

A row with `status = Implemented` plus a test result `Effectiveness::Ineffective` remains **ineffective**. Registry queries must not promote status to effectiveness.

### 4.2 One control, several implementations

A canonical `ControlId` MAY have several `ControlImplementation` rows on the same `AssessmentDefinition`, distinguished by:

- different subject populations (`applies_to`)
- different systems/assets (`asset_ids`)
- supersession (current vs historical)

Empty `applies_to` **and** empty `asset_ids` means “the whole assessment inventory / IR `AssessmentScope`” for overlap purposes (fail closed). Two coverage-active rows of the same control that both claim the whole scope overlap.

### 4.3 Additive `ImplementationStatus`

Keep the six existing variants and serde names. Add:

| Variant | Serde | Meaning (organizational, not a test) |
| --- | --- | --- |
| `Ineffective` | `ineffective` (alias `disabled` allowed) | Present in the org but switched off / known not operating as a **state record** |
| `Unknown` | `unknown` | State has not been determined |

Do **not** add `Effective`. Do **not** reuse `Implemented` for “tests passed”.

Coverage-active states (count toward overlap / population coverage): `Planned`, `PartiallyImplemented`, `Implemented`, `Ineffective`, `Unknown`.  
Non-covering: `NotImplemented`, `NotApplicable`, `Retired`, and any row that is superseded (`superseded_by` is `Some`).

`PartiallyImplemented` is the organizational “partial rollout” state. It still does not imply `PartiallyEffective`.

### 4.4 Extended `ControlImplementation` fields

Extend the existing struct. Fields stay private with getters. New fields `#[serde(default)]` + skip-if-empty/none so the golden fixture still loads.

| Field | Required conceptually | Type (normative intent) | Notes |
| --- | --- | --- | --- |
| `schema_version` | yes | `String` | still `assurance-ir/v1` |
| `id` | yes | `ControlImplementationId` | stable; unique per assessment |
| `control_id` | yes | `ControlId` | canonical control |
| `owner` | operational | `Option<PrincipalRef>` | keep `PrincipalRef`. Validation does **not** fail-closed on missing owner (golden fixture + first-match lineage stay loadable). |
| `description` | operational | `Option<String>` | how **this org** implements it. Same optional-at-validate rule as `owner`. |
| `status` | yes | `ImplementationStatus` | §4.3 |
| `applies_to` | scoped populations | `Vec<SubjectSelector>` | reuse IR selector; getter **must** exist |
| `asset_ids` | systems/assets | `Vec<AssetId>` | inventory refs; empty = not asset-scoped |
| `implemented_at` | keep | `Option<DateTime<Utc>>` | when marked implemented (existing) |
| `effective_from` | effective date | `Option<DateTime<Utc>>` | operational start; may equal `implemented_at` |
| `review_cadence` | review cadence | `Option<ReviewCadence>` | see §4.5 |
| `next_review` | next review | `Option<DateTime<Utc>>` | required when cadence is set on coverage-active rows |
| `evidence_expectations` | evidence expectations | `Vec<EvidenceRequirementId>` | **refs**, not conclusions |
| `document_refs` | policy/document | `Vec<DocumentRef>` | opaque until controlled documents |
| `risk_ids` | linked risks | `Vec<RiskId>` | existing |
| `treatment_ids` | linked treatments | `Vec<String>` or newtyped id | see §2; no `TreatmentPlan` type here |
| `exception_ids` | exceptions | `Vec<ExceptionId>` | existing |
| `automation` | classification | `Option<ImplementationAutomation>` | §4.6 |
| `compensating_controls` | keep | `Vec<ControlId>` | validate when non-empty |
| `supersedes` | history | `Option<ControlImplementationId>` | prior snapshot this row replaces |
| `superseded_by` | history | `Option<ControlImplementationId>` | set on the **prior** row; do not delete it |
| `superseded_at` | history | `Option<DateTime<Utc>>` | on the prior row or both; deterministic |

`ReviewCadence` lives in `implementation.rs` as `weeping-angel-assurance-ir::implementation::ReviewCadence` (`interval_days: u32`, `interval_days ≥ 1`). It is **not** crate-root re-exported: `lib.rs` already exports risk-register `ReviewCadence` (`interval_seconds`). JSON: `{ "intervalDays": 90 }`. Do not encode ISO “periodic review” Annex A text.

`DocumentRef`:

```text
DocumentRef { id: String, title?: String, kind?: Policy | Standard | Procedure | Record | Plan | Runbook }
```

`ImplementationAutomation`: `Manual` | `Automated` | `Hybrid` (`manual` / `automated` / `hybrid`). Not a provider collector id. Not `EvidenceCollectionKind` on an envelope.

Builders (additive, chainable, existing names unchanged):

```text
new / with_status / with_risk / with_exception
with_owner / with_description / with_applies_to / with_asset
with_implemented_at / with_effective_from
with_review(cadence, next_review)
with_evidence_expectation / with_document / with_treatment
with_automation / with_compensating_control
superseding(prior_id)  # sets supersedes on self; validator wires superseded_by on the prior row
```

Getters: every stored field listed above MUST be readable (today’s five plus the new ones, including `applies_to`, `owner`, `description`).

### 4.5 Review

Coverage-active implementations SHOULD carry `review_cadence` and `next_review`. Target validation:

- If `review_cadence` is present, `interval_days ≥ 1` and `next_review` is present.
- Missing both cadence and next review on `Implemented` or `PartiallyImplemented` fails closed (review is part of operational implementation).
- `Retired` / `NotApplicable` / `NotImplemented` may omit review.

The registry does **not** auto-expire status when `next_review` is in the past; that is temporal assurance/15. It only stores and validates shape.

### 4.6 Evidence expectations (not conclusions)

`evidence_expectations` lists `EvidenceRequirementId`s this implementation claims to satisfy observationally. The record MUST NOT store:

- `Effectiveness`
- pass/fail
- missing-envelope conclusions
- collector payloads

Integrity:

1. Every id exists on `assessment.evidence_requirements` (dangling expectation fails closed).
2. **Missing evidence expectations:** coverage-active `Implemented` or `PartiallyImplemented` with an empty `evidence_expectations` vec fails closed.
3. **Missing required evidence refs:** if the canonical `Control` lists `evidence_requirements`, each of those ids whose `EvidenceRequirement.criticality` is `Required` (default) MUST appear on the implementation’s `evidence_expectations`. Supporting/optional control requirements may be omitted.
4. Presence of a ref is **not** proof the evidence exists. Tests still produce `missing_evidence` / `InsufficientEvidence`.

### 4.7 Supersession / history

Material change (status, `applies_to`, `asset_ids`, `evidence_expectations`, owner, description, automation, document refs, risk/treatment/exception links, compensating controls) MUST create a **new** implementation id that `supersedes` the prior id.

Rules:

- Prior row remains in `assessment.implementations` and stays queryable by id (`implementation_by_id`).
- Authors write **both** sides: successor `supersedes = prior_id` (`superseding(prior_id)`); prior `superseded_by = new_id` (and optional `superseded_at`). The validator does **not** auto-complete `superseded_by`.
- Cycles fail closed (`A supersedes B supersedes A`).
- Dangling `supersedes` / `superseded_by` fail closed.
- A superseded row is not coverage-active even if `status` is still `Implemented` (history snapshot). Coverage-active requires `superseded_by` is `None` **and** a covering status.
- `implementation_by_id` returns that snapshot; `current_implementations_for` ignores superseded and non-covering rows.

Do not mutate a row in place to forget the previous population or status.

### 4.8 Overlap — no silent double-count

For a given `control_id`, coverage-active implementations MUST NOT double-count the same population **on the same asset set**. Overlap is **two-dimensional**: `validate_assessment_ir` / `overlap_report` emit an error only when **both** the population selectors and the asset sets collide (or are universal). Disjoint systems with empty `applies_to` are allowed (CIR-006). Disjoint selectors with empty `asset_ids` are allowed (CIR-001). Empty `applies_to` is a universal population. Empty `asset_ids` is a universal asset set. Two coverage-active rows that are both universal on both axes fail closed.

**Population overlap** (fail closed):

1. Collect coverage-active rows with that `control_id`.
2. Treat empty `applies_to` as the universal selector for that assessment (all IR scope subjects).
3. Two selectors intersect when a subject that could match both exists under deterministic rules:
   - same `kind`
   - `SelectorScope::All` overlaps any other selector of that kind (including empty-ids All)
   - `AnyOf` overlaps `AnyOf` if `ids` intersect **or** both have empty ids (empty AnyOf = all of kind → overlap)
   - `NoneOf` is an exclusion filter; overlap is computed on the remaining positive set. If the remainder cannot be proven disjoint, **fail closed**
   - tag maps: if both specify tags, overlap requires equal keys with equal values on the shared keys **and** id overlap as above; if tags conflict (`env=prod` vs `env=staging`) they are disjoint
4. On intersection, return an error that names **both implementation ids**, the `control_id`, and an explainable selector summary (kind, ids, tags, scope). No silent union.

**Asset overlap:** intersecting `asset_ids` collide on the asset axis. Empty `asset_ids` is universal on that axis. Combined with §4.8: intersecting assets plus a population collision (including two empty `applies_to`) fail closed; intersecting assets plus **disjoint** selectors do not.

**Split populations (allowed):** `kind=Identity` ids `{alice, bob}` vs `{carol}` — no overlap. Employees vs contractors via disjoint tags — no overlap.

**Partial rollout (allowed):** one `PartiallyImplemented` row on a subset; no second coverage-active row covering the same subset.

**Retired (allowed):** `Retired` (or superseded) rows may describe the same selectors as a current row.

Registry query (IR crate; required for target tests and used by `validate_assessment_ir`):

```text
overlap_report(assessment) -> Vec<ImplementationOverlap>
ImplementationOverlap { control_id, left_id, right_id, reason, selectors_or_assets }
```

IR validation MUST fail closed on the same cases so compile never sees a double-counted assessment. The overlap error names both implementation ids, the `control_id`, the reason, and an explainable selector/asset summary.

### 4.9 Dangling references (fail closed)

`validate_assessment_ir` MUST reject:

| Ref | Against |
| --- | --- |
| `control_id` | `assessment.controls` (existing message ok) |
| `compensating_controls` | `assessment.controls` |
| `risk_ids` | `assessment.risks` (IR-019) |
| `exception_ids` | `assessment.exceptions` (IR-020) |
| `evidence_expectations` | `assessment.evidence_requirements` |
| `asset_ids` | `assessment.assets` |
| `applies_to` ids when `kind` is `Asset` / `Identity` / `Vendor` / `Organization` (org strings may match `scope.organizations` or an `Asset` with `AssetKind::Organization`) | corresponding inventory; unknown id → dangling subject/asset |
| `supersedes` / `superseded_by` | `assessment.implementations` |
| `treatment_ids` | `assessment.risk_treatments` **when that vec is non-empty**; if the collection is empty, do not fail merely for non-empty ids |

Duplicate `ControlImplementation.id` fails closed.

Do not require scope engine `ScopeResolution`. Do not walk GitHub/cloud APIs.

### 4.10 Lineage compatibility

- Type `ControlExplanation.implementation: Option<ControlImplementation>` keeps compiling after additive fields (serde default).
- `explain_control` may continue to first-match by `control_id`.
- Do not change lineage snapshot schema ids.
- Adding implementations must not break `sdd_assessment_lineage_target`.

### 4.11 Queries (`weeping-angel-assurance-ir::registry`)

Shipped (re-exported from crate root):

- `implementations_for(assessment, control_id)` — all snapshots, including retired/superseded
- `current_implementations_for(assessment, control_id)` — coverage-active, not superseded
- `implementation_by_id(assessment, id)` — historical snapshot
- `overlap_report(assessment)` as in §4.8

Keep these pure over IR. Do not call collectors. Do not write evidence conclusions.

---

## 5. Tests (target MUST FAIL on current code for the right reason)

Registered in root `Cargo.toml` (not auto-discovered):

```toml
[[test]]
name = "sdd_control_implementation_registry_baseline"
path = "tests/contracts/control_implementation_registry.baseline.rs"

[[test]]
name = "sdd_control_implementation_registry_target"
path = "tests/contracts/control_implementation_registry.target.rs"
```

Protocol completed: target CIR-001–015 GREEN; baseline characterization `#[ignore]` + SUPERSEDED banner after it failed on the new contract.

| Id | Scenario | Expected |
| --- | --- | --- |
| CIR-001 | Split-population implementations of one control (disjoint selectors) | `validate_assessment_ir` Ok; both rows queryable |
| CIR-002 | Partial rollout (`PartiallyImplemented` on a subset) | Ok; status is partial **organizational** state; no effectiveness field |
| CIR-003 | Retired implementation sharing selectors with a current row | Ok; retired excluded from coverage-active / overlap |
| CIR-004 | Missing evidence expectations on `Implemented` | fail closed |
| CIR-005 | Control required `EvidenceRequirement` omitted from implementation | fail closed (missing required evidence **refs**) |
| CIR-006 | One control, multiple systems (`asset_ids` disjoint) | Ok |
| CIR-007 | Overlapping `SubjectSelector`s (same kind + intersecting ids, or universal vs subset) | fail closed **or** explicit overlap error naming both ids + explainable selectors; **no** silent double-count |
| CIR-008 | `Implemented != Effective` | status `Implemented` + test `Ineffective` remains `Ineffective`; JSON/schema of `ControlImplementation` has no `effectiveness` key |
| CIR-009 | Supersession | replacing a row keeps prior id queryable; current query returns the successor |
| CIR-010 | Dangling subject / asset / risk / control | fail closed |
| CIR-011 | New states | `ineffective` / `unknown` deserialize; old six strings unchanged |
| CIR-012 | Golden fixture + IR-008/009 still hold | `control-implementation.json` loads; types remain distinct from `Control` / `Effectiveness` |
| CIR-013 | Additive serde | fixture without new fields still deserializes |
| CIR-014 | No competing type | still `ControlImplementation` in `implementation.rs`; no second registry struct as the SSOT |
| CIR-015 | Neighbor targets | `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, `sdd_assessment_lineage_target` stay GREEN (documented as verify, may be invoked from this suite or CI) |

Target tests that compile **only if** new getters/variants exist will fail to compile on current main — that is an acceptable RED **if** the missing item is the specified field/variant. Prefer tests that `from_value` / grep source / call `validate_assessment_ir` so RED is an assertion, not an unrelated type error.

Each target test title in the suite should be `CIR-00N: <exact subject>` matching the table.

---

## 6. Dual-suite / SDD protocol (abort rather than skip)

1. **Spec** (this file) — no product feature code.
2. **Baseline GREEN** on current main (characterization §3.9).
3. **Target RED** on current main for the right reasons (§5).
4. **Implement** by extending `ControlImplementation` + `ImplementationStatus` + `validate_assessment_ir` (+ optional assurance registry queries).
5. **Docs + ADR** — ADR is **Accepted**; keep this spec as SSOT.
6. **Iterate until Target GREEN**.
7. **Prove Baseline FAILS** or additive-document, then **supersede** baseline (`#[ignore]` + `SUPERSEDED` banner like population/applicability suites).
8. **Target still GREEN**.

Traces only under `.sdd/runs/` and `.sdd/artifacts/`. `docs/sdd/` stays a pointer README (ADR 0004).

---

## 7. Non-goals / out of scope

- ISO Annex A fields, pack `applicability.toml`, or framework-specific columns on `ControlImplementation`
- Provider APIs, collector adapters, GitHub mapping
- Evidence **conclusions** (pass/fail, missing envelopes, freshness verdicts) on the implementation record
- Residual-risk projection (residual risk)
- Full scope engine / `ScopeResolution` (scope engine)
- Risk treatment workflow types (risk treatment) beyond id links
- Controlled-document registry (controlled documents) beyond `DocumentRef`
- SoA generation (operational SoA)
- Changing `explain_control` first-match semantics (keep compiling; optional later pin-by-implementation-id)
- Forking `assurance-ir/v1`
- Dashboards / UI
- Putting effectiveness on `ControlImplementation`

---

## 8. Crate homes and files (implement phase)

| Path | Role |
| --- | --- |
| `crates/weeping-angel-assurance-ir/src/implementation.rs` | Type extension (`ControlImplementation`, `ImplementationStatus`, `implementation::ReviewCadence`, `DocumentRef`, `ImplementationAutomation`) |
| `crates/weeping-angel-assurance-ir/src/registry.rs` | Pure queries + overlap detection |
| `crates/weeping-angel-assurance-ir/src/validation.rs` | Integrity (`validate_assessment_ir`) |
| `crates/weeping-angel-assurance-ir/src/lib.rs` | Re-export types and query fns. Crate-root `ReviewCadence` remains the **risk** type. |
| `tests/contracts/control_implementation_registry.baseline.rs` | Characterization (skip-superseded) |
| `tests/contracts/control_implementation_registry.target.rs` | Normative (CIR-001–015) |
| `Cargo.toml` | `[[test]]` registration |
| `tests/contracts/documentation_layout.rs` | `CANONICAL_SPECS` includes this file |
| `docs/adr/0003-control-implementation-registry.md` | Decision |

Do not add a crate. Do not edit collision-fenced paths in §0.

---

## 9. Acceptance criteria

- Canonical control, organizational implementation, and test effectiveness are three different records; `Implemented` never implies `Effective`.
- `ControlImplementation` remains the only implementation type; schema stays `assurance-ir/v1`.
- Required control-implementation registry surfaces exist: stable id, control id, owner, description, state, scoped subjects, systems/assets, effective date, review cadence, next review, evidence expectation refs, document refs, risks, treatments, exceptions, automation, supersession.
- Status set includes planned, partially implemented, implemented, ineffective/disabled, retired, unknown, without reinterpreting existing serde values.
- One control may have several implementations over disjoint populations or assets.
- `validate_assessment_ir` fails closed on dangling control/subject/asset/risk/exception/evidence-expectation refs and on overlapping coverage-active selectors.
- Material replacement keeps the prior snapshot queryable by id.
- Golden fixture and `sdd_compliance_ir_target` IR-008/009/019/020 remain valid.
- Lineage `ControlExplanation.implementation` still compiles.
- Dual-suite registered; after landing, target GREEN and baseline superseded.
- Collision fence honored.

---

## 10. Risks

- Confusing `ImplementationStatus::Ineffective` with `Effectiveness::Ineffective` (mitigate: different types, IR-009, CIR-008, no effectiveness field).
- Silent serde aliasing if `disabled` were ever an old value (it is not; alias is new-only).
- Over-strict overlap on `NoneOf` / tags causing false fails (fail closed is required; error text must show selectors).
- Empty `applies_to` meaning “universal” may surprise authors of today’s fixtures that leave it empty — existing fixtures with a **single** implementation per control remain valid; two empty-scope rows of the same control become an error (correct).
- risk treatment/12 absence: treatment and document refs must stay opaque so this slice does not invent competing registries.
- Lineage first-match hides extra implementations until explain is extended (documented; do not break pin).
- scope engine not landed: dangling-id checks against inventories are weaker than full scope resolution.

---

## 11. Definition of done

The assurance engine can distinguish **what the control means**, **how this organization implements it**, and **whether that implementation is actually effective**, with effectiveness coming **only** from control tests.
