# ADR 0003 — Governance family in the canonical assurance catalog

| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md). Does **not** replace [ADR 0002](0002-iso-27001-assurance-vertical.md) or [ISO remap](0003-iso27001-canonical-remap.md). |
| Extends | [Catalog infrastructure](0003-canonical-assurance-catalog-v1.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [population / coverage](0003-subject-population-runtime-and-coverage-semantics.md), [IAM family](0003-iam-canonical-assurance-catalog.md) |
| Spec | [`docs/sdd/governance-canonical-assurance-catalog.md`](../sdd/governance-canonical-assurance-catalog.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Prompt | [`docs/prompts/canonical-assurance-v1/08-governance-catalog.md`](../prompts/canonical-assurance-v1/08-governance-catalog.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Fixture clock | `2026-08-18T12:00:00Z` (`stale-documents` uses `2024-08-01T12:00:00Z`) |
| Tests | `sdd_governance_catalog_target` GREEN (GOV-001…016). Absence-characterization baseline `sdd_governance_catalog_baseline` superseded / `#[ignore]`. |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. Accepted after `sdd_governance_catalog_target` GREEN.

## Context

ADR 0001 delivered the inwardly extensible assurance spine. ADR 0002 shipped the first ISO 27001 vertical, including a **thin organizational sliver inside the ISO pack** (`incident.response-process`, `supplier.security-assessment`, `personnel.access-termination`, `access.periodic-review`) tested as presence/hybrid/manual checks on pack evidence (`policy.security.reviewed`, `policy.supplier.assessed`, `personnel.access.terminated`, `policy.access.reviewed`). [Prompt 12 remap](0003-iso27001-canonical-remap.md) later retired those pack-local slivers and left the corresponding ISO clauses **unmapped**.

Canonical catalog infrastructure (Prompt 01), typed evidence (Prompt 02), subject-population coverage (Prompt 03), and the IAM family (Prompt 04) landed as sibling ADRs. They provide the loader/validator/digest, fact encoding, `AllSubjects` / `CoverageAtLeast` / `FreshWithin` / `ManualReview` runtime, Exception subject binding, and an identity **technical** library. They do not own policy, risk-governance, personnel-process, supplier, incident-governance, or continuity-plan content.

`manual_attestation` was a compile capability and a legacy envelope type. It was not first-class catalog evidence with principal, timestamp, subject, artifact, freshness, and review state.

Without a provider-neutral governance family, a future GRC/ITSM collector has nowhere canonical to emit organizational facts, and tests such as “all required personnel have current training” cannot be declared without pretending a PDF is effectiveness.

Parallel Prompts 05–07 specify (and later landed) SDLC, vulnerability, and infrastructure families. This decision must not overwrite those files or steal their evidence ids.

Questions this decision answers:

1. Where do policy, risk governance, personnel process, supplier, incident governance, and BCP/DR *governance* controls live, if not in `frameworks/iso-27001/2022/metadata.toml`?
2. What public ID contract do future collectors and a later honest ISO remap consume?
3. How is manual evidence first-class and immutable rather than a boolean bypass?
4. How are organizational tests freshness/population predicates rather than “document exists”?
5. Do we fork the catalog loader, evidence values, population evaluator, or Exception/Risk types for this family?
6. How do we coexist with Prompt 04 IAM, Prompt 05 SDLC policy, Prompt 06 finding-level risk, and Prompt 07 operational resilience?
7. How do approved unexpired IR exceptions avoid silent `Effective` when excepted subjects leave the coverage denominator?

## Decision

This is what shipped.

### 1. Governance is canonical catalog content, not a pack and not a GRC product

Independently assessable governance controls live in the Prompt 01 tree (single files, not a split):

```text
catalog/canonical/v1/controls/governance.toml
catalog/canonical/v1/evidence/governance.toml
catalog/canonical/v1/tests/governance.toml
```

Listed in `catalog/canonical/v1/manifest.toml` `[files]`. Loaded by `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` — **no second loader**. Continuity/DR **governance** IDs live here even though Prompt 07 owns `resilience.toml` for operational restore (`evidence.resilience.recovery-plan`). This slice did **not** create or overwrite `resilience.toml`.

Public IDs:

```text
control.{governance,risk,personnel,vendor,incident,resilience}.<slug>
evidence.{governance,risk,personnel,vendor,incident,resilience,manual}.<slug>
test.{governance,risk,personnel,vendor,incident,resilience}.<slug>
```

Incorrect: `control.vanta.ismp`, `control.servicenow.incident`, `control.iso27001.a.5.1`, or growing the ISO pack organizational list as the long-term library.

GRC/ITSM details belong only in future collectors that **emit** canonical facts. Framework details belong only in later mappings (not this slice).

### 2. Thirty-four provider-neutral controls (30–45 band)

Shipped family: 34 independently assessable controls (25 Hybrid, 9 Manual, 0 Automated). Freshness of a typed record may be automated; quality of an ISMS, training effectiveness, and diligence are not.

| Control | Automation |
| --- | --- |
| `control.governance.information-security-policy` | hybrid |
| `control.governance.policy-review` | hybrid |
| `control.governance.roles-and-responsibilities` | **manual** |
| `control.governance.security-objectives` | **manual** |
| `control.governance.documented-scope` | hybrid |
| `control.governance.internal-audit` | hybrid |
| `control.governance.management-review` | hybrid |
| `control.governance.corrective-action` | hybrid |
| `control.governance.continual-improvement` | **manual** |
| `control.governance.data-classification-policy` | hybrid |
| `control.governance.acceptable-use-policy` | hybrid |
| `control.governance.asset-ownership` | hybrid |
| `control.governance.document-control` | hybrid |
| `control.governance.evidence-retention` | hybrid |
| `control.governance.audit-program` | **manual** |
| `control.risk.assessment` | hybrid |
| `control.risk.treatment` | hybrid |
| `control.risk.ownership` | hybrid |
| `control.risk.acceptance` | **manual** |
| `control.incident.response-plan` | hybrid |
| `control.incident.exercise` | hybrid |
| `control.incident.postmortem` | hybrid |
| `control.personnel.security-awareness` | hybrid |
| `control.personnel.role-specific-training` | hybrid |
| `control.personnel.onboarding-offboarding` | **manual** |
| `control.personnel.confidentiality-commitment` | hybrid |
| `control.personnel.policy-acknowledgement` | hybrid |
| `control.vendor.inventory` | hybrid |
| `control.vendor.risk-review` | hybrid |
| `control.vendor.security-requirements` | **manual** |
| `control.vendor.reassessment` | hybrid |
| `control.vendor.cloud-governance` | **manual** |
| `control.resilience.business-continuity-plan` | hybrid |
| `control.resilience.disaster-recovery-governance` | **manual** |

Each control has stable id, domain(s), evidence requirements, and a matching `test.*` ref. Canonical governance TOML contains no ISO/SOC2/NIS2/DORA/GDPR or GRC-product tokens (`vanta`, `drata`, `servicenow`, `jira`).

### 3. Manual evidence is first-class immutable evidence

Thirteen catalog types (catalog id → envelope `evidenceType`):

```text
evidence.manual.attestation
evidence.governance.{policy,policy-review,management-review,internal-audit}
evidence.risk.{assessment,treatment}
evidence.personnel.{training,acknowledgement}
evidence.vendor.{inventory,risk-review}
evidence.incident.exercise
evidence.resilience.continuity-plan
```

Shared attestation shape, stored via `EvidenceValue::with_value`: `subject_id`, `attested_by`, `attested_at`, `kind` (`attestation` \| `document-reference` \| `approval` \| `meeting-record` \| `auditor-observation` \| `training-record` \| `exercise-record` \| `risk-acceptance` \| `policy-acknowledgement`), `artifact_ref?`, `review_state` (`draft` \| `submitted` \| `reviewed` \| `accepted` \| `rejected`), `valid_until?`, `current?`. Domain types add dated fields (`reviewed_at`, `trained_at`, `exercised_at`, …). Not a boolean bypass. Document existence does not prove operational effectiveness. No secret material; seal still rejects credential-shaped keys and compliance narratives.

Legacy `manual_attestation` (capability / pack type / `collector.manual`) remains. This family does not retarget it and `ManualEvidence` does not emit `evidence.manual.attestation`.

### 4. Tests are freshness, population, and manual-review predicates

Thirty-four tests, one per control. Required Prompt-08 scenarios:

```text
test.governance.policy-current
test.governance.management-review-current
test.governance.internal-audit-current
test.personnel.training-current-all
test.vendor.critical-risk-review-current
test.incident.exercise-current
```

`test.personnel.training-current-all` means **all in-scope required personnel have current training** (`op = "all-subjects"` on `evidence.personnel.training` / `current`). It does not mean “some training envelope exists.” Policy / audit / management-review / exercise tests use `op = "fresh-within"` on the dated field (`reviewed_at` / `audited_at` / `exercised_at`, `365d`). Hybrid/manual quality tests use `op = "manual-review"` → `ManualReviewRequired`.

Missing evidence ⇒ `InsufficientEvidence`. Partial populations cannot be `Effective` on all-subjects tests. A single document-present flag cannot auto-pass a hybrid/manual control that requires operational evidence.

### 5. Exceptions and risks stay on existing IR; silent Effective is forbidden

Approved unexpired subject-bound IR `Exception` ⇒ `ExceptionApproved` for that subject — **never silent `Effective`**. Expired exceptions must not suppress failing results. Empty `subjects` is not the whole inventory.

IR `Risk` is reused as an attestation record, not grown into a GRC engine. Finding-level `evidence.vulnerability.exception` remains Prompt 06.

**Shipped runtime honesty (Prompt 03 evaluator, not a second engine):** `evaluate_coverage` still removes excepted subjects from the coverage denominator, and still promotes `Ineffective` → `ExceptionApproved` when every remaining failing subject is identity break-glass. Additionally, when `conclude` would return `Effective` solely because approved unexpired bound exceptions emptied the remainder (`excepted` non-empty; `failing` / `missing` / `stale` / `technical` empty), overall effectiveness is `ExceptionApproved` with rationale `approved unexpired exception bound to excepted subjects; not silent Effective`. Same IR `Exception` type. No `GovernanceException`.

### 6. Coexist with siblings; ISO organizational clauses stay unmapped

| Sibling | Boundary |
| --- | --- |
| Prompt 04 IAM | Technical MFA / membership / account status vs personnel *process* / training / confidentiality |
| Prompt 05 SDLC | Secure-development policy / change source vs IS policy / AUP / document-control / retention |
| Prompt 06 vuln | Finding-level risk acceptance vs organizational risk attestations |
| Prompt 07 infra | Operational restore / `evidence.resilience.recovery-plan` / `resilience.toml` vs BCP/DR governance / `evidence.resilience.continuity-plan` in `governance.toml` |
| ISO pack | This slice does **not** retarget mappings. Prompt 12 already retired pack-local organizational slivers; `iso27001:a.5.19` / `a.5.24` / `a.5.1` / `5.2` and clauses 4–10 stay unmapped rather than claimed as `control.governance.*` / `control.incident.*` |

Reserved Prompt-07 `control.resilience.*` slugs (`recovery-procedure`, `disaster-recovery-exercise`, `redundancy`, `recovery-objectives`, `recovery-evidence-freshness`) are not reused here.

### 7. Deterministic fixtures

Eight frozen sets under `fixtures/assurance/canonical/v1/governance/`:

| Fixture | Distinguishes |
| --- | --- |
| `current-documents` | Fresh typed records can be `Effective`; quality tests stay `ManualReviewRequired` without complete operational attestations |
| `stale-documents` | Dated envelopes outside window → `StaleEvidence` |
| `missing-documents` | No required envelopes → `InsufficientEvidence` |
| `incomplete-training-population` | Authoritative N personnel, N−1 training envelopes → not `Effective` |
| `vendor-review-gaps` | Critical vendor missing current review → not `Effective` |
| `approved-exception` | Bound approved unexpired IR exception → `ExceptionApproved`, not silent `Effective` |
| `expired-exception` | Expired exception does **not** suppress fail/missing |
| `manual-review-despite-evidence` | Supporting document present; `op = "manual-review"` → `ManualReviewRequired` |

Populations use Prompt 03 generic `inventory.subject` / `inventory.complete` (or explicit fixture population). No personnel/vendor resolver fork. Fixtures emit canonical `evidence.{governance,risk,personnel,vendor,incident,resilience,manual}.*` only.

### 8. Consume Prompts 01–03; do not fork infrastructure

No second catalog loader, typed `EvidenceValue`, or population evaluator. Prompt 01’s SSOT is not overwritten (pointer-only). No ServiceNow / Jira / Vanta / Drata collector. Validator GRC-product gaps are covered by target greps of **catalog TOML**, never by a self-referential “this test file does not contain `vanta`” assert.

## Consequences

**Positive**

- Future organizational collectors have a stable emit contract (`evidence.manual.attestation`, `evidence.governance.*`, …).
- Assessments can be honest about manual/hybrid controls instead of faking automation.
- Approved unexpired exceptions are visible as `ExceptionApproved` on any population test, not only IAM break-glass.

**Negative / cost**

- ISO organizational clauses remain unmapped; catalog `control.incident.*` is not an ISO projection.
- Hybrid/manual tests will not auto-pass from documents alone; assessments need real attestations.
- Continuity IDs share the `control.resilience.*` namespace with Prompt 07; filenames and slugs must stay disjoint (`governance.toml` vs `resilience.toml`).
- The generic ExceptionApproved promotion changes Prompt 03 coverage conclusions for every family that binds IR exceptions (not governance-only).

**Rejected**

- GRC-product-prefixed control IDs.
- Encoding “policy current” as `Exists(evidence.governance.policy)` without a review window.
- Building a document editor or GRC workflow.
- Rewriting ISO `metadata.toml` / `mappings.toml` in this slice.
- Inventing a second exception/risk engine or catalog loader.
- Implementing Prompts 05–07 in this slice.

## Non-goals (reaffirmed)

GRC SaaS; document editors; ISO/SOC2/NIS2 mappings; generic rule-engine expansion; ServiceNow/Jira/Vanta/Drata collectors; certification language; IAM MFA/role-membership; SDLC technical change controls; vulnerability finding exceptions; operational backup/HA.

## Access and security

- Catalog load remains local-filesystem only.
- Governance fixtures store principals, timestamps, subject ids, artifact locators — never tokens, passwords, or recovered secrets.
- Seal still rejects credential-shaped fact keys and compliance narratives.

## Related

- Spec SSOT: [`docs/sdd/governance-canonical-assurance-catalog.md`](../sdd/governance-canonical-assurance-catalog.md)
- Public contract: [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md)
- Catalog infrastructure: [`0003-canonical-assurance-catalog-v1.md`](0003-canonical-assurance-catalog-v1.md)
- Typed evidence: [`0003-typed-evidence-canonical-serialization.md`](0003-typed-evidence-canonical-serialization.md)
- Population runtime: [`0003-subject-population-runtime-and-coverage-semantics.md`](0003-subject-population-runtime-and-coverage-semantics.md)
- IAM (personnel technical sibling): [`0003-iam-canonical-assurance-catalog.md`](0003-iam-canonical-assurance-catalog.md)
- Infrastructure (operational resilience sibling): [`0003-infrastructure-canonical-assurance-catalog.md`](0003-infrastructure-canonical-assurance-catalog.md)
- ISO remap (organizational clauses unmapped): [`0003-iso27001-canonical-remap.md`](0003-iso27001-canonical-remap.md)
- ISO vertical: [`0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
