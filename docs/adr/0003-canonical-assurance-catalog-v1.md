# ADR 0003 — Canonical Assurance Catalog v1 (infrastructure)

<!-- weeping-angel-adr-meta
id = "0003"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-18 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. **Extends** [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) and [ADR 0002](0002-iso-27001-assurance-vertical.md). Does **not** replace pack IDs, ISO mappings, or IR `ControlId` validation. |
| Spec | [`docs/specs/canonical-assurance-catalog-v1.md`](../specs/canonical-assurance-catalog-v1.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Planning baseline | `5fa3a23a77e63e39b4a6ff142e64ff8001e0b91b` |
| Tests | `sdd_canonical_assurance_catalog_target` GREEN (CAT-001…016). Baseline absence asserts superseded / ignored. |

> Filename `0003-*` is shared with later program drafts (typed evidence, population, IAM). This file is the **accepted catalog-infrastructure** decision. Cite it by path.

## Context

ADR 0001 delivered an inwardly extensible assurance spine. ADR 0002 delivered the first ISO 27001:2022 **framework pack** and thin canonical stubs inside `frameworks/iso-27001/2022/metadata.toml` (`source.branch-protection`, `test.source.branch-protection`, synthesized `ev.<type>`).

Canonical content therefore lived in the **wrong layer**: a regime pack owned the reusable control library. Downstream Canonical Assurance Catalog v1 work (typed evidence, population, IAM/SDLC/vuln/infra/governance, GitHub collector, ISO remap) needs a **framework-neutral, provider-neutral, versioned catalog** with a public ID API and a deterministic offline digest.

Questions this decision answers:

1. Is the catalog a new schema, or a reuse of `assurance-ir/v1` / `weeping-angel/framework-pack/v1`?
2. Which crate loads it, given ACT-003 / collector-blindness?
3. Do catalog IDs replace IR `ControlId` validation in this slice?
4. How do we keep ISO packs compiling while introducing `control.*` / `evidence.*` / `test.*`?
5. What is the public digest string for later lineage snapshots?

## Decision

This is what shipped.

### 1. New artifact schema, new tree

Versioned on-disk catalog, not a framework pack and not IR documents:

```text
catalog/canonical/v1/{manifest.toml,controls/,evidence/,tests/}
```

Schema identifier: **`weeping-angel/canonical-catalog/v1`** (`CATALOG_SCHEMA`).

Do **not** fork `assurance-ir/v1`. Do **not** store canonical controls as another framework pack. Packs remain `frameworks/<id>/<version>/` with schema `weeping-angel/framework-pack/v1`. Compile still does **not** load this tree (`compile_framework` is unchanged).

Default root: repository-relative `catalog/canonical/v1`. `CanonicalCatalog::load` and the CLI accept an explicit path.

Manifest keys that the loader reads:

```toml
schema = "weeping-angel/canonical-catalog/v1"

[catalog]
id = "canonical"
version = "1"

[files]
controls = ["controls/….toml"]
evidence = ["evidence/….toml"]
tests = ["tests/….toml"]
```

`[digest]` in the shipped fixture (`algorithm = "sha256"`, `canonicalization = "canon/v1"`) is **documentary**. The crate does not parse or fail on that table; digest algorithm is fixed in code.

Fail closed when:

- `schema` is missing or not `weeping-angel/canonical-catalog/v1` (file-level schema, if present, must match);
- a listed file is missing;
- an extra `*.toml` exists under `controls/`, `evidence/`, or `tests/` that is not listed;
- listed paths escape the catalog root (`..`, absolute paths).

Load and validate perform **zero** network I/O.

### 2. Dedicated crate `weeping-angel-canonical-catalog`

Seventh workspace member. Dependencies: **IR + `toml` + `serde` / `serde_json` + `thiserror` + `sha2` / `hex`**. No framework, collector, control-test, evidence, or network/provider SDK.

| Must not depend on | Why |
| --- | --- |
| `weeping-angel-framework` | Packs must not own the catalog; catalog must not compile assessments |
| `weeping-angel-collector` | Collectors advertise evidence types, never controls |
| `weeping-angel-control-test` | Expression runtime stays provider-blind and catalog-blind; catalog validates a TOML `op` subset |
| `weeping-angel-evidence` | Observations stay off the catalog crate |
| network / provider SDKs | Offline contract |

`weeping-angel-framework` must **not** depend on the catalog crate (would couple every pack load to catalog I/O). `weeping-angel-collector` must not depend on it. The root CLI does.

Rejected alternatives:

- **Module inside IR** — IR is identity/documents only and has no `toml`/`fs`; adding them pollutes the bottom crate.
- **Loader inside `weeping-angel-framework`** — packs already load TOML; adding catalog there collapses “regime data” and “canonical library”.
- **Loader inside collector** — collectors must stay framework- and catalog-blind.

Landed API:

```text
CATALOG_SCHEMA   = "weeping-angel/canonical-catalog/v1"
DIGEST_PREFIX    = "wa:canonical-catalog:weeping-angel/canonical-catalog/v1:"

CanonicalCatalog::load(path) -> Result<Self, CatalogError>   # parse + validate
CanonicalCatalog::validate(&self) -> Result<(), CatalogError>
CanonicalCatalog::digest(&self) -> Result<CatalogDigest, CatalogError>
CanonicalCatalog::stats(&self) -> Result<CatalogStats, CatalogError>
CanonicalCatalog::control(&self, id) -> Result<&CatalogControl, CatalogError>
CanonicalCatalog::projection(&self) -> Result<CatalogProjection, CatalogError>  # IR adapter; [ADR 0011]
CanonicalCatalog::controls() / evidence() / tests() / root()
```

`load` always runs `validate` before returning. Downstream domain files are listed in the manifest; the crate does not hard-code fixture names.

### 3. Two ID layers until ISO remap

| Layer | Examples | Enforcement |
| --- | --- | --- |
| IR newtypes (`ControlId`, …) | `source.branch-protection`, `control.access.mfa` | Existing `validate_stable_id` (charset/length/no UUID). **Unchanged**. |
| Canonical catalog IDs | `control.source.protected-branch` | Catalog validator: required `control.*` / `evidence.*` / `test.*`; reserved provider/framework segments fail closed. |

Grammar: `kind.segment.segment+` (`kind` ∈ {`control`,`evidence`,`test`}), lowercase ASCII `[a-z][a-z0-9-]*` segments, at least three parts. No `:`, `/`, `_`, uppercase.

Reserved **exact** segments (any position after the kind):

```text
github gitlab bitbucket aws azure gcp google cloudflare vercel
okta entra auth0 workspace
iso27001 iso27701 iso27007 soc2 nis2 dora gdpr
iso-27001 iso-27701 iso-27007 soc-2 nis-2
```

`control.identity.mfa` and `control.source.protected-branch` are legal. `control.github.mfa` is not.

`IdError::InvalidNamespace` remains unused by IR. Tightening IR constructors here would fail ISO pack load and ISO-004’s `access.` / `source.` prefixes.

### 4. Digest and offline load

`CatalogDigest` displays as `{DIGEST_PREFIX}{sha256hex}`.

Hex is `weeping-angel-assurance-ir::canonical_digest` over parsed documents (schema, catalog id/version, controls/evidence/tests as `BTreeMap` values — sorted by id). Payload is `serde_json::to_vec`, **not** raw file bytes (CRLF / TOML key order must not change the hash).

The prefix is a **display / identity** prefix concatenated onto the hex. It is **not** mixed into the SHA-256 input (unlike IR `typed_canonical_digest`). assessment lineage snapshots must persist the full display string.

Nondeterminism is prevented by construction (`BTreeMap` / sorted file checks). There is no `NondeterministicDigest` error variant.

### 5. Validation (fail closed)

Landed `CatalogError` variants: `UnsupportedSchema`, `Io`, `Toml`, `Duplicate`, `Dangling`, `Orphaned`, `Reserved`, `MalformedId`, `UnknownOperator`, `MalformedExpression`, `Unlisted`, `PathEscape`, `MissingFile`, `UnknownKind`, `UnknownControl`, `Digest`.

Rejected: duplicate IDs; unknown control/evidence/test references; tests not listed on their control; unknown `op`; missing `op` when an expression table is present; unknown subject `kind` (IR `SubjectKind::parse_name`); unsupported schema; reserved provider/framework segments; unlisted extra section files; path escape; missing listed files.

Empty `expression` is allowed (declared later / hybrid-manual). Unknown `op` values fail closed. Allowed ops include the bounded subset (`exists`, `missing`, comparisons, `count`, `fresh-within`, coverage/population aliases, `all`/`any`/`not`, `manual-review`).

Control rows accept `description` or `narrative`; `automation` / `class` / `kind` collapse to `CatalogControl.automation` (default `automated`).

### 6. CLI surface

```text
weeping-angel assurance catalog validate [path]
weeping-angel assurance catalog stats [path]
weeping-angel assurance catalog inspect <control-id> [path]
```

Parser: `src/cli.rs` (`AssuranceCommand::Catalog`). Execution: `src/assurance_catalog.rs`. Dispatch: `src/main.rs` matches only the catalog arm; other `assurance` subcommands keep the not-certification stub (exit 0). Catalog commands print the readiness banner first. Exit 0 on success; non-zero on validation / inspect-unknown-control failure.

### 7. Content bound

This ADR authorizes **infrastructure + a minimal provider/framework-neutral fixture**:

| Kind | ID |
| --- | --- |
| control | `control.source.protected-branch` |
| evidence | `evidence.source.protected-branch` |
| test | `test.source.protected-branch` |

It does **not** authorize SOC 2 / NIS2 / DORA / ISO normative text in `catalog/`, or remapping `frameworks/iso-27001/2022/mappings.toml`. Later slices add `*.toml` files and manifest `[files]` lines **without editing the loader**. Those files must still pass the same validator. The IAM family (`controls/identity.toml` and siblings) is authorized by [`0022-iam-canonical-assurance-catalog.md`](0022-iam-canonical-assurance-catalog.md), not by expanding this ADR’s content bound.

## Consequences

**Positive**

- Downstream slices add TOML + a manifest entry; loader source stays unchanged.
- assessment lineage can persist `CanonicalCatalogSnapshot` using the catalog digest display string.
- ISO remap remaps ISO `to =` onto `control.*` IDs ([ADR 0003 remap](0027-iso27001-canonical-remap.md)).
- Architecture tests (CAT-012…014) keep framework SDK-free and collector framework-blind.

**Negative / cost**

- IR `ControlId` stays permissive so historical sliver strings still parse; live ISO mappings target `control.*`.
- Manifest `[digest]` is not enforced; changing algorithm requires a code change, not just TOML.
- Expression ops remain a catalog string allow-list (nested tables validated). Binding onto `TestExpr` JSON is `CanonicalCatalog::projection` ([ADR 0011](0046-catalog-framework-digest-and-pin-ownership.md)); the catalog crate still does not depend on `weeping-angel-control-test`.

**Rejected alternatives** (also)

- Enforcing `control.*` on IR `ControlId` in this slice.
- Teaching `compile_framework` / the framework crate to parse `catalog/canonical/v1` TOML (packs consume `CatalogProjection` instead; [ADR 0011](0046-catalog-framework-digest-and-pin-ownership.md)).
- Hashing raw TOML bytes.

## Access and security

- Catalog load is local-filesystem only (no HTTP fetch of catalogs).
- Path escape and unlisted extra files fail closed.
- No credentials belong in catalog TOML; this crate does not seal evidence.

## Related

- Spec SSOT: [`docs/specs/canonical-assurance-catalog-v1.md`](../specs/canonical-assurance-catalog-v1.md)
- Public contract: [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md)
- ADR 0001: [`0001-inwardly-extensible-assurance-runtime.md`](0001-inwardly-extensible-assurance-runtime.md)
- ADR 0002: [`0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
- Packs (unchanged ownership): [`frameworks/README.md`](../../frameworks/README.md)
- Catalog/framework/readiness trust boundary: [`0046-catalog-framework-digest-and-pin-ownership.md`](0046-catalog-framework-digest-and-pin-ownership.md)
- Siblings (cite by path): [`0036-typed-evidence-canonical-serialization.md`](0036-typed-evidence-canonical-serialization.md), [`0034-subject-population-runtime-and-coverage-semantics.md`](0034-subject-population-runtime-and-coverage-semantics.md), [`0022-iam-canonical-assurance-catalog.md`](0022-iam-canonical-assurance-catalog.md)
