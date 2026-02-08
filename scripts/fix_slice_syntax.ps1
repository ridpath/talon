# Fix Python-style slice syntax [start:end] to Rust-style [start..end]

$files = @(
    "exploit_chain_buffer_overflow.talon",
    "exploit_chain_format_string.talon",
    "memory_scrubbing.talon",
    "orchestrator_graph.talon",
    "orchestrator_parallel.talon",
    "orchestrator_resilient.talon",
    "orchestrator_timetravel.talon",
    "phase21_meta_programming.talon",
    "phase22_demo.talon",
    "phase22_symbiotic_execution.talon",
    "polymorphic_shellcode.talon",
    "time_travel_debugging.talon"
)

$examplesDir = "examples"
$fixedCount = 0

Write-Host "`nFixing slice syntax..." -ForegroundColor Cyan

foreach ($filename in $files) {
    $filepath = Join-Path $examplesDir $filename
    
    if (-not (Test-Path $filepath)) {
        continue
    }
    
    $content = Get-Content $filepath -Raw -Encoding UTF8
    $originalContent = $content
    
    # Replace Python-style slicing [start:end] with Rust-style [start..end]
    # Only replace in array indexing context (not in map literals or function args)
    
    # Pattern: variable[number:number] -> variable[number..number]
    $content = $content -replace '(\w+)\[(\d+):(\d+)\]', '$1[$2..$3]'
    
    # Pattern: expression[number:number] -> expression[number..number]
    $content = $content -replace '(\))\[(\d+):(\d+)\]', '$1[$2..$3]'
    
    # Pattern: variable[var:number] or [number:var]
    $content = $content -replace '(\w+)\[(\w+):(\d+)\]', '$1[$2..$3]'
    $content = $content -replace '(\w+)\[(\d+):(\w+)\]', '$1[$2..$3]'
    
    if ($content -ne $originalContent) {
        Set-Content -Path $filepath -Value $content -NoNewline -Encoding UTF8
        $fixedCount++
        Write-Host "  [FIXED] $filename" -ForegroundColor Green
    } else {
        Write-Host "  [SKIP] $filename - no slicing found" -ForegroundColor Gray
    }
}

Write-Host "`nFixed $fixedCount files" -ForegroundColor Cyan
