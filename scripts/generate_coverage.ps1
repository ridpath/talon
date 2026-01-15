# TALON Code Coverage Generator (Windows)
# Usage: .\scripts\generate_coverage.ps1 [-Profile <profile>]
# Profiles: quick, comprehensive, ci (default: comprehensive)

[CmdletBinding()]
param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("quick", "comprehensive", "ci")]
    [string]$Profile = "comprehensive"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
Set-Location $ProjectRoot

$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$CoverageDir = Join-Path $ProjectRoot "coverage"
$ReportDir = Join-Path $CoverageDir "reports\$Timestamp"

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "TALON Code Coverage Generator" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Profile: $Profile"
Write-Host "Coverage Directory: $CoverageDir"
Write-Host "Report Directory: $ReportDir"
Write-Host ""

New-Item -ItemType Directory -Force -Path $ReportDir | Out-Null

function Test-TarpaulinInstalled {
    $installed = Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue
    if (-not $installed) {
        Write-Host "⚠️  cargo-tarpaulin not found. Installing..." -ForegroundColor Yellow
        cargo install cargo-tarpaulin
    } else {
        Write-Host "✅ cargo-tarpaulin found" -ForegroundColor Green
    }
}

function Invoke-CoverageGeneration {
    Write-Host ""
    Write-Host "Running coverage with profile: $Profile" -ForegroundColor Cyan
    Write-Host "==================================================" -ForegroundColor Cyan
    
    $StartTime = Get-Date
    
    try {
        switch ($Profile) {
            "quick" {
                cargo tarpaulin `
                    --out Stdout `
                    --out Html `
                    --output-dir $ReportDir `
                    --timeout 60 `
                    --verbose
            }
            
            "comprehensive" {
                cargo tarpaulin `
                    --out Html `
                    --out Xml `
                    --out Lcov `
                    --out Json `
                    --output-dir $ReportDir `
                    --all-features `
                    --workspace `
                    --timeout 300 `
                    --run-types Tests,Doctests `
                    --verbose
            }
            
            "ci" {
                cargo tarpaulin `
                    --out Xml `
                    --output-dir $ReportDir `
                    --all-features `
                    --workspace `
                    --timeout 300 `
                    --run-types Tests,Doctests `
                    --fail-under 80 `
                    --verbose
            }
            
            default {
                throw "Unknown profile: $Profile"
            }
        }
    } catch {
        Write-Host "❌ Coverage generation failed: $_" -ForegroundColor Red
        exit 1
    }
    
    $EndTime = Get-Date
    $Duration = ($EndTime - $StartTime).TotalSeconds
    
    Write-Host ""
    Write-Host "Coverage generation completed in $([math]::Round($Duration, 2))s" -ForegroundColor Green
}

function Show-CoverageSummary {
    Write-Host ""
    Write-Host "==================================================" -ForegroundColor Cyan
    Write-Host "Coverage Summary" -ForegroundColor Cyan
    Write-Host "==================================================" -ForegroundColor Cyan
    
    $coberturaPath = Join-Path $ReportDir "cobertura.xml"
    if (Test-Path $coberturaPath) {
        [xml]$cobertura = Get-Content $coberturaPath
        $lineRate = [double]$cobertura.coverage.'line-rate'
        $coveragePercent = [math]::Round($lineRate * 100, 2)
        
        Write-Host "Line Coverage: $coveragePercent%" -ForegroundColor Cyan
        
        if ($coveragePercent -ge 80) {
            Write-Host "✅ Coverage meets target (≥80%)" -ForegroundColor Green
        } else {
            Write-Host "⚠️  Coverage below target (<80%)" -ForegroundColor Yellow
        }
    }
    
    Write-Host ""
    
    $htmlReport = Join-Path $ReportDir "tarpaulin-report.html"
    if (Test-Path $htmlReport) {
        Write-Host "HTML Report: $htmlReport"
    }
    
    $xmlReport = Join-Path $ReportDir "cobertura.xml"
    if (Test-Path $xmlReport) {
        Write-Host "XML Report: $xmlReport"
    }
    
    $lcovReport = Join-Path $ReportDir "lcov.info"
    if (Test-Path $lcovReport) {
        Write-Host "LCOV Report: $lcovReport"
    }
    
    $jsonReport = Join-Path $ReportDir "tarpaulin-report.json"
    if (Test-Path $jsonReport) {
        Write-Host "JSON Report: $jsonReport"
    }
}

function Set-LatestSymlink {
    $latestLink = Join-Path $CoverageDir "reports\latest"
    
    if (Test-Path $latestLink) {
        Remove-Item $latestLink -Force -Recurse
    }
    
    # Create junction point on Windows
    cmd /c mklink /J "$latestLink" "$ReportDir" | Out-Null
    
    Write-Host ""
    Write-Host "Latest report linked to: $latestLink"
}

function Open-Report {
    if ($Profile -ne "ci") {
        Write-Host ""
        $response = Read-Host "Open HTML report in browser? (y/N)"
        if ($response -match "^[Yy]$") {
            $htmlReport = Join-Path $ReportDir "tarpaulin-report.html"
            if (Test-Path $htmlReport) {
                Start-Process $htmlReport
            }
        }
    }
}

# Main execution
try {
    Test-TarpaulinInstalled
    Invoke-CoverageGeneration
    Show-CoverageSummary
    Set-LatestSymlink
    Open-Report
    
    Write-Host ""
    Write-Host "==================================================" -ForegroundColor Cyan
    Write-Host "Coverage generation complete!" -ForegroundColor Cyan
    Write-Host "==================================================" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Error: $_" -ForegroundColor Red
    exit 1
}
