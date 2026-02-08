#!/usr/bin/env pwsh
# Test all examples and categorize errors

$BinaryPath = ".\target\debug\talon.exe"
$ExamplesDir = ".\examples"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Testing All TALON Examples" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$allExamples = Get-ChildItem "$ExamplesDir\*.talon" | Sort-Object Name
$passCount = 0
$failCount = 0
$errors = @{}

foreach ($example in $allExamples) {
    $name = $example.Name
    Write-Host "Testing: $name" -NoNewline
    
    $output = & $BinaryPath run $example.FullName --dry-run 2>&1 | Out-String
    
    if ($output -match '\[ERROR\]') {
        # Extract error type
        if ($output -match 'UNKNOWN METHOD') {
            $errorType = "UNKNOWN_METHOD"
        } elseif ($output -match 'UNDEFINED VARIABLE') {
            $errorType = "UNDEFINED_VAR"
        } elseif ($output -match 'Syntax Error') {
            $errorType = "SYNTAX_ERROR"
        } elseif ($output -match 'TYPE ERROR') {
            $errorType = "TYPE_ERROR"
        } elseif ($output -match 'thread.*overflowed') {
            $errorType = "STACK_OVERFLOW"
        } else {
            $errorType = "OTHER_ERROR"
        }
        
        if (-not $errors.ContainsKey($errorType)) {
            $errors[$errorType] = @()
        }
        $errors[$errorType] += $name
        
        Write-Host " [FAIL] $errorType" -ForegroundColor Red
        $failCount++
    } else {
        Write-Host " [PASS]" -ForegroundColor Green
        $passCount++
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "RESULTS SUMMARY" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "Total Examples: $($allExamples.Count)"
Write-Host "Passed: $passCount ($([math]::Round($passCount / $allExamples.Count * 100, 1))%)" -ForegroundColor Green
Write-Host "Failed: $failCount ($([math]::Round($failCount / $allExamples.Count * 100, 1))%)" -ForegroundColor Red

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "ERROR BREAKDOWN" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

foreach ($errorType in $errors.Keys | Sort-Object) {
    $count = $errors[$errorType].Count
    Write-Host "`n$errorType ($count files):" -ForegroundColor Yellow
    foreach ($file in $errors[$errorType] | Sort-Object) {
        Write-Host "  - $file"
    }
}
