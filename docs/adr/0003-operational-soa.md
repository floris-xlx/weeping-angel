# ADR 0003 — Operational Statement of Applicability (graph projection + immutable snapshots)

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_operational_soa_target` GREEN; baseline skip-superseded after proving the `implementation_state = assessed` shortcut failed |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “SoA is a pack-TOML convenience projection with `implementation_state = assessed`” *shortcut* of [ADR 0002](0002-iso-27001-assurance-vertical.md) Phase 34 **as implemented** in `soa.rs`. Does **not** supercede remap pack IDs, Kleene evaluation, lineage persist schema `StatementOfApplicabilitySnapshot`, or non-certification language |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [applicability engine](0003-applicability-engine.md), [assessment lineage](0003-assessment-lineage.md), [ISO remap](0003-iso27001-canonical-remap.md) |
| Spec | [`docs/specs/operational-soa.md`](../specs/operational-soa.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Prompt | [`docs/prompts/operational-isms-v1/11-operational-soa.md`](../prompts/operational-isms-v1/11-operational-soa.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` plus current `soa.rs` / CLI (not stale remap-baseline SHA text) |
| Tests | `sdd_operational_soa_baseline` skip-superseded · `sdd_operational_soa_target` GREEN. Cite `tests/contracts/operational_soa.{baseline,target}.rs` |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**. `0004` remains documentation architecture.

## Context

Canonical Assurance shipped a generic three-state SoA (`Applicable` / `NotApplicable` / `Unresolved`) and a lineage persist type `StatementOfApplicabilitySnapshot`. ISO remap requires justified NA and representable unresolved. That is necessary and **not** sufficient for an operational ISMS.

On characterization HEAD:

1. `project_soa(framework, version)` reread live pack `applicability.toml` via `resolve_pack_dir`.
2. `implementation_state` was hardcoded `"assessed"`; `automated_effectiveness` was always `None`; evidence and exceptions were empty.
3. Missing implementation could not be a first-class applicable row; it was easy to confuse with NA.
4. NA had pack rationale but no approval/review lifecycle; expiry could not surface a readiness gap.
5. `project_soa_from_snapshot` cloned a snapshot but was not crate-root exported; live CLI `assurance soa` banner-and-exit-0.
6. `SnapshotDiff` had no SoA cause taxonomy (applicability / implementation / effectiveness / exception-expiry / mapping / treatment).

Operational ISMS v1 Prompt 11 requires the SoA to be generated from the operational graph and to explain every inclusion, exclusion, implementation, and effectiveness state — still a readiness projection, never a certificate.

Questions this decision answers:

1. Is SoA a pack-file copy or a graph projection?
2. Are applicability, implementation, and effectiveness the same dimension?
3. What is required to call a row not applicable?
4. How is a prior audit-period SoA reconstructed, and how do snapshots differ?
5. What happens when treatment / risk-register / implementation-registry engines are not landed?
6. May generic IR objects grow ISO Annex A fields?

## Decision

This is what shipped. Field-level law is [`docs/specs/operational-soa.md`](../specs/operational-soa.md).

### 1. SoA is an operational-graph projection

`weeping-angel-assurance::soa` projects rows over:

- Kleene `ApplicabilityDecision` when `OperationalSoaInput.kleene` is present (preferred);
- pack `applicability.toml` as **default rules / structural flags only** when Kleene has no decision for that requirement;
- IR `AssessmentDefinition` (mappings, implementations, risks, exceptions, scope);
- `ControlTestResult` / `Effectiveness`;
- minimum versioned `RiskTreatmentRef` / `RiskRegisterRef` when treatment-driven reasons are claimed.

Generic IR `Control` / `ControlImplementation` **MUST NOT** gain ISO Annex A fields. SoA remains a projection over ISO pack requirement IDs + catalog mappings. Licensed ISO/IEC normative text is not stored.

Public API (crate-root):

```text
project_soa(framework, version) → StatementOfApplicability
project_operational_soa(&OperationalSoaInput) → Result<StatementOfApplicability, OperationalSoaError>
project_soa_from_snapshot(&StatementOfApplicabilitySnapshot) → StatementOfApplicability
pin_soa_snapshot(soa, frameworkPackDigest) → StatementOfApplicabilitySnapshot
diff_soa_snapshots(previous, next) → SnapshotDiff
```

Schemas:

```text
weeping-angel/operational-soa-input/v1
weeping-angel/risk-treatment-ref/v1
weeping-angel/risk-register-ref/v1
```

Lineage persist schema for the sealed document stays `weeping-angel/assessment-lineage/v1`.

### 2. Three independent dimensions

```text
applicability    ≠ implementation ≠ effectiveness
```

- Missing registry row ⇒ `implementationStatus = notImplemented` on an **Applicable** row. Never coerced to SoA `NotApplicable`.
- Insufficient / missing evidence ⇒ `Effectiveness::InsufficientEvidence` (and gap `insufficientEvidence`), not NA.
- `Implemented` does not imply `Effective`. Effectiveness comes only from `ControlTestResult`.
- IR `ImplementationStatus::NotApplicable` does not set SoA applicability.

Live `project_soa(framework, version)` builds an empty-graph input (no implementations, results, or treatments) and therefore emits applicable + `notImplemented` + no effectiveness. It is a convenience, not history.

### 3. NA is governed, not silent

Not-applicable requires explicit context rationale **and** accountable approval/review (principal + review state). Pack-declared ISO `A.5.19` remains representable as `NotApplicable` (remap ISO-R-009 / g07). Incomplete context remains `Unresolved` (`A.8.13` ↔ Kleene `ManualDeterminationRequired`, remap g08).

Expired or missing NA approval surfaces a **readiness gap** (`expiredNaApproval` / `missingNaApproval`). The row is not dropped and is not silently coerced to Applicable. Context rationale must not be “missing evidence.”

`owner` is projected as `None` until Prompt 10 exposes an IR getter.

### 4. History is pinned; live convenience is not history

`pin_soa_snapshot` computes `typed_canonical_digest("soa-snapshot", body)` over schema + framework pack digest + SoA payload, **not** later live pack-file bytes. Reconstruct with `project_soa_from_snapshot` (clone; crate-root export). A later `applicability.toml` edit does not rewrite a pinned digest.

`diff_soa_snapshots` extends `SnapshotDiff.soaCauses` with:

```text
SoaDiffCause =
  ApplicabilityChange
  | ImplementationChange
  | EffectivenessRegression
  | ExceptionExpiry
  | MappingChange
  | TreatmentChange
```

Effectiveness regression is `Effective` → `Ineffective` / `PartiallyEffective` / `InsufficientEvidence` / `StaleEvidence`.

### 5. Fail closed on missing engines; do not implement them here

Prompts 02 / 06 / 08 / 10 may be unlanded. This slice defines **minimum versioned references** and returns `OperationalSoaError` when a treatment-driven projection lacks them:

| Condition | Error |
| --- | --- |
| Treatment-driven requirement ids and empty/missing cited treatment | `MissingRiskTreatment` |
| Non-empty treatments without a register digest | `MissingRiskRegister` |
| Present treatment/register ref with empty digest | `MissingInputDigest` |
| `require_kleene` and no Kleene snapshot | `MissingApplicabilitySnapshot` |

Live `project_soa` does **not** fail closed for missing treatment engines. Partial canonical mapping (`PartiallySatisfies` / `Supports` / `Related` / `EvidenceFor` / `SubsetOf`) stays applicable unless Kleene/NA says otherwise, lists catalog ids, and records `partialCanonicalMapping`. Pack `to =` remaps are not rewritten here.

### 6. Public CLI/report contract stays non-certifying

Disclaimer: `"This Statement of Applicability projection is a readiness aid and is not certification."`

`src/assurance_soa.rs` dispatches `assurance soa`: banner then JSON. `latest` / empty / named assessment reconstructs from `assurance-ledger.sqlite` via `replay_assessment` + `project_soa_from_snapshot` ([ADR 0011](0011-temporal-lineage-evidence-soa-integrity.md)). Missing ledger, missing run, or replay failure exits non-zero — live `project_soa` is **not** history. No `compilerTopology`. Never emit `ISO 27001 certified` / `compliant` / `certification guaranteed` / `audit passed`.

## Crate homes (no new crate)

| Concern | Home |
| --- | --- |
| Operational projection, NA governance, fail-closed refs, pin, SoA diff | `crates/weeping-angel-assurance/src/soa.rs` |
| `SoaDiffCause` + additive `SnapshotDiff.soaCauses` | `crates/weeping-angel-assurance/src/snapshot.rs` |
| Persist type `StatementOfApplicabilitySnapshot` | `crates/weeping-angel-assurance/src/lineage.rs` (unchanged schema) |
| Crate-root re-exports | `crates/weeping-angel-assurance/src/lib.rs` |
| CLI | `src/assurance_soa.rs`; parser `src/cli.rs`; dispatch `src/main.rs` |

## Non-goals

- Licensed ISO/IEC 27001 normative text.
- Certification / compliance / audit-passed claims; dashboards.
- Reimplementing Kleene; remapping ISO pack IDs or catalog domain TOML.
- Residual-risk, control-implementation registry, scope, risk-register, or treatment engines.

## Consequences

- `SoaEntry` grows serde-default operational fields (`linkedRisks`, `implementationStatus`, `effectivenessStatus`, `readinessGaps`, `approval`, …). Existing remap JSON readers keep working if they ignore unknowns / use current keys. `implementationState` is a compatibility alias of `implementationStatus`.
- `SnapshotDiff` grows additive `soaCauses`.
- Neighbor suites `sdd_iso27001_remap_target`, `sdd_iso27001_assurance_target`, `sdd_assessment_lineage_target`, `sdd_applicability_engine_target` stay GREEN.
- Collision fence: do not edit residual-risk, control-implementation-registry, GitHub collector, catalog domain TOML, ISO pack IDs / `to =`, remap tests, or Kleene modules.

## Related

- Spec: [`docs/specs/operational-soa.md`](../specs/operational-soa.md)
- Tests: `tests/contracts/operational_soa.baseline.rs`, `tests/contracts/operational_soa.target.rs`
- Remap SoA: [`docs/specs/iso-27001-canonical-remap.md`](../specs/iso-27001-canonical-remap.md) §4.6
- Runtime contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Lineage persist: [`docs/specs/assessment-lineage.md`](../specs/assessment-lineage.md)
- Historical SoA / four-clock evidence: [ADR 0011](0011-temporal-lineage-evidence-soa-integrity.md)
