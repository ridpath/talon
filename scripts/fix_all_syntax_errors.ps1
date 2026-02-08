# Batch fix all SYNTAX_ERROR examples by converting from 'end' keyword to curly braces
# This script handles all common patterns systematically

$ErrorActionPreference = "Stop"

# List of files with SYNTAX_ERROR
$filesToFix = @(
    "06_ctf_automation.talon",
    "ctf_blind_rop.talon",
    "ctf_kernel_exploit.talon",
    "ctf_multi_stage_pwn.talon",
    "ctf_one_gadget_pwn.talon",
    "ctf_shellcode_encoder.talon",
    "exploit_chain_buffer_overflow.talon",
    "exploit_chain_format_string.talon",
    "exploit_chain_heap_uaf.talon",
    "exploit_chain_with_recovery.talon",
    "memory_scrubbing.talon",
    "orchestrator_graph.talon",
    "orchestrator_parallel.talon",
    "orchestrator_resilient.talon",
    "orchestrator_timetravel.talon",
    "phase21_meta_programming.talon",
    "phase22_demo.talon",
    "phase22_symbiotic_execution.talon",
    "polymorphic_shellcode.talon",
    "swarm_libc_leak.talon",
    "swarm_mass_exploit.talon",
    "swarm_mass_pwn.talon",
    "swarm_subnet_scan.talon",
    "time_travel_debugging.talon",
    "tutorial_01_basics.talon",
    "tutorial_02_exploitation.talon",
    "world_class_exploit.talon"
)

$examplesDir = "examples"
$convertedCount = 0
$failedFiles = @()

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Batch Converting SYNTAX_ERROR Examples" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

foreach ($filename in $filesToFix) {
    $filepath = Join-Path $examplesDir $filename
    
    if (-not (Test-Path $filepath)) {
        Write-Host "  [SKIP] File not found: $filename" -ForegroundColor Yellow
        continue
    }
    
    Write-Host "Processing: $filename" -ForegroundColor White
    
    try {
        # Read file content
        $content = Get-Content $filepath -Raw -Encoding UTF8
        $originalContent = $content
        $changed = $false
        
        # Pattern 1: Convert nested if/else with end to curly braces
        # This is the most complex pattern - handle innermost first
        $maxIterations = 10  # Prevent infinite loops
        $iteration = 0
        
        while ($iteration -lt $maxIterations) {
            $iteration++
            $beforeLength = $content.Length
            
            # Match if...else...end pattern (innermost first)
            $pattern = '(?m)^(\s*)(if\s+[^\r\n]+)\r?\n((?:(?!^\s*(?:if|else|end)\b)[^\r\n]+\r?\n)*?)^(\s*)else\r?\n((?:(?!^\s*(?:if|else|end)\b)[^\r\n]+\r?\n)*?)^(\s*)end\s*$'
            if ($content -match $pattern) {
                $content = $content -replace $pattern, '$1$2 {$3$4} else {$5$6}'
                $changed = $true
            }
            
            # Match simple if...end pattern (no else)
            $pattern = '(?m)^(\s*)(if\s+[^\r\n]+)\r?\n((?:(?!^\s*(?:if|else|end)\b)[^\r\n]+\r?\n)*?)^(\s*)end\s*$'
            if ($content -match $pattern) {
                $content = $content -replace $pattern, '$1$2 {$3$4}'
                $changed = $true
            }
            
            # If no changes in this iteration, we're done
            if ($content.Length -eq $beforeLength) {
                break
            }
        }
        
        # Pattern 2: Convert for loops
        $pattern = '(?m)^(\s*)(for\s+\w+\s+in\s+[^\r\n]+)\r?\n((?:(?!^\s*end\b)[^\r\n]+\r?\n)*?)^(\s*)end\s*$'
        if ($content -match $pattern) {
            $content = $content -replace $pattern, '$1$2 {$3$4}'
            $changed = $true
        }
        
        # Pattern 3: Convert while loops
        $pattern = '(?m)^(\s*)(while\s+[^\r\n]+)\r?\n((?:(?!^\s*end\b)[^\r\n]+\r?\n)*?)^(\s*)end\s*$'
        if ($content -match $pattern) {
            $content = $content -replace $pattern, '$1$2 {$3$4}'
            $changed = $true
        }
        
        # Pattern 4: Convert function definitions
        $pattern = '(?m)^(\s*)((?:async\s+)?define\s+function\s+\w+\([^\)]*\)(?:\s*:\s*\w+)?)\r?\n((?:(?!^\s*end\b)[^\r\n]+\r?\n)*?)^(\s*)end\s*$'
        if ($content -match $pattern) {
            $content = $content -replace $pattern, '$1$2 {$3$4}'
            $changed = $true
        }
        
        if ($changed) {
            # Write back to file
            Set-Content -Path $filepath -Value $content -NoNewline -Encoding UTF8
            $convertedCount++
            Write-Host "  [CONVERTED] Successfully fixed" -ForegroundColor Green
        } else {
            Write-Host "  [NO CHANGE] No patterns matched" -ForegroundColor Gray
        }
        
    } catch {
        $failedFiles += $filename
        Write-Host "  [ERROR] Failed: $_" -ForegroundColor Red
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "CONVERSION SUMMARY" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Total files to convert: $($filesToFix.Count)"
Write-Host "Successfully converted: $convertedCount" -ForegroundColor Green

if ($failedFiles.Count -gt 0) {
    Write-Host "Failed conversions: $($failedFiles.Count)" -ForegroundColor Red
    foreach ($file in $failedFiles) {
        Write-Host "  - $file" -ForegroundColor Red
    }
}

Write-Host "`nConversion complete!" -ForegroundColor Cyan
