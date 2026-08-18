# ADR 0001 — Inwardly extensible polyglot assurance runtime

| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-18 |
| Deciders | Weeping Angel maintainers |
| Slice | Phases 0–8 (spine). Catalogs and hosted collectors are later phases. |
| Spec | [`docs/sdd/assurance-runtime-spine.md`](../sdd/assurance-runtime-spine.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Tests | `sdd_assurance_runtime_target` GREEN; `sdd_assurance_runtime_baseline` superseded |

## Context

Weeping Angel is a Rust security toolchain. It emits Codex Security documents (`EngineHit` → `SemanticFinding` / `Candidate`, sealed with `ArtifactRecord` and `CoverageDocument`). The CLI (`scan`, `scan-code`, `scan-diff`, `depcheck`, `workbench`, `finalize`) has no assurance vocabulary.

The product must speak ISO 27001, ISO 27701, GDPR, SOC 2, NIS2, DORA, and ISO 27007 **without** becoming a Vanta-style pile of framework-specific checks on findings (`finding.iso_27001`, `finding.gdpr`, `finding.soc2`).

Athena solved the analogous problem for queries: a dialect-neutral `Statement` IR, `CompileTarget { profile, capabilities }`, fail-closed `compile`, and backends as **internal** compilers. Callers do not carry PostgreSQL vs D1 vs CQL types through the public SDK.

## Decision

Weeping Angel is an **inwardly extensible polyglot assurance runtime**. This slice implemented the compiler spine, not the catalogs.

1. **Public contract** is capabilities + compiled assessment + control-test results. Callers select a profile selector and flags; they do not import per-regime implementations.
2. **Internal adapters** implement framework profiles. `compile_framework` dispatches on `FrameworkProfile`. Phases 9–17 catalogs are empty stubs (`stub_catalog` returns `[]`).
3. **IR** (`weeping-angel-assurance-ir`) is framework-neutral. Schema `assurance-ir/v1`. `Requirement` ≠ `Control`. `Mapping` carries direction and completeness. `Control` has no ISO-specific fields.
4. **Compile** is `compile_framework(assessment, target) → CompiledFramework | FrameworkCompileError` with a fixed eight-stage pipeline: normalize → resolve applicability → validate capabilities → resolve control mappings → resolve evidence requirements → construct test plan → construct framework projection → integrity digest.
5. **Collectors** advertise **evidence types**, never frameworks. `CollectorDescriptor.frameworks` does not exist. They emit immutable `EvidenceEnvelope` observations, never compliance sentences.
6. **Control-tests** are pure: no network I/O, provider-blind `EvidenceSet` (keyed by digest), fail-closed on missing / stale / manual-without-attestation. Empty findings are not `Effective`. A breaking observation may be `Ineffective`.
7. **Crosswalks** (`ComplianceGraph`) preserve direction. Equivalence exists only for an explicit **full bidirectional** mapping. Partial paths never upgrade to equivalent.
8. Existing security-domain types remain uncollapsed. The one-way bridge is `EngineHit` / `SemanticFinding` → `EvidenceObservation` (`security_finding`). `to_semantic_finding` is unchanged.

Governing rule: *Framework adapters internally. Capabilities externally. Canonical assurance contract publicly.*

### Public facade (implemented)

```text
AssuranceEngine::builder()
    .collector(fixture_or_later_hosted)
    .framework(FrameworkTarget { profile, capabilities, version, context })
    .assess(scope) → AssessmentReport | AssuranceError
```

`AssessmentReport` is `{ assessmentId, profile, digest, results, evidenceCount }`. It does not expose compiler or collector topology.

CLI `weeping-angel assurance {…}` is specified for a later slice. This slice does **not** add an `Assurance` command.

### Crate graph (as built)

Workspace: root package `weeping-angel` (scanner, bins, tests) + six members under `crates/`.

```text
weeping-angel-assurance-ir
        ├── weeping-angel-framework          (IR only; no network/SDK)
        └── weeping-angel-evidence
                └── weeping-angel-collector  (evidence + IR identity types;
                                              no framework crate, no ISO/GDPR/SOC2 packages)

weeping-angel-assurance-ir + weeping-angel-evidence
        └── weeping-angel-control-test       (offline)

weeping-angel-framework
  + weeping-angel-collector
  + weeping-angel-control-test
  + weeping-angel                            (scanner types, bridge only)
        └── weeping-angel-assurance          (facade)
```

Forbidden edges, enforced by `ACT-003` / `ACT-013`:

- framework ↛ collector, control-test, reqwest, AWS/GitHub/Cloudflare SDKs
- collector ↛ framework / ISO / GDPR / SOC2 / NIS2 / DORA crates
- control-test ↛ collector / network clients
- IR ↛ any upper crate

### Five invariants (frozen)

| ID | Invariant | Implemented fail-closed |
| --- | --- | --- |
| INV-1 | A Finding is not a compliance result | No `iso_27001` / `gdpr` / `soc2` fields on `SemanticFinding` / `EngineHit`. Bridge projects an observation. |
| INV-2 | A collector cannot declare compliance | `EvidenceEnvelope::seal` and `FixtureCollector` reject compliance narratives and `control_test_result`. |
| INV-3 | A framework cannot perform network I/O | `weeping-angel-framework` depends only on serde/thiserror/IR. |
| INV-4 | A Control-Test cannot know which provider produced evidence | `evaluate(CompiledControlTest, EvidenceSet, AssessmentContext)` has no provider id. |
| INV-5 | A crosswalk cannot manufacture equivalence | `ComplianceGraph::equivalent` requires full bidirectional edges. |

## Consequences

**Positive**

- New regimes are new compile profiles, not new fields on `SemanticFinding`.
- Scanners stay honest: a vuln can prove a control ineffective; silence cannot prove compliance.
- Capability flags make “this profile cannot do SoA/RoPA/sampling” a typed `CapabilityViolation` instead of a silent skip.
- Dual-suite TDD: baseline characterized the pre-spine scanner; target encodes ACT-001…015 and COL-001…006. Baseline is superseded.

**Negative / cost**

- Workspace split (scanner stays at repo root so packager / `CARGO_MANIFEST_DIR` fixtures stay valid).
- Callers cannot add a GDPR column to findings; they go through IR + compile + test.
- Facade `assess` currently compiles a **canonical stub assessment** (one partial mapping, `branch_protection` evidence). Full catalogs are Phases 9–14.
- Collector crate may depend on IR **identity** types (`AssetId`, `EvidenceType`) but not on framework catalogs.

**Rejected alternatives**

- Annotating `SemanticFinding` with framework maps.
- Per-framework check crates that call GitHub/AWS directly and print “compliant”.
- Letting collectors declare `descriptor.frameworks`.
- Inferring equivalence by walking a related-control graph.
- Treating a sealed empty findings bundle (`coverage.completeness == complete`) as a control pass.

## Access and security

- Evidence envelopes MUST NOT carry credentials (`authorization`, `token`, `cookie`, `password`, `api_key`, …). Seal fails closed.
- Collector scope is fail-closed (`CollectorError::OutOfScope`).
- Framework compile and control-tests are offline.
- Envelope identity is SHA-256 over canonical serde JSON (`BTreeMap` facts). Retry is idempotent by digest (`EvidenceSet` is a digest map).
- Facade `assess` uses a 24h evidence max-age.

## Deferred (not this decision)

- Phases 9–14: real ISO 27001 / 27701 / GDPR / SOC 2 / NIS2 / DORA catalogs.
- Phase 15: hosted GitHub / AWS / Cloudflare collectors.
- Phase 16: persistent orchestrator (this `assess` is in-process).
- Phase 17: auditor UX / ISO 27007 program product.
- CLI `weeping-angel assurance …`.

## Related

- Spec SSOT: [`docs/sdd/assurance-runtime-spine.md`](../sdd/assurance-runtime-spine.md)
- Public contract: [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md)
- Scan contract (security-only): [`codex-security/references/scan-contract.md`](../../codex-security/references/scan-contract.md)
- Athena analogue: `athena-query` `Statement` / `CompileTarget` / `compile` / `CapabilityViolation` (pattern only; no crate dependency)
