# PowerShell script for fuzzing on Windows

$ErrorActionPreference = "Stop"

$TARGETS = @(
    "fuzz_parser",
    "fuzz_elf_parser",
    "fuzz_pe_parser",
    "fuzz_shellcode_generator",
    "fuzz_format_string",
    "fuzz_heap_tools",
    "fuzz_packing_tools",
    "fuzz_rop_gadget_finder",
    "fuzz_rop_chain_builder",
    "fuzz_auto_solver"
)

param(
    [int]$Duration = 300,
    [string]$Target = ""
)

Write-Host "╔══════════════════════════════════════════════════════════╗" -ForegroundColor Blue
Write-Host "║           TALON Fuzzing Test Suite                      ║" -ForegroundColor Blue
Write-Host "╚══════════════════════════════════════════════════════════╝" -ForegroundColor Blue
Write-Host ""

if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: cargo not found. Please install Rust toolchain." -ForegroundColor Red
    exit 1
}

if (!(cargo fuzz --version 2>$null)) {
    Write-Host "Installing cargo-fuzz..." -ForegroundColor Yellow
    cargo install cargo-fuzz
}

$TargetsToRun = $TARGETS
if ($Target -ne "") {
    if ($TARGETS -notcontains $Target) {
        Write-Host "ERROR: Unknown target '$Target'" -ForegroundColor Red
        Write-Host "Available targets: $($TARGETS -join ', ')"
        exit 1
    }
    $TargetsToRun = @($Target)
}

$TotalCrashes = 0
$FailedTargets = @()

foreach ($target in $TargetsToRun) {
    Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
    Write-Host "Running: $target (${Duration}s)" -ForegroundColor Blue
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Blue
    
    $StartTime = Get-Date
    
    $result = & cargo +nightly fuzz run $target -- "-max_total_time=$Duration" "-print_final_stats=1" 2>&1
    $ExitCode = $LASTEXITCODE
    
    $EndTime = Get-Date
    $Elapsed = ($EndTime - $StartTime).TotalSeconds
    
    if ($ExitCode -eq 0) {
        Write-Host "✓ $target completed in $([math]::Round($Elapsed))s (no crashes)" -ForegroundColor Green
    } else {
        Write-Host "✗ $target failed after $([math]::Round($Elapsed))s (exit code: $ExitCode)" -ForegroundColor Red
        $FailedTargets += $target
        
        $ArtifactPath = "fuzz\artifacts\$target"
        if (Test-Path $ArtifactPath) {
            $Crashes = Get-ChildItem -Path $ArtifactPath -Filter "crash-*" -ErrorAction SilentlyContinue
            if ($Crashes.Count -gt 0) {
                $TotalCrashes += $Crashes.Count
                Write-Host "  Found $($Crashes.Count) crash artifact(s):" -ForegroundColor Red
                Get-ChildItem -Path $ArtifactPath | Format-Table -AutoSize
                
                Write-Host "`n  Sample crash (first artifact):" -ForegroundColor Yellow
                $FirstCrash = $Crashes | Select-Object -First 1
                if ($FirstCrash) {
                    Format-Hex $FirstCrash.FullName | Select-Object -First 20
                }
            }
        }
    }
}

Write-Host "`n╔══════════════════════════════════════════════════════════╗" -ForegroundColor Blue
Write-Host "║                  Fuzzing Summary                         ║" -ForegroundColor Blue
Write-Host "╚══════════════════════════════════════════════════════════╝" -ForegroundColor Blue
Write-Host ""
Write-Host "Total targets tested: $($TargetsToRun.Count)"
Write-Host "Failed targets: $($FailedTargets.Count)"
Write-Host "Total crashes: $TotalCrashes"

if ($FailedTargets.Count -gt 0) {
    Write-Host "`nFailed targets:" -ForegroundColor Red
    foreach ($failed in $FailedTargets) {
        Write-Host "  - $failed"
    }
}

if ($TotalCrashes -gt 0) {
    Write-Host "`nCRITICAL: Found $TotalCrashes crash(es)!" -ForegroundColor Red
    Write-Host "Please review artifacts in fuzz\artifacts\ directory" -ForegroundColor Yellow
    exit 1
} else {
    Write-Host "`n✓ All fuzz tests passed successfully!" -ForegroundColor Green
    exit 0
}
