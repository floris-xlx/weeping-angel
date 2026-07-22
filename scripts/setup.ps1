# One-shot local setup for weeping-angel
$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

Write-Host "==> Building weeping-angel (+ demo)" -ForegroundColor Cyan
cargo build --bins
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Running tests" -ForegroundColor Cyan
cargo test
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not (Test-Path "weeping-angel.toml")) {
    Copy-Item "weeping-angel.example.toml" "weeping-angel.toml"
    Write-Host "==> Created weeping-angel.toml from example" -ForegroundColor Green
}

Write-Host ""
Write-Host "Setup complete." -ForegroundColor Green
Write-Host "  Scanner:  cargo run --bin weeping-angel -- --help"
Write-Host "  Lab demo: cargo run --bin weeping-angel-demo"
Write-Host "  Full lab: .\scripts\demo-scan.ps1"
