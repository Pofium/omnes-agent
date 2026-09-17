# ТЗ: Интеграция Handy (офлайн-диктовка) в OmnesAgent

| | |
|---|---|
| **Статус** | Черновик на утверждение |
| **Дата** | 2026-09-16 |
| **Скоуп v1** | Windows x64 (desktop-сценарий: Flutter-клиент + локальный Rust-бэкенд) |
| **Связанные документы** | `AGENTS.md`, `backend/AGENTS.md`, `DESIGN_SYSTEM.md`, `.gemini/style-guide.md`, `PLAN_WINDOWS_INSTALLER.md` |
| **Внешний продукт** | [cjpais/handy](https://github.com/cjpais/handy) — офлайн speech-to-text, лицензия MIT (имя/логотип из MIT исключены) |

---

## 1. Резюме

Добавить в настройки агента, в раздел настроек микрофона, три возможности:

1. **Калибровка микрофона** — выбор устройства ввода, проверка уровня сигнала с наглядным индикатором и подсказками.
2. **Автоустановка Handy** — когда пользователь пытается включить голосовой ввод, а Handy не установлен, агент предлагает автоматически скачать с официального GitHub Releases, установить (тихо, без прав администратора) и запустить Handy.
3. **Интеграция в агента** — диктовка в поле ввода чата через глобальную горячую клавишу Handy, кнопка «Диктовать» в композере, а на этапе M4 — использование Handy как офлайн-STT-движка агента (`handy --transcribe-file`).

Исполнителем установки является **Rust-бэкенд** (единственный владелец состояния и системных действий по `AGENTS.md`), Flutter-клиент работает только через `GatewayHttpClient`/`GatewayWsClient`.

---

## 2. Терминология

| Термин | Значение |
|---|---|
| **Handy** | Внешнее десктоп-приложение (Tauri 2, Rust) офлайн-диктовки: горячая клавиша → запись с микрофона → транскрипт вставляется в активное окно |
| **Установщик** | NSIS-инсталлятор Handy (`*_x64-setup.exe`) из официального GitHub Releases |
| **Манифест обновлений** | `latest.json` — Tauri-updater-манифест релиза (версия, дата, URL артефакта, minisign-подпись) |
| **Job** | Фоновая задача бэкенда (установка/обновление/удаление) с прогрессом |
| **Калибровка** | Проверка микрофона: выбор устройства, измерение уровня (dBFS), вердикт «тихо/норма/клиппинг» |
| **Диктовка (Mode A)** | Пользователь говорит в горячую клавишу Handy, текст вставляется в сфокусированное поле ввода OmnesAgent |
| **STT-движок (Mode B)** | Бэкенд OmnesAgent вызывает `handy --transcribe-file file.wav --json` и получает текст (офлайн-провайдер STT) |

---

## 3. Обзор Handy (проверенные факты)

Всё нижеперечисленное подтверждено по репозиторию `cjpais/handy` (master, версия в конфиге 0.9.0; актуальный релиз на момент написания — **v0.9.6**):

- **Платформы**: Windows x64, Linux x64, macOS (Intel/Apple Silicon). **Windows ARM64 не предоставляется.**
- **Лицензия**: MIT для кода; имя, логотип и бренд-ассеты из лицензии **исключены** (форки обязаны ребрендироваться). Мы не форкаем и не ребрендим — только устанавливаем официальный билд и ссылаемся на продукт.
- **Движки распознавания**: Whisper (GGML/GGUF, GPU, transcribe-cpp) и Parakeet V2/V3 (CPU, transcribe-rs); VAD — Silero; аудио — cpal. Часть моделей лежит в `resources/models` репозитория, т.е. поставляется внутри установщика; докачка моделей — из UI самого Handy.
- **Вывод текста**: через буфер обмена в активное окно (с восстановлением прежнего содержимого; «Reliable Paste» — бета).
- **CLI** (`src-tauri/src/cli.rs`):
  - `--start-hidden`, `--no-tray`, `--debug` — режимы запуска;
  - `--toggle-transcription`, `--toggle-post-process`, `--cancel` — **дистанционное управление уже запущенным экземпляром** (через `tauri-plugin-single-instance` второй процесс пересылает аргументы первому);
  - `--transcribe-file <WAV>` (16 кГц mono) + `--model <id>` + `--device-index N` + `--json` — **headless-транскрипция файла без запуска GUI** (без микрофона, VAD и скачивания моделей — модель должна быть уже установлена);
  - `--list-models --json`, `--list-devices` — машинночитаемый вывод (модели и compute-устройства).
- **Установщик (Windows)**: кастомный NSIS-шаблон (`src-tauri/nsis/installer.nsi`). Режим установки — `currentUser` (дефолт Tauri, `RequestExecutionLevel user`, **UAC не требуется**), каталог по умолчанию `%LOCALAPPDATA%\Handy`, бинарник `Handy.exe`. Флаги:
  - `/S` — тихая установка (silent);
  - `/P` — passive (прогресс без вопросов);
  - `/NS` — без ярлыков;
  - `/UPDATE` — режим обновления (не удаляет данные/ярлыки);
  - `/R [/ARGS "..."]` — после тихой установки запустить приложение с аргументами;
  - понижение версии в silent-режиме блокируется (выводит `silentDowngrades` и прерывается).
  - Установщик сам ставит WebView2, если его нет (скачивает bootstrapper с `go.microsoft.com`) — требует сеть при первой установке на «чистой» машине.
  - Реестр после установки: `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Handy` → `DisplayVersion`, `InstallLocation`, `UninstallString`; `HKCU\Software\<manufacturer>\Handy` → путь установки. Удаление: `uninstall.exe` в каталоге установки; деинсталлятор снимает autostart-запись `HKCU\...\Run\Handy`.
  - Важно: NSIS-деинсталлятор копирует себя во `%TEMP%` (Au_.exe) и **завершается сразу** — нельзя ждать завершения процесса, нужно поллить исчезновение ключа реестра (см. §9.7).
- **Подпись**: установщик подписан через Azure Trusted Signing (`signCommand` в `tauri.conf.json`); апдейтер использует minisign-подпись манифеста с публичным ключом, зашитым в `tauri.conf.json` (`plugins.updater.pubkey`).
- **Портативный режим**: файл-маркер `portable` рядом с exe → данные в `Data/` рядом с exe (детект по magic-string `Handy Portable Mode`).
- **Конфигурация**: `tauri-plugin-store`, файл `settings_store.json` в каталоге данных `%APPDATA%\com.pais.handy\` (портативно: `<exe>\Data\`); Rust-структура `AppSettings` с миграциями. Autostart — собственная настройка Handy (значение в `HKCU\...\Run\Handy`).
- **IPC/HTTP API у Handy нет** (кроме CLI-флагов и SIGUSR2 на Linux/macOS). Deep-link-протоколы не регистрируются.

---

## 4. Текущее состояние (as-is) в OmnesAgent

| Область | Факт | Путь |
|---|---|---|
| Настройки | Единый модальный диалог с левой навигацией; секции на ключах `general … integrations`; карточки `_buildSettingCard`, `_buildSectionCard`, `_buildFormInput` | `frontend/desktop/lib/features/settings/desktop_settings_dialog.dart`, `frontend/web/lib/features/settings/desktop_settings_dialog.dart` (почти идентичные копии) |
| Голосовой ввод | Секция `_buildSttSection()` («Voice Input & STT»): провайдер (groq/openai/cloudflare/custom), endpoint, ключ, модель, язык, «Test connection», тумблер `setSttEnabled`. Настройки хранятся **только на клиенте** (GetStorage: `stt_provider/stt_url/stt_key/stt_model/stt_lang/stt_verified/stt_enabled`). Ключ `stt` есть в switch контента, **но отсутствует в навигации** — секция недостижима из UI | там же, ~строка 3257 |
| Микрофон в чате | Кнопка микрофона в композере рабочего пространства | `frontend/desktop/lib/features/workspace/task_workspace_view.dart` |
| Дуплексный голос | WS-события `speech_start/speech_end/barge_in/tts_*` | `backend/crates/omnesagent-gateway/src/voice_duplex.rs` (feature `gateway-voice-duplex`) |
| Gateway-клиент | `GatewayHttpClient` (base `http://127.0.0.1:42617`, Bearer, таймауты, защитные дефолты) и `GatewayWsClient` (фреймы `GatewayFrame`) | `frontend/shared/lib/core/gateway/gateway_http.dart`, `gateway_ws.dart` |
| Прецедент «скачать → проверить → установить» | Самообновление бэкенда: 6 фаз (Preflight → Download с GitHub Releases → Backup → Validate sha256 → Swap → Smoke test), job-API `POST /api/version/upgrade` (202 + handoff) / `GET /api/version/upgrade/status`, гейт `gateway.allow_self_upgrade` | `backend/src/commands/update.rs`, `backend/crates/omnesagent-gateway/src/version.rs` |
| Прецедент «инструменты/внешние CLI» | `discover_cli_tools()` → `GET /api/cli-tools`; `file_download.rs` | `backend/crates/omnesagent-tools/src/cli_discovery.rs`, `file_download.rs` |
| Роутинг gateway | Единая цепочка `Router::new().route(...)` в `run_gateway` (`lib.rs`, ~1616), хендлеры по доменам (`api.rs`, `version.rs`, …), `require_auth`, SSE `/api/events` | `backend/crates/omnesagent-gateway/src/lib.rs`, `sse.rs` |
| Конфиг бэкенда | TOML `~/.omnesagent/config.toml` (атомарные записи), API `GET/PUT /api/config/prop`, секции, quickstart | `backend/crates/omnesagent-config/src/schema.rs` |
| Дизайн-система | `ShadcnDialog`, `ShadcnCard`, `ShadcnButton`, `ShadcnBadge`, `ShadcnInput`; Switch/ProgressBar/Stepper **компонентами ДС не являются** — используются Material-аналоги и hand-rolled степперы (как в quickstart) | `frontend/shared/lib/design_system/` |
| i18n | `DesktopI18n.tr('RU', 'EN')` (дублируется в `frontend/shared` и `frontend/desktop`), часть строк захардкожена по-русски | `frontend/shared/lib/utils/desktop_i18n.dart` |

**Проблемы, которые решаем попутно:**
- секция STT недостижима из навигации;
- диалог настроек продублирован desktop/web — новую голосовую секцию делаем **общей** в `frontend/shared` (требование Single Source of Truth из `AGENTS.md`).

---

## 5. Целевое состояние (to-be). UX-сценарий

Раздел «Голосовой ввод» (ключ `stt`, появляется в левой навигации настроек) состоит из трёх карточек:

```
┌ Голосовой ввод ──────────────────────────────────────────────┐
│                                                              │
│ ── Диктовка (Handy) ──────────────────────────────────────── │
│  [Switch] Включить голосовой ввод                            │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ Состояние A: Handy не установлен                     │    │
│  │  «Для диктовки требуется локальное приложение        │    │
│  │   Handy (~N МБ, офлайн-распознавание).»              │    │
│  │  [Установить автоматически]  [Подробнее]  [Не сейчас]│    │
│  ├──────────────────────────────────────────────────────┤    │
│  │ Состояние B: установка                               │    │
│  │  Этап: Загрузка  ────────░░░░░░  62% (128/206 МБ)    │    │
│  │  [Отменить]                                          │    │
│  ├──────────────────────────────────────────────────────┤    │
│  │ Состояние C: установлен                              │    │
│  │  Handy v0.9.6 • запущен   [ShadcnBadge success]      │    │
│  │  Горячая клавиша диктовки: Ctrl+Space (моно-шрифт)   │    │
│  │  [Открыть настройки Handy] [Обновить] [Удалить]      │    │
│  │  [Switch] Запускать вместе с Windows                 │    │
│  └──────────────────────────────────────────────────────┘    │
│                                                              │
│ ── Микрофон ──────────────────────────────────────────────── │
│  Устройство ввода: [Dropdown: Микрофон (Realtek…)]           │
│  [Проверить микрофон]  ▁▂▅█▇▃▁▂▁  -18 dBFS  ✓ Норма          │
│  Подсказка: то же устройство выберите в настройках Handy.    │
│                                                              │
│ ── STT-провайдер (облако, для голосовых сообщений) ───────── │
│  …существующая секция _buildSttSection без изменений…        │
└──────────────────────────────────────────────────────────────┘
```

### 5.1 Флоу «включение микрофона → установка»

1. Пользователь переводит тумблер «Включить голосовой ввод» в ON.
2. Клиент запрашивает `GET /api/voice/handy/status`.
3. Если `installed == false` — показывается `ShadcnDialog`:
   - заголовок «Установить Handy?»;
   - текст: что это, официальный источник (GitHub cjpais/handy), размер, «распознавание полностью локальное, аудио никуда не отправляется», «установка выполняется от имени текущего пользователя, без прав администратора»;
   - кнопки: **[Установить]** (primary) / **[Выбрать папку вручную]** / **[Отмена]**; чекбокс «Больше не спрашивать» (пишет `install_policy`).
4. По кнопке «Установить» — `POST /api/voice/handy/install` → 202 + job; карточка переходит в состояние B; прогресс через SSE `/api/events` (`handy.install.progress`) с фолбэком-поллингом `GET /api/voice/handy/status`.
5. Успех: карточка → состояние C, тумблер остаётся ON, подсказка «Нажмите Ctrl+Space, наведите фокус на поле сообщения и говорите». Бэкенд запускает `Handy.exe --start-hidden` (если `launch_on_agent_start`).
6. Отказ: тумблер возвращается в OFF с inline-ошибкой (код из §12) и кнопкой «Повторить».

### 5.2 Калибровка микрофона

- Dropdown устройств ввода (перечисление на клиенте).
- «Проверить микрофон»: 5 секунд записи, live-шкала уровня в dBFS (зоны: серая < −50, янтарная −50…−25, зелёная −25…−6, красная > −3 — клиппинг), затем вердикт:
  - «Тишина — микрофон не слышно» (проверьте устройство по умолчанию в Windows);
  - «Тихо — говорите ближе к микрофону»;
  - «Отлично, уровень в норме»;
  - «Слишком громко — сигнал клиппует, снизьте усиление».
- Выбранное устройство сохраняется в GetStorage (`stt_device_id`) и используется веб-звуковым трактом агента; для Handy это **рекомендация** — Handy держит собственный выбор устройства, даём кнопку «Открыть настройки Handy».

### 5.3 Интеграция в композер (Mode A)

Кнопка «Диктовать» рядом с микрофоном в `task_workspace_view.dart`: вызывает `POST /api/voice/handy/toggle` → бэкенд исполняет `Handy.exe --toggle-transcription` (одиночный экземпляр Handy пересылает команду себе). Пользователь наводит фокус на поле сообщения и диктует — текст печатается в поле штатным механизмом вставки Handy. Повторное нажатие останавливает. В состоянии записи кнопка меняет иконку на «стоп» и подсвечивается (прогнозируем по собственному таймеру/состоянию, без IPC от Handy — см. риски).

### 5.4 Офлайн-STT-движок агента (Mode B, M4)

- В конфиге бэкенда появляется провайдер `stt.provider = "handy"` (TOML `[stt]`).
- Транскрипция: бэкенд сохраняет WAV (16 кГц mono) во временный файл в `~/.omnesagent/data/handy/tmp/`, запускает `Handy.exe --transcribe-file <path> --json`, парсит JSON, удаляет файл. Таймаут 120 с, лимит размера 25 МБ.
- Если модель не установлена (`--list-models --json` пуст) — ошибочный код `HANDY_NO_MODEL` с подсказкой «откройте Handy и скачайте модель».

---

## 6. Архитектура

```
Flutter (desktop/web)                 Rust gateway                     Система
─────────────────────                 ────────────                     ───────
Настройки «Голосовой ввод»
  │  GatewayHttpClient                 omnesagent-gateway
  ├─ GET  /api/voice/handy/status ───► handy.rs ──► omnesagent-tools
  ├─ POST /api/voice/handy/install                     handy::
  ├─ SSE  /api/events  ◄── handy.install.progress       ├─ detect.rs   (реестр, пути, процесс)
  ├─ POST /api/voice/handy/toggle ─────────────────────►│─ manifest.rs (latest.json, minisign)
  └─ POST /api/voice/handy/transcribe (M4)              │─ installer.rs(скачивание, /S, поллинг)
                                                        └─ launcher.rs (spawn, CLI-флаги)
                                                             │
                                                             ▼
                                                  %LOCALAPPDATA%\Handy\Handy.exe
                                                  HKCU\...\Uninstall\Handy
                                                  HKCU\...\Run\Handy (autostart)
                                                  %APPDATA%\com.pais.handy\ (данные Handy)
```

**Принципы:**
- Бэкенд — единственный исполнитель системных действий (запуск процессов, реестр, файлы). Клиент — только REST/SSE.
- Никакой прямой сети из Flutter для установки: скачивание делает бэкенд (единые SSRF-гварды, §11).
- Повторное использование паттерна job из `version.rs` (202 + id + статус-эндпоинт) и фаз из `update.rs`.
- Секция UI — общий виджет в `frontend/shared`, подключаемый обеими копиями диалога настроек.

---

## 7. Конфигурация (config.toml)

Новая секция (доступна клиенту через штатный `GET/PUT /api/config/prop`):

```toml
[voice.handy]
enabled = false                 # диктовка включена (состояние тумблера, дублируется в GetStorage клиента)
install_policy = "ask"          # ask | auto | manual — auto: устанавливать без диалога
pin_version = ""                # пусто = последний релиз; иначе строгая версия, напр. "0.9.6"
allow_downgrade = false         # разрешить откат версии при установке
launch_on_agent_start = true    # бэкенд стартует Handy при старте gateway (если installed)
auto_start_with_os = false      # значение HKCU\...\Run\Handy (поддерживается и самим Handy)
start_hidden = true             # запускать с --start-hidden
download_host_allowlist = [
  "github.com",
  "objects.githubusercontent.com",
  "release-assets.githubusercontent.com",
]
max_download_bytes = 786432000  # 750 MiB — жёсткий предохранитель
transcribe_timeout_secs = 120   # для Mode B
```

Дефолты зашиты в `omnesagent-config` (`schema.rs`), документируются в каталоге конфига (`/api/config/catalog`).

Клиентский GetStorage (только UI-состояние): `handy_install_policy_ack`, `stt_device_id`.

---

## 8. Детекция установленного Handy (detect.rs)

Порядок (первый успешный — итог; все шаги логируются):

1. **Реестр** `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Handy`:
   `InstallLocation`, `DisplayVersion`, `UninstallString`. Ключ читается через `windows-registry`/`winreg` (уже в дереве зависимостей бэкенда — проверить; иначе добавить `winreg`).
2. **Пути**: `%LOCALAPPDATA%\Handy\Handy.exe` (+ `InstallLocation` из `HKCU\Software\<manufacturer>\Handy`). Версия exe — из version-resource (крейт `windows` / PE-парсер), не по имени файла.
3. **Портативные/scoop-установки**: ключей в реестре нет — `installed = false`, но `running` может быть true; UI в этом случае предлагает «Указать путь к Handy.exe вручную» (запись `voice.handy.custom_exe_path` в config.toml; путь валидируется: существует `Handy.exe`, version-resource содержит ProductName == "Handy").
4. **Процесс**: `Handy.exe` в списке процессов (toolhelp32 snapshot / `sysinfo`), чтобы отличать «установлен, но не запущен».
5. **Портативность собственной установки**: если найден маркер `portable` рядом с exe — `portable = true` (обновление такого экземпляра штатным установщиком запрещаем, код `PORTABLE_UNSUPPORTED`).

Возвращаемая структура `HandyStatus`:

```json
{
  "supported": true,             // windows x64
  "installed": true,
  "version": "0.9.6",
  "install_path": "C:\\Users\\u\\AppData\\Local\\Handy",
  "exe_path": "C:\\Users\\u\\AppData\\Local\\Handy\\Handy.exe",
  "portable": false,
  "running": true,
  "custom_exe_path": null,
  "latest": { "version": "0.9.6", "pub_date": "2026-09-01T00:00:00Z", "update_available": false },
  "active_job": null,
  "config": { "enabled": false, "install_policy": "ask", "auto_start_with_os": false }
}
```

`latest` кэшируется на 10 минут (`~/.omnesagent/cache/handy/manifest.json` + etag); при недоступности сети — последнее закэшированное значение и `update_available = null`.

---

## 9. Спецификация бэкенда

### 9.1 Модули

| Модуль | Назначение |
|---|---|
| `backend/crates/omnesagent-tools/src/handy/mod.rs` | Публичный API, типы, `HandyError` |
| `…/handy/detect.rs` | §8 |
| `…/handy/manifest.rs` | Загрузка `latest.json`, парсинг, проверка minisign-подписи (крейт `minisign-verify`), кэш |
| `…/handy/installer.rs` | Машина состояний job: Download → Verify → Install → Postcheck; отмена |
| `…/handy/launcher.rs` | Spawn `Handy.exe` с флагами, `--toggle-transcription`, `--transcribe-file`, graceful stop |
| `…/handy/autostart.rs` | Чтение/запись/удаление `HKCU\...\Run\Handy` |
| `backend/crates/omnesagent-gateway/src/handy.rs` | HTTP-хендлеры + широковещательные SSE-события (по образцу `version.rs`) |

Публичный ключ minisign берётся из `tauri.conf.json` Handy (`plugins.updater.pubkey`, base64-строка `dW50cnVzdGVkIGNvbW1lbnQ6…`) и хранится **константой в исходнике** с комментарием-источником; смена ключа upstream = `SIGNATURE_INVALID` до ручного обновления константы.

### 9.2 Манифест и выбор артефакта

- Primary: `GET https://github.com/cjpais/Handy/releases/latest/download/latest.json` (302 → `release-assets.githubusercontent.com` — редирект валиден, обе площадки в allowlist).
- Формат: `{ "version": "0.9.6", "pub_date": "...", "platforms": { "windows-x86_64": { "signature": "<minisign>", "url": "https://github.com/cjpais/Handy/releases/download/v0.9.6/<asset>.exe" } } }`.
- URL артефакта берём **только** из манифеста; самостоятельное конструирование имён ассетов запрещено (имена могут меняться).
- Fallback при недоступности манифеста: GitHub API `https://api.github.com/repos/cjpais/handy/releases/latest` → ассет по регулярке `x64-setup\.exe$` → в этом режиме minisign-проверка невозможна, обязателен шаг Authenticode-проверки (§9.4) и `pin_version` игнорируется. Fallback отключаем, если `gateway.strict_handy_source = true` (опция для параноидальных инсталляций).
- Фиксация версии: если `pin_version != ""`, берём `releases/tags/v<version>/latest.json` (path-параметр вместо `latest`).

### 9.3 Загрузка (installer.rs, фаза Download)

1. Pre-checks: `supported`, отсутствие активного job (мьютекс в `AppState`), свободное место ≥ 2×size + 200 МБ (size — из `Content-Length`; манифест размера не даёт), каталог `~/.omnesagent/cache/handy/downloads/`.
2. HTTPS-only; до коннекта резолвим DNS хоста и отвергаем loopback/приватные/зарезервированные адреса (RFC 1918/4193/3927, link-local 169.254/fe80::, 0.0.0.0, ::1, multicast, документационные 100.64/198.18/192.0.2 и т.п.); повторная проверка после каждого редиректа (§11).
3. Стрим в `<tmp>.part` с прогрессом (каждые 512 КБ → событие), лимит `max_download_bytes`, таймаут простоя 30 с, 3 ретрая с экспоненциальной задержкой (1/4/16 с), возобновление не требуется (файл начинается заново при ретрае — артефакт < 1 ГБ).
4. `*.part` → атомарный rename по завершении.

### 9.4 Проверка (фаза Verify)

1. **Minisign** — проверяем подпись манифеста по публичному ключу из §9.1 по канонической схеме Tauri updater (payload = `<url><sha256 файла>`; если upstream изменит схему — см. M0-spike). Несовпадение → `SIGNATURE_INVALID`, файл удаляется, job падает.
2. **Authenticode** (обязателен в fallback-режиме, оптимизирован в primary): PowerShell `Get-AuthenticodeSignature <file>` → статус `Valid` и субъект подписи, содержащий `CN=cjpais` (уточнить точный DN на M0). Проверка в отдельном процессе с таймаутом 60 с.
3. Контрольный sha256 записывается в `~/.omnesagent/data/handy/installed.json` (`{version, sha256, installed_at, source}`) — база для будущих проверок и для детекта подмены при переустановке той же версии.

### 9.5 Установка (фаза Install)

```
<downloads>/Handy_<v>_x64-setup.exe /S          # тихая установка, per-user, UAC не требуется
```

- Запуск: `std::process::Command`/`tokio::process` без shell, `CREATE_NO_WINDOW`, cwd = каталог загрузок, env наследуется; таймаут 600 с.
- Успех = поллинг (500 мс, до 600 с): ключ `HKCU\...\Uninstall\Handy` существует **и** `DisplayVersion == ожидаемой` **и** `%LOCALAPPDATA%\Handy\Handy.exe` существует.
- Коды завершения: `0` — ок; `1` — отмена пользователя (у нас UI-отмены инсталлятора нет, трактуем как `INSTALLER_EXIT_CODE`); остальные — `INSTALLER_EXIT_CODE {code}`. Отдельно парсим текст обрыва silent-установки при downgrade → `DOWNGRADE_BLOCKED`.
- Отмена job разрешена **только** в фазах Download/Verify; в Install кнопка «Отменить» блокируется («идёт установка, прерывать нельзя»).
- Post-check (фаза Postcheck): запуск smoke `Handy.exe --list-models --json` с таймаутом 20 с — проверяем, что бинарник исполняется и печатает валидный JSON (не требует GUI и микрофона). Запуск с `--start-hidden` не нужен — флаги не GUI-режима. Ошибка smoke → статус `installed, но degraded` + `HANDY_SMOKE_FAILED` в статусе job.
- WebView2: если установщик сам качает bootstrapper, фаза Install может занять дополнительные минуты на чистой машине — в UI предупреждение «на чистой системе установка может занять до 5 минут».

### 9.6 Запуск и остановка (launcher.rs)

| Действие | Команда |
|---|---|
| Запуск | `Handy.exe --start-hidden` (+ `--no-tray` если `voice.handy.start_hidden && hide_tray`), detached, `Job`-объект не привязываем — Handy должен переживать рестарт бэкенда |
| Toggle диктовки | `Handy.exe --toggle-transcription` (одиночный экземпляр пересылает сам себе); перед этим launch, если не запущен |
| Отмена диктовки | `Handy.exe --cancel` |
| Список моделей (Mode B) | `Handy.exe --list-models --json` |
| Транскрипция (Mode B) | `Handy.exe --transcribe-file <wav> --json [--model <id>]`, таймаут `transcribe_timeout_secs` |
| Остановка | 1) `WM_CLOSE` через `taskkill /IM Handy.exe` (без `/F`); 2) через 5 с — `taskkill /F /IM Handy.exe`. Оба — только если процесс запускали мы или он есть в снапшоте |

Autostart с ОС (`auto_start_with_os`): значение `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, имя `Handy`, данные `"<exe>" --start-hidden`. Тоггл синхронизируется с настройкой самого Handy (та же запись реестра).

### 9.7 Удаление

```
<install_path>\uninstall.exe /S _?=<install_path>
```

- NSIS-деинсталлятор копирует себя в `%TEMP%` и мгновенно завершается → **не ждать процесс**; поллить исчезновение ключа `HKCU\...\Uninstall\Handy` (до 120 с).
- `delete_app_data=true` → дополнительно `RmDir %APPDATA%\com.pais.handy` (спросить подтверждение в UI: «удалить модели и историю Handy»).
- Портативные и custom-path установки удалять программно нельзя — показываем инструкцию.

### 9.8 Обновление

Тот же install-job: если `installed && latest.version > installed.version` (semver-сравнение крейтом `semver`), установщик запускается с `/S /UPDATE` — обновляет поверх без удаления данных и ярлыков. Даунгрейд при `pin_version` ниже установленной — запрещён, если `allow_downgrade = false` (код `DOWNGRADE_BLOCKED`, не запускаем установщик вовсе).

---

## 10. HTTP API gateway

Все хендлеры — в `backend/crates/omnesagent-gateway/src/handy.rs`, регистрация в общей цепочке `lib.rs`, `require_auth` как у соседей. Формат ответов и коды ошибок — по образцу `version.rs`.

| Метод и путь | Назначение | Ответ |
|---|---|---|
| `GET /api/voice/handy/status` | §8 + активный job | 200 `HandyStatus` |
| `POST /api/voice/handy/install` | Запуск установки/обновления. Body: `{"version": "0.9.6" \| null, "allow_downgrade": bool}` | 202 `{"job_id": "..."}`; 409 если job уже идёт |
| `POST /api/voice/handy/install/cancel` | Отмена (только Download/Verify) | 200 / 409 `{"error": "phase_not_cancellable"}` |
| `POST /api/voice/handy/launch` | Запуск `Handy.exe --start-hidden` | 200 `{"running": true}` |
| `POST /api/voice/handy/stop` | Graceful-stop | 200 |
| `POST /api/voice/handy/toggle` | `--toggle-transcription` (с auto-launch) | 200 `{"dispatched": true}` |
| `POST /api/voice/handy/uninstall` | Body `{"delete_app_data": bool}`; установленное через custom path — 400 | 202 `{job_id}` |
| `POST /api/voice/handy/transcribe` **(M4)** | multipart WAV ≤25 МБ | 200 `{"text": "..."}` / 4xx код ошибки |
| `GET /api/voice/handy/models` **(M4)** | `--list-models --json` | 200 `[{id, …}]` |

**SSE**: в существующий поток `/api/events` добавляется событие:

```json
{ "type": "handy.install.progress",
  "job_id": "…", "kind": "install|update|uninstall",
  "stage": "download|verify|install|postcheck|done|failed",
  "percent": 62, "downloaded_bytes": 134217728, "total_bytes": 216006656,
  "message": "Загрузка…" }
```

Частота событий — не чаще 10/с (коалесинг). Клиент всегда может сфолбэчиться на поллинг `status`.

**Гварды**: все операции локальной установки разрешены только когда gateway слушает на loopback и запрос пришёл с loopback (`ConnectInfo<SocketAddr>`); для remote-бэкенда статус остаётся читаемым, а install/launch/uninstall возвращают `403 {"error": "local_only"}` — клиент в этом случае показывает бейдж «Доступно только для локального агента».

---

## 11. Безопасность

1. **SSRF (по обязательным гвардиям репозитория)**: только `https://`; перед соединением и после каждого редиректа — резолв хоста и отказ для loopback, приватных и зарезервированных сетей; allowlist хостов из `voice.handy.download_host_allowlist`; редирект на хост вне allowlist → `HOST_NOT_ALLOWED`; количество редиректов ≤ 5.
2. **Целостность**: minisign-подпись манифеста (ключ зашит в коде) — primary; Authenticode `Valid` + ожидаемый издатель — fallback/опция; sha256 в `installed.json`.
3. **Запуск процессов**: без shell (`Command::arg`, без `cmd /c`), фиксированные массивы аргументов, пути только из проверенных источников (реестр/манифест), `CREATE_NO_WINDOW`.
4. **Права**: никаких повышений привилегий; per-user установка. Если у пользователя Handy установлен per-machine (Program Files) — update-job требует UAC → запрещаем программное обновление, предлагаем «обновите вручную» (`ELEVATION_REQUIRED`).
5. **Приватность**: аудио никуда не отправляется (Handy офлайн; Mode B — локальный файл, удаляется после транскрипции). В логи не пишем содержимое транскриптов (только длительность/код результата).
6. **Ресурсы**: лимит размера загрузки, лимиты времени фаз, очистка `*.part` при отмене/ошибке, каталог кэша вне системного temp (меньше шансов для эвристики антивируса на запуск инсталлятора из %TEMP%).
7. **Трейд-марки**: нигде не ребрендим и не модифицируем артефакты; в UI — «Handy — стороннее приложение (MIT), устанавливается с официального сайта разработчика» со ссылкой.

---

## 12. Коды ошибок

| Код | HTTP/место | Причина | Действие UI |
|---|---|---|---|
| `UNSUPPORTED_PLATFORM` | status | не Windows/x64 | карточка скрыта, бейдж «не поддерживается» |
| `LOCAL_ONLY` | 403 | удалённый бэкенд | бейдж «только для локального агента» |
| `JOB_ALREADY_RUNNING` | 409 | повторный install | скрыть кнопку |
| `HOST_NOT_ALLOWED` | job failed | хост вне allowlist / SSRF | «Источник загрузки отклонён» |
| `NETWORK_ERROR` | job failed | таймауты/сеть | «Проверьте сеть» + «Повторить» |
| `SIZE_LIMIT_EXCEEDED` | job failed | > max_download_bytes | «Файл больше ожидаемого» |
| `SIGNATURE_INVALID` | job failed | minisign/подпись | «Подпись не совпала — установка отменена» + ссылка на issue |
| `AUTHENTICODE_INVALID` | job failed | fallback-проверка | то же |
| `DOWNGRADE_BLOCKED` | job failed | версия ниже установленной | «Установлена более новая версия» |
| `ELEVATION_REQUIRED` | job failed | per-machine установка | «Обновите Handy вручную» |
| `INSTALL_TIMEOUT` / `INSTALLER_EXIT_CODE` | job failed | сбой установщика | лог-файл + «Повторить» |
| `SMOKE_FAILED` | warning | `--list-models` не ответил | «Установлено, но проверка не прошла» |
| `DISK_SPACE` | job failed | мало места | освободить место |
| `HANDY_NO_MODEL` | transcribe | модель не скачана | «Откройте Handy и скачайте модель» |
| `HANDY_TRANSCRIBE_TIMEOUT` | transcribe | > timeout | «Не удалось распознать (таймаут)» |
| `CANCELLED` | job | отмена пользователем | тихо, карточка в состояние A |

Логирование: структурированные события `omnesagent-log` (module `handy::…`, action, outcome, attrs без PII), отдельный файловый лог job в `~/.omnesagent/data/handy/job-<id>.log` для диагностики.

---

## 13. Frontend-спецификация

### 13.1 Структура кода

```
frontend/shared/lib/features/settings/voice/
  voice_settings_section.dart      // общая секция «Голосовой ввод» (stateful)
  handy_install_card.dart          // карточка состояний A/B/C
  handy_install_dialog.dart        // ShadcnDialog предложения установки
  mic_calibration_card.dart        // устройство + индикатор уровня
  handy_models.dart                // DTO: HandyStatus, HandyJob, InstallProgress (fromJson/toJson)
frontend/shared/lib/core/gateway/gateway_http.dart
  // ── Handy API (/api/voice/handy) ──  методы по §10
```

- Подключение: обе копии `desktop_settings_dialog.dart` (desktop и web) рендерят `VoiceSettingsSection` вместо текущей `_buildSttSection()`; существующий облачный STT-блок переносится внутрь общей секции как подвью.
- Навигация: добавить nav-item `Голосовой ввод` → ключ `stt` (сейчас ключ существует, пункта нет) с иконкой `Icons.mic_none`, позиция после «Устройства».
- i18n: все новые строки через `DesktopI18n.tr(ru, en)` (таблица в §14); моно-шрифт (Consolas) для версий, путей и горячих клавиш — по `DESIGN_SYSTEM.md`.
- Дизайн: карточки `_buildSectionCard`-стиль ДС (L1/L2 фоны, border-subtle, радиусы 12 у диалогов), прогресс — линейный `LinearProgressIndicator` цвета акцента `#00D2FF`, статусы — `ShadcnBadge` (success/warning/error), без emoji.

### 13.2 Состояния карточки Handy (машина состояний клиента)

`idle → checking → not_installed → dialog_confirm → installing(stage,percent) → installed(running?) → updating → uninstalling`, плюс терминальные `error(code)` и `degraded`. Переходы инициируются ответами `status` и SSE-событиями; при разрыве SSE — поллинг `status` раз в 2 с, при восстановлении — обратно на SSE.

### 13.3 Калибровка (реализация)

- Desktop: крейт-пакет `record` (Flutter): `record.enumerateDevices()` для dropdown; `startRecorder(deviceId, …)` + `onAmplitudeChanged` для шкалы dBFS; 5 с, затем stop. Выбранное устройство — GetStorage `stt_device_id`.
- Web: `getUserMedia` + `AnalyserNode` (тот же UI, другой источник амплитуды) — реализуется за интерфейсом `MicLevelSource` с двумя реализациями.
- Отдельный поток: если системное устройство по умолчанию изменено в Windows после калибровки — не наша ответственность; повторная проверка доступна всегда.

### 13.4 Композер (Mode A)

- В `task_workspace_view.dart` рядом с кнопкой микрофона — кнопка «Диктовать» (видима только при `handy.enabled && installed`; состояние приходит из статус-стора, обновляемого в `VoiceSettingsSection` и при старте приложения).
- Нажатие: оптимистичная инверсия иконки + `POST /api/voice/handy/toggle`. Таймаут ответа 3 с; при ошибке — revert и snackbar.
- Ограничение честности: фокус ввода должен быть на поле сообщения — перед первым включением показываем одноразовую подсказку (GetStorage `handy_dictation_hint_shown`).

---

## 14. Строки i18n (ru/en, `DesktopI18n.tr`)

| Ключ/место | RU | EN |
|---|---|---|
| nav.item | Голосовой ввод | Voice input |
| handy.card.title | Диктовка (Handy) | Dictation (Handy) |
| handy.card.subtitle | Офлайн-распознавание речи сторонним приложением | Offline speech recognition via a third-party app |
| handy.toggle | Включить голосовой ввод | Enable voice input |
| handy.state.missing | Для диктовки требуется приложение Handy | Handy app is required for dictation |
| handy.install.cta | Установить автоматически | Install automatically |
| handy.install.details | Подробнее | Learn more |
| handy.install.later | Не сейчас | Not now |
| handy.dialog.title | Установить Handy? | Install Handy? |
| handy.dialog.body | Handy — бесплатное офлайн-приложение диктовки (MIT). Агент скачает его с официального GitHub разработчика (~%SIZE% МБ) и установит для текущего пользователя без прав администратора. Аудио обрабатывается только на вашем компьютере. | Handy is a free offline dictation app (MIT). The agent will download it from the developer's official GitHub (~%SIZE% MB) and install it for the current user without administrator rights. Audio is processed only on your computer. |
| handy.progress.download | Загрузка… | Downloading… |
| handy.progress.verify | Проверка подписи… | Verifying signature… |
| handy.progress.install | Установка… Это может занять несколько минут. | Installing… This may take a few minutes. |
| handy.progress.postcheck | Проверка установки… | Verifying installation… |
| handy.state.installed | Handy v%V% • запущен | Handy v%V% • running |
| handy.state.installed_stopped | Handy v%V% • не запущен | Handy v%V% • not running |
| handy.hotkey | Горячая клавиша диктовки: %K% | Dictation hotkey: %K% |
| handy.action.openSettings | Открыть настройки Handy | Open Handy settings |
| handy.action.update | Обновить | Update |
| handy.action.uninstall | Удалить | Uninstall |
| handy.action.run | Запустить | Launch |
| handy.action.stop | Остановить | Stop |
| handy.autostart | Запускать вместе с Windows | Start with Windows |
| handy.error.localOnly | Доступно только для локального агента | Available for the local agent only |
| handy.error.network | Проблема с сетью. Проверьте подключение и повторите. | Network problem. Check the connection and retry. |
| handy.error.signature | Подпись загрузки не совпала — установка отменена. | Download signature mismatch — installation aborted. |
| mic.device | Устройство ввода | Input device |
| mic.test | Проверить микрофон | Test microphone |
| mic.verdict.silent | Тишина — микрофон не слышно | Silence — no mic input detected |
| mic.verdict.quiet | Тиховато — говорите ближе к микрофону | A bit quiet — speak closer to the mic |
| mic.verdict.ok | Отлично, уровень в норме | Great, level looks good |
| mic.verdict.clipping | Слишком громко — сигнал клиппует | Too loud — signal is clipping |
| mic.hint.handy | Для Handy выберите то же устройство в его настройках. | Pick the same device in Handy's own settings. |

---

## 15. Тестирование

### 15.1 Unit (cargo test, `omnesagent-tools`)

1. `manifest.rs`: парсинг `latest.json` (фикстура реального манифеста v0.9.6), выбор `windows-x86_64`, битые/чужие платформы, истёкший кэш.
2. `detect.rs` на моках: версии из version-resource, портативный маркер (magic-string/legacy), приоритет реестр → путь → custom.
3. SSRF-гвард: таблица хостов/IP — приватные, loopback, 169.254.*, allowlist ok/отказ, редиректы.
4. Semver-логика: update/downgrade/pin.
5. Парсер вывода `--transcribe-file --json` и `--list-models --json` (фикстуры).
6. Мок-инсталлер: fake-exe, который пишет ключ реестра в temp-куст/создаёт файлы — полный прогон job-машины (успех/таймаут/exit-code/отмена).

### 15.2 Frontend

- `flutter analyze` / `flutter test`: DTO `fromJson`, машина состояний карточки (фиктивный клиент), i18n-подстановки `%V%/%K%/%SIZE%`.

### 15.3 Ручной E2E-чеклист (Windows 11)

| # | Сценарий | Ожидание |
|---|---|---|
| 1 | Чистая машина, нет Handy: включить тумблер | Диалог → «Установить» → прогресс по фазам → состояние C, Handy в трее, `--start-hidden` |
| 2 | То же без сети | `NETWORK_ERROR`, ретраи, повтор после включения сети успешен |
| 3 | Handy уже установлен (вручную, currentUser) | статус installed, версия читается, кнопка «Обновить» при выходе новой версии |
| 4 | Handy установлен per-machine | `ELEVATION_REQUIRED`, предложение ручного обновления |
| 5 | Портативный Handy запущен | running=true, installed=false, «указать путь вручную» работает |
| 6 | Обновление через `/S /UPDATE` | данные Handy сохранены, версия выросла |
| 7 | Удаление с `delete_app_data` | ключ реестра исчез, каталог удалён, трей пуст |
| 8 | Диктовка: фокус в поле чата, Ctrl+Space | текст появляется в поле |
| 9 | Кнопка «Диктовать» в композере | toggle отрабатывает на запущенном и на незапущенном Handy (auto-launch) |
| 10 | Калибровка | вердикты соответствуют реальному уровню; dropdown содержит устройства |
| 11 | Web-клиент к локальному бэкенду | секция работает идентично |
| 12 | Web-клиент к удалённому бэкенду | бейдж LOCAL_ONLY, кнопки установки скрыты |
| 13 | Отмена на фазе Download | job отменён, `*.part` удалён, место освобождено |
| 14 | Отмена на фазе Install | кнопка заблокирована |
| 15 | Заполненный диск (<2×size) | `DISK_SPACE` до начала загрузки |

---

## 16. Этапы и критерии приёмки

### M0 — Spike-исследование (0.5–1 д) ⛳ гейт
- На чистой ВМ Windows 11: проверить `/S` установку v0.9.6 (каталог, реестр, длительность, установка WebView2), точный DN субъекта Authenticode, фактический размер установщика, наличие рабочих моделей после установки, поведение `--transcribe-file --json` и схему подписи манифеста (какой payload подписан).
- **Гейт**: все вопросы §17 закрыты, иначе пересмотр §9.4–9.5.

### M1 — Бэкенд-ядро
- `detect.rs`, `manifest.rs` + SSRF-гварды + кэш; `GET /api/voice/handy/status`, `GET …/latest`.
- Юнит-тесты §15.1 (1–4) зелёные; `cargo clippy` чист.
- **Приёмка**: статус корректен на машинах из сценариев 3–5.

### M2 — Установка/обновление/удаление
- `installer.rs` (job-машина, отмена, лимиты), `launcher.rs`, `autostart.rs`; ручной smoke `--list-models`; SSE-события; gvard `local_only`.
- **Приёмка**: сценарии 1–7, 13–15 чеклиста проходят на реальной машине.

### M3 — Frontend: секция, мастер, калибровка
- Общая `VoiceSettingsSection` в shared, подключена в desktop и web, nav-item добавлен; карточка Handy со всеми состояниями; диалог; калибровка; i18n-таблица; композер-кнопка «Диктовать».
- **Приёмка**: сценарии 8, 10–12; `flutter analyze` чист; обе копии диалога используют общий виджет (нет дублей строк UI).

### M4 — Глубокая интеграция (Mode B)
- `POST /api/voice/handy/transcribe`, `/models`; провайдер `stt.provider = "handy"` в конфиге и выборе STT; очистка tmp; метрики длительности.
- **Приёмка**: голосовое сообщение транскрибируется офлайн при отключённом интернете; `HANDY_NO_MODEL` обрабатывается.

### M5 — Полировка
- Документация (`docs/`): пользовательская инструкция + troubleshooting; телеметрия-логи job; обработка кейса «Handy обновился сам» (детект расхождения версий при старте).
- **Приёмка**: walkthrough обновлён, все чеклисты пройдены повторно.

---

## 17. Открытые вопросы (закрываются M0)

1. **Payload minisign-подписи** манифеста: подписан ли канонический Tauri-пейлоад `<url><sha256>` и возможно ли проверить его без использования самого tauri-updater (иначе — взять мини-реализацию или проверять Authenticode как обязательный шаг).
2. **Точный DN издателя** Trusted Signing-сертификата для allowlist Authenticode.
3. **Размер установщика** (для текста диалога и `max_download_bytes`) и состав предустановленных моделей.
4. **Формат `settings_store.json`** текущей версии (если решим preseed-ать настройки Handy) — в v1 preseed не делаем.
5. Поведение апдейтера Handy при параллельном обновлении нами и им самим (запретить одновременность job-мьютексом на файл-лок `~/.omnesagent/data/handy/lock`).

---

## 18. Риски

| Риск | Мера |
|---|---|
| У Handy нет публичного IPC: статус записи/текст хотим, а событий нет | Не полагаемся на внутренности: кнопка-тоггл оптимистична; транскрипты получаем только через поле ввода (Mode A) или `--transcribe-file` (Mode B). Отслеживаем upstream (Raycast-расширение намекает на интерес к API) |
| Изменение CLI/манифеста в новых версиях Handy | `pin_version` по умолчанию на проверенной версии; smoke-тест при установке; обновление pin — осознанный коммит |
| Ложные срабатывания антивируса на silent-NSIS из кэша | Кэш в `~/.omnesagent` (не %TEMP%), подписанный артефакт, Authenticode-проверка; раздел в troubleshooting |
| WebView2 отсутствует на чистой машине | Установщик ставит сам (нужна сеть в момент установки) — учитываем в таймаутах фазы Install и тексте UI |
| Пользователь ставит/удаляет Handy параллельно с нашим job | Файл-лок + перепроверка реестра перед каждой фазой; конфликты → понятные коды ошибок |
| Дублирование desktop/web диалогов расползается | Новая секция только в shared; рефактор существующих секций — вне скоупа |

---

## 19. Что осознанно не делаем (v1)

- Не бандлим установщик Handy в дистрибутив OmnesAgent (вес + лицензионная чистота бренда + дублирование апдейтеров) — только download-at-runtime с официального источника.
- Не форкаем и не модифицируем Handy; не пишем в его `settings_store.json`.
- Linux/macOS — backlog (сигнатурный путь через SIGUSR2 и .deb/AppImage описан в §3, реализация после Windows-v1).
- Не трогаем существующий облачный STT-провайдер и дуплексный голос.
