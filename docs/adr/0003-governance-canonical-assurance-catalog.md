# ADR 0003 — Governance family in the canonical assurance catalog (DRAFT)

| Field | Value |
| --- | --- |
| Status | **Draft** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md). Does **not** replace [ADR 0002](0002-iso-27001-assurance-vertical.md) or the ISO pack organizational sliver. |
| Extends | [Catalog infrastructure](0003-canonical-assurance-catalog-v1.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [population / coverage](0003-subject-population-runtime-and-coverage-semantics.md), [IAM family](0003-iam-canonical-assurance-catalog.md) |
| Spec | [`docs/sdd/governance-canonical-assurance-catalog.md`](../sdd/governance-canonical-assurance-catalog.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Prompt | [`docs/prompts/canonical-assurance-v1/08-governance-catalog.md`](../prompts/canonical-assurance-v1/08-governance-catalog.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | `sdd_governance_catalog_target` not yet GREEN. Accept this ADR only after that suite is GREEN. Absence-characterization baseline `sdd_governance_catalog_baseline` is superseded / `#[ignore]` at that time. |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. **Keep Draft** until `sdd_governance_catalog_target` is GREEN, then set Status to **Accepted**.

## Context

ADR 0001 delivered the inwardly extensible assurance spine. ADR 0002 shipped the first ISO 27001 vertical, including a **thin organizational sliver inside the ISO pack** (`incident.response-process`, `supplier.security-assessment`, `personnel.access-termination`, `access.periodic-review`) tested as presence/hybrid/manual checks on pack evidence (`policy.security.reviewed`, `policy.supplier.assessed`, `personnel.access.terminated`, `policy.access.reviewed`).

Canonical catalog infrastructure (Prompt 01), typed evidence (Prompt 02), subject-population coverage (Prompt 03), and the IAM family (Prompt 04) landed as sibling ADRs. They provide the loader/validator/digest, fact encoding, `AllSubjects` / `CoverageAtLeast` / `FreshWithin` / `ManualReview` runtime, Exception subject binding, and an identity **technical** library. They do not own policy, risk-governance, personnel-process, supplier, incident-governance, or continuity-plan content.

`manual_attestation` today is a compile capability and a legacy envelope type. It is not first-class catalog evidence with principal, timestamp, subject, artifact, freshness, and review state.

Without a provider-neutral governance family, a future GRC/ITSM collector has nowhere canonical to emit organizational facts, and tests such as “all required personnel have current training” cannot be declared without pretending a PDF is effectiveness.

Parallel Prompts 05–07 specify SDLC, vulnerability, and infrastructure families. This decision must not overwrite those files or steal their evidence ids.

Questions this decision answers:

1. Where do policy, risk governance, personnel process, supplier, incident governance, and BCP/DR *governance* controls live, if not in `frameworks/iso-27001/2022/metadata.toml`?
2. What public ID contract do future collectors and Prompt 12 ISO remapping consume?
3. How is manual evidence first-class and immutable rather than a boolean bypass?
4. How are organizational tests freshness/population predicates rather than “document exists”?
5. Do we fork the catalog loader, evidence values, population evaluator, or Exception/Risk types for this family?
6. How do we coexist with Prompt 04 IAM, Prompt 05 SDLC policy, Prompt 06 finding-level risk, and Prompt 07 operational resilience?

## Decision (proposed)

This is what will ship. Accept after target GREEN.

### 1. Governance is canonical catalog content, not a pack and not a GRC product

Independently assessable governance controls live in the Prompt 01 tree:

```text
catalog/canonical/v1/controls/governance.toml
catalog/canonical/v1/evidence/governance.toml
catalog/canonical/v1/tests/governance.toml
```

Optional split into `risk.toml` / `personnel.toml` / `vendor.toml` / `incident.toml`. **Do not** create `resilience.toml` (Prompt 07). Listed in `catalog/canonical/v1/manifest.toml` `[files]`. Loaded by `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` — **no second loader**.

Public IDs:

```text
control.{governance,risk,personnel,vendor,incident,resilience}.<slug>
evidence.{governance,risk,personnel,vendor,incident,resilience,manual}.<slug>
test.{governance,risk,personnel,vendor,incident,resilience}.<slug>
```

Incorrect: `control.vanta.ismp`, `control.servicenow.incident`, `control.iso27001.a.5.1`, or growing the ISO pack organizational list as the long-term library.

GRC/ITSM details belong only in future collectors that **emit** canonical facts. Framework details belong only in later mappings (Prompt 12).

### 2. Thirty-six provider-neutral controls (30–45 band)

Family covers information-security policy, policy review, roles, objectives, scope, internal audit, management review, corrective action, continual improvement, classification, acceptable use, asset ownership, document control, evidence retention, audit program, risk assessment/treatment/ownership/acceptance, incident plan/exercise/postmortem, awareness, role training, onboarding/offboarding *process*, confidentiality, policy acknowledgement, supplier inventory/review/requirements/reassessment/cloud governance, business-continuity plan, and DR *governance*.

Automation is honest: nearly all Hybrid or Manual. Freshness of a typed record may be automated; quality of an ISMS, training effectiveness, and diligence are not.

### 3. Manual evidence is first-class immutable evidence

Catalog type `evidence.manual.attestation` plus domain types:

```text
evidence.governance.{policy,policy-review,management-review,internal-audit}
evidence.risk.{assessment,treatment}
evidence.personnel.{training,acknowledgement}
evidence.vendor.{inventory,risk-review}
evidence.incident.exercise
evidence.resilience.continuity-plan
evidence.manual.attestation
```

Shared shape: principal/author, timestamp, subject, artifact reference where relevant, freshness/validity, review state. Stored via `EvidenceValue::with_value`. Not a boolean bypass. Document existence does not prove operational effectiveness.

Legacy `manual_attestation` (capability / pack type) remains for ISO compile; this family does not retarget it.

### 4. Tests are freshness, population, and manual-review predicates

`test.personnel.training-current-all` means **all in-scope required personnel have current training**, using Prompt 03 arms. It does not mean “some training envelope exists.”

Required tests include: policy current inside review window; all required personnel trained; all critical vendors have current risk reviews; management review inside period; internal audit current; incident exercise inside window.

Missing evidence ⇒ `InsufficientEvidence`. Partial populations cannot be `Effective` on all-subjects tests. Hybrid/manual controls stay Hybrid/Manual and cannot auto-pass from a document-present flag (`TestExpr::ManualReview` → `ManualReviewRequired`).

### 5. Exceptions and risks stay on existing IR

Approved unexpired subject-bound IR `Exception` ⇒ `ExceptionApproved` for that subject — **never silent `Effective`**. Expired exceptions must not suppress failing results. Empty `subjects` is not the whole inventory.

IR `Risk` is reused as an attestation record, not grown into a GRC engine. Finding-level `evidence.vulnerability.exception` is Prompt 06.

If `evaluate_coverage` would convert excepted subjects into remaining-all-pass `Effective`, a **minimal** generic ExceptionApproved promotion in the existing evaluator is allowed. No second exception type.

### 6. Coexist with siblings and the ISO sliver until Prompt 12 remaps

| Sibling | Boundary |
| --- | --- |
| Prompt 04 IAM | Technical MFA / membership / account status vs personnel *process* / training / confidentiality |
| Prompt 05 SDLC | Secure-development policy / change source vs IS policy / AUP / document-control / retention |
| Prompt 06 vuln | Finding-level risk acceptance vs organizational risk attestations |
| Prompt 07 infra | Operational restore / `evidence.resilience.recovery-plan` / `resilience.toml` vs BCP/DR governance / `evidence.resilience.continuity-plan` |
| ISO pack | Frozen sliver; do not retarget mappings |

Two libraries coexist until Prompt 12: pack `incident.response-process` vs catalog `control.incident.*`.

### 7. Deterministic fixtures

Eight frozen sets under `fixtures/assurance/canonical/v1/governance/`:

`current-documents`, `stale-documents`, `missing-documents`, `incomplete-training-population`, `vendor-review-gaps`, `approved-exception`, `expired-exception`, `manual-review-despite-evidence`.

Populations use Prompt 03 generic `inventory.subject` / `inventory.complete` (or explicit fixture population). No personnel/vendor resolver fork.

### 8. Consume Prompts 01–03; do not fork infrastructure

No second catalog loader, typed `EvidenceValue`, or population evaluator. Prompt 01’s SSOT is not overwritten (pointer-only). No ServiceNow / Jira / Vanta / Drata collector. Validator GRC-product gaps are covered by target greps of **catalog TOML**, never by a self-referential “this test file does not contain `vanta`” assert.

## Consequences

**Positive**

- Future organizational collectors have a stable emit contract (`evidence.manual.attestation`, `evidence.governance.*`, …).
- Prompt 12 can map ISO organizational annexes onto `control.governance.*` / `control.incident.*` / `control.vendor.*` without rewriting collectors.
- Assessments can be honest about manual/hybrid controls instead of faking automation.

**Negative / cost**

- Two organizational libraries until remap (pack vs catalog).
- Hybrid/manual tests will not auto-pass from documents alone; assessments need real attestations.
- ExceptionApproved honesty may require a small generic promotion in Prompt 03 evaluation if excepted subjects would otherwise become silent `Effective`.
- Continuity IDs share the `control.resilience.*` namespace with Prompt 07; filenames and slugs must stay disjoint.

**Rejected**

- GRC-product-prefixed control IDs.
- Encoding “policy current” as `Exists(evidence.governance.policy)` without a review window.
- Building a document editor or GRC workflow.
- Rewriting ISO `metadata.toml` / `mappings.toml`.
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
- ISO vertical (sliver frozen): [`0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
