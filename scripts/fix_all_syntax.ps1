# Fix all SYNTAX_ERROR examples by converting if/else blocks to proper curly brace syntax

$ErrorActionPreference = "Stop"

$syntaxErrorFiles = @(
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
    "complete_exploitation_workflow.talon"
)

Write-Host "Fixing SYNTAX_ERROR examples..." -ForegroundColor Cyan

$fixed = 0
$failed = 0

foreach ($file in $syntaxErrorFiles) {
    $path = "examples\$file"
    
    if (-not (Test-Path $path)) {
        Write-Host "  [!] File not found: $file" -ForegroundColor Yellow
        $failed++
        continue
    }
    
    try {
        $content = Get-Content $path -Raw
        $original = $content
        
        # Fix pattern 1: if condition {    statements on same line
        $content = $content -replace '(if\s+[^{]+\{)\s+([^\n]+)(\r?\n)', "`$1`r`n    `$2`$3"
        
        # Fix pattern 2: } else {    statements on same line
        $content = $content -replace '}\s+else\s+\{([^\n]+)', "}`r`nelse {`r`n    `$1`r`n}"
        $content = $content -replace '}\s+else\s+\{', "}`r`nelse {`r`n"
        
        # Fix pattern 3: Remove malformed nested braces like { }
        $content = $content -replace '\{\s+\}', ''
        
        # Fix pattern 4: if condition without opening brace
        $content = $content -replace '(if\s+[^{]+)\r?\n(\s+)', "`$1 {`r`n`$2"
        
        # Check if we made changes
        if ($content -ne $original) {
            Set-Content -Path $path -Value $content -NoNewline
            Write-Host "  [+] Fixed: $file" -ForegroundColor Green
            $fixed++
        } else {
            Write-Host "  [-] No changes: $file" -ForegroundColor Gray
        }
    }
    catch {
        Write-Host "  [!] Error fixing $file : $_" -ForegroundColor Red
        $failed++
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Fixed: $fixed files" -ForegroundColor Green
Write-Host "Failed: $failed files" -ForegroundColor $(if ($failed -gt 0) { "Red" } else { "Gray" })
Write-Host "========================================" -ForegroundColor Cyan
