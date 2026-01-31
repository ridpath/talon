# PowerShell script for running TALON benchmarks on Windows

Write-Host "[*] Running TALON benchmarks..." -ForegroundColor Cyan

Set-Location "$PSScriptRoot\..\.."

Write-Host "[*] Building TALON in release mode..." -ForegroundColor Cyan
cargo build --release 2>&1 | Select-Object -First 20

Write-Host "[*] Running criterion benchmarks..." -ForegroundColor Cyan
cargo bench --bench vs_pwntools_bench 2>&1 | Out-File -FilePath "scripts\benchmarks\talon_results_raw.txt"

Write-Host "[*] Extracting benchmark results..." -ForegroundColor Cyan
python scripts\benchmarks\extract_talon_results.py

Write-Host "[+] TALON benchmark complete" -ForegroundColor Green
Write-Host "[+] Results written to scripts\benchmarks\talon_results.txt" -ForegroundColor Green
