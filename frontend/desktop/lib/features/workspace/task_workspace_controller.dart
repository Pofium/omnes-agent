// Desktop Task Workspace Controller with ZCode ADE Permission Modes, Goal Tracking, and Context.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';

class DesktopTaskWorkspaceController extends GetxController {
  final inputController = TextEditingController();
  final scrollController = ScrollController();

  final isRunning = false.obs;
  final activeTaskTitle = 'Goal: Рефакторинг UI десктопа под ZCode ADE'.obs;

  // ZCode ADE Core State
  final permissionMode = PermissionMode.askBeforeChanges.obs;
  final thoughtLevel = 'Max'.obs;
  final activeModel = 'GLM-5.3'.obs;
  final tokenCount = '28.4k tokens'.obs;

  // Goal Mode State
  final Rxn<GoalStatus> activeGoal = Rxn<GoalStatus>();

  // Attachments & Mentions (@file, #chat, chips)
  final attachments = <String>[].obs;

  final messages = <ChatMessage>[].obs;
  final pendingApprovals = <Map<String, dynamic>>[].obs;
  final runTimelineSteps = <Map<String, dynamic>>[].obs;

  @override
  void onInit() {
    super.onInit();
    _loadSampleWorkspaceState();
  }

  void _loadSampleWorkspaceState() {
    // Initialize sample goal mode
    activeGoal.value = GoalStatus(
      objective: 'Полная адаптация десктопного UI под ZCode ADE и проверка тестами',
      elapsedSeconds: 245,
      currentIteration: 2,
      maxIterations: 10,
      checklist: [
        GoalChecklistItem(
          id: '1',
          title: 'Создать трехпанельный ADE лэйаут и переключатель тем',
          iteration: 1,
          isCompleted: true,
        ),
        GoalChecklistItem(
          id: '2',
          title: 'Реализовать сайдбар с задачами и бейджами Git Diff (+142 -28)',
          iteration: 1,
          isCompleted: true,
        ),
        GoalChecklistItem(
          id: '3',
          title: 'Внедрить Composer с селектором Permission Mode (Shift+Tab)',
          iteration: 2,
          isCompleted: true,
        ),
        GoalChecklistItem(
          id: '4',
          title: 'Добавить Live Browser с Element Picker и встроенный терминал',
          iteration: 2,
          isCompleted: false,
        ),
      ],
    );

    messages.addAll([
      ChatMessage(
        text: '/goal Полная адаптация десктопного UI под ZCode ADE и проверка тестами',
        chatMessageType: ChatMessageType.user,
      ),
      ChatMessage(
        text: 'Цель принята. Запускаю автономный итерационный цикл разработки с подтверждением по тестам (Goal Mode).\n\nШаг 1: Развёрнута трёхпанельная архитектура ZCode ADE.\nШаг 2: Внедрены цветовые схемы Dark (#090D12, #00D2FF) и Light (#F8FAFC, #0284C7).\nШаг 3: Сайдбар оснащен бейджами изменений строк кода `+142 -28`.\n\nПриступаю к интеграции Live Browser с Element Picker и инспектора терминала `⌘J`.',
        chatMessageType: ChatMessageType.bot,
        thinking: 'Thought (Max):\n1. Анализ layout: центральный холст отведен под агента, правая панель — под Live Browser и Terminal.\n2. Реализация Permission Mode (Ask before changes / Edit auto / Plan / Full access).\n3. Проверка тестами компиляции через `flutter analyze`.',
        toolCalls: [
          ToolCallInfo(
            name: 'git_status_check',
            args: '{"branch": "main", "diff_stats": true}',
            output: 'Changes: +142 -28 in 4 modules',
          ),
          ToolCallInfo(
            name: 'flutter_analyze',
            args: '{"target": "lib/widgets/desktop_sidebar.dart"}',
            output: 'No issues found! (0 errors)',
          ),
        ],
      ),
    ]);

    runTimelineSteps.addAll([
      {
        'title': 'Setup ZCode ADE Architecture',
        'tool': 'architect_ade',
        'duration': '140ms',
        'status': 'success',
      },
      {
        'title': 'Compile Theme Tokens',
        'tool': 'desktop_theme',
        'duration': '95ms',
        'status': 'success',
      },
      {
        'title': 'Run Test Suite Verification',
        'tool': 'flutter_analyze',
        'duration': '210ms',
        'status': 'success',
      },
    ]);
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

    // Simulate Agent Step with Thought & Tools
    Future.delayed(const Duration(milliseconds: 700), () {
      messages.add(ChatMessage(
        text: 'Шаг выполнен в режиме **${permissionMode.value.label}** с моделью **${activeModel.value}** (Thought: ${thoughtLevel.value}). Результаты верифицированы.',
        chatMessageType: ChatMessageType.bot,
        thinking: 'Thought (${thoughtLevel.value}): Выполнен анализ контекста. Код верифицирован без ошибок.',
        isStreaming: false,
      ));
      isRunning.value = false;
      _scrollToBottom();
    });

    _scrollToBottom();
  }

  void _handleGoalCommand(String cmd) {
    final parts = cmd.split(' ');
    if (parts.length == 1) {
      // /goal -> print current
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
      // /goal <objective>
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

  void abortRun() {
    isRunning.value = false;
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
    inputController.dispose();
    scrollController.dispose();
    super.onClose();
  }
}
