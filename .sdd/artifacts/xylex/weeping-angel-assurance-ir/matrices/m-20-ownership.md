# M-20 Ownership

| Concern | Canonical owner | Must not own |
| --- | --- | --- |
| IR entity writes | `weeping-angel-assurance-ir` | framework, facade, scanner |
| AssessmentDefinition writes | IR | framework |
| AssessmentScope (semantic) writes | IR | facade |
| compile_framework writes | `weeping-angel-framework` | IR |
| EvidenceEnvelope writes | `weeping-angel-evidence` | IR |
| collect writes | `weeping-angel-collector` | IR, framework |
| evaluate writes | `weeping-angel-control-test` | IR |
| SemanticFinding writes | root `src/` | IR |
