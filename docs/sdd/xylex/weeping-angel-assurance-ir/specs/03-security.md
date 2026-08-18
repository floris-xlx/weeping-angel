# 03 Security

| Rule | Owner |
| --- | --- |
| No credentials in IR documents | `ValidateIr` rejects known credential keys in extensions |
| INV-1 findings stay security-only | ACT-001 ACT-015 |
| Collectors cannot declare compliance | ACT-002 COL-002 |
| Envelope payload credential denylist unchanged | evidence crate |
| Empty ID must not become a confused-deputy subject | IR-001 |
| External requirement id is not internal identity | IR-024 |
