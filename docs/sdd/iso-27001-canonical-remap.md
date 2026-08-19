# SDD: ISO 27001:2022 remapping onto the Canonical Assurance Catalog

| Field | Value |
| --- | --- |
| Status | **Implemented — target GREEN** |
| Program | Canonical Assurance Catalog v1 |
| Slice | Prompt 12 — ISO 27001:2022 framework pack remaps onto the completed canonical catalog |
| Source prompt | [`docs/prompts/canonical-assurance-v1/12-iso27001-remap.md`](../prompts/canonical-assurance-v1/12-iso27001-remap.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` (`main`, 2026-08-19) — re-read and verified on this HEAD |
| Dual-suite | **Already registered** in root `Cargo.toml`: `sdd_iso27001_remap_baseline` → `tests/sdd/iso27001_remap.baseline.rs` (**12 GREEN** characterization tests); `sdd_iso27001_remap_target` → `tests/sdd/iso27001_remap.target.rs` (**registration stub only**) |
| Do **not** reuse | `tests/sdd/iso27001_assurance.{baseline,target}.rs` (MVP dual-suite ISO-001…010 / EVD / CTL / GH; freezes pack-local slivers) |
| Transition | **replacement** of pack-local control library + ISO special-case load/serialize |
| ADR | Draft [`docs/adr/0003-iso27001-canonical-remap-draft.md`](../adr/0003-iso27001-canonical-remap-draft.md) — **accept after target GREEN** (drop `-draft`) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — update in implement, not this spec-only phase |
| MVP SSOT (still law for spine/legal/CLI) | [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), [ADR 0002](../adr/0002-iso-27001-assurance-vertical.md) |
| Catalog infra | [`docs/sdd/canonical-assurance-catalog-v1.md`](canonical-assurance-catalog-v1.md) |
| IAM family (landed) | [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md) — 23 `control.identity.*` |
| Fixture (landed, exists-only) | `control.source.protected-branch` / `test.source.protected-branch` (`op = exists`) |
| SDLC / vuln / infra / governance | Specified; **product unlanded** on this SHA. Map IDs only if present at implement time. Do not invent them. |
| Concurrent (do not collide) | Prompt 09 [`github-collector.md`](github-collector.md); Prompt 10 applicability engine (consume generic `Applicable` / `NotApplicable` / `Unresolved`); Prompt 11 [`assessment-lineage.md`](assessment-lineage.md) |
| Workspace verify | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --features demo`; `weeping-angel assurance catalog validate`; `weeping-angel assurance framework validate frameworks/iso-27001/2022`; `cargo test --test sdd_iso27001_remap_target --offline` |

This document is the durable SSOT for **Prompt 12**. It owns ISO 27001:2022 **framework content**, **mappings**, **applicability references**, and **projection integration**. It must **not** redesign canonical controls, provider collectors, the catalog loader, or the control-test evaluator.

Architecture law (unchanged):

```text
ISO 27001 requirement
        ↓ mapping
Canonical control
        ↓
Canonical control test
        ↓
Canonical evidence requirement
        ↓
Provider-independent evidence
```

Never:

```text
ISO requirement → GitHub check
ISO requirement → AWS API
ISO requirement → scanner engine
```

This is **readiness/assurance only**, not certification automation.

**This spec-only phase does not change production source.** Dual-suite files and this markdown are characterization + contract. Implement starts by authoring ISO-R-001…020 so they are **RED on current sliver HEAD**, then changing only owned files.

---

## 1. Problem / user-visible goal

The ISO 27001 MVP vertical landed a real structural pack and a **pack-local sliver library** (`access.mfa.privileged`, `source.branch-protection`, `vulnerability.remediation`, …) inside `frameworks/iso-27001/2022/metadata.toml`. Prompts 01–04 moved the reusable library to `catalog/canonical/v1/` (`control.identity.*` is landed; SDLC/vuln/infra/governance families are specified and must not be forked here). Two competing ID spaces now exist for the same semantic controls.

Meanwhile the generic runtime still treats ISO as a special case: `normalize` / `stub_catalog` / `assessment_for_target` / `AssessmentReport::serialize` / `AssessmentRun` construction call `load_framework_pack("iso-27001", "2022")`. SoA rereads today’s `applicability.toml` booleans. `AssessmentRun` pins `frameworkPackDigest` only. The pack loader accepts only five mapping relations and validates `to` against pack metadata, not the catalog. Readiness attaches **every compiled control** to every requirement and hard-codes `has_partial = true`. Coverage is invented `"NN%"` strings.

**User-visible goal:** an ISO 27001:2022 readiness assessment whose every material requirement traces:

```text
iso27001:<clause> → honest Mapping → control.* (catalog) → test.* → evidence.* → provider-independent envelopes
```

Annex A / SoA output uses the **generic** applicability engine (applicable / not applicable / unresolved), justified NA, mapped catalog controls, implementation/evidence state, effectiveness, exceptions, missing evidence, and manual-review requirements. Lineage pins **both** pack digest and catalog digest. Reports never say certified / compliant / audit passed. Coverage is five separate metrics, not one compliance percentage.

The MVP dual-suite (`sdd_iso27001_assurance_*`) remains the historical ISO-001…010 / EVD / CTL / GH contract. This slice already registered a **new** dual-suite and, in the same implement cut, must supersede the two assertions that would otherwise keep the workspace red: IAM-008 (pack sliver frozen) and `EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES` (pack-local ids).

---

## 2. Dependencies, ownership, and fail-closed blockers

| Prompt | Owns | On SHA `e430980c…` | This slice may |
| --- | --- | --- | --- |
| 01 catalog | `CanonicalCatalog::{load,validate,digest}`, ID grammar | **Landed.** Tree lists `fixture.example` + `identity` only. | Resolve mapping `to` against this catalog. Do not invent a second loader. Do not change catalog IDs to make ISO mapping easier. |
| 02 typed evidence | `EvidenceValue`, seal | **Landed.** | Consume. No second value enum. |
| 03 population | coverage / missing / stale / fail | **Landed.** | Consume catalog tests as declared. Do not reimplement coverage. |
| 04 IAM | 23 `control.identity.*` | **Landed.** | Remap ISO IAM slivers onto these IDs. **Supersede IAM-008** in the same implement cut. |
| 05–08 domain catalogs | `control.source.*` / `vulnerability.*` / infra / governance | **Specified; product unlanded** except fixture `control.source.protected-branch`. | Map **as comprehensively as the catalog present at implement time permits**. Do not invent missing catalog IDs. Do not grow pack-local stubs to fill gaps. |
| 09 collectors | GitHub / local / manual | GitHub emits `source.*`; no ISO IDs in collector crate. Dual-suite `sdd_github_collector_*` in flight. | **Do not add ISO requirement IDs to collectors.** Do not edit `tests/sdd/github_collector.*` or collector product code. |
| 10 applicability | org-context three-state evaluator | **Not landed** (`ApplicabilityRule::statically_applicable` only; SoA reads pack TOML). | Integrate the **generic** engine when present. If still absent, SoA must still consume a generic result type (`Applicable` / `NotApplicable` / `Unresolved` / `ManualDeterminationRequired`) rather than a boolean from `applicability.toml`. Do not implement a second ISO-only evaluator. |
| 11 lineage | persistable run, pure serialize, generic facade, catalog digest pin | **Specified; product unlanded.** Facade still hard-codes ISO pack load. Dual-suite `sdd_assessment_lineage_*` in flight. | Consume Prompt 11 types if they landed. If still in flight, this slice **must not** leave ISO special-cases in generic serialize/test runtime; it must use the same `(id, version) → load_framework_pack` path Prompt 11 specifies. Do not invent a competing lineage model. Do not edit `tests/sdd/assessment_lineage.*`. |

Rebase rule: follow landed catalog IDs and Prompt 11 field names. Prefer adapting pack mappings to those contracts over extending this slice’s scope.

### 2.1 Files this slice owns (implement)

| Own | Do not own / do not collide |
| --- | --- |
| `frameworks/iso-27001/2022/{manifest,requirements,mappings,applicability,metadata}.toml` | `catalog/canonical/v1/**` IDs and family TOML |
| ISO projection/mapping/SoA integration (`soa.rs`, `readiness.rs`, generic assess/serialize ISO literals) | Collector crate; `tests/sdd/github_collector.*` |
| `crates/weeping-angel-framework/src/pack.rs` relation parse + mapping row provenance/`valid_for` + catalog-target validation **seam** | Prompt 10 org-context evaluator implementation |
| `tests/sdd/iso27001_remap.{baseline,target}.rs` | `tests/sdd/iso27001_assurance.{baseline,target}.rs` wholesale rewrite |
| Same-PR supersession of IAM-008 and `EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES` | `tests/sdd/assessment_lineage.*`; Prompt 05–08 catalog product |

Harness: root `Cargo.toml` does **not** auto-discover `tests/sdd/*.rs`. Dual-suite entries already exist — do not add a second pair; do not reuse `iso27001_assurance.*` names.

---

## 3. Current behavior (baseline — GREEN on characterization SHA)

Characterized and re-verified against `e430980c0d27a8138a153d49b62ddf3c57827891`. The remap **baseline** suite (`sdd_iso27001_remap_baseline`, **12 tests**) must stay GREEN on this HEAD until the target suite is GREEN and the baseline is skip-superseded.

### 3.1 Pack is structural-only and pack-local

`frameworks/iso-27001/2022/`:

| File | Role today |
| --- | --- |
| `manifest.toml` | `schema = weeping-angel/framework-pack/v1`, `id = iso-27001`, `version = 2022`, `content_mode = StructuralOnly`, SoA/applicability/risk/manual capabilities true |
| `requirements.toml` | **42** structural requirement ids (`iso27001:4.1` … `iso27001:10.2` plus a **subset** of Annex A references: `a.5.1`, `a.5.7`, `a.5.9`, `a.5.15`–`a.5.19`, `a.5.23`, `a.5.24`, `a.5.28`, `a.6.1`, `a.6.5`, `a.8.2`, `a.8.3`, `a.8.5`, `a.8.8`, `a.8.9`, `a.8.13`, `a.8.15`, `a.8.24`–`a.8.26`, `a.8.32`). Short titles only; no ISO/IEC normative wording |
| `mappings.toml` | **27** rows; all `to` values are pack slivers (`access.*`, `source.*`, `vulnerability.*`, `logging.*`, `incident.*`, `backup.*`, `encryption.*`, `change.*`, `security.*`, `supplier.*`, `personnel.*`, `asset.*`). Completeness is `partial` or `related`. Relations used: `PartiallySatisfies`, `Supports`, `Related`. No `Equivalent`. No `EvidenceFor` / `SupersetOf` / `SubsetOf`. No `provenance` / `valid_for` fields |
| `applicability.toml` | **10** SoA-oriented `[[entry]]` rows (`A.5.1`, `A.5.15`, `A.5.18`, `A.8.2`, `A.8.5`, `A.8.8`, `A.8.25`, `A.8.13`, `A.5.19`, `A.6.5`); `applicable = true` booleans + original rationales. No unresolved/manual determination |
| `metadata.toml` | **22** `[[control]]` + **22** `[[test]]` rows — the competing library. Tests require GitHub-shaped or pack-local evidence types (`source.admin.permissions`, `source.branch.protection`, `security.vulnerability.present`, …) |

The 22 pack-local control ids:

```text
access.mfa.privileged
access.least-privilege
access.periodic-review
source.branch-protection
source.required-review
source.code-ownership
source.security-scanning
source.commit-signing
vulnerability.remediation
logging.security-events
logging.audit-trail
incident.response-process
backup.recovery-testing
encryption.data-at-rest
encryption.data-in-transit
supplier.security-assessment
personnel.access-termination
asset.inventory
change.approval
security.headers
security.tls
security.secret-exposure
```

`sdd_iso27001_assurance_target` freezes prefixes `access.`, `source.`, `vulnerability.`, … and `EXPECTED_CANONICAL_CONTROLS` including `access.mfa.privileged`.

### 3.2 Two ID spaces for the same semantics

| Semantic | Pack sliver (live mapping target) | Catalog v1 (when present) |
| --- | --- | --- |
| Privileged MFA | `access.mfa.privileged` + `test.access.mfa.privileged` requiring **some** `source.admin.permissions` | `control.identity.privileged-mfa` + population `test.identity.privileged-mfa-enabled` |
| MFA / strong auth | (none as first-class sliver) | `control.identity.mfa`, `control.identity.strong-authentication-policy` |
| Least privilege | `access.least-privilege` | `control.identity.least-privilege`, `control.identity.privileged-access-minimization` |
| Periodic access review | `access.periodic-review` | `control.identity.periodic-access-review`, `control.identity.access-approval` |
| Identity lifecycle | `personnel.access-termination` | `control.identity.terminated-user-removal`, `control.identity.joiner-mover-leaver`, `control.identity.access-revocation-timeliness`, `control.identity.unique-user-identities` |
| Branch protection | `source.branch-protection` | specified `control.source.default-branch-protection` (product unlanded); fixture `control.source.protected-branch` is **exists-only** (`op = exists` on `evidence.source.protected-branch`) |
| Vulnerability | `vulnerability.remediation` / `security.vulnerability.present` | specified `control.vulnerability.*` (product unlanded) |

Catalog `manifest.toml` lists only `fixture.example` + `identity`. **No** `control.identity.*` ids live in the ISO pack today (IAM-008 asserts this). Pack slivers are **not** in the catalog.

Landed catalog control ids on this SHA (map only these unless more land before implement):

```text
control.source.protected-branch          # fixture, exists-only
control.identity.unique-user-identities
control.identity.mfa
control.identity.privileged-mfa
control.identity.strong-authentication-policy
control.identity.privileged-inventory
control.identity.least-privilege
control.identity.privileged-access-minimization
control.identity.access-approval
control.identity.periodic-access-review
control.identity.inactive-account-lifecycle
control.identity.terminated-user-removal
control.identity.joiner-mover-leaver
control.identity.service-account-inventory
control.identity.service-account-ownership
control.identity.service-account-credential-governance
control.identity.break-glass-access
control.identity.shared-account-restriction
control.identity.credential-management
control.identity.privileged-role-change-monitoring
control.identity.external-guest-access
control.identity.stale-privileged-membership
control.identity.access-revocation-timeliness
control.identity.segregation-of-duties
```

### 3.3 Pack loader does not accept the full IR relation set

IR `MappingRelation` (`crates/weeping-angel-assurance-ir/src/mapping.rs`) is:

```text
Equivalent | Satisfies | PartiallySatisfies | Supports | EvidenceFor | SupersetOf | SubsetOf | Related
```

IR `Mapping` already has `provenance` (`MappingProvenance`) and `valid_for` (`MappingVersionConstraint`).

[`crates/weeping-angel-framework/src/pack.rs`](../../crates/weeping-angel-framework/src/pack.rs) `MappingRow` has only `from`, `to`, `direction`, `completeness`, `relation`, `rationale`. The relation match accepts `Equivalent` / `Satisfies` / `PartiallySatisfies` / `Supports` / `Related` plus empty → `from_completeness`. `EvidenceFor` / `SupersetOf` / `SubsetOf` return `PackError::UnsupportedRelation`.

Mapping `to` is dangling unless it matches a `metadata.toml` control **when that list is non-empty**. A catalog id such as `control.identity.privileged-mfa` currently **fails pack load** as `PackError::Dangling` (baseline proves this). Catalog existence is not checked.

`ComplianceGraph::equivalent` already requires explicit full bidirectional mappings. Readiness does **not** consult the mapping graph.

### 3.4 ISO is hardcoded on the generic path

| Site | Behavior on SHA `e430980c…` |
| --- | --- |
| `weeping-angel-framework::normalize` | If profile is `Iso27001` **and** version `"2022"`, merge `load_framework_pack("iso-27001", "2022")` |
| `stub_catalog` | ISO profile loads that pack’s 42 requirements; other profiles `[]` |
| `assessment_for_target` | Same ISO branch; else production stub `canonical:stub-1` |
| `AssessmentReport::serialize` | Calls `load_framework_pack("iso-27001", "2022")` for `frameworkPackDigest`; invents `automationCoverage` / `evidenceCoverage` `"NN%"` strings; **no** `catalogDigest` |
| `AssuranceEngineBuilder::assess` | Builds `AssessmentRun` with `framework_pack_digest` from that ISO load, then **drops** it (`let _run`). No `catalog_digest` field on `AssessmentRun` |
| `project_soa` | `resolve_pack_dir` → read live `applicability.toml` → boolean `applicable`; `SoaEntry.applicable: bool`; no `Unresolved` / `ManualDeterminationRequired` |
| `project_readiness` | Pins pack digest only. Hard-codes `has_partial = true`. Maps **every compiled control** onto every requirement (`compiled.controls.iter().map(|c| c.id().clone())`). Coverage fields are `"NN%"` strings. No subject / control / framework-requirement metrics |
| `resolve_applicability` | Keeps requirements unless `statically_applicable() == Some(false)` |

There is **no** generic framework registry type. `load_framework_pack(id, version)` exists but generic serialize/orchestrate ignore the assessed target identity.

### 3.5 Collectors and control-test runtime

- `weeping-angel-collector` sources contain **no** `iso27001` requirement ids (already true; must stay true).
- `weeping-angel-control-test` has **no** ISO branches / `iso27001` tokens (already true; must stay true).
- Pack tests are presence/hybrid/manual on pack-local types, not catalog population tests.

### 3.6 Neighbor tests that freeze the sliver

| Test | Freeze |
| --- | --- |
| `iam_008_iso_pack_is_unchanged_and_has_no_control_identity` | Pack must keep `access.mfa.privileged` / `access.least-privilege` / `access.periodic-review` / `personnel.access-termination`; mappings must **not** contain `control.identity.` |
| `mvp_ships_at_least_twenty_canonical_controls` / ISO-004 | `EXPECTED_CANONICAL_CONTROLS` + prefixes `access.`, `source.`, … |
| SDLC/vuln specs | Explicitly **do not** retarget ISO mappings (that is this slice) |

If implement remaps without superseding IAM-008 and `EXPECTED_CANONICAL_CONTROLS` in the **same** cut, `cargo test --workspace --features demo` stays red.

### 3.7 What “ISO assessment” means today

A caller can `compile_framework` the ISO pack and run `test.access.mfa.privileged`, which requires **some** `source.admin.permissions` envelope. It cannot:

- evaluate privileged MFA across the canonical privileged population;
- distinguish missing inventory from a privileged identity without MFA;
- justify Annex A as not-applicable from organization context (only a pre-authored boolean);
- pin which catalog digest produced the result;
- serialize a report without re-loading the ISO pack from disk;
- load a mapping that uses `EvidenceFor` / `SupersetOf` / `SubsetOf`;
- target `control.identity.privileged-mfa` without the pack loader treating it as dangling.

The remap **baseline** therefore characterizes **today’s sliver + ISO special-cases**, not absence of the ISO pack.

### 3.8 Baseline suite inventory (already GREEN)

`tests/sdd/iso27001_remap.baseline.rs` — keep GREEN until target GREEN + skip-supersede:

1. `dual_suite_is_registered`
2. `mappings_target_pack_slivers_not_catalog_identity` (27 rows; no `control.identity.*`)
3. `metadata_toml_is_the_competing_control_library` (22/22; compiled `access.mfa.privileged`)
4. `assessment_report_serialize_hard_loads_iso_pack`
5. `generic_paths_special_case_iso27001_2022`
6. `project_soa_rereads_applicability_toml_booleans`
7. `assessment_run_has_pack_digest_only`
8. `catalog_has_identity_and_fixture_only_no_iso_slivers`
9. `pack_loader_rejects_evidence_for_superset_of_subset_of`
10. `neighbor_tests_still_freeze_the_sliver`
11. `collectors_and_control_test_have_no_iso_requirement_ids`
12. `pack_is_structural_only_with_forty_two_requirements`

`tests/sdd/iso27001_remap.target.rs` is a **registration stub** (`dual_suite_target_file_is_registered` only). Desired ISO-R-001…020 + golden 1–10 + architecture-boundary asserts are **not yet encoded**. Implement must write those first so they FAIL on current sliver HEAD for the right reasons (not compile noise).

---

## 4. Desired behavior (after this slice)

### 4.1 Placement and legal boundary

Work stays in the versioned structural pack:

```text
frameworks/iso-27001/2022/
  manifest.toml
  requirements.toml
  mappings.toml
  applicability.toml
  metadata.toml
```

Preserve `content_mode = StructuralOnly` and the `FrameworkContentProvider` abstraction (`StructuralOnly` | `LicensedContent` | `UserSuppliedContent`). Public files store **only**:

- identifiers (`iso27001:4.1`, `iso27001:a.8.5`, external refs `4.1` / `A.8.5`);
- legally safe short titles (existing “structural” / “reference” style);
- structural hierarchy (`kind`, `parent`);
- automation classification;
- applicability metadata and references;
- mappings (relation, completeness, direction, rationale, provenance, version constraints).

Do **not** redistribute protected ISO/IEC normative wording. Framework validate continues to fail on protected-text markers / known normative excerpts. Do not add “the organization shall” or quoted ISO/IEC 27001 normative text to titles, rationales, or notes.

`metadata.toml` **must not** remain a competing control/test library. After implement it may hold pack-only annotations (requirement automation class, SoA flags, review notes) but **must not** declare `access.*` / `source.*` / `vulnerability.*` slivers that duplicate catalog semantics. Tests and evidence requirements live in `catalog/canonical/v1/`.

### 4.2 Mapping honesty

Remap every material ISO requirement that the **landed** catalog can honestly support onto **existing** catalog control IDs from Prompts 04–08. Use the full IR relation set **honestly**:

| Relation | Meaning for readiness | May fully satisfy a requirement? |
| --- | --- | --- |
| `Equivalent` | Defensible bidirectional semantic identity | Yes, only if completeness is `full` **and** the catalog control is the whole requirement |
| `Satisfies` | Catalog control covers the requirement’s assessable obligation | Yes, only with completeness `full` |
| `PartiallySatisfies` | Necessary technical/governance slice | **Never** |
| `Supports` | Helpful signal; not coverage | **Never** |
| `EvidenceFor` | Control/test produces evidence used by the requirement’s assessment | **Never** |
| `SupersetOf` | Catalog control’s scope includes the requirement | Only with completeness `full` |
| `SubsetOf` | Catalog control is a slice of the requirement | **Never** |
| `Related` | Related only | **Never** |

Rules:

1. Do **not** use `Equivalent` as a convenience. Almost every ISO clause that includes policy, process, or judgement is **not** equivalent to a technical control. On this SHA, **zero** `Equivalent` rows are expected for the identity remaps (IAM controls are slices of Annex A, not the clause).
2. A requirement whose mappings are only `PartiallySatisfies` / `Supports` / `Related` / `EvidenceFor` / `SubsetOf` **cannot** project as fully satisfied / fully effective for that requirement. Readiness must keep `partially covered` (or weaker) even if every mapped test is `Effective`.
3. Every material mapping includes a non-empty `rationale` and `provenance` (`MappingProvenance`: source at least `BuiltIn` / `UserDefined` / `LicensedFrameworkContent`, plus optional author/reference). Pack `MappingRow` must deserialize these fields; the loader must populate IR `Mapping` (not leave default `BuiltIn` + unconstrained `valid_for` silently).
4. Apply `valid_for` / version constraints where the mapping is edition-specific (ISO 27001:2022).
5. Mapping `to` must be a **catalog** control id that `CanonicalCatalog::control` accepts at implement time. Unknown / deleted catalog ids fail pack validate / compile closed.
6. Pack loader **must** accept `EvidenceFor`, `SupersetOf`, and `SubsetOf` (and still reject unknown strings).
7. Do **not** change catalog IDs to make mapping prettier.
8. `ComplianceGraph::equivalent` must stay fail-closed (full bidirectional only). Readiness aggregation must walk the **actual** mapping graph, not attach every compiled control to every requirement.

### 4.3 Coverage — as comprehensive as catalog v1 permits

Map the **structural** requirement set in `requirements.toml` (42 ids on this SHA; do not invent extra Annex A clauses just to inflate coverage). For each requirement:

| Catalog family landed? | Action |
| --- | --- |
| Yes, honest control exists | Add explicit mapping(s) with the correct relation |
| Yes, only a governance/manual control exists | Map to that control; keep requirement `Manual` / `Hybrid` |
| No matching landed control | Leave **unmapped** (status: insufficient mapping / manual review required). Do **not** keep a pack-local stub “to fill the hole.” Do **not** invent a catalog id |

Governance, judgement, documentation, leadership, risk, internal audit, management review, and similar clauses map to **governance/manual** catalog controls when those IDs exist. Do **not** invent automated technical tests for them.

#### 4.3.1 Required remaps on this SHA (identity is landed)

These mappings **must** exist after implement (relation typical; never `Equivalent` unless a later accepted rationale proves bidirectional identity):

| ISO requirement | Catalog control(s) | Typical relation |
| --- | --- | --- |
| `iso27001:a.8.5` | `control.identity.privileged-mfa`, `control.identity.mfa`, `control.identity.strong-authentication-policy` | `PartiallySatisfies` / `Supports` |
| `iso27001:a.8.2` | `control.identity.privileged-access-minimization`, `control.identity.least-privilege` | `PartiallySatisfies` |
| `iso27001:a.8.3` | `control.identity.least-privilege` | `Supports` |
| `iso27001:a.5.15` | `control.identity.least-privilege` | `PartiallySatisfies` |
| `iso27001:a.5.18` | `control.identity.periodic-access-review`, `control.identity.access-approval` | `PartiallySatisfies` |
| `iso27001:a.5.16` | `control.identity.unique-user-identities`, `control.identity.joiner-mover-leaver` | `Supports` / `PartiallySatisfies` |
| `iso27001:a.6.5` | `control.identity.terminated-user-removal`, `control.identity.access-revocation-timeliness` | `PartiallySatisfies` |

Privileged-MFA **failure** must surface through `control.identity.privileged-mfa` / `test.identity.privileged-mfa-enabled`, not `access.mfa.privileged` and not a GitHub admin-permission existence check.

#### 4.3.2 Optional remaps if still only the fixture exists

| ISO requirement | Catalog control | Constraint |
| --- | --- | --- |
| `iso27001:a.8.25` | `control.source.protected-branch` | `PartiallySatisfies` or `Supports` **only**. Exists-only test must **not** fully satisfy A.8.25. Do not treat the fixture as `control.source.default-branch-protection`. |

If Prompt 05 lands `control.source.default-branch-protection` / `control.source.required-review` / `control.source.secure-development-policy` before or during implement, remap A.8.25 onto those ids instead of (or in addition to, honestly) the fixture.

#### 4.3.3 Must stay unmapped on this SHA (no catalog successor)

Unless the named family lands before implement, **delete the sliver mapping** and leave the requirement unmapped (insufficient mapping / manual review required):

```text
iso27001:a.8.8    # vulnerability.remediation / source.security-scanning
iso27001:a.8.26   # source.security-scanning / security.headers
iso27001:a.5.9    # asset.inventory
iso27001:a.5.19   # supplier.security-assessment
iso27001:a.5.24   # incident.response-process
iso27001:a.8.13   # backup.recovery-testing
iso27001:a.8.15   # logging.security-events
iso27001:a.8.24   # encryption.* / security.tls
iso27001:a.8.32   # change.approval
iso27001:5.2      # Related → incident.response-process
iso27001:a.5.1    # Related → incident.response-process
```

Clauses 4.* / 5.* / 6.* / 7.* / 8.1 / 9.* / 10.* remain Manual/Hybrid and unmapped until governance catalog IDs exist.

Illustrative (not a license to invent IDs) if families land:

| ISO requirement | Catalog control (examples) | Typical relation |
| --- | --- | --- |
| `iso27001:a.8.8` | `control.vulnerability.*` remediation / scan-coverage | `PartiallySatisfies` / `EvidenceFor` |
| `iso27001:a.8.13` / `a.8.15` / `a.8.24` | infra logging / backup / crypto | `PartiallySatisfies` / `Supports` |
| `iso27001:4.*` / `5.*` / `6.*` / `9.*` / `10.*` / `a.5.1` | governance policy / roles / risk / internal-audit / management-review / corrective-action | `PartiallySatisfies` / `Related`; remain Manual/Hybrid |

### 4.4 Delete / migrate competing stubs

- Remove or migrate pack-local controls/tests/evidence rows superseded by the catalog.
- After implement, `load_framework_pack("iso-27001", "2022")` must **not** expose `access.mfa.privileged` / `source.branch-protection` / `vulnerability.remediation` as compiled controls if the catalog has the semantic successor **or** if the sliver is retired without a successor (unmapped requirement, no stub).
- Do not leave two public control IDs for the same semantic control (no `access.mfa.privileged` **and** `control.identity.privileged-mfa`).
- Framework validate fails if a mapping `to` is a retired sliver or a non-catalog id.

### 4.5 Generic registry / loader (no ISO special-case)

ISO must resolve through the **same** path as every framework:

```text
(framework id, version) → resolve_pack_dir / load_framework_pack / load_framework_pack_from
                         → compile_framework
                         → catalog lookup for mapped control/test/evidence ids
```

Remove hardcoded `load_framework_pack("iso-27001", "2022")` from:

- `AssessmentReport` serialization (serialization must be **pure**; Prompt 11);
- `AssessmentRun` construction (use the assessed target’s pack + catalog digests);
- generic test runtime / report helpers.

`normalize` / `assessment_for_target` / `stub_catalog` must key off **target identity**, not an ISO-only branch. If Prompt 11 has already done this, consume it; do not add a second facade.

Compile/validate of a remapped pack:

1. load pack requirements + mappings;
2. resolve each mapping `to` via `CanonicalCatalog`;
3. attach catalog tests and evidence requirements;
4. fail closed on unknown catalog ids, illegal relations, empty required rationale, or competing sliver IDs.

`weeping-angel-framework` still must **not** depend on the catalog crate if that would violate ACT-003 / ADR 0003 (catalog I/O belongs at the orchestrator / CLI / validate path). Implement may resolve catalog IDs in `weeping-angel-assurance` + CLI `framework validate`, or add a narrow validation hook documented in the accepted ADR. Do **not** put provider types in the pack.

When `metadata.toml` no longer lists `[[control]]` rows, the current `!meta.control.is_empty()` dangling check must not be the only target validator — otherwise catalog-targeted mappings would either all dangle or all pass unchecked.

### 4.6 Applicability and SoA

Integrate the **generic** applicability engine (Prompt 10). Annex A / SoA-oriented output must preserve:

- `applicable` vs `not applicable` vs `unresolved` / `manual determination required`;
- rationale and the facts/predicates that caused the result;
- mapped canonical controls;
- implementation / evidence state;
- control-test effectiveness;
- exceptions;
- missing evidence;
- manual-review requirements.

`NotApplicable` must be justified by **context** (rule + known facts), never by “we have no evidence.” Unknown facts stay unresolved (`ManualDeterminationRequired`), not false.

`project_soa` consumes **generic applicability results** (and, when Prompt 11 lands, a pinned applicability snapshot). It must not be “reread `applicability.toml` and copy booleans.” Pack `applicability.toml` may still declare default rules / structural SoA flags; evaluation is generic.

`SoaEntry.applicable: bool` is not a sufficient public type after this slice. Three-state (or bool + explicit unresolved enum) is required.

Zero subjects ≠ not applicable unless the rule/context says so.

### 4.7 Readiness projection and language

Preserve explicit non-certification language. **Never** emit:

```text
ISO 27001 certified
ISO 27001 compliant
certification guaranteed
audit passed
```

Allowed: `ready`, `effective`, `ineffective`, `partially effective`, `insufficient evidence`, `stale evidence`, `manual review required`, `not applicable`, `assessment coverage`, `partially covered`.

Expose **separate** metrics (counts or ratios — not one compliance percentage):

```text
automation coverage
evidence coverage
subject coverage
control coverage
framework-requirement coverage
```

Do not emit `compliancePercent` / `isoCompliant`. Existing `"NN%"` string invention on `AssessmentReport` serialize / `FrameworkReadinessSnapshot` must be replaced or supplemented so the five metrics are independently inspectable (counts preferred; a percentage of one dimension is allowed only as a derived view of that dimension, never as “ISO compliance %”).

Requirement status must follow the **actual mapping graph** (not “all compiled controls apply to every requirement”). Partial mappings cannot become equivalence (`ComplianceGraph::equivalent` and readiness aggregation).

Every ISO readiness result traces to catalog control id(s), catalog test id(s), and evidence envelope digest(s) or a recorded missing-evidence reason.

### 4.8 Lineage pins

Every ISO assessment lineage record pins at least:

- `frameworkPackDigest` (existing);
- `catalogDigest` (`CanonicalCatalog::digest` display string);
- assessment-definition digest;
- applicability decision identity (when Prompt 11 snapshot exists).

Replay after framework/catalog files change must detect digest mismatch (Prompt 11). This slice requires the pins to be **present on ISO runs**; it does not re-own the ledger API.

`AssessmentRun` today has no `catalog_digest`. Add it (serde default ok) here if Prompt 11 has not already.

### 4.9 Golden scenarios (target suite)

Create/refresh ISO end-to-end fixtures (prefer `fixtures/assurance/iso27001/remap/` or reuse canonical family fixtures by composition). Each is deterministic (`collectedAt` fixed). Expected highlights:

| # | Scenario | Expected |
| --- | --- | --- |
| 1 | Technically strong org **with** governance/manual evidence | Automated mapped tests `Effective`; hybrid/manual `Effective` only with attestations; requirement status remains `partially covered` where mappings are partial |
| 2 | Strong technical controls, **missing** manual governance evidence | Technical tests `Effective`; governance/manual `InsufficientEvidence` or `ManualReviewRequired`; no requirement with only partial mappings becomes fully satisfied |
| 3 | Partial repository / identity population | All-subjects tests `InsufficientEvidence` (not Effective, not Ineffective-as-empty) |
| 4 | Privileged MFA failure | `control.identity.privileged-mfa` / `test.identity.privileged-mfa-enabled` → `Ineffective` naming the subject. Mapping path is `iso27001:a.8.5` → that control. Pack sliver `access.mfa.privileged` is gone |
| 5 | Stale evidence | `StaleEvidence` (not Ineffective-as-missing) |
| 6 | Approved unexpired exception | Bound subject `ExceptionApproved`; expired/revoked does not pass |
| 7 | Applicability-driven not-applicable | SoA entry `NotApplicable` with context rationale; **not** because evidence is missing |
| 8 | Incomplete org context | Applicability `Unresolved` / `ManualDeterminationRequired`; not coerced to NA or applicable |
| 9 | Historical snapshot replay after pack/catalog files change | Pinned pack + catalog digests; mismatch detected; result identity unchanged if snapshots are used |
| 10 | Empty scanner findings + **unknown** coverage | No false-positive `Effective` on vuln/scan controls; unknown coverage ≠ clean. If vuln family is unlanded, assert the unmapped A.8.8 path cannot become Effective via leftover slivers or empty-finding existence checks |

### 4.10 Architecture-boundary tests (target suite)

Assert:

1. Framework pack sources contain **no** provider-specific types (`octocrab`, AWS SDK, `github.com` client types).
2. Collectors contain **no** ISO requirement IDs (`iso27001:`).
3. Control-test runtime contains **no** ISO branches / `iso27001` tokens.
4. Partial mappings cannot become equivalence (IR graph + readiness aggregation).
5. Every mapping `to` resolves via `CanonicalCatalog::control`.
6. Every ISO readiness result traces to canonical controls and evidence (or missing-evidence).
7. SoA uses generic applicability results (three-state + rationale), not a raw pack boolean copy.
8. Assessment lineage pins **both** pack digest and catalog digest.
9. Generic serialize / assess path has no `load_framework_pack("iso-27001", "2022")` literal.
10. Reports/CLI output contain none of the forbidden certification phrases.
11. Coverage fields are separate; no `compliancePercent` / `isoCompliant`.

### 4.11 Dual-suite protocol

Already registered. `tests/sdd` is not auto-discovered.

| Suite | Path | Role |
| --- | --- | --- |
| Baseline | `tests/sdd/iso27001_remap.baseline.rs` · `sdd_iso27001_remap_baseline` | **GREEN on current HEAD** — characterizes §3 (12 tests). Do not weaken. |
| Target | `tests/sdd/iso27001_remap.target.rs` · `sdd_iso27001_remap_target` | **RED on current HEAD**, **GREEN after implement** |

After target GREEN: skip-supersede the remap baseline (`#[ignore = "superseded by sdd_iso27001_remap_target"]`). Do not leave “pack sliver + ISO special-case serialize” as required CI green.

#### Implement order (dual-suite law)

1. **This spec** (done).
2. Keep baseline GREEN (already).
3. **Write ISO-R-001…020 + golden 1–10 + architecture-boundary asserts** in the target file so they FAIL on current sliver HEAD for the right reasons (sliver targets, ISO serialize special-case, no catalog digest, SoA boolean file, loader rejects full relation set) — not unrelated compile noise.
4. Implement owned product files until target GREEN.
5. Skip-supersede remap baseline. Same PR: IAM-008 + `EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES`.

#### Target must assert (false now)

Suggested ids (titles include the id):

| ID | Asserts |
| --- | --- |
| ISO-R-001 | Pack mappings reference existing catalog IDs (`control.identity.privileged-mfa` not `access.mfa.privileged`) |
| ISO-R-002 | Relations used honestly; at least one non-`Equivalent` material mapping; `Equivalent` rows (if any) are full + rationale + provenance |
| ISO-R-003 | Partial / Supports / Related / EvidenceFor / SubsetOf cannot fully satisfy a requirement |
| ISO-R-004 | Every mapping `to` exists in `CanonicalCatalog`; pack validate fails on unknown catalog ids |
| ISO-R-005 | Pack metadata no longer owns a competing control library (no sliver ids; no two IDs for privileged MFA) |
| ISO-R-006 | Pack loader accepts `EvidenceFor`, `SupersetOf`, `SubsetOf` |
| ISO-R-007 | Material mappings carry rationale + provenance; version constraints applied where relevant |
| ISO-R-008 | Generic registry/loader: no ISO pack-load literal in report serialize / generic assess |
| ISO-R-009 | SoA uses generic applicability (applicable / not-applicable / unresolved + justified NA) |
| ISO-R-010 | Lineage pins pack digest **and** catalog digest |
| ISO-R-011 | Ten golden scenarios §4.9 |
| ISO-R-012 | Architecture-boundary tests §4.10 |
| ISO-R-013 | No forbidden certification language in pack, projections, CLI banner, or fixture expected output |
| ISO-R-014 | Separate automation / evidence / subject / control / requirement coverage — no single compliance % |
| ISO-R-015 | Governance/judgement requirements remain Manual/Hybrid; no invented automated tests |
| ISO-R-016 | Collectors still have no ISO requirement IDs; control-test has no ISO branches |
| ISO-R-017 | IAM-008 and `EXPECTED_CANONICAL_CONTROLS` have been superseded in this cut (see §4.12) |
| ISO-R-018 | `weeping-angel assurance framework validate frameworks/iso-27001/2022` and `catalog validate` succeed |
| ISO-R-019 | Structural-only legal boundary still holds (no ISO/IEC normative text in the public pack) |
| ISO-R-020 | Neighbor spine / catalog / IAM (post-supersession) / typed-evidence / population targets stay green |

### 4.12 Neighbor supersession (same implement slice)

Must land in the **same** implement PR as the remapped pack:

1. **IAM-008** — replace “ISO sliver unchanged / must not mention `control.identity.*`” with: ISO mappings for A.8.5 / A.8.2 / A.8.3 / A.5.15 / A.5.18 / A.5.16 / A.6.5 target landed `control.identity.*` ids; pack metadata does **not** declare `access.mfa.privileged` (or siblings) as a second library. IAM-016’s “ISO sliver not rewritten” clause is retired for this reason.
2. **`EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES`** in `tests/sdd/iso27001_assurance.target.rs` — stop requiring pack-local `access.*` / `source.*` slivers as the canonical library. Either update those assertions to catalog prefixes (`control.identity.`, and other landed `control.*` families) **or** skip-supersede the specific tests with a pointer to `sdd_iso27001_remap_target`. ISO-004’s “no `iso27001.` / `.github.` in control ids” stays.

Do not delete the entire MVP target suite; EVD/CTL/GH contracts remain useful.

### 4.13 Documentation after implement

- Accept the ADR (drop `-draft`).
- Update [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md): mapping relation list includes `EvidenceFor` / `SupersetOf` / `SubsetOf`; pack mappings target catalog IDs; reports pin catalog digest; SoA is generic applicability.
- Point IAM / SDLC / vuln specs’ “Prompt 12 will remap” notes at this file’s §13 (implement log).
- Do **not** overwrite Prompt 01–11 SSOTs.

---

## 5. Acceptance criteria

Testable. Product implementation is out of this spec phase.

1. Dual-suite `sdd_iso27001_remap_baseline` + `sdd_iso27001_remap_target` is registered in root `Cargo.toml`. Files are **not** `iso27001_assurance.{baseline,target}.rs`. Baseline stays GREEN on sliver HEAD; target is first authored RED (ISO-R-001…020), then GREEN after implement.
2. On SHA `e430980c…` (current sliver behavior): baseline GREEN; target RED for the right reasons (sliver targets, ISO serialize special-case, no catalog digest, SoA boolean file, loader rejects full relation set) — not unrelated compile noise.
3. After implement: target GREEN; remap baseline skip-superseded; `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --features demo` GREEN.
4. `weeping-angel assurance catalog validate` and `weeping-angel assurance framework validate frameworks/iso-27001/2022` succeed.
5. ISO mappings reference existing catalog control IDs; pack metadata is not a competing control library; no two IDs for the same semantic control.
6. Mapping relations are honest; partial/support/related/evidence-for/subset mappings cannot fully satisfy a requirement; material mappings have rationale + provenance.
7. Pack loader accepts `EvidenceFor`, `SupersetOf`, `SubsetOf`.
8. ISO resolves through the generic framework loader; generic report serialization performs no ISO pack I/O.
9. SoA uses generic applicability results (applicable / not-applicable / unresolved) with justified NA (context, not missing evidence).
10. Assessment lineage pins pack digest **and** catalog digest.
11. Ten golden scenarios and architecture-boundary tests pass.
12. No `certified` / `compliant` / `audit passed` / `certification guaranteed` language; coverage metrics are five separate dimensions, not one compliance percentage.
13. Governance/judgement requirements stay Manual/Hybrid; no invented automated tests; catalog IDs are not renamed for ISO convenience; unlanded families stay unmapped rather than stubbed.
14. Collectors have no ISO requirement IDs; control-test runtime has no ISO branches; pack has no provider types.
15. IAM-008 and ISO MVP `EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES` are superseded in the same implement slice.
16. Structural-only legal boundary preserved; licensed/user-supplied narrative still layers via content-provider abstraction.
17. Public contract / ADR updated when mapping/framework contracts land.

---

## 6. Out of scope

- SOC 2, NIS2, DORA, PCI, or HIPAA framework packs.
- Changing canonical catalog IDs so ISO mapping is easier.
- Redesigning canonical controls, tests, or evidence contracts (Prompts 04–08).
- Implementing missing domain catalog families (05–08) inside this slice (map only what exists).
- Provider collectors / APIs (GitHub, AWS, Entra, scanners).
- Scanner engine changes; empty findings still must not become false-positive effectiveness (scenario 10 consumes existing evidence types).
- Auditor or certification equivalence claims; certification automation.
- Replacing the MVP dual-suite wholesale; rewriting Prompt 01–11 SSOTs.
- Implementing Prompt 10’s org-context evaluator **as an ISO-only fork** (consume generic engine; if absent, still use a generic result type).
- Inventing a second lineage/ledger model (Prompt 11).
- Expanding `requirements.toml` with extra Annex A clauses to inflate mapping counts.
- Editing `tests/sdd/github_collector.*` or `tests/sdd/assessment_lineage.*`.

---

## 7. Risks

| Risk | Mitigation |
| --- | --- |
| Prompts 05–08 / 10 / 11 still in flight | Map only landed catalog IDs; consume Prompt 11 generic facade if present; do not invent IDs or a second registry |
| IAM-008 / `EXPECTED_CANONICAL_CONTROLS` left unchanged | Same-slice supersession is an acceptance gate (ISO-R-017) |
| `Equivalent` used to “finish” Annex A | Target suite forbids convenience equivalence; partial never fully satisfies |
| Pack validate still requires mapping `to` ∈ `metadata.toml` | Change loader/validate to resolve against catalog; empty competing control list |
| Framework crate depending on catalog crate | Keep catalog I/O at orchestrator/CLI; document the seam in the ADR |
| SoA still copies pack booleans | Target asserts three-state generic results + justified NA |
| Serialize still loads ISO pack | Shared with Prompt 11; this slice fails if the literal remains in generic serialize |
| Legal: normative ISO text sneaks into remapped titles/rationales | Structural-only + validate markers; short titles only |
| Historical assessments break when sliver IDs vanish | Prompt 11 digest pins; scenario 9; old runs keep old pack digest |
| Readiness still attaches every control to every requirement | Fix projection to walk the mapping graph |
| Over-claiming automation on clauses 4–10 | Manual/Hybrid honesty + no invented tests |
| Fixture `control.source.protected-branch` treated as full SDLC coverage | Exists-only; Partial/Supports at most; scenario 10 / A.8.25 cannot become fully satisfied |

---

## 8. ADR

`adr_needed = true`. Public contracts change: mapping targets, pack metadata role, loader relation set, SoA input type, lineage catalog digest, generic (non-ISO-special) resolution.

Draft: [`docs/adr/0003-iso27001-canonical-remap-draft.md`](../adr/0003-iso27001-canonical-remap-draft.md). Accept after target GREEN; update `docs/contracts/assurance-runtime.md` in implement.

---

## 9. Final verification (implement)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --features demo
weeping-angel assurance catalog validate
weeping-angel assurance framework validate frameworks/iso-27001/2022
cargo test --test sdd_iso27001_remap_target --offline
```

Run architecture-boundary tests in the remap target suite.

---

## 10. Definition of done

ISO 27001:2022 is a clean **framework projection** over the canonical assurance catalog: pack content is data, mappings are explicit and defensible, controls/tests/evidence remain framework-neutral, applicability is contextual and explainable, historical lineage pins both framework and catalog digests, and the system produces readiness/SoA output without certification claims.

---

## 11. Implement sequence (next slice)

1. Author ISO-R-001…020 + golden 1–10 + architecture-boundary tests in `tests/sdd/iso27001_remap.target.rs` until `cargo test --test sdd_iso27001_remap_target --offline` is **RED for the right reasons**.
2. Remap `frameworks/iso-27001/2022` onto landed catalog IDs; retire pack slivers.
3. Loader accepts eight relations + provenance/`valid_for`; validate `to` against catalog at CLI/orchestrator seam.
4. Remove ISO pack-load literals from generic serialize/assess; pin `catalogDigest` on `AssessmentRun` / readiness if Prompt 11 has not.
5. SoA consumes generic three-state applicability; readiness walks the mapping graph; five coverage metrics.
6. Supersede IAM-008 and `EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES` in the same PR.
7. Target GREEN; skip-supersede remap baseline; accept ADR; update public contract.

---

## 13. Implement log

- 2026-08-19: remapped `frameworks/iso-27001/2022` onto landed catalog IDs (`control.identity.*` plus landed `control.source.*` SDLC). Pack slivers retired. Loader accepts all eight IR relations + provenance/`valid_for`. Generic `(id, version)` load/serialize; `catalogDigest` pinned. SoA is three-state. IAM-008 / IAM-016 / EXPECTED_CANONICAL_CONTROLS / CANONICAL_CONTROL_PREFIXES superseded. Remap baseline skip-superseded.
- Catalog families mapped: identity (required remaps) + SDLC (A.8.25 / A.8.26). Unlanded vuln/infra/governance remain unmapped.
