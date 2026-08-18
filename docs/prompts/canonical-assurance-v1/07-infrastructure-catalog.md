# Grok 4.6 Prompt 07 — Infrastructure, Data, Logging, Crypto, and Resilience Catalog

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Canonical Assurance Catalog v1
Dependencies: Prompts 01–03

## Mission

Implement canonical assurance content for infrastructure security: network controls, cryptography, data protection, databases, logging/monitoring, backup, recovery, and resilience. This prompt owns provider-neutral catalog content only.

## Required control families

Build approximately 35–50 meaningful controls across these areas.

### Network security

- administrative interfaces restricted;
- public exposure governed;
- network segmentation where applicable;
- firewall policy present/current;
- databases not unnecessarily public;
- management access protected;
- TLS enforced for sensitive traffic;
- insecure protocols restricted.

### Cryptography and secrets

- encryption at rest;
- encryption in transit;
- managed key lifecycle;
- secret storage;
- credential storage;
- key/secret rotation where required;
- certificate validity/rotation;
- sensitive backup encryption.

### Data protection/database

- production data stores inventoried;
- access restricted;
- encryption enabled;
- backup enabled;
- retention policy represented;
- sensitive data subject classification where available;
- database auditing enabled where applicable.

### Logging and monitoring

- audit logging enabled;
- administrative event logging;
- authentication/security event logging;
- retention meets policy;
- time synchronization/consistent timestamps;
- security alerting configured;
- privileged actions observable;
- log integrity/protected storage;
- monitoring coverage across critical assets.

### Resilience

- backup enabled;
- backup population coverage;
- backup retention;
- restore testing;
- recovery procedure evidence;
- disaster-recovery exercise;
- redundancy/high availability where required;
- recovery objectives documented;
- recovery evidence freshness.

## Canonical evidence

Use provider-neutral evidence contracts such as:

```text
evidence.network.exposure
evidence.network.firewall-policy
evidence.network.tls-configuration
evidence.data.encryption-at-rest
evidence.data.encryption-in-transit
evidence.crypto.key-state
evidence.secret.storage-configuration
evidence.database.inventory
evidence.database.access-configuration
evidence.logging.configuration
evidence.logging.retention
evidence.logging.alerting
evidence.backup.configuration
evidence.backup.run
evidence.backup.restore-test
evidence.resilience.recovery-plan
```

Do not create `evidence.aws.cloudtrail`, `evidence.cloudflare.tls`, or equivalent provider-specific canonical contracts.

## Tests

Implement reusable tests including population-aware variants:

- all critical databases encrypt data at rest;
- all public-facing endpoints enforce acceptable TLS policy;
- all critical assets have current audit logging evidence;
- logging retention meets configured threshold;
- all required data stores have current backups;
- restore tests are within freshness window;
- no prohibited public database exposure;
- required secrets use approved storage evidence.

Thresholds should come from catalog/test configuration or assessment policy rather than hardcoded framework assumptions.

## Hybrid/manual boundaries

Disaster recovery exercises, recovery objectives, network segmentation rationale, and similar controls may be hybrid/manual. Represent this honestly.

## Fixtures

Create realistic multi-resource fixtures with healthy, partial, stale, missing, failing, and exception-approved cases.

## Non-goals

Do not implement AWS/Azure/GCP/Cloudflare collectors. Do not map ISO. Do not build a remote inventory service. Do not encode provider-specific resource names in canonical IDs.

## Definition of done

The infrastructure catalog provides broad, reusable assurance semantics that future cloud/database/network collectors can populate without framework knowledge, and all content passes catalog validation and workspace verification.