# ADR 0020 — GitHub collector emits canonical evidence contracts

<!-- weeping-angel-adr-meta
id = "0020"
status = "accepted"
supersedes = ["0019-github-collector-canonical-evidence-mapping-draft"]
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “GitHub evidence types are canonical (`source.branch.protection`, …)” clause of [ADR 0002](0002-iso-27001-assurance-vertical.md) §6 **for newly emitted collector observations**. Does **not** replace ISO GH-007 / GH-009 fail-closed/redaction law, ISO remap remap, or the ISO pack as a projection. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [typed evidence](0036-typed-evidence-canonical-serialization.md), [population / coverage](0034-subject-population-runtime-and-coverage-semantics.md), [IAM](0022-iam-canonical-assurance-catalog.md), [SDLC](0033-sdlc-canonical-assurance-catalog.md) |
| Spec | [`docs/specs/github-collector.md`](../specs/github-collector.md) |
| Run | [`.sdd/runs/sdd-github-collector.md`](../../.sdd/runs/sdd-github-collector.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | `sdd_github_collector_target` GREEN (`ghc_000`–`ghc_024`, 30 pass). Baseline `sdd_github_collector_baseline` superseded (`#[ignore]`, 30 ignored). |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. Draft filename dropped after target GREEN.

## Context

ADR 0002 shipped the first ISO 27001 vertical and treated GitHub-shaped types (`source.branch.protection`, `source.branch.required_reviews`, `source.admin.permissions`, …) as the collector’s public evidence taxonomy. ISO GH-012 still requires those **strings** to appear in collector crate sources. IAM-015 asserts crate `GITHUB_EVIDENCE_TYPES` contains no `evidence.identity.*`.

catalog infrastructure–05 landed a provider-neutral catalog spine, typed `EvidenceValue`, population completeness (`inventory.subject` / `inventory.complete`), `evidence.identity.*`, and SDLC `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*`. SDLC catalog TOML forbids GitHub tokens in catalog IDs and told that slice not to expand the collector — expansion is this decision (GitHub collector).

On SHA `e430980c…` the collector still emitted `source.*` string facts, advertised types it did not collect, aborted the whole run on HTTP 403, hardcoded protection on `main`, and recorded an empty `CollectionRun`.

Questions this decision answers:

1. Are newly collected GitHub observations ADR 0002 `source.*` types or IAM/SDLC catalogs `evidence.*` types?
2. May the collector dual-emit both so ISO existence tests keep passing on GitHub-only types?
3. Where do GitHub node ids, ruleset ids, and deploy-key ids live?
4. How do ISO GH-012 and IAM-015 stay green **without rewriting those suites**?
5. Is `evidence.github.*` allowed?
6. Must shared `CollectorDescriptor` grow a `failure_behavior` field?

## Decision

This is what shipped.

### 1. New observations use canonical catalog evidence ids

`GitHubCollector` maps GitHub API JSON to these **emitted** types (`GITHUB_CANONICAL_EVIDENCE_TYPES` → `CollectorDescriptor.evidence_types`):

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
evidence.cicd.status-checks
evidence.cicd.workflow-permissions
evidence.deployment.environment-protection
evidence.identity.privileged-membership
evidence.identity.external-access
inventory.subject
inventory.complete
```

Facts use typed `EvidenceValue` via `with_value`. The collector does **not** emit `source.repository.exists` / `source.branch.protection` / … as observation types.

Incorrect: `evidence.github.branch-protection`, `control.github.*`, or teaching canonical tests GitHub object names.

### 2. Do not dual-emit ISO-sliver types

Dual-emitting `source.*` and `evidence.repository.*` would make “another provider, same test results” false: ISO existence checks would still pass only for GitHub. ISO remap remaps the ISO pack. ISO GH-012 is satisfied by retaining the historical type **strings** in a collector-owned mapping table (`SOURCE_TO_CANONICAL`), not by sealing `source.*` envelopes.

### 3. Mapping table is the ISO/IAM compatibility surface

Exported `GITHUB_EVIDENCE_TYPES` remains the ADR 0002 `source.*` name list (no `evidence.identity.*` prefix) so IAM-015 stays green.

`CollectorDescriptor.evidence_types` is a **different**, honest set: canonical types this build actually emits. Identity types join the descriptor via `GITHUB_CANONICAL_EVIDENCE_TYPES`, never via `GITHUB_EVIDENCE_TYPES`.

### 4. Provider-native identifiers stay off the type id

Subject ids are stable labels (`repo:owner/name`, `user:{login}`, `deploy-key:{id}`). Provenance records `collector.github`, scope label, and asset. Deploy-key **material** is never stored.

An observation `extensions` object and extra `EvidenceProvenance` fields were **not** added (typed-evidence digest law / typed evidence ownership). Canonical tests ignore any future provider-native extras. `evidence.github.*` is not emitted and is not required by goldens.

### 5. Facts stay facts

The collector never computes `Effective` / `Ineffective`, never stores ISO/SOC2/NIS2/DORA requirement IDs, and never treats HTTP 403 as a negative boolean.

| HTTP | Shipped behavior |
| --- | --- |
| 401 / 403 | `PermissionDenied` / insufficient-evidence diagnostic; other subjects continue; never `protected=false` / `enabled=false` |
| 404 on protection / ruleset | Observed absence (`protected=false`) |
| 404 on repository | Insufficient / not visible — not `exists=false` |
| 429 | Fixture client advances to the next matching fixture when present; else partial. Never a boolean observation |
| 5xx / transport | Partial run; keep prior envelopes |
| Pagination hole or list 403 | `inventory.complete` `authoritative=false` (or omitted for explicit-repo holes) |

`GITHUB_FAILURE_BEHAVIOR` documents this in GitHub-owned sources. Shared `CollectorDescriptor` did **not** grow a `failure_behavior` field.

### 6. Honest descriptor, pagination, and CollectionRun

| Field | Shipped value |
| --- | --- |
| `id` | `collector.github` |
| `provider_family` | `source-control` |
| `subject_types` | `repository`, `branch`, `organization`, `identity`, `deployment` |
| `capabilities.pagination` | `true` — `GitHubClient::get_pages` walks `Link` / `per_page` |
| `capabilities.incremental` | `false` |
| `required_permissions` | `contents:read`, `metadata:read`, `administration:read`, `actions:read`, `members:read`, `security_events:read` |

Scope labels: `org:{login}`, `repo:owner/name` (or `owner/name`), comma-lists, plus GitHub-owned `exclude_archived`. Protection/ruleset paths use the repo’s `default_branch`, never hardcoded `main`. Fixture match is longest-prefix-safe.

`collect_batch` fills `CollectionRun`: collector version, scope label, secret-free `configuration_digest` (id + version + scope + advertised types + transport mode), `started_at` / `completed_at`, evidence/error counts, `complete` / `partial` / `failed`. Tokens are not in the digest.

### 7. Identity mapping is GitHub-observable only

- Repo admins → `evidence.identity.privileged-membership` + `inventory.subject` (`kind=identity`)
- Outside collaborators → `evidence.identity.external-access` + `inventory.subject` (`kind=user`)
- Deploy keys → privileged-membership (`roles=["deploy-key"]`) + identity subject; write-capable keys also emit `external-access`. No `evidence.identity.service-account` type (not advertised; control coverage is supporting via membership facts)

The collector is not an IdP: no MFA, last-active, or termination facts.

### 8. Security

- Shared `weeping_angel_evidence::redact` still folds `Bearer `, `token=`, `ghp_`, `gho_`, `github_pat_`
- GitHub-owned `sanitize_diagnostic` additionally folds `ghs_`, `ghu_`, `ghr_`
- `authorization_header()` remains `"Bearer [redacted]"`
- Fixtures and `configuration_digest` contain no live-shaped tokens

### 9. File ownership

Only `crates/weeping-angel-collector/src/github/**`, goldens under `fixtures/assurance/canonical/v1/github/`, `tests/contracts/github_collector.*`, and this program’s SDD/ADR/contract text. No vulnerability, infrastructure, and governance catalogs catalog TOML rewrite. No ISO pack rewrite.

Shipped goldens: `healthy-org`, `unprotected-repo`, `missing-branch-protection-permission`, `paginated-inventory`, `paginated-inventory-truncated`, `archived-excluded-by-selector`, `disabled-security-scanning`, `protected-environment-absent`, `privileged-membership-population`, `api-partial-failure`, `rate-limit-retry`.

Healthy-org type/fact coverage enables ≥25 independently assessable catalog controls (SDLC source/CI/release + observable IAM). The target suite enumerates those pairs (`EXERCISABLE_CONTROLS`); it does not compute `Effective`.

## Alternatives considered

1. **Keep emitting `source.*` forever** — GitLab cannot populate those types; rejected by GitHub collector and SDLC catalog.
2. **Dual-emit `source.*` and canonical** — preserves ISO existence tests at the cost of provider-neutral evaluation; rejected.
3. **Change IAM-015 / ISO GH-012 in this slice** — protocol forbids rewriting those suites; rejected.
4. **Invent `evidence.github.*` as the public contract** — rejected; not emitted.
5. **Extend `EvidenceProvenance` with `extensions`** — digest/contract risk owned by typed evidence; deferred. Traceability uses subject_id + provenance.asset.
6. **Add `failure_behavior` to shared `CollectorDescriptor`** — couples every collector to a GitHub collector field; rejected.

## Consequences

- `docs/specs/assurance-runtime.md` GitHub type list is the canonical emit set, not the ADR 0002 `source.*` list.
- ISO pack mappings stay on catalog `control.*` (ISO remap). GH-012 needles remain mapping-table strings in collector sources.
- A GitLab / Bitbucket collector can emit the same types and share tests.
- SDLC catalog landing already named these evidence ids; this collector consumes them. It does not invent a third taxonomy.
- IAM-015 remains true: `GITHUB_EVIDENCE_TYPES` has no `evidence.identity.*`. The descriptor still advertises identity types via the second const.
- Live HTTP transport and a SaaS credential store remain out of scope.
- Who seals envelopes (adapters emit observations; `EnvelopeFactory` is the only seal site) is [ADR 0013](0013-collector-hexagonal-modular-monolith.md). This ADR still owns GitHub → canonical type/fact mapping.

## Related

- Spec (evidence contract): [`docs/specs/github-collector.md`](../specs/github-collector.md)
- Crate layout: [`docs/specs/collector-hexagonal.md`](../specs/collector-hexagonal.md), [ADR 0013](0013-collector-hexagonal-modular-monolith.md)
