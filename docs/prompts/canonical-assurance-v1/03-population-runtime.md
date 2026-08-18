# Grok 4.6 Prompt 03 — Subject Population Runtime and Coverage Semantics

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1

## Mission

Implement first-class subject populations and real coverage semantics so assurance tests can evaluate an entire in-scope population rather than merely ask whether one evidence envelope exists.

This prompt owns population resolution, subject-aware evaluation, coverage metrics, missing-subject semantics, and the completion of `CoverageAtLeast`. It should consume the canonical catalog and typed-evidence contracts rather than redefining them.

## Architectural law

Absence of evidence must never become positive evidence unless the runtime knows the authoritative population and can prove the observation covers it.

Example:

```text
50 in-scope repositories
47 branch-protection observations passing
2 observations failing
1 repository missing evidence
```

The runtime must distinguish all four numbers. It must not report `94% passing` while hiding the missing subject.

## First actions

Inspect `SubjectSelector`, `Asset`, `Identity`, `AssessmentScope`, evidence provenance, `EvidenceSet`, `TestExpr`, `CoverageAtLeast`, and existing applicability/compiler behavior. Rebase on Prompt 01/02 outputs if available.

## Required subject model

Support populations for at least these conceptual subject kinds where current IR permits:

```text
organization
repository
branch
application
service
database
cloud account
cloud resource
identity
privileged identity
service account
endpoint
vendor
data store
processing activity
network
deployment
```

Do not create a second competing subject selector type if the IR already owns one. Extend narrowly only where required.

## Population resolution

Create a deterministic runtime abstraction conceptually similar to:

```rust
Population {
    selector,
    subject_ids,
    authoritative,
    observed_at,
}
```

Exact shape may differ.

The evaluator must know when a population is authoritative versus partial/unknown. Unknown population completeness must prevent strong all-subject conclusions.

## Test expression semantics

Implement/complete:

```text
Count
CountWhere
AllSubjects
AnySubject
NoneSubjects
CoverageAtLeast
CoverageExactly
MissingSubjects
```

Use existing `TestExpr` style where possible rather than creating a parallel rule engine.

`CoverageAtLeast` must calculate actual coverage rather than returning a placeholder `PartiallyEffective` result.

## Evaluation output

Control-test results or their explanation metadata must expose population information sufficient to produce:

```json
{
  "population": 50,
  "evaluated": 49,
  "passing": 47,
  "failing": 2,
  "missing": 1,
  "coverage": 0.98,
  "failingSubjects": ["repo:a", "repo:b"],
  "missingSubjects": ["repo:c"]
}
```

Do not necessarily force this exact JSON shape into the core result if a dedicated evaluation detail object is cleaner.

## Effectiveness rules

Define deterministic rules for:

- full population passes;
- threshold passes despite some failures where catalog policy allows it;
- population threshold fails;
- evidence missing for one or more known subjects;
- population itself unknown/incomplete;
- stale evidence for part of population;
- approved exceptions for selected subjects.

Missing evidence and technical failure must remain distinct.

## Performance

Avoid O(subjects × all_evidence) scans. Build indexes by evidence type and subject, or adapt the ledger/query layer appropriately.

Add benchmarks or test fixtures for:

```text
100 subjects
1,000 subjects
10,000 subjects
100,000 evidence envelopes
```

## Tests

Add golden tests for:

1. 50/50 passing;
2. 47/50 passing with 3 explicit failures;
3. 47 passing, 2 failing, 1 missing evidence;
4. unknown/incomplete population;
5. stale evidence on subset;
6. exceptions on subset;
7. zero population;
8. duplicated evidence envelopes;
9. latest/superseding evidence selection;
10. deterministic subject ordering.

Zero population must not accidentally produce `Effective` without explicit applicability semantics.

## Non-goals

Do not build provider discovery here. Do not add ISO-specific coverage rules. Do not build the organization graph beyond what is required to resolve subject populations. Do not redesign catalog schema.

## Handoff contract

Domain catalog prompts must be able to declare tests such as:

```text
all privileged identities have MFA
100% of non-archived repositories protect default branch
no critical vulnerability exceeds SLA
at least 95% of endpoints report encryption enabled
```

without provider-specific logic.

## Definition of done

Population-aware test evaluation is real, `CoverageAtLeast` is no longer a placeholder, missing/failing/stale subjects are separately represented, evaluation is efficient enough for realistic inventories, and the architecture remains provider/framework neutral.