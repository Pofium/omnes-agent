import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:http/http.dart' as http;
import 'package:omnes_shared/omnes_shared.dart';
import 'package:path/path.dart' as p;
import 'package:universal_io/io.dart';
import 'package:universal_io/io.dart' as universal_io;

import '../../utils/desktop_i18n.dart';
import 'task_session.dart';
import 'workspace_git_helper.dart';
import 'workspace_stream_coordinator.dart';
import 'workspace_terminal_controller.dart';

export 'task_session.dart';
export 'workspace_git_helper.dart';
export 'workspace_stream_coordinator.dart';
export 'workspace_terminal_controller.dart';

class DesktopTaskWorkspaceController extends GetxController with WidgetsBindingObserver {

  final inputController = TextEditingController();
  final inputFocusNode = FocusNode();
  final scrollController = ScrollController();
  final isAutoScrollEnabled = true.obs;
  final isScrolledUp = false.obs;

  /// Resolves the initial project root path dynamically.
  static String resolveInitialProjectPath() => WorkspaceGitHelper.resolveInitialProjectPath();

  /// Resolves the initial project name dynamically from the directory path.
  static String resolveInitialProjectName([String? projectPath]) =>
      WorkspaceGitHelper.resolveInitialProjectName(projectPath);

  /// Synchronously reads current Git branch from .git/HEAD in 0ms without running sub-processes.
  static String? readGitHeadBranchSync(String? projectPath) =>
      WorkspaceGitHelper.readGitHeadBranchSync(projectPath);

  /// Synchronously reads available local branches from .git/refs/heads in 0ms.
  static List<String> readGitLocalBranchesSync(String? projectPath) =>
      WorkspaceGitHelper.readGitLocalBranchesSync(projectPath);

  final isRunning = false.obs;
  DateTime? _lastStreamChunkTime;

  // Active Session info
  final activeSessionId = 'session-init'.obs;
  late final activeTaskTitle = RxString('${DesktopI18n.tr('Рабочая область', 'Workspace')} ${resolveInitialProjectName()}');
  late final activeGroup = RxnString(resolveInitialProjectName());
  late final activeProject = RxnString(resolveInitialProjectName());
  late final activeProjectPath = RxnString(resolveInitialProjectPath());
  final activeProjectIcon = RxString('code');
  final activeProjectColor = RxString('#00D2FF');
  late final activeBranch = RxnString(readGitHeadBranchSync(resolveInitialProjectPath()) ?? 'main');
  late final availableBranches = <String>[
    ...readGitLocalBranchesSync(resolveInitialProjectPath()).isNotEmpty
        ? readGitLocalBranchesSync(resolveInitialProjectPath())
        : (readGitHeadBranchSync(resolveInitialProjectPath()) != null
            ? [readGitHeadBranchSync(resolveInitialProjectPath())!]
            : ['main'])
  ].obs;
  final isNewTask = false.obs;

  // Multi-session tabs
  final openSessionTabs = <String>[].obs;

  // Project Git Changes Summary (contextual to active project/group)
  final hasGitRepo = true.obs;
  final gitAdditions = 0.obs;
  final gitDeletions = 0.obs;
  final gitModifiedFiles = <String>[].obs;
  final gitStagedFiles = <String>{}.obs;
  final gitSummary = ''.obs;
  final isTaskBarExpanded = false.obs;

  // Project File Viewer in Inspector
  final selectedFilePath = RxnString();
  final selectedFileContent = RxnString();

  // Gateway Clients & Connectivity
  final GatewayHttpClient httpClient = GatewayHttpClient();
  final EditorApi editorApi = EditorApi();
  late final WorkspaceStreamCoordinator streamCoordinator = WorkspaceStreamCoordinator(
    onFrameReceived: _handleGatewayFrame,
    onSseReceived: _handleSseEvent,
  );

  RxString get wsStatus => streamCoordinator.wsStatus;
  final isGatewayHealthy = false.obs;
  final backendSessions = <SessionInfo>[].obs;

  // OmnesAgent ADE Core State
  final permissionMode = PermissionMode.fullAccess.obs;
  final thoughtLevel = 'Max'.obs;
  final activeModel = DesktopI18n.tr('Провайдер не настроен', 'Provider not configured').obs;
  final activeProvider = ''.obs;
  final configuredProviders = <Map<String, dynamic>>[].obs;
  final isSttConfiguredAndEnabled = false.obs;
  final configuredModels = <String>[].obs;
  final tokenCount = '28.4k tokens'.obs;
  // Context Gauge & Token Meter
  final usedTokens = 0.obs;
  final maxTokens = 128000.obs;
  final estimatedCost = 0.0.obs;

  // DeepSeek / OmnesAgent Harness state
  final trajectorySteps = <TrajectoryStep>[].obs;
  final currentContextBudget = Rxn<ContextBudgetSnapshot>();
  final currentExecutionMode = AgentExecutionMode.fast.obs;

  // Project Rules Inspector
  final projectRulesContent = ''.obs;
  final isProjectRulesEnabled = true.obs;

  // Session Mode & UI Flags from Orchestrator
  final currentSessionMode = 'chat'.obs;
  final showTodoWidget = false.obs;

  late final WorkspaceTerminalController terminalController = WorkspaceTerminalController(
    getWorkingDirectory: () =>
        sessions[activeSessionId.value]?.projectPath ??
        activeProjectPath.value ??
        universal_io.Directory.current.path,
    onTerminalToggled: (isOpen) {
      final current = sessions[activeSessionId.value];
      if (current != null) {
        current.isTerminalOpen = isOpen;
      }
    },
  );

  RxBool get isTerminalOpen => terminalController.isTerminalOpen;
  RxList<String> get terminalLines => terminalController.terminalLines;
  RxBool get isTerminalRunning => terminalController.isTerminalRunning;

  void toggleTerminal() => terminalController.toggleTerminal();
  Future<void> executeTerminalCommand(String rawCmd) =>
      terminalController.executeTerminalCommand(rawCmd);
  void clearTerminal() => terminalController.clearTerminal();

  // Voice Duplex Mode State
  final isVoiceDuplexActive = false.obs;
  final voiceStatusText = DesktopI18n.tr('Голосовой дуплекс готов', 'Voice duplex ready').obs;

  void toggleVoiceDuplex() {
    isVoiceDuplexActive.value = !isVoiceDuplexActive.value;
    if (isVoiceDuplexActive.value) {
      voiceStatusText.value = DesktopI18n.tr('Слушаю... (голосовой поток в шлюз активен)', 'Listening... (audio stream to gateway active)');
    } else {
      voiceStatusText.value = DesktopI18n.tr('Голосовой дуплекс отключен', 'Voice duplex disabled');
    }
  }

  // Goal Mode State
  final Rxn<GoalStatus> activeGoal = Rxn<GoalStatus>();

  // Attachments & Mentions (@file, #chat, chips)
  final attachments = <String>[].obs;

  // Pending prompt queued while agent is running
  final queuedPrompt = Rxn<String>();
  http.Client? _activeDirectClient;

  final messages = <ChatMessage>[].obs;
  final pendingApprovals = <Map<String, dynamic>>[].obs;
  final runTimelineSteps = <Map<String, dynamic>>[].obs;

  // Multi-session repository
  final RxMap<String, TaskSession> sessions = <String, TaskSession>{}.obs;

  @override
  void onInit() {
    super.onInit();
    inputFocusNode.onKeyEvent = (node, event) {
      if (event is KeyDownEvent &&
          (event.logicalKey == LogicalKeyboardKey.enter ||
           event.logicalKey == LogicalKeyboardKey.numpadEnter) &&
          !HardwareKeyboard.instance.isShiftPressed) {
        sendMessage();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    };
    WidgetsBinding.instance.addObserver(this);
    scrollController.addListener(_onScrollChanged);
    final isSeeded = GetStorage().read<bool>('desktop_sessions_seeded') ?? false;
    if (!isSeeded) {
      _initSampleSessions();
      GetStorage().write('desktop_sessions_seeded', true);
    }
    _loadLocalCustomSessions();
    loadConfiguredProviders();
    _loadSttSettings();
    loadProjectRules();
    final savedMode = GetStorage().read<String>('desktop_execution_mode');
    if (savedMode != null) {
      currentExecutionMode.value = AgentExecutionMode.fromId(savedMode);
    }
    final savedTabs = GetStorage().read<List>('desktop_open_tabs');
    if (savedTabs != null && savedTabs.isNotEmpty) {
      final validTabs = savedTabs
          .map((e) => e.toString())
          .where((id) => !_isSessionDeleted(id) && sessions.containsKey(id))
          .toList();
      openSessionTabs.assignAll(validTabs);
    }
    final startupMode = GetStorage().read<String>('startup_session_mode') ?? 'last';
    if (startupMode == 'new') {
      createNewTask();
    } else {
      final savedActive = GetStorage().read<String>('desktop_active_session_id');
      if (savedActive != null && sessions.containsKey(savedActive) && !_isSessionDeleted(savedActive)) {
        switchToSession(savedActive);
      } else {
        final available = sessions.keys.where((k) => !_isSessionDeleted(k)).toList();
        if (available.isNotEmpty) {
          switchToSession(available.first);
        } else {
          createNewTask();
        }
      }
    }
    if (openSessionTabs.isEmpty && activeSessionId.value.isNotEmpty && sessions.containsKey(activeSessionId.value) && !_isSessionDeleted(activeSessionId.value)) {
      openSessionTabs.add(activeSessionId.value);
      GetStorage().write('desktop_open_tabs', openSessionTabs.toList());
    }
    refreshGitStatus();
    _sanitizeAllSessionMessages();
    initGatewayConnection();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      _handleAppResume();
    }
  }

  /// Cleans up any zombie streams or hanging spinners after system wake-up from sleep/hibernate
  void _handleAppResume() {
    if (isRunning.value) {
      if (_lastStreamChunkTime == null ||
          DateTime.now().difference(_lastStreamChunkTime!).inSeconds > 20) {
        abortRun();
      }
    } else {
      _sanitizeAllSessionMessages();
      messages.refresh();
    }
  }

  /// Sanitizes all session messages, ensuring no lingering streaming flags or running spinners persist across launches
  void _sanitizeAllSessionMessages() {
    for (final session in sessions.values) {
      for (final msg in session.messages) {
        msg.isStreaming = false;
        msg.isThinkingFinished = true;
        msg.isThinkingExpanded = false;
        for (final step in msg.steps) {
          step.isRunning = false;
        }
      }
    }
    for (final msg in messages) {
      msg.isStreaming = false;
      msg.isThinkingFinished = true;
      msg.isThinkingExpanded = false;
      for (final step in msg.steps) {
        step.isRunning = false;
      }
    }
    isRunning.value = false;
  }

  /// Initializes live gateway connection (Health, SSE, Sessions, and active WS).
  Future<void> initGatewayConnection() async {
    try {
      final healthy = await httpClient.checkHealth();
      isGatewayHealthy.value = healthy;

      // Start global SSE stream
      streamCoordinator.startSse();

      // Load backend sessions
      await fetchBackendSessions();

      // Connect WS for currently selected session
      _connectWebSocket(activeSessionId.value);
    } catch (_) {}
  }

  void _handleSseEvent(GatewaySystemEvent event) {
    if (event.type == 'message' && event.sessionId == activeSessionId.value) {
      final content = event.data['content']?.toString() ?? '';
      if (content.isNotEmpty && (messages.isEmpty || messages.last.text != content)) {
        messages.add(ChatMessage(text: content, chatMessageType: ChatMessageType.bot));
        _scrollToBottom();
      }
    } else if (event.type == 'session_created' || event.type == 'session_deleted') {
      fetchBackendSessions();
    }
  }

  /// Set of deleted session IDs that should never be resurrected
  Set<String> _getDeletedSessionIds() {
    try {
      final list = GetStorage().read<List>('desktop_deleted_sessions');
      if (list != null) {
        return list.map((e) => e.toString()).toSet();
      }
    } catch (_) {}
    return <String>{};
  }

  /// Check if a session ID or any of its prefixed/unprefixed forms was marked as deleted
  bool _isSessionDeleted(String id) {
    final deleted = _getDeletedSessionIds();
    if (deleted.contains(id)) return true;
    if (id.startsWith('gw_')) {
      if (deleted.contains(id.substring(3))) return true;
    } else {
      if (deleted.contains('gw_$id')) return true;
    }
    return false;
  }

  void _recordDeletedSession(String id) {
    try {
      final set = _getDeletedSessionIds();
      set.add(id);
      if (id.startsWith('gw_')) {
        set.add(id.substring(3));
      } else {
        set.add('gw_$id');
      }
      GetStorage().write('desktop_deleted_sessions', set.toList());
    } catch (_) {}
  }

  void _initSampleSessions() {
    final deleted = _getDeletedSessionIds();
    final initProj = resolveInitialProjectName();
    final initPath = resolveInitialProjectPath();
    final initBranch = readGitHeadBranchSync(initPath) ?? 'main';
    final hasGit = universal_io.Directory(p.join(initPath, '.git')).existsSync();

    // 1. Primary workspace session for currently opened project
    if (!deleted.contains('session-init')) {
      sessions['session-init'] = TaskSession(
        id: 'session-init',
        title: 'Рабочая область $initProj',
        group: initProj,
        project: initProj,
        projectPath: initPath,
        branch: initBranch,
        hasGitRepo: hasGit,
        additions: 0,
        deletions: 0,
        activeModel: 'GLM-5.3-Flash',
        messages: [
          ChatMessage(
            text: 'Добро пожаловать в OmnesAgent. Рабочая область проекта "$initProj" ($initPath) готова к решению задач.',
            chatMessageType: ChatMessageType.bot,
          ),
        ],
      );
    }

    // 2. quick-draft (Default group "Разное", without repository)
    if (!deleted.contains('quick-draft')) {
      sessions['quick-draft'] = TaskSession(
        id: 'quick-draft',
        title: 'Заметки и идеи',
        group: 'Разное',
        project: null,
        projectPath: null,
        branch: null,
        hasGitRepo: false,
        additions: 0,
        deletions: 0,
        activeModel: 'GLM-5.3-Flash',
        messages: [
          ChatMessage(
            text: 'Напомни ключевые концепции протокола Model Context Protocol (MCP).',
            chatMessageType: ChatMessageType.user,
          ),
          ChatMessage(
            text: 'Основные концепции MCP:\n\n1. **Tools**: функции с JSON-Schema, вызываемые моделью.\n2. **Resources**: URI-адресуемые данные (файлы, базы данных, логи).\n3. **Prompts**: шаблоны взаимодействия.\n4. **Sampling**: запрос агента к хост-модели для выполнения рассуждений.',
            chatMessageType: ChatMessageType.bot,
          ),
        ],
      );
    }
  }

  /// Switch active session or create new one if needed
  void switchToSession(
    String id, {
    String? title,
    String? group,
    String? project,
    String? projectPath,
    String? branch,
  }) {
    if (_isSessionDeleted(id)) {
      final available = sessions.keys.where((k) => !_isSessionDeleted(k)).toList();
      if (available.isNotEmpty) {
        switchToSession(available.first);
      } else {
        createNewTask();
      }
      return;
    }

    _saveCurrentSessionState();

    var session = sessions[id];
    if (session == null) {
      final effectiveGroup = group ?? project ?? 'Разное';
      session = TaskSession(
        id: id,
        title: title ?? id,
        group: effectiveGroup,
        project: project,
        projectPath: projectPath,
        branch: branch ?? 'main',
        hasGitRepo: project != null,
        additions: 0,
        deletions: 0,
        messages: [
          ChatMessage(
            text: 'Сессия: **${title ?? id}**\n${project != null ? "Проект: `$project` | " : ""}Группа: `$effectiveGroup`',
            chatMessageType: ChatMessageType.bot,
          ),
        ],
      );
      sessions[id] = session;
      _saveLocalCustomSessions();
      sessions.refresh();
    }

    if (!openSessionTabs.contains(id)) {
      openSessionTabs.add(id);
      GetStorage().write('desktop_open_tabs', openSessionTabs.toList());
      openSessionTabs.refresh();
    }

    activeSessionId.value = id;
    activeTaskTitle.value = session.title;
    activeGroup.value = session.group;
    activeProject.value = session.project;
    activeProjectPath.value = session.projectPath;
    _updateProjectMeta(session.project);
    activeBranch.value = session.branch;
    hasGitRepo.value = session.hasGitRepo;
    gitAdditions.value = session.additions;
    gitDeletions.value = session.deletions;
    permissionMode.value = session.permissionMode;
    currentExecutionMode.value = session.executionMode;
    thoughtLevel.value = session.thoughtLevel;
    activeModel.value = (configuredModels.contains(session.activeModel))
          ? session.activeModel
          : (configuredModels.isNotEmpty ? configuredModels.first : 'Провайдер не настроен');
    tokenCount.value = session.tokenCount;
    activeGoal.value = session.activeGoal;
    isTerminalOpen.value = session.isTerminalOpen;

    session.messages.removeWhere((m) =>
        m.isError &&
        (m.text.contains('Connection failed: WebSocketChannelException') ||
         m.text.contains('Удаленный компьютер отклонил это сетевое подключение')));

    // Restore persistent messages from local storage if available
    final hasLoaded = _loadSessionMessages(id);
    if (!hasLoaded) {
      messages.assignAll(session.messages);
    }
    runTimelineSteps.assignAll(session.runTimelineSteps);
    trajectorySteps.clear();
    currentContextBudget.value = null;
    refreshGitStatus();
    attachments.clear();
    inputController.clear();

    isNewTask.value = false;
    _updateTokenUsageStats();
    loadProjectRules();
    _scrollToBottom();

    // Connect WebSocket and fetch backend messages for this session
    _connectWebSocket(id);
    fetchSessionMessages(id);
  }

  void _updateProjectMeta(String? projName) {
    if (projName == null || projName.isEmpty) return;
    try {
      final raw = GetStorage().read<List>('user_projects_registry');
      if (raw != null) {
        for (final item in raw) {
          if (item is Map && (item['name'] == projName || item['id'] == projName)) {
            if (item['iconName'] != null) activeProjectIcon.value = item['iconName'].toString();
            if (item['colorHex'] != null) activeProjectColor.value = item['colorHex'].toString();
            return;
          }
        }
      }
    } catch (_) {}
  }

  void closeSessionTab(String id) {
    openSessionTabs.remove(id);
    openSessionTabs.remove('gw_$id');
    if (id.startsWith('gw_')) {
      openSessionTabs.remove(id.substring(3));
    }
    GetStorage().write('desktop_open_tabs', openSessionTabs.toList());
    openSessionTabs.refresh();
    if (activeSessionId.value == id ||
        activeSessionId.value == 'gw_$id' ||
        (id.startsWith('gw_') && activeSessionId.value == id.substring(3))) {
      if (openSessionTabs.isNotEmpty) {
        switchToSession(openSessionTabs.last);
      } else {
        createNewTask();
      }
    }
  }

  /// Sets the active agent execution mode, updating current session and persistent storage
  void setExecutionMode(AgentExecutionMode mode) {
    currentExecutionMode.value = mode;
    final currentSession = sessions[activeSessionId.value];
    if (currentSession != null) {
      currentSession.executionMode = mode;
    }
    GetStorage().write('desktop_execution_mode', mode.id);
  }

  /// Shows file in Windows Explorer
  Future<void> showFileInExplorer(String path) async {
    final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
    String fullPath = path;
    if (projPath != null && !path.contains(':')) {
      fullPath = '$projPath\\$path'.replaceAll('/', '\\');
    }
    if (universal_io.Platform.isWindows) {
      await Process.run('explorer.exe', ['/select,', fullPath]);
    }
  }

  /// Extensions served as binary previews (image viewer) — never read as text.
  static const Set<String> _kBinaryPreviewExtensions = {
    'png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'wbmp', 'avif', 'svg',
  };

  Future<void> openProjectFile(String path, [String? content]) async {
    selectedFilePath.value = path;
    if (content != null) {
      selectedFileContent.value = content;
      return;
    }
    final ext = path.contains('.') ? path.split('.').last.toLowerCase() : '';
    if (_kBinaryPreviewExtensions.contains(ext)) {
      // Binary asset: the image viewer fetches bytes via the gateway
      // (`/api/v1/editor/raw`); reading it as text here is meaningless.
      selectedFileContent.value = null;
      return;
    }
    try {
      final res = await editorApi.openFile(
        path: path,
        root: activeProjectPath.value,
      );
      selectedFileContent.value = res.content;
    } catch (e) {
      selectedFileContent.value = '// Ошибка открытия файла через шлюз: $e';
    }
  }

  
  /// Loads actively configured LLM providers and updates available models.
  void loadConfiguredProviders() {
    try {
      final storage = GetStorage();
      final List<Map<String, dynamic>> providersList = [];

      final standardDefs = [
        {'id': 'openai', 'name': 'OpenAI', 'models': ['gpt-4o', 'gpt-4o-mini', 'o1-preview', 'o3-mini']},
        {'id': 'anthropic', 'name': 'Anthropic', 'models': ['claude-3-5-sonnet', 'claude-3-5-haiku', 'claude-3-opus']},
        {'id': 'deepseek', 'name': 'DeepSeek', 'models': ['deepseek-chat', 'deepseek-reasoner']},
        {'id': 'gemini', 'name': 'Gemini', 'models': ['gemini-1.5-pro', 'gemini-1.5-flash', 'gemini-2.0-flash']},
        {'id': 'glm', 'name': 'Zhipu GLM', 'models': ['GLM-5.3', 'GLM-5.3-Flash', 'GLM-4-Plus']},
        {'id': 'groq', 'name': 'Groq', 'models': ['llama-3.3-70b-versatile', 'deepseek-r1-distill-llama-70b']},
        {'id': 'openrouter', 'name': 'OpenRouter', 'models': ['anthropic/claude-3.5-sonnet', 'deepseek/deepseek-r1']},
        {'id': 'mistral', 'name': 'Mistral', 'models': ['mistral-large-latest', 'codestral-latest']},
        {'id': 'moonshot', 'name': 'Moonshot (Kimi)', 'models': ['kimi-k2.6', 'moonshot-v1-128k']},
        {'id': 'minimax', 'name': 'MiniMax', 'models': ['abab6.5s-chat']},
        {'id': 'qwen', 'name': 'Alibaba Qwen', 'models': ['qwen-max', 'qwen-plus', 'qwen-coder-plus']},
      ];

      for (final p in standardDefs) {
        final id = p['id'] as String;
        final isExplicitlyEnabled = storage.read<bool>('provider_enabled_$id');
        if (isExplicitlyEnabled == false) {
          continue; // User explicitly disabled this provider
        }

        final savedKey = storage.read<String>('provider_key_$id') ?? '';
        final hasSavedKey = savedKey.trim().isNotEmpty;

        var key = savedKey;
        final bool isFromEnv;
        if (key.trim().isEmpty) {
          key = universal_io.Platform.environment['${id.toUpperCase()}_API_KEY'] ??
              (id == 'qwen' ? universal_io.Platform.environment['DASHSCOPE_API_KEY'] : null) ??
              (id == 'moonshot' ? universal_io.Platform.environment['KIMI_API_KEY'] : null) ?? '';
          isFromEnv = key.trim().isNotEmpty;
        } else {
          isFromEnv = false;
        }

        final shouldInclude = (hasSavedKey && isExplicitlyEnabled != false) ||
            (isFromEnv && isExplicitlyEnabled == true);

        if (shouldInclude && key.trim().isNotEmpty) {
          final cachedModels = storage.read<List>('provider_models_$id');
          final modelsList = cachedModels != null && cachedModels.isNotEmpty
              ? List<String>.from(cachedModels.map((e) => e.toString()))
              : List<String>.from(p['models'] as List);

          providersList.add({
            'id': id,
            'name': p['name'] as String,
            'models': modelsList,
            'isCustom': false,
          });
        }
      }

      final ollamaEnabled = storage.read<bool>('provider_enabled_ollama') ?? false;
      if (ollamaEnabled) {
        final cachedModels = storage.read<List>('provider_models_ollama');
        final modelsList = cachedModels != null && cachedModels.isNotEmpty
            ? List<String>.from(cachedModels.map((e) => e.toString()))
            : ['qwen2.5-coder:32b', 'deepseek-r1:14b', 'llama3.2'];

        providersList.add({
          'id': 'ollama',
          'name': 'Ollama (Local)',
          'models': modelsList,
          'isCustom': false,
        });
      }

      final lmStudioEnabled = storage.read<bool>('provider_enabled_lmstudio') ?? false;
      if (lmStudioEnabled) {
        final cachedModels = storage.read<List>('provider_models_lmstudio');
        final modelsList = cachedModels != null && cachedModels.isNotEmpty
            ? List<String>.from(cachedModels.map((e) => e.toString()))
            : ['local-model', 'qwen2.5-coder-7b-instruct', 'deepseek-r1-distill-qwen-7b'];

        providersList.add({
          'id': 'lmstudio',
          'name': 'LM Studio (Local)',
          'models': modelsList,
          'isCustom': false,
        });
      }

      final savedCustom = storage.read<List>('custom_providers_registry') ?? [];
      for (final item in savedCustom) {
        if (item is Map) {
          final id = item['id']?.toString() ?? '';
          final isEnabled = storage.read<bool>('provider_enabled_$id') ?? true;
          if (!isEnabled) continue;

          final name = item['name']?.toString() ?? 'Custom';
          final key = storage.read<String>('provider_key_$id') ?? item['key']?.toString() ?? '';
          final url = storage.read<String>('provider_url_$id') ?? item['url']?.toString() ?? '';
          final defaultModel = storage.read<String>('provider_model_$id') ?? item['defaultModel']?.toString() ?? 'custom-model';
          final cachedModels = storage.read<List>('provider_models_$id');
          final customModels = cachedModels != null && cachedModels.isNotEmpty
              ? List<String>.from(cachedModels.map((e) => e.toString()))
              : (item['models'] is List ? List<String>.from(item['models'] as List) : [defaultModel]);
          if (!customModels.contains(defaultModel) && defaultModel.isNotEmpty) {
            customModels.insert(0, defaultModel);
          }

          if (id.isNotEmpty && (key.trim().isNotEmpty || url.trim().isNotEmpty || defaultModel.isNotEmpty)) {
            providersList.add({
              'id': id,
              'name': name,
              'models': customModels,
              'isCustom': true,
              'defaultModel': defaultModel,
            });
          }
        }
      }

      configuredProviders.assignAll(providersList);

      if (configuredProviders.isEmpty) {
        activeProvider.value = '';
        configuredModels.clear();
        activeModel.value = 'Провайдер не настроен';
        return;
      }

      final currentProv = configuredProviders.firstWhereOrNull((p) => p['id'] == activeProvider.value);
      if (currentProv == null) {
        activeProvider.value = configuredProviders.first['id'] as String;
      }

      _updateModelsForActiveProvider();

      // Automatically query BASE_URL in background to discover live models without blocking
      if (activeProvider.value.isNotEmpty) {
        fetchModelsForProvider(activeProvider.value);
      }
    } catch (_) {}
  }

  void _updateModelsForActiveProvider() {
    final currentProv = configuredProviders.firstWhereOrNull((p) => p['id'] == activeProvider.value);
    if (currentProv != null) {
      final models = List<String>.from(currentProv['models'] as List);
      configuredModels.assignAll(models);
      if (!configuredModels.contains(activeModel.value)) {
        activeModel.value = configuredModels.isNotEmpty ? configuredModels.first : 'Провайдер не настроен';
      }
    } else if (configuredProviders.isNotEmpty) {
      final models = <String>[];
      for (final p in configuredProviders) {
        models.addAll(List<String>.from(p['models'] as List));
      }
      configuredModels.assignAll(models);
      if (!configuredModels.contains(activeModel.value)) {
        activeModel.value = configuredModels.isNotEmpty ? configuredModels.first : 'Провайдер не настроен';
      }
    } else {
      configuredModels.clear();
      activeModel.value = 'Провайдер не настроен';
    }
  }

  void setProvider(String providerId) {
    activeProvider.value = providerId;
    _updateModelsForActiveProvider();
    final current = sessions[activeSessionId.value];
    if (current != null) {
      current.activeModel = activeModel.value;
    }
    update();
    // Auto-fetch latest models from the provider's BASE_URL in background
    fetchModelsForProvider(providerId);
  }

  /// Fetches the list of models from an OpenAI-compatible /v1/models or Ollama /api/tags endpoint.
  static Future<List<String>> fetchModelsFromBaseUrl(String baseUrl, {String? apiKey}) async {
    if (baseUrl.trim().isEmpty) return [];
    try {
      var clean = baseUrl.trim();
      while (clean.endsWith('/')) {
        clean = clean.substring(0, clean.length - 1);
      }
      if (clean.endsWith('/chat/completions')) {
        clean = clean.substring(0, clean.length - '/chat/completions'.length);
      }
      while (clean.endsWith('/')) {
        clean = clean.substring(0, clean.length - 1);
      }

      final uri = Uri.parse(clean.endsWith('/models') ? clean : '$clean/models');
      final headers = <String, String>{
        'Accept': 'application/json',
        if (apiKey != null && apiKey.trim().isNotEmpty) 'Authorization': 'Bearer ${apiKey.trim()}',
      };

      final client = http.Client();
      final res = await client.get(uri, headers: headers).timeout(const Duration(seconds: 12));
      if (res.statusCode == 200) {
        final decoded = jsonDecode(utf8.decode(res.bodyBytes));
        final result = <String>[];
        if (decoded is Map<String, dynamic>) {
          if (decoded['data'] is List) {
            for (final item in decoded['data'] as List) {
              if (item is Map && item['id'] != null) {
                result.add(item['id'].toString());
              } else if (item is String) {
                result.add(item);
              }
            }
          } else if (decoded['models'] is List) {
            for (final item in decoded['models'] as List) {
              if (item is Map && (item['name'] != null || item['id'] != null)) {
                result.add((item['name'] ?? item['id']).toString());
              } else if (item is String) {
                result.add(item);
              }
            }
          }
        } else if (decoded is List) {
          for (final item in decoded) {
            if (item is Map && item['id'] != null) {
              result.add(item['id'].toString());
            } else if (item is String) {
              result.add(item);
            }
          }
        }
        if (result.isNotEmpty) {
          return result.toSet().toList();
        }
      }

      // Fallback for Ollama if /models returned non-200 or empty
      if (clean.contains(':11434') || clean.contains('ollama')) {
        final ollamaUri = Uri.parse('$clean/api/tags');
        final ollamaRes = await client.get(ollamaUri).timeout(const Duration(seconds: 8));
        if (ollamaRes.statusCode == 200) {
          final decoded = jsonDecode(utf8.decode(ollamaRes.bodyBytes));
          if (decoded is Map && decoded['models'] is List) {
            final result = <String>[];
            for (final item in decoded['models'] as List) {
              if (item is Map && item['name'] != null) {
                result.add(item['name'].toString());
              }
            }
            if (result.isNotEmpty) return result.toSet().toList();
          }
        }
      }
    } catch (_) {}
    return [];
  }

  /// Discovers and updates available models for a provider using its configured BASE_URL.
  Future<List<String>> fetchModelsForProvider(String providerId, {bool notify = false}) async {
    if (providerId.isEmpty) return [];
    final storage = GetStorage();

    String customKey = (storage.read<String>('provider_key_$providerId') ?? '').trim();
    String customUrl = (storage.read<String>('provider_url_$providerId') ?? '').trim();

    // Check custom providers registry
    final savedCustom = storage.read<List>('custom_providers_registry') ?? [];
    for (final item in savedCustom) {
      if (item is Map && (item['id'] == providerId || item['name'] == providerId)) {
        if (customKey.isEmpty) customKey = (item['key']?.toString() ?? '').trim();
        if (customUrl.isEmpty) customUrl = (item['url']?.toString() ?? '').trim();
      }
    }

    if (customKey.isEmpty) {
      if (providerId == 'openai') {
        customKey = universal_io.Platform.environment['OPENAI_API_KEY'] ?? '';
      } else if (providerId == 'deepseek') {
        customKey = universal_io.Platform.environment['DEEPSEEK_API_KEY'] ?? '';
      } else if (providerId == 'anthropic') {
        customKey = universal_io.Platform.environment['ANTHROPIC_API_KEY'] ?? '';
      } else if (providerId == 'gemini') {
        customKey = universal_io.Platform.environment['GEMINI_API_KEY'] ?? '';
      } else if (providerId == 'groq') {
        customKey = universal_io.Platform.environment['GROQ_API_KEY'] ?? '';
      } else if (providerId == 'openrouter') {
        customKey = universal_io.Platform.environment['OPENROUTER_API_KEY'] ?? '';
      } else if (providerId == 'mistral') {
        customKey = universal_io.Platform.environment['MISTRAL_API_KEY'] ?? '';
      } else if (providerId == 'moonshot') {
        customKey = universal_io.Platform.environment['MOONSHOT_API_KEY'] ?? universal_io.Platform.environment['KIMI_API_KEY'] ?? '';
      } else if (providerId == 'qwen') {
        customKey = universal_io.Platform.environment['QWEN_API_KEY'] ?? universal_io.Platform.environment['DASHSCOPE_API_KEY'] ?? '';
      }
    }

    if (customUrl.isEmpty) {
      if (providerId == 'deepseek') {
        customUrl = universal_io.Platform.environment['DEEPSEEK_BASE_URL'] ?? 'https://api.deepseek.com/v1';
      } else if (providerId == 'openai') {
        customUrl = universal_io.Platform.environment['OPENAI_BASE_URL'] ?? 'https://api.openai.com/v1';
      } else if (providerId == 'anthropic') {
        customUrl = universal_io.Platform.environment['ANTHROPIC_BASE_URL'] ?? 'https://api.anthropic.com/v1';
      } else if (providerId == 'glm') {
        customUrl = 'https://open.bigmodel.cn/api/paas/v4';
      } else if (providerId == 'groq') {
        customUrl = 'https://api.groq.com/openai/v1';
      } else if (providerId == 'openrouter') {
        customUrl = universal_io.Platform.environment['OPENROUTER_BASE_URL'] ?? 'https://openrouter.ai/api/v1';
      } else if (providerId == 'mistral') {
        customUrl = 'https://api.mistral.ai/v1';
      } else if (providerId == 'moonshot') {
        customUrl = 'https://api.moonshot.cn/v1';
      } else if (providerId == 'qwen') {
        customUrl = universal_io.Platform.environment['QWEN_BASE_URL'] ?? 'https://dashscope.aliyuncs.com/compatible-mode/v1';
      } else if (providerId == 'ollama') {
        customUrl = universal_io.Platform.environment['OLLAMA_HOST'] ??
            universal_io.Platform.environment['OLLAMA_BASE_URL'] ??
            'http://localhost:11434/v1';
      } else if (providerId == 'lmstudio' || providerId == 'local') {
        customUrl = universal_io.Platform.environment['LMSTUDIO_BASE_URL'] ??
            'http://localhost:1234/v1';
      }
    }

    if (customUrl.isEmpty) return [];

    final fetchedModels = await fetchModelsFromBaseUrl(customUrl, apiKey: customKey);
    if (fetchedModels.isNotEmpty) {
      storage.write('provider_models_$providerId', fetchedModels);

      for (final p in configuredProviders) {
        if (p['id'] == providerId) {
          p['models'] = fetchedModels;
        }
      }

      final rawRegistry = storage.read<List>('custom_providers_registry') ?? [];
      final registry = List<Map<String, dynamic>>.from(
        rawRegistry.whereType<Map>().map((m) => Map<String, dynamic>.from(m)),
      );
      bool regUpdated = false;
      for (final item in registry) {
        if (item['id'] == providerId) {
          item['models'] = fetchedModels;
          regUpdated = true;
        }
      }
      if (regUpdated) {
        storage.write('custom_providers_registry', registry);
      }

      if (activeProvider.value == providerId) {
        configuredModels.assignAll(fetchedModels);
        if (!configuredModels.contains(activeModel.value)) {
          activeModel.value = configuredModels.first;
          final current = sessions[activeSessionId.value];
          if (current != null) {
            current.activeModel = activeModel.value;
          }
        }
      }

      if (notify) {
        Get.snackbar(
          'Модели обновлены',
          'Получено ${fetchedModels.length} моделей от провайдера $providerId',
          snackPosition: SnackPosition.BOTTOM,
          duration: const Duration(seconds: 3),
        );
      }
      return fetchedModels;
    }
    return [];
  }

  String _generateTopicTitle(String prompt) {
    String clean = prompt.trim()
        .replaceAll(RegExp(r'^(/[\w\-]+|\*+|#+|[-*•"«»])\s*'), '')
        .trim();
    final firstLine = clean.split('\n').first.trim();
    if (firstLine.isEmpty) return 'Новая сессия';
    final formatted = firstLine.length > 1
        ? '${firstLine[0].toUpperCase()}${firstLine.substring(1)}'
        : firstLine.toUpperCase();
    if (formatted.length <= 35) return formatted;
    final truncated = formatted.substring(0, 35);
    final lastSpace = truncated.lastIndexOf(' ');
    if (lastSpace > 12) {
      return '${truncated.substring(0, lastSpace)}...';
    }
    return '$truncated...';
  }

  List<Map<String, dynamic>> _generateDynamicStepsForPrompt(String prompt) {
    if (currentExecutionMode.value == AgentExecutionMode.ralphLoop) {
      return [
        {'title': DesktopI18n.tr('Ralph: Анализ контекста и AST-зависимостей', 'Ralph: Context & AST dependency analysis'), 'status': 'running'},
        {'title': DesktopI18n.tr('Ralph: Формулирование верификационного теста', 'Ralph: Verification test formulation'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Ralph: Итеративная разработка и правка кода', 'Ralph: Iterative coding & patch loop'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Ralph: Проверка регрессий и вердикт L0/L1/L2', 'Ralph: Regression check & L0/L1/L2 verdict'), 'status': 'pending'},
      ];
    }

    if (currentExecutionMode.value == AgentExecutionMode.architect) {
      return [
        {'title': DesktopI18n.tr('Architect: Системный анализ и границы модулей', 'Architect: System analysis & module boundaries'), 'status': 'running'},
        {'title': DesktopI18n.tr('Architect: Проектирование контрактов и схем', 'Architect: Contracts & schema design'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Architect: Оценка рисков и компромиссов (Trade-offs)', 'Architect: Risk & trade-off evaluation'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Architect: Формирование архитектурной спецификации', 'Architect: Architectural specification synthesis'), 'status': 'pending'},
      ];
    }

    if (currentExecutionMode.value == AgentExecutionMode.deepCode) {
      return [
        {'title': DesktopI18n.tr('DeepCode: Сканирование кодовой базы и AST', 'DeepCode: Codebase & AST deep scan'), 'status': 'running'},
        {'title': DesktopI18n.tr('DeepCode: Анализ краевых случаев и зависимостей', 'DeepCode: Edge cases & dependency analysis'), 'status': 'pending'},
        {'title': DesktopI18n.tr('DeepCode: Комплексная реализация решения', 'DeepCode: End-to-end implementation'), 'status': 'pending'},
        {'title': DesktopI18n.tr('DeepCode: Верификация компиляции и типов', 'DeepCode: Compilation & type verification'), 'status': 'pending'},
      ];
    }

    final lower = prompt.toLowerCase();

    if (lower.contains('тест') || lower.contains('test') || lower.contains('spec')) {
      String target = DesktopI18n.tr('модуля', 'module');
      final match = RegExp(r'([\w_]+(?:_service|_controller|_repository|\.dart|\.rs))', caseSensitive: false).firstMatch(prompt);
      if (match != null) {
        target = match.group(1)!;
      }
      return [
        {'title': DesktopI18n.tr('Анализ спецификаций и кода $target', 'Analyzing specifications & code of $target'), 'status': 'running'},
        {'title': DesktopI18n.tr('Проверка Git-ветки и тестового окружения', 'Verifying Git branch & test environment'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Реализация тестовых сценариев и моков', 'Implementing test suites & mock fixtures'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Запуск тестов и валидация результатов', 'Running test suites & validating results'), 'status': 'pending'},
      ];
    }

    if (lower.contains('исправ') || lower.contains('fix') || lower.contains('ошибк') || lower.contains('баг') || lower.contains('bug')) {
      return [
        {'title': DesktopI18n.tr('Локализация проблемы и анализ зависимостей', 'Isolating root cause & analyzing dependencies'), 'status': 'running'},
        {'title': DesktopI18n.tr('Подготовка целевого патча решения', 'Preparing targeted solution patch'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Применение правок в кодовой базе', 'Applying code changes to project codebase'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Проверка компиляции и регрессионный контроль', 'Verifying compilation & regression testing'), 'status': 'pending'},
      ];
    }

    if (lower.contains('созда') || lower.contains('добав') || lower.contains('новый') || lower.contains('feat') || lower.contains('реализ')) {
      return [
        {'title': DesktopI18n.tr('Проектирование структуры и интерфейсов', 'Designing structure & public interfaces'), 'status': 'running'},
        {'title': DesktopI18n.tr('Генерация кода и реализация компонентов', 'Generating code & component implementation'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Интеграция с архитектурой проекта', 'Integrating with existing project architecture'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Верификация целостности артефактов', 'Verifying artifact integrity & build'), 'status': 'pending'},
      ];
    }

    if (lower.contains('рефактор') || lower.contains('оптимиз') || lower.contains('ревью') || lower.contains('review')) {
      return [
        {'title': DesktopI18n.tr('Анализ архитектуры и выявление узких мест', 'Analyzing architecture & identifying bottlenecks'), 'status': 'running'},
        {'title': DesktopI18n.tr('Рефакторинг структур данных и логики', 'Refactoring data structures & business logic'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Проверка сборки и совместимости', 'Checking project build & backwards compatibility'), 'status': 'pending'},
        {'title': DesktopI18n.tr('Формирование итогового отчёта изменений', 'Synthesizing final changelog summary'), 'status': 'pending'},
      ];
    }

    String cleanQuery = prompt.split('\n').first.trim();
    if (cleanQuery.length > 38) {
      cleanQuery = '${cleanQuery.substring(0, 35)}...';
    }
    if (cleanQuery.isEmpty) cleanQuery = DesktopI18n.tr('задачи', 'task');

    return [
      {'title': DesktopI18n.tr('Анализ запроса: $cleanQuery', 'Analyzing request: $cleanQuery'), 'status': 'running'},
      {'title': DesktopI18n.tr('Инспекция контекста и релевантных файлов', 'Inspecting workspace context & relevant files'), 'status': 'pending'},
      {'title': DesktopI18n.tr('Выполнение операций и подготовка решения', 'Executing operations & preparing solution'), 'status': 'pending'},
      {'title': DesktopI18n.tr('Формирование итоговых артефактов', 'Synthesizing output artifacts'), 'status': 'pending'},
    ];
  }

  void bindSessionToProject(String id, String projectName, String projectPath) {
    final s = sessions[id];
    if (s != null) {
      s.project = projectName;
      s.projectPath = projectPath;
      s.hasGitRepo = true;
      if (activeSessionId.value == id) {
        activeProject.value = projectName;
        activeProjectPath.value = projectPath;
        hasGitRepo.value = true;
      }
      sessions.refresh();
      _saveCurrentSessionState();
      _saveLocalCustomSessions();
      update();
    }
  }

  void moveSessionToGroup(String id, String targetGroup) {
    final s = sessions[id];
    if (s != null) {
      s.group = targetGroup;
      if (activeSessionId.value == id) {
        activeGroup.value = targetGroup;
      }
      sessions.refresh();
      _saveCurrentSessionState();
      _saveLocalCustomSessions();
      update();
    }
  }

  /// Connects WebSocket to the active session and handles streaming frames.
  Future<void> _connectWebSocket(String sessionId) async {
    await streamCoordinator.connectWebSocket(sessionId);
  }

  /// Handles incoming streaming frames from Gateway WebSocket.
  /// Handles incoming streaming frames from Gateway WebSocket.
  void _handleGatewayFrame(GatewayFrame frame) {
    if (frame is ConnectedFrame) {
      wsStatus.value = 'connected';
    } else if (frame is ChunkFrame) {
      if (messages.isNotEmpty && messages.last.isStreaming && messages.last.chatMessageType == ChatMessageType.bot) {
        if (messages.last.thinking?.isNotEmpty == true && !messages.last.isThinkingFinished) {
          messages.last.isThinkingFinished = true;
          messages.last.isThinkingExpanded = false; // Auto-collapse!
        }
        messages.last.text += frame.content;
        messages.refresh();
      } else {
        messages.add(ChatMessage(
          text: frame.content,
          chatMessageType: ChatMessageType.bot,
          isStreaming: true,
        ));
      }
      _scrollToBottom();
    } else if (frame is ThinkingFrame) {
      if (messages.isNotEmpty && messages.last.isStreaming && messages.last.chatMessageType == ChatMessageType.bot) {
        messages.last.thinking = (messages.last.thinking ?? '') + frame.content;
        messages.last.isThinkingExpanded = true;
        messages.refresh();
      } else {
        messages.add(ChatMessage(
          text: '',
          chatMessageType: ChatMessageType.bot,
          thinking: frame.content,
          isStreaming: true,
          isThinkingExpanded: true,
        ));
      }
      _scrollToBottom();
    } else if (frame is ToolCallFrame) {
      if (messages.isNotEmpty && messages.last.chatMessageType == ChatMessageType.bot) {
        messages.last.toolCalls.add(ToolCallInfo(
          id: frame.id,
          name: frame.name,
          args: frame.args,
        ));

        final lower = frame.name.toLowerCase();
        AgentActionType actionType;
        String title = frame.name;
        String? details;
        String? command;
        String? filePath;
        int? additions;

        if (lower.contains('terminal') || lower.contains('bash') || lower.contains('command') || lower.contains('run_command')) {
          actionType = AgentActionType.runningTerminal;
          if (frame.args is Map) {
            command = (frame.args['command'] ?? frame.args['CommandLine'] ?? frame.name).toString();
            title = command;
          }
        } else if (lower.contains('grep') || lower.contains('search') || lower.contains('find') || lower.contains('explore')) {
          actionType = AgentActionType.searchingFiles;
          if (frame.args is Map) {
            title = (frame.args['query'] ?? frame.args['Query'] ?? frame.name).toString();
          }
        } else if (lower.contains('view') || lower.contains('read') || lower.contains('cat')) {
          actionType = AgentActionType.readingFile;
          if (frame.args is Map) {
            filePath = (frame.args['path'] ?? frame.args['AbsolutePath'] ?? frame.name).toString();
            title = filePath;
          }
        } else if (lower.contains('write') || lower.contains('edit') || lower.contains('replace') || lower.contains('patch')) {
          actionType = AgentActionType.editingFile;
          if (frame.args is Map) {
            filePath = (frame.args['TargetFile'] ?? frame.args['path'] ?? frame.name).toString();
            title = filePath;
            if (frame.args['ReplacementChunks'] is List) {
              additions = (frame.args['ReplacementChunks'] as List).length * 5;
            }
          }
        } else {
          actionType = AgentActionType.analyzingCode;
        }

        if (frame.args != null) {
          details = frame.args.toString();
        }

        messages.last.steps.add(AgentActionStep(
          title: title,
          type: actionType,
          filePath: filePath,
          command: command,
          additions: additions,
          details: details,
          isRunning: true,
        ));
        messages.refresh();
      }
      final currentRunning = runTimelineSteps.firstWhereOrNull((s) => s['status'] == 'running');
      if (currentRunning != null) {
        currentRunning['status'] = 'completed';
        final nextPending = runTimelineSteps.firstWhereOrNull((s) => s['status'] == 'pending');
        if (nextPending != null) {
          nextPending['status'] = 'running';
        }
      }
      runTimelineSteps.add({
        'id': frame.id,
        'name': frame.name,
        'args': frame.args,
        'status': 'running',
      });
      runTimelineSteps.refresh();
      _scrollToBottom();
    } else if (frame is ToolResultFrame) {
      if (messages.isNotEmpty && messages.last.chatMessageType == ChatMessageType.bot) {
        final existingCall = messages.last.toolCalls.firstWhereOrNull(
          (c) => (frame.id?.isNotEmpty == true && c.id == frame.id) || c.name == frame.name,
        );
        if (existingCall != null) {
          existingCall.output = frame.output;
        }
        AgentActionStep? runningStep;
        for (int i = messages.last.steps.length - 1; i >= 0; i--) {
          if (messages.last.steps[i].isRunning) {
            runningStep = messages.last.steps[i];
            break;
          }
        }
        if (runningStep != null) {
          runningStep.isRunning = false;
          if (frame.output.isNotEmpty) {
            runningStep.details = frame.output;
            final lowerOut = frame.output.toLowerCase();
            if (lowerOut.contains('failed') || lowerOut.contains('error:') || lowerOut.contains('exception:')) {
              runningStep.isError = true;
              runningStep.failureReason = 'Failed';
            }
            // Parse line additions if available in output (e.g. +15 or 15 insertions)
            final addMatch = RegExp(r'\+(\d+)').firstMatch(frame.output);
            if (addMatch != null) {
              runningStep.additions = int.tryParse(addMatch.group(1)!);
            }
          }
        }
        messages.refresh();
      }
      final step = runTimelineSteps.firstWhereOrNull(
        (s) => (frame.id?.isNotEmpty == true && s['id'] == frame.id) || s['name'] == frame.name,
      );
      if (step != null) {
        step['output'] = frame.output;
        step['status'] = 'completed';
        runTimelineSteps.refresh();
      }
      _scrollToBottom();
    } else if (frame is DoneFrame) {
      if (messages.isNotEmpty && messages.last.isStreaming) {
        messages.last.isStreaming = false;
        messages.last.isThinkingFinished = true;
        messages.last.isThinkingExpanded = false; // Auto-collapse!
        if (frame.fullResponse.isNotEmpty) {
          messages.last.text = frame.fullResponse;
        }
        _extractTodoBlocksForMessage(messages.last);
        messages.refresh();
      }
      for (final step in runTimelineSteps) {
        if (step['status'] != 'error' && step['status'] != 'failed') {
          step['status'] = 'completed';
        }
      }
      runTimelineSteps.refresh();
      isRunning.value = false;
      _saveCurrentSessionState();
      _scrollToBottom();
      _checkAndExecuteQueuedPrompt();
    } else if (frame is AbortedFrame) {
      if (messages.isNotEmpty && messages.last.isStreaming) {
        messages.last.isStreaming = false;
        messages.last.isThinkingFinished = true;
        messages.last.isThinkingExpanded = false;
        messages.refresh();
      }
      isRunning.value = false;
    } else if (frame is ApprovalRequestFrame) {
      pendingApprovals.add({
        'request_id': frame.requestId,
        'tool_name': frame.toolName,
        'arguments_summary': frame.argumentsSummary,
        'timeout_secs': frame.timeoutSecs,
      });
      _scrollToBottom();
    } else if (frame is ErrorFrame) {
      if (messages.isNotEmpty && messages.last.isStreaming) {
        messages.last.isStreaming = false;
        messages.last.isError = true;
        messages.last.isThinkingFinished = true;
        messages.last.isThinkingExpanded = false;
        messages.last.text += '\n\nОшибка шлюза: ${frame.message}';
        messages.refresh();
      } else if (isRunning.value) {
        messages.add(ChatMessage(
          text: 'Ошибка шлюза: ${frame.message}',
          chatMessageType: ChatMessageType.bot,
          isError: true,
        ));
      }
      isRunning.value = false;
      wsStatus.value = 'error';
    } else if (frame is SessionMetaFrame) {
      currentSessionMode.value = frame.mode;
      showTodoWidget.value = frame.showTodoWidget;
    } else if (frame is ModeChangedFrame) {
      currentSessionMode.value = frame.newMode;
      showTodoWidget.value = frame.showTodoWidget;
    } else if (frame is RalphProgressFrame) {
      currentSessionMode.value = 'ralph';
      showTodoWidget.value = true;
      runTimelineSteps.add({
        'id': frame.taskId,
        'name': 'Ralph: ${frame.phase}',
        'details': frame.description,
        'completed': frame.completedSteps,
        'total': frame.totalSteps,
        'status': frame.completedSteps >= frame.totalSteps && frame.totalSteps > 0 ? 'completed' : 'running',
      });
      runTimelineSteps.refresh();
      _scrollToBottom();
    } else if (frame is TrajectoryStepFrame) {
      trajectorySteps.add(frame.step);
      trajectorySteps.refresh();
    } else if (frame is ContextBudgetFrame) {
      currentContextBudget.value = frame.budget;
      usedTokens.value = frame.budget.totalUsedTokens;
      maxTokens.value = frame.budget.maxWindowTokens;
    } else if (frame is GroundingCitationFrame) {
      if (messages.isNotEmpty && messages.last.chatMessageType == ChatMessageType.bot) {
        messages.last.citations.add(frame.citation);
        messages.refresh();
      }
    } else if (frame is S1EventFrame) {
      runTimelineSteps.add({
        'id': 's1_${DateTime.now().millisecondsSinceEpoch}',
        'name': 's1: ${frame.status}',
        'details': frame.payload.toString(),
        'status': 'completed',
      });
      runTimelineSteps.refresh();
    }
  }

  void _extractTodoBlocksForMessage(ChatMessage msg) {
    final isTask = showTodoWidget.value ||
        currentSessionMode.value == 'task' ||
        currentSessionMode.value == 'ralph' ||
        currentSessionMode.value == 'admin';
    if (!isTask) {
      msg.todoBlocks.clear();
      return;
    }
    final todos = parseTodoBlocksFromText(msg.text);
    if (todos.isNotEmpty) {
      msg.todoBlocks.clear();
      msg.todoBlocks.addAll(todos);
    }
  }

  static List<TodoBlockData> parseTodoBlocksFromText(String text) {
    final result = <TodoBlockData>[];
    if (text.isEmpty) return result;

    final lines = text.split('\n');
    String? currentTitle;
    int currentDone = 0;
    int currentTotal = 0;
    List<TodoTaskItem>? currentItems;

    void flushCurrent() {
      if (currentItems != null && currentItems!.isNotEmpty) {
        final total = currentTotal > 0 ? currentTotal : currentItems!.length;
        final done = currentDone > 0 ? currentDone : currentItems!.where((i) => i.status == TodoTaskStatus.completed).length;
        result.add(TodoBlockData(
          title: currentTitle ?? 'Todo',
          completedCount: done,
          totalCount: total,
          items: List.from(currentItems!),
          isExpanded: true,
        ));
        currentItems = null;
        currentTitle = null;
        currentDone = 0;
        currentTotal = 0;
      }
    }

    final todoHeaderRegex = RegExp(r'^(?:[^\w\s]*\s*)?(?:Todo|TODO|Задачи|План)[:\s]+(.*?)(?:\s+(\d+)\/(\d+))?$', caseSensitive: false);
    final taskItemRegex = RegExp(r'^\s*(?:[-*]\s*)?(?:\[([ xX>/-])\]|->|\([vV ]\)|•)\s*(.*)$');

    for (final line in lines) {
      final trimmed = line.trim();
      final headerMatch = todoHeaderRegex.firstMatch(trimmed);
      if (headerMatch != null) {
        flushCurrent();
        final rawTitle = headerMatch.group(1)?.trim();
        currentTitle = (rawTitle != null && rawTitle.isNotEmpty) ? rawTitle : 'Todo';
        if (headerMatch.group(2) != null && headerMatch.group(3) != null) {
          currentDone = int.tryParse(headerMatch.group(2)!) ?? 0;
          currentTotal = int.tryParse(headerMatch.group(3)!) ?? 0;
        }
        currentItems = [];
        continue;
      }

      final items = currentItems;
      if (items != null) {
        final itemMatch = taskItemRegex.firstMatch(trimmed);
        if (itemMatch != null) {
          final mark = (itemMatch.group(1) ?? '').toLowerCase();
          final title = itemMatch.group(2)?.trim() ?? '';
          if (title.isNotEmpty) {
            TodoTaskStatus status;
            if (mark == 'x' || trimmed.startsWith('(v)') || trimmed.startsWith('(V)') || trimmed.startsWith('✓')) {
              status = TodoTaskStatus.completed;
            } else if (mark == '>' || mark == '/' || trimmed.startsWith('->')) {
              status = TodoTaskStatus.running;
            } else {
              status = TodoTaskStatus.pending;
            }
            items.add(TodoTaskItem(title: title, status: status));
            continue;
          }
        } else if (trimmed.isEmpty) {
          continue;
        } else {
          flushCurrent();
        }
      }
    }
    flushCurrent();
    return result;
  }

  /// Fetches real sessions from Gateway API and adds to local map.
  Future<void> fetchBackendSessions() async {
    try {
      final list = await httpClient.getSessionsList();
      // Purge any locally present sessions that are in deleted list
      sessions.removeWhere((key, _) => _isSessionDeleted(key));
      if (list.isNotEmpty) {
        backendSessions.assignAll(list);
        for (final s in list) {
          final isDel = _isSessionDeleted(s.sessionId);
          final sessionTitle = (s.name != null && s.name!.trim().isNotEmpty)
              ? s.name!.trim()
              : (s.previewText.trim().isNotEmpty ? s.previewText.trim() : s.sessionId);
          final hasExplicitName = s.name != null && s.name!.trim().isNotEmpty;

          if (!sessions.containsKey(s.sessionId) && !isDel) {
            final matchedProject = s.workspaceDir?.split(RegExp(r'[\\/]')).last;
            sessions[s.sessionId] = TaskSession(
              id: s.sessionId,
              title: sessionTitle,
              tokenCount: '${s.messageCount} сообщ.',
              group: matchedProject ?? 'Разное',
              project: matchedProject,
              projectPath: s.workspaceDir,
              hasGitRepo: s.workspaceDir != null,
              isCustomNamed: hasExplicitName,
            );
          } else if (sessions.containsKey(s.sessionId) && hasExplicitName) {
            final existing = sessions[s.sessionId]!;
            if (!existing.isCustomNamed &&
                (existing.title.startsWith('sess_') ||
                 existing.title.startsWith('sess-') ||
                 existing.title == 'Новая сессия' ||
                 existing.title == existing.id)) {
              existing.title = s.name!.trim();
              existing.isCustomNamed = true;
              if (activeSessionId.value == s.sessionId) {
                activeTaskTitle.value = s.name!.trim();
              }
            }
          }
        }
      }
    } catch (_) {}
  }

  /// Fetches transcript history from backend for a specific session.
  Future<void> fetchSessionMessages(String sessionId) async {
    try {
      final history = await httpClient.getSessionMessages(sessionId);
      if (history.isNotEmpty) {
        final loadedMessages = history.map((m) {
          return ChatMessage(
            text: m.content,
            chatMessageType: m.role == 'user' ? ChatMessageType.user : ChatMessageType.bot,
          );
        }).toList();

        messages.assignAll(loadedMessages);
        final current = sessions[sessionId];
        if (current != null) {
          current.messages = List.from(loadedMessages);
        }
        _scrollToBottom();
      }
      final steps = await httpClient.getSessionSteps(sessionId);
      if (steps.isNotEmpty) {
        trajectorySteps.assignAll(steps);
      }
    } catch (_) {}
  }

  /// Start a clean New Task (Hero Composer screen without pinned branch/project)
  void createNewTask({String? defaultProject}) {
    _saveCurrentSessionState();

    final newId = 'sess-${DateTime.now().millisecondsSinceEpoch}';
    activeSessionId.value = newId;
    activeTaskTitle.value = 'Новая сессия';
    activeProject.value = defaultProject;
    activeBranch.value = null;

    messages.clear();
    attachments.clear();
    runTimelineSteps.clear();
    trajectorySteps.clear();
    currentContextBudget.value = null;
    activeGoal.value = null;
    inputController.clear();

    isNewTask.value = true;
    _connectWebSocket(newId);
  }

  void _saveCurrentSessionState() {
    if (isNewTask.value) return;
    final current = sessions[activeSessionId.value];
    if (current != null) {
      current.messages = List.from(messages);
      current.permissionMode = permissionMode.value;
      current.executionMode = currentExecutionMode.value;
      current.thoughtLevel = thoughtLevel.value;
      current.activeModel = activeModel.value;
      current.activeGoal = activeGoal.value;
      current.runTimelineSteps = List.from(runTimelineSteps);
      current.isTerminalOpen = isTerminalOpen.value;
    }
  }

  void _saveLocalCustomSessions() {
    try {
      final storage = GetStorage();
      final customList = sessions.values
          .where((s) => !_isSessionDeleted(s.id))
          .map((s) {
        return {
          'id': s.id,
          'title': s.title,
          'group': s.group,
          'project': s.project,
          'projectPath': s.projectPath,
          'branch': s.branch,
          'hasGitRepo': s.hasGitRepo,
          'additions': s.additions,
          'deletions': s.deletions,
          'executionMode': s.executionMode.id,
          'activeModel': s.activeModel,
          'tokenCount': s.tokenCount,
          'isCustomNamed': s.isCustomNamed,
        };
      }).toList();
      storage.write('desktop_custom_sessions', customList);
      storage.write('desktop_open_tabs', openSessionTabs.where((tabId) => !_isSessionDeleted(tabId)).toList());
    } catch (_) {}
  }

  void _loadLocalCustomSessions() {
    try {
      final storage = GetStorage();
      final raw = storage.read<List>('desktop_custom_sessions');
      if (raw != null) {
        for (final item in raw) {
          if (item is Map) {
            final id = item['id']?.toString() ?? '';
            if (id.isNotEmpty && !_isSessionDeleted(id)) {
              // Load full message history from session_messages_$id
              final msgRaw = storage.read<List>('session_messages_$id');
              final msgs = <ChatMessage>[];
              if (msgRaw != null && msgRaw.isNotEmpty) {
                for (final m in msgRaw) {
                  if (m is Map) {
                    try {
                      final chatMsg = ChatMessage.fromJson(Map<String, dynamic>.from(m));
                      chatMsg.isStreaming = false;
                      chatMsg.isThinkingFinished = true;
                      chatMsg.isThinkingExpanded = false;
                      for (final s in chatMsg.steps) {
                        s.isRunning = false;
                      }
                      msgs.add(chatMsg);
                    } catch (_) {}
                  }
                }
              }

              sessions[id] = TaskSession(
                id: id,
                title: item['title']?.toString() ?? id,
                group: item['group']?.toString() ?? 'Разное',
                project: item['project']?.toString(),
                projectPath: item['projectPath']?.toString(),
                branch: item['branch']?.toString() ?? 'main',
                hasGitRepo: item['hasGitRepo'] == true,
                additions: item['additions'] as int? ?? 0,
                deletions: item['deletions'] as int? ?? 0,
                executionMode: AgentExecutionMode.fromId(item['executionMode']?.toString() ?? 'fast'),
                activeModel: item['activeModel']?.toString() ?? 'GLM-5.3-Flash',
                tokenCount: item['tokenCount']?.toString() ?? '28.4k tokens',
                isCustomNamed: item['isCustomNamed'] == true,
                messages: msgs,
              );
            }
          }
        }
      }
    } catch (_) {}
  }

  Future<void> renameSession(String id, String newTitle, {bool userInitiated = false}) async {
    final s = sessions[id];
    if (s != null) {
      s.title = newTitle;
      if (userInitiated) {
        s.isCustomNamed = true;
      }
      if (activeSessionId.value == id) {
        activeTaskTitle.value = newTitle;
      }
      _saveCurrentSessionState();
      _saveLocalCustomSessions();
      update();
    }
    await httpClient.sessionRename(id, newTitle);
  }

  Future<void> deleteSession(String id) async {
    _recordDeletedSession(id);
    sessions.remove(id);
    sessions.remove('gw_$id');
    if (id.startsWith('gw_')) {
      sessions.remove(id.substring(3));
    }
    backendSessions.removeWhere((s) =>
        s.sessionId == id ||
        'gw_${s.sessionId}' == id ||
        s.sessionId == 'gw_$id' ||
        (id.startsWith('gw_') && s.sessionId == id.substring(3)));
    openSessionTabs.remove(id);
    openSessionTabs.remove('gw_$id');
    if (id.startsWith('gw_')) {
      openSessionTabs.remove(id.substring(3));
    }
    sessions.refresh();
    openSessionTabs.refresh();
    backendSessions.refresh();
    GetStorage().remove('session_messages_$id');
    if (id.startsWith('gw_')) {
      GetStorage().remove('session_messages_${id.substring(3)}');
    } else {
      GetStorage().remove('session_messages_gw_$id');
    }
    _saveLocalCustomSessions();
    GetStorage().write('desktop_open_tabs', openSessionTabs.toList());
    try {
      await httpClient.deleteSession(id);
    } catch (_) {}
    if (activeSessionId.value == id ||
        activeSessionId.value == 'gw_$id' ||
        (id.startsWith('gw_') && activeSessionId.value == id.substring(3))) {
      final available = sessions.keys.where((k) => !_isSessionDeleted(k)).toList();
      if (available.isNotEmpty) {
        switchToSession(available.first);
      } else {
        createNewTask();
      }
    }
    sessions.refresh();
    openSessionTabs.refresh();
    update();
  }

  void clearSessionHistory(String id) {
    final s = sessions[id];
    if (s != null) {
      s.messages.clear();
      if (activeSessionId.value == id) {
        messages.clear();
      }
      _saveCurrentSessionState();
      _saveLocalCustomSessions();
      update();
    }
  }

  void cyclePermissionMode() {
    permissionMode.value = permissionMode.value.next();
  }

  void setPermissionMode(PermissionMode mode) {
    permissionMode.value = mode;
  }

  void setThoughtLevel(String level) {
    thoughtLevel.value = level;
  }

  final isHandyInstalled = false.obs;

  bool get hasSttModel => isSttConfiguredAndEnabled.value || isHandyInstalled.value;

  Future<void> _loadSttSettings() async {
    try {
      final storage = GetStorage();
      final enabled = storage.read<bool>('stt_enabled') ?? false;
      final verified = storage.read<bool>('stt_verified') ?? false;
      isSttConfiguredAndEnabled.value = enabled && verified;

      // Check Handy installation
      final handyStatus = await httpClient.getHandyStatus();
      if (handyStatus != null) {
        isHandyInstalled.value = handyStatus['installed'] == true;
      }
    } catch (_) {}
  }

  void setSttEnabled(bool enabled) {
    try {
      GetStorage().write('stt_enabled', enabled);
      isSttConfiguredAndEnabled.value = enabled;
    } catch (_) {}
  }

  Future<void> checkSttStatus() async {
    await _loadSttSettings();
  }

  /// Opens native Windows FileDialog to pick any files or images
  Future<void> pickFilesFromDisk() async {
    try {
      const script = r'''
Add-Type -AssemblyName System.Windows.Forms
$dlg = New-Object System.Windows.Forms.OpenFileDialog
$dlg.Multiselect = $true
$dlg.Filter = "Все файлы (*.*)|*.*|Изображения (*.png;*.jpg;*.jpeg;*.webp)|*.png;*.jpg;*.jpeg;*.webp|Код и документы (*.dart;*.rs;*.py;*.md;*.json)|*.dart;*.rs;*.py;*.md;*.json"
$dlg.Title = "Прикрепить файлы или изображения к промту"
if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    $dlg.FileNames | ForEach-Object { Write-Output $_ }
}
''';
      final result = await Process.run('powershell', ['-NoProfile', '-Command', script]);
      if (result.exitCode == 0) {
        final lines = (result.stdout as String).split(RegExp(r'\r?\n')).map((l) => l.trim()).where((l) => l.isNotEmpty);
        for (final f in lines) {
          addAttachment(f);
        }
      }
    } catch (_) {}
  }

  /// Extracts image from Windows Clipboard (e.g. after Win+Shift+S) and attaches as PNG
  Future<bool> pasteScreenshotFromClipboard() async {
    try {
      final userProfile = Platform.environment['USERPROFILE'] ?? '.';
      final tempDir = Directory('$userProfile\\.omnesagent\\attachments');
      if (!tempDir.existsSync()) {
        tempDir.createSync(recursive: true);
      }
      final fileName = 'screenshot_${DateTime.now().millisecondsSinceEpoch}.png';
      final filePath = '${tempDir.path}\\$fileName';
      final script = '''
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
\$img = [System.Windows.Forms.Clipboard]::GetImage()
if (\$img) {
    \$img.Save('$filePath', [System.Drawing.Imaging.ImageFormat]::Png)
    Write-Output "OK"
} else {
    Write-Output "NO_IMAGE"
}
''';
      final result = await Process.run('powershell', ['-NoProfile', '-Command', script]);
      if (result.exitCode == 0 && (result.stdout as String).contains('OK')) {
        addAttachment('$fileName ($filePath)');
        return true;
      }
    } catch (_) {}
    return false;
  }

  /// Builds rich, authoritative OmnesAgent ADE workstation context and tool descriptions
  String _buildOmnesAgentSystemPrompt() {
    final currentSession = sessions[activeSessionId.value];
    final projName = activeProject.value ?? currentSession?.project ?? 'OmnesAgent Monorepo';
    final projPath = currentSession?.projectPath ?? r'C:\Projects\Omnes-agent';
    final branch = activeBranch.value ?? currentSession?.branch ?? 'feat/ade-split';

    final currentMode = permissionMode.value;
    String modeInstructions = '';
    switch (currentMode) {
      case PermissionMode.askBeforeChanges:
        modeInstructions = '''
## CURRENT RUNTIME MODE: ASK BEFORE CHANGES (Запрос подтверждений)
- You MUST ask the user before performing any modifications to files or running state-changing terminal commands.
- Before editing or applying diffs, summarize what you intend to change and ask for user confirmation using an interactive question block.
''';
        break;
      case PermissionMode.editAutomatically:
        modeInstructions = '''
## CURRENT RUNTIME MODE: EDIT AUTOMATICALLY (Авто-правка файлов)
- You have permission to directly inspect, create, and edit code files automatically as needed to solve the task.
- Dangerous or destructive shell commands (e.g. git reset --hard, rm, drop) must still be confirmed with the user.
''';
        break;
      case PermissionMode.planMode:
        modeInstructions = '''
## CURRENT RUNTIME MODE: PLAN MODE (Режим планирования - READ-ONLY)
- You are strictly in planning, inspection, and analysis mode.
- DO NOT modify or write files to disk!
- DO NOT execute commands that alter project state!
- Read and explore the codebase, reason about architecture, and produce a clear, structured implementation plan with step-by-step tasks.
- If decisions or architectural choices are needed, ask the user using an interactive question block.
''';
        break;
      case PermissionMode.fullAccess:
        modeInstructions = '''
## CURRENT RUNTIME MODE: FULL ACCESS (Полный доступ)
- You have full autonomous continuous execution authority to inspect, edit, create files, and run commands without stopping for unnecessary confirmations.
- Solve the user's task end-to-end efficiently and verify results.
''';
        break;
    }

    final execMode = currentExecutionMode.value;
    String execModeInstructions = '';
    switch (execMode) {
      case AgentExecutionMode.ralphLoop:
        execModeInstructions = '''
## ACTIVE AGENT EXECUTION PROFILE: RALPH LOOP (Автономный TDD-цикл разработки)
- YOU ARE OPERATING IN RALPH ORCHESTRATOR MODE (RalphLoop).
- Follow the strict Ralph Loop methodology for every task:
  1. [Ralph Phase 1: Problem Analysis & AST Mapping]: Deeply explore the codebase, check dependencies, and identify the root cause or target feature.
  2. [Ralph Phase 2: Test-First Formulation]: Formulate explicit verification tests, checks, or assertions BEFORE modifying production code.
  3. [Ralph Phase 3: Minimal Diff Implementation]: Apply precise, atomic code edits.
  4. [Ralph Phase 4: Self-Verification & AST Check]: Validate types, syntax, and logic against the target tests.
  5. [Ralph Phase 5: Verdict & Zero-Debt Audit]: Conclude with a clear Ralph Verdict (L0/L1/L2 Pass) and verify zero regressions.
- Explicitly structure your output around these Ralph phases!
''';
        break;
      case AgentExecutionMode.deepCode:
        execModeInstructions = '''
## ACTIVE AGENT EXECUTION PROFILE: DEEP CODE (Глубокий анализ и разработка)
- Perform comprehensive codebase exploration, deep AST navigation, edge-case analysis, and end-to-end implementation with syntax checks.
''';
        break;
      case AgentExecutionMode.architect:
        execModeInstructions = '''
## ACTIVE AGENT EXECUTION PROFILE: ARCHITECT / COT (Системное проектирование)
- Focus on high-level architecture, module boundaries, data structures, trade-offs, and RFC specifications before low-level details.
''';
        break;
      case AgentExecutionMode.fast:
        execModeInstructions = '''
## ACTIVE AGENT EXECUTION PROFILE: SPEED / FAST (Быстрый лаконичный ответ)
- Provide concise, targeted solutions and direct code snippets without unnecessary verbosity.
''';
        break;
    }

    final rulesSection = isProjectRulesEnabled.value && projectRulesContent.value.trim().isNotEmpty
        ? '''

## PROJECT SPECIFIC RULES & CONVENTIONS (AGENTS.md)
The project maintainer has defined specific repository rules and instructions below. You MUST follow them strictly:
\${projectRulesContent.value.trim()}
'''
        : '';

    return '''
You are OmnesAgent, the resident autonomous AI coding assistant running natively inside the OmnesAgent ADE workstation on Windows 11.
You have direct, full, and unrestricted access to the project workspace and ADE tools.

## WORKSPACE & PROJECT CONTEXT
- Project Root: $projPath
- Active Project: $projName
- Active Git Branch: $branch
- Operating System: Windows 11 (paths use backslashes `\\`)
- Architecture: OmnesAgent Monorepo:
  * Rust Backend (`backend/crates/`): 22 modular crates including `omnesagent-gateway` (WebSocket/SSE port 42617), `omnesagent-runtime` (ReAct execution loop), `omnesagent-tools` (system & developer tools), `omnesagent-memory` / `ob2h_runtime` (AST indexing, knowledge graph, Blast Radius), `omnesagent-commands`.
  * Frontend (`frontend/`): Flutter desktop workstation (`frontend/desktop`), web client (`frontend/web`), shared components (`frontend/shared`).
  * Workspace configuration: `~/.omnesagent/config.toml`.

$modeInstructions

$execModeInstructions

## INTERACTIVE USER QUESTIONS (КАРТОЧКА ВОПРОСА В ЧАТЕ)
When you need to ask the user a question, clarify ambiguous requirements, or offer choices, ALWAYS format your question in an interactive block so ADE can render clickable buttons for the user:
```question
{
  "question": "Короткий и понятный вопрос пользователю?",
  "options": [
    "Вариант ответа 1",
    "Вариант ответа 2",
    "Вариант ответа 3"
  ],
  "allowCustom": true
}
```
Do NOT use plain text numbered lists for interactive choices; use the ```question block above so the user can answer with one click!

## YOUR AGENTIC TOOLS & CAPABILITIES
1. **AST & Code Analysis (ob2h / codegraph)**: Deterministic AST syntax tree parsing, symbol lookup, call-graph exploration, and Blast Radius impact calculation across the codebase.
2. **File System Operations**: Direct inspection, reading, editing, and creation of files (`read_file`, `edit_file`, `write_file`, `glob_search`, `content_search`).
3. **Terminal & Execution Sandbox**: Running console commands (`cargo check`, `flutter analyze`, `git status`, powershell, cmd) directly from ADE.
4. **Git Engine**: Inspected status, diffs, branches, and commit tracking.
5. **Canvas & Artifacts**: Generating code files, markdown documents, and UI designs rendered live in ADE Canvas.
6. **Browser Inspector**: Edge WebView2 DOM inspector and visual regression inspection.
$rulesSection

## CRITICAL BEHAVIORAL RULES
1. **NEVER SAY YOU LACK ACCESS**: You are NOT a web chatbot. You are running with full workstation authority inside OmnesAgent ADE. NEVER tell the user "I don't have access to the file system or tools" or ask the user to paste repository files.
2. **CONCRETE, ROOT-CAUSE ANALYSIS**: When the user asks about issues (e.g. «Изменения» indicator, TaskBar checklist, gateway errors, or UI bugs), immediately perform real analysis using the project structure and facts. Name exact files, functions, lines of code, and provide working diffs or terminal commands.
3. **ACTIONABLE RESPONSES**: Format generated files with clear paths (e.g. `// file: lib/features/...` or fenced code blocks with language) so ADE turns them into interactive Canvas Artifacts with 1-click execution.
4. **LANGUAGE**: Reason internally in English; respond to the user in Russian (unless requested otherwise).
'''.trim();
  }

  /// Submits user's answer from an interactive QuestionCard directly into chat
  void submitQuestionAnswer(ChatMessage msg, String answer) {
    msg.selectedQuestionAnswer = answer;
    messages.refresh();
    _saveCurrentSessionState();
    inputController.text = answer;
    sendMessage();
  }

  /// Executes a 1-click follow up action from an action chip
  void executeFollowUpAction(String actionText) {
    inputController.text = actionText;
    sendMessage();
  }

  /// Sends stdin input to the active terminal process or executes command
  void sendTerminalStdin(String input) => terminalController.sendTerminalStdin(input);

  /// Creates a real Git commit/stash checkpoint for a message
  Future<String?> createCheckpointForMessage(ChatMessage msg) async {
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath == null || !universal_io.Directory(projPath).existsSync()) return null;

      // 1. Try to create a real git stash snapshot of current working tree without moving HEAD
      final stashRes = await universal_io.Process.run('git', ['stash', 'create'], workingDirectory: projPath);
      if (stashRes.exitCode == 0 && (stashRes.stdout as String).trim().isNotEmpty) {
        final hash = (stashRes.stdout as String).trim();
        msg.checkpointHash = hash;
        _saveSessionMessages(activeSessionId.value);
        return hash;
      }

      // 2. If working directory is clean or stash create returned empty, snapshot current HEAD
      final res = await universal_io.Process.run('git', ['rev-parse', 'HEAD'], workingDirectory: projPath);
      if (res.exitCode == 0) {
        final hash = (res.stdout as String).trim();
        msg.checkpointHash = hash;
        _saveSessionMessages(activeSessionId.value);
        return hash;
      }
    } catch (_) {}
    return null;
  }

  /// Reverts project files to the state of this message checkpoint
  Future<bool> revertToCheckpoint(ChatMessage msg) async {
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath == null || !universal_io.Directory(projPath).existsSync()) return false;

      final hash = msg.checkpointHash;
      if (hash != null && hash.isNotEmpty && !hash.startsWith('ckpt_')) {
        // Rollback working tree to this specific git stash/commit sha
        await universal_io.Process.run('git', ['checkout', hash, '--', '.'], workingDirectory: projPath);
      } else {
        // Rollback uncommitted changes to current HEAD
        await universal_io.Process.run('git', ['restore', '--staged', '--worktree', '.'], workingDirectory: projPath);
        await universal_io.Process.run('git', ['checkout', '--', '.'], workingDirectory: projPath);
      }

      // Remove any newly created untracked files
      await universal_io.Process.run('git', ['clean', '-fd'], workingDirectory: projPath);
      await refreshGitStatus();

      Get.snackbar(
        'Откат выполнен',
        'Проект возвращен к состоянию контрольной точки (${hash != null && hash.length > 7 ? hash.substring(0, 7) : (hash ?? "HEAD")})',
        snackPosition: SnackPosition.BOTTOM,
        duration: const Duration(seconds: 3),
      );
      return true;
    } catch (e) {
      Get.snackbar('Ошибка отката', '$e', snackPosition: SnackPosition.BOTTOM);
      return false;
    }
  }

  /// Accepts a single file (stages it in Git with `git add`)
  Future<void> acceptFile(String filePath) async {
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath == null || !universal_io.Directory(projPath).existsSync()) return;
      await universal_io.Process.run('git', ['add', '--', filePath], workingDirectory: projPath);
      await refreshGitStatus();
      Get.snackbar(
        'Файл принят',
        'Изменения $filePath зафиксированы в индексе Git (staged)',
        snackPosition: SnackPosition.BOTTOM,
        duration: const Duration(seconds: 2),
      );
    } catch (_) {}
  }

  /// Rejects / discards changes for a single file
  Future<void> rejectFile(String filePath) async {
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath == null || !universal_io.Directory(projPath).existsSync()) return;

      final checkRes = await universal_io.Process.run('git', ['status', '--porcelain', '--', filePath], workingDirectory: projPath);
      final out = (checkRes.stdout as String).trim();

      if (out.startsWith('??')) {
        final fullPath = projPath.endsWith(r'\') || projPath.endsWith('/')
            ? '$projPath$filePath'
            : '$projPath${universal_io.Platform.isWindows ? r'\' : '/'}$filePath';
        final file = universal_io.File(fullPath);
        if (file.existsSync()) {
          file.deleteSync();
        }
      } else {
        await universal_io.Process.run('git', ['restore', '--staged', '--worktree', '--', filePath], workingDirectory: projPath);
        await universal_io.Process.run('git', ['checkout', '--', filePath], workingDirectory: projPath);
      }
      await refreshGitStatus();
      Get.snackbar(
        'Изменения отклонены',
        'Файл $filePath возвращен к исходному состоянию',
        snackPosition: SnackPosition.BOTTOM,
        duration: const Duration(seconds: 2),
      );
    } catch (_) {}
  }

  /// Accepts all currently modified files
  Future<void> acceptAllFiles() async {
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath == null || !universal_io.Directory(projPath).existsSync()) return;
      await universal_io.Process.run('git', ['add', '-A'], workingDirectory: projPath);
      await refreshGitStatus();
      Get.snackbar(
        'Все изменения приняты',
        'Все файлы зафиксированы в индексе Git (staged)',
        snackPosition: SnackPosition.BOTTOM,
        duration: const Duration(seconds: 2),
      );
    } catch (_) {}
  }

  /// Rejects/discards all modified files
  Future<void> rejectAllFiles() async {
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath == null || !universal_io.Directory(projPath).existsSync()) return;
      await universal_io.Process.run('git', ['restore', '--staged', '--worktree', '.'], workingDirectory: projPath);
      await universal_io.Process.run('git', ['checkout', '--', '.'], workingDirectory: projPath);
      await universal_io.Process.run('git', ['clean', '-fd'], workingDirectory: projPath);
      await refreshGitStatus();
      Get.snackbar(
        'Все изменения отклонены',
        'Рабочая директория очищена и возвращена к состоянию HEAD',
        snackPosition: SnackPosition.BOTTOM,
        duration: const Duration(seconds: 2),
      );
    } catch (_) {}
  }

  /// Updates context window usage and estimated token cost
  void _updateTokenUsageStats() {
    int totalChars = 0;
    for (final m in messages) {
      totalChars += m.text.length;
      if (m.thinking != null) totalChars += m.thinking!.length ~/ 2;
    }
    final tokens = (totalChars / 3.6).round();
    usedTokens.value = tokens;
    tokenCount.value = '${(tokens / 1000).toStringAsFixed(1)}k tokens';

    final m = activeModel.value.toLowerCase();
    if (m.contains('claude') || m.contains('sonnet')) {
      maxTokens.value = 200000;
    } else if (m.contains('gemini')) {
      maxTokens.value = 1000000;
    } else {
      maxTokens.value = 128000;
    }
    estimatedCost.value = (tokens / 1000.0) * 0.003;
  }

  /// Compacts context by replacing older conversation turns with an executive summary
  void compactContext() {
    if (messages.length <= 2) {
      Get.snackbar(
        'Контекст минимален',
        'Недостаточно сообщений для сжатия (минимум 3 сообщения)',
        snackPosition: SnackPosition.BOTTOM,
        duration: const Duration(seconds: 2),
      );
      return;
    }
    final lastTwo = messages.sublist(messages.length - 2);
    final countCompacted = messages.length - 2;

    final compactedGoals = <String>[];
    for (final m in messages.take(countCompacted)) {
      if (m.chatMessageType == ChatMessageType.user) {
        final line = m.text.trim().split('\n').first;
        if (line.isNotEmpty && !compactedGoals.contains(line)) {
          compactedGoals.add(line.length > 80 ? '${line.substring(0, 80)}...' : line);
        }
      }
    }
    final goalsText = compactedGoals.isNotEmpty
        ? compactedGoals.map((g) => '• $g').join('\n')
        : '• Выполнение текущей инженерной задачи';

    final summaryMessage = ChatMessage(
      text: '''**[Контекст сжат и сохранен в памяти]**
Сжато предыдущих реплик диалога: **$countCompacted**
Ключевые цели сессии:
$goalsText

*Контекстное окно оптимизировано. Агент помнит ключевые решения и продолжает диалог с сохраненной историей.*''',
      chatMessageType: ChatMessageType.bot,
    );

    messages.assignAll([summaryMessage, ...lastTwo]);
    _updateTokenUsageStats();
    _saveSessionMessages(activeSessionId.value);
    _saveCurrentSessionState();

    Get.snackbar(
      'Контекст оптимизирован',
      'Сжато $countCompacted реплик, освобождено контекстное окно',
      snackPosition: SnackPosition.BOTTOM,
      duration: const Duration(seconds: 2),
    );
  }

  /// Loads project rules from AGENTS.md
  Future<void> loadProjectRules() async {
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath == null) return;
      final file = universal_io.File('$projPath${universal_io.Platform.isWindows ? r'\' : '/'}AGENTS.md');
      if (file.existsSync()) {
        projectRulesContent.value = await file.readAsString();
      } else {
        projectRulesContent.value = '# AGENTS.md — OmnesAgent\n\nПравила проекта загружаются из корневого файла AGENTS.md.';
      }
    } catch (_) {}
  }

  /// Saves updated project rules to AGENTS.md
  Future<void> saveProjectRules(String content) async {
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath == null) return;
      final file = universal_io.File('$projPath${universal_io.Platform.isWindows ? r'\' : '/'}AGENTS.md');
      await file.writeAsString(content);
      projectRulesContent.value = content;
      Get.snackbar(
        'Правила сохранены',
        'Файл AGENTS.md успешно обновлен',
        snackPosition: SnackPosition.BOTTOM,
        duration: const Duration(seconds: 2),
      );
    } catch (_) {}
  }

  /// Runs real Git commands in active project to extract live changed files and diff stats
  /// Switches current git branch using `git checkout <branch>`
  Future<bool> switchGitBranch(String branchName) async {
    try {
      final currentSession = sessions[activeSessionId.value];
      final projPath = currentSession?.projectPath ?? activeProjectPath.value ?? resolveInitialProjectPath();
      final res = await Process.run('git', ['checkout', branchName], workingDirectory: projPath);
      if (res.exitCode == 0) {
        activeBranch.value = branchName;
        if (currentSession != null) {
          currentSession.branch = branchName;
        }
        await refreshGitStatus();
        return true;
      }
    } catch (_) {}
    return false;
  }

  /// Creates and switches to a new git branch using `git checkout -b <branch>`
  Future<bool> createNewGitBranch(String branchName) async {
    try {
      final currentSession = sessions[activeSessionId.value];
      final projPath = currentSession?.projectPath ?? activeProjectPath.value ?? resolveInitialProjectPath();
      final res = await Process.run('git', ['checkout', '-b', branchName], workingDirectory: projPath);
      if (res.exitCode == 0) {
        activeBranch.value = branchName;
        if (currentSession != null) {
          currentSession.branch = branchName;
        }
        await refreshGitStatus();
        return true;
      }
    } catch (_) {}
    return false;
  }

  Future<void> refreshGitStatus() async {
    try {
      final currentSession = sessions[activeSessionId.value];
      final projPath = currentSession?.projectPath ?? activeProjectPath.value ?? resolveInitialProjectPath();
      if (projPath.isEmpty || !Directory(projPath).existsSync()) {
        gitModifiedFiles.clear();
        gitStagedFiles.clear();
        gitAdditions.value = 0;
        gitDeletions.value = 0;
        return;
      }

      // 0. Synchronous Git HEAD & branches resolution (instant 0ms response)
      final syncBranch = readGitHeadBranchSync(projPath);
      if (syncBranch != null && syncBranch.isNotEmpty) {
        activeBranch.value = syncBranch;
        if (currentSession != null) {
          currentSession.branch = syncBranch;
        }
        if (!availableBranches.contains(syncBranch)) {
          availableBranches.add(syncBranch);
        }
      }
      final syncBranches = readGitLocalBranchesSync(projPath);
      if (syncBranches.isNotEmpty) {
        availableBranches.assignAll(syncBranches);
      }

      // 1. Run git status --porcelain to separate staged and unstaged files
      final statusResult = await Process.run('git', ['status', '--porcelain'], workingDirectory: projPath);
      if (statusResult.exitCode == 0) {
        final lines = (statusResult.stdout as String).split('\n').map((l) => l.trim()).where((l) => l.isNotEmpty).toList();
        final files = <String>[];
        final staged = <String>{};
        for (final line in lines) {
          if (line.length > 3) {
            final filePart = line.substring(3).trim();
            files.add(filePart);
            final indexStatus = line[0];
            if (indexStatus != ' ' && indexStatus != '?') {
              staged.add(filePart);
            }
          }
        }
        gitModifiedFiles.assignAll(files);
        gitStagedFiles.assignAll(staged);
      } else {
        gitModifiedFiles.clear();
        gitStagedFiles.clear();
      }

      // 2. Run git diff HEAD --shortstat with fallback to git diff --shortstat
      int ins = 0;
      int del = 0;
      var diffResult = await Process.run('git', ['diff', 'HEAD', '--shortstat'], workingDirectory: projPath);
      if (diffResult.exitCode != 0) {
        diffResult = await Process.run('git', ['diff', '--shortstat'], workingDirectory: projPath);
      }
      if (diffResult.exitCode == 0) {
        final out = (diffResult.stdout as String).trim();
        gitSummary.value = out;
        final insMatch = RegExp(r'(\d+)\s+insertion').firstMatch(out);
        if (insMatch != null) ins = int.tryParse(insMatch.group(1) ?? '0') ?? 0;
        final delMatch = RegExp(r'(\d+)\s+deletion').firstMatch(out);
        if (delMatch != null) del = int.tryParse(delMatch.group(1) ?? '0') ?? 0;
      }
      gitAdditions.value = ins;
      gitDeletions.value = del;
      if (currentSession != null) {
        currentSession.additions = ins;
        currentSession.deletions = del;
      }

      // 3. Query current branch: git branch --show-current
      final branchRes = await Process.run('git', ['branch', '--show-current'], workingDirectory: projPath);
      if (branchRes.exitCode == 0) {
        final curBranch = (branchRes.stdout as String).trim();
        if (curBranch.isNotEmpty) {
          activeBranch.value = curBranch;
          if (currentSession != null) {
            currentSession.branch = curBranch;
          }
        }
      }

      // 4. Query available local branches: git branch --list
      final branchListRes = await Process.run('git', ['branch', '--list'], workingDirectory: projPath);
      if (branchListRes.exitCode == 0) {
        final bList = (branchListRes.stdout as String)
            .split('\n')
            .map((b) => b.replaceAll('*', '').trim())
            .where((b) => b.isNotEmpty)
            .toList();
        if (bList.isNotEmpty) {
          availableBranches.assignAll(bList);
        }
      }
    } catch (_) {}
  }

  /// Saves all messages of a session to persistent storage
  void _saveSessionMessages(String sessionId) {
    try {
      final storage = GetStorage();
      final list = messages.map((m) => m.toJson()).toList();
      storage.write('session_messages_$sessionId', list);
      storage.write('desktop_active_session_id', activeSessionId.value);
    } catch (_) {}
  }

  /// Loads saved messages of a session from persistent storage
  bool _loadSessionMessages(String sessionId) {
    try {
      final storage = GetStorage();
      final raw = storage.read<List>('session_messages_$sessionId');
      if (raw != null && raw.isNotEmpty) {
        final loaded = raw.map((item) {
          final m = ChatMessage.fromJson(Map<String, dynamic>.from(item as Map));
          m.isStreaming = false;
          m.isThinkingFinished = true;
          m.isThinkingExpanded = false;
          for (final s in m.steps) {
            s.isRunning = false;
          }
          return m;
        }).where((m) {
          // Filter out unwanted background socket connection errors that were previously saved
          if (m.isError &&
              (m.text.contains('Connection failed: WebSocketChannelException') ||
               m.text.contains('Удаленный компьютер отклонил это сетевое подключение'))) {
            return false;
          }
          return true;
        }).toList();
        messages.assignAll(loaded);
        final current = sessions[sessionId];
        if (current != null) {
          current.messages = List.from(loaded);
        }
        return true;
      }
    } catch (_) {}
    return false;
  }

  void setModel(String model) {
    activeModel.value = model;
  }

  void addAttachment(String item) {
    if (!attachments.contains(item)) {
      attachments.add(item);
    }
  }

  void removeAttachment(int index) {
    if (index >= 0 && index < attachments.length) {
      attachments.removeAt(index);
    }
  }

  /// Attaches a file or folder path to the prompt.
  /// If inside the active project, creates an `@relativePath` reference.
  /// If outside, attaches the normalized path or quotes if containing spaces.
  /// Adds to [attachments] and inserts reference into [inputController] at cursor position.
  void attachPathToPrompt(String rawPath, {bool insertIntoText = true}) {
    if (rawPath.trim().isEmpty) return;
    final path = rawPath.trim();
    final projectRoot = activeProjectPath.value ?? universal_io.Directory.current.path;

    String refTag;
    try {
      final normPath = p.normalize(path);
      final normRoot = p.normalize(projectRoot);
      if (p.isWithin(normRoot, normPath)) {
        final rel = p.relative(normPath, from: normRoot).replaceAll('\\', '/');
        refTag = '@$rel';
      } else {
        final clean = normPath.replaceAll('\\', '/');
        refTag = clean.contains(' ') ? '"$clean"' : clean;
      }
    } catch (_) {
      refTag = path.replaceAll('\\', '/');
    }

    // 1. Add to attachments chips
    addAttachment(refTag);

    // 2. Insert into inputController at cursor or append
    if (insertIntoText) {
      final currentText = inputController.text;
      final sel = inputController.selection;
      if (sel.isValid && sel.start >= 0 && sel.end >= sel.start) {
        final before = currentText.substring(0, sel.start);
        final after = currentText.substring(sel.end);
        final spacerBefore = (before.isNotEmpty && !before.endsWith(' ') && !before.endsWith('\n')) ? ' ' : '';
        final spacerAfter = (after.isNotEmpty && !after.startsWith(' ')) ? ' ' : ' ';
        final inserted = '$spacerBefore$refTag$spacerAfter';
        inputController.text = '$before$inserted$after';
        final newCursorPos = sel.start + inserted.length;
        inputController.selection = TextSelection.collapsed(offset: newCursorPos);
      } else {
        if (currentText.isEmpty) {
          inputController.text = '$refTag ';
        } else {
          final prefix = currentText.endsWith(' ') || currentText.endsWith('\n') ? '' : ' ';
          inputController.text = '$currentText$prefix$refTag ';
        }
        inputController.selection = TextSelection.collapsed(offset: inputController.text.length);
      }
    }
  }

  void addElementContext({
    required String selector,
    required String text,
    required String tag,
  }) {
    final chip = 'DOM: <$tag> $selector ("$text")';
    addAttachment(chip);
  }

  /// Edits the queued prompt by restoring it to the input field
  void editQueuedPrompt() {
    final prompt = queuedPrompt.value;
    if (prompt == null) return;
    queuedPrompt.value = null;
    inputController.text = prompt;
    inputController.selection = TextSelection.fromPosition(
      TextPosition(offset: prompt.length),
    );
    inputFocusNode.requestFocus();
  }

  /// Cancels the queued prompt
  void cancelQueuedPrompt() {
    queuedPrompt.value = null;
  }

  /// Forces immediate execution of the queued prompt by aborting active run
  void forceSendQueuedPrompt() {
    final prompt = queuedPrompt.value;
    if (prompt == null || prompt.trim().isEmpty) return;
    queuedPrompt.value = null;
    abortRun();
    Future.delayed(const Duration(milliseconds: 150), () {
      sendMessage(prompt);
    });
  }

  /// Auto-executes queued prompt if any after current task finishes
  void _checkAndExecuteQueuedPrompt() {
    final nextPrompt = queuedPrompt.value;
    if (nextPrompt != null && nextPrompt.trim().isNotEmpty) {
      queuedPrompt.value = null;
      Future.microtask(() {
        sendMessage(nextPrompt);
      });
    }
  }

  void sendMessage([String? overrideText]) {
    final rawText = (overrideText ?? inputController.text).trim();
    if (rawText.isEmpty && (overrideText == null ? attachments.isEmpty : false)) return;
    if (rawText.length > 50000) {
      Get.snackbar('Ошибка', 'Сообщение превышает лимит в 50 000 символов');
      return;
    }

    if (isRunning.value) {
      // Put prompt into pending queue instead of sending
      String pendingMessage = rawText;
      if (overrideText == null && attachments.isNotEmpty) {
        final attachList = attachments.map((a) => '- $a').join('\n');
        pendingMessage = '$rawText\n\nКонтекстные вложения:\n$attachList';
      }
      queuedPrompt.value = pendingMessage;
      if (overrideText == null) {
        inputController.clear();
        attachments.clear();
      }
      return;
    }

    final currentSession = sessions[activeSessionId.value];
    final hasUserMessages = messages.any((m) => m.chatMessageType == ChatMessageType.user);
    final title = currentSession?.title.trim() ?? activeTaskTitle.value.trim();

    final isDefaultPattern = title.isEmpty ||
        title == 'Новая сессия' ||
        title == 'New Session' ||
        title == 'Новая задача' ||
        title == 'New Task' ||
        title.startsWith('sess_') ||
        title.startsWith('sess-') ||
        title.startsWith('session-') ||
        title.startsWith('session_') ||
        title.startsWith('gw_') ||
        title.startsWith('Рабочая область') ||
        title == activeSessionId.value ||
        (currentSession != null && title == currentSession.id);

    final shouldAutoRename = isNewTask.value ||
        isDefaultPattern ||
        (!hasUserMessages && currentSession?.isCustomNamed != true);

    if (shouldAutoRename && rawText.isNotEmpty) {
      isNewTask.value = false;
      final autoTitle = _generateTopicTitle(rawText);
      activeTaskTitle.value = autoTitle;

      if (currentSession != null) {
        currentSession.title = autoTitle;
      } else {
        sessions[activeSessionId.value] = TaskSession(
          id: activeSessionId.value,
          title: autoTitle,
          group: activeGroup.value ?? activeProject.value ?? 'Разное',
          project: activeProject.value,
          projectPath: activeProjectPath.value,
          branch: activeBranch.value ?? 'main',
          permissionMode: permissionMode.value,
          thoughtLevel: thoughtLevel.value,
          activeModel: activeModel.value,
          hasGitRepo: hasGitRepo.value,
        );
      }

      if (!openSessionTabs.contains(activeSessionId.value)) {
        openSessionTabs.add(activeSessionId.value);
        GetStorage().write('desktop_open_tabs', openSessionTabs.toList());
      }

      renameSession(activeSessionId.value, autoTitle);
      sessions.refresh();
      openSessionTabs.refresh();
    }

    // Check if input is a /goal command
    if (rawText.startsWith('/goal')) {
      _handleGoalCommand(rawText);
      if (overrideText == null) {
        inputController.clear();
      }
      return;
    }

    // Compose final message with attachments if any
    String fullMessage = rawText;
    if (overrideText == null && attachments.isNotEmpty) {
      final attachList = attachments.map((a) => '- $a').join('\n');
      fullMessage = '$rawText\n\nКонтекстные вложения:\n$attachList';
    }

    final userMsg = ChatMessage(
      text: fullMessage,
      chatMessageType: ChatMessageType.user,
    );
    messages.add(userMsg);
    _saveSessionMessages(activeSessionId.value);
    _scrollToBottom(force: true);

    // Auto-expand TaskBar and populate dynamic contextual steps for this request
    isTaskBarExpanded.value = true;
    runTimelineSteps.assignAll(_generateDynamicStepsForPrompt(rawText));

    if (overrideText == null) {
      inputController.clear();
      attachments.clear();
    }
    isRunning.value = true;

    final storage = GetStorage();
    final provId = activeProvider.value;
    final modelName = activeModel.value;

    String customKey = (storage.read<String>('provider_key_$provId') ?? '').trim();
    String customUrl = (storage.read<String>('provider_url_$provId') ?? '').trim();

    // Check custom providers registry if key or url missing
    final savedCustom = storage.read<List>('custom_providers_registry') ?? [];
    for (final item in savedCustom) {
      if (item is Map && (item['id'] == provId || item['name'] == provId)) {
        if (customKey.isEmpty) customKey = (item['key']?.toString() ?? '').trim();
        if (customUrl.isEmpty) customUrl = (item['url']?.toString() ?? '').trim();
      }
    }

    if (customKey.isEmpty) {
      if (provId == 'openai') {
        customKey = universal_io.Platform.environment['OPENAI_API_KEY'] ?? '';
      } else if (provId == 'deepseek') {
        customKey = universal_io.Platform.environment['DEEPSEEK_API_KEY'] ?? '';
      } else if (provId == 'anthropic') {
        customKey = universal_io.Platform.environment['ANTHROPIC_API_KEY'] ?? '';
      } else if (provId == 'gemini') {
        customKey = universal_io.Platform.environment['GEMINI_API_KEY'] ?? '';
      } else if (provId == 'groq') {
        customKey = universal_io.Platform.environment['GROQ_API_KEY'] ?? '';
      } else if (provId == 'openrouter') {
        customKey = universal_io.Platform.environment['OPENROUTER_API_KEY'] ?? '';
      } else if (provId == 'mistral') {
        customKey = universal_io.Platform.environment['MISTRAL_API_KEY'] ?? '';
      } else if (provId == 'moonshot') {
        customKey = universal_io.Platform.environment['MOONSHOT_API_KEY'] ?? universal_io.Platform.environment['KIMI_API_KEY'] ?? '';
      } else if (provId == 'qwen') {
        customKey = universal_io.Platform.environment['QWEN_API_KEY'] ?? universal_io.Platform.environment['DASHSCOPE_API_KEY'] ?? '';
      }
    }

    if (customUrl.isEmpty) {
      if (provId == 'deepseek') {
        customUrl = universal_io.Platform.environment['DEEPSEEK_BASE_URL'] ?? 'https://api.deepseek.com/v1';
      } else if (provId == 'openai') {
        customUrl = universal_io.Platform.environment['OPENAI_BASE_URL'] ?? 'https://api.openai.com/v1';
      } else if (provId == 'anthropic') {
        customUrl = universal_io.Platform.environment['ANTHROPIC_BASE_URL'] ?? 'https://api.anthropic.com/v1';
      } else if (provId == 'glm') {
        customUrl = 'https://open.bigmodel.cn/api/paas/v4';
      } else if (provId == 'groq') {
        customUrl = 'https://api.groq.com/openai/v1';
      } else if (provId == 'openrouter') {
        customUrl = universal_io.Platform.environment['OPENROUTER_BASE_URL'] ?? 'https://openrouter.ai/api/v1';
      } else if (provId == 'mistral') {
        customUrl = 'https://api.mistral.ai/v1';
      } else if (provId == 'moonshot') {
        customUrl = 'https://api.moonshot.cn/v1';
      } else if (provId == 'qwen') {
        customUrl = universal_io.Platform.environment['QWEN_BASE_URL'] ?? 'https://dashscope.aliyuncs.com/compatible-mode/v1';
      } else if (provId == 'ollama') {
        customUrl = universal_io.Platform.environment['OLLAMA_HOST'] ??
            universal_io.Platform.environment['OLLAMA_BASE_URL'] ??
            'http://localhost:11434/v1';
      } else if (provId == 'lmstudio' || provId == 'local') {
        customUrl = universal_io.Platform.environment['LMSTUDIO_BASE_URL'] ??
            'http://localhost:1234/v1';
      }
    }

    final hasDirectCredentials = customKey.isNotEmpty || provId == 'ollama' || provId == 'lmstudio' || provId == 'local';
    final isCustomProvider = provId.startsWith('custom_') ||
        provId.toLowerCase().contains('daluobo') ||
        !configuredProviders.any((p) => p['id'] == provId && p['isCustom'] == false);

    // If custom provider or gateway is not connected, stream directly via SSE!
    if (hasDirectCredentials && (isCustomProvider || !streamCoordinator.isConnected)) {
      _streamFromProviderDirectly(
        prompt: fullMessage,
        providerId: provId,
        model: modelName,
        apiKey: customKey,
        baseUrl: customUrl,
      );
      _scrollToBottom();
      return;
    }

    // Try Gateway WebSocket if connected
    if (streamCoordinator.isConnected) {
      final initialMessageCount = messages.length;
      streamCoordinator.sendMessage(fullMessage, mode: currentExecutionMode.value.id);

      // Fallback timer: if after 5 seconds the gateway hasn't emitted any answer or thinking, and we have direct credentials, switch to direct stream!
      Future.delayed(const Duration(seconds: 5), () {
        if (isRunning.value && messages.length == initialMessageCount && hasDirectCredentials) {
          _streamFromProviderDirectly(
            prompt: fullMessage,
            providerId: provId,
            model: modelName,
            apiKey: customKey,
            baseUrl: customUrl,
          );
        } else if (isRunning.value && messages.length == initialMessageCount) {
          // Timeout with no credentials
          isRunning.value = false;
          final gwAddress = GatewayConfig.getBaseUrl().replaceAll('http://', '');
          messages.add(ChatMessage(
            text: 'Шлюз OmnesAgent ($gwAddress) не ответил вовремя на запрос.\nПроверьте подключение шлюза или настройте прямой API ключ провайдера в Настройках.',
            chatMessageType: ChatMessageType.bot,
            isError: true,
          ));
          messages.refresh();
          _scrollToBottom();
        }
      });
    } else {
      // Gateway not connected, try connecting
      _connectWebSocket(activeSessionId.value).then((_) {
        final sent = streamCoordinator.sendMessage(fullMessage, mode: currentExecutionMode.value.id);
        if (!sent) {
          if (hasDirectCredentials) {
            _streamFromProviderDirectly(
              prompt: fullMessage,
              providerId: provId,
              model: modelName,
              apiKey: customKey,
              baseUrl: customUrl,
            );
          } else {
            messages.add(ChatMessage(
              text: 'Шлюз OmnesAgent не доступен, а для провайдера "$provId" не указан API ключ.\nОткройте Настройки (шестерёнка внизу слева) и укажите ключ для прямого подключения к LLM.',
              chatMessageType: ChatMessageType.bot,
              isError: true,
            ));
            isRunning.value = false;
            _saveCurrentSessionState();
            _scrollToBottom();
          }
        }
      });
    }

    _scrollToBottom();
  }

  /// Direct SSE streaming from any OpenAI-compatible endpoint (Qwen, Claude, DeepSeek, Daluobo, Ollama, LM Studio).
  Future<void> _streamFromProviderDirectly({
    required String prompt,
    required String providerId,
    required String model,
    required String apiKey,
    required String baseUrl,
  }) async {
    // Take initial snapshot to isolate changes per message
    final initialSnapshotFiles = Set<String>.from(gitModifiedFiles);
    final initialAdditions = gitAdditions.value;
    final initialDeletions = gitDeletions.value;
    String? preExecutionHash;
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath != null && universal_io.Directory(projPath).existsSync()) {
        final res = await universal_io.Process.run('git', ['rev-parse', 'HEAD'], workingDirectory: projPath);
        if (res.exitCode == 0) {
          preExecutionHash = (res.stdout as String).trim();
        }
      }
    } catch (_) {}

    final initialActionStep = AgentActionStep(
      title: DesktopI18n.tr('Инспекция контекста $activeProject и подготовка ответа', 'Inspecting context of $activeProject and preparing response'),
      type: AgentActionType.analyzingCode,
      isRunning: true,
      details: DesktopI18n.tr('Чтение структуры рабочей области ${activeProjectPath.value ?? resolveInitialProjectPath()}', 'Reading workspace structure ${activeProjectPath.value ?? resolveInitialProjectPath()}'),
    );

    final botMessage = ChatMessage(
      text: '',
      chatMessageType: ChatMessageType.bot,
      isStreaming: true,
      thinking: '',
      thinkingSeconds: 0,
      isThinkingFinished: false,
      isThinkingExpanded: true,
      steps: [initialActionStep],
      createdAt: DateTime.now(),
    );
    messages.add(botMessage);
    messages.refresh();
    _scrollToBottom();

    // Populate initial trajectory steps for Direct Chat mode
    final turnId = 'turn_${DateTime.now().millisecondsSinceEpoch}';
    final sessId = activeSessionId.value;
    trajectorySteps.clear();
    trajectorySteps.add(TrajectoryStep(
      id: 'step_${DateTime.now().millisecondsSinceEpoch}_0',
      sessionId: sessId,
      turnId: turnId,
      stepIndex: 0,
      stepType: 's1_gate',
      payloadJson: jsonEncode({
        'decision': 'direct_stream',
        'provider': providerId,
        'model': model,
        'prompt_preview': prompt.length > 100 ? '${prompt.substring(0, 100)}...' : prompt,
      }),
      tokensUsed: ((prompt.length + 3) ~/ 4),
      durationUs: 15000,
      createdAt: DateTime.now().toUtc().toIso8601String(),
    ));
    trajectorySteps.add(TrajectoryStep(
      id: 'step_${DateTime.now().millisecondsSinceEpoch}_1',
      sessionId: sessId,
      turnId: turnId,
      stepIndex: 1,
      stepType: 'context_inject',
      payloadJson: jsonEncode({
        'project': activeProject.value,
        'workspace': activeProjectPath.value ?? resolveInitialProjectPath(),
      }),
      tokensUsed: 64,
      durationUs: 8000,
      createdAt: DateTime.now().toUtc().toIso8601String(),
    ));
    trajectorySteps.refresh();

    final stopwatch = Stopwatch()..start();
    _lastStreamChunkTime = DateTime.now();
    Timer? thinkingTimer;
    thinkingTimer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (botMessage.isStreaming && !botMessage.isThinkingFinished) {
        botMessage.thinkingSeconds = stopwatch.elapsed.inSeconds;
        messages.refresh();
      } else {
        timer.cancel();
      }
    });

    try {
      final sanitizedBase = baseUrl.endsWith('/') ? baseUrl.substring(0, baseUrl.length - 1) : baseUrl;
      final endpoint = sanitizedBase.endsWith('/chat/completions')
          ? sanitizedBase
          : '$sanitizedBase/chat/completions';

      // Assemble conversation context with full OmnesAgent ADE workstation context
      final historyList = <Map<String, dynamic>>[
        {
          'role': 'system',
          'content': _buildOmnesAgentSystemPrompt(),
        }
      ];

      // Add recent messages without polluting with thoughts
      for (final m in messages.take(messages.length - 1)) {
        if (m.isError) continue;
        final cleanText = m.text.replaceAll(RegExp(r'<think>[\s\S]*?</think>'), '').trim();
        if (cleanText.isEmpty) continue;
        historyList.add({
          'role': m.chatMessageType == ChatMessageType.user ? 'user' : 'assistant',
          'content': cleanText,
        });
      }

      final payload = {
        'model': model.isNotEmpty && model != 'Провайдер не настроен' ? model : 'default',
        'messages': historyList,
        'stream': true,
      };

      final client = http.Client();
      _activeDirectClient = client;
      final request = http.Request('POST', Uri.parse(endpoint));
      request.headers.addAll({
        'Content-Type': 'application/json; charset=utf-8',
        'Accept': 'text/event-stream',
        if (apiKey.isNotEmpty) 'Authorization': 'Bearer $apiKey',
      });
      request.body = jsonEncode(payload);

      final streamedResponse = await client.send(request).timeout(const Duration(seconds: 120));

      if (streamedResponse.statusCode != 200) {
        final errBody = await streamedResponse.stream.bytesToString();
        botMessage.isStreaming = false;
        botMessage.isError = true;
        botMessage.isThinkingFinished = true;
        botMessage.isThinkingExpanded = false;
        if (streamedResponse.statusCode == 401) {
          botMessage.text = DesktopI18n.tr(
            'Ошибка авторизации (HTTP 401): Неверный API ключ для провайдера ($providerId).\nПроверьте настройки API ключа в диалоге Настроек.',
            'Authorization Error (HTTP 401): Invalid API key for provider ($providerId).\nCheck API key settings in the Settings dialog.',
          );
        } else if (streamedResponse.statusCode == 404) {
          botMessage.text = DesktopI18n.tr(
            'Ошибка (HTTP 404): Модель "$model" или эндпоинт не найден на сервере провайдера ($providerId).\nBASE_URL: $endpoint\nОтвет: $errBody',
            'Error (HTTP 404): Model "$model" or endpoint not found on provider ($providerId).\nBASE_URL: $endpoint\nResponse: $errBody',
          );
        } else if (streamedResponse.statusCode == 429) {
          botMessage.text = DesktopI18n.tr(
            'Превышен лимит запросов (HTTP 429): Quota exceeded / Rate limit на стороне провайдера ($providerId).\n$errBody',
            'Rate Limit Exceeded (HTTP 429): Quota exceeded / Rate limit from provider ($providerId).\n$errBody',
          );
        } else {
          botMessage.text = DesktopI18n.tr(
            'Ошибка провайдера ($providerId, HTTP ${streamedResponse.statusCode}):\n$errBody',
            'Provider error ($providerId, HTTP ${streamedResponse.statusCode}):\n$errBody',
          );
        }
        isRunning.value = false;
        thinkingTimer.cancel();
        trajectorySteps.add(TrajectoryStep(
          id: 'step_${DateTime.now().millisecondsSinceEpoch}_err',
          sessionId: sessId,
          turnId: turnId,
          stepIndex: trajectorySteps.length,
          stepType: 'synthesis',
          payloadJson: jsonEncode({
            'status': 'error',
            'error': 'HTTP ${streamedResponse.statusCode}',
          }),
          tokensUsed: 0,
          durationUs: stopwatch.elapsedMicroseconds,
          createdAt: DateTime.now().toUtc().toIso8601String(),
        ));
        trajectorySteps.refresh();
        messages.refresh();
        _saveCurrentSessionState();
        _scrollToBottom();
        return;
      }

      bool inThinkTag = false;

      final lineStream = streamedResponse.stream
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .timeout(
            const Duration(seconds: 40),
            onTimeout: (sink) => sink.close(),
          );

      await for (final line in lineStream) {
        _lastStreamChunkTime = DateTime.now();
        final trimmed = line.trim();
        if (!trimmed.startsWith('data:')) continue;
        final data = trimmed.substring(5).trim();
        if (data == '[DONE]') break;

        try {
          final json = jsonDecode(data);
          final choices = json['choices'] as List?;
          if (choices == null || choices.isEmpty) continue;
          final delta = choices[0]['delta'] as Map<String, dynamic>?;
          if (delta == null) continue;

          // Check reasoning_content (DeepSeek R1, Qwen 2.5 Max)
          final reasoning = delta['reasoning_content'] as String?;
          if (reasoning != null && reasoning.isNotEmpty) {
            botMessage.thinking = (botMessage.thinking ?? '') + reasoning;
            botMessage.isThinkingExpanded = true;
            botMessage.isThinkingFinished = false;
            messages.refresh();
            _scrollToBottom();
          }

          // Check content
          final content = delta['content'] as String?;
          if (content != null && content.isNotEmpty) {
            if (content.contains('<think>')) {
              inThinkTag = true;
              final after = content.substring(content.indexOf('<think>') + 7);
              botMessage.thinking = (botMessage.thinking ?? '') + after;
            } else if (inThinkTag) {
              if (content.contains('</think>')) {
                inThinkTag = false;
                botMessage.isThinkingFinished = true;
                botMessage.isThinkingExpanded = false;
                final parts = content.split('</think>');
                botMessage.thinking = (botMessage.thinking ?? '') + parts[0];
                if (parts.length > 1) {
                  botMessage.text += parts.sublist(1).join();
                }
              } else {
                botMessage.thinking = (botMessage.thinking ?? '') + content;
              }
            } else {
              if (!botMessage.isThinkingFinished && (botMessage.thinking?.isNotEmpty ?? false)) {
                botMessage.isThinkingFinished = true;
                botMessage.isThinkingExpanded = false; // Auto-collapse!
              }
              botMessage.text += content;
            }
            messages.refresh();
            _scrollToBottom();
          }
        } catch (_) {}
      }

      botMessage.isStreaming = false;
      botMessage.isThinkingFinished = true;
      botMessage.isThinkingExpanded = false; // Auto-collapse on finish!
      if (botMessage.steps.isNotEmpty) {
        botMessage.steps.first.isRunning = false;
      }
      if (botMessage.thinking != null && botMessage.thinking!.isNotEmpty) {
        trajectorySteps.add(TrajectoryStep(
          id: 'step_${DateTime.now().millisecondsSinceEpoch}_cot',
          sessionId: sessId,
          turnId: turnId,
          stepIndex: trajectorySteps.length,
          stepType: 'cot_thinking',
          payloadJson: jsonEncode({
            'duration_seconds': botMessage.thinkingSeconds,
            'summary': botMessage.thinking!.length > 150
                ? '${botMessage.thinking!.substring(0, 150)}...'
                : botMessage.thinking!,
          }),
          tokensUsed: ((botMessage.thinking!.length + 3) ~/ 4),
          durationUs: botMessage.thinkingSeconds * 1000000,
          createdAt: DateTime.now().toUtc().toIso8601String(),
        ));
      }
      if (botMessage.text.contains('```')) {
        botMessage.steps.add(AgentActionStep(
          title: DesktopI18n.tr('Генерация кода и артефактов решения', 'Generating code & solution artifacts'),
          type: AgentActionType.editingFile,
          isRunning: false,
          details: DesktopI18n.tr('Сформированы исполняемые блоки кода и артефакты', 'Formed executable code blocks & artifacts'),
        ));
        trajectorySteps.add(TrajectoryStep(
          id: 'step_${DateTime.now().millisecondsSinceEpoch}_tool',
          sessionId: sessId,
          turnId: turnId,
          stepIndex: trajectorySteps.length,
          stepType: 'tool_call',
          payloadJson: jsonEncode({
            'action': 'generate_artifacts',
            'blocks_count': RegExp(r'```').allMatches(botMessage.text).length ~/ 2,
          }),
          tokensUsed: 32,
          durationUs: 20000,
          createdAt: DateTime.now().toUtc().toIso8601String(),
        ));
      }
      trajectorySteps.add(TrajectoryStep(
        id: 'step_${DateTime.now().millisecondsSinceEpoch}_synth',
        sessionId: sessId,
        turnId: turnId,
        stepIndex: trajectorySteps.length,
        stepType: 'synthesis',
        payloadJson: jsonEncode({
          'status': 'completed',
          'response_length': botMessage.text.length,
        }),
        tokensUsed: ((botMessage.text.length + 3) ~/ 4),
        durationUs: stopwatch.elapsedMicroseconds,
        createdAt: DateTime.now().toUtc().toIso8601String(),
      ));
      trajectorySteps.refresh();
      await _updateMessageFileChanges(
        botMessage,
        initialFiles: initialSnapshotFiles.toList(),
        initialAdditions: initialAdditions,
        initialDeletions: initialDeletions,
        preExecutionHash: preExecutionHash,
      );
      if (botMessage.text.trim().isEmpty && (botMessage.thinking?.trim().isNotEmpty ?? false)) {
        botMessage.text = botMessage.thinking!.trim();
      }
      for (final s in runTimelineSteps) {
        s['status'] = 'completed';
      }
      runTimelineSteps.refresh();
      _updateTokenUsageStats();
      isRunning.value = false;
      thinkingTimer.cancel();
      _saveCurrentSessionState();
      _saveSessionMessages(activeSessionId.value);
      messages.refresh();
      _scrollToBottom();
      _checkAndExecuteQueuedPrompt();
    } catch (e) {
      thinkingTimer.cancel();
      botMessage.isStreaming = false;
      botMessage.isError = true;
      botMessage.isThinkingFinished = true;
      botMessage.isThinkingExpanded = false;
      if (e is TimeoutException || e.toString().contains('TimeoutException')) {
        botMessage.text = DesktopI18n.tr(
          'Превышен таймаут ответа от модели ($providerId / $model, лимит 120 секунд).\n'
              'Сервер провайдера перегружен или модель генерирует очень длинную цепочку рассуждений (CoT).\n'
              'Попробуйте повторить запрос кнопкой "Повторить" ниже или переключитесь на другую модель.',
          'Model response timed out ($providerId / $model, 120s limit).\n'
              'Provider server is overloaded or model is generating an extended chain of thought (CoT).\n'
              'Please retry with the "Retry" button below or switch to another model.',
        );
      } else {
        botMessage.text = DesktopI18n.tr(
          'Ошибка подключения к провайдеру ($providerId / $model):\n$e\n\nПроверьте настройки API ключа и сетевое подключение.',
          'Connection error with provider ($providerId / $model):\n$e\n\nPlease check API key configuration and network connectivity.',
        );
      }
      isRunning.value = false;
      _saveCurrentSessionState();
      messages.refresh();
      _scrollToBottom();
    } finally {
      _activeDirectClient = null;
      thinkingTimer.cancel();
      isRunning.value = false;
      botMessage.isStreaming = false;
      botMessage.isThinkingFinished = true;
      botMessage.isThinkingExpanded = false;
      for (final s in botMessage.steps) {
        s.isRunning = false;
      }
      messages.refresh();
    }
  }

  /// Analyzes git numstat diff and updates message with changed files details
  Future<void> _updateMessageFileChanges(
    ChatMessage botMessage, {
    List<String>? initialFiles,
    int initialAdditions = 0,
    int initialDeletions = 0,
    String? preExecutionHash,
  }) async {
    try {
      await refreshGitStatus();
      final currentSession = sessions[activeSessionId.value];
      final projPath = currentSession?.projectPath ?? activeProjectPath.value ?? resolveInitialProjectPath();
      if (projPath.isEmpty || !Directory(projPath).existsSync()) return;

      final numstatRes = await Process.run('git', ['diff', 'HEAD', '--numstat'], workingDirectory: projPath);
      final changedFilesList = <Map<String, dynamic>>[];
      if (numstatRes.exitCode == 0) {
        final lines = (numstatRes.stdout as String).split('\n').map((l) => l.trim()).where((l) => l.isNotEmpty);
        for (final l in lines) {
          final parts = l.split(RegExp(r'\s+'));
          if (parts.length >= 3) {
            final ins = int.tryParse(parts[0]) ?? 0;
            final del = int.tryParse(parts[1]) ?? 0;
            final relPath = parts.sublist(2).join(' ').replaceAll('/', '\\');
            final name = relPath.contains('\\') ? relPath.split('\\').last : relPath;
            final dir = relPath.length > name.length ? relPath.substring(0, relPath.length - name.length) : '';
            changedFilesList.add({
              'path': relPath,
              'name': name,
              'dir': dir,
              'additions': ins,
              'deletions': del,
            });
          }
        }
      }

      final taskModifiedFiles = gitModifiedFiles.where((f) => initialFiles == null || !initialFiles.contains(f)).toList();
      final taskAdditions = (gitAdditions.value - initialAdditions).clamp(0, 999999).toInt();
      final taskDeletions = (gitDeletions.value - initialDeletions).clamp(0, 999999).toInt();

      if (changedFilesList.isNotEmpty || taskModifiedFiles.isNotEmpty || (taskAdditions > 0 && (initialFiles?.length ?? 0) != gitModifiedFiles.length)) {
        final totalIns = changedFilesList.isNotEmpty
            ? changedFilesList.fold<int>(0, (sum, f) => sum + (f['additions'] as int? ?? 0))
            : (taskAdditions > 0 ? taskAdditions : 1);
        final totalDel = changedFilesList.isNotEmpty
            ? changedFilesList.fold<int>(0, (sum, f) => sum + (f['deletions'] as int? ?? 0))
            : taskDeletions;

        botMessage.metadata = {
          ...?botMessage.metadata,
          'changed_files': changedFilesList,
          'is_changed_files_expanded': true,
        };
        botMessage.filesChangedCount = changedFilesList.isNotEmpty ? changedFilesList.length : taskModifiedFiles.length;
        botMessage.additions = totalIns;
        botMessage.deletions = totalDel;
        botMessage.checkpointHash = preExecutionHash ?? 'ckpt_${DateTime.now().millisecondsSinceEpoch}';
        botMessage.suggestedActions = [
          'Запустить тесты проекта',
          'Закоммитить изменения в Git',
          'Объяснить архитектуру решения',
        ];
      } else {
        botMessage.filesChangedCount = null;
        botMessage.additions = null;
        botMessage.deletions = null;
      }
    } catch (_) {}
  }

  /// Retries the last user message after an error.
  void retryLastMessage() {
    if (messages.isEmpty) return;
    String? lastUserText;
    for (int i = messages.length - 1; i >= 0; i--) {
      if (messages[i].chatMessageType == ChatMessageType.user) {
        lastUserText = messages[i].text;
        break;
      }
    }
    while (messages.isNotEmpty && (messages.last.isError || messages.last.chatMessageType == ChatMessageType.bot)) {
      messages.removeLast();
    }
    if (lastUserText != null && lastUserText.isNotEmpty) {
      if (messages.isNotEmpty && messages.last.chatMessageType == ChatMessageType.user) {
        messages.removeLast();
      }
      inputController.text = lastUserText;
      sendMessage();
    }
  }

  /// Rolls back project files and truncates dialog messages up to [targetMsg].
  Future<void> rollbackToMessage(ChatMessage targetMsg, {bool putTextInInput = true}) async {
    final idx = messages.indexOf(targetMsg);
    if (idx < 0) return;

    // 1. Revert git changes if repository exists
    try {
      final projPath = sessions[activeSessionId.value]?.projectPath ?? activeProjectPath.value;
      if (projPath != null && universal_io.Directory(projPath).existsSync()) {
        // Discard uncommitted changes created after this point
        await universal_io.Process.run('git', ['restore', '--staged', '--worktree', '.'], workingDirectory: projPath);
        await universal_io.Process.run('git', ['checkout', '--', '.'], workingDirectory: projPath);
        await universal_io.Process.run('git', ['clean', '-fd'], workingDirectory: projPath);

        // If target message or previous message had a checkpoint hash, restore to it
        if (targetMsg.checkpointHash != null && !targetMsg.checkpointHash!.startsWith('ckpt_')) {
          await universal_io.Process.run('git', ['checkout', targetMsg.checkpointHash!, '--', '.'], workingDirectory: projPath);
        }
        await refreshGitStatus();
      }
    } catch (_) {}

    // 2. Truncate messages list: keep up to targetMsg (or remove subsequent)
    final savedText = targetMsg.text;
    final isUser = targetMsg.chatMessageType == ChatMessageType.user;

    // Remove all messages after targetMsg
    while (messages.length > idx + 1) {
      messages.removeLast();
    }

    if (putTextInInput && isUser) {
      // Also remove targetMsg from chat and put in input for editing and resending
      messages.removeLast();
      inputController.text = savedText;
      inputFocusNode.requestFocus();
    }

    messages.refresh();
    _saveCurrentSessionState();
    _saveSessionMessages(activeSessionId.value);

    Get.snackbar(
      'Откат выполнен',
      isUser && putTextInInput
          ? 'Диалог и файлы возвращены к сообщению. Текст помещен в поле ввода.'
          : 'Диалог и файлы успешно возвращены к выбранной точке.',
      snackPosition: SnackPosition.BOTTOM,
      duration: const Duration(seconds: 3),
    );
  }

  /// Repeats a user message (loads it into the input box and optionally sends immediately).
  void repeatMessage(ChatMessage msg, {bool autoSend = false}) {
    inputController.text = msg.text;
    inputFocusNode.requestFocus();
    if (autoSend) {
      sendMessage();
    } else {
      Get.snackbar(
        'Сообщение в поле ввода',
        'Текст сообщения загружен в диалоговое окно для редактирования или повторной отправки.',
        snackPosition: SnackPosition.BOTTOM,
        duration: const Duration(seconds: 2),
      );
    }
  }

  void _handleGoalCommand(String cmd) {
    final parts = cmd.split(' ');
    if (parts.length == 1) {
      messages.add(ChatMessage(
        text: cmd,
        chatMessageType: ChatMessageType.user,
      ));
      final obj = activeGoal.value?.objective ?? 'Цель не установлена';
      messages.add(ChatMessage(
        text: 'Текущая цель: **$obj** (Итерация ${activeGoal.value?.currentIteration ?? 0})',
        chatMessageType: ChatMessageType.bot,
      ));
    } else if (parts[1] == 'pause') {
      activeGoal.value = activeGoal.value?.copyWith(isPaused: true);
      messages.add(ChatMessage(text: cmd, chatMessageType: ChatMessageType.user));
      messages.add(ChatMessage(text: 'Цель приостановлена.', chatMessageType: ChatMessageType.bot));
    } else if (parts[1] == 'resume') {
      activeGoal.value = activeGoal.value?.copyWith(isPaused: false);
      messages.add(ChatMessage(text: cmd, chatMessageType: ChatMessageType.user));
      messages.add(ChatMessage(text: 'Цель возобновлена.', chatMessageType: ChatMessageType.bot));
    } else if (parts[1] == 'clear') {
      activeGoal.value = null;
      messages.add(ChatMessage(text: cmd, chatMessageType: ChatMessageType.user));
      messages.add(ChatMessage(text: 'Цель сброшена.', chatMessageType: ChatMessageType.bot));
    } else {
      final objective = parts.sublist(1).join(' ');
      activeGoal.value = GoalStatus(
        objective: objective,
        currentIteration: 1,
        maxIterations: 10,
        checklist: [
          GoalChecklistItem(id: '1', title: 'Планирование архитектуры', iteration: 1, isCompleted: true),
          GoalChecklistItem(id: '2', title: 'Реализация изменений', iteration: 1, isCompleted: false),
          GoalChecklistItem(id: '3', title: 'Верификация тестами', iteration: 1, isCompleted: false),
        ],
      );
      messages.add(ChatMessage(text: cmd, chatMessageType: ChatMessageType.user));
      messages.add(ChatMessage(
        text: 'Новая цель установлена: **$objective**.\nАгент приступил к циклу планирования и исполнения.',
        chatMessageType: ChatMessageType.bot,
        thinking: 'Thought (Max): Формирую декомпозицию цели $objective на подзадачи.',
      ));
    }
    _scrollToBottom();
  }

  /// Aborts active generation both on WebSocket and via REST endpoint.
  void abortRun() {
    streamCoordinator.abort();
    httpClient.abortSession(activeSessionId.value);
    _activeDirectClient?.close();
    _activeDirectClient = null;
    isRunning.value = false;
    for (final m in messages) {
      if (m.isStreaming) {
        m.isStreaming = false;
        m.isThinkingFinished = true;
        m.isThinkingExpanded = false;
      }
      for (final step in m.steps) {
        step.isRunning = false;
      }
    }
    final curSess = sessions[activeSessionId.value];
    if (curSess != null) {
      for (final m in curSess.messages) {
        if (m.isStreaming) {
          m.isStreaming = false;
          m.isThinkingFinished = true;
          m.isThinkingExpanded = false;
        }
        for (final step in m.steps) {
          step.isRunning = false;
        }
      }
    }
    messages.refresh();
    _saveCurrentSessionState();
    _saveSessionMessages(activeSessionId.value);
  }

  void addDomSelectorChip(String selector) {
    addAttachment('DOM: $selector');
  }

  void approveAction(int index) {
    if (index < pendingApprovals.length) {
      final item = pendingApprovals[index];
      final reqId = item['request_id']?.toString() ?? '';
      if (reqId.isNotEmpty) {
        streamCoordinator.sendApprovalResponse(reqId, 'approve');
      }
      pendingApprovals.removeAt(index);
    }
  }

  void denyAction(int index) {
    if (index < pendingApprovals.length) {
      final item = pendingApprovals[index];
      final reqId = item['request_id']?.toString() ?? '';
      if (reqId.isNotEmpty) {
        streamCoordinator.sendApprovalResponse(reqId, 'deny');
      }
      pendingApprovals.removeAt(index);
    }
  }

  void _onScrollChanged() {
    if (!scrollController.hasClients) return;
    final max = scrollController.position.maxScrollExtent;
    final current = scrollController.position.pixels;
    final distance = max - current;

    // If distance from bottom is notable, user scrolled up
    if (distance > 70) {
      if (!isScrolledUp.value) isScrolledUp.value = true;
      if (distance > 100 && isAutoScrollEnabled.value) {
        isAutoScrollEnabled.value = false;
      }
    } else {
      if (isScrolledUp.value) isScrolledUp.value = false;
      if (!isAutoScrollEnabled.value) isAutoScrollEnabled.value = true;
    }
  }

  void _scrollToBottom({bool force = false}) {
    if (!isAutoScrollEnabled.value && !force) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (scrollController.hasClients) {
        final target = scrollController.position.maxScrollExtent;
        if ((scrollController.position.pixels - target).abs() > 4) {
          scrollController.animateTo(
            target,
            duration: const Duration(milliseconds: 180),
            curve: Curves.easeOutCubic,
          );
        }
      }
    });
  }

  void scrollToBottomDirect() {
    isAutoScrollEnabled.value = true;
    isScrolledUp.value = false;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (scrollController.hasClients) {
        scrollController.animateTo(
          scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 320),
          curve: Curves.easeInOutCubic,
        );
      }
    });
  }

  /// Forks the active session at a specific trajectory step.
  Future<void> forkSessionAtStep(String stepId) async {
    try {
      final res = await httpClient.forkSession(activeSessionId.value, atStepId: stepId);
      final newId = res?['forked_session_id']?.toString() ?? res?['session_id']?.toString();
      if (newId != null && newId.isNotEmpty) {
        switchToSession(newId, title: 'Форк ($stepId)');
        Get.snackbar(
          'Сессия разветвлена',
          'Создана новая сессия: $newId',
          backgroundColor: const Color(0xFF1E293B),
          colorText: Colors.white,
        );
      }
    } catch (e) {
      Get.snackbar(
        'Ошибка форка',
        e.toString(),
        backgroundColor: const Color(0xFF7F1D1D),
        colorText: Colors.white,
      );
    }
  }

  /// Exports current session trajectory trace for replay or debugging.
  Future<Map<String, dynamic>?> exportCurrentTrace() async {
    try {
      final trace = await httpClient.exportSessionTrace(activeSessionId.value);
      final count = trace != null ? ((trace['steps'] as List?)?.length ?? 0) : 0;
      Get.snackbar(
        'Трассировка экспортирована',
        'Собрано $count шагов выполнения',
        backgroundColor: const Color(0xFF1E293B),
        colorText: Colors.white,
      );
      return trace;
    } catch (e) {
      Get.snackbar(
        'Ошибка экспорта трассировки',
        e.toString(),
        backgroundColor: const Color(0xFF7F1D1D),
        colorText: Colors.white,
      );
      return null;
    }
  }

  /// Replays a session trajectory trace.
  Future<void> replayTrace(Map<String, dynamic> trace) async {
    try {
      final res = await httpClient.replaySessionTrace(trace);
      Get.snackbar(
        'Воспроизведение запущено',
        'Результат: ${res?['status'] ?? 'ok'}',
        backgroundColor: const Color(0xFF1E293B),
        colorText: Colors.white,
      );
    } catch (e) {
      Get.snackbar(
        'Ошибка воспроизведения',
        e.toString(),
        backgroundColor: const Color(0xFF7F1D1D),
        colorText: Colors.white,
      );
    }
  }

  @override
  void onClose() {
    WidgetsBinding.instance.removeObserver(this);
    scrollController.removeListener(_onScrollChanged);
    terminalController.dispose();
    streamCoordinator.dispose();
    httpClient.dispose();
    inputController.dispose();
    inputFocusNode.dispose();
    scrollController.dispose();
    super.onClose();
  }
}
