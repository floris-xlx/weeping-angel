# SDD: Risk Methodology IR and Scoring

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_risk_methodology_target` GREEN; baseline skip-superseded |
| Program | Operational ISMS v1 — SDLC catalog |
| Slice | Canonical risk-methodology IR + deterministic scoring/validation primitives |
| Dual-suite | `sdd_risk_methodology_baseline` · `sdd_risk_methodology_target` (registered in root [`Cargo.toml`](../../Cargo.toml); directory is **not** auto-discovered) |
| Contract files | `tests/contracts/risk_methodology.{baseline,target}.rs` |
| ADR | Accepted [`docs/adr/0005-risk-methodology.md`](../adr/0005-risk-methodology.md) (next unused numeric prefix after 0004; **not** a `0003-*` sibling). Cite by **path**. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) (landed Risk methodology section) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), [ADR 0001](../adr/0001-inwardly-extensible-assurance-runtime.md) |
| Typed evidence | [`docs/specs/typed-evidence.md`](typed-evidence.md) — collectors still emit facts, never ratings |
| Governance catalog (neighbor) | [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) — still attests risk *work*; does not become a scoring engine |
| catalog infrastructure ISMS context | Neighbor consumer. `IsmsContext.riskMethodologyId` is a typed **reference** only. This slice owns `RiskMethodology` + `RiskMethodologyId` + `score_risk`. Do **not** reimplement context here. |
| vulnerability catalog risk register | Neighbor (landed). This slice does not add score/rating fields to `Risk`. Register stores `MethodologyValue` snapshots and calls `score_inherent`; `score_risk` remains SSOT. Keep `Risk::new` and `tests/fixtures/assurance-ir/v1/risk.json` compatible. Do **not** expand the register here. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Characterization SHA | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Canonicalization | `canon/v1` via `weeping_angel_assurance_ir::canonical_digest` (compact serde JSON; no `f64`) |
| Workspace verify (after implement) | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for the SDLC catalog. It owns **methodology types**, **scale/matrix/threshold validation**, **pure scoring**, **versioned immutability + supersession**, and **the split between raw inputs and derived ratings**. It does **not** own risk identification, the operational register expansion, treatment, residual risk, acceptance workflows, or ISMS context.

Architecture law (frozen):

```text
Provider → Canonical Evidence → Canonical Test → Canonical Control → Framework Mapping
```

Risk scoring is a **pure function over IR types** (same layer as `canonical_digest`). It is not Kleene applicability evaluation and not collector output.

### Landed surface

| Item | Home |
| --- | --- |
| Types, `validate_risk_methodology`, `score_risk`, `lock` / `supersede` | [`crates/weeping-angel-assurance-ir/src/risk_methodology.rs`](../../crates/weeping-angel-assurance-ir/src/risk_methodology.rs) |
| `CanonicalDecimal` | [`crates/weeping-angel-assurance-ir/src/decimal.rs`](../../crates/weeping-angel-assurance-ir/src/decimal.rs) |
| `RiskMethodologyId` | `id.rs` `typed_id!` |
| Re-exports | `weeping-angel-assurance-ir` `lib.rs` |
| Goldens | `tests/fixtures/assurance-ir/v1/risk-methodology-{3x3,5x5,expected-loss}.json` |

`Risk` remains an inventory record in this slice. Register adapter `risk_scoring.rs` is **not** the ISMS scoring engine.

---

## 1. Problem / user-visible goal

Today a `Risk` is four fields and a comment: *“Minimal risk record. Not a risk engine.”* There is no likelihood scale, impact scale, matrix, score, rating, appetite, tolerance, acceptance threshold, or scoring mode. Any later register, treatment, or residual-risk work would have to invent a 5×5 (or worse, let a collector emit `RiskRating::High` as if that were evidence).

That is not an organization-configurable methodology. ISO-style ISMS operation requires the organization to **declare** how it scores, then reproduce that declaration on every finalized assessment.

**User-visible goal:** replace implicit/hardcoded scoring assumptions with an explicit, versioned, immutable-once-used methodology. An organization can use 1–3, 1–5, Low/Medium/High, or a quantitative expected-loss model **without modifying control logic**. Scoring takes raw likelihood/impact (or quantitative loss) and **derives** a rating. Collectors cannot emit a rating as compliance evidence.

```text
score(methodology, raw input) → ScoredRisk { input, score, rating }
                                 ↑ derived, never a collector fact
```

---

## 2. Compatibility / dependencies

Pinned at characterization SHA `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `Risk` / `RiskStatus` / `Risk::new` | `crates/weeping-angel-assurance-ir/src/risk.rs` | **Keep.** Do not add score/rating/methodology fields. vulnerability catalog expands the register. |
| `tests/fixtures/assurance-ir/v1/risk.json` | golden IR | Must still decode. Do not change required fields. |
| `RiskId` | `id.rs` `typed_id!` | Keep. Add sibling `RiskMethodologyId`. |
| IR-019 | `validation.rs` + `sdd_compliance_ir_target` | Still: dangling `RiskId` on implementations fails closed. Do not retarget IR-019 at methodologies. |
| `canonical_digest` / `typed_canonical_digest` | `digest.rs` | **Reuse.** No second digest system. No IEEE-754 in digest-critical bytes. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Do **not** fork. Methodology documents carry this schema (or omit a nested schema id and inherit). |
| `IsmsContext` / methodology *reference* | catalog infrastructure | **Consume, do not own.** Context stores `Option<RiskMethodologyId>`. Do not build context, issues, parties, objectives, cadence in this slice. |
| Applicability engine | `weeping-angel-assurance::applicability` | Collision fence. Scoring is not Kleene evaluation. |
| Collectors | `crates/weeping-angel-collector` | **No risk types.** They advertise evidence types and emit facts. |
| Codex `severity_policy.rs` | scanner attack-path matrix | **Different domain.** Do not reuse or merge with ISMS methodology. |
| Governance catalog `control.risk.*` | catalog TOML | Do not rewrite. Attestation ≠ scoring. |
| Dual-suite neighbors | root `Cargo.toml` | Register `sdd_risk_methodology_*` next to existing `sdd_*`. Contracts are **not** auto-discovered. |
| Docs layout | ADR 0004 | Human SSOT is this file. Traces go to `.sdd/runs`. Implement phase may add this path to `sdd_documentation_layout` `CANONICAL_SPECS`. |

Landed product adjustments: IR `risk_methodology` module + `RiskMethodologyId`; `CanonicalDecimal`; re-exports from `lib.rs`; golden methodology fixtures **in addition to** `risk.json`; dual-suite registration.

Do **not** redesign `Risk`, `AssessmentDefinition` inventories, collectors, catalog TOML, or applicability.

---

## 3. Current behavior (baseline — historical characterization)

Characterized against `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a`. After implement, `sdd_risk_methodology_baseline` is `#[ignore]` skip-superseded; this section remains the found-case record, not current HEAD.

### 3.1 `Risk` is a four-field record

[`crates/weeping-angel-assurance-ir/src/risk.rs`](../../crates/weeping-angel-assurance-ir/src/risk.rs):

```text
//! Minimal risk record. Not a risk engine.

RiskStatus = Open | Accepted | Mitigated | Closed   // camelCase

Risk { id: RiskId, title: String, description: String, status: RiskStatus }

Risk::new(id, title, description) → status = Open
```

There is **no** `level`, `likelihood`, `impact`, `score`, `rating`, `methodology`, `appetite`, or `residual` field. Applicability baseline already asserts `risk.rs` does not contain `"level"`. Governance catalog baseline historically characterized the same four-field record (now `#[ignore]` superseded by the catalog target, not by a scoring engine).

Golden fixture [`tests/fixtures/assurance-ir/v1/risk.json`](../../tests/fixtures/assurance-ir/v1/risk.json):

```json
{
  "id": "risk:source-tamper",
  "title": "Source tampering",
  "description": "Unauthorized change to the source of record.",
  "status": "open"
}
```

`sdd_compliance_ir_target` `ir_golden_fixtures_round_trip` decodes this file and asserts `id == "risk:source-tamper"`.

### 3.2 Identifiers and validation

- `RiskId` exists (`typed_id!`). `RiskMethodologyId` does **not**.
- `AssessmentDefinition.risks: Vec<Risk>` is an inventory list. Empty by default. Golden `assessment.json` has `"risks": []`.
- IR-019 (`ir_019_risk_references_must_resolve`): an implementation `with_risk(RiskId::new("risk:missing"))` fails `validate()` with a dangling-risk error. Resolved `RiskId`s are the **only** risk integrity check.
- Duplicate risk ids on the assessment are **not** currently rejected (risk ids are collected into a `BTreeSet` for membership only). This slice does not have to add duplicate-risk checks (vulnerability catalog).

### 3.3 No methodology / scoring APIs

Product crates (`crates/**/src/**/*.rs`) contain none of:

- `RiskMethodology`, `RiskMethodologyId`
- `LikelihoodScale`, `ImpactScale`, `RiskMatrix`
- `RiskScore`, `RiskRating`, `ScoredRisk`, `ScoringMode`
- `RiskAppetite`, `RiskTolerance`, `AcceptanceThreshold`
- `score_risk`, `validate_risk_methodology`

`lib.rs` re-exports `risk::{Risk, RiskStatus}` only.

### 3.4 Collectors have no risk types

`crates/weeping-angel-collector` has **zero** matches for `risk` in Rust sources. Collectors emit `EvidenceEnvelope` facts. Seal already rejects compliance-shaped narratives (`looks_like_compliance_claim`). There is no path for a collector to emit `RiskRating::High`.

### 3.5 Scoring is not in assurance or control-test

- Applicability is Kleene three-state over org facts (`weeping-angel-assurance::applicability`). It does not score risks. IR `Risk` has no level, so `RiskLevel` predicates stay explicit facts.
- Control-test evaluates evidence against `TestExpr`. It does not derive ISMS ratings.
- Scanner `src/contract/severity_policy.rs` is a **Codex attack-path** impact×likelihood matrix for finding severity. It is not an ISMS methodology and must not become one.

### 3.6 catalog infrastructure / register / treatment

| --- | --- |
| 01 `IsmsContext` + methodology reference | No |
| 06 operational `Risk` register fields | No (`Risk::new` unchanged) |
| 07 identification / `RiskCandidate` | No |
| 08 treatment / acceptance | No |
| 09 residual risk | No |

### 3.7 What current tests lock

- `sdd_compliance_ir_target` IR-019 dangling `RiskId`; golden `risk.json` decode.
- `sdd_applicability_engine_baseline` (ignored) historically: `Risk` has no `level`; `Risk::new` exists.
- `sdd_governance_catalog_baseline` (ignored) historically: four-field JSON, no `treatment` / `owner` / `residualScore`.
- No workspace test constructs a 3×3 or 5×5 ISMS matrix, rejects malformed matrices, or scores expected loss.

---

## 4. Desired behavior

### 4.1 Home and purity

All methodology types and scoring live in **`weeping-angel-assurance-ir`**.

Landed layout:

| Item | Home |
| --- | --- |
| `RiskMethodologyId` | `id.rs` via existing `typed_id!` (empty / too-long / invalid charset / uuid-v4 still fail) |
| Types + validate + score + supersede | `crates/weeping-angel-assurance-ir/src/risk_methodology.rs` (do not dump into `risk.rs`; keep the minimal record module) |
| Re-exports | `lib.rs` |
| `Risk` record | **unchanged** `risk.rs` |

Scoring is a pure function: same inputs + same methodology bytes ⇒ same `ScoredRisk` and same `canonical_digest`. No clock, no I/O, no `FrameworkProfile`, no collector id, no ISO clause numbers on types.

Do **not** put scoring in `weeping-angel-assurance` (that crate owns Kleene applicability / facade). Do **not** put it in `weeping-angel-collector` or `weeping-angel-control-test`.

IR must not depend on `weeping-angel-evidence`. Quantitative amounts use an IR `CanonicalDecimal` (grammar aligned with evidence `DecimalText`); they are **not** `EvidenceValue::Decimal`.

No `f64` / `f32` variant, probe, product, or compare path on methodology or scores.

### 4.2 Scoring mode (not a hardcoded 5×5)

```text
ScoringMode =
  Qualitative
  | SemiQuantitative
  | Quantitative          // expected loss
  | CustomBounded
```

Serde: `camelCase` (`qualitative`, …). Exhaustive. Unknown JSON tags fail closed.

| Mode | Raw input | How rating is derived |
| --- | --- | --- |
| Qualitative | declared likelihood **and** impact **level ids** (e.g. `low`/`medium`/`high`) | Matrix cell at those ordinals |
| SemiQuantitative | integers inside declared scale domains (e.g. 1–3 or 1–5) | Matrix cell **or** documented integer function (`product` / `sum`) then bands |
| Quantitative | probability ∈ [0, 1] **and** loss amount as `CanonicalDecimal` | `expected_loss = probability × loss` (exact decimal; §4.8) then bands |
| CustomBounded | one value inside a declared `[min, max]` domain | Bands only (no implied 5×5) |

A methodology **declares** its scales, matrix (if used), rating vocabulary, bands, and policy thresholds. Control logic never switches on “we are in a 5×5 world.” Fixture 5×5 is data, not a compiler constant.

Forbidden in product scoring/control paths:

```text
const LIKELIHOOD_MAX: u32 = 5;
enum RiskRating { Low, Medium, High }   // global closed vocabulary
```

Well-known **labels** (`low`, `medium`, `high`, `critical`, …) may appear as `id`s **inside a methodology document**. They are not a crate-wide enum that collectors can construct as evidence.

### 4.3 Scales, ratings, matrix, bands

```text
ScaleLevel     { id, label, ordinal: u32 ≥ 1, description? }
LikelihoodScale { id, levels: [ScaleLevel; n ≥ 1] }
ImpactScale     { id, levels: [ScaleLevel; n ≥ 1] }

RatingLevel    { id, label, ordinal: u32 ≥ 1, description? }
RatingScale    { levels: [RatingLevel; n ≥ 1] }

MatrixCell     { likelihoodOrdinal, impactOrdinal, ratingId }
RiskMatrix     { cells: [MatrixCell] }

NumericDomain  { min: CanonicalDecimal, max: CanonicalDecimal }  // min < max
RatingBand     { ratingId, minInclusive, maxExclusive? }
                 last band: maxExclusive omitted ⇒ domain.max inclusive
```

**Scale laws (fail closed):**

1. ≥ 1 level.
2. `id` unique within the scale (case-sensitive).
3. `ordinal` unique within the scale (no duplicate ordinal positions).
4. Ordinals are a contiguous range `1..=n` after sorting (no holes: a 1,2,4 scale is invalid).
5. `label` non-empty after trim.

**Matrix laws (qualitative; semi-quantitative when `combination = matrix`):**

1. Cell count **equals** `|L| × |I|`.
2. Every pair `(likelihoodOrdinal, impactOrdinal)` in the cartesian product appears **exactly once**.
3. Every `ratingId` on a cell exists on `RatingScale`.
4. Every `RatingScale` level is reachable from at least one cell (**no unreachable ratings**).
5. Unknown ordinals, extra cells, or sparse holes ⇒ malformed matrix.

**Band laws (quantitative, custom-bounded, and semi-quantitative `product`/`sum`):**

1. Bands non-overlapping; no gaps across the declared numeric domain.
2. Adjacent: band *k+1* `minInclusive` == band *k* exclusive upper.
3. First `minInclusive` == domain min; last covers domain max inclusive.
4. `minInclusive <` exclusive upper (or last band `minInclusive ≤ domain.max`).
5. Every band `ratingId` exists; every rating is reachable from at least one band.
6. A boundary value belongs to **exactly one** band (inclusive lower; exclusive upper; last includes max).

Invalid boundaries (overlap, gap, inverted min/max, empty domain) are rejected at methodology validate, not at score time with a silent clamp.

### 4.4 Appetite, tolerance, acceptance threshold

These are **methodology policy primitives**, not treatment actions.

```text
RiskAppetite          { maxRatingId }           // desired ceiling
RiskTolerance         { maxRatingId }           // mandatory-treatment ceiling; ordinal ≥ appetite
AcceptanceThreshold   { maxRatingId }           // governance catalog MAY accept at or below; this slice never accepts
```

Laws:

- All three rating ids exist on `RatingScale`.
- `ordinal(appetite) ≤ ordinal(tolerance)`.
- `ordinal(acceptance) ≤ ordinal(tolerance)` (cannot accept above tolerance).
- This slice **stores and validates** the thresholds. It does **not** accept a risk, suppress treatment, or write `RiskStatus::Accepted`.

### 4.5 `RiskMethodology` document

```text
RiskMethodology {
  schemaVersion: "assurance-ir/v1"
  id: RiskMethodologyId          // revision-unique, e.g. "rm:acme-default:2"
  logicalId: String              // family, e.g. "rm:acme-default" (stable-id charset)
  revision: u32                  // ≥ 1, monotonic within logicalId
  title: String                  // non-empty
  scoringMode: ScoringMode
  likelihoodScale: LikelihoodScale
  impactScale: ImpactScale
  ratingScale: RatingScale
  combination: Combination       // matrix | product | sum | expectedLoss | identity
  matrix?: RiskMatrix            // required iff combination = matrix
  domain?: NumericDomain         // required for quantitative + customBounded
  bands?: [RatingBand]           // required when combination ∈ {product,sum,expectedLoss,identity}
  currency?: String              // quantitative loss unit label (e.g. "EUR"); not a provider id
  appetite: RiskAppetite
  tolerance: RiskTolerance
  acceptanceThreshold: AcceptanceThreshold
  supersedes?: RiskMethodologyId
  locked: bool                   // true once used in a finalized assessment
}
```

`Combination` must match `ScoringMode`:

| Mode | Allowed combination | Required sections |
| --- | --- | --- |
| Qualitative | `matrix` only | matrix; scales used as labels |
| SemiQuantitative | `matrix` \| `product` \| `sum` | matrix **or** domain+bands; integer scale domains |
| Quantitative | `expectedLoss` only | domain+bands; probability/loss inputs; no required matrix |
| CustomBounded | `identity` (score is the raw bounded value) | domain+bands; scales may be unused but still valid 1-level placeholders **or** omitted only if the type uses `Option` consistently — prefer **always present** scales so qualitative/semi-quant fixtures stay uniform; custom-bounded may use a 1-level dummy scale **or** the same L/I scales for documentation. **Normative:** all four modes carry likelihood+impact scales; custom-bounded scoring **does not read them**. |

Validate rejects mode/combination/section mismatches (qualitative without matrix, quantitative with a required matrix and no bands, custom-bounded with combination `matrix`, etc.).

Constructor: `RiskMethodology::try_new(...) -> Result<Self, RiskMethodologyError>` (or `validate(self)`) so invalid documents cannot be scored.

### 4.6 Versioning, lock, supersession

1. **Revision identity** is `id` (`RiskMethodologyId`). `logicalId` + `revision` group a family.
2. `revision` starts at 1. `supersede` produces a **new** document: new `id`, `revision = old.revision + 1`, same `logicalId`, `supersedes = Some(old.id)`, `locked = false`.
3. **Immutability once used:** `lock()` sets `locked = true`. A locked document:
   - may be cloned for digest/score/serialize;
   - must **not** offer in-place setters that change scoring semantics (shipped: no scoring mutators; `RiskMethodologyError::Locked` is reserved);
   - `lock()` is idempotent;
   - `supersede` is the only legal evolution (new revision; old stays locked and byte-identical).
4. `supersedes` if present must not equal `id` (no self-supersession). Cycles are out of scope for a single-document validate; the supersede constructor sets a linear parent pointer.
5. Scoring always takes **one** methodology value. Callers that pin a finalized assessment pass the locked revision, not “whatever is current for `logicalId`.”
6. This slice does **not** implement assessment finalization. It provides `lock` + `supersede` + `locked` so catalog infrastructure/06/11 can pin a revision. Treat “used in a finalized assessment” as `locked == true` for tests.

Two methodologies with the same `logicalId` and different `revision`s are different documents. Digest differs. Scoring a 3×3 v1 input against v2 is a caller error if v2’s scale ids/ordinals no longer contain that input (out-of-domain).

### 4.7 Raw input vs derived rating

```text
RiskScoreInput =
  Qualitative       { likelihoodId, impactId }
  | SemiQuantitative { likelihood: u32, impact: u32 }
  | Quantitative    { probability: CanonicalDecimal, loss: CanonicalDecimal }
  | CustomBounded   { value: CanonicalDecimal }

RiskScore =
  Qualitative       { likelihoodOrdinal, impactOrdinal }
  | SemiQuantitative { value: u32 }             // product, sum, or unused when matrix-only
  | Quantitative    { expectedLoss: CanonicalDecimal }
  | CustomBounded   { value: CanonicalDecimal }

DerivedRating { methodologyId, revision, ratingId }

ScoredRisk { input: RiskScoreInput, score: RiskScore, rating: DerivedRating }
```

API:

```text
validate_risk_methodology(&RiskMethodology) -> Result<(), RiskMethodologyError>
score_risk(&RiskMethodology, &RiskScoreInput) -> Result<ScoredRisk, RiskMethodologyError>
```

Laws:

1. `score_risk` **requires** a methodology that already `validate`s. Invalid methodology ⇒ error, no rating.
2. Input variant must match `scoringMode`. Cross-mode (qualitative input on a quantitative methodology) fails closed.
3. Level ids must exist. Integers must lie in `1..=n` (or the declared numeric domain for custom-bounded). Probability must be in `[0, 1]`. Loss must be ≥ 0 and `expected_loss` must fall inside `domain`.
4. Scores **outside declared domains** fail closed. **No clamp** to the nearest band.
5. `ScoredRisk.rating` is always derived. There is no `score_risk` overload that accepts a `ratingId`.
6. **No** public `RiskRating::High` unit variant. Tests that need a high cell look up `ratingId == "high"` (or the fixture’s declared id) **after** scoring.
7. Collectors must not import these types. Target tests grep collector sources for `RiskRating`, `RiskMethodology`, `score_risk`, `DerivedRating`.
8. `EvidenceValue` gains **no** rating variant. Seal still rejects compliance narratives. A fact named `rating` with string `"high"` is still a fact, not an ISMS rating — infrastructure catalog+ must not treat it as `DerivedRating`.

### 4.8 `CanonicalDecimal` (no `f64`)

Grammar (same idea as typed-evidence decimal text):

```text
-? (0 | [1-9][0-9]*) ( '.' [0-9]+ )?
```

Forbidden: empty, `+`, exponent, `NaN`, `Inf`, leading zeros (`01`), trailing dot (`1.`), lone `-`.

| Use | Identity rule |
| --- | --- |
| Authored thresholds / inputs | Lexical as stored (`1.0` ≠ `1.00` as document bytes) |
| **Derived** `expectedLoss` | Canonicalize: no exponent, no trailing fractional zeros (`50.0` → `50`), no leading zeros; `-0` → `0` |

Multiplication for expected loss is **exact decimal** (scale-align integer multiply, then canonicalize). Values that are not binary-exact in IEEE-754 must still be stable (test `0.1 * 0.2` → `0.02`, not a float).

Compare for band membership uses numeric equality (scale-align), not lexical, so authored `1.0` and `1.00` compare equal **as amounts**. Document bytes remain lexical.

IR does **not** import `weeping-angel-evidence`. Duplicate the small newtype in the IR crate (or a tiny `decimal` module). Do not add a `f64` serde remote.

### 4.9 Deterministic serialization

- `#[serde(rename_all = "camelCase")]` like other IR types.
- `canonical_digest` / compact `serde_json::to_vec`.
- Scale levels serialized in ordinal order; matrix cells in `(likelihoodOrdinal, impactOrdinal)` order; bands in `minInclusive` order — constructors sort so insertion order cannot change bytes.
- Optional `None` / empty vec: follow existing IR (`skip_serializing_if` where neighbors do).
- Equivalent semantic documents constructed with different insert order share a digest.
- `locked: true` vs `false` **does** change digest (lock is part of the pinned artifact).
- Superseding revision has a different `id` and digest from its parent.

Golden fixtures (add at implement; do not mutate `risk.json`):

| File | Purpose |
| --- | --- |
| `tests/fixtures/assurance-ir/v1/risk-methodology-3x3.json` | Qualitative L/M/H matrix |
| `tests/fixtures/assurance-ir/v1/risk-methodology-5x5.json` | Semi-quantitative 1–5 matrix |
| `tests/fixtures/assurance-ir/v1/risk-methodology-expected-loss.json` | Quantitative bands + custom thresholds |

### 4.10 Fixture matrices (normative for tests)

#### 3×3 qualitative (L/M/H)

Scales: ordinals 1=`low`, 2=`medium`, 3=`high` for both likelihood and impact.

Ratings: `low` < `medium` < `high`.

```text
              Impact
              L      M      H
Likelihood L  low    low    medium
           M  low    medium high
           H  medium high   high
```

Appetite `medium`, tolerance `high`, acceptance `medium`.

Example: input `(medium, high)` → rating `high`. Boundary cell `(low, high)` → `medium`.

#### 5×5 semi-quantitative

Scales: ordinals 1..=5, ids `"1"`..`"5"` (labels `"Rare"` … `"Almost certain"` / `"Insignificant"` … `"Catastrophic"` are documentation; ids stay stable).

Ratings: `low` < `medium` < `high` < `critical` (four ratings; **not** every cell unique). Combination `matrix`. Unreachable fifth rating must be rejected if declared.

Illustrative cell law for the fixture (product-shaped, but **stored as cells**, not computed from a hardcoded product in control logic):

```text
rating =
  critical  if L*I ≥ 15
  high      if L*I ≥ 8
  medium    if L*I ≥ 4
  low       otherwise
```

The fixture **materializes** the 25 cells. Scoring looks up the cell. A later custom 4×4 must work with the same `score_risk` function.

#### Custom thresholds / quantitative

Domain expected loss `[0, 1000000]`, currency `"EUR"`.

| Band | minInclusive | maxExclusive |
| --- | --- | --- |
| `low` | `0` | `1000` |
| `medium` | `1000` | `10000` |
| `high` | `10000` | *(omitted → 1000000 inclusive)* |

Boundary tests: `1000` → `medium`; `999.99` → `low`; `10000` → `high`; `1000000` → `high`; `1000000.01` → out of domain; `0` → `low`.

`0.5 * 2000` → expected loss `1000` → `medium`.

#### Custom bounded 1–3

Domain `[1, 9]` integers-as-decimal (e.g. product of 1–3 scales stored as a single custom value). Bands: `1..=3` low, `4..=6` medium, `7..=9` high (encode with exclusive uppers `4`, `7`, last inclusive `9`).

### 4.11 Validation error surface

`RiskMethodologyError` (or equivalent) is deterministic `Display` (stable needles for tests):

| Class | Examples |
| --- | --- |
| Malformed matrix | missing cell, extra cell, unknown ordinal |
| Duplicate ordinals | two levels with ordinal `2` |
| Unreachable rating | rating `critical` never on a cell/band |
| Invalid boundaries | overlap, gap, min ≥ max, appetite > tolerance |
| Out of domain | score `6` on a 1–5 scale; probability `1.1`; expected loss above max |
| Mode mismatch | qualitative input on quantitative methodology |
| Locked | semantic mutate attempted on `locked` document |
| Identity | empty `title` / `logicalId`; invalid `RiskMethodologyId`; self-`supersedes` |
| Decimal | illegal text; `f64` not accepted |

Do not stringify as opaque `"invalid"`. Callers distinguish classes (enum variants or stable prefixes).

### 4.12 `Risk` compatibility (vulnerability catalog boundary)

- `Risk::new` signature unchanged.
- `risk.json` still deserializes.
- Do **not** add `methodologyId`, likelihood, score, or rating onto `Risk` in this slice.
- Assessment inventory remains `Vec<Risk>`. Methodologies are **not** required to live inside `AssessmentDefinition` in this slice (catalog infrastructure will reference `RiskMethodologyId` from context). Tests may score without an assessment.

### 4.13 Collision fences

| Do not touch | Owner |
| --- | --- |
| `IsmsContext`, issues, parties, objectives, cadence | catalog infrastructure |
| Scope engine | typed evidence |
| Expanding `Risk` register fields / statuses / CIA split | vulnerability catalog |
| `RiskCandidate`, promotion | infrastructure catalog |
| Treatment / acceptance workflow | governance catalog |
| Residual risk / control effectiveness projection | residual risk ([`residual-risk.md`](residual-risk.md)) |
| `crates/weeping-angel-collector/**`, `GITHUB_EVIDENCE_TYPES` | Collectors |
| `weeping-angel-assurance::applicability` | applicability engine (catalog program; already landed) |
| `src/contract/severity_policy.rs` | Scanner attack-path |
| Catalog domain TOML / ISO pack IDs | Catalog owners |
| New `0003-*` ADR filename | ADR 0004 numbering: this decision is **0005** |

---

## 5. Dual-suite protocol (HARD SDD)

`tests/contracts` is **not** auto-discovered. Dual-suite is registered:

```toml
[[test]]
name = "sdd_risk_methodology_baseline"
path = "tests/contracts/risk_methodology.baseline.rs"

[[test]]
name = "sdd_risk_methodology_target"
path = "tests/contracts/risk_methodology.target.rs"
```

Protocol (completed):

```text
Spec first (this file; no product feature code)
  → Register dual-suite at implement
  → Baseline GREEN on CURRENT code
  → Target RED on CURRENT code (right reasons: missing types/APIs, not half-written stubs)
  → Implement (IR methodology module + id + fixtures only)
  → Docs/ADR finalize (0005 Draft → Accepted)
  → Target GREEN
  → Prove baseline FAILS or skip-supersede (`#[ignore = "superseded by sdd_risk_methodology_target"]`)
  → Target still GREEN
  → cargo test --workspace --features demo; fmt --check; clippy -D warnings
```

Write tests **before** product scoring (RED → fix → GREEN). One regression test per case, titled `P05: <exact subject>`.

Absence/characterization baseline is a **replacement** transition (like catalog families): after target GREEN, baseline must fail or be `#[ignore]` superseded. Keep dual-suite registration.

---

## 6. Acceptance criteria (testable)

### 6.1 Baseline suite (historical; skip-superseded)

Encode **current** HEAD. Titles `P05: …` for the found case.

| ID | Assertion |
| --- | --- |
| P05-B01 | `risk.rs` module docs still contain `Minimal risk record. Not a risk engine.` |
| P05-B02 | `Risk::new` yields `{ id, title, description, status: Open }`; JSON has no `likelihood` / `impact` / `score` / `rating` / `methodology` |
| P05-B03 | `tests/fixtures/assurance-ir/v1/risk.json` decodes; `id == "risk:source-tamper"` |
| P05-B04 | Product crate sources have no `struct RiskMethodology`, `RiskMethodologyId`, `ScoringMode`, `fn score_risk` |
| P05-B05 | `lib.rs` re-exports `Risk` / `RiskStatus` and does not export methodology types |
| P05-B06 | Collector crate sources have no `RiskRating` / `RiskMethodology` / `score_risk` |
| P05-B07 | IR-019 still fails closed on dangling `RiskId` (`risk:missing` on an implementation) |
| P05-B08 | `id.rs` has `typed_id!(RiskId)` and does not have `RiskMethodologyId` |
| P05-B09 | No `risk_methodology.rs` module under the IR crate |
| P05-B10 | Collision fence: this suite does not import GitHub collector types or change `severity_policy.rs` |

### 6.2 Target suite (GREEN after implement)

Failed on characterization HEAD because the types and `score_risk` did not exist. After implement, the same tests pass.

| ID | Title / assertion |
| --- | --- |
| P05-T01 | `P05: 3x3 qualitative fixture scores derived ratings` — `(medium, high) → high`; `(low, high) → medium`; digest stable |
| P05-T02 | `P05: 5x5 semi-quantitative fixture is data not a compiler constant` — 25 cells; `L=5,I=5 → critical`; no `LIKELIHOOD_MAX = 5` in scoring source |
| P05-T03 | `P05: custom quantitative thresholds and expected loss` — `0.5 * 2000 → 1000 → medium`; `0.1 * 0.2 → 0.02` without `f64` |
| P05-T04 | `P05: custom bounded 1-3 domain` — value `9 → high`; value `3 → low`; value `10` rejected |
| P05-T05 | `P05: malformed matrix rejected` — missing cell / extra cell |
| P05-T06 | `P05: duplicate ordinals rejected` |
| P05-T07 | `P05: unreachable ratings rejected` |
| P05-T08 | `P05: invalid boundaries rejected` — overlap, gap, inverted domain, appetite > tolerance |
| P05-T09 | `P05: scores outside declared domain rejected` — no clamp |
| P05-T10 | `P05: deterministic canonical serialization` — insert-order independent digest; 3×3 fixture round-trip |
| P05-T11 | `P05: methodology lock and supersession` — locked parent unchanged; child `revision+1` + `supersedes`; scores pin the given revision |
| P05-T12 | `P05: boundary calculations` — band edges `1000` and `10000` as §4.10 |
| P05-T13 | `P05: raw input separated from derived rating` — `ScoredRisk` retains input; API has no rating-in parameter |
| P05-T14 | `P05: collectors cannot emit RiskRating as evidence` — no `enum RiskRating { High }`; collector sources still free of scoring types; `EvidenceValue` has no rating variant |
| P05-T15 | `P05: Risk::new and risk.json remain compatible` |
| P05-T16 | `P05: qualitative vs quantitative modes without control-logic change` — same `score_risk` function; mode is methodology data |
| P05-T17 | `P05: catalog infrastructure can name RiskMethodologyId` — type exists and validates like other stable ids; no `IsmsContext` required |

After implement: workspace verify GREEN; neighbor targets (`sdd_compliance_ir_target`, `sdd_assurance_runtime_target`, `sdd_governance_catalog_target`, `sdd_applicability_engine_target`) stay GREEN.

---

## 7. Out of scope

- Identifying risks, `RiskCandidate`, promotion/rejection (infrastructure catalog).
- Applying controls, treatment plans, `Mitigate`/`Accept`/`Avoid`/`Transfer` (governance catalog).
- Calculating residual risk or mapping `Effective` → lower score ([`residual-risk.md`](residual-risk.md)).
- Accepting risks or recording acceptance evidence (thresholds only).
- Expanding `Risk` into the operational register (vulnerability catalog).
- Building `IsmsContext`, scope engine, interested parties, or security objectives (catalog infrastructure through IAM).
- Hardcoding a crate-wide 5×5 or global `RiskRating::{Low,Medium,High}`.
- Kleene applicability evaluation; collector code; GitHub evidence types.
- Scanner Codex `severity_policy` / finding severity.
- Catalog TOML / ISO clause fields on methodology types.
- UI, persistence service, policy editor, auditor portal.
- IEEE-754 in digest-critical bytes; a second identity/digest system.
- CIA multi-axis impact product (single impact scale here; vulnerability catalog may add dimensions that still call this API).

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Accidental global 5×5 / `RiskRating::High` enum | T02/T14; ratings are declared ids; scoring source greps forbid `LIKELIHOOD_MAX = 5` and unit-variant `High` |
| Collectors emit ratings as compliance | Types stay in IR; collector grep; no `EvidenceValue` rating; seal still rejects compliance narratives |
| `f64` expected-loss drift | `CanonicalDecimal` + exact multiply; T03 uses `0.1 * 0.2` |
| Silent clamp of out-of-domain scores | T09 fail closed |
| Unreachable ratings / sparse matrices accepted | T05–T07 validate-before-score |
| Mutating a methodology under a finalized assessment | `locked` + no semantic setters; T11 byte-equality of parent after supersede |
| vulnerability catalog/01 rebase conflicts | Do not add fields to `Risk`; export `RiskMethodologyId` only |
| Scoring lands in assurance/Kleene or control-test | Module home is IR; target forbids `score_risk` in those crates |
| Merging with scanner severity matrix | Hard fence on `severity_policy.rs` |
| Dual decimal types drift from evidence grammar | Same regex; IR copy documented; no crate edge evidence → IR |
| Baseline absence tests block CI after implement | Skip-supersede like other replacement suites; keep registration |
| IR-019 accidentally retargeted at methodology ids | Leave dangling-`RiskId` semantics unchanged (B07/T15) |

---

## 9. ADR

This is an architecture/contract decision (crate home, no hardcoded matrix, input/rating split, lock/supersede, decimal law). Accepted: [`docs/adr/0005-risk-methodology.md`](../adr/0005-risk-methodology.md).

Do not add a `0003-risk-methodology.md` sibling. Catalog-program ADRs share `0003-*`; documentation architecture is `0004`; this is the next unused numeric prefix.

---

## 10. Implementation notes (landed)

Owned crate: `weeping-angel-assurance-ir` only (plus tests/fixtures/docs).

Exports:

```text
RiskMethodologyId
RiskMethodology, ScoringMode, Combination
LikelihoodScale, ImpactScale, ScaleLevel
RatingScale, RatingLevel, RiskMatrix, MatrixCell
NumericDomain, RatingBand
RiskAppetite, RiskTolerance, AcceptanceThreshold
CanonicalDecimal
RiskScoreInput, RiskScore, DerivedRating, ScoredRisk
RiskMethodologyError
validate_risk_methodology, score_risk
RiskMethodology::{try_new, lock, supersede, is_locked}
```

This slice did not add scoring fields to `Risk` / `Risk::new`.

Fixtures: 3×3, 5×5, expected-loss JSON beside `risk.json`.

Docs: [`docs/specs/assurance-runtime.md`](assurance-runtime.md) Risk methodology section; this path is in `sdd_documentation_layout` `CANONICAL_SPECS`; ADR 0005 **Accepted**.

Traces: `.sdd/runs/` only (ADR 0004). `docs/sdd` remains a stub.

---

## 11. Handoff contract (downstream slices)

```text
catalog infrastructure  IsmsContext.riskMethodologyId: RiskMethodologyId   (reference only)
vulnerability catalog  Risk record stores raw inputs + methodology revision id;
           calls score_risk; never stores a collector-supplied rating as truth
infrastructure catalog  optional score suggestion is RiskScoreInput, not DerivedRating
governance catalog  reads acceptanceThreshold / appetite / tolerance; does not reimplement bands
residual risk  residual projection takes a locked methodology revision + inherent/treatment/control-test pins
               ([`residual-risk.md`](residual-risk.md); not the GitHub collector)
```

Downstream must **not** teach tests that `High` is evidence. Downstream must **not** hardcode 5×5 in treatment or residual engines.

---

## 12. Definition of done

Risk scoring is explicit, organization-configurable, versioned, reproducible, and no longer an ad-hoc field on a risk record. 3×3 and 5×5 are fixtures. Invalid matrices fail closed. Collectors cannot emit ratings as compliance evidence. Dual-suite completed: spec (this file) → baseline GREEN → target RED → implement → target GREEN → baseline superseded → target still GREEN. Types, fixtures, and `score_risk` are in tree.
