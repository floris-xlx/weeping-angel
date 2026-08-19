# ADR 0011 — Catalog parse ownership, semantic framework digest, and pin-pure readiness identity

<!-- weeping-angel-adr-meta
id = "0011"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0003-canonical-assurance-catalog-v1", "0003-iso27001-canonical-remap", "0004-documentation-architecture"]
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — catalog SSOT, fail-closed pack parse, semantic digest, pin-pure serialize/project, single readiness owner |
| Date | 2026-08-20 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in catalog ID grammar, ISO mapping tables, or the spine crate graph. **Amends operational practice** of [ADR 0003 catalog](0003-canonical-assurance-catalog-v1.md) (silent second catalog parser in the framework crate) and [ADR 0003 remap](0003-iso27001-canonical-remap.md) (pack digest as merged id lists; serialize-time live catalog reload; pack-local catalog index). Does **not** supercede ADR 0003’s “framework crate must not depend on the catalog crate”. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [ADR 0003 catalog](0003-canonical-assurance-catalog-v1.md), [ADR 0003 remap](0003-iso27001-canonical-remap.md), [ADR 0004](0004-documentation-architecture.md), [ADR 0010](0010-architecture-as-law.md) |
| Spec | [`docs/specs/catalog-framework-readiness-trust-boundary.md`](../specs/catalog-framework-readiness-trust-boundary.md) |
| Human law this does not replace | [`canonical-assurance-catalog-v1.md`](../specs/canonical-assurance-catalog-v1.md), [`iso-27001-canonical-remap.md`](../specs/iso-27001-canonical-remap.md), [`iso-27001-automated-assurance-mvp.md`](../specs/iso-27001-automated-assurance-mvp.md) |
| Tests | `sdd_canonical_assurance_catalog_target`, `sdd_iso27001_remap_target`, `sdd_iso27001_assurance_target`; `cargo test -p weeping-angel-canonical-catalog`; `-p weeping-angel-framework`; `-p weeping-angel-assurance readiness` |

> Filename **`0011-*`**. Cite **this file by path**. Concurrent cleanup ADRs share the `0011` prefix; do **not** add a `0003-catalog-framework-readiness.md` sibling.

## Context

Cleanup Prompt 2 (architectural-cleanup phases 2, 3, 7, 21) closed P0 ambiguity:

1. `CanonicalCatalog::load` was fail-closed, but `weeping-angel-framework::pack::discover_catalog_index` re-parsed `catalog/canonical/v1` with `Option` + `continue` and injected controls/tests without expressions.
2. Pack digest hashed requirement/control/test **id lists after catalog merge**. Mapping completeness and expressions were invisible; catalog injection was visible.
3. `FrameworkReadinessSnapshot::serialize` and empty `AssessmentRun` pins reloaded live `catalog/canonical/v1` (`catalog-unavailable` fallback). Scheduler reloaded `load_framework_pack` at projection time.
4. `construct_test_plan` set `CompiledTest.expr = None`, so catalog `all`/`any`/`not`/threshold trees never reached evaluation.
5. Unknown mapping completeness/direction defaulted to Partial/Forward.
6. `project_readiness` was not the only requirement-status owner (`overlay_privileged_mfa_presence`, invented `"0%"` coverage).

ADR 0003 catalog forbids `weeping-angel-framework` depending on `weeping-angel-canonical-catalog`. That constraint remains.

## Decision (shipped)

### 1. One catalog TOML parser; IR projection adapter

- **`weeping-angel-canonical-catalog::CanonicalCatalog` is the only parser of catalog TOML.**
- `discover_catalog_index` / `CatalogIndex` / `IndexedControl` / `IndexedTest` are removed. The framework crate does not `toml::from_str` catalog control/evidence/test files.
- `weeping-angel-framework` does **not** depend on the catalog crate.
- Adapter (no new crate, no forbidden edge):
  - IR type `CatalogProjection { digest, controls, tests }` in `weeping-angel-assurance-ir` (`PlannedControlTest.expr` carries lossless JSON `TestExpr`).
  - `CanonicalCatalog::projection()` builds that view (controls + planned tests including expression JSON).
  - Catalog crate registers `WorkspaceCatalogLoader` via `inventory`. Named pack load (`load_framework_pack`) consumes `workspace_catalog_projection()`. Callers may pass an explicit projection with `load_framework_pack_from_with` / `validate_framework_pack_with`.
- Hypothetical package `weeping-angel-catalog` remains forbidden.

Rejected alternatives:

- **Framework depends on catalog crate** — violates ADR 0003 crate graph.
- **Keep silent index** — two semantics for the same files.
- **Generate a second on-disk catalog** — a new SSOT.

### 2. Packs are projections, not control libraries

- `frameworks/**/metadata.toml` `[[control]]` / `[[test]]` rows are `PackError::CompetingLibrary`. Pack-only annotations (`[pack] library = …`) remain.
- Mapping `to` must be a `control.*` id present on the supplied `CatalogProjection`. Unknown / missing catalog → `PackError::Dangling`. Packs do not invent hidden alternative controls.
- `load_framework_pack_from` without a projection does not walk `catalog/canonical/v1`; mappings then fail closed as dangling.

### 3. Semantic pack digest; separate catalog pin

`FrameworkPackDigest` is IR `canonical_digest` over a BTree-sorted JSON body of **pack-authored** semantics:

```text
schema, framework, version, contentMode, capabilities,
requirements {id, title, kind},
mappings {from, to, relation, completeness, direction, rationale, provenance, validFor},
applicability {reference, requirement, applicability, applicable, rationale}
```

It is **insensitive** to whitespace, comments, TOML key order, and filesystem enumeration.

It **must change** when those pack semantics change (including completeness vs relation).

It does **not** include live catalog control injection, catalog test expressions, or raw file bytes. Catalog identity is `CanonicalCatalog::digest` (`wa:canonical-catalog:weeping-angel/canonical-catalog/v1:<hex>`), stored beside the pack digest on `LoadedPack`, `CompiledFramework`, `FrameworkReadinessSnapshot.catalogDigest`, and `AssessmentRun.canonicalCatalogDigest` / `catalogDigest`.

Declared `manifest.digest`, when present and non-empty, must equal the computed semantic digest (`PackError::DigestMismatch`).

Rejected alternatives:

- Hash raw file bytes (formatting churn).
- Hash merged id lists (injection churn; missing completeness).
- Mix catalog bytes into pack digest (two identities become one).

### 4. Pins are execution identity; serialize/project are pin-pure

- Compile records `CompiledFramework.framework_pack_digest` and `catalog_digest` from the pack/projection used for that compile.
- `project_readiness` copies `compiled.catalog_digest` onto `FrameworkReadinessSnapshot.catalog_digest` (additive serde; JSON `catalogDigest`).
- Snapshot `Serialize` emits the **stored** field. It does not call `snapshot::catalog_digest()`.
- `AssessmentRun::serialize` emits stored `canonical_catalog_pin` as both `canonicalCatalogDigest` and `catalogDigest`. Empty pin stays empty; it does not reload `catalog/canonical/v1`.
- Scheduler `run_project` / `run_snapshot` use `self.compiled` digests. They do not `load_framework_pack` to refresh identity.
- `snapshot::catalog_digest` / facade `load_catalog_pin` remain **assess-start** pin establishment only (first existing catalog root; miss → `"catalog-unavailable"`). Serialize must not call them.
- `serialize_assessment_report` remains pin-pure (lineage / Prompt 3). This ADR does not reopen filesystem lookup there.

Rejected alternatives:

- Serialize-time `CanonicalCatalog::load` “to be helpful”.
- Substituting today’s catalog for a historical snapshot pin.

### 5. Expressions are lossless through compile

Catalog `[test.expression]` projects onto `PlannedControlTest.expr` as JSON `TestExpr` (`CanonicalCatalog::projection` / `expression_to_json`). Nested `all` / `any` / `not` / `none` and threshold / population operators remain distinct. Nested unknown `op` fails closed (`validate_expression_table` walks `children` / `of` / `args` / `expressions`).

`construct_test_plan` copies `t.expr`. It must not drop a catalog-authored tree. Presence-only overlays that replace the catalog predicate are forbidden (`overlay_privileged_mfa_presence` removed).

Evaluation (`evaluate_compiled`) attaches `CompiledTest.expr` when JSON-deserializable. Do not fork a third AST.

### 6. One readiness status owner

`weeping-angel-assurance::readiness::project_readiness` is the exclusive implementation of framework requirement-status aggregation, including mapping honesty (`relation_may_fully_satisfy`: `PartiallySatisfies` / `Supports` / `Related` / `EvidenceFor` / `SubsetOf` never fully satisfy; `Equivalent` / `Satisfies` / `SupersetOf` only with completeness `Full`).

Callers invoke it. They do not fork `has_partial` or status strings. Snapshot serialize may **format** coverage counts from stored rows; it does not decide requirement `status`.

`empty_readiness` is an empty constructor (zero counts, empty pin strings). It does not invent `"0%"` status labels.

SoA, CAPA, residual risk, and objectives keep their own domain projections but must not reimplement ISO/framework requirement status.

### 7. Fail-closed pack parse

Typed `PackError` (no best-effort weaker interpretation):

| Input | Error |
| --- | --- |
| schema ≠ `weeping-angel/framework-pack/v1` | `Schema` |
| unknown / empty `completeness` | `UnknownCompleteness` (not silent `Partial`) |
| unknown / empty `direction` | `UnknownDirection` (not silent `Forward`) |
| unknown `relation` | `UnsupportedRelation` |
| empty `relation` + known completeness | still `MappingRelation::from_completeness` |
| unknown / empty provenance `source` | `UnknownSource` (not silent `BuiltIn`) |
| dangling requirement or catalog `to` | `Dangling` |
| competing `[[control]]` / `[[test]]` | `CompetingLibrary` |
| duplicate requirement id | `DuplicateRequirement` |
| duplicate mapping `(from, to, relation)` | `DuplicateMapping` |
| declared digest ≠ computed | `DigestMismatch` |
| unknown `content_mode` | `Schema` |
| malformed TOML / IO | `Parse` / `Io` |

Assessment must not proceed on a pack that failed parse.

## Public API (additive unless noted)

Compatible extensions used to close the correctness hole:

- `CanonicalCatalog::projection() -> Result<CatalogProjection, CatalogError>`
- IR `CatalogProjection`, `WorkspaceCatalogLoader`, `workspace_catalog_projection()`
- IR `PlannedControlTest.expr: Option<serde_json::Value>`
- `load_framework_pack_from_with` / `validate_framework_pack_with`
- `LoadedPack.catalog_digest`, `CompiledFramework.{framework_pack_digest, catalog_digest}`
- `FrameworkReadinessSnapshot.catalog_digest` (JSON `catalogDigest`)
- `PackError::{UnknownCompleteness, UnknownDirection, UnknownSource, CompetingLibrary, MalformedExpression, DuplicateRequirement, DuplicateMapping, DigestMismatch}`

No `project_readiness_v2`. No new workspace crate.

## Consequences

- ISO pack `FrameworkPackDigest` strings change once (semantic body). Lineage fixtures that pin hex must follow.
- Loosely written packs that omit completeness/direction fail validate; owned `frameworks/**` must be explicit.
- Guard 05–08 (Prompt 1) can consume: single catalog parser, `CatalogProjection`, pack digest function, `project_readiness`, pin fields. This ADR does not implement those guards.
- Removing serialize-time live reload is a correctness fix. Tests that expected `"catalog-unavailable"` **at serialize** of an empty pin are wrong; empty stays empty.
- `assess` may still **establish** pins by loading the named pack and walking catalog roots. That is start-of-run identity, not serialize substitution. A failed pack reload on that path may still record the string `"unpinned"` on the report; scheduler projection no longer does.
- Prompt 3 continues to own lineage persist/replay and `serialize_assessment_report`.

## Related

- Increment spec: [`docs/specs/catalog-framework-readiness-trust-boundary.md`](../specs/catalog-framework-readiness-trust-boundary.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- Catalog infrastructure: [ADR 0003 catalog](0003-canonical-assurance-catalog-v1.md)
- ISO remap: [ADR 0003 remap](0003-iso27001-canonical-remap.md)
- Architecture-as-law: [ADR 0010](0010-architecture-as-law.md)
