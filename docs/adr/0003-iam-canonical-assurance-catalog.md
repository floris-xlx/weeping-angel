# ADR 0003 — IAM family in the canonical assurance catalog

| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md). Does **not** replace [ADR 0002](0002-iso-27001-assurance-vertical.md). ISO sliver collapse is [ADR 0003 remap](0003-iso27001-canonical-remap.md). |
| Extends | [Catalog infrastructure](0003-canonical-assurance-catalog-v1.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [population / coverage](0003-subject-population-runtime-and-coverage-semantics.md) |
| Spec | [`docs/specs/iam-canonical-assurance-catalog.md`](../specs/iam-canonical-assurance-catalog.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Planning baseline | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |
| Tests | `sdd_iam_catalog_target` GREEN (IAM-001…016). Absence-characterization baseline `sdd_iam_catalog_baseline` superseded / `#[ignore]`. ISO remap of identity slivers is [ADR 0003 remap](0003-iso27001-canonical-remap.md). |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0001 delivered the inwardly extensible assurance spine. ADR 0002 shipped the first ISO 27001 vertical, including a **thin IAM sliver inside the ISO pack** (`access.mfa.privileged`, `access.least-privilege`, `access.periodic-review`, `personnel.access-termination`) tested mostly as presence/hybrid checks on GitHub-shaped evidence (`source.admin.permissions`, `source.collaborator.permission`).

Canonical catalog infrastructure, typed evidence, and subject-population coverage (population runtime) landed as sibling ADRs. They provide the loader/validator/digest, fact encoding, and `AllSubjects` / `CoverageAtLeast` / `NoneSubjects` runtime. They do not own identity-domain content.

Without a provider-neutral IAM family, a future Entra / Okta / Google Workspace collector has nowhere canonical to emit facts, and “all privileged identities have MFA” cannot be declared as a catalog test.

Questions this decision answers:

1. Where do identity, MFA, privileged-access, and lifecycle controls live, if not in `frameworks/iso-27001/2022/metadata.toml`?
2. What public ID contract do future identity collectors and ISO remap ISO remapping consume?
3. Are IAM tests existence checks or subject-population assertions?
4. How are governance-heavy controls (approval, SoD, periodic review) marked without fake automation?
5. Do we fork the catalog loader, evidence values, or population evaluator for this family?

## Decision

This is what shipped.

### 1. IAM is canonical catalog content, not a pack and not a collector

Independently assessable IAM controls live in the catalog infrastructure tree:

```text
catalog/canonical/v1/controls/identity.toml
catalog/canonical/v1/evidence/identity.toml
catalog/canonical/v1/tests/identity.toml
```

Listed in `catalog/canonical/v1/manifest.toml` `[files]`. Loaded by `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` — **no second loader**.

Public IDs:

```text
control.identity.<slug>
evidence.identity.<slug>
test.identity.<slug>
```

Incorrect: `control.okta.mfa`, `control.entra.admin-mfa`, `control.iso27001.a.8.5`, or growing the ISO pack `access.*` list as the long-term IAM library.

Provider details belong only in future collectors that **emit** `evidence.identity.*` facts. Framework details belong only in later mappings (ISO remap).

### 2. Twenty-three provider-neutral controls

Shipped family (20–30 independently assessable; no micro-controls):

| Control | Automation |
| --- | --- |
| `control.identity.unique-user-identities` | automated |
| `control.identity.mfa` | automated |
| `control.identity.privileged-mfa` | automated |
| `control.identity.strong-authentication-policy` | hybrid |
| `control.identity.privileged-inventory` | automated |
| `control.identity.least-privilege` | hybrid |
| `control.identity.privileged-access-minimization` | hybrid |
| `control.identity.access-approval` | hybrid |
| `control.identity.periodic-access-review` | hybrid |
| `control.identity.inactive-account-lifecycle` | automated |
| `control.identity.terminated-user-removal` | automated |
| `control.identity.joiner-mover-leaver` | hybrid |
| `control.identity.service-account-inventory` | automated |
| `control.identity.service-account-ownership` | automated |
| `control.identity.service-account-credential-governance` | hybrid |
| `control.identity.break-glass-access` | hybrid |
| `control.identity.shared-account-restriction` | automated |
| `control.identity.credential-management` | hybrid |
| `control.identity.privileged-role-change-monitoring` | hybrid |
| `control.identity.external-guest-access` | automated |
| `control.identity.stale-privileged-membership` | automated |
| `control.identity.access-revocation-timeliness` | hybrid |
| `control.identity.segregation-of-duties` | **manual** |

Each control has stable id, domain(s), evidence requirements, and test refs. Validator rejects provider/framework segments in catalog IDs. Canonical IAM TOML contains no ISO/SOC2/NIS2/DORA/GDPR tokens.

### 3. Evidence types are facts, not conclusions

Declared in `evidence/identity.toml` (catalog id → envelope `evidenceType`):

```text
evidence.identity.inventory
evidence.identity.authentication-state
evidence.identity.mfa-status
evidence.identity.privileged-membership
evidence.identity.role-membership
evidence.identity.last-active
evidence.identity.account-status
evidence.identity.account-owner
evidence.identity.access-review
evidence.identity.lifecycle-event
evidence.identity.service-account
evidence.identity.external-access
```

Fixtures emit these types. No `source.admin.permissions` in IAM fixtures. Credential-shaped keys and compliance narratives remain rejected at seal (typed evidence).

### 4. Tests are population predicates

`test.identity.privileged-mfa-enabled` means **all in-scope privileged identities have MFA**, using population runtime arms (`coverage-at-least` 100%, `all-subjects`, `none-subjects`). It does not mean “some `mfa-status` envelope exists.”

Required reusable tests:

```text
test.identity.mfa-enabled
test.identity.privileged-mfa-enabled
test.identity.no-inactive-privileged-accounts
test.identity.no-terminated-active-accounts
test.identity.all-service-accounts-have-owner
test.identity.access-review-current
test.identity.no-unapproved-guest-access
```

Each of the 23 controls has a test. Hybrid/manual tests use `op = "manual-review"`. Results distinguish missing evidence, stale evidence, technical failure, manual review, and approved exception. Unknown/partial inventory must not yield `Effective` on all-subjects tests.

This slice **declares** those tests. It does not reimplement `AllSubjects` / `CoverageAtLeast` (no `IamPopulation` fork). Population resolution already consumes `evidence.identity.inventory` (plus privileged-membership / service-account) in the population runtime runtime.

### 5. Hybrid/manual controls stay honest

Access approval, segregation of duties, and periodic access review are Hybrid or Manual. A single technical signal must not auto-pass them. Break-glass uses existing IR `Exception` and `Effectiveness::ExceptionApproved` for an approved, unexpired, subject-scoped exception — not a second exception system.

### 6. ISO sliver coexistence (ISO remap remapped)

This slice **did not** retarget ISO mappings. Two libraries coexisted until ISO remap.

**Later:** [ADR 0003 remap](0003-iso27001-canonical-remap.md) collapsed ISO A.8.5 / A.8.2 / A.8.3 / A.5.15 / A.5.18 / A.5.16 / A.6.5 onto `control.identity.*` and retired pack slivers `access.mfa.privileged` / siblings. See [`docs/specs/iso-27001-canonical-remap.md`](../specs/iso-27001-canonical-remap.md) §13.

### 7. Deterministic fixtures

Eight frozen evidence sets under `fixtures/assurance/canonical/v1/identity/`:

| Fixture | Distinguishes |
| --- | --- |
| `healthy-org` | Authoritative population, automated tests can pass |
| `privileged-without-mfa` | Privileged MFA **Ineffective** (named subject), not existence-pass |
| `inactive-admin-active` | Inactive privileged still active → **Ineffective** |
| `terminated-employee-active` | Left/terminated still active → **Ineffective** |
| `service-account-without-owner` | Ownerless SA → **Ineffective** |
| `partial-inventory` | Non-authoritative inventory → **InsufficientEvidence**, never Effective |
| `stale-access-review` | Review outside freshness → **StaleEvidence** |
| `break-glass-approved-exception` | Approved unexpired exception → **ExceptionApproved** for that subject |

### 8. Consume catalog infrastructure, typed evidence, and population runtime; do not fork infrastructure

No second catalog loader, typed `EvidenceValue`, or population evaluator. catalog infrastructure’s SSOT (`docs/specs/canonical-assurance-catalog-v1.md`) is not overwritten. No Entra / Okta / Google Workspace / GitHub-identity **IdP** collector. Later: [GitHub collector mapping](0003-github-collector-canonical-evidence-mapping.md) emits `evidence.identity.privileged-membership` / `external-access` from GitHub membership APIs; this family still does not own MFA / last-active / termination facts.

Catalog stats after this family (including the catalog infrastructure protected-branch fixture): 24 controls, 13 evidence types, 24 tests. Digest is catalog infrastructure `CatalogDigest` over parsed documents.

## Consequences

**Positive**

- Future identity collectors have a stable emit contract (`evidence.identity.*`).
- ISO remap can map `iso27001:a.8.5` → `control.identity.privileged-mfa` (and siblings) without rewriting collectors.
- Population tests are explainable (failing/missing/stale/excepted subjects) using the population runtime evaluation object.

**Negative / cost**

- Historical sliver IDs may still appear on pre-remap snapshots; live ISO mappings target `control.identity.*` ([ADR 0003 remap](0003-iso27001-canonical-remap.md)).
- Hybrid/manual IAM tests will not auto-pass from technical facts alone; assessments need attestations for those controls.
- Identity collectors are still future work; catalog evaluation of live IdP populations is fixture-only until GitHub collector and later collector slices siblings.

**Rejected**

- Provider-prefixed control IDs.
- Encoding MFA as `Exists(evidence.identity.mfa-status)`.
- Completing `CoverageAtLeast` inside this slice.
- Rewriting ISO `metadata.toml` / `mappings.toml`.
- Inventing a second exception engine.

## Non-goals (reaffirmed)

Entra / Okta / Google Workspace / AD / Cognito collectors; ISO / SOC 2 / NIS 2 / DORA text or mappings; generic rule-engine expansion; PAM / IGA product integrations; certification language; SDLC / vulnerability / infrastructure / governance catalog families (SDLC, vulnerability, infrastructure, and governance families).

## Access and security

- Catalog load remains local-filesystem only.
- IAM fixtures store booleans, timestamps, and subject ids — never tokens, passwords, or recovered secrets.
- Seal still rejects credential-shaped fact keys and compliance narratives.

## Related

- Spec SSOT: [`docs/specs/iam-canonical-assurance-catalog.md`](../specs/iam-canonical-assurance-catalog.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Catalog infrastructure: [`0003-canonical-assurance-catalog-v1.md`](0003-canonical-assurance-catalog-v1.md)
- Typed evidence: [`0003-typed-evidence-canonical-serialization.md`](0003-typed-evidence-canonical-serialization.md)
- Population runtime: [`0003-subject-population-runtime-and-coverage-semantics.md`](0003-subject-population-runtime-and-coverage-semantics.md)
- ISO vertical (sliver frozen): [`0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
