# Analyze example failures to categorize issues
$TalonBin = ".\target\debug\talon.exe"
$ExampleDir = ".\examples"

$examples = Get-ChildItem -Path $ExampleDir -Filter "*.talon"

$categories = @{
    "BOM" = @()
    "ColonInArgs" = @()
    "UndefinedVar" = @()
    "UnknownMethod" = @()
    "Other" = @()
    "Passing" = @()
}

foreach ($example in $examples) {
    $output = & $TalonBin run $example.FullName --dry-run 2>&1 | Out-String
    
    if ($LASTEXITCODE -eq 0) {
        $categories["Passing"] += $example.Name
    }
    elseif ($output -match "expected program") {
        $categories["BOM"] += $example.Name
    }
    elseif ($output -match "expected.*comp_op.*ident" -and $output -match '":') {
        $categories["ColonInArgs"] += $example.Name
    }
    elseif ($output -match "UNDEFINED VARIABLE") {
        $categories["UndefinedVar"] += $example.Name
    }
    elseif ($output -match "Unknown method") {
        $categories["UnknownMethod"] += $example.Name
    }
    else {
        $categories["Other"] += $example.Name
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "EXAMPLE FAILURE ANALYSIS" -ForegroundColor Cyan
Write-Host "========================================`n"

foreach ($category in $categories.Keys | Sort-Object) {
    $count = $categories[$category].Count
    Write-Host "$category : $count examples" -ForegroundColor Yellow
    if ($count -gt 0 -and $count -le 10) {
        foreach ($file in $categories[$category]) {
            Write-Host "  - $file" -ForegroundColor Gray
        }
    }
}

Write-Host "`n========================================`n"
