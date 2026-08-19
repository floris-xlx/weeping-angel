# Grok 4.6 Prompt 03 — Interested Parties and Obligations

Repository: `floris-xlx/weeping-angel`
Base: latest `main`
Program: Operational ISMS v1
Dependencies: Prompt 01

## Mission

Create a canonical obligation layer between organizational context and controls. Represent why security requirements exist without encoding every law, contract, or framework directly into controls.

## Required IR

Add provider-neutral concepts for `InterestedParty`, `RequirementSource`, `Obligation`, and `ObligationMapping`. Sources should distinguish contractual, legal/regulatory, customer, internal policy, insurer, supplier, employment, and other categories while remaining extensible.

An obligation must have stable ID, source reference, title/short description, applicability scope, owner, effective/review dates where relevant, and lifecycle state. Do not copy protected normative text from external standards.

Mappings must support obligation -> risk, obligation -> canonical control, obligation -> policy/document, and obligation -> external requirement references with explicit relation and rationale. Partial or supporting mappings must not be silently treated as equivalence.

## Behavior

Provide deterministic validation and explainability. An obligation that no longer applies should be retired/superseded, not deleted from history. Conflicting or overlapping obligations may coexist.

## Tests

Fixtures should include a customer security commitment, employment confidentiality obligation, regulatory retention requirement, and supplier contractual requirement. Cover supersession, expired applicability, dangling mappings, duplicate stable IDs, and partial mapping semantics.

## Non-goals

Do not create a legal advice engine, scrape regulations, implement contract NLP, or hardcode ISO controls. Do not allow collectors to declare obligations satisfied.

## Definition of done

Weeping Angel can answer `why does this control/policy exist?` through a stable obligation graph and can later include obligation changes in management review and risk treatment.