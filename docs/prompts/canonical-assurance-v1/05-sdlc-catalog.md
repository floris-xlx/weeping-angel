# Grok 4.6 Prompt 05 — SDLC and Source-Control Canonical Assurance Catalog

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1
Dependencies: Prompts 01–03

## Mission

Implement the canonical source-control, change-management, CI/CD, secure-development, release-integrity, and software-supply-chain control catalog. This prompt owns domain content only and must remain provider-neutral.

## Required control families

Target roughly 20–30 useful controls across:

- repository inventory and ownership;
- repository visibility governance;
- protected default branch;
- force-push restriction;
- branch deletion restriction;
- required pull-request review;
- minimum reviewer count;
- review ownership/CODEOWNERS-style ownership;
- required status checks;
- administrator bypass governance;
- signed commits/artifacts where policy requires;
- secret scanning;
- code scanning/SAST;
- dependency vulnerability scanning;
- dependency update monitoring;
- dependency pinning/lockfile integrity where applicable;
- CI workflow permission minimization;
- protected deployment environments;
- release authorization;
- separation of development/release authority where applicable;
- build provenance;
- artifact integrity;
- change traceability;
- security review for material changes;
- secure-development policy evidence;
- unsupported/deprecated component handling.

## Stable IDs

Prefer semantics such as:

```text
control.source.default-branch-protection
control.source.required-review
control.source.force-push-restricted
control.source.secret-scanning
control.cicd.workflow-permissions
control.release.protected-environment
control.supply-chain.dependency-integrity
```

Never use `github`, `gitlab`, `azure-devops`, ISO, SOC 2, or similar provider/framework names in canonical IDs.

## Evidence contracts

Define/reuse canonical evidence types such as:

```text
evidence.repository.inventory
evidence.repository.visibility
evidence.repository.default-branch
evidence.repository.branch-protection
evidence.repository.review-policy
evidence.repository.review-ownership
evidence.repository.security-scanning
evidence.repository.dependency-scanning
evidence.cicd.workflow-permissions
evidence.cicd.status-checks
evidence.deployment.environment-protection
evidence.release.authorization
evidence.supply-chain.build-provenance
evidence.supply-chain.lockfile-state
```

Do not encode GitHub-native object names when a provider-neutral semantic can express the same observation.

## Tests

Use population-aware tests. Required examples include:

- all non-archived in-scope repositories have protected default branches;
- all protected branches prohibit unauthorized force-push;
- all production repositories require review;
- required review count meets policy threshold;
- security scanning is enabled where applicable;
- CI workflows do not have overbroad write permissions;
- production deployment environments require authorization;
- critical repositories have current dependency scans;
- release artifacts have integrity/provenance evidence where required.

Missing evidence must not be converted into technical failure.

## Scanner relationship

The existing scanner may produce canonical evidence for source/dependency/security findings. Do not couple catalog tests to scanner internals. Depend only on evidence contracts.

## Fixtures

Include healthy and degraded multi-repository populations, including partial coverage, one unprotected repository, one repository missing scan evidence, stale evidence, and an approved exception.

## Non-goals

Do not expand the GitHub collector here. Do not implement ISO mappings. Do not change generic population semantics. Do not create a GitHub-specific canonical catalog.

## Definition of done

The SDLC catalog is substantial enough that a GitHub, GitLab, or Bitbucket collector could independently populate the same evidence contracts and receive the same control results. Catalog validation, deterministic tests, and full workspace checks must pass.