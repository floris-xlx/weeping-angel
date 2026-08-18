# SDD run: ISO 27001 Automated Assurance MVP

| Field | Value |
| --- | --- |
| Run id | `sdd-08804180dc82` |
| Date | 2026-08-18 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `08804180dc82965f` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | ISO 27001:2022 vertical on the six-crate assurance spine (pack, ledger, TestExpr, collectors, readiness/SoA, CLI) |
| Spec | [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md) |
| ADR | [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Telemetry | [`sdd-iso-27001-automated-assurance-mvp-telemetry.json`](sdd-iso-27001-automated-assurance-mvp-telemetry.json) |
| Dual-suite | `tests/sdd/iso27001_assurance.baseline.rs` (retired / `#[ignore]`) · `tests/sdd/iso27001_assurance.target.rs` (active) |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

---

## Spec

- **Title:** ISO 27001 Automated Assurance MVP
- **Problem:** The six-crate assurance spine exists but cannot produce a useful ISO 27001:2022 readiness assessment: catalogs are empty stubs, only `FixtureCollector` ships, tests are presence/freshness only, there is no ledger/pack/CLI, and callers still have no safe end-to-end assess path that is not a certification claim.
- **Current behavior (pre-vertical, 8c0f36ed):** `compile_framework` runs the eight-stage pipeline against an in-memory Assessment; `stub_catalog` returns `[]`; facade `assess` hard-codes `canonical:stub-1` / `canonical.source-control` with a partial mapping and `branch_protection`; `EvidenceEnvelope` is `{observation, provenance, digest}` with `BTreeMap<String,String>` facts; only sync `FixtureCollector` exists; `evaluate` is a 4-state presence/break/freshness/manual-attestation function; `AssessmentReport` is `{assessmentId, profile, digest, results, evidenceCount}`; `Commands` has no Assurance variant. ACT-001..015 and COL-001..006 are green.
- **Desired behavior:** A versioned structural ISO 27001:2022 pack compiles deterministically into canonical controls and tests; GitHub/local/manual/scanner evidence is sealed into an immutable ledger; a bounded provider-blind `TestExpr` evaluator yields richer fail-closed effectiveness; readiness and SoA are projections (never certified/compliant); snapshots compare; CLI `weeping-angel assurance assess --framework iso-27001` traces requirement→mapping→control→test→envelope→collection run. IR types are consumed via contracts, not forked.
- **ADR:** needed — accepted at [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md)

### Acceptance criteria (this slice)

1. Dual-suite registered in root `Cargo.toml`; baseline GREEN on stub spine; target RED until pack/ledger/DSL/GitHub/local/manual/readiness/SoA/CLI land then GREEN.
2. ACT-001..015 and COL-001..006 remain green; `cargo test --workspace --features demo` stays green.
3. Versioned ISO 27001:2022 pack compiles deterministically with stable `FrameworkPackDigest`.
4. Public pack has no protected ISO normative text (ISO-002).
5. ≥20 canonical automated/hybrid controls; no `iso27001.*` or GitHub-specific test ids (ISO-004).
6. GitHub collector passes GH-001..012; 403 is permission-denied not false; no token leakage; descriptor has evidence types only.
7. Scanner bridge remains one-way; vuln present may be Ineffective; empty scan is not Effective.
8. Local collector and manual evidence add work; attestation is never synthesized.
9. Immutable ledger + digest artifacts + collection runs; dedupe by digest; no `set_compliant`/`set_control_status`.
10. Control-Test DSL is provider-blind, network-free, fail-closed on missing/stale/type-mismatch/manual (CTL-001..012).
11. Partial mappings cannot fully satisfy a requirement (ISO-005); ACT-005 semantics preserved.
12. Readiness is not a single percentage; SoA preserves applicability rationale (ISO-010).
13. Assessment snapshots are immutable and comparable.
14. CLI family `framework`/`collect`/`evidence`/`assess`/`result`/`compare`/`soa` without leaking compiler topology.
15. Reports never emit certified/compliant/audit passed; explicit not-certification banner.
16. Framework/compiler/control-test stay network-free; no secrets in evidence or logs.
17. Every automated result traces requirement→mapping→control→test→evidence requirement→envelope→collection run.
18. Workspace verify remains `cargo test --workspace --features demo`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Out of scope

- Canonical Compliance IR redesign (concurrent program)
- SOC 2 / GDPR / NIS2 / DORA / ISO 27701 production catalogs
- AWS / Azure / GCP / Cloudflare / Vercel / Okta / Workspace collectors
- HRIS, MDM, Trust Center, questionnaires, vendor risk, auditor portal, SaaS RBAC/billing
- Full ISO 27007 audit engine and certification-ready formal SoA
- Rewriting `SemanticFinding`/`EngineHit` or adding framework fields
- Treating empty scan / `coverage.complete` as a control pass
- Second remediation authority or second report engine
- Live GitHub as a required unit-test dependency
- Automated certified/compliant claims

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Concurrent IR rebase vs forked types | IR types consumed via contracts; no fork of Canonical Compliance IR. |
| ISO copyright in a public pack | Structural pack only; ISO-002: no protected ISO normative text. |
| GitHub checks becoming ISO checks | Collectors advertise evidence types; no `iso27001.*` or GitHub-specific test ids (ISO-004). |
| Partial mappings treated as full satisfaction | ISO-005 + ACT-005: partial cannot fully satisfy a requirement. |
| Absence of vulns treated as Effective | Scanner bridge one-way; empty scan is not Effective. |
| 403/permission holes scored Ineffective | GH-001..012: 403 is permission-denied, not false. |
| Secrets in artifacts or logs | No token leakage; no secrets in evidence or logs. |
| Expression runtime becoming a script host | Bounded provider-blind `TestExpr`; network-free, fail-closed. |
| Ledger storing control conclusions | Immutable ledger stores envelopes/runs; no `set_compliant`/`set_control_status`. |
| Certification-shaped UX copy | Reports never emit certified/compliant/audit passed; explicit not-certification banner. |
| Network deps leaking into framework/control-test | Framework/compiler/control-test stay network-free. |
| Shared-type thrash across swarms / one mega-PR | Slice scoped to ISO vertical; spine (ADR 0001) unchanged as the IR host. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md) |
| Baseline | PASS on old | `cargo test --workspace --features demo --test sdd_iso27001_assurance_baseline --test sdd_iso27001_assurance_target` → exit 0. Baseline: **21 passed**; target: **1 passed; 7 ignored**. Characterizes stub spine (empty catalogs, hard-coded facade assess, string-fact envelopes, sync FixtureCollector only, 4-state evaluator, no Assurance CLI). Target binary registered so the assigned command can run; ISO/EVD/CTL/GH checks stay ignored until MVP. No feature code added. |
| Target pre | FAIL on old | Same command. Baseline still **21 passed**. Target: **FAILED. 2 passed; 47 failed; 0 ignored**. Two target passes intentional: Cargo.toml registration and existing ISO-006 `CapabilityViolation`. Product code unchanged. Excerpts: `ISO-001: expected versioned pack at .../frameworks/iso-27001/2022/manifest.toml`; `CLI must accept assurance assess --framework iso-27001 --scope .`; `GH-001…012: expected collector module at .../weeping-angel-collector/src/github`; `CTL-001 is missing required surface ["enum TestExpr", "FreshWithin", "CoverageAtLeast", "ManualReview", "EvidenceSelector"]`. |
| Implement | target PASS | Same command after pack/ledger/TestExpr/collectors/readiness/SoA/CLI. Target: **49 passed; 0 failed**. Also `sdd_assurance_runtime_target` **21 passed** (ACT-001..015, COL-001..006). Baseline default run: **0 passed; 21 ignored** (superseded stub spine). |
| Baseline post | FAIL or retired | Retired via `#[ignore]` (`supersede_kind=skip`). Default: **0 passed; 21 ignored**. Forced `--ignored`: **FAILED. 5 passed; 16 failed**. Examples: `stub_catalog_is_empty` panicked `stub_catalog(Iso27001) currently returns []`; `no_on_disk_framework_pack` `current tree has no frameworks/ pack tree`; `commands_has_no_assurance_variant` left includes `assurance`; `evaluate` left `StaleEvidence` right `Inconclusive`. Not additive. File stays registered. |
| Supersede | target still PASS | After skip-supersede: target **49/49** still GREEN. Baseline not the CI gate. Delete/move would break Cargo.toml dual-suite registration and the SDD rollback narrative. |
| Docs/ADR | updated | [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md), [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), [`docs/sdd/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/sdd/sdd-assurance-runtime-spine.md`](sdd-assurance-runtime-spine.md), [`README.md`](../../README.md), [`frameworks/README.md`](../../frameworks/README.md), [`codex-security/references/scan-contract.md`](../../codex-security/references/scan-contract.md) |

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

First ISO 27001 assurance vertical on the existing six-crate spine:

- Versioned structural pack at `frameworks/iso-27001/2022/` (manifest, requirements, mappings, applicability, metadata) plus fixture pack under `fixtures/assurance/iso27001/`.
- Deterministic compile to canonical controls/tests with stable `FrameworkPackDigest`; public pack has no protected ISO normative text.
- Immutable evidence ledger + digest artifacts + collection runs; dedupe by digest; no control-status mutators.
- Bounded provider-blind `TestExpr` (`FreshWithin`, `CoverageAtLeast`, `ManualReview`, `EvidenceSelector`); network-free; fail-closed on missing/stale/type-mismatch/manual.
- GitHub / local / manual collectors; scanner bridge remains one-way; 403 is permission-denied; attestation is never synthesized.
- Readiness is not a single percentage; SoA preserves applicability rationale; assessment snapshots are immutable and comparable.
- CLI family `framework` / `collect` / `evidence` / `assess` / `result` / `compare` / `soa`. Library `assess` is the execution path; the binary prints the explicit non-certification banner.
- Reports never emit certified / compliant / audit passed.

### Files changed (implement)

`crates/weeping-angel-assurance-ir/src/lib.rs`, `crates/weeping-angel-assurance-ir/src/crosswalk.rs`, `crates/weeping-angel-framework/src/lib.rs`, `crates/weeping-angel-framework/src/pack.rs`, `crates/weeping-angel-framework/Cargo.toml`, `crates/weeping-angel-evidence/src/lib.rs`, `crates/weeping-angel-evidence/src/ledger.rs`, `crates/weeping-angel-evidence/Cargo.toml`, `crates/weeping-angel-control-test/src/lib.rs`, `crates/weeping-angel-control-test/src/expr.rs`, `crates/weeping-angel-control-test/src/run.inc`, `crates/weeping-angel-control-test/src/result.inc`, `crates/weeping-angel-collector/src/lib.rs`, `crates/weeping-angel-collector/src/github/mod.rs`, `crates/weeping-angel-collector/src/local/mod.rs`, `crates/weeping-angel-assurance/src/lib.rs`, `crates/weeping-angel-assurance/src/bridge.rs`, `crates/weeping-angel-assurance/src/readiness.rs`, `crates/weeping-angel-assurance/src/soa.rs`, `crates/weeping-angel-assurance/src/snapshot.rs`, `src/cli.rs`, `src/main.rs`, `frameworks/iso-27001/2022/manifest.toml`, `frameworks/iso-27001/2022/requirements.toml`, `frameworks/iso-27001/2022/mappings.toml`, `frameworks/iso-27001/2022/applicability.toml`, `frameworks/iso-27001/2022/metadata.toml`, `fixtures/assurance/iso27001/manifest.toml`, `tests/sdd/iso27001_assurance.baseline.rs`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-08804180dc82` |
| `agents_ok` | 7 |
| `agents_fail` | 0 |
| `agents_total` | 7 |
| `tokens_used_sum` | 12 025 104 |
| `duration_ms_sum` | 7 657 031 (~127.6 min) |
| `budget.total` | 128 |
| `budget.spent` | 7 |
| `budget.remaining` | 121 |
| `event_count` | 28 |
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
| Scope | `sdd-scope` | ok | 445 120 | 297 331 |
| Spec | `sdd-spec` | ok | 705 023 | 602 788 |
| BaselineGreen | `sdd-baseline-green` | ok | 771 316 | 964 081 |
| TargetRed | `sdd-target-red` | ok | 700 749 | 721 570 |
| Implement | `sdd-implement` | ok | 3 237 796 | 8 029 914 |
| DocsAdr | `sdd-docs-adr` | ok | 1 519 039 | 1 195 897 |
| Supersede | `sdd-supersede` | ok | 277 988 | 213 523 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-iso-27001-automated-assurance-mvp-telemetry.json`](sdd-iso-27001-automated-assurance-mvp-telemetry.json).

---

## Remaining backlog (not this slice)

1. Canonical Compliance IR redesign (concurrent program)
2. SOC 2 / GDPR / NIS2 / DORA / ISO 27701 production catalogs
3. AWS / Azure / GCP / Cloudflare / Vercel / Okta / Workspace collectors
4. HRIS, MDM, Trust Center, questionnaires, vendor risk, auditor portal, SaaS RBAC/billing
5. Full ISO 27007 audit engine and certification-ready formal SoA
6. Live GitHub as a required unit-test dependency (explicitly not required)
7. Automated certified / compliant claims (forbidden)
8. Treating empty scan / `coverage.complete` as a control pass (forbidden)

---

## Summary

ISO 27001 Automated Assurance MVP landed under dual-suite SDD: spec + ADR 0002, baseline GREEN on the stub spine (21 passed / 7 target ignored), target RED (47 failed) until pack/ledger/TestExpr/GitHub/local/manual/readiness/SoA/CLI, then target GREEN 49/49. ACT-001..015 and COL-001..006 remain green. Stub-spine baseline skip-superseded (`#[ignore]`; forced `--ignored` is not green). Target remains the only required ISO suite. Reports stay non-certification projections.
