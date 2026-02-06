# Build static TALON binary for Windows (MSVC)
# Run this script in PowerShell

$ErrorActionPreference = "Stop"

Write-Host "=== TALON Static Binary Builder (Windows MSVC) ===" -ForegroundColor Cyan
Write-Host ""

# Check if Rust is installed
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust is not installed. Install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check if MSVC toolchain is installed
$msvcTargets = rustup target list | Select-String "x86_64-pc-windows-msvc"
if (-not ($msvcTargets -match "installed")) {
    Write-Host "Installing x86_64-pc-windows-msvc target..." -ForegroundColor Yellow
    rustup target add x86_64-pc-windows-msvc
}

# Optional: Install x86 target
$response = Read-Host "Install 32-bit (i686) target? (y/N)"
if ($response -eq "y" -or $response -eq "Y") {
    rustup target add i686-pc-windows-msvc
}

# Check for protoc (required for some dependencies)
if (-not (Get-Command protoc -ErrorAction SilentlyContinue)) {
    Write-Host "Warning: protoc not found. Install via: choco install protoc" -ForegroundColor Yellow
    $continue = Read-Host "Continue without protoc? (y/N)"
    if ($continue -ne "y" -and $continue -ne "Y") {
        exit 1
    }
}

# Build x64 MSVC binary
Write-Host ""
Write-Host "Building x86_64 MSVC binary (static CRT)..." -ForegroundColor Green
cargo build --release --target x86_64-pc-windows-msvc

# Check if build succeeded
if (-not (Test-Path "target\x86_64-pc-windows-msvc\release\talon.exe")) {
    Write-Host "Error: Build failed" -ForegroundColor Red
    exit 1
}

# Get binary size
$binaryPath = "target\x86_64-pc-windows-msvc\release\talon.exe"
$size = (Get-Item $binaryPath).Length
$sizeMB = [math]::Round($size / 1MB, 2)

Write-Host ""
Write-Host "=== Build Complete ===" -ForegroundColor Green
Write-Host "Binary: $binaryPath"
Write-Host "Size: ${sizeMB}MB"

if ($sizeMB -gt 50) {
    Write-Host "Warning: Binary exceeds 50MB target (${sizeMB}MB)" -ForegroundColor Yellow
}

# Check dependencies with dumpbin (if available)
$dumpbin = Get-Command dumpbin -ErrorAction SilentlyContinue
if ($dumpbin) {
    Write-Host ""
    Write-Host "=== Dependency Check ===" -ForegroundColor Green
    & dumpbin /DEPENDENTS $binaryPath | Select-String -Pattern "\.dll"
} else {
    Write-Host ""
    Write-Host "Note: dumpbin not found (install Visual Studio for dependency analysis)" -ForegroundColor Yellow
}

# Test execution
Write-Host ""
Write-Host "=== Testing binary ===" -ForegroundColor Green
& $binaryPath --version

# Generate checksum
Write-Host ""
Write-Host "=== Generating checksum ===" -ForegroundColor Green
$hash = Get-FileHash -Path $binaryPath -Algorithm SHA256
$checksumPath = "$binaryPath.sha256"
"$($hash.Hash.ToLower())  talon.exe" | Out-File -FilePath $checksumPath -Encoding ascii
Write-Host "SHA256: $($hash.Hash.ToLower())"
Write-Host "Checksum saved to: $checksumPath"

Write-Host ""
Write-Host "Build successful! Static binary ready for distribution." -ForegroundColor Green
