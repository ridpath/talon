# Remove UTF-8 BOM from all .talon files
$ExampleDir = ".\examples"
$files = Get-ChildItem -Path $ExampleDir -Filter "*.talon" -Recurse

$fixedCount = 0

foreach ($file in $files) {
    $content = Get-Content -Path $file.FullName -Raw -Encoding UTF8
    
    # Check for BOM
    if ($content[0] -eq [char]0xFEFF) {
        Write-Host "Fixing BOM in: $($file.Name)" -ForegroundColor Yellow
        
        # Remove BOM and save as UTF-8 without BOM
        $content = $content.TrimStart([char]0xFEFF)
        [System.IO.File]::WriteAllText($file.FullName, $content, (New-Object System.Text.UTF8Encoding $false))
        
        $fixedCount++
    }
}

Write-Host "`nFixed $fixedCount files with BOM issues" -ForegroundColor Green
