# SDD run: inwardly extensible assurance runtime (Phases 0–8)

| Field | Value |
| --- | --- |
| Run id | `sdd-ac039e6a258f` |
| Date | 2026-08-18 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `ac039e6a258fbd93` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Phases **0–8** spine only (catalogs / hosted collectors remain stub/spec) |
| Spec | [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md) |
| ADR | [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Telemetry | [`sdd-assurance-runtime-spine-telemetry.json`](sdd-assurance-runtime-spine-telemetry.json) |
| Dual-suite | `tests/sdd/assurance_runtime.baseline.rs` (retired / `#[ignore]`) · `tests/sdd/assurance_runtime.target.rs` (active) |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** Inwardly extensible polyglot assurance runtime (Phases 0–8)
- **Problem:** Weeping Angel is a security scanner (`EngineHit` → `SemanticFinding`, sealed Codex bundles) with no assurance contract. Bolting ISO 27001 / GDPR / SOC 2 onto findings or letting collectors print “compliant” would create a Vanta-style pile of framework-specific checks and hide partial mappings as equivalence.
- **Current behavior (pre-spine):** Single-package crate (no `[workspace]`). CLI `Commands` are Scan, Finalize, ScanCode, ScanDiff, Workbench, Depcheck, Version, Completions — no assurance. `SemanticFinding` serde is security-only (no `iso_27001` / `gdpr` / `soc2`). `EngineHit::to_semantic_finding` and `web_finding_to_semantic` write engine/snippet or module/url extensions only. `Candidate` / `ArtifactRecord` / `CoverageDocument` remain Codex Security types. No `weeping-angel-assurance*` crates. Documented suite: `cargo test --features demo`.
- **Desired behavior:** Workspace of assurance crates following Athena compile architecture: framework-neutral IR; `FrameworkTarget { profile, capabilities, version, context }`; fail-closed `compile_framework`; immutable `EvidenceEnvelope` observations; collectors advertise evidence types not frameworks; engines bridge to observations without rewrite; control-tests are offline and cannot auto-pass on empty/stale/manual evidence; crosswalks preserve direction and never upgrade partial to equivalent. Public facade `AssuranceEngine::builder().collector().framework().assess(scope)`. CLI/app must not know which framework implementation ran.
- **ADR:** needed — accepted at [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md)

### Acceptance criteria (this slice)

1. Baseline suite GREEN on current (pre-spine) tree: no `crates/weeping-angel-assurance*`; `Commands` has no Assurance; `SemanticFinding` serde has no `iso_27001`/`gdpr`/`soc2`; `to_semantic_finding` stays security-only; `contract_spine` + engines types still valid.
2. Target suite encodes ACT-001..015 and is RED until the spine exists, then GREEN.
3. Collector rules GREEN: declared evidence types only; no framework results; no credentials; deterministic normalize; retry does not duplicate immutable evidence; scope fail-closed.
4. Workspace members: assurance-ir → framework + evidence; evidence → collector; ir + evidence → control-test; framework + collector + control-test → assurance facade.
5. Framework crate MUST NOT depend on AWS SDK / GitHub / Cloudflare / reqwest; collector MUST NOT depend on ISO/GDPR/SOC2 types.
6. `compile_framework` pipeline is normalize → applicability → capabilities → mappings → evidence requirements → test plan → projection → digest; `CapabilityViolation` is fail-closed.
7. `Control` has no ISO-specific fields; `Requirement` stays separate; relationship is Requirement → Mapping → Control → Control Test → Evidence Requirement.
8. Control-test: zero network I/O; absence of vuln is not Effective; stale/missing evidence cannot be Effective; manual controls cannot auto-pass; a breaking observation may be Ineffective.
9. Crosswalk: direction preserved; partial path never becomes equivalent.
10. Existing types `EngineHit`, `SemanticFinding`, `Candidate`, `ArtifactRecord`, `CoverageDocument` remain uncollapsed.
11. Facade callers do not branch on ISO vs GDPR vs SOC 2 implementations; compiler/collector topology is debug-only.
12. `cargo test --workspace --features demo` keeps existing scanner tests green.
13. After target GREEN, baseline is explicitly superseded.
14. Phases 9–17 catalogs and hosted collectors are stubs/spec only in this slice.

### Out of scope

- Full ISO 27001/27701/27007 catalogs and SoA product
- GDPR RoPA / processing-activity product
- SOC 2 TSC library, NIS2, DORA full mappings
- Production GitHub/AWS/Cloudflare collectors
- Hosted auditor workflows and sampling campaigns
- Shipping weeping-angel assurance CLI subcommands in this slice
- Rewriting `src/engines`, `src/checks`, or the Codex contract spine
- Adding `finding.iso_27001` / `finding.gdpr` / `finding.soc2`
- Treating sealed no-findings bundles as compliant
- `pnpm` / `apps/docs` product work

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Workspace split breaks cargo-dist, WiX, packager, `CARGO_MANIFEST_DIR` fixtures | Root stays the scanner package; workspace members added under `crates/`. Scanner tests (`contract_spine`, `cli_parse`, `code_engines`) kept green. |
| Convenience fields on `SemanticFinding` reintroduce framework coupling | INV-1: serde stays security-only; ACT-001/015 + `contract_spine`. |
| Profile stubs fake Effective / always-pass catalogs | Stub catalogs return `[]`; control-tests fail-closed on empty/stale/manual. |
| Bridge treats empty scans as control passes | Absence of vuln is not `Effective`. |
| Crosswalk `related` reported as equivalent | Direction preserved; partial never upgrades. |
| Collector retries duplicate `EvidenceEnvelope`s | Immutable digest; retry does not duplicate. |
| Framework crate gains network via a transitive helper | Framework MUST NOT depend on AWS/GitHub/Cloudflare/reqwest (ACT-013). |
| Baseline and target suites both remain required after supersession | Baseline skip-superseded (`#[ignore]`); target is the CI gate. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md) |
| Baseline | PASS on old | `cargo test --workspace --features demo --test sdd_assurance_runtime_baseline --test sdd_assurance_runtime_target` → exit 0. Baseline: **13 passed**; target bin was an empty harness so the invocation resolved. Excerpt: `test result: ok. 13 passed; 0 failed` / `test result: ok. 1 passed` |
| Target pre | FAIL on old | Same command → exit 1. Unresolved imports: `weeping_angel_assurance`, `weeping_angel_assurance_ir`, `weeping_angel_collector`, `weeping_angel_control_test`, `weeping_angel_evidence`, `weeping_angel_framework`. Baseline unchanged (13 passed). `could not compile weeping-angel (test "sdd_assurance_runtime_target") due to 8 previous errors` |
| Implement | target PASS | Same command after six-crate spine. Target: **21 passed** (ACT-001..015 + COL-001..006). Also `cargo test --workspace --features demo --test contract_spine --test cli_parse --test code_engines` → 38+3+3 passed. |
| Baseline post | FAIL or retired | Retired via `#[ignore]` (`supersede_kind=skip`). Default run: **0 passed; 13 ignored**. Forced `--ignored`: exit 1 — 11 passed; **2 failed** (`assurance_crates_are_absent`, `package_is_single_crate_not_workspace`). Not additive. Characterization text kept. |
| Supersede | target still PASS | After skip-supersede: target **21/21** still GREEN. Baseline not the CI gate. |
| Docs/ADR | updated | [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`README.md`](../../README.md), [`codex-security/references/scan-contract.md`](../../codex-security/references/scan-contract.md) |

### Supersede structured fields

| Field | Value |
| --- | --- |
| `supersede_kind` | `skip` |
| `baseline_retired` | `true` |
| `additive_baseline` | `false` |
| `baseline_not_green` | `true` |
| `target_still_green` | `true` |

`verify_ok` = `target_still_green` ∧ (`baseline_retired` ∧ `baseline_not_green` ∨ `additive_baseline`) = **true**.

---

## What landed

Workspace: root scanner `weeping-angel` + six crates.

```text
weeping-angel-assurance-ir
  → weeping-angel-framework
  → weeping-angel-evidence
       → weeping-angel-collector
  ir + evidence → weeping-angel-control-test
  framework + collector + control-test → weeping-angel-assurance (facade)
```

- Findings stay security-only (INV-1). No `finding.iso_27001` / `gdpr` / `soc2`.
- Collectors emit immutable `EvidenceEnvelope` observations; they advertise evidence types, not frameworks.
- `compile_framework` is fail-closed (`CapabilityViolation` stops the pipeline).
- Control-tests: zero network I/O; cannot auto-pass on empty / stale / manual evidence; absence of a vuln is not `Effective`.
- Crosswalks preserve direction; a partial path never becomes equivalent.
- Facade: `AssuranceEngine::builder().collector().framework().assess(scope)`. Callers do not branch on ISO vs GDPR vs SOC 2.
- Phases 9–17 catalogs and hosted collectors are stubs/spec only.

### Files changed (implement)

`Cargo.toml`, `Cargo.lock`, six crate trees under `crates/weeping-angel-{assurance-ir,framework,evidence,collector,control-test,assurance}/`, `tests/sdd/assurance_runtime.baseline.rs`, `tests/sdd/assurance_runtime.target.rs`, plus the docs listed above.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-ac039e6a258f` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 4 545 804 |
| `duration_ms_sum` | 5 401 141 (~90.0 min) |
| `budget.total` | 48 |
| `budget.spent` | 8 |
| `budget.remaining` | 40 |
| `event_count` | 29 |
| `max_iters` | 3 |
| `iters_used` | 0 |
| `dry_run` | false |
| `no_delta` | false |

### Gates (final snapshot)

| Gate | Value |
| --- | --- |
| `baseline_green` | true |
| `target_red` | true |
| `target_green` | true |
| `baseline_superseded` | true |
| `dry_run` | false |
| `no_delta` | false |

### Agents

| Phase | Label | Success | Duration (ms) | Tokens |
| --- | --- | --- | --- | --- |
| Scope | `sdd-scope` | ok | 358 671 | 128 035 |
| Spec | `sdd-spec` | ok | 828 985 | 705 075 |
| BaselineGreen | `sdd-baseline-green` | ok | 727 327 | 560 754 |
| TargetRed | `sdd-target-red` | ok | 707 149 | 510 847 |
| Implement | `sdd-implement` | ok | 1 294 557 | 1 659 432 |
| DocsAdr | `sdd-docs-adr` | ok | 939 102 | 600 380 |
| Iterate | `sdd-baseline-post-check` | ok | 341 991 | 259 505 |
| Supersede | `sdd-supersede` | ok | 203 359 | 121 776 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-assurance-runtime-spine-telemetry.json`](sdd-assurance-runtime-spine-telemetry.json).

---

## Remaining backlog (not this slice)

1. Full ISO 27001 / 27701 / 27007 catalogs and Statement of Applicability product
2. GDPR RoPA / processing-activity product
3. SOC 2 TSC library; NIS2 and DORA full mappings
4. Production GitHub / AWS / Cloudflare collectors
5. Hosted auditor workflows and sampling campaigns
6. Shipping `weeping-angel` assurance CLI subcommands
7. Any rewrite of `src/engines`, `src/checks`, or the Codex contract spine (explicitly forbidden as a compliance shortcut)
8. Never add `finding.iso_27001` / `finding.gdpr` / `finding.soc2` or treat sealed no-findings bundles as compliant

---

## Summary

Phases 0–8 of the inwardly extensible assurance runtime landed under dual-suite SDD: spec + ADR, baseline GREEN on the scanner-only tree, target RED on missing crates, six-crate workspace implemented until ACT-001..015 + COL-001..006 GREEN, docs/contracts updated, pre-spine baseline skip-superseded (`#[ignore]`; forced `--ignored` is not green). Target remains the only required spine suite. Follow-on ISO 27001 vertical (pack, ledger, TestExpr, GitHub/local/manual collectors, readiness/SoA, clap family) landed as ADR 0002 / `sdd_iso27001_assurance_target`.
