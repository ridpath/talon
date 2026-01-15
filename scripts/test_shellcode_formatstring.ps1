# Shellcode & Format String Test Runner (PowerShell)
# Validates all payload generation and exploitation primitives

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "TALON Shellcode & Format String Tests" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Test counter
$TOTAL_TESTS = 99
$PASSED = 0
$FAILED = 0

# Phase 1: Shellcode Tests
Write-Host "Phase 1: Shellcode Tests (46 tests)" -ForegroundColor Yellow
Write-Host "==========================================" -ForegroundColor Yellow

try {
    $output = cargo test shellcode_test --quiet 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ All shellcode tests passed" -ForegroundColor Green
        $PASSED += 46
    } else {
        Write-Host "✗ Some shellcode tests failed" -ForegroundColor Red
        Write-Host $output
        $FAILED += 46
    }
} catch {
    Write-Host "✗ Error running shellcode tests: $_" -ForegroundColor Red
    $FAILED += 46
}

Write-Host ""

# Phase 2: Format String Tests
Write-Host "Phase 2: Format String Tests (53 tests)" -ForegroundColor Yellow
Write-Host "==========================================" -ForegroundColor Yellow

try {
    $output = cargo test format_string_test --quiet 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ All format string tests passed" -ForegroundColor Green
        $PASSED += 53
    } else {
        Write-Host "✗ Some format string tests failed" -ForegroundColor Red
        Write-Host $output
        $FAILED += 53
    }
} catch {
    Write-Host "✗ Error running format string tests: $_" -ForegroundColor Red
    $FAILED += 53
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Test Results Summary" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Total Tests:  $TOTAL_TESTS"
Write-Host "Passed:       $PASSED" -ForegroundColor Green

if ($FAILED -gt 0) {
    Write-Host "Failed:       $FAILED" -ForegroundColor Red
} else {
    Write-Host "Failed:       0"
}

Write-Host "==========================================" -ForegroundColor Cyan

if ($FAILED -eq 0) {
    Write-Host "✓ ALL TESTS PASSED!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "✗ SOME TESTS FAILED" -ForegroundColor Red
    Write-Host ""
    Write-Host "To debug failures, run:"
    Write-Host "  cargo test shellcode_test -- --nocapture"
    Write-Host "  cargo test format_string_test -- --nocapture"
    exit 1
}
