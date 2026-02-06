# OpSec Sanitization Audit Script (Windows PowerShell)
# Verifies that release binaries are properly sanitized

param(
    [string]$BinaryPath = "target\release\talon.exe"
)

$ErrorActionPreference = "Stop"

Write-Host "========================================"
Write-Host "TALON OpSec Sanitization Audit"
Write-Host "========================================"
Write-Host ""

# Check if binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Host "[FAIL] Binary not found: $BinaryPath" -ForegroundColor Red
    Write-Host "Build the release binary first:"
    Write-Host "  cargo build --release"
    exit 1
}

Write-Host "Auditing binary: $BinaryPath"
Write-Host ""

# Initialize counters
$PassCount = 0
$FailCount = 0
$WarnCount = 0

# Function to extract strings from binary (simplified)
function Get-BinaryStrings {
    param([string]$Path)
    
    # Use findstr to search binary (basic string extraction)
    # Note: This is a simplified version. For production, consider using strings.exe from SysInternals
    $content = [System.IO.File]::ReadAllText($Path, [System.Text.Encoding]::ASCII)
    return $content -split "`0" | Where-Object { $_.Length -gt 3 }
}

Write-Host "Extracting strings from binary (this may take a moment)..."
$strings = Get-BinaryStrings -Path $BinaryPath

# Test 1: Check for Zenflow references
Write-Host -NoNewline "Test 1: Checking for 'Zenflow' references... "
$zenflowRefs = $strings | Select-String -Pattern "zenflow" -SimpleMatch -CaseSensitive:$false
if ($zenflowRefs) {
    Write-Host "[FAIL]" -ForegroundColor Red
    Write-Host "  Found Zenflow references in binary:"
    $zenflowRefs | ForEach-Object { Write-Host "  $_" }
    $FailCount++
} else {
    Write-Host "[PASS]" -ForegroundColor Green
    $PassCount++
}

# Test 2: Check for interactivetalon references
Write-Host -NoNewline "Test 2: Checking for 'interactivetalon' references... "
$interactivetalonRefs = $strings | Select-String -Pattern "interactivetalon" -SimpleMatch -CaseSensitive:$false
if ($interactivetalonRefs) {
    Write-Host "[FAIL]" -ForegroundColor Red
    Write-Host "  Found interactivetalon references in binary:"
    $interactivetalonRefs | ForEach-Object { Write-Host "  $_" }
    $FailCount++
} else {
    Write-Host "[PASS]" -ForegroundColor Green
    $PassCount++
}

# Test 3: Check for excessive src/ paths
Write-Host -NoNewline "Test 3: Checking for source file paths... "
$srcRefs = $strings | Select-String -Pattern "src[/\\]" -SimpleMatch
$srcCount = ($srcRefs | Measure-Object).Count
if ($srcCount -gt 50) {
    Write-Host "[FAIL]" -ForegroundColor Red
    Write-Host "  Found $srcCount instances of 'src/' (threshold: 50)"
    $FailCount++
} elseif ($srcCount -gt 10) {
    Write-Host "[WARN]" -ForegroundColor Yellow
    Write-Host "  Found $srcCount instances of 'src/' (consider reducing)"
    $WarnCount++
} else {
    Write-Host "[PASS] ($srcCount instances)" -ForegroundColor Green
    $PassCount++
}

# Test 4: Check binary size
Write-Host -NoNewline "Test 4: Checking binary size... "
$binarySize = (Get-Item $BinaryPath).Length
$binarySizeMB = [math]::Round($binarySize / 1MB, 2)
if ($binarySizeMB -gt 50) {
    Write-Host "[WARN]" -ForegroundColor Yellow
    Write-Host "  Binary size: ${binarySizeMB}MB (target: <50MB)"
    $WarnCount++
} else {
    Write-Host "[PASS] (${binarySizeMB}MB)" -ForegroundColor Green
    $PassCount++
}

# Test 5: Check for common sensitive strings
Write-Host -NoNewline "Test 5: Checking for sensitive strings... "
$sensitiveKeywords = @("TODO", "FIXME", "XXX", "HACK", "password", "secret", "api_key")
$sensitiveFound = $false
foreach ($keyword in $sensitiveKeywords) {
    $matches = $strings | Select-String -Pattern $keyword -SimpleMatch -CaseSensitive:$false
    if ($matches) {
        if (-not $sensitiveFound) {
            Write-Host "[WARN]" -ForegroundColor Yellow
            $sensitiveFound = $true
        }
        Write-Host "  Found potentially sensitive keyword: $keyword"
    }
}
if (-not $sensitiveFound) {
    Write-Host "[PASS]" -ForegroundColor Green
    $PassCount++
} else {
    $WarnCount++
}

# Test 6: Check for panic messages with file paths
Write-Host -NoNewline "Test 6: Checking for panic messages with file paths... "
$panicPaths = $strings | Select-String -Pattern "panicked at.*:\d+:\d+"
$panicCount = ($panicPaths | Measure-Object).Count
if ($panicCount -gt 5) {
    Write-Host "[FAIL]" -ForegroundColor Red
    Write-Host "  Found $panicCount panic messages with file:line:col info"
    $FailCount++
} elseif ($panicCount -gt 0) {
    Write-Host "[WARN]" -ForegroundColor Yellow
    Write-Host "  Found $panicCount panic messages with location info"
    $WarnCount++
} else {
    Write-Host "[PASS]" -ForegroundColor Green
    $PassCount++
}

# Summary
Write-Host ""
Write-Host "========================================"
Write-Host "Audit Summary"
Write-Host "========================================"
Write-Host "Passed:   $PassCount" -ForegroundColor Green
Write-Host "Warnings: $WarnCount" -ForegroundColor Yellow
Write-Host "Failed:   $FailCount" -ForegroundColor Red
Write-Host ""

if ($FailCount -gt 0) {
    Write-Host "OpSec audit FAILED" -ForegroundColor Red
    Write-Host "Review failures above and rebuild with proper sanitization."
    exit 1
} elseif ($WarnCount -gt 0) {
    Write-Host "OpSec audit passed with warnings" -ForegroundColor Yellow
    Write-Host "Review warnings and consider additional sanitization if needed."
    exit 0
} else {
    Write-Host "OpSec audit PASSED" -ForegroundColor Green
    Write-Host "Binary is properly sanitized for production release."
    exit 0
}
