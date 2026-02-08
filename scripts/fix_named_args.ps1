#!/usr/bin/env pwsh
# Fix Python-style named arguments to TALON Map literal syntax
# Pattern: func(arg1="val", arg2=123) -> func({arg1: "val", arg2: 123})

$ExamplesDir = ".\examples"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Fixing Named Argument Syntax" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$allExamples = Get-ChildItem "$ExamplesDir\*.talon"
$fixedCount = 0

foreach ($file in $allExamples) {
    $content = Get-Content $file.FullName -Raw
    $originalContent = $content
    
    # Pattern: func(name="value") or func(name=value) or func(name=123)
    # Need to convert to: func({name: "value"})
    
    # This regex finds function calls with = assignments in parentheses
    # Match: identifier followed by ( then content with = signs
    $pattern = '(\w+)\s*\(\s*([^)]*=+[^)]*)\s*\)'
    
    $matches = [regex]::Matches($content, $pattern)
    
    if ($matches.Count -gt 0) {
        Write-Host "Processing: $($file.Name)" -ForegroundColor Yellow
        
        foreach ($match in $matches) {
            $funcName = $match.Groups[1].Value
            $argsString = $match.Groups[2].Value
            
            # Skip if it's already a map literal (contains ':')
            if ($argsString -match ':') {
                continue
            }
            
            # Skip operators like +=, -=, *=, /=
            if ($argsString -match '\+=' -or $argsString -match '-=' -or 
                $argsString -match '\*=' -or $argsString -match '/=') {
                continue
            }
            
            # Skip comparisons like ==, !=, <=, >=
            if ($argsString -match '==' -or $argsString -match '!=' -or 
                $argsString -match '<=' -or $argsString -match '>=') {
                continue
            }
            
            # Check if this looks like named args (contains single = for assignment)
            if ($argsString -match '^\s*\w+\s*=\s*') {
                # Convert: name="value", name2=123
                # To: {name: "value", name2: 123}
                $converted = $argsString -replace '(\w+)\s*=\s*', '$1: '
                $newCall = "$funcName({$converted})"
                
                $originalCall = $match.Value
                Write-Host "  - $originalCall" -ForegroundColor Gray
                Write-Host "  + $newCall" -ForegroundColor Green
                
                $content = $content.Replace($originalCall, $newCall)
            }
        }
        
        if ($content -ne $originalContent) {
            Set-Content -Path $file.FullName -Value $content -NoNewline
            $fixedCount++
            Write-Host "  [FIXED] $($file.Name)" -ForegroundColor Green
        }
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "SUMMARY" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan
Write-Host "Files fixed: $fixedCount" -ForegroundColor Green
