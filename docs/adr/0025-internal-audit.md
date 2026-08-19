# ADR 0025 — Internal audit as an operational evidence-backed process

<!-- weeping-angel-adr-meta
id = "0025"
status = "accepted"
supersedes = []
superseded_by = []
depends_on = []
-->


| Field | Value |
| --- | --- |
| Status | **Accepted** — `sdd_internal_audit_target` GREEN; baseline skip-superseded |
| Date | 2026-08-19 |
| Deciders | Weeping Angel maintainers |
| Supercedes | Nothing. Does **not** replace ADR 0001 compile pipeline, ADR 0002 ISO vertical, ADR 0003 assessment-lineage snapshot law, or ADR 0004 documentation architecture. |
| Extends | [ADR 0001](0001-inwardly-extensible-assurance-runtime.md) (graph + fail-closed capabilities), [ADR 0003 assessment lineage](0015-assessment-lineage.md) (immutable `EvidenceSnapshot` / `AssessmentRun` pins), [ADR 0003 governance catalog](0021-governance-canonical-assurance-catalog.md) (freshness facts, not “audit passed”), [temporal assurance](0018-evidence-validity-temporal-assurance.md) (period / as-of consume) |
| Spec | [`docs/specs/internal-audit.md`](../specs/internal-audit.md) |
| Public contract | [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md) |
| Characterization | `6e31bf1ae8f4a69227e0557d878f2e76d0cb8f2a` |
| Tests | `sdd_internal_audit_target` GREEN (IA-001–IA-009); `sdd_internal_audit_baseline` skip-superseded (`#[ignore = "superseded by sdd_internal_audit_target"]`) |

> Filename `0003-*` is shared with catalog-program siblings. **0004** is documentation architecture. Cite this decision by **path**.

## Context

On SHA `6e31bf1` the IR already had `AuditProgramId` and fail-closed `AssessmentRequests.audit_program` / `FrameworkCapabilities.supports_audit_program` (and the same pattern for `sampling`). `FrameworkProfile::Iso27007` was a compile selector with **no** pack. Governance catalog attested that an `evidence.governance.internal-audit` record is fresh (`audited_at` within 365d) and that a program is manually attested. Lineage could pin envelope digests on `EvidenceSnapshot` / `AssessmentRun`.

That still meant: no `AuditProgram` / `Audit` documents, no assessment inventories, no reproducible sample engine, no independence declaration, no audit finding type, no human sign-off, and no binding between an auditor’s review and an immutable snapshot. Enabling `supports_audit_program` compiled nothing. A later `assess()` could change what the organization appeared to have audited.

Operational ISMS v1 Prompt 21 requires internal audit to become a first-class process **on the same evidence graph**, while preserving auditor independence and judgment.

Questions this decision answers:

1. Are audit programs IR documents or folders of markdown?
2. May the machine conclude or sign an audit from green control tests?
3. How does sampling stay explicit and reproducible?
4. How is “what the auditor reviewed” protected from later graph mutation?
5. How do findings relate to scanner `Finding` and Prompt 22 CAPA?
6. What does `Iso27007` mean after this slice?

## Decision

Field names and gates are specified in [`docs/specs/internal-audit.md`](../specs/internal-audit.md). Landed in `weeping-angel-assurance-ir::audit` + `weeping-angel-assurance::audit`; schema stays `assurance-ir/v1`.

### 1. Programs and audits are IR, not a side folder

`AuditProgram` and child `Audit` hang on `AssessmentDefinition` as additive `#[serde(default)]` inventories (`audit_programs`, `audits`). Findings are a **side inventory** `audit_findings`; `Audit.findings` is the id list. Keep `AuditProgramId`; add `AuditId` / `AuditFindingId`. No new crate. No new database.

Existing `audit_program` / `supports_audit_program` (and `sampling` / `supports_sampling`) remain fail-closed compile gates. Flags do not replace documents. `validate_assessment_ir` walks inventories whenever they are non-empty (cannot smuggle dangling programs past validation).

### 2. The machine prepares; humans judge and sign

Landed engine (`weeping-angel-assurance::audit`):

```text
prepare_audit_program / prepare_audit
propose_sample / accept_sample
pin_evidence / record_finding
conclude_audit / sign_off
replay_audit / reviewed_envelopes
```

Prepare emits candidate scope, stale/failed controls (`Ineffective` / `StaleEvidence` / `InsufficientEvidence` / `PartiallyEffective`), open `Risk` ids, an advisory sample proposal, prior findings from **signed** audits, and empty remediation refs.

Forbidden and not present:

- defaulting `Audit.conclusion` or `Audit.signOff` from `Effectiveness`
- auto-accepting a sample proposal
- auto-accepting independence (`accepted: true` is never written by prepare)
- auto-signing (`AuditSignOff` has no `Default`)

Incomplete audits (missing accepted sample, evidence pin, independence declaration, or unfinished procedures) cannot conclude. `sign_off` requires a human `PrincipalRef`, non-empty statement, and an explicit `AuditConclusion` other than `notConcluded`. Signed / withdrawn audits freeze sample, pin, and findings.

### 3. Sampling is a digested plan, not a vibe

An accepted `AuditSample` records population digest, method (`census` \| `systematic` \| `seededRandom` \| `judgmental`), seed (required for systematic / seededRandom), selected ids, acceptor, and `sampleDigest` (`sha256:` + IR `canonical_digest`). Same inputs ⇒ same selection. Population members are sorted unique before selection.

`AuditSampleProposal` is a distinct type (`kind = "proposal"`). Attaching it does not set `Audit.sample`. `propose_sample` refuses `judgmental`; that method requires the auditor to supply `selectedIds`.

### 4. Review is pinned to lineage snapshots

`pin_evidence(audit, snapshot, principal, clock)` copies `EvidenceSnapshot.digest`, `envelope_digests`, and `collection_run_ids`, plus the audit `period`. It does not reseal envelopes. After pin, live collect/assess must not mutate the pin. `replay_audit` / `reviewed_envelopes` return the pinned digest set, not the current ledger prefix.

Optional IR fields for `AssessmentRun` / pack / catalog / `asOf` exist on `AuditEvidencePin` but are not filled by this engine path. Do not fork a second snapshot schema; reuse `weeping-angel/assessment-lineage/v1`.

### 5. Findings are auditor records

`AuditFinding` is not `src/finding.rs` `Finding` and not Prompt 22 `Nonconformity`. Failed tests may appear on the prepare bundle as candidates only. `record_finding` is explicit. Optional opaque `nonconformityId` (`NonconformityRef = String`) is a seam for Prompt 22. This slice does not implement CAPA. Finding evidence digests must be ⊆ pin envelopes once pinned.

### 6. Independence is declared, not inferred

`IndependenceRecord` carries auditor, principal, statement, evidence digest(s), `conflictFlags`, and `accepted`. `flag_independence_conflicts` may emit `auditorOwnsControl`. Flags persist and never set `accepted`. Sign-off / conclude require `is_accepted_declaration()` (accepted + non-empty statement + ≥1 evidence ref). There is no separate override-rationale field; an accepted declaration **is** the human override. Absence of flags is not “independent.”

### 7. `Iso27007` stays a selector; catalog facts stay facts

Do not ship `frameworks/iso-27007/`. Do not rewrite governance TOML or ISO remaps. Optional projection of a signed audit to `evidence.governance.internal-audit` (`audited_at`, `auditor_id`, never “audit passed”) is **not landed** in this slice. Hosted auditor UX and Prompt 24 certification packs stay out of scope. Default `supports_audit_program` stays `false`.

### 8. Dual-suite law

Executable law is `tests/contracts/internal_audit.{baseline,target}.rs` registered as `sdd_internal_audit_{baseline,target}` in root `Cargo.toml`. Baseline characterized SHA `6e31bf1` absence and is skip-superseded. Neighbors `sdd_assurance_runtime_target`, `sdd_governance_catalog_target`, `sdd_assessment_lineage_target`, `sdd_compliance_ir_target` stay GREEN.

## Consequences

- Internal audit is queryable, reproducible, and graph-backed.
- Green automated tests cannot impersonate an auditor.
- Compile defaults stay fail-closed; inventories validate when present.
- Prompt 22/24 can consume signed audits and opaque finding/nonconformity refs without this slice owning those workflows.

## Related

- Spec: [`docs/specs/internal-audit.md`](../specs/internal-audit.md)
- Prompt: [`docs/prompts/operational-isms-v1/21-internal-audit.md`](../prompts/operational-isms-v1/21-internal-audit.md)
- Layout: [ADR 0004](0004-documentation-architecture.md)
