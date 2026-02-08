$files = @(
    '06_ctf_automation.talon',
    'ctf_blind_rop.talon',
    'ctf_kernel_exploit.talon',
    'ctf_multi_stage_pwn.talon',
    'ctf_one_gadget_pwn.talon',
    'ctf_shellcode_encoder.talon'
)

foreach ($f in $files) {
    $path = "examples\$f"
    if (Test-Path $path) {
        $bytes = [System.IO.File]::ReadAllBytes($path)
        $hasBOM = ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF)
        Write-Output "$f : BOM=$hasBOM"
    } else {
        Write-Output "$f : FILE NOT FOUND"
    }
}
