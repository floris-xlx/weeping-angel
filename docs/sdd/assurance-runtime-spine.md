# SDD: Weeping Angel inwardly extensible assurance runtime (Phases 0–8)

| Field | Value |
| --- | --- |
| Status | **Implemented** (Phases 0–8 spine). ISO 27001 vertical: ADR 0002 / [`iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md). Canonical catalog infrastructure: ADR 0003 / [`canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) (seventh crate; not part of Phases 0–8). |
| Slice | Phases **0–8** (key checkpoint) |
| Dual-suite | Baseline characterized pre-spine HEAD → Target RED → implement → Target GREEN → **baseline superseded** |
| ADR | Accepted [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Head | Workspace: root scanner `weeping-angel` + assurance crates under `crates/` (six from this slice; `weeping-angel-canonical-catalog` is ADR 0003). ISO pack + `assurance` clap family: ADR 0002. |
| Analog | Athena query/compiler: `Statement` IR → `CompileTarget` + capabilities → fail-closed `compile` → dialect adapters internally |

This document is the durable SDD for the assurance spine. Phases 0–8 landed as six workspace crates. Later phases must not invent a Vanta-style pile of framework-specific checks. Machine contract: [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md).

---

## 1. Problem / user-visible goal

Weeping Angel is a working authorized security toolchain (web recon/DAST, code SAST, depcheck) that emits Codex Security–compatible **security** documents. Organizations that already run those scanners still need ISO 27001 / ISO 27701 / GDPR / SOC 2 / NIS2 / DORA assurance.

If we bolt `finding.iso_27001` / `finding.gdpr` / `finding.soc2` onto `SemanticFinding`, or let GitHub/AWS collectors print “ISO 27001 compliant”, the product becomes a framework-shaped check pile: every new regime forks the scanner, partial mappings look like equivalence, and absence of a vuln is misread as a control pass.

**User-visible goal:** one public assurance contract. Callers select a **profile** and **capabilities**, collect **observations**, and receive **control-test results**. They must not know or care whether ISO 27001, GDPR, or SOC 2 are separate implementations internally. Framework adapters stay inside the compiler. Capabilities are the external language.

---

## 2. Pre-spine characterization (baseline, superseded)

Frozen 2026-08-18 against the scanner-only tree. Kept as rollback narrative. **`sdd_assurance_runtime_baseline` is ignored** after target GREEN. Post-spine packaging and crates are §2.7.

### 2.1 Packaging (pre-spine)

- Root [`Cargo.toml`](../../Cargo.toml) was a **single package** (`name = "weeping-angel"`, edition `2024`) with no `[workspace]`.
- Bins: `weeping-angel` (`src/main.rs`), `weeping-angel-docs-export`.
- Features: `default = ["web"]`, `demo` (lab + e2e).
- Integration tests are either auto-discovered under `tests/*.rs` or explicitly listed (`e2e_demo`, `e2e_recon`). **`tests/sdd/` is not auto-discovered** (must be listed).
- README test command was `cargo test --features demo`.
- `pnpm` is packager + `apps/docs` only.

### 2.2 CLI (`src/cli.rs`)

`Commands` is exactly:

`Scan`, `Finalize`, `ScanCode`, `ScanDiff`, `Workbench`, `Depcheck`, `Version`, `Completions`.

There is **no** `assurance` subcommand and no framework/profile/capability surface.

### 2.3 Security-domain types (must remain valid)

| Type | Location | Role today |
| --- | --- | --- |
| `EngineHit` | `src/engines/mod.rs` | Intermediate rule-pack hit. Fields: rule/anchor/title/severity/CWE/location/snippet. No framework fields. |
| `SemanticFinding` | `src/contract/types.rs` | Codex Security finding (camelCase JSON). Identity, fingerprints, taxonomy (category + CWE), locations, provenance, optional validation/attack_path, `extensions: Value`. **No** `iso_27001`, `gdpr`, `soc2` fields. |
| `Candidate` | `src/contract/ledger.rs` | Durable discovery row (CWE, locations, summary, evidence). Scope-checked. Unsupported extra fields fail closed. |
| `ArtifactRecord` | `src/contract/types.rs` | `{ path, sha256, mediaType }` on the sealed manifest. |
| `CoverageDocument` | `src/contract/types.rs` | Scan coverage / completeness / surfaces. Security inventory, not a SoA. |

`EngineHit::to_semantic_finding` copies security evidence into `SemanticFinding` and sets `extensions` to `{ engine, snippet, validationMethod }` only. `web_finding_to_semantic` sets `{ module, url, webFindingId }`. Neither writes compliance claims.

`finding::Finding` (web DAST) is `{ id, title, severity, url, module, description, remediation?, cwe?, evidence, found_at }`.

### 2.4 Contract spine

`src/contract` seals Codex Security v1 bundles: fingerprint → finding/occurrence ids → `findings.json` + `coverage.json` + `scan-manifest.json` + `report.md`. `tests/contract_spine.rs` proves fixture identity and empty-bundle seal.

Absence of findings (`write_no_findings_bundle`) means `coverage.completeness == "complete"` and a surface disposition `no_issue_found`. That is a **security coverage** statement, not a control effectiveness result.

### 2.5 Engines stay in place

`src/engines/*` (path traversal, cmd injection, secrets, SQLi, SSRF, XSS, authz routes, taint_lite, depcheck_engine, git_diff, web_adapt, security_md) and `src/checks/*` perform detection. They do not emit `EvidenceObservation`, `EvidenceEnvelope`, or control results. The one-way bridge lives in `weeping-angel-assurance::bridge`.

### 2.6 Negative inventory (pre-spine baseline)

Was true on the characterized HEAD; **no longer true** after this slice (except the `src/**` security-type denylist):

- No crate named `weeping-angel-assurance*` / `weeping-angel-framework` / `weeping-angel-evidence` / `weeping-angel-collector` / `weeping-angel-control-test` / `weeping-angel-assurance-ir`.
- No `iso_27001` / `gdpr` / `soc2` identifiers in `src/**/*.rs` (**still true** — findings stay security-only).
- Collectors (GitHub/AWS/Cloudflare) do not exist (**hosted** collectors still do not; `FixtureCollector` exists).
- Framework compile / control-test / crosswalk do not exist (**now implemented**).

### 2.7 Post-spine tree (implemented)

- Root remains package `weeping-angel` (scanner + bins). `[workspace]` members are the six crates under `crates/`.
- Scanner is **not** a workspace member of itself; members list only the assurance crates. Root still builds bins and integration tests.
- Dual-suite tests are listed in root `Cargo.toml` as `sdd_assurance_runtime_baseline` and `sdd_assurance_runtime_target`.
- Public facade: `weeping_angel_assurance::AssuranceEngine`.
- `Commands` still has no `Assurance` variant (CLI later).
- `cargo test --workspace --features demo` is the green bar (scanner + assurance).

---

## 3. Implemented behavior (Phases 0–8)

Weeping Angel is an **inwardly extensible polyglot assurance runtime** (spine; catalogs later):

- **Externally:** capabilities + canonical assurance contract.
- **Internally:** framework adapters (ISO/GDPR/SOC2/NIS2/DORA catalogs are compiler profiles, not public types on findings).
- **Publicly:** `AssuranceEngine::builder().collector(…).framework(…).assess(scope)` (facade). CLI topology is later and must not leak compiler/collector graphs except debug.

Athena mapping (normative analogy, not a dependency):

| Athena | Weeping Angel assurance |
| --- | --- |
| `Statement` (dialect-neutral IR) | Compliance IR in `weeping-angel-assurance-ir` |
| `CompileTarget { profile, capabilities, … }` | `FrameworkTarget { profile, capabilities, version, context }` |
| `compile(statement, target) → CompiledStatement \| QueryCompileError` | `compile_framework(assessment, target) → CompiledFramework \| FrameworkCompileError` |
| `CapabilityViolation` fail-closed | same |
| Dialects (PG/SQLite/CQL) hidden behind compile | Framework profiles hidden behind compile |
| Drivers execute compiled SQL | Collectors gather observations; control-tests consume evidence |

### 3.1 Bridge (normative)

```text
EngineHit → SemanticFinding → EvidenceObservation → Control Test → Control Result
```

`SemanticFinding` remains a **security** document. The bridge *projects* an observation; it does not annotate the finding with framework columns.

### 3.2 Governing rule

> Framework adapters internally. Capabilities externally. Canonical assurance contract publicly. CLI/app must not know whether ISO 27001, GDPR, SOC 2, NIS2, DORA, or ISO 27701 are separate implementations internally.

---

## 4. Phase 0 — five invariants (frozen)

These are law for every later phase. Tests encode them as ACT-001…005.

| ID | Invariant | Fail-closed meaning |
| --- | --- | --- |
| **INV-1** | A Finding is not a compliance result. | `SemanticFinding` / `Finding` / `EngineHit` MUST NOT grow `iso_27001` / `gdpr` / `soc2` (or siblings). A finding cannot be serialized as a control result. |
| **INV-2** | A collector cannot declare compliance. | Collectors emit observations (“GitHub branch protection is enabled”, `exposed_without_auth`). Emitting “ISO 27001 compliant” / `ControlTestResult` is a type/API error. |
| **INV-3** | A framework cannot perform network I/O. | `weeping-angel-framework` has no AWS/GitHub/Cloudflare/reqwest/tokio-net deps. Profile code is pure compile. |
| **INV-4** | A Control-Test cannot know which provider produced its evidence. | Tests consume `EvidenceSet` keyed by evidence type + asset, never `GitHubClient` / collector id as a decision input. |
| **INV-5** | A crosswalk cannot manufacture equivalence through graph traversal. | `A --partial--> B --partial--> C` never yields `A ≡ C`. Direction is preserved. Partial ≠ equivalent. |

Absence of a vulnerability **does not** prove a control Effective. Presence of a vulnerability **may** prove a control Ineffective. Stale or missing evidence cannot produce Effective. Manual controls cannot auto-pass.

---

## 5. Phase 1 — crate `weeping-angel-assurance-ir`

Framework-neutral Compliance IR (Athena `Statement` analogue).

### 5.1 Typed IDs (newtypes, stable string form)

`FrameworkId`, `FrameworkVersion`, `RequirementId`, `ControlId`, `ControlImplementationId`, `ControlTestId`, `AssetId`, `IdentityId`, `VendorId`, `ProcessingActivityId`, `EvidenceRequirementId`, `RiskId`, `ExceptionId`, `AssessmentId`, `AuditProgramId`.

IDs are deterministic (no random v4 in persisted IR identity). Schema version is explicit on every serialized document.

### 5.2 Core types (normative relationships)

```text
Requirement → Mapping → Canonical Control → Control Test → Evidence Requirement
```

- `Control` has **no** ISO-specific fields (no annex letter, no SoA clause number on the control type).
- `Requirement` stays a separate type from `Control`.
- `Mapping` carries direction, completeness (`full` \| `partial` \| `related`), and MUST NOT collapse into identity.
- Deterministic serialization (sorted maps/sets; canonical JSON for digests).
- **Forbidden in IR:** GitHub, AWS, Cloudflare, Octokit, SDK client types, HTTP request types.

### 5.3 Schema

- `schema_version` string on IR documents (start `assurance-ir/v1`).
- Serde camelCase or an explicit documented convention; digest over canonical bytes.

---

## 6. Phase 2 — crate `weeping-angel-framework`

```text
FrameworkTarget { profile, capabilities, version, context }
```

### 6.1 Profiles

`Iso27001`, `Iso27701`, `Gdpr`, `Soc2`, `Nis2`, `Dora`, and `Iso27007` (audit program). Profile is a compile **selector**, not a crate the CLI imports as a catalog.

This slice: profile **dispatch stubs** sufficient for `compile_framework` (empty/minimal catalog + capability gates). Full catalogs are Phases 9+.

### 6.2 `FrameworkCapabilities` flags

| Flag | Meaning |
| --- | --- |
| `supports_control_applicability` | Compiler may mark controls N/A |
| `supports_statement_of_applicability` | SoA projection allowed |
| `supports_privacy_processing` | Processing-activity / RoPA inputs allowed |
| `supports_risk_treatment` | Risk/treatment objects in context |
| `supports_manual_attestation` | Tests may be `Manual` (never auto-Effective) |
| `supports_sampling` | Sampling plan in audit context |
| `supports_audit_program` | ISO 27007-style program objects |
| `supports_nonconformities` | Nonconformity records in projection |

Requesting a compile step that needs a flag the target does not set → **`CapabilityViolation`** (fail-closed). Never silently drop the step.

`weeping-angel-framework` **MUST NOT** depend on AWS SDK, GitHub, Cloudflare, or reqwest.

---

## 7. Phase 3 — `compile_framework(assessment, target)`

```text
compile_framework(assessment, target) → CompiledFramework | FrameworkCompileError
```

Pipeline (fixed order):

1. **normalize** assessment + target
2. **resolve applicability**
3. **validate capabilities** (fail-closed)
4. **resolve control mappings**
5. **resolve evidence requirements**
6. **construct test plan**
7. **construct framework projection**
8. **integrity validation** (digest)

`CompiledFramework` MUST include:

- `applicable_requirements`
- `controls`
- `tests`
- `evidence_requirements`
- `validation`
- `digest` (stable over canonical serialization)

`FrameworkCompileError` includes at least `CapabilityViolation`, identity/schema errors, mapping integrity errors, and digest mismatch. Unknown profile → typed error, not a generic panic.

---

## 8. Phase 4 — crate `weeping-angel-evidence`

Immutable **`EvidenceEnvelope`**:

- payload: `EvidenceObservation` (typed evidence kind + observed facts; facts are `EvidenceValue` / `evidence-value/v1`, [ADR 0003](../adr/0003-typed-evidence-canonical-serialization.md))
- provenance (collector id, collected_at, scope, asset)
- integrity digest over canonical payload+provenance
- once sealed, bytes are append-only; mutation is a new envelope

Collectors / bridges say:

- “repository `X` has `branch_protection` enabled”
- “route `Y` is `exposed_without_auth`”

They never say “ISO 27001 compliant”.

---

## 9. Phase 5 — crate `weeping-angel-collector`

```text
trait EvidenceCollector {
    fn descriptor(&self) -> CollectorDescriptor;
    fn collect(&self, scope: &CollectorScope) -> Result<Vec<EvidenceEnvelope>, CollectorError>;
}
```

`CollectorDescriptor`:

- `id`, `version`
- `evidence_types: BTreeSet<EvidenceType>` (e.g. `branch_protection`, `repository_visibility`)
- **`frameworks` is INVALID** — the field MUST NOT exist. Adding it is an ACT failure.

Rules:

- Emit only declared evidence types.
- No framework results (`ControlTestResult`, “compliant”).
- No credentials in payloads (tokens, cookies, `Authorization` headers).
- Normalize is deterministic (same input → same observation identity + digest).
- Retry MUST NOT duplicate immutable evidence (same digest is idempotent).
- Scope fail-closed (out-of-scope asset → error or omit with explicit denial; never silently collect).

**MUST NOT** depend on ISO/GDPR/SOC2 IR types.

This slice: trait + descriptor + in-memory / fixture collector sufficient for tests. Hosted GitHub/AWS/Cloudflare collectors are Phase 15.

---

## 10. Phase 6 — scanner bridge (do not rewrite engines)

Existing engines stay in `src/engines`, `src/checks`, `src/contract`.

Add an **adapter** (owned by assurance, not a rewrite of `to_semantic_finding`) that maps:

- `EngineHit` / `SemanticFinding` / web `Finding` → `EvidenceObservation`

Rules:

- `to_semantic_finding` remains security-only.
- Bridge is one-way; observations do not write back onto findings.
- No-issue / empty scan ≠ Effective control.

---

## 11. Phase 7 — crate `weeping-angel-control-test`

Zero network I/O. No reqwest, no SDK clients.

```text
evaluate(CompiledControlTest, EvidenceSet, AssessmentContext) → ControlTestResult
```

| Situation | Allowed results |
| --- | --- |
| Matching observation that control is operating | `Effective` (if freshness + completeness hold) |
| Observation that control is broken (e.g. exposure) | `Ineffective` |
| No vuln / empty findings | **Not** `Effective` — `Inconclusive` / `InsufficientEvidence` |
| Stale evidence | cannot be `Effective` |
| Missing required evidence | cannot be `Effective` |
| Manual control, no attestation | cannot auto-pass |

`EvidenceSet` is provider-blind (INV-4).

---

## 12. Phase 8 — compliance graph / crosswalk

- Edges have **direction** and **completeness**.
- `equivalent` only when an explicit full bidirectional mapping exists.
- Partial mapping never upgrades to equivalent via path length or shared neighbors.
- Graph walk for “related requirements” is allowed; walk for “therefore compliant with Y because X passed” is forbidden.

---

## 13. Crate graph (strict)

```text
weeping-angel-assurance-ir
        ├── weeping-angel-framework
        └── weeping-angel-evidence
                └── weeping-angel-collector

weeping-angel-assurance-ir + weeping-angel-evidence
        └── weeping-angel-control-test

weeping-angel-framework
  + weeping-angel-collector
  + weeping-angel-control-test
        └── weeping-angel-assurance   (facade)
```

Workspace conversion (**done**):

1. Root `Cargo.toml` is a virtual-style `[workspace]` **plus** the existing scanner `[package]` at `.` (not moved under `crates/weeping-angel`, so packager / WiX / `CARGO_MANIFEST_DIR` fixtures stay valid).
2. Members: `crates/weeping-angel-assurance-ir`, `…-framework`, `…-evidence`, `…-collector`, `…-control-test`, `…-assurance`.
3. Implemented extra edges (allowed): collector → IR **identity** types only (`AssetId`, `EvidenceType`); facade → IR + evidence + root `weeping-angel` (bridge).
4. Forbidden edges (tests): framework ↛ collector / SDKs; collector ↛ framework catalogs; control-test ↛ collector / network; IR ↛ upper crates. Facade is the public composition root.

Facade sketch (not CLI):

```rust
AssuranceEngine::builder()
    .collector(github_like_fixture)
    .framework(FrameworkTarget { profile: Iso27001, capabilities, version, context })
    .assess(scope)
```

Later CLI (out of this slice’s product requirement, specified for contract):

```text
weeping-angel assurance {frameworks,capabilities,collect,test,assess,compare,evidence show,controls show,audit}
```

Do not expose compiler/collector topology except a debug flag.

---

## 14. Phases 9–17 (specified, not implemented in this slice)

| Phase | Content | This slice |
| --- | --- | --- |
| 9 | Full ISO 27001 catalog, Annex A controls, SoA generation | Spec + profile stub only |
| 10 | ISO 27701 PIMS overlay | Spec + stub |
| 11 | GDPR processing / RoPA / DPIA hooks | Spec + stub (`supports_privacy_processing`) |
| 12 | SOC 2 TSC catalog | Spec + stub |
| 13 | NIS2 | Spec + stub |
| 14 | DORA ICT-risk overlay | Spec + stub |
| 15 | Real collectors (GitHub, AWS, Cloudflare, …) | Trait only |
| 16 | Assessment orchestrator (schedule, retry, persist) | Facade `assess` may be in-process only |
| 17 | Hosted auditor workflows, ISO 27007 program UX | `Iso27007` profile flag only |

Stubs exist so `compile_framework` can dispatch on profile without shipping catalogs.

---

## 15. Dual-suite TDD protocol

Cargo does not auto-discover `tests/sdd/`. Implementation MUST add:

```toml
[[test]]
name = "sdd_assurance_runtime_baseline"
path = "tests/sdd/assurance_runtime.baseline.rs"

[[test]]
name = "sdd_assurance_runtime_target"
path = "tests/sdd/assurance_runtime.target.rs"
```

Existing `cargo test --workspace --features demo` must stay green for scanner tests.

### 15.1 Baseline suite (GREEN on **current** tree; then superseded)

Characterize today’s product. Must **pass now** and remain the rollback characterization until target is green.

| Check | Expected on current HEAD |
| --- | --- |
| No `crates/weeping-angel-assurance*` (and sibling assurance crates) | absent |
| `Commands` has no `Assurance` variant | true |
| `SemanticFinding` serde field names exclude `iso27001` / `iso_27001` / `gdpr` / `soc2` | true |
| `EngineHit::to_semantic_finding` extensions stay security-only | true |
| `contract_spine` + engines types still construct and serialize | true |

After target GREEN, mark baseline **superseded** (file banner + `#[ignore = "superseded by sdd_assurance_runtime_target"]` or move to `tests/sdd/superseded/`). Do not delete the characterization text.

**Done:** `tests/sdd/assurance_runtime.baseline.rs` is superseded; target suite is GREEN.

### 15.2 Target suite (RED until Phases 0–8 exist; then GREEN)

Encodes ACT-001…015 and collector rules.

#### ACT register

| ID | Requirement |
| --- | --- |
| **ACT-001** | Finding is not a compliance result (INV-1). Projecting `SemanticFinding` into a `ControlTestResult` without a control-test is a compile/type error or API reject. |
| **ACT-002** | Collector cannot declare compliance (INV-2). Fixture collector returning a compliance sentence / control result fails. |
| **ACT-003** | Framework crate dependency graph has no network/SDK crates (INV-3). |
| **ACT-004** | Control-test API accepts only `EvidenceSet` + compiled test + context; no provider id field on the evaluate signature (INV-4). |
| **ACT-005** | Crosswalk: A—partial→B—partial→C is not equivalent; direction preserved (INV-5). |
| **ACT-006** | IR IDs exist; `Control` has no ISO-specific fields; `Requirement` ≠ `Control`; mapping is an explicit type. |
| **ACT-007** | `FrameworkTarget` + capability flags; missing flag → `CapabilityViolation`. |
| **ACT-008** | `compile_framework` runs the 8-stage pipeline; `CompiledFramework` has requirements, controls, tests, evidence_requirements, validation, digest. |
| **ACT-009** | `EvidenceEnvelope` is immutable; digest changes if payload mutates; observation text is not a compliance claim. |
| **ACT-010** | `CollectorDescriptor` has `evidence_types` and **no** `frameworks` field (compile-fail or schema reject). |
| **ACT-011** | Bridge: `EngineHit`/`SemanticFinding` → `EvidenceObservation` without changing `to_semantic_finding`. |
| **ACT-012** | Control-test: no I/O; empty findings ≠ Effective; stale/missing ≠ Effective; manual ≠ auto-pass; presence of break can be Ineffective. |
| **ACT-013** | Crate graph matches §13 (dep check / rustc metadata). |
| **ACT-014** | Facade `AssuranceEngine` composes collector + framework + assess; callers do not import profile catalog types to run a generic assess. |
| **ACT-015** | Existing security types remain uncollapsed: `EngineHit`, `SemanticFinding`, `Candidate`, `ArtifactRecord`, `CoverageDocument` still compile and match current serde shape (no framework fields). |

#### Collector rules (target suite)

| ID | Rule |
| --- | --- |
| **COL-001** | Emit only declared evidence types. |
| **COL-002** | No framework results in collector output. |
| **COL-003** | No credentials in payloads (scan for token-like keys / Authorization). |
| **COL-004** | Deterministic normalize (same fixture twice → same digest). |
| **COL-005** | Retry does not duplicate immutable evidence (set semantics by digest). |
| **COL-006** | Scope fail-closed. |

---

## 16. Acceptance criteria (this slice, testable)

1. Baseline suite is GREEN on the pre-implementation tree using the characterization in §15.1.
2. Target suite is RED until Phases 0–8 crates exist, then GREEN for ACT-001…015 and COL-001…006.
3. `cargo test --workspace --features demo` keeps existing scanner tests green (`contract_spine`, engines, CLI parse, e2e with `demo`).
4. Workspace members include the six assurance crates with the §13 dependency edges only.
5. `SemanticFinding` / `EngineHit::to_semantic_finding` serde and extensions remain security-only (no `iso_27001` / `gdpr` / `soc2`).
6. `compile_framework` is pure and fail-closed on capability mismatch.
7. Collectors advertise evidence types; `frameworks` is not a descriptor field.
8. Control-test cannot return `Effective` for missing, stale, or manual-without-attestation evidence; empty vuln set is not `Effective`.
9. Crosswalk refuses to treat a partial path as equivalence.
10. Public facade does not require the caller to branch on ISO vs GDPR vs SOC 2 implementations.
11. Phases 9–17 catalogs and hosted collectors are **not** required for this slice (stubs only).
12. After target GREEN, baseline is explicitly superseded.

---

## 17. Out of scope (this slice)

- Full ISO 27001 / 27701 / 27007 catalogs, GDPR RoPA product, SOC 2 TSC library, NIS2/DORA mappings (Phases 9–14).
- Production GitHub/AWS/Cloudflare collectors and credential vaults (Phase 15).
- Hosted auditor UX, sampling campaigns, nonconformity workflows (Phase 17).
- Adding `assurance` to `Commands` (specified; implement with CLI slice).
- Rewriting `src/engines`, `src/checks`, or collapsing Codex Security types.
- Collapsing `Finding` into `ControlResult`.
- pnpm / docs-site product work.
- Treating “no findings” sealed bundles as compliant.

---

## 18. Risks

| Risk | Mitigation |
| --- | --- |
| Workspace split breaks `cargo-dist`, WiX, packager paths, `CARGO_MANIFEST_DIR` fixtures | Keep scanner package identity stable; update metadata in the same implementation PR; run packager dry paths. |
| Implementers add `finding.iso_27001` for convenience | ACT-001/015 + serde field denylist in target tests. |
| Profile stubs grow into fake catalogs that always pass | Stubs must not emit `Effective`; empty catalog → insufficient evidence / compile of empty plan, not a pass. |
| Bridge silently treats no-hit as control pass | ACT-012. |
| Crosswalk “related” used as “equivalent” in reports | ACT-005; completeness enum on every edge. |
| Collector retries create duplicate envelopes | COL-005 digest identity. |
| Framework crate gains `reqwest` via a transitive “helper” | ACT-003 cargo metadata deny list. |
| Dual-suite left with both baseline and target required forever | Supersession step is mandatory after target GREEN. |

---

## 19. Implementation notes (spine landed)

- Serde camelCase; `assurance-ir/v1`; `BTreeMap`/`BTreeSet` for digest stability; SHA-256 hex via `canonical_digest`.
- Facade `assess` compiles a **canonical stub assessment** (one partial mapping + `branch_protection`) so every profile selector runs the same pipeline. Stubs must not emit `Effective` without evidence.
- `FixtureCollector` uses a fixed `collectedAt` (`2026-08-18T12:00:00Z`) so normalize is deterministic.
- `EvidenceSet` is a digest-keyed map (COL-005).
- Cargo 1.96 `PackageId` helpers in the target suite read the real crate graph for ACT-003/013.
- Do not add AWS/GitHub/Cloudflare types to IR or framework.
- Next product slices after the spine: catalogs, hosted collectors, CLI `assurance`. ISO 27001 pack + GitHub/local/manual collectors + clap family landed under ADR 0002; remaining regimes/collectors stay later work.

---

## 20. Traceability

| Artifact | Path |
| --- | --- |
| This spec | `docs/sdd/assurance-runtime-spine.md` |
| Accepted ADR | `docs/adr/0001-inwardly-extensible-assurance-runtime.md` |
| ISO 27001 vertical ADR | `docs/adr/0002-iso-27001-assurance-vertical.md` |
| Public contract | `docs/contracts/assurance-runtime.md` |
| IR | `crates/weeping-angel-assurance-ir` |
| Framework compile | `crates/weeping-angel-framework` |
| Evidence | `crates/weeping-angel-evidence` |
| Collector | `crates/weeping-angel-collector` |
| Control-test | `crates/weeping-angel-control-test` |
| Facade + bridge | `crates/weeping-angel-assurance` |
| Baseline test (superseded) | `tests/sdd/assurance_runtime.baseline.rs` |
| Target test (normative) | `tests/sdd/assurance_runtime.target.rs` |
| Athena analogue | `athena-query`: `Statement` + `CompileTarget` + `compile` + `CapabilityViolation` |
