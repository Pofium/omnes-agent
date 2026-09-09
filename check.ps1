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

# 2. Frontend Analyzer (Desktop, Shared & Web)
Write-Host ""
Write-Host "[2/3] Frontend: flutter analyze (desktop, shared & web)..." -ForegroundColor Yellow
$DesktopDir = Join-Path $FrontendDir "desktop"
$SharedDir = Join-Path $FrontendDir "shared"
$WebDir = Join-Path $FrontendDir "web"

Push-Location $DesktopDir
flutter analyze --no-pub
$frontAnalyzeDesktop = $LASTEXITCODE
Pop-Location

Push-Location $SharedDir
flutter analyze --no-pub
$frontAnalyzeShared = $LASTEXITCODE
Pop-Location

$frontAnalyzeWeb = 0
if (Test-Path $WebDir) {
    Push-Location $WebDir
    flutter analyze --no-pub
    $frontAnalyzeWeb = $LASTEXITCODE
    Pop-Location
}

$frontAnalyzeCode = $frontAnalyzeDesktop + $frontAnalyzeShared + $frontAnalyzeWeb

# 3. Web Platform-Isolation Check (no dart:io, no webview_windows)
Write-Host ""
Write-Host "[3/4] Frontend: Web platform purity assertions..." -ForegroundColor Yellow
$driftErrors = 0
if (Test-Path $WebDir) {
    $dartIoFound = Get-ChildItem -Path (Join-Path $WebDir "lib") -Recurse -Filter "*.dart" | Select-String -Pattern "import\s+['""]dart:io['""]"
    if ($dartIoFound) {
        Write-Host "  ERROR: dart:io detected in frontend/web/lib!" -ForegroundColor Red
        $driftErrors++
    }
    $webviewWindowsFound = Get-ChildItem -Path (Join-Path $WebDir "lib") -Recurse -Filter "*.dart" | Select-String -Pattern "webview_windows"
    if ($webviewWindowsFound) {
        Write-Host "  ERROR: webview_windows detected in frontend/web/lib!" -ForegroundColor Red
        $driftErrors++
    }
    if ($driftErrors -eq 0) {
        Write-Host "  Web platform isolation clean (0 dart:io, 0 webview_windows)." -ForegroundColor Green
    }
}

# 4. Frontend Tests
Write-Host ""
Write-Host "[4/4] Frontend: flutter test (desktop & web)..." -ForegroundColor Yellow
$frontTestCode = 0
if (Test-Path (Join-Path $DesktopDir "test")) {
    Push-Location $DesktopDir
    flutter test --no-pub
    $frontTestCode += $LASTEXITCODE
    Pop-Location
}
if (Test-Path (Join-Path $WebDir "test")) {
    Push-Location $WebDir
    flutter test --no-pub
    $frontTestCode += $LASTEXITCODE
    Pop-Location
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
$totalErrors = $backExitCode + $frontAnalyzeCode + $driftErrors + $frontTestCode
if ($totalErrors -eq 0) {
    Write-Host "  RESULT: ALL SYSTEMS HEALTHY (0 ISSUES)! " -ForegroundColor Green
} else {
    Write-Host "  RESULT: ISSUES DETECTED!                " -ForegroundColor Red
}
Write-Host "==========================================" -ForegroundColor Cyan

exit $totalErrors
