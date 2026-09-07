<#
.SYNOPSIS
    Fast App Launcher for Omnes Agent (device, emulator or Windows).
.PARAMETER Device
    Target device ID or name (e.g., windows, chrome, emulator-5554).
.EXAMPLE
    .\scripts\run_app.ps1
    .\scripts\run_app.ps1 -Device windows
#>
[CmdletBinding()]
param(
    [string]$Device = ""
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Omnes Agent - Launch Application        " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Directory: $ProjectRoot" -ForegroundColor DarkGray

# Check Flutter in PATH
$flutterCmd = Get-Command flutter -ErrorAction SilentlyContinue
if (-not $flutterCmd) {
    Write-Error "Flutter SDK not found in PATH! Make sure Flutter is installed and added to environment variables."
    exit 1
}

Push-Location $ProjectRoot
try {
    if ($Device -ne "") {
        Write-Host "Launching on target device: $Device..." -ForegroundColor Yellow
        flutter run -d $Device
    } else {
        Write-Host "Detecting connected devices..." -ForegroundColor Yellow
        flutter devices
        Write-Host ""
        Write-Host "Starting app..." -ForegroundColor Green
        flutter run
    }
}
finally {
    Pop-Location
}
