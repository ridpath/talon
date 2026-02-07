# Remove emoticons from example files

$files = @(
    'examples\beginner_ctf_template.talon',
    'examples\natural_language_examples.talon',
    'examples\tutorial_01_basics.talon',
    'examples\tutorial_02_exploitation.talon',
    'examples\tutorial_03_web_exploitation.talon',
    'examples\tutorial_04_ctf_toolkit.talon'
)

$rootDir = Split-Path -Parent $PSScriptRoot

foreach ($file in $files) {
    $fullPath = Join-Path $rootDir $file
    if (Test-Path $fullPath) {
        Write-Host "Processing: $file"
        
        $content = Get-Content $fullPath -Raw -Encoding UTF8
        
        # Remove any remaining UTF-8 emoticon characters (4-byte sequences starting with 0xF0)
        # This will remove most emoji
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($content)
        $cleanBytes = New-Object System.Collections.Generic.List[byte]
        
        for ($i = 0; $i -lt $bytes.Length; $i++) {
            # Skip 4-byte UTF-8 sequences (most emoji)
            if ($bytes[$i] -eq 0xF0 -and $i + 3 -lt $bytes.Length) {
                $i += 3  # Skip the next 3 bytes
                continue
            }
            # Skip some 3-byte UTF-8 sequences (checkmarks, etc.)
            if ($bytes[$i] -eq 0xE2 -and $i + 2 -lt $bytes.Length -and 
                $bytes[$i+1] -in @(0x9C, 0x98, 0x9D, 0x9A, 0x9B, 0x9E)) {
                $i += 2  # Skip the next 2 bytes
                continue
            }
            
            $cleanBytes.Add($bytes[$i])
        }
        
        $cleanContent = [System.Text.Encoding]::UTF8.GetString($cleanBytes.ToArray())
        
        # Write back to file
        $cleanContent | Set-Content $fullPath -Encoding UTF8 -NoNewline
        Write-Host "  Cleaned: $file"
    }
}

Write-Host "`nDone!"
