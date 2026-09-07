// Desktop Task Workspace Controller with WebSocket Streaming and Tool Execution state.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';

class DesktopTaskWorkspaceController extends GetxController {
  final inputController = TextEditingController();
  final scrollController = ScrollController();

  final isRunning = false.obs;
  final isSupervisedMode = true.obs;
  final activeTaskTitle = 'Refactor agent architecture & split desktop client'.obs;

  final messages = <ChatMessage>[].obs;
  final pendingApprovals = <Map<String, dynamic>>[].obs;
  final runTimelineSteps = <Map<String, dynamic>>[].obs;

  @override
  void onInit() {
    super.onInit();
    _loadSampleWorkspaceState();
  }

  void _loadSampleWorkspaceState() {
    // Initial loaded state representing an autonomous agent session
    messages.addAll([
      ChatMessage(
        text: 'Проанализируй архитектуру OmnesAgent и подготовь план разделения мобильной и десктопной версий по ТЗ DESIGN_DESKTOP.md',
        chatMessageType: ChatMessageType.user,
      ),
      ChatMessage(
        text: 'Анализ кодовой базы завершён. Выявлены следующие ключевые модули для выноса в shared-пакет:\n- `core/gateway` (REST и WebSocket)\n- `helper/local_storage` (PIN, настройки, токены)\n- `design_system` (ReUI токены)\n- `utils/strings` (RU/EN словари)\n\nСоздан архитектурный план `PLAN_DESKTOP_MOBILE_SPLIT.md`. Приступаю к реализации десктопной трёхпанельной рабочей станции.',
        chatMessageType: ChatMessageType.bot,
        thinking: '1. Проверить зависимости и платформы\n2. Оценить влияние ScreenUtil на Desktop\n3. Разработать чистый Desktop Shell на ReUI',
        toolCalls: [
          ToolCallInfo(
            name: 'analyze_codebase',
            args: '{"scope": "frontend", "target": "desktop"}',
            output: 'Completed. 0 errors, 42 modules analyzed.',
          ),
          ToolCallInfo(
            name: 'write_plan',
            args: '{"path": "PLAN_DESKTOP_MOBILE_SPLIT.md"}',
            output: 'Plan written successfully.',
          ),
        ],
      ),
    ]);

    runTimelineSteps.addAll([
      {
        'title': 'Analyze Project Structure',
        'tool': 'list_dir',
        'duration': '140ms',
        'status': 'success',
      },
      {
        'title': 'Read Desktop Design Spec',
        'tool': 'view_file',
        'duration': '210ms',
        'status': 'success',
      },
      {
        'title': 'Create Shared Core Package',
        'tool': 'write_to_file',
        'duration': '350ms',
        'status': 'success',
      },
    ]);
  }

  void sendMessage() {
    final text = inputController.text.trim();
    if (text.isEmpty) return;

    messages.add(ChatMessage(
      text: text,
      chatMessageType: ChatMessageType.user,
    ));
    inputController.clear();
    isRunning.value = true;

    // Simulate Agent Step
    Future.delayed(const Duration(milliseconds: 600), () {
      messages.add(ChatMessage(
        text: 'Принято. Выполняю шаг в режиме ${isSupervisedMode.value ? "Supervised" : "Autonomous"}...',
        chatMessageType: ChatMessageType.bot,
        isStreaming: true,
      ));
      isRunning.value = false;
      _scrollToBottom();
    });

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
