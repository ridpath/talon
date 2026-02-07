# Documentation Accuracy & Cleanup Verification Script
# This script verifies all CLI commands work as documented

$ErrorActionPreference = "Continue"
$RootDir = "C:\Users\rootless\.zenflow\worktrees\iamtalon-d954"
$TalonBin = "$RootDir\target\debug\talon.exe"
$ExamplesDir = "$RootDir\examples"
$Results = @()

Write-Host "========================================"
Write-Host "TALON Documentation Verification Script"
Write-Host "========================================"
Write-Host ""

# Check if binary exists
if (-not (Test-Path $TalonBin)) {
    Write-Host "[ERROR] TALON binary not found at $TalonBin" -ForegroundColor Red
    Write-Host "        Run: cargo build --bin talon" -ForegroundColor Yellow
    exit 1
}

Write-Host "[OK] Binary found: $TalonBin" -ForegroundColor Green
Write-Host ""

# Test 1: Verify main help command
Write-Host "[TEST] Verifying main --help command..." -ForegroundColor Cyan
$help = & $TalonBin --help 2>&1
if ($LASTEXITCODE -eq 0 -and $help -match "USAGE") {
    Write-Host "  [PASS] Main help command works" -ForegroundColor Green
    $Results += "main_help:PASS"
} else {
    Write-Host "  [FAIL] Main help command failed" -ForegroundColor Red
    $Results += "main_help:FAIL"
}

# Test 2: Verify version command
Write-Host "[TEST] Verifying --version command..." -ForegroundColor Cyan
$version = & $TalonBin --version 2>&1
if ($LASTEXITCODE -eq 0 -and $version -match "TALON") {
    Write-Host "  [PASS] Version command works" -ForegroundColor Green
    $Results += "version:PASS"
} else {
    Write-Host "  [FAIL] Version command failed" -ForegroundColor Red
    $Results += "version:FAIL"
}

# Test 3: Verify cache stats command
Write-Host "[TEST] Verifying cache stats command..." -ForegroundColor Cyan
$cache = & $TalonBin cache stats 2>&1
if ($LASTEXITCODE -eq 0 -and $cache -match "Cache") {
    Write-Host "  [PASS] Cache stats command works" -ForegroundColor Green
    $Results += "cache_stats:PASS"
} else {
    Write-Host "  [FAIL] Cache stats command failed" -ForegroundColor Red
    $Results += "cache_stats:FAIL"
}

# Test 4: Verify repl command starts (with timeout)
Write-Host "[TEST] Verifying repl command..." -ForegroundColor Cyan
$replTest = Start-Job -ScriptBlock {
    param($bin)
    echo "exit" | & $bin repl 2>&1
} -ArgumentList $TalonBin
$replTest | Wait-Job -Timeout 5 | Out-Null
$replOutput = Receive-Job -Job $replTest
Stop-Job -Job $replTest -ErrorAction SilentlyContinue
Remove-Job -Job $replTest -Force -ErrorAction SilentlyContinue

if ($replOutput -match "REPL|Welcome|talon>") {
    Write-Host "  [PASS] REPL command starts" -ForegroundColor Green
    $Results += "repl:PASS"
} else {
    Write-Host "  [FAIL] REPL command failed to start" -ForegroundColor Red
    $Results += "repl:FAIL"
}

# Test 5: Verify dry-run flag works
Write-Host "[TEST] Verifying --dry-run flag..." -ForegroundColor Cyan
$dryRun = & $TalonBin run "$ExamplesDir\01_basic_overflow.talon" --dry-run 2>&1
if ($dryRun -match "DRY-RUN") {
    Write-Host "  [PASS] Dry-run flag works" -ForegroundColor Green
    $Results += "dry_run:PASS"
} else {
    Write-Host "  [FAIL] Dry-run flag failed" -ForegroundColor Red
    $Results += "dry_run:FAIL"
}

# Test 6: Verify examples syntax
Write-Host "[TEST] Checking example files for syntax errors..." -ForegroundColor Cyan
$exampleFiles = Get-ChildItem "$ExamplesDir\*.talon" -File
$syntaxErrors = @()
$syntaxPassed = 0
$syntaxFailed = 0

foreach ($example in $exampleFiles) {
    $output = & $TalonBin run $example.FullName --dry-run 2>&1
    if ($output -match "ERROR.*Syntax|Unknown method") {
        $syntaxFailed++
        $syntaxErrors += $example.Name
    } else {
        $syntaxPassed++
    }
}

Write-Host "  Examples tested: $($exampleFiles.Count)" -ForegroundColor White
Write-Host "  Syntax valid: $syntaxPassed" -ForegroundColor Green
Write-Host "  Syntax errors: $syntaxFailed" -ForegroundColor Red

if ($syntaxFailed -gt 0) {
    Write-Host "  Files with syntax errors:" -ForegroundColor Yellow
    foreach ($errFile in $syntaxErrors) {
        Write-Host "    - $errFile" -ForegroundColor Yellow
    }
    $Results += "examples_syntax:FAIL($syntaxFailed)"
} else {
    Write-Host "  [PASS] All examples have valid syntax" -ForegroundColor Green
    $Results += "examples_syntax:PASS"
}

# Test 7: Check for emoticons
Write-Host "[TEST] Checking for emoticons in code..." -ForegroundColor Cyan
$emoticonCheck = git grep -P '[\x{1F600}-\x{1F64F}\x{1F300}-\x{1F5FF}\x{1F680}-\x{1F6FF}\x{1F1E0}-\x{1F1FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}]' . 2>&1
if ($emoticonCheck.Length -eq 0 -or $LASTEXITCODE -ne 0) {
    Write-Host "  [PASS] No emoticons found" -ForegroundColor Green
    $Results += "emoticons:PASS"
} else {
    Write-Host "  [FAIL] Emoticons found in code" -ForegroundColor Red
    $Results += "emoticons:FAIL"
}

# Test 8: Check for marketing language
Write-Host "[TEST] Checking for marketing language..." -ForegroundColor Cyan
$marketingCheck = git grep -iE 'world.class|best.in.class|leading|cutting.edge|revolutionary|game.changing|titan|god.tier' src/ docs/ README.md 2>&1
if ($marketingCheck.Length -eq 0 -or $LASTEXITCODE -ne 0) {
    Write-Host "  [PASS] No marketing language found" -ForegroundColor Green
    $Results += "marketing:PASS"
} else {
    Write-Host "  [FAIL] Marketing language found" -ForegroundColor Red
    $Results += "marketing:FAIL"
}

# Test 9: Check for tool comparisons
Write-Host "[TEST] Checking for tool comparisons..." -ForegroundColor Cyan
$comparisonCheck = git grep -iE 'better than|faster than|superior to|pwntools|metasploit' docs/ README.md examples/ 2>&1
if ($comparisonCheck.Length -eq 0 -or $LASTEXITCODE -ne 0) {
    Write-Host "  [PASS] No tool comparisons found" -ForegroundColor Green
    $Results += "comparisons:PASS"
} else {
    Write-Host "  [FAIL] Tool comparisons found" -ForegroundColor Red
    $Results += "comparisons:FAIL"
}

# Test 10: Check for test artifacts
Write-Host "[TEST] Checking for test artifacts in repo..." -ForegroundColor Cyan
$artifacts = Get-ChildItem -Recurse -Include '*.test','*.tmp','debug_*.log' -File -ErrorAction SilentlyContinue
if ($artifacts.Count -eq 0) {
    Write-Host "  [PASS] No test artifacts found" -ForegroundColor Green
    $Results += "artifacts:PASS"
} else {
    Write-Host "  [FAIL] Found $($artifacts.Count) test artifacts" -ForegroundColor Red
    $Results += "artifacts:FAIL($($artifacts.Count))"
}

# Summary
Write-Host ""
Write-Host "========================================"
Write-Host "VERIFICATION SUMMARY"
Write-Host "========================================"
$passCount = ($Results | Where-Object { $_ -match ":PASS" }).Count
$failCount = ($Results | Where-Object { $_ -match ":FAIL" }).Count
Write-Host "Tests Passed: $passCount" -ForegroundColor Green
Write-Host "Tests Failed: $failCount" -ForegroundColor Red
Write-Host ""

foreach ($result in $Results) {
    $parts = $result -split ":"
    $testName = $parts[0]
    $status = $parts[1]
    
    if ($status -match "PASS") {
        Write-Host "  [PASS] $testName" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] $testName - $status" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "========================================"
if ($failCount -eq 0) {
    Write-Host "ALL TESTS PASSED" -ForegroundColor Green
    exit 0
} else {
    Write-Host "SOME TESTS FAILED - SEE ABOVE" -ForegroundColor Red
    exit 1
}
