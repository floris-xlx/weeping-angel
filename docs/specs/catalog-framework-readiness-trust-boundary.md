# SDD: Canonical catalog, framework, and readiness trust boundary

| Field | Value |
| --- | --- |
| Status | **Implemented** — catalog SSOT, fail-closed pack parse, semantic digest, pin-pure serialize, single readiness owner |
| Program | Architectural-cleanup PROGRAM — **Prompt 2 / increment 2** (phases **2 + 3 + 7 + 21**) |
| Slice | Catalog SSOT, fail-closed pack parse, semantic framework digest, expression preservation, pinned execution identity, single readiness projection owner |
| Characterization | Current `HEAD` of `floris-xlx/weeping-angel` (inspected 2026-08-19) |
| Dual-suite (implement, not this phase) | Extend `sdd_canonical_assurance_catalog_*`, `sdd_iso27001_assurance_*`, `sdd_iso27001_remap_*` under `tests/contracts/`. **Do not** create `tests/sdd/` |
| Extends (do **not** fork) | [`canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md), [`iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md), [`iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md) |
| Program law | [`architectural-cleanup-program.md`](architectural-cleanup-program.md) phases 2, 3, 7, 21 |
| Documentation architecture | [ADR 0004](../adr/0004-documentation-architecture.md) — this file is the human increment SSOT under `docs/specs/`. Generated traces go to `.sdd/runs/` and `.sdd/artifacts/`. `docs/sdd/` is a stub |
| ADR | **Accepted** [`docs/adr/0011-catalog-framework-digest-and-pin-ownership.md`](../adr/0011-catalog-framework-digest-and-pin-ownership.md) (cite **by path**; unique number **0011**, not another `0003-*`) |
| Predecessor ADRs (still law) | [ADR 0001](../adr/0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](../adr/0002-iso-27001-assurance-vertical.md), [ADR 0003 catalog](../adr/0003-canonical-assurance-catalog-v1.md), [ADR 0003 remap](../adr/0003-iso27001-canonical-remap.md), [ADR 0010](../adr/0010-architecture-as-law.md) |
| Public contract | [`assurance-runtime.md`](assurance-runtime.md) (untouched unless a documented breaking change is required to close a correctness hole) |
| IR schema (do not fork) | `assurance-ir/v1` |
| Catalog schema (do not fork) | `weeping-angel/canonical-catalog/v1` |
| Pack schema (do not fork) | `weeping-angel/framework-pack/v1` |
| `adr_needed` | **true** — digest canonicalization, catalog-parse ownership vs crate graph, pin ownership at serialize/project time |
| Workspace verify | `cargo test --test sdd_canonical_assurance_catalog_target`; `cargo test --test sdd_iso27001_assurance_target`; `cargo test --test sdd_iso27001_remap_target`; `cargo test -p weeping-angel-canonical-catalog`; `cargo test -p weeping-angel-framework`; `cargo test -p weeping-angel-assurance readiness`; `cargo fmt --all -- --check`; `cargo check --workspace` |

This document is the durable human SSOT for **cleanup Prompt 2**. It **extends** catalog infrastructure, ISO remap, and ISO MVP law. It does **not** replace catalog ID grammar, domain family TOML, ISO mapping tables, copyright/legal boundary, collector contracts, SoA graph law, or lineage persist schema.

Definition of done: *two representations of the same assurance semantics cannot drift silently; malformed or ambiguous catalog/pack input fails closed; assessment/readiness identity is the pinned catalog + pack that produced the result; readiness/effectiveness status rules have one owner.*

---

## 0. Collision fence (concurrent Prompts 1, 3, 4)

Prompt 2 may change **only** the trees below. Spec + accepted ADR live under `docs/specs/` and `docs/adr/`.

| Allowed | Forbidden (other prompts) |
| --- | --- |
| `crates/weeping-angel-canonical-catalog/**` | `xtask/**`, `architecture/**`, `docs/debt/register.toml` (Prompt 1) |
| `crates/weeping-angel-framework/**` | temporal / lineage persist / evidence `current()` vs `latest` / `soa.rs` implementation (Prompt 3) |
| readiness-specific code in `crates/weeping-angel-assurance/**`, especially `readiness.rs` and **directly related** pin/projection call sites | `serialize_assessment_report` (already pin-pure — leave it) |
| `catalog/**`, `frameworks/**` | broad test/schema/README hygiene (Prompt 4) |
| catalog/framework/readiness contract tests under `tests/contracts/**` | `tests/sdd/` (forbidden by ADR 0004) |
| spec/ADR text necessary to describe this increment | hypothetical crates `weeping-angel-catalog`, `weeping-angel-assurance-cli` |

Guard 05–08 (Prompt 1) must **consume** APIs/metadata this increment exposes. Do not edit guard implementation.

Prompt 3 owns `AssessmentRun` rebuild beyond pin fields already present, evidence ledger temporal APIs, and SoA. This increment may add **pin-pure fields** on `FrameworkReadinessSnapshot` and stop **live catalog reload** in `readiness.rs` / `snapshot.rs` serialize. Tiny backwards-compatible interface tweaks only if otherwise the readiness owner cannot be unique.

---

## 1. Problem / user-visible goal

Canonical catalog v1, ISO remap, and readiness projection already exist, but the **trust boundary is leaky**:

1. **Two catalog parsers.** `CanonicalCatalog::load` is fail-closed. `weeping-angel-framework::pack::discover_catalog_index` is a second silent parser (`Option`, `continue` on IO/TOML, no schema/duplicate/expression validation) that injects controls/tests into every pack load.
2. **Pack metadata can still be a competing library.** Loader still reads `metadata.toml` `[[control]]` / `[[test]]`, silently skips non-`control.*` rows, and can merge pack tests beside catalog tests.
3. **Expression trees are dropped.** Catalog tests declare `all-subjects` / `coverage-at-least` / `none-subjects` / `manual-review` (and allow nested `all` / `any` / `not`). Pack index does not copy expressions. `construct_test_plan` sets `CompiledTest.expr = None`. Assessment therefore cannot distinguish those operators.
4. **Pack parse is best-effort.** Unknown mapping `completeness` → `Partial`; unknown `direction` → `Forward`; unknown provenance source → `BuiltIn`; empty relation → `from_completeness`. Malformed catalog files during pack load do not fail the pack.
5. **Framework digest is not semantic.** It hashes sorted **id lists** (and `Debug` of relation) **after** catalog merge. Whitespace-equivalent packs can hash the same (good) but incidental catalog injection, mapping completeness/direction/rationale/expression, and applicability are missing; a semantic change that affects assessment can collide; a non-semantic catalog growth can change the pack digest.
6. **Pins are not sticky.** `FrameworkReadinessSnapshot::serialize` and empty `AssessmentRun.canonical_catalog_pin` call `snapshot::catalog_digest()` which reloads live `catalog/canonical/v1` (fallback `"catalog-unavailable"`). Scheduler `run_project` / `run_snapshot` call `load_framework_pack` again and treat failure as `"unpinned"`.
7. **Readiness rules fork.** Authoritative aggregation is `project_readiness`, but snapshot serialize recomputes coverage; scheduler `overlay_privileged_mfa_presence` overwrites a control result; `empty_readiness` invents `"0%"` strings; lineage report serialize (Prompt 3) reconstructs a snapshot without calling `project_readiness`.

**User-visible goal:** a readiness assessment whose catalog items, framework mappings, test expressions, and status labels are **one story**:

```text
catalog/canonical/v1  (only semantic source for control.* / evidence.* / test.*)
        ↓ CanonicalCatalog::load (fail-closed)
frameworks/<id>/<ver> (requirements, honest mappings, applicability — not a control library)
        ↓ load_framework_pack (fail-closed, semantic digest)
compile_framework     (expression-preserving CompiledTest.expr)
        ↓ pin (catalogDigest, frameworkPackDigest)
project_readiness     (only status-rule owner)
        ↓ serialize/report using pins, never live files
```

Never:

```text
pack metadata.toml sliver  ≈  catalog control
discover_catalog_index     ≈  CanonicalCatalog::load
CompiledTest { expr: None } ≈  catalog [test.expression]
live catalog/canonical/v1  ≈  the catalog that produced this snapshot
overlay_privileged_mfa     ≈  catalog coverage expression
```

This remains **readiness/assurance**, not certification.

---

## 2. Compatibility / dependencies

| Surface | Rule for this increment |
| --- | --- |
| Catalog ID grammar, reserved segments, `weeping-angel-canonical-catalog` crate name | Unchanged ([catalog v1](canonical-assurance-catalog-v1.md), ADR 0003 catalog) |
| Crate graph | Framework **must not** gain a Cargo dependency on `weeping-angel-canonical-catalog` (ACT-003 / catalog ADR). Catalog remains the only TOML parser; packs consume an **adapter/projection** supplied by the facade or an IR-shaped view — not a second parser |
| IR `Control` / `Requirement` / `Mapping` / `PlannedControlTest` | Do not redesign. Tiny additive field on `PlannedControlTest` or compile-side expression carry is allowed if required for lossless `expr` |
| ISO mapping tables / legal structural pack | Remap spec still owns which ISO ids map where. This increment does not remap new Annex A clauses |
| `project_readiness` signature | Stay the public projection entry. Add pin fields / params rather than a second `project_*_v2` |
| `serialize_assessment_report` | Prompt 3 — pin-pure; do not rewrite. Expose snapshot fields it can already carry |
| Public APIs | Compatible unless a breaking change is required to close a correctness hole, in which case document it in ADR 0011 |
| Workspace members | Keep seven `weeping-angel-*` crates + `xtask`. Never `weeping-angel-catalog` / `weeping-angel-assurance-cli` |

---

## 3. Current behavior (baseline — GREEN on CURRENT code)

Characterized against the tree at spec time. Increment **characterization** tests added in the dual-suite protocol (§7) must **PASS on this behavior** before any product change.

### 3.1 Inventory — paths that can define or reinterpret catalog semantics

| Path | What it can define today | Authority |
| --- | --- | --- |
| `catalog/canonical/v1/{manifest,controls,evidence,tests}/**` | control / evidence / test identity, expression TOML, subjects | **Intended SSOT.** `CanonicalCatalog::load` fail-closed |
| `crates/weeping-angel-canonical-catalog/src/lib.rs` | schema, ID grammar, digest, expression `op` allow-list | Intended loader |
| `crates/weeping-angel-framework/src/pack.rs` `discover_catalog_index` | **Second parser** of the same TOML: control id/title/description, test id/control/kind/`required_evidence`. **Drops** expression, subjects, break_on, evidence documents, schema, duplicates | **Competing.** `Option`; `continue` on missing/unreadable/unparseable files and rows |
| `frameworks/*/metadata.toml` `[[control]]` / `[[test]]` | Pack-local control/test library if present. ISO pack currently has annotations only (`library = "catalog/canonical/v1"`) but the **loader still implements** the library | **Latent competing library.** Non-`control.*` rows `continue` (silent drop) |
| `frameworks/*/mappings.toml` | requirement → control, relation, completeness, direction | Pack SSOT for **projection**, but defaults rewrite unknown fields |
| `frameworks/*/applicability.toml` | SoA-oriented applicability rows | Pack data; Prompt 3 owns `project_soa` |
| `construct_test_plan` | `CompiledTest.expr: None` always | **Drops** catalog expression trees |
| `stub_catalog` | Loads pack **requirements**, not canonical catalog | Name collision only; not a control library |
| `snapshot::catalog_digest` / `load_catalog_pin` | Reloads live `catalog/canonical/v1`; `"catalog-unavailable"` on failure | **Mutable current** substituted for pin |
| `scheduler::overlay_privileged_mfa_presence` | Re-evaluates `control.identity.privileged-mfa` with a presence test on `identity.privileged.mfa`, overwriting catalog coverage results | **Second effectiveness rule** |
| Collector evidence types / GitHub normalize | Facts, not controls | Must stay catalog- and ISO-blind |

Exactly one authoritative semantic source per catalog item is **not** true today: pack load can invent `Control` / `PlannedControlTest` from a partial catalog parse **or** from metadata rows.

### 3.2 Canonical catalog load (authoritative crate — already fail-closed for *its* API)

`CanonicalCatalog::load`:

- requires `schema = weeping-angel/canonical-catalog/v1`;
- listed-file + no-unlisted-extra + no path escape;
- duplicate ids → `CatalogError::Duplicate`;
- dangling / orphaned refs → error;
- reserved provider/framework segments → error;
- unknown `op` / missing `op` when expression present → error;
- unknown subject `kind` → error;
- digest = IR `canonical_digest` over parsed BTree-sorted documents (not raw bytes).

This path is **not** used by `load_framework_pack`. Existing CAT-004…010 remain law.

`validate_expression` only checks top-level `op` ∈ allow-list. Nested `all`/`any`/`not` children are not structurally validated beyond “table present”. Nested trees are stored on `CatalogTest.expression: BTreeMap<String, toml::Value>` and never projected into `TestExpr`.

### 3.3 Silent catalog parser (`discover_catalog_index`)

```584:669:crates/weeping-angel-framework/src/pack.rs
fn discover_catalog_index() -> Option<CatalogIndex> {
    // find first manifest.toml among catalog_search_roots()
    // parse as toml::Value; on IO/TOML failure return None
    // for each listed controls/tests file: read_to_string.ok / from_str.ok / continue
    // skip rows without id (and tests without control)
    // does not read evidence/, expressions, schema, duplicates
}
```

Callers (`load_framework_pack_from`):

- treat missing catalog as “no catalog” (`Option`);
- inject catalog controls for mapping `to` when `control.*` and (`in_pack` or `in_catalog`);
- attach `index.tests_for(control)` **without expressions**;
- then append `metadata.toml` tests.

Partial catalog parse **never** becomes `PackError`.

### 3.4 Pack parse defaults (not fail-closed)

| Input | Current interpretation |
| --- | --- |
| unknown `completeness` (including empty except the `full`/`related` arms) | `MappingCompleteness::Partial` |
| unknown / empty `direction` | `MappingDirection::Forward` |
| empty `relation` | `MappingRelation::from_completeness(completeness)` |
| unknown provenance `source` | `MappingSource::BuiltIn` |
| unknown relation string | `PackError::UnsupportedRelation` (this arm **is** fail-closed) |
| mapping `to` not `control.*` or not in pack/index | `PackError::Dangling` |
| `metadata.toml` `[[control]]` id not starting `control.` | **skipped** |
| `metadata.toml` `[[test]]` control not starting `control.` | **skipped** |
| `content_provider` | always `StructuralOnly` regardless of manifest `content_mode` |
| pack `[[digest]]` / digest field mismatch | **not parsed** |

ISO `metadata.toml` currently has no `[[control]]`/`[[test]]` library (remap landed). The skip/merge **code path** remains.

### 3.5 Expression drop

Catalog tests (example: `catalog/canonical/v1/tests/identity.toml`) use `op = "all-subjects"` / `"coverage-at-least"` / `"manual-review"`. `IndexedTest` has no expression field. `PlannedControlTest` (IR) has no `expr`. `construct_test_plan`:

```367:384:crates/weeping-angel-framework/src/lib.rs
// maps PlannedControlTest → CompiledTest { …, expr: None }
```

`evaluate_compiled` only applies `test.expr` when `Some` and JSON-deserializable. Today that is never catalog-derived. `all`/`any`/`not`/`threshold`/`coverage` distinctions cannot affect assessment output.

`overlay_privileged_mfa_presence` then **replaces** the privileged-MFA result with a presence-only `CompiledControlTest` requiring `identity.privileged.mfa` — a different predicate than `test.identity.privileged-mfa-enabled` (`coverage-at-least` 100% on `evidence.identity.mfa-status`).

### 3.6 Framework digest (non-semantic)

`load_framework_pack_from` hashes:

```text
schema, framework id, version,
requirement ids, control ids (after catalog injection + pack metadata),
mappings as (from, to, format!("{:?}", relation)),
test ids
```

Not hashed: titles, mapping completeness/direction/rationale/provenance/`valid_for`, applicability, test kind/required evidence/break_on/**expression**, evidence requirement types, content_mode.

Consequences:

- adding a catalog control that a mapping already targeted (injection) **changes** pack digest without changing pack files;
- changing `PartiallySatisfies` ↔ `Supports` **does** change digest (relation Debug);
- changing completeness `partial` → `full` (same relation string) **may not** change digest while **does** change `project_readiness` (`relation_may_fully_satisfy`);
- TOML key order / comments / whitespace typically do **not** change digest (JSON of ids) — incidental formatting is already ignored, but **semantic** content is incomplete.

### 3.7 Pin reload (stale/live substitution)

| Site | Behavior |
| --- | --- |
| `AssuranceEngineBuilder::assess` | `load_framework_pack` again after compile; failure → `"unpinned"`. `load_catalog_pin()` walks live catalog roots; failure → `"catalog-unavailable"`. Stores those strings on `AssessmentReport` / `AssessmentRun.canonical_catalog_pin` |
| `FrameworkReadinessSnapshot::serialize` | **Always** `serialize_field("catalogDigest", &catalog_digest())` — live reload, ignores any pin that might exist |
| `AssessmentRun::serialize` | If `canonical_catalog_pin` empty, calls `catalog_digest()` (live) |
| `snapshot::catalog_digest` | `CanonicalCatalog::load` on first existing root; else `"catalog-unavailable"` |
| `scheduler::run_project` / `run_snapshot` | `load_framework_pack(framework, version)` for digest; failure → `"unpinned"`. Snapshot pin left empty so later JSON serialize fills live catalog |
| `lineage::serialize_assessment_report` | **Pin-pure** (`carried_pack_pin` / `carried_catalog_pin` only). **Do not change** (Prompt 3) |

Reporting/serializing a historical snapshot can therefore describe **today’s** catalog, not the catalog that produced the results.

### 3.8 Readiness / effectiveness calculation sites

| Site | Role today |
| --- | --- |
| `weeping-angel-assurance::readiness::project_readiness` | **Intended owner.** Walks compiled mappings; `PartiallySatisfies`/`Supports`/… cannot fully satisfy; requirement status strings |
| `readiness::coverage_metrics` (private, used only in snapshot `Serialize`) | Re-derives five coverage counts from snapshot rows (not from `project_readiness` stored strings) |
| `lineage::assessment_summary` / `lineage::coverage_metrics` | Count effectiveness on `ControlTestResult` (Prompt 3 module). Not requirement-status rules |
| `scheduler::overlay_privileged_mfa_presence` | Overwrites one control’s `ControlTestResult` |
| `scheduler::empty_readiness` | Blank snapshot with `"0%"` coverage strings |
| `lineage::serialize_assessment_report` | Builds `FrameworkReadinessSnapshot` **without** `project_readiness` (empty `requirements`, copies control effectiveness). Prompt 3 owns this file |
| `soa.rs` `combine_effectiveness` | SoA dimension — Prompt 3; must **call** control effectiveness already computed, not reimplement catalog/framework status |
| `capa` / `residual` / `objectives` / `remediation` | Domain-specific effectiveness of **those** engines — out of this increment except they must not become a second ISO/readiness status algebra |

`project_readiness` already distinguishes mapping relations for requirement status (`relation_may_fully_satisfy`). Snapshot serialize still reloads catalog. Scheduler still reloads pack.

### 3.9 Semantic distinctions that already exist and must not collapse

Already law (MVP / remap / spine); this increment must not regress:

- evidence envelope ≠ framework status;
- scanner `security_finding` ≠ compliance result;
- accepted risk / exception ≠ remediation;
- document existence ≠ operational effectiveness;
- missing coverage / empty population ≠ `Effective`;
- `Equivalent`, `PartiallySatisfies`, and `Supports` stay distinct;
- packs project onto `control.*` rather than hidden slivers;
- collector/catalog APIs must not leak ISO-specific status into evidence (`iso27001:` ids stay out of collectors and out of `evidence.*` ids).

---

## 4. Desired behavior (after implement)

### 4.1 One canonical catalog semantic source

1. **`catalog/canonical/v1` + `CanonicalCatalog` is the only parser of catalog TOML.** No other crate may `toml::from_str` catalog control/evidence/test files.
2. **Delete or convert** `discover_catalog_index` / `CatalogIndex` / `IndexedControl` / `IndexedTest`. Pack load must not walk `catalog/canonical/v1` on its own.
3. **Adapter (ADR 0011):** the assurance facade (already depends on both crates) loads `CanonicalCatalog` fail-closed and supplies an IR-shaped projection (controls + planned tests **including expressions**) into compile/assess. `weeping-angel-framework` stays catalog-crate-free.
4. **`metadata.toml` is not a control/test library.** Presence of `[[control]]` or `[[test]]` rows that declare catalog-competing ids is `PackError` (typed). Pack-only annotations (`[pack] library = …`) remain. Silent skip of non-`control.*` rows is forbidden: those rows are errors.
5. Mapping `to` must resolve against the **loaded CanonicalCatalog** (or the supplied projection) at assess/validate time. Unknown catalog ids fail closed. Packs must not create hidden alternative controls to satisfy dangling mappings.

Generated/projection forms allowed: in-memory IR `Control` / `PlannedControlTest` / `CompiledTest` derived from the catalog. Those are not a second authoring SSOT.

### 4.2 Catalog loading remains deterministic and fail-closed

Keep existing `CanonicalCatalog::load` / `validate` law. Additionally:

- secondary/partial parse paths are **errors**, not `Option`/`continue`;
- duplicate IDs, malformed records, unknown relation kinds (when catalog files grow relations), invalid references, unsupported schema versions, impossible mappings, and partial parse failures are explicit `CatalogError` / `PackError`;
- never silently drop records;
- nested expression tables with unknown child `op` fail closed (extend `validate_expression` as needed without pulling `weeping-angel-control-test` into the catalog crate).

### 4.3 Lossless expression preservation

1. Catalog `[test.expression]` is the semantic source for canonical tests.
2. Compile **must** carry the complete tree onto `CompiledTest.expr` as JSON `TestExpr` (or an equivalent lossless encoding). `construct_test_plan` must **not** set `expr: None` when the planned test has an expression.
3. Parse/serialize/reload of any pack- or catalog-authored expression must preserve `all` / `any` / `not` / nested combinations, thresholds (`coverage-at-least`, `count`, `count-where`), `all-subjects` / `any-subject` / `none-subjects` / `missing-subjects`, and `manual-review`. Do not normalize `all`↔`any`, drop `not`, or collapse `PartiallySatisfies` meaning into `Satisfies`.
4. Round-trip and **adversarial** fixtures: reorder TOML keys, add comments/whitespace, swap equivalent list order where order is not semantic, mutate one operator, mutate threshold, insert extra `not`.
5. Evaluation uses the preserved tree (`evaluate_compiled`). Presence-only overlays that replace catalog expressions are forbidden.

Tiny IR/compile field on `PlannedControlTest` is allowed if that is the adapter; do not fork `TestExpr` into a third AST.

### 4.4 Framework pack parsing fail-closed

`load_framework_pack` / `load_framework_pack_from` / `validate_framework_pack` must return a **typed** `PackError` (no best-effort weaker interpretation) when:

- manifest schema ≠ `weeping-angel/framework-pack/v1` or required identity/version missing;
- requirements/mappings/applicability TOML malformed;
- unknown mapping `relation`, `completeness`, `direction`, or provenance `source` (empty relation may remain “derive from completeness” **only** if completeness itself is an explicit known token; unknown completeness is an error, not `Partial`);
- dangling requirement or catalog control / evidence / test reference;
- malformed or unknown expression when a pack carries one;
- unsupported/mismatched declared digest field (if present, must match computed semantic digest);
- competing `[[control]]`/`[[test]]` library rows in metadata;
- duplicate requirement ids or duplicate mapping identity `(from, to, relation)` as specified by tests.

Assessment must not proceed on a pack that failed parse. No `"unpinned"` success path for a failed load on the assess/project critical path.

### 4.5 Semantic framework digest

Redesign `FrameworkPackDigest` around **canonical semantic content of the pack**, not filesystem order or formatting:

**Must be identical when** whitespace, comments, TOML key order, and path enumeration differ but semantics are identical.

**Must change when** any of the following that can affect assessment output changes: requirement identity/title/kind set; mapping from/to/relation/completeness/direction/rationale/provenance/`valid_for`; applicability entries that compile consumes; test plan identity **authored in the pack** (if any remain); content_mode / capabilities that gate compile.

**Must not** include:

- incidental catalog control injection (catalog identity is a **separate pin**);
- raw file bytes;
- directory walk order.

Payload should be `serde_json` of BTree-sorted structured values via IR `canonical_digest` (same family as catalog digest). Prefix/display may follow existing `FrameworkPackDigest(String)` to keep serde compatible.

Catalog digest remains `CanonicalCatalog::digest` (`wa:canonical-catalog:weeping-angel/canonical-catalog/v1:<hex>`). Do not mix catalog bytes into pack digest.

### 4.6 Bind execution to pinned identities

1. Every assessment/readiness result that this increment owns carries **both** `frameworkPackDigest` and `catalogDigest` / `canonicalCatalogDigest` of the catalog and pack **actually used** to compile/evaluate.
2. `project_readiness` records `catalogDigest` on `FrameworkReadinessSnapshot` (new stored field; serde default ok). Snapshot `Serialize` emits **that field**, never `catalog_digest()`.
3. `AssessmentRun::serialize` emits `canonical_catalog_pin` as stored. Empty pin stays empty or a typed missing-pin error; it must **not** reload live catalog.
4. Scheduler projection/snapshot uses the digest already on `CompiledFramework` / assessment identity. It must not `load_framework_pack` at project time to “refresh” identity.
5. `load_catalog_pin()` at **start of assess** is allowed to **establish** the pin from the catalog used for that run. After that, serialize/report/compare use the stored pin.
6. `serialize_assessment_report` stays pin-pure (Prompt 3). This increment must not force it to load files.

### 4.7 Single readiness projection owner

1. **`weeping-angel-assurance::readiness::project_readiness` is the only implementation of framework requirement-status and readiness aggregation rules** (including `partially covered` vs `effective` given mapping honesty).
2. Callers invoke it (or a thin wrapper that only supplies pins/clocks). They do not reimplement status strings, `has_partial`, or “every control maps to every requirement”.
3. Remove or reduce `overlay_privileged_mfa_presence` so privileged-MFA effectiveness is the catalog test expression, not a second predicate.
4. `empty_readiness` may exist as an empty snapshot constructor but must not invent coverage percentages or status labels that bypass `project_readiness`.
5. Snapshot coverage JSON may format counts, but the **rules** that decide requirement `status` live in `project_readiness`.
6. Do not add `project_readiness_iso` / duplicated match arms in scheduler/CLI.
7. Lineage report serialize (Prompt 3) should eventually embed a snapshot produced by `project_readiness`; this increment does **not** rewrite `lineage.rs` to get there. Expose pin-pure snapshot fields so Prompt 3 can call rather than fork.

### 4.8 Preserve project semantic distinctions

Non-negotiable (executable in target tests):

- evidence is not framework status;
- scanner findings are not compliance results;
- accepted risk is not remediation;
- document existence is not operational effectiveness;
- missing coverage is not success;
- `Equivalent`, `PartiallySatisfies`, and `Supports` remain distinct in parse, digest, graph, and readiness;
- framework packs project onto canonical controls; no hidden alternative controls;
- collector-facing and catalog-facing APIs do not leak ISO-specific status into evidence (`iso27001:` / `ISO 27001 compliant` stay out of `evidence.*` and collector facts).

### 4.9 Public API compatibility

Prefer adapters into one authoritative model over duplicating types or `#[cfg]` compatibility branches.

Breaking changes allowed only to close a correctness hole, and only if ADR 0011 documents them. Candidates (decide at implement, document in ADR):

- `FrameworkReadinessSnapshot` gains `catalog_digest: String` (additive serde);
- `project_readiness` gains a catalog-digest argument **or** reads it from compiled identity;
- `PackError` gains variants (`UnknownCompleteness`, `CompetingLibrary`, `MalformedExpression`, …);
- removal of public `discover_catalog_index` (it is currently private — good).

Do not paper over with new `#[ignore]`, broad allowlists, or new debt unless unavoidable, narrowly scoped, owned, expiring, and justified (Prompt 1 owns the debt register — avoid new rows).

---

## 5. Acceptance criteria (testable)

1. **SSOT.** Grep/architecture-level target: no second catalog TOML parser; `CanonicalCatalog::load` is the only parse of `catalog/canonical/v1` control/evidence/test files; pack load does not `continue` over catalog IO/TOML.
2. **Catalog fail-closed.** Duplicate IDs, dangling refs, unsupported schema, malformed expression/selector, unlisted files still fail (`CanonicalCatalog`). Pack-time catalog gaps fail as `PackError`/`AssuranceError`, not silent omission.
3. **Expression lossless.** A fixture with nested `all`/`any`/`not` and a threshold round-trips through catalog → compile → `CompiledTest.expr` → JSON → `TestExpr` with equal trees. Mutating `all`→`any` or dropping `not` changes evaluation and digest/pin identity as specified.
4. **Pack fail-closed.** Fixtures: unknown completeness; unknown direction; unknown relation; dangling `to`; competing metadata `[[control]]`; malformed manifest schema. Each returns typed error; assessment does not run.
5. **Semantic digest.** Two packs differing only by comments/whitespace/key order share `FrameworkPackDigest`. Changing mapping completeness, relation, or requirement id changes it. Injecting additional catalog controls **without** pack semantic change does **not** by itself redefine pack digest (catalog pin tracks catalog).
6. **Pins.** After assess, mutating on-disk `catalog/canonical/v1` or the pack directory does not change serialized `catalogDigest` / `frameworkPackDigest` of an in-memory snapshot/run. Empty pin does not become live `catalog-unavailable` via serialize-time reload.
7. **Readiness owner.** Requirement status for partial mappings is computed only in `project_readiness`. Scheduler does not overlay a second privileged-MFA predicate. Target grep: no `let has_partial = true`; no second `status = "partially covered"` implementation in scheduler/readiness callers.
8. **Negative tests** listed in §7.3 all fail closed rather than degrade.
9. **Distinctions.** Remap/ISO targets still prove `PartiallySatisfies` ≠ `Equivalent`; collectors still have no `iso27001:` requirement ids.
10. **Hygiene.** `cargo fmt --all -- --check`; listed verify commands GREEN; `cargo check --workspace` succeeds. Full workspace `cargo test` is **out of increment gate**.

---

## 6. Out of scope

1. Prompt 1: `xtask/**`, `architecture/**`, `docs/debt/register.toml`, implementing Guard 05–08 (consume APIs only).
2. Prompt 3: `soa.rs`, lineage persist/rebuild, `serialize_assessment_report`, evidence `current()` / `as_of(t)`, temporal move.
3. Prompt 4: baseline mass-retirement, schema fixtures, README/docs layout `CANONICAL_SPECS` unless a path is added here (this file may be listed later by Prompt 4).
4. New crates `weeping-angel-catalog`, `weeping-angel-assurance-cli`.
5. `tests/sdd/` or new root `[[test]]` binaries beyond extending the three existing dual-suites.
6. Remapping additional ISO Annex A clauses / domain catalog content authorship (IAM/SDLC/vuln/infra/governance family SSOT files).
7. Collector implementation, GitHub normalize, scanner bridge redesign.
8. Control-test evaluator rewrite (consume `TestExpr`; do not add a script host).
9. Certification language, licensed ISO normative text.
10. ADR mass-renumber; minting another `0003-*`.
11. Making `weeping-angel-framework` depend on `weeping-angel-canonical-catalog`.
12. Full `cargo test --workspace` as a gate (compile + named tests only).
13. Closing `DEBT-GUARD-05…08` in the debt register (Prompt 1).
14. Changing `ASSURANCE_IR_SCHEMA` / inventing `assurance-ir/v2`.

---

## 7. Dual-suite protocol (mandatory, ordered)

```text
(1) spec first (this file + extensions + draft ADR)     ← this phase
(2) characterization baseline GREEN on CURRENT code
(3) desired-behavior target RED on CURRENT for the right reason
(4) implement in allowed trees until target GREEN
(5) prove increment baselines FAIL or document additive-then-supersede
(6) target still GREEN
```

Do **not** create `tests/sdd/`. Suites stay registered in root `Cargo.toml`:

| Suite | Path | This increment |
| --- | --- | --- |
| Catalog baseline | `tests/contracts/canonical_assurance_catalog.baseline.rs` | Add **characterization** tests that PASS on CURRENT seams (§3). Keep existing `#[ignore]` absence asserts |
| Catalog target | `tests/contracts/canonical_assurance_catalog.target.rs` | Add desired-behavior tests that FAIL on CURRENT (second parser, expression drop, live pin reload, …). Keep CAT-001…016 |
| ISO remap baseline/target | `tests/contracts/iso27001_remap.{baseline,target}.rs` | Pack fail-closed, semantic digest, mapping default holes, pin/readiness owner as they touch remap |
| ISO MVP target | `tests/contracts/iso27001_assurance.target.rs` | Expression preservation / readiness distinctions already in ISO-00x; extend only if needed |

Neighbor crate packages have **no** in-crate `[[test]]`; verify with `cargo test -p weeping-angel-canonical-catalog`, `-p weeping-angel-framework`, `-p weeping-angel-assurance readiness`.

### 7.1 Characterization baseline (must PASS on CURRENT)

Suggested names (implement may refine; encode the **original found case**):

| ID | Asserts on CURRENT |
| --- | --- |
| CAT-SSOT-B01 | `pack.rs` contains `fn discover_catalog_index` returning `Option` and `continue` on IO/TOML |
| CAT-SSOT-B02 | `construct_test_plan` sets `expr: None` |
| CAT-SSOT-B03 | ISO `metadata.toml` has no `[[control]]` library **and** pack.rs still deserializes `[[control]]`/`[[test]]` with skip of non-`control.*` |
| FRW-B01 | unknown mapping completeness defaults to Partial (fixture or source) |
| FRW-B02 | pack digest JSON keys are id lists (`"requirements"`, `"controls"`, `"tests"`) after merge |
| PIN-B01 | `FrameworkReadinessSnapshot::serialize` calls `catalog_digest()` |
| PIN-B02 | `snapshot::catalog_digest` / `load_catalog_pin` fallback `"catalog-unavailable"` |
| PIN-B03 | scheduler `run_project` calls `load_framework_pack` |
| RDY-B01 | `overlay_privileged_mfa_presence` exists and overwrites privileged-MFA results |

Existing ignored catalog absence asserts stay ignored.

### 7.2 Target (must FAIL on CURRENT for the right reason, then GREEN after implement)

| ID | Desired |
| --- | --- |
| CAT-SSOT-T01 | No second catalog TOML parser in `weeping-angel-framework`; pack load does not `continue` over catalog files |
| CAT-SSOT-T02 | Competing `metadata.toml` `[[control]]` fails pack load |
| CAT-SSOT-T03 | Duplicate catalog ids / invalid refs still fail via `CanonicalCatalog` (keep CAT-004…); pack validate fails if supplied catalog projection is incomplete |
| FRW-EXPR-T01 | Nested `all`/`any`/`not` + threshold survive compile; `CompiledTest.expr` is `Some` and round-trips |
| FRW-EXPR-T02 | Adversarial: `all` vs `any` vs `not(all)` are not normalized together |
| FRW-PARSE-T01 | Unknown completeness/direction/relation → typed `PackError` |
| FRW-PARSE-T02 | Dangling catalog control id → typed error, no assessment |
| FRW-DIG-T01 | Whitespace/comment/key-order identical digest |
| FRW-DIG-T02 | Completeness or relation change changes digest |
| FRW-DIG-T03 | Catalog injection without pack semantic change does not masquerade as pack identity (pack digest stable; catalog pin changes) |
| PIN-T01 | Snapshot serialize uses stored catalog digest; mutating live catalog files does not change serialized pin |
| PIN-T02 | Empty `AssessmentRun` pin does not invoke live `CanonicalCatalog::load` during serialize |
| PIN-T03 | Scheduler projection does not reload pack for digest |
| RDY-T01 | Only `project_readiness` assigns requirement `partially covered` / mapping-honesty status |
| RDY-T02 | No privileged-MFA overlay replacing catalog coverage expr |
| RDY-T03 | `PartiallySatisfies` effective tests stay `partially covered` (existing ISO-R law) |

Titles in the target suite should be stable (`cat_ssot_t01_…` etc.). One regression test per closed comment/seam, encoding the **original found case**.

### 7.3 Mutation / negative battery (target)

Must prove failure, not degradation:

- catalog duplicate control/evidence/test id;
- malformed pack manifest / mapping / expression;
- unknown mapping completeness, direction, relation, catalog id;
- altered expression tree (`all`→`any`, dropped `not`, threshold 100→0);
- digest instability (same semantics, different formatting → same digest);
- digest collision-by-normalization (different semantics, same id list → **different** digest);
- stale pin usage (disk changed after pin);
- duplicated readiness calculation (grep / behavioral: two functions disagree on the same compiled+results fixture).

---

## 8. Risks

1. **Crate-graph trap.** Teaching `weeping-angel-framework` to `CanonicalCatalog::load` would add a forbidden dependency. Mitigation: facade adapter (ADR 0011).
2. **Digest churn.** Semantic digest will change ISO pack digest strings; lineage/compare tests that pin hex must update. Mitigation: treat as intended identity change; document in ADR.
3. **Prompt 3 overlap.** Stopping live reload in `readiness.rs`/`snapshot.rs` while leaving `serialize_assessment_report` alone can leave **two** JSON shapes for catalogDigest until Prompt 3 consumes snapshot fields. Mitigation: additive field + pin-pure report already prefers stored report pins.
4. **Expression encoding.** Catalog TOML `op` strings vs `TestExpr` enum (`coverage-at-least` vs `CoverageAtLeast`) can lose meaning if the adapter is sloppy. Mitigation: round-trip fixtures including aliases already in `ALLOWED_OPS`.
5. **ISO remap tests freeze loader skip behavior.** Target extensions must not fight ISO-R-001…020; extend rather than rewrite goldens unless goldens encode the bug.
6. **Scheduler overlay** may be load-bearing for `sdd_continuous_assurance_scheduler_*` (not owned). Removing overlay can RED neighbor suites. Mitigation: preserve evaluation outcomes via catalog expr + `evaluate_compiled`, not a second predicate; do not edit scheduler tests unless a tiny readiness hook is required.
7. **Public serde.** Adding `catalogDigest` on snapshots is additive; changing serialize to stop live reload can RED tests that expected `"catalog-unavailable"` on missing trees. Mitigation: characterization captures current; target asserts pin-pure.
8. **Best-effort defaults** (`Partial`/`Forward`/`BuiltIn`) may be relied on by loosely written packs. Fail-closed will reject them. Mitigation: fix owned `frameworks/**` packs in this increment; do not keep silent defaults.

---

## 9. ADR

`adr_needed = true`. **Accepted:** [`docs/adr/0011-catalog-framework-digest-and-pin-ownership.md`](../adr/0011-catalog-framework-digest-and-pin-ownership.md).

Decisions landed:

1. Catalog TOML parse ownership (`CanonicalCatalog` only). Framework stays catalog-crate-free via IR `CatalogProjection` + `inventory` `WorkspaceCatalogLoader`.
2. Pack digest is semantic JSON of pack-authored fields (not merged id lists). Catalog identity is a separate pin.
3. Pin ownership: stored identity wins; snapshot / `AssessmentRun` serialize and scheduler project never reload mutable current files.
4. Expression carry: `CanonicalCatalog::projection` → `PlannedControlTest.expr` → `construct_test_plan` → `CompiledTest.expr`.
5. `project_readiness` exclusive owner for framework requirement status (`overlay_privileged_mfa_presence` removed).

Do not mint `0003-catalog-framework-readiness.md`. Cite ADR **0011** by path.

---

## 10. What shipped

| Surface | Landed |
| --- | --- |
| Catalog SSOT | `CanonicalCatalog::load` only TOML parser. `CanonicalCatalog::projection()` → IR `CatalogProjection`. `discover_catalog_index` gone |
| Pack load | `load_framework_pack` uses `workspace_catalog_projection()`. `load_framework_pack_from_with` / `validate_framework_pack_with` take an explicit projection. Competing `[[control]]`/`[[test]]` → `PackError::CompetingLibrary` |
| Fail-closed parse | Unknown/empty completeness, direction, provenance source; unknown relation; dangling `to`; duplicate requirement/mapping; schema; declared digest mismatch |
| Digest | `FrameworkPackDigest` = IR `canonical_digest` over pack schema/id/version/contentMode/capabilities/requirements/mappings/applicability. Not catalog injection |
| Pins | `LoadedPack.catalog_digest`, `CompiledFramework.{framework_pack_digest,catalog_digest}`, `FrameworkReadinessSnapshot.catalog_digest`. Serialize emits stored pins |
| Readiness | `project_readiness` copies `compiled.catalog_digest`. Scheduler project/snapshot use compiled pins. `empty_readiness` invents no coverage % |
| Expressions | Nested `all`/`any`/`not`/`none` + thresholds survive compile. Nested unknown `op` fails catalog validate |

`assess` may still **establish** report pins by loading the named pack and walking catalog roots (`"unpinned"` / `"catalog-unavailable"` only on that start-of-run path). Serialize and scheduler projection do not re-walk files.

---

## 11. Verify commands (implement gate)

```bash
cargo test --test sdd_canonical_assurance_catalog_target
cargo test --test sdd_iso27001_assurance_target
cargo test --test sdd_iso27001_remap_target
cargo test -p weeping-angel-canonical-catalog
cargo test -p weeping-angel-framework
cargo test -p weeping-angel-assurance readiness
cargo fmt --all -- --check
cargo check --workspace
```

Full workspace `cargo test` is out of increment gate.
