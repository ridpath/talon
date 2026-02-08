# Script to convert TALON examples from 'end' keyword style to curly brace style
# This fixes the SYNTAX_ERROR issues caused by nested if/else with end_block

param(
    [string]$Path = "examples",
    [switch]$DryRun
)

$files = Get-ChildItem -Path $Path -Filter "*.talon" -Recurse
$totalConverted = 0
$conversionLog = @()

foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw
    $originalContent = $content
    $changed = $false
    
    Write-Host "`nProcessing: $($file.Name)" -ForegroundColor Cyan
    
    # Pattern 1: Convert simple if without braces to if with braces
    # if condition\n    statements\nend
    # -> if condition {\n    statements\n}
    $pattern1 = '(?m)^(\s*)(if\s+[^\n]+)\n((?:(?!\belse\b|\bend\b)(?:\s{4}|\t).+\n)+)^(\s*)end\s*$'
    if ($content -match $pattern1) {
        $content = $content -replace $pattern1, '$1$2 {$3$4}'
        $changed = $true
        Write-Host "  - Converted simple if statement" -ForegroundColor Green
    }
    
    # Pattern 2: Convert if/else without braces
    # if condition\n    statements\nelse\n    statements\nend
    # -> if condition {\n    statements\n} else {\n    statements\n}
    $pattern2 = '(?m)^(\s*)(if\s+[^\n]+)\n((?:(?!\belse\b|\bend\b)(?:\s{4}|\t).+\n)+)^(\s*)else\s*\n((?:(?!\bend\b)(?:\s{4}|\t).+\n)+)^(\s*)end\s*$'
    if ($content -match $pattern2) {
        $content = $content -replace $pattern2, '$1$2 {$3$4} else {$5$6}'
        $changed = $true
        Write-Host "  - Converted if/else statement" -ForegroundColor Green
    }
    
    # Pattern 3: Convert for loops without braces
    # for var in expr\n    statements\nend
    # -> for var in expr {\n    statements\n}
    $pattern3 = '(?m)^(\s*)(for\s+\w+\s+in\s+[^\n]+)\n((?:(?!\bend\b)(?:\s{4}|\t).+\n)+)^(\s*)end\s*$'
    if ($content -match $pattern3) {
        $content = $content -replace $pattern3, '$1$2 {$3$4}'
        $changed = $true
        Write-Host "  - Converted for loop" -ForegroundColor Green
    }
    
    # Pattern 4: Convert while loops without braces
    # while condition\n    statements\nend
    # -> while condition {\n    statements\n}
    $pattern4 = '(?m)^(\s*)(while\s+[^\n]+)\n((?:(?!\bend\b)(?:\s{4}|\t).+\n)+)^(\s*)end\s*$'
    if ($content -match $pattern4) {
        $content = $content -replace $pattern4, '$1$2 {$3$4}'
        $changed = $true
        Write-Host "  - Converted while loop" -ForegroundColor Green
    }
    
    # Pattern 5: Convert function definitions without braces
    # define function name(args)\n    statements\nend
    # -> define function name(args) {\n    statements\n}
    $pattern5 = '(?m)^(\s*)((?:async\s+)?define\s+function\s+\w+\([^\)]*\)(?:\s*:\s*\w+)?)\n((?:(?!\bend\b)(?:\s{4}|\t).+\n)+)^(\s*)end\s*$'
    if ($content -match $pattern5) {
        $content = $content -replace $pattern5, '$1$2 {$3$4}'
        $changed = $true
        Write-Host "  - Converted function definition" -ForegroundColor Green
    }
    
    if ($changed) {
        $totalConverted++
        $conversionLog += $file.Name
        
        if (-not $DryRun) {
            Set-Content -Path $file.FullName -Value $content -NoNewline
            Write-Host "  ✓ File updated" -ForegroundColor Green
        } else {
            Write-Host "  [DRY RUN] Would update file" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  - No changes needed" -ForegroundColor Gray
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "CONVERSION SUMMARY" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Total files processed: $($files.Count)"
Write-Host "Files converted: $totalConverted"

if ($totalConverted -gt 0) {
    Write-Host "`nConverted files:" -ForegroundColor Green
    $conversionLog | ForEach-Object { Write-Host "  - $_" }
}

if ($DryRun) {
    Write-Host "`n[DRY RUN MODE] No files were modified" -ForegroundColor Yellow
    Write-Host "Run without DryRun to apply changes" -ForegroundColor Yellow
}
