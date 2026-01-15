# Pre-commit hook for TALON development (Windows PowerShell)
# This runs automatically before each commit when installed

$ErrorActionPreference = "Continue"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Write-Header {
    Write-ColorOutput "╔══════════════════════════════════════════════════╗" "Blue"
    Write-ColorOutput "║         TALON Pre-Commit Checks                  ║" "Blue"
    Write-ColorOutput "╚══════════════════════════════════════════════════╝" "Blue"
    Write-Host ""
}

function Write-Status {
    param(
        [bool]$Success,
        [string]$Message
    )
    if ($Success) {
        Write-ColorOutput "✓ $Message" "Green"
    } else {
        Write-ColorOutput "✗ $Message" "Red"
    }
}

function Run-Check {
    param(
        [string]$Name,
        [scriptblock]$Command
    )
    Write-ColorOutput "▶ Running: $Name" "Yellow"
    
    try {
        $output = & $Command 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Status $true "$Name passed"
            return $true
        } else {
            Write-Status $false "$Name failed"
            Write-Host $output
            return $false
        }
    } catch {
        Write-Status $false "$Name failed with exception"
        Write-Host $_.Exception.Message
        return $false
    }
}

Write-Header
$Failed = $false

# 1. Check for Rust toolchain
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-ColorOutput "ERROR: cargo not found. Install Rust toolchain first." "Red"
    exit 1
}

# 2. Format check
Write-Host ""
Write-ColorOutput "[1/7] Checking code formatting..." "Blue"
if (-not (Run-Check "cargo fmt" { cargo fmt --all -- --check })) {
    Write-ColorOutput "→ Run 'cargo fmt' to fix formatting" "Yellow"
    $Failed = $true
}

# 3. Clippy lints
Write-Host ""
Write-ColorOutput "[2/7] Running Clippy lints..." "Blue"
if (-not (Run-Check "cargo clippy" { cargo clippy --all-features --all-targets -- -D warnings })) {
    Write-ColorOutput "→ Fix clippy warnings before committing" "Yellow"
    $Failed = $true
}

# 4. Compilation check
Write-Host ""
Write-ColorOutput "[3/7] Checking compilation..." "Blue"
if (-not (Run-Check "cargo check" { cargo check --all-features })) {
    Write-ColorOutput "→ Fix compilation errors before committing" "Yellow"
    $Failed = $true
}

# 5. Fast unit tests
Write-Host ""
Write-ColorOutput "[4/7] Running fast unit tests..." "Blue"
if (-not (Run-Check "cargo test (fast)" { cargo test --lib --bins --all-features -- --test-threads=4 })) {
    Write-ColorOutput "→ Fix failing tests before committing" "Yellow"
    $Failed = $true
}

# 6. Security check (if cargo-deny is installed)
Write-Host ""
Write-ColorOutput "[5/7] Security audit..." "Blue"
if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
    if (-not (Run-Check "cargo deny" { cargo deny check advisories })) {
        Write-ColorOutput "→ Address security advisories" "Yellow"
        $Failed = $true
    }
} else {
    Write-ColorOutput "⚠ cargo-deny not installed (optional), skipping..." "Yellow"
}

# 7. Check for forbidden patterns
Write-Host ""
Write-ColorOutput "[6/7] Checking for forbidden patterns..." "Blue"

# Get staged files
$StagedFiles = git diff --cached --name-only --diff-filter=ACM

if ($StagedFiles) {
    $PatternFailed = $false
    
    # Check for large files
    foreach ($File in $StagedFiles) {
        if (Test-Path $File) {
            $Size = (Get-Item $File).Length
            if ($Size -gt 1MB) {
                Write-ColorOutput "✗ Large file detected: $File ($Size bytes)" "Red"
                $Failed = $true
                $PatternFailed = $true
            }
        }
    }
    
    # Check for sensitive patterns
    if ($StagedFiles -match '\.(key|pem|crt|p12|pfx)$') {
        Write-ColorOutput "✗ Private key files detected in commit" "Red"
        $Failed = $true
        $PatternFailed = $true
    }
    
    if ($StagedFiles -match '\.(exploit|payload)$') {
        Write-ColorOutput "✗ Exploit artifacts detected in commit" "Red"
        $Failed = $true
        $PatternFailed = $true
    }
    
    # Check for secrets in file content
    foreach ($File in $StagedFiles) {
        if (Test-Path $File) {
            $Content = Get-Content $File -Raw -ErrorAction SilentlyContinue
            if ($Content -match '(api[_-]?key|secret[_-]?key|password\s*=|token\s*=)') {
                Write-ColorOutput "⚠ Potential secret in: $File" "Yellow"
                Write-ColorOutput "  Review carefully before committing" "Yellow"
            }
        }
    }
    
    if (-not $PatternFailed) {
        Write-Status $true "No forbidden patterns detected"
    }
} else {
    Write-ColorOutput "⚠ No staged files to check" "Yellow"
}

# 8. Check for debug statements
Write-Host ""
Write-ColorOutput "[7/7] Checking for debug statements..." "Blue"
$RustFiles = $StagedFiles | Where-Object { $_ -match '\.rs$' }
if ($RustFiles) {
    $DebugFound = $false
    foreach ($File in $RustFiles) {
        if (Test-Path $File) {
            $Content = Get-Content $File
            $DebugLines = $Content | Select-String -Pattern '(println!|dbg!|eprintln!)' | 
                          Where-Object { $_ -notmatch '//' -and $File -notmatch 'tests/' }
            
            if ($DebugLines) {
                if (-not $DebugFound) {
                    Write-ColorOutput "⚠ Debug statements found (review before commit):" "Yellow"
                    $DebugFound = $true
                }
                $DebugLines | Select-Object -First 3 | ForEach-Object { Write-Host $_ }
            }
        }
    }
    
    if (-not $DebugFound) {
        Write-Status $true "No debug statements found"
    }
}

# Final result
Write-Host ""
Write-ColorOutput "╔══════════════════════════════════════════════════╗" "Blue"
if (-not $Failed) {
    Write-ColorOutput "║  ✓ All pre-commit checks passed!                ║" "Green"
    Write-ColorOutput "╚══════════════════════════════════════════════════╝" "Blue"
    exit 0
} else {
    Write-ColorOutput "║  ✗ Pre-commit checks failed                     ║" "Red"
    Write-ColorOutput "╚══════════════════════════════════════════════════╝" "Blue"
    Write-Host ""
    Write-ColorOutput "Fix the issues above or use:" "Yellow"
    Write-ColorOutput "  git commit --no-verify" "Yellow"
    Write-ColorOutput "(not recommended)" "Yellow"
    Write-Host ""
    exit 1
}
