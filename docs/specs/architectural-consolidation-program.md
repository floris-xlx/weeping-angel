# SDD: Weeping Angel Architectural Consolidation Program — Phase 0 (freeze + baseline + backlog schema)

| Field | Value |
| --- | --- |
| Status | **Implemented (Phase 0).** Consolidation mode, frozen baseline snapshot, and v2 duplication backlog schema are landed. Dual-suite target is GREEN; baseline suite deleted (`INV-NO-SUPERSEDED-BASELINES`). |
| Program | Architectural Consolidation Program (preserve architecture; eliminate parallel truths). |
| Slice | **Phase 0 only** (0.1 consolidation mode, 0.2 frozen baseline snapshot, 0.3 structural-duplication backlog schema). **Not** Phases 1+ consumer migrations, duplicate deletes, or product rewrites. |
| Dual-suite | `xtask/tests/sdd_architectural_consolidation_target.rs` via `cargo test -p xtask` auto-discovery. Phase 0 baseline suite **deleted** (`INV-NO-SUPERSEDED-BASELINES`; do not `#[ignore]`). **Do not** create `tests/sdd/` ([ADR 0004](../adr/0004-documentation-architecture.md) / `FORBID-TESTS-SDD` / architectural-cleanup Phase 0 freeze item 6). Do not invent `test/sdd/*.ts`. Do not add a new root `[[test]]`. |
| ADR | **Accepted** [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md). Next unused prefix is **0050**. Do **not** mint `0003-*` or a colliding `0011-*`. |
| Human SSOT | **This file** under `docs/specs/`. [`docs/sdd/architectural-consolidation-program.md`](../sdd/architectural-consolidation-program.md) is the SDD run pointer only. Generated traces stay in `.sdd/`. |
| Predecessor law | [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md) (ACP Phase 0 is **spec-law-only freeze**, a different program), [`docs/specs/structural-reconciliation.md`](structural-reconciliation.md), [`docs/specs/repository-integrity.md`](repository-integrity.md), ADRs 0004, 0009–0012, 0048. |
| Public contract | [`docs/specs/assurance-runtime.md`](assurance-runtime.md) untouched. |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), ADR 0001 |
| Neighbors (must stay GREEN) | `sdd_documentation_layout` `CANONICAL_SPECS`, Guard **15** `architecture/spec-lifecycle.toml`, `cargo xtask guard` 01–15, `sdd_architectural_cleanup_target`, `sdd_structural_reconciliation_target`. |
| Collision fence | Do **not** treat architectural-cleanup Phase 0 (spec-law-only freeze) as this program. This slice makes freeze + baseline + backlog schema **machine-readable**. Do not invent `weeping-angel-catalog` / `weeping-angel-assurance-cli`. |
| Toolchain | Cargo workspace (seven `weeping-angel-*` crates + unpublished `xtask`). **pnpm / `apps/docs` out of scope.** |
| Inventory SSOT | `cargo xtask inventory` / `xtask/src/inventory.rs` / schema `weeping-angel/inventory/v1` / live snapshot [`docs/debt/current.md`](../debt/current.md). Extend that projection; **do not** invent a second counter (DUP-014). |
| `adr_needed` | **true** — architecture policy table, guard evaluation of extra TOML, freeze vs expansion. |
| Protocol | **Complete.** Spec first → baseline GREEN on pre-implement tree → target RED for the three missing artifacts → implement 0.1–0.3 → target GREEN → **delete** baseline suite → Accept ADR 0049. |

This document is the durable human SSOT for **Phase 0** of the Architectural Consolidation Program.

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

Phase 0 may change **only** (implement slice; this spec-first commit is docs/ADR + neighbor index):

| Concern | Home |
| --- | --- |
| This SSOT | `docs/specs/architectural-consolidation-program.md` |
| SDD pointer | `docs/sdd/architectural-consolidation-program.md` |
| Accepted ADR | `docs/adr/0049-architectural-consolidation-phase-0.md` |
| Consolidation mode | `architecture/architecture.toml` `[program.architectural_consolidation]` + parse on `ArchitectureManifest` |
| Freeze evaluation | Guard **01 / 02 / 03 / 04** and/or a new `[[invariant]]` with a predicate in `evaluate_invariant` — **not** Guard 16 |
| Frozen snapshot | `docs/debt/consolidation-baseline.json` + `docs/debt/consolidation-baseline.md` projected from `xtask/src/inventory.rs` |
| Live mechanical snapshot | `docs/debt/current.md` (unchanged role) |
| Backlog schema | `docs/debt/structural-duplication.toml` (`weeping-angel/structural-duplication/v2`) |
| Dual-suite | `xtask/tests/sdd_architectural_consolidation_target.rs` (baseline deleted) |
| Neighbor index | `architecture/spec-lifecycle.toml`, `tests/contracts/documentation_layout.rs` `CANONICAL_SPECS` |

| Do not touch | Owner |
| --- | --- |
| Phases 1+ consumer migrations; deleting DUP duplicate files | Later program phases |
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
| `INV-NO-SUPERSEDED-BASELINES` | Guard 13 / 04 | Phase 0 baseline suite **deleted**. Do not restore it as `#[ignore]`. |
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

## 9. remaining_backlog (not this slice)

1. Phase 1+ migrate consumers onto canonical owners (DUP-002 helpers, DUP-004 snapshot shapes, DUP-011 readiness constructors, …)
2. Delete duplicate files once close law holds (e.g. remaining schema copies)
3. Merge inventory and `RepositoryModel` walks (DUP-014) without a third walker
4. Semantic detection of new projection paths beyond count/ownership heuristics
5. Flipping `feature_expansion` to unrestricted (requires a later ADR)
6. Architectural-cleanup Phases 2–28 product work (separate program)

---

## 10. Related

- Accepted decision: [`docs/adr/0049-architectural-consolidation-phase-0.md`](../adr/0049-architectural-consolidation-phase-0.md)
- Pointer: [`docs/sdd/architectural-consolidation-program.md`](../sdd/architectural-consolidation-program.md)
- [`docs/specs/architectural-cleanup-program.md`](architectural-cleanup-program.md) — **different** Phase 0
- [`docs/specs/structural-reconciliation.md`](structural-reconciliation.md), [ADR 0048](../adr/0048-structural-reconciliation.md)
- [`docs/specs/repository-integrity.md`](repository-integrity.md), ADRs [0004](../adr/0004-documentation-architecture.md), [0009](../adr/0009-repository-health-gate.md), [0010](../adr/0010-architecture-as-law.md), [0011](../adr/0011-repository-guard-governance.md)
- [`docs/debt/current.md`](../debt/current.md), [`docs/debt/structural-duplication.toml`](../debt/structural-duplication.toml)
