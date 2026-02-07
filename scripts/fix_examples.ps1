# Fix Example Files Script
# Fixes string multiplication and other common syntax issues

$ExamplesDir = "C:\Users\rootless\.zenflow\worktrees\iamtalon-d954\examples"
$FixedCount = 0
$TotalCount = 0

Write-Host "Fixing example files..." -ForegroundColor Cyan

# Get all .talon files
$files = Get-ChildItem -Path $ExamplesDir -Filter "*.talon" -File -Recurse

foreach ($file in $files) {
    $TotalCount++
    $content = Get-Content $file.FullName -Raw
    $originalContent = $content
    $changed = $false
    
    # Fix Pattern 1: String multiplication like "=" * 50, "-" * 60, etc.
    # Replace with actual repeated strings
    $patterns = @(
        @{Pattern = '"="\s*\*\s*50'; Replacement = '"=================================================="'},
        @{Pattern = '"="\s*\*\s*60'; Replacement = '"============================================================"'},
        @{Pattern = '"="\s*\*\s*70'; Replacement = '"======================================================================"'},
        @{Pattern = '"="\s*\*\s*80'; Replacement = '"================================================================================"'},
        @{Pattern = '"-"\s*\*\s*50'; Replacement = '"--------------------------------------------------"'},
        @{Pattern = '"-"\s*\*\s*60'; Replacement = '"------------------------------------------------------------"'},
        @{Pattern = '"-"\s*\*\s*70'; Replacement = '"----------------------------------------------------------------------"'},
        @{Pattern = '"-"\s*\*\s*80'; Replacement = '"--------------------------------------------------------------------------------"'},
        @{Pattern = '"#"\s*\*\s*50'; Replacement = '"##################################################"'},
        @{Pattern = '"#"\s*\*\s*60'; Replacement = '"############################################################"'},
        @{Pattern = '"\*"\s*\*\s*50'; Replacement = '"**************************************************"'},
        @{Pattern = '"\*"\s*\*\s*60'; Replacement = '"************************************************************"'},
        @{Pattern = '"_"\s*\*\s*50'; Replacement = '"__________________________________________________"'},
        @{Pattern = '"_"\s*\*\s*60'; Replacement = '"____________________________________________________________"'},
        @{Pattern = '" "\s*\*\s*50'; Replacement = '"                                                  "'},
        @{Pattern = '" "\s*\*\s*100'; Replacement = '"                                                                                                    "'}
    )
    
    foreach ($p in $patterns) {
        if ($content -match $p.Pattern) {
            $content = $content -replace $p.Pattern, $p.Replacement
            $changed = $true
        }
    }
    
    # Fix Pattern 2: elf.base_addr -> elf.base (though accessing is problematic, at least fix the name)
    if ($content -match 'elf\.base_addr') {
        $content = $content -replace 'elf\.base_addr', 'elf.base'
        $changed = $true
    }
    
    # Fix Pattern 3: Remove or comment out problematic property accesses
    # For now, just note them - we'll handle manually if needed
    
    if ($changed) {
        $content | Set-Content $file.FullName -NoNewline
        Write-Host "  [FIXED] $($file.Name)" -ForegroundColor Green
        $FixedCount++
    } else {
        Write-Host "  [SKIP] $($file.Name) - no changes needed" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Fix Summary:" -ForegroundColor Cyan
Write-Host "  Total files: $TotalCount" -ForegroundColor White
Write-Host "  Fixed: $FixedCount" -ForegroundColor Green
Write-Host "  Skipped: $($TotalCount - $FixedCount)" -ForegroundColor Gray
Write-Host "========================================" -ForegroundColor Cyan
