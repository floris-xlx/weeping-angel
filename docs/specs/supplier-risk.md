# SDD: Operational Supplier Risk Lifecycle (ISMS v1)

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_supplier_risk_target` GREEN (SR-001–SR-015); baseline found-case tests fail as expected |
| Program | Operational ISMS v1 — Prompt 18 supplier risk |
| Slice | Expand IR `Vendor` from `{ id, name }` into an operational supplier-security lifecycle linked to services, assets, obligations, evidence, exceptions, and organizational risks; keep `assurance-ir/v1` and `Vendor::new` |
| Dual-suite | `sdd_supplier_risk_baseline` (found-case; fails on this HEAD) · `sdd_supplier_risk_target` GREEN (`tests/contracts/supplier_risk.{baseline,target}.rs`) — **not auto-discovered**; listed as `[[test]]` in root [`Cargo.toml`](../../Cargo.toml) |
| ADR | Accepted [`docs/adr/0007-supplier-risk.md`](../adr/0007-supplier-risk.md). Numeric **0007** because `0005-*` is methodology/register/scheduler and `0006-*` is treatment. Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (Supplier risk section; do not fork the spine) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Documentation architecture | [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md) |
| Catalog family (consume, do not rewrite) | [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) — `control.vendor.*` / `test.vendor.critical-risk-review-current` / `evidence.vendor.risk-review` already exist |
| Consumes (landed neighbors) | risk register [`risk-register.md`](risk-register.md) (`Risk.vendor_ids`); risk treatment [`risk-treatment.md`](risk-treatment.md) (do not mint `RiskAcceptance`); controlled documents [`controlled-documents.md`](controlled-documents.md) (opaque contract refs, no DMS); interested parties / obligations (Prompt 03 — opaque `obligationIds` until an assessment inventory exists) |
| Neighbors (do not implement here) | personnel security lifecycle; ISMS events/drift (`VendorRiskChanged`); continuous-assurance scheduler; questionnaire SaaS |
| Collision fence | Catalog TOML, ISO packs, GitHub collector, existing `sdd_*` suites except additive `Cargo.toml` / `documentation_layout.rs` registration |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonical digest | `serde_json` struct field order + `BTreeMap` / `BTreeSet` (`canon/v1`) |
| Workspace verify | `cargo test --test sdd_supplier_risk_target`; keep `sdd_compliance_ir_target`, `sdd_applicability_engine_target`, and `sdd_governance_catalog_target` GREEN; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable human SSOT for Prompt 18. It owns the **canonical operational `Vendor` record**, **supplier lifecycle transitions**, **risk-tiered review cadence**, **evidence ≠ acceptance**, **access / offboarding integrity**, **contract security-requirement presence**, **exception honesty**, and **supplier ↔ `Risk` linkage**. It does **not** own catalog TOML IDs, obligation/document/treatment engines (Prompts 03/08/12), the operational `Risk` record (Prompt 06 — consume), or an ISMS event bus (Prompt 15 — seam only).

**Shipped surface:** `weeping-angel-assurance-ir::vendor` plus `critical_suppliers` / `validate_supplier_reviews_at` in `validation.rs`. Typed ids: `SupplierReviewId`, `SupplierRequirementId`, `SupplierIssueId`. Contract/obligation/evidence refs on `Vendor` are opaque strings.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

A supplier is a **management-system dependency** over that graph. A questionnaire envelope is **not** risk acceptance. A catalog population test is **not** a procurement workflow. Collectors must not emit `Approved` / `Accepted` as compliance conclusions.

Generated SDD traces belong in [`.sdd/runs/`](../../.sdd/) and [`.sdd/artifacts/`](../../.sdd/) (ADR 0004). `docs/sdd/` is a stub only.

---

## 0. Collision fence (concurrent SDD)

Prompts 06 / 08 / 12 have landed as neighbor types. Prompt 03 obligation engine is still not an `AssessmentDefinition` inventory — store opaque refs. Do **not** fork a procurement schema, a second `Vendor` type, or a new crate.

| Do not touch | Owner |
| --- | --- |
| `catalog/canonical/v1/**` domain TOML (`control.vendor.*`, `test.vendor.*`, `evidence.vendor.*`) | Governance catalog — **already landed**; this slice must not invent a second supplier family |
| `fixtures/assurance/canonical/v1/governance/vendor-*` | Governance catalog fixtures (`current-documents`, `vendor-review-gaps`, approved/expired exception) |
| `tests/contracts/governance_catalog.*` rewrite | Stay GREEN |
| `tests/contracts/{compliance_ir,applicability_engine}.*` rewrite | Stay GREEN; `HasVendor` remains presence-only |
| Prompt 03 `InterestedParty` / `Obligation` modules | Consume opaque obligation strings; do not re-implement the obligation engine |
| Prompt 06 operational `Risk` fields / status table | Consume landed `Risk` / `RiskId` / `vendor_ids`; store `Vendor.risk_ids` |
| Prompt 08 `TreatmentPlan` / `RiskAcceptance` | Store treatment/acceptance **refs** only |
| Prompt 12 `ControlledDocument` | Store opaque `DocumentRef`s; no DMS |
| Prompt 15 event bus | Record vendor-local history + a named seam (`SupplierAssessmentExpired` / `VendorRiskChanged`); do not build transport |
| `src/finding.rs`, collectors, ISO pack `to =` remaps | Out of scope |

Tiny allowed adjustments at implement: expand [`crates/weeping-angel-assurance-ir/src/vendor.rs`](../../crates/weeping-angel-assurance-ir/src/vendor.rs); optional sibling modules under the same crate; additive `typed_id!` aliases; `validation.rs` vendor-graph checks; `lib.rs` re-exports; `serde(default)` / `skip_serializing_if` on additive fields; reuse `PrincipalRef`, `Exception`, `Risk`, `Asset`, `EvidenceRequirementId`.

Do **not** redesign `AssessmentDefinition` core inventories. `AssessmentDefinition.vendors` remains `Vec<Vendor>`. Do **not** bump `ASSURANCE_IR_SCHEMA`.

---

## 1. Problem / user-visible goal

Weeping Angel stores vendors as a **two-field inventory stub** so applicability can answer `HasVendor` and processing activities can cite `processors: Vec<VendorId>`. The module is documented *“Minimal vendor node for the compliance graph.”* Operators cannot classify a supplier, name criticality, bind supplied services or data/system access, assign an owner, run an onboarding review, attach security requirements or contract evidence, record approval, schedule reassessment, monitor issues, terminate with access revocation, or link the dependency to an organizational `Risk`.

The governance catalog already evaluates `test.vendor.critical-risk-review-current` against **fixture envelopes**. That is a catalog population predicate over `evidence.vendor.risk-review`. It is not an IR lifecycle: a vendor named in `AssessmentDefinition.vendors` has no criticality, no review clock, and no approval record. Evidence presence in a fixture cannot be confused with risk acceptance — and today the IR cannot even *state* acceptance separately from a name string.

Today that means:

- `Vendor::new(id, name)` JSON is exactly `{ "id", "name" }`;
- `AssessmentDefinition.vendors` is an unvalidated bag; duplicate ids are not an error; dangling `ProcessingActivity.processors` are not an error;
- `validate_assessment_ir` never walks vendor fields;
- `HasVendor(true)` is true iff the in-scope vendor vec is non-empty (or an explicit fact); criticality and lifecycle are invisible;
- `SubjectKind::Vendor` matching is id-only (`vendor_matches`); tags do not select critical suppliers;
- there is no onboarding / approval / termination state machine, no review cadence, no access grants, no contract security-requirement row, no supplier↔risk edge on `Vendor`;
- Prompt 06 `Risk.vendor_ids` was Specified, not landed — `Risk` was still four fields (now landed; this slice stores `Vendor.risk_ids` and fail-closes dangling `RiskId`s).

**User-visible goal:** critical suppliers are continuously represented as dependencies with accountable risk, evidence, review cadence, and control impact.

```text
Vendor { classification, criticality, services, access, owner }
  → Candidate → under review → approved → active
  → restricted/suspended → terminating → terminated
  → onboarding review + security requirements + risk assessment + approval
  → contract/document evidence (presence ≠ acceptance)
  → risk-tiered reassessment cadence + monitoring + issues
  → linked organizational RiskIds
  → expired assessment ⇒ gap / event seam
  → termination with lingering access ⇒ fail closed
```

Population tests the IR must support (catalog tests already exist; IR must make the same honesty true of `AssessmentDefinition.vendors`):

```text
all critical suppliers have current security review
low-risk suppliers do not inherit critical-tier requirements
privileged access elevates requirements regardless of name-only inventory
terminated + still-active grant ⇒ lingering-access gap
missing contract security requirement on a tier that requires it ⇒ gap
expired exception bound to a vendor must not suppress the review gap
supplier-related Risk linkage is N:N and fail-closed when ids dangle
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Vendor` | `weeping-angel-assurance-ir::vendor` | **SSOT record.** Expand in place. Keep `Vendor::new(id, name)` and public `id` / `name`. Keep `#[serde(rename_all = "camelCase")]`. |
| `VendorId` | `id.rs` `typed_id!(VendorId)` | Unchanged. Landed: `SupplierReviewId`, `SupplierRequirementId`, `SupplierIssueId`. Contract/obligation refs stay opaque strings. |
| `AssessmentDefinition.vendors` | `assessment.rs` | Stay `Vec<Vendor>`. No parallel register crate. |
| `ValidateIr` | `validation.rs` | Today **ignores** vendors. **Add** vendor-graph checks. Do not remove IR-019/020. |
| `ProcessingActivity.processors` | `privacy.rs` | Keep `Vec<VendorId>`. **Add** dangling-processor validation. |
| `HasVendor` | IR predicate + `applicability::evaluator::infer_vendors` | **Presence-only remains law** for Kleene applicability. Do not change `HasVendor` into a criticality test. |
| `vendor_matches` | `applicability/context.rs` | Id + `SubjectKind::Vendor`. Do not require applicability to understand new fields. Optional later: tag `critical` — **out of this slice** unless it is a pure additive matcher that keeps current tests green. |
| `PrincipalRef` | `implementation.rs` | **Reuse** for `owner` / reviewer / approver. Do not invent `VendorOwner`. |
| `Asset` / `AssetKind::Service` | `asset.rs` | Supplied services **are assets**, not a new type. |
| `Identity` | `identity.rs` | Vendor-operated identities / service accounts cited on access grants. |
| `Risk` / `RiskId` | `risk.rs` | Linkage. Prompt 06 landed `Risk.vendor_ids`. This slice always stores `Vendor.risk_ids` and fail-closes dangling `RiskId`s. Reverse listing is not required. |
| `Exception` | `exception.rs` | Reuse. Expired `expires_at` must not suppress supplier review gaps. |
| `EvidenceRequirementId` | `evidence.rs` | Evidence *expectation* ids on reviews/requirements. Envelope pass/fail stays in control tests. |
| Governance catalog | `catalog/canonical/v1/{controls,tests,evidence}/governance.toml` | **Do not rewrite.** IR lifecycle is the organizational record those tests can later read; this slice does not retarget TOML. |
| Golden assessment | `tests/fixtures/assurance-ir/v1/assessment.json` | `"vendors": []` must keep decoding. |
| `sdd_compliance_ir_target` | `Vendor::new` compile + golden round-trip | Must stay GREEN. |
| `sdd_applicability_engine_target` | `HasVendor` presence | Must stay GREEN. |
| `sdd_governance_catalog_target` | `test.vendor.critical-risk-review-current` | Must stay GREEN. |

Serde compatibility law:

- Existing JSON `{ "id", "name" }` deserializes.
- `Vendor::new` JSON stays two keys (`id`, `name`) via `skip_serializing_if` on every additive field (including default lifecycle).
- Do **not** deny unknown fields (other slices add keys).
- Additive enums use **new** camelCase names; never remap `name` into `criticality`.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 IR `Vendor` is two public fields

[`crates/weeping-angel-assurance-ir/src/vendor.rs`](../../crates/weeping-angel-assurance-ir/src/vendor.rs):

```text
//! Minimal vendor node for the compliance graph.

Vendor {
  id: VendorId,
  name: String,
}

Vendor::new(id, name) → exactly those fields
serde rename_all = "camelCase"
```

No classification, criticality, services, access, owner, lifecycle, review, requirements, approval, cadence, monitoring, issues, termination, history, or risk ids.

### 3.2 Assessment inventory is an unvalidated vec

[`assessment.rs`](../../crates/weeping-angel-assurance-ir/src/assessment.rs): `vendors: Vec<Vendor>` default empty. Golden `tests/fixtures/assurance-ir/v1/assessment.json` has `"vendors": []`.

### 3.3 Validation never walks vendors

[`validation.rs`](../../crates/weeping-angel-assurance-ir/src/validation.rs):

- Builds id sets for requirements, controls, evidence requirements, risks, exceptions.
- **Does not** collect `vendor` ids.
- **Does not** reject duplicate `VendorId`.
- **Does not** walk `ProcessingActivity.processors`.
- No vendor-graph, review, access, or exception-honesty checks.

### 3.4 Processing activities hold processor ids with no integrity

[`privacy.rs`](../../crates/weeping-angel-assurance-ir/src/privacy.rs): `processors: Vec<VendorId>` default empty, skipped when empty. A processor id that is absent from `assessment.vendors` still `validate()`s.

### 3.5 Applicability `HasVendor` is presence-only

[`evaluator.rs`](../../crates/weeping-angel-assurance/src/applicability/evaluator.rs) `infer_vendors`:

- non-empty `context.vendors` → `True`
- empty + authoritative `InventoryFamily::Vendors` → `False`
- else `Unknown`

`vendor_matches` requires `SubjectKind::Vendor`, ignores tags, matches id. Criticality cannot be selected.

### 3.6 Catalog already has a supplier *family*, not an IR lifecycle

Governance catalog (GREEN) declares:

| ID | Role |
| --- | --- |
| `control.vendor.inventory` | Authoritative inventory |
| `control.vendor.risk-review` | Current risk review for critical suppliers |
| `control.vendor.security-requirements` | Attested requirements (manual) |
| `control.vendor.reassessment` | Cadence uses the same current-review predicate |
| `control.vendor.cloud-governance` | Hosted-service attestation (manual) |
| `evidence.vendor.inventory` / `evidence.vendor.risk-review` | Facts |
| `test.vendor.critical-risk-review-current` | `all-subjects` + `field = current` over `kind = vendor` |

Fixtures `vendor-review-gaps` / `expired-exception` prove catalog honesty. They do **not** populate `Vendor` lifecycle fields. This slice must **not** retarget those IDs or add a second `control.supplier.*` family.

### 3.7 Risks and documents did not yet form a supplier graph

At characterization:

- `Risk` was `{ id, title, description, status }` (*“Not a risk engine.”*). Operational register fields including `vendor_ids` have since landed (Prompt 06).
- Prompt 03 obligation engine was not an assessment inventory (still opaque refs here).
- `Exception` had `expires_at` + `subjects` but validation did not apply expiry to vendor reviews.

### 3.8 Callers construct with `Vendor::new`

| Caller | Use |
| --- | --- |
| `sdd_compliance_ir_target` | `Vendor::new(VendorId::new("vendor:acme"), "Acme")` |
| `sdd_applicability_engine_{baseline,target}` | Payroll / one-vendor presence for `HasVendor` |
| Golden assessment | empty vec |

---

## 4. Desired behavior (target)

### 4.1 Product home

Record lives in `weeping-angel-assurance-ir`. Landed layout:

```text
weeping-angel-assurance-ir
  vendor.rs          # Vendor, lifecycle, criticality, classification, helpers
  id.rs              # VendorId; SupplierReviewId; SupplierRequirementId; SupplierIssueId
  validation.rs      # vendor graph, duplicate VendorId, lingering access, missing contract requirement, validate_supplier_reviews_at
  lib.rs             # crate-root re-exports
  implementation.rs  # PrincipalRef (consumed)
  exception.rs       # consumed for expired-exception honesty
  risk.rs            # consumed for RiskId / vendor_ids
```

Network-free. No ISO annex numbers, no vendor-management product names (Vanta, Whistic, SecurityScorecard, Coupa) in generic IR. No questionnaire runtime.

### 4.2 Operational record (additive fields)

JSON names are **camelCase**. Keep `id`, `name`. New fields **default on deserialize** and **omit when empty** on serialize so `Vendor::new` remains `{ id, name }`.

| Field (Rust) | JSON | Required on `new` | Semantics |
| --- | --- | --- | --- |
| `id` | `id` | yes | Stable `VendorId` |
| `name` | `name` | yes | Display name |
| `classification` | `classification` | no | §4.3 |
| `criticality` | `criticality` | no | §4.4 risk tier |
| `supplied_service_ids` | `suppliedServiceIds` | no | `AssetId`s (typically `AssetKind::Service`) |
| `processing_activity_ids` | `processingActivityIds` | no | Affected processes (RoPA rows) |
| `access` | `access` | no | §4.6 grants / privileged flag |
| `owner` | `owner` | no | `PrincipalRef` |
| `status` | `status` | default `unspecified` | §4.5 lifecycle; omitted on `new` |
| `onboarding_review` | `onboardingReview` | no | `SupplierReview` (§4.7) |
| `reviews` | `reviews` | no | Periodic / ad-hoc reviews (history of assessments) |
| `security_requirements` | `securityRequirements` | no | §4.8 |
| `risk_assessment` | `riskAssessment` | no | Latest supplier-side assessment snapshot (§4.7) — **not** org `Risk` acceptance |
| `approval` | `approval` | no | §4.9 explicit approval; not implied by evidence |
| `contract_document_refs` | `contractDocumentRefs` | no | Opaque non-empty strings (Prompt 12 registry is not resolved here) |
| `obligation_ids` | `obligationIds` | no | Opaque strings; fail closed **only when** an obligation inventory exists (none on `AssessmentDefinition`) |
| `reassessment_cadence` | `reassessmentCadence` | no | `{ "intervalSeconds": u64 }` or the shared duration type |
| `next_review` | `nextReview` | no | `DateTime<Utc>` |
| `monitoring_status` | `monitoringStatus` | no | §4.10 |
| `issues` | `issues` | no | §4.10 |
| `exception_ids` | `exceptionIds` | no | `ExceptionId`s bound to this supplier |
| `risk_ids` | `riskIds` | no | Organizational `RiskId`s (§4.12) |
| `control_ids` | `controlIds` | no | Canonical controls impacted (typically `control.vendor.*`) |
| `evidence_refs` | `evidenceRefs` | no | Evidence-requirement ids and/or envelope digests |
| `version` | `version` | default `1` | Monotonic; skip-serialize when `1` |
| `history` | `history` | no | Append-only `VendorEvent` (§4.11) |

`Vendor::new` sets empty/None/`Unspecified` for every additive field and **must not** start emitting `status`, `criticality`, `owner`, `nextReview`, or `riskIds`.

Public `id` / `name` remain public. Lifecycle and review mutations that are semantically transitions **must** go through `transition` / `record_review` / `approve` / `revise` (names flexible) so `history` is appended. Assessment validation rejects a `history` that contains an illegal transition even if `status` was mutated.

### 4.3 Classification

```text
SupplierClassification =
  Unspecified          // default; omitted on new
  | Supplier           // generic third party
  | Processor          // processes personal data (RoPA processor)
  | HostedService      // SaaS / cloud app
  | CloudProvider
  | ProfessionalServices
  | Other
```

JSON camelCase (`hostedService`, `cloudProvider`, `professionalServices`). Classification does **not** set criticality and does **not** imply `HasVendor` changes.

### 4.4 Risk tiers (not one-size-fits-all)

```text
SupplierCriticality = Unspecified | Low | Medium | High | Critical
```

Default `Unspecified` (omitted on `new`). Unspecified is **not** Low: a nameless inventory row that an operator later marks critical must not have been silently treated as reduced-requirements.

**Tier policy (defaults; override per record via `reassessmentCadence`):**

| Tier | Current security review | Contract security requirement | Onboarding review | Privileged-access override |
| --- | --- | --- | --- | --- |
| `Critical` | required; overdue/missing is a gap | required when `status` ∈ {Approved, Active, Restricted, Suspended, Terminating} | required before `Approved` | still required; cadence may only shorten |
| `High` | required | required in the same statuses | required before `Approved` | same |
| `Medium` | required unless an unexpired bound exception applies | required if processor **or** privileged | required before `Approved` | elevates to High rules |
| `Low` | **reduced**: inventory + owner sufficient; no current-review gap from missing onboarding assessment | not required unless privileged or `Processor` | not required | privileged ⇒ High rules |
| `Unspecified` | treated as **not current** for population tests that select critical/high; does not count as Low | fail closed for population “all critical …” (cannot be in that population until classified) | n/a | privileged ⇒ cannot remain Unspecified for access grants (`validate` error) |

Do **not** hardcode ISO 27001:2022 Annex A.5.19–5.22 clause satisfaction from a tier label.

Population helper (mandatory):

```text
fn critical_suppliers<'a>(assessment: &'a AssessmentDefinition) -> impl Iterator<Item = &'a Vendor>
fn review_current(&self, as_of: DateTime<Utc>) -> bool
fn validate_supplier_reviews_at(assessment, as_of) -> Result<(), IrValidationError>
```

`review_current` is true iff a recorded review (onboarding or later) has `valid_until` / implied `nextReview` **≥ as_of** (or the review record itself carries `validUntil >= as_of`). Unscheduled (`nextReview` None and no review `validUntil`) is **not** current. Low-tier without privileged access is **exempt** from this predicate (reduced requirements) but still appears in inventory tests.

Catalog `test.vendor.critical-risk-review-current` stays the catalog-level `all-subjects` predicate. This slice must make the **same honesty** available as an IR query so `all critical suppliers have current security review` can be evaluated against `AssessmentDefinition.vendors`, not only governance fixtures.

### 4.5 Lifecycle and transitions

Prompt law:

```text
Candidate → under review → approved → active → restricted/suspended → terminating → terminated
```

IR encoding (`#[serde(rename_all = "camelCase")]`):

```text
SupplierLifecycleStatus =
  Unspecified          // default for old JSON / Vendor::new; omitted when Unspecified
  | Candidate
  | UnderReview        // "underReview"
  | Approved
  | Active
  | Restricted
  | Suspended
  | Terminating
  | Terminated
```

`Restricted` and `Suspended` occupy the same lifecycle slot (prompt `restricted/suspended`) but are distinct JSON values.

Fail-closed table (from → allowed to). Any other pair is an error. `Unspecified` may be adopted into the machine without pretending the vendor was always a Candidate.

| From | Allowed targets |
| --- | --- |
| `Unspecified` | `Candidate`, `UnderReview` |
| `Candidate` | `UnderReview`, `Terminated` |
| `UnderReview` | `Candidate`, `Approved`, `Terminated` |
| `Approved` | `Active`, `UnderReview`, `Terminated` |
| `Active` | `Restricted`, `Suspended`, `Terminating`, `UnderReview` |
| `Restricted` | `Active`, `Suspended`, `Terminating` |
| `Suspended` | `Active`, `Restricted`, `Terminating` |
| `Terminating` | `Terminated`, `Active` (reinstatement requires history event) |
| `Terminated` | ∅ terminal |

`fn SupplierLifecycleStatus::can_transition(from, to) -> bool` and `Vendor::transition(to) -> Result<…>` are mandatory. Invalid transitions return a deterministic error (no panic in library paths). Recorded `history` status steps must satisfy the same table.

`Approved` **requires** an `approval` record (principal + time + decision `Approved`). Evidence of a questionnaire or `evidence.vendor.risk-review` envelope **must not** move status to `Approved` or `Active` by itself. A helper that only attaches evidence must not call `transition(Approved)`.

Clockless `AssessmentDefinition::validate()` does **not** auto-transition expired reviews. It may still fail closed on graph errors and lingering access.

### 4.6 Access, privileged vendors, termination

```text
SupplierAccess {
  privileged: bool,                    // default false; omit if false
  data_access: bool,                   // default false
  grants: Vec<SupplierAccessGrant>,    // omit if empty
}

SupplierAccessGrant {
  asset_id: Option<AssetId>,
  identity_id: Option<IdentityId>,
  privileged: bool,
  status: Active | Revoked,            // camelCase
  revoked_at: Option<DateTime<Utc>>,
}
```

At least one of `asset_id` / `identity_id` must be set on a grant. `Vendor.access.privileged == true` **or** any grant with `privileged` counts as **privileged access** and applies the §4.4 override.

**Lingering access (mandatory fail-closed):**

```text
fn Vendor::has_lingering_access(&self) -> bool
```

True when `status ∈ {Terminating, Terminated}` **and** any grant has `status = Active` (or `privileged` remains true with no grants but `data_access` / `privileged` still set). `validate()` **fails** when `has_lingering_access()` is true. A terminated vendor with all grants `Revoked` and `privileged = false` and `data_access = false` is valid.

Do not invent an IAM deprovisioning connector. The grant list is the organizational statement of remaining access.

### 4.7 Reviews, assessments, evidence ≠ acceptance

```text
SupplierReview {
  id: SupplierReviewId,                // or stable string
  kind: Onboarding | Periodic | AdHoc | Offboarding,
  performed_at: DateTime<Utc>,
  valid_until: Option<DateTime<Utc>>,
  reviewer: Option<PrincipalRef>,
  evidence_refs: Vec<…>,               // questionnaire, manual review, or automated posture
  outcome: Option<String>,             // narrative; not RiskStatus
  source: Questionnaire | ManualReview | AutomatedPosture | Other,
}

SupplierRiskAssessment {
  performed_at: DateTime<Utc>,
  methodology_ref: Option<String>,     // Prompt 05 version pin if present; opaque otherwise
  residual_placeholder: omitted,       // do not compute residual here
  evidence_refs: Vec<…>,
  linked_risk_ids: Vec<RiskId>,        // may duplicate Vendor.risk_ids
}
```

Invariants:

1. Attaching `evidence_refs` (including `evidence.vendor.risk-review`) does **not** set `SupplierLifecycleStatus::Approved` or `Active`.
2. Completing a questionnaire source does **not** set `Risk.status = Accepted` and does **not** create a Prompt 08 `RiskAcceptance`.
3. `review_current(as_of)` uses `valid_until` / `nextReview`, not “has any evidence_ref”.
4. Expired review (`valid_until < as_of` or `nextReview < as_of`) ⇒ **gap**. Clocked `validate_supplier_reviews_at` fails for Critical/High (and elevated Medium/Low) that are in `{Approved, Active, Restricted, Suspended, Terminating}` without a current review. `Terminated` is not required to have a current periodic review.
5. Expired assessments **must** be representable as a gap **and** as a history event `AssessmentExpired` (vendor-local). Prompt 15 may later project `VendorRiskChanged` / `SupplierAssessmentExpired` from that fact; this slice does not implement the event bus.

### 4.8 Security requirements and missing contract requirement

```text
SupplierSecurityRequirement {
  id: SupplierRequirementId,
  title: String,
  source: Contract | Policy | Obligation | Internal,
  obligation_id: Option<String>,       // opaque; Prompt 03 inventory not on AssessmentDefinition
  document_ref: Option<String>,        // opaque contract/document ref
  control_ids: Vec<ControlId>,
  evidence_refs: Vec<…>,
  required: bool,                      // default true
}
```

Do **not** store `met` / `effective` / `accepted` on the requirement. Satisfaction is a control-test conclusion.

**Missing contract security requirement (mandatory gap):**

An in-lifecycle vendor whose §4.4 row says contract security requirement is required, with `status ∈ {Approved, Active, Restricted, Suspended, Terminating}`, fails `validate()` (or a dedicated query used by the target suite — prefer `validate()` so it is fail-closed) when **no** `security_requirements` item has `source = Contract` **or** (if `source = Contract`) both `document_ref` and `obligation_id` are missing **and** `contract_document_refs` is empty.

Low-tier without privileged access and without `Processor` classification is exempt.

### 4.9 Approval

```text
SupplierApproval {
  principal: PrincipalRef,
  at: DateTime<Utc>,
  decision: Approved | Rejected | Conditional,
  rationale: Option<String>,
}
```

`transition(Approved)` fails unless `approval.decision == Approved`. `Conditional` does not satisfy `Approved`. Evidence-only vendors remain `UnderReview`.

### 4.10 Monitoring and issues

```text
SupplierMonitoringStatus =
  Unspecified | NotMonitored | Healthy | Degraded | Incident

SupplierIssue {
  id: SupplierIssueId,
  title: String,
  status: Open | Closed,
  opened_at: Option<DateTime<Utc>>,
}
```

Issues are organizational notes, not a ticket product. Open issues do not auto-suspend the vendor; `Restricted` / `Suspended` remain explicit transitions.

### 4.11 History

```text
VendorEvent {
  version: u32,
  at: DateTime<Utc>,
  principal: Option<PrincipalRef>,
  kind: Created
      | FieldsRevised
      | StatusTransition { from, to }
      | ReviewRecorded
      | AssessmentExpired { as_of: DateTime<Utc> }
      | ApprovalRecorded
      | AccessRevoked
      | Terminated,
}
```

`revise` increments `version` and appends. `Vendor::new` may have empty `history`. Old two-key JSON decodes as empty history.

### 4.12 Supplier ↔ organizational risk linkage

N:N:

```text
Vendor.risk_ids: Vec<RiskId>
Risk.vendor_ids: Vec<VendorId>     // Prompt 06 landed
```

Rules (as shipped):

1. Every `Vendor.risk_ids` entry must exist in `assessment.risks` (`validate` fail closed).
2. Every `Risk.vendor_ids` entry must exist in `assessment.vendors` (register validation). One-sided linkage is valid; reverse listing is **not** required.
3. Linking a `RiskId` does **not** accept residual risk and does **not** complete treatment (Prompt 08).
4. `Risk::new` remains valid with no vendor keys.
5. A supplier-related risk may exist with empty `finding_refs`; `RiskSource::Supplier` is optional.

### 4.13 Expired exception honesty

Reuse `Exception`. If `Vendor.exception_ids` (or `Exception.subjects` with `SubjectKind::Vendor` + this id) contains an exception that is `Expired` **or** `expires_at < as_of`, that exception **must not** suppress `validate_supplier_reviews_at` / review-current gaps.

Clockless `validate()` still accepts an `ExceptionStatus::Expired` row as a stored fact (IR-020 only requires the id to exist). Clocked supplier review validation treats expired exceptions as absent.

This matches governance catalog `expired-exception` honesty at the IR layer.

### 4.14 Reference integrity (fail closed)

On `AssessmentDefinition::validate()` (in addition to existing IR-019/020):

| Reference | Rule |
| --- | --- |
| Duplicate `Vendor.id` | error (today duplicates are silent) |
| `supplied_service_ids` / grant `asset_id` | ∈ `assessment.assets` |
| `processing_activity_ids` | ∈ `assessment.processing_activities` |
| `ProcessingActivity.processors` | ∈ `assessment.vendors` |
| grant `identity_id` | ∈ `assessment.identities` |
| `owner` / approval principal `Identity(_)` | ∈ `assessment.identities` |
| `risk_ids` | ∈ `assessment.risks` |
| `exception_ids` | ∈ `assessment.exceptions` |
| `control_ids` / requirement `control_ids` | ∈ `assessment.controls` |
| `evidence_refs` | opaque strings (questionnaire / manual / posture / envelope digest); not resolved as `EvidenceRequirementId` |
| `obligation_ids` | opaque strings; AssessmentDefinition has no obligation inventory — do not invent Prompt 03 here |
| `contract_document_refs` | opaque non-empty strings (Prompt 12 `ControlledDocument` is not resolved from this inventory) |
| `history` transitions | each consecutive status pair obeys §4.5 |
| lingering access | §4.6 |
| missing contract security requirement | §4.8 |
| privileged + `criticality = Unspecified` | error |
| `version` | `>= 1`; default 1 |

Clocked (not part of clockless `validate()`):

| Query | Rule |
| --- | --- |
| `validate_supplier_reviews_at(as_of)` | §4.4 / §4.7 overdue/missing current review by tier |
| expired exception | §4.13 does not suppress those gaps |
| expired assessment event | recording `AssessmentExpired` is valid; absence of the event does not hide the gap |

### 4.15 Serialization and constructor

- Schema stays `assurance-ir/v1`.
- `canonical_digest` remains SHA-256 of `serde_json::to_vec` (struct field order + BTree maps).
- Empty vectors / None / Unspecified / false / version `1` skip serialize so `Vendor::new` bytes stay `{id, name}`.
- Keep:

```text
Vendor::new(id, name) -> Vendor
```

Additive builders (names flexible): `with_criticality`, `with_owner`, `with_services`, `with_access`, `with_review`, `with_requirement`, `with_risk`, `with_exception`.

Applicability and compliance tests that only call `new` must keep compiling.

---

## 5. Dual-suite protocol

Follow [`docs/adr/0004-documentation-architecture.md`](../adr/0004-documentation-architecture.md). Directory `tests/contracts/` is **not** Cargo auto-discovery.

| Suite | File | Cargo `[[test]]` name | On this HEAD |
| --- | --- | --- | --- |
| Baseline | `tests/contracts/supplier_risk.baseline.rs` | `sdd_supplier_risk_baseline` | found-case tests **fail** (two-field stub characterization) |
| Target | `tests/contracts/supplier_risk.target.rs` | `sdd_supplier_risk_target` | **GREEN** (SR-001–SR-015) |

Protocol (landed through implement):

1. Spec + ADR 0007 + dual-suite files + `Cargo.toml` `[[test]]`.
2. Baseline found-case encoded §3.
3. Target encoded SR-001–SR-015 (not `#[ignore]`).
4. Product implemented in `vendor.rs` / `validation.rs` / `id.rs` / `lib.rs`.
5. Target GREEN; `sdd_compliance_ir_target`, `sdd_applicability_engine_target`, `sdd_governance_catalog_target` stay GREEN.
6. Baseline found-case tests fail on this HEAD as expected (two-field stub characterization).
7. Target still GREEN.

One regression test per invariant titled from the spec ids below, encoding the **original found case** in baseline.

Register this spec path in `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS`.

---

## 6. Acceptance criteria (testable)

Target suite must encode at least:

- **SR-001** `Vendor::new` still exists; serialized JSON is `{ id, name }` (no `status` / `criticality` / `owner` / `nextReview` / `riskIds` when unset). Golden assessment `"vendors": []` still decodes.
- **SR-002** Fully populated operational vendor round-trips camelCase JSON; additive keys persist; `canonical_digest` is stable for equivalent `BTree` ordering.
- **SR-003** Lifecycle JSON: `candidate`, `underReview`, `approved`, `active`, `restricted`, `suspended`, `terminating`, `terminated`. Illegal transitions fail (`Terminated → Active` without the allowed reinstatement path from `Terminating`, `Candidate → Active`, `Active → Terminated` skipping `Terminating`). Legal transitions append history.
- **SR-004 Critical vendor current review:** an `Active` + `Critical` vendor with `nextReview` / review `validUntil` ≥ `as_of` satisfies `review_current`; `validate_supplier_reviews_at` Ok for that population.
- **SR-005 Stale review:** same vendor with `nextReview` / `validUntil` < `as_of` is **not** current; clocked validation fails; expired assessment is a gap (and representable as `AssessmentExpired` in history). Evidence refs alone do not make it current.
- **SR-006 Low-risk reduced requirements:** `Low` + not privileged + not `Processor` does **not** fail clocked review validation for a missing onboarding assessment / missing contract requirement. `Critical` with the same missing review **does** fail.
- **SR-007 Privileged access:** `access.privileged = true` (or a privileged grant) elevates a `Low` vendor to High/Critical review rules; missing current review fails clocked validation.
- **SR-008 Termination with lingering access:** `Terminated` (or `Terminating`) + any `Active` grant or leftover `privileged`/`dataAccess` fails clockless `validate()`. All grants `Revoked` and flags clear succeeds.
- **SR-009 Missing contract security requirement:** `Active` + `Critical` with no `source = Contract` requirement and empty `contractDocumentRefs` fails `validate()`. Adding a contract requirement with a document or obligation ref succeeds.
- **SR-010 Expired exception:** an `Exception` bound to the vendor with `status = expired` or `expiresAt < as_of` does **not** suppress SR-005. An approved unexpired bound exception may skip the clocked review gap for that vendor only.
- **SR-011 Supplier-related risk linkage:** `Vendor.risk_ids` containing a present `RiskId` round-trips; dangling `RiskId` fails `validate()`. A `Risk` may list the vendor via landed `vendorIds`. Linkage does not set `RiskStatus::Accepted`.
- **SR-012 Evidence ≠ acceptance:** a vendor with `evidenceRefs` / questionnaire review source and **no** `approval` cannot `transition(Approved)`. Deserialize+validate of evidence-only payload does not yield `status = approved`.
- **SR-013** Duplicate `VendorId` fails `validate()`. Dangling `ProcessingActivity.processors` fails. Dangling supplied `AssetId` fails. IR-019/020 still work.
- **SR-014** `HasVendor` remains presence-only (applicability target stays GREEN). This slice does not retarget `control.vendor.*` TOML.
- **SR-015** Dual-suite names `sdd_supplier_risk_baseline` / `sdd_supplier_risk_target` are listed in root `Cargo.toml`.

Baseline suite must encode the found case in §3 (two fields, no lifecycle, no validation walk, presence-only HasVendor, catalog family already exists but IR is thin).

---

## 7. Out of scope

- Questionnaire SaaS, scoring portals, or email campaigns.
- Procurement / source-to-pay suite, POs, invoices, vendor master data beyond security lifecycle.
- Contract authoring, e-signature, or a document management system (Prompt 12 owns controlled documents).
- External trust-center scraping (SecurityScorecard, CSA STAR, vendor public pages).
- Rewriting catalog TOML or adding a second `control.supplier.*` family.
- Changing Kleene `HasVendor` from presence to criticality.
- Implementing Prompt 03/06/08/12/15 engines; only typed refs + seams.
- New crate or `assurance-ir/v2`.
- UI, persistence service, ticketing, Slack.
- Claiming ISO 27001 supplier-clause certification from a `Vendor` row.
- Auto-advancing `nextReview` from cadence (scheduler is Prompt 13).

---

## 8. Risks

- Expanding public `Vendor` fields can break exhaustive struct literals. Mitigation: keep `Vendor::new`; serde defaults; search/fix literals in this slice only.
- Emitting `status` from `Vendor::new` would change constructor JSON and break SR-001. Mitigation: default `Unspecified` + skip-serialize.
- Treating governance catalog fixtures as the IR lifecycle would fork SSOT. Mitigation: catalog stays; IR vendors become the organizational record.
- Implementing obligations/documents/treatments here would collide with Prompts 03/08/12. Mitigation: opaque typed refs; fail closed only when those inventories exist.
- `HasVendor` changes would RED `sdd_applicability_engine_target`. Mitigation: presence-only remains law.
- Evidence envelopes used as implicit approval would violate “evidence ≠ acceptance.” Mitigation: SR-012; approval is a distinct record.
- Clockless `validate()` failing on stale reviews would break golden empty assessments if defaults were wrong. Mitigation: overdue is clocked; lingering access is clockless because it is a stored grant fact.
- Prompt 06 `Risk.vendor_ids` landed in parallel. Mitigation: consume `vendor_ids`; always store `Vendor.risk_ids`; dangling ids fail closed.
- Hardcoding review intervals (365d) in control logic would fight organization policy. Mitigation: cadence is per-record; tests supply explicit `nextReview`.

---

## 9. Landed files

- `crates/weeping-angel-assurance-ir/src/vendor.rs`
- `crates/weeping-angel-assurance-ir/src/id.rs` (additive `Supplier*Id`)
- `crates/weeping-angel-assurance-ir/src/validation.rs` (`validate_supplier_reviews_at`, vendor graph)
- `crates/weeping-angel-assurance-ir/src/lib.rs` re-exports
- Dual-suite `tests/contracts/supplier_risk.{baseline,target}.rs`; root `Cargo.toml` `[[test]]`; `CANONICAL_SPECS`

---

## 10. Definition of done

Critical suppliers are continuously represented as IR dependencies with classification, criticality, supplied services, access, owner, onboarding review, security requirements, risk assessment, explicit approval, contract/document evidence, reassessment cadence, monitoring, issues, termination/offboarding, and linked organizational risks. Review requirements are risk-tiered. Evidence presence does not imply acceptance. Expired assessments are gaps. Lingering access after termination fails closed.

Dual-suite SDD protocol: spec first, baseline found-case on characterization, target RED then GREEN (SR-001–SR-015). Baseline found-case tests fail on this HEAD as expected.
