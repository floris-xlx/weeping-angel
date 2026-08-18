# Grok 4.6 Prompt 02 — Typed Evidence and Canonical Serialization

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1

## Mission

Upgrade canonical evidence from string-only fact bags into a deterministic typed value model suitable for serious assurance evaluation, without weakening the immutable evidence envelope or allowing provider/framework semantics into evidence.

This prompt owns typed evidence values, canonical serialization, digest compatibility, and evidence-level validation. It does not own population semantics, domain catalogs, or provider collectors beyond fixtures.

## First actions

Rebase on the latest catalog-infrastructure work if available. Inspect `crates/weeping-angel-evidence`, `crates/weeping-angel-assurance-ir`, the control-test expression runtime, ledger schema, bridge code, and every existing call to `EvidenceObservation::with_fact`.

Run the full workspace baseline before modification.

## Required model

Evolve the current `BTreeMap<String, String>` facts representation toward a typed model conceptually equivalent to:

```rust
pub enum EvidenceValue {
    String(String),
    Bool(bool),
    Integer(i64),
    Decimal(String),
    Timestamp(DateTime<Utc>),
    DurationSeconds(u64),
    StringList(Vec<String>),
    Object(BTreeMap<String, EvidenceValue>),
}
```

The exact enum may differ if repository conventions provide a cleaner solution. Do not use ordinary floating-point values in digest-critical canonical serialization.

Typed values must support equality, numeric comparisons, timestamps/freshness, arrays/sets where needed, and nested structured observations without requiring collectors to stringify everything.

## Canonicalization

Evidence identity must remain deterministic. Equivalent semantic evidence must produce identical canonical serialization and identical digest regardless of map insertion order.

Define and test explicit canonicalization behavior for:

- object key ordering;
- decimal representation;
- timestamp normalization;
- empty lists/objects;
- integer boundaries;
- nested values;
- backward-compatible string decoding if retained.

Do not silently coerce ambiguous strings such as `"01"`, `"1.0"`, or `"true"` into other types unless the schema explicitly requests it.

## Compatibility

Existing scanner/collector fixtures and serialized evidence may still use strings. Provide a migration/compatibility path that is explicit and deterministic. Avoid maintaining two unrelated evidence-value systems.

Keep these invariants intact:

- evidence is immutable;
- observations are facts, never compliance conclusions;
- credential-shaped fields remain rejected/redacted;
- collectors cannot emit `ControlTestResult` semantics;
- provenance and collection-run identity remain external to the fact value itself.

## Control-test integration

Adapt the expression runtime so typed comparisons consume the canonical value model directly rather than reparsing arbitrary strings.

Support at minimum:

- bool equality/inequality;
- integer/decimal comparisons;
- timestamp comparisons where useful;
- string equality;
- string-list membership where the existing DSL can express it.

Fail closed on incompatible type comparisons and produce deterministic diagnostic rationale.

## Tests

Add comprehensive tests for:

1. deterministic digest under map insertion-order changes;
2. nested object determinism;
3. typed bool/integer/string comparison;
4. invalid comparison type handling;
5. credential rejection with typed values;
6. serialization/deserialization round trips;
7. historical string fixture compatibility;
8. evidence ledger append/get round trips with typed values;
9. no framework/provider fields added to evidence.

## Non-goals

Do not implement catalog domain content. Do not implement GitHub/AWS/etc. semantics. Do not redesign the evidence ledger into a remote service. Do not infer compliance status from typed facts.

## Handoff contract

Downstream population, catalog, and collector prompts must receive one documented typed value API and examples for declaring evidence such as:

```text
branch_protected = true
required_reviewers = 2
retention_days = 365
privileged_roles = ["owner", "admin"]
```

Document the stable canonical representation and digest rules.

## Definition of done

All evidence paths use or cleanly adapt to one typed representation, the control-test runtime no longer depends on lossy string parsing for core comparisons, digests remain deterministic, old fixtures have an explicit migration path, and the workspace remains green under fmt/clippy/tests.