# SDD: Governance Canonical Assurance Catalog (v1 slice)

| Field | Value |
| --- | --- |
| Status | **Implemented — target GREEN; baseline superseded** |
| Program | Canonical Assurance Catalog v1 |
| Slice | Prompt 08 — policy / risk governance / personnel security / supplier / incident governance / continuity governance / first-class manual evidence |
| Source prompt | [`docs/prompts/canonical-assurance-v1/08-governance-catalog.md`](../prompts/canonical-assurance-v1/08-governance-catalog.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` (`main`, 2026-08-19; found-case: no governance family) |
| Dual-suite | `sdd_governance_catalog_target` GREEN (GOV-001…016); `sdd_governance_catalog_baseline` superseded (`#[ignore]`) |
| Landed family | 34 controls (25 Hybrid / 9 Manual), 13 evidence types, 34 tests, eight fixtures; clock `2026-08-18T12:00:00Z` |
| ADR | Accepted [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — governance-family pointer + `evidence.manual.attestation` + ExceptionApproved honesty |
| Prompt-01 SSOT (do not overwrite) | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |
| Prompt-02 / 03 (consumed) | [`docs/sdd/typed-evidence.md`](typed-evidence.md), [`docs/sdd/population-runtime.md`](population-runtime.md) |
| Prompt-04 sibling (do not overwrite) | [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md) |
| Concurrent siblings (do not overwrite) | Prompt 05 [`sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md); Prompt 06 [`vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md); Prompt 07 [`infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md) — product landed in parallel |
| Spine / ISO law | [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for the **governance catalog slice** (Prompt 08). It does not replace Prompt 01 catalog infrastructure, Prompt 02 typed evidence, Prompt 03 population runtime, or Prompt 04 IAM content. Prompts 01–04 have landed in product code; this slice consumes their loader, `EvidenceValue`, population evaluator, Exception/Risk IR, and catalog tree and **must not** invent a second copy.

Product TOML, fixtures, dual-suite Rust, `Cargo.toml` `[[test]]` rows, and the public-contract pointer have landed. Consume Prompts 01–03; do not invent a second loader.

Architecture law (unchanged):

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

Core law for this slice: **manual evidence is first-class immutable evidence, not a boolean bypass.** Do not force technical automation onto inherently organizational controls. Document existence does not prove implementation effectiveness when the control requires operational evidence. Approved exceptions must **never** be silently converted into `Effective`. Expired exceptions must not suppress failing results.

---

## 1. Problem / user-visible goal

Organizations need to assess information-security policy, risk governance, personnel security (training, onboarding/offboarding process, confidentiality, acknowledgements), supplier management, incident *governance*, internal review, and business-continuity *plans* using **provider-neutral** canonical controls.

On SHA `e430980c…` the only governance-adjacent product content is a **thin ISO 27001 pack sliver** of organizational-adjacent ids, tested as presence / hybrid / manual checks on pack evidence types — not as first-class manual attestations or population predicates:

| Pack control | Pack automation | Test kind | Required evidence | What it can say today |
| --- | --- | --- | --- | --- |
| `incident.response-process` | Manual | manual | `policy.security.reviewed` | some policy-reviewed envelope exists |
| `supplier.security-assessment` | Manual | manual | `policy.supplier.assessed` | some supplier-assessed envelope exists |
| `personnel.access-termination` | Hybrid | hybrid | `personnel.access.terminated` | some termination envelope exists (IAM Prompt 04 owns the *technical* account-status sibling) |
| `access.periodic-review` | Hybrid / test manual | manual | `policy.access.reviewed` | some access-review policy envelope exists |

Those tests cannot say “the information-security policy is inside its review window,” “all required personnel have current training,” “all critical vendors have current risk reviews,” “management review occurred within the policy period,” or “an incident tabletop occurred inside the required window.” They cannot distinguish missing evidence from a stale document, a partial training population from a healthy one, or an approved unexpired exception from a silent pass.

The canonical catalog at `catalog/canonical/v1/` lists only `fixture.example.toml` and the IAM family (`identity.toml`). There is no `control.governance.*` / `control.risk.*` / `control.personnel.*` / `control.vendor.*` / `control.incident.*` library, no `evidence.manual.attestation` or `evidence.governance.*` / `evidence.risk.*` / `evidence.personnel.*` / `evidence.vendor.*` / `evidence.incident.*` / `evidence.resilience.continuity-plan` contracts, and no `fixtures/assurance/canonical/v1/governance/*` sets.

`manual_attestation` exists as a **compile capability flag** (`supports_manual_attestation`) and a **legacy pack evidence-type name** consumed by ISO presence evaluation. It is **not** catalog evidence `evidence.manual.attestation` with principal, timestamp, subject, artifact, freshness, and review state.

**User-visible goal:** a coherent governance catalog (~30–45 independently assessable controls) that treats organizational evidence as first-class immutable facts, evaluates freshness and population coverage honestly, distinguishes missing ≠ stale ≠ fail ≠ manual review ≠ approved exception, and passes catalog validation plus full workspace verification.

This slice does **not** claim ISO / SOC 2 / NIS 2 coverage. Framework remapping is Prompt 12. This slice does **not** implement a GRC product, document editor, or collectors for ServiceNow / Jira / Vanta / Drata.

---

## 2. Dependencies and fail-closed blockers

| Prompt | Owns | On `e430980c…` | This slice may |
| --- | --- | --- | --- |
| 01 catalog contract | `catalog/canonical/v1/`, `CanonicalCatalog::{load,validate,digest}`, stable-ID rules | **Landed.** Identity + fixture.example listed in `manifest.toml`. | Add governance family TOML + manifest `[files]` lines. Do not invent a second loader/validator/digest. Do not delete fixture.example IDs. |
| 02 typed evidence | Typed `EvidenceValue`, `with_value`, seal rules | **Landed.** | Declare required fact *names* and semantic types. Store via `with_value`. No second value enum. No secret material. |
| 03 population runtime | `AllSubjects` / `CoverageAtLeast` / `NoneSubjects` / `FreshWithin` / `ManualReview`, missing/stale/fail split, `inventory.subject` + `inventory.complete`, Exception subject binding | **Landed.** Identity inventory special-case + generic inventory. | Declare population-based tests. **Do not locally reimplement coverage math. Do not add `resolve_personnel_inventory` / `resolve_vendor_inventory`.** |
| 04 IAM | `control.identity.*` technical identity / MFA / privileged membership / account status | **Landed.** | Leave `identity.toml`, identity fixtures, and `sdd_iam_catalog_target` green. Personnel *governance* (process, training, confidentiality, acknowledgements) lives here. |
| 05 SDLC | `control.source.*` / CI / supply-chain, including `control.source.secure-development-policy` | **Landed** (`sdlc.toml`). | Do **not** rewrite SDLC. Information-security policy, AUP, document-control, and evidence-retention *governance* live here. |
| 06 vulnerability | Finding-level risk acceptance, `evidence.vulnerability.exception` | **Landed** (`vulnerability.toml`; SSOT [`vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md)). | Do **not** rewrite `vulnerability.toml` or `evidence.secret.exposure`. Organizational risk assessment / treatment / ownership / acceptance *attestations* live here. |
| 07 infrastructure | Operational backup/restore/HA, `evidence.resilience.recovery-plan`, `*/resilience.toml` | **Landed** (`network` / `crypto` / `data` / `database` / `logging` / `backup` / `resilience.toml`). | Do **not** overwrite operational TOML. Continuity *plan* and DR *governance* live in `governance.toml` (`evidence.resilience.continuity-plan`). |
| ISO pack | Organizational slivers listed in §3.3 | Frozen | **Do not retarget mappings or grow the pack** (Prompt 12). |

Rebase rule: adapt governance content to the landed Prompt 01 file layout (`controls/*.toml`, `evidence/*.toml`, `tests/*.toml`, manifest `[files]`). If Prompts 05–07 land files during this session, **do not overwrite them**. Add only this slice’s files and manifest lines.

Harness rule (root `Cargo.toml` does **not** auto-discover `tests/sdd/*.rs`). Implement **must** add:

```toml
[[test]]
name = "sdd_governance_catalog_baseline"
path = "tests/sdd/governance_catalog.baseline.rs"

[[test]]
name = "sdd_governance_catalog_target"
path = "tests/sdd/governance_catalog.target.rs"
```

Without those stanzas, `cargo test --test sdd_governance_catalog_{baseline,target}` fails with `no test target named …` before any `#[test]` runs.

---

## 3. Current behavior (characterization on `e430980c…`)

This section is the **found case** for the baseline suite. Unlike the superseded IAM baseline (which assumed no `catalog/` at all), this characterization is of the **current** tree after Prompts 01–04.

### 3.1 Catalog tree and loader

`catalog/canonical/v1/manifest.toml` `[files]` lists **only**:

```text
controls = ["controls/fixture.example.toml", "controls/identity.toml"]
evidence = ["evidence/fixture.example.toml", "evidence/identity.toml"]
tests    = ["tests/fixture.example.toml", "tests/identity.toml"]
```

`weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` exists and validates the listed files. Extra unlisted `*.toml` in those directories fail closed. Provider/framework segments in IDs fail closed.

Validator denylists (`PROVIDER_SEGMENTS` / `FRAMEWORK_SEGMENTS` in `crates/weeping-angel-canonical-catalog/src/lib.rs`) include cloud/SCM/IdP and ISO/SOC2/NIS2/DORA/GDPR tokens. They **omit GRC-product tokens** (`vanta`, `drata`, `servicenow`, `jira`). Target suite **must grep those tokens** out of canonical governance IDs and narrative; do not rely on the loader denylist alone.

There is **no**:

- `catalog/canonical/v1/{controls,evidence,tests}/governance.toml` (nor `risk.toml` / `personnel.toml` / `vendor.toml` / `incident.toml` for this slice)
- `control.governance.*` / `control.risk.*` / `control.personnel.*` / `control.vendor.*` / `control.incident.*` ids
- `evidence.manual.attestation` / `evidence.governance.*` / `evidence.risk.*` / `evidence.personnel.*` / `evidence.vendor.*` / `evidence.incident.*` / `evidence.resilience.continuity-plan`
- `fixtures/assurance/canonical/v1/governance/`
- `sdd_governance_catalog_{baseline,target}` `[[test]]` rows

`control.resilience.*` / `evidence.resilience.recovery-plan` were **specified** by Prompt 07 and **not** product on this SHA. This slice must not create `resilience.toml` (Prompt 07 later did; continuity IDs still live in `governance.toml`).

### 3.2 IAM sibling (present; do not disturb)

23 `control.identity.*` controls, 12 `evidence.identity.*` types, 23 `test.identity.*` tests, eight fixtures under `fixtures/assurance/canonical/v1/identity/`. Dual-suite `sdd_iam_catalog_target` is the CI gate; `sdd_iam_catalog_baseline` is `#[ignore]` superseded.

Prompt 04 owns technical identity facts (MFA, privileged membership, account status, JML *events as identity facts*). This slice must not retarget those tests to personnel-governance evidence.

### 3.3 ISO pack organizational sliver (frozen)

`frameworks/iso-27001/2022/metadata.toml` still owns the pack ids in §1. Mappings (do not retarget):

| Requirement | Pack control |
| --- | --- |
| `iso27001:a.5.19` | `supplier.security-assessment` |
| `iso27001:a.5.24` | `incident.response-process` |
| `iso27001:5.2` / `iso27001:a.5.1` | `incident.response-process` (related) |
| `iso27001:a.5.16` / `a.6.5` | `personnel.access-termination` |
| `iso27001:a.5.18` | `access.periodic-review` |

`sdd_iso27001_assurance_target` freezes prefixes including `incident.`, `supplier.`, `personnel.`, `access.` and expected evidence names `policy.security.reviewed`, `policy.supplier.assessed`, `personnel.access.terminated`, `policy.access.reviewed`.

### 3.4 Manual evidence today

| Surface | What it is |
| --- | --- |
| `AssessmentRequests.manual_attestation` / `supports_manual_attestation` | Compile capability. Requested-and-unsupported → `CapabilityViolation`. |
| Envelope type `manual_attestation` | Legacy pack/runtime name. ISO manual tests without that type → `InsufficientEvidence` / `ManualReviewRequired`. |
| `ManualEvidence` / `collector.manual` | Requires `--attested-by`. Seals `attested_by` + `reason` string facts plus a narrative. **Never synthesized.** Does not emit `evidence.manual.attestation`. |
| `TestExpr::ManualReview` | Always `ManualReviewRequired` (“expression requires manual review”). |
| Catalog | No `evidence.manual.attestation` declaration. No shared attestation shape (principal, timestamp, subject, artifact, freshness, review state) as first-class catalog content. |

A PDF path or “document present” boolean is not operational effectiveness.

### 3.5 Evidence, population, exceptions, risk (consumed as-is)

- Facts are `BTreeMap<String, EvidenceValue>`. Use `with_value` for typed facts. `with_fact` stores `String` only.
- Population resolution: explicit `EvidenceSet` population → closed selector `ids` → `evidence.identity.inventory` special-case → generic `inventory.subject` + `inventory.complete` → else inferred observations (**Unknown**). Strong all-subjects tests refuse `Effective` on Partial/Unknown.
- `SubjectKind` already includes `Organization`, `User`, `Identity`, `Vendor`. Do not add a third `SubjectSelector`.
- `Exception` IR: `{ id, controlId, rationale, status, approvedBy, expiresAt, subjects }`. Empty `subjects` does **not** mean the entire inventory. `subject_is_excepted` skips approved unexpired subject-bound exceptions from the population denominator.
- `evaluate_coverage` additionally promotes `Ineffective` → `ExceptionApproved` when **every** remaining failing subject is an identity `account_kind=break-glass`. That promotion is IAM-shaped. See §4.8.
- `Risk` IR is a minimal record (`id`, `title`, `description`, `status` ∈ Open/Accepted/Mitigated/Closed). **Not** a risk engine. This slice attests organizational risk work; it does not grow `Risk` into a GRC workflow.

### 3.6 What “governance assessment” means today

A caller can compile the ISO pack and run `test.incident.response-process`, which requires **some** `policy.security.reviewed` envelope (manual). It cannot:

- require a current information-security policy inside a review window;
- require current training evidence for every in-scope person;
- require current risk reviews for every critical vendor;
- distinguish missing vs stale management-review / internal-audit / tabletop evidence;
- treat an attestation as an immutable fact with author, time, subject, and artifact;
- accept ServiceNow/Jira/Vanta/Drata-shaped facts without teaching tests about those products.

The baseline suite therefore characterizes **absence of a canonical governance family and first-class `evidence.manual.attestation`**, plus **presence of the ISO organizational sliver and the IAM sibling**, not a working GRC evaluator.

### 3.7 Spec/ADR existence vs baseline asserts

This file and the draft ADR **exist after the spec phase**. Baseline tests written in the implement phase **must not** assert that these two markdown paths are missing (that would be immediately false). Baseline asserts product absence (TOML, fixtures, catalog ids, `[[test]]` rows before they are added) and ISO/IAM coexistence.

---

## 4. Desired behavior (after this slice)

### 4.1 Placement

Governance domain content lands in the Prompt 01 catalog tree:

```text
catalog/canonical/v1/
  manifest.toml                 # add listings only
  controls/governance.toml      # control.governance|risk|personnel|vendor|incident|resilience (continuity only)
  evidence/governance.toml
  tests/governance.toml
```

Optional split if cleaner (each file still listed in `[files]`):

```text
{controls,evidence,tests}/{governance,risk,personnel,vendor,incident}.toml
```

**Do not** create `{controls,evidence,tests}/resilience.toml` — Prompt 07 specifies that filename for operational resilience (`evidence.resilience.recovery-plan`). Continuity-governance IDs (`control.resilience.business-continuity-plan`, `control.resilience.disaster-recovery-governance`, `evidence.resilience.continuity-plan`) live in `governance.toml` (or `continuity.toml` if a split is needed).

Do **not** add these controls to `frameworks/iso-27001/2022/metadata.toml`. Do **not** edit `identity.toml`, `fixture.example.toml`, or Prompt 05/06/07 product paths if they land.

Deterministic fixtures (preferred path):

```text
fixtures/assurance/canonical/v1/governance/
  current-documents/
  stale-documents/
  missing-documents/
  incomplete-training-population/
  vendor-review-gaps/
  approved-exception/
  expired-exception/
  manual-review-despite-evidence/
```

Each directory contains a frozen `evidence.json` (+ optional Exception / Risk records) with a fixed `collectedAt`.

### 4.2 ID and neutrality rules

Stable public IDs:

```text
control.{governance,risk,personnel,vendor,incident,resilience}.<slug>
evidence.{governance,risk,personnel,vendor,incident,resilience,manual}.<slug>
test.{governance,risk,personnel,vendor,incident,resilience}.<slug>
```

`control.resilience.*` / `evidence.resilience.*` / `test.resilience.*` in **this** slice are continuity-plan / DR **governance** only. Slugs must not collide with Prompt 07’s specified operational ids (`recovery-procedure`, `disaster-recovery-exercise`, `redundancy`, `recovery-objectives`, `recovery-evidence-freshness`, `recovery-plan`).

Reject in canonical governance content (validator + target suite):

- provider / GRC-product tokens in IDs or as the subject of a control (`vanta`, `drata`, `servicenow`, `jira`, `okta`, `entra`, `github`, `aws`, …);
- framework tokens in IDs or narrative (`iso27001`, `iso-27001`, `soc2`, `soc-2`, `nis2`, `dora`, `gdpr`);
- orphaned evidence types or tests;
- duplicate IDs;
- existence-only tests masquerading as freshness/population tests (a single PDF-present flag must not pass “all personnel trained”);
- `evidence.vulnerability.exception`, `evidence.secret.exposure`, `evidence.resilience.recovery-plan` created by this slice.

Correct: `control.governance.information-security-policy`. Incorrect: `control.vanta.ismp`, `control.iso27001.a.5.1`, `test.servicenow.incident-plan`.

### 4.3 Control family (34 independently assessable controls)

Stay in the 30–45 band. Do not split into micro-controls. Titles and objectives are framework-neutral. Almost all of these are **Manual** or **Hybrid**. Automated is reserved for honest freshness/presence of a required *typed* evidence record (still not “the PDF proves the ISMS works”).

| Control id | Title | Automation | Primary subjects | Required evidence (min) | Tests |
| --- | --- | --- | --- | --- | --- |
| `control.governance.information-security-policy` | Information-security policy | Hybrid | organization | `evidence.governance.policy` | `test.governance.policy-current` |
| `control.governance.policy-review` | Policy review cadence | Hybrid | organization | `evidence.governance.policy-review` | `test.governance.policy-review-current` |
| `control.governance.roles-and-responsibilities` | Security roles and responsibilities | Manual | organization | `evidence.manual.attestation` (+ policy/roles artifact) | `test.governance.roles-attested` |
| `control.governance.security-objectives` | Security objectives | Manual | organization | `evidence.manual.attestation` | `test.governance.objectives-attested` |
| `control.governance.documented-scope` | Documented assurance / ISMS scope | Hybrid | organization | `evidence.governance.policy`, `evidence.manual.attestation` | `test.governance.scope-documented` |
| `control.governance.internal-audit` | Internal audit | Hybrid | organization | `evidence.governance.internal-audit` | `test.governance.internal-audit-current` |
| `control.governance.management-review` | Management review | Hybrid | organization | `evidence.governance.management-review` | `test.governance.management-review-current` |
| `control.governance.corrective-action` | Corrective action / nonconformity handling | Hybrid | organization | `evidence.manual.attestation` | `test.governance.corrective-action-recorded` |
| `control.governance.continual-improvement` | Continual improvement | Manual | organization | `evidence.manual.attestation` | `test.governance.improvement-attested` |
| `control.governance.data-classification-policy` | Data classification policy | Hybrid | organization | `evidence.governance.policy` | `test.governance.classification-policy-current` |
| `control.governance.acceptable-use-policy` | Acceptable-use policy | Hybrid | organization | `evidence.governance.policy` | `test.governance.acceptable-use-current` |
| `control.governance.asset-ownership` | Asset ownership | Hybrid | organization / asset | `evidence.manual.attestation` | `test.governance.asset-ownership-attested` |
| `control.governance.document-control` | Document-control governance | Hybrid | organization | `evidence.manual.attestation` | `test.governance.document-control-attested` |
| `control.governance.evidence-retention` | Evidence collection / retention governance | Hybrid | organization | `evidence.manual.attestation` | `test.governance.retention-attested` |
| `control.governance.audit-program` | Internal audit program | Manual | organization | `evidence.governance.internal-audit`, `evidence.manual.attestation` | `test.governance.audit-program-attested` |
| `control.risk.assessment` | Organizational risk assessment | Hybrid | organization | `evidence.risk.assessment` | `test.risk.assessment-current` |
| `control.risk.treatment` | Risk treatment | Hybrid | organization | `evidence.risk.treatment` | `test.risk.treatment-current` |
| `control.risk.ownership` | Risk ownership | Hybrid | organization | `evidence.risk.assessment`, `evidence.manual.attestation` | `test.risk.owners-assigned` |
| `control.risk.acceptance` | Organizational risk-acceptance attestation | Manual | organization | `evidence.manual.attestation` + IR `Risk` | `test.risk.acceptance-attested` |
| `control.incident.response-plan` | Incident-response plan | Hybrid | organization | `evidence.incident.exercise` is **not** sufficient; plan artifact via `evidence.manual.attestation` | `test.incident.plan-current` |
| `control.incident.exercise` | Incident exercises / tabletops | Hybrid | organization | `evidence.incident.exercise` | `test.incident.exercise-current` |
| `control.incident.postmortem` | Incident postmortem / review | Hybrid | organization | `evidence.manual.attestation` | `test.incident.postmortem-recorded` |
| `control.personnel.security-awareness` | Security awareness | Hybrid | user / identity | `evidence.personnel.training` | `test.personnel.awareness-current-all` |
| `control.personnel.role-specific-training` | Role-specific training | Hybrid | user / identity | `evidence.personnel.training` | `test.personnel.training-current-all` |
| `control.personnel.onboarding-offboarding` | Onboarding / offboarding governance | Hybrid / manual | user | `evidence.manual.attestation` (process evidence, **not** MFA / role-membership) | `test.personnel.jml-process-attested` |
| `control.personnel.confidentiality-commitment` | Confidentiality commitments | Hybrid | user | `evidence.personnel.acknowledgement` | `test.personnel.confidentiality-acknowledged-all` |
| `control.personnel.policy-acknowledgement` | Policy acknowledgement | Hybrid | user | `evidence.personnel.acknowledgement` | `test.personnel.policy-acknowledged-all` |
| `control.vendor.inventory` | Supplier inventory | Hybrid | vendor | `evidence.vendor.inventory` | `test.vendor.inventory-authoritative` |
| `control.vendor.risk-review` | Supplier risk review | Hybrid | vendor | `evidence.vendor.risk-review` | `test.vendor.critical-risk-review-current` |
| `control.vendor.security-requirements` | Supplier security requirements | Manual | vendor | `evidence.manual.attestation` | `test.vendor.requirements-attested` |
| `control.vendor.reassessment` | Supplier reassessment | Hybrid | vendor | `evidence.vendor.risk-review` | `test.vendor.reassessment-current` |
| `control.vendor.cloud-governance` | Cloud / vendor governance | Manual | organization / vendor | `evidence.manual.attestation` | `test.vendor.cloud-governance-attested` |
| `control.resilience.business-continuity-plan` | Business continuity plan | Hybrid / manual | organization | `evidence.resilience.continuity-plan` | `test.resilience.continuity-plan-current` |
| `control.resilience.disaster-recovery-governance` | Disaster-recovery governance | Manual | organization | `evidence.resilience.continuity-plan`, `evidence.manual.attestation` | `test.resilience.dr-governance-attested` |

Each control record must carry: stable id, title, description/objective, domain(s) from existing `ControlDomain` (`Governance`, `PersonnelSecurity`, `SupplierManagement`, `IncidentResponse`, `Resilience`, `AssetManagement` as appropriate), evidence-requirement refs, test refs, and an honest automation class (`automated` \| `hybrid` \| `manual`).

**Do not invent technical automation** for management review quality, training effectiveness, supplier due-diligence quality, or BCP exercise quality. Those stay Hybrid or Manual even if a single document-present flag exists.

Sibling boundaries (hard):

| Topic | This slice | Not this slice |
| --- | --- | --- |
| MFA / privileged membership / account status | — | Prompt 04 `control.identity.*` |
| Onboarding/offboarding *process*, confidentiality, training, acknowledgements | here | — |
| Secure-development policy, change-management source controls | — | Prompt 05 `control.source.secure-development-policy` etc. |
| IS policy, AUP, document-control, evidence-retention | here | — |
| Finding-level risk acceptance / vuln exceptions | — | Prompt 06 |
| Organizational risk assessment / treatment / ownership / acceptance attestations | here | — |
| Operational backup/restore/HA, `evidence.resilience.recovery-plan` | — | Prompt 07 |
| BCP / DR *governance*, `evidence.resilience.continuity-plan` | here | — |

### 4.4 Canonical evidence (facts, not conclusions)

Prefer a **shared attestation shape** on `evidence.manual.attestation`, then domain types that add fields. All retain: principal/author, timestamp, subject, artifact reference where relevant, freshness/validity, and review state.

Shared attestation facts (canonical names; store via `EvidenceValue::with_value`):

| Fact | Type | Notes |
| --- | --- | --- |
| `subject_id` | String | Org, user, or vendor id |
| `attested_by` | String | Principal / author (never a secret) |
| `attested_at` | Timestamp | When the attestation was made |
| `kind` | String | `attestation` \| `document-reference` \| `approval` \| `meeting-record` \| `auditor-observation` \| `training-record` \| `exercise-record` \| `risk-acceptance` \| `policy-acknowledgement` |
| `artifact_ref` | String? | Locator / digest id — **not** file bytes |
| `review_state` | String | `draft` \| `submitted` \| `reviewed` \| `accepted` \| `rejected` |
| `valid_until` | Timestamp? | Explicit validity; else catalog freshness window |
| `current` | Bool? | Collector-derived “inside window” is allowed; evaluator must still apply `FreshWithin` / temporal fields |

| Evidence type | Additional facts | Not allowed |
| --- | --- | --- |
| `evidence.manual.attestation` | shared shape | “control effective”, compliance sentences |
| `evidence.governance.policy` | `policy_kind` (`information-security` \| `acceptable-use` \| `classification` \| `other`), `version?`, `reviewed_at` | “ISMS certified” |
| `evidence.governance.policy-review` | `reviewed_at`, `reviewer_id?` | “review effective” |
| `evidence.governance.management-review` | `reviewed_at`, `period?` | “management review passed” |
| `evidence.governance.internal-audit` | `audited_at`, `auditor_id?` | “audit passed” |
| `evidence.risk.assessment` | `assessed_at`, `owner_id?` | residual-risk scores as compliance |
| `evidence.risk.treatment` | `treated_at`, `owner_id?` | “risks closed / certified” |
| `evidence.personnel.training` | `trained_at`, `training_kind` (`awareness` \| `role-specific`), `current` | exam scores / PII dumps |
| `evidence.personnel.acknowledgement` | `acknowledged_at`, `ack_kind` (`confidentiality` \| `policy`) | legal conclusions |
| `evidence.vendor.inventory` | `critical` (bool), `authoritative?` | provider SKU dumps as type id |
| `evidence.vendor.risk-review` | `reviewed_at`, `critical` | “supplier approved / certified” |
| `evidence.incident.exercise` | `exercised_at`, `exercise_kind` (`tabletop` \| `walkthrough` \| `simulation`) | “IR capability proven” |
| `evidence.resilience.continuity-plan` | `reviewed_at`, `plan_kind` (`bcp` \| `dr-governance`) | operational restore results (Prompt 07) |

Seal rules still apply: no credential-shaped keys; no compliance narratives. Additional supporting types may be added only if referenced by a control and a test (no orphans). Prefer extending facts on the types above.

Envelope `evidenceType` in fixtures and `TestExpr` selectors uses the **catalog id** (`evidence.governance.policy`), matching the IAM fixture convention.

### 4.5 Tests (freshness / population / manual-review — not existence of a PDF)

Required reusable tests (Prompt 08 list + extras so no control is untested):

```text
test.governance.policy-current
test.governance.management-review-current
test.governance.internal-audit-current
test.personnel.training-current-all
test.vendor.critical-risk-review-current
test.incident.exercise-current
```

Semantics (authoritative intent; exact `TestExpr` spelling follows Prompt 03):

| Test | Population | Pass | Fail | Missing | Stale | Manual / exception |
| --- | --- | --- | --- | --- | --- | --- |
| `policy-current` | in-scope organization | required policy envelope exists **and** `reviewed_at` / `valid_until` inside window | policy present but review failed / rejected | no policy envelope | `reviewed_at` outside window → `StaleEvidence` | Hybrid: document-present without review metadata → `InsufficientEvidence` or `ManualReviewRequired`, never Effective |
| `management-review-current` | organization | management-review record inside policy period | review rejected | no review envelope | stale `reviewed_at` | — |
| `internal-audit-current` | organization | internal-audit record inside window | — | missing | stale | quality of audit remains Manual/Hybrid |
| `training-current-all` | all required personnel (authoritative user/identity inventory via Prompt 03 generic inventory or explicit fixture population) | every subject has current `evidence.personnel.training` | subject with `current=false` | known person lacks training envelope | stale `trained_at` | Partial/unknown population → `InsufficientEvidence`, **never Effective** |
| `critical-risk-review-current` | all vendors with `critical=true` | each has current `evidence.vendor.risk-review` | critical vendor review failed / `current=false` | critical vendor missing review | stale `reviewed_at` | Partial vendor inventory → `InsufficientEvidence` |
| `exercise-current` | organization | `exercised_at` inside required window | — | missing | stale | A plan PDF without an exercise record does **not** pass this test |

**Forbidden encoding:** `Exists(evidence.governance.policy)` as the body of `test.governance.policy-current` when the control requires a review window. Existence of some policy file is not a current policy.

**Forbidden encoding:** `Exists(evidence.personnel.training)` as the body of `test.personnel.training-current-all`. One training record is not the population.

Unknown / non-authoritative personnel or vendor population **must not** produce `Effective` for an all-subjects test. Missing evidence ⇒ `InsufficientEvidence`, not a technical `Ineffective` invented from an empty set.

Hybrid/manual controls that require operational evidence stay Hybrid/Manual and **cannot auto-pass from a single document-present flag**. Use `op = "manual-review"` (and/or `fresh-within` on a supporting record) so `TestExpr::ManualReview` yields `ManualReviewRequired` until a complete attestation exists.

Result metadata (Prompt 03 `PopulationEvaluation`) must be sufficient to explain: population size, evaluated, passing, failing, missing, coverage, failing/missing/stale/excepted subject ids.

### 4.6 Manual / hybrid honesty

| Control | Why not fully automated |
| --- | --- |
| Information-security policy / AUP / classification | Text quality, approval authority, and communication are organizational. Freshness of a versioned record is the automatable slice. |
| Management review / internal audit | Cadence can be dated; independence and coverage cannot be inferred from a PDF. |
| Corrective action / continual improvement | Require attested nonconformity handling, not a ticket-id existence check. |
| Training / acknowledgements | Completeness of the *population* is automatable given inventory; effectiveness of learning is not. |
| Onboarding/offboarding governance | Process evidence (checklists, confidentiality, manager attestation). Technical disablement is Prompt 04. |
| Supplier risk review | Review existence + freshness + critical coverage. Diligence quality is manual. |
| Incident exercise | Date of a tabletop is automatable; “IR works” is not. |
| BCP / DR governance | Plan + governance attestation. Operational restore tests are Prompt 07. |
| Risk acceptance | Attestation + IR `Risk` status. Finding-level exceptions are Prompt 06. |

Do not add a synthetic collector that auto-passes these controls.

### 4.7 Fixtures (deterministic)

Each fixture is a frozen evidence set (+ optional Exception / Risk records) with a fixed `collectedAt`. Expected effectiveness is part of the target suite.

| Fixture | Intent | Expected highlights |
| --- | --- | --- |
| `current-documents` | Authoritative org + personnel + critical vendors; current policy, reviews, audit, training, vendor reviews, exercise, BCP plan | Freshness tests `Effective` **only** where typed current records exist. Hybrid/manual quality tests `ManualReviewRequired` / `InsufficientEvidence` unless attestations are present — document the fixture’s attestation choice and keep it deterministic. |
| `stale-documents` | Policy / management-review / audit / exercise envelopes exist but `reviewed_at` / `exercised_at` outside window | Matching tests → `StaleEvidence`, not Ineffective-as-missing and not Effective. |
| `missing-documents` | Authoritative org, no policy / review / audit / exercise envelopes | Matching tests → `InsufficientEvidence`. |
| `incomplete-training-population` | Authoritative personnel inventory of N; training envelopes for N−1 | `training-current-all` → **not** Effective (`InsufficientEvidence` if missing, `Ineffective` only if a subject has explicit `current=false`). Partial population cannot be Effective. |
| `vendor-review-gaps` | Authoritative vendor inventory; one critical vendor lacks a current risk review | `critical-risk-review-current` → not Effective. |
| `approved-exception` | Named subject (e.g. one vendor or one person) lacks required current evidence; IR `Exception` `status=Approved`, unexpired, **bound** to that subject and control | Bound subject is excepted. Overall **must not** be silent `Effective`. Prefer `ExceptionApproved`. |
| `expired-exception` | Same gap; exception `status=Expired` or `expiresAt` in the past | Failing / missing result is **not** suppressed. |
| `manual-review-despite-evidence` | Supporting document/attestation envelope exists; control still requires operational review (`op = "manual-review"`) | `ManualReviewRequired`. Document-present must not auto-pass. |

Fixtures emit canonical `evidence.{governance,risk,personnel,vendor,incident,resilience,manual}.*` only. No `policy.security.reviewed` pack types. No GRC-product type ids. No secret material.

Personnel/vendor populations use Prompt 03 **generic** `inventory.subject` + `inventory.complete` (and/or explicit `EvidenceSet` population), not a new resolver.

### 4.8 Integration rules (consume, do not redesign)

- Loader / validate / digest: Prompt 01 `CanonicalCatalog`. Governance files must pass `validate` (no orphans, no provider/framework tokens, deterministic digest).
- Typed facts: `EvidenceValue::with_value`. Do not fork a second enum. Do not put secrets in facts.
- Population evaluation: Prompt 03 `evaluate_coverage` / `FreshWithin` / `ManualReview`. Governance tests are **declarations**.
- Personnel / vendor inventory: generic `inventory.subject` + `inventory.complete` (or explicit fixture population). **Do not** add `resolve_personnel_inventory`.
- Exception: reuse IR `Exception` + existing `Effectiveness::ExceptionApproved`. Bind `subjects`. Empty subjects ≠ whole inventory. Expired/revoked must not pass.
- **Honesty (shipped):** `evaluate_coverage` still (a) removes excepted subjects from the denominator and (b) promotes `Ineffective` → `ExceptionApproved` for identity break-glass. It additionally promotes remaining-all-pass `Effective` → `ExceptionApproved` when `excepted` is non-empty and `failing` / `missing` / `stale` / `technical` are empty (rationale: `approved unexpired exception bound to excepted subjects; not silent Effective`). Same IR `Exception`; not a second engine and not a `GovernanceException` type. Coverage arithmetic is otherwise unchanged.
- Risk: reuse IR `Risk` as an attestation subject / record. Do not build a treatment workflow.
- ISO pack, GitHub collector, Prompt 04–07 families, generic `TestExpr` semantics: **untouched** except the ExceptionApproved honesty promotion above. This slice does not remap ISO; organizational clauses stay unmapped after Prompt 12 sliver retirement.

### 4.9 Dual-suite protocol

Follow the existing root `[[test]]` pattern. `tests/sdd` is **not** autodiscovered.

| Suite | Path (planned) | Role |
| --- | --- | --- |
| Baseline | `tests/sdd/governance_catalog.baseline.rs` · `sdd_governance_catalog_baseline` | GREEN on **current** product tree (no governance family). After target GREEN: `#[ignore = "superseded by sdd_governance_catalog_target"]`. |
| Target | `tests/sdd/governance_catalog.target.rs` · `sdd_governance_catalog_target` | RED on current tree for the **right** reason; then the CI gate. |

**I4a / xylex AC-2 trap:** never write a target test that reads **its own source file** and asserts it does not contain a substring that appears in the assertion (e.g. reading `governance_catalog.target.rs` and asserting it lacks `vanta` while the assertion string contains `vanta`). Grep **catalog TOML and product crates**, not the test file.

Suggested target assertion clusters (titles include the id):

| ID | Asserts |
| --- | --- |
| GOV-001 | Catalog tree / `CanonicalCatalog::load` loads governance content offline |
| GOV-002 | Digest of the catalog (with governance files listed) is deterministic |
| GOV-003 | All 34 `control.{governance,risk,personnel,vendor,incident,resilience}` ids present; prefixes only as specified; count in 30–45 |
| GOV-004 | Required evidence types declared including `evidence.manual.attestation` and the Prompt-08 domain types; no orphans |
| GOV-005 | Required tests declared and referenced (`policy-current`, `training-current-all`, `critical-risk-review-current`, `management-review-current`, `internal-audit-current`, `exercise-current`) |
| GOV-006 | Catalog IDs contain no provider/GRC-product tokens; target greps `vanta` / `drata` / `servicenow` / `jira` in **catalog TOML**, not in this test’s own source |
| GOV-007 | Catalog TOML / ids contain no `iso27001` / `soc2` / `nis2` tokens |
| GOV-008 | No governance control lives in the ISO pack as `control.governance.*`; this slice does not retarget ISO mappings (organizational clauses stay unmapped after Prompt 12 sliver retirement) |
| GOV-009 | `training-current-all` is population-based (incomplete-training fixture not Effective; a single training envelope does not pass) |
| GOV-010 | Missing vs stale vs fail vs manual vs exception distinguished on the eight fixtures |
| GOV-011 | Partial training / vendor populations cannot yield Effective on all-subjects tests |
| GOV-012 | Approved unexpired IR exception → `ExceptionApproved` (or at minimum **not** silent Effective) for the bound subject; expired does not suppress fail |
| GOV-013 | Hybrid/manual controls requiring operational evidence stay hybrid/manual; `manual-review-despite-evidence` is `ManualReviewRequired` |
| GOV-014 | Credential-shaped facts still rejected; no secret material in governance fixtures |
| GOV-015 | IAM / fixture.example ids still present; no Prompt 05/06/07 files overwritten by this slice |
| GOV-016 | `sdd_iso27001_assurance_target` / remap and `sdd_iam_catalog_target` stay green (this slice does not retarget ISO; IAM untouched) |

Baseline clusters (current tree; stay true until superseded):

- `manifest.toml` lists only fixture.example + identity (no governance/risk/personnel/vendor/incident family files)
- `CanonicalCatalog::{load,validate,digest}` and `EvidenceValue::with_value` exist
- IAM family + `fixtures/assurance/canonical/v1/identity/*` exist
- no `control.governance.*` / `evidence.manual.attestation` in loaded catalog
- no `fixtures/assurance/canonical/v1/governance/`
- ISO sliver ids listed in §3.3 still present
- `manual_attestation` is capability/legacy type, not catalog `evidence.manual.attestation`

After target GREEN: `#[ignore = "superseded by sdd_governance_catalog_target"]` on the baseline (or delete). Target remains the gate.

### 4.10 Documentation after implement

- This file: status → Implemented; record landed SHA, digest, file list.
- ADR: Draft → Accepted (same path; no rename required).
- [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md): add a governance-family pointer and `evidence.manual.attestation` **only if** the public contract would otherwise omit those families. Do not rewrite generic runtime sections.
- Prompt 01 SSOT: pointer-only at most. Do not overwrite.
- Do not overwrite IAM / SDLC / vuln / infra SSOTs.

---

## 5. Acceptance criteria

Testable. Implementation is out of this spec phase.

1. Dual-suite `sdd_governance_catalog_baseline` + `sdd_governance_catalog_target` is registered in root `Cargo.toml` (tests/sdd is not autodiscovered).
2. On current tree (pre-governance TOML): baseline GREEN characterizing §3; target RED for missing `control.governance.*` / `evidence.manual.attestation` / population fixtures — not for unrelated compile errors.
3. After implement: target GREEN; baseline ignored so absence-of-governance-catalog is not a CI requirement; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
4. Thirty-four `control.{governance,risk,personnel,vendor,incident,resilience}` controls exist with stable ids, domains, evidence requirements, test refs, and honest automation class; count stays in 30–45 with no artificial micro-controls.
5. Evidence types include `evidence.manual.attestation` and `evidence.governance.{policy,policy-review,management-review,internal-audit}`, `evidence.risk.{assessment,treatment}`, `evidence.personnel.{training,acknowledgement}`, `evidence.vendor.{inventory,risk-review}`, `evidence.incident.exercise`, `evidence.resilience.continuity-plan`, declared as facts (shared attestation shape), not conclusions.
6. Tests include the six Prompt-08 scenarios and evaluate freshness / populations / manual-review, not existence of one PDF.
7. Evaluator outcomes distinguish missing, stale, actual failure, manual review, and approved exception on the eight named fixtures.
8. Hybrid/manual controls requiring operational evidence stay Hybrid or Manual; they cannot auto-pass from a document-present flag. `manual-review-despite-evidence` is `ManualReviewRequired`.
9. Partial training or vendor populations cannot be `Effective` on all-subjects tests. Missing evidence ⇒ `InsufficientEvidence`.
10. Approved unexpired IR exceptions are `ExceptionApproved` for the bound subject (never silent `Effective`). Expired exceptions do not suppress failing results.
11. Catalog validator accepts the slice: no duplicate/orphan/dangling ids; no provider / GRC-product / framework tokens in canonical governance IDs or TOML narrative (target greps `vanta`/`drata`/`servicenow`/`jira`/`iso`/`soc2`/`nis2` in catalog files, **not** in the target test’s own source).
12. This slice does not retarget ISO mappings; `sdd_iso27001_assurance_target` / remap remain green. IAM family remains green. Fixture.example IDs remain. Organizational ISO clauses stay unmapped.
13. No second `CanonicalCatalog` loader, no second `EvidenceValue`, no local personnel/vendor population fork, no second Exception/Risk engine.
14. No GRC product, document editor, ISO remap, certification language, or Prompt 05/06/07 collectors/families implemented here. Sibling TOML (`vulnerability.toml`, `resilience.toml`, network/crypto/…) is not overwritten. Continuity IDs live in `governance.toml`.
15. Prompt 01 SSOT path `docs/sdd/canonical-assurance-catalog-v1.md` is not overwritten (pointer-only). IAM / SDLC / vuln / infra SSOTs are not overwritten.

---

## 6. Out of scope

- A full GRC workflow / SaaS product, ticketing, or document editors.
- Remapping ISO 27001 (or SOC 2 / NIS 2) onto `control.governance.*` (Prompt 12).
- Growing or retargeting `frameworks/iso-27001/2022` organizational slivers.
- Redesign of `CanonicalCatalog` loader/validator/digest (Prompt 01).
- Redesign of typed evidence (Prompt 02) or reimplementing coverage math (Prompt 03), except the shipped ExceptionApproved honesty promotion in §4.8.
- Prompt 04 IAM technical identity/MFA/role-membership content.
- Prompt 05 SDLC / change-management technical evidence.
- Prompt 06 vulnerability finding-level risk acceptance / `evidence.secret.exposure`.
- Prompt 07 operational backup/restore/HA / `evidence.resilience.recovery-plan` / `resilience.toml`.
- Collectors for ServiceNow, Jira, Vanta, Drata, or any GRC/ITSM product.
- Certification, “compliant”, or audit-passed language.
- Deleting fixture.example IDs or overwriting Prompt 01 / IAM / sibling SSOTs.

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Silent `Effective` when excepted subjects leave the denominator | AC 10; §4.8 shipped ExceptionApproved promotion; expired fixtures must still fail. |
| Existence of a PDF treated as operational effectiveness | GOV-013; `manual-review-despite-evidence`; forbidden Exists-as-population encodings. |
| Partial training/vendor inventory auto-passes | GOV-009/011; consume Prompt 03 Partial/Unknown refusal. |
| GRC-product tokens leak; validator omits them | Target greps catalog TOML; do not expand Prompt 01 denylist unless a documented one-line add is needed. |
| I4a self-grep trap (`vanta` in the assertion string) | Grep catalog files only; never the target source for “must not contain X”. |
| Collision with Prompt 07 `resilience.toml` / `control.resilience.*` slugs | No `resilience.toml` here; reserved operational slugs listed in §4.2. |
| Collision with Prompt 05 secure-development policy / Prompt 06 vuln exceptions | Distinct IDs and evidence; do not implement those families. |
| ISO pack rewritten or ISO suite broken | AC 12; do not touch pack metadata/mappings. |
| Second loader / EvidenceValue / population / Exception engine | AC 13; fail-closed. |
| Baseline remains required-green absence-of-family after target lands | `#[ignore = "superseded by sdd_governance_catalog_target"]`. |
| Secrets or compliance narratives in governance fixtures | Seal + GOV-014. |
| Parallel Prompt 05–07 product overwrites | Add-only files; rebase if siblings land first. |

---

## 8. Dual-suite and SDD protocol (implement phase)

Hard protocol (do not skip):

```text
Spec + draft ADR (this phase; no product TOML)
  → Register [[test]] rows + write suites
  → Baseline GREEN on CURRENT product tree
  → Target RED for missing control.governance.* / evidence.manual.attestation / population fixtures
  → Implement catalog + fixtures + docs/ADR accept + contract pointer if needed
  → Target GREEN
  → Prove baseline FAILS or is additive-documented
  → #[ignore = "superseded by sdd_governance_catalog_target"]
  → Target still GREEN
  → cargo test --workspace --features demo
  → cargo fmt --all -- --check
  → cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Fail-closed if: baseline cannot go green on current characterization; target cannot go red for the **right** reason; or target never greens within max_iters.

---

## 9. ADR

Architecture / public-contract decision: governance / vendor / personnel / incident / continuity-governance content is a **canonical catalog family** with **first-class manual evidence**, not an ISO-pack extension, not a GRC-product integration, and not fake technical automation of organizational controls.

Accepted: [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md).

---

## 10. Characterization SHA record

```text
characterization_sha = e430980c0d27a8138a153d49b62ddf3c57827891
branch               = main
note                 = Prompts 01–04 landed (fixture.example + identity);
                       no governance family TOML or fixtures;
                       ISO organizational sliver present;
                       Prompts 05–07 specified in parallel, product unlanded
```

---

## 11. Baseline suite record (to fill at implement)

| Field | Value |
| --- | --- |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Suite | `sdd_governance_catalog_baseline` · `tests/sdd/governance_catalog.baseline.rs` |
| Expected on current tree | **ignored** (`#[ignore = "superseded by sdd_governance_catalog_target"]`) |
| After target GREEN | `#[ignore = "superseded by sdd_governance_catalog_target"]` |
| Command | `cargo test --workspace --features demo --test sdd_governance_catalog_baseline` |

---

## 12. Target suite record (to fill at implement)

| Field | Value |
| --- | --- |
| Suite | `sdd_governance_catalog_target` · `tests/sdd/governance_catalog.target.rs` |
| Expected on current tree | **RED** — missing `control.governance.*` / `evidence.manual.attestation` / governance fixtures |
| Expected after implement | **GREEN** (CI gate; met) |
| Command | `cargo test --workspace --features demo --test sdd_governance_catalog_target` |

---

## 13. Landed record

| Surface | Location |
| --- | --- |
| Controls | `catalog/canonical/v1/controls/governance.toml` — 34 ids (25 Hybrid / 9 Manual); no split files |
| Evidence | `catalog/canonical/v1/evidence/governance.toml` — 13 types including `evidence.manual.attestation` |
| Tests | `catalog/canonical/v1/tests/governance.toml` — 34 tests (fresh-within / all-subjects / manual-review) |
| Manifest listing | `catalog/canonical/v1/manifest.toml` `[files]` (`controls/governance.toml` and siblings) |
| Fixtures | `fixtures/assurance/canonical/v1/governance/<name>/` (eight sets; clock `2026-08-18T12:00:00Z`) |
| Loader / digest | Prompt 01 crate; no governance-specific load path |
| Exception honesty | `evaluate_coverage` promotes remaining-all-pass excepted sets to `ExceptionApproved` |
| Target suite | `tests/sdd/governance_catalog.target.rs` GREEN GOV-001…016 |
| Baseline suite | `tests/sdd/governance_catalog.baseline.rs` superseded (`#[ignore]`) |
| ADR | Accepted [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md) |
| ISO pack | Organizational slivers already retired by Prompt 12; this slice does not map `control.governance.*` onto ISO |
| Collectors | No GRC/ITSM collector; `ManualEvidence` still does not emit `evidence.manual.attestation` |

Protocol:

```text
Spec (this file) + draft ADR
  → Baseline GREEN on current characterization
  → Target RED for missing governance family / manual attestation / fixtures
  → Implement TOML + fixtures
  → Target GREEN → Baseline skip-superseded → Target still GREEN
```
