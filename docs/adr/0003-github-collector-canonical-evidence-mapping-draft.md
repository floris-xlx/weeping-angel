# ADR 0003 — GitHub collector emits canonical evidence contracts (DRAFT)

| Field | Value |
| --- | --- |
| Status | **Draft** (accept after `sdd_github_collector_target` is GREEN) |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “GitHub evidence types are canonical (`source.branch.protection`, …)” clause of [ADR 0002](0002-iso-27001-assurance-vertical.md) §6 **for newly emitted collector observations**. Does **not** replace the ISO pack `source.*` sliver, Prompt 12 remap, or ISO GH-007/GH-009 fail-closed/redaction law. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), typed evidence, population coverage, [IAM](0003-iam-canonical-assurance-catalog.md), [SDLC draft](0003-sdlc-canonical-assurance-catalog-draft.md) |
| Spec | [`docs/sdd/github-collector.md`](../sdd/github-collector.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) (GitHub type list updates at implement) |
| Prompt | [`docs/prompts/canonical-assurance-v1/09-github-collector.md`](../prompts/canonical-assurance-v1/09-github-collector.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. **Keep Draft** until the Prompt 09 target suite is GREEN.

## Context

ADR 0002 shipped the first ISO 27001 vertical and treated GitHub-shaped types (`source.branch.protection`, `source.branch.required_reviews`, `source.admin.permissions`, …) as the collector’s public evidence taxonomy. ISO pack tests and GH-012 needles still name those strings.

Prompts 01–04 landed a provider-neutral catalog spine, typed `EvidenceValue`, population completeness (`inventory.subject` / `inventory.complete`), and `evidence.identity.*`. Prompt 05 (not landed as TOML) names `evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` and forbids GitHub tokens in catalog IDs. Its draft ADR said “do not expand the GitHub collector” **in that slice** — expansion is this decision (Prompt 09).

On SHA `e430980c…` the collector still emits `source.*` string facts, advertises types it does not collect, and cannot feed canonical population tests. IAM-015 asserts crate `GITHUB_EVIDENCE_TYPES` contains no `evidence.identity.*`. ISO GH-012 asserts every historical `source.*` string still appears in collector sources.

Questions this decision answers:

1. Are newly collected GitHub observations `source.*` (ADR 0002) or Prompt 04/05 `evidence.*` types?
2. May the collector dual-emit both so ISO existence tests keep passing on GitHub-only types?
3. Where do GitHub node ids, ruleset ids, and environment ids live?
4. How do we keep ISO GH-012 and IAM-015 green **without rewriting those suites**?
5. Is `evidence.github.*` allowed?
6. Must shared `CollectorDescriptor` grow a `failure_behavior` field?

## Decision (proposed)

### 1. New observations use canonical catalog evidence ids

`GitHubCollector` maps provider JSON to:

```text
evidence.repository.*
evidence.cicd.*
evidence.deployment.*
evidence.identity.privileged-membership
evidence.identity.external-access
```

plus generic Prompt 03 envelopes `inventory.subject` and `inventory.complete`.

It does **not** emit `source.repository.exists` / `source.branch.protection` / … as observation types after implement.

Incorrect: `evidence.github.branch-protection`, `control.github.*`, or teaching tests GitHub object names.

### 2. Do not dual-emit ISO-sliver types

Dual-emitting `source.*` and `evidence.repository.*` would make “another provider, same test results” false: ISO existence checks would still pass only for GitHub. Prompt 12 remaps the ISO pack. Until then, ISO GH-012 is satisfied by retaining the historical type **strings** in a collector-owned mapping table, not by sealing `source.*` envelopes.

### 3. Mapping table is the ISO/IAM compatibility surface

Keep exported `GITHUB_EVIDENCE_TYPES` as the ADR 0002 `source.*` name list (no `evidence.identity.*` prefix) so IAM-015 stays green.

`CollectorDescriptor.evidence_types` is a **different**, honest set: canonical types the build actually emits. Identity types join the descriptor via a second const (e.g. `GITHUB_CANONICAL_EVIDENCE_TYPES`), not via `GITHUB_EVIDENCE_TYPES`.

The mapping table (keys = `source.*`, values = canonical ids) is how ISO GH-012 continues to find those strings in crate sources.

### 4. Provider-native identifiers are extensions, not types

GitHub `node_id`, numeric repo id, ruleset id, environment id, collaborator login, deploy-key id may appear in an optional observation fact `extensions` (`EvidenceValue::Object`) or an equivalent collector-private field that **canonical tests must ignore**.

Do **not** add required fields to `EvidenceProvenance` in this slice (typed-evidence digest law / Prompt 02 ownership). Do not invent `evidence.github.*` unless a later central catalog row exists — and even then canonical tests must not require it.

### 5. Facts stay facts

The collector never computes `Effective` / `Ineffective`, never stores framework requirement IDs, and never treats HTTP 403 as a negative boolean. 404 on a protection resource is an observed absence; 403 is insufficient evidence.

### 6. Advertise failure behavior without redesigning shared collector types

`CollectorDescriptor` (collector `lib.rs`) has no `failure_behavior` field. This slice documents 401/403/404/429 semantics in **GitHub-owned** sources (const + mapping-table comments + SDD). Do **not** add a shared field unless a later implement cannot advertise otherwise.

### 7. File ownership

Only `crates/weeping-angel-collector/src/github/**`, GitHub goldens, `tests/sdd/github_collector.*`, and this program’s SDD/ADR docs. No Prompt 05/06/07/08 catalog TOML. No ISO pack rewrite.

## Alternatives considered

1. **Keep emitting `source.*` forever** — GitLab cannot populate those types; rejected by Prompt 09 and Prompt 05.
2. **Dual-emit `source.*` and canonical** — preserves ISO existence tests at the cost of provider-neutral evaluation; rejected.
3. **Change IAM-015 / ISO GH-012 in this slice** — user/protocol forbid rewriting those suites; rejected.
4. **Invent `evidence.github.*` as the public contract** — rejected; extensions only.
5. **Extend `EvidenceProvenance` with `extensions`** — digest/contract risk owned by Prompt 02; deferred.
6. **Add `failure_behavior` to shared `CollectorDescriptor`** — couples every collector to a Prompt 09 field; rejected unless later proven strictly required.

## Consequences

- Implementers rewrite `normalize.rs` / `protection.rs` / stubs to emit Prompt 04/05 contracts and honest `CollectionRun` / pagination / 403 diagnostics.
- `docs/contracts/assurance-runtime.md` GitHub type list updates when the ADR is accepted.
- ISO pack mappings stay on `source.*` until Prompt 12.
- A GitLab collector can emit the same types and share tests.
- Prompt 05 catalog landing rebases fact names to the landed TOML; this ADR’s type ids are the Prompt 05 list, not a third taxonomy.
