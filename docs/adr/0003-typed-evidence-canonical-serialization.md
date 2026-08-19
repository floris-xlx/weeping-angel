# ADR 0003 — Typed evidence values and canonical serialization

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-18 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The “facts remain `BTreeMap<String, String>` / evaluator `parse_fact`” clauses of [ADR 0002](0002-iso-27001-assurance-vertical.md) §5 and older [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) Evidence text. **Does not** supercede envelope immutability, ledger ownership of observations, or INV-1…5. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md) |
| Spec | [`docs/specs/typed-evidence.md`](../specs/typed-evidence.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Planning baseline | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |
| Tests | `sdd_typed_evidence_target` GREEN (15/15). `sdd_typed_evidence_baseline` superseded (`#[ignore]`, 9 ignored). |

> Filename `0003-*` is shared with catalog / population / IAM siblings. This file is the **accepted typed-evidence** decision. Cite it by path.

## Context

ADR 0002 stored observation facts as strings so seal digests stayed a simple `serde_json` of a `BTreeMap<String, String>`. The control-test crate grew a **second** `EvidenceValue` and `parse_fact`, which coerced `"true"`, integer-looking text (including `"01"`), and `f64`-looking text. Collectors stringified everything. Nested structure and lists had no native form.

Canonical Assurance Catalog v1 needs typed facts (`branch_protected = true`, `required_reviewers = 2`, `privileged_roles = ["owner","admin"]`) without:

- weakening the immutable evidence envelope;
- putting framework/provider semantics on evidence;
- using IEEE-754 in digest-critical bytes;
- silently coercing ambiguous strings.

catalog infrastructure catalog infrastructure is a sibling decision. This decision is independent of catalog file format.

## Decision

This is what shipped.

### 1. One value type in `weeping-angel-evidence`

`EvidenceValue` lives in [`crates/weeping-angel-evidence/src/value.rs`](../../crates/weeping-angel-evidence/src/value.rs). Schema id: `evidence-value/v1` (`EVIDENCE_VALUE_SCHEMA`). Envelope schema remains `evidence/v1`.

```text
EvidenceValue =
  String | Bool | Integer(i64) | Decimal(DecimalText)
  | Timestamp(DateTime<Utc>) | DurationSeconds(u64)
  | StringList | Object(BTreeMap<String, EvidenceValue>)
```

`weeping-angel-control-test` **re-exports** this enum (`pub use weeping_angel_evidence::EvidenceValue`). It does not define a second enum. There is no stored `Null` / `Identifier` / `StringSet`. Absence is a missing key.

No `f64` / `f32` variant, probe, or compare path.

### 2. Observation API

`EvidenceObservation.facts` is `BTreeMap<String, EvidenceValue>`.

| Constructor / accessor | Behavior |
| --- | --- |
| `with_fact(key, impl Into<String>)` | Stores `String`. Never coerces `"true"` / `"01"` / `"1.0"`. |
| `with_value(key, EvidenceValue)` | Typed insert. |
| `fact(key) -> Option<&str>` | `Some` only when the stored variant is `String`. |
| `fact_value(key) -> Option<&EvidenceValue>` | Typed accessor used by the evaluator. |

Handoff constructors (facts, not conclusions):

```rust
obs.with_value("branch_protected", EvidenceValue::Bool(true))
   .with_value("required_reviewers", EvidenceValue::Integer(2))
   .with_value("retention_days", EvidenceValue::Integer(365))
   .with_value("privileged_roles",
        EvidenceValue::StringList(vec!["owner".into(), "admin".into()]));
```

Existing collectors may keep `with_fact`. They are not required to retype in this slice.

### 3. Hybrid canonical JSON (`evidence-value/v1`)

Used for envelope JSON **and** for `DigestBody { observation, provenance }` (still hashed by IR `canonical_digest`: SHA-256 hex of compact `serde_json`).

| Variant | JSON | Notes |
| --- | --- | --- |
| `String(s)` | JSON string | Historical `"enabled":"true"` stays a string. |
| `Bool(b)` | JSON boolean | `true` / `false` — not `"true"`. |
| `Integer(n)` | JSON number, no fraction, no exponent | `i64` only. |
| `StringList` | JSON array of strings | `[]` is valid. Order is identity. |
| `Object` | JSON object | `BTreeMap` key order. `{}` is valid. |
| `Decimal` | `{"$evidenceValue":"decimal","value":"<text>"}` | Validated decimal *text*. Lexical identity (`1.0` ≠ `1.00`). |
| `Timestamp` | `{"$evidenceValue":"timestamp","value":"<rfc3339>"}` | UTC, fixed `YYYY-MM-DDTHH:MM:SS.sssZ`. Sub-millisecond remainder rejected. |
| `DurationSeconds` | `{"$evidenceValue":"durationSeconds","value":<u64>}` | Number, not string. |

Tagged wrappers have exactly two keys, order fixed: `$evidenceValue` then `value`. `Object` facts **must not** contain the reserved key `$evidenceValue` (seal rejects it). Decode never turns `"01"`, `"1.0"`, or `"true"` into other variants. JSON `null` and mixed-type arrays fail closed.

Equivalent semantic evidence produces identical canonical bytes and identical digest regardless of map insertion order. New typed facts have **different** digests from their string lookalikes (`true` ≠ `"true"`). Historical string-only envelopes stay digest-stable.

### 4. Evaluator consumes stored types

`compare_eq` / `compare_numeric` / membership read `observation().fact_value`. `parse_fact` is deleted from the evaluate path.

| Operator | Allowed |
| --- | --- |
| `Eq` / `Neq` | Same variant (`typed_eq`). |
| `Gt` / `Gte` / `Lt` / `Lte` | Integer↔Integer, Decimal↔Decimal, Integer↔Decimal via exact decimal-string compare (never `f64`), Timestamp↔Timestamp, DurationSeconds↔DurationSeconds. |
| `Contains` / `NotContains` | Stored `StringList` contains expected `String`. |
| `In` | Stored value `typed_eq` any listed expected value. |

Incompatible pairs (`Bool` vs `String("true")`, `String("2")` vs `Integer(2)`) → `Ineffective` with deterministic rationale `type mismatch: expected …, got …`.

### 5. Seal / ledger / invariants unchanged in role

- Still `evidence/v1`. Still `canonical_digest(observation+provenance)`.
- Still reject compliance / `ControlTestResult` narratives.
- Credential-shaped keys rejected on **top-level facts and nested `Object` keys**.
- Ledger `append` / `get` round-trips typed values via the hybrid codec. Historical string-only payloads load as `String`.
- Envelope JSON still has no `frameworks` / `iso27001` / `gdpr` / `soc2` / `controlTestResult`.
- Ledger four-clock APIs and persistence integrity names do not change `DigestBody` ([ADR 0011](0011-temporal-lineage-evidence-soa-integrity.md)).
- Collection-run identity stays outside fact values.

## Consequences

- Downstream population / catalog / collector slices have one typed API and one codec.
- Decimal identity is lexical (`1.0` ≠ `1.00`). Callers who want numeric uniqueness pick one scale.
- `StringList` order is part of identity; set-like collectors sort+dedup before insert.
- ADR 0002 / contract text that said facts are only strings is superseded by this ADR.

## Non-decisions

Catalog domain content, provider collector semantics, remote ledger, and inferring compliance from typed facts remain out of scope.

## Related

- Spec SSOT: [`docs/specs/typed-evidence.md`](../specs/typed-evidence.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- ADR 0001: [`0001-inwardly-extensible-assurance-runtime.md`](0001-inwardly-extensible-assurance-runtime.md)
- ADR 0002: [`0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
- Catalog sibling: [`0003-canonical-assurance-catalog-v1.md`](0003-canonical-assurance-catalog-v1.md)
- Persistence integrity / four-clock ledger (does not change `DigestBody`): [ADR 0011](0011-temporal-lineage-evidence-soa-integrity.md)
