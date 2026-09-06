# План: переделка AdBot (Flutter/Dart) в клиент агента «Omnes» на бекенде ZeroClaw (Rust)

> Версия: 1.1 (Firebase → Supabase) · Дата: 2026-09-05 · Воркспейс: `C:\Projects\Omnes-agent`
> Анализ выполнен: codegraph MCP + ob2h MCP (AST-скан обоих проектов: adbot-app — 97 файлов / 679 узлов графа; zeroclaw — 1258 файлов / 40 790 узлов / 45 265 связей, циклов нет).

---

## 0. Резюме

**Что делаем:** Flutter-приложение `OmAgent-front` превращаем из «AI-контент-генератора с прямыми вызовами api.openai.com и Firebase-беком» в **клиента персонального агента**: весь интеллект, состояние и файлы переезжают на self-hosted бекенд **ZeroClaw** (`OmAgent-back`, Rust) — его HTTP/WS-gateway уже даёт чат со стримингом, сессии, multi-agent, файловый workspace, память, cron, SOP и approvals. На флаттер-стороне переписываем транспортный слой и добавляем отсутствующий функционал: **проекты, папки, файлы, историю сессий, память, расписания на бекенде**. Облачный контур самого приложения (вход Google/Apple, профили, подсказки, аватары, синхронизация настроек между устройствами) переезжает с Firebase на **Supabase** — см. §2.1 и фазу 6.

**Ключевой вывод анализа:** ZeroClaw уже реализует ~90% нужного бекенда «из коробки» — включая то, чего нет в AdBot (workspace API `list/read/mkdir/move/delete/upload`, сессии, память, cron, SOP). Rust-сторону трогать почти не придётся: план предусматривает максимум 2 точечные доработки gateway. Основной объём работ — Flutter.

**Порядок работ:** 7 фаз, от транспорта чата до чистки монетизации. Каждая фаза самодостаточна и тестируема.

---

## 1. Текущее состояние (что показал анализ)

### 1.1 AdBot (`OmAgent-front`, Flutter, Dart ≥3.4)

**Стек:** GetX (стейт/роутинг/DI), flutter_screenutil, GetStorage, Firebase (Auth: Google/Apple, Firestore, Messaging, Storage), http, syncfusion PDF, speech_to_text/flutter_tts, реклама AdMob + Unity Ads, платёжные SDK: Stripe, PayPal, Paystack, SSLCommerz, `pay` (Google/Apple Pay). i18n: en/ar/bn/hi/es.

**Архитектура:** классический GetX MVC — `lib/controller/*` (17 контроллеров) → `lib/services/api_services.dart` → `lib/views/*` (19 экранов). God Node по AST-графу — `lib/helper/local_storage.dart` (103 исходящие связи): ~40 статических геттеров/сеттеров GetStorage (счётчики freemium, ключи, профиль, язык).

**Как работает чат сейчас** (`chat_controller.dart:69` `proccessChat` → `_apiProcess`):
- сообщение добавляется в локальный `RxList<ChatMessage>`,
- `ApiServices.generateResponse2()` POST **напрямую на `api.openai.com/v1/chat/completions`** с ключом, который пользователь ввёл в настройках (`LocalStorage.getChatGptApiKey()`);
- **нет стриминга** (только полный ответ), **нет истории** (список живёт в памяти контроллера, `shareMessages` — плоская строка для share), **нет сессий/тредов**, второй вызов идёт на легаси `/v1/completions`.

**Контентные генераторы** — по одному контроллеру на фичу (content, hashtags, cover letter, diet chart, love notes, images/DALL·E), каждый дублирует паттерн «промпт → OpenAI → экран результата».

**Freemium:** локальные счётчики `textCount/imageCount/contentCount/hashTagCount` в GetStorage + лимиты из `ApiConfig` (free 1–2, premium 50–100/30 дней, $19). Обходится очисткой хранилища.

**⚠️ Критические проблемы безопасности (найдены в коде):**
- `lib/utils/config.dart`: **Stripe SECRET key и PayStack server key захардкожены в клиенте** (test-ключи, но паттерн опасен);
- ключ OpenAI хранится на устройстве и ходит напрямую в OpenAI;
- freemium-логика полностью клиентская.

**Расписания:** `set_schedule_screen` + `flutter_local_notifications` — чисто локальные, на устройство не полагаются при переустановке.

### 1.2 ZeroClaw (`OmAgent-back`, Rust 2024, v0.8.4)

**Монорепо:** 20 крейтов (`api, channels, commands, config, eval, gateway, hardware, infra, log, macros, memory, plugins, providers, relay-proto, runtime, sop-graph, spawn, tls, tool-call-parser, tools`) + приложения `apps/zerocode` (TUI), `apps/tauri` (desktop), `apps/zerorelay` + `web/` (React+Vite dashboard — **эталонный клиент, по нему копируем протокол**).

**God Nodes по AST-отчёту ob2h:** `zeroclaw-config/src/schema.rs` (1743 связи), `channels/orchestrator` (872), `apps/zerocode/src/chat.rs` (547), `runtime/sop/engine.rs`, `runtime/agent/loop_.rs`, `gateway/src/lib.rs` (319).

**Gateway — уже готовый бекенд для мобильного клиента:**

| Блок | Маршруты (фактические, из кода) |
|---|---|
| Чат | `WS /ws/chat?agent=<alias>[&session_id=][&workspaceDir=]` + `?token=`/`Authorization`/`Sec-WebSocket-Protocol` |
| Фреймы WS (→клиент) | `chunk` (дельты текста), `thinking` (reasoning), `chunk_reset`, `tool_call` (name+input), `done` (full_response), `message`, `aborted`, `error` |
| Сессии | `GET/POST/DELETE /api/sessions`, `/api/sessions/running`, `/{id}/abort`, `/{id}/state`, `/{id}/messages` |
| **Файлы workspace агента** | `/api/agents/{alias}/workspace/list`, `/read`, `/mkdir`, `/move`, `/path`, DELETE (удаление) |
| Файлы хоста | `/api/browse`, `/api/browse/mkdir`, `/api/browse/rmdir`, `/api/upload?agent=` |
| Память | `/api/memory`, `/api/memory/{key}` (CRUD) |
| Расписания | `/api/cron`, `/{id}/run`, `/{id}/runs` |
| Агенты/модели | `/api/status`, `/api/tools`, `/api/personality`, `/api/skills/bundles`, `/api/config/*` (sections/props/catalog/models) |
| События | SSE `/api/events` + `/api/events/history` |
| Автоматизация | `/ws/sops/runs`, `/admin/sop/{pending,approve,deny}` (approval-гейты), `/api/sops/runs` |
| Прочее | `/api/cost`, `/api/channels`, `/api/devices`, `/api/logs`, `/api/health`, `/api/openapi.json`, `/ws/canvas/{id}`, `/ws/nodes`, `/a2a/{alias}`, `/acp`, voice duplex |
| Авторизация | Bearer-токен шлюза (HTTP `require_auth`), pairing-флоу (`/api/pairing/initiate`, `/pair/code`, `/api/devices`, webauthn), rate-limit |

**Рантайм:** агентный цикл с инструментами (shell/browser/http/fs/…, scoped registry), память (SQLite + вектор, namespaces/tenants/agent_alias), 30+ провайдеров (Anthropic/OpenAI/Ollama/OpenRouter/любой OpenAI-compatible + fallback-цепочки), SOP-движок (триггеры webhook/cron/mqtt/peripheral, approval-гейты, возобновляемые прогоны), автономность `supervised` по умолчанию, tool-receipts, sandbox.

**Чего нет (для нашей задачи):** мультитенантных «пользователей» в SaaS-смысле (модель — один владелец + много агентов/устройств), push-уведомлений на мобильные, скачивания бинарных файлов из workspace (только текстовый `read`).

---

## 2. Целевая архитектура

```
┌─────────────────────────────┐        ┌──────────────────────────────────────────┐
│  Flutter «Omnes Agent»      │  HTTPS │  ZeroClaw Gateway (Rust)                 │
│  (Android/iOS/десктоп)      ├───────►│  /ws/chat ────────────► Agent runtime    │
│                             │  WSS   │  /api/sessions ───────►  ├─ tools (fs…)  │
│  core/gateway (WS+HTTP)     ├───────►│  /api/agents/*/workspace► ├─ memory      │
│  features: chat, projects,  │        │  /api/memory, /api/cron ► ├─ providers   │
│  files, sessions, memory,   │        │  /api/events (SSE) ────► └─ SOP/cron     │
│  cron, approvals, settings  │        │  workspace/  (файлы проектов)            │
└─────────────────────────────┘        └──────────────────────────────────────────┘
```

- **Один бекенд** — `zeroclaw gateway` (VPS или домашняя машина), TLS через `zeroclaw-tls` или reverse-proxy.
- **Аутентификация приложения** = Supabase Auth (OAuth Google/Apple — тот же UX, что в старом AdBot, но без Firebase). После входа приложение один раз получает/вводит адрес и Bearer-токен шлюза и хранит их в `flutter_secure_storage`; между устройствами они синхронизируются через `user_settings` в Supabase. Прямой ручной ввод токена и pairing `/api/pairing/initiate` — fallback/опция.
- **Проекты** = папки в workspace агента (`projects/<id>/…`) + привязка сессий; чат проекта открывает WS с `workspaceDir=<папка проекта>`, поэтому агент «видит» файлы проекта из коробки.
- **Мультиагентность:** выбор `agent alias` в клиенте (например, `writer`, `coder`, `chief`) — заменяет «модели» из старых настроек AdBot.

### 2.1 Supabase — облачный контур приложения (вместо Firebase)

Замена «один-в-один» по классам задач, без смены логики приложения:

| Было (Firebase) | Стало (Supabase) |
|---|---|
| FirebaseAuth (Google/Apple, guest) | Supabase Auth: `signInWithOAuth(google)` / `signInWithOAuth(apple)`, deep-link `com.omagent.front://login` |
| Cloud Firestore: профиль пользователя, suggested-контент, расписания | Postgres: таблицы `profiles`, `suggested_prompts`, `user_settings` (RLS: доступ только владельцу) |
| Firebase Storage (аватары, файлы профиля) | Supabase Storage: bucket `avatars` (публичное чтение, запись только владельцем по пути `<uid>/*`) |
| Firebase Messaging (push) | v1 — без push: `flutter_local_notifications` + Realtime-подписка на события в открытом приложении; v2 — опционально Edge Function + FCM, если push понадобится |

**Схема Postgres (RLS включён на всех таблицах):**
- `profiles` — `id uuid PK = auth.users.id`, `display_name`, `avatar_url`, `created_at`;
- `suggested_prompts` — `id`, `category`, `title`, `prompt`, `sort` (замена Firestore-коллекции подсказок на главном экране; запись — владельцу проекта, чтение — всем аутентифицированным);
- `user_settings` — `user_id PK`, `settings jsonb` (последний агент/проект, язык, нешифруемые настройки; gateway-токен здесь **не** хранится — он только в secure storage устройства).

**Граница ответственности:** Supabase — только identity и лёгкие данные приложения; весь AI-контент, чаты, файлы и автоматизация — на ZeroClaw gateway.

---

## 3. Инвентаризация Dart-кода: сохранить / переработать / удалить

| Категория | Файлы | Судьба |
|---|---|---|
| **Сохранить как есть** | `utils/custom_color.dart`, `custom_style.dart`, `dimensions.dart`, `assets.dart`, `res/assets_res.dart`, `Flutter Theam/themes.dart`; базовые виджеты (`widgets/buttons`, `inputs_widgets`, `appbar/*`, `custom_dropdown_widget`); `utils/language/*` (5 локалей) | UI-слой остаётся визуально прежним |
| **Переработать** (меняем источник данных) | `chat_screen.dart`, `chat_message_widget.dart`, `send_input_field.dart` → стриминг-чат; `home_screen.dart`, `drawer_screen.dart` → навигация по проектам/агентам; `settings_screen.dart` → gateway URL/token/agent/язык; `splash/login` → вход через Supabase (Google/Apple), затем проверка токена шлюза; `pdf_view_screen.dart` — оставить (просмотр файлов); `speech_to_text`/`flutter_tts` — оставить как локальный ввод/озвучку | Раздел 5, фазы 1–6 |
| **Удалить** | `services/api_services.dart` (прямые вызовы OpenAI), все платёжные (`stripe_service`, `paypal_service`, `paystack/*`, `ssl_commerz`, `views/payment_method/*`, `purchase_plan_screen`, ключи из `utils/config.dart`), реклама (`admob_helper`, `unity_ad*`, `unit_id_helper`, `work_manager`) | — |
| **Заменить на Supabase** | `firebase_options.dart` → `lib/core/supabase/supabase_config.dart`; `login_controller.dart` (FirebaseAuth) → Supabase Auth; Firestore-зависимости (`user_model`, suggested-подсказки в `chat_controller`, `schedule_history`) → таблицы `profiles`/`suggested_prompts`/`user_settings`; `firebase_storage` (аватар в `update_profile`) → Supabase Storage | §2.1, фаза 6 |
| **Заменить бекендом** | `set_schedule_screen` + `schedule_history` (локальные уведомления → `/api/cron`), счётчики freemium в `local_storage.dart` (удалить; лимиты — политика агента/токена), `support_screen` → ссылка/канал владельца | Фаза 4 |

---

## 4. Новые функции (чего в AdBot нет и что добавляем)

1. **Проекты.** Модель: `{id, name, icon/color, agentAlias, rootPath: "projects/<id>", createdAt}`. UI: список/создание/переименование/удаление; внутри проекта — три раздела: Чаты, Файлы, Заметки (заметка = `NOTES.md` в корне проекта). Реализация 100% на существующих API: workspace mkdir/list/move/delete + сессии с `workspaceDir`. Метаданные — `project.json` в папке проекта (пишется агентом через тот же API).
2. **Папки и файлы.** Файловый менеджер по `/api/agents/{alias}/workspace/*`: дерево (`list`), просмотр/редактирование текста (`read` + сохранение), создание папок (`mkdir`), перемещение/переименование (`move`), удаление, загрузка с устройства (`/api/upload`), скачивание текстовых. Отдельный режим «Файлы хоста» (`/api/browse`) — опционально, за переключателем в настройках.
3. **История сессий.** `/api/sessions` + `/{id}/messages` → список чатов с возобновлением (`WS` с `session_id`), переименование, удаление, abort (`/{id}/abort`).
4. **Память агента.** Просмотр/поиск/редактирование записей `/api/memory` (экран «Память» с поиском).
5. **Расписания на бекенде.** CRUD `/api/cron` + список прогонов `/{id}/runs` + ручной запуск `/{id}/run`. Локальные уведомления остаются только как «напоминалка открыть приложение» (опционально).
6. **Approvals.** Banner «агент ждёт подтверждения» по `/admin/sop/pending` + approve/deny — критично при autonomy=supervised.
7. **Расходы.** Экран статистики по `/api/cost`.
8. **Стриминг с reasoning и tool-карточками.** Отображение `thinking` (сворачиваемый блок), `tool_call` (карточка с именем инструмента и input), `done`.
9. **Голос.** v1: локальные STT/TTS (уже в проекте) поверх текстового чата; v2: дуплексный голос gateway (`voice_duplex`).
10. **Canvas (v2).** Совместное редактирование документа агента через `/ws/canvas/{id}`.
11. **Синхронизация между устройствами (Supabase).** Профиль и `user_settings` (последний агент/проект, язык) едут за пользователем; подсказки на главном экране — из `suggested_prompts`.

---

## 5. Фазы реализации

### Фаза 0 — Фундамент (0.5–1 день)
1. Собрать и запустить ZeroClaw: `zeroclaw quickstart`, создать агентов-алиасов (например `chief`, `writer`), включить gateway (`[gateway]` в `~/.zeroclaw/config.toml`), зафиксировать токен.
2. Проверка протокола руками: wscat к `/ws/chat?agent=chief` — убедиться в фреймах `chunk/thinking/tool_call/done`.
3. Flutter: создать ветку `omnes-agent`, обновить `pubspec.yaml` (добавить `web_socket_channel`/`socket_daemon`-совместимый клиент, `markdown_widget`, `flutter_secure_storage` для токена шлюза, `supabase_flutter`; Firebase-пакеты пока остаются до фазы 6; платёжные/рекламные можно убирать поэтапно).
4. **Критерий:** `flutter analyze` зелёный, gateway отвечает на `/api/health`.

### Фаза 1 — Транспорт и чат (ядро, 3–5 дней)
Новые файлы:
- `lib/core/gateway/gateway_config.dart` — базовый URL, token (secure storage), agent alias;
- `lib/core/gateway/gateway_http.dart` — тонкий HTTP-клиент (Bearer) поверх `http`;
- `lib/core/gateway/gateway_ws.dart` — WS-клиент: подключение `?agent=&session_id=&workspaceDir=&token=`, автопереподключение, очередь отправки, маппинг фреймов в `sealed class WsFrame {Chunk, Thinking, ChunkReset, ToolCall, Done, Message, Aborted, Error}`;
- `lib/core/gateway/models/*` — session, workspace node, memory entry, cron job (JSON по фактическим ответам gateway);
- `lib/controller/chat_controller.dart` — **переписать**: `sendMessage()` → WS, состояниеturn-стрима по образцу `web/src/contexts/turnStream.logic.ts` (накопление `pendingContent/pendingThinking`, обработка `chunk_reset`, классификация завершения: контент / только-размышления / только-инструменты);
- `chat_message_widget.dart` — markdown-рендер, блок reasoning, карточки tool_call.
**Критерий:** чат со стримингом к реальному агенту, переподключение после sleep устройства, abort.

### Фаза 2 — Агенты и сессии (2–3 дня)
- `lib/features/agents/`: выбор агента (alias) из `/api/status`; экран personality (`/api/personality`) — read-only в v1.
- `lib/features/sessions/`: список (`/api/sessions`), открытие чата с `session_id`, удаление, abort; локальный кэш списка.
- Drawer: Сессии | Проекты | Память | Расписания | Настройки.
**Критерий:** закрыл приложение → открыл → продолжил тот же тред.

### Фаза 3 — Проекты, папки, файлы (4–6 дней)
- `lib/features/projects/project_repository.dart` — CRUD поверх workspace API; конвенция `projects/<uuid>/project.json`.
- `lib/features/files/workspace_browser.dart` — дерево + список, действия: создать папку/файл, переименовать/переместить, удалить, загрузить (`multipart` на `/api/upload`), открыть текстовый в редакторе, выгрузить в PDF-вьюер/шеринг (`share_plus` остаётся).
- Экран проекта: табы Чаты (сессии, отфильтрованные по `workspaceDir`) / Файлы / Заметки (`NOTES.md`).
- Чат проекта: WS всегда с `workspaceDir=projects/<id>` → агент работает с файлами проекта нативными fs-инструментами.
**Критерий:** создал проект «Отчёт», загрузил `.md` файл, попросил агента переписать — увидел изменённый файл в приложении.

### Фаза 4 — Память, расписания, approvals, стоимость (3–4 дня)
- `lib/features/memory/`: список/поиск/правка `/api/memory`.
- `lib/features/automation/cron_screen.dart`: список задач, вкл/выкл, история прогонов, «запустить сейчас»; создание в v1 — текстом через агента (промпт-шаблон), в v2 — форма.
- Approvals banner: поллинг `/admin/sop/pending` (или `/api/events` SSE) + кнопки approve/deny.
- `lib/features/stats/`: `/api/cost` за период.
**Критерий:** созданное через приложение расписание срабатывает на сервере без открытого приложения; gated-действие одобряется из телефона.

### Фаза 5 — Голос и полировка (2–3 дня)
- STT (speech_to_text) в поле ввода, TTS-озвучка ответов (переключатель; озвучивать `done.full_response`).
- Локализация новых строк во все 5 языков (`utils/language/*`).
- Оффлайн-состояния, empty-стейты, обработка ошибок gateway (единый `GatewayException` → toast).
**Критерий:** голосовой вопрос → голосовой ответ.

### Фаза 6 — Supabase вместо Firebase, чистка и релиз (Завершена ✅)
1. Создана SQL-миграция схемы (`supabase/schema.sql`): таблицы `profiles`, `suggested_prompts`, `user_settings` с Row Level Security (RLS) и storage bucket `avatars`.
2. Dart: реализован слой `lib/core/supabase/` (`SupabaseConfig`, `ProfilesRepository`, `SettingsRepository`, `SuggestedPromptsRepository`). `LoginController` переведён на Supabase Auth (OAuth Google/Apple, Email, Guest). `UpdateProfileController` переведён на Supabase Storage и `ProfilesRepository`.
3. Полностью удалены пакеты `firebase_*` (`firebase_auth`, `cloud_firestore`, `firebase_messaging`, `firebase_storage`, `the_apple_sign_in`) из `pubspec.yaml`, удалён `firebase_options.dart`.
4. Полная чистка рекламы и платежей: удалены `unity_ads_plugin`, `google_mobile_ads`, `flutter_stripe`, `flutter_sslcommerz`, `pay`, `paystack_for_flutter`, удалены экраны оплат и старые сервисы; `PurchasePlanScreen` превращён в информационный экран «Omnes Server & System Status»; `AdManager` и `AdMobHelper` заменены на чистые no-op стабы.
5. Анализатор: `flutter analyze --no-pub` → 0 issues. Все тесты (33 теста) успешно проходят.
**Критерий:** В коде нет Firebase/Stripe/AdMob; авторизация и профили работают через Supabase; статический анализ и тесты зелёные.

### Фаза 7 (опционально) — Rust-доработки gateway
1. `GET /api/agents/{alias}/workspace/raw?path=` (Реализовано ✅) — отдача сырых бинарных файлов (картинки, PDF, аудио) с автоматическим определением MIME-типа через `mime_guess` и заголовком `Content-Disposition`. Во Flutter-клиент добавлены `workspaceRawFileBytes`, `downloadRawFile` и `getRawFileUrl`.
2. Long-lived device tokens: пересмотреть TTL pairing-токенов для мобильных устройств (или выпуск персонального токена из admin paircode).

---

## 6. Ключевые контракты (шпаргалка для Dart-разработки)

| Действие в UI | Вызов |
|---|---|
| Подключиться к чату агента | `WS {base}/ws/chat?agent=<alias>&session_id=<sid>&workspaceDir=<dir>&token=<t>` |
| Отправить сообщение | WS-сообщение `{type:"message", content:"..."}` (оптимистично в UI) |
| Принять ответ | фреймы `chunk`/`thinking` → склейка; `chunk_reset` → сброс буферов; `tool_call` → карточка; `done.full_response` → финал turn |
| Список чатов | `GET /api/sessions` |
| История чата | `GET /api/sessions/{id}/messages` |
| Прервать генерацию | `POST /api/sessions/{id}/abort` |
| Файлы проекта | `GET /api/agents/{a}/workspace/list?path=`, `GET .../read?path=`, `POST .../mkdir`, `POST .../move`, `DELETE .../workspace?path=` |
| Загрузить файл | `POST /api/upload?agent=<a>` (multipart) |
| Память | `GET/PUT/DELETE /api/memory[/{key}]` |
| Расписания | `GET/POST/DELETE /api/cron`, `POST /{id}/run`, `GET /{id}/runs` |
| Ожидает одобрения | `GET /admin/sop/pending` → `POST /admin/sop/approve|deny` |
| Статус/агенты/инструменты | `GET /api/status?`, `GET /api/tools` |
| Живые события | SSE `GET /api/events` |

Ссылка на эталон реализации клиента: `OmAgent-back/web/src/` (`lib/ws.ts`, `lib/sse.ts`, `lib/api.ts`, `contexts/turnStream.logic.ts`).

---

## 7. Риски и решения

| Риск | Решение |
|---|---|
| Токен шлюза на устройстве = полный доступ | v1: отдельный «мобильный» агент-алиас с ограниченным risk-profile/allowed_tools; v2: pairing + device-токены (Фаза 7) |
| WS умирает в фоне (мобильные ОС) | Экспоненциальный reconnect + догрузка истории `/{id}/messages` после reconnect; push — вне скоупа v1 |
| Бинарные файлы в workspace | Фаза 7.1 (`workspace/raw`); до того — только тексты и upload |
| Переход с Firebase на Supabase в логине | OAuth-провайдеры те же (Google/Apple) — UX входа не меняется; переноса старых пользователей нет (аудитория новая); splash-цепочка: сессия Supabase → gateway-конфиг из `user_settings`/secure storage → `GET /api/health` |
| Разрастание скоупа | Фикс-приоритет: чат → сессии → проекты/файлы → всё остальное |

---

## 8. Чек-лист приёмки (сквозной сценарий)

1. Установил приложение → вошёл через Google/Apple (Supabase) → указал URL и токен шлюза (один раз; на других устройствах подтянется из `user_settings`) → увидел агентов.
2. Создал проект «Диплом», папки `черновики`, `источники`; загрузил 2 файла.
3. Написал в чат проекта «собери план по файлам из источников» → агент прочитал файлы (tool-карточки видны), ответил со стримингом, положил `план.md` в проект — файл виден в разделе Файлы.
4. Закрыл приложение, открыл через час — история чата и файлов на месте.
5. Создал расписание «каждый день в 9:00 — сводка новостей в проект Дневник» — утром результат появился в чате проекта.
6. Агент запросил подтверждение рискованного действия — баннер одобрения пришёл в приложение, действие одобрено.
7. Экран «Стоимость» показывает расход токенов за неделю.
8. В APK нет `firebase_*`/Stripe/AdMob (вместо Firebase — `supabase_flutter`); `flutter analyze` и `ob2h project scan adbot-app` (проверка циклов) зелёные.
