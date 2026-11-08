# Перегонка golden-эталона dual-scan (задача Ф1.1 плана ob2h-интеграции).
#
# Сканирует фиксированный корпус `corpus/` внешним ob2h в изолированном
# OB2H_DATA_DIR (реальные данные ob2h не затрагиваются) и выгружает
# эталонный набор символов/рёбер в `ob2h-<version>.expected.json`.
#
# Запуск (из любой директории):
#   pwsh -File regenerate.ps1
#   pwsh -File regenerate.ps1 -Ob2hVersion 1.9.3
#
# После перегонки — прогнать тест:
#   cargo test -p omnesagent-kag --test test_golden_dual_scan

param(
    [string]$Ob2h = "C:\Projects\ob2h\target\release\ob2h.exe",
    [string]$Python = "C:\Program Files\Python313\python.exe",
    [string]$Ob2hVersion = "1.9.2"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Ob2h)) { throw "ob2h not found: $Ob2h" }
if (-not (Test-Path $Python)) { throw "python not found: $Python" }

$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$corpus = Join-Path $here "corpus"
if (-not (Test-Path $corpus)) { throw "corpus not found: $corpus" }

# Изоляция: ob2h пишет в одноразовый каталог, а не в OB2H_DATA_DIR пользователя.
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("kag-golden-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tmp | Out-Null
$env:OB2H_DATA_DIR = $tmp

Write-Host "OB2H_DATA_DIR=$tmp"
& $Ob2h project init --id kag_golden --name "KAG Golden" --path $corpus
& $Ob2h project scan --id kag_golden --path $corpus --full

$db = Join-Path $tmp "ob2h.db"
if (-not (Test-Path $db)) { throw "ob2h db not created: $db" }

$out = Join-Path $here "ob2h-$Ob2hVersion.expected.json"
& $Python (Join-Path $here "export_ob2h.py") $db "kag_golden" $out $Ob2hVersion

Remove-Item -Recurse -Force $tmp
Write-Host "done: $out"
