# Fix function call kwargs: change "param: value" to "param=value"
# This only affects function calls, not map literals in {}

$ExampleDir = ".\examples"
$files = Get-ChildItem -Path $ExampleDir -Filter "*.talon"

$fixedFiles = 0
$totalReplacements = 0

foreach ($file in $files) {
    $content = Get-Content -Path $file.FullName -Raw
    $originalContent = $content
    
    # Pattern: function_name(..., identifier: value, ...)
    # We need to be careful not to match map literals {}
    # This regex looks for function calls with colon syntax
    
    # Match: word(... word: ... where not inside {}
    $pattern = '(\w+)\s*\(\s*([^{]*?)\s*(\w+)\s*:\s*([^,\)]+)'
    
    $replacements = 0
    while ($content -match $pattern) {
        # Check if this is inside braces (map literal)
        $beforeMatch = $content.Substring(0, $matches.Index)
        $openBraces = ($beforeMatch.ToCharArray() | Where-Object { $_ -eq '{' }).Count
        $closeBraces = ($beforeMatch.ToCharArray() | Where-Object { $_ -eq '}' }).Count
        
        # If braces are balanced, we're NOT in a map literal
        if ($openBraces -eq $closeBraces) {
            # Replace : with =
            $matchText = $matches[0]
            $replacement = $matchText -replace '(\w+)\s*:\s*', '$1='
            $content = $content -replace [regex]::Escape($matchText), $replacement
            $replacements++
        } else {
            # Skip this match (it's in a map literal)
            break
        }
        
        # Prevent infinite loop
        if ($replacements > 100) { break }
    }
    
    if ($content -ne $originalContent) {
        Write-Host "Fixed $replacements kwargs in: $($file.Name)" -ForegroundColor Yellow
        [System.IO.File]::WriteAllText($file.FullName, $content, [System.Text.UTF8Encoding]::new($false))
        $fixedFiles++
        $totalReplacements += $replacements
    }
}

Write-Host "`nFixed $totalReplacements kwargs in $fixedFiles files" -ForegroundColor Green
