<#
.SYNOPSIS
    Unified App and Gateway Launcher for Omnes Agent.
.PARAMETER Service
    What to launch: All (starts Gateway + Frontend), Backend (Gateway only), Frontend (App only).
.PARAMETER Device
    Flutter device for frontend (e.g. windows, emulator-5554).
.EXAMPLE
    .\run.ps1
    .\run.ps1 -Service Backend
    .\run.ps1 -Service Frontend -Device windows
#>
[CmdletBinding()]
param(
    [ValidateSet("All", "Backend", "Frontend")]
    [string]$Service = "All",

    [string]$Device = ""
)

$ErrorActionPreference = "Stop"
$ProjectRoot = $PSScriptRoot
$BackendDir = Join-Path $ProjectRoot "backend"
$FrontendDir = Join-Path $ProjectRoot "frontend"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Omnes Agent - Unified Service Launcher  " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Service: $Service" -ForegroundColor DarkGray
Write-Host ""

# -------------------------------------------------------------
# 1. Start Backend Gateway
# -------------------------------------------------------------
if ($Service -eq "All" -or $Service -eq "Backend") {
    Write-Host "[1] Checking/Starting Omnes Gateway Daemon..." -ForegroundColor Yellow

    # Locate binary (debug or release)
    $releaseBin = Join-Path $BackendDir "target\release\omnesagent.exe"
    $debugBin = Join-Path $BackendDir "target\debug\omnesagent.exe"
    $binPath = if (Test-Path $releaseBin) { $releaseBin } elseif (Test-Path $debugBin) { $debugBin } else { $null }

    if (-not $binPath) {
        Write-Host "  Backend binary not found. Building debug binary first..." -ForegroundColor Yellow
        Push-Location $BackendDir
        cargo build -p omnesagent
        Pop-Location
        $binPath = $debugBin
    }

    # Test if gateway is already running
    $isRunning = $false
    try {
        $res = Invoke-RestMethod -Uri "http://127.0.0.1:42617/health" -TimeoutSec 2 -ErrorAction SilentlyContinue
        if ($res.status -eq "ok") {
            $isRunning = $true
        }
    } catch {}

    if ($isRunning) {
        Write-Host "  [OK] Gateway is already running at http://127.0.0.1:42617" -ForegroundColor Green
    } else {
        Write-Host "  Starting Gateway in background: $binPath gateway start..." -ForegroundColor Cyan
        Start-Process -FilePath $binPath -ArgumentList "gateway", "start" -WorkingDirectory $BackendDir -WindowStyle Hidden
        Start-Sleep -Seconds 2
        Write-Host "  [OK] Gateway started at http://127.0.0.1:42617" -ForegroundColor Green
    }
    Write-Host ""
}

# -------------------------------------------------------------
# 2. Start Frontend App
# -------------------------------------------------------------
if ($Service -eq "All" -or $Service -eq "Frontend") {
    Write-Host "[2] Launching Flutter Frontend Client..." -ForegroundColor Yellow
    Push-Location $FrontendDir
    try {
        if ($Device -ne "") {
            flutter run -d $Device
        } else {
            flutter run
        }
    }
    finally {
        Pop-Location
    }
}
