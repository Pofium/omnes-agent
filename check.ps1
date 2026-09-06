<#
.SYNOPSIS
    Full Stack Health Check for Omnes Agent (Backend + Frontend).
.EXAMPLE
    .\check.ps1
#>
$ErrorActionPreference = "Continue"
$ProjectRoot = $PSScriptRoot
$BackendDir = Join-Path $ProjectRoot "backend"
$FrontendDir = Join-Path $ProjectRoot "frontend"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Omnes Agent - Full Stack Health Check   " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# 1. Backend Check
Write-Host "[1/3] Backend: cargo check (omnesagent-gateway)..." -ForegroundColor Yellow
Push-Location $BackendDir
cargo check -p omnesagent-gateway
$backExitCode = $LASTEXITCODE
Pop-Location

# 2. Frontend Analyzer
Write-Host ""
Write-Host "[2/3] Frontend: flutter analyze..." -ForegroundColor Yellow
Push-Location $FrontendDir
flutter analyze --no-pub
$frontAnalyzeCode = $LASTEXITCODE
Pop-Location

# 3. Frontend Tests
Write-Host ""
Write-Host "[3/3] Frontend: flutter test..." -ForegroundColor Yellow
Push-Location $FrontendDir
flutter test --no-pub
$frontTestCode = $LASTEXITCODE
Pop-Location

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
$totalErrors = $backExitCode + $frontAnalyzeCode + $frontTestCode
if ($totalErrors -eq 0) {
    Write-Host "  RESULT: ALL SYSTEMS HEALTHY (0 ISSUES)! " -ForegroundColor Green
} else {
    Write-Host "  RESULT: ISSUES DETECTED!                " -ForegroundColor Red
}
Write-Host "==========================================" -ForegroundColor Cyan

exit $totalErrors
