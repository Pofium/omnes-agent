<#
.SYNOPSIS
    Health check for Omnes Agent project (static analysis & tests).
.EXAMPLE
    .\scripts\check_health.ps1
#>
$ErrorActionPreference = "Continue"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Omnes Agent - Health Check & Diagnostics" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Directory: $ProjectRoot" -ForegroundColor DarkGray

Push-Location $ProjectRoot
try {
    Write-Host ""
    Write-Host "[1/2] Running static analyzer (flutter analyze --no-pub)..." -ForegroundColor Yellow
    flutter analyze --no-pub
    $analyzeExitCode = $LASTEXITCODE

    Write-Host ""
    Write-Host "[2/2] Running automated test suite (flutter test --no-pub)..." -ForegroundColor Yellow
    flutter test --no-pub
    $testExitCode = $LASTEXITCODE

    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    if ($analyzeExitCode -eq 0 -and $testExitCode -eq 0) {
        Write-Host "  RESULT: ALL CHECKS PASSED (0 ERRORS)!   " -ForegroundColor Green
    } else {
        Write-Host "  RESULT: ISSUES DETECTED!                " -ForegroundColor Red
    }
    Write-Host "==========================================" -ForegroundColor Cyan

    exit ($analyzeExitCode + $testExitCode)
}
finally {
    Pop-Location
}
