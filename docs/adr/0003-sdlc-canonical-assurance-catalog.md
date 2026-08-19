# ADR 0003 — SDLC family in the canonical assurance catalog

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md). Does **not** replace [ADR 0002](0002-iso-27001-assurance-vertical.md), [catalog infrastructure](0003-canonical-assurance-catalog-v1.md), or [IAM](0003-iam-canonical-assurance-catalog.md). |
| Extends | Catalog infrastructure, typed evidence, subject-population coverage, IAM family placement pattern |
| Spec | [`docs/specs/sdlc-canonical-assurance-catalog.md`](../specs/sdlc-canonical-assurance-catalog.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Planning baseline | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | `sdd_sdlc_catalog_target` GREEN (SDLC-001…016). Absence-characterization baseline `sdd_sdlc_catalog_baseline` superseded / `#[ignore = "superseded by sdd_sdlc_catalog_target"]`. |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. Draft filename `0003-sdlc-canonical-assurance-catalog-draft.md` is retired.

## Context

ADR 0001 delivered the inwardly extensible assurance spine. ADR 0002 shipped the first ISO 27001 vertical, including a **thin source-control sliver inside the ISO pack** (`source.branch-protection`, `source.required-review`, `source.code-ownership`, `source.security-scanning`, `source.commit-signing`) tested as presence checks on GitHub-shaped evidence (`source.branch.protection`, …). ISO remap later remapped A.8.25 / A.8.26 onto catalog `control.source.*` and retired those slivers.

Catalog infrastructure (catalog infrastructure) ships a pinned exists-only fixture `control.source.protected-branch`. Typed evidence (typed evidence), population coverage (population runtime), and the IAM family (IAM catalog) landed as sibling ADRs. They do not own SDLC-domain content.

Without a provider-neutral SDLC family, a GitLab / Bitbucket / GitHub collector has nowhere canonical to emit repository, CI/CD, release, and supply-chain facts, and “all non-archived in-scope repositories have a protected default branch” cannot be declared as a catalog test.

Questions this decision answers:

1. Where do source-control, CI/CD, release, and supply-chain controls live, if not in `frameworks/iso-27001/2022/metadata.toml` and not as GitHub-specific IDs?
2. How do we avoid colliding with the infrastructure fixture `control.source.protected-branch`?
3. Are SDLC tests existence checks or subject-population assertions?
4. Do we expand the GitHub collector or couple tests to scanner internals?
5. Do we fork the catalog loader, evidence values, or population evaluator?

## Decision

This is what shipped.

### 1. SDLC is canonical catalog content, not a pack and not a collector

Independently assessable SDLC controls live in the catalog infrastructure tree:

```text
catalog/canonical/v1/controls/sdlc.toml
catalog/canonical/v1/evidence/sdlc.toml
catalog/canonical/v1/tests/sdlc.toml
```

Listed in `catalog/canonical/v1/manifest.toml` `[files]`. There is no `evidence/repository.toml` (GitHub collector `ghc_b028` pin). Loaded by `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` — **no second loader**.

Public IDs:

```text
control.source.* / control.cicd.* / control.release.* / control.supply-chain.*
evidence.repository.* / evidence.cicd.* / evidence.deployment.* / evidence.release.* / evidence.supply-chain.*
test.source.* / test.cicd.* / test.release.* / test.supply-chain.*
```

Incorrect: `control.github.branch-protection`, `control.gitlab.protected-branch`, growing the ISO pack `source.*` list as the long-term library, or a GitHub-specific canonical catalog.

The population control for default-branch protection is `control.source.default-branch-protection`. The infrastructure fixture `control.source.protected-branch` (`op = "exists"`) remains.

### 2. Twenty-six provider-neutral controls (20–30 independently assessable)

| Control | Automation |
| --- | --- |
| `control.source.repository-inventory` | automated |
| `control.source.visibility-governance` | automated |
| `control.source.default-branch-protection` | automated |
| `control.source.force-push-restricted` | automated |
| `control.source.branch-deletion-restricted` | automated |
| `control.source.required-review` | automated |
| `control.source.minimum-reviewer-count` | automated |
| `control.source.review-ownership` | automated |
| `control.source.required-status-checks` | automated |
| `control.source.admin-bypass-governance` | **hybrid** |
| `control.source.signed-commits` | automated |
| `control.source.secret-scanning` | automated |
| `control.source.code-scanning` | automated |
| `control.source.dependency-scanning` | automated |
| `control.source.dependency-update-monitoring` | automated |
| `control.supply-chain.dependency-integrity` | automated |
| `control.cicd.workflow-permissions` | automated |
| `control.release.protected-environment` | automated |
| `control.release.authorization` | **hybrid** |
| `control.release.authority-separation` | **hybrid** |
| `control.supply-chain.build-provenance` | automated |
| `control.supply-chain.artifact-integrity` | automated |
| `control.source.change-traceability` | **hybrid** |
| `control.source.security-review` | **hybrid** |
| `control.source.secure-development-policy` | **manual** |
| `control.supply-chain.unsupported-components` | **hybrid** |

Each control has stable id, domain(s) from existing `ControlDomain`, evidence requirements, and a test ref. Validator rejects provider/framework segments in catalog IDs. Canonical SDLC TOML contains no GitHub/GitLab/Bitbucket/Azure DevOps tokens and no ISO/SOC2/NIS2/DORA/GDPR tokens.

Scanning-*enabled* belongs here. Finding-as-evidence belongs to vulnerability catalog.

### 3. Evidence types are facts, not conclusions

Declared in `evidence/sdlc.toml` (catalog id → envelope `evidenceType`):

```text
evidence.repository.inventory
evidence.repository.visibility
evidence.repository.default-branch
evidence.repository.branch-protection
evidence.repository.review-policy
evidence.repository.review-ownership
evidence.repository.security-scanning
evidence.repository.dependency-scanning
evidence.repository.commit-signing
evidence.repository.change-trace
evidence.repository.security-review
evidence.repository.secure-development-policy
evidence.cicd.workflow-permissions
evidence.cicd.status-checks
evidence.deployment.environment-protection
evidence.release.authorization
evidence.supply-chain.build-provenance
evidence.supply-chain.artifact-integrity
evidence.supply-chain.lockfile-state
evidence.supply-chain.component-support
```

Fixtures emit these types plus generic population runtime `inventory.subject` / `inventory.complete`. No `source.branch.protection` in SDLC fixtures. Catalog tests do not read `GITHUB_EVIDENCE_TYPES` or scanner engines.

Population predicates bind the following facts (fixtures may also carry inverse/supporting keys):

| Evidence | Test-bound facts |
| --- | --- |
| `branch-protection` | `protected`, `force_push_restricted`, `deletion_restricted` (fixtures also store `force_push_allowed` / `deletion_allowed` / `admin_bypass_*`) |
| `review-policy` | `reviews_required`, `meets_review_threshold` |
| `security-scanning` | `secret_scanning_enabled`, `code_scanning_enabled` |
| `dependency-scanning` | `scanned_at`, `updates_monitored` |
| `workflow-permissions` | `permissions_minimized` |
| `environment-protection` | `authorization_required` |
| `lockfile-state` | `pins_direct_deps` |

Seal still rejects credential-shaped keys and compliance narratives (`certified`, `compliant`, `audit passed`).

### 4. Tests are population predicates

`test.source.default-branches-protected` means **all in-scope non-archived repositories have a protected default branch**, using population runtime arms (`all-subjects` / `coverage-at-least` 100%). It does not mean “some protection envelope exists.”

Each of the 26 controls has one test. Hybrid/manual tests use `op = "manual-review"` for:

```text
test.source.admin-bypass-governed
test.release.authorization-recorded
test.release.authority-separated
test.source.security-review-recorded
test.source.secure-development-policy-attested
test.supply-chain.unsupported-components-handled
```

`test.source.changes-traceable` is hybrid automation but still a population predicate on `traceable`.

Missing evidence is `InsufficientEvidence`, not a technical failure. Partial/unknown population cannot yield `Effective` on all-subjects tests. Approved unexpired IR exceptions yield `ExceptionApproved` for the bound subject. Stale `scanned_at` / collectedAt yields `StaleEvidence`.

This slice **declares** those tests. It does not reimplement `AllSubjects` / `CoverageAtLeast` and does not add `resolve_repository_inventory`. Authoritative repository / deployment populations use generic `inventory.subject` + `inventory.complete` and/or `EvidenceSet::set_population`.

### 5. Hybrid/manual controls stay honest

Release authorization, authority separation, security review, and secure-development policy must not auto-pass from a single technical flag. Admin-bypass policy acceptance and unsupported-component handling stay hybrid. Absence of attestation → `ManualReviewRequired` or `InsufficientEvidence`, never `Effective`.

### 6. ISO sliver coexistence (ISO remap remapped)

This slice **did not** retarget ISO mappings or grow the ISO pack. Two libraries coexisted until ISO remap.

**Later:** [ADR 0003 remap](0003-iso27001-canonical-remap.md) projected A.8.25 / A.8.26 onto `control.source.default-branch-protection` / `required-review` / `secure-development-policy` / `secret-scanning` / `security-review` and retired pack `source.*` slivers. See [`docs/specs/iso-27001-canonical-remap.md`](../specs/iso-27001-canonical-remap.md) §13.

### 7. Deterministic fixtures

Seven frozen evidence sets under `fixtures/assurance/canonical/v1/sdlc/`:

| Fixture | Distinguishes |
| --- | --- |
| `healthy-org` | Authoritative multi-repo population; automated tests can pass |
| `degraded-org` | Independent defects (unprotected + overbroad workflow + unprotected prod env) → **Ineffective** on those tests |
| `partial-coverage` | Non-authoritative / Partial inventory → **InsufficientEvidence** / **Inconclusive**, never Effective |
| `unprotected-default-branch` | One named in-scope repo `protected=false` → **Ineffective** (missing ≠ fail) |
| `missing-scan-evidence` | One repo lacks scan envelopes → **InsufficientEvidence**, not Ineffective |
| `stale-dependency-scan` | Scan envelopes exist but `scanned_at` outside freshness → **StaleEvidence** |
| `approved-exception` | Approved unexpired subject-scoped IR exception → **ExceptionApproved** for that subject |

Clock in fixtures: `2026-08-19T11:00:00Z`. Booleans stored as string-compat `"true"` / `"false"` (typed evidence `with_fact`).

### 8. Consume catalog infrastructure, typed evidence, and population runtime; do not fork infrastructure

No second catalog loader, typed `EvidenceValue`, or population evaluator. catalog infrastructure’s SSOT (`docs/specs/canonical-assurance-catalog-v1.md`) is pointer-only for this family. This slice does not expand the GitHub collector. Scanner findings remain evidence, not control results (vulnerability catalog).

## Alternatives considered

1. **Grow the ISO pack `source.*` list** — couples the reusable library to one regime; rejected (ADR 0003 catalog infrastructure).
2. **GitHub-native catalog IDs** (`control.github.*`, CODEOWNERS/rulesets as type ids) — a GitLab collector could not populate them; rejected by SDLC catalog.
3. **Replace the exists-only fixture with the population control** — breaks CAT-015 pins; rejected.
4. **Add `resolve_repository_inventory` in control-test** — changes generic population semantics owned by population runtime; rejected.
5. **Emit GitHub `source.*` types from SDLC fixtures** — couples tests to one collector; rejected.
6. **Split `evidence/repository.toml`** — breaks sibling `ghc_b028`; rejected in favor of `*/sdlc.toml`.

## Consequences

**Positive**

- Future GitHub / GitLab / Bitbucket collectors have a stable emit contract (`evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.release.*` / `evidence.supply-chain.*`).
- ISO remap can map ISO A.8.25 / A.8.26 onto `control.source.*` without rewriting collectors.
- Population tests are explainable (failing/missing/stale/excepted subjects) using the population runtime evaluation object.

**Negative / cost**

- Historical ISO sliver IDs may still appear on pre-remap snapshots; live ISO mappings target `control.source.*` ([ADR 0003 remap](0003-iso27001-canonical-remap.md)).
- Hybrid/manual SDLC tests will not auto-pass from technical facts alone; assessments need attestations for those controls.
- Live SCM/CI collectors that emit these contracts are sibling work (GitHub collector); catalog evaluation of live populations is fixture-proven here.

**Rejected**

- Provider-prefixed control IDs.
- Encoding default-branch protection as `Exists(evidence.repository.branch-protection)`.
- Completing coverage math or adding `resolve_repository_inventory` inside this slice.
- Rewriting ISO `metadata.toml` / `mappings.toml` in this slice.
- Inventing a second exception engine or a parallel ADR.

## Non-goals (reaffirmed)

GitHub / GitLab / Bitbucket / Azure DevOps collector expansion; ISO / SOC 2 / NIS 2 / DORA mappings (ISO remap); generic population-runtime redesign; vulnerability catalog finding/SLA family; infrastructure catalog / 08 families; scanner engine / depcheck / SARIF changes; new `SubjectKind` variants; certification language.

## Access and security

- Catalog load remains local-filesystem only.
- SDLC fixtures store booleans, timestamps, subject ids, and branch names — never tokens, passwords, or recovered secrets.
- Seal still rejects credential-shaped fact keys and compliance narratives.

## Related

- Spec SSOT: [`docs/specs/sdlc-canonical-assurance-catalog.md`](../specs/sdlc-canonical-assurance-catalog.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Catalog infrastructure: [`0003-canonical-assurance-catalog-v1.md`](0003-canonical-assurance-catalog-v1.md)
- Typed evidence: [`0003-typed-evidence-canonical-serialization.md`](0003-typed-evidence-canonical-serialization.md)
- Population runtime: [`0003-subject-population-runtime-and-coverage-semantics.md`](0003-subject-population-runtime-and-coverage-semantics.md)
- IAM sibling: [`0003-iam-canonical-assurance-catalog.md`](0003-iam-canonical-assurance-catalog.md)
- ISO remap: [`0003-iso27001-canonical-remap.md`](0003-iso27001-canonical-remap.md)
- ISO vertical (historical sliver): [`0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
