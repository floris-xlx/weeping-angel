# ADR 0024 — Infrastructure family in the canonical assurance catalog

<!-- weeping-angel-adr-meta
id = "0024"
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
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md). Does **not** replace [ADR 0002](0002-iso-27001-assurance-vertical.md) or the ISO pack logging/crypto/backup/TLS sliver. |
| Extends | [Catalog infrastructure](0003-canonical-assurance-catalog-v1.md), [typed evidence](0036-typed-evidence-canonical-serialization.md), [population / coverage](0034-subject-population-runtime-and-coverage-semantics.md), [IAM family](0022-iam-canonical-assurance-catalog.md) |
| Spec | [`docs/specs/infrastructure-canonical-assurance-catalog.md`](../specs/infrastructure-canonical-assurance-catalog.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Planning baseline | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | `sdd_infrastructure_catalog_target` GREEN (INFRA-001…016). Absence-characterization baseline `sdd_infrastructure_catalog_baseline` superseded / `#[ignore = "superseded by sdd_infrastructure_catalog_target"]`. |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. Draft filename `0003-infrastructure-canonical-assurance-catalog-draft.md` is retired. Accepted after `sdd_infrastructure_catalog_target` GREEN.

## Context

ADR 0001 delivered the inwardly extensible assurance spine. ADR 0002 shipped the first ISO 27001 vertical, including a **thin infrastructure sliver inside the ISO pack** (`logging.security-events`, `logging.audit-trail`, `backup.recovery-testing`, `encryption.data-at-rest`, `encryption.data-in-transit`, `security.tls`) tested as presence/hybrid checks (TLS additionally `break_on` a scanner-shaped `security.tls.misconfiguration` finding). Those tests cannot evaluate populations such as “all critical databases encrypt at rest.”

Canonical catalog infrastructure, typed evidence, subject-population coverage (population runtime), and the IAM family (IAM catalog) landed as sibling ADRs. They provide the loader/validator/digest, fact encoding, `AllSubjects` / `NoneSubjects` / `CoverageAtLeast` runtime, and an identity domain library. They do not own network, crypto, database, logging, backup, or resilience domain content.

SDLC catalog (SDLC) and vulnerability catalog (vulnerability, including `evidence.secret.exposure`) are specified concurrently and must not share files with this family.

Without a provider-neutral infrastructure family, a future AWS / Azure / GCP / Cloudflare / on-prem collector has nowhere canonical to emit facts, and “no prohibited public database” cannot be declared as a catalog test.

Questions this decision answers:

1. Where do network, crypto, secret-*storage*, data, database, logging, backup, and resilience controls live, if not in `frameworks/iso-27001/2022/metadata.toml` and not as `control.aws.*`?
2. What public ID contract do future cloud/database/network collectors and ISO remap ISO remapping consume?
3. Are infrastructure tests existence checks or subject-population assertions?
4. Where do retention / TLS / restore-freshness / approved-storage **thresholds** live?
5. How are DR exercises, recovery objectives, and segmentation rationale marked without fake automation?
6. Do we fork the catalog loader, evidence values, or population evaluator?
7. How do we avoid colliding with vulnerability catalog `evidence.secret.exposure`?

## Decision

This is what shipped.

### 1. Infrastructure is canonical catalog content, not a pack and not a collector

Independently assessable infrastructure controls live in the catalog infrastructure tree as **per-family files**:

```text
catalog/canonical/v1/controls/{network,crypto,data,database,logging,backup,resilience}.toml
catalog/canonical/v1/evidence/{network,crypto,data,database,logging,backup,resilience}.toml
catalog/canonical/v1/tests/{network,crypto,data,database,logging,backup,resilience}.toml
```

Listed in `catalog/canonical/v1/manifest.toml` `[files]`. Loaded by `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` — **no second loader**.

Do **not** create `secret.toml` or `vulnerability.toml`. `control.secret.*` and `evidence.secret.storage-configuration` live in `crypto.toml`. vulnerability catalog owns `evidence.secret.exposure`.

Public IDs:

```text
control.{network,crypto,secret,data,database,logging,backup,resilience}.<slug>
evidence.{network,data,crypto,secret,database,logging,backup,resilience}.<slug>
test.{network,crypto,secret,data,database,logging,backup,resilience}.<slug>
```

Incorrect: `control.aws.rds-encryption`, `evidence.cloudflare.tls`, `evidence.aws.cloudtrail`, `test.iso27001.a.8.24`, or growing the ISO pack `logging.*` / `encryption.*` list as the long-term library.

Provider details belong only in future collectors that **emit** canonical facts. Framework details belong only in later mappings (ISO remap).

### 2. Forty-three provider-neutral controls (35–50 band)

Shipped family (no micro-controls). `control.secret.*` lives in `crypto.toml` (no `secret.toml`).

| Control | Automation |
| --- | --- |
| `control.network.admin-interface-restriction` | automated |
| `control.network.public-exposure-governance` | hybrid |
| `control.network.segmentation` | hybrid (test `kind = "manual"`, `op = "manual-review"`) |
| `control.network.firewall-policy-current` | automated |
| `control.network.no-unnecessary-public-databases` | automated |
| `control.network.management-access-protection` | automated |
| `control.network.tls-sensitive-traffic` | automated |
| `control.network.insecure-protocol-restriction` | automated |
| `control.crypto.encryption-at-rest` | automated |
| `control.crypto.encryption-in-transit` | automated |
| `control.crypto.key-lifecycle` | hybrid |
| `control.secret.storage` | automated |
| `control.secret.credential-storage` | automated |
| `control.crypto.key-rotation` | hybrid |
| `control.crypto.certificate-validity` | automated |
| `control.crypto.backup-encryption` | automated |
| `control.data.production-inventory` | automated |
| `control.data.access-restriction` | hybrid |
| `control.data.retention-policy` | hybrid |
| `control.data.sensitive-classification` | hybrid |
| `control.database.inventory` | automated |
| `control.database.access-restriction` | automated |
| `control.database.encryption` | automated |
| `control.database.backup-enabled` | automated |
| `control.database.auditing` | hybrid |
| `control.logging.audit-enabled` | automated |
| `control.logging.admin-events` | automated |
| `control.logging.auth-security-events` | automated |
| `control.logging.retention-meets-policy` | automated |
| `control.logging.time-synchronization` | hybrid |
| `control.logging.security-alerting` | automated |
| `control.logging.privileged-actions-observable` | hybrid |
| `control.logging.integrity-protected-storage` | hybrid |
| `control.logging.monitoring-coverage` | automated |
| `control.backup.enabled` | automated |
| `control.backup.population-coverage` | automated |
| `control.backup.retention` | automated |
| `control.backup.restore-testing` | automated |
| `control.resilience.recovery-procedure` | hybrid |
| `control.resilience.disaster-recovery-exercise` | **manual** |
| `control.resilience.redundancy` | hybrid |
| `control.resilience.recovery-objectives` | **manual** |
| `control.resilience.recovery-evidence-freshness` | automated |

Hybrid/manual honesty: DR exercise, recovery objectives, and network-segmentation rationale use `op = "manual-review"` and do not auto-pass from a single technical flag. Each control has stable id, domain(s) from existing `ControlDomain`, evidence requirements, and a test ref. Validator rejects provider/framework segments. Canonical infrastructure TOML contains no ISO/SOC2/NIS2/PCI tokens.

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

Fixtures emit these types (plus generic `inventory.subject` / `inventory.complete`). No `encryption.at-rest.configured` in infrastructure fixtures. No `evidence.aws.cloudtrail`. Credential-shaped keys and compliance narratives remain rejected at seal (typed evidence).

### 4. Tests are population predicates; thresholds are configuration

`test.database.critical-encrypt-at-rest` means **all in-scope critical databases encrypt at rest**, using population runtime arms. It does not mean “some encryption envelope exists.”

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

Shipped `[test.expression]` keys (documentary policy; evaluator classifies bool / `*_at` facts): `min_days = 90`, `acceptable_min_protocol = "1.2"`, `approved_backends = ["vault", "kms"]`, `window = "24h"`. Assessment policy may still supply freshness via `AssessmentContext.max_age`. Not hardcoded ISO/PCI constants in Rust.

Missing evidence is `InsufficientEvidence`, not a technical failure. Partial/unknown population cannot yield `Effective` on all-subjects tests. Approved unexpired IR exceptions yield `ExceptionApproved` for the bound subject.

### 5. Do not change population runtime semantics

Authoritative database / endpoint / data-store populations use existing generic paths (`inventory.subject` + `inventory.complete` and/or explicit `EvidenceSet` population). No `resolve_database_inventory` / `resolve_network_inventory`. No tag-filter compiler on the thin `{ kind, id }` selector.

In-scope “critical” / “public” / “required” subsets are the **kind inventory** fixtures construct (e.g. only critical DBs are `kind=database`). Documentary facts (`critical`, `public`, `required`) stay on envelopes for future collectors.

`AllSubjects` / `NoneSubjects` classify only truthy/falsey fields (and temporal `*_at`). Integer `retention_days` and string `min_protocol` are not compared by the evaluator. Threshold tests bind boolean facts (`meets_threshold`, `meets_policy`, `approved_storage`) whose values fixtures compute from catalog `[test.expression]` keys (`min_days`, `acceptable_min_protocol`, `approved_backends`) or `AssessmentContext.max_age`.

### 6. ISO sliver coexistence (ISO remap retired unmapped)

This slice does not retarget ISO mappings. **Later:** [ADR 0003 remap](0027-iso27001-canonical-remap.md) retired pack `logging.*` / `backup.*` / `encryption.*` / `security.tls` slivers and left A.8.13 / A.8.15 / A.8.24 unmapped rather than claiming catalog equivalence. See [`docs/specs/iso-27001-canonical-remap.md`](../specs/iso-27001-canonical-remap.md) §13.

### 7. Deterministic fixtures

Twenty-nine frozen evidence sets under `fixtures/assurance/canonical/v1/{network,crypto,data,database,logging,backup,resilience}/` (clock `2026-08-19T12:00:00Z` unless a stale fixture):

| Fixture | Distinguishes |
| --- | --- |
| `network/healthy` | Authoritative endpoint/network inventory; automated network tests can pass |
| `network/public-db-exposed` | Prohibited public DB → **Ineffective** |
| `network/insecure-tls` | Public endpoint below policy → **Ineffective** |
| `network/partial-inventory` | Non-authoritative population → **InsufficientEvidence** |
| `network/stale-firewall-policy` | Policy older than freshness → **StaleEvidence** |
| `network/exception-approved-exposure` | Named public DB + approved unexpired Exception → **ExceptionApproved** |
| `crypto/healthy` | Approved storage, managed keys, valid certs |
| `crypto/unapproved-secret-storage` | `approved_storage=false` → **Ineffective** |
| `crypto/stale-certificate` | Certificate/rotation freshness fail |
| `data/healthy` | Production stores inventoried / classified |
| `data/partial-classification` | Missing classification → not Effective |
| `database/healthy` | Critical DBs encrypted → **Effective** |
| `database/unencrypted-critical-db` | One `encrypted=false` → **Ineffective** (lone encrypted sibling must not pass) |
| `database/partial-inventory` | Non-authoritative DB population → **InsufficientEvidence** |
| `database/missing-encryption` | Known DB, no envelope → **InsufficientEvidence** |
| `logging/healthy` | Current audit, retention ≥ catalog `min_days`, alerting on |
| `logging/retention-below-threshold` | `meets_threshold=false` → **Ineffective** |
| `logging/stale-audit-log` | Envelope older than freshness → **StaleEvidence** |
| `logging/missing-alerting` | No alerting envelope → **InsufficientEvidence** |
| `logging/partial-coverage` | Incomplete monitoring coverage → not Effective |
| `logging/partial-inventory` | Non-authoritative asset population → **InsufficientEvidence** |
| `backup/healthy` | Current successful backups + restore tests |
| `backup/missing-backup` | Required store without backup evidence |
| `backup/stale-restore-test` | Restore outside window → **StaleEvidence** |
| `backup/failing-restore` | `success=false` → **Ineffective** |
| `resilience/healthy` | Plan + objectives attested; automated freshness can pass |
| `resilience/stale-recovery-plan` | Plan/exercise outside window → **StaleEvidence** |
| `resilience/missing-dr-exercise` | No exercise attestation → **ManualReviewRequired** / **InsufficientEvidence** |
| `resilience/exception-approved-rto` | Named store + approved Exception → **ExceptionApproved** |

Healthy primary populations use n ≥ 3 subjects. Fixtures emit canonical types plus generic `inventory.subject` / `inventory.complete`. No pack-local `encryption.at-rest.configured`. No secret material.

### 8. Consume catalog infrastructure, typed evidence, and population runtime; do not fork infrastructure

No second catalog loader, typed `EvidenceValue`, or population evaluator. No `resolve_database_inventory` / `resolve_network_inventory`. catalog infrastructure’s SSOT is not overwritten. No AWS / Azure / GCP / Cloudflare collector.

A compile-graph cycle (facade depending on the root package) was broken with scanner-view traits in `weeping-angel-evidence` / `weeping-angel-assurance`. That is not a catalog API fork.

### 9. Do not add cloud collectors

Provider details belong only in future collectors that **emit** canonical facts. This slice does not implement AWS/Azure/GCP/Cloudflare collectors or a remote inventory service. Operational restore stays here; governance catalog owns continuity/DR **governance** (`evidence.resilience.continuity-plan`), not `evidence.backup.restore-test`.

## Alternatives considered

1. **Grow the ISO pack `logging.*` / `encryption.*` / `backup.*` list** — couples the reusable library to one regime; rejected (ADR 0003 catalog infrastructure).
2. **Provider-native catalog IDs** (`control.aws.rds-encryption`, `evidence.cloudflare.tls`) — an Azure collector could not populate them; rejected by infrastructure catalog.
3. **One `infrastructure.toml`** — harder to own beside concurrent SDLC and vulnerability catalogs files; rejected in favor of per-family files.
4. **`evidence/secret.toml` for storage-configuration** — collides with vulnerability catalog `evidence.secret.exposure`; rejected. Storage lives in `crypto.toml`.
5. **Add `resolve_database_inventory` in control-test** — changes generic population semantics owned by population runtime; rejected.
6. **Hardcode TLS 1.2 / 365-day retention in Rust** — smuggles a framework assumption into the runtime; rejected. Thresholds are catalog/policy configuration.
7. **Encode encryption as `Exists(evidence.data.encryption-at-rest)`** — a single store would pass “all critical databases”; rejected.
8. **Add `resolve_database_inventory` so `critical=true` is a runtime filter** — changes population runtime; rejected. Fixtures construct the kind population.

## Consequences

**Positive**

- Future cloud/database/network collectors have a stable emit contract.
- ISO remap retired pack infra slivers and left A.8.13 / A.8.15 / A.8.24 unmapped; a later honest remap can still target `control.crypto.*` / `control.logging.*` ([ADR 0003 remap](0027-iso27001-canonical-remap.md)).
- Population tests are explainable using the population runtime evaluation object.
- vulnerability catalog can land `evidence.secret.exposure` without a file conflict.

**Negative / cost**

- Annex A infra clauses remain unmapped after sliver retirement; catalog `control.logging.*` / `control.crypto.*` are not yet an ISO projection.
- Hybrid/manual tests will not auto-pass from technical facts alone; assessments need attestations for DR, objectives, and segmentation.
- Collectors are still future work; catalog evaluation of live cloud populations is fixture-only until later slices.

**Rejected**

- Provider-prefixed control IDs and provider-specific canonical evidence contracts.
- Encoding population tests as existence checks.
- Completing inventory resolution inside this slice.
- Rewriting ISO `metadata.toml` / `mappings.toml`.
- Inventing a second exception engine or catalog loader.
- Creating `secret.toml` / `evidence.secret.exposure` / `vulnerability.toml`.

## Non-goals (reaffirmed)

AWS / Azure / GCP / Cloudflare collectors; ISO / SOC 2 / NIS 2 / PCI text or mappings; generic rule-engine expansion; remote inventory service; certification language; IAM / SDLC / vulnerability / governance catalog families (IAM catalog–06, 08); `evidence.secret.exposure`.

## Access and security

- Catalog load remains local-filesystem only.
- Infrastructure fixtures store booleans, timestamps, subject ids, and protocol names — never private keys, connection strings, or recovered secrets.
- Seal still rejects credential-shaped fact keys and compliance narratives.

## Related

- Spec SSOT: [`docs/specs/infrastructure-canonical-assurance-catalog.md`](../specs/infrastructure-canonical-assurance-catalog.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Catalog infrastructure: [`0003-canonical-assurance-catalog-v1.md`](0003-canonical-assurance-catalog-v1.md)
- Typed evidence: [`0036-typed-evidence-canonical-serialization.md`](0036-typed-evidence-canonical-serialization.md)
- Population runtime: [`0034-subject-population-runtime-and-coverage-semantics.md`](0034-subject-population-runtime-and-coverage-semantics.md)
- IAM family: [`0022-iam-canonical-assurance-catalog.md`](0022-iam-canonical-assurance-catalog.md)
- SDLC sibling: [`0033-sdlc-canonical-assurance-catalog.md`](0033-sdlc-canonical-assurance-catalog.md)
- Vulnerability sibling (do not collide on `evidence.secret.exposure`): [`0037-vulnerability-canonical-assurance-catalog.md`](0037-vulnerability-canonical-assurance-catalog.md)
- Governance sibling (continuity **governance** only): [`0021-governance-canonical-assurance-catalog.md`](0021-governance-canonical-assurance-catalog.md)
- ISO vertical (sliver frozen): [`0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
