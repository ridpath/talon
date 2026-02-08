#!/usr/bin/env pwsh
# Analyze plan.md completion status

$PlanFile = ".\.zenflow\tasks\iamtalon-d954\plan.md"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Plan.md Completion Analysis" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

$content = Get-Content $PlanFile -Raw

# Count total steps
$totalSteps = ([regex]::Matches($content, '### \[.\] Step:')).Count
Write-Host "Total Steps: $totalSteps"

# Count completed steps
$completedSteps = ([regex]::Matches($content, '### \[x\] Step:')).Count
Write-Host "Completed Steps: $completedSteps" -ForegroundColor Green

# Count incomplete steps
$incompleteSteps = ([regex]::Matches($content, '### \[ \] Step:')).Count
Write-Host "Incomplete Steps: $incompleteSteps" -ForegroundColor Yellow

# Calculate percentage
if ($totalSteps -gt 0) {
    $percentage = [math]::Round(($completedSteps / $totalSteps) * 100, 1)
    Write-Host "`nCompletion: $percentage%" -ForegroundColor $(if ($percentage -ge 90) { "Green" } elseif ($percentage -ge 70) { "Yellow" } else { "Red" })
} else {
    Write-Host "`nError: No steps found!" -ForegroundColor Red
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Phase Breakdown" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Count by phase
$phases = @(
    @{Name="Phase 1: Core Performance & Infrastructure"; Pattern="## Phase 1:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 2: User Experience & Discoverability"; Pattern="## Phase 2:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 3: Stub Remediation & Hardening"; Pattern="## Phase 3:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 4: Advanced Exploitation Features"; Pattern="## Phase 4:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 5: Top 10 Module Elevation"; Pattern="## Phase 5:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 5.5: OpSec, Evasion & Forensics"; Pattern="## Phase 5.5:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 6: AI Integration & Distributed Swarm"; Pattern="## Phase 6:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 6.5: Advanced Architecture"; Pattern="## Phase 6.5:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 7: Build, Distribution & Validation"; Pattern="## Phase 7:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 7.5: Interpreter Enhancements"; Pattern="## Phase 7.5:.*?(?=## Phase|## Verification|$)"},
    @{Name="Phase 7.6: Example Validation - Remaining"; Pattern="## Phase 7.6:.*?(?=## Phase|## Verification|$)"}
)

foreach ($phase in $phases) {
    $phaseMatch = [regex]::Match($content, $phase.Pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)
    if ($phaseMatch.Success) {
        $phaseContent = $phaseMatch.Value
        $phaseTotal = ([regex]::Matches($phaseContent, '### \[.\] Step:')).Count
        $phaseComplete = ([regex]::Matches($phaseContent, '### \[x\] Step:')).Count
        
        if ($phaseTotal -gt 0) {
            $phasePercent = [math]::Round(($phaseComplete / $phaseTotal) * 100, 1)
            $status = if ($phasePercent -eq 100) { "[COMPLETE]" } else { "[IN PROGRESS]" }
            Write-Host "$status $($phase.Name): $phaseComplete/$phaseTotal ($phasePercent%)"
        }
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Incomplete Steps List" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Extract incomplete step names
$incompleteMatches = [regex]::Matches($content, '### \[ \] Step: ([^\n]+)')
if ($incompleteMatches.Count -gt 0) {
    foreach ($match in $incompleteMatches) {
        Write-Host "  - $($match.Groups[1].Value)" -ForegroundColor Yellow
    }
} else {
    Write-Host "  All steps complete!" -ForegroundColor Green
}
