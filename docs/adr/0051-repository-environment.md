# ADR 0051 — Repository environment (Cargo workspace SSOT, CLI path, toolchain pin, test harness)

| Field | Value |
| --- | --- |
| Status | **Accepted** — P0 target GREEN (`xtask/tests/sdd_consolidation_debt_env_target.rs`); characterization baseline deleted. |
| Date | 2026-08-20 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing in the assurance spine, catalog, or collector decisions. **Does not replace** the health-gate decision in [ADR 0009](0009-repository-health-gate.md) or rewrite [ADR 0010](0010-architecture-as-law.md), [ADR 0011](0011-repository-guard-governance.md), [ADR 0048](0048-structural-reconciliation.md), [ADR 0049](0049-architectural-consolidation-phase-0.md), or [ADR 0050](0050-domain-ownership-model.md) bodies. **Does not** mint a second program SSOT. **Path-fact amends** [ADR 0009](0009-repository-health-gate.md) (CLI path + harness discovery) and [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (crate home). **Amends consumers:** [ADR 0004](0004-documentation-architecture.md) (per-suite `[[test]]` catalog → one harness) and [ADR 0012](0012-repository-hygiene.md) (root listing + deferred workspace restructure for this slice). |
| Extends | [ADR 0004](0004-documentation-architecture.md), [ADR 0009](0009-repository-health-gate.md), [ADR 0011](0011-repository-guard-governance.md), [ADR 0012](0012-repository-hygiene.md) |
| Spec | [`docs/sdd/debt-env-p0-workspace-ssot-run/spec.md`](../sdd/debt-env-p0-workspace-ssot-run/spec.md) (increment). Master order stays [`docs/specs/architectural-consolidation-program.md`](../specs/architectural-consolidation-program.md). |
| Tests | `xtask/tests/sdd_consolidation_debt_env_target.rs` via `cargo test -p xtask`. Characterization baseline **deleted** (`INV-NO-SUPERSEDED-BASELINES`). |

<!-- weeping-angel-adr-meta
id = "0051"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = ["0004-documentation-architecture", "0009-repository-health-gate", "0011-repository-guard-governance", "0012-repository-hygiene", "0049-architectural-consolidation-phase-0"]
-->

> Filename **`0051-repository-environment.md`**. Cite **this file by path**. Do **not** add a `0003-repository-environment*.md` sibling or a colliding `0011-*`. Next unused unique prefix after this file is **0052**.

## Context

Hygiene ([ADR 0012](0012-repository-hygiene.md)) governs generated artifacts, ignored tests, panic budget, and schema duplication. It **did not** restructure the Cargo/pnpm workspace. Consolidation Phase 0 ([ADR 0049](0049-architectural-consolidation-phase-0.md)) froze expansion metrics and left `apps/docs` / pnpm out of scope. Before this increment, ADR 0004 required each contract suite as a root `[[test]]`.

P0 (implemented) collapsed those competing representations onto architecture + xtask. No other ADR decided:

1. Virtual vs fused root `Cargo.toml`
2. `[workspace.package]` / `[workspace.dependencies]` as dependency SSOT
3. CLI filesystem path vs package name
4. Pinned `rust-toolchain.toml` vs `dtolnay/rust-toolchain@stable`
5. One integration-test harness vs a 45-row manifest catalog

Questions this decision answers:

1. What is the **canonical environment owner** (without a second health CLI)?
2. Where does the **publishable CLI package** live, and what is its **name**?
3. Who owns **crate metadata and internal dependency paths**?
4. Who owns the **Rust toolchain** version in CI?
5. How are **contract tests discovered** after the root stops being a package?

## Decision (accepted — field-level law is the P0 increment spec)

1. **Owner.** Canonical environment policy is **architecture + xtask** ([ADR 0009](0009-repository-health-gate.md) / [ADR 0011](0011-repository-guard-governance.md)). Do **not** add Guard 16, Turbo, `cargo xtask health`, or package `weeping-angel-assurance-cli` (`FORBID-HYPOTHETICAL-ASSURANCE-CLI`). Topology/dependency-direction/generated-artifact **enforcement** folds into existing Guard **01 / 04 / 15** in later DEBT-ENV increments (P2). P0 lands the Cargo SSOT those guards will read.
2. **Virtual workspace.** Root `Cargo.toml` is workspace-only: `[workspace]`, `[workspace.package]`, `[workspace.dependencies]`. It is not a package, binary, release-metadata, or `[[test]]` registry.
3. **CLI identity.** Package **name** stays `weeping-angel`. Filesystem home is `apps/cli/`. Never `weeping-angel-assurance-cli`. Workspace members = seven `crates/weeping-angel-*` libraries + `xtask` + `apps/cli` (**9** crates). Do not add a tenth crate. Inventory `workspace_crates` must not increase vs freeze 9.
4. **Inherited metadata and workspace-owned internals.** Members inherit workspace package fields. Internal crates depend with `workspace = true` on paths declared once under `[workspace.dependencies]`. Relative `path = "../weeping-angel-*"` is debt, not law.
5. **Toolchain.** `rust-toolchain.toml` at repo root pins a **versioned** channel plus `rustfmt` and `clippy`. GitHub Actions consume that file. `dtolnay/rust-toolchain@stable` is not compiler SSOT.
6. **One test harness.** Package `weeping-angel` has **one** `[[test]]` binary that runs former root-registered contract and e2e suites. `tests/contracts/` remains the human/executable invariant directory ([ADR 0004](0004-documentation-architecture.md) path law). xtask dual-suites stay auto-discovered under `xtask/tests/*.rs` with **no** xtask `[[test]]`. Inventory `root_test_binaries` counts the CLI package’s `[[test]]` tables and must be **1** after P0 (decrease 45→1). Consumers that grepped root `Cargo.toml` for `sdd_*` names migrate in the same change.
7. **Expansion freeze.** `INV-CONSOLIDATION-EXPANSION-RESTRICTED` remains in force. Decrease is allowed; increase is not. Product CLI/scanner/assurance behavior is unchanged.
8. **Sequencing.** Full environment reconciliation is P0–P3. **This ADR’s implement slice is P0 only.** P1 (pnpm workspace, nested lock/postinstall, command separation) and P2–P3 (xtask doctor/ci/docs, topology guards, release SSOT) are subsequent increments of **DEBT-ENV**, not a parallel program document.

## Consequences

- Root `Cargo.toml` stops being a test catalog and release dumping ground.
- Moving the CLI requires migrating `CARGO_MANIFEST_DIR` fixtures, dist/packager/WiX, architecture paths, and CI example/binary invocations in the same PR.
- ADR 0004 / 0012 are amended consumers of the one-harness law; they are not a second environment SSOT.
- Later DEBT-ENV increments may extend this ADR or add a follow-on ADR; they must not create Guard 16 or a second health CLI.

## Rejected alternatives

- Package rename to `weeping-angel-assurance-cli` — forbidden hypothetical ([ADR 0009](0009-repository-health-gate.md)).
- Keeping a dummy root package so tests stay auto-discovered — fused workspace remains.
- A dedicated `weeping-angel-contract-tests` crate — 10th crate; freeze violation.
- Turbo as JS/Rust orchestrator — no JS package graph that needs it; extra layer.
- Guard 16 / `cargo xtask health` — forks the health plane ([ADR 0011](0011-repository-guard-governance.md)).
- Floating CI `@stable` forever — compiler changes underneath the repo.
- Authoring a second 32-phase program spec under `docs/specs/` — master order stays the consolidation program; this ADR + the small `docs/sdd/` increment spec are enough.
- `#[ignore]`-superseding the DEBT-ENV baseline — violates `INV-NO-SUPERSEDED-BASELINES`.

## Status

**Accepted** after `sdd_consolidation_debt_env_target` is GREEN and the characterization baseline is deleted.

## Related

- Increment spec: [`docs/sdd/debt-env-p0-workspace-ssot-run/spec.md`](../sdd/debt-env-p0-workspace-ssot-run/spec.md)
- Health gate: [ADR 0009](0009-repository-health-gate.md)
- Guard governance: [ADR 0011](0011-repository-guard-governance.md)
- Hygiene (did not restructure workspace): [ADR 0012](0012-repository-hygiene.md)
- Docs layout (consumer): [ADR 0004](0004-documentation-architecture.md)
- Consolidation freeze: [ADR 0049](0049-architectural-consolidation-phase-0.md)
