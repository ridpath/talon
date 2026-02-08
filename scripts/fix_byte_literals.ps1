# Fix Python-style byte literals b"..." to TALON syntax

$files = @(
    "memory_scrubbing.talon"
)

$examplesDir = "examples"

Write-Host "`nFixing byte literal syntax..." -ForegroundColor Cyan

foreach ($filename in $files) {
    $filepath = Join-Path $examplesDir $filename
    
    if (-not (Test-Path $filepath)) {
        continue
    }
    
    $content = Get-Content $filepath -Raw -Encoding UTF8
    $originalContent = $content
    
    # Convert b"string" to bytes("string")
    # This will handle b"A", b"$ ", etc.
    $content = $content -replace 'b"([^"\\]*)"', 'bytes("$1")'
    
    # Convert b"\xHH\xHH..." hex sequences to 0xHHHH...
    # This is more complex - need to extract and convert hex escapes
    $pattern = 'b"((?:\\x[0-9a-fA-F]{2})+)"'
    $matches = [regex]::Matches($content, $pattern)
    foreach ($match in $matches) {
        $hexString = $match.Groups[1].Value
        # Extract just the hex digits
        $hexDigits = $hexString -replace '\\x', ''
        $replacement = "0x$hexDigits"
        $content = $content -replace [regex]::Escape($match.Value), $replacement
    }
    
    if ($content -ne $originalContent) {
        Set-Content -Path $filepath -Value $content -NoNewline -Encoding UTF8
        Write-Host "  [FIXED] $filename" -ForegroundColor Green
    } else {
        Write-Host "  [SKIP] $filename - no byte literals found" -ForegroundColor Gray
    }
}

Write-Host "`nByte literal conversion complete!" -ForegroundColor Cyan
