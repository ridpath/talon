# Remove UTF-8 BOM (EF BB BF) from all .talon files using byte operations
$ExampleDir = ".\examples"
$files = Get-ChildItem -Path $ExampleDir -Filter "*.talon" -Recurse

$fixedCount = 0
$bom = [byte[]](0xEF, 0xBB, 0xBF)

foreach ($file in $files) {
    $bytes = [System.IO.File]::ReadAllBytes($file.FullName)
    
    # Check if file starts with UTF-8 BOM
    if ($bytes.Length -ge 3 -and 
        $bytes[0] -eq 0xEF -and 
        $bytes[1] -eq 0xBB -and 
        $bytes[2] -eq 0xBF) {
        
        Write-Host "Fixing BOM in: $($file.Name)" -ForegroundColor Yellow
        
        # Remove first 3 bytes (BOM) and write back
        $newBytes = $bytes[3..($bytes.Length-1)]
        [System.IO.File]::WriteAllBytes($file.FullName, $newBytes)
        
        $fixedCount++
    }
}

Write-Host "`nFixed $fixedCount files with BOM issues" -ForegroundColor Green
