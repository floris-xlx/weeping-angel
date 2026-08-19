# SDD: Infrastructure Canonical Assurance Catalog (v1 slice)

| Field | Value |
| --- | --- |
| Status | **Implemented** |
| Program | Canonical Assurance Catalog v1 |
| Slice | Prompt 07 — network / cryptography / secrets / data / database / logging / backup / resilience |
| Source prompt | [`docs/prompts/canonical-assurance-v1/07-infrastructure-catalog.md`](../prompts/canonical-assurance-v1/07-infrastructure-catalog.md) |
| Planning baseline SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` (`main`, 2026-08-19) |
| Dual-suite (register at implement) | `sdd_infrastructure_catalog_baseline` · `sdd_infrastructure_catalog_target` |
| ADR | Accepted [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Prompt-01 SSOT (do not overwrite) | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |
| Prompt-02 / 03 (consumed) | [`docs/sdd/typed-evidence.md`](typed-evidence.md), [`docs/sdd/population-runtime.md`](population-runtime.md) |
| Prompt-04 pattern (do not overwrite) | [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md) |
| Concurrent siblings (do not collide) | Prompt 05 [`sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md); Prompt 06 [`vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md) |
| Spine / ISO law | [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0001 / 0002 |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for the **infrastructure catalog slice**. It does not replace the Prompt 01 catalog-infrastructure SSOT, the Prompt 02 typed-evidence contract, the Prompt 03 population-runtime contract, or the Prompt 04 IAM family. Prompts 01–04 have landed; this slice consumes their loader, `EvidenceValue`, and population evaluator and **must not** invent a second copy.

This spec phase writes **documentation only**. Product TOML, fixtures, dual-suite Rust, and `Cargo.toml` `[[test]]` rows land in the implement phase after baseline GREEN + target RED on current code.

**Verified 2026-08-19** against workspace HEAD `e430980c0d27a8138a153d49b62ddf3c57827891` (`main`). Characterization in §3 is current product state, not a historical planning note. Sibling working-tree files (GitHub collector / lineage / remap / vulnerability / SDLC SDD) must not be treated as this slice.

Architecture law (unchanged):

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

A future AWS, Azure, GCP, Cloudflare, on-prem, or database collector must be able to populate the same evidence contracts and receive the same control results. This slice is provider-neutral and framework-neutral.

---

## 1. Problem / user-visible goal

Organizations need to assess network exposure, cryptography and secret storage, data-store protection, database access, audit logging, backup, and disaster recovery using **provider-neutral** canonical controls.

On SHA `e430980c…` the only infrastructure-adjacent product content is a **thin ISO 27001 pack sliver**:

| Pack control | Automation (pack) | Test kind | Required evidence | Evaluation today |
| --- | --- | --- | --- | --- |
| `logging.security-events` | Hybrid | hybrid | `logging.security-events` | existence of some envelope |
| `logging.audit-trail` | Hybrid | hybrid | `logging.audit-trail` | existence of some envelope |
| `backup.recovery-testing` | Hybrid | hybrid | `backup.configuration.present` | existence of some envelope |
| `encryption.data-at-rest` | Hybrid | hybrid | `encryption.at-rest.configured` | existence of some envelope |
| `encryption.data-in-transit` | Automated | automated | `encryption.in-transit.configured` | existence of some envelope |
| `security.tls` | Automated | automated | `security.tls.misconfiguration` (`break_on` same) | finding-shaped presence / break |

Those tests cannot say “all critical databases encrypt at rest,” “no prohibited public database,” or “restore tests are inside the configured freshness window.” They cannot distinguish missing inventory from a failing store, stale logging from absent logging, or an approved exposure exception from a silent pass.

The canonical catalog at `catalog/canonical/v1/` lists only `fixture.example.toml` and the IAM family (`identity.toml`). There is no `control.network.*` / `control.crypto.*` / `control.secret.*` / `control.data.*` / `control.database.*` / `control.logging.*` / `control.backup.*` / `control.resilience.*` library, no `evidence.network.*` / `evidence.data.*` / `evidence.crypto.*` / `evidence.secret.storage-configuration` / `evidence.database.*` / `evidence.logging.*` / `evidence.backup.*` / `evidence.resilience.*` contracts, and no infrastructure population fixtures.

**User-visible goal:** a coherent infrastructure catalog (35–50 independently assessable controls) that future cloud, database, and network collectors can populate **without framework knowledge**, produce deterministic explainable results (missing ≠ stale ≠ failure ≠ manual review ≠ approved exception), and pass catalog validation plus full workspace verification.

This slice does **not** claim ISO/SOC 2/NIS2 coverage. Framework remapping is Prompt 12 — pack infra slivers retired and A.8.13 / A.8.15 / A.8.24 left unmapped; see [`iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) §13. This slice does **not** implement AWS/Azure/GCP/Cloudflare collectors or a remote inventory service.

---

## 2. Dependencies and fail-closed blockers

| Prompt | Owns | On `e430980c…` | This slice may |
| --- | --- | --- | --- |
| 01 catalog contract | `catalog/canonical/v1/`, `CanonicalCatalog::{load,validate,digest}`, stable-ID rules | **Landed.** Identity + fixture.example listed in `manifest.toml`. | Add infrastructure family TOML + manifest lines. Do not invent a second loader/validator/digest. Do not delete fixture.example IDs. |
| 02 typed evidence | Typed `EvidenceValue`, seal rules | **Landed.** | Declare required fact *names* and semantic types. No second value enum. No secret material in facts. |
| 03 population runtime | Subject populations, `AllSubjects` / `CoverageAtLeast` / `NoneSubjects`, missing/stale/fail split | **Landed.** Identity inventory special-case + generic `inventory.subject` / `inventory.complete`. | Declare population-based tests. **Do not locally reimplement coverage math. Do not add `resolve_database_inventory` / `resolve_network_inventory`.** |
| 04 IAM | `control.identity.*` | **Landed.** | Leave `identity.toml`, identity fixtures, and `sdd_iam_catalog_target` green. |
| 05 SDLC | `control.source.*` / CI / release | **Specified; product unlanded** (run-dir specs + ADR draft). Catalog still only `control.source.protected-branch` fixture. | Do not implement SDLC. Do not edit Prompt 05 paths. |
| 06 vulnerability | `control.vulnerability.*`, `evidence.secret.exposure` | **Specified; product unlanded.** Durable SSOT [`vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md). | Do **not** create `vulnerability.toml`, `evidence.secret.exposure`, `fixtures/.../vulnerability/`, or `tests/sdd/vulnerability_catalog.*`. Secret **storage** (`evidence.secret.storage-configuration`) is this slice; secret **exposure** is Prompt 06. |

Rebase rule: adapt infrastructure content to the landed contracts. Prefer existing `CanonicalCatalog`, `EvidenceValue`, and `evaluate_coverage` over extending this slice’s scope.

Harness rule (Prompt 06 I3): root `Cargo.toml` does **not** auto-discover `tests/sdd/*.rs`. Implement **must** add:

```toml
[[test]]
name = "sdd_infrastructure_catalog_baseline"
path = "tests/sdd/infrastructure_catalog.baseline.rs"

[[test]]
name = "sdd_infrastructure_catalog_target"
path = "tests/sdd/infrastructure_catalog.target.rs"
```

Target command after register:

```text
cargo test --workspace --features demo --test sdd_infrastructure_catalog_target -- --nocapture
```

---

## 3. Current behavior (characterization on `e430980c…`)

Inspected: `catalog/canonical/v1/`, `crates/weeping-angel-canonical-catalog`, `weeping-angel-control-test` (`population.rs`, `expr.rs`), `weeping-angel-collector/src/github/descriptor.rs`, `frameworks/iso-27001/2022/{metadata,mappings}.toml`, `tests/sdd/{iam,canonical,iso27001,population}_*`, root `Cargo.toml` `[[test]]` table, Prompt 07, IAM SSOT, Prompt 06 SSOT, IR `SubjectKind` / `ControlDomain`.

### 3.1 Canonical catalog tree

`catalog/canonical/v1/manifest.toml` lists only:

```text
controls = ["controls/fixture.example.toml", "controls/identity.toml"]
evidence = ["evidence/fixture.example.toml", "evidence/identity.toml"]
tests = ["tests/fixture.example.toml", "tests/identity.toml"]
```

No `network.toml`, `crypto.toml`, `data.toml`, `database.toml`, `logging.toml`, `backup.toml`, or `resilience.toml`. Grep of product TOML/Rust finds **zero** `control.network.*`, `control.crypto.*`, `control.secret.*`, `control.data.*`, `control.database.*`, `control.logging.*`, `control.backup.*`, `control.resilience.*` ids.

Loader: `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest}` exists and rejects provider/framework ID segments (`aws`, `azure`, `gcp`, `cloudflare`, `iso27001`, …). It does **not** ship infrastructure domain content.

### 3.2 ISO pack sliver (existence / hybrid, not population)

`frameworks/iso-27001/2022/metadata.toml` still owns the infrastructure-adjacent pack library. IDs are **not** in the Prompt 01 `control.*` namespace.

`test.logging.security-events` / `test.logging.audit-trail` / `test.backup.recovery-testing` / `test.encryption.data-at-rest` require a single envelope of the named pack evidence type. `test.encryption.data-in-transit` is automated existence. `test.security.tls` requires `security.tls.misconfiguration` and **breaks on** that type (scanner-shaped finding, not TLS-policy population).

ISO mappings (must not be retargeted here):

| From | To |
| --- | --- |
| `iso27001:a.8.13` | `backup.recovery-testing` |
| `iso27001:a.8.15` | `logging.security-events` |
| `iso27001:a.8.24` | `encryption.data-at-rest`, `encryption.data-in-transit`, `security.tls` |

`sdd_iso27001_assurance_target` freezes prefixes `logging.`, `backup.`, `encryption.`, `security.` and expected ids including `logging.security-events`, `backup.recovery-testing`, `encryption.data-at-rest`, `encryption.data-in-transit`. This slice must not rewrite those rows.

### 3.3 Evidence and evaluation

- Facts are `BTreeMap<String, EvidenceValue>` (`weeping-angel-evidence`). Fixtures must use typed names; `with_fact` is string-compat only.
- `TestExpr` includes real `AllSubjects` / `NoneSubjects` / `CoverageAtLeast` / `FreshWithin` / `ManualReview`. Prompt 03 `evaluate_coverage` splits passing / failing / missing / stale / excepted subjects.
- `resolve_population` uses (1) explicit `EvidenceSet` population, (2) identity special-case on `evidence.identity.inventory`, (3) generic `inventory.subject` + `inventory.complete`, else (4) inferred unknown population from observation type. **Unknown / partial populations must not yield `Effective` on all-subjects tests.**
- `Effectiveness::ExceptionApproved` is emitted for approved, unexpired, subject-scoped IR `Exception` rows.
- `AssessmentContext.max_age` is the assessment-policy freshness default.

No infrastructure-shaped envelopes exist under `fixtures/assurance/canonical/`. Identity fixtures only.

### 3.4 Subject and domain model (already sufficient)

IR `SubjectKind` already includes `Database`, `DataStore`, `Endpoint`, `Network`, `Asset`, `Service`, `CloudResource`, `CloudAccount`. Catalog validator accepts those kinds.

IR `ControlDomain` already includes `NetworkSecurity`, `Cryptography`, `DataProtection`, `LoggingMonitoring`, `Resilience`. No new domain enum is required.

### 3.5 Collectors

- `GitHubCollector` advertises `source.*` types only (`source.repository.*`, `source.branch.*`, `source.admin.permissions`, `source.collaborator.permission`, `source.security.*`, `source.workflow.*`, `source.ruleset.present`, `source.commit.signing`). No network, database, backup, TLS-policy, logging, or resilience inventory.
- No AWS / Azure / GCP / Cloudflare collector crate or module.
- Scanner bridge still emits `security.tls.misconfiguration` as a **finding** type, not `evidence.network.tls-configuration`.

### 3.6 Tests and CLI

Root `Cargo.toml` does **not** auto-discover `tests/sdd/*.rs`. Registered dual suites include IAM (`sdd_iam_catalog_{baseline,target}`), catalog infrastructure (`sdd_canonical_assurance_catalog_*`), population, typed evidence, ISO, and (in a dirty working tree) sibling lineage/collector/remap rows. **No** `sdd_infrastructure_catalog_baseline` / `sdd_infrastructure_catalog_target`. No `tests/sdd/infrastructure_catalog.*.rs`.

`CanonicalCatalog::{load,validate,digest}` is the only catalog API. `EvidenceObservation::with_value` is the typed fact writer. Prompt 03 `evaluate_coverage` is the only coverage evaluator. `resolve_population` special-cases `evidence.identity.inventory` only; generic non-identity populations use `inventory.subject` + `inventory.complete`. There is **no** `resolve_database_inventory` / `resolve_network_inventory`.

`TestExpr` extra keys already live on `[test.expression]` as `BTreeMap<String, toml::Value>`. Allowed `op` values include `all-subjects`, `none-subjects`, `coverage-at-least`, `fresh-within`, `manual-review`. Prompt 03 `classify_value` treats only truthy/falsey bools/`0`/`1`/known strings as pass/fail; **integers such as `retention_days=90` are `Technical`**, not a threshold comparison. Threshold tests must therefore bind a **boolean** fact (`meets_threshold`, `meets_policy`, `approved_storage`, `encrypted`) or a temporal `*_at` field — not a raw day count.

Validator `FRAMEWORK_SEGMENTS` includes `iso27001` / `soc2` / `nis2` / `dora` / `gdpr` and hyphenated variants. It does **not** include `pci` / `pci-dss`. Target INFRA-007 must still reject those tokens in infrastructure catalog **file text**.

### 3.7 Public contract

[`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) documents the IAM family (`control.identity.*`) and fixture `control.source.protected-branch`. It does **not** yet name `control.network.*` / `evidence.database.*` / infrastructure fixtures. After implement, that paragraph must be extended so the contract does not lie. This spec write does not edit the contract.

### 3.8 What “infrastructure assessment” means today

A caller can compile the ISO pack and run `test.encryption.data-at-rest`, which requires **some** `encryption.at-rest.configured` envelope (hybrid). It cannot:

- require encryption on every **critical** database;
- fail a public database that policy marks prohibited;
- evaluate TLS on the public-endpoint population;
- fail logging retention below a catalog-configured threshold;
- treat a stale restore test as `StaleEvidence` rather than missing or effective;
- accept AWS/Azure/GCP-shaped facts without teaching tests those providers.

The baseline suite therefore characterizes **absence of a canonical infrastructure catalog** and **presence of the ISO-pack logging/crypto/backup/TLS sliver**, not a working infrastructure-population evaluator.

---

## 4. Desired behavior (after this slice)

### 4.1 Placement (owned files only)

Infrastructure domain content lands in the Prompt 01 catalog tree as **per-family files** (not one `infrastructure.toml`, not `secret.toml`):

```text
catalog/canonical/v1/
  manifest.toml                          # add listings only
  controls/
    network.toml                         # control.network.*
    crypto.toml                          # control.crypto.* and control.secret.*
    data.toml                            # control.data.*
    database.toml                        # control.database.*
    logging.toml                         # control.logging.*
    backup.toml                          # control.backup.*
    resilience.toml                      # control.resilience.*
  evidence/
    network.toml                         # evidence.network.*
    crypto.toml                          # evidence.crypto.* and evidence.secret.storage-configuration
    data.toml                            # evidence.data.*
    database.toml                        # evidence.database.*
    logging.toml                         # evidence.logging.*
    backup.toml                          # evidence.backup.*
    resilience.toml                      # evidence.resilience.*
  tests/
    network.toml
    crypto.toml                          # test.crypto.* and test.secret.approved-storage
    data.toml
    database.toml
    logging.toml
    backup.toml
    resilience.toml
```

Rationale for not creating `*/secret.toml`: Prompt 06 may land `evidence/secret.toml` for `evidence.secret.exposure`. Storage configuration is declared here; exposure is not.

Do **not** add infrastructure controls to `frameworks/iso-27001/2022/metadata.toml`. Do **not** edit `controls/identity.toml`, `fixture.example.toml`, or Prompt 05/06 product paths.

Deterministic fixtures:

```text
fixtures/assurance/canonical/v1/
  network/{healthy,public-db-exposed,insecure-tls,partial-inventory,stale-firewall-policy,exception-approved-exposure}/
  crypto/{healthy,unapproved-secret-storage,stale-certificate}/
  data/{healthy,partial-classification}/
  database/{healthy,unencrypted-critical-db,partial-inventory,missing-encryption}/
  logging/{healthy,retention-below-threshold,stale-audit-log,missing-alerting,partial-coverage}/
  backup/{healthy,missing-backup,stale-restore-test,failing-restore}/
  resilience/{healthy,stale-recovery-plan,missing-dr-exercise,exception-approved-rto}/
```

Each fixture directory contains a frozen `evidence.json` (+ optional Exception records) with a fixed `collectedAt`.

### 4.2 ID and neutrality rules

Stable public IDs:

```text
control.{network,crypto,secret,data,database,logging,backup,resilience}.<slug>
evidence.{network,data,crypto,secret,database,logging,backup,resilience}.<slug>
test.{network,crypto,secret,data,database,logging,backup,resilience}.<slug>
```

Reject in canonical infrastructure content (validator + target suite):

- provider tokens in IDs or as the subject of a control (`aws`, `azure`, `gcp`, `google`, `cloudflare`, `vercel`, plus the existing Prompt 01 denylist);
- provider-specific **canonical contracts** such as `evidence.aws.cloudtrail`, `evidence.cloudflare.tls`, `evidence.azure.nsg`, `control.gcp.kms`;
- provider-specific resource names as catalog IDs (`s3-bucket`, `rds-instance`, `cloudsql`, `security-group-id` as type ids);
- framework tokens in IDs or narrative (`iso27001`, `iso-27001`, `soc2`, `soc-2`, `nis2`, `dora`, `gdpr`, `pci`, `pci-dss`);
- orphaned evidence types or tests;
- duplicate IDs;
- existence-only tests masquerading as population tests (see §4.5).

Correct: `control.database.encryption`, `evidence.network.tls-configuration`. Incorrect: `control.aws.rds-encryption`, `test.iso27001.a.8.24`, `evidence.cloudflare.tls`.

Provider field names (`kms_key_arn`, `cloudflare_ssl_mode`) must not appear in evidence **type** ids. They may appear only inside a future collector’s private normalize step that **emits** canonical facts.

### 4.3 Control family (43 independently assessable controls)

Do not split these into micro-controls to inflate count. Stay in the 35–50 band. Titles and objectives are framework-neutral.

| Control id | Title | Automation | Primary subjects | Required evidence (min) | Tests |
| --- | --- | --- | --- | --- | --- |
| `control.network.admin-interface-restriction` | Administrative interfaces restricted | Automated | endpoint / network | `evidence.network.exposure` | `test.network.admin-interfaces-restricted` |
| `control.network.public-exposure-governance` | Public exposure governed | Hybrid | endpoint / service | `evidence.network.exposure` | `test.network.public-exposure-governed` |
| `control.network.segmentation` | Network segmentation | Hybrid / manual | network | `evidence.network.exposure` (+ segmentation rationale attestation) | `test.network.segmentation-rationale` |
| `control.network.firewall-policy-current` | Firewall policy present and current | Automated | network | `evidence.network.firewall-policy` | `test.network.firewall-policy-current` |
| `control.network.no-unnecessary-public-databases` | Databases not unnecessarily public | Automated | database | `evidence.network.exposure`, `evidence.database.inventory` | `test.network.no-prohibited-public-databases` |
| `control.network.management-access-protection` | Management access protected | Automated | endpoint | `evidence.network.exposure` | `test.network.management-access-protected` |
| `control.network.tls-sensitive-traffic` | TLS for sensitive traffic | Automated | endpoint | `evidence.network.tls-configuration`, `evidence.data.encryption-in-transit` | `test.network.public-endpoints-acceptable-tls` |
| `control.network.insecure-protocol-restriction` | Insecure protocols restricted | Automated | endpoint | `evidence.network.tls-configuration` | `test.network.insecure-protocols-restricted` |
| `control.crypto.encryption-at-rest` | Encryption at rest | Automated | dataStore / database | `evidence.data.encryption-at-rest`, `evidence.crypto.key-state` | `test.crypto.encryption-at-rest-enabled` |
| `control.crypto.encryption-in-transit` | Encryption in transit | Automated | endpoint / service | `evidence.data.encryption-in-transit` | `test.crypto.encryption-in-transit-enabled` |
| `control.crypto.key-lifecycle` | Managed key lifecycle | Hybrid | asset | `evidence.crypto.key-state` | `test.crypto.key-lifecycle-managed` |
| `control.secret.storage` | Secret storage | Automated | asset | `evidence.secret.storage-configuration` | `test.secret.approved-storage` |
| `control.secret.credential-storage` | Credential storage | Automated | asset | `evidence.secret.storage-configuration` | `test.secret.credentials-approved-storage` |
| `control.crypto.key-rotation` | Key and secret rotation | Hybrid | asset | `evidence.crypto.key-state` | `test.crypto.keys-rotated` |
| `control.crypto.certificate-validity` | Certificate validity and rotation | Automated | endpoint | `evidence.crypto.key-state` (certificate facts) | `test.crypto.certificates-valid` |
| `control.crypto.backup-encryption` | Sensitive backup encryption | Automated | dataStore | `evidence.data.encryption-at-rest`, `evidence.backup.configuration` | `test.crypto.backups-encrypted` |
| `control.data.production-inventory` | Production data stores inventoried | Automated | dataStore | `evidence.data.inventory` *or* `inventory.subject` + `evidence.database.inventory` | `test.data.production-stores-inventoried` |
| `control.data.access-restriction` | Data-store access restricted | Hybrid | dataStore | `evidence.database.access-configuration` | `test.data.access-restricted` |
| `control.data.retention-policy` | Data retention policy represented | Hybrid | dataStore | `evidence.logging.retention` *or* supporting retention facts on backup/data | `test.data.retention-policy-represented` |
| `control.data.sensitive-classification` | Sensitive-data classification | Hybrid | dataStore / dataset | classification facts on inventory | `test.data.sensitive-classification-present` |
| `control.database.inventory` | Production databases inventoried | Automated | database | `evidence.database.inventory` | `test.database.inventoried` |
| `control.database.access-restriction` | Database access restricted | Automated | database | `evidence.database.access-configuration` | `test.database.access-restricted` |
| `control.database.encryption` | Database encryption enabled | Automated | database | `evidence.data.encryption-at-rest`, `evidence.database.inventory` | `test.database.critical-encrypt-at-rest` |
| `control.database.backup-enabled` | Database backup enabled | Automated | database | `evidence.backup.configuration` | `test.database.backup-enabled` |
| `control.database.auditing` | Database auditing enabled | Hybrid | database | `evidence.logging.configuration`, `evidence.database.access-configuration` | `test.database.auditing-enabled` |
| `control.logging.audit-enabled` | Audit logging enabled | Automated | asset | `evidence.logging.configuration` | `test.logging.critical-assets-audit-current` |
| `control.logging.admin-events` | Administrative event logging | Automated | asset | `evidence.logging.configuration` | `test.logging.admin-events-recorded` |
| `control.logging.auth-security-events` | Authentication and security event logging | Automated | asset | `evidence.logging.configuration` | `test.logging.auth-security-events-recorded` |
| `control.logging.retention-meets-policy` | Logging retention meets policy | Automated | asset | `evidence.logging.retention` | `test.logging.retention-meets-threshold` |
| `control.logging.time-synchronization` | Time synchronization / consistent timestamps | Hybrid | asset | `evidence.logging.configuration` | `test.logging.time-synchronized` |
| `control.logging.security-alerting` | Security alerting configured | Automated | asset | `evidence.logging.alerting` | `test.logging.alerting-configured` |
| `control.logging.privileged-actions-observable` | Privileged actions observable | Hybrid | asset | `evidence.logging.configuration` | `test.logging.privileged-actions-observable` |
| `control.logging.integrity-protected-storage` | Log integrity / protected storage | Hybrid | asset | `evidence.logging.configuration` | `test.logging.integrity-protected` |
| `control.logging.monitoring-coverage` | Monitoring coverage of critical assets | Automated | asset | `evidence.logging.configuration`, `evidence.logging.alerting` | `test.logging.monitoring-coverage` |
| `control.backup.enabled` | Backup enabled | Automated | dataStore | `evidence.backup.configuration` | `test.backup.enabled` |
| `control.backup.population-coverage` | Backup population coverage | Automated | dataStore | `evidence.backup.configuration`, `evidence.backup.run` | `test.backup.required-stores-current` |
| `control.backup.retention` | Backup retention | Automated | dataStore | `evidence.backup.configuration` | `test.backup.retention-meets-threshold` |
| `control.backup.restore-testing` | Restore testing | Automated | dataStore | `evidence.backup.restore-test` | `test.backup.restore-test-fresh` |
| `control.resilience.recovery-procedure` | Recovery procedure evidence | Hybrid | organization | `evidence.resilience.recovery-plan` | `test.resilience.recovery-procedure-present` |
| `control.resilience.disaster-recovery-exercise` | Disaster-recovery exercise | Hybrid / manual | organization | `evidence.resilience.recovery-plan` | `test.resilience.dr-exercise-recorded` |
| `control.resilience.redundancy` | Redundancy / high availability | Hybrid | service / dataStore | `evidence.resilience.recovery-plan` | `test.resilience.redundancy-where-required` |
| `control.resilience.recovery-objectives` | Recovery objectives documented | Hybrid / manual | organization | `evidence.resilience.recovery-plan` | `test.resilience.recovery-objectives-documented` |
| `control.resilience.recovery-evidence-freshness` | Recovery evidence freshness | Automated | organization | `evidence.resilience.recovery-plan`, `evidence.backup.restore-test` | `test.resilience.recovery-evidence-fresh` |

Each control record must carry: stable id, title, description/objective, domain(s) from existing `ControlDomain` (`networkSecurity`, `cryptography`, `dataProtection`, `loggingMonitoring`, `resilience`, `accessControl` as appropriate), evidence-requirement refs, test refs, and an honest automation class (`automated` | `hybrid` | `manual`).

**Do not invent technical automation** for DR exercises, recovery-objective quality, or network-segmentation rationale. Those remain Hybrid or Manual even if a single technical signal exists.

Supporting evidence types may be added **only** if referenced by a control and a test (no orphans). Prefer facts on the required types in §4.4. If `evidence.data.inventory` is introduced, it must be listed and referenced; otherwise production-store inventory uses generic `inventory.subject` plus `evidence.database.inventory`.

### 4.4 Canonical evidence (facts, not conclusions)

Reuse Prompt 01/02 evidence declarations. This slice **defines** the infrastructure family.

Required contracts (Prompt 07 list — all must exist):

| Evidence type | Observed facts (canonical names; store via `EvidenceValue`) | Not allowed |
| --- | --- | --- |
| `evidence.network.exposure` | `subject_id`, `public` (bool), `admin_interface` (bool), `management_plane` (bool), `exposure_class` (`none` \| `internal` \| `public` \| `restricted`), `prohibited_public`? (bool) | `compliant`, SG/NSG dumps as type id |
| `evidence.network.firewall-policy` | `subject_id`, `policy_present` (bool), `reviewed_at` (timestamp), `current` (bool) | “firewall effective” |
| `evidence.network.tls-configuration` | `subject_id`, `min_protocol` (string), `meets_policy` (bool), `insecure_protocol` (bool) | raw key material; `evidence.cloudflare.tls` |
| `evidence.data.encryption-at-rest` | `subject_id`, `encrypted` (bool), `key_managed`? (bool) | “control passed” |
| `evidence.data.encryption-in-transit` | `subject_id`, `encrypted` (bool), `min_protocol`? | — |
| `evidence.crypto.key-state` | `subject_id`, `key_id` (non-secret handle), `managed` (bool), `rotated_at`? (timestamp), `cert_not_after`? (timestamp), `valid`? (bool) | private keys, ARNs-as-type-id |
| `evidence.secret.storage-configuration` | `subject_id`, `storage_class` (`approved` \| `unapproved` \| `unknown`), `approved_storage` (bool), `backend_kind`? (generic: `vault` \| `kms` \| `file` \| `other` — **not** `aws-secrets-manager` as type id) | secret values; `evidence.secret.exposure` (Prompt 06) |
| `evidence.database.inventory` | `subject_id`, `kind` (`database`), `critical` (bool), `environment` (`production` \| `non-production` \| `unknown`), `classified`? | provider instance ids as type id |
| `evidence.database.access-configuration` | `subject_id`, `publicly_accessible` (bool), `restricted` (bool), `audit_enabled`? (bool) | “least privilege effective” |
| `evidence.logging.configuration` | `subject_id`, `audit_enabled` (bool), `admin_events` (bool), `auth_events` (bool), `privileged_actions` (bool), `integrity_protected`? (bool), `time_synchronized`? (bool) | “logging control passed” |
| `evidence.logging.retention` | `subject_id`, `retention_days` (integer), `meets_threshold`? (bool; collector may compute against **assessment policy**, not a hardcoded ISO day count) | — |
| `evidence.logging.alerting` | `subject_id`, `alerting_configured` (bool), `coverage`? (bool) | — |
| `evidence.backup.configuration` | `subject_id`, `backup_enabled` (bool), `encrypted`? (bool), `retention_days`? (integer), `required`? (bool) | — |
| `evidence.backup.run` | `subject_id`, `ran_at` (timestamp), `success` (bool) | — |
| `evidence.backup.restore-test` | `subject_id`, `tested_at` (timestamp), `success` (bool) | “DR effective” |
| `evidence.resilience.recovery-plan` | `subject_id` or org id, `procedure_present` (bool), `objectives_documented` (bool), `exercise_at`? (timestamp), `redundant`? (bool), `reviewed_at`? | “certified recoverable” |

Seal rules still apply: no credential-shaped keys (`token`, `password`, `secret`, `private_key`); no compliance narratives (`compliant`, `certified`).

Authoritative populations use Prompt 03 generic paths **only**:

```text
inventory.subject   + inventory.complete (authoritative=true)
```

and/or an explicit `EvidenceSet` population. Do **not** add `resolve_database_inventory` / `resolve_network_inventory`. Do **not** extend thin `SubjectSelector` (`{ kind, id }`) with tag filters in this slice.

**In-scope subset encoding (critical / public / required):** Prompt 03 does not filter `critical=true` on inventory. Construct the intended subset as the `database` / `endpoint` / `dataStore` / `asset` kind population in fixtures (non-critical stores may be `dataStore` or omitted). Facts `critical`, `public`, `required` remain documentary for future collectors. Target evaluations use that kind inventory, not a new resolver.

Do **not** introduce `evidence.data.inventory` unless a control and a test both reference it. Default: production-store inventory = generic `inventory.subject` + `evidence.database.inventory`.

### 4.5 Tests (population-based; thresholds from catalog / policy)

Required reusable population tests (Prompt 07 list):

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

Semantics (authoritative intent; exact `TestExpr` spelling follows Prompt 03):

Prompt 03 `classify_value` is the evaluator. Tests must bind **truthy/falsey fields** or temporal `*_at` fields. Raw integers (`retention_days`) and protocol strings (`min_protocol`) are documentary / collector inputs; they are **not** compared by `AllSubjects`.

Recommended `TestExpr` encodings (IAM pattern; extra `[test.expression]` keys are documentary policy, not a new AST):

| Test | `op` | Evidence type | Field (pass/fail) | Catalog/policy keys |
| --- | --- | --- | --- | --- |
| `test.database.critical-encrypt-at-rest` | `all-subjects` | `evidence.data.encryption-at-rest` | `encrypted` | population = `kind = "database"` |
| `test.network.public-endpoints-acceptable-tls` | `all-subjects` | `evidence.network.tls-configuration` | `meets_policy` | `acceptable_min_protocol` (documentary; fixtures set `meets_policy` from it) |
| `test.logging.critical-assets-audit-current` | `all-subjects` | `evidence.logging.configuration` | `audit_enabled` | freshness via `AssessmentContext.max_age` |
| `test.logging.retention-meets-threshold` | `all-subjects` | `evidence.logging.retention` | `meets_threshold` | `min_days` on `[test.expression]` |
| `test.backup.required-stores-current` | `all-subjects` | `evidence.backup.configuration` + current `evidence.backup.run` (`success`) | `backup_enabled` / `success` | `window` or `max_age` on `ran_at` |
| `test.backup.restore-test-fresh` | `all-subjects` | `evidence.backup.restore-test` | `tested_at` (temporal) **and** `success` | `window` / `max_age` |
| `test.network.no-prohibited-public-databases` | `all-subjects` **or** `none-subjects` | `evidence.database.access-configuration` | `restricted` (truthy = not prohibited-public) | — |
| `test.secret.approved-storage` | `all-subjects` | `evidence.secret.storage-configuration` | `approved_storage` | `approved_backends` (documentary; fixtures set the bool) |

| Test | Population | Pass | Fail | Missing | Stale | Manual / exception |
| --- | --- | --- | --- | --- | --- | --- |
| `critical-encrypt-at-rest` | in-scope **critical** databases (kind=`database` inventory) | every subject `encrypted=true` | ≥1 critical DB `encrypted=false` | inventory unknown **or** known critical DB lacks encryption envelope | encryption / inventory older than catalog freshness | approved exception for a named store → `ExceptionApproved` for that subject only |
| `public-endpoints-acceptable-tls` | public-facing endpoints (kind=`endpoint` inventory) | every subject `meets_policy=true` | public endpoint below policy | public population incomplete or TLS envelope missing | stale TLS config | — |
| `critical-assets-audit-current` | critical assets (kind=`asset`) | every subject `audit_enabled=true` and current | critical asset audit off | missing logging.configuration | stale logging.configuration → `StaleEvidence` | — |
| `retention-meets-threshold` | in-scope logged assets | `meets_threshold=true` (collector/fixture computed from `retention_days` vs catalog `min_days`) | `meets_threshold=false` | missing retention envelope | stale retention | — |
| `required-stores-current` | required data stores (kind=`dataStore`) | each has `backup_enabled=true` and a current successful `backup.run` | required store without current backup / `success=false` | missing backup config/run | stale `ran_at` → `StaleEvidence` | — |
| `restore-test-fresh` | required stores | `tested_at` within window **and** `success=true` | restore failed | no restore-test envelope | `tested_at` outside window → `StaleEvidence` | approved exception may `ExceptionApproved` that subject |
| `no-prohibited-public-databases` | databases | every subject `restricted=true` (none are prohibited-public) | `restricted=false` / `publicly_accessible=true` on a prohibited DB | inventory incomplete | stale exposure | approved public-exposure exception → `ExceptionApproved` for that DB |
| `approved-storage` | required secret / credential subjects (kind=`asset`) | every subject `approved_storage=true` | unapproved storage | missing storage-configuration | stale storage-configuration | — |

**Forbidden encoding:** `Exists(evidence.data.encryption-at-rest)` as the body of `test.database.critical-encrypt-at-rest`. One encrypted store is not “all critical databases encrypt at rest.”

**Thresholds must not be hardcoded framework assumptions in Rust.** Encode them as catalog/test configuration:

- Extra keys on the existing `[test.expression]` map (already `BTreeMap<String, toml::Value>` — no loader change): `percentage`, `threshold`, `min_days`, `window`, `acceptable_min_protocol`, `approved_backends`.
- Assessment policy may supply freshness via `AssessmentContext.max_age` when the test does not set a window.
- Do **not** add `const ISO_RETENTION_DAYS: u64 = 365` or `const MIN_TLS = "1.2"` in product crates as the source of truth.
- Target tests may **read** catalog `min_days` / `acceptable_min_protocol` / `approved_backends` and assert fixture `meets_*` facts match that config. They must not bake ISO/PCI constants into `crates/`.

Each of the 43 controls has a test. Hybrid/manual tests use `op = "manual-review"` where honesty requires it. Unknown / non-authoritative population **must not** produce `Effective` for an all-subjects test.

Result metadata (Prompt 03 `PopulationEvaluation`) must explain: population size, evaluated, passing, failing, missing, coverage, failing subject ids, missing subject ids, stale subject ids.

### 4.6 Manual / hybrid honesty

| Control | Why not fully automated |
| --- | --- |
| `control.network.segmentation` | Segmentation *rationale* and trust-zone design are organizational. Exposure facts are supporting, not sufficient. Default: Hybrid/Manual; absence of attestation → `ManualReviewRequired` / `InsufficientEvidence`. |
| `control.network.public-exposure-governance` | Acceptable public surface is a risk decision. Technical `public=true` is the automatable slice; governance of *why* remains hybrid. |
| `control.crypto.key-lifecycle` | Custody, dual-control, and ceremony quality are hybrid. Rotation timestamps are automatable. |
| `control.data.sensitive-classification` | Classification schemes are org-specific. Presence of a classification fact is automatable; scheme quality is not. |
| `control.data.retention-policy` | Legal hold / purpose limitation is organizational. Day-count vs catalog threshold is automatable. |
| `control.logging.time-synchronization` | NTP/chrony inventory can be technical; consistent-timestamp *policy* is hybrid. |
| `control.logging.integrity-protected-storage` | WORM / tamper-evidence quality is hybrid. |
| `control.resilience.disaster-recovery-exercise` | An exercise is a governed event. A timestamp is supporting, not proof of exercise quality. |
| `control.resilience.recovery-objectives` | RTO/RPO values are documented policy, not a collector conclusion. |
| `control.resilience.recovery-procedure` | Procedure *existence* can be attested; effectiveness is hybrid. |
| `control.resilience.redundancy` | “Where required” is an applicability/policy question. |

Do not add a synthetic collector that auto-passes these controls.

### 4.7 Fixtures (deterministic)

Each fixture is a frozen evidence set (+ optional Exception records) with a fixed `collectedAt`. Expected effectiveness is part of the target suite. Fixtures emit **canonical** infrastructure types plus generic `inventory.subject` / `inventory.complete` as needed. No `encryption.at-rest.configured`. No `security.tls.misconfiguration` as the catalog type. No collector id in evidence type.

| Fixture path | Intent | Expected highlights |
| --- | --- | --- |
| `network/healthy` | Authoritative endpoint/network inventory; no public DBs; admin interfaces internal; TLS meets catalog policy | Automated network tests `Effective` |
| `network/public-db-exposed` | Production DB `publicly_accessible=true` / `prohibited_public=true` | `no-prohibited-public-databases` → `Ineffective` naming that subject |
| `network/insecure-tls` | Public endpoint `meets_policy=false` or insecure protocol | `public-endpoints-acceptable-tls` → `Ineffective` |
| `network/partial-inventory` | Non-authoritative / incomplete endpoint population | All-subjects network tests → `InsufficientEvidence`, never Effective |
| `network/stale-firewall-policy` | Firewall policy present but `reviewed_at` outside freshness | `firewall-policy-current` → `StaleEvidence` |
| `network/exception-approved-exposure` | Named public DB with approved unexpired IR Exception bound to `control.network.no-unnecessary-public-databases` | That subject → `ExceptionApproved`, not silent Effective |
| `crypto/healthy` | Keys managed; secrets `approved_storage=true`; certs valid | Automated crypto/secret tests `Effective` |
| `crypto/unapproved-secret-storage` | Required secret `approved_storage=false` | `approved-storage` → `Ineffective` |
| `crypto/stale-certificate` | `cert_not_after` or `rotated_at` outside window | certificate/rotation test → `StaleEvidence` or `Ineffective` (document which; stale if only freshness fails) |
| `data/healthy` | Production stores inventoried and classified | Inventory / classification tests pass or honestly `ManualReviewRequired` if attestation omitted — document the choice |
| `data/partial-classification` | Stores present; classification missing for some | classification test → `InsufficientEvidence` or `Ineffective` per declared expression; not Effective |
| `database/healthy` | Critical DBs encrypted, access restricted, backups on | `critical-encrypt-at-rest` `Effective` |
| `database/unencrypted-critical-db` | One critical DB `encrypted=false` | `critical-encrypt-at-rest` → `Ineffective` naming that DB. A lone encrypted sibling must **not** pass. |
| `database/partial-inventory` | Non-authoritative DB population | All-subjects DB tests → `InsufficientEvidence` |
| `database/missing-encryption` | Known critical DB, no encryption envelope | `InsufficientEvidence` (missing ≠ fail) |
| `logging/healthy` | Critical assets current audit logging; retention ≥ threshold; alerting on | Automated logging tests `Effective` |
| `logging/retention-below-threshold` | `retention_days` < catalog threshold | `retention-meets-threshold` → `Ineffective` |
| `logging/stale-audit-log` | Audit enabled but envelope older than freshness | `critical-assets-audit-current` → `StaleEvidence` |
| `logging/missing-alerting` | Known critical asset, no alerting envelope | alerting / coverage → `InsufficientEvidence` |
| `logging/partial-coverage` | 2 of 3 critical assets have logging | `critical-assets-audit-current` / `monitoring-coverage` not Effective |
| `backup/healthy` | Required stores current successful backups; restore tests inside window | backup population + restore-fresh `Effective` |
| `backup/missing-backup` | Required store, no backup config/run | `required-stores-current` → `InsufficientEvidence` or `Ineffective` if config explicitly `backup_enabled=false` — distinguish both in target assertions |
| `backup/stale-restore-test` | Restore test exists outside window | `restore-test-fresh` → `StaleEvidence` |
| `backup/failing-restore` | Restore `success=false` inside window | `restore-test-fresh` → `Ineffective` |
| `resilience/healthy` | Recovery plan + objectives attested; evidence fresh | Automated freshness `Effective`; hybrid DR/objectives `Effective` only if attestations present |
| `resilience/stale-recovery-plan` | Plan present, `reviewed_at` / exercise outside window | `recovery-evidence-fresh` → `StaleEvidence` |
| `resilience/missing-dr-exercise` | No exercise attestation | `dr-exercise-recorded` → `ManualReviewRequired` / `InsufficientEvidence`, never Effective |
| `resilience/exception-approved-rto` | Named store misses restore freshness; approved unexpired Exception | `ExceptionApproved` for that subject |

Healthy fixtures may share a multi-resource inventory (several DBs, endpoints, stores) so population math is non-trivial (n ≥ 3 subjects for the primary population).

### 4.8 Integration rules (consume, do not redesign)

- Loader / validate / digest: Prompt 01 `CanonicalCatalog`. Infrastructure files must pass `validate` (no orphans, no provider/framework tokens, deterministic digest).
- Typed facts: Prompt 02. Store via `weeping-angel-evidence::EvidenceValue` (`with_value`).
- Population evaluation: Prompt 03 (`evaluate_coverage`, generic inventory). Infrastructure tests are **declarations**. No `InfraPopulation` fork.
- Exception: reuse IR `Exception` + `Effectiveness::ExceptionApproved`.
- Subject kinds: consume existing IR kinds (`Database`, `DataStore`, `Endpoint`, `Network`, `Asset`, `Service`, `Organization`). Do not add a third `SubjectSelector` type.
- ISO pack, GitHub collector, scanner bridge, framework compiler, generic `TestExpr` semantics: **untouched** unless a documented compile blocker requires a one-line compatibility fix, called out in the implement-phase SDD log.
- Prompt 01 SSOT, IAM SSOT, Prompt 06 SSOT, `identity.toml`, `fixture.example.toml`: off-limits.

### 4.9 Dual-suite protocol

Follow the existing root `[[test]]` pattern (IAM).

| Suite | Path (planned) | Role |
| --- | --- | --- |
| Baseline | `tests/sdd/infrastructure_catalog.baseline.rs` · `sdd_infrastructure_catalog_baseline` | GREEN on current tree: **absence** of infrastructure family + **presence** of ISO sliver. After target GREEN, `#[ignore]` so absence-of-catalog is not CI green (`supersede_kind=skip`, matching IAM/ISO). |
| Target | `tests/sdd/infrastructure_catalog.target.rs` · `sdd_infrastructure_catalog_target` | RED on current tree for missing family / fixtures / population semantics. GREEN after implement — CI gate (INFRA-001…016). |

Suggested target assertion clusters (titles include the id):

| ID | Asserts |
| --- | --- |
| INFRA-001 | Catalog tree / Prompt 01 loader loads infrastructure content offline |
| INFRA-002 | Digest of catalog including infrastructure slice is deterministic |
| INFRA-003 | All 43 `control.{network,crypto,secret,data,database,logging,backup,resilience}.*` ids present, stable, correctly prefixed |
| INFRA-004 | Required `evidence.network.{exposure,firewall-policy,tls-configuration}`, `evidence.data.{encryption-at-rest,encryption-in-transit}`, `evidence.crypto.key-state`, `evidence.secret.storage-configuration`, `evidence.database.{inventory,access-configuration}`, `evidence.logging.{configuration,retention,alerting}`, `evidence.backup.{configuration,run,restore-test}`, `evidence.resilience.recovery-plan` declared; no orphans |
| INFRA-005 | Required eight population tests declared and referenced |
| INFRA-006 | Validator rejects provider tokens and `evidence.aws.cloudtrail` / `evidence.cloudflare.tls` style ids |
| INFRA-007 | Validator / target rejects ISO/SOC2/NIS2/PCI tokens in infrastructure catalog file text |
| INFRA-008 | No infrastructure control lives in the ISO pack as `control.network.*` etc.; ISO sliver ids unchanged |
| INFRA-009 | `critical-encrypt-at-rest` is population-based (fails `unencrypted-critical-db`; does not pass on a single encryption envelope) |
| INFRA-010 | Missing vs stale vs fail vs manual vs exception distinguished on the named fixtures |
| INFRA-011 | Partial inventory cannot yield Effective on all-subjects tests |
| INFRA-012 | Approved unexpired exposure / RTO exception → `ExceptionApproved` for that subject |
| INFRA-013 | DR exercise, recovery objectives, and segmentation rationale marked Hybrid or Manual |
| INFRA-014 | Thresholds for retention, TLS acceptability, restore freshness, and approved-storage backends come from catalog/test config or `AssessmentContext`, not Rust framework constants |
| INFRA-015 | No AWS/Azure/GCP/Cloudflare collector; framework crate still has no provider SDK; `evidence.secret.exposure` is **not** created here |
| INFRA-016 | Existing `sdd_iso27001_assurance_target`, `sdd_iam_catalog_target`, and `sdd_assurance_runtime_target` stay green |

### 4.10 Documentation after implement

Later docs pass (not this spec write): this file’s landed record, accept the ADR draft (drop `-draft`), pointer on [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md). Prompt 01 / 04 / 06 SSOTs are not overwritten. No cloud collection or ISO remap is claimed.

After target GREEN the public contract **must** gain an IAM-style paragraph naming:

- 43 `control.{network,crypto,secret,data,database,logging,backup,resilience}.*` ids;
- the sixteen required evidence contracts;
- per-family TOML paths + `fixtures/assurance/canonical/v1/{network,crypto,data,database,logging,backup,resilience}/`;
- population tests (not existence);
- hybrid honesty for DR / objectives / segmentation;
- ISO sliver **not** remapped.

Do not edit the contract in this spec phase.

### 4.11 TOML authoring contract (IAM mirror)

Match `catalog/canonical/v1/{controls,evidence,tests}/identity.toml`.

**Controls** (`controls/{network,crypto,data,database,logging,backup,resilience}.toml`):

```toml
schema = "weeping-angel/canonical-catalog/v1"

[[control]]
id = "control.network.admin-interface-restriction"
title = "Administrative interfaces restricted"
description = "…"
objective = "…"
domains = ["networkSecurity"]
evidence = ["evidence.network.exposure"]
tests = ["test.network.admin-interfaces-restricted"]
automation = "automated"   # or hybrid | manual
```

Domain strings are existing `ControlDomain` camelCase: `networkSecurity`, `cryptography`, `dataProtection`, `loggingMonitoring`, `resilience`, `accessControl` as needed. No new domain enum.

**Evidence** (`evidence/{family}.toml`):

```toml
[[evidence]]
id = "evidence.network.exposure"
title = "Network exposure fact"
evidence_type = "network.exposure"
collection = "automated"
criticality = "required"
```

`evidence_type` is the short fact kind (IAM uses `identity.inventory` beside id `evidence.identity.inventory`). Fixtures and `EvidenceType::new` may use the **full** `evidence.*` id (IAM fixtures do). Pick one convention per family and use it consistently; target INFRA-004 accepts either as long as the catalog `id` is the full `evidence.*` form.

`control.secret.*` and `evidence.secret.storage-configuration` live in `crypto.toml`. Do **not** create `secret.toml`.

**Tests** (`tests/{family}.toml`):

```toml
[[test]]
id = "test.database.critical-encrypt-at-rest"
control = "control.database.encryption"
kind = "automated"
required_evidence = ["evidence.data.encryption-at-rest", "evidence.database.inventory"]
break_on = []

[test.expression]
op = "all-subjects"
evidence = "evidence.data.encryption-at-rest"
field = "encrypted"

[[test.subjects]]
kind = "database"
```

Hybrid/manual honesty: `kind = "hybrid"` / `"manual"` and `op = "manual-review"` for DR exercise, recovery objectives, and segmentation rationale.

`manifest.toml` `[files]` **appends** (does not replace) the seven family files under `controls`, `evidence`, and `tests`. Keep `fixture.example.toml` and `identity.toml`.

### 4.12 Dual-suite authorship (xylex-sdd AC-2 / I4a)

Never write a target (or baseline) test that **reads its own source file** and asserts the source does not contain a substring that also appears in the assertion. That tautology is the I4a trap.

Allowed: scan **product** trees (`catalog/canonical/v1/**`, `crates/**`, `frameworks/iso-27001/2022/**`, fixture JSON) for forbidden tokens / missing ids.

Follow `tests/sdd/iam_catalog.{baseline,target}.rs`:

- Baseline GREEN on **current** code: absence of infrastructure family + presence of ISO sliver.
- Target RED on current code because `control.network.*` / required evidence / population fixtures are missing — not because the test crate fails to compile.
- After GREEN: `#[ignore = "superseded by sdd_infrastructure_catalog_target"]` on the baseline (IAM/ISO pattern).

Register both `[[test]]` rows in root `Cargo.toml` at implement (not this spec write).

### 4.13 Complete test id list (43 — one per control)

Eight Prompt-07 required tests plus one test per remaining control. Do not invent extra micro-tests to inflate count.

```text
test.network.admin-interfaces-restricted
test.network.public-exposure-governed
test.network.segmentation-rationale
test.network.firewall-policy-current
test.network.no-prohibited-public-databases
test.network.management-access-protected
test.network.public-endpoints-acceptable-tls
test.network.insecure-protocols-restricted
test.crypto.encryption-at-rest-enabled
test.crypto.encryption-in-transit-enabled
test.crypto.key-lifecycle-managed
test.secret.approved-storage
test.secret.credentials-approved-storage
test.crypto.keys-rotated
test.crypto.certificates-valid
test.crypto.backups-encrypted
test.data.production-stores-inventoried
test.data.access-restricted
test.data.retention-policy-represented
test.data.sensitive-classification-present
test.database.inventoried
test.database.access-restricted
test.database.critical-encrypt-at-rest
test.database.backup-enabled
test.database.auditing-enabled
test.logging.critical-assets-audit-current
test.logging.admin-events-recorded
test.logging.auth-security-events-recorded
test.logging.retention-meets-threshold
test.logging.time-synchronized
test.logging.alerting-configured
test.logging.privileged-actions-observable
test.logging.integrity-protected
test.logging.monitoring-coverage
test.backup.enabled
test.backup.required-stores-current
test.backup.retention-meets-threshold
test.backup.restore-test-fresh
test.resilience.recovery-procedure-present
test.resilience.dr-exercise-recorded
test.resilience.redundancy-where-required
test.resilience.recovery-objectives-documented
test.resilience.recovery-evidence-fresh
```

### 4.14 Fixture JSON shape (IAM mirror)

Each fixture directory contains `evidence.json`:

```json
{
  "fixture": "healthy",
  "collectedAt": "2026-08-19T12:00:00Z",
  "authoritative": true,
  "evidence": [
    {
      "type": "inventory.subject",
      "subject_id": "db:orders",
      "facts": { "id": "db:orders", "kind": "database", "critical": true }
    },
    {
      "type": "inventory.complete",
      "subject_id": "org:infra",
      "facts": { "kind": "database", "authoritative": true }
    },
    {
      "type": "evidence.data.encryption-at-rest",
      "subject_id": "db:orders",
      "facts": { "subject_id": "db:orders", "encrypted": true, "key_managed": true }
    }
  ]
}
```

Target suite loads JSON into sealed envelopes via `EvidenceValue::with_value` (typed bool/int/timestamp), not string-only `with_fact` as the long-term path. Optional `exceptions.json` for approved unexpired IR `Exception` rows bound to a named subject + control.

Healthy primary populations use **n ≥ 3** subjects so coverage math is non-trivial.

No private keys, connection strings, recovered secrets, or compliance narratives in fixtures.

---

## 5. Acceptance criteria

Testable. Implementation is out of this spec phase.

1. Dual-suite `sdd_infrastructure_catalog_baseline` + `sdd_infrastructure_catalog_target` is registered in root `Cargo.toml` like IAM.
2. On SHA `e430980c…` (current code, pre-infrastructure content): baseline GREEN; target RED for missing `control.{network,crypto,secret,data,database,logging,backup,resilience}.*` / required evidence / population fixtures — not for unrelated compile errors.
3. After implement: target GREEN; baseline ignored so absence-of-catalog is not a CI requirement; `cargo test --workspace --features demo`, `fmt --check`, and `clippy -D warnings` stay green.
4. Forty-three independently assessable controls exist in the 35–50 band with stable ids, domains, evidence requirements, test refs, and honest automation class; no artificial micro-controls.
5. The sixteen required evidence types in §4.4 are declared as facts, not conclusions; no `evidence.aws.*` / `evidence.cloudflare.*` / `evidence.secret.exposure`.
6. Tests include at least the eight Prompt-07 population ids and evaluate **populations**, not existence of one envelope.
7. Evaluator outcomes distinguish missing data, stale data, actual failure, manual review, and approved exception on the named fixtures.
8. DR exercise, recovery objectives, and network segmentation rationale are Hybrid or Manual; they cannot auto-pass without attestation.
9. Catalog validator (Prompt 01) accepts the infrastructure slice: no duplicate/orphan/dangling ids, no provider names, no ISO/SOC2/NIS2/PCI references in canonical infrastructure content.
10. ISO pack control ids and mappings are unchanged; `sdd_iso27001_assurance_target` remains green.
11. No AWS / Azure / GCP / Cloudflare / remote-inventory collector is added; GitHub continues to emit `source.*` only.
12. No second `CanonicalCatalog` loader, no second `EvidenceValue` enum, no local `InfraPopulation` fork. Prompt 03 coverage is consumed as-is (generic `inventory.subject` / `inventory.complete`).
13. Thresholds for retention, TLS policy, restore freshness, and approved storage come from catalog/test configuration or assessment policy.
14. Credential keys and compliance narratives remain rejected on infrastructure evidence.
15. Prompt 01 / 04 / 06 SSOTs, `identity.toml`, `fixture.example.toml`, and Prompt 05/06 product paths are not overwritten.
16. Owned files stay under the infrastructure family paths in §4.1.
17. Target/baseline suites never read their own source and assert it lacks a substring that appears in the assertion (xylex-sdd AC-2 / I4a).
18. Population tests bind Prompt-03-classifiable fields (`encrypted`, `meets_policy`, `meets_threshold`, `restricted`, `approved_storage`, temporal `*_at`); they do not expect `AllSubjects` to compare raw `retention_days` integers.

---

## 6. Out of scope

- AWS, Azure, GCP, Cloudflare, on-prem firewall, or database-engine collectors.
- Remapping ISO 27001 (or SOC 2 / NIS 2 / PCI) onto `control.network.*` / `control.crypto.*` / … (Prompt 12 — slivers retired, infra Annex A still unmapped; [remap §13](iso-27001-canonical-remap.md#13-implement-log)).
- Redesign of `CanonicalCatalog` loader/validator/digest (Prompt 01).
- Redesign of typed evidence / digest canonicalization (Prompt 02).
- Reimplementing `CoverageAtLeast` / `AllSubjects` / population indexes, or adding `resolve_database_inventory` (Prompt 03 owns them).
- Rewriting ISO `metadata.toml` / `mappings.toml` (`logging.security-events`, `backup.recovery-testing`, `encryption.data-*`, `security.tls` stay until Prompt 12).
- Prompt 05 SDLC family, Prompt 06 vulnerability family, `evidence.secret.exposure`, `vulnerability.toml`.
- Editing `catalog/canonical/v1/{controls,evidence,tests}/identity.toml` or `fixture.example.toml`.
- Remote inventory service; live cloud API calls from catalog evaluation.
- Certification, “compliant”, or audit-passed language.
- Governance catalog (Prompt 08).

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Concurrent Prompt 06 lands `evidence/secret.toml` / `evidence.secret.exposure` | This slice never creates `secret.toml` or `evidence.secret.exposure`; storage lives in `crypto.toml`. |
| Concurrent Prompt 05 lands `control.source.*` | Do not touch SDLC paths; infrastructure ids are network/crypto/data/database/logging/backup/resilience. |
| Existence checks sneak in as infrastructure tests | INFRA-009: unencrypted-critical-db must fail; a lone encryption envelope must not pass. |
| Thresholds hardcoded as ISO/PCI constants | INFRA-014; values live in catalog expression/config or `AssessmentContext`. |
| ISO pack rewritten or broken | AC 10; do not touch `frameworks/iso-27001/2022` infrastructure rows. |
| Provider names leak into IDs or fixture types | Validator + INFRA-006/007. |
| Hybrid controls auto-pass from one technical fact | Honest automation class; DR/objectives/segmentation cannot Effective without attestation. |
| Population resolver special-cased per family | Use generic `inventory.subject` / `inventory.complete`; no `resolve_database_inventory`. |
| Prompt 01 / 04 / 06 SSOTs overwritten | This file is the infrastructure slice SSOT. |
| Baseline remains a CI green that asserts catalog absence | After target GREEN, ignore/delete/move baseline. |
| Secrets in fixtures (keys, connection strings) | Seal + AC 14; fixtures use booleans/timestamps/ids only. |
| File-layout collision via one `infrastructure.toml` vs siblings | Per-family files in §4.1 only. |
| Target test I4a tautology (self-read + “does not contain”) | §4.12; scan product trees only. |
| AllSubjects on `retention_days` integer → Technical | Bind `meets_threshold` bool; keep days documentary. |

---

## 8. Dual-suite and SDD protocol (implement phase)

Hard protocol (do not skip):

```text
Spec (this file) → Baseline GREEN on CURRENT code → Target RED on CURRENT code
  → Implement infrastructure catalog content only → Docs/ADR finalize if needed
  → Target GREEN → Prove Baseline FAILS or is skip-superseded
  → Supersede Baseline → Target still GREEN
```

Fail-closed if: baseline cannot go green on current characterization; target cannot go red for the **right** reason (missing infrastructure catalog / population semantics); or target never greens within max_iters.

Workspace verify command is unchanged. Record the implement-phase HEAD SHA in this document when product work starts.

This spec write does **not** register the suites or add product TOML.

---

## 9. ADR

Architecture / public-contract decision: infrastructure content is a **canonical catalog family** (`control.network.*`, `control.crypto.*`, `control.secret.*`, `control.data.*`, `control.database.*`, `control.logging.*`, `control.backup.*`, `control.resilience.*`) consumed later by framework mappings, not an ISO-pack extension and not provider-prefixed checks.

Accepted: [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md).

---

## 10. Planning SHA record

```text
planning_sha = e430980c0d27a8138a153d49b62ddf3c57827891
branch       = main
note         = prompts 01–04 landed (catalog + typed evidence + population + IAM);
               Prompt 05/06 specified, product families unlanded;
               CoverageAtLeast / AllSubjects real; ISO logging/crypto/backup/TLS sliver only
```

---

## 11. Baseline suite record (planned)

| Field | Value |
| --- | --- |
| Planning SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Suite | `sdd_infrastructure_catalog_baseline` · `tests/sdd/infrastructure_catalog.baseline.rs` |
| Expected on current code | **GREEN** (absence characterization) |
| Expected after target GREEN | **ignored** so absence-of-catalog is not CI green |
| Command | `cargo test --workspace --features demo --test sdd_infrastructure_catalog_baseline` |

Baseline asserts (found case):

- catalog tree exists (Prompt 01) but has no infrastructure family files or `control.{network,crypto,secret,data,database,logging,backup,resilience}.*` ids
- no required `evidence.network.*` / `evidence.data.encryption-*` / `evidence.crypto.key-state` / `evidence.secret.storage-configuration` / `evidence.database.*` / `evidence.logging.{configuration,retention,alerting}` / `evidence.backup.*` / `evidence.resilience.recovery-plan`
- no `fixtures/assurance/canonical/v1/{network,crypto,data,database,logging,backup,resilience}/`
- ISO pack still ships `logging.security-events`, `logging.audit-trail`, `backup.recovery-testing`, `encryption.data-at-rest`, `encryption.data-in-transit`, `security.tls` as existence/hybrid (TLS is finding-shaped `break_on`)
- no AWS/Azure/GCP/Cloudflare collectors
- `sdd_infrastructure_catalog_*` not yet a CI requirement on this SHA until registered at implement

---

## 12. Target suite record (planned; RED until implement)

| Field | Value |
| --- | --- |
| Suite | `sdd_infrastructure_catalog_target` · `tests/sdd/infrastructure_catalog.target.rs` |
| Expected on current code | **RED** (missing family / fixtures / population semantics) |
| Expected after implement | **GREEN** (CI gate) |
| Command | `cargo test --workspace --features demo --test sdd_infrastructure_catalog_target -- --nocapture` |
| Planned catalog | files in §4.1 listed in `manifest.toml` |
| Planned fixtures | §4.7 |
| Loader | Prompt 01 `CanonicalCatalog::{load,validate,digest}` — no second loader |
| Population | Prompt 03 `evaluate_coverage` / generic inventory — no `InfraPopulation` fork |

---

## 13. Landed record

Implemented 2026-08-19 on working-tree HEAD `f46cc4690ca131e5eaa932adea1b31fbc3de9965` (planning SHA `e430980c0d27a8138a153d49b62ddf3c57827891`).

- Catalog: `catalog/canonical/v1/{controls,evidence,tests}/{network,crypto,data,database,logging,backup,resilience}.toml` listed in `manifest.toml` `[files]`. `control.secret.*` / `evidence.secret.storage-configuration` live in `crypto.toml`.
- Forty-three independently assessable controls; sixteen required evidence contracts; eight Prompt-07 population tests plus one test per remaining control.
- Fixtures: `fixtures/assurance/canonical/v1/{network,crypto,data,database,logging,backup,resilience}/`.
- Dual-suite registered; baseline `#[ignore = "superseded by sdd_infrastructure_catalog_target"]` after target GREEN.
- ADR accepted: [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md).
- Public contract names the family.

Protocol:

```text
Spec (this file) → Baseline GREEN on planning characterization
  → Target RED for missing infrastructure family / population fixtures
  → Implement family TOML + fixtures + Cargo.toml [[test]] rows
  → ADR / contract pointer finalized
  → Target GREEN → Baseline skip-superseded → Target still GREEN
```

## 14. Implement-phase owned files (allowlist)

Create/update **only**:

```text
catalog/canonical/v1/manifest.toml
catalog/canonical/v1/controls/{network,crypto,data,database,logging,backup,resilience}.toml
catalog/canonical/v1/evidence/{network,crypto,data,database,logging,backup,resilience}.toml
catalog/canonical/v1/tests/{network,crypto,data,database,logging,backup,resilience}.toml
fixtures/assurance/canonical/v1/{network,crypto,data,database,logging,backup,resilience}/**/evidence.json
tests/sdd/infrastructure_catalog.baseline.rs
tests/sdd/infrastructure_catalog.target.rs
Cargo.toml                                          # two [[test]] rows only
docs/sdd/infrastructure-canonical-assurance-catalog.md   # landed SHA
docs/adr/0003-infrastructure-canonical-assurance-catalog.md  # accept; drop -draft
docs/contracts/assurance-runtime.md                 # family pointer only
```

Do **not** create `secret.toml`, `vulnerability.toml`, `infrastructure.toml`, `tests/sdd/vulnerability_catalog.*`, or `fixtures/.../vulnerability/`. Do not edit `identity.toml`, `fixture.example.toml`, `frameworks/iso-27001/2022/**`, Prompt 01/04/06 SSOTs, or Prompt 03 population Rust.

## 15. Target RED reasons (must be these)

On current code, `sdd_infrastructure_catalog_target` must fail because:

1. `catalog/canonical/v1` has no `controls/network.toml` (or any `control.network.*` id).
2. Required evidence contracts (`evidence.database.inventory`, `evidence.network.tls-configuration`, …) are undeclared.
3. `fixtures/assurance/canonical/v1/{network,database,logging,backup,…}` do not exist.

It must **compile**. A missing `[[test]]` row is a harness bug (I3), not a product RED. Register the suites before claiming RED.

Baseline GREEN asserts the same absences plus the ISO sliver still present (`logging.security-events`, `logging.audit-trail`, `backup.recovery-testing`, `encryption.data-at-rest`, `encryption.data-in-transit`, `security.tls`).
