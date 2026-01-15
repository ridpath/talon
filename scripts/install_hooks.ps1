# Install Git hooks for TALON development (Windows PowerShell)
# Run this script once after cloning the repository

$ErrorActionPreference = "Stop"

Write-Host "========================================================" -ForegroundColor Blue
Write-Host "      TALON Git Hooks Installation" -ForegroundColor Blue
Write-Host "========================================================" -ForegroundColor Blue
Write-Host ""

# Detect repository root
try {
    $RepoRoot = git rev-parse --show-toplevel 2>$null
    if (-not $RepoRoot) {
        throw "Not in a git repository"
    }
    $RepoRoot = $RepoRoot -replace '/', '\'
} catch {
    Write-Host "ERROR: Not in a git repository" -ForegroundColor Red
    exit 1
}

$HooksDir = Join-Path $RepoRoot ".git\hooks"
$ScriptsDir = Join-Path $RepoRoot "scripts"

# Check if scripts directory exists
if (-not (Test-Path $ScriptsDir)) {
    Write-Host "ERROR: scripts\ directory not found" -ForegroundColor Red
    exit 1
}

# Check if pre-commit script exists
$PreCommitScript = Join-Path $ScriptsDir "pre-commit.ps1"
if (-not (Test-Path $PreCommitScript)) {
    Write-Host "ERROR: scripts\pre-commit.ps1 not found" -ForegroundColor Red
    exit 1
}

Write-Host "[1/4] Checking prerequisites..." -ForegroundColor Yellow

# Check for cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "X cargo not found" -ForegroundColor Red
    Write-Host "  Install Rust toolchain from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}
Write-Host "OK cargo found" -ForegroundColor Green

# Check for git
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "X git not found" -ForegroundColor Red
    exit 1
}
Write-Host "OK git found" -ForegroundColor Green

Write-Host ""
Write-Host "[2/4] Installing Git hooks..." -ForegroundColor Yellow

# Create hooks directory if it doesn't exist
if (-not (Test-Path $HooksDir)) {
    New-Item -ItemType Directory -Path $HooksDir -Force | Out-Null
}

# Install pre-commit hook
$HookFile = Join-Path $HooksDir "pre-commit"

if (Test-Path $HookFile) {
    $BackupFile = "${HookFile}.backup.$((Get-Date).ToString('yyyyMMdd_HHmmss'))"
    Write-Host "-> Backing up existing hook to: $BackupFile" -ForegroundColor Yellow
    Move-Item $HookFile $BackupFile -Force
}

# Create hook wrapper
$HookContent = @"
#!/usr/bin/env pwsh
# TALON pre-commit hook wrapper for Windows

`$ScriptPath = Join-Path (Split-Path `$PSScriptRoot -Parent) "scripts\pre-commit.ps1"

if (Test-Path `$ScriptPath) {
    & powershell -ExecutionPolicy Bypass -File `$ScriptPath
    exit `$LASTEXITCODE
}
else {
    Write-Host "ERROR: pre-commit script not found at `$ScriptPath" -ForegroundColor Red
    exit 1
}
"@

Set-Content -Path $HookFile -Value $HookContent -Encoding UTF8
Write-Host "OK Created hook wrapper: .git\hooks\pre-commit" -ForegroundColor Green

Write-Host ""
Write-Host "[3/4] Optional: Installing pre-commit framework..." -ForegroundColor Yellow

# Check if Python is available
$PythonCmd = Get-Command python -ErrorAction SilentlyContinue
if (-not $PythonCmd) {
    $PythonCmd = Get-Command python3 -ErrorAction SilentlyContinue
}

if ($PythonCmd) {
    Write-Host "OK Python found: $($PythonCmd.Source)" -ForegroundColor Green
    
    try {
        $PreCommitCmd = Get-Command pre-commit -ErrorAction Stop
        Write-Host "OK pre-commit framework already installed" -ForegroundColor Green
        
        Push-Location $RepoRoot
        try {
            pre-commit install 2>&1 | Out-Null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "OK pre-commit framework configured" -ForegroundColor Green
            }
            else {
                Write-Host "WARNING Failed to configure pre-commit framework" -ForegroundColor Yellow
            }
        }
        finally {
            Pop-Location
        }
    }
    catch {
        Write-Host "-> pre-commit framework not installed" -ForegroundColor Yellow
        Write-Host "  To install: pip install pre-commit" -ForegroundColor Yellow
        Write-Host "  Then run: pre-commit install" -ForegroundColor Yellow
    }
}
else {
    Write-Host "WARNING Python not found - skipping pre-commit framework" -ForegroundColor Yellow
    Write-Host "  The PowerShell hook will still work" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[4/4] Installing recommended tools..." -ForegroundColor Yellow

# Check for cargo-deny
if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
    Write-Host "OK cargo-deny installed" -ForegroundColor Green
}
else {
    Write-Host "-> cargo-deny not installed (optional)" -ForegroundColor Yellow
    Write-Host "  To install: cargo install cargo-deny" -ForegroundColor Yellow
}

# Check for cargo-audit
if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    Write-Host "OK cargo-audit installed" -ForegroundColor Green
}
else {
    Write-Host "-> cargo-audit not installed (optional)" -ForegroundColor Yellow
    Write-Host "  To install: cargo install cargo-audit" -ForegroundColor Yellow
}

# Check execution policy
try {
    $ExecutionPolicy = Get-ExecutionPolicy
    if ($ExecutionPolicy -eq "Restricted") {
        Write-Host "WARNING PowerShell execution policy is Restricted" -ForegroundColor Yellow
        Write-Host "  Run as Administrator: Set-ExecutionPolicy RemoteSigned" -ForegroundColor Yellow
    }
}
catch {
    # Ignore errors
}

# Final summary
Write-Host ""
Write-Host "========================================================" -ForegroundColor Blue
Write-Host "  Git hooks installation complete!" -ForegroundColor Green
Write-Host "========================================================" -ForegroundColor Blue
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Test the hook: .\scripts\pre-commit.ps1" -ForegroundColor White
Write-Host "  2. Make a commit to see it in action" -ForegroundColor White
Write-Host "  3. To skip hooks: git commit --no-verify (not recommended)" -ForegroundColor White
Write-Host ""
Write-Host "Recommended optional tools:" -ForegroundColor Yellow
Write-Host "  - pre-commit framework: pip install pre-commit" -ForegroundColor White
Write-Host "  - cargo-deny: cargo install cargo-deny" -ForegroundColor White
Write-Host "  - cargo-audit: cargo install cargo-audit" -ForegroundColor White
Write-Host ""
