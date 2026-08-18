# 02 Contracts

JSON: serde `camelCase`. Every document carries `schemaVersion`.

## Runtime surface

See elite plan module allowlist. Crate root is re-exports only.

## Must not appear on the wire for IR documents

- provider client names
- `iso27001` / `gdpr` / `soc2` fields on `Control`
- `owner` / `implemented` on `Control`
- `Effectiveness` on `Control` or `ControlImplementation`

## Digest

```text
wa:assurance-ir:<schema>:<type>:<canonical-json-bytes>
```

SHA-256 hex. `IrSchemaVersion` ≠ `CanonicalizationVersion`.

## AssessmentDefinition (target)

Contains: requirements, controls, mappings, evidence requirements, tests, implementations, scope, requests. Lives in IR after Phase 5.

## Kept out of IR

`EvidenceEnvelope`, `ControlTestResult`, `Effectiveness`, `CollectorDescriptor`, `FrameworkTarget`, `CompiledFramework`.
