# Assurance runtime contract

Machine-facing contract for the assurance spine **and** the first ISO 27001 vertical. Security scan documents stay in [`codex-security/references/scan-contract.md`](../../codex-security/references/scan-contract.md).

Decisions:

- Spine: [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md)
- ISO vertical: [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md)
- Canonical catalog: [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md)
- Typed evidence: [`docs/adr/0003-typed-evidence-canonical-serialization.md`](../adr/0003-typed-evidence-canonical-serialization.md)
- Population / coverage: [`docs/adr/0003-subject-population-runtime-and-coverage-semantics.md`](../adr/0003-subject-population-runtime-and-coverage-semantics.md)
- IAM catalog family: [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md)
- SDLC catalog family: [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md)
- Vulnerability catalog family: [`docs/adr/0003-vulnerability-canonical-assurance-catalog.md`](../adr/0003-vulnerability-canonical-assurance-catalog.md)
- Infrastructure catalog family: [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md)
- Governance catalog family: [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md)
- Applicability engine: [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md)
- Assessment lineage: [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md)
- ISO remap onto catalog: [`docs/adr/0003-iso27001-canonical-remap.md`](../adr/0003-iso27001-canonical-remap.md)
- GitHub collector mapping: [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](../adr/0003-github-collector-canonical-evidence-mapping.md)

Public composition root is `weeping-angel-assurance`. Callers select a profile + capabilities; they do not import per-regime adapters.

This is an **automated readiness/assurance** contract, not a certification authority. Automated evaluation must never emit `ISO 27001 certified`, `ISO 27001 compliant`, `certification guaranteed`, or `audit passed`.

Allowed language: `ready`, `effective`, `ineffective`, `insufficient evidence`, `requires manual review`, `not applicable`, `assessment coverage`, `partially covered`.

## Schema

| Item | Value |
| --- | --- |
| IR schema | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Evidence schema | `evidence/v1` (`EVIDENCE_SCHEMA`) |
| Evidence value encoding | `evidence-value/v1` (`EVIDENCE_VALUE_SCHEMA`) — hybrid JSON inside observation facts |
| Framework pack schema | `weeping-angel/framework-pack/v1` (`FRAMEWORK_PACK_SCHEMA`) |
| Canonical catalog schema | `weeping-angel/canonical-catalog/v1` (`CATALOG_SCHEMA`) |
| Applicability snapshot schema | `weeping-angel/applicability-snapshot/v1` (`APPLICABILITY_SNAPSHOT_SCHEMA`) — Kleene document in `::applicability` |
| Lineage snapshot schema | `weeping-angel/assessment-lineage/v1` (`LINEAGE_SNAPSHOT_SCHEMA`) — persist / replay documents in `::lineage` | |
| JSON | serde `camelCase` on public documents |
| Digests | SHA-256 hex of `serde_json` bytes (struct field order; `BTreeMap`/`BTreeSet` for maps/sets) |
| Catalog digest | Display `wa:canonical-catalog:weeping-angel/canonical-catalog/v1:` + SHA-256 hex of parsed documents (`DIGEST_PREFIX` + IR `canonical_digest`; prefix not mixed into the hash) |

Every IR document (`Control`, `Requirement`, `Mapping`, `EvidenceRequirement`, `PlannedControlTest`) carries `schemaVersion`. Compile rejects any other version. `Assessment` remains a **framework-crate** in-memory document compiled by `compile_framework` (not an IR type). Concurrent IR work may later introduce `AssessmentDefinition`; consume it by rebase, do not fork.

## Identity

Newtypes, stable string form, no random v4 in persisted identity:

`FrameworkId`, `FrameworkVersion`, `RequirementId`, `ControlId`, `ControlImplementationId`, `ControlTestId`, `AssetId`, `IdentityId`, `VendorId`, `ProcessingActivityId`, `EvidenceRequirementId`, `RiskId`, `ExceptionId`, `AssessmentId`, `AuditProgramId`, `EvidenceType`.

`EvidenceType` names a **fact kind**. Canonical names used by this vertical include the SDLC family `evidence.repository.*` (`inventory`, `visibility`, `default-branch`, `branch-protection`, `review-policy`, `review-ownership`, `security-scanning`, `dependency-scanning`, `commit-signing`, `change-trace`, `security-review`, `secure-development-policy`), `evidence.cicd.*` (`workflow-permissions`, `status-checks`), `evidence.deployment.environment-protection`, `evidence.release.authorization`, `evidence.supply-chain.*` (`build-provenance`, `artifact-integrity`, `lockfile-state`, `component-support`), generic population envelopes `inventory.subject` / `inventory.complete`, historical ISO-sliver strings `source.branch.protection` / `source.branch.required_reviews` / `source.codeowners.present` (compatibility needles, **not** newly emitted GitHub types), `security_finding`, `manual_attestation` (legacy pack / capability / `collector.manual` envelope — **not** the catalog type), the IAM family `evidence.identity.*` (`inventory`, `authentication-state`, `mfa-status`, `privileged-membership`, `role-membership`, `last-active`, `account-status`, `account-owner`, `access-review`, `lifecycle-event`, `service-account`, `external-access`), the infrastructure family `evidence.network.*` (`exposure`, `firewall-policy`, `tls-configuration`), `evidence.data.*` (`encryption-at-rest`, `encryption-in-transit`), `evidence.crypto.key-state`, `evidence.secret.storage-configuration` (storage class — **not** `evidence.secret.exposure`), `evidence.database.*` (`inventory`, `access-configuration`), `evidence.logging.*` (`configuration`, `retention`, `alerting`), `evidence.backup.*` (`configuration`, `run`, `restore-test`), `evidence.resilience.recovery-plan` (operational restore; continuity **governance** is `evidence.resilience.continuity-plan`), the vulnerability family `evidence.vulnerability.*` (`finding`, `scan-run`, `scan-coverage`, `remediation-state`, `owner`, `exception`, `exposure-review`), `evidence.secret.exposure`, `evidence.dependency.*` (`vulnerability`, `confusion-risk`), `evidence.asset.software-inventory`, and the governance family `evidence.manual.attestation` plus `evidence.governance.*` (`policy`, `policy-review`, `management-review`, `internal-audit`), `evidence.risk.*` (`assessment`, `treatment`), `evidence.personnel.*` (`training`, `acknowledgement`), `evidence.vendor.*` (`inventory`, `risk-review`), `evidence.incident.exercise`, `evidence.resilience.continuity-plan` (plan/DR **governance**; operational restore stays `evidence.resilience.recovery-plan`). Scanner-bridge `security_*` types (`security_finding`, `security.vulnerability.present`, `security.secret.exposure`, `security.dependency_confusion_risk`) remain the **bridge taxonomy**, not the catalog library. It is not a framework name and must not be prefixed `github.*` / `iso27001.*` unless the provider or regime is genuinely part of the fact.

`SubjectKind` (IR SSOT): `organization`, `asset`, `repository`, `service`, `identity`, `user`, `privilegedIdentity`, `device`, `vendor`, `dataset`, `processingActivity`, `branch`, `application`, `database`, `cloudAccount`, `cloudResource`, `serviceAccount`, `endpoint`, `dataStore`, `network`, `deployment`.

`IdentityKind` includes `serviceAccount`. `AssetKind` includes `branch` and `deployment`. `Exception.subjects` is `Vec<SubjectSelector>` (default empty — **not** the entire inventory).

## Normative relationship

```text
Requirement → Mapping → Canonical Control → Control Test → Evidence Requirement
         ↘ EvidenceEnvelope ← Collector / scanner bridge
```

- `Control` fields: `schemaVersion`, `id`, `title`, `description`. No annex / SoA / clause / ISO fields.
- `Requirement` is a distinct type (`id`, `frameworkId`, `frameworkVersion`, `title`, `description`).
- `Mapping` is `{ fromRequirement, toControl, direction, completeness, relation, rationale, provenance?, validFor? }`.
  - `direction`: `forward` \| `reverse` \| `bidirectional`
  - `completeness`: `full` \| `partial` \| `related`
  - `relation`: `Equivalent` \| `Satisfies` \| `PartiallySatisfies` \| `Supports` \| `EvidenceFor` \| `SupersetOf` \| `SubsetOf` \| `Related`
  - `provenance.source`: `BuiltIn` \| `UserDefined` \| `LicensedFrameworkContent` (optional author/reference)
  - `validFor`: optional `{ from, to }` framework-version constraint
  - Mapping is never identity. Partial does not become `full`. `relation` defaults from completeness (`full` → `Satisfies`, `partial` → `PartiallySatisfies`, `related` → `Related`). Material mappings carry rationale and provenance. `PartiallySatisfies` / `Supports` / `Related` / `EvidenceFor` / `SubsetOf` cannot fully satisfy a requirement. `Equivalent` is never a convenience.

`ComplianceGraph::equivalent(a, b)` is true only when both `a→b` and `b→a` exist with `completeness = full`. A partial path `A → B → C` is never `A ≡ C`. Reverse edges are not invented. `Supports` never upgrades to `Satisfies`.

## Framework packs

Packs live on disk, not as compiled Rust catalogs:

```text
frameworks/<id>/<version>/{manifest,requirements,mappings,applicability,metadata}.toml
```

ISO 27001:2022: `frameworks/iso-27001/2022`. Thin baseline: `frameworks/wa-baseline/1`.

Rules:

- Deterministic, versioned, network-free, provider-independent.
- Validated before compile (`validate_framework_pack`).
- Public ISO pack is `StructuralOnly` — identifiers and mappings, no protected ISO normative wording.
- `FrameworkPackDigest` is computed over canonical pack content and recorded on snapshots.
- Old pack versions migrate explicitly or fail with guidance; they are never silently reinterpreted.
- Packs resolve by `(id, version)` via `load_framework_pack`. Mapping `to` values are catalog control IDs (`control.*`); the pack loader fails closed on unknown catalog targets and retired slivers. The ISO pack is a projection over the catalog, not a second control library (`metadata.toml` must not declare `access.*` / `source.*` slivers).
- Reports pin `frameworkPackDigest` and the catalog digest (`canonicalCatalogDigest` on `AssessmentReport` / `AssessmentRun`; `catalogDigest` on readiness JSON). Serialize uses carried pins — no pack load, network, or filesystem lookup inside `Serialize`.
- SoA consumes generic three-state applicability (`Applicable` / `NotApplicable` / `Unresolved`). `Unresolved` is the SoA spelling of `ManualDeterminationRequired`. Not-applicable is justified by organization context, never by missing evidence.

Content modes: `StructuralOnly` | `LicensedContent` | `UserSuppliedContent`.

## Canonical catalog

Versioned, offline, framework-neutral and provider-neutral library:

```text
catalog/canonical/v1/{manifest.toml,controls/,evidence/,tests/}
```

- Schema: `weeping-angel/canonical-catalog/v1` (`CATALOG_SCHEMA`)
- Loader: `weeping-angel-canonical-catalog::CanonicalCatalog::{load,validate,digest,stats,control}` (`load` always validates)
- Default path: `catalog/canonical/v1`. Manifest `[files]` lists participating TOML; extra section `*.toml` and path escape fail closed. `[digest]` in the fixture is documentary.
- Catalog IDs: `control.*` / `evidence.*` / `test.*` (IR newtypes stay permissive)
- Provider/framework segments (`github`, `iso27001`, …) fail closed at the catalog boundary
- Digest display: `wa:canonical-catalog:weeping-angel/canonical-catalog/v1:<hex>` over parsed documents, not raw bytes
- Infrastructure ships fixture IDs `control.source.protected-branch` / `evidence.source.protected-branch` / `test.source.protected-branch`. Domain files are added via the manifest without changing the loader.
- IAM family (Prompt 04): 23 `control.identity.*` controls, 12 `evidence.identity.*` fact types, 23 `test.identity.*` tests in `catalog/canonical/v1/{controls,evidence,tests}/identity.toml`. Tests are population predicates (`coverage-at-least` / `all-subjects` / `none-subjects`), not existence of one envelope. Access-approval, SoD, and periodic review stay hybrid/manual. Prompt 12 remaps ISO Annex A identity/SDLC rows onto these IDs ([`docs/sdd/iso-27001-canonical-remap.md`](../sdd/iso-27001-canonical-remap.md) §13).
- Fixtures: `fixtures/assurance/canonical/v1/identity/` (`healthy-org`, `privileged-without-mfa`, `inactive-admin-active`, `terminated-employee-active`, `service-account-without-owner`, `partial-inventory`, `stale-access-review`, `break-glass-approved-exception`).
- SDLC family (Prompt 05): 26 independently assessable `control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*` controls, 20 fact types (`evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.release.*` / `evidence.supply-chain.*`), and 26 population tests in `catalog/canonical/v1/{controls,evidence,tests}/sdlc.toml`. Population default-branch id is `control.source.default-branch-protection` (the exists-only fixture `control.source.protected-branch` remains). Release authorization, authority separation, security review, and secure-development policy stay hybrid/manual. Missing scan evidence is `InsufficientEvidence`. SSOT: [`docs/sdd/sdlc-canonical-assurance-catalog.md`](../sdd/sdlc-canonical-assurance-catalog.md). Fixtures: `fixtures/assurance/canonical/v1/sdlc/` (`healthy-org`, `degraded-org`, `partial-coverage`, `unprotected-default-branch`, `missing-scan-evidence`, `stale-dependency-scan`, `approved-exception`).
- Infrastructure family (Prompt 07): 43 independently assessable controls with public IDs `control.network.*`, `control.crypto.*`, `control.secret.*`, `control.data.*`, `control.database.*`, `control.logging.*`, `control.backup.*`, and operational `control.resilience.*` in `catalog/canonical/v1/{controls,evidence,tests}/{network,crypto,data,database,logging,backup,resilience}.toml` (`control.secret.*` and `evidence.secret.storage-configuration` live in `crypto.toml`; no `secret.toml`). Continuity-plan / DR **governance** IDs (`control.resilience.business-continuity-plan`, `control.resilience.disaster-recovery-governance`, `evidence.resilience.continuity-plan`) live in `governance.toml`, not `resilience.toml`. Sixteen required operational evidence contracts: `evidence.network.exposure`, `evidence.network.firewall-policy`, `evidence.network.tls-configuration`, `evidence.data.encryption-at-rest`, `evidence.data.encryption-in-transit`, `evidence.crypto.key-state`, `evidence.secret.storage-configuration`, `evidence.database.inventory`, `evidence.database.access-configuration`, `evidence.logging.configuration`, `evidence.logging.retention`, `evidence.logging.alerting`, `evidence.backup.configuration`, `evidence.backup.run`, `evidence.backup.restore-test`, `evidence.resilience.recovery-plan`. Tests are population predicates (`all-subjects` / `none-subjects` / `coverage-at-least` / `manual-review`), not existence of one envelope. Required reusable tests: `test.database.critical-encrypt-at-rest`, `test.network.public-endpoints-acceptable-tls`, `test.logging.critical-assets-audit-current`, `test.logging.retention-meets-threshold`, `test.backup.required-stores-current`, `test.backup.restore-test-fresh`, `test.network.no-prohibited-public-databases`, `test.secret.approved-storage`. Thresholds (`min_days`, `acceptable_min_protocol`, `approved_backends`, `window`) live on `[test.expression]` or `AssessmentContext.max_age`, not Rust ISO/PCI constants. Missing evidence is `InsufficientEvidence`. Partial/unknown population cannot be `Effective` on all-subjects tests. Approved unexpired IR exceptions are `ExceptionApproved` for the bound subject. DR exercise, recovery objectives, and network-segmentation rationale stay hybrid/manual and cannot auto-pass from one technical flag. ISO pack `logging.*` / `encryption.*` / `backup.*` / `security.tls` ids are **not** remapped here. No AWS/Azure/GCP/Cloudflare collector. SSOT: [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](../sdd/infrastructure-canonical-assurance-catalog.md).
- Fixtures: `fixtures/assurance/canonical/v1/network`, `fixtures/assurance/canonical/v1/database`, plus `crypto`, `data`, `logging`, `backup`, `resilience` under the same `canonical/v1` root — `network/{healthy,public-db-exposed,insecure-tls,partial-inventory,stale-firewall-policy,exception-approved-exposure}`, `crypto/{healthy,unapproved-secret-storage,stale-certificate}`, `data/{healthy,partial-classification}`, `database/{healthy,unencrypted-critical-db,partial-inventory,missing-encryption}`, `logging/{healthy,retention-below-threshold,stale-audit-log,missing-alerting,partial-coverage,partial-inventory}`, `backup/{healthy,missing-backup,stale-restore-test,failing-restore}`, `resilience/{healthy,stale-recovery-plan,missing-dr-exercise,exception-approved-rto}`. Clock `2026-08-19T12:00:00Z` on healthy sets.
- Vulnerability family (Prompt 06): 20 `control.vulnerability.*` controls, evidence types `evidence.vulnerability.*` / `evidence.secret.exposure` / `evidence.dependency.*` / `evidence.asset.software-inventory`, and population tests including `test.vulnerability.{scan-current,scan-coverage,no-critical-over-sla,no-high-over-sla,findings-have-owner}`, `test.secret.no-active-exposure`, and `test.dependency.no-critical-over-sla` in `catalog/canonical/v1/{controls,evidence,tests}/vulnerability.toml`. A scanner finding is evidence, not a compliance result. Accepted-risk and approved-exception are not remediation. Empty findings plus unknown coverage are never Effective. SSOT: [`docs/sdd/vulnerability-canonical-assurance-catalog.md`](../sdd/vulnerability-canonical-assurance-catalog.md).
- Fixtures: `fixtures/assurance/canonical/v1/vulnerability/` (`complete-clean-scan`, `critical-inside-sla`, `critical-overdue`, `critical-approved-exception`, `critical-expired-exception`, `incomplete-scan-coverage`, `stale-scan`, `unresolved-secret-exposure`, `duplicate-superseded`, `zero-findings-unknown-coverage`). Clock `2026-08-19T12:00:00Z`; SLA critical 7d / high 30d.
- Governance family (Prompt 08): 34 `control.{governance,risk,personnel,vendor,incident,resilience}` controls (25 Hybrid / 9 Manual; continuity/DR **governance** only; operational restore stays Prompt 07), 13 first-class evidence types (`evidence.manual.attestation` plus `evidence.governance.{policy,policy-review,management-review,internal-audit}`, `evidence.risk.{assessment,treatment}`, `evidence.personnel.{training,acknowledgement}`, `evidence.vendor.{inventory,risk-review}`, `evidence.incident.exercise`, `evidence.resilience.continuity-plan`), and 34 freshness/population/manual-review tests in `catalog/canonical/v1/{controls,evidence,tests}/governance.toml`. Manual evidence is immutable fact, not a boolean bypass. Missing evidence is `InsufficientEvidence`. Partial training/vendor populations cannot be `Effective`. Approved unexpired IR exceptions are `ExceptionApproved`, never silent `Effective`. This family does not remap ISO. SSOT: [`docs/sdd/governance-canonical-assurance-catalog.md`](../sdd/governance-canonical-assurance-catalog.md). ADR: [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md).
- Fixtures: `fixtures/assurance/canonical/v1/governance/` (`current-documents`, `stale-documents`, `missing-documents`, `incomplete-training-population`, `vendor-review-gaps`, `approved-exception`, `expired-exception`, `manual-review-despite-evidence`). Clock `2026-08-18T12:00:00Z` (`stale-documents` uses `2024-08-01T12:00:00Z`).
- No Entra / Okta / Google Workspace collector. GitHub is the first reference-grade provider collector and emits Prompt 04/05 canonical types (`evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.identity.privileged-membership` / `external-access` plus `inventory.subject` / `inventory.complete`). It does not emit `source.*` envelopes. Historical `source.*` strings remain in `GITHUB_EVIDENCE_TYPES` as the ISO GH-012 / IAM-015 mapping table. SSOT: [`docs/sdd/github-collector.md`](../sdd/github-collector.md). Goldens: `fixtures/assurance/canonical/v1/github/`.
- Framework packs are **not** remapped here. Framework crate must not depend on the catalog crate; collector stays catalog-blind.

See [`docs/sdd/canonical-assurance-catalog-v1.md`](../sdd/canonical-assurance-catalog-v1.md), [`docs/sdd/iam-canonical-assurance-catalog.md`](../sdd/iam-canonical-assurance-catalog.md), [`docs/sdd/sdlc-canonical-assurance-catalog.md`](../sdd/sdlc-canonical-assurance-catalog.md), [`docs/sdd/vulnerability-canonical-assurance-catalog.md`](../sdd/vulnerability-canonical-assurance-catalog.md), [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](../sdd/infrastructure-canonical-assurance-catalog.md), [`docs/sdd/governance-canonical-assurance-catalog.md`](../sdd/governance-canonical-assurance-catalog.md), [`docs/sdd/github-collector.md`](../sdd/github-collector.md), [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md), [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md), [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md), [`docs/adr/0003-vulnerability-canonical-assurance-catalog.md`](../adr/0003-vulnerability-canonical-assurance-catalog.md), [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md), [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md), and [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](../adr/0003-github-collector-canonical-evidence-mapping.md).

## Compile

```text
compile_framework(assessment, target) → CompiledFramework | FrameworkCompileError
```

`FrameworkTarget = { profile, capabilities, version, context }`.

**Profiles** (compile selectors, not public catalog crates): `iso-27001`, `iso-27701`, `gdpr`, `soc-2`, `nis-2`, `dora`, `iso-27007`. Unknown string → `UnknownProfileError` / `FrameworkCompileError::UnknownProfile`, not a panic. Production pack content exists for `iso-27001` / `2022` only.

**Capability flags** (default all `false`, fail-closed):

| Flag | Request on `Assessment.requests` |
| --- | --- |
| `supports_control_applicability` | `control_applicability` |
| `supports_statement_of_applicability` | `statement_of_applicability` |
| `supports_privacy_processing` | `privacy_processing` |
| `supports_risk_treatment` | `risk_treatment` |
| `supports_manual_attestation` | `manual_attestation` |
| `supports_sampling` | `sampling` |
| `supports_audit_program` | `audit_program` |
| `supports_nonconformities` | `nonconformities` |

Requested and not supported → `FrameworkCompileError::CapabilityViolation`. The step is not skipped.

Pipeline (recorded on `CompiledFramework.validation.stages`, exact keys):

1. `normalize`
2. `resolve_applicability`
3. `validate_capabilities`
4. `resolve_control_mappings`
5. `resolve_evidence_requirements`
6. `construct_test_plan`
7. `construct_framework_projection`
8. `integrity_validation`

`CompiledFramework` **must** include: `applicableRequirements`, `controls`, `tests`, `evidenceRequirements`, `validation`, `digest`.

Without an organization context, `resolve_applicability` still keeps a requirement unless `statically_applicable() == Some(false)`. With a context, callers consume `weeping-angel-assurance::applicability` (see below) and drop only `NotApplicable`.

Additional compile errors from packs: `UnknownPack`, `UnknownRequirement`, dangling / unsupported / empty-rationale mappings.

## Applicability engine

Generic, network-free Kleene evaluator in `weeping-angel-assurance::applicability`. No provider or framework branches. IR `ApplicabilityRule` stays declarative (`statically_applicable` is the no-context fold). Tiny IR addition: `Control::subjects()`.

```text
build_applicability_context(definition, extras) → ApplicabilityContext
evaluate_applicability(rule, context) → ApplicabilityOutcome
evaluate_assessment_applicability(definition, context) → ApplicabilitySnapshot
```

`ApplicabilityContext` is a **derived view** of IR inventories + `AssessmentScope` / exclusions + optional `ContextExtras` (explicit `FactValue`, per-family `InventoryCompleteness`, pack-entry artifacts). Empty unmarked inventory is `Unknown`, not authoritative false. Explicit facts win over inference.

```text
FactValue             = true | false | unknown
ApplicabilityDecision = applicable | notApplicable | manualDeterminationRequired
```

`Not(unknown)` stays `unknown`. Unknown facts are never treated as false. `ApplicabilityDecision::remains_in_compiled_set` is false only for `notApplicable`. SoA `Applicability::unresolved` is the projection alias of `manualDeterminationRequired`. `project_soa(framework, version)` consumes pack applicability rows as generic three-state results (not a boolean copy).

`ApplicabilityOutcome` carries ordered rationale, predicate traces, named unknown facts, lex-sorted `selectedSubjects`, and `excludedSubjects` (`id`, `reason`, `exclusionIndex`). Zero selected subjects does **not** flip the decision to `notApplicable`. Hand selected ids to population evaluation via `EvidenceSet::set_population`.

Snapshot schema `weeping-angel/applicability-snapshot/v1` (`APPLICABILITY_SNAPSHOT_SCHEMA`):

```text
schema, assessmentId, scope,
requirementDecisions[], controlDecisions[],
packEntries[], digest
```

`packEntries` are artifacts, not Kleene inputs. Digest is IR `canonical_digest` over the body excluding `digest`. This engine **produces** the Kleene snapshot. Lineage persist/explain is landed (see below); crate-root `ApplicabilitySnapshot` is the lineage persist document, not this type.

See [`docs/sdd/applicability-engine.md`](../sdd/applicability-engine.md) and [ADR 0003 applicability engine](../adr/0003-applicability-engine.md).

## Evidence

Collectors and the scanner bridge emit **observations**, never compliance.

Allowed: `"repository X has branch_protection enabled"`, `"route Y is exposed_without_auth"`.

Forbidden (seal / collect error): `"ISO 27001 compliant"`, `"GDPR compliant"`, `"SOC 2 compliant"`, `"NIS2 compliant"`, `"DORA compliant"`, `ControlTestResult`.

`EvidenceEnvelope` is immutable once sealed. Mutation is a new envelope. Digest covers observation + provenance. Same payload+provenance → same digest.

Envelope (`evidence/v1`):

```text
evidenceId, schemaVersion, observation, provenance, digest,
artifactRef?, collectionRunId, contentDigest, sensitivity, scope, supersedes?
```

`provenance = { collectorId, collectedAt, scope, asset }`.

Payload facts MUST NOT use credential keys (`authorization`, `token`, `cookie`, `password`, `api_key`, `apikey`, `secret`, `access_token`, `refresh_token`, `private_key`), including nested `Object` keys. Nested objects MUST NOT use the reserved key `$evidenceValue`.

Facts are `BTreeMap<String, EvidenceValue>` (`evidence-value/v1` hybrid JSON). One type in `weeping-angel-evidence`; control-test re-exports it. No `f64`. No stored `Null`.

| Variant | Canonical JSON | Identity notes |
| --- | --- | --- |
| `String` | JSON string | `"true"` / `"01"` / `"1.0"` stay strings |
| `Bool` | JSON boolean | `true` ≠ `"true"` |
| `Integer` | JSON number, no fraction | `i64` only |
| `StringList` | JSON array of strings | order is identity; `[]` valid |
| `Object` | JSON object | `BTreeMap` key order; `{}` valid |
| `Decimal` | `{"$evidenceValue":"decimal","value":"<text>"}` | lexical (`1.0` ≠ `1.00`) |
| `Timestamp` | `{"$evidenceValue":"timestamp","value":"<rfc3339>"}` | UTC `YYYY-MM-DDTHH:MM:SS.sssZ` |
| `DurationSeconds` | `{"$evidenceValue":"durationSeconds","value":<u64>}` | number, not string |

`with_fact(key, string)` stores `String` (collector compatibility; historical string envelopes keep the same digest). `with_value` is the typed constructor. `fact()` returns `&str` only for `String`; evaluators use `fact_value()`.

```text
branch_protected   = Bool(true)
required_reviewers = Integer(2)
retention_days     = Integer(365)
privileged_roles   = StringList(["owner", "admin"])
```

The evaluator compares stored types (`typed_eq` / `cmp_numeric` / `list_contains`) and fails closed on a `type mismatch`. It does not reparse `"01"` / `"1.0"` / `"true"`. Integer↔Decimal numeric compare is exact decimal-string scale-align, never IEEE-754. Same semantic facts + provenance ⇒ same `canonical_digest` regardless of map insertion order. See [`docs/sdd/typed-evidence.md`](../sdd/typed-evidence.md) and [ADR 0003 typed evidence](../adr/0003-typed-evidence-canonical-serialization.md).

### Ledger

`EvidenceLedger` (SQLite file or in-memory) owns **observations**, never conclusions.

```text
append, get, query, latest, for_subject, for_type,
for_collection_run, within_window, supersede, record_collection_run,
persist_assessment_run, load_assessment_run,
persist_control_test_run, load_control_test_run,
persist_framework_snapshot, load_framework_snapshot
```

Forbidden: `set_compliant`, `set_control_status`.

`append` is idempotent by digest (EVD-002). `supersede` records history via `supersedes`; it does not mutate the previous envelope.

Lineage persist APIs store **opaque JSON**. A second write of different bytes for the same assessment / control-test / snapshot key is `LedgerError::Immutable`. Identical bytes are idempotent. `framework_snapshots` is digest-keyed and may hold pack, catalog, definition, applicability, evidence, readiness, or SoA payloads. `record_collection_run` remains `INSERT OR REPLACE` for in-flight collection identity.

`CollectionRun` = `{ runId, collectorId, collectorVersion, startedAt, completedAt, scope, status, evidenceCount, errorCount, configurationDigest }`.

`EvidenceArtifactRef` = `{ artifactId, digest, mediaType, size, storageLocator, redactionState }`.

## Collectors

```text
trait EvidenceCollector {
    fn descriptor(&self) -> CollectorDescriptor;
    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError>;
}
```

The trait is synchronous. `CollectionRequest` / `CollectionBatch` exist for run provenance. Do not add mandatory `Send + Sync` bounds for WASM/runtime compatibility.

`CollectorDescriptor = { id, version, evidenceTypes, providerFamily, subjectTypes, capabilities, requiredPermissions }`. **`frameworks` is invalid** — the field must not exist.

`CollectorCapabilities`: `incremental`, `pagination`, `historical`, `point_in_time`, `event_driven`, `sensitive_artifacts`, `offline`, `worker_safe`.

Rules (COL-001…006):

| ID | Rule |
| --- | --- |
| COL-001 | Emit only declared `evidenceTypes`. |
| COL-002 | No framework results (`control_test_result`, compliance sentences). |
| COL-003 | No credentials in payloads. |
| COL-004 | Deterministic normalize (fixture collector uses a fixed `collectedAt`). |
| COL-005 | Retry does not duplicate: `EvidenceSet` / ledger is keyed by digest. |
| COL-006 | Out-of-scope asset → `CollectorError::OutOfScope` (never silent collect). |

Shipped implementations:

| Type | Descriptor id | Notes |
| --- | --- | --- |
| `FixtureCollector` | caller | Tests / golden fixtures |
| `GitHubCollector` | `collector.github` | First reference-grade provider collector. Emits canonical evidence only; provider JSON stays inside `github/`. |
| `LocalCollector` | `collector.local` | Structural files only (`CODEOWNERS`, policy, workflow presence). Presence ≠ effectiveness. |
| `ManualEvidence` | `collector.manual` | Requires `--attested-by` / `attested_by`. Never synthesized. Emits legacy `manual_attestation`, not catalog `evidence.manual.attestation`. |

GitHub (`collector.github`, [ADR](../adr/0003-github-collector-canonical-evidence-mapping.md)):

- Emitted evidence types (`CollectorDescriptor.evidence_types` / `GITHUB_CANONICAL_EVIDENCE_TYPES`): `evidence.repository.{inventory,visibility,default-branch,branch-protection,review-policy,review-ownership,security-scanning,dependency-scanning,commit-signing}`, `evidence.cicd.{status-checks,workflow-permissions}`, `evidence.deployment.environment-protection`, `evidence.identity.{privileged-membership,external-access}`, `inventory.subject`, `inventory.complete`. Typed facts via `with_value`. No `evidence.github.*`.
- Compatibility table `GITHUB_EVIDENCE_TYPES` still lists historical ADR 0002 `source.*` names (ISO GH-012 needles; IAM-015 forbids `evidence.identity.*` on that const). Those names are **not** emitted types.
- Subject types: `repository`, `branch`, `organization`, `identity`, `deployment`. `providerFamily`: `source-control`.
- Capabilities: `pagination=true` (Link / `per_page` walker; `inventory.complete` `authoritative=true` only after a complete walk). `incremental=false`.
- Required permissions: `contents:read`, `metadata:read`, `administration:read`, `actions:read`, `members:read`, `security_events:read`.
- Scope: `org:{login}` and `repo:owner/name` (comma-lists); GitHub-owned `exclude_archived`. Protection uses the repo `default_branch`.
- HTTP 401/403 → `PermissionDenied` / insufficient-evidence diagnostic (downstream `InsufficientEvidence`), never boolean false; batch continues (`partial`). 404 on protection/ruleset → observed `protected=false`. 404 on repo → insufficient. 429 → fixture retry then partial; never a boolean. Failure behavior is `GITHUB_FAILURE_BEHAVIOR` (not a shared descriptor field).
- `collect_batch` records a filled `CollectionRun` (version, scope, secret-free configuration digest, start/completion, counts, `complete`/`partial`/`failed`).
- Tokens (`ghp_`, `gho_`, `github_pat_`, `ghs_`, `ghu_`, `ghr_`, `Bearer`) are redacted from diagnostics and never persisted. Shared `redact` plus GitHub `sanitize_diagnostic`.

Hard flow: `Provider → Evidence → Canonical Test → Canonical Control → Framework`. Never `GitHub → ISO check`.

## Control tests

```text
evaluate(CompiledControlTest, EvidenceSet, AssessmentContext) → ControlTestResult
```

Zero network I/O. Signature has no provider / collector id. `EvidenceSet` is provider-blind.

`Effectiveness`: `effective` \| `ineffective` \| `partiallyEffective` \| `notApplicable` \| `notTested` \| `insufficientEvidence` \| `staleEvidence` \| `manualReviewRequired` \| `exceptionApproved` \| `inconclusive`.

| Situation | Result |
| --- | --- |
| Fresh observation matching required types / `TestExpr` | `effective` |
| Observation in `breakOn` (e.g. `exposed_without_auth`) | `ineffective` |
| Missing required evidence / empty set / no required types | `insufficientEvidence` (never `effective`) |
| “No vulns” / `security_findings_absent` | not `effective` |
| Stale evidence (`now - collectedAt > maxAge` or `FreshWithin` fail) | `staleEvidence` or `inconclusive` (never `effective`) |
| Manual test without `manual_attestation` | `insufficientEvidence` / `manualReviewRequired` (cannot auto-pass) |
| Catalog `op = "manual-review"` (`TestExpr::ManualReview`) | `manualReviewRequired` even when a supporting document envelope exists |
| Type mismatch on a field predicate | fail-closed (not `effective`) |
| Approved unexpired IR `Exception` bound to remaining subjects; no residual fail/missing/stale/technical | `exceptionApproved` (never silent `effective`) |
| Expired / revoked exception | does not suppress fail / missing |

Bounded `TestExpr` (not a script host): `Exists`, `Missing`, `Eq`/`Neq`/`Gt`/`Gte`/`Lt`/`Lte`, `Contains`/`NotContains`, `In`, `Count`, `CountWhere`, `FreshWithin`, `CoverageAtLeast`, `CoverageExactly`, `AllSubjects`, `AnySubject`, `NoneSubjects`, `MissingSubjects`, `All`/`Any`/`None`/`Not`, `ManualReview`.

`CoverageAtLeast` is **not** a placeholder. Population arms resolve a deterministic `Population { selector, subjectIds, authoritative, observedAt, completeness }` (`authoritative` \| `partial` \| `unknown`). Resolution: explicit `EvidenceSet` population → closed selector `ids` → identity / `inventory.subject` + `inventory.complete` → else inferred observations (**Unknown**). Unknown completeness cannot yield strong all-subject `Effective`. Partial completeness on those arms is `insufficientEvidence`.

Coverage arithmetic (excepted subjects leave the denominator):

- `evaluated` = `passing + failing`
- `coverage` = `evaluated / P` when `P > 0` and completeness is not `unknown`; otherwise omitted
- pessimistic = `passing / P`; optimistic = `(passing + missing + stale) / P`
- `percentage` is a percent in `[0, 100]` (`"95"` / `"95%"`). `"0.95"` is 0.95%, not 95%.

`CoverageAtLeast(t)`: unknown → `inconclusive`; optimistic `< t` → `ineffective`; pessimistic `< t ≤` optimistic → `insufficientEvidence`; stale as the deciding defect → `staleEvidence`; pessimistic `≥ t` → `effective` (residual failures allowed only when `t < 1`). Authoritative empty population is `insufficientEvidence`, never `effective`. After that conclude, `evaluate_coverage` still promotes `ineffective` → `exceptionApproved` when every remaining failing subject is identity break-glass, and promotes remaining-all-pass `effective` → `exceptionApproved` when `exceptedSubjects` is non-empty and failing/missing/stale/technical are empty (approved unexpired bound IR `Exception`; never silent `effective`).

Results may include nested `population` (`PopulationEvaluation`): `population`, `evaluated`, `passing`, `failing`, `missing`, `coverage?`, `failingSubjects`, `missingSubjects`, `staleSubjects`, `exceptedSubjects`, `technicalSubjects`. Missing evidence, explicit fail, stale evidence, and technical failure stay distinct.

Evaluation indexes envelopes by `(evidenceType, subject)` (`EvidenceIndex`). Latest / `supersedes` wins; digest-identical duplicates count once.

`EvidenceSelector = { evidenceType, subjectSelector, field, freshness }`. No collector id in test definitions. Control-test `subjectSelector` JSON `{ kind, id }` folds `id` into IR `ids`.

`ControlTestResult = { testId, controlId, effectiveness, rationale, evidenceRefs, missingEvidence, evaluatedAt, testVersion, inputDigest, duration?, population? }`. Wall-clock `duration` is not part of semantic identity. Same test + evidence snapshot + evaluation context → same semantic result. `CompiledTest.expr` is an optional JSON `TestExpr`; `evaluate_compiled` attaches it.

Tests are **canonical** (`test.source.required-review`), never `iso27001.a.x.y.github.*`.

## Scanner bridge

Owned by `weeping-angel-assurance::bridge`. Does **not** rewrite `EngineHit::to_semantic_finding`.

| Source | Observation type | Facts |
| --- | --- | --- |
| `EngineHit` | `security_finding` | `rule_id`, `path`, `category`, `canonical_type` + title narrative |
| `SemanticFinding` | `security_finding` | `rule_id`, `finding_id` + title narrative |

`canonical_type` is one of: `security.finding`, `security.vulnerability.present`, `security.exposure.present`, `security.authz.weakness`, `security.secret.exposure`, `security.tls.misconfiguration`, `security.header.misconfiguration`, `security.dependency_confusion_risk`.

One-way. Observations do not write back onto findings. Empty scan ≠ Effective control. Do not emit `security.no_vulnerabilities` as a passable fact. Bridge `security_*` types are **not** the catalog library; catalog evaluation consumes `evidence.vulnerability.*` / `evidence.secret.exposure` / `evidence.dependency.*` (Prompt 06). A later adapter may emit those types from engine hits. Accepted-risk and approved-exception are not remediation.

Security types that remain valid and uncollapsed: `EngineHit`, `SemanticFinding`, `Candidate`, `ArtifactRecord`, `CoverageDocument`. They must not grow `iso27001` / `gdpr` / `soc2` (or siblings).

## Facade

```text
AssuranceEngine::builder()
    .collector(C: EvidenceCollector)
    .framework(FrameworkTarget)
    .assess(AssessmentScope) → AssessmentReport | AssuranceError
```

`AssessmentReport` rust fields: `{ assessmentId, profile, digest, results, evidenceCount }` plus carried `run`, `summary`, `coverageMetrics`, `frameworkPackDigest`, and `canonicalCatalogDigest`. Serialization is **pure**: no `load_framework_pack`, network I/O, filesystem lookup, or hidden current-state resolution. It writes `disclaimer`, `banner`, `resultDigest`, `assessmentRun`, `status`, `collectionRuns`, explicit `summary` / `coverageMetrics`, and lists derived from in-memory results. No `compilerTopology` / `collectorGraph`. No `compliancePercent` / `isoCompliant`.

`assess` loads the pack for the assessed `(profile, version)` via `load_framework_pack`. Missing pack is `AssuranceError::UnknownPack` (fail closed). There is no production stub assessment. Callers must not branch on ISO vs GDPR vs SOC 2 implementations to run a generic assess. Collector failure does not abort the call: `AssessmentRun.status` is `completed` | `partial` | `failed`.

`AssessmentRun` pins `frameworkPackDigest`, `canonicalCatalogDigest`, `assessmentDefinitionDigest`, `applicabilitySnapshotId`, `collectorRuns`, `evidenceSnapshotDigest`, and `resultDigest`. Distinct identities — not the compile digest copied three times. The run is returned on `AssessmentReport.run`; `assess` does not open a ledger.

Projections (not certificates):

```text
project_readiness(...) → FrameworkReadinessSnapshot
project_soa(framework, version) → StatementOfApplicability   # live convenience
project_soa_from_snapshot(snap) → StatementOfApplicability
compare(previous, next) → SnapshotDiff
compare_runs / compare_lineage(previous, next) → SnapshotDiff
explain_control(report, controlId, assessment?, applicability?) → ControlExplanation
reconstruct / replay_assessment(bundle) → AssessmentReport
```

`CoverageMetrics` exposes seven separate families (`controlEffectiveness`, `evidence`, `automation`, `subject`, `frameworkRequirement`, `freshEvidence`, `manualReviewBurden`). Do not collapse them into one compliance percentage.

A requirement mapped only with `PartiallySatisfies` / `Supports` / `Related` / `EvidenceFor` / `SubsetOf` cannot become fully `effective` even if every mapped control is `Effective` (`partially covered`).

## Lineage

Immutable execution documents in `weeping-angel-assurance::lineage` (schema `weeping-angel/assessment-lineage/v1`):

```text
FrameworkPackSnapshot, CanonicalCatalogSnapshot, AssessmentDefinitionSnapshot,
ApplicabilitySnapshot, EvidenceSnapshot, ControlTestRun,
StatementOfApplicabilitySnapshot, LineageBundle,
ControlExplanation, AssessmentSummary, CoverageMetrics
```

Replay uses the pinned bundle only. Consulting current pack/catalog files is allowed solely to compare digests (`verify_current_against_pins`); mismatch is `DigestMismatch`.

Crate-root `ApplicabilitySnapshot` is the persist document (static IR fold + `packEntries` artifacts). The Kleene document remains `weeping-angel-assurance::applicability::ApplicabilitySnapshot` (`weeping-angel/applicability-snapshot/v1`).

Result identity (`assessment_result_digest`) is SHA-256 of canonical JSON over test id, control id, effectiveness, evidence refs, missing evidence, test version, input digest, and population. Wall-clock `duration` / `evaluatedAt` are excluded.

See [`docs/sdd/assessment-lineage.md`](../sdd/assessment-lineage.md) and [ADR 0003 assessment lineage](../adr/0003-assessment-lineage.md).

## CLI

Clap family (no compiler topology in ordinary help):

```text
weeping-angel assurance framework list|validate|show
weeping-angel assurance collect
weeping-angel assurance evidence list|show|add
weeping-angel assurance assess --framework iso-27001 [--scope .] [--github-repo …]
weeping-angel assurance result show
weeping-angel assurance compare
weeping-angel assurance soa
weeping-angel assurance catalog validate|stats|inspect <control-id> [path]
weeping-angel assurance explain --assessment <id> --control <id>
```

`assurance catalog` and `assurance explain` are dispatched (parser in `src/cli.rs`; execution in `src/assurance_catalog.rs` and `src/assurance_explain.rs`). Explain prints the not-certification banner then JSON; unknown assessment or control exits non-zero; it does not resolve a pack from disk. Other `assurance` subcommands still print the non-certification banner and exit 0; library `assess` / `project_readiness` / `project_soa` / `compare` remain the execution path for those.

## Crate graph

See ADR 0001. Public composition root is `weeping-angel-assurance`. Do not take a dependency on compiler internals except for tests / debug.

Hard negatives (Phase 53 / ACT-003 / ACT-013):

```text
assurance-ir        ↛ HTTP / provider / storage / framework adapter logic
framework           ↛ GitHub / AWS / Cloudflare SDKs, collector, control-test, canonical-catalog
collector           ↛ framework / catalog / ISO / GDPR / SOC2 packages; never declares compliance
canonical-catalog   ↛ framework / collector / control-test / evidence / network
evidence            ↛ control effectiveness
control-test        ↛ network I/O / collector
scanner types       ↛ ISO / GDPR / SOC statuses
assurance           = composition authority (includes applicability engine)
applicability eval  ↛ collector / framework adapter / IR fact evaluation / pack-TOML as Kleene input
lineage persist     ↛ current catalog/pack files on serialize or replay; mismatch is DigestMismatch
```

## Tests

| Suite | Cargo name | Status |
| --- | --- | --- |
| Spine target (normative) | `sdd_assurance_runtime_target` | GREEN — ACT-001…015, COL-001…006 |
| Spine baseline (historical) | `sdd_assurance_runtime_baseline` | superseded / ignored |
| ISO target (normative) | `sdd_iso27001_assurance_target` | GREEN — ISO-001…010, EVD-001…010, CTL-001…012, GH-001…012 + MVP assess |
| ISO baseline (historical) | `sdd_iso27001_assurance_baseline` | superseded / ignored |
| ISO remap target (normative) | `sdd_iso27001_remap_target` | GREEN — ISO-R-001…020, catalog-targeted mappings, generic SoA, dual digests |
| ISO remap baseline (historical) | `sdd_iso27001_remap_baseline` | superseded / ignored |
| Catalog target (normative) | `sdd_canonical_assurance_catalog_target` | GREEN — CAT-001…016 |
| Catalog baseline (historical) | `sdd_canonical_assurance_catalog_baseline` | absence asserts superseded / ignored |
| Typed evidence target (normative) | `sdd_typed_evidence_target` | GREEN — digest order, nested objects, typed compare, type-mismatch, credentials, codec, string-fixture compat, ledger |
| Typed evidence baseline (historical) | `sdd_typed_evidence_baseline` | superseded / ignored |
| Population target (normative) | `sdd_population_runtime_target` | GREEN — goldens 1–10, real `CoverageAtLeast`, index fixtures |
| Population baseline (historical) | `sdd_population_runtime_baseline` | placeholder characterization superseded / ignored |
| IAM catalog target (normative) | `sdd_iam_catalog_target` | GREEN — IAM-001…016 (`control.identity.*`, population fixtures) |
| IAM catalog baseline (historical) | `sdd_iam_catalog_baseline` | absence characterization superseded / ignored |
| SDLC catalog target (normative) | `sdd_sdlc_catalog_target` | GREEN — SDLC-001…016 (`control.source|cicd|release|supply-chain.*`, population fixtures) |
| SDLC catalog baseline (historical) | `sdd_sdlc_catalog_baseline` | absence characterization superseded / ignored |
| Infrastructure catalog target (normative) | `sdd_infrastructure_catalog_target` | GREEN — INFRA-001…016 (`control.network.*` / `evidence.database.*`, population fixtures) |
| Infrastructure catalog baseline (historical) | `sdd_infrastructure_catalog_baseline` | absence characterization superseded / ignored |
| Applicability engine target (normative) | `sdd_applicability_engine_target` | GREEN — P10-T01…T16 (Kleene three-state, unknown≠false, snapshot shape) |
| Applicability engine baseline (historical) | `sdd_applicability_engine_baseline` | static-only / IR-declarative characterization; B06/B07/B09 absence asserts superseded / ignored |
| Assessment lineage target (normative) | `sdd_assessment_lineage_target` | GREEN — LIN-001…015 (replay, digest mismatch, explain, pure serialize, partial runs, compare, exceptions, generic loader) |
| Assessment lineage baseline (historical) | `sdd_assessment_lineage_baseline` | dropped-run / serialize-time ISO / stub-assessment characterization superseded / ignored |

`cargo test --workspace --features demo` must keep scanner tests green.

Focused crate checks: `cargo test -p weeping-angel-framework`, `-p weeping-angel-evidence`, `-p weeping-angel-collector`, `-p weeping-angel-control-test`, `-p weeping-angel-assurance`, `-p weeping-angel-canonical-catalog`.
