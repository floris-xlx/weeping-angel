# Grok 4.6 Prompt 01 — Canonical Catalog Infrastructure

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1

## Mission

Implement the versioned canonical assurance catalog infrastructure that all downstream catalog, collector, framework, and test work will consume. This prompt owns the catalog format, loader, validator, digest, stable-ID rules, and offline compilation contract. It does not own domain content beyond minimal fixtures.

The architecture law is:

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

The catalog must be framework-neutral and provider-neutral.

## First actions

1. Fetch latest `main` and record the exact baseline SHA in `docs/specs/canonical-assurance-catalog-v1.md`.
2. Run:
   - `cargo test --workspace --features demo`
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. Inspect `crates/weeping-angel-assurance-ir`, `weeping-angel-framework`, `weeping-angel-control-test`, `frameworks/`, and existing SDD/contract tests before changing types.

## Required implementation

Create a versioned catalog rooted at:

```text
catalog/canonical/v1/
  manifest.toml
  controls/
  evidence/
  tests/
```

Use schema identifier `weeping-angel/canonical-catalog/v1` unless an existing repository convention requires a narrowly compatible variant.

Implement a loader API conceptually equivalent to:

```rust
CanonicalCatalog::load(...)
CanonicalCatalog::validate(...)
CanonicalCatalog::digest(...)
```

Exact names may follow current crate conventions. Prefer a dedicated crate only if it improves dependency boundaries; otherwise extend the most appropriate existing crate without coupling framework packs to collectors.

The manifest must identify schema version, catalog version, files/sections, and deterministic content digest inputs. Loading and validation must require zero network I/O.

## Stable IDs

Enforce these namespaces:

```text
control.*
evidence.*
test.*
```

Reject provider-specific canonical IDs such as `control.github.*`, `evidence.aws.*`, `test.cloudflare.*`. Reject framework-specific canonical IDs such as `control.iso27001.*` or `test.soc2.*`.

Stable IDs are public API. Add validation preventing duplicates, malformed IDs, dangling references, and accidental renames where repository fixtures can detect them.

## Validation requirements

Reject at least:

- duplicate IDs;
- unknown control references;
- unknown evidence references;
- unknown test references;
- orphaned tests;
- malformed selectors/expressions;
- unsupported schema versions;
- provider/framework names in canonical IDs;
- nondeterministic catalog serialization/digests.

Add CLI support:

```bash
weeping-angel assurance catalog validate
weeping-angel assurance catalog stats
weeping-angel assurance catalog inspect <control-id>
```

If current CLI architecture requires the command surface to land in a separate bounded module, keep parser and execution separated.

## Architecture tests

Add SDD/contract tests covering at least:

- catalog loads offline;
- catalog digest is deterministic;
- duplicate IDs fail closed;
- dangling references fail closed;
- provider names cannot appear in canonical IDs;
- framework names cannot appear in canonical IDs;
- framework crate still has no collector/provider SDK dependency;
- collector crate still has no framework knowledge.

## Non-goals

Do not implement the full IAM, SDLC, vulnerability, infrastructure, or governance catalogs here. Do not redesign `AssessmentDefinition`, `Control`, `Requirement`, `Mapping`, `EvidenceRequirement`, or `PlannedControlTest` unless a compile blocker requires a small compatibility change. Do not add SOC 2/NIS2/DORA content. Do not put ISO content in the canonical catalog.

## Handoff contract

Downstream prompts must be able to rely on:

- stable schema/version;
- stable ID conventions;
- deterministic loader and digest;
- validator API;
- fixture examples showing how to declare controls/evidence/tests;
- documented extension points.

Update `docs/specs/canonical-assurance-catalog-v1.md` with the final API and file format.

## Definition of done

The catalog infrastructure loads, validates, hashes, inspects, and reports statistics for a minimal fixture; all workspace tests remain green; no provider/framework coupling is introduced; and downstream agents can add content without modifying the loader.