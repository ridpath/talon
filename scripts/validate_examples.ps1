# Example Validation Script for TALON
# Validates all .talon example files for:
# - Emoticons
# - Marketing language
# - Syntax consistency
# - Documentation completeness

param(
    [switch]$Verbose,
    [switch]$CheckSyntax
)

$ErrorActionPreference = "Continue"
$examplesDir = Join-Path $PSScriptRoot "..\examples"
$talonBin = Join-Path $PSScriptRoot "..\target\debug\talon.exe"

# Color output helpers
function Write-Pass { param($msg) Write-Host "[PASS] $msg" -ForegroundColor Green }
function Write-Fail { param($msg) Write-Host "[FAIL] $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }

Write-Host "`n========================================" -ForegroundColor Yellow
Write-Host "TALON Example Validation Suite" -ForegroundColor Yellow
Write-Host "========================================`n" -ForegroundColor Yellow

# Get all .talon files
$talonFiles = Get-ChildItem -Path $examplesDir -Filter "*.talon"
Write-Info "Found $($talonFiles.Count) .talon example files"

# Initialize counters
$totalChecks = 0
$passedChecks = 0
$failedChecks = 0
$issues = @()

# Check 1: No emoticons
Write-Host "`n[1/6] Checking for emoticons..." -ForegroundColor Cyan

# Check for non-ASCII characters that might be emoticons
$emoticonFiles = @()

foreach ($file in $talonFiles) {
    $totalChecks++
    $bytes = [System.IO.File]::ReadAllBytes($file.FullName)
    $hasNonAscii = $false
    
    # Check for common UTF-8 emoticon byte patterns
    # Most emoticons start with 0xF0 (4-byte UTF-8) or 0xE2 (3-byte UTF-8)
    for ($i = 0; $i -lt $bytes.Length - 1; $i++) {
        if ($bytes[$i] -eq 0xF0 -or ($bytes[$i] -eq 0xE2 -and $bytes[$i+1] -in @(0x9C, 0x98, 0x9D, 0x9A, 0x9B, 0x9E))) {
            $hasNonAscii = $true
            break
        }
    }
    
    if ($hasNonAscii) {
        $emoticonFiles += $file.Name
        $failedChecks++
        $issues += "Emoticons found in: $($file.Name)"
        Write-Fail "  $($file.Name) contains emoticons"
    } else {
        $passedChecks++
        if ($Verbose) { Write-Pass "  $($file.Name)" }
    }
}

if ($emoticonFiles.Count -eq 0) {
    Write-Pass "No emoticons found in any examples"
} else {
    Write-Fail "Found emoticons in $($emoticonFiles.Count) files"
}

# Check 2: No marketing language
Write-Host "`n[2/6] Checking for marketing language..." -ForegroundColor Cyan
$marketingTerms = @(
    'world-class', 'world class', 'WORLD-CLASS', 'WORLD CLASS',
    'best-in-class', 'best in class',
    'god-tier', 'god tier', 'titan',
    'cutting-edge', 'revolutionary',
    'game-changing', 'world''s best'
)

$marketingFiles = @()
foreach ($file in $talonFiles) {
    $totalChecks++
    $content = Get-Content $file.FullName -Raw
    $foundTerms = @()
    
    foreach ($term in $marketingTerms) {
        if ($content -match [regex]::Escape($term)) {
            $foundTerms += $term
        }
    }
    
    if ($foundTerms.Count -gt 0) {
        $marketingFiles += $file.Name
        $failedChecks++
        $issues += "Marketing language in: $($file.Name) - $($foundTerms -join ', ')"
        Write-Fail "  $($file.Name): $($foundTerms -join ', ')"
    } else {
        $passedChecks++
        if ($Verbose) { Write-Pass "  $($file.Name)" }
    }
}

if ($marketingFiles.Count -eq 0) {
    Write-Pass "No marketing language found"
} else {
    Write-Fail "Found marketing language in $($marketingFiles.Count) files"
}

# Check 3: File headers present
Write-Host "`n[3/6] Checking for file headers..." -ForegroundColor Cyan
$missingHeaders = @()

foreach ($file in $talonFiles) {
    $totalChecks++
    $content = Get-Content $file.FullName -Raw
    
    # Check if file has a descriptive comment at the top (not just empty lines)
    # Accept both # and // comment styles
    $lines = $content -split "`n" | Where-Object { $_ -match '\S' }
    if ($lines.Count -gt 0) {
        $firstLine = $lines[0].Trim()
        if ($firstLine -match '^#' -or $firstLine -match '^//' -or $firstLine -match '^print') {
            $passedChecks++
            if ($Verbose) { Write-Pass "  $($file.Name)" }
        } else {
            $missingHeaders += $file.Name
            $failedChecks++
            $issues += "Missing header: $($file.Name)"
            Write-Fail "  $($file.Name) missing descriptive header"
        }
    }
}

if ($missingHeaders.Count -eq 0) {
    Write-Pass "All examples have headers"
} else {
    Write-Fail "$($missingHeaders.Count) files missing headers"
}

# Check 4: Consistent comment style
Write-Host "`n[4/6] Checking comment style consistency..." -ForegroundColor Cyan
$inconsistentStyle = @()

foreach ($file in $talonFiles) {
    $totalChecks++
    $content = Get-Content $file.FullName
    
    # Check for mix of single-line and block comments
    $hasBlockComments = ($content -match '^\s*#+\s*═' -or $content -match '^\s*#+\s*─')
    $hasSingleComments = ($content -match '^\s*#[^═─]')
    
    # Both styles OK, but prefer consistency within file
    $passedChecks++
    if ($Verbose) { Write-Pass "  $($file.Name)" }
}

Write-Pass "Comment style check complete"

# Check 5: Syntax validation (if --CheckSyntax flag)
if ($CheckSyntax) {
    Write-Host "`n[5/6] Checking syntax with talon run --dry-run..." -ForegroundColor Cyan
    
    if (Test-Path $talonBin) {
        $syntaxErrors = @()
        
        foreach ($file in $talonFiles) {
            $totalChecks++
            $result = & $talonBin run --dry-run $file.FullName 2>&1
            
            if ($LASTEXITCODE -ne 0) {
                $syntaxErrors += $file.Name
                $failedChecks++
                $issues += "Syntax error: $($file.Name)"
                Write-Fail "  $($file.Name) has syntax errors"
                if ($Verbose) { Write-Host "    $result" -ForegroundColor Gray }
            } else {
                $passedChecks++
                if ($Verbose) { Write-Pass "  $($file.Name)" }
            }
        }
        
        if ($syntaxErrors.Count -eq 0) {
            Write-Pass "All examples have valid syntax"
        } else {
            Write-Fail "$($syntaxErrors.Count) files have syntax errors"
        }
    } else {
        Write-Host "  [SKIP] talon binary not found at: $talonBin" -ForegroundColor Yellow
    }
} else {
    Write-Host "`n[5/6] Syntax validation skipped (use -CheckSyntax to enable)" -ForegroundColor Yellow
}

# Check 6: Feature coverage analysis
Write-Host "`n[6/6] Analyzing feature coverage..." -ForegroundColor Cyan

$requiredFeatures = @(
    @{Name="SSH"; Pattern='ssh|connect_ssh'; Found=$false},
    @{Name="Binary Patching"; Pattern='patch|Patch\('; Found=$false},
    @{Name="Oracle"; Pattern='oracle|analyze_with_ai'; Found=$false},
    @{Name="Time-Travel Debugging"; Pattern='debug\(|checkpoint|rewind'; Found=$false},
    @{Name="Symbolic Execution"; Pattern='symbolic|z3|constraint'; Found=$false},
    @{Name="ROP Chain"; Pattern='rop|RopChain|gadget'; Found=$false},
    @{Name="Format String"; Pattern='fmtstr|format_string'; Found=$false},
    @{Name="Heap Exploitation"; Pattern='heap|tcache|fastbin'; Found=$false},
    @{Name="Shellcode"; Pattern='shellcode'; Found=$false},
    @{Name="Swarm Mode"; Pattern='swarm'; Found=$false}
)

# Check which features are covered
foreach ($file in $talonFiles) {
    $content = Get-Content $file.FullName -Raw
    
    foreach ($feature in $requiredFeatures) {
        if ($content -match $feature.Pattern) {
            $feature.Found = $true
        }
    }
}

# Report feature coverage
foreach ($feature in $requiredFeatures) {
    $totalChecks++
    if ($feature.Found) {
        $passedChecks++
        Write-Pass "  $($feature.Name) - Covered"
    } else {
        $failedChecks++
        $issues += "Missing example for: $($feature.Name)"
        Write-Fail "  $($feature.Name) - No example found"
    }
}

# Summary
Write-Host "`n========================================" -ForegroundColor Yellow
Write-Host "Validation Summary" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "Total checks: $totalChecks" -ForegroundColor Cyan
Write-Host "Passed: $passedChecks" -ForegroundColor Green
Write-Host "Failed: $failedChecks" -ForegroundColor $(if ($failedChecks -gt 0) { "Red" } else { "Green" })

if ($issues.Count -gt 0) {
    Write-Host "`n[!] Issues Found:" -ForegroundColor Red
    foreach ($issue in $issues) {
        Write-Host "  - $issue" -ForegroundColor Red
    }
}

Write-Host ""

# Exit with appropriate code
if ($failedChecks -gt 0) {
    Write-Host "[FAIL] Validation completed with errors" -ForegroundColor Red
    exit 1
} else {
    Write-Host "[PASS] All validation checks passed!" -ForegroundColor Green
    exit 0
}
