// Desktop Task Workspace Canvas: Task Header, Reasoning Blocks, Tool Cards, and Input Bar.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';

import '../../theme/desktop_theme.dart';
import 'task_workspace_controller.dart';

class DesktopTaskWorkspaceView extends StatelessWidget {
  final DesktopTaskWorkspaceController controller;

  const DesktopTaskWorkspaceView({super.key, required this.controller});

  @override
  Widget build(BuildContext context) {
    return Container(
      color: DesktopTheme.bgCanvas,
      child: Column(
        children: [
          // 1. Task Objective Header
          _buildTaskHeader(context),

          // 2. Chat / Event Stream Canvas
          Expanded(
            child: Obx(
              () => controller.messages.isEmpty
                  ? _buildEmptyState(context)
                  : ListView.builder(
                      controller: controller.scrollController,
                      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
                      itemCount: controller.messages.length,
                      itemBuilder: (context, index) {
                        final msg = controller.messages[index];
                        return _buildMessageItem(context, msg);
                      },
                    ),
            ),
          ),

          // 3. Bottom Prompt Input Station
          _buildInputStation(context),
        ],
      ),
    );
  }

  Widget _buildTaskHeader(BuildContext context) {
    return Container(
      height: 44,
      padding: const EdgeInsets.symmetric(horizontal: 20),
      decoration: const BoxDecoration(
        color: DesktopTheme.bgSurface,
        border: Border(
          bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Row(
        children: [
          const Icon(
            FontAwesomeIcons.circleDot,
            size: 11,
            color: DesktopTheme.accentCyan,
          ),
          const SizedBox(width: 8),
          const Text(
            'ACTIVE TASK:',
            style: TextStyle(
              fontSize: 11,
              fontWeight: FontWeight.bold,
              letterSpacing: 0.8,
              color: DesktopTheme.textMuted,
            ),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Obx(
              () => Text(
                controller.activeTaskTitle.value,
                style: const TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.w600,
                  color: DesktopTheme.textPrimary,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ),
          const SizedBox(width: 12),

          // Supervised / Autonomous Switcher
          Obx(
            () => InkWell(
              onTap: () {
                controller.isSupervisedMode.value =
                    !controller.isSupervisedMode.value;
              },
              borderRadius: BorderRadius.circular(4),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: controller.isSupervisedMode.value
                      ? DesktopTheme.accentBlue.withOpacity(0.15)
                      : DesktopTheme.statusSuccess.withOpacity(0.15),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(
                    color: controller.isSupervisedMode.value
                        ? DesktopTheme.accentBlue.withOpacity(0.4)
                        : DesktopTheme.statusSuccess.withOpacity(0.4),
                  ),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      controller.isSupervisedMode.value
                          ? Icons.security_rounded
                          : Icons.bolt_rounded,
                      size: 13,
                      color: controller.isSupervisedMode.value
                          ? DesktopTheme.accentSky
                          : DesktopTheme.statusSuccess,
                    ),
                    const SizedBox(width: 4),
                    Text(
                      controller.isSupervisedMode.value
                          ? 'Supervised'
                          : 'Autonomous',
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.bold,
                        color: controller.isSupervisedMode.value
                            ? DesktopTheme.accentSky
                            : DesktopTheme.statusSuccess,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMessageItem(BuildContext context, ChatMessage msg) {
    final isBot = msg.chatMessageType == ChatMessageType.bot;

    if (!isBot) {
      // User message
      return Container(
        margin: const EdgeInsets.only(bottom: 18, left: 60),
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: DesktopTheme.bgSurfaceElevated,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: DesktopTheme.borderSubtle),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              padding: const EdgeInsets.all(6),
              decoration: BoxDecoration(
                color: DesktopTheme.borderMedium,
                borderRadius: BorderRadius.circular(4),
              ),
              child: const Icon(Icons.person, size: 14, color: Colors.white),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: SelectableText(
                msg.text,
                style: const TextStyle(
                  fontSize: 14,
                  height: 1.5,
                  color: DesktopTheme.textPrimary,
                ),
              ),
            ),
          ],
        ),
      );
    }

    // Bot message
    return Container(
      margin: const EdgeInsets.only(bottom: 18),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header: Agent Icon + Name + Model
          Row(
            children: [
              Container(
                width: 24,
                height: 24,
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(5),
                ),
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(5),
                  child: Image.asset(
                    'assets/Logo/app_launcher.png',
                    fit: BoxFit.cover,
                  ),
                ),
              ),
              const SizedBox(width: 10),
              const Text(
                'OmnesAgent',
                style: TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.bold,
                  color: DesktopTheme.textPrimary,
                ),
              ),
              const SizedBox(width: 8),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                decoration: BoxDecoration(
                  color: DesktopTheme.bgSurfaceElevated,
                  borderRadius: BorderRadius.circular(3),
                ),
                child: const Text(
                  'Claude 3.5 Sonnet',
                  style: TextStyle(
                    fontSize: 10,
                    fontFamily: 'Consolas',
                    color: DesktopTheme.textMuted,
                  ),
                ),
              ),
            ],
          ),

          // Collapsible Thinking Block
          if (msg.thinking != null && msg.thinking!.isNotEmpty) ...[
            const SizedBox(height: 12),
            _buildThinkingBlock(msg.thinking!),
          ],

          // Tool execution blocks
          if (msg.toolCalls.isNotEmpty) ...[
            const SizedBox(height: 10),
            ...msg.toolCalls.map((t) => _buildToolCallBlock(t)),
          ],

          const SizedBox(height: 12),

          // Message content
          SelectableText(
            msg.text,
            style: const TextStyle(
              fontSize: 14,
              height: 1.6,
              color: DesktopTheme.textPrimary,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildThinkingBlock(String thinking) {
    return Container(
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: Colors.black26,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: Colors.amber.withOpacity(0.2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Row(
            children: [
              Icon(Icons.psychology_outlined, size: 14, color: Colors.amberAccent),
              SizedBox(width: 6),
              Text(
                'Reasoning & Plan',
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  color: Colors.amberAccent,
                ),
              ),
            ],
          ),
          const SizedBox(height: 6),
          SelectableText(
            thinking,
            style: TextStyle(
              fontSize: 12,
              height: 1.4,
              fontFamily: 'Consolas',
              color: Colors.white.withOpacity(0.75),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildToolCallBlock(ToolCallInfo tool) {
    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurfaceElevated,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: DesktopTheme.accentCyan.withOpacity(0.2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.build_circle_outlined, size: 14, color: DesktopTheme.accentCyan),
              const SizedBox(width: 6),
              Text(
                'tool: ${tool.name}',
                style: const TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                  fontFamily: 'Consolas',
                  color: DesktopTheme.accentCyan,
                ),
              ),
              const Spacer(),
              const Icon(Icons.check_circle, size: 13, color: DesktopTheme.statusSuccess),
            ],
          ),
          if (tool.output != null) ...[
            const SizedBox(height: 4),
            Text(
              tool.output!,
              style: const TextStyle(
                fontSize: 11,
                fontFamily: 'Consolas',
                color: DesktopTheme.textMuted,
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildInputStation(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: const BoxDecoration(
        color: DesktopTheme.bgSidebar,
        border: Border(
          top: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Container(
        decoration: BoxDecoration(
          color: DesktopTheme.bgSurface,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: DesktopTheme.borderMedium),
        ),
        child: Column(
          children: [
            // Multiline prompt field
            RawKeyboardListener(
              focusNode: FocusNode(),
              onKey: (event) {
                if (event is RawKeyDownEvent &&
                    event.logicalKey == LogicalKeyboardKey.enter &&
                    !event.isShiftPressed) {
                  controller.sendMessage();
                }
              },
              child: TextField(
                controller: controller.inputController,
                maxLines: 4,
                minLines: 2,
                style: const TextStyle(
                  fontSize: 14,
                  color: DesktopTheme.textPrimary,
                  fontFamily: 'Segoe UI',
                ),
                decoration: const InputDecoration(
                  hintText: 'Ask OmnesAgent to plan, write code, run commands, or execute tasks...',
                  hintStyle: TextStyle(
                    color: DesktopTheme.textMuted,
                    fontSize: 13,
                  ),
                  filled: false,
                  border: InputBorder.none,
                  enabledBorder: InputBorder.none,
                  focusedBorder: InputBorder.none,
                  contentPadding: EdgeInsets.all(14),
                ),
              ),
            ),

            // Bottom toolbar inside input station
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
              decoration: const BoxDecoration(
                border: Border(
                  top: BorderSide(color: DesktopTheme.borderSubtle, width: 0.8),
                ),
              ),
              child: Row(
                children: [
                  IconButton(
                    tooltip: 'Attach Workspace File',
                    icon: const Icon(Icons.attach_file, size: 16, color: DesktopTheme.textMuted),
                    onPressed: () {},
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(),
                  ),
                  const SizedBox(width: 12),
                  const Text(
                    'Shift + Enter for new line',
                    style: TextStyle(
                      fontSize: 11,
                      color: DesktopTheme.textMuted,
                    ),
                  ),
                  const Spacer(),
                  Obx(
                    () => controller.isRunning.value
                        ? OutlinedButton.icon(
                            style: OutlinedButton.styleFrom(
                              foregroundColor: Colors.redAccent,
                              side: const BorderSide(color: Colors.redAccent),
                              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                            ),
                            icon: const Icon(Icons.stop_circle_outlined, size: 15),
                            label: const Text('Abort Run', style: TextStyle(fontSize: 12)),
                            onPressed: controller.abortRun,
                          )
                        : ElevatedButton.icon(
                            style: ElevatedButton.styleFrom(
                              backgroundColor: DesktopTheme.accentSky,
                              foregroundColor: Colors.black,
                              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(6),
                              ),
                            ),
                            icon: const Icon(Icons.send_rounded, size: 14),
                            label: const Text(
                              'Run',
                              style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
                            ),
                            onPressed: controller.sendMessage,
                          ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: SingleChildScrollView(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 84,
              height: 84,
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(20),
                boxShadow: [
                  BoxShadow(
                    color: const Color(0xFF00D2FF).withOpacity(0.3),
                    blurRadius: 30,
                    offset: const Offset(0, 8),
                  ),
                ],
              ),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(20),
                child: Image.asset(
                  'assets/Logo/app_launcher.png',
                  fit: BoxFit.cover,
                ),
              ),
            ),
            const SizedBox(height: 20),
            const Text(
              'OmnesAgent ADE',
              style: TextStyle(
                fontSize: 22,
                fontWeight: FontWeight.bold,
                letterSpacing: 0.6,
                color: DesktopTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              'Agentic Development Environment — постановка целей и автономная разработка',
              style: TextStyle(
                fontSize: 13,
                color: DesktopTheme.textMuted,
              ),
            ),
            const SizedBox(height: 24),
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                _buildQuickHintChip('/goal <цель>', 'Автономный цикл с верификацией'),
                const SizedBox(width: 10),
                _buildQuickHintChip('@file', 'Файлы воркспейса'),
                const SizedBox(width: 10),
                _buildQuickHintChip('Shift + Tab', 'Режим разрешений'),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildQuickHintChip(String shortcut, String desc) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: DesktopTheme.bgCanvas,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: DesktopTheme.borderMedium),
            ),
            child: Text(
              shortcut,
              style: const TextStyle(
                fontSize: 11,
                fontFamily: 'Consolas',
                fontWeight: FontWeight.bold,
                color: DesktopTheme.accentCyan,
              ),
            ),
          ),
          const SizedBox(width: 8),
          Text(
            desc,
            style: const TextStyle(
              fontSize: 11,
              color: DesktopTheme.textSecondary,
            ),
          ),
        ],
      ),
    );
  }
}
