# План 1 — Установщик Windows для десктопной части (OmnesAgent-Setup-x64.exe)

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Один `install.exe` (`OmnesAgent-Setup-<ver>-x64.exe`): классическая установка «как у крупных вендоров» — Program Files, ярлыки Пуск/Рабочий стол, запись в «Установка и удаление программ», деинсталляция; внутри — и Flutter-клиент, и Rust-шлюз. Пользователю НЕ нужны Rust, Flutter или что-либо ещё — всё из коробки (как Adobe Photoshop).

**Architecture:** Сборочный скрипт `scripts/package_windows.ps1` делает: `cargo build --release -p omnesagent` → `flutter build windows --release` → staging-каталог (клиент + `bin\omnesagent.exe`) → компиляция Inno Setup-скрипта `installer/windows/omnesagent.iss` через `iscc.exe`. Установщик кладёт приложение в `{autopf}\OmnesAgent`, лэйаут совпадает с тем, что уже умеет искать `DesktopBackendManager`.

**Tech Stack:** Rust (cargo), Flutter Windows (release), Inno Setup 6 (ISCC), PowerShell 5+/7.

---

## Контекст (проверено в репо, 2026-09-11)

| Факт | Где |
|---|---|
| Десктоп-клиент: Flutter, артефакты `build\windows\x64\runner\Release\` — `OmnesAgent.exe`, `flutter_windows.dll`, `webview_windows_plugin.dll`, `WebView2Loader.dll`, `data\` | `frontend/desktop/` |
| Бэкенд-бинарник: `backend\target\release\omnesagent.exe` (`cargo build --release -p omnesagent`) | `build.ps1` |
| В Release-каталоге уже вручную лежит `omnesagent-backend.exe` — пакетный скрипт заменяет ручной приём на воспроизводимый | `frontend/desktop/build/windows/x64/runner/Release/` |
| `DesktopBackendManager.findOmnesAgentBinary()` ищет шлюз: `exeDir\backend\omnesagent.exe`, затем `exeDir\bin\omnesagent.exe` → **берём лэйаут `bin\omnesagent.exe`** | `frontend/desktop/lib/utils/desktop_backend_manager.dart:40-46` |
| Клиент сам стартует шлюз на `127.0.0.1:42617` при запуске и гасит при выходе | `desktop_backend_manager.dart` |
| Шлюз health: клиент пингует `http://127.0.0.1:42617/health`; VPS-скрипт использует `/api/health` — при тестах проверить оба, какой реально отвечает | `desktop_backend_manager.dart:19`, `scripts/vps_install.sh:322` |
| Иконка приложения: `frontend/desktop/assets/app_icon.ico` (untracked; трекается в Плане 0, Task 2) | `frontend/desktop/assets/` |
| Версия источника: `[workspace.package] version` в `backend/Cargo.toml` (после Плана 0 = `0.1.0`) | `backend/Cargo.toml` |
| Плагин `webview_windows` требует **WebView2 Runtime** (Win11 есть из коробки; Win10 — обычно есть через Edge; нужен детект+установка) | pubspec-плагин в билде |
| Шлюз слушает loopback — firewall-правило не требуется (проверить в Task 6) | config по умолчанию |
| Данные шлюза: `~/.omnesagent/` (сессии SQLite и т.д.) — деинсталлятор НЕ удаляет | `PLAN.md` (данные сессий) |

---

## Task 1: Пререквизиты — Inno Setup

**Objective:** `iscc.exe` доступен для компиляции установщика.

**Step 1:**
```powershell
winget install --id JRSoftware.InnoSetup -e --accept-source-agreements --accept-package-agreements
```
Expected: Inno Setup 6 установлен в `C:\Program Files (x86)\Inno Setup 6\`.
**Step 2:** Проверка: `& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" /?` → справка компилятора. Если winget недоступен — скачать `innosetup-6.x.x.exe` с jrsoftware.org и установить тихо (`/VERYSILENT /NORESTART`); НЕ сдаваться после первой неудачи (альтернативы: direct download, choco install innosetup).

**Verify:** файл `ISCC.exe` существует по пути выше.

---

## Task 2: Packaging-скрипт `scripts/package_windows.ps1`

**Objective:** Одна команда собирает staging-каталог и запускает ISCC. Повторяемый пайплайн вместо ручного копирования.

**Files:**
- Create: `scripts/package_windows.ps1`

**Поведение скрипта (параметры: `-Mode Release` (только), `-SkipBackend`, `-SkipFrontend`, `-OutDir dist`):**
1. Пререквизиты: `cargo` в PATH, `flutter` в PATH; иначе осмысленная ошибка.
2. Версия: распарсить `backend/Cargo.toml` `[workspace.package] version` → `$Version`.
3. `cargo build --release -p omnesagent` (в `backend/`). Ожидаемый продукт: `backend\target\release\omnesagent.exe`.
4. `Push-Location frontend/desktop; flutter build windows --release; Pop-Location`. Продукт: `frontend\desktop\build\windows\x64\runner\Release\`.
5. Staging: `$env:TEMP\omnesagent-pkg\v$Version\` — скопировать из Release-каталога **все** файлы и `data\`, НО исключить ручной `omnesagent-backend.exe`; затем:
   ```
   <staging>\OmnesAgent.exe          (клиент)
   <staging>\*.dll                   (flutter_windows, webview_windows_plugin, WebView2Loader и др.)
   <staging>\data\                   (Flutter-ассеты)
   <staging>\bin\omnesagent.exe      (копия backend\target\release\omnesagent.exe) ← лэйаут DesktopBackendManager
   <staging>\LICENSE
   <staging>\README.txt              (кратко: что это, порт 42617, данные в %USERPROFILE%\.omnesagent)
   ```
6. Хэши: `Get-FileHash` для `omnesagent.exe` и `OmnesAgent.exe` → лог в stdout.
7. **(Опционально) Подпись:** параметр `-Sign` — если задан и `signtool.exe` в PATH + сертификат доступен (`$env:SIGN_PFX_PATH` / `$env:SIGN_PFX_PASSWORD`), подписать установщик: `signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 <installer>`; если что-то из этого отсутствует — WARNING и пропуск (не падать). Без `-Sign` шаг не выполняется вовсе.
8. Запуск ISCC: `iscc /DAppVersion=$Version /DStagingDir=<staging> installer\windows\omnesagent.iss` → `dist\OmnesAgent-Setup-v$Version-x64.exe`.
9. В конце: путь к установщику + `Get-FileHash` + размер.

**Verify (смоук):**
```powershell
.\scripts\package_windows.ps1
```
Expected: в конце напечатан `dist\OmnesAgent-Setup-v0.1.0-x64.exe` и staging содержит `bin\omnesagent.exe`. Полная сборка занимает время (первый cargo release-билд — десятки минут) — запускать с большим таймаутом.

**Commit:** `build(windows): reproducible packaging script for desktop installer`

---

## Task 3: Inno Setup-скрипт `installer/windows/omnesagent.iss`

**Objective:** Установщик с поведением «как у Adobe»: каталог в Program Files, ярлыки, uninstall-запись, опциональный автозапуск, детект WebView2.

**Files:**
- Create: `installer/windows/omnesagent.iss`

**Шаблон (копия-основа, доработать GUID и тексты):**

```ini
#define AppName "OmnesAgent"
#ifndef AppVersion
#define AppVersion "0.0.0"
#endif

[Setup]
AppId={{ЗАМЕНИТЬ-НА-СГЕНЕРИРОВАННЫЙ-GUID}}   ; один раз: [guid]::NewGuid() в PowerShell.
                                              ; ВАЖНО: после первого релиза GUID НЕ менять —
                                              ; иначе обновление поверх не будет обнаружено
                                              ; как апгрейд того же приложения
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=Omnes Agent
AppPublisherURL=https://github.com/Pofium/omnes-agent
AppUpdatesURL=https://github.com/Pofium/omnes-agent/releases
VersionInfoVersion={#AppVersion}              ; корректные «Свойства файла» установщика
VersionInfoProductVersion={#AppVersion}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName=Omnes Agent
DisableProgramGroupPage=yes
OutputBaseFilename=OmnesAgent-Setup-v{#AppVersion}-x64
OutputDir=..\..\dist
Compression=lzma2/max
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
PrivilegesRequired=admin
WizardStyle=modern
UninstallDisplayIcon={app}\OmnesAgent.exe
SetupIconFile=..\..\frontend\desktop\assets\app_icon.ico
CloseApplications=yes
SetupLogging=yes                              ; лог установки в %TEMP% — упрощает разбор багов

[Languages]
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "autostart"; Description: "Запускать Omnes Agent при входе в Windows"; Flags: unchecked

[Files]
Source: "{#StagingDir}\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion
; WebView2 Bootstrapper (скачать заранее в installer\windows\redist\):
Source: "redist\MicrosoftEdgeWebview2Setup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall

[Icons]
Name: "{group}\Omnes Agent"; Filename: "{app}\OmnesAgent.exe"
Name: "{group}\Удалить Omnes Agent"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Omnes Agent"; Filename: "{app}\OmnesAgent.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "OmnesAgent"; ValueData: """{app}\OmnesAgent.exe"""; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/install"; StatusMsg: "Установка WebView2 Runtime..."; Check: not WebView2Installed
Filename: "{app}\OmnesAgent.exe"; Description: "{cm:LaunchProgram,Omnes Agent}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; гасим клиента и шлюз, если остались висеть после падения/зависания
Filename: "{cmd}"; Parameters: "/C taskkill /F /IM OmnesAgent.exe /T"; Flags: runhidden; RunOnceId: "KillClient"
Filename: "{cmd}"; Parameters: "/C taskkill /F /IM omnesagent.exe /T"; Flags: runhidden; RunOnceId: "KillGateway"

[UninstallDelete]
; Данные пользователя (%USERPROFILE%\.omnesagent) СОЗНАТЕЛЬНО не удаляются автоматически —
; удаление только по явному согласию пользователя, см. CurUninstallStepChanged ниже

[Code]
function WebView2Installed: Boolean;
begin
  Result :=
    RegKeyExists(HKEY_LOCAL_MACHINE, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}') or
    RegKeyExists(HKEY_CURRENT_USER, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}');
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then
    if MsgBox('Также удалить данные пользователя?' #13#10 +
              '(сессии и настройки в %USERPROFILE%\.omnesagent)' #13#10 #13#10 +
              '«Нет» — сохранить данные для будущих установок.',
              mbConfirmation, MB_YESNO) = IDYES then
      DelTree(ExpandConstant('{%USERPROFILE}\.omnesagent'), True, True, True);
end;
```

**Замечания к шаблону:**
- **`PrivilegesRequired=admin` + `{autodesktop}`:** ярлык на рабочий стол попадает на стол того пользователя, который запускал установку (админа). Для релиза 0 принимается; если захочется per-user установки — добавить `PrivilegesRequiredOverridesAllowed=dialog` (тогда Inno предложит «для всех / только для меня»), но потребуется проверить поведение WebView2-бутстрэппера в per-user режиме.
- Проверка WebView2 (`WebView2Installed`) покрывает HKLM и HKCU — пер-юзерные установки Runtime тоже детектятся.

**Подготовка bootstrapper:** скачать `https://go.microsoft.com/fwlink/?linkid=2124701` → `installer/windows/redist/MicrosoftEdgeWebview2Setup.exe` (ссылку проверить на этапе сборки; в git класть бинарник НЕ обязательно — скрипт может скачивать при сборке, если файла нет).

**Verify:** `iscc /DAppVersion=0.1.0 /DStagingDir=... installer\windows\omnesagent.iss` компилируется без ошибок; выход `dist\OmnesAgent-Setup-v0.1.0-x64.exe` существует.

**Commit:** `build(windows): Inno Setup installer script (shortcuts, uninstall, WebView2)`

---

## Task 4: Проверка обнаружения шлюза из установленного лэйаута

**Objective:** Убедиться, что из staging-каталога клиент находит `bin\omnesagent.exe` (логика `findOmnesAgentBinary`), до установки в Program Files.

**Step 1:** Из staging-каталога запустить `OmnesAgent.exe` (двойной клик или `Start-Process`). Expected: в debug-логе `DesktopBackendManager` находит бэкенд (либо шлюз отвечает).
**Step 2:** `netstat -ano | findstr :42617` → процесс слушает **127.0.0.1:42617** (не 0.0.0.0 — иначе появится вопрос firewall и надо будет добавить правило/флаг в установщик; если bind не loopback — фикс: конфиг шлюза в `{app}` с `host="127.0.0.1"`, это отдельная правка в Task 3).
**Step 3:** `curl http://127.0.0.1:42617/health` → 200 (если эндпоинт другой — проверить `/api/health`, взять тот, что отвечает в рантайме).

**Verify:** все три пункта выполнены; закрытие приложения гасит шлюз (`netstat` пуст).

---

## Task 5: E2E-тест установщика (чистая среда)

**Objective:** Поведение как у крупного ПО: установка → работа → обновление поверх → деинсталляция.

**Скрипт проверки (на той же машине или чистой Win10/11 VM / отдельной учётке):**
1. Запустить `dist\OmnesAgent-Setup-v0.1.0-x64.exe` → NEXT → установка в `C:\Program Files\OmnesAgent` → ярлык на рабочем столе (если галка) и в Пуске.
2. Запуск из меню «Пуск» → шлюз поднялся (`netstat :42617`) → UI отвечает; выбор модели показывает честное «Провайдер не настроен» (это ожидаемое из коробки состояние, План ADE не требует ключей для запуска).
3. **Обновление поверх:** запустить установщик ещё раз → должен пройти в режиме обновления без потери данных в `~/.omnesagent` (сессии на месте).
4. **Деинсталляция:** Параметры → Приложения → Omnes Agent → Удалить. Expected: задан вопрос про данные пользователя; при ответе «Нет» — папка `{app}` удалена, оба ярлыка (Пуск + рабочий стол) и ярлык «Удалить Omnes Agent» удалены, запись из реестра Run удалена (если ставили), `~/.omnesagent` **остался**; при повторной установке и ответе «Да» — `~/.omnesagent` удалён.
5. Повторная установка после деинсталляции работает.

**Verify:** чеклист выше полностью зелёный; на неподписанном exe SmartScreen предупредит — занести в известные ограничения (см. Риски).

**Коммит на этом этапе не нужен** (тестовый прогон), но если потребовались правки .iss — фиксировать отдельным коммитом.

---

## Task 6: Подключение к GitHub Release и README

**Objective:** Артефакт доступен пользователям, README больше не содержит заглушку.

**Шаги:**
1. `gh release upload v0.1.0 dist/OmnesAgent-Setup-v0.1.0-x64.exe#OmnesAgent-Setup-v0.1.0-x64.exe` (смонтировать как non-draft после проверки; release создан Планом 0, Task 8).
2. Дописать в README (ru/en) в секцию «Установка» реальную ссылку:
   `https://github.com/Pofium/omnes-agent/releases/latest` → скачать `OmnesAgent-Setup-*.exe`.
3. `gh release edit v0.1.0 --draft=false` — только когда прикреплены артефакты (хотя бы этот).

**Verify:** `gh release view v0.1.0` показывает ассет; ссылка в README ведёт на рабочий релиз; скачивание exe с GitHub проходит.

**Commit:** `docs(readme): link Windows installer release (ru/en)`

---

## Риски, компромиссы, открытые вопросы

- **SmartScreen / антивирус:** неподписанный установщик вызовет предупреждение «Unknown publisher». Полноценное решение — Authenticode-сертификат (signtool в package-скрипте, флаг `-Sign`). **Открытый вопрос:** покупать ли сертификат OV/EV или остаться unsigned для Релиза 0 (рекомендация: unsigned, задокументировать обход SmartScreen).
- **WebView2:** линк bootstrapper — внешняя зависимость Microsoft; при сборке проверять живость ссылки. Альтернатива: положить standalone-инсталлятор в релиз.
- **Размер:** Flutter Windows + Rust release ≈ 100–200 МБ installer (lzma2 сожмёт). Приемлемо для Photoshop-класса «из коробки».
- **Первый cargo release-билд долгий** (десятки минут) — в CI кэшировать (вне скоупа Релиза 0).
- **Автообновление внутри приложения** — сознательно НЕ делаем (YAGNI для Релиза 0); обновление = повторный запуск установщика (Inno умеет поверх).
- `omnesagent-backend.exe` из Release-каталога — артефакт ручной возни; пакетный скрипт его исключает и кладёт канонический `bin\omnesagent.exe`.
- Если дефолтный конфиг шлюза слушает не 127.0.0.1 — добавить в установщик `config.toml` с явным loopback (выясняется в Task 4, Step 2).
- **GUID (AppId) фиксируется навсегда** при первом релизе — смена GUID между версиями ломает обнаружение апгрейда и запись «Установка и удаление программ» (дубль записи).
- **Ярлык на стол при admin-установке** попадает только устанавливающему пользователю — известное ограничение релиза 0 (см. замечания к шаблону Task 3).

## Зависимости
- План 0: версия `0.1.0`, тег `v0.1.0`, трекнутая `app_icon.ico`, заглушка в README.
- План 2 не зависит от этого плана.
