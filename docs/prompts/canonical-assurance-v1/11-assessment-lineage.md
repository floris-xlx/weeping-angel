# Grok 4.6 Prompt 11 — Immutable Assessment Lineage, Explainability, and Report Cleanup

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1
Dependencies: Prompts 01–10

## Mission

Complete the assurance runtime's immutable execution lineage and remove remaining MVP-style report/orchestrator shortcuts. Every assessment result must be reproducible and explainable down to framework snapshot, catalog snapshot, applicability decision, collector run, evidence digest, test version, affected subject, exception, and mapping.

This prompt owns persistence/orchestration lineage, explanation projections, generic report cleanup, and framework-generic facade fixes. It does not own domain catalog content.

## Required immutable chain

Persist or otherwise durably represent:

```text
FrameworkPackSnapshot
CanonicalCatalogSnapshot
AssessmentDefinitionSnapshot
ApplicabilitySnapshot
CollectionRun[]
EvidenceEnvelope[]
EvidenceSnapshot
ControlTestRun[]
AssessmentRun
FrameworkReadinessSnapshot
StatementOfApplicabilitySnapshot
```

An assessment must pin at least:

```text
framework pack digest
canonical catalog digest
assessment definition digest
collector IDs and versions
collection run IDs
evidence digests
test IDs and versions
applicability decisions
result digest
```

Historical evidence is append-only. Replay must never depend on mutable current catalog/framework files without detecting digest mismatch.

## Assessment run model

Make `AssessmentRun` a real execution record rather than an unused/transient object. Record start/completion, scope, status, collector runs, evidence snapshot identity, framework/catalog digests, and result identity.

Ensure failed/partial collection can be represented without rewriting history.

## Explainability

Implement a generic explanation projection conceptually similar to:

```rust
ControlExplanation {
    control,
    applicability,
    implementation,
    population,
    tests,
    evidence_requirements,
    evidence,
    missing_evidence,
    failing_subjects,
    missing_subjects,
    exceptions,
    mappings,
    effectiveness,
}
```

Exact shape should fit the repository.

Add CLI support equivalent to:

```bash
weeping-angel assurance explain \
  --assessment <id> \
  --control control.identity.privileged-mfa
```

The output must answer why the control was evaluated, what population was evaluated, what evidence was used or missing, which subjects failed, which expression/test version ran, which exceptions influenced the result, and which framework requirements map to the control.

## Report cleanup

Remove framework-specific behavior from generic serialization/orchestration. In particular, eliminate patterns equivalent to hardcoded:

```rust
load_framework_pack("iso-27001", "2022")
```

inside generic `AssessmentReport` serialization.

Serialization must be pure: no pack loading, network I/O, filesystem lookup, or hidden current-state resolution while serializing an already-produced assessment.

Replace computed-on-serialize summary behavior with explicit projection structures such as:

```text
AssessmentSummary
FrameworkReadinessSnapshot
CoverageMetrics
```

Expose separate metrics for:

```text
control effectiveness coverage
evidence coverage
automation coverage
subject coverage
framework requirement coverage
fresh-evidence coverage
manual-review burden
```

Do not collapse them into one misleading compliance percentage.

## Framework-generic facade

Resolve every framework through one registry/loader path. Remove ISO-only fallback branches and hidden stub assessments from normal production execution. Fixture/stub paths may remain explicitly test-only.

## Snapshot comparison

Ensure comparison can identify meaningful change across assessment runs:

- applicability changes;
- subject population changes;
- evidence additions/removals/supersession;
- test result changes;
- exceptions introduced/expired;
- framework/catalog digest changes.

## Tests

Add replay and explainability tests proving:

1. historical assessment can be reconstructed from pinned snapshots;
2. current catalog changes do not silently rewrite old results;
3. result explanation references exact evidence digests;
4. serialization performs no framework resolution;
5. partial collector runs remain distinguishable;
6. assessment diff identifies changed subjects/results;
7. exceptions are visible in lineage;
8. deterministic snapshot/result digests.

## Non-goals

Do not build a multi-tenant SaaS backend. Do not build UI. Do not add new frameworks. Do not redesign domain catalogs.

## Definition of done

An assessment is a reproducible immutable execution artifact rather than only a current-state report; every result is explainable; report serialization is pure; framework resolution is generic; and snapshot replay/diff is deterministic.