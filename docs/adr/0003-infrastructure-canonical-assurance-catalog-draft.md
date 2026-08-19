# ADR 0003 — Infrastructure family in the canonical assurance catalog (DRAFT)

| Field | Value |
| --- | --- |
| Status | **Draft** (accept after implement) |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md). Does **not** replace [ADR 0002](0002-iso-27001-assurance-vertical.md) or the ISO pack logging/crypto/backup/TLS sliver. |
| Extends | [Catalog infrastructure](0003-canonical-assurance-catalog-v1.md), [typed evidence](0003-typed-evidence-canonical-serialization.md), [population / coverage](0003-subject-population-runtime-and-coverage-semantics.md), [IAM family](0003-iam-canonical-assurance-catalog.md) |
| Spec | [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](../sdd/infrastructure-canonical-assurance-catalog.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Prompt | [`docs/prompts/canonical-assurance-v1/07-infrastructure-catalog.md`](../prompts/canonical-assurance-v1/07-infrastructure-catalog.md) |
| Planning baseline | `e430980c0d27a8138a153d49b62ddf3c57827891` |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. Accept and drop `-draft` after the target suite is GREEN.

## Context

ADR 0001 delivered the inwardly extensible assurance spine. ADR 0002 shipped the first ISO 27001 vertical, including a **thin infrastructure sliver inside the ISO pack** (`logging.security-events`, `logging.audit-trail`, `backup.recovery-testing`, `encryption.data-at-rest`, `encryption.data-in-transit`, `security.tls`) tested as presence/hybrid checks (TLS additionally `break_on` a scanner-shaped `security.tls.misconfiguration` finding). Those tests cannot evaluate populations such as “all critical databases encrypt at rest.”

Canonical catalog infrastructure (Prompt 01), typed evidence (Prompt 02), subject-population coverage (Prompt 03), and the IAM family (Prompt 04) landed as sibling ADRs. They provide the loader/validator/digest, fact encoding, `AllSubjects` / `NoneSubjects` / `CoverageAtLeast` runtime, and an identity domain library. They do not own network, crypto, database, logging, backup, or resilience domain content.

Prompt 05 (SDLC) and Prompt 06 (vulnerability, including `evidence.secret.exposure`) are specified concurrently and must not share files with this family.

Without a provider-neutral infrastructure family, a future AWS / Azure / GCP / Cloudflare / on-prem collector has nowhere canonical to emit facts, and “no prohibited public database” cannot be declared as a catalog test.

Questions this decision answers:

1. Where do network, crypto, secret-*storage*, data, database, logging, backup, and resilience controls live, if not in `frameworks/iso-27001/2022/metadata.toml` and not as `control.aws.*`?
2. What public ID contract do future cloud/database/network collectors and Prompt 12 ISO remapping consume?
3. Are infrastructure tests existence checks or subject-population assertions?
4. Where do retention / TLS / restore-freshness / approved-storage **thresholds** live?
5. How are DR exercises, recovery objectives, and segmentation rationale marked without fake automation?
6. Do we fork the catalog loader, evidence values, or population evaluator?
7. How do we avoid colliding with Prompt 06 `evidence.secret.exposure`?

## Decision (proposed)

### 1. Infrastructure is canonical catalog content, not a pack and not a collector

Independently assessable infrastructure controls live in the Prompt 01 tree as **per-family files**:

```text
catalog/canonical/v1/controls/{network,crypto,data,database,logging,backup,resilience}.toml
catalog/canonical/v1/evidence/{network,crypto,data,database,logging,backup,resilience}.toml
catalog/canonical/v1/tests/{network,crypto,data,database,logging,backup,resilience}.toml
```

Listed in `catalog/canonical/v1/manifest.toml` `[files]`. Loaded by `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` — **no second loader**.

Do **not** create `secret.toml` or `vulnerability.toml`. `control.secret.*` and `evidence.secret.storage-configuration` live in `crypto.toml`. Prompt 06 owns `evidence.secret.exposure`.

Public IDs:

```text
control.{network,crypto,secret,data,database,logging,backup,resilience}.<slug>
evidence.{network,data,crypto,secret,database,logging,backup,resilience}.<slug>
test.{network,crypto,secret,data,database,logging,backup,resilience}.<slug>
```

Incorrect: `control.aws.rds-encryption`, `evidence.cloudflare.tls`, `evidence.aws.cloudtrail`, `test.iso27001.a.8.24`, or growing the ISO pack `logging.*` / `encryption.*` list as the long-term library.

Provider details belong only in future collectors that **emit** canonical facts. Framework details belong only in later mappings (Prompt 12).

### 2. Forty-three provider-neutral controls (35–50 band)

Family covers: admin-interface restriction, public-exposure governance, segmentation, current firewall policy, no unnecessary public databases, management-access protection, TLS for sensitive traffic, insecure-protocol restriction; encryption at rest/in transit, key lifecycle, secret/credential storage, key/secret rotation, certificate validity, backup encryption; production data-store and database inventory, access restriction, database encryption/backup/auditing, retention policy, sensitive classification; audit / admin / auth logging, retention vs threshold, time sync, alerting, privileged-action observability, log integrity, monitoring coverage; backup enablement/coverage/retention, restore testing; recovery procedure, DR exercise, redundancy, recovery objectives, recovery-evidence freshness.

Hybrid/manual honesty: DR exercise, recovery objectives, and network-segmentation rationale do not auto-pass from a single technical flag.

### 3. Evidence types are facts, not conclusions

Required contracts:

```text
evidence.network.exposure
evidence.network.firewall-policy
evidence.network.tls-configuration
evidence.data.encryption-at-rest
evidence.data.encryption-in-transit
evidence.crypto.key-state
evidence.secret.storage-configuration
evidence.database.inventory
evidence.database.access-configuration
evidence.logging.configuration
evidence.logging.retention
evidence.logging.alerting
evidence.backup.configuration
evidence.backup.run
evidence.backup.restore-test
evidence.resilience.recovery-plan
```

Fixtures emit these types (plus generic `inventory.subject` / `inventory.complete`). No `encryption.at-rest.configured` in infrastructure fixtures. No `evidence.aws.cloudtrail`. Credential-shaped keys and compliance narratives remain rejected at seal (Prompt 02).

### 4. Tests are population predicates; thresholds are configuration

`test.database.critical-encrypt-at-rest` means **all in-scope critical databases encrypt at rest**, using Prompt 03 arms. It does not mean “some encryption envelope exists.”

Required reusable tests:

```text
test.database.critical-encrypt-at-rest
test.network.public-endpoints-acceptable-tls
test.logging.critical-assets-audit-current
test.logging.retention-meets-threshold
test.backup.required-stores-current
test.backup.restore-test-fresh
test.network.no-prohibited-public-databases
test.secret.approved-storage
```

Retention days, acceptable TLS minimum, restore-test freshness window, and approved secret-storage backends come from **catalog/test configuration** (`[test.expression]` keys) or assessment policy (`AssessmentContext.max_age`) — not hardcoded ISO/PCI constants in Rust.

Missing evidence is `InsufficientEvidence`, not a technical failure. Partial/unknown population cannot yield `Effective` on all-subjects tests. Approved unexpired IR exceptions yield `ExceptionApproved` for the bound subject.

### 5. Do not change Prompt 03 semantics

Authoritative database / endpoint / data-store populations use existing generic paths (`inventory.subject` + `inventory.complete` and/or explicit `EvidenceSet` population). No `resolve_database_inventory` / `resolve_network_inventory`. No tag-filter compiler on the thin `{ kind, id }` selector.

In-scope “critical” / “public” / “required” subsets are the **kind inventory** fixtures construct (e.g. only critical DBs are `kind=database`). Documentary facts (`critical`, `public`, `required`) stay on envelopes for future collectors.

`AllSubjects` / `NoneSubjects` classify only truthy/falsey fields (and temporal `*_at`). Integer `retention_days` and string `min_protocol` are not compared by the evaluator. Threshold tests bind boolean facts (`meets_threshold`, `meets_policy`, `approved_storage`) whose values fixtures compute from catalog `[test.expression]` keys (`min_days`, `acceptable_min_protocol`, `approved_backends`) or `AssessmentContext.max_age`.

### 6. Coexist with the ISO sliver until Prompt 12 remaps

ISO mappings still target `logging.security-events`, `backup.recovery-testing`, `encryption.data-at-rest`, `encryption.data-in-transit`, and `security.tls`. This slice does not retarget those mappings. Two libraries coexist until Prompt 12.

### 7. Do not add cloud collectors

Provider details belong only in future collectors that **emit** canonical facts. This slice does not implement AWS/Azure/GCP/Cloudflare collectors or a remote inventory service.

## Alternatives considered

1. **Grow the ISO pack `logging.*` / `encryption.*` / `backup.*` list** — couples the reusable library to one regime; rejected (ADR 0003 catalog infrastructure).
2. **Provider-native catalog IDs** (`control.aws.rds-encryption`, `evidence.cloudflare.tls`) — an Azure collector could not populate them; rejected by Prompt 07.
3. **One `infrastructure.toml`** — harder to own beside concurrent Prompt 05/06 files; rejected in favor of per-family files.
4. **`evidence/secret.toml` for storage-configuration** — collides with Prompt 06 `evidence.secret.exposure`; rejected. Storage lives in `crypto.toml`.
5. **Add `resolve_database_inventory` in control-test** — changes generic population semantics owned by Prompt 03; rejected.
6. **Hardcode TLS 1.2 / 365-day retention in Rust** — smuggles a framework assumption into the runtime; rejected. Thresholds are catalog/policy configuration.
7. **Encode encryption as `Exists(evidence.data.encryption-at-rest)`** — a single store would pass “all critical databases”; rejected.
8. **Add `resolve_database_inventory` so `critical=true` is a runtime filter** — changes Prompt 03; rejected. Fixtures construct the kind population.

## Consequences

**Positive**

- Future cloud/database/network collectors have a stable emit contract.
- Prompt 12 can map `iso27001:a.8.24` → `control.crypto.encryption-at-rest` (and siblings) without rewriting collectors.
- Population tests are explainable using the Prompt 03 evaluation object.
- Prompt 06 can land `evidence.secret.exposure` without a file conflict.

**Negative / cost**

- Two infrastructure libraries until remap (pack `logging.*` / `encryption.*` vs catalog `control.logging.*` / `control.crypto.*`).
- Hybrid/manual tests will not auto-pass from technical facts alone; assessments need attestations for DR, objectives, and segmentation.
- Collectors are still future work; catalog evaluation of live cloud populations is fixture-only until later prompts.

**Rejected**

- Provider-prefixed control IDs and provider-specific canonical evidence contracts.
- Encoding population tests as existence checks.
- Completing inventory resolution inside this slice.
- Rewriting ISO `metadata.toml` / `mappings.toml`.
- Inventing a second exception engine or catalog loader.
- Creating `secret.toml` / `evidence.secret.exposure` / `vulnerability.toml`.

## Non-goals (reaffirmed)

AWS / Azure / GCP / Cloudflare collectors; ISO / SOC 2 / NIS 2 / PCI text or mappings; generic rule-engine expansion; remote inventory service; certification language; IAM / SDLC / vulnerability / governance catalog families (Prompts 04–06, 08); `evidence.secret.exposure`.

## Access and security

- Catalog load remains local-filesystem only.
- Infrastructure fixtures store booleans, timestamps, subject ids, and protocol names — never private keys, connection strings, or recovered secrets.
- Seal still rejects credential-shaped fact keys and compliance narratives.

## Related

- Spec SSOT: [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](../sdd/infrastructure-canonical-assurance-catalog.md)
- Public contract: [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md)
- Catalog infrastructure: [`0003-canonical-assurance-catalog-v1.md`](0003-canonical-assurance-catalog-v1.md)
- Typed evidence: [`0003-typed-evidence-canonical-serialization.md`](0003-typed-evidence-canonical-serialization.md)
- Population runtime: [`0003-subject-population-runtime-and-coverage-semantics.md`](0003-subject-population-runtime-and-coverage-semantics.md)
- IAM family: [`0003-iam-canonical-assurance-catalog.md`](0003-iam-canonical-assurance-catalog.md)
- Vulnerability sibling (do not collide): [`0003-vulnerability-canonical-assurance-catalog-draft.md`](0003-vulnerability-canonical-assurance-catalog-draft.md)
- ISO vertical (sliver frozen): [`0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
