# План: удаление Android, отдельный web-клиент копированием десктопа, автоустановка на VPS

Дата: 07.09.2026 · Статус: ПЛАНИРУЕТСЯ (не к исполнению сейчас)

## Ключевые решения

1. **Android-приложение убирается полностью** — из проекта удаляются все мобильные клиенты и android/ios-обвязка. Мобильный доступ к агенту идёт через **веб-версию в браузере телефона** (адаптивный UI), а не через APK.
2. **Десктоп остаётся десктопом.** `frontend/desktop` — полноценное Windows-приложение (Flutter ADE, готовность ~85%) — **не конвертируется и не ломается**: `webview_windows`, локальный `dart:io`-терминал, `windows/`-раннер и сборка `flutter build windows` продолжают жить в нём как есть.
3. **Web — отдельный проект, созданный КОПИРОВАНИЕМ десктопа.** Десктоп копируется в новую папку `frontend/web`, и уже **эта копия** переделывается под полноценное веб-приложение (`flutter build web`). Старый `frontend/web` (веб-скаффолд мобильного клиента) удаляется вместе с мобильным приложением — на его месте встаёт новый проект-копия.
4. **Общее ядро — `frontend/shared`** (GatewayHttpClient, WS/SSE, модели, i18n RU/EN, токены темы): переиспользуется и десктопом, и веб-копией без дублирования.
5. **Автоустановка на VPS**: один установщик, который сам прописывает nginx, домен (имя от пользователя или fallback `ip:port`), SSL, systemd-сервис шлюза, первичный вход и **принудительную смену пароля после первого захода**.

## Целевая структура

```text
frontend/
├── shared/         # Общее ядро: gateway http/ws/sse, модели, локализация, тема
├── desktop/        # Windows ADE — ОСТАЁТСЯ ДЕСКТОПОМ (без изменений цели)
├── web/            # ★ КОПИЯ desktop/ → переделывается под flutter build web
├── (android/)      # ✖ УДАЛИТЬ   (было у корневого omagent_front)
├── (ios/)          # ✖ УДАЛИТЬ
├── (mobile/)       # ✖ УДАЛИТЬ   (мобильный клиент после сплита)
└── (supabase/)     # ✖ УДАЛИТЬ   (мобильный бэк-аутентификатор)
```

---

## Этап 1 — Удаление Android/мобильного клиента

- [ ] Выделить удалённый мобильный клиент в архивный тег/ветку (`archive/mobile-android-2026-09`) — история git сохраняется, физически из main всё уходит.
- [ ] Удалить: `frontend/android/`, `frontend/ios/`, `frontend/mobile/`, `frontend/supabase/`, старый `frontend/web/` (скаффолд мобильного), корневой мобильный `frontend/pubspec.yaml` (`omagent_front`) и `frontend/lib` (мобильный код), `flutter_launcher_icons.yaml`, мобильные скрипты.
- [ ] Вычистить мобильные зависимости из CI/сборки (`build.ps1`, `check.ps1`, `scripts/`): `flutter_screenutil`, `permission_handler`, `image_picker`, `saver_gallery`, `speech_to_text`, `flutter_tts`, `flutter_inappwebview`, `webview_flutter`, `supabase_flutter`, `syncfusion_pdf*`, `device_info_plus`, `share_plus`, `flutter_local_notifications` — все они только в мобильном pubspec и уходят вместе с ним.
- [ ] Обновить `.gitignore`, `AGENTS.md`/`CLAUDE.md` (monorepo-layout: frontend = shared + desktop + web; убрать Android/iOS).
- [ ] Критерий готовности: в репозитории нет ни одного `android/`, `ios/`, `supabase/`; `flutter analyze` в `desktop` и `shared` чистый.

## Этап 2 — Создание web-проекта копированием десктопа

### 2.0 Копирование (чистый старт отдельного приложения)
- [ ] `cp -r frontend/desktop frontend/web` — копия целиком (lib/, assets/, web/, pubspec).
- [ ] В копии: переименовать пакет `omnes_desktop` → `omnes_web` (pubspec + все `package:omnes_desktop/...` импорты, если есть самоссылки); обновить `name`/`description` в `web/index.html` и `manifest.json` (PWA-имя «OmnesAgent Web»).
- [ ] В копии удалить папку `windows/` (раннер, Runner.rc, app_icon.ico) — десктопная сборка в web-проекте не нужна; добавить платформy только web (`flutter create --platforms=web .` для свежих веб-артефактов поверх).
- [ ] `frontend/desktop` после копирования **не трогаем** — он и дальше развивается как Windows-приложение.

### 2.1 Платформенные блокеры — правятся ТОЛЬКО в копии `frontend/web`
| Файл в копии | Проблема на web | Решение в копии |
|---|---|---|
| `lib/features/inspector/inspector_panel.dart`, `lib/features/inspector/browser/desktop_webview_controller.dart` | `webview_windows` (WebView2) не существует в браузере | Live Browser → **iframe-панель** с деградацией (сайты с `X-Frame-Options` → кнопка «Открыть в новой вкладке»); Element Picker вынести на бэкенд (browser automation уже есть в `omnesagent-tools`). Зависимость `webview_windows` из `web/pubspec.yaml` удалить |
| `lib/features/terminal/desktop_terminal_service.dart` | `dart:io Process.start` запрещён в dart2js | Терминал = **PTY-over-WebSocket на шлюзе**: новый эндпоинт gateway `/ws/terminal/{session}` (spawn pty на сервере, stdin/stdout-кадры, resize). В desktop-копии ничего не меняется — там остаётся локальный Process |
| Оконные контролы (`— □ ✕` в навбарах) | в браузере бессмысленны | В копии удалить (не прятать — в копии можно править открыто) |

### 2.2 Адаптация UI в копии под браузер и телефоны
- [ ] Трёхпанельный ADE-layout сделать адаптивным: ≥1280px — сплиттер как в десктопе; планшет — 2 панели; телефон — одна панель + нижняя навигация (это заменяет Android-приложение).
- [ ] Убрать остатки `ScreenUtil`-координат, если притащились из старого кода (см. `PLAN_DESKTOP_MOBILE_SPLIT.md` п.1).
- [ ] Горячие клавиши (`Ctrl+K`, `Shift+Tab`, `⌘J`) оставить; на тач-устройствах продублировать кнопками.
- [ ] `GetStorage` на web = localStorage: проверить лимиты (~5 МБ) для истории чатов; тяжёлые данные держать на сервере через `/api/sessions` (уже синхронизируется).
- [ ] Базовый URL шлюза: клиент берёт **origin страницы** через `--dart-define=GATEWAY_BASE_URL=/` + чтение `window.__OMNESAGENT_BASE__` (шлюз уже инжектит его в SPA-fallback, см. `static_files.rs`) — убрать захардкоженный `127.0.0.1:42617` в веб-сборке.

### 2.3 Сборка и раздача
- [ ] `flutter build web --release --web-renderer canvaskit` из `frontend/web/` → артефакты в `frontend/web/build/web`.
- [ ] Научить `install.sh` / VPS-установщик собирать web-проект и раскладывать артефакты в `web_dist` шлюза (фича `embedded-web` include_dir — по желанию).
- [ ] Проверить, что SPA-fallback шлюза (`/_app/*`, index.html) отдаёт копию корректно, WS goes через тот же путь.
- [ ] PWA-манифест + иконки (`frontend/web/web/`) для «Установить» с телефона.

### 2.4 Контроль расхождения (desktop ↔ web)
- [ ] Всё, что не зависит от платформы (модели, клиенты шлюза, i18n, токены) — держать **только в `frontend/shared`**, правки там автоматически идут обоим.
- [ ] Правила дрейфа: общие фиксы UI портировать между desktop и web отдельными коммитами с префиксом `[port]`; раз в спринт — чек-лист паритета экранов.
- [ ] CI: сборка `flutter build windows` (desktop) и `flutter build web` (web) + analyze обоих — обязательны.

Критерий готовности этапа: копия собирается `flutter build web` без ошибок; в хроме открывается с того же origin, что API; чат, сессии, конфиг, SOP Studio, Canvas работают против живого шлюза; терминал работает через PTY-WS; при этом `flutter build windows` в `frontend/desktop` по-прежнему зелёный.

## Этап 3 — Первичный вход и смена пароля (добавить в шлюз)

Сейчас у шлюза нет парольной аутентификации (pairing/токены/WebAuthn). Для установки «как сервера» добавляем:

- [ ] **Учётка администратора**: `admin_auth.json` — argon2id-хеш пароля, флаги `password_hash`, `must_change_password`, `created_at`.
- [ ] **Эндпоинты** в gateway: `POST /api/auth/login` (→ HttpOnly session-cookie + CSRF-token), `POST /api/auth/logout`, `POST /api/auth/change-password`, `GET /api/auth/me` (возвращает `must_change_password`).
- [ ] **Middleware**: все `/api/*`, `/ws/*`, `/_app/*` (кроме `/api/auth/login`, `/health`) требуют сессию; при `must_change_password=true` любой запрос, кроме login/change-password/logout/me, отдаёт 403 с кодом `password_change_required`.
- [ ] **Rate-limit** на login через существующий `auth_rate_limit.rs`.
- [ ] **Во фронтенде (копии `frontend/web`)**: экран логина + экран «Сменить пароль» (старый → новый → повтор, требования длины), показываются до доступа к рабочей области; сессия переживает F5 (cookie).
- [ ] **Первый заход**: если пароля нет — установщик генерирует временный, пишет в `.install-credentials` (chmod 600, одна выдача), печатает в консоль; фронт при первом входе обязывает сменить.

## Этап 4 — Автоматическая установка на VPS (единый скрипт)

Новый `scripts/vps_install.sh` (может вызывать существующий `backend/install.sh`). Запуск: `curl -fsSL .../vps_install.sh | sudo bash -s -- [--domain NAME] [--port N] [--email ...]`.

### 4.1 Параметры от пользователя (интерактив или флаги)
- Домен **или** подтверждение работы на `ip:port` (если домен не задан).
- Порт публичного входа (по умолчанию 80/443; для ip-режима — например 8443).
- Email для Let's Encrypt (только в доменном режиме).

### 4.2 Что скрипт делает автоматически
1. Зависимости: rust toolchain + сборка шлюза (`cargo build --release --features embedded-web`), Flutter SDK + **сборка `frontend/web` (flutter build web)**, nginx, certbot.
2. Размещение: бинарь в `/opt/omnesagent/`, веб-артефакты копии в `web_dist` шлюза, данные в `/var/lib/omnesagent/`.
3. Конфиг шлюза: `gateway.host=127.0.0.1`, порт 42617, `web_dist_dir`, путь данных — через `.env`/`config.toml`.
4. **systemd-юнит** `omnesagent.service` (Restart=always, отдельный пользователь, sandbox) + включение.
5. **nginx** из шаблона с параметрами:
   - `server_name` = домен или `_` (ip:port-режим);
   - `location /` → `proxy_pass http://127.0.0.1:42617` (шлюз сам раздаёт SPA+API — один источник статики, без дублирования в nginx);
   - `location /ws/` + `Upgrade`-заголовки; `client_max_body_size`; gzip/brotli;
   - доменный режим: certbot obtain+install --redirect, авто-продление; ip-режим: self-signed + предупреждение (или HTTP-only).
6. Firewall: ufw/firewalld allow 80,443 (+выбранный порт), лишнего не открывать; опционально fail2ban на SSH/login.
7. **Инициализация учётки** (Этап 3): генерация временного пароля, запись `.install-credentials`, `must_change_password=true`.
8. **Самопроверка**: `systemctl is-active`, `curl -f localhost/_app/index.html`, `curl -f localhost/api/health`, не-авторизованный `/api/sessions` → 401, `/` → SPA с логином.
9. **Вывод**: URL входа, временный пароль, где сменить; команда обновления `omnesagent-update.sh` (git pull → пересборка шлюза и `frontend/web` → `systemctl restart` → `nginx -t && nginx -s reload`).

### 4.3 Идемпотентность и обновления
- [ ] Повторный запуск = режим обновления: не пересоздавать пароль, не затирать данные/конфиг, `nginx -t` перед reload с откатом при ошибке.
- [ ] Роллбэк: `/opt/omnesagent/releases/<sha>/` + симлинка `current` (бинарь и веб-артефакты парами).

---

## Порядок и связи

```text
Этап 1 (удаление Android)   — независим, можно сразу
Этап 2 (копия desktop→web)  — первый подэтап: копирование и сборка;
                              терминал-PTY и iframe — дальше; auth-экраны — после Этапа 3
Этап 3 (auth в шлюзе)       — независим от фронтенда (backend)
Этап 4 (VPS-инсталлер)      — после 2 и 3 (собирает всё вместе)
```

Оценочная трудоёмкость: Этап 1 — 0.5 дня; Этап 2 — 3–4 дня (копирование дёшево, тяжёлые части — PTY-терминал и адаптивный layout); Этап 3 — 2–3 дня; Этап 4 — 2–3 дня.

## Риски
- **Дрейф desktop ↔ web** — две кодовые базы UI расходятся со временем. Митигация: всё платформенно-независимое в `shared/`, правило `[port]`-коммитов, CI-сборка обоих, периодический чек-лист паритета.
- `webview_windows`-сценарии (Element Picker, полноценный Live Browser) на web слабее из-за X-Frame-Options/CSP чужих сайтов — компенсируем серверным браузером (`omnesagent-tools` уже умеет) и кнопкой «открыть в новой вкладке».
- Flutter Web на телефоне тяжелее нативного APK (первая загрузка ~2–4 МБ) — лечится canvaskit-кэшем, PWA и brotli.
- Десктоп после копирования развивается независимо: любые правки, которые должны попасть в web, обязаны портироваться явно — молчаливое «и так же обновилось» исключено.
