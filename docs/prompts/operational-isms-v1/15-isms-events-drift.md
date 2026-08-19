# Grok 4.6 Prompt 15 — ISMS Event and Drift Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompts 13, 14 and operational risk/control models

## Mission

Create a canonical event stream for meaningful management-system state changes and deterministic drift detection between immutable snapshots.

## Events

Support events such as `ControlRegressed`, `ControlRecovered`, `EvidenceExpired`, `EvidenceRevoked`, `RiskIncreased`, `RiskDecreased`, `RiskAccepted`, `ExceptionExpired`, `NewAssetDetected`, `AssetRemoved`, `VendorRiskChanged`, `ObjectiveMissed`, `PolicyExpired`, `AuditFindingOpened`, `NonconformityOpened`, `CorrectiveActionOverdue`, and equivalent extensible variants.

Events must include stable ID, time, source snapshot(s), affected subjects, cause references, severity/classification where applicable, and deterministic payload. Events are observations of state transition, not mutable workflow tickets.

## Drift

Implement snapshot diff for scope, assets, risk, controls, implementations, evidence, tests, SoA, objectives, exceptions, and governance records. Prevent noisy events when serialization ordering changes but semantics do not.

## Tests

Cover no-op snapshots, one control regression, evidence expiry, risk increase caused by a control regression, new asset in scope, expired exception, and event deduplication on repeated diff.

## Non-goals

No notification transport, Slack integration, or generic event bus infrastructure.

## Definition of done

Operational changes are first-class, explainable events that later remediation, management review and reporting can consume.