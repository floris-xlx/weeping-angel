# Moved

The human assurance contract is [`docs/specs/assurance-runtime.md`](../specs/assurance-runtime.md).

ISMS context IR: [`docs/specs/isms-context.md`](../specs/isms-context.md), [`docs/adr/0008-isms-context.md`](../adr/0008-isms-context.md), `tests/contracts/isms_context.{baseline,target}.rs`.

Organizational scope engine: [`docs/specs/scope-engine.md`](../specs/scope-engine.md), [`docs/adr/0008-scope-engine.md`](../adr/0008-scope-engine.md), `tests/contracts/scope_engine.{baseline,target}.rs`. Snapshot schema `weeping-angel/scope-resolution/v1`.

Interested parties / obligations: [`docs/specs/interested-parties-obligations.md`](../specs/interested-parties-obligations.md), [`docs/adr/0008-interested-parties-obligations.md`](../adr/0008-interested-parties-obligations.md), `tests/contracts/interested_parties_obligations.{baseline,target}.rs`.

Security objectives: [`docs/specs/security-objectives.md`](../specs/security-objectives.md), [`docs/adr/0008-security-objectives.md`](../adr/0008-security-objectives.md), `tests/contracts/security_objectives.{baseline,target}.rs`.

Operational risk register: [`docs/specs/risk-register.md`](../specs/risk-register.md), [`docs/adr/0005-operational-risk-register.md`](../adr/0005-operational-risk-register.md), `tests/contracts/risk_register.{baseline,target}.rs`.

Risk identification: [`docs/specs/risk-identification.md`](../specs/risk-identification.md), [`docs/adr/0007-risk-identification-candidate-correlation.md`](../adr/0007-risk-identification-candidate-correlation.md), `tests/contracts/risk_identification.{baseline,target}.rs`.

ISMS events / drift: [`docs/specs/isms-events-drift.md`](../specs/isms-events-drift.md), [`docs/adr/0003-isms-events-drift.md`](../adr/0003-isms-events-drift.md) (sibling notes [`docs/adr/0005-isms-events-drift.md`](../adr/0005-isms-events-drift.md)). Schema `weeping-angel/isms-event/v1`. Library APIs `detect_events` / `detect_isms_drift` in `weeping-angel-assurance::drift` (not crate-root, not `SnapshotDiff`, not a bus). Dual-suite: `tests/contracts/isms_events_drift.{baseline,target}.rs`.

Internal audit: [`docs/specs/internal-audit.md`](../specs/internal-audit.md), [`docs/adr/0003-internal-audit.md`](../adr/0003-internal-audit.md), `tests/contracts/internal_audit.{baseline,target}.rs`.

Controlled documents: [`docs/specs/controlled-documents.md`](../specs/controlled-documents.md), [`docs/adr/0003-controlled-documents.md`](../adr/0003-controlled-documents.md), `tests/contracts/controlled_documents.{baseline,target}.rs`.

Personnel security lifecycle: [`docs/specs/personnel-security.md`](../specs/personnel-security.md), [`docs/adr/0003-personnel-security-lifecycle.md`](../adr/0003-personnel-security-lifecycle.md), `tests/contracts/personnel_security.{baseline,target}.rs`.

Continuity / resilience: [`docs/specs/continuity-resilience.md`](../specs/continuity-resilience.md), [`docs/adr/0005-continuity-resilience.md`](../adr/0005-continuity-resilience.md), `tests/contracts/continuity_resilience.{baseline,target}.rs`.

Executable invariants live in [`tests/contracts/`](../../tests/contracts/).
