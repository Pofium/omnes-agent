// Desktop Task Workspace View: Goal Mode Summary, Reasoning Stream, and ADE Composer.

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
          // 1. Goal Mode Summary Card (Sticky Header if Active)
          Obx(() => _buildGoalHeader(context)),

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
                        return _buildMessageItem(context, msg, index);
                      },
                    ),
            ),
          ),

          // 3. Central Prompt Box (Composer)
          _buildComposer(context),
        ],
      ),
    );
  }

  Widget _buildGoalHeader(BuildContext context) {
    final goal = controller.activeGoal.value;
    if (goal == null) {
      return Container(
        height: 40,
        padding: const EdgeInsets.symmetric(horizontal: 16),
        decoration: BoxDecoration(
          color: DesktopTheme.bgSurface,
          border: Border(
            bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
          ),
        ),
        child: Row(
          children: [
            const Icon(Icons.gps_fixed, size: 13, color: DesktopTheme.textMutedDark),
            const SizedBox(width: 8),
            Text(
              'No active goal. Type /goal <objective> to start an autonomous loop.',
              style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted),
            ),
          ],
        ),
      );
    }

    return Container(
      padding: const EdgeInsets.fromLTRB(16, 10, 16, 10),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        border: Border(
          bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
                decoration: BoxDecoration(
                  color: DesktopTheme.accentCyan.withOpacity(0.12),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: DesktopTheme.accentCyan.withOpacity(0.3)),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: const [
                    Icon(Icons.flag_rounded, size: 12, color: DesktopTheme.accentCyan),
                    SizedBox(width: 4),
                    Text(
                      'GOAL MODE',
                      style: TextStyle(
                        fontSize: 10,
                        fontWeight: FontWeight.bold,
                        letterSpacing: 0.5,
                        color: DesktopTheme.accentCyan,
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: Text(
                  goal.objective,
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.w600,
                    color: DesktopTheme.textPrimary,
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 12),
              Text(
                'Elapsed: ${goal.formattedElapsed}',
                style: TextStyle(
                  fontSize: 11,
                  fontFamily: 'Consolas',
                  color: DesktopTheme.textMuted,
                ),
              ),
              const SizedBox(width: 12),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: DesktopTheme.bgSurfaceElevated,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  'Iteration ${goal.currentIteration}/${goal.maxIterations}',
                  style: TextStyle(
                    fontSize: 11,
                    fontFamily: 'Consolas',
                    fontWeight: FontWeight.bold,
                    color: DesktopTheme.textSecondary,
                  ),
                ),
              ),
            ],
          ),
          if (goal.checklist.isNotEmpty) ...[
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 4,
              children: goal.checklist.map((item) {
                return Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                  decoration: BoxDecoration(
                    color: item.isCompleted
                        ? DesktopTheme.statusSuccess.withOpacity(0.08)
                        : DesktopTheme.bgCanvas,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(
                      color: item.isCompleted
                          ? DesktopTheme.statusSuccess.withOpacity(0.25)
                          : DesktopTheme.borderSubtle,
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        item.isCompleted ? Icons.check_circle : Icons.radio_button_unchecked,
                        size: 12,
                        color: item.isCompleted
                            ? DesktopTheme.statusSuccess
                            : DesktopTheme.textMuted,
                      ),
                      const SizedBox(width: 5),
                      Text(
                        item.title,
                        style: TextStyle(
                          fontSize: 11,
                          color: item.isCompleted
                              ? DesktopTheme.textSecondary
                              : DesktopTheme.textMuted,
                          decoration: item.isCompleted ? TextDecoration.lineThrough : null,
                        ),
                      ),
                    ],
                  ),
                );
              }).toList(),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildMessageItem(BuildContext context, ChatMessage msg, int index) {
    final isBot = msg.chatMessageType == ChatMessageType.bot;

    // Optional: Iteration Divider before a bot message if goal is active
    Widget? iterationDivider;
    if (isBot && index == 1) {
      iterationDivider = Container(
        margin: const EdgeInsets.symmetric(vertical: 16),
        child: Row(
          children: [
            Expanded(child: Divider(color: DesktopTheme.borderSubtle)),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurface,
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: DesktopTheme.borderSubtle),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.refresh_rounded, size: 12, color: DesktopTheme.accentCyan),
                  const SizedBox(width: 6),
                  Text(
                    'Iteration 1 · Goal not met, task continues',
                    style: TextStyle(
                      fontSize: 11,
                      fontFamily: 'Consolas',
                      color: DesktopTheme.textSecondary,
                    ),
                  ),
                ],
              ),
            ),
            Expanded(child: Divider(color: DesktopTheme.borderSubtle)),
          ],
        ),
      );
    }

    if (!isBot) {
      // User message
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (iterationDivider != null) iterationDivider,
          Container(
            margin: const EdgeInsets.only(bottom: 16, left: 80),
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
                  padding: const EdgeInsets.all(5),
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
                    style: TextStyle(
                      fontSize: 13,
                      height: 1.5,
                      color: DesktopTheme.textPrimary,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      );
    }

    // Bot message
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (iterationDivider != null) iterationDivider,
        Container(
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
              // Header: Agent Icon + Model & Thought Level Pill
              Row(
                children: [
                  ClipRRect(
                    borderRadius: BorderRadius.circular(5),
                    child: Image.asset(
                      'assets/Logo/app_launcher.png',
                      width: 24,
                      height: 24,
                      fit: BoxFit.cover,
                    ),
                  ),
                  const SizedBox(width: 10),
                  Text(
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
                    child: Text(
                      '${controller.activeModel.value} (Thought: ${controller.thoughtLevel.value})',
                      style: TextStyle(
                        fontSize: 10,
                        fontFamily: 'Consolas',
                        color: DesktopTheme.textMuted,
                      ),
                    ),
                  ),
                ],
              ),

              // Thought Accordion Block
              if (msg.thinking != null && msg.thinking!.isNotEmpty) ...[
                const SizedBox(height: 12),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: DesktopTheme.bgCanvas,
                    borderRadius: BorderRadius.circular(6),
                    border: Border(
                      left: BorderSide(color: DesktopTheme.accentCyan, width: 3),
                      top: BorderSide(color: DesktopTheme.borderSubtle),
                      right: BorderSide(color: DesktopTheme.borderSubtle),
                      bottom: BorderSide(color: DesktopTheme.borderSubtle),
                    ),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(Icons.lightbulb_outline, size: 13, color: DesktopTheme.accentCyan),
                          const SizedBox(width: 6),
                          Text(
                            'Agent Thought (${controller.thoughtLevel.value})',
                            style: const TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.bold,
                              letterSpacing: 0.4,
                              color: DesktopTheme.accentCyan,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 6),
                      SelectableText(
                        msg.thinking!,
                        style: TextStyle(
                          fontSize: 12,
                          height: 1.4,
                          fontFamily: 'Consolas',
                          color: DesktopTheme.textSecondary,
                        ),
                      ),
                    ],
                  ),
                ),
              ],

              // Main Message Text
              const SizedBox(height: 12),
              SelectableText(
                msg.text,
                style: TextStyle(
                  fontSize: 13,
                  height: 1.5,
                  color: DesktopTheme.textPrimary,
                ),
              ),

              // Tool Calls
              if (msg.toolCalls.isNotEmpty) ...[
                const SizedBox(height: 14),
                ...msg.toolCalls.map((tool) => _buildToolCard(tool)),
              ],
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildToolCard(ToolCallInfo tool) {
    return Container(
      margin: const EdgeInsets.only(top: 8),
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: DesktopTheme.bgCanvas,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(FontAwesomeIcons.wrench, size: 11, color: DesktopTheme.accentSky),
              const SizedBox(width: 8),
              Text(
                'Tool: ${tool.name}',
                style: const TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  fontFamily: 'Consolas',
                  color: DesktopTheme.accentSky,
                ),
              ),
              const Spacer(),
              const Icon(Icons.check_circle, size: 13, color: DesktopTheme.statusSuccess),
            ],
          ),
          if (tool.output != null && tool.output!.isNotEmpty) ...[
            const SizedBox(height: 6),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurface,
                borderRadius: BorderRadius.circular(4),
              ),
              child: SelectableText(
                tool.output!,
                style: TextStyle(
                  fontSize: 11,
                  fontFamily: 'Consolas',
                  color: DesktopTheme.textMuted,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  // 3. Central Prompt Box (Composer)
  Widget _buildComposer(BuildContext context) {
    return Container(
      padding: const EdgeInsets.fromLTRB(20, 10, 20, 14),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        border: Border(
          top: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Attachment Chips (Mentions, DOM elements, etc.)
          Obx(() {
            if (controller.attachments.isEmpty) return const SizedBox.shrink();
            return Container(
              margin: const EdgeInsets.only(bottom: 8),
              child: Wrap(
                spacing: 6,
                runSpacing: 4,
                children: controller.attachments.asMap().entries.map((entry) {
                  final idx = entry.key;
                  final text = entry.value;
                  return Chip(
                    label: Text(text, style: const TextStyle(fontSize: 11, fontFamily: 'Consolas')),
                    backgroundColor: DesktopTheme.bgCanvas,
                    deleteIcon: const Icon(Icons.close, size: 12),
                    onDeleted: () => controller.removeAttachment(idx),
                    padding: const EdgeInsets.symmetric(horizontal: 4),
                    materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                  );
                }).toList(),
              ),
            );
          }),

          // Input Text Field with Context Button "+"
          Container(
            decoration: BoxDecoration(
              color: DesktopTheme.bgCanvas,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: DesktopTheme.borderSubtle),
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                // Quick Context Insert "+" Menu
                PopupMenuButton<String>(
                  tooltip: 'Добавить контекст (+)',
                  icon: const Icon(Icons.add_circle_outline, size: 18, color: DesktopTheme.accentCyan),
                  onSelected: (action) {
                    if (action == 'mention') {
                      controller.addAttachment('@lib/main.dart');
                    } else if (action == 'chat') {
                      controller.addAttachment('#session-prev');
                    } else if (action == 'cmd') {
                      controller.inputController.text = '/goal ';
                    } else if (action == 'skill') {
                      controller.addAttachment(r'$ob2h');
                    }
                  },
                  itemBuilder: (context) => [
                    const PopupMenuItem(
                      value: 'mention',
                      child: Text('Вставить @ Mention (файл/директория)'),
                    ),
                    const PopupMenuItem(
                      value: 'chat',
                      child: Text('Подключить # Conversation (история)'),
                    ),
                    const PopupMenuItem(
                      value: 'cmd',
                      child: Text('Выполнить / Command (/goal, /side)'),
                    ),
                    const PopupMenuItem(
                      value: 'skill',
                      child: Text(r'Вызвать $ Skill ($ob2h, $flutter)'),
                    ),
                  ],
                ),

                // Text Input
                Expanded(
                  child: RawKeyboardListener(
                    focusNode: FocusNode(),
                    onKey: (event) {
                      // Shift + Tab listener to cycle permission mode
                      if (event is RawKeyDownEvent &&
                          event.isShiftPressed &&
                          event.logicalKey == LogicalKeyboardKey.tab) {
                        controller.cyclePermissionMode();
                      }
                    },
                    child: TextField(
                      controller: controller.inputController,
                      maxLines: 4,
                      minLines: 1,
                      style: TextStyle(fontSize: 13, color: DesktopTheme.textPrimary),
                      decoration: const InputDecoration(
                        hintText: 'Задайте цель (/goal) или промпт агенту (@file, #chat, /cmd)...',
                        border: InputBorder.none,
                        enabledBorder: InputBorder.none,
                        focusedBorder: InputBorder.none,
                        fillColor: Colors.transparent,
                        contentPadding: EdgeInsets.symmetric(horizontal: 10, vertical: 12),
                      ),
                      onSubmitted: (_) => controller.sendMessage(),
                    ),
                  ),
                ),
              ],
            ),
          ),

          const SizedBox(height: 8),

          // Sub-bar Controls: Permission Mode Switcher + Model & Thought Level + Token Count + Send
          Row(
            children: [
              // Permission Mode Switcher (Shift+Tab)
              Obx(() {
                final mode = controller.permissionMode.value;
                return PopupMenuButton<PermissionMode>(
                  tooltip: 'Режим разрешений (${mode.description})',
                  onSelected: controller.setPermissionMode,
                  itemBuilder: (context) => PermissionMode.values.map((m) {
                    return PopupMenuItem(
                      value: m,
                      child: Row(
                        children: [
                          Icon(
                            m == mode ? Icons.radio_button_checked : Icons.radio_button_unchecked,
                            size: 14,
                            color: DesktopTheme.accentCyan,
                          ),
                          const SizedBox(width: 8),
                          Text(m.label, style: const TextStyle(fontSize: 12)),
                        ],
                      ),
                    );
                  }).toList(),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: DesktopTheme.bgCanvas,
                      borderRadius: BorderRadius.circular(5),
                      border: Border.all(color: DesktopTheme.borderSubtle),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const Icon(Icons.shield_outlined, size: 12, color: DesktopTheme.accentSky),
                        const SizedBox(width: 6),
                        Text(
                          mode.label,
                          style: TextStyle(fontSize: 11, color: DesktopTheme.textSecondary),
                        ),
                        const SizedBox(width: 4),
                        const Text(
                          '⇧⇥',
                          style: TextStyle(
                            fontSize: 10,
                            fontFamily: 'Consolas',
                            color: DesktopTheme.textMutedDark,
                          ),
                        ),
                      ],
                    ),
                  ),
                );
              }),

              const SizedBox(width: 8),

              // Model & Thought Level Dropdown
              Obx(() => Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: DesktopTheme.bgCanvas,
                      borderRadius: BorderRadius.circular(5),
                      border: Border.all(color: DesktopTheme.borderSubtle),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(
                          controller.activeModel.value,
                          style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: DesktopTheme.textSecondary),
                        ),
                        const SizedBox(width: 6),
                        PopupMenuButton<String>(
                          tooltip: 'Уровень рассуждений (Thought Level)',
                          onSelected: controller.setThoughtLevel,
                          itemBuilder: (context) => ['Low', 'High', 'Max'].map((lvl) {
                            return PopupMenuItem(value: lvl, child: Text(lvl));
                          }).toList(),
                          child: Container(
                            padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                            decoration: BoxDecoration(
                              color: DesktopTheme.bgSurfaceElevated,
                              borderRadius: BorderRadius.circular(3),
                            ),
                            child: Text(
                              controller.thoughtLevel.value,
                              style: const TextStyle(fontSize: 10, color: DesktopTheme.accentCyan),
                            ),
                          ),
                        ),
                      ],
                    ),
                  )),

              const Spacer(),

              // Token Count Meter
              Obx(() => Text(
                    controller.tokenCount.value,
                    style: TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.textMuted),
                  )),

              const SizedBox(width: 12),

              // Send / Abort Button
              Obx(() {
                return controller.isRunning.value
                    ? OutlinedButton.icon(
                        style: OutlinedButton.styleFrom(
                          foregroundColor: Colors.redAccent,
                          side: const BorderSide(color: Colors.redAccent),
                          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                        ),
                        icon: const Icon(Icons.stop, size: 14),
                        label: const Text('Stop', style: TextStyle(fontSize: 11)),
                        onPressed: controller.abortRun,
                      )
                    : ElevatedButton.icon(
                        style: ElevatedButton.styleFrom(
                          backgroundColor: DesktopTheme.accentCyan,
                          foregroundColor: Colors.black,
                          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
                          elevation: 0,
                        ),
                        icon: const Icon(Icons.arrow_upward_rounded, size: 14),
                        label: const Text('Run', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                        onPressed: controller.sendMessage,
                      );
              }),
            ],
          ),
        ],
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
            Text(
              'OmnesAgent ADE',
              style: TextStyle(
                fontSize: 22,
                fontWeight: FontWeight.bold,
                letterSpacing: 0.6,
                color: DesktopTheme.textPrimary,
              ),
            ),
            const SizedBox(height: 8),
            Text(
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
            style: TextStyle(
              fontSize: 11,
              color: DesktopTheme.textSecondary,
            ),
          ),
        ],
      ),
    );
  }
}
