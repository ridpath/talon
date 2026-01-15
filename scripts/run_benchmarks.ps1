#!/usr/bin/env pwsh

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Set-Location $ProjectRoot

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "TALON Performance Benchmarking Suite" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

$OutputDir = "benchmark-results"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$ReportFile = "$OutputDir/benchmark_report_$Timestamp.md"

$RustVersion = (rustc --version)
$Platform = "$([System.Environment]::OSVersion.Platform) $([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)"

"# TALON Benchmark Report" | Out-File -FilePath $ReportFile -Encoding UTF8
"" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"**Date:** $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"**Platform:** $Platform" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"**Rust Version:** $RustVersion" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"---" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"" | Out-File -FilePath $ReportFile -Append -Encoding UTF8

function Run-Benchmark {
    param(
        [string]$BenchName,
        [string]$BenchFile
    )
    
    Write-Host ""
    Write-Host "Running $BenchName benchmarks..." -ForegroundColor Yellow
    Write-Host "======================================" -ForegroundColor Yellow
    
    $OutputFile = "$OutputDir/${BenchFile}_${Timestamp}.txt"
    cargo bench --bench $BenchFile -- --output-format bencher | Tee-Object -FilePath $OutputFile
    
    "" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
    "## $BenchName" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
    "" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
    '```' | Out-File -FilePath $ReportFile -Append -Encoding UTF8
    Get-Content $OutputFile | Select-String "test" | Select-Object -First 30 | Out-File -FilePath $ReportFile -Append -Encoding UTF8
    '```' | Out-File -FilePath $ReportFile -Append -Encoding UTF8
    "" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
}

Run-Benchmark -BenchName "Parser" -BenchFile "parser_bench"
Run-Benchmark -BenchName "Interpreter" -BenchFile "interpreter_bench"
Run-Benchmark -BenchName "Binary Analysis" -BenchFile "binary_analysis_bench"
Run-Benchmark -BenchName "ROP Tools" -BenchFile "rop_bench"

Write-Host ""
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Performance Summary" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

"" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"---" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"## Summary" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"All benchmarks completed successfully. Results are stored in:" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"- Report: ``$ReportFile``" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"- Detailed results: ``$OutputDir/``" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"- Criterion HTML reports: ``target/criterion/``" | Out-File -FilePath $ReportFile -Append -Encoding UTF8
"" | Out-File -FilePath $ReportFile -Append -Encoding UTF8

Write-Host ""
Write-Host "✓ All benchmarks completed successfully" -ForegroundColor Green
Write-Host ""
Write-Host "Results saved to:" -ForegroundColor Cyan
Write-Host "  - Report: $ReportFile"
Write-Host "  - Raw data: $OutputDir/"
Write-Host "  - HTML reports: target/criterion/"
Write-Host ""
Write-Host "To view HTML reports, open target/criterion/report/index.html in a browser" -ForegroundColor Yellow
Write-Host ""

$OpenReport = Read-Host "Open HTML report in browser? [y/N]"
if ($OpenReport -eq "y" -or $OpenReport -eq "Y") {
    Start-Process "target/criterion/report/index.html"
}
