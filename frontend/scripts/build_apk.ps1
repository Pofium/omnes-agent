<#
.SYNOPSIS
    Fast Android APK Build for Omnes Agent.
.PARAMETER Mode
    Build mode: Debug (default, fastest) or Release.
.EXAMPLE
    .\scripts\build_apk.ps1
    .\scripts\build_apk.ps1 -Mode Release
#>
[CmdletBinding()]
param(
    [ValidateSet("Debug", "Release")]
    [string]$Mode = "Debug"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Omnes Agent - Android APK Build ($Mode) " -ForegroundColor Cyan
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
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

    Write-Host "[1/2] Running flutter build apk ($Mode)..." -ForegroundColor Yellow
    if ($Mode -eq "Release") {
        flutter build apk --release
    } else {
        flutter build apk --debug
    }

    $stopwatch.Stop()
    $duration = [math]::Round($stopwatch.Elapsed.TotalSeconds, 1)

    $apkPath = if ($Mode -eq "Release") {
        Join-Path $ProjectRoot "build\app\outputs\flutter-apk\app-release.apk"
    } else {
        Join-Path $ProjectRoot "build\app\outputs\flutter-apk\app-debug.apk"
    }

    if (Test-Path $apkPath) {
        $fileItem = Get-Item $apkPath
        $sizeMb = [math]::Round($fileItem.Length / 1MB, 2)
        Write-Host ""
        Write-Host "==========================================" -ForegroundColor Green
        Write-Host "  BUILD COMPLETED SUCCESSFULLY in $duration s!" -ForegroundColor Green
        Write-Host "==========================================" -ForegroundColor Green
        Write-Host "APK Path: $apkPath" -ForegroundColor White
        Write-Host "Size:     $sizeMb MB" -ForegroundColor White
        Write-Host ""
    } else {
        Write-Warning "Build finished, but APK was not found at: $apkPath"
    }
}
finally {
    Pop-Location
}
