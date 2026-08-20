# SDD: Weeping Angel Architectural Consolidation Program — Phase 0 (implemented) + Phase 1 (domain ownership law)

| Field | Value |
| --- | --- |
| Status | **Phase 0 implemented.** Consolidation mode, frozen baseline snapshot, and v2 duplication backlog schema are landed (CON-T01–T10 GREEN; Phase 0 baseline deleted). **Phase 1 implemented:** `architecture/domain-ownership.toml` is the concept-level SSOT; xtask parses five roles fail-closed (CON-T11–T20 GREEN; Phase 1 baseline deleted). |
| Program | Architectural Consolidation Program (preserve architecture; eliminate parallel truths). |
| Slice | **Phase 0 complete.** **This slice: Phase 1 ownership law only** (who owns which concept; which roles may split). **Not** Phase 2+ consumer migrations, duplicate deletes, or product semantic rewrites of applicability / readiness / lineage. |
| Dual-suite | `xtask/tests/sdd_architectural_consolidation_target.rs` via `cargo test -p xtask` auto-discovery. Phase 0 CON-T01–T06 / T08–T09 **stay GREEN**. Phase 1 CON-T11–T20 **GREEN**. CON-T07 allowed the live non-ignored Phase 1 characterization file during the window; the file is **deleted** after GREEN (`INV-NO-SUPERSEDED-BASELINES`). **Do not** create `tests/sdd/` ([ADR 0004](../adr/0004-documentation-architecture.md) / `FORBID-TESTS-SDD`). Do not invent `test/sdd/*.ts`. Do not add a new root or xtask `[[test]]`. |
| ADR | **Accepted** [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md). **Accepted** [`docs/adr/0050-domain-ownership-model.md`](../adr/0050-domain-ownership-model.md) (`weeping-angel-adr-meta` `id = "0050"`). Do **not** mint `0003-*` or a colliding `0011-*`. Environment P0 (not a second program SSOT): [ADR 0051](../adr/0051-repository-environment.md). Next unused prefix after 0051 is **0052**. |
| Human SSOT | **This file** under `docs/specs/`. [`docs/sdd/architectural-consolidation-program.md`](../sdd/architectural-consolidation-program.md) is the SDD run pointer only. Do **not** add a second program spec. Generated traces stay in `.sdd/`. Guard **15** keeps the **existing** consolidation spec row. |
| Predecessor law | [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md) (ACP Phase 0 is **spec-law-only freeze**, a different program), [`docs/specs/structural-reconciliation.md`](structural-reconciliation.md), [`docs/specs/repository-integrity.md`](repository-integrity.md), ADRs 0004, 0009–0012, 0048, 0049. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) untouched. |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| Neighbors (must stay GREEN) | `sdd_documentation_layout` `CANONICAL_SPECS`, Guard **15** `architecture/spec-lifecycle.toml`, `cargo xtask guard` 01–15, `sdd_architectural_cleanup_target`, `sdd_structural_reconciliation_target`. |
| Collision fence | Do **not** treat architectural-cleanup Phase 0 (spec-law-only freeze) as this program. Phase 0 freeze remains **active** (`feature_expansion=restricted`). Phase 1 is `allowed_change_classes` **consolidation** + **consolidation-docs**. Do not invent `weeping-angel-catalog` / `weeping-angel-assurance-cli`. |
| Toolchain | Cargo workspace (seven `weeping-angel-*` libraries + unpublished `xtask` + package `weeping-angel` at `apps/cli`; [ADR 0051](../adr/0051-repository-environment.md)). **pnpm / `apps/docs` out of scope of this program.** |
| Inventory SSOT | `cargo xtask inventory` / `xtask/src/inventory.rs` / schema `weeping-angel/inventory/v1` / live snapshot [`docs/debt/current.md`](../debt/current.md). Do **not** invent a second counter (DUP-014). Phase 1 must not raise frozen expansion metrics (`INV-CONSOLIDATION-EXPANSION-RESTRICTED`). |
| `adr_needed` | **true** — concept-level role split vs crate-kind exclusivity; sibling domain-ownership SSOT; honesty-amend `INV-NO-SUPERSEDED-BASELINES` for the dual-suite window. |
| Protocol | Phase 0 **complete** (Accept ADR 0049). Phase 1 **complete** (Accept ADR 0050): spec + dual-suite + parser/TOML/invariants GREEN; Phase 1 baseline suite **deleted**. |

This document is the durable human SSOT for the Architectural Consolidation Program. **Phase 0** (§0–§10) is implemented law. **Phase 1** (§11) is implemented ownership law (who owns which concept; which roles may split). Do not fork a second program SSOT.

---

## 0. Program law

```text
One concept → one semantic owner → one canonical representation
  → one computation path → multiple projections.
```

Do **not** rewrite Weeping Angel. Preserve the existing architecture (Providers → Collectors → Canonical Evidence → Ledger(`current()` / `as_of(t)`) → Tests → Assessments → Applicability+Risk/ISMS → immutable AssessmentRun → Readiness/SoA/Explain → Framework Projection) while eliminating **parallel truths**: second public types, second persistence representations, second projection paths, second dual-suite conventions, duplicated helpers, compatibility aliases that mint a second SSOT.

Enforcement seat remains `cargo xtask guard` (CI already runs it on PRs). Do **not** add a 16th product-semantic `ProductLawCheck` unless a later phase names it. Do **not** add a second health CLI.

---

## 1. Collision fence

Phase 0 may change **only** (implement slice; Phase 0 spec-first was docs/ADR + neighbor index). Phase 1 allowed homes are in §11.0; Phase 0 freeze remains active.

| Concern | Home |
| --- | --- |
| This SSOT | `docs/specs/architectural-consolidation-program.md` |
| SDD pointer | `docs/sdd/architectural-consolidation-program.md` |
| Accepted ADR | `docs/adr/0049-architectural-consolidation-phase-0.md` (Phase 0). `docs/adr/0050-domain-ownership-model.md` (Phase 1). |
| Consolidation mode | `architecture/architecture.toml` `[program.architectural_consolidation]` + parse on `ArchitectureManifest` |
| Concept-level ownership | `architecture/domain-ownership.toml` (`weeping-angel/domain-ownership/v1`; five roles; Guard 01/04) |
| Freeze evaluation | Guard **01 / 02 / 03 / 04** and/or a new `[[invariant]]` with a predicate in `evaluate_invariant` — **not** Guard 16 |
| Frozen snapshot | `docs/debt/consolidation-baseline.json` + `docs/debt/consolidation-baseline.md` projected from `xtask/src/inventory.rs` |
| Live mechanical snapshot | `docs/debt/current.md` (unchanged role) |
| Backlog schema | `docs/debt/structural-duplication.toml` (`weeping-angel/structural-duplication/v2`) |
| Dual-suite | `xtask/tests/sdd_architectural_consolidation_target.rs` (Phase 0 and Phase 1 baselines deleted — §11.5) |
| Neighbor index | `architecture/spec-lifecycle.toml`, `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` |

| Do not touch | Owner |
| --- | --- |
| Phase 1+ **consumer migrations**; deleting DUP duplicate files | Phase 2+ (Phase 1 is ownership law only — §11) |
| Rewriting assurance / collector / catalog / IR | Product programs |
| New frameworks, collectors, ISMS engines, SARIF/report formats, product scanners | Never this slice |
| Hypothetical packages `weeping-angel-catalog`, `weeping-angel-assurance-cli` | Forbidden (Guard 03) |
| `tests/sdd/`, `test/sdd/*.ts`, new root `[[test]]` | Forbidden |
| Mass ADR renumber; ignore-baseline mass delete | Hygiene / later ACP |
| pnpm / `apps/docs` | Out of scope |
| Replacing `cargo xtask guard` or adding a second health CLI | Forbidden |
| Treating architectural-cleanup Phase 0 freeze as this freeze | Different program (spec-law-only; not machine-readable) |

---

## 2. Problem / user-visible goal

**Found-case (pre-implement).** Before Phase 0, architecture-as-law (`cargo xtask guard` 01–15 pass), a mechanical inventory (`docs/debt/current.md`), and a Structural Reconciliation Phase 2 duplication map (`docs/debt/structural-duplication.toml` v1) existed. None of those surfaces froze feature expansion or gave later consolidation phases a machine-readable starting line:

1. `architecture/architecture.toml` is `[policy]` + `[ownership.*]` only. There is no `[program.architectural_consolidation]`. `load_architecture_manifest` reads `schema`, `[policy]`, and `[ownership]` and **ignores extra tables**, so a paper TOML table would not be a gate.
2. There is **no** `docs/debt/consolidation-baseline.json` or `.md`. `current.md` is the **live** inventory projection (`weeping-angel/inventory/v1`) and must stay that. A frozen Phase 0 baseline does not exist.
3. `structural-duplication.toml` is `schema = "weeping-angel/structural-duplication/v1"`, `program = "structural-reconciliation"`, `phase = 2`, with 17 `[[duplication]]` rows (DUP-001..017). Statuses are `candidate | confirmed | migrating | resolved | false-positive`. Rows have `id`, `concept`, `canonical_owner`, `duplicates`, `status`, `action`, `debt_id`, `consumers`, `migration`, `guard`, `evidence`. They are **missing** `severity`, `canonical_symbol`, `migration_state`, `removal_blockers`, `public_api_impact`, `serialization_impact`, `tests`. Status `resolved` can be set without the four-part close law. **No Rust parser** loads this file (grep of `*.rs` is empty).
4. Architectural-cleanup Phase 0 is **out-of-scope law + review bar**, not a parsed program table. Contributors can still land a new public domain type, a second persistence representation, another `[[test]]`, a duplicated helper, or a compatibility alias without a fail-closed freeze gate.

**User-visible goal (Phase 0):** before any semantic migrations, the repo must (a) declare consolidation mode as architecture policy, (b) freeze a machine-readable snapshot of structural debt, and (c) promote `structural-duplication.toml` into a program backlog whose rows cannot silently close.

```text
Phase 0.1  [program.architectural_consolidation] parsed + fail-closed
Phase 0.2  docs/debt/consolidation-baseline.{json,md} from inventory
Phase 0.3  structural-duplication.toml v2 backlog schema + close law
        ↓
PRs cannot expand parallel truths while status=active / feature_expansion=restricted
```

Definition of done for Phase 0: *a contributor cannot merge a second SSOT / new public domain type / extra `[[test]]` / extra schema tree / extra duplicated helper without `cargo xtask guard` failing, and every DUP row is a typed backlog item with a canonical owner path.*

---

## 3. Compatibility / dependencies

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| Health command | `cargo xtask guard` | Preserved. Freeze evaluation folds into existing 01–04 / invariants. No Guard **16**. No second health CLI. |
| Inventory | `cargo xtask inventory` | Preserved. `InventoryReport` projects live `current.md` and print-only `--consolidation-baseline` / `--consolidation-baseline-markdown`. Frozen files are Guard 04, not rewritten on every inventory run. |
| Architecture loader | `xtask/src/architecture.rs` `load_architecture_manifest` | Today ignores unknown tables. Must **parse and require** `[program.architectural_consolidation]` (fail-closed if missing/malformed). |
| `ArchitectureManifest` | `schema` + `policy` + `ownership` | Additive field for the program table. Guard 01 stays “manifest loaded”; missing program table must not look like a pass. |
| Duplication map | `docs/debt/structural-duplication.toml` | Schema bump to v2; 17 rows retained; statuses remapped **without** silent `verified`/`removed`. |
| Debt register | `docs/debt/register.toml` | Do not reopen resolved `DEBT-GUARD-*`. Optional new finding only if a live exemption is unavoidable (prefer none). |
| Forbidden patterns | `architecture/forbidden-patterns.toml` | Keep `FORBID-TESTS-SDD` and hypothetical packages. May add data-driven patterns for extra schema trees / extra crates; not a new grep crate. |
| Dual-suite discovery | `xtask/tests/*.rs` | Auto-discovered. **Never** root `[[test]]` for this program. |
| `INV-NO-SUPERSEDED-BASELINES` | Guard 13 / 04 | Phase 0 baseline suite **deleted** (do not `#[ignore]`). Phase 1 **honesty-amends** the predicate for the dual-suite window (§11.4.5 / [ADR 0050](../adr/0050-domain-ownership-model.md)); leftover after GREEN still fail-closes. |
| Docs layout / lifecycle | `CANONICAL_SPECS`, `spec-lifecycle.toml` | This path listed; `state = "active"`; `ownership = ["repository_guard"]`. |
| `ASSURANCE_IR_SCHEMA` | `assurance-ir/v1` | Untouched. |
| Maintainability budget | `architecture/maintainability.toml` | Not a second inventory. Freeze uses inventory + baseline comparison. |

---

## 4. Current behavior (baseline — GREEN on CURRENT code before Phase 0 product)

§4 is the **pre-implement absence / incomplete-schema characterization** (found-case record). It is **not** current law. The executable baseline suite was deleted after target GREEN (`INV-NO-SUPERSEDED-BASELINES`). Current plane: header Status + §5 (implemented).

### 4.1 Consolidation mode (Phase 0.1 found-case)

- `architecture/architecture.toml` `schema = "weeping-angel/architecture/v1"` contains `[policy]` (`ownership_kinds`, `required_concepts`) and `[ownership.*]` rows only (catalog, framework_compilation, readiness_projection, temporal_evidence_selection, assessment_lineage, evidence_persistence, assurance_cli, repository_guard).
- There is **no** `[program]` / `[program.architectural_consolidation]` table.
- `ArchitectureManifest` fields are `schema`, `policy`, `ownership`. No consolidation-program struct.
- `load_architecture_manifest` (`xtask/src/architecture.rs`) requires schema + `[policy]` + `[ownership]`, then returns. Extra TOML tables are **not** read and **not** validated. A paper `[program.architectural_consolidation]` would still yield Guard **01** pass.
- Guard **01** (`ArchitectureManifestCheck`): pass if the manifest loaded. Guard **02**: ownership kinds/paths/live crates. Guard **03**: forbidden patterns. Guard **04**: known `[[invariant]]` predicates only (`INV-OWNERSHIP-LIVE-CRATES`, `INV-NO-HYPOTHETICAL-PACKAGES`, `INV-DEBT-RESOLVED-HAS-PROOF`, `INV-INVARIANTS-EVALUATED`, `INV-ADR-NAMESPACE-UNIQUE`, `INV-NO-SUPERSEDED-BASELINES`). Unknown invariant ids fail closed — there is **no** consolidation-mode invariant.

### 4.2 Consolidation baseline snapshot (Phase 0.2 found-case)

- `docs/debt/consolidation-baseline.json` is **not a file**.
- `docs/debt/consolidation-baseline.md` is **not a file**.
- Live mechanical snapshot is [`docs/debt/current.md`](../debt/current.md), generated/checked by `cargo xtask inventory --markdown` / `--check`.
- Inventory schema `weeping-angel/inventory/v1` `counts` (CURRENT, 2026-08-20):

| Metric | Count |
| --- | --- |
| Root `[[test]]` binaries | 45 |
| `tests/*.rs` (auto-discovered) | 16 |
| `tests/contracts/*.rs` | 43 |
| ignored tests (`#[ignore`) | 5 |
| `.unwrap()` in `*.rs` | 1239 |
| `.expect(` in `*.rs` | 635 |
| Files defining `fn require_needles` | 18 |
| ADR markdown files | 48 |
| Framework packs | 2 |
| `*.schema.json` files | 3 |

- Inventory does **not** project: workspace crate/module lists, public symbol / `pub use` / public struct/enum counts, duplicate helper definitions beyond `require_needles`, duplicate type names, architecture ownership rows, debt-register row counts, spec-file counts, schema **locations** (only a count).
- `InventoryAbsences` keys are `inventory_module`, `debt_current_md`, `structural_reconciliation_spec` only — no consolidation-baseline absence key.
- Walkers: `inventory.rs` `walk_included` vs `model.rs` `SKIP_DIR_NAMES` (DUP-014). Phase 0 **reuses inventory**; it must not add a third walker.

### 4.3 Structural duplication backlog (Phase 0.3 found-case)

File: `docs/debt/structural-duplication.toml`

```text
schema = "weeping-angel/structural-duplication/v1"
program = "structural-reconciliation"
phase = 2
[[duplication]] × 17  (DUP-001 … DUP-017)
```

| Present | Absent (required by this program) |
| --- | --- |
| `id`, `concept`, `canonical_owner`, `duplicates`, `status`, `action`, `debt_id`, `consumers`, `migration`, `guard`, `evidence` | `severity`, `canonical_symbol`, `migration_state`, `removal_blockers`, `public_api_impact`, `serialization_impact`, `tests` |

Closed status set today: `candidate | confirmed | migrating | resolved | false-positive`.

CURRENT row statuses (honesty; do not silently upgrade to `verified`):

| Status | Rows |
| --- | --- |
| `candidate` | DUP-006, DUP-015, DUP-016, DUP-017 |
| `confirmed` | DUP-002, DUP-003, DUP-007, DUP-011, DUP-014 |
| `migrating` | DUP-004, DUP-005 |
| `resolved` | DUP-001, DUP-008, DUP-010, DUP-013 |
| `false-positive` | DUP-009, DUP-012 |

`resolved` rows still list duplicate paths and/or open debt (e.g. DUP-001 / `DEBT-SCHEMA-DUP`). Close law is **not** enforced. No xtask type loads the file.

### 4.4 Baseline suite IDs (must be GREEN on CURRENT pre-implement code)

| ID | Characterization |
| --- | --- |
| CON-B01 | `architecture/architecture.toml` has no `[program.architectural_consolidation]` (and no `[program]` table) |
| CON-B02 | `ArchitectureManifest` / `load_architecture_manifest` have no consolidation-program field; extra tables are ignored |
| CON-B03 | `docs/debt/consolidation-baseline.json` and `docs/debt/consolidation-baseline.md` are not files |
| CON-B04 | `structural-duplication.toml` schema is `…/v1`; at least one row uses `migrating` or `resolved` or `false-positive`; required Phase 0.3 fields are missing from row text |
| CON-B05 | Inventory JSON schema is `weeping-angel/inventory/v1` without consolidation-baseline projection keys (`workspace_crates`, `pub_use_count`, `public_structs`, `public_enums`, `spec_count`, `debt_rows`, `schema_locations`, …) |
| CON-B06 | Live `cargo xtask guard` checks 01–15 are still `pass` (honesty hinge: product plane green; Phase 0 gates absent) |

Target RED must fail **because CON-T\* obligations are unmet**, not because Guard 05–12 / scanners / catalog product code regress.

---

## 5. Desired behavior (target — RED on CURRENT, GREEN after Phase 0 implement)

**Implemented.** CON-T01–T10 GREEN on the healthy tree. Field-level law below is the live Phase 0 contract.

### 5.1 Phase 0.1 — consolidation mode

Add to `architecture/architecture.toml` (exact key names may match; behavior must):

```toml
[program.architectural_consolidation]
status = "active"
feature_expansion = "restricted"
allowed_change_classes = [
  "bug-fix",
  "security-fix",
  "consolidation",
  "non-semantic-collector",
  "consolidation-docs",
]
forbidden_change_classes = [
  "new-public-domain-type",
  "new-persistence-representation",
  "new-projection-path",
  "new-root-test-binary",
  "new-duplicated-helper",
  "new-compatibility-alias",
  "second-ssot",
]
```

| Key | Closed set / law |
| --- | --- |
| `status` | `active` (this program) or `inactive` (only a later ADR may flip). Missing → fail closed. |
| `feature_expansion` | `restricted` while `status = "active"`. `unrestricted` is illegal until a later phase ADR opens expansion. |
| `allowed_change_classes` | Non-empty; must include the five allowed classes above (bug/security, consolidation, evidence collectors that do **not** change core semantics, consolidation docs). |
| `forbidden_change_classes` | Non-empty; must include the seven forbidden classes above. |

**Loader:** `load_architecture_manifest` **must** parse `[program.architectural_consolidation]`. Missing table, missing keys, empty arrays, or illegal enum values → `Err` (Guard 01 fail) **or** a dedicated invariant fail (Guard 04) — never ignore.

**`ArchitectureManifest`** gains an additive `consolidation: ConsolidationProgram` (name flexible; field required). Tests may read it from `RepositoryModel.architecture`.

**PR blocking (machine bar while restricted):** using the **same** inventory walker plus the frozen consolidation baseline (§5.2), `cargo xtask guard` fails closed when live counts **increase** vs the frozen snapshot for expansion metrics:

| Expansion metric | Why |
| --- | --- |
| `root_test_binaries` | another `[[test]]` |
| `schema_json_files` / `schema_locations` | another persistence/schema SSOT |
| workspace crate / package count | new crate / second owner seat |
| `public_structs` / `public_enums` | new public domain types |
| `pub_use_count` | another compatibility alias / re-export SSOT (net increase) |
| `require_needles_fns` (and documented duplicate-helper names) | new duplicated helper |
| new `docs/specs/*.md` that is **not** this program / lifecycle-registered consolidation docs | second SSOT / unindexed spec (Guard 15 already covers unlisted specs) |

Decreases (consolidation) are **allowed**. Non-countable classes (new projection **path** inside an existing crate, new persistence representation that is not a `*.schema.json`) are additionally fail-closed via:

1. Forbidden-pattern / ownership kind violations already in Guards 02–03.
2. Backlog rows: landing a new parallel type without a `[[duplication]]` row in `candidate|confirmed` is a Phase 1+ concern; Phase 0 requires the **schema** and freeze counts, not full semantic alias detection.
3. Human + dual-suite needles for “do not add `tests/sdd/` / hypothetical packages / extra framework packs” (already Guard 03).

Allowed while restricted: bug/security fixes, consolidation (migrating consumers onto a canonical owner), non-semantic evidence collectors, consolidation docs (this spec, ADR 0049, baseline artifacts, backlog schema).

**Enforcement seat (normative):** parse on `ArchitectureManifest`; fail-closed through **Guard 01** (manifest invalid without the table) **and** Guard **04** invariant(s), for example:

| Invariant id | Predicate sketch |
| --- | --- |
| `INV-CONSOLIDATION-MODE-ACTIVE` | Table present; `status=active`; `feature_expansion=restricted`; allowed/forbidden class sets contain the required members |
| `INV-CONSOLIDATION-BASELINE-PRESENT` | Both baseline artifacts exist; JSON schema + required keys; shared inventory counts match `InventoryReport` for keys that `current.md` already owns |
| `INV-CONSOLIDATION-EXPANSION-RESTRICTED` | Live inventory expansion metrics ≤ frozen baseline (or equal); increases fail with the metric name |
| `INV-STRUCTURAL-DUPLICATION-BACKLOG` | File parses as v2; every row has required fields; status ∈ new closed set; no `migrating`/`resolved`/`false-positive`; no row `verified`/`removed` without close-law evidence |

Unknown invariant ids already fail closed — implement **must** add predicates. Prefer this over ProductLawCheck 16.

Do **not** add `cargo xtask consolidation` or another health binary.

### 5.2 Phase 0.2 — snapshot

Generate and commit:

- `docs/debt/consolidation-baseline.json`
- `docs/debt/consolidation-baseline.md`

**Source of counts:** `xtask/src/inventory.rs` `InventoryReport::collect` (extend in place). `docs/debt/current.md` remains the **live** mechanical snapshot (`inventory --markdown` / `--check`). The consolidation baseline is the **frozen Phase 0** projection used by the expansion invariant.

Do **not** invent a second filesystem walker. If extra metrics need another file class, extend `walk_included` / count helpers. Document any new exclusion in JSON `exclusions` (today: `target/`, `target-*`, `node_modules/`).

JSON schema string: `weeping-angel/consolidation-baseline/v1`.

Required top-level:

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | string | Exactly `weeping-angel/consolidation-baseline/v1` |
| `program` | string | `architectural-consolidation` |
| `phase` | integer | `0` |
| `source` | string | `weeping-angel/inventory/v1` |
| `exclusions` | string array | Must include inventory exclusions |
| `inventory_counts` | object | **Same keys and values** as live inventory `counts` at generation time (`root_test_binaries`, `tests_rs_autodiscovered`, `tests_contracts_rs`, `ignored_test_attrs`, `unwrap_calls`, `expect_calls`, `unwrap_plus_expect`, `require_needles_fns`, `require_needles_calls`, `adr_markdown_files`, `catalog_test_toml`, `framework_packs`, `schema_json_files`) |
| `extended` | object | Phase 0 extra metrics (below) |
| `architecture_ownership` | array | Concept / crate / kind / paths from `architecture.toml` `[ownership.*]` |
| `schema_locations` | string array | Repo-relative `*.schema.json` paths under inclusion rule |
| `adr_count` | integer | Equals `inventory_counts.adr_markdown_files` |
| `spec_count` | integer | `docs/specs/*.md` file count |
| `debt_rows` | integer | `[[finding]]` count in `docs/debt/register.toml` |

Required `extended` keys (integers ≥ 0 unless noted):

| Key | Counts |
| --- | --- |
| `workspace_crates` | Workspace members + root package as documented in collect() (do not invent hypothetical crates) |
| `rust_modules` | `*.rs` files under inclusion (or crate `src/**` + `xtask/src/**`; document which) |
| `public_symbols` | `pub fn` / `pub struct` / `pub enum` / `pub trait` / `pub type` / `pub const` occurrences in `*.rs` (heuristic line scan is acceptable if documented) |
| `pub_use_count` | `pub use` occurrences in `*.rs` |
| `public_structs` | `pub struct` occurrences |
| `public_enums` | `pub enum` occurrences |
| `duplicate_helper_definitions` | At least `require_needles_fns`; may include other same-name `fn` defs listed in the markdown |
| `duplicate_type_names` | Count of type identifiers (`struct`/`enum` name) that appear in more than one `*.rs` file (characterization, not a uniqueness ban) |

Markdown `consolidation-baseline.md` must:

- State it is the **frozen Phase 0** consolidation snapshot, **not** the live `current.md`.
- Table the same `inventory_counts` and `extended` metrics.
- List `schema_locations` and ownership rows (compact table).
- Carry a stable marker analogous to `<!-- weeping-angel-inventory-stable -->` (e.g. `<!-- weeping-angel-consolidation-baseline-stable -->`).

`cargo xtask inventory --check` continues to verify **`current.md`**. Consolidation baseline presence/schema/monotonicity is a **guard invariant**, not a second CLI. Print-only flags `--consolidation-baseline` and `--consolidation-baseline-markdown` project the same walker; they do **not** rewrite committed frozen files and are **not** a new subcommand. The frozen JSON is the expansion reference (live `collect()` is compared to it).

Honesty: shared `inventory_counts` in the frozen JSON **must equal** `current.md` / `InventoryReport` **at the moment the baseline is first committed**. Later live `current.md` may move (unwrap down, etc.); expansion metrics must not **rise**.

### 5.3 Phase 0.3 — structural-duplication.toml is the program backlog

Bump schema to:

```toml
schema = "weeping-angel/structural-duplication/v2"
program = "architectural-consolidation"
phase = 0
```

Keep all 17 `[[duplication]]` ids DUP-001..017. **Do not delete rows. Do not delete duplicate source files in Phase 0.**

Every row **must** have:

| Field | Law |
| --- | --- |
| `id` | Unique `DUP-NNN` |
| `concept` | Non-empty |
| `severity` | `p0` \| `p1` \| `p2` \| `info` (closed set; implement documents mapping from current `action`/debt) |
| `canonical_owner` | Non-empty path/crate/symbol seat |
| `canonical_symbol` | Non-empty; may be `unknown` **only** while `status = candidate` |
| `duplicates` | Array (may be empty only if status is `removed` or `verified` **and** close law holds) |
| `migration_state` | Non-empty prose or enum-equivalent string describing consumer migration |
| `removal_blockers` | Array of strings (empty array allowed only if nothing blocks removal) |
| `public_api_impact` | Non-empty (`none` \| `additive` \| `breaking` \| `unknown` plus optional prose field) |
| `serialization_impact` | Non-empty (`none` \| `format-change` \| `unknown` plus optional prose) |
| `tests` | Array of test names/paths that pin the concept (may be empty only for `candidate`) |
| `guard` | Non-empty (existing field; keep) |
| `status` | New closed set (below) |

Retain useful existing fields (`action`, `debt_id`, `consumers`, `migration`, `evidence`) as optional extras; they do **not** replace the required set.

**Status closed set:**

```text
candidate | confirmed | canonicalized | consumers-migrating
  | compatibility-only | removed | verified
```

**Mandatory mapping from v1 (no silent close):**

| v1 status | v2 status | Rule |
| --- | --- | --- |
| `candidate` | `candidate` | Unchanged |
| `confirmed` | `confirmed` | Unchanged |
| `migrating` | `consumers-migrating` | Never `verified` |
| `resolved` | `canonicalized` **or** `consumers-migrating` **or** `compatibility-only` | **Never** auto-map to `removed` or `verified`. If duplicate paths still exist or debt is open, must **not** be `canonicalized` unless canonical owner exists **and** duplicates are documented as remaining consumers. Prefer `consumers-migrating` when duplicates listed are still on disk. |
| `false-positive` | `candidate` (reconfirm) **or** `canonicalized` with `severity = info` and evidence that the “duplicate” is an intentional projection | **Never** `verified` without close law |

**Close law — a row may become `removed` or `verified` only when all hold:**

1. Canonical owner exists (`canonical_owner` + `canonical_symbol` not `unknown`).
2. All consumers use the canonical owner (`consumers-migrating` complete; no remaining call sites on the old path except documented `compatibility-only` shims with a removal date/blocker).
3. Old path is removed **or** status is `compatibility-only` (shim still present — **not** `verified`).
4. A regression `guard` exists and is cited (`tests` non-empty **or** a live Guard / forbidden-pattern id).

`verified` additionally requires the guard to be **executable** on the healthy tree (named `xtask`/`contracts` test or `repository_guard` check). Parser/invariant fail-closed if `status` is `verified` or `removed` while `canonical_symbol = "unknown"`, `tests` empty **and** `guard` empty, or `duplicates` still list tracked source paths for `removed`.

xtask **must** parse this file (module may live next to `debt.rs` or `inventory.rs` — one loader). Fail closed on schema ≠ v2, missing required fields, illegal status, duplicate `id`.

### 5.4 Dual-suite target IDs

| ID | Obligation |
| --- | --- |
| CON-T01 | `[program.architectural_consolidation]` parsed into `ArchitectureManifest`; `status=active`; `feature_expansion=restricted`; allowed/forbidden class sets present |
| CON-T02 | Missing/malformed program table fails Guard 01 and/or 04 (fixture); paper extra table without parse is **not** a pass |
| CON-T03 | `docs/debt/consolidation-baseline.json` and `.md` exist; JSON schema + required keys from §5.2 |
| CON-T04 | Frozen `inventory_counts` match `InventoryReport` shared keys **as of commit**; `current.md` still live `--check` |
| CON-T05 | `structural-duplication.toml` is v2; every row has the required field set; status ∈ new closed set; no `migrating`/`resolved`/`false-positive` |
| CON-T06 | No row is `verified` or `removed` unless close law holds; v1 `resolved` did not silently become `verified` |
| CON-T07 | Suites live under `xtask/tests/sdd_architectural_consolidation_*.rs`; no `tests/sdd/`; no new root `[[test]]` |
| CON-T08 | This spec in `CANONICAL_SPECS` + `spec-lifecycle.toml`; ADR 0049 file exists with meta `id = "0049"` |
| CON-T09 | Expansion invariant: increasing a frozen expansion metric (e.g. `root_test_binaries` or `schema_json_files`) fails Guard 04 (fixture) |
| CON-T10 | Neighbors: `sdd_architectural_cleanup_target`, `sdd_structural_reconciliation_target`, `cargo xtask guard` 01–15 still pass on the healthy tree |

Target RED on CURRENT must fail CON-T01–T06/T09 because those artifacts/schema/enforcement are **absent**, not because of unrelated product assertions.

### 5.5 Dual-suite and verify commands

```text
cargo test -p xtask --test sdd_architectural_consolidation_target
cargo xtask guard
cargo xtask inventory --check
cargo xtask inventory --consolidation-baseline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features demo -- -D warnings
cargo test --workspace --features demo --all-targets
```

Protocol (complete):

1. Spec first (this file) + Draft ADR 0049.
2. Baseline GREEN on CURRENT (CON-B01–B06).
3. Target RED on CURRENT (CON-T01–T10) for the right reason.
4. Implement 0.1–0.3 until target GREEN; baseline FAIL on the new tree.
5. **Deleted** the baseline suite (did not `#[ignore]`).
6. **Accepted** ADR 0049.

---

## 6. Acceptance criteria (testable)

- [x] Dual-suite lives in `xtask/tests/sdd_architectural_consolidation_target.rs` (not `tests/sdd/`, not `test/sdd/*.ts`, not a new root `[[test]]`). Baseline suite deleted after GREEN.
- [x] Baseline CON-B01–B06 PASS on the pre-implement tree (missing program table, ignored extra TOML, missing consolidation-baseline artifacts, v1 duplication schema/old statuses, inventory without extended projection), then the suite was **deleted**.
- [x] Target CON-T01–T10 FAIL on CURRENT **because** those three artifacts/schema/enforcement are absent (not unrelated product code), then PASS after implement.
- [x] `architecture.toml` `[program.architectural_consolidation]` `status=active`, `feature_expansion=restricted`, allowed/forbidden classes as §5.1; `load_architecture_manifest` fails closed if missing.
- [x] Guard 01 and 04 evaluate the program table; expansion increases vs frozen baseline fail closed; no Guard 16; no second health CLI.
- [x] `docs/debt/consolidation-baseline.json` + `.md` exist; schema `weeping-angel/consolidation-baseline/v1`; coverage in §5.2; shared counts projected from existing inventory; `docs/debt/current.md` remains live.
- [x] `structural-duplication.toml` schema v2; every row has `id`, `concept`, `severity`, `canonical_owner`, `canonical_symbol`, `duplicates`, `migration_state`, `removal_blockers`, `public_api_impact`, `serialization_impact`, `tests`, `guard`, `status`; statuses are the new closed set; v1 `migrating`/`resolved`/`false-positive` mapped without silent `verified`/`removed`.
- [x] Close law: no `verified`/`removed` until canonical owner exists, consumers migrated, old path removed (or `compatibility-only`), and a regression guard exists.
- [x] After target GREEN, baseline suite **deleted** (`INV-NO-SUPERSEDED-BASELINES`).
- [x] Neighbors stay green: `CANONICAL_SPECS`, Guard 15 lifecycle row, ACP target, SR target, `cargo xtask guard` 01–15.
- [x] ADR 0049 **Accepted** after target GREEN.
- [x] Phase 0 freeze list in §1 is not violated (no Phases 1+ migrations, no new frameworks/collectors/ISMS, no hypothetical crates).

---

## 7. Out of scope

1. Phases 1+ consumer migrations and deleting DUP duplicate source trees
2. Rewriting assurance, collector, catalog, or IR crates
3. New frameworks, collectors, ISMS / risk engines, SARIF/report formats, product scanners
4. Hypothetical packages `weeping-angel-catalog` / `weeping-angel-assurance-cli`
5. `tests/sdd/`, `test/sdd/*.ts`, new root `[[test]]` for this program
6. Mass ADR renumber; ignore-baseline mass delete
7. pnpm / `apps/docs`
8. A 16th `ProductLawCheck` / second health CLI
9. A second inventory walker (DUP-014)
10. Changing `ASSURANCE_IR_SCHEMA` or catalog identities
11. Treating architectural-cleanup Phase 0 as this program’s freeze
12. Reopening resolved `DEBT-GUARD-05`…`15` as skip hatches
13. Silently marking DUP rows `verified` because v1 said `resolved`
14. Expanding feature work (new public domain types, second persistence, second projection path) under the freeze

---

## 8. Risks

- A paper `[program.architectural_consolidation]` without loader changes looks like freeze but is not a gate (this is CURRENT; CON-T02 exists to prevent it).
- Extending inventory with public-symbol heuristics can flake on comments/strings; document the scan as line-based and pin counts in the frozen JSON.
- Monotonic expansion checks can fail legitimate bug-fixes that add a helper; allowed classes do not waive count increases — prefer consolidating helpers or documenting an exception only via a later ADR / debt row with proof (fail-closed default).
- Mapping v1 `resolved` → `verified` would silently close rows that still have duplicates on disk.
- Adding Guard 16 or a new xtask subcommand forks the health plane (forbidden).
- `#[ignore]` on the baseline suite after GREEN violates `INV-NO-SUPERSEDED-BASELINES`.
- Confusing this freeze with architectural-cleanup Phase 0 (review-bar only) lets expansion land again.
- Dual-suite assertions that needle unrelated product modules make target RED for the wrong reason.
- Generating consolidation-baseline with a different exclusion set than inventory desyncs `current.md` from the freeze (DUP-014).
- Neighbor Guard 15 goes red if this spec file is not in `spec-lifecycle.toml`.

---

## 9. remaining_backlog (not Phase 0)

1. **Phase 1 (implemented):** canonical domain-ownership SSOT + parser + Guard 01/04 fold-in. **Not** consumer migration.
2. Phase 2+ migrate consumers onto canonical owners (DUP-002 helpers, DUP-004 snapshot shapes, DUP-011 readiness constructors, …)
3. Delete duplicate files once close law holds (e.g. remaining schema copies)
4. Merge inventory and `RepositoryModel` walks (DUP-014) without a third walker
5. Semantic detection of new projection paths beyond count/ownership heuristics
6. Flipping `feature_expansion` to unrestricted (requires a later ADR)
7. Architectural-cleanup Phases 2–28 product work (separate program)

---

## 10. Related

- Accepted decision: [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md)
- Accepted decision: [`docs/adr/0050-domain-ownership-model.md`](../adr/0050-domain-ownership-model.md)
- Pointer: [`docs/sdd/architectural-consolidation-program.md`](../sdd/architectural-consolidation-program.md)
- [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md) — **different** Phase 0
- [`docs/specs/structural-reconciliation.md`](structural-reconciliation.md), [ADR 0048](../adr/0048-structural-reconciliation.md)
- [`docs/specs/repository-integrity.md`](repository-integrity.md), ADRs [0004](../adr/0004-documentation-architecture.md), [0009](../adr/0009-repository-health-gate.md), [0010](../adr/0010-architecture-as-law.md), [0011](../adr/0011-repository-guard-governance.md)
- [`docs/debt/current.md`](../debt/current.md), [`docs/debt/structural-duplication.toml`](../debt/structural-duplication.toml)

---

## 11. Phase 1 — canonical domain ownership (implemented)

Program law (unchanged):

```text
One concept → one semantic owner → one canonical representation
  → one computation path → multiple projections.
```

Phase 1 **names** the owner seats. It does **not** migrate consumers onto those seats (Phase 2+). Preserve existing crate architecture. Phase 0 freeze stays **active**.

```text
architecture.toml [ownership.*]     crate-level kinds (exclusive|facade|projection|adapter|shared-primitive)
        ↓ expand, do not replace
architecture/domain-ownership.toml  concept-level five-role SSOT (this slice)
```

### 11.0 Collision fence

Phase 1 **implement** (landed) may change **only**:

| Concern | Home |
| --- | --- |
| This SSOT (Phase 1 section) | `docs/specs/architectural-consolidation-program.md` — **same** Guard 15 / `CANONICAL_SPECS` row |
| SDD pointer / run note | `docs/sdd/architectural-consolidation-program.md` (pointer only; not a second spec) |
| Accepted ADR | `docs/adr/0050-domain-ownership-model.md` |
| Semantic ownership SSOT | `architecture/domain-ownership.toml` (new file; fail-closed if missing/malformed) |
| Parser | `xtask/src/architecture.rs` (`load_domain_ownership` or equivalent) + `RepositoryModel` / `ArchitectureManifest` additive field |
| Enforcement | Guard **01** (manifest load) **and/or** Guard **04** `[[invariant]]` (`INV-DOMAIN-OWNERSHIP-PRESENT`, `INV-DOMAIN-OWNERSHIP-ROLES` or equivalent). **Not** Guard **16**. **Not** `cargo xtask consolidation`. |
| Dual-suite | `xtask/tests/sdd_architectural_consolidation_{baseline,target}.rs` only |
| `INV-NO-SUPERSEDED-BASELINES` | Honesty-amend predicate for the dual-suite **window**; after GREEN, leftovers still fail-closed |

| Do not touch | Owner |
| --- | --- |
| Consumer migrations; deleting DUP duplicate files | Phase 2+ |
| Changing applicability / readiness / lineage **product** semantics (`ApplicabilitySnapshot`, `project_readiness`, `replay_assessment`, …) | Product programs |
| Collapsing five roles into crate `kind = exclusive` | Forbidden (fake exclusivity) |
| Hypothetical packages `weeping-angel-catalog`, `weeping-angel-assurance-cli` | Forbidden (`FORBID-HYPOTHETICAL-*`) |
| Sixth role key `persistence_owner` | Map to `storage_owner` |
| `tests/sdd/`, `test/sdd/*.ts`, new root/xtask `[[test]]`, Guard 16, second health CLI | Forbidden |
| Second program spec / second Guard 15 row | Forbidden |
| pnpm / `apps/docs` | Out of scope |
| Raising frozen expansion metrics (`pub struct`/`pub enum`/`pub use`/`[[test]]`/crates) | Freeze (`INV-CONSOLIDATION-EXPANSION-RESTRICTED`). Parser types stay crate-private unless a later ADR rebases the freeze. |

### 11.1 Problem / user-visible goal

**Found-case (CURRENT, 2026-08-20).** Phase 0 made consolidation mode machine-readable. Ownership is still **crate-level**:

1. `architecture/domain-ownership.toml` **does not exist**.
2. `architecture.toml` `[ownership.*]` rows are crate + `kind` + `paths` for `catalog`, `framework_compilation`, `readiness_projection`, `temporal_evidence_selection`, `assessment_lineage`, `evidence_persistence`, `assurance_cli`, `repository_guard`. Kinds are `exclusive | facade | projection | adapter | shared-primitive`. There are **no** `semantic_owner` / `storage_owner` / `projection_owner` / `evaluation_primitive_owner` / `adapter_owner` keys.
3. `load_architecture_manifest` (`xtask/src/architecture.rs`) parses `schema` + `[policy]` + `[ownership]` + `[program.architectural_consolidation]` from **`architecture/architecture.toml` only**. Extra files under `architecture/` are not loaded. A paper `domain-ownership.toml` would not be a gate (same class of bug Phase 0 fixed for extra tables).
4. `architecture.toml` `[ownership.temporal_evidence_selection]` is `kind = exclusive` on `weeping-angel-assurance` / `src/temporal.rs` (timeline / temporal-diff **projection**). The evaluation primitive `select_latest_as_of` still lives in `crates/weeping-angel-control-test/src/temporal.rs`. Forcing exclusivity here is a **lie**; domain-ownership must record the **split**.
5. There is **no** `INV-DOMAIN-OWNERSHIP*` row. Guard IDs remain **01–15** (`KNOWN_CHECK_IDS`; implemented 01–04, 13–14, 15; `ProductLawCheck` 05–12).
6. `INV-NO-SUPERSEDED-BASELINES` (`eval_no_superseded_baselines`) fail-closes on any `RepositoryModel.filesystem` path ending in `.baseline.rs` or `_baseline.rs`. `RepositoryModel::load` indexes `architecture`, `docs/adr`, `docs/specs`, `docs/debt`, `frameworks`, `catalog`, `crates`, `src`, `tests` — **not** `xtask/`. CON-T07 **additionally** asserts `xtask/tests/sdd_architectural_consolidation_baseline.rs` is deleted. Recreating that file **without** amending CON-T07 (and, if the walk grows to include `xtask/`, the invariant) REDS the dual-suite / Guard 04 on CURRENT.

**User-visible goal (Phase 1):** later SSOT phases have a named owner per concept and a **required** distinction of roles that are **allowed to be split**. Contributors cannot treat crate `kind = exclusive` as “this crate owns the whole concept.”

```text
Phase 1.1  architecture/domain-ownership.toml (five roles; seeded concepts)
Phase 1.2  xtask parses it fail-closed (missing/malformed → Guard 01 and/or 04)
Phase 1.3  honesty-amend INV-NO-SUPERSEDED-BASELINES for the dual-suite window
        ↓
Ownership law is executable; consumers are not migrated
```

Definition of done for Phase 1: *a paper `domain-ownership.toml` is not a pass; seeded concepts cite live crates/modules/symbols; temporal evaluation vs temporal evidence storage/selection is recorded as a split; no hypothetical crates; dual-suite GREEN then baseline **deleted** (not `#[ignore]`).*

### 11.2 Compatibility / dependencies

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| Crate ownership | `architecture.toml` `[ownership.*]` | **Preserved.** Domain-ownership **expands** it. Do not delete crate-kind rows. |
| Architecture loader | `load_architecture_manifest` | Must **also** parse `architecture/domain-ownership.toml` (same module; sibling file). Missing/malformed → `Err` (Guard 01) **or** dedicated invariant fail (Guard 04) — never ignore. |
| `ArchitectureManifest` / `RepositoryModel` | Additive field | Domain-ownership struct required; tests may read it from the model. Prefer **crate-private** types so frozen `public_structs` / `public_enums` do not rise. |
| Guard IDs | `KNOWN_CHECK_IDS` length 15 | No Guard **16**. No `cargo xtask consolidation`. |
| Phase 0 freeze | `[program.architectural_consolidation]` | `status=active`, `feature_expansion=restricted`. This slice is `consolidation` + `consolidation-docs`. |
| `INV-NO-SUPERSEDED-BASELINES` | Guard 04 predicate; `guard_check = "13"` metadata | Honesty-amend meaning: **superseded leftovers**, not “any `*_baseline.rs` during an in-flight dual-suite window.” See §11.5. |
| CON-T01–T10 | `sdd_architectural_consolidation_target.rs` | Keep CON-T01–T06 / T08–T09 GREEN. Amend CON-T07 for the window; CON-T10 stays live guard 01–15 pass. |
| Spec lifecycle | `architecture/spec-lifecycle.toml` | **Existing** row for this file; no second program spec. |
| Forbidden patterns | `FORBID-HYPOTHETICAL-*`, `FORBID-TESTS-SDD` | Unchanged. |
| Product law needles | Guard 05–12 | Unchanged (`CanonicalCatalog::load`, `project_readiness`, ledger `current`/`as_of`/`latest`, `replay_assessment`, `project_soa_from_snapshot`, …). |

### 11.3 Current behavior (baseline — GREEN on CURRENT before Phase 1 product)

§11.3 is the **pre-implement characterization**. It is **not** current law after implement. Executable CON-B11+ must PASS on the pre-implement tree.

#### 11.3.1 Domain-ownership file and loader

- `architecture/domain-ownership.toml` is **not a file**.
- `load_architecture_manifest` reads only `architecture/architecture.toml`. There is no `load_domain_ownership`. Extra architecture files are not fail-closed.
- `ArchitectureManifest` fields: `schema`, `policy`, `ownership`, `consolidation`. No domain-ownership field.
- Guard **01** passes when `architecture.toml` loads. A paper sibling file cannot fail Guard 01.

#### 11.3.2 Crate-level kinds without the five roles

Live `[ownership.*]` (crate-level):

| Concept key | crate | kind | paths |
| --- | --- | --- | --- |
| `catalog` | `weeping-angel-canonical-catalog` | exclusive | `crates/weeping-angel-canonical-catalog` |
| `framework_compilation` | `weeping-angel-framework` | exclusive | `crates/weeping-angel-framework` |
| `readiness_projection` | `weeping-angel-assurance` | projection | `…/src/readiness.rs` |
| `temporal_evidence_selection` | `weeping-angel-assurance` | **exclusive** | `…/src/temporal.rs` |
| `assessment_lineage` | `weeping-angel-assurance` | exclusive | `…/src/lineage.rs` |
| `evidence_persistence` | `weeping-angel-evidence` | exclusive | `crates/weeping-angel-evidence` |
| `assurance_cli` | `weeping-angel` | facade | `src/main.rs`, `src/cli.rs` |
| `repository_guard` | `xtask` | exclusive | `xtask` |

Comment on `temporal_evidence_selection` already admits the primitive still lives in control-test; `kind = exclusive` still claims exclusivity. Domain-ownership must not copy that lie.

#### 11.3.3 Temporal split (honesty hinge)

| Seat | Live evidence |
| --- | --- |
| Assurance temporal **projection** / timeline / diff | `crates/weeping-angel-assurance/src/temporal.rs` (`project_timeline`, `EvidenceTimeline`, `TemporalDiff`) |
| Control-test **evaluation primitive** | `crates/weeping-angel-control-test/src/temporal.rs` `pub fn select_latest_as_of` |
| Evidence **storage** / validity | `weeping-angel-evidence` ledger `current()` / `as_of()` / `latest()`; `src/validity.rs` `project_validity` |

#### 11.3.4 Live workspace (do not invent crates)

`Cargo.toml` `[workspace].members`: `weeping-angel-assurance-ir`, `weeping-angel-framework`, `weeping-angel-evidence`, `weeping-angel-collector`, `weeping-angel-control-test`, `weeping-angel-assurance`, `weeping-angel-canonical-catalog`, `xtask`. Root package `weeping-angel`. **No** `weeping-angel-catalog`. **No** `weeping-angel-assurance-cli`.

#### 11.3.5 Dual-suite / invariant CURRENT

- Phase 0 baseline file is **deleted**. CON-T07 asserts `!xtask/tests/sdd_architectural_consolidation_baseline.rs.exists()`.
- CON-T01–T10 exist and are GREEN on the healthy tree.
- No CON-T11+. No CON-B11+.
- `eval_no_superseded_baselines` treats **any** indexed `*.baseline.rs` / `*_baseline.rs` as leftover. Root `tests/contracts/*.baseline.rs` are already deleted (product dual-suites use `*.target.rs`). `xtask/` is **not** in `repo.filesystem` today.

#### 11.3.6 Baseline suite IDs (must be GREEN on CURRENT pre-implement code)

Recreate `xtask/tests/sdd_architectural_consolidation_baseline.rs` in the **implement** dual-suite window (not this spec-first commit). Do **not** `#[ignore]`. Characterization:

| ID | Characterization |
| --- | --- |
| CON-B11 | `architecture/domain-ownership.toml` is not a file |
| CON-B12 | `load_architecture_manifest` / `architecture.rs` have no domain-ownership parser; sibling files are not loaded |
| CON-B13 | `[ownership.*]` uses crate-level `kind` only; source/text has no required five role keys as ownership law |
| CON-B14 | `temporal_evidence_selection` `kind=exclusive` on assurance while `select_latest_as_of` lives in `weeping-angel-control-test` |
| CON-B15 | No `INV-DOMAIN-OWNERSHIP*` in `invariants.toml`; `evaluate_invariant` has no such predicates |
| CON-B16 | Workspace members do not include `weeping-angel-catalog` or `weeping-angel-assurance-cli` |

Target RED must fail **because CON-T11+ obligations are unmet**, not because Guard 05–12 / catalog product code regress.

### 11.4 Desired behavior (target — GREEN after Phase 1 implement)

**Implemented.** CON-T11–T20 GREEN on the healthy tree. Field-level law below is the live Phase 1 contract.

#### 11.4.1 File and schema

`architecture/domain-ownership.toml` (schema + roles as shipped):

```toml
schema = "weeping-angel/domain-ownership/v1"

required_roles = [
  "semantic_owner",
  "storage_owner",
  "projection_owner",
  "evaluation_primitive_owner",
  "adapter_owner",
]
```

| Key | Law |
| --- | --- |
| `schema` | Exactly `weeping-angel/domain-ownership/v1` |
| `required_roles` | Non-empty; **must** include the five names above. A sixth key `persistence_owner` is **illegal**. The program brief’s `persistence_owner` **maps to** `storage_owner`. |
| `[concept.<id>]` | One table per named concept. Missing seeded ids → fail closed. |

Every `[concept.*]` table **must** include the five role keys (string values). Additional evidence keys (`module`, `function`, `parser`, `compiler`, `representation`, `split`) are **not** roles.

**Owner-seat values** (closed):

- a **live** workspace package name (`weeping-angel-*`, `xtask`, `weeping-angel`), **or**
- `none` when that role is unoccupied, **or**
- `divided` **only** for `semantic_owner` when `split = "divided"` and the other four roles still name occupying seats.

Never `weeping-angel-catalog` or `weeping-angel-assurance-cli`. Module-only seats (e.g. lineage storage) use `storage_owner` = the crate plus `storage_module` / `representation` evidence keys — do not invent a crate named `lineage`.

`split` closed set: `unified | divided | facade`. Default `unified` if omitted.

Parser fail-closed if: file missing; not parseable TOML; wrong/missing schema; empty `required_roles`; missing any of the five role names; a concept missing a required role key; illegal owner-seat (unknown package that is not `none`/`divided`); hypothetical package names; `persistence_owner` used as a role key.

#### 11.4.2 Seeded concepts (live evidence only)

Needles must exist in the named crate **on CURRENT**. Do not rewrite product code to invent them.

| Concept id | semantic_owner | Evidence (CURRENT paths/symbols) | Other roles / split |
| --- | --- | --- | --- |
| `applicability` | `weeping-angel-assurance` | module `applicability`; representation `ApplicabilitySnapshot` in `crates/weeping-angel-assurance/src/applicability/snapshot.rs` | `projection_owner = weeping-angel-assurance`; `storage_owner = weeping-angel-assurance` with `storage_module = "lineage"` and `LineageApplicabilitySnapshot` (`src/lineage.rs`). Brief `persistence_owner=lineage` **is** this storage seat. |
| `readiness` | `weeping-angel-assurance` | module `readiness`; `pub fn project_readiness` (`src/readiness.rs`) | `projection_owner = weeping-angel-assurance`; other roles `none` unless a later phase names storage |
| `catalog` | `weeping-angel-canonical-catalog` | parser `CanonicalCatalog::load` (`pub fn load`) | `adapter_owner = none` |
| `framework` | `weeping-angel-framework` | compiler `compile_framework` | |
| `evidence` | `weeping-angel-evidence` | ledger `current()` / `as_of()` / `latest()` | `storage_owner = weeping-angel-evidence`; validity via `evidence_validity` |
| `temporal_evaluation` | `weeping-angel-control-test` | `evaluation_primitive_owner` + function `select_latest_as_of` (`control-test/src/temporal.rs`) | **`split = "divided"`**. `storage_owner` / `projection_owner` = `weeping-angel-assurance` (`src/temporal.rs` timeline/diff). Must **not** copy architecture.toml `kind=exclusive` as if control-test did not own the primitive. |
| `assessment_replay` | `weeping-angel-assurance` | `pub fn replay_assessment` (`src/lineage.rs`) | |
| `soa` | `weeping-angel-assurance` | `pub fn project_soa_from_snapshot` (`src/soa.rs`; Guard **12** needle) | `projection_owner = weeping-angel-assurance` |
| `control_status` | `divided` | `ImplementationStatus` in `weeping-angel-assurance-ir` (`src/implementation.rs`); `Effectiveness` in `weeping-angel-control-test` (`src/lib.rs`); SoA projection in assurance | **`split = "divided"`**. `evaluation_primitive_owner = weeping-angel-control-test`; `projection_owner = weeping-angel-assurance`; `storage_owner` may be `none` this slice. Do **not** force one exclusive crate. |
| `control_test_kernel` | `weeping-angel-control-test` | `evaluate` in `src/run.inc` | `evaluation_primitive_owner = weeping-angel-control-test` |
| `evidence_validity` | `weeping-angel-evidence` | `pub fn project_validity` (`src/validity.rs`) | |
| `catalog_loading` | `weeping-angel-canonical-catalog` | `CanonicalCatalog::load` | same parser seat as `catalog` (named for later SSOT phases) |
| `framework_compilation` | `weeping-angel-framework` | `compile_framework` | aligned with architecture.toml concept key |
| `assurance_cli` | `weeping-angel` | facade `src/main.rs` + `src/cli.rs` | `adapter_owner`/`split = facade` as needed; **not** `weeping-angel-assurance-cli` |
| `collectors` | `weeping-angel-collector` | `CollectorAdapter` in `src/ports/adapter.rs` | `adapter_owner = weeping-angel-collector` |

`repository_guard` may be recorded (`semantic_owner = xtask`) so later guard SSOT phases have a named owner; optional this slice if the required seed list above is complete.

#### 11.4.3 Loader and Guard fold-in

**Loader:** `xtask/src/architecture.rs` parses `architecture/domain-ownership.toml`. Call from `load_architecture_manifest` **or** `RepositoryModel::load` such that Guard **01** cannot pass when the file is missing/malformed. Paper file on disk without parse is **not** a pass (fixture CON-T12).

**Invariants (Guard 04)** — implement **must** add predicates (unknown ids already fail closed). Suggested ids:

| Invariant id | Predicate sketch |
| --- | --- |
| `INV-DOMAIN-OWNERSHIP-PRESENT` | File exists; schema exact; parsed into the model; required_roles contains the five names |
| `INV-DOMAIN-OWNERSHIP-ROLES` | Every `[concept.*]` has the five keys; no `persistence_owner` role; no hypothetical packages; seeded concepts present; `temporal_evaluation.split = divided` and `select_latest_as_of` cited; `control_status.split = divided` |

Fold into existing 01–15. Do **not** add Guard 16. Do **not** add a second health CLI.

#### 11.4.4 Dual-suite target IDs (GREEN on the healthy tree)

Extend `xtask/tests/sdd_architectural_consolidation_target.rs` (same auto-discovered binary; no new `[[test]]`):

| ID | Obligation |
| --- | --- |
| CON-T11 | `architecture/domain-ownership.toml` parsed; schema `weeping-angel/domain-ownership/v1`; five `required_roles`; seeded concept tables present |
| CON-T12 | Missing/malformed file fails Guard 01 and/or 04 (fixture); paper extra file without parse is **not** a pass |
| CON-T13 | Roles not collapsed: a concept that only copies `kind=exclusive` without the five keys fails; `persistence_owner` as a sixth role fails |
| CON-T14 | Seeded concepts cite live symbols (`ApplicabilitySnapshot`, `project_readiness`, `CanonicalCatalog::load` / `pub fn load`, `compile_framework`, `select_latest_as_of`, `replay_assessment`, `project_soa_from_snapshot`, `ImplementationStatus`, `Effectiveness`, `evaluate` in `run.inc`, `project_validity`, `CollectorAdapter`); no hypothetical crate names |
| CON-T15 | `temporal_evaluation` records the control-test primitive **and** assurance projection/storage split (`split = divided`); does not claim fake exclusivity |
| CON-T16 | `INV-DOMAIN-OWNERSHIP-PRESENT` and `INV-DOMAIN-OWNERSHIP-ROLES` (or equivalent) have `evaluate_invariant` predicates; `KNOWN_CHECK_IDS.len()==15`; no Guard 16; no `cargo xtask consolidation` |
| CON-T17 | Dual-suite under `xtask/tests/sdd_architectural_consolidation_*.rs`; no `tests/sdd/`; no new root/xtask `[[test]]`; Phase 0 CON-T01–T06 / T08–T09 still pass |
| CON-T18 | This spec remains the **only** program spec in `CANONICAL_SPECS` + `spec-lifecycle.toml`; ADR 0050 file exists with meta `id = "0050"` (Draft until GREEN, then Accepted) |
| CON-T19 | `INV-NO-SUPERSEDED-BASELINES` honesty: live **non-ignored** `xtask/tests/sdd_*_baseline.rs` allowed **during** the dual-suite window; `#[ignore]` baselines, `tests/sdd/` leftovers, and leftover after GREEN fail-closed |
| CON-T20 | Neighbors: ACP target, SR target, `cargo xtask guard` 01–15 pass; applicability/readiness/lineage product needles unchanged |

Keep Phase 0 CON-T01–T06 / T08–T09 GREEN. CON-T07: **during** the slice, **allow** `xtask/tests/sdd_architectural_consolidation_baseline.rs` if it is non-ignored and encodes CON-B11–B16; **after GREEN**, reassert deletion (`INV-NO-SUPERSEDED-BASELINES`).

#### 11.4.5 `INV-NO-SUPERSEDED-BASELINES` honesty amendment (ADR 0050)

**CURRENT (too coarse):** any indexed path ending `.baseline.rs` / `_baseline.rs` fails.

**DESIRED meaning:** fail-closed on **superseded leftovers**:

1. `#[ignore]` on a dual-suite baseline (skip-supersede), **or**
2. baseline under `tests/sdd/` / `test/sdd/` (forbidden location), **or**
3. leftover `xtask/tests/sdd_*_baseline.rs` **after** the owning slice’s target is GREEN and the protocol requires delete.

**Allow:** one live, **non-ignored** `xtask/tests/sdd_*_baseline.rs` characterization suite **during** an in-flight dual-suite window (Phase 1 CON-B11–B16). Do **not** `#[ignore]` that file.

If implementation expands `RepositoryModel` filesystem to include `xtask/`, the amended predicate is **mandatory** before recreating the file, or Guard 04 / CON-T10 go RED for the wrong reason.

After CON-T11–T20 GREEN: baseline suite **FAIL** on the new tree, then **DELETE** (not ignore). CON-T07 reasserts absence.

### 11.5 Dual-suite and verify commands

```text
cargo test -p xtask --test sdd_architectural_consolidation_target
cargo xtask guard
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --features demo -- -D warnings
cargo test --workspace --features demo --all-targets
```

Protocol (complete):

1. Spec first (this §11) + Draft ADR 0050.
2. Dual-suite: recreated baseline CON-B11–B16 GREEN on CURRENT; CON-T11–T20 RED on CURRENT for the right reason; CON-T07 allowed the window; honesty-amended the invariant **before** a walk that would see `xtask/tests/*_baseline.rs` as leftover.
3. Implemented 1.1–1.3 until CON-T11–T20 GREEN; baseline FAIL.
4. **Deleted** the baseline suite (did not `#[ignore]`).
5. **Accepted** ADR 0050.

### 11.6 Acceptance criteria (testable)

- [x] Dual-suite lives in `xtask/tests/sdd_architectural_consolidation_{baseline,target}.rs` (not `tests/sdd/`, not `test/sdd/*.ts`, not a new `[[test]]`). After GREEN, baseline **deleted**.
- [x] CON-B11–B16 PASS on the pre-implement tree (missing file, no parser, crate-level kinds without five roles, temporal exclusive lie vs `select_latest_as_of`, no domain-ownership invariants, no hypothetical crates).
- [x] CON-T11–T20 FAIL on CURRENT **because** domain-ownership file/parser/roles/seeds/split/invariants are absent (not unrelated product code), then PASS after implement.
- [x] Phase 0 CON-T01–T06 / T08–T09 stay GREEN. CON-T07 allows the live non-ignored baseline during the window and reasserts deletion after GREEN.
- [x] `architecture/domain-ownership.toml` schema `weeping-angel/domain-ownership/v1`; five role keys required; `persistence_owner` is not a sixth role; seeded concepts + live symbols as §11.4.2.
- [x] `temporal_evaluation` and `control_status` are `split = divided` (no fake exclusivity).
- [x] Guard 01 and/or 04 fail-closed on missing/malformed/paper-unparsed file. No Guard 16. No second health CLI.
- [x] `INV-NO-SUPERSEDED-BASELINES` means superseded leftovers; live dual-suite window file allowed; leftover after GREEN fail-closed.
- [x] Neighbors stay green: `CANONICAL_SPECS`, Guard 15 **existing** consolidation spec row, ACP target, SR target, `cargo xtask guard` 01–15.
- [x] ADR 0050 **Accepted** only after target GREEN. No `0003-*` / colliding `0011-*`.
- [x] Phase 1 does not migrate consumers, delete DUP sources, or change applicability/readiness/lineage product semantics.
- [x] Implementation does not raise frozen expansion metrics; no hypothetical crates.

### 11.7 Out of scope (Phase 1)

1. Phase 2+ consumer migrations and deleting DUP duplicate source trees
2. Rewriting assurance / collector / catalog / IR product semantics
3. New frameworks, collectors, ISMS engines, SARIF/report formats, product scanners
4. Hypothetical packages `weeping-angel-catalog` / `weeping-angel-assurance-cli`
5. `tests/sdd/`, `test/sdd/*.ts`, new root or xtask `[[test]]`
6. Guard 16 / `cargo xtask consolidation` / second health CLI
7. A second program spec or second Guard 15 consolidation row
8. Mass ADR renumber; `#[ignore]`-superseding the baseline
9. pnpm / `apps/docs`
10. Flipping `feature_expansion` to unrestricted
11. Collapsing five roles into crate `kind = exclusive`
12. Treating Phase 1 as “move `select_latest_as_of` into assurance” (that is a later phase; this slice **records** the split)

### 11.8 Risks

- A paper `domain-ownership.toml` without loader changes looks like ownership law but is not a gate (CON-T12 exists to prevent it).
- Copying architecture.toml `kind=exclusive` onto `temporal_evaluation` or `control_status` hides the live split (CON-T15 / CON-T13).
- Recreating `sdd_architectural_consolidation_baseline.rs` without amending CON-T07 REDS Phase 0 CON-T07 for the wrong protocol reason.
- Expanding `RepositoryModel.filesystem` to `xtask/` without the honesty-amended invariant REDS Guard 04 / CON-T10 while the dual-suite window is open.
- `#[ignore]` on the baseline after GREEN violates `INV-NO-SUPERSEDED-BASELINES`.
- Adding `pub struct` parser types in xtask raises frozen `public_structs` (`INV-CONSOLIDATION-EXPANSION-RESTRICTED`).
- Inventing `weeping-angel-catalog` / `weeping-angel-assurance-cli` as owner seats trips `FORBID-HYPOTHETICAL-*`.
- A second `docs/specs/*consolidation*` file forks the program SSOT (Guard 15 / expansion / `CANONICAL_SPECS`).
- Dual-suite needles that rewrite product modules make target RED for the wrong reason.
- Adding Guard 16 or a new xtask health subcommand forks the health plane ([ADR 0011](../adr/0011-repository-guard-governance.md)).

### 11.9 remaining_backlog (not Phase 1)

1. Phase 2+ migrate consumers onto the seats named here
2. Delete duplicate files once close law holds
3. Move or unify temporal primitives only under a later phase that **changes** product architecture (Phase 1 records the divide)
4. Merge `control_status` into one semantic owner (today honestly divided)
5. Rebase the frozen consolidation baseline if a later ADR must add public parser types
6. Flip `feature_expansion` to unrestricted (later ADR)

---

## 12. Convergence increments (C01–C16)

Phase 0 freeze and Phase 1 ownership law stay in **this file**. Do **not** author a second giant “architectural cleanup” spec.

After ownership law exists, cleanup uses `/workflow xylex-sdd-consolidation` (not expansion SDD). That workflow inverts dual-suite toward **net reduction**:

```text
Does this capability already exist, partially or under another representation?
If yes: EXTEND / MIGRATE / CONSOLIDATE. Do not CREATE.
```

Hard rules per run:

1. No feature creation.
2. Exactly one `debt_id` (C01…C16 or one `DUP-*`).
3. Canonical owner named before implementation.
4. Delete or migrate the old path — do not add a parallel abstraction.
5. Incomplete until the old representation is gone or compatibility-only.
6. Guards and specs may not be weakened to pass.
7. Public API, duplicate-type, root-test, and duplicate-helper counts may not increase.
8. Finish by regenerating `cargo xtask inventory` and updating debt state from evidence.

Per-increment spec is the **small** run spec (`docs/sdd/<slug>-<run>/spec.md`). Dual-suite lives under `xtask/tests/sdd_consolidation_<id>_*.rs`. Never `tests/sdd/`. Prefer compiler / visibility / dependency direction / `cargo xtask guard` over source-string probes when the property can move.

### 12.1 Order

| Increment | Target | Debt | Expected reduction | Parallelism |
| --- | --- | --- | --- | --- |
| C01 | Contract-test support consolidation | DUP-002 | `fn require_needles` 18 → 1 | Independent of C04–C09 |
| C02 | `RepositoryModel` consolidation | DUP-014 | filesystem walkers 2 → 1 | Independent of C04–C09 |
| C03 | Temporal ownership reconciliation | DUP-017 | metadata matches physical architecture | After Phase 1 seats exist |
| C04 | Readiness SSOT | DUP-011 | readiness derivation paths → 1 | **Serial semantic graph** |
| C05 | Applicability SSOT | DUP-004 | duplicate applicability models removed | **Serial semantic graph** |
| C06 | Framework pack parse SSOT | DUP-013 | parser path → 1 | **Serial semantic graph** |
| C07 | Lineage replay boundary | DUP-005 | unverified public reconstruction removed | **Serial semantic graph** |
| C08 | SoA temporal semantics | DUP-006 | explicit live/pinned paths | **Serial semantic graph** |
| C09 | Temporal leaf primitive | DUP-007 | duplicate algorithm removed/shared | **Serial semantic graph** |
| C10 | Collector migration | DUP-015 | old collector façade removed | Independent of C04–C09 |
| C11 | Evidence persistence boundary | (evidence crate / SQLite adapter) | domain isolated from adapter | Independent of C04–C09 |
| C12 | Assurance internal structure | (assurance crate coupling) | flatter coupling | After C04–C09 |
| C13 | Root application thinning | (root `src/` orchestration) | orchestration leaves root lib | After C12 |
| C14 | Scanner restructuring | DUP-016 / `engine` vs `engines` | ambiguity removed | Independent of C04–C09 |
| C15 | Public surface reduction | `pub` / re-export / alias | public symbol count down | After C04–C09 and C07 |
| C16 | Test architecture reduction | repository-law tests → xtask | source-scanning contract tests leave `tests/contracts` | After C01; can overlap C14 |

C04–C09 share one semantic graph. **Do not run them in parallel:**

```text
Applicability → Assessment → Lineage → Readiness → SoA → Temporal reconstruction
```

C01 (test support) and C14 (scanner) may run beside that serial chain. C03 waits for Phase 1 domain-ownership seats. C01 increment spec: [`docs/sdd/c01-contract-test-support-consolidation-run/spec.md`](../sdd/c01-contract-test-support-consolidation-run/spec.md) (**implemented**; DUP-002 verified; live `require_needles_fns` = 1).

Close a row only when canonical owner exists, consumers use it, old path is gone or compatibility-only, **and** a regression guard exists. Green tests with competing semantic paths are not done.
