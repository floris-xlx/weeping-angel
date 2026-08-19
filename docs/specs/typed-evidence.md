# SDD: Typed Evidence and Canonical Serialization

| Field | Value |
| --- | --- |
| Status | **Implemented** — `sdd_typed_evidence_target` GREEN; baseline superseded |
| Program | Canonical Assurance Catalog v1 — typed evidence |
| Slice | Typed evidence values + canonical serialization + digest compatibility + evidence-level validation + control-test typed comparisons |
| Dual-suite | `sdd_typed_evidence_target` GREEN; `sdd_typed_evidence_baseline` superseded (`#[ignore]`) |
| ADR | Accepted [`docs/adr/0003-typed-evidence-canonical-serialization.md`](../adr/0003-typed-evidence-canonical-serialization.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) Evidence + `evidence-value/v1` |
| Consumes | Spine ADR 0001, ISO vertical ADR 0002 (string-bag clauses superseded). catalog infrastructure catalog infrastructure is a sibling — consumed, not redesigned. |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Planning baseline SHA | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |
| Characterization SHA | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` (HEAD when `sdd_typed_evidence_baseline` was written; still string-bag facts) |
| Baseline suite | `tests/contracts/typed_evidence.baseline.rs` (`sdd_typed_evidence_baseline`) — GREEN on this SHA |
| Evidence schema | remains `evidence/v1` (`EVIDENCE_SCHEMA`) |
| Value encoding | `evidence-value/v1` (this slice; nested inside observation facts) |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |

This document is the durable SSOT for typed evidence. The slice has landed: one `EvidenceValue` in `weeping-angel-evidence`, hybrid `evidence-value/v1` codec, control-test re-export + stored-type comparisons, `with_fact` string compatibility. catalog infrastructure catalog infrastructure is a sibling (`docs/specs/canonical-assurance-catalog-v1.md`); this slice does **not** invent a catalog format.

---

## 1. Problem / user-visible goal

Canonical evidence today is a string bag (`BTreeMap<String, String>`). Collectors stringify booleans, integers, lists, and structured observations (`"true"`, `"2"`, `"owner,admin"`). The control-test runtime then **reparses** those strings via `EvidenceValue::parse_fact`, which silently coerces `"true"` / `"false"` / integer-looking / `f64`-looking text. That is lossy and ambiguous:

- `"01"`, `"1.0"`, and `"true"` are not safe to coerce.
- Nested objects and string lists cannot be stored without ad-hoc encoding.
- `EvidenceValue` lives in `weeping-angel-control-test` and is **not** what the envelope stores, so there are two unrelated value systems.
- Ordinary `f64` probing (`trimmed.parse::<f64>()`) is on a path that influences evaluation, which must never enter digest-critical canonical bytes.

**User-visible goal:** one typed evidence value model, stored on the observation, serialized canonically, digested deterministically, compared by the control-test runtime without reparsing arbitrary strings. Downstream population / catalog / collector slices receive a single documented API:

```text
branch_protected   = true
required_reviewers = 2
retention_days     = 365
privileged_roles   = ["owner", "admin"]
```

This remains **facts**, never compliance conclusions. Typed `true` is not “ISO 27001 effective”.

---

## 2. Dependency on catalog infrastructure

| Surface | Status on `5fa3a23` | This slice |
| --- | --- | --- |
| `docs/specs/canonical-assurance-catalog-v1.md` | absent | do not create as a substitute catalog spec |
| `catalog/canonical/v1/` | absent | do not invent |
| Catalog loader / validator / digest | absent | do not implement |
| Domain catalog content (IAM, SDLC, …) | later slices | out of scope |
| Evidence envelopes, ledger, `TestExpr` | present (ADR 0001/0002) | **this slice consumes and extends** |

If catalog infrastructure lands first: rebase onto it; keep evidence-value types independent of catalog documents. Catalog entries may *name* fields and expected types later; this slice does not add schema-driven coercion.

---

## 3. Current behavior (baseline on planning SHA)

Characterized against `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b`. The baseline suite **must stay GREEN on this HEAD** until the target suite is GREEN and the baseline is explicitly superseded.

### 3.1 Observation facts are strings

[`crates/weeping-angel-evidence/src/lib.rs`](../../crates/weeping-angel-evidence/src/lib.rs):

```text
EvidenceObservation {
  evidence_type: EvidenceType,
  facts: BTreeMap<String, String>,
  narrative: String,
}
```

- `with_fact(self, key: impl Into<String>, value: impl Into<String>)` inserts a string.
- `fact(&self, key) -> Option<&str>` returns the raw string.
- `facts() -> &BTreeMap<String, String>`.
- There is **no** stored `Bool` / `Integer` / `Decimal` / `Timestamp` / `DurationSeconds` / `StringList` / `Object`.

Every production caller stringifies:

| Caller | Example |
| --- | --- |
| Scanner bridge | `with_fact("rule_id", …)`, `with_fact("canonical_type", ty)` |
| GitHub normalize | `("exists", "true")`, `("archived", "true"\|"false")` |
| GitHub protection | `("enabled", "true")`, `("count", reviews.to_string())` |
| Local collector | `with_fact("present", "true"\|"false")` |
| Manual evidence | `with_fact("attested_by", …)`, `with_fact("reason", …)` |
| SDD fixtures | `with_fact("enabled", "true")` |

### 3.2 Seal, digest, immutability

`EvidenceEnvelope::seal` rejects compliance-shaped narratives (`looks_like_compliance_claim`) and credential-shaped **fact keys** (`authorization`, `token`, `cookie`, `password`, `api_key`, `apikey`, `secret`, `access_token`, `refresh_token`, `private_key`; hyphen/underscore folded). It does **not** walk nested values (there are none).

Digest:

```text
DigestBody { observation, provenance }
digest = SHA-256 hex(serde_json::to_vec(DigestBody))   // weeping_angel_assurance_ir::canonical_digest
evidence_id = "ev:sha256:{digest}"
content_digest = digest
schema_version = "evidence/v1"
```

`canonical_digest` is **not** a typed evidence codec. It is compact `serde_json` bytes (struct field order + `BTreeMap` key order). Equivalent string facts inserted in any order already share a digest because facts are a `BTreeMap`. Map insertion order is therefore already irrelevant **for string bags**. There is no documented canonical form for decimals, timestamps-as-values, empty lists/objects, or nested objects.

Envelope identity fields (ISO vertical) sit **outside** `DigestBody`: `evidenceId`, `artifactRef`, `collectionRunId`, `sensitivity`, `scope`, `supersedes`. Optional temporal fields (`observedAt`, `validFrom`, `validUntil`, `sourceRevision`) are also outside the digest (serde-default / omitted). Collection-run identity is not a fact. Validity / revocation is a sibling `evidence-validity/v1` event, not a reseal ([`temporal-assurance.md`](temporal-assurance.md)).

Envelope JSON has no `frameworks`, `iso27001`, `gdpr`, `soc2`, `controlTestResult`, or provider-specific compliance columns.

### 3.3 Ledger

[`crates/weeping-angel-evidence/src/ledger.rs`](../../crates/weeping-angel-evidence/src/ledger.rs): SQLite (file or memory). `append` stores `serde_json::to_string(&envelope)` in `payload`; `get` / query paths `from_str` it back. Idempotent by digest (`INSERT OR IGNORE`). Validity events live in `evidence_validity_events` (append-only; not envelope payload). No `set_compliant` / `set_control_status`. Round-trips **string** facts. There is no remote ledger.

### 3.4 Control-test `EvidenceValue` is a second system

[`crates/weeping-angel-control-test/src/expr.rs`](../../crates/weeping-angel-control-test/src/expr.rs):

```text
enum EvidenceValue {
  Null, Boolean, Integer, Decimal(String), String,
  Timestamp(String), Duration(String), StringSet(Vec<String>), Identifier
}
```

`parse_fact(raw)`:

1. trim;
2. case-insensitive `"true"` / `"false"` → `Boolean`;
3. `i64` parse → `Integer` (so `"01"` becomes `1`);
4. `f64` probe → `Decimal(trimmed)` (so `"1.0"` becomes decimal text, after losing the integer path);
5. else `String` (original, not trimmed).

[`compare_eq` / `compare_numeric`](../../crates/weeping-angel-control-test/src/lib.rs) call `observation().fact(field)` then `parse_fact`. `as_integer` also parses leftover strings. `Contains` / `NotContains` / `In` exist on `TestExpr` but evaluate as **unsupported** (`NotTested`). Type mismatches currently surface as `Ineffective` with a `"type mismatch: …"` rationale (numeric path only). Equality does **not** fail closed on Boolean-vs-String once `parse_fact` has coerced the stored string.

### 3.5 What current tests lock

- `sdd_assurance_runtime_target` COL-003: `with_fact` + credential **keys** rejected at seal.
- `sdd_iso27001_assurance_target` EVD-*: envelope identity, ledger needles, secret keys, no framework claims; CTL-009 needles `enum EvidenceValue`, `Integer`, `type mismatch`.
- `sdd_iso27001_assurance.baseline` (ignored) historically asserted `facts: BTreeMap<String, String>` and `fact("enabled") == Some("true")`.
- Bridge tests: `obs.fact("iso27001").is_none()`.
- No workspace test asserts typed storage, object/list/timestamp/duration natives, or insertion-order digest for **typed** nested maps.

### 3.6 catalog infrastructure / catalog

Not on this SHA. Baseline characterization must not require `catalog/canonical/v1`.

---

## 4. Desired behavior

### 4.1 One `EvidenceValue` — evidence crate is source of truth

Move (or introduce) a single type in `weeping-angel-evidence`. `weeping-angel-control-test` **re-exports and consumes** it; it must not define a second enum.

Exact names may follow crate conventions. Semantically required variants:

```rust
pub enum EvidenceValue {
    String(String),
    Bool(bool),
    Integer(i64),
    Decimal(DecimalText),          // newtype over a validated decimal *string*
    Timestamp(DateTime<Utc>),
    DurationSeconds(u64),
    StringList(Vec<String>),
    Object(BTreeMap<String, EvidenceValue>),
}
```

| Rule | Meaning |
| --- | --- |
| No `f64` / `f32` | Not a variant. Not used to probe, normalize, or hash digest-critical bytes. |
| No stored `Null` | Absence is a missing key. Do not persist JSON `null` facts. |
| No `Identifier` | Persist as `String`. |
| No `StringSet` | Persist as `StringList`. Collectors that want set identity **sort and dedup before insert**. Digest treats list order as significant. |
| `Eq` + `Hash` + `Clone` | Required. Decimal equality is lexical on the stored canonical decimal text (not numeric coerce). Timestamp equality is the UTC instant. |
| `PartialOrd` where defined | Same-variant only. Cross-variant order is a type error, not a silent cast. |

Fold the current control-test variants into this model when adapting `TestExpr` literals.

### 4.2 Observation API

`facts` becomes `BTreeMap<String, EvidenceValue>`.

Keep **string `with_fact` as explicit compatibility** so existing collectors and GREEN SDD targets that pass `&str` continue to compile and store `EvidenceValue::String`:

```text
with_fact(self, key, value: impl Into<String>) -> Self
    // stores EvidenceValue::String(value). Never coerces "true"/"01"/"1.0".

with_value(self, key, value: EvidenceValue) -> Self
    // typed insert (name may be with_typed_fact).

fact_value(&self, key) -> Option<&EvidenceValue>

facts(&self) -> &BTreeMap<String, EvidenceValue>
```

`fact(&self, key) -> Option<&str>` may remain as a **string-only** accessor (`Some` only when the stored variant is `String`). It must not stringify bools/integers. Call sites that need types use `fact_value`.

Handoff constructors (normative examples, not domain semantics):

```rust
obs.with_value("branch_protected", EvidenceValue::Bool(true))
   .with_value("required_reviewers", EvidenceValue::Integer(2))
   .with_value("retention_days", EvidenceValue::Integer(365))
   .with_value("privileged_roles",
        EvidenceValue::StringList(vec!["owner".into(), "admin".into()]));
```

### 4.3 Canonical encoding `evidence-value/v1`

Used for envelope JSON **and** for `DigestBody` (seal still calls `canonical_digest` on observation + provenance). Encoding must be injective for distinct semantic values and independent of `BTreeMap` insertion order.

#### 4.3.1 Hybrid JSON (string-compatible)

| Variant | JSON | Notes |
| --- | --- | --- |
| `String(s)` | JSON string | Historical `"enabled":"true"` stays a string. |
| `Bool(b)` | JSON boolean | `true` / `false` — **not** `"true"`. |
| `Integer(n)` | JSON number, no fraction, no exponent | `i64` range only. |
| `StringList` | JSON array of strings | `[]` is valid. Mixed-type arrays are invalid. |
| `Object` | JSON object | Keys sorted (`BTreeMap`). `{}` is valid. |
| `Decimal` | `{"$evidenceValue":"decimal","value":"<text>"}` | See §4.3.3. |
| `Timestamp` | `{"$evidenceValue":"timestamp","value":"<rfc3339>"}` | See §4.3.4. |
| `DurationSeconds` | `{"$evidenceValue":"durationSeconds","value":<u64>}` | Number, not string. |

Object key order in tagged wrappers is **fixed**: `$evidenceValue` then `value`. Compact `serde_json` (no extra whitespace).

Reserved: an `Object` fact **must not** contain the key `$evidenceValue`. Seal/validation rejects it. That keeps tagged scalars distinguishable from nested objects.

#### 4.3.2 Decode (fail closed, no silent coerce)

Apply **in this order**:

1. JSON string → `String` (do **not** interpret `"true"`, `"01"`, `"1.0"`).
2. JSON bool → `Bool`.
3. JSON number:
   - integer within `i64` → `Integer`;
   - otherwise **error** (fraction, exponent, or out of range). Never decode via `f64`.
4. JSON array → every element a string → `StringList`; else error.
5. JSON object with `$evidenceValue` → tagged decode; unknown tag is error.
6. JSON object without `$evidenceValue` → `Object` (recurse).
7. JSON `null` → error.

Historical fixtures whose `facts` are entirely JSON strings therefore load as `String` values with **byte-identical** fact JSON, so existing sealed string envelopes keep the same `canonical_digest` after a load/seal of the same observation+provenance.

New typed facts (`true`, `2`, lists, tagged decimal/timestamp/duration, objects) produce **different** digests from their string lookalikes. That is required.

#### 4.3.3 Decimal text

`DecimalText` is a string matching:

```text
-? (0 | [1-9][0-9]*) ( '.' [0-9]+ )?
```

Forbidden: empty, `+`, exponent, `NaN`, `Inf`, leading zeros (`01`, `00.1` — use `0.1`), trailing dot (`1.`), lone `-`.

Canonical form **is the validated text as stored**. Do not strip trailing fractional zeros (`1.0` ≠ `1.00` ≠ `Integer(1)` ≠ `String("1.0")`). Do not introduce a binary float.

#### 4.3.4 Timestamp

Always UTC. Canonical lexical form:

```text
YYYY-MM-DDTHH:MM:SS.sssZ
```

Fixed millisecond precision (zero-padded). Offset timestamps normalize to UTC before encoding. Sub-millisecond remainder is an error or must be truncated **only** if the constructor documents truncation; prefer reject-if-nonzero-nanos-beyond-millis so two constructors cannot disagree. Digest uses the canonical string inside the tagged wrapper.

`EvidenceProvenance.collected_at` encoding is **unchanged** (existing chrono serde). This slice does not restyle envelope provenance timestamps.

#### 4.3.5 Other canonical cases (must have tests)

| Case | Rule |
| --- | --- |
| Object key order | `BTreeMap`; inserting `b` then `a` equals `a` then `b` (same digest). |
| Nested objects | Recursive encoding; nested key order also sorted. |
| Empty list / object | `[]` / `{}`. |
| Integer boundaries | `i64::MIN` and `i64::MAX` round-trip; overflow rejected. |
| String `"01"` / `"1.0"` / `"true"` | Remain `String`. Distinct from `Integer(1)`, `Decimal("1.0")`, `Bool(true)`. |
| `StringList` order | Significant for identity/digest. `[a,b]` ≠ `[b,a]`. |

### 4.4 Digest compatibility

- Seal continues to hash `DigestBody { observation, provenance }` via `canonical_digest` (SHA-256 hex of compact JSON).
- Determinism: equivalent **semantic** typed evidence ⇒ identical canonical bytes ⇒ identical digest, regardless of map insertion order.
- Historical **string** observations: same keys/values/provenance ⇒ same digest as on this planning SHA (because `String` still encodes as a JSON string).
- Do not add framework/provider fields to `DigestBody` or the envelope.
- `collection_run_id` / `supersedes` / artifact refs / optional validity clocks remain outside fact values. Changing only `collection_run_id` after seal (builder helper) still must not rewrite `digest` / `content_digest` unless a later ADR says otherwise — current code assigns run id inside `seal` from provenance; keep that. Validity windows are events, not digest fields ([ADR 0003 temporal assurance](../adr/0003-temporal-assurance.md)).

### 4.5 Evidence-level validation (invariants kept)

| Invariant | Still true |
| --- | --- |
| Immutable envelope | Mutation is a new envelope. |
| Facts ≠ conclusions | Seal still rejects compliance / `ControlTestResult` narratives. Collectors cannot emit result semantics. |
| Credentials | Reject credential-shaped keys on **top-level facts and nested `Object` keys**. Values are never a reason to persist a secret key. `redact` stays for diagnostics. |
| Provenance external | No `collector_id` / run id / framework id stuffed into fact values by this API. |
| No framework columns | Envelope JSON still has no `frameworks` / `iso27001` / `gdpr` / `soc2` / `controlTestResult`. |

### 4.6 Ledger

Same SQLite schema and APIs. `append` / `get` must round-trip typed facts (bool, integer, list, object, tagged decimal/timestamp/duration) via the hybrid codec. Historical rows whose payload has string-only facts must `get` as `String` values. Do not turn the ledger into a remote service.

### 4.7 Control-test integration

Evaluator **reads stored `EvidenceValue`**. It must not call `parse_fact` (delete from the evaluate path; do not keep a silent coerce helper on the hot path).

Minimum comparisons (fail closed + deterministic rationale containing `type mismatch` when types cannot be compared):

| Operator | Allowed |
| --- | --- |
| `Eq` / `Neq` | Same variant: `Bool`, `Integer`, `String`, `Decimal` (lexical), `Timestamp`, `DurationSeconds`. `StringList` equality is ordered elementwise. `Object` equality is structural. |
| `Gt` / `Gte` / `Lt` / `Lte` | `Integer`↔`Integer`; `Decimal`↔`Decimal`; `Integer`↔`Decimal` via exact decimal compare **without** `f64` (scale-align decimal strings); `Timestamp`↔`Timestamp`; `DurationSeconds`↔`DurationSeconds`. |
| `Contains` / `NotContains` | Stored `StringList` contains expected `String` (exact). |
| `In` | Stored value `Eq` any listed expected value (typed). |

Incompatible pairs (e.g. `Bool` vs `String("true")`, `String("2")` vs `Integer(2)`, `StringList` vs `Integer`) → fail closed. Use `Ineffective` with a stable `type mismatch: expected …, got …` rationale (matches CTL-009 needles), **not** `Effective` and not a coerced success.

`Contains` / `In` become implemented (today they are `NotTested`). Existing ISO SDD tests that only grep source needles must remain GREEN.

Literal `TestExpr` values use the same `EvidenceValue`. Drop or alias obsolete variants (`Null` → not a stored fact; `Identifier` → `String`; `StringSet` → `StringList`; stringly `Timestamp`/`Duration` → typed).

### 4.8 Collectors / fixtures (this slice)

- Do **not** rewrite GitHub/AWS/local production semantics.
- Keep `with_fact(..., "true")` working (stores `String`).
- Tests/fixtures **may** start using `with_value` to prove the typed path.
- Population runtime and domain catalogs are later slices.

### 4.9 Public contract / docs (implementation phase)

Landed: [`docs/specs/assurance-runtime.md`](assurance-runtime.md) Evidence documents `BTreeMap<String, EvidenceValue>` and `evidence-value/v1`. ADR 0002 §5 points at [ADR 0003](../adr/0003-typed-evidence-canonical-serialization.md).

---

## 5. Dual-suite protocol (HARD SDD)

`tests/contracts` is **not** auto-discovered. Register in root [`Cargo.toml`](../../Cargo.toml) (same pattern as `assurance_runtime` / `iso27001_assurance`):

```toml
[[test]]
name = "sdd_typed_evidence_baseline"
path = "tests/contracts/typed_evidence.baseline.rs"

[[test]]
name = "sdd_typed_evidence_target"
path = "tests/contracts/typed_evidence.target.rs"
```

Protocol:

1. **Spec first** (this file). No product feature code.
2. **Baseline GREEN on current HEAD** — characterize §3.
3. **Target RED on current HEAD** for the right reasons (§6), not compile errors from a half-written API.
4. Implement product code.
5. Target GREEN. Workspace verify GREEN.
6. **Supersede baseline** (`#[ignore = "superseded by sdd_typed_evidence_target"]`) so string-only behavior is not CI-required. Prefer ignore/skip over leaving old behavior as a required green bar.
7. Target still GREEN. Keep `with_fact` string compat so existing collector/SDD targets stay green.

Fail closed if baseline cannot go green, target cannot go red for the right reason, or target never greens.

---

## 6. Acceptance criteria (testable)

### 6.1 Baseline suite (GREEN now; later ignored)

Must encode **current** HEAD:

1. `EvidenceObservation` facts are `BTreeMap<String, String>`; `with_fact` / `fact` are strings (`fact("enabled") == Some("true")`).
2. Seal digest is `canonical_digest` of serde JSON `DigestBody { observation, provenance }`; same string facts + provenance ⇒ same digest; `BTreeMap` already ignores insertion order for string keys.
3. `EvidenceValue` lives in `weeping-angel-control-test`; `parse_fact` coerces `true`/`false` / `i64` / `f64`-looking strings.
4. `compare_eq` / `compare_numeric` read `observation().fact()` then `parse_fact`.
5. Ledger JSON payload round-trips string facts (`append`/`get`).
6. Credential-key reject is on fact **keys** (`token`, …).
7. Sealed envelope JSON has no framework/provider compliance fields (`iso27001`, `gdpr`, `soc2`, `frameworks`, `controlTestResult`).

### 6.2 Target suite (RED on current HEAD; GREEN after implement)

Must fail today because:

- typed facts are not stored;
- no object / list / timestamp / duration natives on the observation;
- silent string coercion (`parse_fact`) still exists on the evaluate path;
- evaluator does not consume stored types;
- there is no documented/implemented canonical typed serialization (`$evidenceValue` tags, decimal/timestamp rules).

After implement, the same tests pass and cover:

1. Deterministic digest under map **insertion-order** changes for typed objects and nested objects.
2. Nested object determinism (same tree, different insert order → same digest and same canonical bytes).
3. Typed `Bool` / `Integer` / `String` comparison without reparsing.
4. Invalid comparison types fail closed with deterministic `type mismatch` rationale (`"true"` string ≠ `Bool(true)`; `"2"` ≠ `Integer(2)`; `"01"` stays string).
5. Credential rejection still holds with typed values, including nested object keys.
6. Serialization/deserialization round trips for every variant, including empty list/object, `i64` bounds, decimal text, timestamp `Z` millis, duration seconds.
7. Historical string fixture compatibility: string-only fact JSON loads as `String`; ambiguous strings are not coerced; existing `with_fact` callers remain valid.
8. Evidence ledger `append`/`get` round trips typed values.
9. No framework/provider fields added to evidence envelopes.
10. Control-test no longer depends on lossy `parse_fact` for core comparisons; `Contains`/`In` work for `StringList` membership.
11. Handoff examples (`branch_protected=true`, `required_reviewers=2`, `retention_days=365`, `privileged_roles=["owner","admin"]`) construct, seal, digest, evaluate, and ledger-round-trip.
12. Workspace remains green under the verify commands in the header.

---

## 7. Out of scope

- Canonical catalog domain content (catalog infrastructure+; IAM/SDLC/vuln/infra/governance catalogs).
- GitHub / AWS / Cloudflare / local collector **semantics** (normalization rules, permissions, hosted APIs) beyond keeping string `with_fact` and allowing fixtures to use the new API.
- Population runtime (population runtime).
- Redesigning the evidence ledger into a remote service.
- Inferring compliance / effectiveness from typed facts (a `Bool(true)` named `branch_protected` is still only a fact).
- Changing `SemanticFinding` / scanner engines / Codex security contract.
- Adding `iso_27001` / `gdpr` / `soc2` (or siblings) to findings or envelopes.
- Replacing `canonical_digest` globally for IR documents (IR keeps existing serde JSON digest).
- Schema-driven coercion of strings (no “schema says this field is bool, so parse it”).
- Ordinary floating-point in digest-critical bytes.

---

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Dual value systems (`control-test` enum vs stored strings) linger | One type in evidence crate; control-test re-exports only. Target test forbids a second `enum EvidenceValue` in control-test `expr.rs`. |
| Digest break for historical string envelopes | Hybrid codec: JSON strings remain `String`. Prove digest equality on a sealed string fixture from this SHA. |
| Untagged JSON cannot distinguish decimal/timestamp/duration from string/int | Tagged wrappers with reserved `$evidenceValue`; reject that key on `Object`. |
| `parse_fact` leftover used “just in case” | Delete from evaluate path; target test greps that `evaluate` / `compare_*` do not call it. |
| `f64` sneaks back via serde or compare | Ban `f64`/`f32` in evidence + control-test comparison/canonical modules; decimal compare is string scale-align. |
| Existing collectors/SDD go red | Keep `with_fact(key, impl Into<String>)`. Do not require collectors to emit typed values in this slice. |
| Nested credential keys bypass reject | Walk `Object` keys recursively. |
| catalog infrastructure rebase conflict | Consume catalog contracts if present; do not fork them. Evidence-value API stays in the evidence crate. |
| `TestExpr` serde change | No committed TestExpr fixtures on this SHA; adapt compile-time AST only. |
| List vs set identity | Document list-order-significant; set-like collectors sort+dedup before insert. |
| Timestamp precision drift | Fixed `.sssZ` form; constructor rejects or documents non-millis. |
| Integer/Decimal mixed compare implemented with floats | Exact decimal algorithm only; tested with values that are not binary-exact in IEEE-754. |

---

## 9. Implementation notes (for the implement phase, not this commit)

Owned crates: `weeping-angel-evidence` (model, codec, validation, ledger round-trip), `weeping-angel-control-test` (consume stored types). Touch collectors/bridge only if signatures require it (`with_fact` should keep compiling).

Suggested modules (names flexible): `crates/weeping-angel-evidence/src/value.rs` (enum + constructors + decode), `canonical.rs` (encode bytes used by tests), keep `seal` / `reject_credentials` in `lib.rs`.

Do not expand `DigestBody`. Do not add provider fields.

---

## 10. Handoff contract (downstream slices)

One typed API. Documented canonical representation is §4.3. Digest rule is §4.4.

```text
# Collectors / population should prefer:
branch_protected    Bool(true)
required_reviewers  Integer(2)
retention_days      Integer(365)
privileged_roles    StringList(["owner", "admin"])

# Still legal (compat, not coerced by the evaluator):
with_fact("branch_protected", "true")  → String("true")
```

Downstream must **not** teach tests about GitHub/AWS. They emit canonical field names + typed values. Control tests compare those types. Framework packs continue to map to controls, not onto evidence JSON.

---

## 11. Definition of done

All evidence paths use or cleanly adapt to one typed representation; the control-test runtime no longer depends on lossy string parsing for core comparisons; digests remain deterministic; old fixtures have an explicit migration path (`String` decode + `with_fact` compat); baseline is superseded; target and the workspace verify command stay GREEN.

---

## 12. Landed record

| Surface | Location |
| --- | --- |
| Value model + hybrid serde | `crates/weeping-angel-evidence/src/value.rs` |
| Observation API / seal / nested credential walk | `crates/weeping-angel-evidence/src/lib.rs` (`with_fact`, `with_value`, `fact`, `fact_value`) |
| Control-test re-export | `crates/weeping-angel-control-test/src/expr.rs` (`pub use weeping_angel_evidence::EvidenceValue`) |
| Comparisons | `EvidenceValue::{typed_eq,cmp_numeric,list_contains}`; evaluator reads `fact_value` |
| Target suite | `tests/contracts/typed_evidence.target.rs` (`sdd_typed_evidence_target`) GREEN 15/15 |
| Baseline suite | `tests/contracts/typed_evidence.baseline.rs` superseded (`#[ignore]`) |
| ADR | Accepted [`docs/adr/0003-typed-evidence-canonical-serialization.md`](../adr/0003-typed-evidence-canonical-serialization.md) |

Stable digest rule remains §4.4. Stable encoding remains §4.3. Downstream slices consume this API; they do not fork a second value enum.

### Remaining increment (Prompt 3 — do not fork this spec)

Typed-value / seal / `DigestBody` law is unchanged. Persistence integrity (`Corrupt` / `IncompatibleSchema` via `PersistenceIntegrity` → `LedgerError::Path`, immutable completed collection-run persist, transactional envelope+validity append, distinct `latest` / `current` / `valid_at` / `as_of`) is implemented in [`temporal-lineage-evidence-soa.md`](temporal-lineage-evidence-soa.md) / [ADR 0011](../adr/0011-temporal-lineage-evidence-soa-integrity.md). Do not put conclusions on envelopes.
