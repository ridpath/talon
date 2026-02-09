# Comprehensive syntax fixer for all TALON examples
# Handles: if/else blocks, try/catch blocks, function definitions, for loops

$ErrorActionPreference = "Stop"

function Fix-TalonSyntax {
    param([string]$FilePath)
    
    $content = Get-Content $FilePath -Raw
    $original = $content
    
    # Step 1: Fix try/catch/end blocks to try { } catch e { }
    # Pattern: try\n...statements...\ncatch e\n...statements...\nend
    while ($content -match '(\s+)try\s*\r?\n(.*?)\s*catch\s+(\w+)\s*\r?\n(.*?)\s*end') {
        $indent = $matches[1]
        $tryBody = $matches[2]
        $catchVar = $matches[3]
        $catchBody = $matches[4]
        
        $newBlock = "${indent}try {`r`n${tryBody}${indent}} catch $catchVar {`r`n${catchBody}${indent}}"
        $content = $content -replace [regex]::Escape($matches[0]), $newBlock
    }
    
    # Step 2: Fix function definitions with code on same line
    # Pattern: define function name(...) {    code
    $content = $content -replace '(define\s+function\s+\w+\([^)]*\)\s*\{)\s*([^\r\n]+)', "`$1`r`n    `$2"
    
    # Step 3: Fix if statements with extra braces and malformed syntax
    # Pattern: if condition {    assignment {    statement
    # This is tricky - let's handle specific patterns
    
    # Pattern 3a: if condition {    let x = y {    statement
    $content = $content -replace '(if\s+[^{]+\{)\s+([^{}\r\n]+)\s+\{([^\r\n]+)', "`$1`r`n    `$2`r`n    `$3"
    
    # Pattern 3b: if condition {    statement with extra {
    $content = $content -replace '(if\s+[^{]+\{)([^{}]+)\{', "`$1`r`n    `$2"
    
    # Step 4: Fix for loops with blocks
    $content = $content -replace '(for\s+\w+\s+in\s+[^{]+)\{', "`$1 {"
    
    # Step 5: Clean up } else { on same line
    $content = $content -replace '}\s+else\s+\{', "}`r`n} else {"
    
    # Step 6: Fix malformed nested braces {{ or }}
    $content = $content -replace '\{\{', '{'
    $content = $content -replace '}}', '}'
    
    # Step 7: Fix if statements without opening brace
    # Pattern: if condition\n    statements
    # This is risky, so be careful
    
    # Step 8: Remove trailing 'end' keywords that shouldn't be there
    $content = $content -replace '\s+end\s*\r?\n', "`r`n"
    
    return $content -ne $original, $content
}

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

Write-Host "Fixing SYNTAX_ERROR examples (comprehensive)..." -ForegroundColor Cyan
Write-Host ""

$fixed = 0
$noChanges = 0
$failed = 0

foreach ($file in $syntaxErrorFiles) {
    $path = "examples\$file"
    
    if (-not (Test-Path $path)) {
        Write-Host "  [!] Not found: $file" -ForegroundColor Yellow
        $failed++
        continue
    }
    
    try {
        $changed, $newContent = Fix-TalonSyntax -FilePath $path
        
        if ($changed) {
            Set-Content -Path $path -Value $newContent -NoNewline
            Write-Host "  [+] Fixed: $file" -ForegroundColor Green
            $fixed++
        } else {
            Write-Host "  [-] No changes: $file" -ForegroundColor Gray
            $noChanges++
        }
    }
    catch {
        Write-Host "  [!] Error in $file : $_" -ForegroundColor Red
        $failed++
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Fixed: $fixed files" -ForegroundColor Green
Write-Host "No changes: $noChanges files" -ForegroundColor Gray
Write-Host "Failed: $failed files" -ForegroundColor $(if ($failed -gt 0) { "Red" } else { "Gray" })
Write-Host "========================================" -ForegroundColor Cyan
