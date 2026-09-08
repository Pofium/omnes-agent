<#
.SYNOPSIS
    Unified Build Script for Omnes Agent (Backend + Frontend).
.DESCRIPTION
    Builds the Rust backend (omnesagent), the Flutter frontend (APK/app), or both in one command.
.PARAMETER Target
    Build target: All (default), Backend, Frontend.
.PARAMETER Mode
    Build mode: Debug (default, fastest) or Release.
.EXAMPLE
    .\build.ps1
    .\build.ps1 -Target Backend
    .\build.ps1 -Target Frontend -Mode Release
#>
[CmdletBinding()]
param(
    [ValidateSet("All", "Backend", "Frontend")]
    [string]$Target = "All",

    [ValidateSet("Debug", "Release")]
    [string]$Mode = "Debug"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = $PSScriptRoot
$BackendDir = Join-Path $ProjectRoot "backend"
$FrontendDir = Join-Path $ProjectRoot "frontend"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Omnes Agent - Unified Build Orchestrator" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Project Root: $ProjectRoot" -ForegroundColor DarkGray
Write-Host "Target:       $Target" -ForegroundColor DarkGray
Write-Host "Mode:         $Mode" -ForegroundColor DarkGray
Write-Host ""

$totalStopwatch = [System.Diagnostics.Stopwatch]::StartNew()

# -------------------------------------------------------------
# 1. Build Backend
# -------------------------------------------------------------
if ($Target -eq "All" -or $Target -eq "Backend") {
    Write-Host "[1/2] Building Backend (Rust omnesagent)..." -ForegroundColor Yellow

    $cargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargoCmd) {
        Write-Error "Rust toolchain (cargo) not found in PATH! Make sure Rust is installed."
        exit 1
    }

    Push-Location $BackendDir
    try {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        if ($Mode -eq "Release") {
            cargo build --release -p omnesagent
        } else {
            cargo build -p omnesagent
        }
        $sw.Stop()
        $sec = [math]::Round($sw.Elapsed.TotalSeconds, 1)

        $binPath = if ($Mode -eq "Release") {
            Join-Path $BackendDir "target\release\omnesagent.exe"
        } else {
            Join-Path $BackendDir "target\debug\omnesagent.exe"
        }

        if (Test-Path $binPath) {
            $f = Get-Item $binPath
            $size = [math]::Round($f.Length / 1MB, 2)
            Write-Host "  [OK] Backend built successfully in $sec s!" -ForegroundColor Green
            Write-Host "       Binary: $binPath ($size MB)" -ForegroundColor DarkGray
        } else {
            Write-Warning "Backend build succeeded but binary was not found at: $binPath"
        }
    }
    finally {
        Pop-Location
    }
    Write-Host ""
}

# -------------------------------------------------------------
# 2. Build Frontend
# -------------------------------------------------------------
if ($Target -eq "All" -or $Target -eq "Frontend") {
    Write-Host "[2/2] Building Frontend (Flutter Windows Desktop)..." -ForegroundColor Yellow

    $flutterCmd = Get-Command flutter -ErrorAction SilentlyContinue
    if (-not $flutterCmd) {
        Write-Error "Flutter SDK not found in PATH! Make sure Flutter is installed."
        exit 1
    }

    $DesktopDir = Join-Path $FrontendDir "desktop"
    Push-Location $DesktopDir
    try {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        if ($Mode -eq "Release") {
            flutter build windows --release
        } else {
            flutter build windows --debug
        }
        $sw.Stop()
        $sec = [math]::Round($sw.Elapsed.TotalSeconds, 1)

        $exePath = if ($Mode -eq "Release") {
            Join-Path $DesktopDir "build\windows\x64\runner\Release\omnes_desktop.exe"
        } else {
            Join-Path $DesktopDir "build\windows\x64\runner\Debug\omnes_desktop.exe"
        }

        if (Test-Path $exePath) {
            $f = Get-Item $exePath
            $size = [math]::Round($f.Length / 1MB, 2)
            Write-Host "  [OK] Frontend Desktop built successfully in $sec s!" -ForegroundColor Green
            Write-Host "       EXE: $exePath ($size MB)" -ForegroundColor DarkGray
        } else {
            Write-Warning "Frontend Desktop build finished. Verify output at $DesktopDir\build\windows"
        }
    }
    finally {
        Pop-Location
    }
    Write-Host ""
}

$totalStopwatch.Stop()
$totalSec = [math]::Round($totalStopwatch.Elapsed.TotalSeconds, 1)
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  BUILD COMPLETE (Total time: $totalSec s) " -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
