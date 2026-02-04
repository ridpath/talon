# TALON Windows Installer

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\talon",
    [string]$BinaryName = "talon.exe",
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

function Write-Header {
    Write-Host ""
    Write-Host "TALON Installer" -ForegroundColor Cyan
    Write-Host "===============" -ForegroundColor Cyan
    Write-Host ""
}

function Get-Architecture {
    $arch = (Get-CimInstance Win32_OperatingSystem).OSArchitecture
    switch -Regex ($arch) {
        "64-bit" { return "x86_64" }
        "32-bit" { return "i686" }
        "ARM64"  { return "aarch64" }
        default  { return "unknown" }
    }
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Install-Talon {
    Write-Header

    $arch = Get-Architecture
    if ($arch -eq "unknown") {
        Write-Host "Error: Unsupported architecture" -ForegroundColor Red
        exit 1
    }

    Write-Host "Detected Architecture: $arch" -ForegroundColor Green
    Write-Host ""

    if (-not (Test-Path $InstallDir)) {
        Write-Host "Creating installation directory: $InstallDir" -ForegroundColor Yellow
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    $binaryPath = "$InstallDir\$BinaryName"

    if (Test-Path ".\target\release\talon.exe") {
        Write-Host "Using locally built binary" -ForegroundColor Green
        Copy-Item ".\target\release\talon.exe" -Destination $binaryPath -Force
    }
    elseif (Test-Path ".\talon.exe") {
        Write-Host "Using local binary" -ForegroundColor Green
        Copy-Item ".\talon.exe" -Destination $binaryPath -Force
    }
    else {
        Write-Host "Error: No binary found. Please build TALON first with 'cargo build --release'" -ForegroundColor Red
        exit 1
    }

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        Write-Host "Adding $InstallDir to user PATH" -ForegroundColor Yellow
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
        $env:Path = "$env:Path;$InstallDir"
    }
    else {
        Write-Host "PATH already contains $InstallDir" -ForegroundColor Green
    }

    $talonDir = "$env:USERPROFILE\.talon"
    if (-not (Test-Path $talonDir)) {
        New-Item -ItemType Directory -Path $talonDir -Force | Out-Null
    }

    Write-Host ""
    Write-Host "Installation complete!" -ForegroundColor Green
    Write-Host ""
    Write-Host "TALON is installed at: $binaryPath" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Run 'talon --help' to get started" -ForegroundColor Yellow
    Write-Host "Run 'talon learn' for an interactive tutorial" -ForegroundColor Yellow
    Write-Host "Run 'talon new' to see available exploit templates" -ForegroundColor Yellow
    Write-Host ""
    
    try {
        & talon --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "TALON is ready to use!" -ForegroundColor Green
        }
    }
    catch {
        Write-Host "Note: You may need to restart your terminal to use the 'talon' command" -ForegroundColor Yellow
    }
}

Install-Talon
