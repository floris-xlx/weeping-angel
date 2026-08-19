# ADR 0041 — Canonical risk methodology IR and pure scoring

<!-- weeping-angel-adr-meta
id = "0041"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_risk_methodology_target` GREEN (P05-T01–T17); `sdd_risk_methodology_baseline` skip-superseded |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Implicit / ad-hoc scoring on a risk record. Does **not** supercede `Risk` as an inventory record, IR-019 dangling `RiskId` checks, collector blindness, Kleene applicability, residual-risk reduction methodologies, or scanner `severity_policy.rs`. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (IR as framework-neutral documents + `canonical_digest`), [ADR 0004](0004-documentation-architecture.md) (spec/ADR/contract paths) |
| Spec | [`docs/specs/risk-methodology.md`](../specs/risk-methodology.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_risk_methodology_target` GREEN (`tests/contracts/risk_methodology.target.rs`). `sdd_risk_methodology_baseline` `#[ignore = "superseded by sdd_risk_methodology_target"]`. |

> Filename **`0005-*`**. Do **not** add a `0003-risk-methodology.md` sibling. Catalog-program decisions share `0003-*`; documentation architecture is `0004`; Operational ISMS siblings also use `0005-*` (register, scheduler, …). Cite **this file by path**.

## Context

On SHA `6e31bf1a…`, `Risk` was `{ id, title, description, status }` with module comment *“Minimal risk record. Not a risk engine.”* Golden `tests/fixtures/assurance-ir/v1/risk.json` was four JSON keys. IR-019 only checked dangling `RiskId` on implementations. There was no `RiskMethodology`, scale, matrix, score, rating, appetite, tolerance, acceptance threshold, or scoring mode.

Operational ISMS v1 needs scoring **explicit and organization-configurable** before the register, identification, treatment, and residual slices consume it. Without a decision, later slices hardcode a 5×5, or collectors emit `RiskRating::High` as if a rating were evidence.

Questions this decision answers:

1. Where does scoring live — IR, assurance Kleene, control-test, or collectors?
2. Is a 5×5 (or `enum RiskRating { Low, Medium, High }`) crate law or methodology data?
3. How are raw inputs kept distinct from derived ratings?
4. How are methodologies versioned and frozen once used in a finalized assessment?
5. How are quantitative amounts hashed without `f64`?

## Decision

This is what shipped. Field-level law is [`docs/specs/risk-methodology.md`](../specs/risk-methodology.md).

### 1. Scoring is a pure IR function

`weeping-angel-assurance-ir` owns types, validation, and `score_risk` in [`crates/weeping-angel-assurance-ir/src/risk_methodology.rs`](../../crates/weeping-angel-assurance-ir/src/risk_methodology.rs). Re-exported from that crate’s `lib.rs`. Same layer as `canonical_digest`. No clock, network, `FrameworkProfile`, ISO clause fields, or collector id.

```text
validate_risk_methodology(&RiskMethodology) -> Result<(), RiskMethodologyError>
score_risk(&RiskMethodology, &RiskScoreInput) -> Result<ScoredRisk, RiskMethodologyError>
RiskMethodology::{try_new, lock, is_locked, supersede}
```

`score_risk` always re-validates the methodology. Invalid documents yield no rating.

Incorrect: an evaluator in `weeping-angel-assurance` (Kleene home), a control-test operator that “is High”, collector-produced ratings, or `weeping-angel-assurance-ir::risk_scoring` as a second ISMS engine (`risk_scoring` remains a register adapter over opaque `MethodologyValue`).

Schema remains `assurance-ir/v1`. Do not fork a parallel GRC schema.

### 2. Methodology is data; 5×5 is a fixture

`ScoringMode = Qualitative | SemiQuantitative | Quantitative | CustomBounded` (serde `camelCase`).

`Combination` must match the mode: qualitative → `matrix`; semi-quantitative → `matrix` \| `product` \| `sum`; quantitative → `expectedLoss`; custom-bounded → `identity`.

An organization can declare 1–3, 1–5, Low/Medium/High, or expected-loss bands **without changing control logic**. `score_risk(&methodology, &input)` is the only scoring entry. A 5×5 matrix is JSON in `tests/fixtures/assurance-ir/v1/risk-methodology-5x5.json`, not `const LIKELIHOOD_MAX: u32 = 5`.

All four modes carry likelihood and impact scales (custom-bounded scoring does not read them).

**No** global `enum RiskRating { Low, Medium, High, Critical }`. Ratings are declared `id`s on the methodology (`RatingScale`). Derived output is `DerivedRating { methodologyId, revision, ratingId }`.

Goldens (additive; `risk.json` unchanged):

| File | Mode |
| --- | --- |
| `tests/fixtures/assurance-ir/v1/risk-methodology-3x3.json` | qualitative 3×3 L/M/H matrix |
| `tests/fixtures/assurance-ir/v1/risk-methodology-5x5.json` | semi-quantitative 1–5 matrix (25 cells) |
| `tests/fixtures/assurance-ir/v1/risk-methodology-expected-loss.json` | quantitative expected-loss bands (EUR) |

### 3. Raw input in; derived rating out

```text
score_risk(methodology, RiskScoreInput) → ScoredRisk { input, score, rating }
```

There is no API that accepts a rating. Input variant must match `scoringMode`. Out-of-domain values fail closed (**no clamp**).

Collectors stay fact-only (`EvidenceValue` gains no rating variant). Target tests grep `weeping-angel-collector` for scoring types. Seal still rejects compliance narratives. A string fact `"rating": "high"` is not an ISMS rating.

Semi-quantitative `combination = matrix` looks up the cell for the rating; `RiskScore::SemiQuantitative { value }` stores `likelihood * impact` as a numeric snapshot, not as control-logic 5×5 law.

### 4. Versioned, lockable, supersedable

`RiskMethodologyId` is revision-unique (`typed_id!` in `id.rs`). `logicalId` + monotonic `revision` (≥ 1) group a family.

- `try_new` sorts scales (ordinal), matrix cells `(likelihoodOrdinal, impactOrdinal)`, and bands (`minInclusive` numeric) so insertion order cannot change `canonical_digest`.
- `lock()` sets `locked = true` (idempotent). Scoring still takes the document the caller passes; callers pin a locked revision for a finalized assessment.
- There are no in-place scoring setters. `supersede(new_id)` is the evolution path: new id, `revision + 1`, `supersedes = old.id`, `locked = false`. Parent bytes stay identical. Self-supersession fails closed.
- `RiskMethodologyError::Locked` is reserved for a semantic mutate of a locked document; the shipped API has no such mutator.

This slice does not implement assessment finalization. `IsmsContext.riskMethodologyId` (catalog infrastructure) is a typed **reference** to this id; it does not own scales or `score_risk`.

### 5. Validate fail-closed; decimals are text

Reject malformed matrices, duplicate ordinals, unreachable ratings, invalid band/appetite boundaries, mode mismatches, and out-of-domain scores.

Quantitative expected loss uses IR `CanonicalDecimal` in [`crates/weeping-angel-assurance-ir/src/decimal.rs`](../../crates/weeping-angel-assurance-ir/src/decimal.rs) (evidence-aligned grammar, exact `times` multiply, no `f64`). Authored identity is lexical (`1.0` ≠ `1.00` as bytes); band membership and expected-loss compare use numeric equality. Derived `expectedLoss` is canonicalized (no trailing fractional zeros). IR does not depend on `weeping-angel-evidence`.

Policy primitives stored on the document (not treatment actions): `appetite.ordinal ≤ tolerance.ordinal`; `acceptanceThreshold.ordinal ≤ tolerance.ordinal`.

### 6. `Risk` stays an inventory record here

This slice does **not** add `methodologyId`, likelihood, score, or rating onto `Risk`. `Risk::new` and `risk.json` remain valid. IR-019 remains dangling `RiskId` on implementations. Register / residual slices may store inputs + a methodology revision id and **must** call `score_risk` rather than inventing a second matrix.

## Consequences

- `IsmsContext` can name `RiskMethodologyId` without a second scoring type.
- Register, identification, treatment, and residual consume `score_risk` and locked revisions; they do not reimplement matrices.
- Scanner `severity_policy.rs` stays a separate attack-path matrix.
- Residual reduction ids (`residual-methodology:*`) are a different methodology family from ISMS scoring documents.
- Dual-suite `sdd_risk_methodology_{baseline,target}` is registered in root `Cargo.toml`; contracts are not auto-discovered. Absence baseline is skip-superseded.

## Non-goals

Risk identification, treatment, residual calculation, acceptance workflows, expanding the operational `Risk` register, Kleene evaluation, collectors, catalog TOML, CIA multi-axis product, merging Codex finding severity with ISMS ratings.

## Related

- Spec: [`docs/specs/risk-methodology.md`](../specs/risk-methodology.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Typed evidence (facts ≠ conclusions): [ADR 0003 typed evidence](0036-typed-evidence-canonical-serialization.md)
- Docs layout: [ADR 0004](0004-documentation-architecture.md)
- Residual (reduction, not scoring): [ADR 0003 residual risk](0032-residual-risk.md)
- Context reference: [ADR 0008 ISMS context](0008-isms-context.md)
