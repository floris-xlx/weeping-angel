# Grok 4.6 Prompt 19 — Incident Governance Engine

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: assets, risks, controls, remediation; Prompt 15

## Mission

Model organizational information-security incident governance without building a SIEM or replacing incident-response tooling.

## Model

Add canonical incident record with stable ID, detection/source reference, classification, severity, status, affected assets/services/data/populations, timeline events, declared time, response owner, containment, eradication/recovery references, communications/notification records where applicable, evidence/artifacts, root cause, lessons learned, linked control failures, linked risks, and corrective actions.

Support external incident-system references. Imported alerts/findings are not automatically incidents; escalation/promotion must be explicit.

Post-incident review should be first-class evidence and may propose risk/control updates and remediations.

## Tests

Cover alert not promoted, declared incident, timeline ordering, control regression linkage, recovery evidence, missing postmortem, incident exercise vs real incident distinction, closed incident with open corrective action, and immutable history.

## Non-goals

No detection rules, log ingestion pipeline, pager system, forensic tooling, or breach-notification legal advice.

## Definition of done

Incidents feed the same risk/control/improvement graph and can be consumed automatically by audit and management-review preparation.