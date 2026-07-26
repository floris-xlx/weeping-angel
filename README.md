# weeping-angel

current version: `0.1.1`
Authorized **web recon + security scanning** CLI (Rust). Discover routes (including SPA/JS surfaces), flag misconfigurations and exposed secrets, map auth, run YAML path templates, compare authenticated vs anonymous access, and optionally fire **gated** active probes.

> **Legal:** Only scan systems you **own** or have **written permission** to test. The tool refuses to run without `--i-own-this` and a host allowlist.

## Quick setup (Windows)

```powershell
.\scripts\setup.ps1
# full lab demo + scan:
.\scripts\demo-scan.ps1
```

## Quick setup (Unix)

```bash
chmod +x scripts/*.sh
./scripts/setup.sh
./scripts/demo-scan.sh
```

## Manual install

```bash
cargo build --release --bins
# scanner: target/release/weeping-angel
# lab:     target/release/weeping-angel-demo   (127.0.0.1 only)
```

## Scan a target you control

```bash
cargo run --bin weeping-angel -- scan https://app.example.com \
  --i-own-this \
  --allow-host app.example.com \
  --profile standard \
  -o report \
  --format terminal,json,sarif,html
```

### Lab demo (local, intentionally weak)

```bash
# terminal 1
cargo run --bin weeping-angel-demo
# listens on http://127.0.0.1:8787

# terminal 2
cargo run --bin weeping-angel -- scan http://127.0.0.1:8787/ \
  --i-own-this --allow-host 127.0.0.1 \
  --profile deep \
  --enable-active \
  --probe xss,sqli,open-redirect,path-traversal \
  --cookie "session=admin-session" \
  --compare-auth \
  --ignore-robots \
  -o report-lab \
  --format terminal,json,html
```

## Modules & profiles

| Profile | Modules |
|---------|---------|
| **recon** | discovery, headers, tls, cookies, secrets, exposures, tech |
| **standard** | recon + cors, auth-surface, wordlist, **templates** |
| **deep** | standard + openapi, auth-compare + all active probes if `--enable-active` |

| Module | Role |
|--------|------|
| `discovery` | HTML crawl, robots/sitemap, JS endpoints, **SPA** (`__NEXT_DATA__`, routers) |
| `wordlist` | Common path probing |
| `templates` | YAML path templates in `templates/` (Nuclei-lite) |
| `headers` / `tls` / `cookies` | Hardening signals |
| `secrets` | Client-visible credential patterns |
| `exposures` | `.env`, `.git`, phpinfo, actuator, dir listing, traces |
| `cors` | Wildcard / reflected Origin |
| `auth-surface` | Login forms, admin paths, session cookies |
| `auth-compare` | Anon vs `--cookie` / `Authorization` (enable with `--compare-auth`) |
| `tech` | Light fingerprinting |
| Active: `xss`, `sqli`, `open-redirect`, `path-traversal` | Opt-in via `--enable-active` |

## Templates

Drop YAML files under `templates/`:

```yaml
id: my-check
name: Example
severity: high
paths:
  - /.env
matchers:
  - type: status
    status: [200]
  - type: body
    regex:
      - "(?i)secret\\s*="
```

```bash
cargo run --bin weeping-angel -- scan http://127.0.0.1:8787/ \
  --i-own-this --allow-host 127.0.0.1 \
  --templates-dir templates
```

## Safety defaults

- Requires `--i-own-this` **and** `--allow-host` (supports `*.example.com`)
- Rate limit (`--rps`) + concurrency caps
- Response body cap 2 MiB; redirects re-validated against allowlist
- No POST/PUT/PATCH/DELETE unless `--allow-write-methods`
- Active modules never run without `--enable-active`
- robots.txt honored unless `--ignore-robots`
- Demo binds **127.0.0.1 only**

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | No findings ≥ `--fail-on` (default `medium`) |
| 1 | Findings at or above threshold |
| 2 | Tool / usage / consent error |

## Config file

Copy `weeping-angel.example.toml` → `weeping-angel.toml`:

```toml
[authorization]
i_own_this = true
allow_hosts = ["127.0.0.1", "localhost", "app.example.com"]

[scan]
profile = "standard"
fail_on = "medium"
```

## Development

```bash
cargo test
cargo run --bin weeping-angel -- --help
```

## Disclaimer

For defensive testing of **authorized** targets only. Misuse may be illegal.
