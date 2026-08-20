# ADR 0050 — Canonical domain ownership model (five roles, sibling SSOT, dual-suite baseline honesty)

| Field | Value |
| --- | --- |
| Status | **Accepted** — Architectural Consolidation Phase 1 target GREEN (`CON-T11–T20`); `xtask/tests/sdd_architectural_consolidation_baseline.rs` deleted. |
| Date | 2026-08-20 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine, catalog, or collector decisions. **Does not rewrite** [ADR 0009](0009-repository-health-gate.md), [ADR 0010](0010-architecture-as-law.md), [ADR 0011](0011-repository-guard-governance.md), [ADR 0048](0048-structural-reconciliation.md), or [ADR 0049](0049-architectural-consolidation-phase-0.md) bodies. **Amends** architecture-as-law: crate-level `[ownership.*]` kinds are **not** the concept-level semantic SSOT. **Honesty-amends** `INV-NO-SUPERSEDED-BASELINES` (see Decision §4). |
| Extends | [ADR 0004](0004-documentation-architecture.md), [ADR 0010](0010-architecture-as-law.md), [ADR 0011](0011-repository-guard-governance.md), [ADR 0049](0049-architectural-consolidation-phase-0.md) |
| Spec | [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) §11 |
| Tests | `xtask/tests/sdd_architectural_consolidation_target.rs` (CON-T11–T20 GREEN). Dual-suite baseline recreated then **deleted** (`INV-NO-SUPERSEDED-BASELINES`). |

<!-- weeping-angel-adr-meta
id = "0050"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0004-documentation-architecture", "0010-architecture-as-law", "0011-repository-guard-governance", "0049-architectural-consolidation-phase-0"]
-->

> Filename **`0050-domain-ownership-model.md`**. Cite **this file by path**. Do **not** add a `0003-domain-ownership*.md` sibling or a colliding `0011-*`. Next unused unique prefix after this file is **0051**.

## Context

Phase 0 ([ADR 0049](0049-architectural-consolidation-phase-0.md)) made consolidation mode, the frozen baseline, and the v2 duplication backlog machine-readable. Ownership was still crate-level:

1. `architecture/architecture.toml` `[ownership.*]` maps concepts to a crate, a `kind` (`exclusive | facade | projection | adapter | shared-primitive`), and paths. That cannot express “semantic owner in crate A, storage in module B, evaluation primitive in crate C.”
2. `[ownership.temporal_evidence_selection]` is `kind = exclusive` on `weeping-angel-assurance` / `src/temporal.rs` while `select_latest_as_of` still lives in `weeping-angel-control-test`. Exclusive crate ownership is **false** for that concept.
3. Before this slice, `architecture/domain-ownership.toml` did not exist. `load_architecture_manifest` parsed `architecture.toml` only. Extra files were not a gate.
4. `INV-NO-SUPERSEDED-BASELINES` fail-closed on any indexed `*.baseline.rs` / `*_baseline.rs`. Phase 1 had to recreate a **live, non-ignored** `xtask/tests/sdd_*_baseline.rs` during the dual-suite window. Treating that window file as a leftover would make the protocol illegal. The invariant’s **intent** is superseded leftovers (`#[ignore]`, `tests/sdd/`, leftover after GREEN).

Questions this decision answers:

1. Where does **concept-level** ownership live relative to crate-level `[ownership.*]`?
2. Which **roles** must be named, and when may they split?
3. How is the file enforced without Guard 16 or a second health CLI?
4. What does `INV-NO-SUPERSEDED-BASELINES` mean during vs after a dual-suite window?

## Decision (Accepted — field-level law is the spec)

Field-level law is [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md) §11. This ADR records the architecture-policy choices **as implemented**:

1. **Sibling SSOT.** Concept-level ownership lives in [`architecture/domain-ownership.toml`](../../architecture/domain-ownership.toml) (`schema = "weeping-angel/domain-ownership/v1"`). It **expands** crate-level `[ownership.*]`; it does not replace `architecture.toml`. `load_architecture_manifest` parses the sibling file into crate-private `ArchitectureManifest.domain_ownership`. Missing/malformed → fail closed (Guard 01). A paper file that is not parsed is **not** a pass.
2. **Five roles.** `required_roles` and every `[concept.*]` table MUST include `semantic_owner`, `storage_owner`, `projection_owner`, `evaluation_primitive_owner`, `adapter_owner`. Do **not** invent a sixth role `persistence_owner` (map that brief name to `storage_owner`). Owner-seat values are live workspace packages, `none`, or `semantic_owner = "divided"` when `split = "divided"`. `split` is `unified | divided | facade`. Never `weeping-angel-catalog` or `weeping-angel-assurance-cli`.
3. **Splits are first-class (shipped seats).**
   - `temporal_evaluation` is **`split = divided`**: `semantic_owner` / `evaluation_primitive_owner` = `weeping-angel-control-test` (`select_latest_as_of`); `storage_owner` / `projection_owner` = `weeping-angel-assurance` (`src/temporal.rs` timeline/diff). Ledger clocks stay on concept `evidence` (`weeping-angel-evidence`). Domain-ownership **records** this split. It does **not** copy architecture.toml `kind = exclusive` and does **not** move the primitive in Phase 1.
   - `control_status` is **`split = divided`**: `semantic_owner = divided`; `ImplementationStatus` in `weeping-angel-assurance-ir`; `evaluation_primitive_owner = weeping-angel-control-test` (`Effectiveness`); `projection_owner = weeping-angel-assurance` (SoA).
   - `assurance_cli` is **`split = facade`**: `semantic_owner` / `adapter_owner` = root package `weeping-angel` (`src/main.rs`, `src/cli.rs`) — not `weeping-angel-assurance-cli`.
4. **`INV-NO-SUPERSEDED-BASELINES` honesty.** The invariant means **superseded leftovers**, not “no `*_baseline.rs` may exist while a dual-suite is in flight.” Allowed: live, non-ignored `xtask/tests/sdd_*_baseline.rs` during the window. Forbidden: `#[ignore]` skip-supersede, `tests/sdd/` leftovers, leftover after GREEN. After Phase 1 GREEN the characterization file was **deleted** (not ignored). CON-T07 follows the same window-then-delete law.
5. **Enforcement seat.** Fold into Guard **01** (load) and Guard **04** invariants `INV-DOMAIN-OWNERSHIP-PRESENT` and `INV-DOMAIN-OWNERSHIP-ROLES`. Do **not** add Guard 16. Do **not** add a second health CLI. Phase 0 freeze remains active; this slice is `consolidation` + `consolidation-docs`. Parser types stay crate-private so frozen expansion metrics do not rise.
6. **Phase 1 is ownership law, not migration.** Do not migrate consumers, delete duplicates, or change applicability/readiness/lineage product semantics in this slice. Seed only live crates/modules/symbols.
7. **Human SSOT.** Spec remains `docs/specs/architectural-consolidation-program.md`. No second program spec / Guard 15 row.

Seeded `[concept.*]` ids (live evidence only): `applicability`, `readiness`, `catalog`, `framework`, `evidence`, `temporal_evaluation`, `assessment_replay`, `soa`, `control_status`, `control_test_kernel`, `evidence_validity`, `catalog_loading`, `framework_compilation`, `assurance_cli`, `collectors`, `repository_guard`.

## Consequences

- Later consolidation phases have a named owner per concept and a typed split when roles live in different crates.
- Crate `kind = exclusive` can no longer silently claim a primitive that lives elsewhere (`temporal_evaluation`, `control_status`).
- Dual-suite protocol can recreate a characterization baseline without lying about `INV-NO-SUPERSEDED-BASELINES`.
- Parser types in xtask must stay under the Phase 0 expansion freeze (crate-private `DomainOwnership` / `ConceptRoles`).
- `architecture.toml` crate-kind rows remain; C03 (DUP-017) still owns reconciling exclusive metadata with the physical split.

## Rejected alternatives

- Collapsing five roles into `kind = exclusive` — fake exclusivity; hides the temporal / control-status splits.
- Encoding concept roles inside `architecture.toml` `[ownership.*]` without a sibling file — mixes crate-kind policy with concept-role law and inflates architecture/v1.
- Paper `domain-ownership.toml` without a parser — same class of bug ADR 0049 closed for extra tables.
- A 16th `ProductLawCheck` or `cargo xtask consolidation` — forks the health plane ([ADR 0011](0011-repository-guard-governance.md)).
- `#[ignore]`-superseding the Phase 1 baseline — violates the honesty-amended leftover rule.
- Inventing `weeping-angel-catalog` / `weeping-angel-assurance-cli` as owner seats.
- Minting `0003-*` / colliding `0011-*` ADR files.
- Treating Phase 1 as consumer migration or as “move `select_latest_as_of` into assurance.”

## Status

**Accepted** after Phase 1 target GREEN (`cargo test -p xtask --test sdd_architectural_consolidation_target`) and deletion of `xtask/tests/sdd_architectural_consolidation_baseline.rs`. Proof: Guard 01 parses `architecture/domain-ownership.toml` (`weeping-angel/domain-ownership/v1`); Guard 04 evaluates `INV-DOMAIN-OWNERSHIP-PRESENT` and `INV-DOMAIN-OWNERSHIP-ROLES`; `temporal_evaluation` and `control_status` are `split = divided`; `INV-NO-SUPERSEDED-BASELINES` allows only an in-flight non-ignored xtask dual-suite window file.

## Related

- Spec §11: [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md)
- Machine SSOT: [`architecture/domain-ownership.toml`](../../architecture/domain-ownership.toml)
- Pointer: [`docs/sdd/architectural-consolidation-program.md`](../sdd/architectural-consolidation-program.md)
- Predecessor: [ADR 0049](0049-architectural-consolidation-phase-0.md), [ADR 0010](0010-architecture-as-law.md)
