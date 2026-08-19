# ADR 0003 — ISO 27001:2022 pack remaps onto the canonical catalog (DRAFT)

| Field | Value |
| --- | --- |
| Status | **Draft** (accept after implement / target GREEN; drop `-draft`) |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | The **pack-local canonical library** decision in [ADR 0002](0002-iso-27001-assurance-vertical.md) §3 (`access.mfa.privileged`, `source.branch-protection`, … in `metadata.toml`) and the ISO-only compile/serialize branches as a long-term contract. Does **not** supercede ADR 0002’s structural pack, legal boundary, ledger, TestExpr, collectors, or non-certification language. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md), [ADR 0002](0002-iso-27001-assurance-vertical.md), [catalog infrastructure](0003-canonical-assurance-catalog-v1.md), [IAM family](0003-iam-canonical-assurance-catalog.md), [lineage draft](0003-assessment-lineage.md) |
| Spec | [`docs/sdd/iso-27001-canonical-remap.md`](../sdd/iso-27001-canonical-remap.md) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — update on accept |
| Prompt | [`docs/prompts/canonical-assurance-v1/12-iso27001-remap.md`](../prompts/canonical-assurance-v1/12-iso27001-remap.md) |
| Characterization | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Tests | Dual-suite **already registered**: `sdd_iso27001_remap_baseline` (12 GREEN characterization tests on sliver HEAD); `sdd_iso27001_remap_target` (registration stub — author ISO-R-001…020 first so they RED, then GREEN). Do **not** reuse `sdd_iso27001_assurance_*`. |

> Filename `0003-*` is shared with catalog-program siblings. Cite this decision by **path**.

## Context

ADR 0002 shipped the first ISO 27001:2022 vertical as a versioned structural pack. Because no catalog tree existed yet, reusable controls lived **inside the pack** (`frameworks/iso-27001/2022/metadata.toml`) — **22** sliver IDs (`access.mfa.privileged`, `source.branch-protection`, `vulnerability.remediation`, …) with 27 mappings onto them. IAM-008 and `sdd_iso27001_assurance_target` froze those IDs so Prompts 04–08 could land without rewriting the pack.

ADR 0003 catalog infrastructure plus the IAM family (and specified SDLC/vuln/infra/governance families) moved the reusable library to `catalog/canonical/v1/` with `control.*` / `evidence.*` / `test.*` IDs. Two libraries now represent the same semantics.

The generic runtime still special-cases ISO: `normalize`, `stub_catalog`, `assessment_for_target`, and `AssessmentReport::serialize` call `load_framework_pack("iso-27001", "2022")`. SoA rereads `applicability.toml`. `AssessmentRun` pins only `frameworkPackDigest`. The pack loader rejects IR relations `EvidenceFor`, `SupersetOf`, and `SubsetOf`, and validates mapping targets against pack metadata, not the catalog.

Prompt 12 must remap ISO onto the catalog without becoming a certification product, without changing catalog IDs, and without GitHub→ISO shortcuts.

Questions this decision answers:

1. Where do reusable ISO-mapped controls live after catalog v1 exists?
2. What mapping relations may a pack declare, and which ones may fully satisfy a requirement?
3. How does ISO resolve at compile/serialize time if not via an ISO-only branch?
4. How does SoA stay honest about not-applicable vs missing evidence?
5. What happens to IAM-008 and the MVP expected-control list?

## Decision

### 1. ISO is a projection, not a control library

The ISO 27001:2022 pack remains `frameworks/iso-27001/2022` with schema `weeping-angel/framework-pack/v1` and `content_mode = StructuralOnly`. It stores identifiers, short titles, hierarchy, automation class, applicability metadata, and mappings.

Canonical controls, tests, and evidence requirements live only in `catalog/canonical/v1/`. Pack `metadata.toml` must not declare a competing sliver library. Mapping `to` values are catalog control IDs (`control.identity.privileged-mfa`, not `access.mfa.privileged`).

On characterization SHA `e430980c…` the landed catalog is **23 `control.identity.*` plus fixture `control.source.protected-branch` (exists-only)**. Implement maps only IDs present at implement time. Unlanded SDLC/vuln/infra/governance families stay **unmapped** rather than pack-stubbed or invented.

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

The pack loader accepts all eight (plus empty → `from_completeness`). `Equivalent` is never a convenience. `PartiallySatisfies`, `Supports`, `Related`, `EvidenceFor`, and `SubsetOf` **cannot** fully satisfy a framework requirement. `SupersetOf` may satisfy only with completeness `full`. Material mappings carry rationale and provenance; edition-specific rows carry version constraints.

`weeping-angel assurance framework validate` fails closed on unknown catalog targets, retired sliver IDs, empty required rationale, and unknown relations.

### 3. Generic resolution; no ISO special-case in generic serialize

Every framework, including ISO, resolves through:

```text
(framework id, version) → load_framework_pack / load_framework_pack_from
```

Generic `AssessmentReport` serialization performs **no** pack I/O. `AssessmentRun` (and readiness snapshots) pin **both** `frameworkPackDigest` and `catalogDigest`. ISO-only branches in `normalize` / `stub_catalog` / `assessment_for_target` are removed in favor of target identity (shared with Prompt 11; do not invent a second registry).

Catalog ID resolution at compile/validate happens at the orchestrator / CLI seam so `weeping-angel-framework` does not take a hard dependency on the catalog crate unless a later accepted ADR documents a narrower hook.

### 4. SoA consumes generic applicability

Statement-of-Applicability projection uses generic applicability results:

```text
Applicable | NotApplicable | Unresolved / ManualDeterminationRequired
```

plus rationale, mapped catalog controls, evidence/implementation state, effectiveness, exceptions, missing evidence, and manual-review flags. Not-applicable is justified by organization context, never by absence of evidence. Pack `applicability.toml` may supply rules/defaults; it is not the SoA document.

### 5. Neighbor tests that froze the sliver are superseded here

In the same implement slice:

- IAM-008 becomes “ISO maps onto `control.identity.*`; pack does not keep the IAM sliver.”
- MVP `EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES` stop requiring pack-local `access.*` / `source.*` slivers.

A new dual-suite `sdd_iso27001_remap_{baseline,target}` is the Prompt 12 gate. `sdd_iso27001_assurance_*` is not reused.

### 6. Readiness language and coverage stay non-certifying

Forbidden: `ISO 27001 certified`, `ISO 27001 compliant`, `certification guaranteed`, `audit passed`.

Allowed: ready / effective / ineffective / partially effective / insufficient evidence / stale evidence / manual review required / not applicable / assessment coverage / partially covered.

Coverage is five separate metrics (automation, evidence, subject, control, framework-requirement), not one compliance percentage.

Map as comprehensively as **landed** catalog v1 permits. Governance/judgement requirements stay Manual/Hybrid. Do not invent automated tests or catalog IDs.

## Alternatives considered

1. **Keep slivers and add catalog aliases** — two IDs per semantic control; rejected by Prompt 12 and ADR 0003 catalog ownership.
2. **Rename catalog IDs to match slivers** (`control.access.mfa.privileged`) — rejected; catalog IDs are stable and framework-neutral.
3. **ISO-specific compile path that embeds catalog** — rejected; collectors/tests stay framework-blind; generic registry is the law.
4. **Treat every Annex A row as `Equivalent` to the nearest technical control** — rejected; would falsify readiness and certification language.
5. **Rewrite `sdd_iso27001_assurance_target` as the remap suite** — rejected; that suite is the landed MVP contract (EVD/CTL/GH). New suite required.

## Consequences

**Positive**

- One control library; ISO is data.
- Partial mappings stay honest.
- Reports and lineage name both pack and catalog snapshots.
- Future SOC 2 / NIS2 packs can map onto the same catalog without copying slivers.

**Negative / cost**

- Incomplete catalog families (SDLC/vuln/infra/governance unlanded) leave some ISO requirements unmapped until those slices land.
- IAM-008 and MVP expected-control asserts must change in the same cut or CI stays red.
- Historical assessments that cited sliver IDs need Prompt 11 digest pins to replay rather than silent reinterpretation.
- Framework validate grows a catalog-aware check at the CLI/orchestrator seam.

## Acceptance

Accept this ADR when `sdd_iso27001_remap_target` is GREEN, the draft suffix is dropped, and `docs/contracts/assurance-runtime.md` lists the eight relations, catalog-targeted mappings, dual digests, and generic SoA semantics.
