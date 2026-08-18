# Assurance runtime contract

Machine-facing contract for the assurance spine **and** the first ISO 27001 vertical. Security scan documents stay in [`codex-security/references/scan-contract.md`](../../codex-security/references/scan-contract.md).

Decisions:

- Spine: [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md)
- ISO vertical: [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md)

Public composition root is `weeping-angel-assurance`. Callers select a profile + capabilities; they do not import per-regime adapters.

This is an **automated readiness/assurance** contract, not a certification authority. Automated evaluation must never emit `ISO 27001 certified`, `ISO 27001 compliant`, `certification guaranteed`, or `audit passed`.

Allowed language: `ready`, `effective`, `ineffective`, `insufficient evidence`, `requires manual review`, `not applicable`, `assessment coverage`, `partially covered`.

## Schema

| Item | Value |
| --- | --- |
| IR schema | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Evidence schema | `evidence/v1` (`EVIDENCE_SCHEMA`) |
| Framework pack schema | `weeping-angel/framework-pack/v1` (`FRAMEWORK_PACK_SCHEMA`) |
| JSON | serde `camelCase` on public documents |
| Digests | SHA-256 hex of `serde_json` bytes (struct field order; `BTreeMap`/`BTreeSet` for maps/sets) |

Every IR document (`Control`, `Requirement`, `Mapping`, `EvidenceRequirement`, `PlannedControlTest`) carries `schemaVersion`. Compile rejects any other version. `Assessment` remains a **framework-crate** in-memory document compiled by `compile_framework` (not an IR type). Concurrent IR work may later introduce `AssessmentDefinition`; consume it by rebase, do not fork.

## Identity

Newtypes, stable string form, no random v4 in persisted identity:

`FrameworkId`, `FrameworkVersion`, `RequirementId`, `ControlId`, `ControlImplementationId`, `ControlTestId`, `AssetId`, `IdentityId`, `VendorId`, `ProcessingActivityId`, `EvidenceRequirementId`, `RiskId`, `ExceptionId`, `AssessmentId`, `AuditProgramId`, `EvidenceType`.

`EvidenceType` names a **fact kind**. Canonical names used by this vertical include `source.branch.protection`, `source.branch.required_reviews`, `source.codeowners.present`, `security_finding`, `manual_attestation`. It is not a framework name and must not be prefixed `github.*` / `iso27001.*` unless the provider or regime is genuinely part of the fact.

## Normative relationship

```text
Requirement → Mapping → Canonical Control → Control Test → Evidence Requirement
         ↘ EvidenceEnvelope ← Collector / scanner bridge
```

- `Control` fields: `schemaVersion`, `id`, `title`, `description`. No annex / SoA / clause / ISO fields.
- `Requirement` is a distinct type (`id`, `frameworkId`, `frameworkVersion`, `title`, `description`).
- `Mapping` is `{ fromRequirement, toControl, direction, completeness, relation, rationale }`.
  - `direction`: `forward` \| `reverse` \| `bidirectional`
  - `completeness`: `full` \| `partial` \| `related`
  - `relation`: `Equivalent` \| `Satisfies` \| `PartiallySatisfies` \| `Supports` \| `Related`
  - Mapping is never identity. Partial does not become `full`. `relation` defaults from completeness (`full` → `Satisfies`, `partial` → `PartiallySatisfies`, `related` → `Related`). Non-trivial mappings require a rationale.

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
- `stub_catalog(profile)` still returns `[]`. The ISO facade loads the pack via `load_framework_pack("iso-27001", "2022")` and `assessment_from_pack`.

Content modes: `StructuralOnly` | `LicensedContent` | `UserSuppliedContent`.

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

Additional compile errors from packs: `UnknownPack`, `UnknownRequirement`, dangling / unsupported / empty-rationale mappings.

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

Payload facts MUST NOT use credential keys (`authorization`, `token`, `cookie`, `password`, `api_key`, `apikey`, `secret`, `access_token`, `refresh_token`, `private_key`).

Facts remain `BTreeMap<String, String>` on the observation (digest-stable). The control-test evaluator parses them into `EvidenceValue` (`Null`, `Boolean`, `Integer`, `Decimal`, `String`, `Timestamp`, `Duration`, `StringSet`, `Identifier`).

### Ledger

`EvidenceLedger` (SQLite file or in-memory) owns **observations**, never conclusions.

```text
append, get, query, latest, for_subject, for_type,
for_collection_run, within_window, supersede, record_collection_run
```

Forbidden: `set_compliant`, `set_control_status`.

`append` is idempotent by digest (EVD-002). `supersede` records history via `supersedes`; it does not mutate the previous envelope.

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
| `GitHubCollector` | `collector.github` | First production collector. Provider types stay inside `github/`. |
| `LocalCollector` | `collector.local` | Structural files only (`CODEOWNERS`, policy, workflow presence). Presence ≠ effectiveness. |
| `ManualEvidence` | `collector.manual` | Requires `--attested-by` / `attested_by`. Never synthesized. |

GitHub:

- Evidence types: `source.repository.exists`, `source.repository.visibility`, `source.default_branch`, `source.branch.protection`, `source.branch.required_reviews`, `source.branch.required_status_checks`, `source.branch.force_push_protection`, `source.branch.deletion_protection`, `source.codeowners.present`, `source.admin.permissions`, `source.collaborator.permission`, `source.security.dependabot.enabled`, `source.security.secret_scanning.enabled`, `source.security.code_scanning.configured`, `source.workflow.permissions`, `source.workflow.review_requirement`, `source.ruleset.present`, `source.repository.archived`, `source.commit.signing`.
- Required permissions: `contents:read`, `administration:read`, `metadata:read`.
- HTTP 403 → `PermissionDenied` (downstream `InsufficientEvidence`), never boolean false.
- Retry 429 / 502 / 503 / 504 / transient network. Do not retry 401, most 403, invalid configuration/scope.
- Tokens are redacted from diagnostics and never persisted.

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
| Type mismatch on a field predicate | fail-closed (not `effective`) |

Bounded `TestExpr` (not a script host): `Exists`, `Missing`, `Eq`/`Neq`/`Gt`/`Gte`/`Lt`/`Lte`, `Contains`/`NotContains`, `In`, `Count`, `FreshWithin`, `CoverageAtLeast`, `All`/`Any`/`None`/`Not`, `ManualReview`.

`EvidenceSelector = { evidenceType, subjectSelector, field, freshness }`. No collector id in test definitions.

`ControlTestResult = { testId, controlId, effectiveness, rationale, evidenceRefs, missingEvidence, evaluatedAt, testVersion, inputDigest, duration? }`. Wall-clock `duration` is not part of semantic identity. Same test + evidence snapshot + evaluation context → same semantic result.

Tests are **canonical** (`test.source.required-review`), never `iso27001.a.x.y.github.*`.

## Scanner bridge

Owned by `weeping-angel-assurance::bridge`. Does **not** rewrite `EngineHit::to_semantic_finding`.

| Source | Observation type | Facts |
| --- | --- | --- |
| `EngineHit` | `security_finding` | `rule_id`, `path`, `category`, `canonical_type` + title narrative |
| `SemanticFinding` | `security_finding` | `rule_id`, `finding_id` + title narrative |

`canonical_type` is one of: `security.finding`, `security.vulnerability.present`, `security.exposure.present`, `security.authz.weakness`, `security.secret.exposure`, `security.tls.misconfiguration`, `security.header.misconfiguration`, `security.dependency_confusion_risk`.

One-way. Observations do not write back onto findings. Empty scan ≠ Effective control. Do not emit `security.no_vulnerabilities` as a passable fact.

Security types that remain valid and uncollapsed: `EngineHit`, `SemanticFinding`, `Candidate`, `ArtifactRecord`, `CoverageDocument`. They must not grow `iso27001` / `gdpr` / `soc2` (or siblings).

## Facade

```text
AssuranceEngine::builder()
    .collector(C: EvidenceCollector)
    .framework(FrameworkTarget)
    .assess(AssessmentScope) → AssessmentReport | AssuranceError
```

`AssessmentReport` rust fields remain `{ assessmentId, profile, digest, results, evidenceCount }`. Serialization also writes `disclaimer`, `banner`, `frameworkPackDigest`, control/requirement summaries, and coverage counts. No `compilerTopology` / `collectorGraph`.

For `FrameworkProfile::Iso27001` + version `2022`, `assess` loads the ISO pack. Other profiles still compile the canonical stub assessment (one partial mapping, `branch_protection` evidence) so ACT spine tests stay honest.

Callers must not branch on ISO vs GDPR vs SOC 2 implementations to run a generic assess.

Projections (not certificates):

```text
project_readiness(...) → FrameworkReadinessSnapshot
project_soa(framework, version) → StatementOfApplicability
compare(previous, next) → SnapshotDiff
```

A requirement mapped only with `PartiallySatisfies` / `Supports` cannot become fully `effective` even if every mapped control is `Effective` (`partially covered`).

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
```

The binary currently prints the non-certification banner and exits 0. Library `assess` / `project_readiness` / `project_soa` / `compare` are the execution path.

## Crate graph

See ADR 0001. Public composition root is `weeping-angel-assurance`. Do not take a dependency on compiler internals except for tests / debug.

Hard negatives (Phase 53 / ACT-003 / ACT-013):

```text
assurance-ir     ↛ HTTP / provider / storage / framework adapter logic
framework        ↛ GitHub / AWS / Cloudflare SDKs, collector, control-test
collector        ↛ framework / ISO / GDPR / SOC2 packages; never declares compliance
evidence         ↛ control effectiveness
control-test     ↛ network I/O / collector
scanner types    ↛ ISO / GDPR / SOC statuses
assurance        = composition authority
```

## Tests

| Suite | Cargo name | Status |
| --- | --- | --- |
| Spine target (normative) | `sdd_assurance_runtime_target` | GREEN — ACT-001…015, COL-001…006 |
| Spine baseline (historical) | `sdd_assurance_runtime_baseline` | superseded / ignored |
| ISO target (normative) | `sdd_iso27001_assurance_target` | GREEN — ISO-001…010, EVD-001…010, CTL-001…012, GH-001…012 + MVP assess |
| ISO baseline (historical) | `sdd_iso27001_assurance_baseline` | superseded / ignored |

`cargo test --workspace --features demo` must keep scanner tests green.

Focused crate checks: `cargo test -p weeping-angel-framework`, `-p weeping-angel-evidence`, `-p weeping-angel-collector`, `-p weeping-angel-control-test`, `-p weeping-angel-assurance`.
