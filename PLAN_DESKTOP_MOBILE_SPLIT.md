# Архитектурный план: Разделение десктопной и мобильной версий OmnesAgent (ZCode ADE Edition)

Настоящий документ определяет стратегию и пошаговый план разделения кодовой базы клиентских приложений OmnesAgent на независимые проекты: **Mobile Client** (Android, iOS) и **Desktop ADE Workstation** (Windows Desktop, macOS, Linux, Web Desktop), с выносом общего ядра в переиспользуемый пакет **Shared Core**.

План составлен на основе дизайн-спецификации [`DESIGN_DESKTOP.md`](./DESIGN_DESKTOP.md) (версия ZCode ADE) и учитывает все доработки мобильного клиента (локализация RU/EN, PIN-код/биометрия, автоподключение к шлюзу, LLM-провайдеры, удаление email-логина, адаптивные пузыри сообщений).

---

## 1. Проблема текущей архитектуры

Текущий проект `frontend/` изначально создавался как мобильное приложение (порт мобильного SaaS-чата):
1. **ScreenUtil и мобильные координаты**: Использование `ScreenUtil(designSize: 414, 896)` приводит к непредсказуемому масштабированию на больших мониторах (раздувание шрифтов, огромные отступы).
2. **Одноколоночная мобильная модель**: Логика экранов завязана на шторку (`DrawerWidget`), полноэкранные модальные окна и мобильный стек страниц.
3. **Несовместимость с десктопным ТЗ ZCode ADE**: Согласно [`DESIGN_DESKTOP.md`](./DESIGN_DESKTOP.md), десктопный клиент — это **Agentic Development Environment (ADE)** (1440×900+, трёхпанельный сплиттер, центральный Composer с 4 режимами разрешений `Shift+Tab`, Goal Mode с верификацией по тестам, встроенный Live Browser с Element Picker, интегрированный терминал `⌘J`, Diff-бейджи `+142 -28` в сайдбаре). Попытка писать это внутри одного мобильного проекта с бесконечными ветвлениями `if (isDesktop)` делает кодовую базу громоздкой, труднотестируемой и хрупкой.

---

## 2. Целевая структура монорепозитория

Чистая модульная структура внутри папки клиентов `frontend/`:

```text
c:\Projects\Omnes-agent\
├── backend\                           # Rust Workspace (gateway, runtime, tools, mcp)
│
├── frontend\                          # Клиентская часть (Flutter Workspace)
│   ├── shared\                        # Переиспользуемое общее ядро (Dart package)
│   │   ├── lib\
│   │   │   ├── core\
│   │   │   │   ├── gateway\           # GatewayHttpClient, GatewayWebSocket, GatewayConfig
│   │   │   │   └── models\            # GatewayFrame, ToolCallInfo, WorkspaceEntry, SessionInfo, GoalStatus
│   │   │   ├── localization\          # Strings, russian.dart, english.dart, LocalString
│   │   │   ├── storage\               # LocalStorage (PIN, ключи, токены, язык, провайдеры, themeMode)
│   │   │   ├── theme\                 # Токены ReUI: Dark Theme (#090D12, #00D2FF) & Light Theme (#F8FAFC, #0284C7)
│   │   │   └── services\              # Звук/TTS, сетевая диагностика, шифрование PIN
│   │   └── pubspec.yaml
│   │
│   ├── mobile\                        # Мобильное приложение (Android, iOS)
│   │   ├── lib\
│   │   │   ├── features\
│   │   │   │   ├── home\              # Мобильный дашборд с карточками и шторкой
│   │   │   │   ├── chat\              # Сенсорный мобильный чат, автоозвучка
│   │   │   │   ├── security\          # Экран ввода PIN (LockScreen) и биометрия
│   │   │   │   ├── settings\          # Мобильные настройки с аккордеонами
│   │   │   │   └── llm_providers\     # Мобильный выбор моделей и добавление custom API
│   │   │   └── main.dart
│   │   ├── android\
│   │   ├── ios\
│   │   └── pubspec.yaml               # dependencies: shared: { path: ../shared }
│   │
│   └── desktop\                       # Десктопная станция ZCode ADE (Windows, macOS, Linux, Web)
│       ├── lib\
│       │   ├── app_shell\             # Window titlebar (frameless), статус шлюза, git branch, theme toggle
│       │   ├── layout\                # 3-х панельный сплиттер (Sidebar | Agent Canvas | Tool Canvas)
│       │   ├── features\
│       │   │   ├── sidebar\           # ADE Сайдбар: воркспейсы, группы, задачи с diff-бейджами (+142 -28)
│       │   │   ├── composer\          # Prompt Box: меню "+" (@, #, /, $), авто-чипы вложений
│       │   │   ├── permissions\       # Permission Mode switcher (Ask before / Edit auto / Plan / Full access)
│       │   │   ├── goal_mode\         # Goal Mode (/goal): Goal Card, Iteration Timeline, Verification checks
│       │   │   ├── agent_canvas\      # Лента выполнения: collapsible thoughts (Low/High/Max), tool execution
│       │   │   ├── tools_panel\       # Правая панель инструментов:
│       │   │   │   ├── live_browser\  # Встроенный браузер + Element Picker + driver visual testing
│       │   │   │   ├── terminal\      # Интегрированный терминал (⌘J), запуск фоновых dev-серверов
│       │   │   │   ├── preview\       # Просмотрщик Markdown, интерактивных Mermaid-диаграмм, CSV-таблиц
│       │   │   │   └── side_chat\     # Side Conversation (/side, /btw) на полях
│       │   │   ├── command_palette\   # Command Center (Ctrl+K / Cmd+K): быстрые команды, темы, файлы
│       │   │   └── approvals\         # Карточки подтверждения опасных операций (SOP)
│       │   └── main.dart
│       ├── windows\
│       ├── macos\
│       ├── linux\
│       ├── web\
│       └── pubspec.yaml               # dependencies: shared: { path: ../shared }
```

---

## 3. Что уходит в `frontend/shared/` (Переиспользуемое ядро)

Все доработки, сделанные в мобильной версии, сохраняются и выносятся в единый пакет:

1. **Сетевой уровень и протокол шлюза**:
   - `GatewayConfig`: хардкод адреса демона `http://127.0.0.1:42617` и `ws://127.0.0.1:42617`, управление токенами и агентом по умолчанию (`chief`).
   - `GatewayHttpClient`: вызовы `/health`, `/api/status`, `/api/config/catalog`, `/api/cost`, `/api/tools`, `/api/skills`, `/api/doctor`.
   - `GatewayWebSocket`: двусторонний потоковый обмен (фреймы `chat.send`, `chat.event`, `tool.call`, `approval.request`, `agent.state`, `goal.event`).
2. **Локальное хранилище и настройки**:
   - `LocalStorage`: сохранение PIN-кода, флагов биометрии, выбранного языка (RU/EN), настроек авто-TTS, списка кастомных LLM-провайдеров и выбранной темы (`dark` / `light` / `system`).
3. **Локализация (i18n)**:
   - Единый файл `Strings` со всеми строковыми константами.
   - Полные словари `russian.dart` и `english.dart` (включая разделы безопасности, режимов автономности ZCode, инструментов, системных статусов).
4. **Модели данных**:
   - `ChatMessage`, `ToolCallInfo`, `ApprovalRequest`, `WorkspaceEntry`, `SessionInfo`, `GoalStatus`, `DiffBadgeInfo`.
5. **Дизайн-токены (ReUI & ZCode Tokens)**:
   - **Dark Theme**: фон `#090D12` / `#0D1117`, карточки `#161D26`, акцентный электрик-циан `#00D2FF`, статусная индикация (зеленый/янтарь/красный).
   - **Light Theme**: фон `#F8FAFC` / `#FFFFFF`, карточки `#FFFFFF`, акцентный сапфир-циан `#0284C7`, высококонтрастный темный терминал `#0F172A`.

---

## 4. Концепция `frontend/desktop/` (по ТЗ `DESIGN_DESKTOP.md`)

Десктопная версия строится по архитектуре **ZCode ADE (Agentic Development Environment)**:

### 4.1. Макет (1440×900 и выше)
- **Кастомный Title Bar (40px)**:
  - Иконка OmnesAgent (20px, cyan/silver).
  - Название активного воркспейса и Git-ветки (`/omnes-agent [git: main*]`).
  - Строка Command Center (`⌘K`).
  - Быстрые переключатели: Терминал (`⌘J`), Браузер, Переключатель темы (Dark/Light), Настройки.
  - Системные кнопки окна (Свернуть / Развернуть / Закрыть).
- **Левая панель (ADE Sidebar, 280px)**:
  - Кнопка `+ New Task` и строка быстрого поиска/фильтрации.
  - Переключатель отображения: **Workspace** / **Grouped** / **Timeline**.
  - Дерево задач со статусами (`Running`, `Done`, `Waiting`, `Failed`) и **брутто-метриками кода (`Diff Badge: +142 -28`)**.
  - Сворачиваемый архив задач (`Archived Tasks`).
  - Подвал сайдбара: статус памяти (Memory), подключение к шлюзу / SSH / Docker, профиль.
- **Центральная панель (Agent Canvas & Prompt Box)**:
  - Лента выполнения задачи с разбивкой по итерациям Goal Mode (`Iteration X · Goal not met, task continues`).
  - Сворачиваемые блоки хода мыслей агента с бейджами уровней рассуждений (`Thought (Max) · 1.4k tokens`).
  - Интерактивные карточки инструментов (File edit diffs, command executions, verification results).
  - **Центральный Composer (Prompt Box)**:
    - Контекстное меню `+` с авто-чипами для больших кусков текста (`Add Attachment`, `@mention`, `#chat`, `/cmd`, `$skill`).
    - **Селектор Permission Mode (`Shift+Tab`)**:
      1. `Ask before changes` (запрос подтверждений на правки файлов и команды);
      2. `Edit automatically` (авто-правка файлов, запрос подтверждений на команды);
      3. `Plan mode` (сначала план, правки только после подтверждения плана);
      4. `Full access` (полная автономность).
    - **Селектор модели и глубины рассуждений**: `GLM-5.3` / `Claude 3.5 Sonnet` / `DeepSeek V3` + уровни `Low` / `High` / `Max`.
    - Счетчик токенов и индикатор Git.
- **Правая инструментальная панель (Tool Canvas, 420–540px)**:
  - **Live Browser Preview**:
    - Встроенный браузер (URL bar, reload, DevTools).
    - **Инструмент Element Picker**: захват кликом элемента страницы (селектор, HTML, координаты) в контекст агента для быстрой правки UI.
    - Агентный драйвер: стриминг действий Browser-Use и скриншоты для мультимодальной оценки.
  - **Интегрированный терминал (`⌘J`)**:
    - Запуск фоновых команд и сборщиков тестов.
  - **Предпросмотр (Preview)**:
    - Рендеринг Markdown, интерактивных диаграмм Mermaid (pan/zoom), таблиц с экспортом CSV.
  - **Боковая беседа (Side Conversation, `/side`, `/btw`)**:
    - Независимый чат для вопросов на полях без загрязнения контекста основной задачи.

---

## 5. Концепция `frontend/mobile/`

Мобильная версия оптимизирована для оперативного мониторинга и быстрых поручений на ходу:
- Одноколоночный вертикальный layout для телефонов и планшетов.
- **Безопасный старт**: `LockScreen` с быстрым вводом PIN-кода (4–6 цифр) или Face ID / отпечатка пальца.
- **Быстрый чат**: голосовой ввод, автоозвучка ответов (TTS).
- **Компактные карточки**: статус агента, подтверждение ожидающих запросов (Approval Banner).
- **Управление LLM**: переключение активных моделей и шлюза в один тап.

---

## 6. Пошаговый план миграции

### Фаза 1: Подготовка пакета `shared`
1. Создать каталог `frontend/shared/` с базовым `pubspec.yaml` (пакет типа `flutter`).
2. Перенести в `shared`:
   - `core/gateway/` (GatewayHttpClient, GatewayConfig, WebSocket)
   - `helper/local_storage.dart` (с методами PIN, языка, настроек, темы)
   - `utils/strings.dart`, `utils/language/` (ru/en словари)
   - `model/` (chat, tools, sessions, files, goal_status)
   - `design_system/` (токены Dark и Light тем по спецификации ZCode ADE)
3. Проверить `shared` через `flutter analyze`.

### Фаза 2: Реорганизация текущего кода в `mobile`
1. Переместить текущее содержимое `frontend/` (за вычетом перенесенного в `shared`) в `frontend/mobile/`.
2. Подключить зависимость `shared: { path: ../shared }` в `frontend/mobile/pubspec.yaml`.
3. Обновить импорты.
4. Проверить сборку мобильного клиента (`flutter analyze`, запуск на эмуляторе / Android / Web).

### Фаза 3: Разработка `desktop` приложения по спецификации ZCode ADE
1. Инициализировать чистый Flutter проект `frontend/desktop/` с поддержкой платформ `windows`, `macos`, `linux`, `web`.
2. Подключить зависимость `shared: { path: ../shared }`.
3. Подключить специализированные десктопные плагины:
   - `window_manager` (кастомный frameless titlebar)
   - `flutter_resizable_container` / `multi_split_view` (трёхпанельный сплиттер ADE)
   - `hotkey_manager` (глобальные хоткеи: `⌘K` поиск, `⌘J` терминал, `Shift+Tab` смена режима, `⌘B` сайдбар)
   - `flutter_inappwebview` / `webview_windows` (компонент Live Browser с Element Picker)
   - `xterm` (встроенный эмулятор терминала PTY)
4. Реализовать компоненты ZCode ADE Desktop:
   - `AppShell` (frameless titlebar + daemon ping badge + theme switcher)
   - `AgySidebar` (воркспейсы, группы задач, Git Diff Badges `+142 -28`)
   - `GoalSummaryCard` и `IterationDivider` (Goal Mode `/goal`)
   - `PromptComposer` (всплывающее меню `+`, авто-чипы вложений, переключатель Permission Mode, селектор модели и Thought Level)
   - `AgentCanvas` (лента сообщений, collapsible thinking cards `Low/High/Max`, tool call карточки, approvals)
   - `ToolCanvas` (вкладки: Live Browser c Element Picker, Terminal `⌘J`, File Preview, Side Conversation `/side`)
   - `CommandCenter` (`⌘K` всплывающее окно быстрого поиска и действий)
5. Настроить билд и интеграцию с backend-демоном.

---

## 7. Скрипты запуска и сборки

Для удобства разработчика в корне репозитория будут обновлены скрипты:

- `run_desktop.bat` / `run_desktop.ps1`: запуск локального Rust-шлюза + Desktop Workstation ADE (`flutter run -d windows` или `web`).
- `run_mobile.bat` / `run_mobile.ps1`: запуск мобильного клиента на эмуляторе/устройстве (`cd frontend/mobile && flutter run`).
- `check.bat` / `check.ps1`: параллельный статический анализ `backend/`, `frontend/shared/`, `frontend/mobile/` и `frontend/desktop/`.

---

## 8. Итог для пользователя
После выполнения данного плана:
- Работа над десктопным интерфейсом не будет ломать мобильные пропорции и верстку.
- Десктопная версия станет полноценной средой **Agentic Development Environment (ZCode ADE)** с поддержкой темной и светлой тем.
- Разработчик получает мощный инструмент для постановки целей агенту, визуальной отладки UI через Live Browser с Element Picker, запуска фоновых тестов в интегрированном терминале и мгновенного контроля объема изменений кода прямо в списке задач.
