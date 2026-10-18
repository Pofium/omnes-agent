# 📋 Детальный план действий по результатам комплексного аудита OmnesAgent ADE

> **Основание**: [omnesagent_desktop_audit.md](file:///c:/Projects/Omnes-agent/omnesagent_desktop_audit.md)  
> **Дата составления**: 2026-10-02  
> **Статус**: Готов к реализации  
> **Ключевой приоритет**: Устранение мертвого кода, циклических зависимостей, декомпозиция God-объектов и **100% покрытие подсветкой синтаксиса в редакторе всех языков, распознаваемых AST-сканером**.

---

## 🎯 Содержание

1. [Сводная таблица проблем и приоритетов](#1-сводная-таблица-проблем-и-приоритетов)
2. [Подсветка синтаксиса: Синхронизация редактора с AST-сканером (18 языков)](#2-подсветка-синтаксиса-синхронизация-редактора-с-ast-сканером-18-языков)
3. [Мертвый код: Анализ 4 365 кандидатов и план очистки](#3-мертвый-код-анализ-4-365-кандидатов-и-план-очистки)
4. [Циклические зависимости: Локализация и устранение](#4-циклические-зависимости-локализация-и-устранение)
5. [Декомпозиция God-объектов (Frontend & Backend)](#5-декомпозиция-god-объектов-frontend--backend)
6. [Безопасность и конфигурация: Устранение хардкода токенов](#6-безопасность-и-конфигурация-устранение-хардкода-токенов)
7. [Пошаговый график реализации (Roadmap)](#7-пошаговый-график-реализации-roadmap)
8. [Критерии приёмки и валидация](#8-критерии-приёмки-и-валидация)

---

## 1. Сводная таблица проблем и приоритетов

| ID | Область | Проблема | Текущее состояние | Целевое состояние | Приоритет |
|---|---|---|---|---|---|
| **P0-1** | **Editor Highlighting** | Редактор не подсвечивает все языки AST-сканера | Поддерживаются ~10 языков; отсутствуют C#, Swift, Ruby, Lua, Elixir, Scala, Svelte, Vue SFC | Все 18 языковых семейств AST-сканера подсвечиваются через Tree-Sitter (`inkjet`) | 🔴 **Критический** |
| **P0-2** | **Security** | Хардкод дефолтного токена `GatewayConfig.defaultToken` | `"omnes-token-secret-12345"` в открытом коде | Генерация CSPRNG-токена в `gateway.json`, передача через ENV/файл | 🔴 **Критический** |
| **P1-1** | **Architecture** | God Object `DesktopTaskWorkspaceController` | **2 987 строк** в одном классе (сессии, WS, SSE, терминал, Git) | Разделение на 5 модульных контроллеров | 🟠 **Высокий** |
| **P1-2** | **Architecture** | God Object `GatewayHttpClient` | **2 341 строка**, 80+ смешанных REST-эндпоинтов | Выделение фасадных доменных клиентов (`SessionClient`, `ConfigClient`...) | 🟠 **Высокий** |
| **P2-1** | **Dead Code** | 4 365 кандидатов мертвого кода, 30+ в `buffer.rs` | Накопленные неиспользуемые методы после миграции из Floem/Lapce | Аудит in-degree, удаление сиротских структур, скрытие приватных API | 🟡 **Средний** |
| **P2-2** | **Cycles** | Циклические зависимости в графе вызовов | `new ↔ with_limit` (editor), `cached_schema ↔ schema_body_value` | Однонаправленный поток управления | 🟡 **Средний** |

---

## 2. Подсветка синтаксиса: Синхронизация редактора с AST-сканером (18 языков)

### 2.1. Анализ текущего разрыва (Gap Analysis)

AST-сканер KAG ([`backend/crates/omnesagent-kag/src/ast/mod.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-kag/src/ast/mod.rs)) поддерживает **18 языковых групп** (расширения: `rs`, `py`, `ts`, `tsx`, `js`, `jsx`, `go`, `sql`, `c`, `cpp`, `cc`, `cxx`, `h`, `hpp`, `hxx`, `php`, `dart`, `java`, `cs`, `kt`, `kts`, `swift`, `rb`, `vue`, `svelte`, `scala`, `lua`, `ex`, `exs`, `sh`, `bash`).

При этом серверный редактор [`omnesagent-editor`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-editor):
1. В `Cargo.toml` не включает соответствующие фичи библиотеки `inkjet 0.11`.
2. В [`language.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-editor/src/language.rs) (`language_id_from_path`) не сопоставляет расширения `.cs`, `.swift`, `.rb`, `.vue`, `.svelte`, `.scala`, `.lua`, `.ex`, `.exs`.
3. В [`highlight.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-editor/src/highlight.rs) (`inkjet_language`) отсутствуют соответствующие ветки `match`.

### 2.2. Матрица покрытия языков

| # | Язык | Расширения в AST-сканере | Статус в AST | Статус в Editor сейчас | Нужная фича `inkjet` | Целевой `inkjet::Language` |
|---|---|---|---|---|---|---|
| 1 | **Rust** | `.rs` | ✅ Полный AST | ✅ Подсвечивается | `language-rust` | `Language::Rust` |
| 2 | **Python** | `.py` | ✅ Полный AST | ✅ Подсвечивается | `language-python` | `Language::Python` |
| 3 | **TypeScript** | `.ts`, `.mts`, `.cts` | ✅ Полный AST | ✅ Подсвечивается | `language-typescript` | `Language::Typescript` |
| 4 | **TSX** | `.tsx` | ✅ Полный AST | ✅ Подсвечивается | `language-tsx` | `Language::Tsx` |
| 5 | **JavaScript** | `.js`, `.mjs`, `.cjs` | ✅ Полный AST | ✅ Подсвечивается | `language-javascript` | `Language::Javascript` |
| 6 | **JSX** | `.jsx` | ✅ Полный AST | ✅ Подсвечивается | `language-jsx` | `Language::Jsx` |
| 7 | **Go** | `.go` | ✅ Полный AST | ✅ Подсвечивается | `language-go` | `Language::Go` |
| 8 | **SQL** | `.sql` | ✅ Схемы и связи | ✅ Подсвечивается | `language-sql` | `Language::Sql` |
| 9 | **PHP** | `.php` | ✅ Полный AST | ✅ Подсвечивается | `language-php` | `Language::Php` |
| 10 | **Dart** | `.dart` | ✅ Полный AST | ✅ Подсвечивается | `language-dart` | `Language::Dart` |
| 11 | **Java** | `.java` | ✅ Полный AST | ✅ Подсвечивается | `language-java` | `Language::Java` |
| 12 | **C#** | `.cs` | ✅ Полный AST | ✅ Подсвечивается | `language-c-sharp` | `Language::CSharp` |
| 13 | **C / C++** | `.c`, `.cpp`, `.cc`, `.cxx`, `.h`, `.hpp`, `.hxx` | ✅ Полный AST | ✅ Подсвечивается | `language-c`, `language-cpp` | `Language::C`, `Language::Cpp` |
| 14 | **Kotlin** | `.kt`, `.kts` | ✅ Полный AST | ✅ Подсвечивается | `language-kotlin` | `Language::Kotlin` |
| 15 | **Swift** | `.swift` | ✅ Полный AST | ✅ Подсвечивается | `language-swift` | `Language::Swift` |
| 16 | **Ruby** | `.rb` | ✅ Полный AST | ✅ Подсвечивается | `language-ruby` | `Language::Ruby` |
| 17 | **Vue** | `.vue` | ✅ SFC script блок | ✅ Подсвечивается | `language-html` / injection | `Language::Html` + JS/TS inject |
| 18 | **Svelte** | `.svelte` | ✅ SFC script блок | ✅ Подсвечивается | `language-svelte` | `Language::Svelte` |
| 19 | **Scala** | `.scala` | ✅ Полный AST | ✅ Подсвечивается | `language-scala` | `Language::Scala` |
| 20 | **Lua** | `.lua` | ✅ Полный AST | ✅ Подсвечивается | `language-lua` | `Language::Lua` |
| 22 | **Shell / Bash** | `.sh`, `.bash` | ✅ Функции/source | ✅ Подсвечивается | `language-bash` | `Language::Bash` |
| 23 | **HTML** | `.html`, `.htm` | ⏳ В планах (Фаза 6) | ✅ Подсвечивается | `language-html` | `Language::Html` |
| 24 | **CSS / SCSS** | `.css`, `.scss` | ⏳ В планах (Фаза 6) | ✅ Подсвечивается | `language-css`, `language-scss` | `Language::Css`, `Language::Scss` |
| 25 | **Protocol Buffers** | `.proto` | ⏳ В планах (Фаза 6) | ❌ Требуется | `language-protobuf` | gRPC/Proto AST |
| 26 | **GraphQL** | `.graphql`, `.gql` | ⏳ В планах (Фаза 6) | ❌ Требуется | `language-graphql` | GraphQL Schema/Query |
| 27 | **Terraform / HCL** | `.tf`, `.hcl` | ⏳ В планах (Фаза 6) | ✅ Подсвечивается | `language-hcl` | IaC Resources/Modules |
| 28 | **Docker / Compose** | `Dockerfile`, `*.yml` | ⏳ В планах (Фаза 6) | ✅ Подсвечивается | `language-dockerfile`, `language-yaml` | Services/Ports/Volumes |
| 29 | **Пакетные манифесты** | `Cargo.toml`, `package.json`... | ⏳ В планах (Фаза 6) | ✅ Подсвечивается | `language-toml`, `language-json` | Граф зависимостей `DEPENDS_ON` |
| 30 | **Jupyter Notebooks** | `.ipynb` | ⏳ В планах (Фаза 6) | ❌ Требуется | JSON Code-cells | Python в ячейках блокнота |
| 31 | **Solidity** | `.sol` | ⏳ В планах (Фаза 6) | ❌ Требуется | `language-solidity` | Contracts/Interfaces/Events |
| 32 | **Zig** | `.zig` | ⏳ В планах (Фаза 6) | ❌ Требуется | `language-zig` | Structs/Functions/Imports |

### 2.3. План технических изменений редактора

#### Шаг 1: Обновление манифеста зависимостей
Файл: [`backend/crates/omnesagent-editor/Cargo.toml`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-editor/Cargo.toml)
- Добавить в список фичей `inkjet`:
  ```toml
  inkjet = { version = "0.11", default-features = false, features = [
      "language-rust", "language-dart", "language-json", "language-yaml",
      "language-toml", "language-bash", "language-python", "language-typescript",
      "language-tsx", "language-javascript", "language-jsx", "language-html", "language-css",
      "language-scss", "language-cpp", "language-c", "language-go",
      "language-dockerfile", "language-diff", "language-sql", "language-hcl",
      "language-java", "language-kotlin", "language-php",
      # Новые фичи под AST-сканер:
      "language-c-sharp", "language-swift", "language-ruby", "language-lua",
      "language-elixir", "language-scala", "language-svelte",
  ] }
  ```

#### Шаг 2: Расширение детектора языков
Файл: [`backend/crates/omnesagent-editor/src/language.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-editor/src/language.rs)
- Добавить маппинг расширений:
  - `"cs" => "csharp"`
  - `"cxx" | "hxx" | "cc" | "hh" | "hpp" | "cpp" => "cpp"`
  - `"swift" => "swift"`
  - `"rb" => "ruby"`
  - `"lua" => "lua"`
  - `"ex" | "exs" => "elixir"`
  - `"scala" => "scala"`
  - `"svelte" => "svelte"`
  - `"vue" => "vue"`

#### Шаг 3: Диспетчеризация Tree-Sitter в движке подсветки
Файл: [`backend/crates/omnesagent-editor/src/highlight.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-editor/src/highlight.rs)
- В функцию `inkjet_language(language_id: &str) -> Option<inkjet::Language>` добавить:
  ```rust
  "csharp" => L::CSharp,
  "swift" => L::Swift,
  "ruby" => L::Ruby,
  "lua" => L::Lua,
  "elixir" => L::Elixir,
  "scala" => L::Scala,
  "svelte" => L::Svelte,
  "vue" => L::Html, // С SFC-инъекциями JS/TS
  ```
- Расширить `class_id_for_capture` для специфичных токенов Swift (`type.qualifier`), Elixir (`atom`, `symbol`), Ruby (`symbol`), C# (`keyword.type`).

#### Шаг 4: Валидация интеграции с клиентом
Файлы:
- [`frontend/shared/lib/model/editor/render_plan.dart`](file:///c:/Projects/Omnes-agent/frontend/shared/lib/model/editor/render_plan.dart)
- [`frontend/desktop/lib/features/inspector/editor/editor_viewport.dart`](file:///c:/Projects/Omnes-agent/frontend/desktop/lib/features/inspector/editor/editor_viewport.dart)
- Проверить, что `styleTable` содержит все необходимые цвета токенов для новых грамматик, а `EditorRow.runs` корректно отрисовывает UTF-16 чанки без артефактов на кириллических комментариях.

---

## 3. Мертвый код: Анализ 4 365 кандидатов и план очистки

### 3.1. Структура кандидатов
Аудит выявил 4 365 символов с нулевым входящим графом вызовов (`in-degree = 0`). Они делятся на 4 категории:

1. **Библиотечные экспорты (False Positives)**:
   - Публичные API в Rust-крейтах (`omnesagent-tools`, `omnesagent-runtime`, `omnesagent-kag`), предназначенные для внешних потребителей или плагинов.
   - **Действие**: Разметить директивами `#[allow(dead_code)]` или сузить видимость до `pub(crate)`.
2. **Floem / Lapce Legacy в `omnesagent-editor`**:
   - В [`buffer.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-editor/src/buffer.rs) перенесены низкоуровневые вспомогательные функции манипуляции курсором, мульти-селектами и снапшотами, которые не вызываются из WebSocket протокола.
   - **Действие**: Ревизия всех 30+ методов `EditorBuffer` с удалением неиспользуемого балласта.
3. **Дублирующиеся и устаревшие файлы во фронтенде**:
   - Упомянутые в аудите `LocalStorage`, `LoginController`, `LockScreen`, оставшиеся от старых прототипов.
   - **Действие**: Проверить и удалить неиспользуемые импорты, консолидировать сервисы хранения в `frontend/shared/lib/helper/local_storage.dart`.
4. **Тестовые утилиты**:
   - Функции фикстур, моков и бенчмарков, скомпилированные без флага `#[cfg(test)]`.
   - **Действие**: Обернуть все тестовые вспомогательные модули в `#[cfg(test)]`.

### 3.2. Автоматизированный конвейер очистки
1. Запуск `cargo udeps --all-targets` в директории `backend/` для выявления неиспользуемых зависимостей в `Cargo.toml`.
2. Выявление неиспользуемых функций через Rust Compiler Warnings:
   - Временное удаление глобальных `#![allow(dead_code)]` в `lib.rs` шлюза и крейтов.
   - Сбор лога предупреждений `cargo check --message-format=json`.
3. Запуск Dart/Flutter анализатора:
   - `flutter analyze --fatal-infos` в `frontend/desktop` и `frontend/shared`.
   - Удаление неиспользуемых импортов и приватных полей.

---

## 4. Циклические зависимости: Локализация и устранение

### 4.1. Цикл 1: `EditorService::new ↔ EditorService::with_limit`
- **Файл**: [`backend/crates/omnesagent-editor/src/service.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-editor/src/service.rs)
- **Причина**: Взаимные вызовы конструкторов или синглтонов в графе AST:
  - `new()` вызывает `with_limit(crate::MAX_BUFFER_BYTES)`.
  - В тестах или обертках встречался вызов `with_limit` с переопределением через `Self::new()`.
- **Решение**:
  - Строго зафиксировать канонический первичный конструктор:
    ```rust
    impl EditorService {
        /// Первичный канонический конструктор
        pub fn with_limit(max_buffer_bytes: u64) -> Self {
            Self {
                registry: Mutex::new(Registry {
                    by_id: HashMap::new(),
                    by_path: HashMap::new(),
                }),
                roots: Mutex::new(Vec::new()),
                next_buffer_id: AtomicU64::new(1),
                max_buffer_bytes,
            }
        }

        /// Вторичный делегирующий конструктор (new -> with_limit)
        pub fn new() -> Self {
            Self::with_limit(crate::MAX_BUFFER_BYTES)
        }
    }
    ```
  - Исключить любые обратные ссылки из вспомогательных методов на `new`.

### 4.2. Цикл 2: `cached_schema ↔ schema_body_value`
- **Файл**: [`backend/crates/omnesagent-gateway/src/api_config.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-gateway/src/api_config.rs)
- **Причина**: Взаимная зависимость вычисления хэша ETag / схемы конфигурации и статического кэша `OnceLock`.
- **Решение**:
  - Выделить чистую функцию генерации схемы `generate_config_schema() -> serde_json::Value`.
  - Сделать `cached_schema()` единственным владельцем `OnceLock`, вызывающим генератор строго сверху вниз.
  - Устранить любые перекрестные вызовы из тестовых модулей.

---

## 5. Декомпозиция God-объектов (Frontend & Backend)

### 5.1. Рефакторинг `DesktopTaskWorkspaceController` (2 987 строк)
Файл: [`frontend/desktop/lib/features/workspace/task_workspace_controller.dart`](file:///c:/Projects/Omnes-agent/frontend/desktop/lib/features/workspace/task_workspace_controller.dart)

Текущий контроллер перегружен 5 различными зонами ответственности.  
**План декомпозиции на суб-контроллеры / миксины**:

```
DesktopTaskWorkspaceController (Тонкий фасад / Координатор)
├── WorkspaceSessionManager (Управление вкладками TaskSession, активная сессия, персистенция GetStorage)
├── WorkspaceStreamCoordinator (Подключение WebSocket, SSE, разбор фреймов GatewayFrame)
├── WorkspaceTerminalController (Встроенный Process, stdout/stdin буфер, очистка, resize)
├── WorkspaceGitService (Чтение ветки .git/HEAD, сбор diff-статистики, additions/deletions)
└── WorkspaceMessageDispatcher (Отправка промптов, Abort, HITL Approval-ответы, retry)
```

**Этапы выполнения**:
1. Выделить `WorkspaceSessionManager`: перенести структуры `TaskSession`, логику создания, закрытия, переключения и сериализации вкладок.
2. Выделить `WorkspaceStreamCoordinator`: инкапсулировать `GatewayWsClient`, слушатели `GatewayFrame`, логику auto-reconnect и fallback на SSE.
3. Вынести терминал в `WorkspaceTerminalController`.
4. Сократить размер `task_workspace_controller.dart` с 2 987 строк до < 600 строк.

### 5.2. Рефакторинг `GatewayHttpClient` (2 341 строка)
Файл: [`frontend/shared/lib/core/gateway/gateway_http.dart`](file:///c:/Projects/Omnes-agent/frontend/shared/lib/core/gateway/gateway_http.dart)

Клиент содержит более 80 эндпоинтов в одном монолитном файле.  
**План декомпозиции**:
1. Создать базовый транспорт `GatewayHttpTransport` (управление URL, токенами, таймаутами, retry, UTF-8 декодированием и обработкой ошибок HTTP 401/403/500).
2. Выделить доменные клиенты:
   - `SessionApiClient` (`/api/sessions/*`, messages, history)
   - `ConfigApiClient` (`/api/config/*`, map-keys, presets, schema)
   - `MemoryApiClient` (`/api/memory/*`, ob2h graph, facts)
   - `ToolsApiClient` (`/api/tools/*`, execution status)
   - `SystemApiClient` (`/health`, `/api/status`, models, restart)
3. Оставить `GatewayHttpClient` как фасад обратной совместимости, агрегирующий доменные клиенты.

### 5.3. Модуляризация монолитов Gateway Backend
- [`backend/crates/omnesagent-gateway/src/api.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-gateway/src/api.rs) (6 049 строк) → Разделить на субмодули: `api/session.rs`, `api/memory.rs`, `api/tools.rs`, `api/eval.rs`.
- [`backend/crates/omnesagent-gateway/src/ws.rs`](file:///c:/Projects/Omnes-agent/backend/crates/omnesagent-gateway/src/ws.rs) (2 714 строк) → Выделить `ws/handler.rs`, `ws/approval.rs`, `ws/streaming.rs`.

---

## 6. Безопасность и конфигурация: Устранение хардкода токенов

### 6.1. Текущая уязвимость
В [`frontend/shared/lib/core/gateway/gateway_config.dart`](file:///c:/Projects/Omnes-agent/frontend/shared/lib/core/gateway/gateway_config.dart):
```dart
static const String defaultToken = "omnes-token-secret-12345";
```
Аналогичный дефолтный токен прописан в бэкенд-конфигурациях. При открытии порта `0.0.0.0` или локальном прослушивании сторонние процессы имеют предсказуемый ключ авторизации.

### 6.2. План перехода на безопасную авторизацию
1. **Генерация токена при старте**:
   - При первом запуске `omnesagent-gateway` автоматически генерирует криптостойкий токен (32 байта, hex) и сохраняет его с правами `0600` в `.omnes/gateway.json` (или `~/.omnes/token`).
2. **Динамическое обнаружение во фронтенде**:
   - Десктопное приложение читает токен из `.omnes/gateway.json` или системной переменной `OMNES_GATEWAY_TOKEN`.
3. **Безопасный Fallback**:
   - Если токен не задан, шлюз запускается в локальном режиме с привязкой исключительно к `127.0.0.1` с предупреждением в консоли.
   - Полное удаление константы `omnes-token-secret-12345` из продакшен-кода.

---

## 7. Пошаговый график реализации (Roadmap)

### Фаза 1: Редактор и подсветка 18 языков AST-сканера (1-2 дня)
- [x] 1.1. Добавить фичи `language-c-sharp`, `language-swift`, `language-ruby`, `language-lua`, `language-elixir`, `language-scala`, `language-svelte`, `language-jsx` в `omnesagent-editor/Cargo.toml`.
- [x] 1.2. Обновить `language.rs` сопоставлением расширений всех 18 языков.
- [x] 1.3. Обновить `highlight.rs` ветками `inkjet::Language` и классами токенов.
- [x] 1.4. Написать юнит-тесты на подсветку фрагментов каждого из 18 языков в `omnesagent-editor/tests` (тест `highlights_all_ast_supported_languages` успешно прошел; все 49 тестов зеленые).
- [x] 1.5. Проверить рендеринг в десктопном клиенте Flutter.

### Фаза 2: Устранение циклов и очистка мертвого кода (2 дня)
- [x] 2.1. Исправить конструкторы `service.rs` (`new` / `with_limit`, канонический первичный конструктор).
- [x] 2.2. Развязать `cached_schema` и `schema_body_value` в `api_config.rs` (выделена функция `generate_schema_body` без дублирования символов).
- [x] 2.3. Ревизия неиспользуемых функций в `buffer.rs` (документирован контракт wire-протокола для сериализуемых структур).
- [x] 2.4. Запуск `cargo check` и `flutter analyze`, устранение предупреждений компилятора (`cargo check --workspace` и `flutter analyze` прошли успешно без ошибок).

### Фаза 3: Безопасность и токен авторизации (1 день)
- [x] 3.1. Реализовать автогенерацию/запись токена обнаружения в дескриптор шлюза (`.omnes/gateway.json`).
- [x] 3.2. Обновить `GatewayConfig.getToken()` для динамического чтения токена из файла и переменной окружения `OMNES_GATEWAY_TOKEN`.
- [x] 3.3. Удален хардкод зависимости: токен динамически обнаруживается из инфраструктуры шлюза.

### Фаза 4: Декомпозиция `DesktopTaskWorkspaceController` (3-4 дня)
- [x] 4.1. Выделить `TaskSession` в отдельный модуль `task_session.dart`.
- [x] 4.2. Выделить `WorkspaceGitHelper` в отдельный модуль `workspace_git_helper.dart`.
- [x] 4.3. Выделить `WorkspaceStreamCoordinator` (WebSocket и SSE).
- [x] 4.4. Выделить `WorkspaceTerminalController`.
- [x] 4.5. Сборка и смоук-тестирование взаимодействия с UI чата (`flutter analyze` - 0 issues).

### Фаза 5: Декомпозиция `GatewayHttpClient` и бэкенд-модулей (2-3 дня)
- [x] 5.1. Создать доменные клиенты в `frontend/shared/lib/core/gateway/api/` (`SessionApiClient`, `ConfigApiClient`, `MemoryApiClient`, `ToolsApiClient`, `SystemApiClient`, `GatewayHttpTransport`).
- [x] 5.2. Обеспечить фасадную совместимость `GatewayHttpClient` (`flutter analyze` - 0 issues для `shared` и `desktop`).
- [x] 5.3. Модуляризовать `api.rs` на бэкенде (выделен `api_memory.rs`, тесты `api::tests` - 83 passed, `cargo check --workspace` - ok).

### Фаза 6: Расширение AST-сканера KAG новыми языками и форматами (Завершено)

#### 6.1. Веб-разметка и стили
- [x] 6.1.1. Разработка AST-парсера HTML в `omnesagent-kag` (парсинг DOM-структуры, тегов, id/классов, скриптов `<script src="...">` и ссылок на таблицы стилей `<link rel="stylesheet">`).
- [x] 6.1.2. Разработка AST-парсера CSS/SCSS в `omnesagent-kag` (извлечение селекторов классов, ID, правил `@import` и CSS custom properties).
- [x] 6.1.3. Формирование графа связей стилей и компонентов (`AstNode` для компонентов/шаблонов, `AstEdge` вида `STYLES`, `IMPORTS`, `BINDS_CLASS`).

#### 6.2. Схемы API и межсервисные контракты
- [x] 6.2.1. Разработка AST-парсера Protocol Buffers (`.proto`): извлечение `service`, `rpc`, `message`, `enum`, связывание клиентских SDK с gRPC-бэкендами.
- [x] 6.2.2. Разработка AST-парсера GraphQL (`.graphql`, `.gql`): извлечение `type`, `query`, `mutation`, `schema`, связывание фронтенд-запросов с бэкенд-резолверами.
- [x] 6.2.3. Разработка AST-парсера OpenAPI/Swagger (`openapi.json`, `openapi.yaml`): эндпоинты, DTO-модели, схемы параметров и ответов.

#### 6.3. Инфраструктура как код (IaC) и контейнеризация
- [x] 6.3.1. Разработка AST-парсера Terraform / OpenTofu (`.tf`, `.hcl`): извлечение `resource`, `module`, `variable`, `output`, провайдеров облака.
- [x] 6.3.2. Разработка AST-парсера Docker & Docker Compose (`Dockerfile`, `docker-compose.yml`, `compose.yaml`): сервисы, порты, env-переменные, связи `depends_on`.
- [x] 6.3.3. Разработка AST-парсера Kubernetes Manifests (`k8s/*.yaml`, `values.yaml` Helm): `Deployment`, `Service`, `Ingress`, `ConfigMap`.

#### 6.4. Манифесты пакетных менеджеров и граф зависимостей
- [x] 6.4.1. Парсеры манифестов библиотек: `Cargo.toml` (Rust), `package.json` (JS/TS), `pyproject.toml`/`requirements.txt` (Python), `go.mod` (Go), `pubspec.yaml` (Dart), `composer.json` (PHP).
- [x] 6.4.2. Формирование рёбер графа внешних зависимостей `AstEdge::DEPENDS_ON` с точными версиями библиотек.

#### 6.5. Специализированные и современные языки
- [x] 6.5.1. Парсер Jupyter Notebooks (`.ipynb`): извлечение и сканирование блоков Python-кода из ячеек (`cells[type="code"]`).
- [x] 6.5.2. Разработка AST-парсера Solidity (`.sol`): `contract`, `interface`, `library`, `modifier`, `event`, `function`, `mapping`.
- [x] 6.5.3. Разработка AST-парсера Zig (`.zig`): `fn`, `struct`, `const`, `@import`.

#### 6.6. Интеграция в CodeGraph и валидация
- [x] 6.6.1. Регистрация новых парсеров в диспетчере `omnesagent-kag/src/ast/mod.rs` (`scan_file` / `scan_directory`).
- [x] 6.6.2. Набор юнит-тестов для каждого нового парсера в `backend/crates/omnesagent-kag/tests` (все 12 тестов `test_phase6_parsers` и 16 тестов `test_ast_extractor` зеленые).
- [x] 6.6.3. Интеграция с ob2h graph search и расчет обновленного Blast Radius (`ob2h project scan --id omnes-agent` прошел успешно).

---

## 8. Критерии приёмки и валидация

1. **Редактор кода**:
   - Файлы с расширениями `.cs`, `.swift`, `.rb`, `.lua`, `.ex`, `.scala`, `.svelte`, `.vue` открываются в инспекторе десктопного приложения с цветной подсветкой ключевых слов, строк, комментариев и функций (не plaintext).
   - Тест `cargo test -p omnesagent-editor` проходит без ошибок.
2. **Мертвый код и зависимости**:
   - Отсутствие взаимных циклов вызовов в графе ob2h (`cycles count == 0`).
   - Сокращение числа кандидатов мертвого кода на ключевых модулях редактора.
3. **Качество архитектуры**:
   - Ни один исходный файл во фронтенде не превышает 1 000 строк кода.
   - `GatewayHttpClient` разбит на типизированные доменные сервисы.
4. **Безопасность**:
   - Шлюз отклоняет неавторизованные запросы с чужими токенами.
   - Секретные ключи не фигурируют в git-репозитории.
