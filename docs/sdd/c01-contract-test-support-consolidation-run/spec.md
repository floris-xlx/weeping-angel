# C01 — Contract-test support consolidation (DUP-002 only)

| Field | Value |
| --- | --- |
| Increment | **C01** |
| Debt | **DUP-002** (exactly one debt id this run) |
| Workflow | `xylex-sdd-consolidation` |
| Disposition | **CONSOLIDATE** (capability exists; no feature creation) |
| Bounded context | `contract-test-support` |
| Canonical owner | `tests/support/mod.rs::require_needles` (directory + `mod.rs` or `require_needles.rs`; included via `include!` or `#[path]`) |
| Governing ADR | [ADR 0049](../../adr/0049-architectural-consolidation-phase-0.md) — **no new ADR** |
| Master program SSOT | [`docs/specs/architectural-consolidation-program.md`](../../specs/architectural-consolidation-program.md) **§12** (this file is the small run spec only) |
| Dual-suite | `xtask/tests/sdd_consolidation_c01_target.rs` via `cargo test -p xtask` auto-discovery (uniqueness pin). C01 baseline **deleted** after target GREEN (`INV-NO-SUPERSEDED-BASELINES`). **Never** `tests/sdd/`. **Never** a new root or xtask `[[test]]`. |
| Status | **Implemented** — DUP-002 close law holds. One crate-private `fn require_needles` in `tests/support/mod.rs`; 17 `sdd_*_target` binaries `include!` it; per-file copies gone; inventory matcher `trimmed.starts_with("fn require_needles")`; live `require_needles_fns = 1`, `require_needles_calls = 206`. No new ADR. |

This is **not** a second architectural-cleanup / consolidation program spec. Laws and increment order stay in the master file. ADR 0012’s concurrent skip-rewrite of Prompt 2/3 `require_needles` targets is **superseded for this increment** by C01/DUP-002; do not mint ADR 0051.

---

## Classify (binding)

| Key | Value |
| --- | --- |
| `capability_exists` | `true` |
| `disposition` | `CONSOLIDATE` |
| `adr_action` | `none` |
| `new_adr_required` | `false` |
| `new_public_surface` | `false` |
| `new_persistence` | `false` |
| `create_justification` | _(empty — do not CREATE)_ |
| `close_law` | verified/removed only when canonical owner exists, all consumers use it, old per-file copies are gone (not compatibility aliases), and a regression guard exists (inventory uniqueness / expansion freeze — **not** Guard 16) |

Reuse APIs: `require_needles(label, src, needles)`, `inventory counts.require_needles_fns`, `inventory counts.require_needles_calls`. `duplicate_helper_definitions` aliases `require_needles_fns`.

---

## Problem

Contract dual-suites each copy a private `fn require_needles` that substring-asserts source surface. Inventory reports **18** files defining the helper while `rg` finds **17** real copies — the 18th is `xtask/src/inventory.rs` matching itself via `trimmed.contains("fn require_needles")`. Maintainers cannot change one helper; counts lie; uniqueness is not a gate.

User-visible goal: one shared test helper, 17 existing contract binaries still assert the same needles, copies gone, mechanical count **18 → 1**. No new product capability.

---

## Current behavior

1. **17 competing definitions** (same types; assert message and parameter name `src` vs `haystack` differ):

   | Copy | Root `[[test]]` | Callers |
   | --- | --- | --- |
   | `tests/contracts/assessment_lineage.target.rs` | `sdd_assessment_lineage_target` | that binary only |
   | `tests/contracts/control_implementation_registry.target.rs` | `sdd_control_implementation_registry_target` | that binary only |
   | `tests/contracts/controlled_documents.target.rs` | `sdd_controlled_documents_target` | that binary only |
   | `tests/contracts/continuity_resilience.target.rs` | `sdd_continuity_resilience_target` | that binary only |
   | `tests/contracts/iso27001_assurance.target.rs` | `sdd_iso27001_assurance_target` | that binary only (`haystack`) |
   | `tests/contracts/interested_parties_obligations.target.rs` | `sdd_interested_parties_obligations_target` | that binary only |
   | `tests/contracts/incident_governance.target.rs` | `sdd_incident_governance_target` | that binary only |
   | `tests/contracts/nonconformity_capa.target.rs` | `sdd_nonconformity_capa_target` | that binary only |
   | `tests/contracts/internal_audit.target.rs` | `sdd_internal_audit_target` | that binary only |
   | `tests/contracts/operational_soa.target.rs` | `sdd_operational_soa_target` | that binary only |
   | `tests/contracts/population_runtime.target.rs` | `sdd_population_runtime_target` | that binary only |
   | `tests/contracts/remediation_engine.target.rs` | `sdd_remediation_engine_target` | that binary only |
   | `tests/contracts/risk_register.target.rs` | `sdd_risk_register_target` | that binary only |
   | `tests/contracts/supplier_risk.target.rs` | `sdd_supplier_risk_target` | that binary only |
   | `tests/contracts/temporal_lineage_evidence_soa.target.rs` | `sdd_temporal_lineage_evidence_soa_target` | that binary only |
   | `tests/contracts/temporal_assurance.target.rs` | `sdd_temporal_assurance_target` | that binary only |
   | `tests/contracts/typed_evidence.target.rs` | `sdd_typed_evidence_target` | that binary only |

2. Each copy is `fn require_needles(label: &str, src: &str, needles: &[&str])` (iso27001: `haystack`) that collects needles with `!haystack.contains(n)` and `assert!`s emptiness. Call sites stay inside the defining file (integration-test crate root).
3. `tests/support/` **does not exist**. `tests/support.rs` must not be created (`tests_rs_autodiscovered` freeze **16**).
4. Live inventory (`docs/debt/current.md`, `docs/debt/consolidation-baseline.json`): `require_needles_fns = 18`, `require_needles_calls = 222`, `duplicate_helper_definitions = 18`, `root_test_binaries = 45`, `tests_rs_autodiscovered = 16`, `tests_contracts_rs = 43`, `public_symbols = 2022`. Matcher: `trimmed.contains("fn require_needles")` (counts `inventory.rs`).
5. DUP-002 `status = confirmed`; `canonical_owner` still “proposed”; consumers listed as a blob, not the 17 binaries.
6. Guard 04 `INV-CONSOLIDATION-EXPANSION-RESTRICTED` fails on **increase** vs freeze; **decrease is allowed**. Freeze does **not** yet pin uniqueness at 1.
7. `KNOWN_CHECK_IDS` length **15**. No Guard 16. xtask is a bin + `lib` used by `cargo test -p xtask`; not a helper crate for root contract tests.
8. Hygiene docs said skip rewriting these 16/17 targets and keep `tests/support` needle-free for **hygiene-owned** tests. That skip is C01’s collision fence, not a ban on extracting the helper for **contract** suites.

---

## Desired behavior

1. **One** crate-private definition:

   ```rust
   fn require_needles(label: &str, src: &str, needles: &[&str])
   ```

   No `pub fn` / `pub async fn` (would raise frozen `public_symbols`). Parameter name `src` (iso27001 `haystack` is the same `&str`). Semantics unchanged: missing needles → panic including `{label}` and `{missing:?}`. Domain-specific assert suffixes are not law; **needles and labels are**.

2. Home: `tests/support/mod.rs` (or `tests/support/require_needles.rs`). Include from each of the 17 binaries with `include!` and/or `#[path]` so Cargo does **not** auto-discover a test crate (`tests/support.rs` and `tests/support/main.rs` forbidden).

3. Every `require_needles(...)` call in those 17 files uses the canonical helper. Delete each per-file `fn require_needles`. **No** compatibility alias (`fn require_needles` forwarding, `use … as`).

4. Inventory matcher becomes `trimmed.starts_with("fn require_needles")`. Then `require_needles_fns` and `duplicate_helper_definitions` equal the true definition count. After extract+delete: **1**. Calls stay **222 or drop** (definition lines also match `require_needles(` today).

5. Land **one change** that adds the owner, migrates call sites, deletes copies, and tightens the matcher so `require_needles_fns` never increases mid-PR (18 → 19 would fail Guard 04).

6. Dual-suite: baseline **GREEN** on CURRENT (17 copies + inventory 18). Target **RED** on CURRENT (owner missing / copies present / count ≠ 1), then **GREEN** after implement. After GREEN, **delete** `sdd_consolidation_c01_baseline.rs` (`INV-NO-SUPERSEDED-BASELINES`; do not `#[ignore]`). Keep the **target** as the uniqueness pin. Do not rebase `consolidation-baseline.json` (Phase 0 freeze stays 18; live 1 is a legal decrease).

7. Refresh `docs/debt/current.md` via `cargo xtask inventory`. Update DUP-002 from evidence (`canonical_owner`, enumerated consumers, empty/gone duplicates, `status` **verified** only if close law holds, `guard` citing inventory uniqueness + Guard 04 — not Guard 16). `debt_id` on the row may be `C01`.

8. Counts **must not increase**: `public_symbols`, duplicate types, `root_test_binaries`, `tests_rs_autodiscovered`, `tests_contracts_rs`, `duplicate_helper_definitions` / `require_needles_fns` (except the intended drop to 1), `KNOWN_CHECK_IDS` len, ADR count.

---

## Work (implementation slice; not this spec)

1. Prove CURRENT: `rg` 17 defs in `tests/contracts`; inventory 18 files / 222 `require_needles(`.
2. Declare canonical owner as above.
3. Enumerate the 17 consumers (table).
4. Migrate every call site.
5. Delete the 17 copies in the same change as the extract.
6. Prefer inventory `starts_with` + staying C01 target + Guard 04 expansion freeze over new source-string tests. Keep existing **product** needles; do not add a second helper-uniqueness grep crate.
7. Verify commands below; update DUP-002; close only if close law holds.

---

## Dual-suite protocol

| File | Role | CURRENT | After implement |
| --- | --- | --- | --- |
| `xtask/tests/sdd_consolidation_c01_baseline.rs` | Characterization | GREEN (debt exists: 17 copies, inventory 18) | **Deleted** after target GREEN |
| `xtask/tests/sdd_consolidation_c01_target.rs` | Desired uniqueness | RED | GREEN; **keep** |

Target must fail on CURRENT because copies/owner/count are wrong — not because unrelated product modules moved. Prefer `InventoryReport::collect` and file-existence over extra needles. Existing contract needles remain migration protection.

xtask tests are auto-discovered (`xtask/tests/*.rs`). Do not add `[[test]]` in root or xtask `Cargo.toml`.

---

## Close law (DUP-002)

| Clause | Done when |
| --- | --- |
| Canonical owner exists | `tests/support/mod.rs` (or `require_needles.rs`) defines the single `fn require_needles` |
| Consumers migrated | All 17 `sdd_*_target` binaries include/call that helper; no local def |
| Old path gone | Zero `fn require_needles` in `tests/contracts/*.target.rs`; no aliases |
| Regression guard | Inventory `require_needles_fns == 1` after `starts_with`; `duplicate_helper_definitions` aliases it; Guard 04 still forbids increase; C01 **target** stays |

Green contract tests with 17 remaining copies are **not** done.

---

## Acceptance criteria

- Baseline GREEN on CURRENT: 17 `tests/contracts` copies; live `require_needles_fns == 18` with today’s `contains` matcher.
- Target RED on CURRENT, then GREEN: exactly one `fn require_needles` (`starts_with`); 17 consumers migrated; copies gone; no `tests/sdd/`; no new root/xtask `[[test]]`; no Guard 16; no `pub fn` helper.
- Signature `fn require_needles(label: &str, src: &str, needles: &[&str])`; iso27001 uses it with the same types; existing needle lists unchanged.
- `tests/support/` is a directory (not `tests/support.rs` / `main.rs`); `tests_rs_autodiscovered` stays 16; `root_test_binaries` stays 45; `tests_contracts_rs` stays 43.
- `require_needles_fns` and `duplicate_helper_definitions` are 1; `require_needles_calls` ≤ 222; `public_symbols` does not increase vs 2022.
- `cargo test -p xtask` green (after target GREEN and baseline delete).
- All 17 `cargo test --test sdd_<name>_target` binaries green.
- `cargo xtask guard` (checks 01–15) green; `cargo xtask inventory` refreshes `docs/debt/current.md`.
- DUP-002 updated from evidence; `verified` only if close law holds.
- After GREEN, C01 baseline file deleted (not `#[ignore]`).
- Master program spec unchanged except a one-line C01 pointer.

---

## Out of scope

- C04–C09 (readiness / applicability / lineage / SoA / temporal)
- DUP-003 filesystem helpers (`manifest_dir`, `read_repo_file`, `crate_sources_joined`, `text_has`, `forbid_needles`)
- `tests/sdd/`, new root or xtask `[[test]]`, Guard 16 / `KNOWN_CHECK_IDS` length 16
- Product types, persistence, public API, new crates, pnpm / `apps/docs`
- Rewriting the master consolidation program spec (one-line C01 pointer only)
- New ADR (uniqueness is not promoted to architecture law)
- Rebasing `docs/debt/consolidation-baseline.json`
- Weakening needles, skipping contract tests, or replacing needle asserts with semantic tests (C16)
- Hygiene-owned suites calling `require_needles`

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Adding `tests/support.rs` raises `tests_rs_autodiscovered` 16 → 17 | Directory + `include!`/`#[path]` only; no `main.rs` |
| `pub fn require_needles` raises `public_symbols` | Crate-private `fn`; `include!` so callers need no `pub` |
| `starts_with("fn require_needles")` would miss `pub(crate) fn` and count 0 | Definition must start with `fn require_needles` |
| Extract without delete raises `require_needles_fns` | One change: owner + migrate + delete + matcher tighten |
| Expansion freeze still allows growth 1 → 18 | Keep C01 **target** asserting `== 1` |
| `include!` in 17 files re-embeds source and double-counts | Inventory reads files on disk; definition lives only under `tests/support/` |
| Target RED for wrong reason (product needles) | Target asserts helper uniqueness / owner / copies, not catalog/IR surface |
| `#[ignore]` leftover baseline | Delete the baseline file after GREEN |
| Hygiene spec collision (`tests/support` must not introduce needles) | Helper is for the 17 contract binaries; hygiene suites still must not call it |

---

## Verify (implementation; not this spec)

```text
cargo test -p xtask
cargo test --test sdd_assessment_lineage_target --test sdd_control_implementation_registry_target --test sdd_controlled_documents_target --test sdd_continuity_resilience_target --test sdd_iso27001_assurance_target --test sdd_interested_parties_obligations_target --test sdd_incident_governance_target --test sdd_nonconformity_capa_target --test sdd_internal_audit_target --test sdd_operational_soa_target --test sdd_population_runtime_target --test sdd_remediation_engine_target --test sdd_risk_register_target --test sdd_supplier_risk_target --test sdd_temporal_lineage_evidence_soa_target --test sdd_temporal_assurance_target --test sdd_typed_evidence_target
cargo xtask guard
cargo xtask inventory
```

Then evidence-edit `docs/debt/current.md` (generated) and DUP-002 in `docs/debt/structural-duplication.toml`.
