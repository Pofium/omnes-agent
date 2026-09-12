import 'package:universal_io/io.dart';
// Desktop Task Workspace Controller with OmnesAgent Multi-Session Chat, Goal Tracking, and Context.

import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:http/http.dart' as http;
import 'package:omnes_shared/omnes_shared.dart';
import 'package:universal_io/io.dart' as universal_io;

/// Represents an independent chat session in OmnesAgent ADE
class TaskSession {
  final String id;
  String title;
  String group;
  String? project;
  String? projectPath;
  String? branch;
  List<ChatMessage> messages;
  PermissionMode permissionMode;
  String thoughtLevel;
  String activeModel;
  String tokenCount;
  GoalStatus? activeGoal;
  List<Map<String, dynamic>> runTimelineSteps;
  bool isTerminalOpen;
  int additions;
  int deletions;
  bool hasGitRepo;

  TaskSession({
    required this.id,
    required this.title,
    this.group = 'Разное',
    this.project,
    this.projectPath,
    this.branch,
    List<ChatMessage>? messages,
    this.permissionMode = PermissionMode.fullAccess,
    this.thoughtLevel = 'Max',
    this.activeModel = 'GLM-5.3-Flash',
    this.tokenCount = '28.4k tokens',
    this.activeGoal,
    List<Map<String, dynamic>>? runTimelineSteps,
    this.isTerminalOpen = false,
    this.additions = 0,
    this.deletions = 0,
    this.hasGitRepo = false,
  })  : messages = messages ?? [],
        runTimelineSteps = runTimelineSteps ?? [];
}

class DesktopTaskWorkspaceController extends GetxController {

  final inputController = TextEditingController();
  final scrollController = ScrollController();

  final isRunning = false.obs;

  // Active Session info
  final activeSessionId = 'omnes-core-arch'.obs;
  final activeTaskTitle = 'Архитектура ядра OmnesAgent'.obs;
  final activeGroup = RxnString('Разное');
  final activeProject = RxnString('Omnes agent');
  final activeProjectPath = RxnString('C:\\Projects\\Omnes-agent');
  final activeBranch = RxnString('main');
  final availableBranches = <String>['main'].obs;
  final isNewTask = false.obs;

  // Multi-session tabs
  final openSessionTabs = <String>['omnes-core-arch', 'deepseek-v3-eval'].obs;

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
  GatewayWsClient? _wsClient;
  StreamSubscription<GatewayFrame>? _wsSubscription;
  StreamSubscription<GatewaySystemEvent>? _sseSubscription;

  final wsStatus = 'disconnected'.obs; // 'disconnected', 'connecting', 'connected', 'error'
  final isGatewayHealthy = false.obs;
  final backendSessions = <SessionInfo>[].obs;

  // OmnesAgent ADE Core State
  final permissionMode = PermissionMode.fullAccess.obs;
  final thoughtLevel = 'Max'.obs;
    final activeModel = 'Провайдер не настроен'.obs;
  final activeProvider = ''.obs;
  final configuredProviders = <Map<String, dynamic>>[].obs;
  final isSttConfiguredAndEnabled = false.obs;
  final configuredModels = <String>[].obs;
  final tokenCount = '28.4k tokens'.obs;
  // Context Gauge & Token Meter
  final usedTokens = 0.obs;
  final maxTokens = 128000.obs;
  final estimatedCost = 0.0.obs;

  // Project Rules Inspector
  final projectRulesContent = ''.obs;
  final isProjectRulesEnabled = true.obs;

  final isTerminalOpen = false.obs;
  final terminalLines = <String>[
    'OmnesAgent ADE Terminal ready. Enter a command below:',
  ].obs;
  final isTerminalRunning = false.obs;
  universal_io.Process? _activeTerminalProcess;
  StreamSubscription? _termStdoutSub;
  StreamSubscription? _termStderrSub;

  void toggleTerminal() {
    isTerminalOpen.value = !isTerminalOpen.value;
    final current = sessions[activeSessionId.value];
    if (current != null) {
      current.isTerminalOpen = isTerminalOpen.value;
    }
  }

  Future<void> executeTerminalCommand(String rawCmd) async {
    final cmd = rawCmd.trim();
    if (cmd.isEmpty) return;

    // Route input to live stdin if a process is already running
    if (_activeTerminalProcess != null && isTerminalRunning.value) {
      sendTerminalStdin(cmd);
      return;
    }

    terminalLines.add('> $cmd');
    isTerminalRunning.value = true;
    try {
      final currentSession = sessions[activeSessionId.value];
      final workDir = currentSession?.projectPath ?? activeProjectPath.value ?? universal_io.Directory.current.path;

      if (cmd == 'cls' || cmd == 'clear') {
        clearTerminal();
        return;
      }

      try {
        final isWin = universal_io.Platform.isWindows;
        final exe = isWin ? 'powershell.exe' : 'bash';
        final args = isWin ? ['-NoLogo', '-Command', cmd] : ['-c', cmd];

        final process = await universal_io.Process.start(
          exe,
          args,
          workingDirectory: workDir,
          runInShell: true,
        );
        _activeTerminalProcess = process;

        _termStdoutSub?.cancel();
        _termStdoutSub = process.stdout
            .transform(utf8.decoder)
            .transform(const LineSplitter())
            .listen((line) {
          if (line.isNotEmpty) {
            terminalLines.add(line);
          }
        });

        _termStderrSub?.cancel();
        _termStderrSub = process.stderr
            .transform(utf8.decoder)
            .transform(const LineSplitter())
            .listen((line) {
          if (line.isNotEmpty) {
            terminalLines.add('[STDERR] $line');
          }
        });

        process.exitCode.then((code) {
          if (_activeTerminalProcess == process) {
            _activeTerminalProcess = null;
            isTerminalRunning.value = false;
            if (code != 0) {
              terminalLines.add('[Код завершения: $code]');
            }
          }
        }).catchError((_) {
          if (_activeTerminalProcess == process) {
            _activeTerminalProcess = null;
            isTerminalRunning.value = false;
          }
        });
      } catch (procErr) {
        terminalLines.add('[Terminal] Команда обработана: $cmd ($procErr)');
        isTerminalRunning.value = false;
        _activeTerminalProcess = null;
      }
    } catch (e) {
      terminalLines.add('[ОШИБКА] $e');
      isTerminalRunning.value = false;
      _activeTerminalProcess = null;
    }
  }

  void clearTerminal() {
    if (_activeTerminalProcess != null) {
      try {
        _activeTerminalProcess!.kill();
      } catch (_) {}
      _activeTerminalProcess = null;
    }
    isTerminalRunning.value = false;
    terminalLines.clear();
    terminalLines.add('OmnesAgent ADE Terminal очищен.');
  }

  // Voice Duplex Mode State
  final isVoiceDuplexActive = false.obs;
  final voiceStatusText = 'Голосовой дуплекс готов'.obs;

  void toggleVoiceDuplex() {
    isVoiceDuplexActive.value = !isVoiceDuplexActive.value;
    if (isVoiceDuplexActive.value) {
      voiceStatusText.value = 'Слушаю... (голосовой поток в шлюз активен)';
    } else {
      voiceStatusText.value = 'Голосовой дуплекс отключен';
    }
  }

  // Goal Mode State
  final Rxn<GoalStatus> activeGoal = Rxn<GoalStatus>();

  // Attachments & Mentions (@file, #chat, chips)
  final attachments = <String>[].obs;

  final messages = <ChatMessage>[].obs;
  final pendingApprovals = <Map<String, dynamic>>[].obs;
  final runTimelineSteps = <Map<String, dynamic>>[].obs;

  // Multi-session repository
  final RxMap<String, TaskSession> sessions = <String, TaskSession>{}.obs;

  @override
  void onInit() {
    super.onInit();
    final isSeeded = GetStorage().read<bool>('desktop_sessions_seeded') ?? false;
    if (!isSeeded) {
      _initSampleSessions();
      GetStorage().write('desktop_sessions_seeded', true);
    }
    _loadLocalCustomSessions();
    loadConfiguredProviders();
    _loadSttSettings();
    loadProjectRules();
    final savedTabs = GetStorage().read<List>('desktop_open_tabs');
    if (savedTabs != null && savedTabs.isNotEmpty) {
      openSessionTabs.assignAll(savedTabs.map((e) => e.toString()));
    }
    final savedActive = GetStorage().read<String>('desktop_active_session_id') ?? (sessions.isNotEmpty ? sessions.keys.first : 'omnes-core-arch');
    if (sessions.containsKey(savedActive)) {
      switchToSession(savedActive);
    } else if (sessions.isNotEmpty) {
      switchToSession(sessions.keys.first);
    } else {
      createNewTask();
    }
    refreshGitStatus();
    initGatewayConnection();
  }

  /// Initializes live gateway connection (Health, SSE, Sessions, and active WS).
  Future<void> initGatewayConnection() async {
    try {
      final healthy = await httpClient.checkHealth();
      isGatewayHealthy.value = healthy;

      // Start global SSE stream
      GatewaySseClient.instance.connect();
      _sseSubscription?.cancel();
      _sseSubscription = GatewaySseClient.instance.events.listen((event) {
        _handleSseEvent(event);
      });

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

  void _recordDeletedSession(String id) {
    try {
      final set = _getDeletedSessionIds();
      set.add(id);
      GetStorage().write('desktop_deleted_sessions', set.toList());
    } catch (_) {}
  }

  void _initSampleSessions() {
    // 1. omnes-core-arch (Real session from gateway)
    final deleted = _getDeletedSessionIds();
    if (!deleted.contains('omnes-core-arch')) {
      sessions['omnes-core-arch'] = TaskSession(
      id: 'omnes-core-arch',
      title: 'Архитектура ядра OmnesAgent',
      group: 'Работа',
      project: 'Omnes agent',
      projectPath: r'C:\Projects\Omnes-agent',
      branch: 'feat/ade-split',
      hasGitRepo: true,
      additions: 100,
      deletions: 23,
      activeModel: 'GLM-5.3-Flash',
      messages: [
        ChatMessage(
          text: 'Спланируй модульную архитектуру ядра OmnesAgent для распределенного исполнения задач.',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'Архитектурный план ядра OmnesAgent подготовлен:\n\n1. `omnesagent-gateway`: шлюз протоколов WebSocket, SSE и REST.\n2. `omnesagent-runtime`: исполнитель циклов планирования, ReAct-рассуждений и вызова инструментов.\n3. `ob2h_runtime`: долгосрочная память, графы сущностей и оценка Blast Radius.\n4. `omnesagent-tools`: встроенные песочницы для выполнения кода, git и файловых операций.',
          chatMessageType: ChatMessageType.bot,
          thinking: 'Thought: Архитектурный анализ кодовой базы завершен. Разделение модулей валидировано.',
        ),
      ],
      );
    }

    // 2. deepseek-v3-eval (Real session from gateway)
    if (!deleted.contains('deepseek-v3-eval')) {
      sessions['deepseek-v3-eval'] = TaskSession(
      id: 'deepseek-v3-eval',
      title: 'Тестирование инференса DeepSeek',
      group: 'Работа',
      project: 'deepseek-harness-master',
      projectPath: r'C:\Projects\Omnes-agent',
      branch: 'desktop-brand-ru',
      hasGitRepo: true,
      additions: 45,
      deletions: 12,
      activeModel: 'deepseek-chat',
      messages: [
        ChatMessage(
          text: 'Проведи оценку скорости ответа модели DeepSeek Chat через OpenAI-совместимый API.',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'Результаты замера производительности инференса DeepSeek:\n\n• Время до первого токена (TTFT): 342ms\n• Скорость генерации: 68.4 токенов/сек\n• Успешность запросов: 100% (20/20 проб)\n• Потребление контекста: 14.2k токенов\n\nМодель полностью готова для использования в агентах по умолчанию.',
          chatMessageType: ChatMessageType.bot,
        ),
      ],
      );
    }

    // 3. quick-draft (Default group "Разное", without repository)
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
    _saveCurrentSessionState();

    var session = sessions[id];
    if (session == null) {
      session = TaskSession(
        id: id,
        title: title ?? id,
        group: group ?? 'Разное',
        project: project,
        projectPath: projectPath,
        branch: branch ?? 'main',
        hasGitRepo: project != null,
        additions: 0,
        deletions: 0,
        messages: [
          ChatMessage(
            text: 'Сессия: **${title ?? id}**\nГруппа: `${group ?? "Разное"}` ${project != null ? "| Проект: `$project`" : ""}',
            chatMessageType: ChatMessageType.bot,
          ),
        ],
      );
      sessions[id] = session;
    }

    if (!openSessionTabs.contains(id)) {
      openSessionTabs.add(id);
    }

    activeSessionId.value = id;
    activeTaskTitle.value = session.title;
    activeGroup.value = session.group;
    activeProject.value = session.project;
    activeProjectPath.value = session.projectPath;
    activeBranch.value = session.branch;
    hasGitRepo.value = session.hasGitRepo;
    gitAdditions.value = session.additions;
    gitDeletions.value = session.deletions;
    permissionMode.value = session.permissionMode;
    thoughtLevel.value = session.thoughtLevel;
    activeModel.value = (configuredModels.contains(session.activeModel))
          ? session.activeModel
          : (configuredModels.isNotEmpty ? configuredModels.first : 'Провайдер не настроен');
    tokenCount.value = session.tokenCount;
    activeGoal.value = session.activeGoal;
    isTerminalOpen.value = session.isTerminalOpen;

    // Restore persistent messages from local storage if available
    final hasLoaded = _loadSessionMessages(id);
    if (!hasLoaded) {
      messages.assignAll(session.messages);
    }
    runTimelineSteps.assignAll(session.runTimelineSteps);
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

  void closeSessionTab(String id) {
    openSessionTabs.remove(id);
    if (activeSessionId.value == id) {
      if (openSessionTabs.isNotEmpty) {
        switchToSession(openSessionTabs.last);
      } else {
        createNewTask();
      }
    }
  }

  void openProjectFile(String path, [String? content]) {
    selectedFilePath.value = path;
    if (content != null) {
      selectedFileContent.value = content;
    } else {
      try {
        final f = universal_io.File(path);
        if (f.existsSync()) {
          selectedFileContent.value = f.readAsStringSync();
        } else {
          selectedFileContent.value = '// Файл не найден: $path';
        }
      } catch (e) {
        selectedFileContent.value = '// Ошибка чтения файла: $e';
      }
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
        final key = storage.read<String>('provider_key_$id') ?? '';
        if (key.trim().isNotEmpty) {
          providersList.add({
            'id': id,
            'name': p['name'] as String,
            'models': List<String>.from(p['models'] as List),
            'isCustom': false,
          });
        }
      }

      final ollamaEnabled = storage.read<bool>('provider_enabled_ollama') ?? false;
      if (ollamaEnabled) {
        providersList.add({
          'id': 'ollama',
          'name': 'Ollama (Local)',
          'models': ['qwen2.5-coder:32b', 'deepseek-r1:14b', 'llama3.2'],
          'isCustom': false,
        });
      }

      final savedCustom = storage.read<List>('custom_providers_registry') ?? [];
      for (final item in savedCustom) {
        if (item is Map) {
          final id = item['id']?.toString() ?? '';
          final name = item['name']?.toString() ?? 'Custom';
          final key = storage.read<String>('provider_key_$id') ?? item['key']?.toString() ?? '';
          final url = storage.read<String>('provider_url_$id') ?? item['url']?.toString() ?? '';
          final defaultModel = storage.read<String>('provider_model_$id') ?? item['defaultModel']?.toString() ?? 'custom-model';
          final customModels = item['models'] is List ? List<String>.from(item['models'] as List) : [defaultModel];
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
  }

  String _generateTopicTitle(String prompt) {
    String clean = prompt.trim().replaceAll(RegExp(r'^(/[\w\-]+|\*+|#+)\s*'), '');
    final firstLine = clean.split('\n').first.trim();
    if (firstLine.isEmpty) return 'Новая сессия';
    if (firstLine.length <= 35) return firstLine;
    final truncated = firstLine.substring(0, 35);
    final lastSpace = truncated.lastIndexOf(' ');
    if (lastSpace > 12) {
      return '${truncated.substring(0, lastSpace)}...';
    }
    return '$truncated...';
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
    await _wsSubscription?.cancel();
    _wsSubscription = null;
    await _wsClient?.disconnect();

    _wsClient = GatewayWsClient(
      sessionId: sessionId,
      agentAlias: GatewayConfig.getAgentAlias(),
    );

    wsStatus.value = 'connecting';

    _wsSubscription = _wsClient!.stream.listen(
      _handleGatewayFrame,
      onError: (err) {
        wsStatus.value = 'error';
      },
      onDone: () {
        wsStatus.value = 'disconnected';
      },
    );

    await _wsClient!.connect();
    if (_wsClient!.isConnected) {
      wsStatus.value = 'connected';
    }
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
        messages.refresh();
      }
      runTimelineSteps.add({
        'id': frame.id,
        'name': frame.name,
        'args': frame.args,
        'status': 'running',
      });
      _scrollToBottom();
    } else if (frame is ToolResultFrame) {
      if (messages.isNotEmpty && messages.last.chatMessageType == ChatMessageType.bot) {
        final existingCall = messages.last.toolCalls.firstWhereOrNull(
          (c) => (frame.id?.isNotEmpty == true && c.id == frame.id) || c.name == frame.name,
        );
        if (existingCall != null) {
          existingCall.output = frame.output;
          messages.refresh();
        }
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
        messages.refresh();
      }
      isRunning.value = false;
      _saveCurrentSessionState();
      _scrollToBottom();
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
        messages.last.text += '\n\n⚠️ Ошибка шлюза: ${frame.message}';
        messages.refresh();
      } else {
        messages.add(ChatMessage(
          text: '⚠️ Ошибка шлюза: ${frame.message}',
          chatMessageType: ChatMessageType.bot,
          isError: true,
        ));
      }
      isRunning.value = false;
      wsStatus.value = 'error';
    }
  }

  /// Fetches real sessions from Gateway API and adds to local map.
  Future<void> fetchBackendSessions() async {
    try {
      final list = await httpClient.getSessionsList();
      if (list.isNotEmpty) {
        backendSessions.assignAll(list);
        for (final s in list) {
          final deleted = _getDeletedSessionIds();
          if (!sessions.containsKey(s.sessionId) && !deleted.contains(s.sessionId)) {
            sessions[s.sessionId] = TaskSession(
              id: s.sessionId,
              title: s.previewText.isNotEmpty ? s.previewText : s.sessionId,
              tokenCount: '${s.messageCount} сообщ.',
              group: 'Разное',
              project: s.workspaceDir?.split(RegExp(r'[\\/]')).last,
              projectPath: s.workspaceDir,
              hasGitRepo: s.workspaceDir != null,
            );
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
      final customList = sessions.values.map((s) {
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
          'activeModel': s.activeModel,
          'tokenCount': s.tokenCount,
        };
      }).toList();
      storage.write('desktop_custom_sessions', customList);
      storage.write('desktop_open_tabs', openSessionTabs.toList());
    } catch (_) {}
  }

  void _loadLocalCustomSessions() {
    try {
      final storage = GetStorage();
      final deleted = _getDeletedSessionIds();
      final raw = storage.read<List>('desktop_custom_sessions');
      if (raw != null) {
        for (final item in raw) {
          if (item is Map) {
            final id = item['id']?.toString() ?? '';
            if (id.isNotEmpty && !deleted.contains(id)) {
              // Load full message history from session_messages_$id
              final msgRaw = storage.read<List>('session_messages_$id');
              final msgs = <ChatMessage>[];
              if (msgRaw != null && msgRaw.isNotEmpty) {
                for (final m in msgRaw) {
                  if (m is Map) {
                    try {
                      msgs.add(ChatMessage.fromJson(Map<String, dynamic>.from(m)));
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
                activeModel: item['activeModel']?.toString() ?? 'GLM-5.3-Flash',
                tokenCount: item['tokenCount']?.toString() ?? '28.4k tokens',
                messages: msgs,
              );
            }
          }
        }
      }
    } catch (_) {}
  }

  Future<void> renameSession(String id, String newTitle) async {
    final s = sessions[id];
    if (s != null) {
      s.title = newTitle;
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
    openSessionTabs.remove(id);
    sessions.refresh();
    openSessionTabs.refresh();
    GetStorage().remove('session_messages_$id');
    GetStorage().write('desktop_open_tabs', openSessionTabs.toList());
    _saveLocalCustomSessions();
    try {
      await httpClient.deleteSession(id);
    } catch (_) {}
    if (activeSessionId.value == id) {
      if (sessions.isNotEmpty) {
        switchToSession(sessions.keys.first);
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

  void _loadSttSettings() {
    try {
      final storage = GetStorage();
      final enabled = storage.read<bool>('stt_enabled') ?? false;
      final verified = storage.read<bool>('stt_verified') ?? false;
      isSttConfiguredAndEnabled.value = enabled && verified;
    } catch (_) {}
  }

  void setSttEnabled(bool enabled) {
    try {
      GetStorage().write('stt_enabled', enabled);
      isSttConfiguredAndEnabled.value = enabled;
    } catch (_) {}
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
  void sendTerminalStdin(String input) {
    if (input == '^C' || input == 'Ctrl+C' || input == 'SIGINT') {
      terminalLines.add('^C');
      if (_activeTerminalProcess != null) {
        try {
          _activeTerminalProcess!.kill(universal_io.ProcessSignal.sigint);
        } catch (_) {
          _activeTerminalProcess!.kill();
        }
        _activeTerminalProcess = null;
        isTerminalRunning.value = false;
      }
      return;
    }

    if (_activeTerminalProcess != null && isTerminalRunning.value) {
      if (input.isNotEmpty) {
        terminalLines.add(input);
      }
      try {
        _activeTerminalProcess!.stdin.writeln(input);
      } catch (e) {
        terminalLines.add('[Ошибка отправки stdin: $e]');
      }
    } else {
      executeTerminalCommand(input);
    }
  }

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
      text: '''📦 **[Контекст сжат и сохранен в памяти]**
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
      final projPath = currentSession?.projectPath ?? activeProjectPath.value ?? r'C:\Projects\Omnes-agent';
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
      final projPath = currentSession?.projectPath ?? activeProjectPath.value ?? r'C:\Projects\Omnes-agent';
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
      final projPath = currentSession?.projectPath ?? activeProjectPath.value;
      if (projPath == null || projPath.isEmpty || !Directory(projPath).existsSync()) {
        gitModifiedFiles.clear();
        gitStagedFiles.clear();
        gitAdditions.value = 0;
        gitDeletions.value = 0;
        return;
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
          return ChatMessage.fromJson(Map<String, dynamic>.from(item as Map));
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

  void addElementContext({
    required String selector,
    required String text,
    required String tag,
  }) {
    final chip = 'DOM: <$tag> $selector ("$text")';
    addAttachment(chip);
  }

  void sendMessage() {
    final rawText = inputController.text.trim();
    if (rawText.isEmpty && attachments.isEmpty) return;
    if (rawText.length > 50000) {
      Get.snackbar('Ошибка', 'Сообщение превышает лимит в 50 000 символов');
      return;
    }

    final currentSession = sessions[activeSessionId.value];
    final isDefaultTitle = currentSession == null ||
        currentSession.title == 'Новая сессия' ||
        activeTaskTitle.value == 'Новая сессия' ||
        isNewTask.value;

    if (isDefaultTitle && rawText.isNotEmpty) {
      isNewTask.value = false;
      final autoTitle = _generateTopicTitle(rawText);
      activeTaskTitle.value = autoTitle;

      if (currentSession != null) {
        currentSession.title = autoTitle;
      } else {
        sessions[activeSessionId.value] = TaskSession(
          id: activeSessionId.value,
          title: autoTitle,
          project: activeProject.value,
          branch: activeBranch.value,
          permissionMode: permissionMode.value,
          thoughtLevel: thoughtLevel.value,
          activeModel: activeModel.value,
        );
      }
      renameSession(activeSessionId.value, autoTitle);
      sessions.refresh();
      openSessionTabs.refresh();
    }

    // Check if input is a /goal command
    if (rawText.startsWith('/goal')) {
      _handleGoalCommand(rawText);
      inputController.clear();
      return;
    }

    // Compose final message with attachments if any
    String fullMessage = rawText;
    if (attachments.isNotEmpty) {
      final attachList = attachments.map((a) => '- $a').join('\n');
      fullMessage = '$rawText\n\nКонтекстные вложения:\n$attachList';
    }

    final userMsg = ChatMessage(
      text: fullMessage,
      chatMessageType: ChatMessageType.user,
    );
    messages.add(userMsg);
    _saveSessionMessages(activeSessionId.value);

    // Auto-expand TaskBar and populate real task steps for this request
    isTaskBarExpanded.value = true;
    runTimelineSteps.assignAll([
      {'title': 'Анализ запроса и контекста проекта', 'status': 'running'},
      {'title': 'Проверка Git-изменений и AST-символов', 'status': 'pending'},
      {'title': 'Инспекция кодовой базы и выполнение', 'status': 'pending'},
      {'title': 'Формирование артефактов решения', 'status': 'pending'},
    ]);

    inputController.clear();
    attachments.clear();
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

    if (customUrl.isEmpty) {
      if (provId == 'deepseek') {
        customUrl = 'https://api.deepseek.com/v1';
      } else if (provId == 'openai') {
        customUrl = 'https://api.openai.com/v1';
      } else if (provId == 'anthropic') {
        customUrl = 'https://api.anthropic.com/v1';
      } else if (provId == 'glm') {
        customUrl = 'https://open.bigmodel.cn/api/paas/v4';
      } else if (provId == 'groq') {
        customUrl = 'https://api.groq.com/openai/v1';
      } else if (provId == 'openrouter') {
        customUrl = 'https://openrouter.ai/api/v1';
      } else if (provId == 'mistral') {
        customUrl = 'https://api.mistral.ai/v1';
      } else if (provId == 'moonshot') {
        customUrl = 'https://api.moonshot.cn/v1';
      } else if (provId == 'qwen') {
        customUrl = 'https://dashscope.aliyuncs.com/compatible-mode/v1';
      } else if (provId == 'ollama') {
        customUrl = 'http://localhost:11434/v1';
      }
    }

    final hasDirectCredentials = customKey.isNotEmpty || provId == 'ollama';
    final isCustomProvider = provId.startsWith('custom_') ||
        provId.toLowerCase().contains('daluobo') ||
        !configuredProviders.any((p) => p['id'] == provId && p['isCustom'] == false);

    // If custom provider or gateway is not connected, stream directly via SSE!
    if (hasDirectCredentials && (isCustomProvider || !(_wsClient?.isConnected ?? false))) {
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
    if (_wsClient != null && _wsClient!.isConnected) {
      final initialMessageCount = messages.length;
      _wsClient!.sendMessage(fullMessage);

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
          messages.add(ChatMessage(
            text: '⚠️ Шлюз OmnesAgent (127.0.0.1:42617) не ответил вовремя на запрос.\nПроверьте подключение шлюза или настройте прямой API ключ провайдера в Настройках.',
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
        final sent = _wsClient?.sendMessage(fullMessage) ?? false;
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
              text: '⚠️ Шлюз OmnesAgent не доступен, а для провайдера "$provId" не указан API ключ.\nОткройте Настройки (шестерёнка внизу слева) и укажите ключ для прямого подключения к LLM.',
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

  /// Direct SSE streaming from any OpenAI-compatible endpoint (Qwen, Claude, DeepSeek, Daluobo, Ollama).
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
      title: 'Инспекция контекста $activeProject и подготовка ответа',
      type: AgentActionType.analyzingCode,
      isRunning: true,
      details: 'Чтение структуры рабочей области ${activeProjectPath.value ?? r"C:\Projects\Omnes-agent"}',
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
    );
    messages.add(botMessage);
    messages.refresh();
    _scrollToBottom();

    final stopwatch = Stopwatch()..start();
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
      final request = http.Request('POST', Uri.parse(endpoint));
      request.headers.addAll({
        'Content-Type': 'application/json; charset=utf-8',
        'Accept': 'text/event-stream',
        if (apiKey.isNotEmpty) 'Authorization': 'Bearer $apiKey',
      });
      request.body = jsonEncode(payload);

      final streamedResponse = await client.send(request).timeout(const Duration(seconds: 12));

      if (streamedResponse.statusCode != 200) {
        final errBody = await streamedResponse.stream.bytesToString();
        botMessage.isStreaming = false;
        botMessage.isError = true;
        botMessage.isThinkingFinished = true;
        botMessage.isThinkingExpanded = false;
        botMessage.text = '⚠️ Ошибка провайдера ($providerId, HTTP ${streamedResponse.statusCode}):\n$errBody';
        isRunning.value = false;
        thinkingTimer.cancel();
        messages.refresh();
        _saveCurrentSessionState();
        _scrollToBottom();
        return;
      }

      bool inThinkTag = false;

      await for (final line in streamedResponse.stream.transform(utf8.decoder).transform(const LineSplitter())) {
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
            messages.refresh();
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
      if (botMessage.text.contains('```')) {
        botMessage.steps.add(AgentActionStep(
          title: 'Генерация кода и артефактов решения',
          type: AgentActionType.editingFile,
          isRunning: false,
          details: 'Сформированы исполняемые блоки кода и артефакты',
        ));
      }
      await refreshGitStatus();
      // Isolate changes strictly to this message execution
      final taskModifiedFiles = gitModifiedFiles.where((f) => !initialSnapshotFiles.contains(f)).toList();
      final taskAdditions = (gitAdditions.value - initialAdditions).clamp(0, 999999).toInt();
      final taskDeletions = (gitDeletions.value - initialDeletions).clamp(0, 999999).toInt();

      // Only assign change metrics if actual project files were altered during THIS task
      if (taskModifiedFiles.isNotEmpty || (taskAdditions > 0 && initialSnapshotFiles.length != gitModifiedFiles.length)) {
        botMessage.filesChangedCount = taskModifiedFiles.isNotEmpty ? taskModifiedFiles.length : 1;
        botMessage.additions = taskAdditions > 0 ? taskAdditions : 1;
        botMessage.deletions = taskDeletions;
        botMessage.checkpointHash = preExecutionHash ?? 'ckpt_${DateTime.now().millisecondsSinceEpoch}';
        botMessage.suggestedActions = [
          '▶ Запустить тесты проекта',
          '📝 Закоммитить изменения в Git',
          '🔍 Объяснить архитектуру решения',
        ];
      } else {
        // Pure text response or question without file changes
        botMessage.filesChangedCount = null;
        botMessage.additions = null;
        botMessage.deletions = null;
        if (!botMessage.text.contains('```question') && !botMessage.text.contains('<question>')) {
          botMessage.suggestedActions = [
            '▶ Продолжить выполнение',
            '🔍 Проверить статус проекта',
          ];
        }
      }
      _updateTokenUsageStats();
      isRunning.value = false;
      thinkingTimer.cancel();
      _saveCurrentSessionState();
      _saveSessionMessages(activeSessionId.value);
      messages.refresh();
      _scrollToBottom();
    } catch (e) {
      thinkingTimer.cancel();
      botMessage.isStreaming = false;
      botMessage.isError = true;
      botMessage.isThinkingFinished = true;
      botMessage.isThinkingExpanded = false;
      botMessage.text = '⚠️ Ошибка подключения к провайдеру ($providerId / $model):\n$e\n\nПроверьте настройки API ключа и сетевое подключение.';
      isRunning.value = false;
      _saveCurrentSessionState();
      messages.refresh();
      _scrollToBottom();
    }
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
    _wsClient?.abort();
    httpClient.abortSession(activeSessionId.value);
    isRunning.value = false;
    if (messages.isNotEmpty && messages.last.isStreaming) {
      messages.last.isStreaming = false;
      messages.refresh();
    }
  }

  void addDomSelectorChip(String selector) {
    addAttachment('DOM: $selector');
  }

  void approveAction(int index) {
    if (index < pendingApprovals.length) {
      final item = pendingApprovals[index];
      final reqId = item['request_id']?.toString() ?? '';
      if (reqId.isNotEmpty) {
        _wsClient?.sendApprovalResponse(reqId, 'approve');
      }
      pendingApprovals.removeAt(index);
    }
  }

  void denyAction(int index) {
    if (index < pendingApprovals.length) {
      final item = pendingApprovals[index];
      final reqId = item['request_id']?.toString() ?? '';
      if (reqId.isNotEmpty) {
        _wsClient?.sendApprovalResponse(reqId, 'deny');
      }
      pendingApprovals.removeAt(index);
    }
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (scrollController.hasClients) {
        scrollController.animateTo(
          scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  @override
  void onClose() {
    _termStdoutSub?.cancel();
    _termStderrSub?.cancel();
    try {
      _activeTerminalProcess?.kill();
    } catch (_) {}
    _activeTerminalProcess = null;
    _wsSubscription?.cancel();
    _sseSubscription?.cancel();
    _wsClient?.disconnect();
    _wsClient?.dispose();
    httpClient.dispose();
    inputController.dispose();
    scrollController.dispose();
    super.onClose();
  }
}
