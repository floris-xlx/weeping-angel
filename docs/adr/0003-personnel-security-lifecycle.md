# ADR 0003 — Personnel security is a lifecycle population, not an HRIS

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_personnel_security_target` GREEN (17); baseline skip-superseded (12 ignored). |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** the governance personnel sliver (`control.personnel.*` in `governance.toml`) with population-honest joiner / mover / leaver tests. Does **not** replace [IAM catalog](0003-iam-canonical-assurance-catalog.md) technical JML or [governance catalog](0003-governance-canonical-assurance-catalog.md). |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [population / coverage](0003-subject-population-runtime-and-coverage-semantics.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [catalog infrastructure](0003-canonical-assurance-catalog-v1.md) |
| Spec | [`docs/specs/personnel-security.md`](../specs/personnel-security.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) (personnel lifecycle pointer) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | Dual-suite `sdd_personnel_security_baseline` / `sdd_personnel_security_target` at `tests/contracts/personnel_security.{baseline,target}.rs`, registered in root `Cargo.toml`. Neighbors `sdd_iam_catalog_target`, `sdd_governance_catalog_target`, `sdd_population_runtime_target` stay GREEN. |

> Filename `0003-*` is shared with catalog-program / operational-slice siblings. **0004** is documentation architecture. Cite this decision by **path**.

## Context

On SHA `6e31bf1` personnel security exists as five governance-catalog controls:

- four all-subjects `current` tests on `evidence.personnel.{training,acknowledgement}`
- one manual `test.personnel.jml-process-attested`

IAM already ships technical JML (`control.identity.joiner-mover-leaver`, `terminated-user-removal`, `access-revocation-timeliness`) with `evidence.identity.lifecycle-event`. Those tests are hybrid/manual-review or `none-subjects` on the string field `status` — not population-honest joiner / mover / leaver lifecycle.

`Identity` is `{ id, kind, displayName }` with no employment class. `Exception` already binds `subjects` and `expiresAt`. Collectors are GitHub + local only. Prompt 10 CIR expansion is out of this slice. Prompt 12 document types may exist as a sibling; this slice does not implement the registry. Prompt 15 events and Prompt 16 remediations may still be in flight.

Operational ISMS v1 Prompt 17 requires continuous, honest testing of personnel-security lifecycle controls across complete in-scope populations — without turning Weeping Angel into an HRIS.

Questions this decision answers:

1. Do we extend `Identity` with Employee / Contractor kinds?
2. Do we add `resolve_personnel_inventory`?
3. Do we retarget IAM JML tests, or compose their facts?
4. Are grace windows IR exceptions?
5. May collectors emit personnel-compliance conclusions?
6. Where do new catalog ids live so GOV-003 (30–45) and sibling suites stay green?

## Decision

This is what shipped. Catalog counts: **six** additive `control.personnel.*` + **four** `evidence.personnel.*` types + **six** tests in `catalog/canonical/v1/{controls,evidence,tests}/personnel.toml`. The five governance personnel rows in `governance.toml` are unchanged. GOV-003 family count stays in 30–45 (40).

### 1. Keep the identity model thin

`Identity` and `SubjectKind` gain **no** `Employee` or `Contractor` variants. Organization-defined populations (employees, contractors, privileged personnel, developers, finance / security / executive, custom) are `SubjectSelector.tags`, `inventory.subject` facts, and `evidence.personnel.population-membership`. Privileged subsets reuse `evidence.identity.privileged-membership`.

Incorrect: `IdentityKind::Contractor`, `SubjectKind::Employee`, HR profile fields on `Identity`.

### 2. Reuse the population runtime; do not fork it

Personnel tests are declarations over `AllSubjects` / `NoneSubjects` / `CoverageAtLeast` / `CountWhere` / `ManualReview` / `FreshWithin`. Resolution stays:

```text
explicit EvidenceSet population
  → closed selector ids
  → evidence.identity.inventory
  → inventory.subject + inventory.complete
  → inferred observations (Unknown)
```

**Do not** add `resolve_personnel_inventory`. Unknown / partial inventory **must not** yield `Effective` on strong personnel tests.

### 3. Additive catalog content; do not rewrite siblings

Keep the five existing `control.personnel.*` rows in `governance.toml`. Add lifecycle controls / evidence / tests in `catalog/canonical/v1/{controls,evidence,tests}/personnel.toml` (listed in the manifest). Added `control.personnel.*` count is **≤ 6** so the governance-family slice stays in 30–45.

IAM JML ids and tests stay. Personnel **composes** `evidence.identity.lifecycle-event`, `account-status`, and `role-membership`. Additive optional booleans (`active`, `excessive`) land on those fixture facts because string `status` is Technical under `classify_str`, not a population fail.

`active` and `excessive` are **defect flags**: truthy inverts to `failing` in `classify_predicate` so `none-subjects` names still-live leavers and over-privileged movers. Other boolean fields keep the default truthy-pass table ([population runtime](../specs/population-runtime.md) §4.5).

Do not rewrite `sdd_iam_catalog_*`, `sdd_governance_catalog_*`, ISO pack `personnel.access-termination`, or the GitHub collector. Access provisioning was **not** merged into joiner-grace; both `control.personnel.access-provisioning` and `control.personnel.joiner-grace` shipped.

### 4. Evidence is facts; collectors normalize only

New types (each referenced by a control and a test):

```text
evidence.personnel.screening
evidence.personnel.joiner-grace
evidence.personnel.population-membership
evidence.personnel.asset-return
```

Reuse `evidence.personnel.{training,acknowledgement}` and `evidence.identity.*`. Screening records `recorded` / `screened_at` / `required` — not “cleared” or “compliant.”

HRIS / IdP / LMS / MDM collectors (later) emit these types. They never set `Effectiveness`. `looks_like_compliance_claim` still fails closed. Control-test and framework stay collector-free. This slice does **not** require live provider adapters.

### 5. Lifecycle honesty is eight fixtures, not one envelope

Required scenarios: complete training population; one overdue user (`current=false` → `Ineffective`); new-joiner grace (`within_grace`, **not** an IR Exception); leaver with `active=true`; mover with `excessive=true`; expired exception does not suppress fail; missing personnel source never `Effective`; manual screening evidence is facts.

One trained user never proves coverage. Approved unexpired subject-bound exceptions are `ExceptionApproved`, not silent `Effective`.

### 6. Consume Prompt 10 / 12 / 15 / 16; do not implement them

`ControlImplementation` is consumed as-is (no CIR field expansion in this slice). Policy acknowledgement keeps `artifact_ref` strings; this slice does **not** implement the document registry.

Sibling Prompt 12 may already define `DocumentRef` (`implementation.rs`) and `ControlledDocument` (`document.rs`). PER-007 greps **`identity.rs` / `subject.rs` only** so those sibling types do not fail this personnel slice. Do not emit Prompt 15 events or Prompt 16 remediations; those slices may later consume personnel lifecycle facts.

## Consequences

**Positive**

- Training, screening, joiner grace, mover privilege, and leaver access can be tested across a complete population.
- Future HRIS/LMS/MDM collectors have a stable emit contract.
- IAM and governance sibling suites remain the SSOT for their ids.

**Negative / cost**

- Governance-family control count grows (34 + 6 = 40); must stay inside 30–45.
- IAM `status` strings remain a poor predicate; personnel tests use additive defect-flag booleans.
- Live personnel assessment stays fixture-only until later collectors land.
- Predicate polarity for `active` / `excessive` is field-name special-case in the population evaluator (not a new `TestExpr` arm).

**Rejected**

- HRIS-extending `Identity`.
- A second population resolver.
- Encoding MFA / JML process as existence of one lifecycle envelope.
- Treating joiner grace as a control-wide Exception.
- Collectors that emit “personnel compliant.”

## Non-goals (reaffirmed)

Payroll, recruiting, profile UI, HR system of record; live Workday/Okta/KnowBe4/Intune adapters; ISO remapping; certification language; Prompt 10/12/15/16 product scope.

## Landed

| Surface | Location |
| --- | --- |
| Additive catalog | `catalog/canonical/v1/{controls,evidence,tests}/personnel.toml` listed in `manifest.toml` |
| Controls | `screening`, `joiner-grace`, `access-provisioning`, `role-change`, `leaver-access`, `asset-return` |
| Evidence | `screening`, `joiner-grace`, `population-membership`, `asset-return` |
| Tests | `screening-recorded`, `joiner-grace-honored`, `joiner-access-provisioned`, `mover-privileges-reduced`, `no-leaver-active-access`, `asset-return-recorded` |
| Fixtures | `fixtures/assurance/canonical/v1/personnel/{complete-training-population,one-overdue-user,new-joiner-grace,leaver-with-active-access,mover-retaining-excessive-privileges,expired-exception,missing-personnel-source,manual-screening-evidence}/` |
| Evaluator | `classify_predicate` in `crates/weeping-angel-control-test/src/population.rs` |
| Dual-suite | `tests/contracts/personnel_security.{baseline,target}.rs` |

## Related

- Spec SSOT: [`docs/specs/personnel-security.md`](../specs/personnel-security.md)
- Population: [`0003-subject-population-runtime-and-coverage-semantics.md`](0003-subject-population-runtime-and-coverage-semantics.md)
- IAM: [`0003-iam-canonical-assurance-catalog.md`](0003-iam-canonical-assurance-catalog.md)
- Governance: [`0003-governance-canonical-assurance-catalog.md`](0003-governance-canonical-assurance-catalog.md)
- Docs layout: [`0004-documentation-architecture.md`](0004-documentation-architecture.md)
