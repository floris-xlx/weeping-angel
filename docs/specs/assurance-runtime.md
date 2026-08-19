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
- Personnel security lifecycle: [`docs/adr/0003-personnel-security-lifecycle.md`](../adr/0003-personnel-security-lifecycle.md) — additive `personnel.toml` population tests; not an HRIS; `active`/`excessive` are defect flags
- Applicability engine: [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md)
- Assessment lineage: [`docs/adr/0003-assessment-lineage.md`](../adr/0003-assessment-lineage.md)
- Continuous assurance scheduler: [`docs/adr/0005-continuous-assurance-scheduler.md`](../adr/0005-continuous-assurance-scheduler.md) — library `tick` (not clap); failed collect does not erase ledger evidence
- Evidence validity / temporal assurance: [`docs/adr/0003-evidence-validity-temporal-assurance.md`](../adr/0003-evidence-validity-temporal-assurance.md), [`docs/adr/0003-temporal-assurance.md`](../adr/0003-temporal-assurance.md)
- Residual risk: [`docs/adr/0003-residual-risk.md`](../adr/0003-residual-risk.md)
- Control implementation registry: [`docs/adr/0003-control-implementation-registry.md`](../adr/0003-control-implementation-registry.md) — IR `ControlImplementation` organizational state (not `Effectiveness`)
- Controlled documents: [`docs/adr/0003-controlled-documents.md`](../adr/0003-controlled-documents.md), spec [`controlled-documents.md`](controlled-documents.md) — standalone `ControlledDocument` registry; immutable versions; eval-at-T (not `Effectiveness`, not a DMS)
- ISMS events / drift: [`docs/adr/0003-isms-events-drift.md`](../adr/0003-isms-events-drift.md) (sibling notes [`docs/adr/0005-isms-events-drift.md`](../adr/0005-isms-events-drift.md)) — `detect_events` / `detect_isms_drift` observations; not `SnapshotDiff` and not a notification bus
- Incident governance: [`docs/adr/0003-incident-governance.md`](../adr/0003-incident-governance.md) — IR `Incident` on `AssessmentDefinition.incidents` (not SIEM, not catalog `control.incident.*`)
- Internal audit: [`docs/adr/0003-internal-audit.md`](../adr/0003-internal-audit.md) — IR `AuditProgram` / `Audit` on `AssessmentDefinition.audit_programs` / `audits`; machine prepare only; humans accept samples and sign (not “audit passed”)
- ISMS context IR: [`docs/adr/0008-isms-context.md`](../adr/0008-isms-context.md), spec [`isms-context.md`](isms-context.md) — durable `IsmsContext` root (not an assessment result, not a parallel GRC schema)
- Organizational scope engine: [`docs/adr/0008-scope-engine.md`](../adr/0008-scope-engine.md), spec [`scope-engine.md`](scope-engine.md) — `ScopeResolution` (`weeping-angel/scope-resolution/v1`); four-state boundary; not crawl URL scope and not facade `AssessmentScope`
- Interested parties / obligations: [`docs/adr/0008-interested-parties-obligations.md`](../adr/0008-interested-parties-obligations.md), spec [`interested-parties-obligations.md`](interested-parties-obligations.md) — standalone `ObligationRegistry` (why a control/policy exists; not a framework `Requirement`, not collector satisfaction)
- Security objectives: [`docs/adr/0008-security-objectives.md`](../adr/0008-security-objectives.md), spec [`security-objectives.md`](security-objectives.md) — `objectives::SecurityObjective` + `evaluate_objective` over pinned evidence (`weeping-angel/objective-evaluation/v1`); not `Control.objective` prose, not crate-root `isms::SecurityObjective`, and not catalog attestation
- Risk methodology: [`docs/adr/0005-risk-methodology.md`](../adr/0005-risk-methodology.md) — IR `score_risk` / `RiskMethodology` (not Kleene, not collector evidence)
- Operational risk register: [`docs/adr/0005-operational-risk-register.md`](../adr/0005-operational-risk-register.md) — additive `Risk` record, status machine, graph integrity, Prompt 05 / `score_inherent` adapter (not treatment workflow, not control-derived residual)
- Risk treatment: [`docs/adr/0006-risk-treatment-engine.md`](../adr/0006-risk-treatment-engine.md) — IR `RiskTreatmentDecision` / immutable `RiskAcceptance` (not compile, not `RiskStatus::Accepted` as evidence)
- Supplier risk: [`docs/adr/0007-supplier-risk.md`](../adr/0007-supplier-risk.md) — operational IR `Vendor` lifecycle (not catalog `control.vendor.*`, not Kleene `HasVendor` criticality)
- Remediation engine: [`docs/adr/0003-remediation-engine.md`](../adr/0003-remediation-engine.md) — IR `Remediation` / `create_from_control_regression` (adapter ticket refs only; not workbench `RemediationRequest`)
- ISO remap onto catalog: [`docs/adr/0003-iso27001-canonical-remap.md`](../adr/0003-iso27001-canonical-remap.md)
- Operational Statement of Applicability: [`docs/adr/0003-operational-soa.md`](../adr/0003-operational-soa.md) — graph projection, NA governance, immutable snapshots/diffs (not pack-TOML `assessed` copy)
- GitHub collector mapping: [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](../adr/0003-github-collector-canonical-evidence-mapping.md)
- Risk identification (candidates, not register rows): [`docs/adr/0007-risk-identification-candidate-correlation.md`](../adr/0007-risk-identification-candidate-correlation.md) — `RiskCandidate` + deterministic correlate; `promote_candidate` is the only insert into `AssessmentDefinition.risks`
- Continuity / resilience capability: [`docs/adr/0005-continuity-resilience.md`](../adr/0005-continuity-resilience.md), spec [`continuity-resilience.md`](continuity-resilience.md) — `evaluate_continuity_resilience` over `AssetKind::Service` profiles; a plan document is not demonstrated recovery

Public composition root is `weeping-angel-assurance`. Callers select a profile + capabilities; they do not import per-regime adapters.

This is an **automated readiness/assurance** contract, not a certification authority. Automated evaluation must never emit `ISO 27001 certified`, `ISO 27001 compliant`, `certification guaranteed`, or `audit passed`.

Allowed language: `ready`, `applicable`, `not applicable`, `unresolved`, `not implemented`, `effective`, `ineffective`, `insufficient evidence`, `stale evidence`, `requires manual review`, `readiness gap`, `assessment coverage`, `partially covered`.

## Schema

| Item | Value |
| --- | --- |
| IR schema | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Evidence schema | `evidence/v1` (`EVIDENCE_SCHEMA`) |
| Evidence validity schema | `evidence-validity/v1` (`EVIDENCE_VALIDITY_SCHEMA`) — append-only usability events; not part of `DigestBody` |
| Evidence value encoding | `evidence-value/v1` (`EVIDENCE_VALUE_SCHEMA`) — hybrid JSON inside observation facts |
| Framework pack schema | `weeping-angel/framework-pack/v1` (`FRAMEWORK_PACK_SCHEMA`) |
| Canonical catalog schema | `weeping-angel/canonical-catalog/v1` (`CATALOG_SCHEMA`) |
| Applicability snapshot schema | `weeping-angel/applicability-snapshot/v1` (`APPLICABILITY_SNAPSHOT_SCHEMA`) — Kleene document in `::applicability` |
| Lineage snapshot schema | `weeping-angel/assessment-lineage/v1` (`LINEAGE_SNAPSHOT_SCHEMA`) — persist / replay documents in `::lineage` |
| Objective evaluation snapshot | `weeping-angel/objective-evaluation/v1` (`OBJECTIVE_EVALUATION_SCHEMA`) — `evaluate_objective` lineage; not a collector fact |
| Scope resolution snapshot | `weeping-angel/scope-resolution/v1` (`SCOPE_RESOLUTION_SCHEMA`) — `resolve_scope` output; not IR and not crawl URL scope |
| Operational SoA input schema | `weeping-angel/operational-soa-input/v1` (`OPERATIONAL_SOA_INPUT_SCHEMA`) — projection input in `::soa` |
| Risk-treatment ref schema | `weeping-angel/risk-treatment-ref/v1` — minimum pin; not the treatment engine |
| Risk-register ref schema | `weeping-angel/risk-register-ref/v1` — minimum pin; not the register |
| ISMS event schema | `weeping-angel/isms-event/v1` (`ISMS_EVENT_SCHEMA`) — immutable observations from `detect_events`; not a ticket and not `SnapshotDiff` |
| JSON | serde `camelCase` on public documents |
| Digests | SHA-256 hex of `serde_json` bytes (struct field order; `BTreeMap`/`BTreeSet` for maps/sets) |
| Catalog digest | Display `wa:canonical-catalog:weeping-angel/canonical-catalog/v1:` + SHA-256 hex of parsed documents (`DIGEST_PREFIX` + IR `canonical_digest`; prefix not mixed into the hash) |

Every IR document (`Control`, `Requirement`, `Mapping`, `EvidenceRequirement`, `PlannedControlTest`, `IsmsContext`, `RiskMethodology`, `RiskTreatmentDecision`, `Remediation`, `AuditProgram`, `Audit`, `AuditFinding`, `ControlledDocument`, `ObligationRegistry`, `objectives::SecurityObjective`) carries `schemaVersion`. Compile rejects any other version. `IsmsContext` is a durable management-system definition (`assurance-ir/v1`); `AssessmentDefinition` remains point-in-time compile/assessment input with optional `isms_context_id` and additive `risk_treatments` / `remediations` / `incidents` / `continuity_profiles` / `audit_programs` / `audits` / `audit_findings` (default empty). `ControlledDocument` lives on a standalone `DocumentControlRegistry` — not an `AssessmentDefinition` field. `ObligationRegistry` is a standalone obligation document (not an assessment inventory). `Assessment` remains a **framework-crate** in-memory document compiled by `compile_framework` (not an IR type).

## Identity

Newtypes, stable string form, no random v4 in persisted identity:

`FrameworkId`, `FrameworkVersion`, `RequirementId`, `ControlId`, `ControlImplementationId`, `ControlTestId`, `AssetId`, `IdentityId`, `VendorId`, `SupplierReviewId`, `SupplierRequirementId`, `SupplierIssueId`, `ProcessingActivityId`, `EvidenceRequirementId`, `RiskId`, `RiskCandidateId`, `PromotionId`, `DismissalId`, `RiskMethodologyId`, `RiskTreatmentId`, `RiskAcceptanceId`, `TreatmentPlanId`, `TreatmentActionId`, `ResidualRiskId`, `ExceptionId`, `AssessmentId`, `IsmsContextId`, `OrganizationId`, `BusinessUnitId`, `ScopeId`, `IssueId`, `InterestedPartyId`, `ObligationId`, `RequirementSourceId`, `ObligationMappingId`, `ControlledDocumentId`, `ObjectiveId`, `SecurityObjectiveId`, `ObjectiveMetricId`, `ObjectiveTargetId`, `ObjectiveMeasurementId`, `AuditProgramId`, `AuditId`, `AuditFindingId`, `IncidentId`, `ContinuityProfileId`, `ContinuityExerciseId`, `RecoveryObjectiveId`, `FindingRef`, `AlertRef`, `EventRef`, `EventId`, `RemediationRef`, `RemediationId`, `RemediationActionId`, `SlaPolicyId`, `EvidenceType`.

ISMS scoring lives in `weeping-angel-assurance-ir` as versioned `RiskMethodology` documents plus pure `score_risk` (see **Risk methodology** below). Collectors still emit facts only. Treatment decisions (`RiskTreatmentDecision` / sealed `RiskAcceptance`) are a separate IR inventory — see **Risk treatment** below.

`EvidenceType` names a **fact kind**. Canonical names used by this vertical include the SDLC family `evidence.repository.*` (`inventory`, `visibility`, `default-branch`, `branch-protection`, `review-policy`, `review-ownership`, `security-scanning`, `dependency-scanning`, `commit-signing`, `change-trace`, `security-review`, `secure-development-policy`), `evidence.cicd.*` (`workflow-permissions`, `status-checks`), `evidence.deployment.environment-protection`, `evidence.release.authorization`, `evidence.supply-chain.*` (`build-provenance`, `artifact-integrity`, `lockfile-state`, `component-support`), generic population envelopes `inventory.subject` / `inventory.complete`, historical ISO-sliver strings `source.branch.protection` / `source.branch.required_reviews` / `source.codeowners.present` (compatibility needles, **not** newly emitted GitHub types), `security_finding`, `manual_attestation` (legacy pack / capability / `collector.manual` envelope — **not** the catalog type), the IAM family `evidence.identity.*` (`inventory`, `authentication-state`, `mfa-status`, `privileged-membership`, `role-membership`, `last-active`, `account-status`, `account-owner`, `access-review`, `lifecycle-event`, `service-account`, `external-access`), the infrastructure family `evidence.network.*` (`exposure`, `firewall-policy`, `tls-configuration`), `evidence.data.*` (`encryption-at-rest`, `encryption-in-transit`), `evidence.crypto.key-state`, `evidence.secret.storage-configuration` (storage class — **not** `evidence.secret.exposure`), `evidence.database.*` (`inventory`, `access-configuration`), `evidence.logging.*` (`configuration`, `retention`, `alerting`), `evidence.backup.*` (`configuration`, `run`, `restore-test`), `evidence.resilience.recovery-plan` (operational restore; continuity **governance** is `evidence.resilience.continuity-plan`), the vulnerability family `evidence.vulnerability.*` (`finding`, `scan-run`, `scan-coverage`, `remediation-state`, `owner`, `exception`, `exposure-review`), `evidence.secret.exposure`, `evidence.dependency.*` (`vulnerability`, `confusion-risk`), `evidence.asset.software-inventory`, and the governance family `evidence.manual.attestation` plus `evidence.governance.*` (`policy`, `policy-review`, `management-review`, `internal-audit`), `evidence.risk.*` (`assessment`, `treatment`), `evidence.personnel.*` (`training`, `acknowledgement`, `screening`, `joiner-grace`, `population-membership`, `asset-return`), `evidence.vendor.*` (`inventory`, `risk-review`), `evidence.incident.exercise`, `evidence.resilience.continuity-plan` (plan/DR **governance**; operational restore stays `evidence.resilience.recovery-plan`). Scanner-bridge `security_*` types (`security_finding`, `security.vulnerability.present`, `security.secret.exposure`, `security.dependency_confusion_risk`) remain the **bridge taxonomy**, not the catalog library. It is not a framework name and must not be prefixed `github.*` / `iso27001.*` unless the provider or regime is genuinely part of the fact.

`SubjectKind` (IR SSOT): `organization`, `asset`, `repository`, `service`, `identity`, `user`, `privilegedIdentity`, `device`, `vendor`, `dataset`, `processingActivity`, `branch`, `application`, `database`, `cloudAccount`, `cloudResource`, `serviceAccount`, `endpoint`, `dataStore`, `network`, `deployment`, plus generic (not provider) `businessUnit`, `location`, `dataDomain`, `personnelPopulation`.

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
- SoA consumes generic three-state applicability (`Applicable` / `NotApplicable` / `Unresolved`). `Unresolved` is the SoA spelling of `ManualDeterminationRequired`. Not-applicable is justified by organization context, never by missing evidence. Pack `applicability.toml` is default/structural flags only. Operational projection, NA approval, and snapshot history: [`docs/specs/operational-soa.md`](operational-soa.md).

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
- IAM family (IAM catalog): 23 `control.identity.*` controls, 12 `evidence.identity.*` fact types, 23 `test.identity.*` tests in `catalog/canonical/v1/{controls,evidence,tests}/identity.toml`. Tests are population predicates (`coverage-at-least` / `all-subjects` / `none-subjects`), not existence of one envelope. Access-approval, SoD, and periodic review stay hybrid/manual. ISO remap remaps ISO Annex A identity/SDLC rows onto these IDs ([`docs/specs/iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) §13).
- Fixtures: `fixtures/assurance/canonical/v1/identity/` (`healthy-org`, `privileged-without-mfa`, `inactive-admin-active`, `terminated-employee-active`, `service-account-without-owner`, `partial-inventory`, `stale-access-review`, `break-glass-approved-exception`).
- SDLC family (SDLC catalog): 26 independently assessable `control.source.*` / `control.cicd.*` / `control.release.*` / `control.supply-chain.*` controls, 20 fact types (`evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.release.*` / `evidence.supply-chain.*`), and 26 population tests in `catalog/canonical/v1/{controls,evidence,tests}/sdlc.toml`. Population default-branch id is `control.source.default-branch-protection` (the exists-only fixture `control.source.protected-branch` remains). Release authorization, authority separation, security review, and secure-development policy stay hybrid/manual. Missing scan evidence is `InsufficientEvidence`. SSOT: [`docs/specs/sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md). Fixtures: `fixtures/assurance/canonical/v1/sdlc/` (`healthy-org`, `degraded-org`, `partial-coverage`, `unprotected-default-branch`, `missing-scan-evidence`, `stale-dependency-scan`, `approved-exception`).
- Infrastructure family (infrastructure catalog): 43 independently assessable controls with public IDs `control.network.*`, `control.crypto.*`, `control.secret.*`, `control.data.*`, `control.database.*`, `control.logging.*`, `control.backup.*`, and operational `control.resilience.*` in `catalog/canonical/v1/{controls,evidence,tests}/{network,crypto,data,database,logging,backup,resilience}.toml` (`control.secret.*` and `evidence.secret.storage-configuration` live in `crypto.toml`; no `secret.toml`). Continuity-plan / DR **governance** IDs (`control.resilience.business-continuity-plan`, `control.resilience.disaster-recovery-governance`, `evidence.resilience.continuity-plan`) live in `governance.toml`, not `resilience.toml`. Sixteen required operational evidence contracts: `evidence.network.exposure`, `evidence.network.firewall-policy`, `evidence.network.tls-configuration`, `evidence.data.encryption-at-rest`, `evidence.data.encryption-in-transit`, `evidence.crypto.key-state`, `evidence.secret.storage-configuration`, `evidence.database.inventory`, `evidence.database.access-configuration`, `evidence.logging.configuration`, `evidence.logging.retention`, `evidence.logging.alerting`, `evidence.backup.configuration`, `evidence.backup.run`, `evidence.backup.restore-test`, `evidence.resilience.recovery-plan`. Tests are population predicates (`all-subjects` / `none-subjects` / `coverage-at-least` / `manual-review`), not existence of one envelope. Required reusable tests: `test.database.critical-encrypt-at-rest`, `test.network.public-endpoints-acceptable-tls`, `test.logging.critical-assets-audit-current`, `test.logging.retention-meets-threshold`, `test.backup.required-stores-current`, `test.backup.restore-test-fresh`, `test.network.no-prohibited-public-databases`, `test.secret.approved-storage`. Thresholds (`min_days`, `acceptable_min_protocol`, `approved_backends`, `window`) live on `[test.expression]` or `AssessmentContext.max_age`, not Rust ISO/PCI constants. Missing evidence is `InsufficientEvidence`. Partial/unknown population cannot be `Effective` on all-subjects tests. Approved unexpired IR exceptions are `ExceptionApproved` for the bound subject. DR exercise, recovery objectives, and network-segmentation rationale stay hybrid/manual and cannot auto-pass from one technical flag. ISO pack `logging.*` / `encryption.*` / `backup.*` / `security.tls` ids are **not** remapped here. No AWS/Azure/GCP/Cloudflare collector. SSOT: [`docs/specs/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md).
- Fixtures: `fixtures/assurance/canonical/v1/network`, `fixtures/assurance/canonical/v1/database`, plus `crypto`, `data`, `logging`, `backup`, `resilience` under the same `canonical/v1` root — `network/{healthy,public-db-exposed,insecure-tls,partial-inventory,stale-firewall-policy,exception-approved-exposure}`, `crypto/{healthy,unapproved-secret-storage,stale-certificate}`, `data/{healthy,partial-classification}`, `database/{healthy,unencrypted-critical-db,partial-inventory,missing-encryption}`, `logging/{healthy,retention-below-threshold,stale-audit-log,missing-alerting,partial-coverage,partial-inventory}`, `backup/{healthy,missing-backup,stale-restore-test,failing-restore}`, `resilience/{healthy,stale-recovery-plan,missing-dr-exercise,exception-approved-rto}`. Clock `2026-08-19T12:00:00Z` on healthy sets.
- Vulnerability family (vulnerability catalog): 20 `control.vulnerability.*` controls, evidence types `evidence.vulnerability.*` / `evidence.secret.exposure` / `evidence.dependency.*` / `evidence.asset.software-inventory`, and population tests including `test.vulnerability.{scan-current,scan-coverage,no-critical-over-sla,no-high-over-sla,findings-have-owner}`, `test.secret.no-active-exposure`, and `test.dependency.no-critical-over-sla` in `catalog/canonical/v1/{controls,evidence,tests}/vulnerability.toml`. A scanner finding is evidence, not a compliance result. Accepted-risk and approved-exception are not remediation. Empty findings plus unknown coverage are never Effective. SSOT: [`docs/specs/vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md).
- Fixtures: `fixtures/assurance/canonical/v1/vulnerability/` (`complete-clean-scan`, `critical-inside-sla`, `critical-overdue`, `critical-approved-exception`, `critical-expired-exception`, `incomplete-scan-coverage`, `stale-scan`, `unresolved-secret-exposure`, `duplicate-superseded`, `zero-findings-unknown-coverage`). Clock `2026-08-19T12:00:00Z`; SLA critical 7d / high 30d.
- Personnel security lifecycle (Prompt 17): six additive controls / four evidence types / six tests in `catalog/canonical/v1/{controls,evidence,tests}/personnel.toml` listed in the canonical manifest. The five governance `control.personnel.*` rows stay in `governance.toml` (GOV-003 family count 40, still 30–45). Lifecycle tests are population predicates (`all-subjects` / `none-subjects`) over `evidence.personnel.{screening,joiner-grace,population-membership,asset-return}` plus reused `evidence.identity.{lifecycle-event,account-status,role-membership}`. Fixture facts `account-status.active` and `role-membership.excessive` are **defect flags** (truthy → fail on `none-subjects`). `Identity` / `SubjectKind` stay thin (no Employee/Contractor). Resolution stays `resolve_population` (no `resolve_personnel_inventory`). Joiner grace is a `within_grace` fact, not an IR Exception. Eight fixtures: `fixtures/assurance/canonical/v1/personnel/{complete-training-population,one-overdue-user,new-joiner-grace,leaver-with-active-access,mover-retaining-excessive-privileges,expired-exception,missing-personnel-source,manual-screening-evidence}/`. Dual-suite `sdd_personnel_security_target` GREEN. SSOT: [`docs/specs/personnel-security.md`](personnel-security.md). ADR: [`docs/adr/0003-personnel-security-lifecycle.md`](../adr/0003-personnel-security-lifecycle.md).
- Governance family (governance catalog): 34 `control.{governance,risk,personnel,vendor,incident,resilience}` controls (25 Hybrid / 9 Manual; continuity/DR **governance** only; operational restore stays infrastructure catalog), 13 first-class evidence types (`evidence.manual.attestation` plus `evidence.governance.{policy,policy-review,management-review,internal-audit}`, `evidence.risk.{assessment,treatment}`, `evidence.personnel.{training,acknowledgement}`, `evidence.vendor.{inventory,risk-review}`, `evidence.incident.exercise`, `evidence.resilience.continuity-plan`), and 34 freshness/population/manual-review tests in `catalog/canonical/v1/{controls,evidence,tests}/governance.toml`. Manual evidence is immutable fact, not a boolean bypass. Missing evidence is `InsufficientEvidence`. Partial training/vendor populations cannot be `Effective`. Approved unexpired IR exceptions are `ExceptionApproved`, never silent `Effective`. This family does not remap ISO. SSOT: [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md). ADR: [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md).
- Fixtures: `fixtures/assurance/canonical/v1/governance/` (`current-documents`, `stale-documents`, `missing-documents`, `incomplete-training-population`, `vendor-review-gaps`, `approved-exception`, `expired-exception`, `manual-review-despite-evidence`). Clock `2026-08-18T12:00:00Z` (`stale-documents` uses `2024-08-01T12:00:00Z`).
- No Entra / Okta / Google Workspace collector. GitHub is the first reference-grade provider collector and emits IAM/SDLC catalogs canonical types (`evidence.repository.*` / `evidence.cicd.*` / `evidence.deployment.*` / `evidence.identity.privileged-membership` / `external-access` plus `inventory.subject` / `inventory.complete`). It does not emit `source.*` envelopes. Historical `source.*` strings remain in `GITHUB_EVIDENCE_TYPES` as the ISO GH-012 / IAM-015 mapping table. SSOT: [`docs/specs/github-collector.md`](github-collector.md). Goldens: `fixtures/assurance/canonical/v1/github/`.
- Framework packs are **not** remapped here. Framework crate must not depend on the catalog crate; collector stays catalog-blind.

See [`docs/specs/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md), [`docs/specs/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md), [`docs/specs/sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md), [`docs/specs/vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md), [`docs/specs/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md), [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md), [`docs/specs/personnel-security.md`](personnel-security.md), [`docs/specs/github-collector.md`](github-collector.md), [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md), [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md), [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md), [`docs/adr/0003-vulnerability-canonical-assurance-catalog.md`](../adr/0003-vulnerability-canonical-assurance-catalog.md), [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md), [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md), and [`docs/adr/0003-github-collector-canonical-evidence-mapping.md`](../adr/0003-github-collector-canonical-evidence-mapping.md).

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

`Not(unknown)` stays `unknown`. Unknown facts are never treated as false. `ApplicabilityDecision::remains_in_compiled_set` is false only for `notApplicable`. SoA `Applicability::unresolved` is the projection alias of `manualDeterminationRequired`. `project_soa(framework, version)` consumes pack applicability rows as generic three-state **default/structural flags** (not a boolean copy) and projects missing implementations as first-class `notImplemented`. The operational graph projection (implementations, treatments, evidence effectiveness, NA approval, immutable snapshot diffs) is landed in [`docs/specs/operational-soa.md`](operational-soa.md). History is reconstructed only from `pin_soa_snapshot` / `project_soa_from_snapshot`.

`ApplicabilityOutcome` carries ordered rationale, predicate traces, named unknown facts, lex-sorted `selectedSubjects`, and `excludedSubjects` (`id`, `reason`, `exclusionIndex`). Zero selected subjects does **not** flip the decision to `notApplicable`. Hand selected ids to population evaluation via `EvidenceSet::set_population`.

Snapshot schema `weeping-angel/applicability-snapshot/v1` (`APPLICABILITY_SNAPSHOT_SCHEMA`):

```text
schema, assessmentId, scope,
requirementDecisions[], controlDecisions[],
packEntries[], digest
```

`packEntries` are artifacts, not Kleene inputs. Digest is IR `canonical_digest` over the body excluding `digest`. This engine **produces** the Kleene snapshot. Lineage persist/explain is landed (see below); crate-root `ApplicabilitySnapshot` is the lineage persist document, not this type.

See [`docs/specs/applicability-engine.md`](applicability-engine.md) and [ADR 0003 applicability engine](../adr/0003-applicability-engine.md).

## Evidence

Collectors and the scanner bridge emit **observations**, never compliance.

Allowed: `"repository X has branch_protection enabled"`, `"route Y is exposed_without_auth"`.

Forbidden (seal / collect error): `"ISO 27001 compliant"`, `"GDPR compliant"`, `"SOC 2 compliant"`, `"NIS2 compliant"`, `"DORA compliant"`, `ControlTestResult`, `"risk accepted"`, `"ISO control failed"` (and the listed variants in [`risk-identification.md`](risk-identification.md)).

`EvidenceEnvelope` is immutable once sealed. Mutation is a new envelope. Digest covers observation + provenance. Same payload+provenance → same digest.

Envelope (`evidence/v1`):

```text
evidenceId, schemaVersion, observation, provenance, digest,
artifactRef?, collectionRunId, contentDigest, sensitivity, scope, supersedes?,
observedAt?, validFrom?, validUntil?, sourceRevision?
```

`provenance = { collectorId, collectedAt, scope, asset }`.

`observedAt` / `validFrom` / `validUntil` / `sourceRevision` are serde-default fields **outside** `DigestBody` (absent JSON matches historical payloads). Accessors default `observedAt → collectedAt` and `validFrom → observedAt`. `validUntil` omitted means open until superseded, revoked, or policy-stale. Changing those fields after seal does **not** rewrite `digest`. ISO Phase 7 names are this projection plus the latest `evidence-validity/v1` assertion — not sealed clocks.

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

The evaluator compares stored types (`typed_eq` / `cmp_numeric` / `list_contains`) and fails closed on a `type mismatch`. It does not reparse `"01"` / `"1.0"` / `"true"`. Integer↔Decimal numeric compare is exact decimal-string scale-align, never IEEE-754. Same semantic facts + provenance ⇒ same `canonical_digest` regardless of map insertion order. See [`docs/specs/typed-evidence.md`](typed-evidence.md) and [ADR 0003 typed evidence](../adr/0003-typed-evidence-canonical-serialization.md).

### Ledger

`EvidenceLedger` (SQLite file or in-memory) owns **observations**, never conclusions.

```text
append, get, query, latest, for_subject, for_type,
for_collection_run, within_window, supersede, record_collection_run,
record_validity_event, validity_events, validity_events_for,
valid_during, latest_as_of,
persist_assessment_run, load_assessment_run,
persist_control_test_run, load_control_test_run,
persist_framework_snapshot, load_framework_snapshot
```

Forbidden: `set_compliant`, `set_control_status`.

`append` is idempotent by digest (EVD-002) and records an initial `asserted` validity event. `supersede` records history via `supersedes` plus a `superseded` event; it does not mutate the previous envelope.

### Temporal validity

Sibling document `evidence-validity/v1` (`EvidenceValidityEvent`: `asserted` \| `superseded` \| `revoked` \| `invalidated`). Event identity is digest-derived (`eventId`); identical bytes are a no-op; a second write of different bytes is `LedgerError::Immutable`. Revoked/invalidated envelopes remain `get`able.

Half-open window: `validFrom <= T` and (`validUntil` omitted or `T < validUntil`). Candidate at `T` requires `collectedAt <= T`, `observedAt <= T`, inside the window, and not revoked/invalidated at or before `T`; among remaining leaves, latest `observedAt` then `collectedAt` then digest.

`within_window` remains inclusive `collected_at`. `valid_during` / `latest_as_of` / `select_latest_as_of` apply the candidate filter (no future, no expired, no revoked-at-T). Digest-order first-hit over the unbounded bag is not an evaluation selector.

`AssessmentContext` is `{ now, maxAge }`. `now` is the injected `as_of` clock. `weeping-angel-control-test::FreshnessPolicy { maxAge, asOf, period }` is the scheduler handoff (cadence is not this contract; do not confuse with `scheduler::FreshnessPolicy`, which is `maxAge` only). Live `assess` still uses `Utc::now()` + 24h.

Point-in-time results stay `Effectiveness`. Period projection emits `PeriodEffectiveness` on `ControlTestResult.period` (`continuouslyEffective` \| `intermittentRegression` \| `insufficientObservationCoverage` \| `ineffective` \| `manualReviewRequired`). Default semantics are `instant`: one `Exists` hit is not continuous operating effectiveness. Unset period uses `[now - maxAge, now)`.

Defects are disjoint: missing → `insufficientEvidence`; future/expired → excluded (not `effective`); candidate older than `maxAge` → `staleEvidence`.

`project_timeline` / `compare_temporal` / `TemporalDiff` (`observationGaps`, `expiredAt`, `revoked`, `superseded`, `intermittentControls`, `coverageInsufficient`) serve readiness and audit library exports; pairwise `compare` / `SnapshotDiff` is unchanged. Catalog fact fields named `valid_until` remain observation facts.

SSOT: [`docs/specs/evidence-validity-temporal-assurance.md`](evidence-validity-temporal-assurance.md), [`docs/specs/temporal-assurance.md`](temporal-assurance.md). ADRs: [`0003-temporal-assurance.md`](../adr/0003-temporal-assurance.md), [`0003-evidence-validity-temporal-assurance.md`](../adr/0003-evidence-validity-temporal-assurance.md).

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
| Stale evidence (candidate at `asOf` but `now - collectedAt > maxAge` or `FreshWithin` fail) | `staleEvidence` or `inconclusive` (never `effective`) |
| Future observation (`observedAt` / `collectedAt` after `asOf`) | excluded from candidates (never `effective`; not “stale of the future”) |
| Expired (`asOf >= validUntil`) | excluded (never `effective`; not `staleEvidence`) |
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

Evaluation indexes envelopes by `(evidenceType, subject)` (`EvidenceIndex` / `build_index_as_of`). Latest / `supersedes` among **candidates at `asOf`** wins; digest-identical duplicates count once. `Exists` at `T` means a candidate exists at `T`, not continuous effectiveness.

`EvidenceSelector = { evidenceType, subjectSelector, field, freshness }`. No collector id in test definitions. Control-test `subjectSelector` JSON `{ kind, id }` folds `id` into IR `ids`.

`ControlTestResult = { testId, controlId, effectiveness, rationale, evidenceRefs, missingEvidence, evaluatedAt, testVersion, inputDigest, duration?, population?, period? }`. `period` is `PeriodEffectiveness` (nested; omitted when not projected). Wall-clock `duration` is not part of semantic identity. Same test + evidence snapshot + evaluation context (`asOf`) → same semantic result. `CompiledTest.expr` is an optional JSON `TestExpr`; `evaluate_compiled` attaches it.

Tests are **canonical** (`test.source.required-review`), never `iso27001.a.x.y.github.*`.

## Scanner bridge

Owned by `weeping-angel-assurance::bridge`. Does **not** rewrite `EngineHit::to_semantic_finding`.

| Source | Observation type | Facts |
| --- | --- | --- |
| `EngineHit` | `security_finding` | `rule_id`, `path`, `category`, `canonical_type` + title narrative |
| `SemanticFinding` | `security_finding` | `rule_id`, `finding_id` + title narrative |

`canonical_type` is one of: `security.finding`, `security.vulnerability.present`, `security.exposure.present`, `security.authz.weakness`, `security.secret.exposure`, `security.tls.misconfiguration`, `security.header.misconfiguration`, `security.dependency_confusion_risk`.

One-way. Observations do not write back onto findings. Empty scan ≠ Effective control. Do not emit `security.no_vulnerabilities` as a passable fact. Bridge `security_*` types are **not** the catalog library; catalog evaluation consumes `evidence.vulnerability.*` / `evidence.secret.exposure` / `evidence.dependency.*` (vulnerability catalog). A later adapter may emit those types from engine hits. Accepted-risk and approved-exception are not remediation.

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

Continuous operation (library, not clap):

```text
AssuranceScheduler::builder()
    .clock(Clock)
    .store(InMemorySchedulerStore)
    .ledger(Arc<Mutex<EvidenceLedger>>)
    .framework(FrameworkTarget)
    .scope(AssessmentScope)
    .collector(C: EvidenceCollector)*
    .register(JobSpec)
    .build() → tick() → TickReport
```

`Clock::now` drives due/not-due, retry backoff, and cooperative timeout (`FakeClock` in tests, `UtcClock` in production). `JobSpec` owns cadence, freshness (`max_age` → `AssessmentContext`), `dependsOn`, retry/backoff, timeout, and optional jitter. `JobKind` is `collection` | `test` | `projection` | `snapshot`; Drift is a snapshot stage that calls existing `compare`. Typed observations are `detect_events` / `detect_isms_drift` (Prompt 15); `tick` does not invent that catalog. Scheduled run identity is a canonical digest of job + cadence slot + collector/config + attempt-policy version — not `Utc::now()` uniqueness. Operational job state lives in `InMemorySchedulerStore`, not envelope payloads.

A failed or timed-out collect does not delete ledger envelopes; evaluate reattaches prior evidence and existing `StaleEvidence` law applies (fresh → `Effective`, stale → `StaleEvidence`). Independent collection jobs may run concurrently. Framework and control-test stay network-free; collectors never set `Effectiveness`. One-shot `assess` is unchanged (collector `Err` still evaluates an empty set).

`tick` records Collect → Normalize → Seal → Ledger → Evaluate → Project → Snapshot → Drift against existing collect/seal/ledger/evaluate/project/compare APIs. A future daemon is `loop { tick(); sleep_until(next) }` outside clap. `weeping-angel isms run` is not shipped; clap must not define cadence/retry/backoff/jitter.

`AssessmentRun` pins `frameworkPackDigest`, `canonicalCatalogDigest`, `assessmentDefinitionDigest`, `applicabilitySnapshotId`, `collectorRuns`, `evidenceSnapshotDigest`, `resultDigest`, and `asOf` (serialized from `startedAt`). Distinct identities — not the compile digest copied three times. The run is returned on `AssessmentReport.run`; `assess` does not open a ledger. Replay at that clock must ignore later envelopes.

Projections (not certificates):

```text
project_readiness(...) → FrameworkReadinessSnapshot
project_soa(framework, version) → StatementOfApplicability   # live convenience (pack flags; not history)
project_soa_from_snapshot(snap) → StatementOfApplicability  # historical reconstruction
project_operational_soa(input) → Result<StatementOfApplicability, OperationalSoaError>
pin_soa_snapshot(soa, packDigest) → StatementOfApplicabilitySnapshot
diff_soa_snapshots(previous, next) → SnapshotDiff           # soaCauses taxonomy
project_residual_risk(store, request) → ResidualRiskProjection   # see Residual risk
query_residual_risk(store, id) → Option<ResidualRiskProjection>
compare(previous, next) → SnapshotDiff
compare_runs / compare_lineage(previous, next) → SnapshotDiff
project_timeline(set, range, type?, subject?) → EvidenceTimeline
compare_temporal / diff_period(range, set, periodByControl) → TemporalDiff
detect_events(previous, next) → Vec<IsmsEvent>                 # Prompt 15; order-insensitive
detect_isms_drift(previous, next) → IsmsDrift                  # readiness compare + events
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

See [`docs/specs/assessment-lineage.md`](assessment-lineage.md) and [ADR 0003 assessment lineage](../adr/0003-assessment-lineage.md).

## Operational Statement of Applicability

Living readiness projection in `weeping-angel-assurance::soa`. Not a certificate and not licensed ISO text. Applicability, implementation, and effectiveness are independent dimensions: missing implementation is `notImplemented` on an Applicable row; insufficient evidence is `Effectiveness::InsufficientEvidence`; neither becomes SoA `NotApplicable`.

Kleene results are preferred when present (`Unresolved` ↔ `ManualDeterminationRequired`). Pack `applicability.toml` supplies structural defaults only. NA requires context rationale plus unexpired approval; expired/missing approval is a readiness gap (`expiredNaApproval` / `missingNaApproval`), not silent NA. ISO pack `A.5.19` stays `NotApplicable`; `A.8.13` stays `Unresolved`.

Treatment-driven inclusion/exclusion fail-closes on missing `RiskTreatmentRef` / register digest (`OperationalSoaError`). Prompt 06/08 engines are not implemented here. `owner` is `None` until the implementation registry exposes it. Partial mappings stay applicable and record `partialCanonicalMapping`.

`pin_soa_snapshot` seals `StatementOfApplicabilitySnapshot` (`typed_canonical_digest("soa-snapshot", …)`). Reconstruct with `project_soa_from_snapshot`. `diff_soa_snapshots` classifies `soaCauses`: applicability, implementation, effectiveness regression, exception expiry, mapping, treatment. Live `project_soa` is not history.

See [`docs/specs/operational-soa.md`](operational-soa.md) and [ADR 0003 operational SoA](../adr/0003-operational-soa.md).

## ISMS context

Durable management-system **definition** in `weeping-angel-assurance-ir::isms` (`IsmsContext`). Schema stays `assurance-ir/v1`. This is the operational root later scope, risk, governance, audit, and readiness work hang off — not a point-in-time `AssessmentDefinition`, not a compiled SoA, and not collector output.

```text
IsmsContext::new(id, organization, scope)
IsmsContext::validate()                          // ValidateIr
validate_assessment_against_context(&AssessmentDefinition, &IsmsContext)
explain_isms_context(&IsmsContext) -> String     // definition, not results
```

One organization per context (`legalName` required). Graph: `Organization` → `ManagementSystemScope` (named handle, not `ScopeResolution`); `InterestedParty` → `Obligation` (id integrity both directions); declared `SecurityObjective`; optional `RiskMethodologyId` (scoring is **Risk methodology**); existing `AssetId` / `VendorId` / `IdentityId` population sets; `GovernanceCadence`; `IsmsLifecycleStatus` (`draft` | `active` | `underReview` | `retired` | `superseded`).

Rules:

- `AssessmentDefinition::new` stays valid. Optional snake_case `isms_context_id` defaults to `None`. Golden `assessment.json` still decodes. `AssessmentDefinition::validate()` does **not** require a context.
- Standalone `IsmsContext::validate()` does not require assessment inventories. Pair validation fails closed on a mismatched pointer and on population ids missing from that assessment.
- Duplicate ids, dangling internal refs, empty/whitespace identity and title fields, unknown enum tags, zero-count cadence, and impossible lifecycle combinations fail closed.
- Generic context JSON has no ISO clause / Annex A / SoA keys and no AWS / GitHub / Entra objects. No `effectiveness` / residual / control-test fields on the context record.
- Crate-root `InterestedParty` / `Obligation` are membership-graph records on this document. The obligation registry (`party` / `obligation` modules) is a distinct type family sharing the same id newtypes.
- Crate-root `SecurityObjective` is a **declaration** (`ObjectiveId`). Measured records and `evaluate_objective` live in `objectives` (see **Security objectives**).
- Golden: `tests/fixtures/assurance-ir/v1/isms-context.json` (one org, two business units, one internal + one external issue, party + obligation, objective, methodology ref, active cadence).

See [`docs/specs/isms-context.md`](isms-context.md) and [ADR 0008 ISMS context](../adr/0008-isms-context.md).

## Interested parties and obligations

Standalone governance-input registry in `weeping-angel-assurance-ir::{party,obligation}` (`ObligationRegistry`). Schema stays `assurance-ir/v1`. Not a framework `Requirement`, not an assessment inventory, not collector satisfaction, and not crate-root `isms::Obligation`.

```text
why the org must care     = obligation::Obligation
who cares                 = party::InterestedParty
where the duty comes from = RequirementSource
what the control means    = Control
whether it is effective   = ControlTestResult.effectiveness
```

```text
ObligationRegistry::validate(&ObligationLinkUniverse)
ObligationRegistry::current_obligations_at(t)
ObligationRegistry::get_obligation(id)           // including retired/superseded/expired
obligation_applies(&Obligation, &universe, t)    // AssessmentScope / SubjectSelector
projects_as_equivalence / projects_as_full_satisfaction
explain_why_control_exists(control_id, registry, t) -> ObligationExplain
explain_why_document_exists(document_id, registry, t) -> ObligationExplain
```

Rules:

- Shared `ObligationId` / `InterestedPartyId` with ISMS context and controlled-documents. Registry structs are **not** crate-root `pub use` (name collision with the context graph).
- `RequirementSourceKind` is extensible (`contractual` | `legalRegulatory` | `customer` | `internalPolicy` | `insurer` | `supplier` | `employment` | `other`). Citations are pointers; protected normative text fails validate.
- Lifecycle is `draft` | `active` | `retired` | `superseded`. No delete API. `current_obligations_at` excludes retired, superseded, expired, out-of-scope, and unknown. Overlapping Active duties may coexist.
- `ObligationMapping` reuses `MappingDirection` / `MappingRelation` / `MappingCompleteness`. `PartiallySatisfies` / `Supports` never project as equivalence or full satisfaction. Illegal `Equivalent` + `partial` fails closed.
- Stored applicability is IR `AssessmentScope` (not provider filters or facade collector allow-sets). Empty scope is the ISMS/unspecified boundary (`InScope`), not every provider resource.
- `ControlExplanation.obligations` is additive default-empty. `explain_control` does not populate it; use the dedicated helpers. Digests stay `canon/v1`.
- Collectors and framework packs must not write obligation lifecycle or satisfaction.

See [`docs/specs/interested-parties-obligations.md`](interested-parties-obligations.md) and [ADR 0008 interested parties / obligations](../adr/0008-interested-parties-obligations.md).

## Organizational scope engine

Pure `weeping-angel-assurance::scope` over `AssessmentDefinition` inventories plus optional `&IsmsContext`. Snapshot schema `weeping-angel/scope-resolution/v1` (`SCOPE_RESOLUTION_SCHEMA`). IR schema stays `assurance-ir/v1`. Facade `AssessmentScope` / `CollectorScope` remain `AssetId` allow-sets filled only from `InScope` ids. Crawl `src/engine/scope.rs` is unchanged.

```text
ScopeInputs::from_assessment(&AssessmentDefinition)
    .with_context(&IsmsContext)          // optional
    .with_candidates(Vec<SubjectRef>)    // optional
resolve_scope(&ScopeInputs, as_of) -> Result<ScopeResolution, ScopeError>
resolve_subject(&SubjectRef, &ScopeInputs, as_of) -> SubjectScopeDecision
in_scope_population(&SubjectSelector, &ScopeResolution, &AssessmentDefinition)
is_definitely_in_scope(ScopeDecision) == (decision == InScope)
ScopeResolution::to_collector_scope() / to_facade_assessment_scope()
ScopeDecision = InScope | OutOfScope | Conditional | Unknown
```

`ManagementSystemScope` is a named handle on `IsmsContext`. This engine **resolves** candidates to a four-state decision with rationale, lineage, and explain traces (`repo:payments -> business-unit:finance -> service:payments -> ISMS scope -> InScope`).

Rules:

- Suppressing `ScopeExclusion` requires rationale, `PrincipalRef` owner, `approvalRef`, `approvedAt`, `reviewBy` and/or `expiresAt`, and `evidenceRefs`. Silent rows fail `validate()` and do not suppress.
- Expired or review-overdue exclusions stay in the trace (`applied=false`) and do not suppress unless renewed. Clock is caller `as_of`.
- Precedence is a rank table, not vec order. Include and exclude share class rank (exact 100 / tag 80 / kind 60 / inherit 40 / org 30). Equal-rank include vs exclude is `Unknown`.
- Empty `subjects` and no bound organization is not implicit “everything in inventory.” Unresolved subjects are `Unknown`. Cycles fail closed.
- `Unknown` / `Conditional` / `OutOfScope` never count as positive in-scope evidence. Collector adapters omit those ids.
- Collectors must not mutate IR scope. No ISO types in this module.
- Additive generic kinds only: `businessUnit`, `location`, `dataDomain`, `personnelPopulation`. No AWS / GitHub / Entra kinds.

See [`docs/specs/scope-engine.md`](scope-engine.md) and [ADR 0008 scope engine](../adr/0008-scope-engine.md).

## Security objectives

Measurable first-class ISMS records in `weeping-angel-assurance-ir::objectives` plus a pure evaluator in `weeping-angel-assurance::objectives`. Schema stays `assurance-ir/v1` for governance records. Evaluation snapshots use `weeping-angel/objective-evaluation/v1`. Not `Control.objective` prose, not catalog `control.governance.security-objectives` attestation, and not crate-root `isms::SecurityObjective`.

```text
evaluate_objective(objective, metric, target, measurements, as_of, evidence)
    -> ObjectiveEvaluation
evaluate_objective_with_resolution(..., Option<&ScopeResolution>)
ObjectiveStatus = OnTrack | AtRisk | Missed | Achieved | InsufficientEvidence
```

`as_of` is an argument. Same bytes + same snapshot digest + same clock + same scope binding ⇒ same status, reason codes, and snapshot digest. No I/O, no collector calls, no Kleene, no formula VM.

Rules:

- `objectives::SecurityObjective` is the measurable record (`SecurityObjectiveId`, metric/target ids, IR `AssessmentScope`, cadence, dates, lifecycle). Crate-root `isms::SecurityObjective` is a **declaration** (`ObjectiveId`, title/description/owner).
- Status is never stored on the governance record. Status is always an `ObjectiveEvaluationSnapshot`.
- Payloads are `EvidenceValue` (`typed_eq` / `cmp_numeric`). IR does not define a second metric-value enum. No `f64`.
- Degradation (missing, stale, partial, unscoped, type/domain, manual without sealed attestation) is `InsufficientEvidence`. It is never `OnTrack` or `Achieved`.
- Non-`Active` evaluation is `ObjectiveError::NotActive`, not a success status. Active records require owner, non-empty scope, and `startAt`.
- Measurement scope is required. Out-of-scope mix is `scopeMismatch`. Optional pinned `ScopeResolution`: only `InScope` subjects count.
- Ongoing objectives (no deadline) stay `OnTrack` / `AtRisk` / `InsufficientEvidence`; the clock alone cannot produce `Achieved` or `Missed`.
- Collectors emit facts only. Crate-root `continuity::ObjectiveStatus` is a different type.
- Golden: `tests/fixtures/assurance-ir/v1/security-objective-vuln-sla.json` (critical-vuln 7-day SLA, target ≥ 98% — fixture data, not an evaluator constant).

See [`docs/specs/security-objectives.md`](security-objectives.md) and [ADR 0008 security objectives](../adr/0008-security-objectives.md).

## Risk methodology

Organization-configurable, versioned scoring in `weeping-angel-assurance-ir` (`risk_methodology.rs`, `CanonicalDecimal` in `decimal.rs`, `RiskMethodologyId` in `id.rs`). Schema stays `assurance-ir/v1`. Scoring is a pure function over IR types — not Kleene applicability, not control-test, not collector output, and not scanner `severity_policy.rs`.

```text
validate_risk_methodology(&RiskMethodology) → Result<(), RiskMethodologyError>
score_risk(&RiskMethodology, &RiskScoreInput) → Result<ScoredRisk, RiskMethodologyError>
```

`ScoredRisk` is `{ input, score, rating }`. `DerivedRating` carries `methodologyId`, `revision`, and a methodology-declared `ratingId`. There is no crate-wide `RiskRating` enum and no `score_risk` overload that accepts a rating. `EvidenceValue` has no rating variant.

`ScoringMode`: `qualitative` | `semiQuantitative` | `quantitative` | `customBounded`. Combination must match (`matrix` / `product`|`sum` / `expectedLoss` / `identity`). Matrices and bands are methodology data (goldens `risk-methodology-3x3.json`, `risk-methodology-5x5.json`, `risk-methodology-expected-loss.json`). Control logic never switches on a hardcoded 5×5.

Rules:

- Malformed matrices, duplicate ordinals, unreachable ratings, invalid band/appetite boundaries, mode mismatches, and out-of-domain scores fail closed. **No clamp.**
- Quantitative expected loss uses IR `CanonicalDecimal` (exact multiply, no `f64`). `0.1 * 0.2` is `0.02`.
- `lock()` pins a revision used in a finalized assessment; `supersede` is the only evolution (`revision + 1`, new id). Scoring pins the document the caller passes.
- Appetite / tolerance / acceptance thresholds are stored and validated; this slice does not accept a risk.
- `IsmsContext.riskMethodologyId` is a typed reference. Register / residual slices call `score_risk`; they do not reimplement matrices.

See [`docs/specs/risk-methodology.md`](risk-methodology.md) and [ADR 0005 risk methodology](../adr/0005-risk-methodology.md).

## Risk register

Operational information-security risk record on the same IR `Risk` type (`weeping-angel-assurance-ir::risk`; adapter `risk_scoring`). Schema stays `assurance-ir/v1`. `AssessmentDefinition.risks` remains `Vec<Risk>`. `Risk::new(id, title, description)` still serializes without `owner` / `treatmentId` / `residualScore`. Golden `tests/fixtures/assurance-ir/v1/risk.json` still decodes. A scanner finding is not a risk.

```text
RiskStatus::can_transition / Risk::transition(to) → Result
Risk::revise(title)                               # history keeps prior title/status/inherentScore
Risk::review_overdue(as_of) → bool
validate_risk_reviews_at(assessment, as_of)
score_inherent(methodology_version, likelihood, impact, cia?)
```

`RiskStatus`: `draft` | `open` (default) | `underTreatment` | `accepted` | `mitigated` | `closed` | `retired`. Fail-closed table: `Open → Mitigated` / `Open → Closed` / `Retired → *` are illegal. `transition` appends `StatusTransition`; `revise` increments `version`.

Rules:

- Additive fields default and omit when empty (`serde(default)` + `skip_serializing_if`). `version` default `1` is omitted unless pinned.
- `FindingRef` is N:N contributor identity. No `Finding` type in IR. No auto-promotion from `src/finding.rs`. `RiskSource` is provenance, not a framework result.
- Inherent score/rating are derived snapshots. Raw `likelihood`/`impact` (`MethodologyValue.levelId`) plus `methodologyVersion` are required when derived fields are present. `score_risk` remains the methodology engine; `score_inherent` is a replaceable register adapter over opaque values (cell id = authored level pair). **No hardcoded 5×5.**
- CIA `{ confidentiality, integrity, availability }` are optional raw `u32` inputs. They do not substitute for methodology ratings.
- `residualScore` / `residualRating` are placeholders. Control-derived residual is `ResidualRiskProjection`.
- Clockless `validate()` fail-closes on duplicate `RiskId`, dangling asset / process / vendor / control / identity-owner / evidence-requirement / supersession refs, malformed evidence digests, illegal recorded transitions, and derived inherent fields without a version pin and raw inputs. `Some(treatmentId)` resolves through `validate_treatment_inventory`. Overdue `nextReview` is `validate_risk_reviews_at` only (terminal `Closed`/`Retired` spared; unscheduled is not overdue).
- Owner is `PrincipalRef`. Crate-root `ReviewCadence` uses `intervalSeconds` and is not `implementation::ReviewCadence`.

See [`docs/specs/risk-register.md`](risk-register.md) and [ADR 0005 operational risk register](../adr/0005-operational-risk-register.md).

## Risk identification

Deterministic candidate discovery from existing evidence. Types live in `weeping-angel-assurance-ir::{risk_candidate,risk_promotion}`. Engine APIs live at `weeping-angel-assurance::risk_identification`. Schema stays `assurance-ir/v1`. A `RiskCandidate` is not a `Risk`. Collectors and the scanner bridge still emit `security_finding` facts only.

```text
identify_risk_candidates(IdentificationContext) → Vec<RiskCandidate>
correlate_candidates(proposals) → Vec<RiskCandidate>
promote_candidate(&mut AssessmentDefinition, candidate, principal, at, rationale, methodology_inputs?)
  → Result<(RiskCandidate, Risk, PromotionRecord), IdentificationError>
dismiss_candidate(candidate, principal, at, rationale)
  → Result<(RiskCandidate, DismissalRecord), IdentificationError>
should_resurface(cluster, dismissal) → bool
```

Rules:

- Correlation key is `ck:sha256:` + hex32 of sorted `SubjectRef`s plus normalized `scenarioKey`. Observation identity excludes `collected_at` / run id (`oi:sha256:` + hex16).
- N findings with the same key collapse to one `Proposed` survivor. One `security.vulnerability.present` finding on a production `Service` that is also a processing-activity system emits two scenario keys (confidentiality vs integrity/availability).
- `identify_risk_candidates` never inserts into `AssessmentDefinition.risks`. Only `promote_candidate` does, via Prompt 06 `Risk::new` plus register slots. Candidate id ≠ risk id.
- Dismissal is retained. Resurface is same-id `Dismissed` → `Resurfaced` only when a new observation identity appears on the same key. Clock-only refresh and empty evidence do not resurface and never auto-promote.
- Stale clusters may be listed; `promote_candidate` fails closed (`stale evidence`). Promotable statuses are `Proposed` and `Resurfaced` only.
- `looks_like_compliance_claim` rejects `risk accepted` / `ISO control failed`. Identify drops matching narratives. Scanners cannot author `RiskStatus::Accepted` or control-test `Effectiveness`.
- Score suggestion is optional and omitted by identify. No second scoring matrix. Clustered category disagreement uses `SuggestedRiskCategory::Other("mixed")`.

See [`docs/specs/risk-identification.md`](risk-identification.md) and [ADR 0007](../adr/0007-risk-identification-candidate-correlation.md).

## Residual risk

Explainable projection over pinned snapshots. Types live in `weeping-angel-assurance-ir::residual` (re-exported from that crate). Projection APIs live at `weeping-angel-assurance::residual` (module path; not a crate-root re-export). Schema stays `assurance-ir/v1`. Evidence envelopes and collectors do not carry residual ratings.

```text
project_residual_risk(store, ResidualRiskRequest) → Result<ResidualRiskProjection, ResidualRiskError>
query_residual_risk(store, ResidualRiskId) → Option<ResidualRiskProjection>
```

`ResidualRiskRequest` pins `mode`, `InherentRiskSnapshot`, `TreatmentPlanSnapshot`, `MethodologyRef`, `ControlTestSnapshotRef` + `[ControlTestResult]`, optional `[Exception]`, optional `ManualResidualAssessment`, and `projectedAt`. Modes: `calculated` | `assessed` | `hybrid`.

Rules:

- `Effectiveness::Effective` never maps to residual ordinal 0 (`MIN_RESIDUAL_FLOOR = 1` on `residual-methodology:control-effectiveness/v1`).
- `residual-methodology:no-reduction/v1` copies inherent; effectiveness does not lower residual.
- Assessed requires principal + rationale + time (`missing manual assessment`). Hybrid requires that **and** `approvedBy` (`missing management assessment`). Hybrid may raise residual above calculated; it cannot skip Calculated fail-closed pins.
- `NotTested` / `InsufficientEvidence` / `StaleEvidence` / dangling relevant control / missing pin versions fail closed. `NotApplicable` on a relevant control is contradiction. `ExceptionApproved` is governance evidence, not a Low floor.
- Identity is `residual:{sha256}` over semantic fields including caller `projectedAt`. `ResidualRiskStore` is first-write-wins; control regression writes a **new** projection.

See [`docs/specs/residual-risk.md`](residual-risk.md) and [ADR 0003 residual risk](../adr/0003-residual-risk.md).

## Risk treatment

Accountable Mitigate / Accept / Avoid / Transfer path in `weeping-angel-assurance-ir::risk_treatment` (re-exported from that crate). Inventory is `AssessmentDefinition.risk_treatments` (`serde(default)` empty). Schema stays `assurance-ir/v1`. Compile `supports_risk_treatment` remains a capability gate; compile does not evaluate plans. `RiskStatus::Accepted` is not evidence.

```text
RiskTreatmentDecision::propose → transition(approved → executing → verification → completed)
TreatmentState::can_transition            # illegal pair → TreatmentError::InvalidTransition
active_treatment(assessment, risk_id)
acceptance_in_force(assessment, risk_id, as_of)
treatment_required(assessment, risk_id, as_of)
validate_treatment_inventory(assessment)  # clockless; IR validate
validate_treatments_at(assessment, as_of) # expired/missing acceptance vs Accepted
```

Rules:

- All four strategies walk the happy path. No `Approved → Completed` shortcut.
- Mitigate completes only when every `required` action is `Done` and cited `ControlId` / `ControlImplementationId` resolve. Partial mitigation cannot complete.
- Accept requires sealed `RiskAcceptance` (principal, rationale, `expiresAt`, ≥ 1 evidence). Post-approve mutation fails `ImmutableAcceptance`. `as_of ≥ expiresAt` ⇒ `treatment_required`.
- Avoid requires organizational-action evidence. Transfer requires non-empty contract + transferee (`MissingContractEvidence`).
- Target residual is `TargetResidualRisk::VersionedPlaceholder` (methodology pin, not a rating). Frozen at approval; completion mismatch fails `TargetResidualMismatch`. This slice does not call `score_risk` or project residual effectiveness.
- `Risk.treatment_id` if `Some` must equal the active decision, or the latest completed id if none is active.
- Collectors must not emit treatment types or `RiskRating`.

See [`docs/specs/risk-treatment.md`](risk-treatment.md) and [ADR 0006 risk treatment](../adr/0006-risk-treatment-engine.md).

## Supplier risk

Operational supplier-security lifecycle on existing IR `Vendor` (`weeping-angel-assurance-ir::vendor`, re-exported from that crate). `AssessmentDefinition.vendors` stays `Vec<Vendor>`. Schema stays `assurance-ir/v1`. `Vendor::new(id, name)` still serializes as `{ id, name }`. Kleene `HasVendor` remains presence-only. Governance catalog `control.vendor.*` IDs are not rewritten.

```text
Candidate → underReview → approved → active → restricted|suspended → terminating → terminated
SupplierLifecycleStatus::can_transition
Vendor::{transition, record_review, approve, attach_evidence, record_assessment_expired}
Vendor::{review_current, has_lingering_access, requires_current_security_review}
critical_suppliers(assessment)
validate_supplier_reviews_at(assessment, as_of)   # clocked; expired exception does not suppress
```

Rules:

- Risk-tiered: Critical/High (and privileged-elevated Low/Medium) in the contract window require a current review. Low without privileged access and without `Processor` classification is reduced requirements.
- Evidence refs / questionnaire source do not set `Approved` / `Active` and do not set `RiskStatus::Accepted`.
- Clockless `validate()` fail-closes on duplicate `VendorId`, dangling processor/service/risk/exception/control/identity refs, lingering access after termination, missing contract security requirement when required, privileged + unspecified criticality, and `Approved` without an Approved decision. Stale reviews are clocked only.
- `Vendor.risk_ids` must resolve in `assessment.risks`. Linkage is not risk acceptance.

See [`docs/specs/supplier-risk.md`](supplier-risk.md) and [ADR 0007 supplier risk](../adr/0007-supplier-risk.md).

## Control implementation registry

Organizational **how this org implements** a canonical control. Types live in `weeping-angel-assurance-ir::implementation`; queries in `weeping-angel-assurance-ir::registry` (crate-root re-exports). Schema stays `assurance-ir/v1`. There is no competing registry type and no `effectiveness` field on the record.

```text
what the control means     = Control
how this org implements it = ControlImplementation
whether it is effective    = ControlTestResult.effectiveness
```

`ImplementationStatus` is additive: existing `notImplemented` / `planned` / `partiallyImplemented` / `implemented` / `notApplicable` / `retired`, plus `ineffective` (alias `disabled`) and `unknown`. Never `effective`. `Implemented` does **not** imply `Effective`.

One `control_id` may have several rows. Empty `applies_to` is a universal population; empty `asset_ids` is a universal asset set. `overlap_report` / `validate_assessment_ir` fail closed only when **both** axes collide (no silent double-count). Retired / superseded rows are not coverage-active.

`validate_assessment_ir` also fail-closes on dangling control / subject / asset / risk / exception / evidence-expectation / supersession refs, missing evidence-expectation refs on `Implemented` / `PartiallyImplemented`, omitted Required control evidence refs, missing review on those statuses, duplicate ids, and supersession cycles. `treatment_ids` resolve against `assessment.risk_treatments` only when that collection is non-empty. `implementation::ReviewCadence` uses `intervalDays` and is not crate-root `ReviewCadence` (risk register, `intervalSeconds`).

Queries: `implementations_for`, `current_implementations_for`, `implementation_by_id`, `overlap_report`. Lineage still first-matches one `ControlExplanation.implementation` by `control_id`. Evidence crate stays conclusion-free.

See [`docs/specs/control-implementation-registry.md`](control-implementation-registry.md) and [ADR 0003 control implementation registry](../adr/0003-control-implementation-registry.md).

## ISMS events and drift

Immutable observations of management-system state change (`weeping-angel/isms-event/v1`) live in `weeping-angel-assurance-ir::event` and `weeping-angel-assurance::{events,drift}`. Callers assemble `IsmsSnapshot` (`snapshotId`, `evaluatedAt`, inventories). There is no crate-root re-export of `detect_events`; import `weeping-angel-assurance::drift`.

```text
detect_events(previous, next) → Vec<IsmsEvent>        # sorted by eventId
detect_isms_drift(previous, next) → IsmsDrift         # SnapshotDiff + events
```

Rules:

- Events are observations, not tickets. JSON camelCase. `kind` unit variants are strings (`ControlRegressed`); `Extensible` is externally tagged. Causes persist as `causeRefs` (alias `causes`). Subject `kind` is camelCase (`control`, `asset`, `event`).
- `eventId = event:sha256:` + SHA-256 `typed_canonical_digest("isms-event", body without id)`. Time is next `evaluatedAt`. Snapshot pins are the caller `snapshotId` pair (`sourceSnapshots`, `previousSnapshotDigest`, `nextSnapshotDigest`). No UUID v4. No event ledger in v1.
- Catalog includes `ControlRegressed`, `ControlRecovered`, `EvidenceExpired`, `EvidenceRevoked`, `RiskIncreased` / `Decreased` / `Accepted`, `ExceptionExpired`, `NewAssetDetected` / `AssetRemoved`, `VendorRiskChanged`, and governance kinds (`ObjectiveMissed`, `PolicyExpired`, `AuditFindingOpened`, `NonconformityOpened`, `CorrectiveActionOverdue`) plus `Extensible`. Empty governance inventories on **both** snapshots are no-ops.
- Regression payload JSON locks `previousEffectiveness` / `nextEffectiveness` (also `fromEffectiveness` / `toEffectiveness`). `ExceptionApproved` → `Ineffective` / `PartiallyEffective` is a regression.
- A linked `RiskIncreased` concurrent with `ControlRegressed` includes `{ kind: event, id: <regression eventId> }` in `causeRefs`. `EvidenceExpired` is `validUntil` vs the snapshot clock, not `StaleEvidence`. `NewAssetDetected` is asset-inventory membership, not `SnapshotDiff.newSubjects`.
- Semantically equal inventories, including Vec reorder, emit nothing. Repeated detect on the same pair yields the same `eventId` set.
- `compare` / `SnapshotDiff` remain readiness helpers; `newSubjects` are still control ids. Scheduler Drift may still call `compare`; semantic events are this API (`tick` does not call `detect_events` in v1). No Slack / notification bus.

See [`docs/specs/isms-events-drift.md`](isms-events-drift.md) and [ADR 0003 ISMS events/drift](../adr/0003-isms-events-drift.md).

## Remediation engine

Canonical ISMS work record in `weeping-angel-assurance-ir::remediation` (`AssessmentDefinition.remediations`, schema `assurance-ir/v1`). Lifecycle/verification queries live in `weeping-angel-assurance` (crate-root re-exports). Scanner workbench `RemediationRequest` is a **different type** (code-patch generator). External Jira/Linear/GitHub issues are adapter refs only; canonical identity is `RemediationId` (`typed_id!`, no random v4).

```text
create_from_source / create_from_control_regression → Remediation (Proposed)
Remediation::transition / close / reopen_expired_waiver
evaluate_verification(remediation, results, as_of, verifier) → VerificationState
sla_overdue(remediation, as_of) → bool
attach_external_ticket / link_treatment_action
waiver_in_force / validate_remediation_inventory
validate_remediation_waivers_at / validate_remediation_slas_at
```

Rules:

- Sources bind to Prompt 15 `IsmsEventKind` names plus `RiskTreatmentAction` / `Manual`. `From<&IsmsEvent>` copies `eventId`; this slice does not emit events or call `detect_isms_drift`.
- Default verification is `SustainedWindow` (14d, ≥ 2 `Effective` results, no intervening fail). One green test does **not** auto-close. `SingleGreenPermitted` may satisfy on one green; `close` is always explicit (`Verified` only) with principal + time + rationale.
- `AcceptedWaived` requires an in-force `Exception` (`Approved` + unexpired `expiresAt`) or Prompt 08 risk acceptance. Expired/revoked/missing-expiry cannot remain waived (`validate_remediation_waivers_at`). `RiskStatus::Accepted` is not a waiver.
- `remoteState` on `ExternalTicketRef` is documentary. Duplicate `(system, key)` fails. No ticket HTTP clients.
- Clockless `validate_assessment_ir` walks remediation graph integrity. Closed records reject mutation (`ImmutableClosure`). Frozen closed `canonical_digest` is stable.
- Incident `correctiveActionIds` are `RemediationRef`. When `remediations` is non-empty, dangling ids fail closed. Incident close does not close remediations.

See [`docs/specs/remediation-engine.md`](remediation-engine.md) and [ADR 0003 remediation engine](../adr/0003-remediation-engine.md).

## Incident governance

Canonical organizational information-security incident record in `weeping-angel-assurance-ir` (`AssessmentDefinition.incidents`, schema `assurance-ir/v1`). Created only by `Incident::declare` / `Incident::promote` (`PrincipalRef` + `declared_at`). Scanner `Finding`, imported alerts, and Prompt 15 events are detection sources, not incidents. `IncidentKind` is `real` | `exercise` on one type; governance-catalog `control.incident.*` / `evidence.incident.exercise` stay capability tests.

`validate_assessment_ir` fail-closes on duplicate ids, dangling graph refs, unordered timelines, illegal status transitions, Real recovered/closed without recovery evidence, and Real closed without `PostIncidentReview`. PIR may propose risk/control/remediation ids; it does not mutate those inventories. When `assessment.remediations` is non-empty, dangling `correctiveActionIds` fail closed. Closed incidents with open corrective-action refs are valid. Audit/management-review **preparation** helpers: `weeping-angel-assurance::{incidents_in_period, incident_postmortem_missing, closed_incidents_with_open_corrective_actions, real_incidents, exercise_incidents}`. No SIEM, pager, forensics, or breach-notification legal engine.

See [`docs/specs/incident-governance.md`](incident-governance.md) and [ADR 0003 incident governance](../adr/0003-incident-governance.md).

## Continuity / resilience

Capability projection over `AssetKind::Service` profiles (`weeping-angel-assurance-ir::continuity`, `AssessmentDefinition.continuity_profiles`, schema `assurance-ir/v1`). Evaluation is `weeping-angel-assurance::evaluate_continuity_resilience` (crate-root re-export). Catalog `procedure_present`, current BCP (`test.resilience.continuity-plan-current`), and `test.resilience.recovery-procedure-present` remain **plan** evidence. They never prove recovery.

```text
evaluate_continuity_resilience(assessment, profile, evidence, as_of)
  → ContinuityResilienceVerdict
```

Rules:

- Dimensions: `plan_existence`, `backup_configuration`, `successful_restore`, `exercise_cadence`, `rto_achievement`, `rpo_achievement`, `unresolved_exercise_findings`, `dependency_coverage`.
- `demonstrated_recovery` is derived and **excludes** `plan_existence`. It requires a `TechnicalRecovery` / `RestoreTest` with restore Demonstrated, RTO/RPO Met, cadence Current, critical dependencies Covered, required backup evidence Satisfied or NotApplicable, and no open exercise findings.
- Tabletop / Walkthrough may satisfy cadence. They cannot set restore Demonstrated or RTO/RPO Met.
- Open exercise issues require `ContinuityRemediationRef` or evaluation fails closed (`untracked exercise finding`). Gaps are `ContinuityGap` rows (optional `RiskRef`).
- `validate_assessment_ir` fail-closes on duplicate profile/objective/exercise ids, non-Service profile subjects, dangling graph refs, zero RTO, and MissionCritical/High without cadence. Staleness is clocked evaluation, not `validate()`.
- No `BusinessService` inventory. No catalog ID rewrite. No backup-vendor types. Durations are integer seconds.

See [`docs/specs/continuity-resilience.md`](continuity-resilience.md) and [ADR 0005 continuity/resilience](../adr/0005-continuity-resilience.md).

## Internal audit

Operational ISMS internal-audit process in `weeping-angel-assurance-ir::audit` (`AssessmentDefinition.audit_programs` / `audits` / `audit_findings`, schema `assurance-ir/v1`). Machine **prepares**; a human auditor **accepts samples, records findings, and signs**. `Effectiveness` never writes `Audit.conclusion` or `Audit.signOff`. `AuditSignOff` has no `Default`. Scanner `Finding` is never auto-promoted.

Engine (`weeping-angel-assurance::audit`, not crate-root re-exports):

```text
prepare_audit_program / prepare_audit → draft program or prepared Audit + AuditPrepareBundle
propose_sample → AuditSampleProposal (refuses judgmental)
accept_sample → AuditSample (proposal alone is not the sample)
pin_evidence(audit, EvidenceSnapshot, principal, clock)
record_finding / conclude_audit / sign_off
replay_audit / reviewed_envelopes
```

Rules:

- `requests.audit_program` / `supports_audit_program` (and sampling) stay fail-closed compile gates. Flags do not replace documents. `validate_assessment_ir` walks inventories whenever they are non-empty.
- Sampling is explicit: method, seed (required for `systematic` / `seededRandom`), sorted population, `populationDigest`, `sampleDigest`. Same inputs replay. `judgmental` is auditor-supplied `selectedIds` only.
- `AuditEvidencePin` copies lineage `EvidenceSnapshot` digest + envelope digests. Later live `assess()` does not rewrite the pin. Replay uses the pin, not the current ledger.
- Incomplete audits (missing accepted sample, pin, accepted independence, or leftover `planned` procedures) cannot conclude. `sign_off` requires a human `PrincipalRef`, non-empty statement, and a conclusion other than `notConcluded`.
- `IndependenceRecord.accepted` is never machine-set. Conflict flags persist (`auditorOwnsControl`); absence of flags is not independence.
- `AuditFinding.nonconformityId` is an opaque Prompt 22 seam. This slice does not start CAPA. Optional catalog fact projection (`evidence.governance.internal-audit`) is not landed. `Iso27007` remains a pack-less compile selector.

See [`docs/specs/internal-audit.md`](internal-audit.md) and [ADR 0003 internal audit](../adr/0003-internal-audit.md).

## Controlled documents

Standalone governed-artifact registry in `weeping-angel-assurance-ir::document` (crate-root re-exports). Schema stays `assurance-ir/v1`. Not an editor, not a DMS, not a second `Control`, and not an `AssessmentDefinition` field.

```text
what the control means              = Control
how this org implements it          = ControlImplementation
whether the control is effective    = ControlTestResult.effectiveness
which policy version was in force   = ControlledDocument evaluation at T
```

`DocumentVersionStatus` is `draft` | `approved` | `retired`. Operational currency is **derived**: `DocumentVersion::is_operational_current_at(t)` is approved + `effectiveFrom <= t` + in review window; `ControlledDocument::is_operational_current_at(t)` requires the current pointer. `effective_version_at(id, t)` selects among operational versions, dropping those superseded by another candidate. Unscheduled `reviewBy` is not in-window. Stale is review-window metadata, not `Effectiveness::StaleEvidence`.

Artifact identity is `EvidenceEnvelope.content_digest`. `approve` requires non-empty approvers **and** approval-evidence digests (`MissingApproval`). Approved `artifactDigest` is immutable (`ImmutableApprovedArtifact`); a content change is `append_version`. CIR `DocumentRef` stays an opaque pointer. Catalog `control.governance.document-control` / `test.governance.document-control-attested` remain hybrid/`manual-review`. A current policy does **not** make an execution-required test `Effective`.

`DocumentControlRegistry::validate(&DocumentLinkUniverse)` fail-closes on dangling `ControlId` / `ObligationId` / `RiskId` / subject ids, duplicate ids, empty artifact digest, missing approval on approved versions, and supersession cycles. `ObligationId` is shared with the obligation registry.

See [`docs/specs/controlled-documents.md`](controlled-documents.md) and [ADR 0003 controlled documents](../adr/0003-controlled-documents.md).

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

`assurance catalog`, `assurance explain`, and `assurance soa` are dispatched (parser in `src/cli.rs`; execution in `src/assurance_catalog.rs`, `src/assurance_explain.rs`, and `src/assurance_soa.rs`). Explain prints the not-certification banner then JSON; unknown assessment or control exits non-zero; it does not resolve a pack from disk. `assurance soa` prints the not-certification banner then the operational SoA JSON (live `iso-27001`/`2022` convenience for `latest`; unknown pinned assessment exits non-zero) without compiler topology ([`docs/specs/operational-soa.md`](operational-soa.md)). Other `assurance` subcommands still print the non-certification banner and exit 0; library `assess` / `project_readiness` / `project_soa` / `compare` remain the execution path for those.

## Crate graph

See ADR 0001. Public composition root is `weeping-angel-assurance`. Do not take a dependency on compiler internals except for tests / debug.

Hard negatives (Phase 53 / ACT-003 / ACT-013):

```text
assurance-ir        ↛ HTTP / provider / storage / framework adapter logic
framework           ↛ GitHub / AWS / Cloudflare SDKs, collector, control-test, canonical-catalog
collector           ↛ framework / catalog / ISO / GDPR / SOC2 packages; never declares compliance
canonical-catalog   ↛ framework / collector / control-test / evidence / network
evidence            ↛ control effectiveness; validity events ↛ DigestBody
control-test        ↛ network I/O / collector
scanner types       ↛ ISO / GDPR / SOC statuses
assurance           = composition authority (includes applicability engine)
applicability eval  ↛ collector / framework adapter / IR fact evaluation / pack-TOML as Kleene input
lineage persist     ↛ current catalog/pack files on serialize or replay; mismatch is DigestMismatch
residual projection ↛ evidence envelopes / collectors; Effective ↛ zero residual
isms context        ↛ assessment results / SoA / Effectiveness / ISO clause fields / provider objects
risk methodology    ↛ Kleene / collectors / crate-wide 5×5 / f64 expected loss
risk treatment      ↛ compile_framework evaluation; Accepted enum ↛ evidence; Completed ↛ residual zero
supplier risk       ↛ HasVendor criticality; evidence_refs ↛ Approved; catalog control.vendor.* rewrite
risk identification ↛ LLM / embeddings / FrameworkProfile; identify ↛ AssessmentDefinition.risks; scanners ↛ RiskStatus::Accepted / Effectiveness
implementation status ↛ Effectiveness; Implemented ↛ Effective
controlled documents ↛ Effectiveness / DMS / editor; current policy ↛ execution-required Effective
remediation records ↛ ticket HTTP clients / kanban / notifications; one green ↛ Closed
internal audit      ↛ auto-sign / Effectiveness conclusion / scanner Finding / ISO 27007 pack
continuity          ↛ catalog rewrite; procedure_present / BCP freshness ↛ demonstrated_recovery; tabletop ↛ RTO/RPO Met
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
| Temporal assurance target (normative) | `sdd_temporal_assurance_target` | GREEN — TMP-001…012 (as-of, validity events, period, no leakage) |
| Temporal assurance baseline (historical) | `sdd_temporal_assurance_baseline` | skip-superseded / ignored |
| Evidence validity / temporal assurance target (normative) | `sdd_evidence_validity_temporal_assurance_target` | GREEN — EVT-001…012 (overlap, supersession, revoke, boundaries, stale/future/expired, period, timeline) |
| Evidence validity / temporal assurance baseline (historical) | `sdd_evidence_validity_temporal_assurance_baseline` | skip-superseded / ignored |
| Residual risk target (normative) | `sdd_residual_risk_target` | GREEN — P09-T01…T20 (modes, lineage pins, fail-closed evidence, no-reduction, exceptions not Low, history) |
| Residual risk baseline (historical) | `sdd_residual_risk_baseline` | absence characterization skip-superseded / ignored |
| ISMS context target (normative) | `sdd_isms_context_target` | GREEN — CTX-T01…T14 (golden round-trip, `AssessmentDefinition::new`, duplicate/dangling/empty/lifecycle fail-closed, framework-neutral, network-free) |
| ISMS context baseline (historical) | `sdd_isms_context_baseline` | absence characterization skip-superseded / ignored |
| Scope engine target (normative) | `sdd_scope_engine_target` | GREEN — SCP-T01…T15 (nested inclusion, exclusion precedence, expired exclusion, unresolved, duplicates, conflict, org-wide, population, out-of-scope evidence) |
| Scope engine baseline (historical) | `sdd_scope_engine_baseline` | silent/descriptive-scope characterization skip-superseded / ignored |
| Risk methodology target (normative) | `sdd_risk_methodology_target` | GREEN — P05-T01…T17 (3×3/5×5/expected-loss, fail-closed validate, lock+supersede, input≠rating) |
| Risk methodology baseline (historical) | `sdd_risk_methodology_baseline` | absence characterization skip-superseded / ignored |
| Control implementation registry target (normative) | `sdd_control_implementation_registry_target` | GREEN — CIR-001…015 (split populations, partial/retired, overlap fail-closed, Implemented ≠ Effective, supersession, dangling refs) |
| Control implementation registry baseline (historical) | `sdd_control_implementation_registry_baseline` | six-status / silent-overlap characterization skip-superseded / ignored |
| Controlled documents target (normative) | `sdd_controlled_documents_target` | GREEN — CD-001…014 (current/stale/draft, missing approval, supersession, immutable digest, acknowledgements, retention, presence ≠ Effective, fail-closed refs) |
| Controlled documents baseline (historical) | `sdd_controlled_documents_baseline` | IR-absence CD-B001/B002 skip-superseded; catalog/envelope/execution-evidence locks remain GREEN |
| Risk treatment target (normative) | `sdd_risk_treatment_target` | GREEN — P08-T01…T16 (four strategies, expiry, partial mitigate, missing contract, supersession, residual mismatch, dangling controls) |
| Risk treatment baseline (historical) | `sdd_risk_treatment_baseline` | skip-superseded (`#[ignore = "superseded by target suite"]`) |
| Supplier risk target (normative) | `sdd_supplier_risk_target` | GREEN — SR-001…015 (lifecycle, tiered review, lingering access, contract requirement, expired exception, risk linkage) |
| Supplier risk baseline (historical) | `sdd_supplier_risk_baseline` | two-field stub found-case; fails on implemented HEAD |
| Remediation engine target (normative) | `sdd_remediation_engine_target` | GREEN — RE-001…014 (control regression, treatment linkage, SLA, tickets, verification, waiver, immutable close) |
| Remediation engine baseline (historical) | `sdd_remediation_engine_baseline` | absence characterization skip-superseded / ignored |
| Risk identification target (normative) | `sdd_risk_identification_target` | GREEN — RI-001…010 (cluster, dual scenario, explicit promote, resurface, stale fail-closed, no-finding, claim-deny, candidate ≠ Risk) |
| Risk identification baseline (historical) | `sdd_risk_identification_baseline` | absence characterization skip-superseded / ignored; additive `Risk::new` / golden `risk.json` remain GREEN. `p07_b06` superseded because duplicate `RiskId` now fails closed |
| Internal audit target (normative) | `sdd_internal_audit_target` | GREEN — IA-001…009 (annual program, scoped audit, independence, deterministic sample, pin, finding, incomplete, signed, replay) |
| Internal audit baseline (historical) | `sdd_internal_audit_baseline` | absence characterization skip-superseded / ignored |
| Continuity / resilience target (normative) | `sdd_continuity_resilience_target` | GREEN — P20-T01…T16 (plan ≠ recovery, technical RTO/RPO, failed restore, stale exercise, dependency gap, missing backup, tabletop, open finding) |
| Continuity / resilience baseline (historical) | `sdd_continuity_resilience_baseline` | absence / no-module characterizations skip-superseded; catalog plan-presence found cases remain GREEN |

`cargo test --workspace --features demo` must keep scanner tests green.

Focused crate checks: `cargo test -p weeping-angel-framework`, `-p weeping-angel-evidence`, `-p weeping-angel-collector`, `-p weeping-angel-control-test`, `-p weeping-angel-assurance`, `-p weeping-angel-canonical-catalog`.
