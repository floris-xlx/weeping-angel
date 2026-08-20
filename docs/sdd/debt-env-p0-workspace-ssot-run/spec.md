# DEBT-ENV P0 — Cargo workspace SSOT (repository-environment consolidation)

| Field | Value |
| --- | --- |
| Increment | **DEBT-ENV P0** (not C01–C16; not the 32-phase env program in one PR) |
| Debt | **DEBT-ENV** (exactly one `debt_id`) |
| Workflow | `xylex-sdd-consolidation` |
| Disposition | **CONSOLIDATE** (capability exists; no feature creation) |
| Bounded context | `repository-guard` |
| Canonical owner | `xtask` + `architecture/` (repository-guard). **Do not CREATE** a crate, Turbo layer, Guard 16, second health CLI, or parallel program SSOT. |
| Governing ADR | [ADR 0009](../../adr/0009-repository-health-gate.md) (health plane). Environment ADR: [ADR 0051](../../adr/0051-repository-environment.md) (**Accepted**). |
| Master program SSOT | [`docs/specs/architectural-consolidation-program.md`](../../specs/architectural-consolidation-program.md) — **unchanged** except a one-line pointer. This file is the small run spec only. |
| Dual-suite | `xtask/tests/sdd_consolidation_debt_env_target.rs` via `cargo test -p xtask` auto-discovery. Characterization baseline **deleted**. **Never** `tests/sdd/`. **Never** `test/sdd/*.ts`. **Never** a new root or xtask `[[test]]`. |
| Classification | Frozen [`.sdd/run/sdd-cf5759e37523/classification.json`](../../../.sdd/run/sdd-cf5759e37523/classification.json) |
| Status | **Implemented** — P0 target GREEN; characterization baseline deleted; ADR 0051 Accepted. P1–P3 remain later DEBT-ENV increments. |

This is **not** a second architectural-cleanup / consolidation / 32-phase environment program spec. Laws and increment order stay in the master file and ADRs 0009–0012 / 0048–0050 plus ADR 0051. P1–P3 remain characterized CURRENT debt, not this implement surface.

---

## Classify (binding)

| Key | Value |
| --- | --- |
| `capability_exists` | `true` |
| `disposition` | `CONSOLIDATE` |
| `adr_action` | `new` |
| `new_adr_required` | `true` |
| `new_public_surface` | `false` |
| `new_persistence` | `false` |
| `debt_id` | `DEBT-ENV` |
| `create_justification` | Do not CREATE a new crate, Turbo layer, Guard 16, second health CLI, or parallel program SSOT. Collapse competing workspace / lockfile / generation / CI / test-registry representations onto architecture + xtask. |
| `close_law` | Canonical environment policy is architecture + xtask (no second health CLI, no Guard 16, no Turbo). One Cargo workspace SSOT (`[workspace.package]` / `[workspace.dependencies]`, workspace-owned internal paths); one pnpm workspace and lockfile at repo root; CLI package name remains `weeping-angel` (path may move to `apps/cli`, never `weeping-angel-assurance-cli`); install/generate/verify/build are separate commands; consumers (CI, dist, packager, apps/docs, crate manifests, contract tests asserting `Cargo.toml`) migrated; nested lockfiles, nested `pnpm-workspace.yaml`, and postinstall repo mutation removed or compatibility-only with sunset; `cargo xtask guard` enforces topology/dependency-direction/generated-artifact rules; `INV-CONSOLIDATION-EXPANSION-RESTRICTED` stays green (metrics must not increase). |

Reuse APIs: `cargo xtask guard`, `cargo xtask inventory`, `xtask::run_guard`, `xtask::ArchitectureCheck`, `xtask::RepositoryModel`, `xtask::evaluate_invariant`, `INV-CONSOLIDATION-EXPANSION-RESTRICTED`, `architecture.toml` `[ownership.repository_guard]`, `domain-ownership.toml` `[concept.repository_guard]`, `FORBID-HYPOTHETICAL-ASSURANCE-CLI`, Guard **01–15** (fold topology later; **no Guard 16**).

---

## Problem

The repository already has a health plane (`cargo xtask guard`, architecture manifests, ADRs 0009–0012 / 0048–0050), but the **workspace itself** is not a single environment policy. Root `Cargo.toml` is simultaneously workspace, publishable CLI, release/dist/packager host, and a 45-row `[[test]]` catalog. Crate metadata and internal paths are copied. CI pins `dtolnay/rust-toolchain@stable` while `rust-toolchain.toml` is absent. Nested pnpm lock/workspace and `postinstall` generation remain (P1). Product ISO/ISMS work should not expand until this environment is collapsed onto the existing owner.

User-visible goal (this increment): **boring Cargo workspace**. Virtual root workspace; package name `weeping-angel` at `apps/cli`; inherited `[workspace.package]` / `[workspace.dependencies]`; pinned toolchain consumed by CI; one integration-test harness; frozen expansion metrics do not rise. Scanner/assurance CLI behavior unchanged.

---

## Current behavior

Live tree (characterization; baseline suite encodes this):

1. **Root `Cargo.toml` is fused.** `[workspace]` members are the seven `crates/weeping-angel-*` paths plus `xtask`. The same file has `[package] name = "weeping-angel"` (`version = "0.2.0"`, `edition = "2024"`, `license = "MIT"`), `[[bin]]` `weeping-angel` → `src/main.rs` and `weeping-angel-docs-export`, `[features]` `web`/`demo`, cargo-dist / WiX / packager metadata, and **45** `[[test]]` tables (`e2e_demo`, `e2e_recon`, plus 43 `tests/contracts/*.rs` registrations). There is **no** `[workspace.package]` and **no** `[workspace.dependencies]`.
2. **`dist-workspace.toml` is a second workspace** (`members = ["cargo:."]`).
3. **Copied crate `[package]` metadata.** Each of the seven internal crates repeats `version = "0.2.0"`, `edition = "2024"`, `license = "MIT"` instead of `*.workspace = true`.
4. **Relative-path internal deps.** Internal crates declare `weeping-angel-* = { path = "../weeping-angel-*" }`. Root declares `path = "crates/weeping-angel-*"`.
5. **Empty `demo = []` features** on the seven internal crates vs CLI `demo`/`web` (P1+ characterization; not P0 implement).
6. **CLI lives at repo-root `src/main.rs` + `src/cli.rs`**, package name `weeping-angel`. `apps/cli/` does not exist. Architecture `[ownership.assurance_cli]` / `[concept.assurance_cli]` paths are `src/main.rs`, `src/cli.rs`. `FORBID-HYPOTHETICAL-ASSURANCE-CLI` forbids package `weeping-angel-assurance-cli`.
7. **Test registration is a Cargo.toml catalog** (ADR 0004 + ADR 0012). `tests/contracts/` is not auto-discovered. Inventory `root_test_binaries = 45` counts root `[[test]]` lines. Many `tests/contracts/*.target.rs` assert their `sdd_*` name appears in root `Cargo.toml`. `tests/support/mod.rs` and autodiscovered `tests/*.rs` use `env!("CARGO_MANIFEST_DIR")` as **repo root**. `xtask/Cargo.toml` has **no** `[[test]]`.
8. **No `rust-toolchain.toml`.** CI/workflows pin `dtolnay/rust-toolchain@stable`: `.github/workflows/ci.yml`, `compliance-regression.yml`, `security-diff.yml`, `release-provenance.yml`. CI inlines `cargo fmt` / `check` / `clippy` / `test` plus `cargo xtask guard` (P2 characterization).
9. **Inventory freeze:** `workspace_crates = 9` (root `[package]` + 8 members). `INV-CONSOLIDATION-EXPANSION-RESTRICTED` forbids **increase**; decrease is allowed. Live `root_test_binaries = 45`. `KNOWN_CHECK_IDS` length **15**.
10. **P1 competing JS representations (characterize only):** root `package.json` installer scripts vs `apps/docs/package.json`; root `pnpm-lock.yaml` empty importer vs `apps/docs/pnpm-lock.yaml`; `apps/docs/pnpm-workspace.yaml` (`allowBuilds` / `packageExtensions` only); `postinstall`/`dev`/`build` all run `generate:docs`.
11. **P2/P3 competing representations (characterize only):** no `cargo xtask doctor|ci|docs`; topology/dep-direction not yet Guard 01/04/15 environment rules; release SSOT split across `release.yml`, `dist-workspace.toml`, packager metadata; mixed `.gitignore` classes; ACP Phase 19 crate-graph policy unimplemented as workspace-owned deps.

### Consumers of competing representations (P0 must migrate those it collapses)

| Representation | Consumers |
| --- | --- |
| Root fused `Cargo.toml` (workspace+package+bins+release+`[[test]]`) | `cargo metadata`, `cargo xtask inventory` (`count_workspace_crates`, `root_test_binaries`), `cargo xtask guard`, `.github/workflows/ci.yml` (`--example weeping-angel-demo`, `--bin weeping-angel` via workspace), `dist-workspace.toml`, `wix/main.wxs` + `[package.metadata.wix]` / packager, root `package.json` installer scripts, `docs/specs/assurance-runtime-spine.md` (root remains package for packager/WiX/`CARGO_MANIFEST_DIR`), ADR 0001 / 0004 / 0009 / 0012, `architecture.toml` `[ownership.assurance_cli]`, `domain-ownership.toml` `[concept.assurance_cli]`, `forbidden-patterns.toml` `FORBID-HYPOTHETICAL-ASSURANCE-CLI` rationale, `tests/support/mod.rs`, `tests/*.rs`, `tests/contracts/*.target.rs` (suite listed in root `Cargo.toml`), `tests/contracts/documentation_layout.rs`, `tests/contracts/repository_integrity.target.rs`, `tests/contracts/repository_hygiene.target.rs`, `docs/contracts/README.md` (`rg` on root `Cargo.toml`) |
| Copied `[package]` metadata | `crates/*/Cargo.toml`, `xtask/Cargo.toml` |
| `path = "../weeping-angel-*"` | seven internal crate manifests; root `[dependencies]` / `[dev-dependencies]` |
| Absent `rust-toolchain.toml` / `@stable` | `ci.yml`, `compliance-regression.yml`, `security-diff.yml`, `release-provenance.yml` |
| `[[test]]` per-suite catalog | ADR 0004 rule 3; ADR 0012 §4; inventory `root_test_binaries`; C01/C02 uniqueness pins that still require consumer binaries to exist; contract targets that `contains("sdd_<name>_target")` on `Cargo.toml` (non-exhaustive: `applicability_engine`, `assessment_lineage`, `canonical_assurance_catalog`, `continuous_assurance_scheduler`, `continuity_resilience`, `control_implementation_registry`, `controlled_documents`, `documentation_layout`, `evidence_validity_temporal_assurance`, `github_collector`, `governance_catalog`, `iam_catalog`, `incident_governance`, `infrastructure_catalog`, `interested_parties_obligations`, `internal_audit`, `isms_context`, `isms_events_drift`, `iso27001_assurance`, `iso27001_remap`, `nonconformity_capa`, `personnel_security`, `population_runtime`, `remediation_engine`, `repository_hygiene`, `residual_risk`, `risk_identification`, `risk_register`, `risk_treatment`, `scope_engine`, `sdlc_catalog`, `security_objectives`, `supplier_risk`, `temporal_assurance`, `temporal_lineage_evidence_soa`, `typed_evidence`, `vulnerability_catalog`) |
| CLI path `src/main.rs` | architecture ownership, domain-ownership facade fields, packager `before-packaging-command`, `cargo build --bin weeping-angel`, `src/bin/weeping-angel-docs-export.rs`, `apps/docs/scripts/generate-cli-reference.mjs` |

P1–P3 consumers (`apps/docs/package.json`, nested lockfiles, postinstall, `cargo xtask doctor/ci/docs`, release SSOT, `.gitignore` classes) stay listed in classification.json; **not** this increment’s implement surface.

---

## Desired behavior (P0 only)

After implement:

1. **Root `Cargo.toml` is workspace-only** (virtual workspace: `[workspace]` + `[workspace.package]` + `[workspace.dependencies]` + resolver/members; **no** root `[package]` / `[[bin]]` / `[[test]]` / packager metadata).
2. **CLI package name remains `weeping-angel`** at `apps/cli/` (`apps/cli/Cargo.toml`). Path may move; name must not become `weeping-angel-assurance-cli`. `[[bin]]` `weeping-angel` and `weeping-angel-docs-export` move with the package. `src/` at repo root is gone (or is not the package). Architecture + domain-ownership paths update to `apps/cli/src/main.rs` and `apps/cli/src/cli.rs`.
3. **Members stay nine crates:** the seven `crates/weeping-angel-*` libraries + `xtask` + `apps/cli`. Do **not** add a tenth crate (no `weeping-angel-contract-tests`, no dummy root package, no Turbo package). Inventory `workspace_crates` remains **9** (virtual root does not count as a package; count members only, or members + CLI — implementer updates `count_workspace_crates` so the metric does not **increase**).
4. **`[workspace.package]`** owns at least `version`, `edition`, `license` (and may own `repository` / `authors` / `rust-version`). Member crates inherit `*.workspace = true` for those keys.
5. **`[workspace.dependencies]`** owns internal crate paths (`path = "crates/weeping-angel-…"`) and shared third-party versions used today. Consumers use `workspace = true`. **No** `path = "../weeping-angel-*"` left in `crates/*/Cargo.toml`. CLI/dev-deps for internal crates also `workspace = true`.
6. **Pinned `rust-toolchain.toml`** at repo root (`[toolchain]` channel is a **versioned** rustc, not floating `stable`; components include `rustfmt`, `clippy`). The four GitHub workflows **consume that file** and must not use `dtolnay/rust-toolchain@stable` as compiler SSOT.
7. **Test-registration collapse.** Product package has **one** `[[test]]` harness (inventory `root_test_binaries` **45 → 1**; `expected_reductions.root_test_binaries = 44`). Harness lives on package `weeping-angel` (`apps/cli`) and wires former explicit contract suites + `e2e_*` (`demo` gated with `cfg`/`required-features` **inside** the harness so contracts do not require `demo`). Repo-root `tests/*.rs` files remain (metric `tests_rs_autodiscovered` must not **increase**; decrease allowed) and still **run** via the harness or equivalent `#[path]` includes — not as 16 new `[[test]]` rows. `tests/contracts/` stays the executable-invariant directory; it is still not a second auto-discovery crate. `xtask/Cargo.toml` still has **no** `[[test]]`.
8. **Inventory** counts `root_test_binaries` from the **weeping-angel package manifest** once the root is virtual (otherwise the metric would drop to 0 for the wrong reason). Decrease to 1 is the intended reduction.
9. **Consumers migrated in the same change:** architecture/domain-ownership paths; `dist-workspace.toml` member; packager/WiX/CI binary and example paths; `tests/support/mod.rs` repo-root resolution (`CARGO_MANIFEST_DIR` is `apps/cli`); contract tests that grepped root `[[test]]` names (assert the suite still **runs** / is a harness module — not a 45-row catalog); ADR 0004 rule 3 and ADR 0012 §4 **amended** to the one-harness law; `docs/contracts/README.md` pointer; compliance-regression path filters if they assume root `Cargo.toml` is the package. No compatibility alias crate.
10. **`INV-CONSOLIDATION-EXPANSION-RESTRICTED` stays green.** No increase of frozen expansion metrics (`root_test_binaries`, `workspace_crates`, `public_structs`/`public_enums`/`pub_use_count`, `require_needles_fns` / `duplicate_helper_definitions`, schema files). Parser types stay crate-private. `KNOWN_CHECK_IDS` length stays 15. No Guard 16. No `cargo xtask health` / second CLI. No Turbo. No `tests/sdd/`.
11. **Product behavior unchanged:** scanner CLI, assurance facade, catalog/IR, collector, and docs-export **semantics** stay. This is refactor of manifests/layout/test registration only.

P1–P3 (root pnpm workspace, nested lock/workspace/postinstall removal, xtask doctor/ci/docs, topology/dep-direction guards, release SSOT, crate directory short names, empty `demo=[]` audit, `.gitignore` classes) stay **CURRENT** until a later increment.

---

## Dual-suite protocol

| File | Role | After P0 |
| --- | --- | --- |
| `xtask/tests/sdd_consolidation_debt_env_baseline.rs` | Characterization of fused workspace / missing SSOT | **Deleted** (`INV-NO-SUPERSEDED-BASELINES`; never `#[ignore]`) |
| `xtask/tests/sdd_consolidation_debt_env_target.rs` | P0 close law | **GREEN**; **keep** as uniqueness/topology pin |

Target must fail on CURRENT because the virtual workspace, `apps/cli` package, workspace tables, toolchain file, `workspace = true` edges, and one harness are absent — not because unrelated product modules moved.

xtask tests are auto-discovered (`xtask/tests/*.rs`). Do not add `[[test]]` in root or xtask `Cargo.toml`.

---

## Close law (DEBT-ENV P0)

| Clause | Done when |
| --- | --- |
| Canonical owner exists | Environment policy remains `xtask` + `architecture/` + ADR 0051; no second health CLI / Guard 16 / Turbo |
| Consumers migrated | Architecture paths, dist/packager/CI, inventory counters, `CARGO_MANIFEST_DIR` helpers, contract `[[test]]` greps, ADR 0004/0012 wording |
| Old path gone | Root is not a package; no `path = "../weeping-angel-*"`; no 45-row `[[test]]` catalog; no `@stable` compiler SSOT; no `weeping-angel-assurance-cli` |
| Regression guard | DEBT-ENV **target** stays; Guard 01–15; `INV-CONSOLIDATION-EXPANSION-RESTRICTED`; `workspace_crates == 9`; `root_test_binaries == 1`; `KNOWN_CHECK_IDS.len() == 15` |

A virtual workspace with a tenth crate, or a renamed `weeping-angel-assurance-cli`, is **not** done.

---

## Acceptance criteria

- Baseline GREEN on CURRENT: fused root workspace+package; no `[workspace.package]` / `[workspace.dependencies]`; relative internal paths; no `rust-toolchain.toml`; four workflows use `dtolnay/rust-toolchain@stable`; `root_test_binaries == 45`; `workspace_crates == 9`; CLI at `src/main.rs`; no `apps/cli/Cargo.toml`; `KNOWN_CHECK_IDS.len() == 15`.
- Target RED on CURRENT, then GREEN after implement: virtual workspace; `apps/cli` package name `weeping-angel`; `[workspace.package]` + `[workspace.dependencies]`; internal `workspace = true`; pinned `rust-toolchain.toml` consumed by CI; one `[[test]]` harness (`root_test_binaries == 1`); `workspace_crates == 9`; consumers migrated; no `weeping-angel-assurance-cli`; no Guard 16; no Turbo; no `tests/sdd/`; no new root/xtask `[[test]]`.
- `INV-CONSOLIDATION-EXPANSION-RESTRICTED` remains pass (metrics must not increase; `root_test_binaries` decrease 45→1 is required).
- `cargo test -p xtask --test sdd_consolidation_debt_env_baseline` GREEN now; `--test sdd_consolidation_debt_env_target` RED now.
- After implement: `cargo xtask guard` 01–15 pass; `cargo xtask inventory` refreshes `docs/debt/current.md`; product CLI/scanner/assurance tests still pass; DEBT-ENV baseline **deleted**.
- ADR 0051 accepted after target GREEN; ADR 0004 / 0012 amended as consumers (not a second program SSOT). Master consolidation program unchanged except a pointer.

---

## Out of scope

- P1: root pnpm workspace; removing `apps/docs` nested lock / `pnpm-workspace.yaml` / `postinstall` generation; JS CI; command split install vs generate vs verify vs build
- P2: `cargo xtask doctor|ci|docs`; topology/lockfile/postinstall guards folded into Guard 01/04/15
- P3: release/packager SSOT; crate directory short names; empty `demo=[]` removal; `.gitignore` class policy; baseline/target deletion beyond this dual-suite
- New crate, Turbo, Guard 16, `KNOWN_CHECK_IDS` length 16, second health CLI, `weeping-angel-assurance-cli`
- `tests/sdd/`, `test/sdd/*.ts`, new root or xtask `[[test]]`
- Product ISO/ISMS features, public API, persistence, catalog/IR semantics
- Rewriting `docs/specs/architectural-consolidation-program.md` into a 32-phase env program
- Rebasing `docs/debt/consolidation-baseline.json`
- `#[ignore]` leftover baseline

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Virtual workspace + `apps/cli` counted as 10th crate | Members = 7 libraries + xtask + CLI = 9; no extra test crate; inventory counts members without a phantom root package |
| `CARGO_MANIFEST_DIR` becomes `apps/cli` and contracts/fixtures break | Migrate `tests/support/mod.rs` (and remaining direct `env!` sites) to repo root = CLI manifest parent chain; same change as the move |
| One harness collides on `#[test]` names | Unique module `#[path]` names; rename colliding tests without changing assertions |
| Collapsing `[[test]]` without migrating grep consumers | Enumerated contract targets + ADR 0004/0012 + documentation_layout + inventory in the same PR |
| `root_test_binaries` drops to 0 because inventory still reads virtual root | Point the counter at the `weeping-angel` package manifest; target asserts `== 1` |
| `@stable` left in one workflow | All four `dtolnay/rust-toolchain@stable` sites must consume `rust-toolchain.toml` |
| Hypothetical `weeping-angel-assurance-cli` | Keep package name `weeping-angel`; keep `FORBID-HYPOTHETICAL-ASSURANCE-CLI` |
| Guard 16 / second CLI for topology | Fold later (P2) into Guard 01/04/15; P0 does not add a check id |
| `#[ignore]` leftover baseline | Delete after GREEN |
| Public structs in xtask inventory parser | Keep types crate-private; expansion freeze |

---

## Verify (implementation; not this spec)

```text
cargo test -p xtask --test sdd_consolidation_debt_env_baseline -- --nocapture
cargo test -p xtask --test sdd_consolidation_debt_env_target -- --nocapture
cargo xtask guard
cargo xtask inventory
```

After P0: `sdd_consolidation_debt_env_target` GREEN; characterization baseline deleted.
