# SDD run: ISO 27001:2022 remapping onto the Canonical Assurance Catalog

| Field | Value |
| --- | --- |
| Run id | `sdd-325e52763f0a` |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` |
| Objective fingerprint | `325e52763f0aff14` |
| Status | **Complete** — protocol gates closed; `verify_ok` |
| Slice | Prompt 12: ISO 27001:2022 pack remaps onto landed `control.*` catalog IDs; honest IR relations; generic load/SoA/lineage; no certification claims |
| Spec | [`docs/sdd/iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) |
| ADR | Accepted [`docs/adr/0003-iso27001-canonical-remap.md`](../adr/0003-iso27001-canonical-remap.md) (draft path retired) |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/12-iso27001-remap.md`](../prompts/canonical-assurance-v1/12-iso27001-remap.md) |
| Telemetry | [`sdd-iso-27001-canonical-remap-telemetry.json`](sdd-iso-27001-canonical-remap-telemetry.json) |
| Dual-suite | `tests/sdd/iso27001_remap.baseline.rs` (skip-superseded) · `tests/sdd/iso27001_remap.target.rs` (active; ISO-R-001…020 + goldens 1–10) |
| Do **not** reuse | `tests/sdd/iso27001_assurance.{baseline,target}.rs` (MVP dual-suite) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Collision fence | Do not edit `tests/sdd/github_collector.*` or `tests/sdd/assessment_lineage.*`. Do not invent unlanded catalog families. |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and telemetry.

This report is the **finalize** artifact for telemetry run `sdd-325e52763f0a` against characterization SHA `e430980c…`.

---

## Spec

- **Title:** ISO 27001:2022 remapping onto the Canonical Assurance Catalog
- **Problem:** ISO 27001:2022 still projects through a pack-local sliver library (`access.mfa.privileged`, `source.branch-protection`, …) and ISO-only load/serialize/SoA paths, while the reusable library now lives in `catalog/canonical/v1` (23 `control.identity.*` plus fixture `control.source.protected-branch`). Two ID spaces and hardcoded ISO branches prevent a clean requirement→mapping→canonical control→test→evidence chain.
- **Current behavior (SHA `e430980c`):** The pack is StructuralOnly with 42 requirement ids, 27 mappings onto slivers (`PartiallySatisfies`/`Supports`/`Related` only; no provenance/`valid_for`), 10 `applicability.toml` booleans, and 22 pack-local controls/tests. Pack loader rejects `EvidenceFor`/`SupersetOf`/`SubsetOf` and treats catalog IDs as dangling. `normalize`/`stub_catalog`/`assessment_for_target`/`AssessmentReport::serialize`/`assess` hard-load `iso-27001`/`2022`. SoA copies file booleans. `AssessmentRun` and readiness pin `framework_pack_digest` only (no `catalog_digest`). Readiness attaches every compiled control to every requirement and invents `NN%` coverage. Dual-suite `sdd_iso27001_remap_{baseline,target}` is registered; baseline 12 tests GREEN; target is a registration stub. IAM-008 and `EXPECTED_CANONICAL_CONTROLS` still freeze slivers.
- **Desired behavior:** ISO is a data projection over the landed catalog: mappings target existing `control.*` IDs only (identity remaps required; unlanded families stay unmapped, not stubbed). Honest rich relations; Partial/Supports/Related/EvidenceFor/SubsetOf never fully satisfy. Loader accepts all eight IR relations plus rationale/provenance/`valid_for`. `metadata.toml` is not a competing library. ISO resolves via generic `(id, version)` loader; generic serialize performs no ISO pack I/O. SoA consumes three-state generic applicability with context-justified NA. Lineage pins pack and catalog digests. Five separate coverage metrics; no certified/compliant/audit-passed language. Golden scenarios 1–10 and ISO-R-001…020 go RED then GREEN; IAM-008 and `EXPECTED_CANONICAL_CONTROLS` superseded in the same implement PR.
- **ADR:** needed — accepted at [`docs/adr/0003-iso27001-canonical-remap.md`](../adr/0003-iso27001-canonical-remap.md)

### Acceptance criteria (this slice)

1. Dual-suite `sdd_iso27001_remap_{baseline,target}` remains registered and is not `iso27001_assurance.*`; baseline GREEN on sliver HEAD until skip-superseded.
2. Target first encodes ISO-R-001…020 + golden 1–10 + architecture-boundary asserts that RED on current sliver HEAD for the right reasons.
3. After implement: target GREEN; remap baseline skip-superseded; `fmt`, `clippy -D warnings`, and `cargo test --workspace --features demo` GREEN.
4. `weeping-angel assurance catalog validate` and `framework validate frameworks/iso-27001/2022` succeed.
5. Mappings reference existing CanonicalCatalog control IDs; no pack sliver library; no two IDs for privileged MFA.
6. Honest relations: Partial/Supports/Related/EvidenceFor/SubsetOf cannot fully satisfy; material mappings have rationale + provenance.
7. Pack loader accepts EvidenceFor, SupersetOf, SubsetOf and still rejects unknown relations.
8. Generic serialize/assess has no `load_framework_pack("iso-27001", "2022")` literal; ISO resolves by target identity.
9. SoA uses generic Applicable/NotApplicable/Unresolved with NA justified by context, not missing evidence.
10. Assessment lineage pins `frameworkPackDigest` and `catalogDigest`.
11. Five separate coverage metrics; never certified/compliant/audit passed/certification guaranteed.
12. Collectors have no `iso27001:` IDs; control-test has no ISO branches; pack has no provider types; StructuralOnly legal boundary holds.
13. IAM-008 and `EXPECTED_CANONICAL_CONTROLS`/`CANONICAL_CONTROL_PREFIXES` superseded in the same implement slice.
14. Unlanded SDLC/vuln/infra/governance families stay unmapped; catalog IDs are not renamed for ISO convenience.

### Out of scope

- SOC 2 / NIS2 / DORA / PCI / HIPAA packs
- Changing canonical catalog IDs to ease mapping
- Redesigning canonical controls, tests, or evidence contracts
- Implementing missing domain catalog families 05–08
- Provider collectors/APIs or collector dual-suite files
- Scanner engine changes
- Auditor or certification claims
- Replacing the MVP `iso27001_assurance` dual-suite wholesale
- ISO-only applicability evaluator fork (Prompt 10)
- Second lineage/ledger model (Prompt 11)
- Expanding `requirements.toml` with extra Annex A clauses
- Editing `tests/sdd/github_collector.*` or `tests/sdd/assessment_lineage.*`

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Prompts 05–08 / 10 / 11 still in flight; map only landed IDs | Mappings target landed catalog IDs (identity + SDLC present at implement); unlanded families stay unmapped. Consumed generic applicability/lineage facade. |
| Leaving IAM-008 or `EXPECTED_CANONICAL_CONTROLS` unchanged keeps workspace tests red | Superseded in the same implement slice (`iso_r_017`). |
| Convenience Equivalent or leftover slivers would falsify readiness | Honest relations only; Partial/Supports/Related/EvidenceFor/SubsetOf cannot fully satisfy; pack slivers retired. |
| Pack validate still requiring mapping `to` ∈ `metadata.toml` would reject catalog targets | Loader validates catalog targets; `metadata.toml` is not a competing control library. |
| Framework crate taking a hard catalog dependency violates ACT-003 | Documented seam; framework loader accepts catalog-targeted mappings without becoming a second catalog. |
| SoA remaining a boolean file copy, or serialize still loading the ISO pack | SoA uses generic three-state applicability; generic serialize has no ISO pack literal. |
| Normative ISO/IEC wording in remapped titles/rationales | StructuralOnly legal boundary holds; no certification language. |
| Historical runs break without dual digest pins when sliver IDs vanish | Lineage pins `frameworkPackDigest` and `catalogDigest`. |
| Readiness still attaching every control to every requirement | Readiness projected as a catalog graph with five inspectable coverage metrics. |
| Exists-only fixture `control.source.protected-branch` treated as full A.8.25 coverage | Fixture stays exists-only; unlanded families stay unmapped rather than over-claimed. |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md) |
| Baseline | PASS on old | `cargo test --offline --test sdd_iso27001_remap_baseline --test sdd_iso27001_remap_target` → exit 0. Existing remap baseline characterizes sliver HEAD (12 GREEN). Target remains a registration stub. No feature implementation. Excerpt: `test result: ok. 12 passed; 0 failed; 0 ignored` / `test result: ok. 1 passed; 0 failed`. Suite: `tests/sdd/iso27001_remap.baseline.rs`. |
| Target pre | FAIL on old | Same dual command → exit 1. Target encodes ISO-R-001…020 + goldens 1–10 + architecture bounds. Baseline untouched and GREEN (12 passed). No product code changed. Four hold-the-line target tests pass (registration, legal boundary, collector neutrality, no certification phrases). Excerpt: `sdd_iso27001_remap_baseline: test result: ok. 12 passed; 0 failed` / `sdd_iso27001_remap_target: test result: FAILED. 4 passed; 26 failed`. RED reasons: `access.mfa.privileged` still mapped; loader rejects EvidenceFor; no provenance/`valid_for`; serialize hard-loads `iso-27001`/`2022`; SoA booleans only; `catalogDigest` missing; IAM-008 not superseded; A.8.8 still stubbed. Suite: `tests/sdd/iso27001_remap.target.rs`. |
| Implement | target PASS | `cargo test --offline --test sdd_iso27001_remap_baseline --test sdd_iso27001_remap_target -- --test-threads=1` → target **ok. 30 passed; 0 failed** (ISO-R-000…020 + goldens 1–10). Remapped pack onto landed catalog IDs (identity + SDLC); retired slivers; full IR relation set with provenance/`valid_for`; generic `(id, version)` load; SoA/readiness as non-certifying catalog graph; dual pack/catalog digest pins. Mid-implement remap baseline skip-superseded: **ok. 0 passed; 0 failed; 12 ignored**. Files: `frameworks/iso-27001/2022/{mappings,metadata,applicability}.toml`, `crates/weeping-angel-framework/src/{pack,lib}.rs`, `crates/weeping-angel-assurance/{Cargo.toml,src/{lib,readiness,snapshot,soa}.rs}`, neighbor SDD suites, spec/contract/ADR draft. |
| Baseline post | FAIL or retired | Skip-supersede (`supersede_kind=skip`). Default dual run: baseline **ok. 0 passed; 0 failed; 12 ignored** (`superseded by sdd_iso27001_remap_target`). Not a live sliver-world pass. File kept registered because the target suite asserts dual-suite registration. Not additive (`additive_baseline=false`, `baseline_not_green=true`, `baseline_retired=true`). |
| Supersede | target still PASS | After skip-supersede: `sdd_iso27001_remap_target` **ok. 30 passed; 0 failed; 0 ignored** (ISO-R-001…020 + goldens). `target_still_green=true`. Baseline file stays registered; all 12 tests remain ignored. |
| Docs/ADR | updated | [`docs/adr/0003-iso27001-canonical-remap.md`](../adr/0003-iso27001-canonical-remap.md), [`docs/sdd/iso-27001-canonical-remap.md`](iso-27001-canonical-remap.md), [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md), [`README.md`](../../README.md), [`frameworks/README.md`](../../frameworks/README.md), [`docs/sdd/iso-27001-automated-assurance-mvp.md`](iso-27001-automated-assurance-mvp.md), [`docs/sdd/iam-canonical-assurance-catalog.md`](iam-canonical-assurance-catalog.md), [`docs/sdd/sdlc-canonical-assurance-catalog.md`](sdlc-canonical-assurance-catalog.md), [`docs/sdd/vulnerability-canonical-assurance-catalog.md`](vulnerability-canonical-assurance-catalog.md), [`docs/sdd/infrastructure-canonical-assurance-catalog.md`](infrastructure-canonical-assurance-catalog.md), [`docs/adr/0001-inwardly-extensible-assurance-runtime.md`](../adr/0001-inwardly-extensible-assurance-runtime.md), [`docs/adr/0002-iso-27001-assurance-vertical.md`](../adr/0002-iso-27001-assurance-vertical.md), [`docs/adr/0003-canonical-assurance-catalog-v1.md`](../adr/0003-canonical-assurance-catalog-v1.md), [`docs/adr/0003-iam-canonical-assurance-catalog.md`](../adr/0003-iam-canonical-assurance-catalog.md), [`docs/adr/0003-sdlc-canonical-assurance-catalog.md`](../adr/0003-sdlc-canonical-assurance-catalog.md), [`docs/adr/0003-vulnerability-canonical-assurance-catalog.md`](../adr/0003-vulnerability-canonical-assurance-catalog.md), [`docs/adr/0003-governance-canonical-assurance-catalog.md`](../adr/0003-governance-canonical-assurance-catalog.md), [`docs/adr/0003-infrastructure-canonical-assurance-catalog.md`](../adr/0003-infrastructure-canonical-assurance-catalog.md) |

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

ISO 27001:2022 is a data projection over the landed Canonical Assurance Catalog:

- Mappings target existing `control.*` IDs (identity remaps required; SDLC mapped where landed). Pack slivers retired. No two IDs for privileged MFA.
- Honest IR relations: Partial / Supports / Related / EvidenceFor / SubsetOf cannot fully satisfy. Material mappings carry rationale, provenance, and `valid_for`.
- Pack loader accepts all eight IR relations (`EvidenceFor`, `SupersetOf`, `SubsetOf` included) and still rejects unknown relations. `metadata.toml` is not a competing control library.
- Generic serialize/assess has no `load_framework_pack("iso-27001", "2022")` literal; ISO resolves by target identity `(id, version)`.
- SoA uses generic Applicable / NotApplicable / Unresolved; NA is context-justified, not missing evidence.
- Assessment lineage pins `frameworkPackDigest` and `catalogDigest` (plus `canonicalCatalogDigest` on report/run/readiness).
- Five separate coverage metrics; no certified / compliant / audit-passed / certification-guaranteed language.
- Collectors have no `iso27001:` IDs; control-test has no ISO branches; pack has no provider types; StructuralOnly legal boundary holds.
- IAM-008 and `EXPECTED_CANONICAL_CONTROLS` / `CANONICAL_CONTROL_PREFIXES` superseded in the same implement slice.
- Unlanded vuln/infra/governance families stay unmapped; catalog IDs are not renamed for ISO convenience.

### Files changed (implement)

`frameworks/iso-27001/2022/mappings.toml`, `frameworks/iso-27001/2022/metadata.toml`, `frameworks/iso-27001/2022/applicability.toml`, `crates/weeping-angel-framework/src/pack.rs`, `crates/weeping-angel-framework/src/lib.rs`, `crates/weeping-angel-assurance/Cargo.toml`, `crates/weeping-angel-assurance/src/lib.rs`, `crates/weeping-angel-assurance/src/readiness.rs`, `crates/weeping-angel-assurance/src/snapshot.rs`, `crates/weeping-angel-assurance/src/soa.rs`, `tests/sdd/iso27001_remap.baseline.rs`, `tests/sdd/iam_catalog.target.rs`, `tests/sdd/iso27001_assurance.target.rs`, `tests/sdd/canonical_assurance_catalog.target.rs`, `tests/sdd/applicability_engine.baseline.rs`, `tests/sdd/sdlc_catalog.target.rs`, `tests/sdd/vulnerability_catalog.target.rs`, `docs/sdd/iso-27001-canonical-remap.md`, `docs/contracts/assurance-runtime.md`, `docs/adr/0003-iso27001-canonical-remap-draft.md`.

---

## Telemetry

| Metric | Value |
| --- | --- |
| `telemetry_run_id` | `sdd-325e52763f0a` |
| `agents_ok` | 8 |
| `agents_fail` | 0 |
| `agents_total` | 8 |
| `tokens_used_sum` | 16 752 219 |
| `duration_ms_sum` | 5 169 404 (~86.2 min) |
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
| Scope | `sdd-scope` | ok | 372 242 | 323 438 |
| Spec | `sdd-spec` | ok | 583 659 | 913 075 |
| BaselineGreen | `sdd-baseline-green` | ok | 53 028 | 128 782 |
| TargetRed | `sdd-target-red` | ok | 688 308 | 4 160 617 |
| Implement | `sdd-implement` | ok | 1 104 479 | 6 699 499 |
| DocsAdr | `sdd-docs-adr` | ok | 1 500 200 | 2 459 593 |
| Iterate | `sdd-baseline-post-check` | ok | 770 113 | 1 936 757 |
| Supersede | `sdd-supersede` | ok | 97 375 | 130 458 |

No iterate-repair loops (`iters_used=0`). Finalize writes this report after the snapshot above (`reason: pre_finalize`).

Full event array: [`sdd-iso-27001-canonical-remap-telemetry.json`](sdd-iso-27001-canonical-remap-telemetry.json).

---

## Remaining backlog (not this slice)

1. SOC 2 / NIS2 / DORA / PCI / HIPAA packs
2. Implementing missing domain catalog families 05–08 (vuln / infra / governance remain unmapped)
3. Provider collectors/APIs or collector dual-suite files
4. Scanner engine changes
5. Replacing the MVP `iso27001_assurance` dual-suite wholesale
6. ISO-only applicability evaluator fork (Prompt 10)
7. Second lineage/ledger model (Prompt 11)
8. Expanding `requirements.toml` with extra Annex A clauses
9. Changing canonical catalog IDs to ease mapping (forbidden)
10. Auditor or certification claims (forbidden)

---

## Summary

Prompt 12 ISO remap landed under dual-suite SDD: spec + accepted ADR 0003 (draft finalized), baseline GREEN on SHA `e430980c` (12 passed characterizing the sliver pack), target RED (26 failed / 4 hold-the-line passed) for sliver mappings, rejected EvidenceFor, missing provenance/`catalogDigest`, ISO hard-load, boolean SoA, and unsuperseded IAM-008, then target GREEN 30/30. Remap baseline skip-superseded (12 ignored; not a live sliver-world pass). ISO is a catalog projection: honest relations, generic `(id, version)` load, three-state SoA, dual pack/catalog digest pins, five coverage metrics, StructuralOnly legal boundary. No certification language.
