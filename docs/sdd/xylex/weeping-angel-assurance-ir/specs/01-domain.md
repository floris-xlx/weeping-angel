# 01 Domain

Ubiquitous language:

| Term | Meaning |
| --- | --- |
| Requirement | Framework-specific obligation (`FrameworkRef` + optional `external_id`) |
| Control | Framework-neutral reusable definition |
| ControlImplementation | Organizational declared state |
| PlannedControlTest | Definition of how to observe a control |
| EvidenceRequirement | What evidence is needed, not how to interpret it |
| Mapping | Explicit directed relation with completeness + provenance |
| AssessmentDefinition | Canonical IR input document |
| Effectiveness | Runtime observation — **not** IR |

Chain (frozen):

```text
Requirement → Mapping → Canonical Control → Planned Control Test → Evidence Requirement
```
