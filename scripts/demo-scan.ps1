# Start lab demo (background) and run a deep authorized scan against it.
$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

$Port = if ($env:PORT) { $env:PORT } else { "8787" }
$Base = "http://127.0.0.1:$Port/"

Write-Host "==> Building" -ForegroundColor Cyan
cargo build --bins
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Kill anything already on the port (best effort)
Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue |
    ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }

Write-Host "==> Starting demo on $Base" -ForegroundColor Cyan
$env:PORT = $Port
# Use Start-Job so cargo inherits env and we can tear down cleanly
$job = Start-Job -ScriptBlock {
    param($root, $port)
    Set-Location $root
    $env:PORT = "$port"
    cargo run --quiet --bin weeping-angel-demo 2>&1
} -ArgumentList (Get-Location).Path, $Port

try {
    $ok = $false
    for ($i = 0; $i -lt 40; $i++) {
        try {
            Invoke-WebRequest -Uri $Base -UseBasicParsing -TimeoutSec 1 | Out-Null
            $ok = $true
            break
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }
    if (-not $ok) {
        Write-Host "Demo did not become ready on $Base" -ForegroundColor Red
        Write-Host (Receive-Job $job | Out-String)
        exit 1
    }

    $out = "report-lab"
    Write-Host "==> Scanning lab (deep + active + auth compare)" -ForegroundColor Cyan
    cargo run --quiet --bin weeping-angel -- scan $Base `
        --i-own-this `
        --allow-host 127.0.0.1 `
        --profile deep `
        --enable-active `
        --probe xss,sqli,open-redirect,path-traversal `
        --cookie "session=admin-session" `
        --compare-auth `
        --ignore-robots `
        --rps 20 `
        --depth 2 `
        --max-urls 120 `
        --fail-on high `
        -o $out `
        --format terminal,json,html,sarif

    $code = $LASTEXITCODE
    Write-Host ""
    Write-Host "Reports: ${out}.json / ${out}.html / ${out}.sarif.json (if written)" -ForegroundColor Green
    Write-Host "Exit code: $code"
    exit $code
}
finally {
    Stop-Job $job -ErrorAction SilentlyContinue
    Remove-Job $job -Force -ErrorAction SilentlyContinue
    Get-Process -Name "weeping-angel-demo" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}
