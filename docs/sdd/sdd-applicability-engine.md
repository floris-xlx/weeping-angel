# SDD run: Organization Context and Applicability Engine

| Field | Value |
| --- | --- |
| Run id | `sdd-applicability-engine` (protocol report; implement assigns telemetry id) |
| Date | 2026-08-19 |
| Workflow | `spec-driven-development` (dual-suite) |
| Status | **Specified** — durable SSOT + draft ADR; **no product feature code**; dual-suite registered; baseline GREEN on current HEAD |
| Slice | Prompt 10: make IR `ApplicabilityRule` / `ApplicabilityPredicate` operational via a generic Kleene evaluator + snapshot |
| Spec | [`docs/sdd/applicability-engine.md`](applicability-engine.md) |
| Draft ADR | [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md) |
| Source prompt | [`docs/prompts/canonical-assurance-v1/10-applicability-engine.md`](../prompts/canonical-assurance-v1/10-applicability-engine.md) |
| Characterization SHA | `e430980c0d27a8138a153d49b62ddf3c57827891` |
| Dual-suite (at implement) | `tests/sdd/applicability_engine.baseline.rs` · `tests/sdd/applicability_engine.target.rs` → `sdd_applicability_engine_baseline` / `sdd_applicability_engine_target` |
| Public contract | [`docs/contracts/assurance-runtime.md`](../contracts/assurance-runtime.md) — update on implement |
| Consumes | Prompts 01–08 + population runtime. IR inventories + `AssessmentScope`. Prompt 11 reserved snapshot shape (fill, do not persist). |
| Collision fence (Prompt 09) | Do not touch `crates/weeping-angel-collector/**`, `tests/sdd/github_collector.*`, `docs/sdd/github-collector.md`, `GITHUB_EVIDENCE_TYPES` |
| Collision fence (Prompt 11) | Do not implement explain/ledger. Own only engine / evaluator / snapshot paths. |

Durable proof for this SDD run. Product spec lives in the linked SDD; this file records protocol evidence, gates, and (later) telemetry.

Treat the prior Prompt 10 session (never started; 4-run cap) as abandoned. This report is a **fresh start** against HEAD `e430980c…`.

---

## Spec

- **Title:** Organization Context and Applicability Engine
- **Problem:** IR `ApplicabilityRule` / `ApplicabilityPredicate` trees exist on every control and requirement but nothing evaluates them against organization or assessment scope. Compile only drops static `Never`. SoA rereads ISO pack booleans. Unknown facts are never resolved; reviewers cannot explain why a control applied, did not apply, which fact was unknown, or which exclusion removed a subject.
- **Current behavior (SHA `e430980c…`):** IR module is declarative (`does not evaluate platform facts`). `statically_applicable` is `Some(true/false)` for `Always`/`Never` and boolean combos, `None` for every predicate (`Not(None)` stays `None`). `resolve_applicability` keeps a requirement unless `statically_applicable() == Some(false)`. SoA `project_soa` copies `applicability.toml` `applicable: bool`. No evaluator module, no `ApplicabilitySnapshot`, no org-context builder. Inventories (`Asset`, `Identity`, `Vendor`, `ProcessingActivity`, `Risk`, `AssessmentScope`) exist and are unused for applicability. Facade `AssessmentScope` is a collector allow-set. Population runtime injects subjects via `EvidenceSet::set_population` and does not walk IR inventories. `Control.subjects` is a private field with no getter. Prompt 11 baseline asserts Prompt 10 absent.
- **Desired behavior:** A network-free generic evaluator in `weeping-angel-assurance` (applicability engine / evaluator / snapshot) builds a **derived** `ApplicabilityContext` from existing IR inventories + scope + explicit tri-state facts (not a second inventory). Kleene evaluation of `Always` / `Never` / `All` / `Any` / `Not` / `Predicate` yields `Applicable` \| `NotApplicable` \| `ManualDeterminationRequired` plus ordered rationale, contributing predicates, unknown facts, selected subjects, and exclusion reasons. Unknown ≠ false; `Not(Unknown)` stays unknown. Zero selected subjects ≠ `NotApplicable` unless the rule is false. Same engine for controls and requirements. `evaluate_assessment_applicability` fills Prompt 11’s reserved `ApplicabilitySnapshot` shape. Compile may drop only `NotApplicable` when a context is supplied. No framework/provider branches, no pack-TOML evaluator, no IR fact engine.
- **ADR:** needed — draft at [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md)

### Acceptance criteria (this slice)

1. Dual-suite registered at implement as `sdd_applicability_engine_baseline` / `sdd_applicability_engine_target`.
2. Baseline GREEN on current static-only / no-evaluator behavior; target RED on current code for §6.2 cases, then GREEN after implement.
3. Static `Always`/`Never` map to Applicable / NotApplicable.
4. Known true/false predicates resolve; unknown predicates are `ManualDeterminationRequired`, never coerced to false.
5. Nested `All`/`Any`/`Not` with unknowns follow Kleene K3; `Not(Unknown)` remains unknown.
6. Jurisdiction, cloud (authoritative-empty vs unknown), personal-data (known vs unknown), vendor presence, and explicit exclusions behave as specified.
7. Deterministic rationale / subject ordering and stable snapshot digest.
8. Zero selected subjects do not flip the decision to `NotApplicable`.
9. Snapshot fields match lineage persist shape; this slice does not persist or explain.
10. No catalog TOML / ISO `applicability.toml` / collector / Prompt 09 file edits; IR stays declarative.
11. Workspace verify after implement: `cargo test --workspace --features demo`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Out of scope

- Framework-specific applicability branches
- Pack `applicability.toml` as a second evaluator
- Provider API calls; GitHub collector tree
- Generic ontology engine
- Canonical catalog redesign
- Prompt 11 explain / ledger persist
- Collapsing the two `AssessmentScope` types
- Growing IR privacy/risk/vendor records into full domain models
- Certification language

### Risks (tracked)

| Risk | Disposition this run |
| --- | --- |
| Unknown-as-false | Normative Kleene + T03/T07/T08/T13 |
| `Not(Unknown)` → true | Explicit law + target test |
| Second inventory | Derived context over IR types only |
| IR fact engine | Evaluator in assurance crate |
| Zero-pop auto-NA | T12 |
| Prompt 09 collision | Hard file fence |
| Prompt 11 absence asserts vs required `ApplicabilitySnapshot` | Fill reserved shape; lineage run skip-supersedes absence tests after this lands |
| SoA boolean path | Unchanged this slice |
| Empty list treated as authoritative false | Completeness defaults to Unknown |

---

## Protocol proof

| Step | Expected | Actual |
| --- | --- | --- |
| Spec | written | [`docs/sdd/applicability-engine.md`](applicability-engine.md) — fresh-start SSOT, re-characterized against HEAD `e430980c…` |
| ADR draft | written (architecture/contract) | [`docs/adr/0003-applicability-engine.md`](../adr/0003-applicability-engine.md) |
| Baseline | PASS on old | **GREEN** — `sdd_applicability_engine_baseline` 18 passed; `sdd_applicability_engine_target` registered (harness only; P10-T01..T16 still implement-phase RED) |
| Target pre | FAIL on old | **Not yet authored** — P10-T01..T16 land in the RED implement phase before product code |
| Implement | target PASS | *pending* |
| Baseline post | FAIL or skip-supersede | *pending* |
| Supersede | target still PASS | *pending* |
| Docs/ADR | updated | Spec + draft ADR written; public contract + ADR accept at implement |
| Workspace verify | after implement | *pending* |

`verify_ok` is **false** until target GREEN and baseline superseded. Spec-only gate for this phase is: durable spec + draft ADR + no product feature code.

---

## What must not be edited (collision fence)

```text
crates/weeping-angel-collector/**
tests/sdd/github_collector.baseline.rs
tests/sdd/github_collector.target.rs
docs/sdd/github-collector.md
docs/sdd/sdd-github-collector.md
GITHUB_EVIDENCE_TYPES
catalog/canonical/v1/**          (catalog TOML)
frameworks/**/applicability.toml
Prompt 11 explain/ledger product code
```

Own files at implement: `crates/weeping-angel-assurance/src/applicability/**` (and/or equivalent evaluator/snapshot module), optional thin control-test helper, `tests/sdd/applicability_engine.*`, this spec/report, draft ADR. Tiny IR getter (`Control::subjects`) allowed.
