# Assurance runtime contract (Phases 0–8)

Machine-facing contract for the assurance spine. Security scan documents stay in [`codex-security/references/scan-contract.md`](../../codex-security/references/scan-contract.md). Architecture decision: [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md).

This contract is **library-only**. There is no `weeping-angel assurance` CLI in this slice.

## Schema

| Item | Value |
| --- | --- |
| IR schema | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| JSON | serde `camelCase` on public documents |
| Digests | SHA-256 hex of `serde_json` bytes (struct field order; `BTreeMap`/`BTreeSet` for maps/sets) |

Every IR document (`Control`, `Requirement`, `Mapping`, `EvidenceRequirement`, `PlannedControlTest`, `Assessment`) carries `schemaVersion`. Compile rejects any other version.

## Identity

Newtypes, stable string form, no random v4 in persisted identity:

`FrameworkId`, `FrameworkVersion`, `RequirementId`, `ControlId`, `ControlImplementationId`, `ControlTestId`, `AssetId`, `IdentityId`, `VendorId`, `ProcessingActivityId`, `EvidenceRequirementId`, `RiskId`, `ExceptionId`, `AssessmentId`, `AuditProgramId`, `EvidenceType`.

`EvidenceType` names a **fact kind** (`branch_protection`, `repository_visibility`, `security_finding`, `exposed_without_auth`, `manual_attestation`). It is not a framework name.

## Normative relationship

```text
Requirement → Mapping → Canonical Control → Control Test → Evidence Requirement
```

- `Control` fields: `schemaVersion`, `id`, `title`, `description`. No annex / SoA / clause / ISO fields.
- `Requirement` is a distinct type (`id`, `frameworkId`, `frameworkVersion`, `title`, `description`).
- `Mapping` is `{ fromRequirement, toControl, direction, completeness }`.
  - `direction`: `forward` \| `reverse` \| `bidirectional`
  - `completeness`: `full` \| `partial` \| `related`
  - Mapping is never identity. Partial does not become `full`.

`ComplianceGraph::equivalent(a, b)` is true only when both `a→b` and `b→a` exist with `completeness = full`. A partial path `A → B → C` is never `A ≡ C`. Reverse edges are not invented.

## Compile

```text
compile_framework(assessment, target) → CompiledFramework | FrameworkCompileError
```

`FrameworkTarget = { profile, capabilities, version, context }`.

**Profiles** (compile selectors, not public catalog crates): `iso-27001`, `iso-27701`, `gdpr`, `soc-2`, `nis-2`, `dora`, `iso-27007`. Unknown string → `UnknownProfileError` / `FrameworkCompileError::UnknownProfile`, not a panic.

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

This slice: profile catalogs are empty stubs. Compile uses the assessment IR. `stub_catalog(profile)` returns no requirements.

## Evidence

Collectors and the scanner bridge emit **observations**, never compliance.

Allowed: `"repository X has branch_protection enabled"`, `"route Y is exposed_without_auth"`.

Forbidden (seal / collect error): `"ISO 27001 compliant"`, `"GDPR compliant"`, `"SOC 2 compliant"`, `"NIS2 compliant"`, `"DORA compliant"`, `ControlTestResult`.

`EvidenceEnvelope` is `{ observation, provenance, digest }`. Once sealed, mutation is a new envelope. Digest covers observation + provenance. Same payload+provenance → same digest.

`provenance = { collectorId, collectedAt, scope, asset }`.

Payload facts MUST NOT use credential keys (`authorization`, `token`, `cookie`, `password`, `api_key`, `apikey`, `secret`, `access_token`, `refresh_token`, `private_key`).

## Collectors

```text
trait EvidenceCollector {
    fn descriptor(&self) -> CollectorDescriptor;
    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError>;
}
```

`CollectorDescriptor = { id, version, evidenceTypes }`. **`frameworks` is invalid** — the field must not exist.

Rules (COL-001…006):

| ID | Rule |
| --- | --- |
| COL-001 | Emit only declared `evidenceTypes`. |
| COL-002 | No framework results (`control_test_result`, compliance sentences). |
| COL-003 | No credentials in payloads. |
| COL-004 | Deterministic normalize (fixture collector uses a fixed `collectedAt`). |
| COL-005 | Retry does not duplicate: `EvidenceSet` is keyed by digest. |
| COL-006 | Out-of-scope asset → `CollectorError::OutOfScope` (never silent collect). |

This slice ships `FixtureCollector` only. Hosted GitHub/AWS/Cloudflare collectors are Phase 15.

## Control tests

```text
evaluate(CompiledControlTest, EvidenceSet, AssessmentContext) → ControlTestResult
```

Zero network I/O. Signature has no provider / collector id. `EvidenceSet` is provider-blind.

`Effectiveness`: `effective` \| `ineffective` \| `insufficientEvidence` \| `inconclusive`.

| Situation | Result |
| --- | --- |
| Fresh observation matching required types | `effective` |
| Observation in `breakOn` (e.g. `exposed_without_auth`) | `ineffective` |
| Missing required evidence / empty set / no required types | `insufficientEvidence` (never `effective`) |
| “No vulns” / `security_findings_absent` | not `effective` |
| Stale evidence (`now - collectedAt > maxAge`) | `inconclusive` (never `effective`) |
| Manual test without `manual_attestation` | `insufficientEvidence` (cannot auto-pass) |

`ControlTestResult = { testId, controlId, effectiveness, rationale }`. `deny_unknown_fields` on serde.

## Scanner bridge

Owned by `weeping-angel-assurance::bridge`. Does **not** rewrite `EngineHit::to_semantic_finding`.

| Source | Observation type | Facts |
| --- | --- | --- |
| `EngineHit` | `security_finding` | `rule_id`, `path`, `category` + title narrative |
| `SemanticFinding` | `security_finding` | `rule_id`, `finding_id` + title narrative |

One-way. Observations do not write back onto findings. Empty scan ≠ Effective control.

Security types that remain valid and uncollapsed: `EngineHit`, `SemanticFinding`, `Candidate`, `ArtifactRecord`, `CoverageDocument`. They must not grow `iso27001` / `gdpr` / `soc2` (or siblings).

## Facade

```text
AssuranceEngine::builder()
    .collector(C: EvidenceCollector)
    .framework(FrameworkTarget)
    .assess(AssessmentScope) → AssessmentReport | AssuranceError
```

`AssessmentReport = { assessmentId, profile, digest, results, evidenceCount }`. No `compilerTopology` / `collectorGraph`.

This slice’s `assess` compiles a **canonical stub assessment** (one partial `Requirement → Control` mapping, `branch_protection` evidence requirement) so every profile selector runs the same pipeline. It is not a product ISO/GDPR catalog.

Callers must not branch on ISO vs GDPR vs SOC 2 implementations to run a generic assess.

## Crate graph

See ADR 0001. Public composition root is `weeping-angel-assurance`. Do not take a dependency on compiler internals except for tests / debug.

## Tests

| Suite | Cargo name | Status |
| --- | --- | --- |
| Target (normative) | `sdd_assurance_runtime_target` | GREEN — ACT-001…015, COL-001…006 |
| Baseline (historical) | `sdd_assurance_runtime_baseline` | superseded / ignored |

`cargo test --workspace --features demo` must keep scanner tests green.
