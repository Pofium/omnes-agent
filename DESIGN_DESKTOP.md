# Дизайн-спецификация и ТЗ: OmnesAgent Desktop ADE (ZCode Edition)

> **Назначение документа**: Официальная архитектурно-дизайнерская спецификация десктопного приложения **OmnesAgent Desktop ADE** (Agentic Development Environment) для генерации макетов в Google Gemini Designer / Figma и реализации во Flutter Desktop (Windows, macOS, Linux).  
> **Концептуальная основа**: Полная адаптация архитектуры и визуального языка **ZCode ADE** (Z.ai / GLM-5.3 Harness), где ИИ-агент и рабочий процесс выполнения задач являются центральным элементом рабочей среды разработчика, а текстовый редактор, терминал и живой браузер — периферийными подключаемыми инструментами.

---

## 0. Сравнительный анализ: Традиционный IDE vs ZCode ADE

| Параметр | Традиционный IDE (VS Code, Cursor) | OmnesAgent Desktop ADE (ZCode-стиль) |
| :--- | :--- | :--- |
| **Центр экрана** | Текстовый редактор кода (buffer) | **AI-агент, лента задач и холст рассуждений** |
| **Роль ИИ** | Боковой чат-ассистент в сайдбаре | **Главный исполнитель и автономный оркестратор проекта** |
| **Окно ввода (Prompt)** | Узкое поле в боковой панели | **Центральный полноразмерный Composer** с контекстным меню `+` (`@`, `#`, `/`, `$`) |
| **Режим выполнения** | Ручное подтверждение каждого шага | **4 режима автономности (Permission Modes)**: от `Ask before changes` до `Full access` |
| **Управление целями** | Линейный чат без сохранения структуры | **Goal Mode (`/goal`)** с итерациями, верификацией по тестам и сводкой |
| **Браузер & UI тест** | Внешнее окно браузера | **Встроенный Live Browser с Element Picker** и драйвером Agent Browser-Use |
| **Терминал & Git** | Нижняя панель на всю ширину | **Интегрированный терминал в верхнем правом углу / вкладке инспектора (`⌘J`)** |
| **Метрики в сайдбаре** | Имена файлов дерева | **Задачи со статусами и бейджами изменений кода (`+142 -28`)** |

---

## 1. Бренд, айдентика и общие ограничения

### 1.1. Логотип и брендинг OmnesAgent
- **Logo-icon**: Металлическая / silver rounded-square иконка с яркими cyan / turquoise крыльями, мотивами микросхем (circuit board) и центрированной монограммой.
- **Размещение**:
  - Title Bar: 20px рядом с заголовком активного воркспейса.
  - Sidebar Header: 32px с надписью `OmnesAgent ADE`.
  - Assistant Avatar в ленте задач: 26px rounded-square.
  - Splash Screen / Empty States: 64–96px с мягким радиальным cyan glow.

### 1.2. Ограничения для дизайн-генерации (Prompt Guidelines)
- **Платформа**: Desktop Workstation (1440×900 base canvas, адаптивно до 2560×1440).
- **Плотность**: Compact developer tool UI (ReUI / shadcn / Tailwind density).
- **Шрифты**:
  - UI Text: `Inter`, `Segoe UI`, `-apple-system`.
  - Code, Terminal, Diffs, Stats: `JetBrains Mono`, `Fira Code`.
- **Иконки**: Lucide Icons (stroke width 1.5–1.75px).
- **Никаких мобильных паттернов**: Запрещены мобильные шторки (drawer), плавающие нижние мобильные панели (bottom navigation bar) и оверлейные экраны на весь экран для базовых действий.

---

## 2. Цветовые палитры: Темная (Dark) и Светлая (Light) темы

Интерфейс спроектирован с поддержкой двух полноценных тем с автоматическим и ручным переключением (`⌘K -> Switch theme`).

### 2.1. Dark Theme (Тема по умолчанию)

Глубокая графитово-нейтральная палитра с неоновыми бирюзовыми и серебристыми акцентами:

```css
:root[data-theme="dark"] {
  /* Базовые фоны */
  --bg-app-shell: #090D12;           /* Глобальный фон окна */
  --bg-sidebar: #0E131A;             /* Левый сайдбар задач и воркспейсов */
  --bg-canvas: #0D1117;              /* Центральный холст ленты агента */
  --bg-panel-tool: #121820;          /* Правая панель (Browser/Terminal/Preview) */
  --bg-card: #161D26;                /* Карточки рассуждений, тулов и целей */
  --bg-card-hover: #1C2430;          /* Hover-состояния карточек */
  --bg-input-box: #131922;           /* Фоновое поле Composer Prompt Box */

  /* Бордеры и разделители */
  --border-subtle: rgba(255, 255, 255, 0.07);
  --border-default: #222B38;
  --border-focus: #00D2FF;

  /* Текст */
  --text-primary: #F1F5F9;           /* Slate-100 */
  --text-secondary: #94A3B8;         /* Slate-400 */
  --text-muted: #64748B;             /* Slate-500 */
  --text-code: #38BDF8;              /* Cyan-code */

  /* Акценты бренда OmnesAgent (Cyan / Sky) */
  --accent-primary: #00D2FF;         /* Электрик-циан */
  --accent-primary-hover: #38BDF8;
  --accent-primary-glow: rgba(0, 210, 255, 0.15);
  --accent-secondary: #3B82F6;       /* Глубокий синий */

  /* Статусы */
  --status-running: #38BDF8;         /* Пульсирующий циан */
  --status-success: #10B981;         /* Изумрудный зеленый (+diff) */
  --status-warning: #F59E0B;         /* Янтарный (ожидание подтверждения) */
  --status-error: #EF4444;           /* Красный (-diff, ошибки) */

  /* Терминал */
  --terminal-bg: #0A0D12;
  --terminal-text: #E2E8F0;
  --terminal-selection: #1E3A8A;
}
```

### 2.2. Light Theme (Светлая тема)

Чистый, высококонтрастный разработческий интерфейс в скандинавском стиле без белого "выжигания" глаз:

```css
:root[data-theme="light"] {
  /* Базовые фоны */
  --bg-app-shell: #F8FAFC;           /* Мягкий светлый slate-фон */
  --bg-sidebar: #F1F5F9;             /* Сайдбар чуть контрастнее */
  --bg-canvas: #FFFFFF;              /* Центральный холст */
  --bg-panel-tool: #F8FAFC;          /* Правая панель */
  --bg-card: #FFFFFF;                /* Карточки */
  --bg-card-hover: #F8FAFC;          /* Hover-состояния */
  --bg-input-box: #FFFFFF;           /* Prompt Box */

  /* Бордеры и разделители */
  --border-subtle: #E2E8F0;
  --border-default: #CBD5E1;
  --border-focus: #0284C7;

  /* Текст */
  --text-primary: #0F172A;           /* Slate-900 */
  --text-secondary: #475569;         /* Slate-600 */
  --text-muted: #94A3B8;             /* Slate-400 */
  --text-code: #0369A1;              /* Cyan-code */

  /* Акценты */
  --accent-primary: #0284C7;         /* Сапфирово-бирюзовый */
  --accent-primary-hover: #0369A1;
  --accent-primary-glow: rgba(2, 132, 199, 0.12);
  --accent-secondary: #2563EB;

  /* Статусы */
  --status-running: #0284C7;
  --status-success: #059669;
  --status-warning: #D97706;
  --status-error: #DC2626;

  /* Терминал в светлой теме (стильный темно-грифельный контейнер для читаемости кода) */
  --terminal-bg: #0F172A;
  --terminal-text: #E2E8F0;
  --terminal-selection: #1E3A8A;
}
```

---

## 3. Архитектура экранов и трёхпанельный лэйаут ADE

```text
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│ Title Bar (40px): [Icon] Workspace: /omnes-agent [git: main*]  [⌘K Search]  [⌘J Term][Preview][Settings] │
├───────────────────┬───────────────────────────────────────────┬──────────────────────────┤
│ LEFT SIDEBAR      │ CENTER AGENT CANVAS                       │ RIGHT TOOL CANVAS        │
│ (280–320px)       │ (Flex 1)                                  │ (420–540px, resizable)   │
│                   │                                           │                          │
│ [+ New Task]      │ ┌───────────────────────────────────────┐ │ [Tabs: Browser | Term |  │
│ [Search & Filter] │ │ Goal Card: "Refactor router & tests"  │ │        Preview | SideChat│
│                   │ │ Elapsed: 4m 12s | Iteration 3/10      │ ├──────────────────────────┤
│ WORKSPACES        │ └───────────────────────────────────────┘ │ LIVE BROWSER / TERMINAL  │
│ ▼ omnes-agent     │                                           │ ┌──────────────────────┐ │
│  • Task #42 (+84) │ Iteration 1 · Goal not met, continues     │ │ URL: localhost:3000  │ │
│  • Task #41 (+12) │ ┌───────────────────────────────────────┐ │ ├──────────────────────┤ │
│  • Task #39 (err) │ │ Agent Thought (Max): Analyzing AST... │ │ │ [Element Picker]     │ │
│                   │ │ Tool Call: grep_search /router/       │ │ │                      │ │
│ GROUPS            │ └───────────────────────────────────────┘ │ │ Webpage Canvas       │ │
│ ▶ Auth Sprint     │                                           │ │ (Agent live click,   │ │
│                   │ ┌───────────────────────────────────────┐ │ │  fill, verify)       │ │
│ ───────────────── │ │ Verification: 12 tests passed, 1 fail │ │ └──────────────────────┘ │
│ [Memory: Active]  │ └───────────────────────────────────────┘ │                          │
│ [SSH: ser.vps.ru] │                                           │ INTEGRATED TERMINAL      │
│ [Marketplace/MCP] │ ═════════════════════════════════════════ │ ┌──────────────────────┐ │
│                   │ CENTRAL PROMPT BOX (COMPOSER)             │ │ $ cargo test         │ │
│                   │ [+] [@mention] [#chat] [/cmd] [$skill]    │ │ test result: ok.     │ │
│                   │ [Mode: Ask before changes ▾] [Model: GLM▾]│ └──────────────────────┘ │
└───────────────────┴───────────────────────────────────────────┴──────────────────────────┘
```

---

## 4. Детальное описание ключевых компонентов ADE

### 4.1. Левый сайдбар (ADE Sidebar)

Ширина: **280px** (по умолчанию), сворачивается в **60px** (только иконки) или скрывается хоткеем `⌘B`.

1. **Верхняя строка действий**:
   - Кнопка `+ New Task`: главная акцентная кнопка (Cyan pill/rounded rect с иконкой пера/плюса).
   - Кнопка `Search / Command Center`: `⌘K` с мгновенным доступом к задачам, файлам воркспейса, командам и смене тем.
   - Кнопка `Skills & Plugins`: доступ к каталогу установленных навыков и MCP-сервисов.
2. **Переключатель режимов отображения задач**:
   - `Workspace`: группировка по локальным папкам или удаленным SSH/Docker проектам.
   - `Grouped`: пользовательские тематические папки (например, "Рефакторинг API", "Баги UI").
   - `Timeline`: хронологический список с сортировкой по времени создания или обновления.
3. **Элемент списка задач (Task Row Item)**:
   - Иконка статуса:
     - Зеленая/бирюзовая точка: в процессе выполнения (`Running`).
     - Серая точка: завершено (`Done`).
     - Желтая точка: ожидает подтверждения пользователя (`Waiting Approval`).
     - Красная точка: ошибка / прервано (`Failed`).
   - Название задачи (truncate в одну строку).
   - Относительное время (`2m`, `1h`, `3d`).
   - **Бейдж изменения строк Git (`Diff Badge`)**: `+142 -28` (зеленый плюс, красный минус в компактном контейнере). Позволяет мгновенно оценить объем затронутого кода.
4. **Секция архивных задач (`Archived Tasks`)**:
   - Сворачиваемый блок со старыми задачами, с кнопками `Restore` и `Delete`.
5. **Подвал сайдбара**:
   - Индикатор Project Memory (`Active` / `Disabled`).
   - Статус удаленного подключения (SSH / Docker container).
   - Профиль пользователя, настройки и биллинг/токены.

---

### 4.2. Центральный Composer (Prompt Box)

Главное окно ввода задач расположено в нижней части центрального холста в виде парящей карточки с закругленными углами и легкой тенью.

1. **Многострочное поле ввода**:
   - Поддержка Markdown-разметки, горячих клавиш (`Enter` для отправки, `Shift+Enter` для новой строки).
   - Автоматическое сворачивание больших блоков текста (пастинг кода): длинные вставки преобразуются в аккуратные чипы-вложения (`snippet_1.dart · 140 lines`), не захламляя поле ввода.
2. **Контекстное меню `+` (Quick Context Insert)**:
   - `Add Attachment`: добавление изображений, PDF, логов.
   - `@ Mention`: всплывающий список файлов и директорий текущего воркспейса с умным поиском.
   - `# Link Conversation`: подключение контекста другой сессии.
   - `/ Slash Command`: быстрый вызов встроенных команд (`/goal`, `/side`, `/plan`, `/test`, `/git`, `/review`).
   - `$ Skill`: вызов специализированного навыка (например, `$ob2h`, `$flutter_driver`).
3. **Нижняя строка управления (Sub-bar Controls)**:
   - **Переключатель режима автономности (Permission Mode Switcher, `Shift+Tab`)**:
     1. **Ask before changes** (по умолчанию): агент запрашивает подтверждение перед каждой модификацией файла и запуском терминальной команды.
     2. **Edit automatically**: агент самостоятельно правит файлы, но спрашивает подтверждение перед деструктивными/терминальными командами.
     3. **Plan mode**: агент сначала формирует и защищает архитектурный план, и начинает кодить только после одобрения пользователем.
     4. **Full access**: максимальная автономность для фоновых длинных задач без промежуточных блокировок.
   - **Селектор модели и уровня рассуждений (Thought Level)**:
     - Выпадающий список подключенных LLM: `GLM-5.3`, `Claude 3.5 Sonnet`, `DeepSeek V3`, `Ollama/Local`.
     - Переключатель глубины мысли:
       - `Low`: быстрые ответы на простые вопросы.
       - `High`: стандартный уровень разработки.
       - `Max`: глубокий пошаговый анализ архитектуры и отладка сложных багов.
   - **Индикатор контекста**: счетчик токенов сессии и статус ветки Git (`git: main*`).
   - **Кнопка запуска / остановки**: Cyan-кнопка отправки стрелкой вверх, трансформирующаяся в красный квадрат `Stop` при работающем агенте.

---

### 4.3. Режим целей (Goal Mode, `/goal`)

Ключевой механизм ZCode ADE для выполнения многошаговых долгосрочных задач (Long-Horizon Tasks).

1. **Команды управления целью**:
   - `/goal <objective>`: установить цель сессии (например, `/goal Сделать полную адаптацию верстки под Flutter Desktop и пройти все тесты`).
   - `/goal pause`: приостановить выполнение.
   - `/goal resume`: продолжить.
   - `/goal clear`: сбросить текущую цель.
2. **Goal Summary Panel**:
   - Закрепленная информационная карточка вверху ленты задач:
     - Текст целевого ориентира (Objective).
     - Таймер прошедшего времени (`Elapsed: 12m 45s`).
     - Текущий статус итерации (`Iteration 4 of max 15`).
     - Интерактивный чек-лист подзадач, сгруппированный по итерациям.
3. **Автономный цикл с верификацией**:
   - `Task Initiation` -> `Plan Generation` -> `Execution (Files, Terminal, Browser)` -> `Verification against real evidence` -> `Review / Summary`.
   - В ленте сообщений между кругами работы отображаются четкие разделители-бейджи:
     - `Iteration X · Goal not met, task continues` (агент обнаружил непрошедший тест или недостающий компонент и автоматически начинает следующий раунд).
     - `Iteration Y · Goal met, task finished` (все условия цели подтверждены фактами и выводом компилятора).
4. **Критерии завершения (Real Evidence)**:
   - Агент не считает задачу выполненной на основе одних лишь рассуждений: обязательно проверяются результаты запуска тестов, компиляция без ошибок и Git status.

---

### 4.4. Правая инструментальная панель (Right Tool Canvas)

Многофункциональный рабочий контейнер, расположенный справа (ширина 420–540px, сплиттер с возможностью изменения размера).

Содержит вкладки:
1. **Live Browser Preview & Browser-Use**:
   - Полноценное окно встроенного браузера с адресной строкой (`http://localhost:3000`, `file://`).
   - Кнопки: Back, Forward, Reload, DevTools Console.
   - **Инструмент Element Picker**: кнопка прицела в тулбаре браузера. При клике на любой элемент страницы агент получает его селектор, координаты, текст и безопасный HTML-сниппет прямо в поле ввода.
   - **Визуальная правка UI**: мультимодальный агент (например, GLM-5.3-Flash / Claude) делает скриншот веб-страницы, визуально оценивает баги позиционирования или шрифтов и правит CSS/код налету.
2. **Интегрированный терминал (Terminal & Git)**:
   - Встроенный эмулятор терминала (PTY/shell).
   - Быстрый хоткей вызова `⌘J` в правом верхнем углу.
   - Выполнение фоновых задач (dev-серверы, watchers) без блокировки основного чата.
   - Интерактивный лог выполнения команд с подсветкой ошибок и возможностью копирования.
3. **Панель предпросмотра файлов (Preview Pane)**:
   - Нативный рендеринг Markdown (релизные отчеты, чеклисты, таблицы).
   - Интерактивные Mermaid-диаграммы: масштабирование колесиком мыши, перетаскивание (pan), полноэкранный просмотр по двойному клику.
   - Таблицы данных: закрепленный заголовок при скролле, экспорт в CSV.
   - Просмотр изображений и PDF с масштабированием.
4. **Боковая беседа (Side Conversation, `/side`, `/btw`)**:
   - Отдельная вкладка для вопросов "на полях" (например, "Объясни, как работает этот вспомогательный класс?").
   - Наследует контекст основного проекта, но не загрязняет основную траекторию выполнения цели.
   - Закрывается без влияния на основной запуск.

---

## 5. UI-компоненты карточек рассуждений и инструментов

### 5.1. Карточка хода мыслей (Agent Thought Box)
- Сворачиваемый аккордеон с бейджем уровня рассуждений (`Thought (Max) · 1.4k tokens`).
- Полупрозрачный фон с легкой бирюзовой обводкой слева.
- Внутри — стриминг внутреннего монолога агента моноширинным или наклонным текстом.

### 5.2. Карточка Tool Call
- Заголовок с типом инструмента: `Terminal: cargo test`, `File Edit: lib/main.dart (+14, -2)`, `Browser: navigate to http://localhost:8080`.
- Индикатор состояния: спиннер (в процессе), зеленая галочка (успех), красный крест (ошибка).
- Возможность развернуть `stdout/stderr` или `unified diff`.

### 5.3. Карточка одобрения опасного действия (Approval Card)
- Появляется в режиме `Ask before changes`.
- Яркий акцент: янтарная рамка и предупреждающая иконка щита.
- Описание действия: точная команда для терминала или путь к файлу на удаление/запись.
- Кнопки действий:
  - `Approve Once` (Разрешить разово, `⌘Enter`);
  - `Always Allow for this Session` (Всегда разрешать для этой сессии);
  - `Deny` (Отклонить, `Esc`);
  - `Edit Command` (Отредактировать перед запуском).

---

## 6. Промпты для Google Gemini Designer / Figma

Для генерации дизайн-макетов экранов в **Gemini Designer** используйте следующие мастер-промпты:

### 6.1. Глобальный системный промпт стиля (System Style Prompt)
```text
Role: Lead UI/UX Product Designer for Desktop Developer Tools.
Product: OmnesAgent Desktop ADE (Agentic Development Environment), inspired by ZCode ADE.
Platform: Desktop Workstation application (1440x900 resolution, not mobile).
Design System: ReUI / shadcn / Tailwind dense layout. Clean borders, compact cards, high information density.
Typography: Inter for UI text, JetBrains Mono for code, terminal, diffs and metrics.
Branding: Metallic silver rounded-square icon with bright electric cyan circuit wings.
Layout: 3-panel ADE architecture (Left Sidebar 280px, Center Agent Canvas with floating Prompt Box Composer, Right Tool Panel 460px with Browser & Terminal).
Provide two color schemes: Dark Theme (primary, slate #090D12, cards #161D26, cyan accent #00D2FF) and Light Theme (crisp slate #F8FAFC, white cards #FFFFFF, sapphire cyan #0284C7).
```

### 6.2. Промпт: Главный экран рабочего пространства ADE (Dark Theme)
```text
Design the main Task Workspace of OmnesAgent Desktop ADE in Dark Theme (1440x900).
1. Title bar (40px): Frameless with macOS traffic lights, OmnesAgent cyan/silver icon, workspace path "/projects/omnes-agent", active git branch badge "main*", search bar "Search or command (Cmd+K)", and top-right toggle buttons: Terminal icon (Cmd+J), Browser icon, Settings.
2. Left Sidebar (280px, bg #0E131A):
   - Header with "+ New Task" bright cyan button, Search input, and Skills icon.
   - Segmented view switcher: [Workspace] [Grouped] [Timeline].
   - Workspace tree: "OmnesAgent" with active task "Goal: Refactor Desktop UI", relative time "3m", green status dot, and Git diff badge "+142 -28".
   - Second task: "Fix Gateway WS frame decode", gray dot, "+18 -4".
   - Bottom status showing "Memory: Enabled", "Runtime: Connected (127.0.0.1:42617)", user avatar.
3. Center Canvas (bg #0D1117):
   - Top sticky Goal Mode Card: "Goal: Implement ZCode ADE Layout", Elapsed "5m 24s", Iteration "2 of 10", checklist with 2 completed items and 1 running item.
   - Chat feed with divider: "Iteration 1 · Goal not met, task continues".
   - Collapsible Thought block: "Thought (Max): Analyzing existing desktop layout...", cyan border.
   - Tool card: "terminal_run: flutter test" with expandable terminal snippet.
   - Bottom Composer (Floating Prompt Box): Rounded card (bg #131922, border #222B38). Context button "+" with menu tags [@file] [#chat] [/cmd] [$skill].
   - Sub-bar underneath prompt: Permission Mode pill "Ask before changes (Shift+Tab)", Model selector "GLM-5.3 (Thought: Max)", token counter "34.2k tokens", cyan circular send button.
4. Right Panel (460px, bg #121820):
   - Tabs at top: [Live Browser] [Terminal] [Diff Preview] [Side Chat].
   - Active tab "Live Browser": address bar "http://localhost:8080/app", Reload, Element Picker crosshair tool button highlighted in cyan.
   - Built-in browser viewport displaying the running application with a blue selection rectangle over a button showing its DOM selector ".submit-btn".
   - Lower split: Compact Integrated Terminal showing live build logs in dark terminal colors.
```

### 6.3. Промпт: Главный экран рабочего пространства ADE (Light Theme)
```text
Design the exact same Task Workspace of OmnesAgent Desktop ADE, but in Light Theme (1440x900).
- Global App Shell background: Crisp #F8FAFC.
- Left Sidebar: Soft slate-gray #F1F5F9, border #E2E8F0. Task items have clean white card backgrounds on hover, sharp text #0F172A, diff badges "+142 -28" with emerald green and crimson red badges.
- Center Canvas: Pure white #FFFFFF. Goal Card has subtle border #CBD5E1 and faint blue-cyan header tint rgba(2,132,199,0.06).
- Agent Thought card: Pale slate-50 container with left border #0284C7.
- Composer Prompt Box: Elevated white card with border #CBD5E1, focused with vivid sapphire cyan #0284C7 ring.
- Right Panel: #F8FAFC background. Live browser preview displays clean white application canvas with precision element inspection tool.
- Terminal block in right panel remains styled in high-contrast dark slate (#0F172A) with bright syntax text (#E2E8F0) for optimal code legibility.
```

---

## 7. Чек-лист соответствия ZCode ADE для разработчиков

- [x] **Центральность агента**: Интерфейс сфокусирован на постановке целей и контроле выполнения агентом, а не на ручном наборе текста в редакторе.
- [x] **Composer с расширенным контекстом**: Меню `+` поддерживает `@` (файлы/папки), `#` (диалоги), `/` (команды), `$` (скиллы), авто-чипы для больших кусков текста.
- [x] **Permission Mode**: 4 режима (`Ask before changes`, `Edit automatically`, `Plan mode`, `Full access`) переключаются через `Shift+Tab`.
- [x] **Goal Mode (`/goal`)**: Циклический процесс с верификацией по тестам и Git diff, карточка статуса цели и разделители итераций.
- [x] **Live Browser & Element Picker**: Встроенный браузер для визуальной отладки с захватом селекторов в контекст агента.
- [x] **Терминал и Git**: Интегрированное окно вывода команд в верхнем правом углу (`⌘J`) с запуском фоновых процессов.
- [x] **Diff-метрики в сайдбаре**: Каждая задача в списке сразу показывает объем изменений (`+142 -28`).
- [x] **Две темы**: Полноценная поддержка Dark и Light тем на уровне дизайн-токенов и компонентов.
