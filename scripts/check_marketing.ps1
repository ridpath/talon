# Check for marketing language in src and examples
$marketingPattern = 'world.class|best.in.class|leading|cutting.edge|revolutionary|game.changing|titan'

Write-Host "Checking for marketing language in src/ and examples/..." -ForegroundColor Cyan

$results = git grep -iE $marketingPattern src/ examples/ 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "Marketing language found:" -ForegroundColor Red
    $results | Select-Object -First 30
    Write-Host "`nTotal matches: $($results.Count)" -ForegroundColor Yellow
} else {
    Write-Host "No marketing language found" -ForegroundColor Green
}
