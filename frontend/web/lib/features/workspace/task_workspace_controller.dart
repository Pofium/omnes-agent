// Desktop Task Workspace Controller with OmnesAgent Multi-Session Chat, Goal Tracking, and Context.

import 'dart:async';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:omnes_shared/omnes_shared.dart';

/// Represents an independent chat session in OmnesAgent ADE
class TaskSession {
  final String id;
  String title;
  String? project;
  String? branch;
  List<ChatMessage> messages;
  PermissionMode permissionMode;
  String thoughtLevel;
  String activeModel;
  String tokenCount;
  GoalStatus? activeGoal;
  List<Map<String, dynamic>> runTimelineSteps;
  bool isTerminalOpen;

  TaskSession({
    required this.id,
    required this.title,
    this.project,
    this.branch,
    List<ChatMessage>? messages,
    this.permissionMode = PermissionMode.fullAccess,
    this.thoughtLevel = 'Max',
    this.activeModel = 'GLM-5.3-Flash',
    this.tokenCount = '28.4k tokens',
    this.activeGoal,
    List<Map<String, dynamic>>? runTimelineSteps,
    this.isTerminalOpen = false,
  })  : messages = messages ?? [],
        runTimelineSteps = runTimelineSteps ?? [];
}

class DesktopTaskWorkspaceController extends GetxController {
  final inputController = TextEditingController();
  final scrollController = ScrollController();

  final isRunning = false.obs;

  // Active Session info
  final activeSessionId = 'deepseek-1'.obs;
  final activeTaskTitle = 'Запуск exe-файла'.obs;
  final activeProject = RxnString('deepseek-harness-master');
  final activeBranch = RxnString('desktop-brand-ru');
  final isNewTask = false.obs;

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
  final activeModel = 'GLM-5.3-Flash'.obs;
  final tokenCount = '28.4k tokens'.obs;
  final isTerminalOpen = false.obs;

  void toggleTerminal() {
    isTerminalOpen.value = !isTerminalOpen.value;
    final current = sessions[activeSessionId.value];
    if (current != null) {
      current.isTerminalOpen = isTerminalOpen.value;
    }
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
  final Map<String, TaskSession> sessions = {};

  @override
  void onInit() {
    super.onInit();
    _initSampleSessions();
    _loadLocalCustomSessions();
    switchToSession('deepseek-1');
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

  void _initSampleSessions() {
    // 1. Omnes agent session
    sessions['omnes-1'] = TaskSession(
      id: 'omnes-1',
      title: 'План миграции Dart фронтенда в агента Zero...',
      project: 'Omnes agent',
      branch: 'feat/ade-split',
      activeModel: 'GLM-5.3-Flash',
      messages: [
        ChatMessage(
          text: 'Составь подробный план разделения монорепозитория OmnesAgent на отдельные клиенты Desktop и Mobile с единым пакетом omnes_shared.',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'План разделения разработан:\n\n1. Создан выделенный пакет `omnes_shared` для общих моделей сессий, тем и шлюза.\n2. Десктопный клиент вынесен в `frontend/desktop` со стилизацией под Windows ADE.\n3. Мобильный клиент `frontend/mobile` адаптирован под компактный Touch-интерфейс.\n4. Общая дизайн-система `shadcn/ui + reui` гарантирует единый стиль на всех платформах.',
          chatMessageType: ChatMessageType.bot,
          thinking: 'Thought: Анализ кодовой базы Flutter завершен. Структура монорепозитория чистая.',
          toolCalls: [
            ToolCallInfo(name: 'analyze_project_structure', args: '{"path": "frontend"}', output: 'Workspace separated: desktop, mobile, omnes_shared'),
          ],
        ),
      ],
    );

    // 2. deepseek-harness-master - Session 1
    sessions['deepseek-1'] = TaskSession(
      id: 'deepseek-1',
      title: 'Запуск exe-файла',
      project: 'deepseek-harness-master',
      branch: 'desktop-brand-ru',
      activeModel: 'GLM-5.3-Flash',
      activeGoal: GoalStatus(
        objective: 'Полная адаптация десктопного UI OmnesAgent ADE и проверка тестами',
        elapsedSeconds: 245,
        currentIteration: 2,
        maxIterations: 10,
        checklist: [
          GoalChecklistItem(id: '1', title: 'Создать трехпанельный ADE лэйаут и переключатель тем', iteration: 1, isCompleted: true),
          GoalChecklistItem(id: '2', title: 'Реализовать сайдбар с задачами и бейджами Git Diff (+142 -28)', iteration: 1, isCompleted: true),
          GoalChecklistItem(id: '3', title: 'Внедрить Composer с селектором Permission Mode (Shift+Tab)', iteration: 2, isCompleted: true),
          GoalChecklistItem(id: '4', title: 'Добавить Live Browser с Element Picker и встроенный терминал', iteration: 2, isCompleted: false),
        ],
      ),
      messages: [
        ChatMessage(
          text: 'вообще отсутствует -> сравнение патча бандла в рантайме против свежего -> diff ровно в этой строке.\n\nСервер 0.1.3 теперь поднимается, exe проверен, поднимается за считанные секунды и загрузил UI. Фикс внесён в сам скрипт сборки, поэтому при будущих синках воспроизведения не будет.\n\nob2h подключён\n• Проект зарегистрирован как dsh-desktop, 362 символа в AST графе.\n• В долговременную память записаны: состояние запуска, факты окружения, мерж ветки.\n\nСостояние ветки:\ndesktop-brand-ru запущена полностью: merge с upstream dsh v0.1.3-alpha.1, дрейфа нет, словари ru под ключ.',
          chatMessageType: ChatMessageType.bot,
          thinking: 'Progress 5/5:\n✓ Graft + merge upstream dsh v0.1.3-alpha.1, resolve 519 conflicts\n✓ typecheck/i18n/tests/build green; ru dictionaries completed\n✓ Runtime synced to 0.1.3; root cause of fileUploads pending found and fixed\n✓ Exe smoke: boots with live URL on synced 0.1.3 runtime\n✓ ob2h подключён (project dsh-desktop), факты в памяти, всё запущено',
          toolCalls: [
            ToolCallInfo(
              name: 'git_status_check',
              args: '{"branch": "desktop-brand-ru", "diff_stats": true}',
              output: 'Changes: +72347 -0 in 48 modules',
            ),
            ToolCallInfo(
              name: 'flutter_analyze',
              args: '{"target": "lib/features/workspace/task_workspace_view.dart"}',
              output: 'No issues found! (0 errors)',
            ),
          ],
        ),
        ChatMessage(
          text: 'То есть, например, картинки Training the Wife лежат в C:\\Users\\ipres\\AppData\\Local\\Temp\\final_tw\\images\\... , а кадры из SWF — в Temp\\final_swf\\Olivia Ellie and Paul rus\\images .\n\nЕсли осмотрели и всё лишнее — можно освободить место одной командой:\n\n```text\nrm -rf /tmp/final_tw /tmp/tw_folder /tmp/final_swf /tmp/swf_folder /tmp/swf_test /tmp/wdpk_test /tmp/wdpk_test2 /tmp/wc\n```\n\nСкажите, если хотите, чтобы я сам их удалил (или наоборот — перенёс куда-нибудь в постоянную папку).',
          chatMessageType: ChatMessageType.bot,
        ),
      ],
    );

    // 3. deepseek-harness-master - Session 2
    sessions['deepseek-2'] = TaskSession(
      id: 'deepseek-2',
      title: 'Переключение на ветку desktop и запуск exe',
      project: 'deepseek-harness-master',
      branch: 'desktop-brand-ru',
      activeModel: 'DeepSeek-V3',
      messages: [
        ChatMessage(
          text: 'Переключи ветку на desktop-brand-ru и выполни чистую компиляцию.',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'Ветка переключена успешно. Выполнен `git checkout desktop-brand-ru`. Зависимости обновлены.',
          chatMessageType: ChatMessageType.bot,
          toolCalls: [
            ToolCallInfo(name: 'git_checkout', args: '{"branch": "desktop-brand-ru"}', output: 'Switched to branch desktop-brand-ru'),
          ],
        ),
      ],
    );

    // 4. us proxy vps - Session 1
    sessions['proxy-1'] = TaskSession(
      id: 'proxy-1',
      title: 'реши проблему с прокси нихера не работает',
      project: 'us proxy vps',
      branch: 'main',
      activeModel: 'Claude-3.5-Sonnet',
      messages: [
        ChatMessage(
          text: 'реши проблему с прокси нихера не работает, запросы к внешним провайдерам падают по таймауту',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'Проблема локализована:\n\n1. SOCKS5 прокси на VPS (порт 1080) требовал авторизации, а в переменной `ALL_PROXY` пароль не передавался.\n2. Добавлен автоматический проброс SSH туннеля через Hermes VPS bridge (`~/hermes/vps_bridge.py`).\n3. Запросы к OpenAI, Anthropic и DeepSeek теперь идут с пингом 85мс без сбросов.',
          chatMessageType: ChatMessageType.bot,
          thinking: 'Diagnostics: curl -I -x socks5://127.0.0.1:1080 https://api.openai.com -> 200 OK. Трафик стабилизирован.',
        ),
      ],
    );

    // 5. us proxy vps - Session 2
    sessions['proxy-2'] = TaskSession(
      id: 'proxy-2',
      title: 'Отладка прокси Hermes и LLM API',
      project: 'us proxy vps',
      branch: 'main',
      activeModel: 'GPT-4o',
      messages: [
        ChatMessage(
          text: 'Проверь логи Hermes агента на VPS сервере.',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'Логи проверены: сессия демона стабильна, 0 ошибок авторизации за последние 24 часа.',
          chatMessageType: ChatMessageType.bot,
        ),
      ],
    );

    // 6. aprh.serv
    sessions['aprh-1'] = TaskSession(
      id: 'aprh-1',
      title: 'проанализируй проект посмотри логи напи...',
      project: 'aprh.serv',
      branch: 'prod',
      activeModel: 'GLM-5.3',
      messages: [
        ChatMessage(
          text: 'проанализируй проект посмотри логи напиши отчет',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'Анализ логов сервера aprh.serv выполнен:\n\n• Системный сервис systemd активен (running).\n• Nginx перенаправляет HTTPS запросы на порт 8080.\n• Выделение памяти: 1.4 ГБ из 8 ГБ (17.5%). Все службы работают в штатном режиме.',
          chatMessageType: ChatMessageType.bot,
        ),
      ],
    );

    // 7. AmoParallel - Session 1
    sessions['amo-1'] = TaskSession(
      id: 'amo-1',
      title: 'Диагностика ошибки агента Antigravity IDE',
      project: 'AmoParallel',
      branch: 'develop',
      activeModel: 'Claude-3.5-Sonnet',
      messages: [
        ChatMessage(
          text: 'Диагностика ошибки агента Antigravity IDE при параллельном запуске',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'Обнаружен race condition при одновременном обращении двух субагентов к одному файлу блокировки. Добавлен мьютекс и атомарная проверка очереди задач.',
          chatMessageType: ChatMessageType.bot,
        ),
      ],
    );

    // 8. AmoParallel - Session 2
    sessions['amo-2'] = TaskSession(
      id: 'amo-2',
      title: 'выполный план, только поставь веху и остан...',
      project: 'AmoParallel',
      branch: 'develop',
      activeModel: 'GLM-5.3-Flash',
      messages: [
        ChatMessage(
          text: 'выполни план, только поставь веху и остановись перед выполнением деструктивных команд',
          chatMessageType: ChatMessageType.user,
        ),
        ChatMessage(
          text: 'Веха установлена: `Milestone 1: Refactoring complete`. Ожидаю вашего подтверждения для дальнейших действий.',
          chatMessageType: ChatMessageType.bot,
        ),
      ],
    );
  }

  /// Switch active session or create new one if needed
  void switchToSession(
    String id, {
    String? title,
    String? project,
    String? branch,
  }) {
    _saveCurrentSessionState();

    var session = sessions[id];
    if (session == null) {
      session = TaskSession(
        id: id,
        title: title ?? 'Задача $id',
        project: project,
        branch: branch ?? 'main',
        messages: [
          ChatMessage(
            text: 'Открыта задача: **${title ?? id}**\nПроект: `${project ?? "Не привязан"}` | Ветка: `${branch ?? "main"}`',
            chatMessageType: ChatMessageType.bot,
          ),
        ],
      );
      sessions[id] = session;
    }

    activeSessionId.value = id;
    activeTaskTitle.value = session.title;
    activeProject.value = session.project;
    activeBranch.value = session.branch;
    permissionMode.value = session.permissionMode;
    thoughtLevel.value = session.thoughtLevel;
    activeModel.value = session.activeModel;
    tokenCount.value = session.tokenCount;
    activeGoal.value = session.activeGoal;
    isTerminalOpen.value = session.isTerminalOpen;

    messages.assignAll(session.messages);
    runTimelineSteps.assignAll(session.runTimelineSteps);
    attachments.clear();
    inputController.clear();

    isNewTask.value = false;
    _scrollToBottom();

    // Connect WebSocket and fetch backend messages for this session
    _connectWebSocket(id);
    fetchSessionMessages(id);
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
  void _handleGatewayFrame(GatewayFrame frame) {
    if (frame is ConnectedFrame) {
      wsStatus.value = 'connected';
    } else if (frame is ChunkFrame) {
      if (messages.isNotEmpty && messages.last.isStreaming && messages.last.chatMessageType == ChatMessageType.bot) {
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
        messages.refresh();
      } else {
        messages.add(ChatMessage(
          text: '',
          chatMessageType: ChatMessageType.bot,
          thinking: frame.content,
          isStreaming: true,
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
        messages.refresh();
      }
      isRunning.value = false;
    } else if (frame is ErrorFrame) {
      if (messages.isNotEmpty && messages.last.isStreaming) {
        messages.last.isStreaming = false;
        messages.last.isError = true;
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
          if (!sessions.containsKey(s.sessionId)) {
            sessions[s.sessionId] = TaskSession(
              id: s.sessionId,
              title: s.previewText.isNotEmpty ? s.previewText : s.sessionId,
              tokenCount: '${s.messageCount} сообщ.',
              project: s.workspaceDir?.split(RegExp(r'[\\/]')).last,
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

    final newId = 'task-${DateTime.now().millisecondsSinceEpoch}';
    activeSessionId.value = newId;
    activeTaskTitle.value = 'Новая задача';
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
      final customList = sessions.values
          .where((s) => !s.id.startsWith('deepseek-') &&
              !s.id.startsWith('proxy-') &&
              !s.id.startsWith('aprh-') &&
              !s.id.startsWith('amo-') &&
              !s.id.startsWith('omnes-'))
          .map((s) {
        return {
          'id': s.id,
          'title': s.title,
          'project': s.project,
          'branch': s.branch,
          'activeModel': s.activeModel,
          'messages': s.messages
              .map((m) => {
                    'text': m.text,
                    'isUser': m.chatMessageType == ChatMessageType.user,
                  })
              .toList(),
        };
      }).toList();
      storage.write('desktop_custom_sessions', customList);
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
            if (id.isNotEmpty && !sessions.containsKey(id)) {
              final msgs = (item['messages'] as List?)?.map((m) {
                return ChatMessage(
                  text: m['text']?.toString() ?? '',
                  chatMessageType:
                      m['isUser'] == true ? ChatMessageType.user : ChatMessageType.bot,
                );
              }).toList() ?? [];
              sessions[id] = TaskSession(
                id: id,
                title: item['title']?.toString() ?? id,
                project: item['project']?.toString(),
                branch: item['branch']?.toString() ?? 'main',
                activeModel: item['activeModel']?.toString() ?? 'GLM-5.3-Flash',
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
    sessions.remove(id);
    _saveLocalCustomSessions();
    await httpClient.deleteSession(id);
    if (activeSessionId.value == id) {
      if (sessions.isNotEmpty) {
        switchToSession(sessions.keys.first);
      } else {
        createNewTask();
      }
    }
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

    if (isNewTask.value) {
      // If we were on New Task screen, initialize session title from prompt
      isNewTask.value = false;
      final displayTitle = rawText.length > 35 ? '${rawText.substring(0, 35)}...' : rawText;
      activeTaskTitle.value = displayTitle;

      sessions[activeSessionId.value] = TaskSession(
        id: activeSessionId.value,
        title: displayTitle,
        project: activeProject.value,
        branch: activeBranch.value,
        permissionMode: permissionMode.value,
        thoughtLevel: thoughtLevel.value,
        activeModel: activeModel.value,
      );
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

    messages.add(ChatMessage(
      text: fullMessage,
      chatMessageType: ChatMessageType.user,
    ));

    inputController.clear();
    attachments.clear();
    isRunning.value = true;

    // Send to live WebSocket
    if (_wsClient != null && _wsClient!.isConnected) {
      _wsClient!.sendMessage(fullMessage);
    } else {
      _connectWebSocket(activeSessionId.value).then((_) {
        final sent = _wsClient?.sendMessage(fullMessage) ?? false;
        if (!sent) {
          // Graceful fallback for offline demo/standalone mode
          Future.delayed(const Duration(milliseconds: 600), () {
            messages.add(ChatMessage(
              text: 'Шлюз OmnesAgent (127.0.0.1:42617) пока не доступен. Сообщение сохранено локально. Запустите шлюз для обработки LLM.',
              chatMessageType: ChatMessageType.bot,
              thinking: 'Thought: WebSocket шлюза в ожидании подключения. Повторите попытку после запуска бэкенда.',
              isStreaming: false,
            ));
            isRunning.value = false;
            _saveCurrentSessionState();
            _scrollToBottom();
          });
        }
      });
    }

    _scrollToBottom();
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
      pendingApprovals.removeAt(index);
    }
  }

  void denyAction(int index) {
    if (index < pendingApprovals.length) {
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
