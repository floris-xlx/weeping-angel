# SDD: Canonical Assurance Catalog v1 infrastructure

| Field | Value |
| --- | --- |
| Status | **Implemented** (target suite is the CI gate; baseline absence asserts are `#[ignore]` superseded) |
| Program | Canonical Assurance Catalog v1 — catalog infrastructure |
| Dual-suite (to register at implement) | `sdd_canonical_assurance_catalog_baseline` · `sdd_canonical_assurance_catalog_target` |
| ADR | Accepted [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md) |
| Spine (still law) | [`docs/specs/assurance-runtime-spine.md`](assurance-runtime-spine.md), [`docs/specs/assurance-runtime.md`](assurance-runtime.md), ADR 0001 |
| ISO vertical (must stay green) | [`docs/specs/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), ADR 0002 |
| Repository | `floris-xlx/weeping-angel` |
| Base branch | `main` |
| Planning / baseline SHA | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |
| IR schema (do not fork) | `assurance-ir/v1` (`ASSURANCE_IR_SCHEMA`) |
| Catalog schema (this program) | `weeping-angel/canonical-catalog/v1` |
| Workspace verify | `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Trust-boundary increment (Prompt 2, **implemented**) | [`catalog-framework-readiness-trust-boundary.md`](catalog-framework-readiness-trust-boundary.md) — extends this file; does not fork ID grammar or CAT-001…016. ADR: [`0011-catalog-framework-digest-and-pin-ownership.md`](../adr/0011-catalog-framework-digest-and-pin-ownership.md) |

This document is the durable SSOT for **catalog infrastructure only**: on-disk format, loader, validator, digest, stable-ID rules, offline compilation contract, CLI surface, crate boundaries, and dual-suite protocol. It does **not** own IAM / SDLC / vulnerability / infrastructure / governance domain catalogs, ISO remapping (ISO remap), typed-evidence redesign (typed evidence — landed separately: [`docs/specs/typed-evidence.md`](typed-evidence.md)), or population runtime. The IAM family is listed in the catalog manifest; its SSOT is [`docs/specs/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md). The SDLC family (SDLC catalog) SSOT is [`docs/specs/sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md) (`catalog/canonical/v1/{controls,evidence,tests}/sdlc.toml`). The vulnerability family (vulnerability catalog) has landed; its SSOT is [`docs/specs/vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md). The infrastructure family (infrastructure catalog) SSOT is [`docs/specs/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md). The governance family (governance catalog) has landed; its SSOT is [`docs/specs/governance-canonical-assurance-catalog.md`](governance-canonical-assurance-catalog.md) (`catalog/canonical/v1/{controls,evidence,tests}/governance.toml`; first-class `evidence.manual.attestation`). Personnel-security lifecycle (Prompt 17) is additive `catalog/canonical/v1/{controls,evidence,tests}/personnel.toml`; SSOT [`docs/specs/personnel-security.md`](personnel-security.md) — it does not replace the five governance `control.personnel.*` rows.

Architecture law (frozen):

```text
Provider -> Canonical Evidence -> Canonical Test -> Canonical Control -> Framework Mapping
```

The catalog must be **framework-neutral** and **provider-neutral**.

---

## 1. Problem / user-visible goal

The six-crate assurance spine and the ISO 27001:2022 vertical exist, but there is **no versioned canonical catalog** that downstream catalog, collector, framework, and test work can consume.

Today, “canonical” controls and tests live as **thin stubs inside framework packs** (`frameworks/iso-27001/2022/metadata.toml`, and nothing of the kind under `frameworks/wa-baseline/1/`). Identifiers are pack-local (`source.branch-protection`, `test.source.branch-protection`, synthesized `ev.<evidence-type>`). There is no offline `CanonicalCatalog::{load,validate,digest}`, no `catalog/canonical/v1/` tree, and no `weeping-angel assurance catalog` CLI.

Without a catalog contract:

- every new domain catalog would invent its own files and IDs;
- ISO remapping (ISO remap) has nothing stable to map onto;
- collectors cannot target a public evidence namespace;
- framework packs keep owning the canonical library (coupling regimes to content);
- accidental `control.github.*` / `control.iso27001.*` IDs cannot be rejected at a catalog boundary.

**User-visible goal:** a versioned, offline, deterministic catalog that loads, validates, hashes, inspects, and reports statistics for a **minimal fixture**, so later agents can add content **without modifying the loader**.

```bash
weeping-angel assurance catalog validate
weeping-angel assurance catalog stats
weeping-angel assurance catalog inspect control.source.protected-branch
```

This is still **not** certification. Catalog tooling must never emit certified / compliant / audit-passed language.

---

## 2. Compatibility note (what this slice must not fork)

Pinned at planning SHA `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b`.

| Surface | Location | Rule for this slice |
| --- | --- | --- |
| `ASSURANCE_IR_SCHEMA` | IR | Stay `assurance-ir/v1`. Do not invent `assurance-ir/v2` for catalog files. |
| `FRAMEWORK_PACK_SCHEMA` | framework pack | Stay `weeping-angel/framework-pack/v1`. Catalog is a **different** schema. |
| `Control`, `Requirement`, `Mapping`, `EvidenceRequirement`, `PlannedControlTest`, `AssessmentDefinition` | IR | Do **not** redesign. Tiny compile-compat only if a new crate cannot build otherwise. |
| `ControlId` / `EvidenceRequirementId` / `ControlTestId` | IR `id.rs` | Remain **permissive** charset validators. ISO pack IDs (`source.branch-protection`) must still construct. Catalog-layer IDs are **stricter** and live in the catalog crate. |
| `validate_stable_id` / `IdError::InvalidNamespace` | IR | `InvalidNamespace` is defined and **never returned**. Do not start rejecting `control.github.*` on the IR newtype in this slice (would break ISO fixtures and remap is ISO remap). |
| ISO / wa-baseline packs | `frameworks/` | Keep owning thin canonical stubs. **Do not remap** pack IDs here. |
| Compile pipeline | `weeping-angel-framework` | Eight stages unchanged. Do not load the canonical catalog from `compile_framework` in this slice. |
| Evidence / collectors / TestExpr | evidence, collector, control-test | Unchanged. Catalog may **store** a bounded expression/selector encoding; it must not pull those crates in. |

Tiny allowed adjustments: re-exports, optional fields already on IR types, or a compile-error fix after adding a workspace member.

---

## 3. Current behavior (characterization on `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b`)

Inspected: `crates/weeping-angel-assurance-ir`, `weeping-angel-framework`, `weeping-angel-control-test`, `weeping-angel-collector`, `weeping-angel-assurance`, `frameworks/`, `src/cli.rs`, `src/main.rs`, `tests/contracts/*`, `docs/specs/*`, `docs/specs/assurance-runtime.md`.

### 3.1 Workspace verify (this SHA)

| Command | Result on current tree |
| --- | --- |
| `cargo test --workspace --features demo` | **GREEN.** Active suites include `sdd_assurance_runtime_target` (21), `sdd_iso27001_assurance_target` (49), `sdd_compliance_ir_target` (27). Spine/ISO baselines are `#[ignore]` superseded. |
| `cargo fmt --all -- --check` | **RED.** Pre-existing rustfmt grouping/import-order drift in IR, assurance crate, and `compliance_ir.target.rs`. Not catalog-related. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **RED.** Pre-existing lints (e.g. `clippy::derivable_impls` on `SubjectKind`, many `collapsible_if` / scanner-crate lints). Not catalog-related. |

Catalog implementation must not add fmt/clippy debt. Fixing the pre-existing scanner/IR hygiene is **out of scope** unless a new file cannot land clean. Dual-suite **baseline** is the characterization suite below, not “clippy green on main”.

### 3.2 No canonical catalog tree or API

- There is **no** `catalog/` directory.
- There is **no** type or function named `CanonicalCatalog`.
- No crate named `weeping-angel-canonical-catalog`.
- Workspace members are exactly: `weeping-angel-assurance-ir`, `weeping-angel-framework`, `weeping-angel-evidence`, `weeping-angel-collector`, `weeping-angel-control-test`, `weeping-angel-assurance`.
- Dual-suite binaries registered today: `sdd_assurance_runtime_*`, `sdd_iso27001_assurance_*`, `sdd_compliance_ir_target`. **No** `sdd_canonical_assurance_catalog_*`.

### 3.3 Where “canonical” content lives today

Framework packs (`weeping-angel/framework-pack/v1`):

```text
frameworks/iso-27001/2022/{manifest,requirements,mappings,applicability,metadata}.toml
frameworks/wa-baseline/1/{manifest,requirements,mappings}.toml
```

`weeping-angel-framework::pack::load_framework_pack_from`:

- reads `metadata.toml` `[[control]]` / `[[test]]` when present;
- constructs IR `Control` with pack IDs such as `source.branch-protection` (no `control.` prefix);
- constructs `PlannedControlTest` with IDs such as `test.source.branch-protection`;
- synthesizes `EvidenceRequirement` IDs as `ev.<EvidenceType>` from each test’s `required` list;
- computes `FrameworkPackDigest` via IR `canonical_digest` over a JSON body of ids/relations.

`wa-baseline/1` has **no** `metadata.toml`; mappings still target `source.branch-protection` (resolved only when that control exists in a merged pack).

`stub_catalog(profile)` still returns `[]`. ISO assess loads the on-disk **framework pack**, not a canonical catalog.

ISO-004 already forbids `iso27001.` and `.github.` **inside pack-compiled IDs**, but it **requires** prefixes `access.`, `source.`, `vulnerability.`, … — **not** `control.*`.

### 3.4 IR identity and validation

`validate_stable_id` (`crates/weeping-angel-assurance-ir/src/id.rs`):

- rejects empty / whitespace-only, `> 256` bytes, whitespace/control chars, non `{alnum . - _ : /}`, UUID-v4-shaped strings;
- **does not** enforce `control.*` / `evidence.*` / `test.*`;
- **does not** reject `control.github.*` or `control.iso27001.*`;
- `IdError::InvalidNamespace` exists and is **never produced**.

Therefore `ControlId::try_new("source.branch-protection")`, `ControlId::try_new("control.github.branch")`, and `ControlId::try_new("control.iso27001.a-8-25")` all succeed today.

IR `validate_assessment_ir` rejects duplicate IDs, dangling mappings/tests, schema mismatch on `AssessmentDefinition`. It does **not** load files from disk and does **not** apply catalog namespace law.

Digest: `canonical_digest` = SHA-256 hex of `serde_json::to_vec`; `typed_canonical_digest` prefixes `wa:assurance-ir:assurance-ir/v1:<type>:`. Canonicalization version constant is `canon/v1`.

### 3.5 CLI and execution

`src/cli.rs` `AssuranceCommand` is **exactly**:

```text
Framework | Collect | Evidence | Assess | Result | Compare | Soa
```

There is **no** `Catalog` variant. Parser tests (`cli_exposes_framework_collect_evidence_result_compare_soa`) do not mention catalog.

`src/main.rs` `Commands::Assurance(_)` prints `This is a readiness assessment and is not certification.` and returns **exit 0**. Subcommands are not dispatched.

### 3.6 Crate graph (must remain)

```text
weeping-angel-assurance-ir          (no toml/fs; no upper crates)
  ├── weeping-angel-framework       (IR + serde + toml + thiserror; no collector/SDK)
  └── weeping-angel-evidence
        └── weeping-angel-collector (evidence + IR; no framework crate)

weeping-angel-control-test          (IR + evidence; offline)
weeping-angel-assurance             (facade: framework + collector + control-test + scanner bridge)
```

Forbidden edges (ACT-003 / ACT-013 / ISO network tests):

- framework ↛ collector, control-test, reqwest, octocrab, AWS/Cloudflare SDKs;
- collector ↛ framework / ISO / GDPR / SOC2 packages;
- control-test ↛ collector / network clients;
- IR ↛ any upper crate.

`weeping-angel-framework` already depends on `toml` for **packs**. Putting the **canonical catalog loader** there would make every pack compile path own catalog I/O and would tempt pack↔catalog coupling. Putting it in collector would teach collectors about controls.

### 3.7 Control-test expressions

`weeping-angel-control-test::TestExpr` exists (Exists/Missing/comparisons/Count/FreshWithin/CoverageAtLeast/All/Any/None/Not). Catalog infrastructure may **serialize a bounded subset** of that idea as TOML. It must **not** depend on the control-test crate.

---

## 4. Desired behavior (after implementation)

### 4.1 On-disk catalog

Create:

```text
catalog/canonical/v1/
  manifest.toml
  controls/
  evidence/
  tests/
```

Schema identifier: **`weeping-angel/canonical-catalog/v1`** (repo already uses `weeping-angel/<artifact>/v1` for packs; do not reuse `assurance-ir/v1` or `weeping-angel/framework-pack/v1`).

Default root: repository-relative `catalog/canonical/v1`. CLI and `load` accept an explicit path (tests use fixtures).

### 4.2 Manifest contract

`manifest.toml` **must** identify:

| Field | Role |
| --- | --- |
| `schema` | Exactly `weeping-angel/canonical-catalog/v1` |
| catalog identity / version | e.g. `catalog.id = "canonical"`, `catalog.version = "1"` (semver-or-integer string; this slice is v1) |
| `files` / sections | Explicit lists or directory section names for controls, evidence, tests |
| digest inputs | Algorithm + which files participate |

Normative sketch (implementation may use equivalent keys; tests pin the landed keys in this SDD after implement):

```toml
schema = "weeping-angel/canonical-catalog/v1"

[catalog]
id = "canonical"
version = "1"

[files]
controls = ["controls/fixture.example.toml"]
evidence = ["evidence/fixture.example.toml"]
tests = ["tests/fixture.example.toml"]

[digest]
algorithm = "sha256"
canonicalization = "canon/v1"
```

**Fail closed** when:

- `schema` missing or not `weeping-angel/canonical-catalog/v1`;
- a listed file is missing;
- an extra `*.toml` exists under `controls/`, `evidence/`, or `tests/` that is **not** listed (digest integrity);
- listed paths escape the catalog root (`..`, absolute paths).

Loading and validation require **zero network I/O** (no `reqwest`, no DNS, no git fetch).

### 4.3 File format (controls / evidence / tests)

TOML, same family as framework packs. One or more tables per file. Downstream agents add **new files** and list them in the manifest; they do not change the loader.

**Control** (minimal fixture shape):

```toml
schema = "weeping-angel/canonical-catalog/v1"

[[control]]
id = "control.source.protected-branch"
title = "Protected default branch"
description = "The default branch rejects force-push and unreviewed merges."
objective = "Prevent unaudited changes to the protected branch."
domains = ["secureDevelopment"]
evidence = ["evidence.source.protected-branch"]
tests = ["test.source.protected-branch"]
```

**Evidence**:

```toml
schema = "weeping-angel/canonical-catalog/v1"

[[evidence]]
id = "evidence.source.protected-branch"
title = "Protected-branch observation"
evidence_type = "source.branch.protection"
collection = "automated"
criticality = "required"
```

`evidence_type` is the **collector fact kind** (existing taxonomy such as `source.branch.protection`). It is **not** a provider id and must not be `github.*` / `iso27001.*`.

**Test**:

```toml
schema = "weeping-angel/canonical-catalog/v1"

[[test]]
id = "test.source.protected-branch"
control = "control.source.protected-branch"
kind = "automated"
required_evidence = ["evidence.source.protected-branch"]
break_on = []

[test.expression]
op = "exists"
evidence = "evidence.source.protected-branch"
```

Optional `[[test.subjects]]` / selector tables must use IR-compatible kinds (`repository`, `organization`, …). Unknown `op`, unknown selector fields, or unknown `kind` **fail closed**.

This slice ships **only** the fixture above (or an equally tiny equivalent). No IAM/SDLC/vuln/infra/governance/SOC2/NIS2/DORA/ISO catalog content.

### 4.4 Loader API

Dedicated crate **`weeping-angel-canonical-catalog`** (see ADR 0003). Conceptual API (names may match crate conventions; tests should call these three operations):

```rust
impl CanonicalCatalog {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CatalogError>;
    pub fn validate(&self) -> Result<(), CatalogError>;
    pub fn digest(&self) -> Result<CatalogDigest, CatalogError>;
}
```

`load` reads + parses + structurally validates enough to build the in-memory catalog. `validate` is the full fail-closed pass (may be invoked by `load`). `digest` is a pure function of validated content.

Suggested supporting types: `CatalogManifest`, `CatalogControl`, `CatalogEvidence`, `CatalogTest`, `CatalogStats`, `CatalogError` (typed variants, not a single string).

In-memory documents may **project** into existing IR `Control` / `EvidenceRequirement` / `PlannedControlTest` **without changing those structs**. Projection is optional this slice if inspect/stats only need catalog types; if implemented, it must not require ISO fields on `Control`.

### 4.5 Stable IDs (catalog layer — public API)

Namespaces (required prefix):

```text
control.*
evidence.*
test.*
```

Grammar:

```text
kind      = "control" | "evidence" | "test"
segment   = [a-z] [a-z0-9-]*
catalog-id = kind "." segment "." segment *("." segment)
```

At least **three** dot-separated parts. Lowercase ASCII only. No `:`, `/`, `_` in catalog IDs (IR still allows those for other identities).

**Reject** (catalog validate, fail closed):

| Class | Examples |
| --- | --- |
| Wrong namespace | `source.branch-protection`, `ev.source.x`, `canonical.source-control` |
| Provider segment | `control.github.branch`, `evidence.aws.iam`, `test.cloudflare.waf` |
| Framework segment | `control.iso27001.access`, `test.soc2.cc6`, `evidence.gdpr.ropa` |
| Duplicates | two `control.*` (or evidence/test) with the same id |
| Malformed | empty, uppercase, spaces, UUID, trailing dot |
| Dangling refs | control.evidence / control.tests / test.control / test.required_evidence unknown |
| Orphaned tests | a `test.*` whose `control` is missing, **or** a test not listed in that control’s `tests` array |
| Unlisted / extra files | see §4.2 |
| Bad expressions | unknown `op`, missing required fields, selector with unknown kind/field |
| Unsupported schema | any schema ≠ `weeping-angel/canonical-catalog/v1` |

Reserved **dot-segments** (exact match, compared lowercase). Implementation must encode this list in one place and test it:

```text
# providers / platforms
github gitlab bitbucket aws azure gcp google cloudflare vercel
okta entra auth0 workspace

# frameworks / regimes
iso27001 iso27701 iso27007 soc2 nis2 dora gdpr
iso-27001 iso-27701 iso-27007 soc-2 nis-2
```

`control.identity.mfa` and `control.source.protected-branch` are legal. `control.github.mfa` is not.

**IR newtypes stay permissive.** ISO pack IDs remain valid `ControlId` values. Accidental-rename detection: the shipped fixture IDs are pinned in the target suite; renaming them without updating the pin fails the suite.

### 4.6 Digest (deterministic, offline)

`CatalogDigest` is SHA-256 hex.

Domain-separated **display** prefix (do **not** reuse `wa:assurance-ir:…`):

```text
wa:canonical-catalog:weeping-angel/canonical-catalog/v1:
```

Landed: prefix is concatenated onto SHA-256 hex of parsed JSON (see §8.2). It is not mixed into the hash input.

Payload: canonical JSON (`serde_json::to_vec` of a struct / `BTreeMap` body) whose inputs are exactly the parsed catalog documents, including:

- schema + catalog id + catalog version;
- every listed control / evidence / test **after TOML parse**, sorted by id;
- not raw file bytes (Windows CRLF / TOML key order must not change the digest).

Invariants:

- two loads of the same tree → identical digest;
- inserting a `HashMap` or walking `read_dir` without sorting must not be able to change the digest;
- changing any id, title, expression, or listed file changes the digest;
- `digest()` after `validate()` is a pure function (no time, no hostname, no RNG).

Nondeterministic serialization is a **validation failure** if detected (e.g. digest computed twice in-process differs).

### 4.7 Crate placement and dependencies

| Crate | May depend on catalog? | Catalog may depend on it? |
| --- | --- | --- |
| `weeping-angel-canonical-catalog` | n/a | IR + `serde` + `serde_json` + `toml` + `thiserror` + `sha2`/`hex` only |
| `weeping-angel-assurance-ir` | no | no (IR stays the bottom) |
| `weeping-angel-framework` | **no** this slice | **no** |
| `weeping-angel-collector` | **no** | **no** |
| `weeping-angel-control-test` | no | **no** |
| `weeping-angel-evidence` | no | no |
| `weeping-angel-assurance` | optional (not required for CLI) | no |
| root `weeping-angel` | **yes** (CLI execution) | no |

Do **not** put the loader in `weeping-angel-framework` or in IR (IR has no `toml`/`std::fs` today; adding fs to IR would pollute the identity crate).

Register the new crate in workspace `members`.

### 4.8 CLI

Extend clap **parser only** in `src/cli.rs`:

```text
AssuranceCommand += Catalog(AssuranceCatalogArgs)
AssuranceCatalogCommand = Validate { path } | Stats { path } | Inspect { control_id, path }
```

Default `path` = `catalog/canonical/v1`.

**Execution stays out of the parser.** Bounded module, e.g. `src/assurance_catalog.rs` (root) or a function on the catalog crate called from `src/main.rs`. Do not implement assess/framework/collect in this slice.

`Commands::Assurance` must **dispatch** `Catalog` (no longer a blanket stub for that arm). Non-catalog assurance subcommands may keep the existing not-certification stub **or** keep current library-only behavior; do not expand them here.

Exit codes: `0` on success; non-zero on validation failure. Still print the readiness-not-certification banner on catalog commands if that is the family convention.

### 4.9 Dual-suite protocol (HARD SDD)

Register in root `Cargo.toml` (same pattern as ISO):

```toml
[[test]]
name = "sdd_canonical_assurance_catalog_baseline"
path = "tests/contracts/canonical_assurance_catalog.baseline.rs"

[[test]]
name = "sdd_canonical_assurance_catalog_target"
path = "tests/contracts/canonical_assurance_catalog.target.rs"
```

`tests/contracts` is **not** auto-discovered.

| Gate | Suite | Expected |
| --- | --- | --- |
| Spec | this file | written before product feature code |
| Baseline on CURRENT | `sdd_canonical_assurance_catalog_baseline` | **GREEN** — characterizes §3 |
| Target on CURRENT | `sdd_canonical_assurance_catalog_target` | **RED for the right reason** — missing catalog/API/CLI/crate |
| Implement | product + fixture + CLI + crate | — |
| Target after | same target suite | **GREEN** |
| Baseline after | baseline | **FAIL** or skip-supersede (`#[ignore]`) like ISO; do not leave old “no catalog” assertions as required CI green |
| Target still | target | **GREEN** |

Fail closed if baseline cannot go green on current code, target cannot go red for the asserted absences, or target never greens within the implement loop.

### 4.10 Baseline suite contents (GREEN on CURRENT)

Assert **today’s** tree:

1. `catalog/canonical/v1` does not exist.
2. No `CanonicalCatalog::load` / `validate` / `digest` in workspace Rust (string/source scan of `crates/` + `src/`).
3. `AssuranceCommand` debug/source lists only Framework, Collect, Evidence, Assess, Result, Compare, Soa.
4. `src/main.rs` Assurance arm is the not-certification stub (exit 0; no catalog dispatch).
5. `ControlId::try_new("source.branch-protection")` is `Ok`.
6. `ControlTestId::try_new("test.source.branch-protection")` is `Ok`.
7. `ControlId::try_new("control.github.branch")` is `Ok` (IR does not reject providers).
8. `ControlId::try_new("control.iso27001.access")` is `Ok` (IR does not reject frameworks).
9. ISO `metadata.toml` still owns thin stubs (`source.branch-protection`, `test.source.branch-protection`).
10. `ASSURANCE_IR_SCHEMA == "assurance-ir/v1"`.
11. Dual-suite names are registered in `Cargo.toml` (the registration test itself may be the only baseline assertion that appears after implementers add the `[[test]]` entries — write it so registration is required and other asserts stay true on current product code).

### 4.11 Target suite contents (RED on CURRENT, GREEN after)

Use stable titles so proof tables can cite them (`P?: <exact subject>` if later reviewed). Minimum:

| ID | Assertion |
| --- | --- |
| CAT-001 | `catalog/canonical/v1/{manifest.toml,controls/,evidence/,tests/}` exists; manifest `schema` is `weeping-angel/canonical-catalog/v1`. |
| CAT-002 | `CanonicalCatalog::load` succeeds **offline** on the shipped fixture (no network feature, no env tokens). |
| CAT-003 | `digest()` twice, and after re-load, is identical; permutation of in-memory insert order does not change it. |
| CAT-004 | Duplicate control/evidence/test IDs fail closed. |
| CAT-005 | Dangling control/evidence/test references fail closed. |
| CAT-006 | Orphaned tests fail closed. |
| CAT-007 | Provider names cannot appear as catalog-ID segments (`control.github.*`, `evidence.aws.*`, `test.cloudflare.*`). |
| CAT-008 | Framework names cannot appear as catalog-ID segments (`control.iso27001.*`, `test.soc2.*`). |
| CAT-009 | Unsupported schema version fails closed. |
| CAT-010 | Malformed selectors/expressions fail closed. |
| CAT-011 | `weeping-angel assurance catalog {validate,stats,inspect <control-id>}` parse in `src/cli.rs`; inspect of the fixture control succeeds after dispatch. |
| CAT-012 | `weeping-angel-framework` Cargo.toml / resolve graph still has no collector, catalog-is-ok-to-forbid, no provider SDK (`reqwest`, `octocrab`, `aws-sdk-*`, `cloudflare`). Framework must **not** depend on `weeping-angel-canonical-catalog`. |
| CAT-013 | `weeping-angel-collector` still has no `weeping-angel-framework` and no catalog crate; no ISO/SOC2/GDPR package names. |
| CAT-014 | Catalog crate depends only on IR + toml/serde/fs/digest crates (no collector, no framework, no control-test). |
| CAT-015 | Fixture IDs are pinned (`control.source.protected-branch`, `evidence.source.protected-branch`, `test.source.protected-branch` or the landed equivalents recorded in §8 after implement). |
| CAT-016 | Dual-suite binaries registered in root `Cargo.toml`. |

One regression test per comment later must be titled `P?: <exact subject>` and encode the original found case (write test first, RED, fix, GREEN).

### 4.12 Extension points (handoff for later catalog slices)

Downstream agents must be able to rely on:

1. **Stable schema** `weeping-angel/canonical-catalog/v1` until a versioned `v2` directory exists.
2. **Stable ID conventions** in §4.5 (`control.*` / `evidence.*` / `test.*`, reserved-segment denylist).
3. **Deterministic** `load` / `validate` / `digest` with no network.
4. **Validator API** returning structured errors.
5. **Fixture examples** under `catalog/canonical/v1/{controls,evidence,tests}/` showing how to declare each kind.
6. **Extension:** add a new `*.toml`, append it to `manifest.toml` `[files]`, re-validate. Loader code stays unchanged.
7. **Non-extension:** do not put framework mappings or provider collector config in this tree (packs stay under `frameworks/`; collectors stay in `weeping-angel-collector`).
8. assessment lineage may persist `CanonicalCatalogSnapshot { schema, catalogVersion, digest }`. This slice must expose a digest string suitable for that snapshot.
9. ISO remap remaps ISO pack `to =` onto `control.*` IDs. This slice must **not** perform that remap.

After implement, update **§8 Final API** in this file with the landed function signatures, error enum, and exact fixture IDs.

---

## 5. Acceptance criteria (testable)

1. Dual-suite registered as `sdd_canonical_assurance_catalog_baseline` / `sdd_canonical_assurance_catalog_target`; baseline GREEN on current product behavior; target RED on current tree for missing catalog surfaces; after implement, target GREEN and baseline fail-or-skip-superseded.
2. `catalog/canonical/v1` exists with `manifest.toml` + `controls/` + `evidence/` + `tests/`; schema `weeping-angel/canonical-catalog/v1`.
3. `CanonicalCatalog::{load,validate,digest}` exist on a dedicated `weeping-angel-canonical-catalog` crate; load/validate perform zero network I/O.
4. Catalog digest is deterministic (CAT-003) and domain-separated from `assurance-ir/v1`.
5. Validator rejects: duplicate IDs; unknown refs; orphaned tests; malformed selectors/expressions; unsupported schema; provider/framework segments in catalog IDs; unlisted/extra section files; path escape.
6. Shipped content is a **minimal** provider- and framework-neutral fixture only.
7. CLI `weeping-angel assurance catalog validate|stats|inspect <control-id>` parses in `src/cli.rs`; execution is not inlined in the clap enum; inspect prints the named control and its linked evidence/tests.
8. Framework crate remains collector/SDK-free and does **not** depend on the catalog crate. Collector remains framework-blind and catalog-blind.
9. IR `assurance-ir/v1` is not forked; `Control` / `Requirement` / `Mapping` / `EvidenceRequirement` / `PlannedControlTest` / `AssessmentDefinition` are not redesigned; ISO/wa-baseline pack IDs are not remapped.
10. Existing `sdd_assurance_runtime_target`, `sdd_iso27001_assurance_target`, and `sdd_compliance_ir_target` stay GREEN. `cargo test --workspace --features demo` stays GREEN.
11. Downstream can add a control/evidence/test TOML + manifest line without editing loader source.
12. No SOC 2 / NIS2 / DORA / ISO normative content in `catalog/canonical/v1`.

---

## 6. Out of scope

- Full IAM, SDLC, vulnerability, infrastructure, or governance catalogs (domain catalog families).
- Typed evidence value model (typed evidence) and population / `CoverageAtLeast` completion (population runtime).
- GitHub collector work (GitHub collector), applicability engine, assessment lineage snapshots (assessment lineage), ISO remapping (ISO remap).
- Redesign of `AssessmentDefinition`, `Control`, `Requirement`, `Mapping`, `EvidenceRequirement`, `PlannedControlTest`.
- Enforcing `control.*` on IR `ControlId` (would break ISO packs).
- SOC 2 / NIS2 / DORA / GDPR / ISO 27701 production content.
- Putting ISO clause/Annex text in the canonical catalog.
- Teaching `compile_framework` to load `catalog/canonical/v1` (later slice).
- Implementing non-catalog `assurance` subcommand execution (still a stub in `main.rs` unless already library-tested).
- Fixing pre-existing workspace `fmt` / `clippy` failures on scanner/IR files.
- Certification claims, auditor portal, licensed ISO narrative.

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Tightening IR `ControlId` breaks ISO-004 / pack load | Catalog-only ID law; IR newtypes stay permissive; remap is ISO remap. |
| Loader in `weeping-angel-framework` couples packs to catalog | Dedicated crate; CAT-012 forbids framework → catalog. |
| Catalog crate depends on collector or control-test | CAT-014; expression validated as TOML subset. |
| Raw-byte digest differs on CRLF vs LF | Parse TOML then canonical JSON; never hash raw bytes. |
| Fixture IDs collide with future IAM catalog | Use a tiny `control.source.protected-branch` (or record rename in §8); pins in CAT-015. |
| `assurance catalog` silently swallowed by main.rs stub | Target suite must **dispatch** and run validate, not only clap-parse. |
| Unlisted TOML files silently omitted from digest | Extra files under section dirs fail validate. |
| Reserved-segment list incomplete (`entra` vs `azure-ad`) | Single denylist + tests; extend list rather than heuristics. |
| Dual-suite baseline left required-green after ship | Skip-supersede like ISO (`#[ignore]`); target is the CI gate. |
| Pre-existing fmt/clippy RED confused with catalog failure | Recorded in §3.1; do not block spec; do not expand hygiene scope. |
| Accidental ISO content in fixture | Review fixture text; no `iso27001` strings in `catalog/`. |

---

## 8. Final API and file format (landed)

### 8.1 Crate

- Path: `crates/weeping-angel-canonical-catalog`
- Package: `weeping-angel-canonical-catalog`
- Rustdoc: crate root (`CanonicalCatalog`, schema/digest constants)
- Depends on: `weeping-angel-assurance-ir`, `serde`, `serde_json`, `toml`, `thiserror`, `sha2`, `hex`
- Must not depend on: framework, collector, control-test, evidence, network/provider SDKs

Workspace members (seventh crate added):

```text
weeping-angel-assurance-ir
weeping-angel-framework
weeping-angel-evidence
weeping-angel-collector
weeping-angel-control-test
weeping-angel-assurance
weeping-angel-canonical-catalog
```

Root CLI depends on the catalog crate. `weeping-angel-framework` and `weeping-angel-collector` do not.

### 8.2 Constants and API

```text
CATALOG_SCHEMA        = "weeping-angel/canonical-catalog/v1"
DIGEST_PREFIX         = "wa:canonical-catalog:weeping-angel/canonical-catalog/v1:"
DEFAULT_CATALOG_PATH  = "catalog/canonical/v1"

CanonicalCatalog::load(path) -> Result<Self, CatalogError>   # parse + validate()
CanonicalCatalog::validate(&self) -> Result<(), CatalogError>
CanonicalCatalog::digest(&self) -> Result<CatalogDigest, CatalogError>
CanonicalCatalog::stats(&self) -> Result<CatalogStats, CatalogError>
CanonicalCatalog::control(&self, id) -> Result<&CatalogControl, CatalogError>
CanonicalCatalog::projection(&self) -> Result<CatalogProjection, CatalogError>  # IR adapter; ADR 0011
CanonicalCatalog::controls() / evidence() / tests() -> &BTreeMap<…>
CanonicalCatalog::root() -> &Path
```

`CatalogDigest` displays as `{DIGEST_PREFIX}{sha256hex}`. Hex is IR `canonical_digest` of parsed documents (schema, catalog id/version, controls/evidence/tests as `BTreeMap` values — sorted by id). The prefix is concatenated for identity; it is **not** mixed into the SHA-256 input.

`CatalogError` variants: `UnsupportedSchema`, `Io`, `Toml`, `Duplicate`, `Dangling`, `Orphaned`, `Reserved`, `MalformedId`, `UnknownOperator`, `MalformedExpression`, `Unlisted`, `PathEscape`, `MissingFile`, `UnknownKind`, `UnknownControl`, `Digest`.

Control rows accept `description` or `narrative`; `automation` / `class` / `kind` collapse to `CatalogControl.automation` (default `automated`). Empty `[[test.expression]]` is valid; a present table must include a known `op`. Unknown subject `kind` is `UnknownKind`.

### 8.3 Manifest keys

```toml
schema = "weeping-angel/canonical-catalog/v1"

[catalog]
id = "canonical"
version = "1"

[files]
controls = ["controls/fixture.example.toml", "controls/identity.toml"]
evidence = ["evidence/fixture.example.toml", "evidence/identity.toml"]
tests = ["tests/fixture.example.toml", "tests/identity.toml"]

[digest]   # documentary; the crate does not parse this table
algorithm = "sha256"
canonicalization = "canon/v1"
```

Listed paths are relative to the catalog root. Extra `*.toml` under `controls/`, `evidence/`, or `tests/` fails closed. `..` / absolute listed paths fail closed. Downstream slices append files here without editing the loader.

### 8.4 Fixture IDs (CAT-015 pins)

| Kind | File | ID |
| --- | --- | --- |
| control | `catalog/canonical/v1/controls/fixture.example.toml` | `control.source.protected-branch` |
| evidence | `catalog/canonical/v1/evidence/fixture.example.toml` | `evidence.source.protected-branch` |
| test | `catalog/canonical/v1/tests/fixture.example.toml` | `test.source.protected-branch` |

### 8.5 CLI output

Parser: `src/cli.rs` (`AssuranceCommand::Catalog`, `AssuranceCatalogArgs`, `AssuranceCatalogCommand`).  
Execution: `src/assurance_catalog.rs` (not inlined in the clap enum).  
Dispatch: `src/main.rs` matches `AssuranceCommand::Catalog`. Non-catalog assurance arms keep the not-certification stub.

All catalog commands print the readiness-not-certification banner first.

```text
# validate
This is a readiness assessment and is not certification.
ok: <path>

# stats (digest token is one whitespace-separated token)
schema: weeping-angel/canonical-catalog/v1
catalog: canonical
version: 1
controls: N
evidence: N
tests: N
digest: wa:canonical-catalog:weeping-angel/canonical-catalog/v1:<hex>

# inspect <control-id>
control: control.source.protected-branch
title: …
objective: …                    # omitted when empty
evidence:
  evidence.source.protected-branch
    title: …
    evidence_type: source.branch.protection
tests:
  test.source.protected-branch
    kind: automated
    control: control.source.protected-branch
    required_evidence: evidence.source.protected-branch
```

Exit 0 on success; non-zero on validation failure or unknown inspect id.

### 8.6 Dual-suite supersession

`sdd_canonical_assurance_catalog_target` is normative.  
`sdd_canonical_assurance_catalog_baseline` absence assertions (`catalog/` missing, no crate, no CLI) are `#[ignore = "superseded by sdd_canonical_assurance_catalog_target"]`. IR permissiveness, ISO pack IDs, and crate-graph characterization tests remain required green.

---

## 9. SDD protocol (this change)

```text
Spec first (this file + draft ADR)
  → Dual-suite registered
  → Baseline GREEN on CURRENT (pre-catalog)
  → Target RED on CURRENT (right reasons)
  → Implement crate + catalog tree + CLI dispatch
  → Docs/ADR finalized to landed signatures (this revision)
  → Target GREEN (22/22)
  → Baseline absence asserts skip-superseded (8 pass / 6 ignore)
  → Target still GREEN
  → Workspace tests remain GREEN
```

Prefer delete/move/skip of baseline assertions so CI does not keep “there is no catalog” as required green. Target suite `sdd_canonical_assurance_catalog_target` is the CI gate.

---

## 10. Increment — catalog / framework / readiness trust boundary (Prompt 2)

**Status:** implemented. Product loader API in §8 remains law. Adapter: `CanonicalCatalog::projection()` → IR `CatalogProjection`.

Cleanup Prompt 2 (architectural-cleanup phases 2 + 3 + 7 + 21) **extends** this file. It does **not** fork catalog ID grammar, schema `weeping-angel/canonical-catalog/v1`, crate name `weeping-angel-canonical-catalog`, or CAT-001…016.

Increment SSOT: [`catalog-framework-readiness-trust-boundary.md`](catalog-framework-readiness-trust-boundary.md). Accepted: [`docs/adr/0011-catalog-framework-digest-and-pin-ownership.md`](../adr/0011-catalog-framework-digest-and-pin-ownership.md).

This crate remains the **only** parser of `catalog/canonical/v1` TOML. `weeping-angel-framework` must not grow a second `discover_catalog_index` (or equivalent) and must not depend on this crate. Packs consume `CatalogProjection` (named load via `inventory` `WorkspaceCatalogLoader`; explicit `load_framework_pack_from_with`). Nested expression `op` values are validated recursively. Catalog loading stays fail-closed; silent `continue` / `Option` drops are defects.

Dual-suite home stays `tests/contracts/canonical_assurance_catalog.{baseline,target}.rs`. Do not create `tests/sdd/`.
