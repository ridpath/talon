# Security audit script for TALON (Windows PowerShell)
# Runs comprehensive security checks using cargo-audit and cargo-deny

$ErrorActionPreference = "Continue"

Write-Host "==================================" -ForegroundColor Blue
Write-Host "TALON Security Audit" -ForegroundColor Blue
Write-Host "==================================" -ForegroundColor Blue
Write-Host ""

# Check if cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: cargo is not installed" -ForegroundColor Red
    Write-Host "Please install Rust: https://rustup.rs/"
    exit 1
}

# Install cargo-audit if not present
if (-not (Get-Command cargo-audit -ErrorAction SilentlyContinue)) {
    Write-Host "Installing cargo-audit..." -ForegroundColor Yellow
    cargo install cargo-audit
}

# Install cargo-deny if not present
if (-not (Get-Command cargo-deny -ErrorAction SilentlyContinue)) {
    Write-Host "Installing cargo-deny..." -ForegroundColor Yellow
    cargo install cargo-deny
}

Write-Host ""
Write-Host "==================================" -ForegroundColor Blue
Write-Host "1. Cargo Audit - Vulnerability Scan" -ForegroundColor Blue
Write-Host "==================================" -ForegroundColor Blue
Write-Host ""

# Run cargo audit
$auditResult = cargo audit --deny warnings
if ($LASTEXITCODE -eq 0) {
    Write-Host "No known vulnerabilities found" -ForegroundColor Green
    $auditPass = $true
} else {
    Write-Host "Security vulnerabilities detected" -ForegroundColor Red
    $auditPass = $false
}

Write-Host ""
Write-Host "==================================" -ForegroundColor Blue
Write-Host "2. Cargo Deny - License & Supply Chain" -ForegroundColor Blue
Write-Host "==================================" -ForegroundColor Blue
Write-Host ""

$denyPass = $true

Write-Host "Checking advisories..." -ForegroundColor Cyan
$advisoriesResult = cargo deny check advisories
if ($LASTEXITCODE -eq 0) {
    Write-Host "No advisory issues" -ForegroundColor Green
} else {
    Write-Host "Advisory issues detected" -ForegroundColor Red
    $denyPass = $false
}

Write-Host ""
Write-Host "Checking licenses..." -ForegroundColor Cyan
$licensesResult = cargo deny check licenses
if ($LASTEXITCODE -eq 0) {
    Write-Host "All licenses approved" -ForegroundColor Green
} else {
    Write-Host "License issues detected" -ForegroundColor Red
    $denyPass = $false
}

Write-Host ""
Write-Host "Checking bans..." -ForegroundColor Cyan
$bansResult = cargo deny check bans
if ($LASTEXITCODE -eq 0) {
    Write-Host "No banned dependencies" -ForegroundColor Green
} else {
    Write-Host "Banned dependencies detected" -ForegroundColor Red
    $denyPass = $false
}

Write-Host ""
Write-Host "Checking sources..." -ForegroundColor Cyan
$sourcesResult = cargo deny check sources
if ($LASTEXITCODE -eq 0) {
    Write-Host "All sources approved" -ForegroundColor Green
} else {
    Write-Host "Source issues detected" -ForegroundColor Red
    $denyPass = $false
}

Write-Host ""
Write-Host "==================================" -ForegroundColor Blue
Write-Host "3. Dependency Tree Analysis" -ForegroundColor Blue
Write-Host "==================================" -ForegroundColor Blue
Write-Host ""

Write-Host "Critical security dependencies:" -ForegroundColor Cyan
try {
    cargo tree -p openssl -p rustls -p ring -p webpki 2>$null
} catch {
    Write-Host "No TLS dependencies found"
}

Write-Host ""
Write-Host "==================================" -ForegroundColor Blue
Write-Host "4. Outdated Dependencies Check" -ForegroundColor Blue
Write-Host "==================================" -ForegroundColor Blue
Write-Host ""

if (Get-Command cargo-outdated -ErrorAction SilentlyContinue) {
    cargo outdated --root-deps-only
} else {
    Write-Host "Skipping (cargo-outdated not installed)" -ForegroundColor Yellow
    Write-Host "Install with: cargo install cargo-outdated"
}

Write-Host ""
Write-Host "==================================" -ForegroundColor Blue
Write-Host "Security Audit Summary" -ForegroundColor Blue
Write-Host "==================================" -ForegroundColor Blue
Write-Host ""

# Summary
if ($auditPass -and $denyPass) {
    Write-Host "All security checks passed" -ForegroundColor Green
    Write-Host ""
    Write-Host "Your project has no known vulnerabilities and complies with"
    Write-Host "security policies defined in deny.toml"
    exit 0
} else {
    Write-Host "Security issues detected" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please review the errors above and:"
    Write-Host "1. Update vulnerable dependencies"
    Write-Host "2. Review and approve licenses if acceptable"
    Write-Host "3. Remove or replace banned dependencies"
    Write-Host "4. Verify source registry configurations"
    exit 1
}
