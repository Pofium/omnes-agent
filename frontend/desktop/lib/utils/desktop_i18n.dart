// OmnesAgent Desktop ADE Centralized Bilingual (RU / EN) Localization System.
// Reactive via GetX: switching language instantly updates all screens in the application.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';

class DesktopI18n {
  static final RxString currentLanguage = 'ru'.obs;

  static bool get isRu => currentLanguage.value == 'ru';
  static bool get isEn => currentLanguage.value == 'en';

  /// Returns [ru] if current language is Russian, otherwise [en].
  static String tr(String ru, String en) {
    return isRu ? ru : en;
  }

  /// Initialize language from persistent storage.
  static void init() {
    try {
      final storage = GetStorage();
      final saved = storage.read<String>('desktop_language');
      if (saved == 'en' || saved == 'ru') {
        currentLanguage.value = saved!;
      }
    } catch (_) {}
  }

  /// Change active language and update GetX locale.
  static void setLanguage(String lang) {
    if (lang != 'ru' && lang != 'en') return;
    currentLanguage.value = lang;
    try {
      GetStorage().write('desktop_language', lang);
      Get.updateLocale(Locale(lang));
    } catch (_) {}
  }

  // ============================================================
  // SIDEBAR TRANSLATIONS
  // ============================================================
  static String get newTask => tr('Новая сессия', 'New session');
  static String get search => tr('Поиск', 'Search');
  static String get automations => tr('Автоматизации', 'Automations');
  static String get group => tr('Группа', 'Group');
  static String get project => tr('Проект', 'Project');
  static String get groups => tr('Группы', 'Groups');
  static String get projects => tr('Проекты', 'Projects');
  static String get timeline => tr('Хронология', 'Timeline');
  static String get collapseAll => tr('Свернуть все', 'Collapse all');
  static String get expandAll => tr('Развернуть все', 'Expand all');
  static String get view => tr('Вид', 'View');
  static String get byProject => tr('По проектам', 'By project');
  static String get sortBy => tr('Сортировка', 'Sort by');
  static String get updated => tr('По обновлению', 'Updated');
  static String get created => tr('По созданию', 'Created');
  static String get remoteControl => tr('Удалённое управление (Mobile Remote)', 'Remote control (Mobile Remote)');
  static String get settings => tr('Настройки (Провайдеры, MCP, Память, Язык)', 'Settings (Providers, MCP, Memory, Language)');
  static String get copy => tr('Копировать', 'Copy');

  // ============================================================
  // WORKSPACE TRANSLATIONS
  // ============================================================
  static String get terminalConsole => tr('Терминал (bash / pwsh) — OmnesAgent Daemon', 'Terminal (bash / pwsh) — OmnesAgent Daemon');
  static String get terminalTooltip => tr('Встроенный терминал', 'Built-in terminal');
  static String get toolsTooltip => tr('Боковая панель инструментов', 'Side tools panel');
  static String get promptPlaceholder => tr('Задайте вопрос или продолжите диалог...', 'Ask for follow-up changes...');
  static String get addContextTooltip => tr('Добавить контекст (@файл, #чат, /команда)', 'Add context (@file, #chat, /command)');
  static String get copiedToClipboard => tr('Скопировано в буфер обмена', 'Copied to clipboard');
  static String get done => tr('выполнено', 'done');
  static String get undo => tr('Отменить', 'Undo');
  static String get gitTools => tr('Инструменты Git', 'Git tools');
  static String get branch => tr('Ветка', 'Branch');
  static String get startNewTaskTitle => tr('Начните новую сессию в проекте omnes-agent', 'Start a new session in the omnes-agent project');
  static String get runSnippet => tr('Запустить в терминале', 'Run in terminal');
  static String get copySnippet => tr('Копировать сниппет', 'Copy snippet');

  // Permission Modes
  static String get fullAccess => tr('Полный доступ', 'Full access');
  static String get normalAccess => tr('Обычный доступ', 'Normal access');
  static String get safeMode => tr('Безопасный режим', 'Safe mode');
  static String get confirmAll => tr('Подтверждать всё', 'Confirm all');

  // Window Controls
  static String get minimize => tr('Свернуть', 'Minimize');
  static String get maximize => tr('Развернуть', 'Maximize');
  static String get close => tr('Закрыть', 'Close');

  // ============================================================
  // AUTOMATIONS TRANSLATIONS
  // ============================================================
  static String get automationsTitle => tr('Автоматизации', 'Automations');
  static String get automationsSubtitle => tr(
    'Планируйте периодические задачи или фоновую работу во время простоя компьютера.',
    'Schedule recurring tasks or queue background work that runs during idle time.',
  );
  static String get noScheduledTasks => tr('Запланированных задач пока нет.', 'No scheduled tasks yet.');
  static String get createScheduledTask => tr('Создать задачу по расписанию', 'Create scheduled task');
  static String get createIdleTask => tr('Создать idle-time задачу', 'Create idle-time task');
  static String get activeTasks => tr('Активные задачи', 'Active tasks');
  static String get add => tr('Добавить', 'Add');
  static String get keepAwakeText => tr(
    'Не переводить компьютер в спящий режим во время работы OmnesAgent.',
    'Keep your computer awake while OmnesAgent is running a chat.',
  );
  static String get idleTimeTemplates => tr('Шаблоны задач фонового простоя', 'Idle-time task template');
  static String get scheduledTemplates => tr('Шаблоны задач по расписанию', 'Scheduled task template');
  static String get soonestAvailable => tr('Как только возможно', 'Soonest available');
  static String get taskName => tr('Название задачи', 'Task name');
  static String get taskPrompt => tr('Промпт / Инструкция агенту', 'Prompt / Agent instructions');
  static String get taskSchedule => tr('Расписание выполнения', 'Execution schedule');
  static String get cancel => tr('Отмена', 'Cancel');
  static String get scheduleAction => tr('Запланировать', 'Schedule');

  // ============================================================
  // SETTINGS TRANSLATIONS
  // ============================================================
  static String get backToWorkspace => tr('Назад в рабочую область', 'Back to workspace');
  static String get mainSettings => tr('Основные настройки', 'General settings');
  static String get agentCapabilities => tr('Возможности агента', 'Agent capabilities');
  static String get dataAndAnalysis => tr('Данные и анализ', 'Data & analysis');
  static String get general => tr('Общие', 'General');
  static String get providers => tr('Провайдеры', 'Providers');
  static String get appearance => tr('Оформление', 'Appearance');
  static String get profile => tr('Профиль', 'Profile');
  static String get mcpServers => tr('MCP Серверы', 'MCP Servers');
  static String get skills => tr('Навыки', 'Skills');
  static String get commands => tr('Команды', 'Commands');
  static String get memoryAst => tr('ob2h AST Память', 'ob2h AST Memory');
  static String get usageStats => tr('Статистика использования', 'Usage statistics');
  static String get interfaceLanguage => tr('Язык интерфейса (Interface Language)', 'Interface Language (Язык интерфейса)');
  static String get selectInterfaceLang => tr('Выберите язык интерфейса рабочего окружения OmnesAgent.', 'Choose the UI language for OmnesAgent workstation.');
  static String get closeSettings => tr('Закрыть настройки', 'Close settings');
  static String get connected => tr('Подключено', 'Connected');
  static String get memorySettingTitle => tr('Долговременная память ob2h AST', 'Long-term ob2h AST Memory');
  static String get memorySettingSubtitle => tr('Управляет использованием графа фактов и символов кода в задачах без лишней траты токенов.', 'Controls AST code graph and facts memory without wasting context tokens.');
  static String get gatewayDaemonTitle => tr('Gateway Runtime Daemon', 'Gateway Runtime Daemon');
  static String get gatewayDaemonSubtitle => tr('Подключение клиента к локальному Rust gateway демону и исполнителям инструментов.', 'Client connection to local Rust gateway daemon and tool runners.');
  static String get terminalFontTitle => tr('Шрифт встроенного терминала', 'Integrated Terminal Font');
  static String get terminalFontSubtitle => tr('Семейство моноширинных шрифтов для встроенной консоли и вывода bash/powershell.', 'Monospace font family for built-in console and bash/powershell output.');
  static String get apply => tr('Применить', 'Apply');
  static String get addProvider => tr('Добавить свой провайдер', 'Add Custom Provider');
  static String get providerName => tr('Название провайдера', 'Provider Name');
  static String get providerUrl => tr('URL базового эндпоинта', 'Base Endpoint URL');
  static String get apiKey => tr('API Ключ (опционально)', 'API Key (optional)');
  static String get defaultModel => tr('Модель по умолчанию', 'Default Model');
  static String get save => tr('Сохранить', 'Save');
  static String get activeModelLabel => tr('Активная модель по умолчанию', 'Default Active Model');

  // ============================================================
  // INSPECTOR TRANSLATIONS
  // ============================================================
  static String get liveBrowser => tr('Live Browser', 'Live Browser');
  static String get terminal => tr('Терминал', 'Terminal');
  static String get preview => tr('Предпросмотр', 'Preview');
  static String get sideChat => tr('Боковой чат (/side)', 'Side Chat (/side)');
  static String get elementPicker => tr('Выбор элемента', 'Element Picker');
  static String get desktopMode => tr('Десктоп', 'Desktop');
  static String get tabletMode => tr('Планшет', 'Tablet');
  static String get mobileMode => tr('Мобильный', 'Mobile');
  static String get refresh => tr('Обновить', 'Refresh');
  static String get openNewTab => tr('Открыть вкладку', 'Open Tab');
  static String get sendSideChat => tr('Спросить без изменения контекста основной сессии...', 'Ask without modifying main session context...');

  // ============================================================
  // WORKSPACE TEMPLATES, COMPOSER & MENUS
  // ============================================================
  static String get heroInputHint => tr(
    'Спросите OmnesAgent, введите @ для файлов, / для команд, \$ для навыков, # для чатов',
    'Ask OmnesAgent, type @ for files, / for commands, \$ for skills, # for chats',
  );
  static String get bannerText => tr(
    'Новая функция: Создание фоновых задач "Idle-time task". Агент выполняет рутинные проверки в периоды простоя.',
    'New feature: "Idle-time tasks". The agent runs routine maintenance and checks during idle periods.',
  );
  static String get standupGitTitle => tr('Standup Git Summary', 'Standup Git Summary');
  static String get standupGitDesc => tr(
    'Сводка последних коммитов, веток и изменений за прошедшую неделю.',
    "Summarize this week's git activity: notable commits, merged branches, and what changed.",
  );
  static String get ciFailuresTitle => tr('CI Failures & Flaky Test Report', 'CI Failures & Flaky Test Report');
  static String get ciFailuresDesc => tr(
    'Отчёт о последних падениях тестов компиляции и интеграционных проверок.',
    'Scan recent CI runs, list failing and flaky tests with likely causes, and propose fixes.',
  );
  static String get customizeTitle => tr('Своя сессия', 'Customize');
  static String get customizeDesc => tr(
    'Прямой ввод свободной сессии без использования готовых шаблонов.',
    'Direct prompt input for custom workflows without preset templates.',
  );
  static String get standupPrompt => tr(
    'Сформируй Git Standup summary за последнюю неделю.',
    "Summarize this week's git activity into a Friday standup: notable commits, merged PRs, and what changed.",
  );
  static String get ciFailuresPrompt => tr(
    'Проанализируй недавние падения CI тестов и предложи исправления.',
    'Scan recent CI runs, list failing and flaky tests with likely causes, and propose fixes ranked by impact.',
  );

  // Composer Add Menu
  static String get attachFileItem => tr('Прикрепить файл (Attachment)', 'Attach file (Attachment)');
  static String get mentionFileItem => tr('Упомянуть файл (@ mention)', 'Mention file (@ mention)');
  static String get linkChatItem => tr('Связать с сессией (# chat)', 'Link to session (# chat)');
  static String get insertCommandItem => tr('Вставить команду (/ command)', 'Insert command (/ command)');

  // Permission Mode Items
  static String get askBeforeChangesTitle => tr('Ask before changes', 'Ask before changes');
  static String get askBeforeChangesDesc => tr('Спрашивать перед правками файлов.', 'Confirm before any file edits.');
  static String get editAutomaticallyTitle => tr('Edit automatically', 'Edit automatically');
  static String get editAutomaticallyDesc => tr('Автоматически вносить правки.', 'Apply code edits automatically.');
  static String get planModeTitle => tr('Plan mode', 'Plan mode');
  static String get planModeDesc => tr('Планировать шаги перед действиями.', 'Plan steps before taking actions.');
  static String get fullAccessTitle => tr('Full access', 'Full access');
  static String get fullAccessDesc => tr('Минимум подтверждений (полный доступ).', 'Minimal prompts (full autonomy).');

  // Model & Thought Menu
  static String get modelProvidersHeader => tr('Провайдеры моделей', 'Model Providers');
  static String get providerSettingsAction => tr('Настройки провайдеров', 'Provider settings');
  static String get modelTooltip => tr('Выбор модели', 'Select model');
  static String get permissionModeTooltip => tr('Режим подтверждений', 'Permission mode');
  static String get thoughtLevelTooltip => tr('Уровень рассуждений (Thought Level)', 'Thought Level');
  static String get commandCopied => tr('Команда скопирована', 'Command copied');
  static String get oneFileChanged => tr('1 файл изменён', '1 file changed');
  static String get changesLabel => tr('Изменения', 'Changes');

  // ============================================================
  // COMMAND PALETTE TRANSLATIONS
  // ============================================================
  static String get searchCommandsOrFiles => tr('Поиск команд, сессий, файлов (Ctrl+P)...', 'Search commands, sessions, files (Ctrl+P)...');
  static String get categoryActions => tr('ДЕЙСТВИЯ', 'ACTIONS');
  static String get categoryPermissionModes => tr('РЕЖИМЫ РАЗРЕШЕНИЙ', 'PERMISSION MODES');
  static String get categoryNavigation => tr('НАВИГАЦИЯ', 'NAVIGATION');
  static String get categorySystem => tr('СИСТЕМА', 'SYSTEM');
  static String get cmdNewTask => tr('Создать новую сессию', 'Create new session');
  static String get cmdToggleTerminal => tr('Открыть / закрыть терминал', 'Toggle terminal');
  static String get cmdCyclePermission => tr('Переключить режим разрешений', 'Cycle permission mode');
  static String get cmdModeAsk => tr('Режим: Ask before changes', 'Mode: Ask before changes');
  static String get cmdModeEdit => tr('Режим: Edit automatically', 'Mode: Edit automatically');
  static String get cmdModePlan => tr('Режим: Plan mode', 'Mode: Plan mode');
  static String get cmdModeFull => tr('Режим: Full access', 'Mode: Full access');
  static String get cmdOpenAutomations => tr('Перейти в Автоматизации (Крон задачи)', 'Go to Automations (Cron tasks)');
  static String get cmdOpenSettings => tr('Открыть Настройки', 'Open Settings');
  static String get cmdToggleRemote => tr('Удалённое управление (Mobile QR)', 'Remote Control (Mobile QR)');

  // ============================================================
  // ONBOARDING & PROFILE TRANSLATIONS
  // ============================================================
  static String get onboardingTitle => tr('Знакомство с OmnesAgent ADE', 'Welcome to OmnesAgent ADE');
  static String get onboardingSubtitle => tr(
    'Пожалуйста, представьтесь и ответьте на пару вопросов. Это позволит агенту лучше подстраиваться под ваш стек и стиль разработки.',
    'Please introduce yourself and answer a few questions. This helps the agent adapt to your stack and coding style.',
  );
  static String get stepPersonal => tr('1. Личные данные', '1. Personal Info');
  static String get stepPreferences => tr('2. Опросник и предпочтения', '2. Questionnaire & Preferences');
  static String get firstName => tr('Имя', 'First Name');
  static String get lastName => tr('Фамилия', 'Last Name');
  static String get roleTitle => tr('Ваша основная роль', 'Your Primary Role');
  static String get primaryStackTitle => tr('Основной стек технологий', 'Primary Tech Stack');
  static String get autonomyStyleTitle => tr('Стиль автономности агента', 'Agent Autonomy Style');
  static String get continueBtn => tr('Далее', 'Continue');
  static String get finishBtn => tr('Завершить и начать работу', 'Finish & Start Working');
  static String get editProfile => tr('Изменить профиль', 'Edit profile');

  // Roles
  static String get roleFullstack => tr('Full-stack разработчик', 'Full-stack Developer');
  static String get roleBackend => tr('Backend / Системный инженер', 'Backend / Systems Engineer');
  static String get roleFrontend => tr('Frontend / Mobile разработчик', 'Frontend / Mobile Developer');
  static String get roleArchitect => tr('Архитектор / Tech Lead', 'Architect / Tech Lead');
  static String get roleDevOps => tr('DevOps / SRE инженер', 'DevOps / SRE Engineer');

  // Autonomy Styles
  static String get autonomyCautious => tr('Осторожный (ревью каждого шага)', 'Cautious (review every step)');
  static String get autonomyBalanced => tr('Сбалансированный (авто-правки + подтверждение команд)', 'Balanced (auto-edits + confirm commands)');
  static String get autonomyAutonomous => tr('Максимально автономный (полная свобода)', 'Highly autonomous (full freedom)');
}

