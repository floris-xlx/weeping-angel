# ADR 0003 — SDLC family in the canonical assurance catalog (DRAFT)

| Field | Value |
| --- | --- |
| Status | **Draft** (accept after implement) |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md). Does **not** replace [ADR 0002](0002-iso-27001-assurance-vertical.md), the ISO pack source sliver, [catalog infrastructure](0003-canonical-assurance-catalog-v1.md), or [IAM](0003-iam-canonical-assurance-catalog.md). |
| Extends | Catalog infrastructure, typed evidence, subject-population coverage, IAM family placement pattern |
| Spec | [`docs/sdd/sdd-sdd-625d28d3-3dbb1ba8da/spec.md`](../sdd/sdd-sdd-625d28d3-3dbb1ba8da/spec.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Prompt | [`docs/prompts/canonical-assurance-v1/05-sdlc-catalog.md`](../prompts/canonical-assurance-v1/05-sdlc-catalog.md) |
| Planning baseline | `f6eb344cacefe44f398730c7e963c98887427f1b` |
| Snapshot | tree `35de6aa30ef5b7fc0019a4f99841306ea3af406b` · commit `f6eb344cacefe44f398730c7e963c98887427f1b` |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0001 delivered the inwardly extensible assurance spine. ADR 0002 shipped the first ISO 27001 vertical, including a **thin source-control sliver inside the ISO pack** (`source.branch-protection`, `source.required-review`, `source.code-ownership`, `source.security-scanning`, `source.commit-signing`) tested as presence checks on GitHub-shaped evidence (`source.branch.protection`, …).

Catalog infrastructure (Prompt 01) ships a pinned exists-only fixture `control.source.protected-branch`. Typed evidence (Prompt 02), population coverage (Prompt 03), and the IAM family (Prompt 04) landed as sibling ADRs. They do not own SDLC-domain content.

Without a provider-neutral SDLC family, a future GitLab / Bitbucket collector has nowhere canonical to emit facts, and “all non-archived in-scope repositories have a protected default branch” cannot be declared as a catalog test.

Questions this decision answers:

1. Where do source-control, CI/CD, release, and supply-chain controls live, if not in `frameworks/iso-27001/2022/metadata.toml` and not as GitHub-specific IDs?
2. How do we avoid colliding with the infrastructure fixture `control.source.protected-branch`?
3. Are SDLC tests existence checks or subject-population assertions?
4. Do we expand the GitHub collector or couple tests to scanner internals?
5. Do we fork the catalog loader, evidence values, or population evaluator?

## Decision

### 1. SDLC is canonical catalog content, not a pack and not a collector

Independently assessable SDLC controls live in the Prompt 01 tree (`controls|evidence|tests` for `source`, `cicd`, `release`, `supply-chain`, plus `evidence.repository.*` / `evidence.deployment.*`), listed in `catalog/canonical/v1/manifest.toml`. Loaded by `CanonicalCatalog::{load,validate,digest}` — **no second loader**.

Public IDs use `control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*` and `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.release.*` / `evidence.supply-chain.*`.

Incorrect: `control.github.branch-protection`, growing the ISO pack `source.*` list, or a GitHub-specific canonical catalog.

The population control for default-branch protection is `control.source.default-branch-protection`. The infrastructure fixture `control.source.protected-branch` (`op = "exists"`) remains.

### 2. Twenty-six provider-neutral controls (20–30 independently assessable)

Cover repository inventory/visibility, default-branch protection, force-push and deletion restriction, required review and reviewer count, review ownership, status checks, admin-bypass governance, signed commits, secret/code/dependency scanning, dependency-update monitoring, lockfile integrity, workflow permission minimization, protected environments, release authorization, authority separation, build provenance, artifact integrity, change traceability, security review, secure-development policy, and unsupported-component handling.

Hybrid/manual honesty: release authorization, authority separation, security review, and secure-development policy do not auto-pass from a single technical flag.

### 3. Evidence types are facts, not conclusions

Declared as `evidence.repository.*` etc. Fixtures emit those types. No `source.branch.protection` in SDLC fixtures. Catalog tests do not read `GITHUB_EVIDENCE_TYPES`.

### 4. Tests are population predicates

`test.source.default-branches-protected` means **all in-scope non-archived repositories have a protected default branch**, using Prompt 03 arms. It does not mean “some protection envelope exists.”

Missing evidence is `InsufficientEvidence`, not a technical failure. Partial/unknown population cannot yield `Effective` on all-subjects tests. Approved unexpired IR exceptions yield `ExceptionApproved` for the bound subject.

### 5. Do not change Prompt 03 semantics

Authoritative repository populations use existing generic paths (`inventory.subject` + `inventory.complete` and/or explicit `EvidenceSet` population). No `resolve_repository_inventory`.

### 6. Coexist with the ISO sliver until Prompt 12 remaps

ISO mappings still target `source.branch-protection` and siblings. This slice does not retarget those mappings. Two libraries coexist until Prompt 12.

### 7. Do not expand the GitHub collector

Provider details belong only in future collectors that **emit** canonical facts (Prompt 09). Scanner findings remain evidence, not control results (Prompt 06).

## Alternatives considered

1. **Grow the ISO pack `source.*` list** — couples the reusable library to one regime; rejected (ADR 0003 catalog infrastructure).
2. **GitHub-native catalog IDs** (`control.github.*`, CODEOWNERS/rulesets as type ids) — a GitLab collector could not populate them; rejected by Prompt 05.
3. **Replace the exists-only fixture with the population control** — breaks CAT-015 pins; rejected.
4. **Add `resolve_repository_inventory` in control-test** — changes generic population semantics owned by Prompt 03; rejected.
5. **Emit GitHub `source.*` types from SDLC fixtures** — couples tests to one collector; rejected.

## Consequences

- Implementers add catalog TOML + fixtures + dual-suite tests only.
- `sdd_iso27001_assurance_target`, `sdd_iam_catalog_target`, and `sdd_canonical_assurance_catalog_target` stay green.
- Prompt 12 later remaps ISO pack source stubs onto `control.source.*`.
- A GitHub/GitLab/Bitbucket collector can independently populate the same contracts.
