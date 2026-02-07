# Dead Code Audit Script
# Finds modules that exist as .rs files but aren't declared in lib.rs or main.rs

$srcDir = "C:\Users\rootless\.zenflow\worktrees\iamtalon-d954\src"
$libRs = Get-Content "$srcDir\lib.rs" -Raw
$mainRs = Get-Content "$srcDir\main.rs" -Raw

# Get all .rs files (excluding lib.rs, main.rs, and subdirectories)
$allModules = Get-ChildItem "$srcDir\*.rs" | Where-Object { 
    $_.Name -ne "lib.rs" -and $_.Name -ne "main.rs" 
} | ForEach-Object {
    $_.BaseName
}

Write-Host "Total modules found: $($allModules.Count)" -ForegroundColor Cyan
Write-Host ""

$undeclaredModules = @()
$declaredInLib = @()
$declaredInMain = @()
$commentedOut = @()

foreach ($module in $allModules) {
    $modPattern = "mod\s+$module"
    $inLib = $libRs -match $modPattern
    $inMain = $mainRs -match $modPattern
    $commented = $mainRs -match "//\s*mod\s+$module" -or $libRs -match "//\s*mod\s+$module"
    
    if ($commented) {
        $commentedOut += $module
    } elseif ($inLib) {
        $declaredInLib += $module
    } elseif ($inMain) {
        $declaredInMain += $module
    } else {
        $undeclaredModules += $module
    }
}

Write-Host "=== AUDIT RESULTS ===" -ForegroundColor Yellow
Write-Host ""
Write-Host "Declared in lib.rs (public API): $($declaredInLib.Count)" -ForegroundColor Green
Write-Host "Declared in main.rs (CLI-only): $($declaredInMain.Count)" -ForegroundColor Green
Write-Host "Commented out (intentionally disabled): $($commentedOut.Count)" -ForegroundColor Magenta
Write-Host "UNDECLARED (true dead code): $($undeclaredModules.Count)" -ForegroundColor Red
Write-Host ""

if ($commentedOut.Count -gt 0) {
    Write-Host "=== COMMENTED OUT MODULES ===" -ForegroundColor Magenta
    $commentedOut | Sort-Object | ForEach-Object { Write-Host "  - $_" }
    Write-Host ""
}

if ($undeclaredModules.Count -gt 0) {
    Write-Host "=== UNDECLARED MODULES (DEAD CODE) ===" -ForegroundColor Red
    $undeclaredModules | Sort-Object | ForEach-Object { Write-Host "  - $_" }
    Write-Host ""
}

# Save detailed report
$reportPath = "C:\Users\rootless\.zenflow\worktrees\iamtalon-d954\DEAD_CODE_AUDIT_REPORT.md"
$report = @"
# Dead Code Audit Report
Generated: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

## Summary
- **Total modules found**: $($allModules.Count)
- **Declared in lib.rs (public API)**: $($declaredInLib.Count)
- **Declared in main.rs (CLI-only)**: $($declaredInMain.Count)
- **Commented out (intentionally disabled)**: $($commentedOut.Count)
- **UNDECLARED (true dead code)**: $($undeclaredModules.Count)

## Commented Out Modules
$(if ($commentedOut.Count -gt 0) { $commentedOut | Sort-Object | ForEach-Object { "- ``$_``" } | Out-String } else { "None" })

## Undeclared Modules (Dead Code Requiring Integration)
$(if ($undeclaredModules.Count -gt 0) { $undeclaredModules | Sort-Object | ForEach-Object { "- ``$_``" } | Out-String } else { "None" })

## Declared in lib.rs (Public API)
$($declaredInLib | Sort-Object | ForEach-Object { "- ``$_``" } | Out-String)

## Declared in main.rs (CLI-Only)
$($declaredInMain | Sort-Object | ForEach-Object { "- ``$_``" } | Out-String)
"@

Set-Content -Path $reportPath -Value $report
Write-Host "Detailed report saved to: $reportPath" -ForegroundColor Cyan
