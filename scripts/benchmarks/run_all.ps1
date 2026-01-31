# PowerShell script for running complete benchmark suite on Windows

$ErrorActionPreference = "Stop"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  TALON vs Pwntools Performance Benchmark Suite" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "[*] Step 1/3: Running TALON benchmarks..." -ForegroundColor Yellow
Write-Host "-----------------------------------------------------------" -ForegroundColor DarkGray
& "$PSScriptRoot\bench_talon.ps1"
Write-Host ""

Write-Host "[*] Step 2/3: Running Pwntools benchmarks..." -ForegroundColor Yellow
Write-Host "-----------------------------------------------------------" -ForegroundColor DarkGray
$pythonCmd = $null
if (Get-Command python3 -ErrorAction SilentlyContinue) {
    $pythonCmd = "python3"
} elseif (Get-Command python -ErrorAction SilentlyContinue) {
    $pythonCmd = "python"
}

if ($pythonCmd) {
    & $pythonCmd "$PSScriptRoot\bench_pwntools.py"
} else {
    Write-Host "[!] Python not found. Skipping pwntools benchmarks." -ForegroundColor Red
    Write-Host "[!] Install Python and pwntools to run full comparison." -ForegroundColor Red
}
Write-Host ""

Write-Host "[*] Step 3/3: Generating comparison report..." -ForegroundColor Yellow
Write-Host "-----------------------------------------------------------" -ForegroundColor DarkGray
if ($pythonCmd) {
    & $pythonCmd "$PSScriptRoot\compare.py"
} else {
    Write-Host "[!] Cannot generate comparison - Python required" -ForegroundColor Red
}
Write-Host ""

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Benchmark Complete" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "[+] Results:" -ForegroundColor Green
Write-Host "    - TALON results:     talon_results.txt"
Write-Host "    - Pwntools results:  pwntools_results.txt"
Write-Host "    - Comparison report: BENCHMARKS.md"
Write-Host ""

$benchmarkFile = "$PSScriptRoot\BENCHMARKS.md"
if (Test-Path $benchmarkFile) {
    Write-Host "[*] Summary from BENCHMARKS.md:" -ForegroundColor Cyan
    Write-Host "-----------------------------------------------------------" -ForegroundColor DarkGray
    
    $content = Get-Content $benchmarkFile -Raw
    if ($content -match '## Performance Summary(.*?)##') {
        $summary = $matches[1]
        Write-Host $summary
    }
    
    Write-Host "-----------------------------------------------------------" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "[+] To view full report: type '$benchmarkFile'" -ForegroundColor Green
