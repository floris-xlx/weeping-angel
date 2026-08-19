# ADR 0003 — ISO 27001:2022 pack remaps onto the canonical catalog

| Field | Value |
| --- | --- |
| Status | **Accepted** |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The **pack-local canonical library** in [ADR 0002](0002-iso-27001-assurance-vertical.md) §3 (`access.mfa.privileged`, `source.branch-protection`, … in `metadata.toml`) and ISO-only compile/serialize branches as a long-term contract. Does **not** supercede ADR 0002’s structural pack, legal boundary, ledger, TestExpr, collectors, or non-certification language. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [catalog infrastructure](0003-canonical-assurance-catalog-v1.md), [IAM family](0003-iam-canonical-assurance-catalog.md), [applicability engine](0003-applicability-engine.md), [assessment lineage](0003-assessment-lineage.md) |
| Spec | [`docs/sdd/iso-27001-canonical-remap.md`](../sdd/iso-27001-canonical-remap.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Prompt | [`docs/prompts/canonical-assurance-v1/12-iso27001-remap.md`](../prompts/canonical-assurance-v1/12-iso27001-remap.md) |
| Characterization | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | `sdd_iso27001_remap_target` GREEN (ISO-R-001…020 + goldens + architecture-boundary). `sdd_iso27001_remap_baseline` skip-superseded. Do **not** reuse `sdd_iso27001_assurance_*` as the remap suite. |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0002 shipped the first ISO 27001:2022 vertical as a versioned structural pack. Because no catalog tree existed yet, reusable controls lived **inside the pack** (`frameworks/iso-27001/2022/metadata.toml`) — 22 sliver IDs (`access.mfa.privileged`, `source.branch-protection`, `vulnerability.remediation`, …) with 27 mappings onto them. IAM-008 and `sdd_iso27001_assurance_target` froze those IDs so Prompts 04–08 could land without rewriting the pack.

Canonical catalog infrastructure plus the IAM and SDLC families moved the reusable library to `catalog/canonical/v1/` (`control.*` / `evidence.*` / `test.*`). Two libraries represented the same semantics.

The generic runtime still special-cased ISO: `normalize`, `stub_catalog`, `assessment_for_target`, and report serialize called `load_framework_pack("iso-27001", "2022")`. SoA reread `applicability.toml` booleans. `AssessmentRun` pinned only `frameworkPackDigest`. The pack loader rejected IR relations `EvidenceFor`, `SupersetOf`, and `SubsetOf`, and validated mapping targets against pack metadata, not the catalog.

Prompt 12 remaps ISO onto landed catalog IDs without becoming a certification product, without renaming catalog IDs, and without GitHub→ISO shortcuts.

Questions this decision answers:

1. Where do reusable ISO-mapped controls live after catalog v1 exists?
2. What mapping relations may a pack declare, and which ones may fully satisfy a requirement?
3. How does ISO resolve at compile/serialize time if not via an ISO-only branch?
4. How does SoA stay honest about not-applicable vs missing evidence?
5. What happens to IAM-008 and the MVP expected-control list?

## Decision

This is what shipped.

### 1. ISO is a projection, not a control library

The ISO 27001:2022 pack remains `frameworks/iso-27001/2022` with schema `weeping-angel/framework-pack/v1` and `content_mode = StructuralOnly`. It stores identifiers, short titles, hierarchy, automation class, applicability metadata, and mappings. Public files do not redistribute ISO/IEC normative wording.

Canonical controls, tests, and evidence requirements live only in `catalog/canonical/v1/`. Pack `metadata.toml` holds pack-only annotations (`library = "catalog/canonical/v1"`) and does **not** declare a competing sliver library.

Mapping `to` values are catalog control IDs. Required remaps that shipped:

| ISO requirement | Catalog control(s) | Relation |
| --- | --- | --- |
| `iso27001:a.8.5` | `control.identity.privileged-mfa`, `control.identity.mfa` | `PartiallySatisfies` |
| `iso27001:a.8.5` | `control.identity.strong-authentication-policy` | `Supports` |
| `iso27001:a.8.2` | `control.identity.privileged-access-minimization`, `control.identity.least-privilege` | `PartiallySatisfies` |
| `iso27001:a.8.3` | `control.identity.least-privilege` | `Supports` |
| `iso27001:a.5.15` | `control.identity.least-privilege` | `PartiallySatisfies` |
| `iso27001:a.5.18` | `control.identity.periodic-access-review`, `control.identity.access-approval` | `PartiallySatisfies` |
| `iso27001:a.5.16` | `control.identity.unique-user-identities` | `Supports` |
| `iso27001:a.5.16` | `control.identity.joiner-mover-leaver` | `PartiallySatisfies` |
| `iso27001:a.6.5` | `control.identity.terminated-user-removal`, `control.identity.access-revocation-timeliness` | `PartiallySatisfies` |
| `iso27001:a.8.25` | `control.source.default-branch-protection`, `control.source.required-review` | `PartiallySatisfies` |
| `iso27001:a.8.25` | `control.source.secure-development-policy` | `Supports` |
| `iso27001:a.8.26` | `control.source.secret-scanning` | `Supports` |
| `iso27001:a.8.26` | `control.source.security-review` | `PartiallySatisfies` |

Zero `Equivalent` rows. Privileged-MFA failure surfaces through `control.identity.privileged-mfa` / `test.identity.privileged-mfa-enabled`, not `access.mfa.privileged`.

Vuln / infrastructure / governance clauses (`iso27001:a.8.8`, `a.8.13`, `a.8.15`, `a.8.24`, `a.8.32`, `a.5.1`, `a.5.9`, `a.5.19`, `a.5.24`, clauses 4–10, …) stay **unmapped** rather than pack-stubbed. Missing catalog coverage is insufficient mapping / manual review, not a second sliver ID.

Desired chain:

```text
ISO requirement → Mapping → canonical control → canonical test → canonical evidence → provider-independent envelopes
```

Never: ISO requirement → GitHub check / AWS API / scanner engine.

### 2. Honest rich mappings; loader matches IR

Pack mappings use the full IR `MappingRelation` set:

```text
Equivalent | Satisfies | PartiallySatisfies | Supports | EvidenceFor | SupersetOf | SubsetOf | Related
```

The pack loader accepts all eight (plus empty → `from_completeness`). `Equivalent` is never a convenience. `PartiallySatisfies`, `Supports`, `Related`, `EvidenceFor`, and `SubsetOf` **cannot** fully satisfy a framework requirement. `Satisfies` / `Equivalent` / `SupersetOf` may fully satisfy only with completeness `full`.

Material mappings carry `rationale`, `provenance` (`BuiltIn` / `UserDefined` / `LicensedFrameworkContent`, optional author/reference), and edition `valid_for` (`from = "2022"`, `to = "2022"`).

`weeping-angel assurance framework validate` fails closed on unknown catalog targets, retired sliver IDs, empty required rationale, and unknown relations.

### 3. Generic resolution; catalog targets without a crate-graph cycle

Every framework, including ISO, resolves through:

```text
(framework id, version) → load_framework_pack / load_framework_pack_from
```

`normalize` / `assessment_for_target` key off the assessed target identity. Generic `AssessmentReport` serialization performs **no** `load_framework_pack("iso-27001", "2022")` I/O; it emits pins already carried on the report / run.

`AssessmentRun` and readiness snapshots pin **both** `frameworkPackDigest` and the catalog digest (`canonicalCatalogDigest` on the run/report; `catalogDigest` on readiness JSON). Assess records the catalog pin via `CanonicalCatalog::digest` at the orchestrator.

Catalog ID resolution uses a narrow pack-loader index: discover `catalog/canonical/v1` by search roots and fail closed on unknown `control.*` mapping targets. The framework crate does **not** depend on `weeping-angel-canonical-catalog`.

Readiness walks the **actual** mapping graph. A requirement whose mappings are only partial / support / related / evidence-for / subset stays `partially covered` even if every mapped test is `Effective`.

### 4. SoA consumes generic three-state applicability

`project_soa(framework, version)` is pack-generic. `SoaEntry.applicability` is:

```text
Applicable | NotApplicable | Unresolved
```

(`Unresolved` is the SoA spelling of `ManualDeterminationRequired`.) Pack `applicability.toml` may supply default rules and structural SoA flags; evaluation is not a boolean copy. Not-applicable is justified by organization context (for example A.5.19: no external suppliers), never by absence of evidence. Incomplete context stays unresolved (A.8.13), not coerced to NA.

Each SoA entry lists mapped catalog controls, rationale, implementation/evidence state, effectiveness, exceptions, and manual-review flags.

### 5. Neighbor tests that froze the sliver are superseded

In the same implement slice:

- IAM-008 / IAM-016: ISO maps onto `control.identity.*`; pack does not keep the IAM sliver.
- MVP `EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES` require catalog prefixes (`control.identity.`, landed `control.source.*`), not pack-local `access.*` / `source.*` slivers.

`sdd_iso27001_remap_{baseline,target}` is the Prompt 12 gate. `sdd_iso27001_assurance_*` remains the historical MVP EVD/CTL/GH contract.

### 6. Readiness language and coverage stay non-certifying

Forbidden: `ISO 27001 certified`, `ISO 27001 compliant`, `certification guaranteed`, `audit passed`.

Allowed: ready / effective / ineffective / partially effective / insufficient evidence / stale evidence / manual review required / not applicable / assessment coverage / partially covered.

Coverage is five separate metrics (automation, evidence, subject, control, framework-requirement), not one compliance percentage. No `compliancePercent` / `isoCompliant`.

Governance/judgement requirements stay Manual/Hybrid. Do not invent automated tests or catalog IDs to finish Annex A.

## Alternatives considered

1. **Keep slivers and add catalog aliases** — two IDs per semantic control; rejected by Prompt 12 and catalog ownership.
2. **Rename catalog IDs to match slivers** (`control.access.mfa.privileged`) — rejected; catalog IDs are stable and framework-neutral.
3. **ISO-specific compile path that embeds catalog** — rejected; collectors/tests stay framework-blind; generic registry is the law.
4. **Treat every Annex A row as `Equivalent` to the nearest technical control** — rejected; would falsify readiness and certification language.
5. **Rewrite `sdd_iso27001_assurance_target` as the remap suite** — rejected; that suite is the landed MVP contract (EVD/CTL/GH). New suite required.
6. **Map A.8.8 onto `control.vulnerability.*` in this slice** — deferred. The vuln family may exist on disk; this remap left vuln/infra/governance unmapped rather than claiming a defensible Annex A projection.

## Consequences

**Positive**

- One control library; ISO is data.
- Partial mappings stay honest.
- Reports and lineage name both pack and catalog snapshots.
- Future SOC 2 / NIS2 packs can map onto the same catalog without copying slivers.

**Negative / cost**

- Incomplete catalog projection: vuln / infrastructure / governance ISO requirements remain unmapped until a later honest remap.
- Historical assessments that cited sliver IDs need digest pins to replay rather than silent reinterpretation.
- Framework validate grows a catalog-aware check at the pack-loader / CLI seam (index, not a crate dependency).

## Non-goals

SOC 2 / NIS2 / DORA / PCI / HIPAA packs; renaming catalog IDs; provider APIs; auditor or certification claims; inventing missing domain catalog IDs inside the pack.

## Related

- Spec SSOT: [`docs/sdd/iso-27001-canonical-remap.md`](../sdd/iso-27001-canonical-remap.md)
- MVP vertical (pack/legal/CLI still law): [`docs/adr/0002-iso-27001-assurance-vertical.md`](0002-iso-27001-assurance-vertical.md)
- Packs: [`frameworks/README.md`](../../frameworks/README.md)
- Public contract: [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md)
