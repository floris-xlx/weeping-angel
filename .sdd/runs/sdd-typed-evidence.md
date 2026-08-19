# SDD run: Typed Evidence and Canonical Serialization

| Field | Value |
| --- | --- |
| Run id | `sdd-4868c52402a1` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `4868c52402a15e62` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 02: one `EvidenceValue` in `weeping-angel-evidence`, hybrid `evidence-value/v1` JSON, deterministic digests, string `with_fact` compat, fail-closed typed comparisons |
| Spec | [`docs/sdd/typed-evidence.md`](typed-evidence.md) |
| ADR | [`docs/adr/0003-typed-evidence-canonical-serialization.md`](../adr/0003-typed-evidence-canonical-serialization.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) Evidence + `evidence-value/v1` |
| Telemetry | [`sdd-typed-evidence-telemetry.json`](sdd-typed-evidence-telemetry.json) |
| Dual-suite | `tests/sdd/typed_evidence.baseline.rs` (retired / `#[ignore]`) · `tests/sdd/typed_evidence.target.rs` (active) |
| Characterization SHA | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** Typed Evidence and Canonical Serialization
- **Problem:** Canonical evidence is a string bag; collectors stringify facts and the control-test runtime silently reparses them (`true` / `01` / `1.0` / `f64`), so typed comparison, nested structure, and digest-stable identity cannot be expressed without lossy coercion.
- **Current behavior (HEAD `5fa3a23`):** `EvidenceObservation.facts` is `BTreeMap<String, String>`; `with_fact` / `fact` are strings. Seal digest is IR `canonical_digest(serde_json` of observation + provenance). `EvidenceValue` lives in control-test with `parse_fact` coercing `true`/`false`/i64/f64-looking strings; `compare_eq` / `compare_numeric` read `observation().fact()` then `parse_fact`. Ledger JSON round-trips string facts. Credential reject is on fact keys. Envelopes have no framework/provider fields. Prompt 01 catalog is absent.
- **Desired behavior:** One `EvidenceValue` in `weeping-angel-evidence` (`String` / `Bool` / `Integer` / `Decimal` text / `Timestamp` / `DurationSeconds` / `StringList` / `Object`) consumed by control-test. Hybrid `evidence-value/v1` JSON keeps historical string facts and `with_fact` compat; tagged wrappers for decimal/timestamp/duration; no `f64`; no silent coerce of `01` / `1.0` / `true`. Digests stay deterministic under map insertion order. Evaluator compares stored types and fail-closes type mismatches. Ledger append/get round-trips typed values.
- **ADR:** needed — accepted at [`docs/adr/0003-typed-evidence-canonical-serialization.md`](../adr/0003-typed-evidence-canonical-serialization.md)

### Acceptance criteria (this slice)

1. Baseline suite GREEN on current HEAD: string facts, `with_fact`/`fact`, seal digest via `canonical_digest(serde_json)`, `parse_fact` coerce, ledger string round-trip, credential-key reject, no framework fields.
2. Target suite RED on current HEAD for typed storage, natives, no silent coerce, evaluator consuming stored types, and documented canonical typed serialization.
3. Deterministic digest under map insertion-order changes including nested objects.
4. Typed bool/integer/string comparisons without `parse_fact`.
5. Incompatible type comparisons fail closed with deterministic type-mismatch rationale (no coerce of `01`/`1.0`/`true`).
6. Credential rejection still holds for typed and nested object keys.
7. Serde/canonical round trips for all variants including empty list/object, i64 bounds, decimal text, timestamp Z millis, duration seconds.
8. Historical string fixtures load as `String` and keep digest-compatible JSON; `with_fact` remains.
9. Evidence ledger append/get round-trips typed values.
10. No framework/provider fields added to evidence envelopes.
11. Handoff examples `branch_protected=true`, `required_reviewers=2`, `retention_days=365`, `privileged_roles=[owner,admin]` construct, seal, digest, evaluate, and ledger-round-trip.
12. After implement: target GREEN, baseline superseded/ignored, workspace verify stays green.

### Out of scope

- Canonical catalog domain content and Prompt 01 catalog format
- GitHub/AWS/Cloudflare collector semantics beyond `with_fact` compat and fixtures
- Population runtime
- Remote evidence ledger
- Inferring compliance from typed facts
- Changing `SemanticFinding` / scanner / Codex contract
- Adding `iso_27001` / `gdpr` / `soc2` to findings or envelopes
- Replacing IR `canonical_digest` globally
- Schema-driven string coercion
- `f64`/`f32` in digest-critical bytes

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Two `EvidenceValue` enums lingering across crates | Single enum in `weeping-angel-evidence`; control-test re-exports and compares stored types. |
| Digest break for historical string envelopes if `String` encoding changes | Hybrid codec: untagged strings stay strings; historical fixtures load as `String` and remain digest-compatible. |
| Untagged JSON cannot distinguish decimal/timestamp/duration from string/int | Tagged wrappers (`$evidenceValue` / `evidence-value/v1`) for those variants. |
| `parse_fact` left on the evaluate path | Target forbids leftover `parse_fact` / `trimmed.parse::<f64>()`; evaluator consumes stored types. |
| `f64` used for decimal compare or decode | Decimal is text; no `f64`/`f32` in digest-critical bytes. |
| Existing collector/SDD targets break if `with_fact` is removed | `with_fact` remains string-compat. |
| Nested credential keys bypass reject | Credential walk covers typed and nested object keys. |
| Prompt 01 rebase conflict if catalog lands later | Catalog format out of scope; evidence-value types stay independent of catalog documents. |
| StringList order vs set identity confusion | Lists are ordered `Vec<String>`; not set identity. |
| Timestamp precision drift across constructors | Timestamp is Z millis; duration is whole seconds. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/typed-evidence.md`](typed-evidence.md) |
| Baseline | PASS on old | `cargo test --test sdd_typed_evidence_baseline --features demo` → exit 0. Characterization SHA `5fa3a23`. **12 passed; 0 failed**. Excerpt: `observation_facts_are_string_bags` / `evidence_crate_has_no_stored_typed_value_model` / `compare_eq_reads_string_facts_then_parse_fact` / `ledger_round_trips_string_facts` / `credential_reject_is_on_fact_keys_not_values` … `test result: ok. 12 passed`. Suite: `tests/sdd/typed_evidence.baseline.rs`. Workspace `cargo test --workspace --features demo` still failed on unrelated RED `sdd_population_runtime_target` (Prompt 03) at characterization time. |
| Target pre | FAIL on old | `cargo test --test sdd_typed_evidence_target --features demo -- --nocapture` → **FAILED. 2 passed; 13 failed**. Compatibility locks intended GREEN: `dual_suite_target_is_registered`, `historical_string_fixtures_remain_digest_compatible`. Failures: `one_evidence_value_lives_in_evidence_crate` missing `pub enum EvidenceValue` / `Bool` / `Integer` / `DurationSeconds` / `StringList` / `Object` / `with_value` / `fact_value` / `$evidenceValue` / `evidence-value/v1`; `evaluator_consumes_stored_types_not_parse_fact` leftover `parse_fact` / `trimmed.parse::<f64>()`; hybrid codec / type-mismatch tests `invalid type: boolean true, expected a string`. Suite: `tests/sdd/typed_evidence.target.rs`. Baseline and product crates not modified to force green. |
| Implement | target PASS | AFTER: `cargo test --test sdd_typed_evidence_target --features demo` → **ok. 15 passed; 0 failed**. One `EvidenceValue` (hybrid `evidence-value/v1`) in `weeping-angel-evidence`; control-test re-exports and compares stored types without `parse_fact`. Digests stay insertion-order deterministic; `with_fact` remains. Clippy: `cargo clippy -p weeping-angel-evidence -p weeping-angel-control-test --all-targets --all-features -- -D warnings` → Finished `dev` profile. |
| Baseline post | FAIL or retired | Skip-retired via `#[ignore = "superseded by sdd_typed_evidence_target"]` (`supersede_kind=skip`). Default: **ok. 0 passed; 0 failed; 9 ignored**. Forced `--ignored`: **FAILED. 6 passed; 3 failed** (`compare_numeric_uses_parse_fact_and_type_mismatch_on_non_integers`; `contains_and_in_are_unsupported_not_tested`; `evidence_crate_has_no_stored_typed_value_model`). Not additive: facts are now typed `EvidenceValue`, Contains/In are implemented, compare no longer uses `parse_fact`. |
| Supersede | target still PASS | After skip-supersede: `cargo test --workspace --features demo --test sdd_typed_evidence_target` → **15 passed; 0 failed; 0 ignored**. Workspace `cargo test --workspace --features demo` exit 0; target 15 passed; baseline 0 passed / 9 ignored. Target remains SSOT. |
| Docs/ADR | updated | [`docs/adr/0003-typed-evidence-canonical-serialization.md`](../adr/0003-typed-evidence-canonical-serialization.md), [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md), [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md), [`docs/sdd/typed-evidence.md`](typed-evidence.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md), [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md) |

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | `skip` |
| `baseline_retired` | `true` |
| `additive_baseline` | `false` |
| `baseline_not_green` | `true` |
| `target_still_green` | `true` |

`verify_ok` = `target_still_green` ∧ (`baseline_retired` ∧ `baseline_not_green` ∨ `additive_baseline`) = **true**.

---

## What landed

- One `EvidenceValue` in `weeping-angel-evidence`: `String` / `Bool` / `Integer` / `Decimal` (text) / `Timestamp` / `DurationSeconds` / `StringList` / `Object`.
- Hybrid `evidence-value/v1` codec: historical string facts stay strings; tagged wrappers for decimal/timestamp/duration; no `f64`.
- Control-test re-exports the evidence-crate type and compares **stored** types; `parse_fact` is off the evaluate path.
- Fail-closed type mismatches with deterministic rationale (no coerce of `01` / `1.0` / `true`).
- Seal digests remain deterministic under map insertion-order changes, including nested objects (still IR `canonical_digest` of serde JSON).
- `with_fact` remains string-compat; historical fixtures load as `String` and keep digest-compatible JSON.
- Credential rejection still holds for typed and nested object keys.
- Evidence ledger append/get round-trips typed values. Envelopes still have no framework/provider fields.
- Handoff examples (`branch_protected=true`, `required_reviewers=2`, `retention_days=365`, `privileged_roles=[owner,admin]`) construct, seal, digest, evaluate, and ledger-round-trip.

### Files changed (implement)

`crates/weeping-angel-evidence/src/value.rs`, `crates/weeping-angel-evidence/src/lib.rs`, `crates/weeping-angel-control-test/src/expr.rs`, `crates/weeping-angel-control-test/src/lib.rs`, `crates/weeping-angel-control-test/src/run.inc`, `tests/sdd/typed_evidence.baseline.rs`, `tests/sdd/iso27001_assurance.baseline.rs`, `docs/sdd/typed-evidence.md`, `docs/adr/0003-typed-evidence-canonical-serialization.md`, `docs/adr/0002-iso-27001-assurance-vertical.md`, `docs/contracts/assurance-runtime.md`, `docs/sdd/population-runtime.md`, `docs/sdd/iam-canonical-assurance-catalog.md`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-4868c52402a1` |
| `agents_ok` | 7 |
| `agents_fail` | 0 |
| `agents_total` | 7 |
| `tokens_used_sum` | 13 259 152 |
| `duration_ms_sum` | 7 465 341 (~124.4 min) |
| `budget.total` | 48 |
| `budget.spent` | 7 |
| `budget.remaining` | 41 |
| `event_count` | 28 |
| `max_iters` | 3 |
| `iters_used` | 0 |
| `dry_run` | false |
| `no_delta` | false |

### Gates (final snapshot)

| Gate | Value |
| --- | --- |
| `baseline_green` | true |
| `target_red` | true |
| `target_green` | true |
| `baseline_superseded` | true |
| `dry_run` | false |
| `no_delta` | false |

### Agents

| Phase | Label | Success | Duration (ms) | Tokens |
| --- | --- | --- | --- | --- |
| Scope | `sdd-scope` | ok | 779 296 | 304 104 |
| Spec | `sdd-spec` | ok | 964 929 | 511 146 |
| BaselineGreen | `sdd-baseline-green` | ok | 723 030 | 655 133 |
| TargetRed | `sdd-target-red` | ok | 818 459 | 1 000 622 |
| Implement | `sdd-implement` | ok | 3 509 445 | 9 688 551 |
| DocsAdr | `sdd-docs-adr` | ok | 542 268 | 904 384 |
| Supersede | `sdd-supersede` | ok | 127 914 | 195 212 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-typed-evidence-telemetry.json`](sdd-typed-evidence-telemetry.json).

---

## Remaining backlog (not this slice)

1. Canonical catalog domain content and Prompt 01 catalog format
2. GitHub / AWS / Cloudflare collector semantics beyond `with_fact` compat and fixtures
3. Population runtime
4. Remote evidence ledger
5. Inferring compliance from typed facts (forbidden)
6. Changing `SemanticFinding` / scanner / Codex contract (forbidden)
7. Adding `iso_27001` / `gdpr` / `soc2` to findings or envelopes (forbidden)
8. Replacing IR `canonical_digest` globally
9. Schema-driven string coercion (forbidden)
10. `f64`/`f32` in digest-critical bytes (forbidden)

---

## Summary

Prompt 02 typed evidence landed under dual-suite SDD: spec + accepted ADR 0003, baseline GREEN on SHA `5fa3a23` (12 passed), target RED (13 failed / 2 compatibility locks), then one `EvidenceValue` + hybrid `evidence-value/v1` until target GREEN 15/15. String-bag baseline skip-superseded (`#[ignore]`; forced `--ignored` 3 FAIL). Control-test compares stored types without `parse_fact`; digests stay insertion-order deterministic; `with_fact` and historical string fixtures remain. Envelopes still have no framework/provider fields. Catalog domain, population runtime, and remote ledger stay out of this slice.
