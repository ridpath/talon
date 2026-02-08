# Check for emoticons in the codebase
$emoticonPattern = '[\x{1F600}-\x{1F64F}\x{1F300}-\x{1F5FF}\x{1F680}-\x{1F6FF}\x{1F1E0}-\x{1F1FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}]'

Write-Host "Checking for emoticons..." -ForegroundColor Cyan

# Search in all files
$results = git grep -P $emoticonPattern . 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "Emoticons found:" -ForegroundColor Red
    $results | Select-Object -First 20
    Write-Host "`nTotal matches: $($results.Count)" -ForegroundColor Yellow
} else {
    Write-Host "No emoticons found" -ForegroundColor Green
}
