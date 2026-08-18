# Grok 4.6 Prompt 12 — ISO 27001:2022 Remapping onto the Canonical Catalog

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1
Dependencies: Prompts 01–11; start only after canonical control/evidence/test IDs are stable

## Mission

Refactor the existing ISO 27001:2022 assurance vertical so its framework pack maps onto the completed canonical assurance catalog rather than relying on thin or ISO-adjacent canonical controls. Preserve the existing structural-only legal/content boundary, framework compiler architecture, evidence ledger, readiness/SoA behavior, and non-certification language.

This prompt owns ISO 27001 framework content, mappings, applicability references, and projection integration. It must not redesign canonical controls or provider collectors.

## Architectural target

The desired chain is:

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
ISO requirement -> GitHub check
ISO requirement -> AWS API
ISO requirement -> scanner engine
```

## Framework pack

Work within the existing versioned structural pack:

```text
frameworks/iso-27001/2022/
  manifest.toml
  requirements.toml
  mappings.toml
  applicability.toml
  metadata.toml
```

Do not redistribute protected ISO/IEC normative wording. Keep identifiers, legally safe short titles, structural hierarchy, automation classification, applicability metadata, and mappings only. Preserve the ability to layer licensed/user-supplied narrative through the framework content provider abstraction.

## Mapping work

Remap ISO requirements to stable canonical control IDs from Prompts 04–08. Use the existing rich mapping model honestly:

```text
Equivalent
Satisfies
PartiallySatisfies
Supports
EvidenceFor
SupersetOf
SubsetOf
Related
```

Do not use `Equivalent` as a convenience. Equivalence must be explicit, defensible, and directionally correct. Partial mappings must not be allowed to fully satisfy a framework requirement.

Every material mapping should include rationale and provenance. Apply version constraints where relevant.

## Coverage

Map the structural ISO 27001:2022 requirement set as comprehensively as the canonical v1 catalog permits. For requirements fundamentally dependent on organizational judgement, documentation, governance, or auditor review, map to governance/manual canonical controls and retain manual/hybrid classification rather than inventing automated technical tests.

The objective is not to claim certification automation. The output remains readiness/assurance only.

## Applicability and SoA

Integrate the generic applicability engine. Annex A/SoA-oriented output must preserve:

- applicable vs not applicable vs unresolved/manual determination;
- rationale;
- mapped canonical controls;
- implementation/evidence state;
- control-test effectiveness;
- exceptions;
- missing evidence;
- manual-review requirements.

A not-applicable decision must be justified by context, not merely by missing evidence.

## Remove legacy shortcuts

Delete or migrate temporary ISO-specific canonical stub controls/tests/evidence requirements that are superseded by the canonical catalog. Do not leave two competing control IDs representing the same semantic control.

Ensure normal assurance execution resolves ISO through the same generic framework registry/loader path used by every framework. No hardcoded ISO pack resolution may remain in generic report serialization or test runtime.

## Readiness projection

Preserve explicit non-certification language. Never emit:

```text
ISO 27001 certified
ISO 27001 compliant
certification guaranteed
audit passed
```

Allowed semantics include:

```text
ready
effective
ineffective
partially effective
insufficient evidence
stale evidence
manual review required
not applicable
assessment coverage
```

Expose separate automation, evidence, subject, control, and framework-requirement coverage metrics rather than one compliance percentage.

## Golden scenarios

Create/refresh ISO end-to-end scenarios for:

1. technically strong organization with governance evidence present;
2. strong technical controls but missing manual governance evidence;
3. partial repository population coverage;
4. privileged MFA failure mapped through canonical IAM controls;
5. stale evidence;
6. approved exception;
7. applicability-driven not-applicable control;
8. incomplete organization context requiring manual applicability determination;
9. historical snapshot replay after framework/catalog files change;
10. empty scanner findings with unknown coverage, proving no false positive effectiveness.

## Architecture tests

Add or update tests asserting:

- framework pack contains no provider-specific types;
- collectors contain no ISO requirement IDs;
- control-test runtime contains no ISO branches;
- partial mappings cannot become equivalence;
- mappings reference existing canonical IDs;
- every ISO readiness result traces to canonical controls and evidence;
- SoA output uses generic applicability results;
- pack digest and catalog digest are both pinned in assessment lineage.

## Non-goals

Do not add SOC 2/NIS2/DORA/PCI/HIPAA packs in this prompt. Do not change canonical IDs simply to make ISO mapping easier. Do not add provider APIs. Do not claim auditor/certification equivalence.

## Final verification

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --features demo
weeping-angel assurance catalog validate
weeping-angel assurance framework validate frameworks/iso-27001/2022
```

Run the ISO SDD target suite and all architecture-boundary tests.

## Definition of done

ISO 27001:2022 is a clean framework projection over the canonical assurance catalog: framework content is data, mappings are explicit and defensible, controls/tests/evidence remain framework-neutral, applicability is contextual and explainable, historical lineage pins both framework and catalog digests, and the system produces readiness/SoA output without certification claims.