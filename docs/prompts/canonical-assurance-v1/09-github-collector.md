# Grok 4.6 Prompt 09 — Reference-Grade GitHub Assurance Collector

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1
Dependencies: Prompts 01–08, especially canonical evidence contracts

## Mission

Turn the existing GitHub collector into the first reference-grade provider collector for the canonical assurance runtime. The collector must emit canonical evidence only. It must never know ISO/SOC2/NIS2/DORA requirement IDs and must never produce control effectiveness.

## Required provider coverage

Collect, subject to available permissions and API support:

- repository inventory;
- repository visibility/archive state;
- default branch;
- branch protection/rulesets;
- required review count;
- force-push/deletion restrictions;
- required status checks;
- review ownership/CODEOWNERS-related facts where observable;
- secret scanning state;
- code scanning state;
- dependency/security update state;
- workflow default permissions;
- Actions/workflow security configuration where appropriate;
- environment protection and reviewers;
- repository administrators/privileged access;
- outside collaborators;
- deploy keys;
- webhooks/integrations where relevant;
- security policy presence where canonical evidence supports it.

## Collector contract

The collector descriptor must accurately advertise evidence types, permissions, subject types, pagination/incremental capabilities, provider family, and failure behavior.

Permission denial must produce explicit collector/insufficient-evidence diagnostics rather than fabricated negative observations.

Pagination must be complete where required for authoritative populations. Partial pagination must not claim complete population coverage.

## Evidence mapping

Map GitHub-native API objects into the existing canonical evidence contracts from domain prompts. Do not invent `evidence.github.*` unless the central catalog explicitly defines provider-native extension evidence outside canonical assurance; such provider-native extension data must never be required by canonical tests.

Preserve provider-native identifiers in provenance/extensions where useful for traceability.

## Security

Never persist access tokens, authorization headers, cookies, or credential material in evidence facts, diagnostics, or fixtures. Reuse existing redaction guards and add tests around GitHub token patterns.

## Collection runs

Use real collection-run identity and record:

- collector version;
- scope;
- configuration digest;
- start/completion;
- evidence count;
- errors;
- partial/complete status.

## Golden scenarios

Build deterministic adapter fixtures for:

- fully protected healthy organization;
- one unprotected repository;
- missing permission to inspect branch protection;
- paginated repository inventory;
- archived repository excluded by selector;
- disabled security scanning;
- protected environment absent;
- privileged membership population;
- API partial failure;
- rate-limit/retry-safe behavior if the collector currently owns retry logic.

## Non-goals

Do not add ISO-specific logic. Do not calculate `Effective`/`Ineffective`. Do not redesign catalog IDs. Do not create a SaaS credential store.

## Definition of done

GitHub can exercise at least 25–40 canonical controls through canonical evidence, population completeness is explicit, permissions fail safely, no credential material leaks, and another provider could emit the same evidence contracts and receive the same test results.