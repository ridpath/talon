param(
    [Parameter(Mandatory=$true)]
    [string]$ExampleFile
)

$TalonBin = ".\target\debug\talon.exe"

Write-Host "Testing: $ExampleFile" -ForegroundColor Cyan
Write-Host "=" * 60

$output = & $TalonBin run $ExampleFile --dry-run 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Host "FAILED with exit code: $LASTEXITCODE" -ForegroundColor Red
}

# Show output
$output

Write-Host "=" * 60
