# ADR 0007 — Operational supplier-security lifecycle (expand `Vendor`)

<!-- weeping-angel-adr-meta
id = "0007"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_supplier_risk_target` GREEN (SR-001–SR-015); baseline found-case tests fail as expected |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The operational reading “`Vendor` is a two-field inventory stub for `HasVendor` presence.” Does **not** supercede IR schema `assurance-ir/v1`, canonical digest `canon/v1`, ADR 0001 spine, Kleene `HasVendor` presence semantics, or governance catalog `control.vendor.*` IDs. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0004](0004-documentation-architecture.md) |
| Spec | [`docs/specs/supplier-risk.md`](../specs/supplier-risk.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_supplier_risk_baseline` found-case (fails on this HEAD); `sdd_supplier_risk_target` GREEN (`tests/contracts/supplier_risk.{baseline,target}.rs`) |

> Filename **`0007-*`**. Methodology / register / scheduler occupy sibling `0005-*`; treatment occupies `0006-*`. Risk identification is a sibling `0007-*`. Do **not** add `0003-supplier-risk.md`. Cite this file by **path**.

## Context

On SHA `6e31bf1a…`, `weeping-angel-assurance-ir::Vendor` was `{ id, name }` with `Vendor::new(id, name)`. Module docs said *“Minimal vendor node for the compliance graph.”* `AssessmentDefinition.vendors` was an unvalidated vec. `ProcessingActivity.processors` held `VendorId`s with no dangling check. `validate_assessment_ir` never walked vendors. Applicability `HasVendor` was presence-only.

Governance catalog already shipped `control.vendor.*` / `test.vendor.critical-risk-review-current` / `evidence.vendor.risk-review` and fixtures (`vendor-review-gaps`, expired exception). That is a catalog population family, not an IR supplier lifecycle.

Operational ISMS v1 Prompt 18 requires critical suppliers to be continuously represented as dependencies with accountable risk, evidence, review cadence, and control impact — risk-tiered, with evidence ≠ acceptance, and with Candidate → … → terminated history.

Questions this decision answers:

1. Does the lifecycle live on existing IR `Vendor`, a new type, or a procurement crate?
2. How do we expand the record without breaking `Vendor::new`, camelCase JSON, and `HasVendor`?
3. What is the status machine, including `restricted/suspended`?
4. How are review requirements risk-tiered without one-size-fits-all?
5. Why does evidence presence not imply approval or `Risk` acceptance?
6. How do we link services, assets, obligations, documents, and risks without forking Prompts 03/08/12?
7. How do expired assessments and lingering access fail closed?
8. Do we rewrite catalog TOML?

## Decision

This is what shipped. Field-level law is [`docs/specs/supplier-risk.md`](../specs/supplier-risk.md). Product home: `weeping-angel-assurance-ir::vendor` (`crates/weeping-angel-assurance-ir/src/vendor.rs`), re-exported from that crate’s `lib.rs`. Clocked and clockless checks live in `validation.rs`.

### 1. Same IR type, additive fields, same schema version

The lifecycle **is** `weeping-angel-assurance-ir::Vendor`. There is no `Supplier`, `VendorV2`, procurement crate, or `assurance-ir/v2`. `ASSURANCE_IR_SCHEMA` stays `assurance-ir/v1`. Additive fields use `serde(default)` and `skip_serializing_if` so `{ id, name }` and `Vendor::new(id, name)` keep working. JSON stays camelCase. Canonical digest stays serde field order + BTree maps.

Typed ids added: `SupplierReviewId`, `SupplierRequirementId`, `SupplierIssueId`. `obligation_ids`, `contract_document_refs`, requirement `document_ref` / `obligation_id`, and `evidence_refs` are **opaque strings** — AssessmentDefinition has no obligation or document inventory to fail-close against. Prompt 12 `DocumentRef` / Prompt 03 `ObligationId` remain neighbor types.

### 2. Default lifecycle is `Unspecified`; explicit machine starts at `Candidate`

```text
Unspecified | Candidate | UnderReview | Approved | Active
| Restricted | Suspended | Terminating | Terminated
```

`Unspecified` is `#[default]` so old JSON does not pretend to be an onboarded Candidate. It skip-serializes. `Restricted` and `Suspended` are distinct values in the prompt’s `restricted/suspended` slot.

`SupplierLifecycleStatus::can_transition` is the fail-closed table (spec §4.5). `Vendor::transition` is the only legal writer; illegal pairs return `VendorTransitionError::Illegal` (no panic). `Terminated` is terminal. `Terminating → Active` is the only reinstatement path. Recorded `history` `StatusTransition` pairs must obey the same table.

`transition(Approved)` requires `approval.decision == Approved`. `Conditional` / `Rejected` / missing approval return `VendorTransitionError::ApprovalRequired`. `attach_evidence` does not call `transition`. Clockless `validate()` also rejects `status = Approved` without that decision.

### 3. Risk-tiered requirements, not identical review for every name in the bag

`SupplierCriticality` (`Unspecified` | `Low` | `Medium` | `High` | `Critical`) selects required review and contract security-requirement depth. Privileged access (`access.privileged` or any privileged grant) elevates Low/Medium to High rules. Unspecified is not Low; privileged + Unspecified is a clockless `validate()` error.

```text
critical_suppliers(assessment)
Vendor::review_current(as_of)
Vendor::requires_current_security_review
validate_supplier_reviews_at(assessment, as_of)
```

`review_current` is true iff `nextReview >= as_of` or any onboarding/periodic review has `validUntil >= as_of`. Unscheduled is not current. Evidence refs do not make a review current. Low-tier without privileged access and without `Processor` classification is exempt from the clocked review gap and from the contract-requirement gap.

`validate_supplier_reviews_at` fails Critical/High (and elevated Medium/Low) in `{Approved, Active, Restricted, Suspended, Terminating}` without a current review. `Terminated` is not required to have a current periodic review. Clockless `validate()` does not auto-transition or fail stale reviews.

Kleene `HasVendor` stays presence-only.

### 4. Evidence presence is not risk acceptance

Reviews may cite questionnaires, manual review, or automated posture (`SupplierReviewSource`). Those `evidence_refs` record **existence**. They do not set lifecycle `Approved`/`Active`, do not set `RiskStatus::Accepted`, and do not mint Prompt 08 `RiskAcceptance`. Approval is a separate `SupplierApproval` record.

### 5. Consume neighbor types; typed refs otherwise

Reuse `PrincipalRef`, `Asset` / `AssetKind::Service`, `Identity`, `Exception`, `Risk`/`RiskId` (including landed `Risk.vendor_ids`), `EvidenceRequirementId` as neighbor types. Clockless `validate()` fail-closes on:

- duplicate `VendorId`
- dangling supplied `AssetId`, processing-activity id, grant asset/identity, owner/approver `Identity`, `RiskId`, `ExceptionId`, `ControlId`
- dangling `ProcessingActivity.processors`
- lingering access after `Terminating`/`Terminated` (`has_lingering_access`)
- missing contract security requirement when the tier/status row requires it
- privileged + unspecified criticality
- illegal recorded history transitions
- `Approved` without an Approved decision

`Risk.vendor_ids` entries must exist in `assessment.vendors`. `Vendor.risk_ids` entries must exist in `assessment.risks`. Reverse listing is not required (one-sided link is valid). Linkage does not accept residual risk.

### 6. Expired assessments are gaps; lingering access is clockless fail-closed

Overdue `nextReview` / review `validUntil` is a **clocked** query (`validate_supplier_reviews_at`). Clockless `validate()` stays valid for golden empty assessments. Expired reviews are representable as vendor-local `VendorEventKind::AssessmentExpired`; Prompt 15 may later project `VendorRiskChanged` / `SupplierAssessmentExpired` — this slice does not build the event bus.

`Terminated`/`Terminating` with any `Active` access grant or leftover privileged/data-access flags **fails clockless `validate()`**. All grants `Revoked` and flags clear succeeds.

An approved unexpired exception bound via `Vendor.exception_ids` or `Exception.subjects` (`SubjectKind::Vendor`) may skip the clocked review gap for that vendor only. `ExceptionStatus::Expired` or `expiresAt < as_of` does not suppress the gap. Clockless `validate()` still accepts an expired exception row as a stored fact (IR-020).

### 7. Catalog TOML is not a second supplier family

Governance catalog IDs remain the assurance-test SSOT. This slice does not rewrite `catalog/canonical/v1/**` and does not add `control.supplier.*`. IR queries make the same honesty available against `AssessmentDefinition.vendors`.

## Non-goals

- Questionnaire SaaS, procurement, contract authoring, trust-center scraping.
- Changing `HasVendor`, ISO pack remaps, GitHub collector, or existing dual-suite bodies except additive registration.
- Implementing Prompt 03/08/12/15 product engines (consume refs / seams only).
- Auto-advancing `nextReview` from cadence (scheduler).
- UI, persistence, ticketing.

## Consequences

- `Vendor` is the operational supplier-lifecycle SSOT; `{ id, name }` remains the constructor and the baseline found-case.
- Applicability continues to treat vendors as a presence inventory; new fields are ignored there.
- Governance catalog population tests stay GREEN; IR `validate_supplier_reviews_at` is the management-system record those tests can later read.
- Neighbor suites (`sdd_compliance_ir_target`, `sdd_applicability_engine_target`, `sdd_governance_catalog_target`) stay green.

## Related

- Spec: [`docs/specs/supplier-risk.md`](../specs/supplier-risk.md)
- Prompt: [`docs/prompts/operational-isms-v1/18-supplier-risk.md`](../prompts/operational-isms-v1/18-supplier-risk.md)
- Governance family: [`docs/specs/governance-canonical-assurance-catalog.md`](../specs/governance-canonical-assurance-catalog.md)
- Risk register: [`docs/specs/risk-register.md`](../specs/risk-register.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
