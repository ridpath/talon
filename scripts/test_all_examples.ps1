# Test All Examples Script (PowerShell)
# Runs all .talon example scripts with timeout and resource limits
# Usage: .\scripts\test_all_examples.ps1 [-Verbose] [-Timeout 30]

param(
    [switch]$Verbose = $false,
    [int]$Timeout = 30,
    [switch]$Help = $false
)

if ($Help) {
    Write-Host "Usage: .\test_all_examples.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Verbose          Show script output"
    Write-Host "  -Timeout SECS     Set timeout per script (default: 30)"
    Write-Host "  -Help             Show this help message"
    exit 0
}

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$ExamplesDir = Join-Path $ProjectRoot "examples"
$FailedCount = 0
$PassedCount = 0
$SkippedCount = 0
$FailedScripts = @()

if (-not (Test-Path $ExamplesDir)) {
    Write-Host "Error: Examples directory not found: $ExamplesDir" -ForegroundColor Red
    exit 1
}

$TalonBin = Join-Path $ProjectRoot "target\debug\talon.exe"
if (-not (Test-Path $TalonBin)) {
    Write-Host "Building talon binary..." -ForegroundColor Blue
    Push-Location $ProjectRoot
    cargo build --quiet
    Pop-Location
    
    if (-not (Test-Path $TalonBin)) {
        Write-Host "Error: Failed to build talon binary" -ForegroundColor Red
        exit 1
    }
}

Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Blue
Write-Host "  TALON Example Script Validation" -ForegroundColor Blue
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Blue
Write-Host "Examples directory: $ExamplesDir"
Write-Host "Timeout per script: ${Timeout}s"
Write-Host "Verbose output: $Verbose"
Write-Host "───────────────────────────────────────────────────────────" -ForegroundColor Blue
Write-Host ""

$Scripts = Get-ChildItem -Path $ExamplesDir -Filter "*.talon"

foreach ($Script in $Scripts) {
    $ScriptName = $Script.Name
    Write-Host "Testing: " -NoNewline
    Write-Host ("{0,-45}" -f $ScriptName) -NoNewline
    Write-Host " ... " -NoNewline
    
    $TempFile = [System.IO.Path]::GetTempFileName()
    
    try {
        $Process = Start-Process -FilePath $TalonBin `
            -ArgumentList "run", $Script.FullName `
            -RedirectStandardOutput $TempFile `
            -RedirectStandardError $TempFile `
            -NoNewWindow `
            -PassThru
        
        $TimedOut = $false
        if (-not $Process.WaitForExit($Timeout * 1000)) {
            $Process.Kill()
            $TimedOut = $true
        }
        
        if ($TimedOut) {
            Write-Host "✗ FAIL (timeout)" -ForegroundColor Red
            $FailedCount++
            $FailedScripts += $ScriptName
        }
        elseif ($Process.ExitCode -eq 0) {
            Write-Host "✓ PASS" -ForegroundColor Green
            $PassedCount++
        }
        else {
            Write-Host "✗ FAIL (exit code: $($Process.ExitCode))" -ForegroundColor Red
            $FailedCount++
            $FailedScripts += $ScriptName
            
            if ($Verbose) {
                Write-Host "Output:" -ForegroundColor Yellow
                Get-Content $TempFile | Select-Object -First 20
                Write-Host ""
            }
        }
    }
    catch {
        Write-Host "✗ FAIL (exception)" -ForegroundColor Red
        $FailedCount++
        $FailedScripts += $ScriptName
        
        if ($Verbose) {
            Write-Host "Error: $_" -ForegroundColor Yellow
        }
    }
    finally {
        if (Test-Path $TempFile) {
            Remove-Item $TempFile -Force
        }
    }
}

Write-Host "───────────────────────────────────────────────────────────" -ForegroundColor Blue
Write-Host "Summary:"
Write-Host "  Passed:  $PassedCount" -ForegroundColor Green
Write-Host "  Failed:  $FailedCount" -ForegroundColor Red
Write-Host "  Skipped: $SkippedCount" -ForegroundColor Yellow

if ($FailedCount -gt 0) {
    Write-Host ""
    Write-Host "Failed examples:" -ForegroundColor Red
    foreach ($FailedScript in $FailedScripts) {
        Write-Host "  - $FailedScript"
    }
    Write-Host ""
    Write-Host "Test suite failed with $FailedCount error(s)" -ForegroundColor Red
    exit 1
}
else {
    Write-Host ""
    Write-Host "All examples passed successfully!" -ForegroundColor Green
    exit 0
}
