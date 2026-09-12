// OmnesAgent ADE Task Workspace View matching authentic Windows ADE layout (Screenshot 2).
// Features Git Tools status card popover, bottom terminal toggle, multi-project breadcrumbs,
// git diff chips, and localized Russian assistant prompts.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';

import '../../theme/desktop_theme.dart';
import '../../utils/desktop_i18n.dart';
import '../../widgets/interactive_question_card.dart';
import 'task_workspace_controller.dart';

class DesktopTaskWorkspaceView extends StatefulWidget {
  final DesktopTaskWorkspaceController controller;
  final VoidCallback? onToggleTools;
  final VoidCallback? onToggleTerminal;
  final VoidCallback? onOpenSettings;
  final VoidCallback? onOpenCanvas;
  final void Function(String? text)? onBranchSideChat;
  final VoidCallback? onReviewChanges;
  final bool isToolsOpen;

  const DesktopTaskWorkspaceView({
    super.key,
    required this.controller,
    this.onToggleTools,
    this.onToggleTerminal,
    this.onOpenSettings,
    this.onOpenCanvas,
    this.onBranchSideChat,
    this.onReviewChanges,
    this.isToolsOpen = false,
  });

  @override
  State<DesktopTaskWorkspaceView> createState() => _DesktopTaskWorkspaceViewState();
}

class _DesktopTaskWorkspaceViewState extends State<DesktopTaskWorkspaceView> {
  bool isBannerDismissed = false;
  bool isGitToolsOpen = false;
  bool isTaskBarOpen = false;
  final TextEditingController _bottomTerminalInput = TextEditingController();
  final ScrollController _bottomTerminalScroll = ScrollController();

  @override
  void dispose() {
    _bottomTerminalInput.dispose();
    _bottomTerminalScroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Obx(() {
      final _ = DesktopI18n.currentLanguage.value;
      final hasMessages = widget.controller.messages.isNotEmpty;

      return Container(
        color: DesktopTheme.bgCanvas,
        child: Stack(
          children: [
            Column(
              children: [
                // 1. Top Navigation Bar
                _buildTopNavBar(hasMessages),

                // 1.1 Project & Branch Sub-Header with Git Changes (Shown if session is in project)
                _buildProjectContextBar(),

                // 2. Main Body: Either Empty State (New Task) or Chat Timeline
                Expanded(
                  child: hasMessages
                      ? _buildChatTimeline()
                      : _buildNewTaskWelcomeScreen(),
                ),

                // 3. Bottom Floating Composer when messages exist (staying ABOVE terminal!)
                if (hasMessages)
                  _buildBottomFloatingComposer(),

                // 4. Bottom Terminal (docked at the very bottom UNDER prompt!)
                if (hasMessages && widget.controller.isTerminalOpen.value)
                  _buildBottomTerminal(),
              ],
            ),

            // Backdrop dismiss for Git Tools and TaskBar
            if (isGitToolsOpen || isTaskBarOpen)
              Positioned.fill(
                child: GestureDetector(
                  behavior: HitTestBehavior.translucent,
                  onTap: () => setState(() {
                    isGitToolsOpen = false;
                    isTaskBarOpen = false;
                  }),
                ),
              ),

            // Floating Git Tools Popover Card (aligned under Changes badge)
            if (isGitToolsOpen)
              Positioned(
                top: 78,
                right: 124,
                width: 360,
                child: _buildGitToolsCard(),
              ),

            // Floating TaskBar Popover Card
            if (isTaskBarOpen)
              _buildTaskBarDropdown(),
          ],
        ),
      );
    });
  }

  // ==========================================
  // PROJECT CONTEXT SUB-HEADER UNDER TABS
  // ==========================================
  // ==========================================
  // GIT BRANCH PICKER DIALOG
  // ==========================================
  void _showBranchPicker(BuildContext context) {
    showDialog(
      context: context,
      builder: (ctx) {
        final branches = widget.controller.availableBranches;
        final curBranch = widget.controller.activeBranch.value ?? 'main';
        final newBranchCtrl = TextEditingController();

        return AlertDialog(
          backgroundColor: DesktopTheme.bgSurface,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
            side: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
          ),
          title: Row(
            children: const [
              Icon(FontAwesomeIcons.codeBranch, size: 14, color: Color(0xFF00D2FF)),
              SizedBox(width: 8),
              Text(
                'Ветки репозитория (Git)',
                style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: Colors.white),
              ),
            ],
          ),
          content: SizedBox(
            width: 380,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Выберите ветку для переключения:',
                  style: TextStyle(fontSize: 12, color: Color(0xFF94A3B8)),
                ),
                const SizedBox(height: 10),
                Container(
                  constraints: const BoxConstraints(maxHeight: 200),
                  decoration: BoxDecoration(
                    color: DesktopTheme.bgSurfaceElevated,
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: DesktopTheme.borderSubtle),
                  ),
                  child: ListView.separated(
                    shrinkWrap: true,
                    itemCount: branches.isNotEmpty ? branches.length : 1,
                    separatorBuilder: (_, __) => Divider(height: 1, color: DesktopTheme.borderSubtle),
                    itemBuilder: (ctx, i) {
                      final b = branches.isNotEmpty ? branches[i] : curBranch;
                      final isSelected = b == curBranch;
                      return InkWell(
                        onTap: () async {
                          Navigator.pop(ctx);
                          if (!isSelected) {
                            await widget.controller.switchGitBranch(b);
                          }
                        },
                        child: Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                          child: Row(
                            children: [
                              Icon(
                                FontAwesomeIcons.codeBranch,
                                size: 12,
                                color: isSelected ? const Color(0xFF00D2FF) : const Color(0xFF64748B),
                              ),
                              const SizedBox(width: 10),
                              Expanded(
                                child: Text(
                                  b,
                                  style: TextStyle(
                                    fontSize: 12,
                                    fontFamily: 'Consolas',
                                    fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                                    color: isSelected ? Colors.white : const Color(0xFFCBD5E1),
                                  ),
                                ),
                              ),
                              if (isSelected)
                                const Icon(Icons.check, size: 14, color: Color(0xFF00D2FF)),
                            ],
                          ),
                        ),
                      );
                    },
                  ),
                ),
                const SizedBox(height: 14),
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: newBranchCtrl,
                        style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Colors.white),
                        decoration: InputDecoration(
                          hintText: 'Имя новой ветки...',
                          hintStyle: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                          isDense: true,
                          contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                          fillColor: DesktopTheme.bgSurfaceElevated,
                          filled: true,
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(6),
                            borderSide: BorderSide(color: DesktopTheme.borderSubtle),
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(width: 8),
                    ElevatedButton(
                      onPressed: () async {
                        final name = newBranchCtrl.text.trim();
                        if (name.isNotEmpty) {
                          Navigator.pop(ctx);
                          await widget.controller.createNewGitBranch(name);
                        }
                      },
                      style: ElevatedButton.styleFrom(
                        backgroundColor: const Color(0xFF00D2FF).withOpacity(0.18),
                        foregroundColor: const Color(0xFF00D2FF),
                        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(6),
                          side: const BorderSide(color: Color(0xFF00D2FF), width: 0.8),
                        ),
                      ),
                      child: const Text('Создать', style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold)),
                    ),
                  ],
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(ctx),
              child: const Text('Закрыть', style: TextStyle(color: Color(0xFF94A3B8))),
            ),
          ],
        );
      },
    );
  }

  // ==========================================
  // DEDICATED TASKBAR DROPDOWN PANEL
  // ==========================================
  Widget _buildTaskBarDropdown() {
    return Positioned(
      top: 78,
      right: 14,
      width: 360,
      child: Material(
        color: Colors.transparent,
        child: Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            borderRadius: BorderRadius.circular(10),
            border: Border.all(color: DesktopTheme.borderSubtle, width: 1.0),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withOpacity(DesktopTheme.isDark ? 0.45 : 0.15),
                blurRadius: 16,
                offset: const Offset(0, 6),
              ),
            ],
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.playlist_add_check, size: 16, color: Color(0xFF00D2FF)),
                  const SizedBox(width: 8),
                  Text(
                    'Таск-бар сессии',
                    style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
                  ),
                  const Spacer(),
                  Obx(() {
                    final steps = widget.controller.runTimelineSteps;
                    final doneCount = steps.where((s) => s['status'] == 'done').length;
                    final totalCount = steps.isNotEmpty ? steps.length : (widget.controller.isRunning.value ? 4 : 0);
                    return Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: const Color(0xFF00D2FF).withOpacity(0.12),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        '$doneCount/$totalCount выполнено',
                        style: const TextStyle(fontSize: 10, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFF00D2FF)),
                      ),
                    );
                  }),
                  const SizedBox(width: 8),
                  InkWell(
                    onTap: () => setState(() => isTaskBarOpen = false),
                    child: Icon(Icons.close, size: 14, color: DesktopTheme.textMuted),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              Obx(() {
                final steps = widget.controller.runTimelineSteps;
                if (steps.isEmpty) {
                  return Container(
                    padding: const EdgeInsets.symmetric(vertical: 16, horizontal: 12),
                    alignment: Alignment.center,
                    child: Text(
                      'Шаги и задачи агента формируются динамически при запуске промпта',
                      textAlign: TextAlign.center,
                      style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                    ),
                  );
                }
                return ListView.separated(
                  shrinkWrap: true,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: steps.length,
                  separatorBuilder: (_, __) => const SizedBox(height: 6),
                  itemBuilder: (ctx, i) {
                    final s = steps[i];
                    final title = s['title']?.toString() ?? s['step']?.toString() ?? 'Шаг ${i + 1}';
                    final status = s['status']?.toString() ?? 'pending';
                    final isDone = status == 'done';
                    final isRunning = status == 'running';

                    return Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgSurfaceElevated,
                        borderRadius: BorderRadius.circular(6),
                        border: Border.all(
                          color: isRunning ? const Color(0xFF00D2FF) : DesktopTheme.borderSubtle,
                          width: 0.8,
                        ),
                      ),
                      child: Row(
                        children: [
                          if (isRunning)
                            const SizedBox(
                              width: 13,
                              height: 13,
                              child: CircularProgressIndicator(strokeWidth: 1.5, color: Color(0xFF00D2FF)),
                            )
                          else
                            Icon(
                              isDone ? Icons.check_circle : Icons.radio_button_unchecked,
                              size: 14,
                              color: isDone ? const Color(0xFF10B981) : const Color(0xFF64748B),
                            ),
                          const SizedBox(width: 9),
                          Expanded(
                            child: Text(
                              title,
                              style: TextStyle(
                                fontSize: 11.5,
                                color: isDone ? const Color(0xFFE2E8F0) : (isRunning ? Colors.white : const Color(0xFF94A3B8)),
                                fontWeight: isRunning ? FontWeight.w600 : FontWeight.normal,
                              ),
                            ),
                          ),
                        ],
                      ),
                    );
                  },
                );
              }),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildProjectContextBar() {
    return Obx(() {
      final proj = widget.controller.activeProject.value;
      if (proj == null || proj.isEmpty) {
        return const SizedBox.shrink();
      }

      final branch = widget.controller.activeBranch.value ?? 'main';
      final path = widget.controller.activeProjectPath.value;

      return Container(
        height: 34,
        padding: const EdgeInsets.symmetric(horizontal: 14),
        decoration: BoxDecoration(
          color: DesktopTheme.bgSurface,
          border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 0.8)),
        ),
        child: Row(
          children: [
            // Project Folder Icon (no emojis!)
            const Icon(Icons.folder_outlined, size: 14, color: Color(0xFF00D2FF)),
            const SizedBox(width: 6),
            Text(
              proj,
              style: TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w600,
                color: DesktopTheme.textPrimary,
              ),
            ),
            if (path != null && path.isNotEmpty) ...[
              const SizedBox(width: 6),
              Flexible(
                child: Text(
                  '— $path',
                  style: TextStyle(
                    fontSize: 11,
                    fontFamily: 'Consolas',
                    color: DesktopTheme.textMuted,
                  ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
            const SizedBox(width: 14),

            // Branch (Clickable to switch or create)
            InkWell(
              onTap: () => _showBranchPicker(context),
              borderRadius: BorderRadius.circular(4),
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(FontAwesomeIcons.codeBranch, size: 11, color: Color(0xFF00D2FF)),
                    const SizedBox(width: 4),
                    Text(
                      branch,
                      style: const TextStyle(
                        fontSize: 11,
                        fontFamily: 'Consolas',
                        fontWeight: FontWeight.w600,
                        color: Color(0xFF00D2FF),
                      ),
                    ),
                    const SizedBox(width: 4),
                    const Icon(Icons.keyboard_arrow_down, size: 12, color: Color(0xFF64748B)),
                  ],
                ),
              ),
            ),

            const Spacer(),

            // Right side: 1. Git Changes Badge
            if (widget.controller.hasGitRepo.value) ...[
              InkWell(
                onTap: () => setState(() {
                  isGitToolsOpen = !isGitToolsOpen;
                  if (isGitToolsOpen) isTaskBarOpen = false;
                }),
                borderRadius: BorderRadius.circular(6),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 3),
                  decoration: BoxDecoration(
                    color: isGitToolsOpen
                        ? const Color(0xFF00D2FF).withOpacity(0.18)
                        : DesktopTheme.bgSurfaceElevated,
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(
                      color: isGitToolsOpen ? const Color(0xFF00D2FF) : DesktopTheme.borderSubtle,
                      width: 0.8,
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.assignment_outlined,
                        size: 12,
                        color: isGitToolsOpen ? const Color(0xFF00D2FF) : DesktopTheme.textMuted,
                      ),
                      const SizedBox(width: 5),
                      Text(
                        '${DesktopI18n.changesLabel} ',
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                          color: isGitToolsOpen ? const Color(0xFF00D2FF) : DesktopTheme.textSecondary,
                        ),
                      ),
                      Text(
                        '+${widget.controller.gitAdditions.value}',
                        style: const TextStyle(
                          fontSize: 11,
                          fontFamily: 'Consolas',
                          fontWeight: FontWeight.bold,
                          color: Color(0xFF10B981),
                        ),
                      ),
                      const SizedBox(width: 4),
                      Text(
                        '-${widget.controller.gitDeletions.value}',
                        style: const TextStyle(
                          fontSize: 11,
                          fontFamily: 'Consolas',
                          fontWeight: FontWeight.bold,
                          color: Color(0xFFEF4444),
                        ),
                      ),
                      if (widget.controller.gitModifiedFiles.isNotEmpty) ...[
                        const SizedBox(width: 5),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                          decoration: BoxDecoration(
                            color: const Color(0xFF00D2FF).withOpacity(0.15),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            '${widget.controller.gitModifiedFiles.length} файл.',
                            style: const TextStyle(fontSize: 10, fontFamily: 'Consolas', color: Color(0xFF00D2FF), fontWeight: FontWeight.bold),
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 8),
            ],

            // Right side: 2. TaskBar Info Badge (placed to the right of Changes!)
            InkWell(
              onTap: () => setState(() {
                isTaskBarOpen = !isTaskBarOpen;
                if (isTaskBarOpen) isGitToolsOpen = false;
              }),
              borderRadius: BorderRadius.circular(6),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 3),
                decoration: BoxDecoration(
                  color: isTaskBarOpen
                      ? const Color(0xFF00D2FF).withOpacity(0.18)
                      : DesktopTheme.bgSurfaceElevated,
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(
                    color: isTaskBarOpen ? const Color(0xFF00D2FF) : DesktopTheme.borderSubtle,
                    width: 0.8,
                  ),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (widget.controller.isRunning.value) ...[
                      const SizedBox(
                        width: 10,
                        height: 10,
                        child: CircularProgressIndicator(strokeWidth: 1.5, color: Color(0xFF00D2FF)),
                      ),
                      const SizedBox(width: 5),
                    ] else ...[
                      Icon(
                        Icons.playlist_add_check,
                        size: 13,
                        color: isTaskBarOpen ? const Color(0xFF00D2FF) : DesktopTheme.textMuted,
                      ),
                      const SizedBox(width: 5),
                    ],
                    Text(
                      'Таск-бар ',
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                        color: isTaskBarOpen ? const Color(0xFF00D2FF) : DesktopTheme.textSecondary,
                      ),
                    ),
                    Obx(() {
                      final steps = widget.controller.runTimelineSteps;
                      final doneCount = steps.where((s) => s['status'] == 'done').length;
                      final totalCount = steps.isNotEmpty ? steps.length : (widget.controller.isRunning.value ? 4 : 0);
                      final label = totalCount > 0 ? '$doneCount/$totalCount' : '0/0';
                      return Text(
                        label,
                        style: const TextStyle(
                          fontSize: 11,
                          fontFamily: 'Consolas',
                          fontWeight: FontWeight.bold,
                          color: Color(0xFF00D2FF),
                        ),
                      );
                    }),
                    const SizedBox(width: 3),
                    Icon(
                      isTaskBarOpen ? Icons.keyboard_arrow_up : Icons.keyboard_arrow_down,
                      size: 12,
                      color: DesktopTheme.textMuted,
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      );
    });
  }

  // ==========================================
  // TOP NAV BAR MATCHING AUTHENTIC WINDOWS ADE
  // ==========================================
  Widget _buildTopNavBar(bool hasMessages) {
    final isNewTask = widget.controller.isNewTask.value;

    return Container(
      height: 44,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 0.8)),
      ),
      child: Row(
        children: [
          // Session Tabs / Title
          Expanded(
            child: Obx(() {
              final tabs = widget.controller.openSessionTabs;
              if (tabs.isEmpty || (isNewTask && !hasMessages)) {
                return Row(
                  children: [
                    Text(
                      DesktopI18n.newTask,
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w600,
                        color: DesktopTheme.textMuted,
                      ),
                    ),
                  ],
                );
              }

              return SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    for (final tabId in tabs) ...[
                      _buildSessionTab(tabId),
                      const SizedBox(width: 4),
                    ],
                    // Quick add new task tab button
                    InkWell(
                      onTap: () => widget.controller.createNewTask(),
                      borderRadius: BorderRadius.circular(4),
                      child: Container(
                        padding: const EdgeInsets.all(5),
                        child: Icon(Icons.add, size: 14, color: DesktopTheme.textMuted),
                      ),
                    ),
                  ],
                ),
              );
            }),
          ),

          const SizedBox(width: 8),



          // Terminal quick toggle (>_)
          Tooltip(
            message: DesktopI18n.terminalTooltip,
            child: InkWell(
              onTap: widget.controller.toggleTerminal,
              borderRadius: BorderRadius.circular(4),
              child: Container(
                padding: const EdgeInsets.all(5),
                decoration: BoxDecoration(
                  color: widget.controller.isTerminalOpen.value
                      ? const Color(0xFF00D2FF).withOpacity(0.18)
                      : DesktopTheme.bgSurfaceElevated,
                  borderRadius: BorderRadius.circular(5),
                  border: Border.all(
                    color: widget.controller.isTerminalOpen.value ? const Color(0xFF00D2FF) : Colors.transparent,
                    width: 0.8,
                  ),
                ),
                child: Icon(
                  FontAwesomeIcons.terminal,
                  size: 12,
                  color: widget.controller.isTerminalOpen.value ? const Color(0xFF00D2FF) : DesktopTheme.textMuted,
                ),
              ),
            ),
          ),
          const SizedBox(width: 8),

          // Side Pane / Inspector toggle ([|])
          Tooltip(
            message: DesktopI18n.toolsTooltip,
            child: InkWell(
              onTap: widget.onToggleTools,
              borderRadius: BorderRadius.circular(4),
              child: Container(
                padding: const EdgeInsets.all(5),
                decoration: BoxDecoration(
                  color: widget.isToolsOpen ? const Color(0xFF00D2FF).withOpacity(0.18) : DesktopTheme.bgSurfaceElevated,
                  borderRadius: BorderRadius.circular(5),
                  border: Border.all(
                    color: widget.isToolsOpen ? const Color(0xFF00D2FF) : Colors.transparent,
                    width: 0.8,
                  ),
                ),
                child: Icon(
                  Icons.view_sidebar_outlined,
                  size: 14,
                  color: widget.isToolsOpen ? const Color(0xFF00D2FF) : DesktopTheme.textMuted,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  // Helper to build open session tab
  Widget _buildSessionTab(String tabId) {
    final session = widget.controller.sessions[tabId];
    final isActive = widget.controller.activeSessionId.value == tabId;
    final title = session?.title ?? tabId;

    return InkWell(
      onTap: () {
        widget.controller.switchToSession(tabId);
      },
      borderRadius: BorderRadius.circular(6),
      child: Container(
        height: 28,
        constraints: const BoxConstraints(maxWidth: 190),
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: BoxDecoration(
          color: isActive ? DesktopTheme.bgSurfaceElevated : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: isActive ? DesktopTheme.borderSubtle : Colors.transparent,
            width: 0.8,
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              isActive ? Icons.chat_bubble : Icons.chat_bubble_outline,
              size: 11,
              color: isActive ? DesktopTheme.accentSky : DesktopTheme.textMuted,
            ),
            const SizedBox(width: 6),
            Flexible(
              child: Text(
                title,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: isActive ? FontWeight.w600 : FontWeight.normal,
                  color: isActive ? DesktopTheme.textPrimary : DesktopTheme.textMuted,
                ),
              ),
            ),
            const SizedBox(width: 6),
            InkWell(
              onTap: () => widget.controller.closeSessionTab(tabId),
              borderRadius: BorderRadius.circular(3),
              child: Padding(
                padding: const EdgeInsets.all(2),
                child: Icon(
                  Icons.close,
                  size: 11,
                  color: isActive ? DesktopTheme.textSecondary : DesktopTheme.textMuted,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ==========================================
  // GIT TOOLS POPOVER CARD (Screenshot 2 Match)
  // ==========================================
  Widget _buildGitToolsCard() {
    return Container(
      width: 360,
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: DesktopTheme.borderSubtle, width: 1.0),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(DesktopTheme.isDark ? 0.45 : 0.15),
            blurRadius: 16,
            offset: const Offset(0, 6),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header without three dots icon
          Row(
            children: [
              Text(
                DesktopI18n.gitTools,
                style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
              ),
              const Spacer(),
              InkWell(
                onTap: () => setState(() => isGitToolsOpen = false),
                child: Icon(Icons.close, size: 14, color: DesktopTheme.textMuted),
              ),
            ],
          ),
          const SizedBox(height: 12),

          // Active Group / Project Info Bar
          Obx(() {
            final proj = widget.controller.activeProject.value;
            final grp = widget.controller.activeGroup.value;
            final hasProj = proj != null && proj.isNotEmpty;
            final label = (hasProj ? proj : grp) ?? 'Разное';

            return Container(
              margin: const EdgeInsets.only(bottom: 12),
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: DesktopTheme.borderSubtle, width: 0.8),
              ),
              child: Row(
                children: [
                  Icon(
                    hasProj ? Icons.folder : Icons.grid_view,
                    size: 13,
                    color: const Color(0xFF00D2FF),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      label,
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                        color: DesktopTheme.textPrimary,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  if (hasProj) ...[
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgSurface,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        grp ?? '',
                        style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted),
                      ),
                    ),
                  ],
                ],
              ),
            );
          }),

          // Changes
          Row(
            children: [
              Icon(Icons.assignment_outlined, size: 14, color: DesktopTheme.textMuted),
              const SizedBox(width: 8),
              Text(DesktopI18n.tr('Изменения', 'Changes'), style: TextStyle(fontSize: 12, color: DesktopTheme.textSecondary)),
              const Spacer(),
              Obx(() => Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    '+${widget.controller.gitAdditions.value}',
                    style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFF10B981)),
                  ),
                  const SizedBox(width: 6),
                  Text(
                    '-${widget.controller.gitDeletions.value}',
                    style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFFEF4444)),
                  ),
                ],
              )),
            ],
          ),
          const SizedBox(height: 10),

          // Branch (Clickable to switch or create)
          InkWell(
            onTap: () => _showBranchPicker(context),
            borderRadius: BorderRadius.circular(6),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 3),
              child: Row(
                children: [
                  Icon(FontAwesomeIcons.codeBranch, size: 12, color: DesktopTheme.textMuted),
                  const SizedBox(width: 8),
                  Obx(() => Text(
                    widget.controller.activeBranch.value ?? 'main',
                    style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Color(0xFF00D2FF)),
                  )),
                  const SizedBox(width: 4),
                  Icon(Icons.keyboard_arrow_down, size: 14, color: DesktopTheme.textMuted),
                ],
              ),
            ),
          ),
          const SizedBox(height: 10),

          // Commit or push
          Row(
            children: [
              Icon(Icons.commit, size: 14, color: DesktopTheme.textMuted),
              const SizedBox(width: 8),
              Text(DesktopI18n.tr('Коммит или пуш', 'Commit or push'), style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted)),
            ],
          ),

          // Granular File List & Accept/Reject actions (Point 2)
          Obx(() {
            final files = widget.controller.gitModifiedFiles;
            if (files.isEmpty) {
              return const SizedBox.shrink();
            }
            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const SizedBox(height: 10),
                Divider(height: 1, color: DesktopTheme.borderSubtle),
                const SizedBox(height: 8),
                Row(
                  children: [
                    Text(
                      'Файлы (${files.length})',
                      style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
                    ),
                    const Spacer(),
                    InkWell(
                      onTap: () => widget.controller.acceptAllFiles(),
                      child: const Text('Принять все', style: TextStyle(fontSize: 10, color: Color(0xFF10B981), fontWeight: FontWeight.w600)),
                    ),
                    const SizedBox(width: 8),
                    InkWell(
                      onTap: () => widget.controller.rejectAllFiles(),
                      child: const Text('Отклонить все', style: TextStyle(fontSize: 10, color: Color(0xFFEF4444), fontWeight: FontWeight.w600)),
                    ),
                  ],
                ),
                const SizedBox(height: 6),
                ConstrainedBox(
                  constraints: const BoxConstraints(maxHeight: 140),
                  child: ListView.builder(
                    shrinkWrap: true,
                    itemCount: files.length,
                    itemBuilder: (context, idx) {
                      final file = files[idx];
                      final shortName = file.contains('/') ? file.split('/').last : (file.contains(r'\') ? file.split(r'\').last : file);
                      final isStaged = widget.controller.gitStagedFiles.contains(file);
                      return Padding(
                        padding: const EdgeInsets.symmetric(vertical: 2),
                        child: Row(
                          children: [
                            Icon(Icons.insert_drive_file_outlined, size: 12, color: DesktopTheme.textMuted),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Row(
                                children: [
                                  Flexible(
                                    child: Text(
                                      shortName,
                                      style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textSecondary),
                                      overflow: TextOverflow.ellipsis,
                                    ),
                                  ),
                                  if (isStaged) ...[
                                    const SizedBox(width: 4),
                                    Container(
                                      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                                      decoration: BoxDecoration(
                                        color: const Color(0xFF10B981).withOpacity(0.15),
                                        borderRadius: BorderRadius.circular(2),
                                      ),
                                      child: const Text('staged', style: TextStyle(fontSize: 9, color: Color(0xFF10B981), fontWeight: FontWeight.bold)),
                                    ),
                                  ],
                                ],
                              ),
                            ),
                            Tooltip(
                              message: isStaged ? 'Файл уже принят в индекс Git' : 'Принять файл (git add)',
                              child: InkWell(
                                onTap: () => widget.controller.acceptFile(file),
                                borderRadius: BorderRadius.circular(3),
                                child: Padding(
                                  padding: const EdgeInsets.all(2),
                                  child: Icon(
                                    isStaged ? Icons.check_circle : Icons.check,
                                    size: 14,
                                    color: const Color(0xFF10B981),
                                  ),
                                ),
                              ),
                            ),
                            const SizedBox(width: 4),
                            Tooltip(
                              message: 'Отклонить изменения (git restore/clean)',
                              child: InkWell(
                                onTap: () => widget.controller.rejectFile(file),
                                borderRadius: BorderRadius.circular(3),
                                child: const Padding(
                                  padding: EdgeInsets.all(2),
                                  child: Icon(Icons.close, size: 14, color: Color(0xFFEF4444)),
                                ),
                              ),
                            ),
                          ],
                        ),
                      );
                    },
                  ),
                ),
              ],
            );
          }),
        ],
      ),
    );
  }



  // ==========================================
  // BOTTOM INLINE TERMINAL
  // ==========================================
  Widget _buildBottomTerminal() {
    return Container(
      height: 220,
      margin: const EdgeInsets.fromLTRB(24, 0, 24, 10),
      decoration: BoxDecoration(
        color: DesktopTheme.isDark ? const Color(0xFF090D14) : const Color(0xFFF8FAFC),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: DesktopTheme.borderSubtle, width: 1),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.3),
            blurRadius: 10,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Container(
            height: 32,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurfaceElevated,
              borderRadius: const BorderRadius.vertical(top: Radius.circular(9)),
              border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
            ),
            child: Row(
              children: [
                const Icon(FontAwesomeIcons.terminal, size: 11, color: Color(0xFF00D2FF)),
                const SizedBox(width: 8),
                Text(
                  DesktopI18n.terminalConsole,
                  style: TextStyle(fontSize: 11, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
                ),
                const SizedBox(width: 10),
                Obx(() {
                  final cur = widget.controller.sessions[widget.controller.activeSessionId.value];
                  final path = cur?.projectPath ?? '';
                  if (path.isEmpty) return const SizedBox.shrink();
                  return ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 260),
                    child: Text(
                      path,
                      style: TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.textMuted),
                      overflow: TextOverflow.ellipsis,
                    ),
                  );
                }),
                const Spacer(),
                // Quick chips
                _buildTerminalChip('cargo check', () => _runBottomCmd('cargo check')),
                const SizedBox(width: 6),
                _buildTerminalChip('git status', () => _runBottomCmd('git status')),
                const SizedBox(width: 6),
                _buildTerminalChip('dir', () => _runBottomCmd('dir')),
                const SizedBox(width: 10),
                InkWell(
                  onTap: widget.controller.clearTerminal,
                  borderRadius: BorderRadius.circular(4),
                  child: Padding(
                    padding: const EdgeInsets.all(4),
                    child: Icon(FontAwesomeIcons.trashCan, size: 11, color: DesktopTheme.textMuted),
                  ),
                ),
                const SizedBox(width: 6),
                InkWell(
                  onTap: () => widget.controller.isTerminalOpen.value = false,
                  borderRadius: BorderRadius.circular(4),
                  child: Padding(
                    padding: const EdgeInsets.all(4),
                    child: Icon(Icons.close, size: 13, color: DesktopTheme.textMuted),
                  ),
                ),
              ],
            ),
          ),

          // Output Lines
          Expanded(
            child: Obx(() {
              final lines = widget.controller.terminalLines;
              return ListView.builder(
                controller: _bottomTerminalScroll,
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                itemCount: lines.length,
                itemBuilder: (context, idx) {
                  final line = lines[idx];
                  Color col = const Color(0xFFE2E8F0);
                  FontWeight fw = FontWeight.normal;
                  if (line.startsWith('>')) {
                    col = const Color(0xFF00D2FF);
                    fw = FontWeight.bold;
                  } else if (line.contains('[STDERR]') || line.contains('[ОШИБКА]') || line.contains('error')) {
                    col = const Color(0xFFF87171);
                  } else if (line.contains('Finished') || line.contains('passed') || line.contains('код 0')) {
                    col = const Color(0xFF34D399);
                  } else if (line.contains('Compiling') || line.contains('Analyzing')) {
                    col = const Color(0xFFFBBF24);
                  }
                  return Padding(
                    padding: const EdgeInsets.symmetric(vertical: 1),
                    child: SelectableText(
                      line,
                      style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: col, fontWeight: fw, height: 1.35),
                    ),
                  );
                },
              );
            }),
          ),

          // Quick stdin responses bar (Point 5)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurfaceElevated,
              border: Border(top: BorderSide(color: DesktopTheme.borderSubtle)),
            ),
            child: Row(
              children: [
                Text(
                  'Быстрый ввод (stdin):',
                  style: TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.textMuted),
                ),
                const SizedBox(width: 8),
                _buildTerminalQuickStdinChip('y', () => widget.controller.sendTerminalStdin('y')),
                const SizedBox(width: 6),
                _buildTerminalQuickStdinChip('n', () => widget.controller.sendTerminalStdin('n')),
                const SizedBox(width: 6),
                _buildTerminalQuickStdinChip('↵ Enter', () => widget.controller.sendTerminalStdin('')),
                const SizedBox(width: 6),
                _buildTerminalQuickStdinChip('✕ Ctrl+C', () => widget.controller.sendTerminalStdin('^C'), isDanger: true),
              ],
            ),
          ),

          // Input prompt
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: const Color(0xFF05070B),
              border: Border(top: BorderSide(color: DesktopTheme.borderSubtle)),
              borderRadius: const BorderRadius.vertical(bottom: Radius.circular(9)),
            ),
            child: Row(
              children: [
                const Text(
                  '> ',
                  style: TextStyle(fontSize: 13, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFF00D2FF)),
                ),
                Expanded(
                  child: TextField(
                    controller: _bottomTerminalInput,
                    style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Colors.white),
                    decoration: const InputDecoration(
                      hintText: 'Введите команду (например: cargo check, git status, dir)...',
                      hintStyle: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF64748B)),
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.symmetric(vertical: 4),
                    ),
                    onSubmitted: (val) => _runBottomCmd(val),
                  ),
                ),
                Obx(() {
                  if (widget.controller.isTerminalRunning.value) {
                    return const SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(strokeWidth: 2, color: Color(0xFF00D2FF)),
                    );
                  }
                  return InkWell(
                    onTap: () => _runBottomCmd(_bottomTerminalInput.text),
                    borderRadius: BorderRadius.circular(4),
                    child: const Padding(
                      padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      child: Icon(Icons.arrow_forward, size: 14, color: Color(0xFF00D2FF)),
                    ),
                  );
                }),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTerminalChip(String label, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
        decoration: BoxDecoration(
          color: DesktopTheme.bgCanvas,
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: DesktopTheme.borderSubtle),
        ),
        child: Text(
          label,
          style: const TextStyle(fontSize: 10, fontFamily: 'Consolas', color: Color(0xFF94A3B8)),
        ),
      ),
    );
  }

  void _runBottomCmd(String raw) {
    final cmd = raw.trim();
    if (cmd.isEmpty) return;
    _bottomTerminalInput.clear();
    widget.controller.executeTerminalCommand(cmd).then((_) {
      if (_bottomTerminalScroll.hasClients) {
        _bottomTerminalScroll.animateTo(
          _bottomTerminalScroll.position.maxScrollExtent + 60,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  // ==========================================
  // NEW TASK / WELCOME SCREEN
  // ==========================================
  Widget _buildNewTaskWelcomeScreen() {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 20),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 780),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const SizedBox(height: 10),
              Text(
                DesktopI18n.startNewTaskTitle,
                style: TextStyle(
                  fontSize: 28,
                  fontWeight: FontWeight.w400,
                  fontFamily: 'Georgia',
                  letterSpacing: 0.4,
                  color: DesktopTheme.textPrimary,
                ),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 32),

              // Central Composer Card
              _buildComposerCard(isHero: true),

              const SizedBox(height: 20),

              // Subscriber announcement banner
              if (!isBannerDismissed)
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                  decoration: BoxDecoration(
                    color: DesktopTheme.bgSurfaceElevated,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: DesktopTheme.borderSubtle),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.campaign_outlined, size: 16, color: DesktopTheme.textMuted),
                      const SizedBox(width: 10),
                      Expanded(
                        child: Text(
                          DesktopI18n.bannerText,
                          style: const TextStyle(
                            fontSize: 12,
                            color: Color(0xFF94A3B8),
                            height: 1.3,
                          ),
                        ),
                      ),
                      InkWell(
                        onTap: () => setState(() => isBannerDismissed = true),
                        child: const Icon(Icons.close, size: 15, color: Color(0xFF64748B)),
                      ),
                    ],
                  ),
                ),

              const SizedBox(height: 20),

              // 3 Quick Action Cards
              Row(
                children: [
                  Expanded(
                    child: _buildTemplateCard(
                      title: DesktopI18n.standupGitTitle,
                      desc: DesktopI18n.standupGitDesc,
                      onTap: () {
                        widget.controller.inputController.text = DesktopI18n.standupPrompt;
                      },
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _buildTemplateCard(
                      title: DesktopI18n.ciFailuresTitle,
                      desc: DesktopI18n.ciFailuresDesc,
                      onTap: () {
                        widget.controller.inputController.text = DesktopI18n.ciFailuresPrompt;
                      },
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _buildTemplateCard(
                      title: DesktopI18n.customizeTitle,
                      desc: DesktopI18n.customizeDesc,
                      onTap: () {
                        widget.controller.inputController.text = DesktopI18n.tr(
                          'Помоги настроить и кастомизировать конфигурацию текущего рабочего пространства под мой стек технологий.',
                          'Help configure and customize the current workspace settings for my tech stack.',
                        );
                      },
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildTemplateCard({
    required String title,
    required String desc,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        height: 110,
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: DesktopTheme.bgSurfaceElevated,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: DesktopTheme.borderSubtle),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.nightlight_round, size: 14, color: Color(0xFF00D2FF)),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    title,
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: FontWeight.bold,
                      color: DesktopTheme.textPrimary,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              desc,
              style: TextStyle(
                fontSize: 11,
                color: DesktopTheme.textMuted,
                height: 1.35,
              ),
              maxLines: 3,
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ),
      ),
    );
  }

  // ==========================================
  // COMPOSER CARD (Hero & Floating)
  // ==========================================
  Widget _buildComposerCard({bool isHero = false}) {
    return Container(
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: DesktopTheme.borderSubtle, width: 1.2),
        boxShadow: [
          BoxShadow(
            color: DesktopTheme.isDark ? Colors.black.withOpacity(0.35) : Colors.black.withOpacity(0.06),
            blurRadius: 24,
            offset: const Offset(0, 8),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 1. Top inside row: Project & Branch switchers (Hero mode)
          if (isHero && widget.controller.activeProject.value != null && widget.controller.activeProject.value!.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(left: 14, top: 12, right: 14),
              child: Row(
                children: [
                  _buildProjectChip(),
                  const SizedBox(width: 8),
                  _buildBranchChip(),
                ],
              ),
            ),

          // 2. Attachment chips if any
          Obx(() {
            if (widget.controller.attachments.isEmpty) return const SizedBox.shrink();
            return Padding(
              padding: const EdgeInsets.only(left: 14, top: 10, right: 14),
              child: Wrap(
                spacing: 6,
                runSpacing: 6,
                children: List.generate(widget.controller.attachments.length, (idx) {
                  final att = widget.controller.attachments[idx];
                  return Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                    decoration: BoxDecoration(
                      color: DesktopTheme.bgSurfaceElevated,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.4)),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          att.contains('screenshot') || att.contains('.png') || att.contains('.jpg')
                              ? Icons.image
                              : Icons.insert_drive_file,
                          size: 12,
                          color: att.contains('screenshot') || att.contains('.png') || att.contains('.jpg')
                              ? const Color(0xFF10B981)
                              : const Color(0xFF00D2FF),
                        ),
                        const SizedBox(width: 4),
                        Text(
                          att.contains(r'\') ? att.split(r'\').last : (att.contains('/') ? att.split('/').last : att),
                          style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textPrimary),
                        ),
                        const SizedBox(width: 4),
                        InkWell(
                          onTap: () => widget.controller.removeAttachment(idx),
                          child: Icon(Icons.close, size: 12, color: DesktopTheme.textMuted),
                        ),
                      ],
                    ),
                  );
                }),
              ),
            );
          }),

          // 3. Multi-line Input field
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
            child: TextField(
              controller: widget.controller.inputController,
              maxLines: isHero ? 4 : 3,
              minLines: isHero ? 2 : 1,
              style: TextStyle(fontSize: 14, color: DesktopTheme.textPrimary, height: 1.4),
              decoration: InputDecoration(
                hintText: isHero
                    ? DesktopI18n.heroInputHint
                    : DesktopI18n.promptPlaceholder,
                hintStyle: TextStyle(fontSize: 13, color: DesktopTheme.textMuted),
                border: InputBorder.none,
                enabledBorder: InputBorder.none,
                focusedBorder: InputBorder.none,
                isDense: true,
                contentPadding: EdgeInsets.zero,
              ),
              onSubmitted: (_) => widget.controller.sendMessage(),
            ),
          ),

          // 4. Bottom Controls Row: [+] [🛡️ Mode ⌵] ... [🟢 Model ⌵] [🧠 Max ⌵] [↑]
          Padding(
            padding: const EdgeInsets.only(left: 12, right: 12, bottom: 10),
            child: Row(
              children: [
                // [+] Add Context Button & Popup
                _buildAddMenuButton(),
                const SizedBox(width: 8),

                // [🛡️ Permission Mode ⌵]
                _buildPermissionModeMenuButton(),
                const SizedBox(width: 8),

                // [📋 Project Rules / AGENTS.md]
                _buildProjectRulesButton(),

                const Spacer(),

                // [📊 Context Gauge & Compact]
                _buildContextGaugeButton(),
                const SizedBox(width: 8),

                // [🏢 Provider ⌵]
                _buildProviderMenuButton(),
                const SizedBox(width: 8),

                // [🟢 Model ⌵]
                _buildModelMenuButton(),
                const SizedBox(width: 8),

                // [🧠 Thought Level ⌵]
                _buildThoughtLevelMenuButton(),
                const SizedBox(width: 8),

                // [🎙️ Voice Duplex Button]
                _buildVoiceDuplexButton(),
                const SizedBox(width: 8),

                // [↑ Send Button]
                _buildSendButton(),
              ],
            ),
          ),
        ],
      ),
    );
  }

  /// Sleek cyber loading indicator when agent is processing
  Widget _buildAgentGeneratingIndicator() {
    return Padding(
      padding: const EdgeInsets.only(bottom: 20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 22,
                height: 22,
                decoration: BoxDecoration(
                  color: const Color(0xFF00D2FF).withOpacity(0.2),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: const Center(
                  child: Icon(Icons.auto_awesome, size: 13, color: Color(0xFF00D2FF)),
                ),
              ),
              const SizedBox(width: 8),
              Text(
                'OmnesAgent',
                style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
              ),
              const SizedBox(width: 8),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                decoration: BoxDecoration(
                  color: DesktopTheme.bgSurfaceElevated,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Obx(() => Text(
                  widget.controller.activeModel.value,
                  style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted, fontFamily: 'Consolas'),
                )),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: DesktopTheme.borderSubtle),
            ),
            child: Row(
              children: [
                Container(
                  width: 20,
                  height: 20,
                  padding: const EdgeInsets.all(3),
                  child: const CircularProgressIndicator(
                    strokeWidth: 2,
                    valueColor: AlwaysStoppedAnimation<Color>(Color(0xFF00D2FF)),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Text(
                        'OmnesAgent формирует ответ...',
                        style: TextStyle(fontSize: 12.5, fontWeight: FontWeight.w600, color: Color(0xFFE2E8F0)),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        'Шлюз выполняет анализ контекста и формирует решение задачи',
                        style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBottomFloatingComposer() {
    return Container(
      padding: const EdgeInsets.fromLTRB(24, 0, 24, 16),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 820),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildApprovalsBanner(),
            _buildComposerCard(isHero: false),
          ],
        ),
      ),
    );
  }

  Widget _buildApprovalsBanner() {
    return Obx(() {
      final approvals = widget.controller.pendingApprovals;
      if (approvals.isEmpty) return const SizedBox.shrink();

      return Padding(
        padding: const EdgeInsets.only(bottom: 12),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: List.generate(approvals.length, (idx) {
            final app = approvals[idx];
            final tool = app['tool_name'] ?? 'tool';
            final summary = app['arguments_summary'] ?? '';
            final timeout = app['timeout_secs'];

            return Container(
              margin: const EdgeInsets.only(bottom: 8),
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: const Color(0xFF1E1B18),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: const Color(0xFFF59E0B).withOpacity(0.6), width: 1.2),
                boxShadow: [
                  BoxShadow(
                    color: const Color(0xFFF59E0B).withOpacity(0.12),
                    blurRadius: 12,
                    offset: const Offset(0, 4),
                  ),
                ],
              ),
              child: Row(
                children: [
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: const Color(0xFFF59E0B).withOpacity(0.2),
                      shape: BoxShape.circle,
                    ),
                    child: const Icon(Icons.security, size: 18, color: Color(0xFFF59E0B)),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Text(
                              DesktopI18n.tr('Требуется согласование действия:', 'Approval required:'),
                              style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: Color(0xFFF59E0B)),
                            ),
                            const SizedBox(width: 6),
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                              decoration: BoxDecoration(
                                color: DesktopTheme.bgSurfaceElevated,
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: Text(
                                tool.toString(),
                                style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.accentCyan, fontWeight: FontWeight.bold),
                              ),
                            ),
                            if (timeout != null) ...[
                              const Spacer(),
                              Text(
                                '${timeout}s',
                                style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                              ),
                            ],
                          ],
                        ),
                        if (summary.toString().isNotEmpty) ...[
                          const SizedBox(height: 4),
                          Text(
                            summary.toString(),
                            style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textPrimary),
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ],
                      ],
                    ),
                  ),
                  const SizedBox(width: 14),
                  OutlinedButton(
                    onPressed: () => widget.controller.denyAction(idx),
                    style: OutlinedButton.styleFrom(
                      foregroundColor: const Color(0xFFEF4444),
                      side: const BorderSide(color: Color(0xFFEF4444)),
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    ),
                    child: Text(DesktopI18n.tr('Запретить', 'Deny'), style: const TextStyle(fontSize: 12)),
                  ),
                  const SizedBox(width: 8),
                  ElevatedButton(
                    onPressed: () => widget.controller.approveAction(idx),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: const Color(0xFF10B981),
                      foregroundColor: Colors.black,
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                    ),
                    child: Text(DesktopI18n.tr('Разрешить', 'Approve'), style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                  ),
                ],
              ),
            );
          }),
        ),
      );
    });
  }

  // ==========================================
  // CHAT TIMELINE (Active Task Run)
  // ==========================================
  Widget _buildChatTimeline() {
    return ListView.builder(
      controller: widget.controller.scrollController,
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
      itemCount: widget.controller.messages.length + (widget.controller.isRunning.value && (widget.controller.messages.isEmpty || !widget.controller.messages.last.isStreaming) ? 1 : 0),
      itemBuilder: (context, index) {
        if (index >= widget.controller.messages.length) {
          return _buildAgentGeneratingIndicator();
        }
        final msg = widget.controller.messages[index];
        final isUser = msg.chatMessageType == ChatMessageType.user;

        return Padding(
          padding: const EdgeInsets.only(bottom: 20),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Author header
              Row(
                children: [
                  Container(
                    width: 22,
                    height: 22,
                    decoration: BoxDecoration(
                      color: isUser ? const Color(0xFF333842) : const Color(0xFF00D2FF).withOpacity(0.2),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Center(
                      child: Icon(
                        isUser ? Icons.person : Icons.auto_awesome,
                        size: 13,
                        color: isUser ? Colors.white : const Color(0xFF00D2FF),
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Text(
                    isUser ? 'You' : 'OmnesAgent',
                    style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
                  ),
                  if (!isUser) ...[
                    const SizedBox(width: 8),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgSurfaceElevated,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        widget.controller.activeModel.value,
                        style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted, fontFamily: 'Consolas'),
                      ),
                    ),
                  ],
                ],
              ),
              const SizedBox(height: 8),

              // Message Body Container
              Container(
                width: double.infinity,
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: isUser ? DesktopTheme.bgSurfaceElevated : DesktopTheme.bgSurface,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: DesktopTheme.borderSubtle),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Agent Real-time Action Steps (reading, editing, terminal, AST)
                    if (!isUser)
                      _buildAgentActionSteps(context, msg),

                    // Thinking accordion with auto-collapse
                    if (!isUser)
                      _buildThinkingAccordion(context, msg),

                    _buildMessageContent(context, msg),

                    // Error banner with retry button
                    if (msg.isError)
                      Container(
                        margin: const EdgeInsets.only(top: 10),
                        padding: const EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: const Color(0xFFEF4444).withOpacity(0.12),
                          borderRadius: BorderRadius.circular(6),
                          border: Border.all(color: const Color(0xFFEF4444).withOpacity(0.4)),
                        ),
                        child: Row(
                          children: [
                            const Icon(Icons.error_outline, size: 16, color: Color(0xFFEF4444)),
                            const SizedBox(width: 10),
                            const Expanded(
                              child: Text(
                                'Произошла ошибка при получении ответа от модели.',
                                style: TextStyle(fontSize: 11.5, color: Color(0xFFEF4444), fontWeight: FontWeight.w500),
                              ),
                            ),
                            ElevatedButton.icon(
                              onPressed: widget.controller.retryLastMessage,
                              icon: const Icon(Icons.refresh, size: 13),
                              label: const Text('Повторить', style: TextStyle(fontSize: 11)),
                              style: ElevatedButton.styleFrom(
                                backgroundColor: const Color(0xFFEF4444),
                                foregroundColor: Colors.white,
                                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                                minimumSize: Size.zero,
                                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                              ),
                            ),
                          ],
                        ),
                      ),

                    // Tool calls
                    if (msg.toolCalls.isNotEmpty) ...[
                      const SizedBox(height: 12),
                      ...msg.toolCalls.map((tool) => Container(
                            margin: const EdgeInsets.only(top: 6),
                            padding: const EdgeInsets.all(8),
                            decoration: BoxDecoration(
                              color: DesktopTheme.bgSurfaceElevated,
                              borderRadius: BorderRadius.circular(4),
                              border: Border.all(color: DesktopTheme.borderSubtle),
                            ),
                            child: Row(
                              children: [
                                const Icon(Icons.terminal, size: 13, color: Color(0xFF00D2FF)),
                                const SizedBox(width: 8),
                                Text(
                                  tool.name,
                                  style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textPrimary),
                                ),
                                const Spacer(),
                                Text(
                                  '✓ ${DesktopI18n.done}',
                                  style: const TextStyle(fontSize: 10, color: Color(0xFF10B981)),
                                ),
                              ],
                            ),
                          )),
                    ],

                    // Live Project Changes Review Pill (e.g. "4 files changed +129 -1 > [📄 Review]")
                    if (!isUser && msg.filesChangedCount != null && msg.filesChangedCount! > 0)
                      _buildReviewPill(
                        context,
                        files: msg.filesChangedCount!,
                        additions: msg.additions ?? 0,
                        deletions: msg.deletions ?? 0,
                      ),

                    // Message Reactions Row (Image 2 match: [⎘ Copy] [👍] [👎] [🔀 Branch] 9/5, 10:07 AM)
                    if (!isUser)
                      _buildMessageReactionsRow(context, msg),

                    // Follow-up Action Chips (1-click suggestions)
                    if (!isUser)
                      _buildFollowUpActionChips(context, msg),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildAgentActionSteps(BuildContext context, ChatMessage msg) {
    if (msg.steps.isEmpty) return const SizedBox.shrink();

    return StatefulBuilder(
      builder: (context, setStepState) {
        return Container(
          margin: const EdgeInsets.only(bottom: 10),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: msg.steps.map((step) {
              IconData iconData;
              Color iconColor;
              switch (step.type) {
                case AgentActionType.readingFile:
                  iconData = Icons.menu_book_outlined;
                  iconColor = const Color(0xFF38BDF8);
                  break;
                case AgentActionType.editingFile:
                  iconData = Icons.edit_note_outlined;
                  iconColor = const Color(0xFFFBBF24);
                  break;
                case AgentActionType.analyzingCode:
                  iconData = FontAwesomeIcons.brain;
                  iconColor = const Color(0xFFA78BFA);
                  break;
                case AgentActionType.runningTerminal:
                  iconData = Icons.terminal;
                  iconColor = const Color(0xFF34D399);
                  break;
                case AgentActionType.searchingFiles:
                  iconData = Icons.search;
                  iconColor = const Color(0xFF2DD4BF);
                  break;
              }

              return Container(
                margin: const EdgeInsets.only(bottom: 6),
                decoration: BoxDecoration(
                  color: const Color(0xFF161B22),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(
                    color: step.isRunning ? const Color(0xFF00D2FF).withOpacity(0.5) : DesktopTheme.borderSubtle,
                    width: 1,
                  ),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    InkWell(
                      onTap: () {
                        if (step.details != null && step.details!.isNotEmpty) {
                          setStepState(() => step.isExpanded = !step.isExpanded);
                        }
                      },
                      borderRadius: BorderRadius.circular(8),
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                        child: Row(
                          children: [
                            if (step.isRunning)
                              const SizedBox(
                                width: 14,
                                height: 14,
                                child: CircularProgressIndicator(strokeWidth: 2, color: Color(0xFF00D2FF)),
                              )
                            else
                              const Icon(Icons.check_circle, size: 14, color: Color(0xFF10B981)),
                            const SizedBox(width: 8),
                            Icon(iconData, size: 13, color: iconColor),
                            const SizedBox(width: 8),
                            Expanded(
                              child: Text(
                                step.title,
                                style: const TextStyle(
                                  fontSize: 12,
                                  fontWeight: FontWeight.w500,
                                  color: Color(0xFFE2E8F0),
                                  fontFamily: 'Consolas',
                                ),
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                            if (step.details != null && step.details!.isNotEmpty)
                              Icon(
                                step.isExpanded ? Icons.keyboard_arrow_up : Icons.keyboard_arrow_down,
                                size: 14,
                                color: const Color(0xFF94A3B8),
                              ),
                          ],
                        ),
                      ),
                    ),
                    if (step.isExpanded && step.details != null && step.details!.isNotEmpty)
                      Container(
                        width: double.infinity,
                        padding: const EdgeInsets.all(10),
                        margin: const EdgeInsets.fromLTRB(10, 0, 10, 8),
                        decoration: BoxDecoration(
                          color: const Color(0xFF0D1117),
                          borderRadius: BorderRadius.circular(6),
                          border: Border.all(color: const Color(0xFF30363D)),
                        ),
                        child: SelectableText(
                          step.details!,
                          style: const TextStyle(
                            fontSize: 11,
                            fontFamily: 'Consolas',
                            color: Color(0xFF8B949E),
                            height: 1.35,
                          ),
                        ),
                      ),
                  ],
                ),
              );
            }).toList(),
          ),
        );
      },
    );
  }

  Widget _buildReviewPill(BuildContext context, {required int files, required int additions, required int deletions}) {
    return Container(
      margin: const EdgeInsets.only(top: 10, bottom: 4),
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xFF16181D),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: const Color(0xFF282D37), width: 1),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.3),
            blurRadius: 6,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Row(
        mainAxisSize: MainAxisSize.max,
        children: [
          Text(
            '$files ${files == 1 ? "file" : "files"} changed ',
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w400,
              color: Color(0xFFE2E8F0),
            ),
          ),
          Text(
            '+$additions ',
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w500,
              color: Color(0xFF4ADE80),
              fontFamily: 'Consolas',
            ),
          ),
          Text(
            '-$deletions ',
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w500,
              color: Color(0xFFF87171),
              fontFamily: 'Consolas',
            ),
          ),
          const SizedBox(width: 2),
          const Icon(
            Icons.chevron_right,
            size: 14,
            color: Color(0xFF64748B),
          ),
          const Spacer(),
          InkWell(
            onTap: () {
              widget.onReviewChanges?.call();
            },
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
              decoration: BoxDecoration(
                color: const Color(0xFF222732),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: const Color(0xFF333A48)),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: const [
                  Icon(Icons.description_outlined, size: 13, color: Color(0xFFCBD5E1)),
                  SizedBox(width: 6),
                  Text(
                    'Review',
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                      color: Color(0xFFE2E8F0),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildThinkingAccordion(BuildContext context, ChatMessage msg) {
    if (msg.thinking == null || msg.thinking!.trim().isEmpty) {
      return const SizedBox.shrink();
    }

    final isStreamingThinking = msg.isStreaming && !msg.isThinkingFinished;
    final wordCount = msg.thinking!.trim().split(RegExp(r'\s+')).length;
    final seconds = msg.thinkingSeconds;

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InkWell(
            onTap: () {
              setState(() {
                msg.isThinkingExpanded = !msg.isThinkingExpanded;
              });
            },
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(
                  color: isStreamingThinking ? const Color(0xFF00D2FF).withOpacity(0.5) : DesktopTheme.borderSubtle,
                ),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (isStreamingThinking)
                    const SizedBox(
                      width: 12,
                      height: 12,
                      child: CircularProgressIndicator(strokeWidth: 2, color: Color(0xFF00D2FF)),
                    )
                  else
                    const Icon(Icons.psychology_outlined, size: 14, color: Color(0xFF00D2FF)),
                  const SizedBox(width: 7),
                  Text(
                    isStreamingThinking
                        ? 'Размышления (${seconds > 0 ? '$seconds с' : 'думает...'})'
                        : 'Размышления (${seconds > 0 ? 'рассуждал $seconds с · ' : ''}$wordCount слов)',
                    style: TextStyle(
                      fontSize: 11,
                      fontWeight: FontWeight.w500,
                      color: isStreamingThinking ? const Color(0xFF00D2FF) : DesktopTheme.textSecondary,
                    ),
                  ),
                  const SizedBox(width: 6),
                  Icon(
                    msg.isThinkingExpanded ? Icons.keyboard_arrow_up : Icons.keyboard_arrow_down,
                    size: 14,
                    color: DesktopTheme.textMuted,
                  ),
                ],
              ),
            ),
          ),
          if (msg.isThinkingExpanded)
            Container(
              margin: const EdgeInsets.only(top: 6),
              padding: const EdgeInsets.all(12),
              width: double.infinity,
              decoration: BoxDecoration(
                color: DesktopTheme.isDark ? const Color(0xFF141720) : const Color(0xFFF1F5F9),
                borderRadius: BorderRadius.circular(6),
                border: const Border(left: BorderSide(color: Color(0xFF00D2FF), width: 2.5)),
              ),
              child: SelectableText(
                msg.thinking!.trim(),
                style: TextStyle(
                  fontSize: 11.5,
                  fontFamily: 'Consolas',
                  height: 1.45,
                  color: DesktopTheme.isDark ? const Color(0xFF94A3B8) : const Color(0xFF475569),
                ),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildArtifactCard(
    BuildContext context, {
    required String lang,
    required String? filePath,
    required String codeBody,
  }) {
    final linesCount = codeBody.split('\n').length;
    final displayName = filePath ?? (lang == 'text' ? 'Текстовый файл' : 'Сниппет .$lang');

    IconData fileIcon = Icons.insert_drive_file_outlined;
    Color iconColor = const Color(0xFF00D2FF);

    final lower = (filePath ?? lang).toLowerCase();
    if (lower.contains('dart') || lower.endsWith('.dart')) {
      fileIcon = Icons.flutter_dash;
      iconColor = const Color(0xFF02569B);
    } else if (lower.contains('rust') || lower.endsWith('.rs')) {
      fileIcon = Icons.settings_suggest_outlined;
      iconColor = const Color(0xFFDEA584);
    } else if (lower.contains('py') || lower.endsWith('.py')) {
      fileIcon = Icons.terminal;
      iconColor = const Color(0xFF3776AB);
    } else if (lower.contains('json') || lower.endsWith('.json') || lower.contains('yaml') || lower.contains('toml')) {
      fileIcon = Icons.data_object;
      iconColor = const Color(0xFF10B981);
    } else if (lower.contains('md') || lower.endsWith('.md')) {
      fileIcon = Icons.article_outlined;
      iconColor = const Color(0xFF38BDF8);
    } else if (lower.contains('html') || lower.contains('css') || lower.contains('js') || lower.contains('ts')) {
      fileIcon = Icons.web;
      iconColor = const Color(0xFFF59E0B);
    }

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: DesktopTheme.isDark ? const Color(0xFF14171F) : const Color(0xFFF1F5F9),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
            decoration: BoxDecoration(
              color: DesktopTheme.isDark ? const Color(0xFF1B1E28) : const Color(0xFFE2E8F0),
              borderRadius: const BorderRadius.vertical(top: Radius.circular(7)),
            ),
            child: Row(
              children: [
                Icon(fileIcon, size: 15, color: iconColor),
                const SizedBox(width: 8),
                Expanded(
                  child: Row(
                    children: [
                      Flexible(
                        child: Text(
                          displayName,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            fontSize: 12,
                            fontFamily: 'Consolas',
                            fontWeight: FontWeight.bold,
                            color: DesktopTheme.textPrimary,
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                        decoration: BoxDecoration(
                          color: DesktopTheme.bgCanvas,
                          borderRadius: BorderRadius.circular(3),
                        ),
                        child: Text(
                          '$lang · $linesCount строк',
                          style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted, fontFamily: 'Consolas'),
                        ),
                      ),
                    ],
                  ),
                ),
                InkWell(
                  onTap: () {
                    widget.controller.openProjectFile(filePath ?? 'artifact.$lang', codeBody);
                    widget.onOpenCanvas?.call();
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(
                        content: Text('Файл "$displayName" открыт в Холсте'),
                        duration: const Duration(seconds: 2),
                      ),
                    );
                  },
                  borderRadius: BorderRadius.circular(4),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                    decoration: BoxDecoration(
                      color: const Color(0xFF00D2FF).withOpacity(0.12),
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.3)),
                    ),
                    child: const Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(Icons.view_quilt_outlined, size: 12, color: Color(0xFF00D2FF)),
                        SizedBox(width: 4),
                        Text(
                          'Открыть в Холсте',
                          style: TextStyle(fontSize: 10.5, fontWeight: FontWeight.bold, color: Color(0xFF00D2FF)),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Tooltip(
                  message: DesktopI18n.copy,
                  child: InkWell(
                    onTap: () {
                      Clipboard.setData(ClipboardData(text: codeBody));
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text(DesktopI18n.commandCopied), duration: const Duration(seconds: 1)),
                      );
                    },
                    borderRadius: BorderRadius.circular(4),
                    child: Padding(
                      padding: const EdgeInsets.all(3),
                      child: Icon(Icons.copy, size: 14, color: DesktopTheme.textMuted),
                    ),
                  ),
                ),
                if (['bash', 'sh', 'zsh', 'shell', 'powershell', 'ps1', 'cmd', 'bat', 'terminal', 'console'].contains(lang.toLowerCase())) ...[
                  const SizedBox(width: 6),
                  Tooltip(
                    message: 'Выполнить в терминале',
                    child: InkWell(
                      onTap: () {
                        widget.controller.isTerminalOpen.value = true;
                        widget.controller.executeTerminalCommand(codeBody.trim());
                        widget.onToggleTerminal?.call();
                      },
                      borderRadius: BorderRadius.circular(4),
                      child: const Padding(
                        padding: EdgeInsets.all(3),
                        child: Icon(Icons.play_arrow_outlined, size: 17, color: Color(0xFF00D2FF)),
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: SelectableText(
              codeBody,
              style: TextStyle(
                fontSize: 12,
                fontFamily: 'Consolas',
                height: 1.4,
                color: DesktopTheme.isDark ? const Color(0xFFE2E8F0) : const Color(0xFF0F172A),
              ),
            ),
          ),
        ],
      ),
    );
  }

  // ==========================================
  // FULL MARKDOWN RENDERER WITH INLINE FORMATTING
  // ==========================================
  Widget _buildFormattedMarkdown(BuildContext context, String rawText) {
    final lines = rawText.split('\n');
    final widgets = <Widget>[];

    for (int i = 0; i < lines.length; i++) {
      final line = lines[i];
      final trimmed = line.trim();

      if (trimmed.isEmpty) {
        widgets.add(const SizedBox(height: 6));
        continue;
      }

      // Markdown Table Detection (GFM Tables: | Header | Header | followed by |---|---|)
      if ((trimmed.startsWith('|') || (trimmed.contains('|') && !trimmed.startsWith('#'))) && i + 1 < lines.length) {
        final nextTrimmed = lines[i + 1].trim();
        if (RegExp(r'^\s*\|?\s*:?-{2,}:?\s*(\|\s*:?-{2,}:?\s*)+\|?\s*$').hasMatch(nextTrimmed)) {
          final tableLines = <String>[line, lines[i + 1]];
          i += 2;
          while (i < lines.length) {
            final rowTrim = lines[i].trim();
            if (rowTrim.contains('|') && !rowTrim.startsWith('#') && rowTrim.isNotEmpty) {
              tableLines.add(lines[i]);
              i++;
            } else {
              break;
            }
          }
          i--; // Step back so loop increment aligns
          widgets.add(_buildMarkdownTable(tableLines));
          continue;
        }
      }

      // H1: # Title
      if (line.startsWith('# ')) {
        widgets.add(Padding(
          padding: const EdgeInsets.only(top: 14, bottom: 6),
          child: SelectableText(
            line.substring(2).trim(),
            style: const TextStyle(fontSize: 17, fontWeight: FontWeight.bold, color: Colors.white, height: 1.3),
          ),
        ));
      }
      // H2: ## Title
      else if (line.startsWith('## ')) {
        widgets.add(Padding(
          padding: const EdgeInsets.only(top: 12, bottom: 5),
          child: Row(
            children: [
              Container(
                width: 3,
                height: 14,
                margin: const EdgeInsets.only(right: 7),
                decoration: BoxDecoration(color: const Color(0xFF00D2FF), borderRadius: BorderRadius.circular(2)),
              ),
              Expanded(
                child: SelectableText(
                  line.substring(3).trim(),
                  style: const TextStyle(fontSize: 14.5, fontWeight: FontWeight.bold, color: Color(0xFF38BDF8), height: 1.3),
                ),
              ),
            ],
          ),
        ));
      }
      // H3: ### Title
      else if (line.startsWith('### ')) {
        widgets.add(Padding(
          padding: const EdgeInsets.only(top: 10, bottom: 4),
          child: SelectableText(
            line.substring(4).trim(),
            style: const TextStyle(fontSize: 13.5, fontWeight: FontWeight.w600, color: Color(0xFFE2E8F0), height: 1.3),
          ),
        ));
      }
      // H4: #### Title
      else if (line.startsWith('#### ')) {
        widgets.add(Padding(
          padding: const EdgeInsets.only(top: 8, bottom: 3),
          child: SelectableText(
            line.substring(5).trim(),
            style: const TextStyle(fontSize: 12.5, fontWeight: FontWeight.w600, color: Color(0xFF94A3B8), height: 1.3),
          ),
        ));
      }
      // Horizontal Rule: ---, ***, ___
      else if (RegExp(r'^[-*_]{3,}$').hasMatch(trimmed)) {
        widgets.add(Divider(height: 16, thickness: 0.8, color: DesktopTheme.borderSubtle));
      }
      // Blockquote: > text
      else if (line.startsWith('> ')) {
        widgets.add(Container(
          margin: const EdgeInsets.symmetric(vertical: 4),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: const Color(0xFF16181D),
            borderRadius: BorderRadius.circular(4),
            border: const Border(left: BorderSide(color: Color(0xFF00D2FF), width: 3)),
          ),
          child: _buildRichInlineText(line.substring(2).trim(), isItalic: true),
        ));
      }
      // Bullet list item: - item or * item
      else if (RegExp(r'^\s*[-*+]\s+').hasMatch(line)) {
        final itemText = line.replaceFirst(RegExp(r'^\s*[-*+]\s+'), '');
        widgets.add(Padding(
          padding: const EdgeInsets.only(left: 4, top: 2, bottom: 2),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                margin: const EdgeInsets.only(top: 7, right: 8),
                width: 4.5,
                height: 4.5,
                decoration: const BoxDecoration(color: Color(0xFF00D2FF), shape: BoxShape.circle),
              ),
              Expanded(child: _buildRichInlineText(itemText)),
            ],
          ),
        ));
      }
      // Numbered list item: 1. item
      else if (RegExp(r'^\s*\d+\.\s+').hasMatch(line)) {
        final match = RegExp(r'^\s*(\d+\.)\s+(.*)$').firstMatch(line);
        final numPrefix = match?.group(1) ?? '• ';
        final itemText = match?.group(2) ?? line;
        widgets.add(Padding(
          padding: const EdgeInsets.only(left: 4, top: 2, bottom: 2),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                numPrefix,
                style: const TextStyle(fontSize: 12.5, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFF00D2FF)),
              ),
              const SizedBox(width: 6),
              Expanded(child: _buildRichInlineText(itemText)),
            ],
          ),
        ));
      }
      // Regular paragraph
      else {
        widgets.add(Padding(
          padding: const EdgeInsets.symmetric(vertical: 2),
          child: _buildRichInlineText(line),
        ));
      }
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: widgets,
    );
  }

  /// Helper parser for a table row into cells
  List<String> _parseTableRow(String line) {
    String trimmed = line.trim();
    if (trimmed.startsWith('|')) trimmed = trimmed.substring(1);
    if (trimmed.endsWith('|')) trimmed = trimmed.substring(0, trimmed.length - 1);
    return trimmed.split('|').map((c) => c.trim()).toList();
  }

  /// Helper parser for table column alignments (:---, :---:, ---:)
  List<Alignment> _parseTableAlignments(String sepLine, int colCount) {
    final cells = _parseTableRow(sepLine);
    final alignments = <Alignment>[];
    for (int i = 0; i < colCount; i++) {
      if (i < cells.length) {
        final c = cells[i];
        final left = c.startsWith(':');
        final right = c.endsWith(':');
        if (left && right) {
          alignments.add(Alignment.center);
        } else if (right) {
          alignments.add(Alignment.centerRight);
        } else {
          alignments.add(Alignment.centerLeft);
        }
      } else {
        alignments.add(Alignment.centerLeft);
      }
    }
    return alignments;
  }

  /// Builds a cyber-styled, horizontally scrollable Markdown table
  Widget _buildMarkdownTable(List<String> tableLines) {
    if (tableLines.length < 2) return const SizedBox.shrink();

    final headerCells = _parseTableRow(tableLines[0]);
    if (headerCells.isEmpty) return const SizedBox.shrink();

    final alignments = _parseTableAlignments(tableLines[1], headerCells.length);

    final dataRows = <List<String>>[];
    for (int r = 2; r < tableLines.length; r++) {
      final cells = _parseTableRow(tableLines[r]);
      if (cells.isNotEmpty) {
        dataRows.add(cells);
      }
    }

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: DesktopTheme.borderSubtle, width: 1.0),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(DesktopTheme.isDark ? 0.25 : 0.05),
            blurRadius: 8,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      clipBehavior: Clip.antiAlias,
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Table(
          defaultColumnWidth: const IntrinsicColumnWidth(),
          children: [
            // Header Row
            TableRow(
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1.0)),
              ),
              children: [
                for (int c = 0; c < headerCells.length; c++)
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                    alignment: alignments[c],
                    child: Text(
                      headerCells[c],
                      style: TextStyle(
                        fontSize: 12,
                        fontWeight: FontWeight.w600,
                        color: DesktopTheme.textPrimary,
                      ),
                    ),
                  ),
              ],
            ),
            // Data Rows (with zebra striping)
            for (int r = 0; r < dataRows.length; r++)
              TableRow(
                decoration: BoxDecoration(
                  color: r % 2 == 1
                      ? DesktopTheme.bgSurfaceElevated.withOpacity(0.35)
                      : Colors.transparent,
                  border: Border(
                    bottom: r < dataRows.length - 1
                        ? BorderSide(color: DesktopTheme.borderSubtle.withOpacity(0.5), width: 0.8)
                        : BorderSide.none,
                  ),
                ),
                children: [
                  for (int c = 0; c < headerCells.length; c++)
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 7),
                      alignment: c < alignments.length ? alignments[c] : Alignment.centerLeft,
                      child: _buildRichInlineText(
                        c < dataRows[r].length ? dataRows[r][c] : '',
                      ),
                    ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  /// Parses inline markdown (bold, italic, code, link, strikethrough)
  Widget _buildRichInlineText(String text, {bool isItalic = false}) {
    final spans = <InlineSpan>[];
    final reg = RegExp(r'(\*\*[^*]+\*\*|\*[^*]+\*|`[^`]+`|~~[^~]+~~|\[[^\]]+\]\([^)]+\))');
    int lastIndex = 0;

    for (final match in reg.allMatches(text)) {
      if (match.start > lastIndex) {
        spans.add(TextSpan(
          text: text.substring(lastIndex, match.start),
          style: TextStyle(
            fontSize: 13,
            height: 1.5,
            fontStyle: isItalic ? FontStyle.italic : FontStyle.normal,
            color: DesktopTheme.textPrimary,
          ),
        ));
      }

      final matchedStr = match.group(0)!;
      // Bold: **text**
      if (matchedStr.startsWith('**') && matchedStr.endsWith('**')) {
        final inner = matchedStr.substring(2, matchedStr.length - 2);
        spans.add(TextSpan(
          text: inner,
          style: const TextStyle(fontSize: 13, height: 1.5, fontWeight: FontWeight.bold, color: Colors.white),
        ));
      }
      // Italic: *text*
      else if (matchedStr.startsWith('*') && matchedStr.endsWith('*')) {
        final inner = matchedStr.substring(1, matchedStr.length - 1);
        spans.add(TextSpan(
          text: inner,
          style: const TextStyle(fontSize: 13, height: 1.5, fontStyle: FontStyle.italic, color: Color(0xFFE2E8F0)),
        ));
      }
      // Inline Code: `code`
      else if (matchedStr.startsWith('`') && matchedStr.endsWith('`')) {
        final inner = matchedStr.substring(1, matchedStr.length - 1);
        spans.add(WidgetSpan(
          alignment: PlaceholderAlignment.middle,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
            margin: const EdgeInsets.symmetric(horizontal: 2),
            decoration: BoxDecoration(
              color: const Color(0xFF1E222B),
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: const Color(0xFF2D3748), width: 0.8),
            ),
            child: Text(
              inner,
              style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Color(0xFF38BDF8)),
            ),
          ),
        ));
      }
      // Strikethrough: ~~text~~
      else if (matchedStr.startsWith('~~') && matchedStr.endsWith('~~')) {
        final inner = matchedStr.substring(2, matchedStr.length - 2);
        spans.add(TextSpan(
          text: inner,
          style: const TextStyle(fontSize: 13, height: 1.5, decoration: TextDecoration.lineThrough, color: Color(0xFF64748B)),
        ));
      }
      // Link: [title](url)
      else if (matchedStr.startsWith('[') && matchedStr.contains('](')) {
        final title = matchedStr.substring(1, matchedStr.indexOf(']('));
        spans.add(TextSpan(
          text: title,
          style: const TextStyle(fontSize: 13, height: 1.5, color: Color(0xFF00D2FF), decoration: TextDecoration.underline),
        ));
      }

      lastIndex = match.end;
    }

    if (lastIndex < text.length) {
      spans.add(TextSpan(
        text: text.substring(lastIndex),
        style: TextStyle(
          fontSize: 13,
          height: 1.5,
          fontStyle: isItalic ? FontStyle.italic : FontStyle.normal,
          color: DesktopTheme.textPrimary,
        ),
      ));
    }

    return SelectableText.rich(
      TextSpan(children: spans),
    );
  }

  Widget _buildMessageContent(BuildContext context, ChatMessage msg) {
    // 1. Check for XML style <question>...</question>
    final xmlMatch = RegExp(r'<question>([\s\S]*?)<\/question>').firstMatch(msg.text);
    if (xmlMatch != null) {
      final qData = InteractiveQuestionData.tryParse(xmlMatch.group(1)!);
      if (qData != null) {
        final prefix = msg.text.substring(0, xmlMatch.start).trim();
        final suffix = msg.text.substring(xmlMatch.end).trim();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (prefix.isNotEmpty) _buildFormattedMarkdown(context, prefix),
            InteractiveQuestionCard(
              data: qData,
              initialAnswer: msg.selectedQuestionAnswer,
              onAnswer: (ans) => widget.controller.submitQuestionAnswer(msg, ans),
            ),
            if (suffix.isNotEmpty) _buildFormattedMarkdown(context, suffix),
          ],
        );
      }
    }

    if (!msg.text.contains('```')) {
      return _buildFormattedMarkdown(context, msg.text);
    }

    final parts = msg.text.split('```');
    final widgets = <Widget>[];

    for (int i = 0; i < parts.length; i++) {
      final part = parts[i];
      if (i % 2 == 0) {
        if (part.trim().isNotEmpty) {
          widgets.add(_buildFormattedMarkdown(context, part.trim()));
        }
      } else {
        final lines = part.split('\n');
        final firstLine = lines.isNotEmpty ? lines.first.trim() : '';
        String? filePath;
        String lang = 'text';

        // Check if this block is an interactive question
        if (firstLine.toLowerCase().startsWith('question')) {
          final codeBody = lines.length > 1 ? lines.sublist(1).join('\n').trim() : '';
          final qData = InteractiveQuestionData.tryParse(codeBody);
          if (qData != null) {
            widgets.add(InteractiveQuestionCard(
              data: qData,
              initialAnswer: msg.selectedQuestionAnswer,
              onAnswer: (ans) => widget.controller.submitQuestionAnswer(msg, ans),
            ));
            continue;
          }
        }

        if (firstLine.contains('title="')) {
          final match = RegExp(r'title="([^"]+)"').firstMatch(firstLine);
          filePath = match?.group(1);
          lang = firstLine.split(' ').first;
        } else if (firstLine.contains(':')) {
          final p = firstLine.split(':');
          lang = p[0].trim();
          filePath = p.sublist(1).join(':').trim();
        } else {
          lang = firstLine.isNotEmpty ? firstLine : 'text';
          if (lines.length > 1) {
            final secondLine = lines[1].trim();
            if (secondLine.startsWith('// file:') || secondLine.startsWith('// path:') || secondLine.startsWith('# file:')) {
              filePath = secondLine.split(':').last.trim();
            } else if (secondLine.startsWith('// ') && secondLine.contains('.')) {
              final candidate = secondLine.substring(3).trim();
              if (!candidate.contains(' ') && candidate.length < 60) {
                filePath = candidate;
              }
            }
          }
        }

        final codeBody = lines.length > 1 ? lines.sublist(1).join('\n').trim() : part.trim();
        widgets.add(_buildArtifactCard(context, lang: lang, filePath: filePath, codeBody: codeBody));
      }
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: widgets,
    );
  }

  Widget _buildMessageReactionsRow(BuildContext context, ChatMessage msg) {
    bool isLiked = false;
    bool isDisliked = false;

    return StatefulBuilder(
      builder: (context, setReactionState) {

        return Padding(
          padding: const EdgeInsets.only(top: 8),
          child: Row(
            children: [
              InkWell(
                onTap: () {
                  Clipboard.setData(ClipboardData(text: msg.text));
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text(DesktopI18n.copiedToClipboard),
                      duration: const Duration(seconds: 1),
                      behavior: SnackBarBehavior.floating,
                    ),
                  );
                },
                borderRadius: BorderRadius.circular(4),
                child: const Padding(
                  padding: EdgeInsets.all(4),
                  child: Icon(Icons.copy, size: 14, color: Color(0xFF94A3B8)),
                ),
              ),
              const SizedBox(width: 4),

              InkWell(
                onTap: () {
                  setReactionState(() {
                    isLiked = !isLiked;
                    if (isLiked) isDisliked = false;
                  });
                },
                borderRadius: BorderRadius.circular(4),
                child: Padding(
                  padding: const EdgeInsets.all(4),
                  child: Icon(
                    isLiked ? Icons.thumb_up : Icons.thumb_up_alt_outlined,
                    size: 14,
                    color: isLiked ? const Color(0xFF00D2FF) : const Color(0xFF94A3B8),
                  ),
                ),
              ),
              const SizedBox(width: 4),

              InkWell(
                onTap: () {
                  setReactionState(() {
                    isDisliked = !isDisliked;
                    if (isDisliked) isLiked = false;
                  });
                },
                borderRadius: BorderRadius.circular(4),
                child: Padding(
                  padding: const EdgeInsets.all(4),
                  child: Icon(
                    isDisliked ? Icons.thumb_down : Icons.thumb_down_alt_outlined,
                    size: 14,
                    color: isDisliked ? const Color(0xFFEF4444) : const Color(0xFF94A3B8),
                  ),
                ),
              ),
              const SizedBox(width: 4),

              InkWell(
                onTap: () {
                  widget.onBranchSideChat?.call(msg.text);
                },
                borderRadius: BorderRadius.circular(4),
                child: const Tooltip(
                  message: 'Открыть боковой диалог (Side Chat)',
                  child: Padding(
                    padding: EdgeInsets.all(4),
                    child: Icon(FontAwesomeIcons.codeBranch, size: 11, color: Color(0xFF94A3B8)),
                  ),
                ),
              ),
              const SizedBox(width: 8),

              if (msg.checkpointHash != null || (msg.filesChangedCount != null && msg.filesChangedCount! > 0)) ...[
                const SizedBox(width: 8),
                Tooltip(
                  message: 'Откатить проект к этой контрольной точке (Rewind)',
                  child: InkWell(
                    onTap: () => _confirmRevertToCheckpoint(context, msg),
                    borderRadius: BorderRadius.circular(4),
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: const Color(0xFFF59E0B).withOpacity(0.12),
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: const Color(0xFFF59E0B).withOpacity(0.3), width: 0.8),
                      ),
                      child: const Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(Icons.history, size: 12, color: Color(0xFFF59E0B)),
                          SizedBox(width: 3),
                          Text(
                            'Откатить',
                            style: TextStyle(fontSize: 10, color: Color(0xFFF59E0B), fontWeight: FontWeight.bold),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ],

              const Text(
                '9/5, 10:07 AM',
                style: TextStyle(fontSize: 11, color: Color(0xFF64748B), fontFamily: 'Consolas'),
              ),
            ],
          ),
        );
      },
    );
  }

  /// Follow-up Action Chips (Point 4: 1-click suggestions)
  Widget _buildFollowUpActionChips(BuildContext context, ChatMessage msg) {
    if (msg.suggestedActions.isEmpty) return const SizedBox.shrink();

    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Wrap(
        spacing: 6,
        runSpacing: 6,
        children: msg.suggestedActions.map((action) {
          return InkWell(
            onTap: () => widget.controller.executeFollowUpAction(action),
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.35), width: 0.8),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.bolt, size: 12, color: Color(0xFF00D2FF)),
                  const SizedBox(width: 5),
                  Text(
                    action,
                    style: TextStyle(fontSize: 11, fontWeight: FontWeight.w500, color: DesktopTheme.textPrimary),
                  ),
                ],
              ),
            ),
          );
        }).toList(),
      ),
    );
  }

  /// Confirmation dialog before rolling back project files to checkpoint
  void _confirmRevertToCheckpoint(BuildContext context, ChatMessage msg) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: const Row(
          children: [
            Icon(Icons.history, size: 20, color: Color(0xFFF59E0B)),
            SizedBox(width: 8),
            Text('Откатить проект?', style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold)),
          ],
        ),
        content: const Text(
          'Вы действительно хотите откатить файлы проекта к состоянию до выполнения этого шага? Текущие несохраненные изменения в рабочей директории будут сброшены.',
          style: TextStyle(fontSize: 12),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text('Отмена', style: TextStyle(color: DesktopTheme.textMuted)),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.of(ctx).pop();
              widget.controller.revertToCheckpoint(msg);
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFFF59E0B),
              foregroundColor: Colors.black,
            ),
            child: const Text('Откатить', style: TextStyle(fontWeight: FontWeight.bold)),
          ),
        ],
      ),
    );
  }

  /// Context Gauge & Auto-Compact Button (Point 3)
  Widget _buildContextGaugeButton() {
    return Obx(() {
      final used = widget.controller.usedTokens.value;
      final max = widget.controller.maxTokens.value;
      final cost = widget.controller.estimatedCost.value;
      final ratio = max > 0 ? (used / max).clamp(0.0, 1.0) : 0.0;
      final pct = (ratio * 100).round();
      final isHigh = ratio > 0.75;
      final isMedium = ratio > 0.5;

      Color badgeColor = const Color(0xFF10B981);
      if (isHigh) {
        badgeColor = const Color(0xFFEF4444);
      } else if (isMedium) {
        badgeColor = const Color(0xFFF59E0B);
      }

      return Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Tooltip(
            message: 'Контекст: $used / $max токенов ($pct%)\nОриентировочная стоимость: \$${cost.toStringAsFixed(4)}',
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: badgeColor.withOpacity(0.4), width: 0.8),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Container(
                    width: 6,
                    height: 6,
                    decoration: BoxDecoration(color: badgeColor, shape: BoxShape.circle),
                  ),
                  const SizedBox(width: 5),
                  Text(
                    '${(used / 1000).toStringAsFixed(1)}k / ${(max / 1000).toStringAsFixed(0)}k ($pct%)',
                    style: TextStyle(
                      fontSize: 10.5,
                      fontFamily: 'Consolas',
                      color: DesktopTheme.textSecondary,
                    ),
                  ),
                ],
              ),
            ),
          ),
          if (isHigh || widget.controller.messages.length > 4) ...[
            const SizedBox(width: 6),
            Tooltip(
              message: 'Оптимизировать контекст сессии (Compact)',
              child: InkWell(
                onTap: () => widget.controller.compactContext(),
                borderRadius: BorderRadius.circular(6),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
                  decoration: BoxDecoration(
                    color: const Color(0xFF00D2FF).withOpacity(0.12),
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.4), width: 0.8),
                  ),
                  child: const Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.bolt, size: 12, color: Color(0xFF00D2FF)),
                      SizedBox(width: 3),
                      Text('Сжать', style: TextStyle(fontSize: 10, color: Color(0xFF00D2FF), fontWeight: FontWeight.bold)),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ],
      );
    });
  }

  /// Project Rules Inspector Button (Point 6)
  Widget _buildProjectRulesButton() {
    return Tooltip(
      message: 'Правила проекта (AGENTS.md)',
      child: InkWell(
        onTap: () => _showProjectRulesDialog(context),
        borderRadius: BorderRadius.circular(6),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurfaceElevated,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: DesktopTheme.borderSubtle, width: 0.8),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.rule_folder_outlined, size: 13, color: Color(0xFF00D2FF)),
              const SizedBox(width: 5),
              Text(
                'AGENTS.md',
                style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textSecondary),
              ),
            ],
          ),
        ),
      ),
    );
  }

  /// Dialog to view and edit AGENTS.md project rules
  void _showProjectRulesDialog(BuildContext context) {
    widget.controller.loadProjectRules();
    final editController = TextEditingController(text: widget.controller.projectRulesContent.value);

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: Row(
          children: [
            const Icon(Icons.rule_folder_outlined, size: 18, color: Color(0xFF00D2FF)),
            const SizedBox(width: 8),
            Text(
              'Правила проекта (AGENTS.md)',
              style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
            ),
          ],
        ),
        content: SizedBox(
          width: 580,
          height: 380,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Инструкции и правила кодирования, передаваемые агенту в системном контексте:',
                style: TextStyle(fontSize: 11.5, color: DesktopTheme.textMuted),
              ),
              const SizedBox(height: 10),
              Expanded(
                child: Container(
                  padding: const EdgeInsets.all(10),
                  decoration: BoxDecoration(
                    color: DesktopTheme.isDark ? const Color(0xFF090D14) : const Color(0xFFF1F5F9),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: DesktopTheme.borderSubtle),
                  ),
                  child: TextField(
                    controller: editController,
                    maxLines: null,
                    expands: true,
                    style: TextStyle(fontSize: 12, fontFamily: 'Consolas', color: DesktopTheme.textPrimary),
                    decoration: const InputDecoration(
                      border: InputBorder.none,
                      isDense: true,
                      contentPadding: EdgeInsets.zero,
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text('Отмена', style: TextStyle(color: DesktopTheme.textMuted)),
          ),
          ElevatedButton(
            onPressed: () {
              widget.controller.saveProjectRules(editController.text);
              Navigator.of(ctx).pop();
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF00D2FF),
              foregroundColor: const Color(0xFF090D14),
            ),
            child: const Text('Сохранить', style: TextStyle(fontWeight: FontWeight.bold)),
          ),
        ],
      ),
    );
  }

  /// Quick interactive terminal stdin chip button
  Widget _buildTerminalQuickStdinChip(String label, VoidCallback onTap, {bool isDanger = false}) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
        decoration: BoxDecoration(
          color: isDanger ? const Color(0xFFEF4444).withOpacity(0.15) : const Color(0xFF00D2FF).withOpacity(0.12),
          borderRadius: BorderRadius.circular(4),
          border: Border.all(
            color: isDanger ? const Color(0xFFEF4444).withOpacity(0.4) : const Color(0xFF00D2FF).withOpacity(0.3),
            width: 0.8,
          ),
        ),
        child: Text(
          label,
          style: TextStyle(
            fontSize: 10,
            fontFamily: 'Consolas',
            fontWeight: FontWeight.bold,
            color: isDanger ? const Color(0xFFEF4444) : const Color(0xFF00D2FF),
          ),
        ),
      ),
    );
  }

  // ==========================================
  // POPUP MENUS
  // ==========================================
  // Helper to show Cyber-minimal anchored menu adhering to DESIGN_SYSTEM.md §4.2:
  // When trigger is near the bottom (like bottom composer), it pops UPWARDS with a 6px margin.
  // When trigger is in the upper area (like hero prompt), it pops DOWNWARDS.
  Future<T?> _showAnchoredCyberMenu<T>({
    required BuildContext buttonContext,
    required List<PopupMenuEntry<T>> items,
    double estimatedHeight = 180,
    double minWidth = 220,
  }) async {
    final RenderBox? button = buttonContext.findRenderObject() as RenderBox?;
    final RenderBox? overlay = Overlay.of(buttonContext).context.findRenderObject() as RenderBox?;
    if (button == null || overlay == null) return null;

    final buttonTopLeft = button.localToGlobal(Offset.zero, ancestor: overlay);
    final buttonBottomRight = button.localToGlobal(button.size.bottomRight(Offset.zero), ancestor: overlay);
    final screenHeight = overlay.size.height;
    final screenWidth = overlay.size.width;

    // Trigger in lower half / near bottom opens UPWARDS (DESIGN_SYSTEM.md §4.2)
    final isBottomHalf = buttonBottomRight.dy > screenHeight / 2;

    final double top;
    if (isBottomHalf) {
      // Pop UPWARDS: menu ends 6px above the button top
      top = (buttonTopLeft.dy - estimatedHeight - 6.0).clamp(8.0, screenHeight - estimatedHeight - 8.0);
    } else {
      // Pop DOWNWARDS: menu starts 4px below the button bottom
      top = buttonBottomRight.dy + 4.0;
    }

    // Horizontal alignment: if in right half, align right edge with button right edge
    final isRightHalf = buttonTopLeft.dx > screenWidth / 2;
    final double left = isRightHalf
        ? (buttonBottomRight.dx - minWidth).clamp(8.0, screenWidth - minWidth - 8.0)
        : buttonTopLeft.dx.clamp(8.0, screenWidth - minWidth - 8.0);

    return showMenu<T>(
      context: buttonContext,
      position: RelativeRect.fromLTRB(left, top, screenWidth - (left + minWidth), screenHeight - top),
      items: items,
      color: DesktopTheme.bgSurface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
      ),
      elevation: 8,
    );
  }

  Widget _buildAddMenuButton() {
    return Builder(
      builder: (btnCtx) => Tooltip(
        message: DesktopI18n.addContextTooltip,
        child: InkWell(
          onTap: () async {
            final val = await _showAnchoredCyberMenu<String>(
              buttonContext: btnCtx,
              estimatedHeight: 155,
              minWidth: 260,
              items: [
                _buildPopupItem('attachment', Icons.attach_file, 'Прикрепить файл / изображение...'),
                _buildPopupItem('paste_image', Icons.content_paste_go, 'Вставить скриншот из буфера (Ctrl+V)'),
                _buildPopupItem('mention', Icons.alternate_email, DesktopI18n.mentionFileItem),
                _buildPopupItem('command', Icons.terminal, DesktopI18n.insertCommandItem),
              ],
            );
            if (val == 'attachment') {
              widget.controller.pickFilesFromDisk();
            } else if (val == 'paste_image') {
              widget.controller.pasteScreenshotFromClipboard().then((hasImage) {
                if (!hasImage) {
                  Get.snackbar(
                    'Скриншот',
                    'В буфере обмена нет изображения. Сделайте снимок (Win+Shift+S) и повторите.',
                    snackPosition: SnackPosition.BOTTOM,
                    duration: const Duration(seconds: 2),
                  );
                }
              });
            } else if (val == 'mention') {
              widget.controller.addAttachment('@lib/main.dart');
            } else if (val == 'command') {
              widget.controller.inputController.text = '/goal ';
            }
          },
          borderRadius: BorderRadius.circular(6),
          child: Container(
            width: 28,
            height: 28,
            decoration: BoxDecoration(
              color: const Color(0xFF282D36),
              borderRadius: BorderRadius.circular(6),
            ),
            child: const Icon(Icons.add, size: 16, color: Color(0xFFCBD5E1)),
          ),
        ),
      ),
    );
  }

  Widget _buildPermissionModeMenuButton() {
    return Obx(() {
      final mode = widget.controller.permissionMode.value;

      return Builder(
        builder: (btnCtx) => Tooltip(
          message: DesktopI18n.permissionModeTooltip,
          child: InkWell(
            onTap: () async {
              final selected = await _showAnchoredCyberMenu<PermissionMode>(
                buttonContext: btnCtx,
                estimatedHeight: 240,
                minWidth: 280,
                items: [
                  _buildPermissionItem(
                    PermissionMode.askBeforeChanges,
                    Icons.pan_tool_outlined,
                    DesktopI18n.askBeforeChangesTitle,
                    DesktopI18n.askBeforeChangesDesc,
                    mode == PermissionMode.askBeforeChanges,
                  ),
                  _buildPermissionItem(
                    PermissionMode.editAutomatically,
                    Icons.shield_outlined,
                    DesktopI18n.editAutomaticallyTitle,
                    DesktopI18n.editAutomaticallyDesc,
                    mode == PermissionMode.editAutomatically,
                  ),
                  _buildPermissionItem(
                    PermissionMode.planMode,
                    Icons.calendar_today_outlined,
                    DesktopI18n.planModeTitle,
                    DesktopI18n.planModeDesc,
                    mode == PermissionMode.planMode,
                  ),
                  _buildPermissionItem(
                    PermissionMode.fullAccess,
                    Icons.security,
                    DesktopI18n.fullAccessTitle,
                    DesktopI18n.fullAccessDesc,
                    mode == PermissionMode.fullAccess,
                  ),
                ],
              );
              if (selected != null) {
                widget.controller.setPermissionMode(selected);
              }
            },
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: const Color(0xFF282D36),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    mode == PermissionMode.fullAccess ? Icons.security : Icons.pan_tool_outlined,
                    size: 13,
                    color: mode == PermissionMode.fullAccess ? const Color(0xFFF59E0B) : const Color(0xFFCBD5E1),
                  ),
                  const SizedBox(width: 6),
                  Text(
                    mode.label,
                    style: TextStyle(
                      fontSize: 12,
                      color: mode == PermissionMode.fullAccess ? const Color(0xFFF59E0B) : Colors.white,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(width: 4),
                  const Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF94A3B8)),
                ],
              ),
            ),
          ),
        ),
      );
    });
  }

  Widget _buildProviderMenuButton() {
    return Obx(() {
      final activeProvId = widget.controller.activeProvider.value;
      final configuredProvs = widget.controller.configuredProviders;
      final hasProviders = configuredProvs.isNotEmpty;

      final currentProv = configuredProvs.firstWhereOrNull((p) => p['id'] == activeProvId);
      final displayName = currentProv?['name'] as String? ?? (hasProviders ? configuredProvs.first['name'] as String : 'Нет провайдеров');

      return Builder(
        builder: (btnCtx) => Tooltip(
          message: hasProviders ? 'Выбор LLM-провайдера' : 'Провайдеры не настроены (нажмите для настройки)',
          child: InkWell(
            onTap: () async {
              final double estHeight = !hasProviders ? 80.0 : (40.0 + configuredProvs.length * 36.0 + 44.0);
              final val = await _showAnchoredCyberMenu<String>(
                buttonContext: btnCtx,
                estimatedHeight: estHeight,
                minWidth: 230,
                items: !hasProviders
                    ? [
                        const PopupMenuItem(
                          enabled: false,
                          height: 28,
                          child: Text(
                            'НЕТ ПОДКЛЮЧЕННЫХ ПРОВАЙДЕРОВ',
                            style: TextStyle(fontSize: 10, color: Color(0xFFEF4444), fontWeight: FontWeight.bold),
                          ),
                        ),
                        PopupMenuItem(
                          value: 'manage',
                          height: 40,
                          child: Row(
                            children: const [
                              Icon(Icons.add_circle_outline, size: 14, color: Color(0xFF00D2FF)),
                              SizedBox(width: 8),
                              Text(
                                'Подключить провайдер (API Key)...',
                                style: TextStyle(fontSize: 12, color: Colors.white),
                              ),
                            ],
                          ),
                        ),
                      ]
                    : [
                        const PopupMenuItem(
                          enabled: false,
                          height: 28,
                          child: Text(
                            'АКТИВНЫЕ ПРОВАЙДЕРЫ',
                            style: TextStyle(fontSize: 10, color: Color(0xFF64748B), fontWeight: FontWeight.bold),
                          ),
                        ),
                        for (final p in configuredProvs)
                          PopupMenuItem<String>(
                            value: p['id'] as String,
                            height: 34,
                            child: Row(
                              children: [
                                Icon(
                                  p['isCustom'] == true ? Icons.cloud_queue : Icons.hub_outlined,
                                  size: 13,
                                  color: activeProvId == p['id'] ? const Color(0xFF00D2FF) : const Color(0xFF94A3B8),
                                ),
                                const SizedBox(width: 8),
                                Expanded(
                                  child: Text(
                                    p['name'] as String,
                                    style: TextStyle(
                                      fontSize: 12,
                                      color: activeProvId == p['id'] ? const Color(0xFF00D2FF) : Colors.white,
                                      fontWeight: activeProvId == p['id'] ? FontWeight.bold : FontWeight.normal,
                                    ),
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ),
                                if (activeProvId == p['id'])
                                  const Icon(Icons.check, size: 14, color: Color(0xFF00D2FF)),
                              ],
                            ),
                          ),
                        const PopupMenuDivider(height: 1),
                        PopupMenuItem(
                          value: 'manage',
                          height: 36,
                          child: Row(
                            children: const [
                              Icon(Icons.settings_outlined, size: 14, color: Color(0xFF94A3B8)),
                              SizedBox(width: 8),
                              Text('Управление провайдерами...', style: TextStyle(fontSize: 12, color: Color(0xFFCBD5E1))),
                            ],
                          ),
                        ),
                      ],
              );
              if (val == 'manage') {
                widget.onOpenSettings?.call();
              } else if (val != null) {
                widget.controller.setProvider(val);
              }
            },
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: const Color(0xFF282D36),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Container(
                    width: 6,
                    height: 6,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: hasProviders ? const Color(0xFF10B981) : const Color(0xFFEF4444),
                    ),
                  ),
                  const SizedBox(width: 6),
                  Icon(
                    currentProv?['isCustom'] == true ? Icons.cloud_queue : Icons.hub_outlined,
                    size: 11,
                    color: hasProviders ? const Color(0xFF00D2FF) : const Color(0xFFFCA5A5),
                  ),
                  const SizedBox(width: 5),
                  Text(
                    displayName,
                    style: TextStyle(
                      fontSize: 12,
                      color: hasProviders ? Colors.white : const Color(0xFFFCA5A5),
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(width: 4),
                  const Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF94A3B8)),
                ],
              ),
            ),
          ),
        ),
      );
    });
  }

  Widget _buildModelMenuButton() {
    return Obx(() {
      final active = widget.controller.activeModel.value;
      final configured = widget.controller.configuredModels;
      final hasConfigured = configured.isNotEmpty;
      final hasActiveModel = hasConfigured && active != 'Провайдер не настроен';

      return Builder(
        builder: (btnCtx) => Tooltip(
          message: hasConfigured ? DesktopI18n.modelTooltip : 'Нет доступных моделей',
          child: InkWell(
            onTap: () async {
              final double estHeight = !hasConfigured ? 80.0 : (36.0 + configured.length * 36.0 + 44.0);
              final val = await _showAnchoredCyberMenu<String>(
                buttonContext: btnCtx,
                estimatedHeight: estHeight,
                minWidth: 230,
                items: !hasConfigured
                    ? [
                        const PopupMenuItem(
                          enabled: false,
                          height: 28,
                          child: Text(
                            'НЕТ ДОСТУПНЫХ МОДЕЛЕЙ',
                            style: TextStyle(fontSize: 10, color: Color(0xFFEF4444), fontWeight: FontWeight.bold),
                          ),
                        ),
                        PopupMenuItem(
                          value: 'manage',
                          height: 40,
                          child: Row(
                            children: const [
                              Icon(Icons.add_circle_outline, size: 14, color: Color(0xFF00D2FF)),
                              SizedBox(width: 8),
                              Text(
                                'Подключить провайдер (API Key)...',
                                style: TextStyle(fontSize: 12, color: Colors.white),
                              ),
                            ],
                          ),
                        ),
                      ]
                    : [
                        PopupMenuItem(
                          enabled: false,
                          height: 28,
                          child: Text(
                            'ДОСТУПНЫЕ МОДЕЛИ',
                            style: const TextStyle(fontSize: 10, color: Color(0xFF64748B), fontWeight: FontWeight.bold),
                          ),
                        ),
                        for (final m in configured)
                          _buildModelItem(m, active == m),
                        const PopupMenuDivider(height: 1),
                        PopupMenuItem(
                          value: 'manage',
                          height: 36,
                          child: Row(
                            children: [
                              const Icon(Icons.settings_outlined, size: 14, color: Color(0xFF94A3B8)),
                              const SizedBox(width: 8),
                              Text(DesktopI18n.providerSettingsAction, style: const TextStyle(fontSize: 12, color: Color(0xFFCBD5E1))),
                            ],
                          ),
                        ),
                      ],
              );
              if (val == 'manage') {
                widget.onOpenSettings?.call();
              } else if (val != null) {
                widget.controller.setModel(val);
              }
            },
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: const Color(0xFF282D36),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Container(
                    width: 6,
                    height: 6,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: hasActiveModel ? const Color(0xFF10B981) : const Color(0xFFEF4444),
                    ),
                  ),
                  const SizedBox(width: 6),
                  Text(
                    hasActiveModel ? active : 'Модель не выбрана',
                    style: TextStyle(
                      fontSize: 12,
                      color: hasActiveModel ? Colors.white : const Color(0xFFFCA5A5),
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(width: 4),
                  const Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF94A3B8)),
                ],
              ),
            ),
          ),
        ),
      );
    });
  }

  Widget _buildThoughtLevelMenuButton() {
    return Obx(() {
      final level = widget.controller.thoughtLevel.value;

      return Builder(
        builder: (btnCtx) => Tooltip(
          message: DesktopI18n.thoughtLevelTooltip,
          child: InkWell(
            onTap: () async {
              final selected = await _showAnchoredCyberMenu<String>(
                buttonContext: btnCtx,
                estimatedHeight: 150,
                minWidth: 160,
                items: [
                  _buildThoughtItem('Low', level == 'Low'),
                  _buildThoughtItem('High', level == 'High'),
                  _buildThoughtItem('Max', level == 'Max'),
                ],
              );
              if (selected != null) {
                widget.controller.setThoughtLevel(selected);
              }
            },
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: const Color(0xFF282D36),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(FontAwesomeIcons.brain, size: 12, color: Color(0xFF00D2FF)),
                  const SizedBox(width: 6),
                  Text(level, style: const TextStyle(fontSize: 12, color: Colors.white, fontWeight: FontWeight.w500)),
                  const SizedBox(width: 4),
                  const Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF94A3B8)),
                ],
              ),
            ),
          ),
        ),
      );
    });
  }

  Widget _buildSendButton() {
    return InkWell(
      onTap: widget.controller.sendMessage,
      borderRadius: BorderRadius.circular(6),
      child: Container(
        width: 30,
        height: 30,
        decoration: BoxDecoration(
          color: const Color(0xFFE2E8F0),
          borderRadius: BorderRadius.circular(6),
        ),
        child: const Icon(Icons.arrow_upward, size: 16, color: Color(0xFF0F172A)),
      ),
    );
  }

  Widget _buildVoiceDuplexButton() {
    return Obx(() {
      final isEnabled = widget.controller.isSttConfiguredAndEnabled.value;
      if (!isEnabled) {
        return const SizedBox.shrink(); // Hidden by default until STT is configured and verified in Settings
      }
      final active = widget.controller.isVoiceDuplexActive.value;
      return IconButton(
        icon: Icon(
          active ? Icons.mic : Icons.mic_none,
          size: 18,
          color: active ? Colors.redAccent : const Color(0xFF00D2FF),
        ),
        tooltip: active ? 'Голосовой ввод активен (нажмите для остановки)' : 'Голосовой ввод (микрофон STT)',
        onPressed: widget.controller.toggleVoiceDuplex,
        splashRadius: 18,
      );
    });
  }

  // Helpers
  Widget _buildProjectChip() {
    final proj = widget.controller.activeProject.value ?? '';
    if (proj.isEmpty) return const SizedBox.shrink();
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurfaceElevated,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.folder_outlined, size: 12, color: Color(0xFF00D2FF)),
          const SizedBox(width: 6),
          Text(proj, style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textPrimary)),
          const SizedBox(width: 4),
          Icon(Icons.keyboard_arrow_down, size: 13, color: DesktopTheme.textMuted),
        ],
      ),
    );
  }

  Widget _buildBranchChip() {
    final branch = widget.controller.activeBranch.value ?? '';
    if (branch.isEmpty) return const SizedBox.shrink();
    return InkWell(
      onTap: () => setState(() => isGitToolsOpen = !isGitToolsOpen),
      borderRadius: BorderRadius.circular(6),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        decoration: BoxDecoration(
          color: DesktopTheme.bgSurfaceElevated,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: isGitToolsOpen ? const Color(0xFF00D2FF) : DesktopTheme.borderSubtle),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(FontAwesomeIcons.codeBranch, size: 10, color: Color(0xFF00D2FF)),
            const SizedBox(width: 6),
            Text(branch, style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF00D2FF))),
            const SizedBox(width: 4),
            Icon(Icons.keyboard_arrow_down, size: 13, color: DesktopTheme.textMuted),
          ],
        ),
      ),
    );
  }

  PopupMenuItem<String> _buildPopupItem(String value, IconData icon, String label) {
    return PopupMenuItem<String>(
      value: value,
      height: 32,
      child: Row(
        children: [
          Icon(icon, size: 15, color: const Color(0xFF94A3B8)),
          const SizedBox(width: 10),
          Text(label, style: const TextStyle(fontSize: 12, color: Color(0xFFE2E8F0))),
        ],
      ),
    );
  }

  PopupMenuItem<PermissionMode> _buildPermissionItem(
    PermissionMode mode,
    IconData icon,
    String title,
    String desc,
    bool isSelected,
  ) {
    return PopupMenuItem<PermissionMode>(
      value: mode,
      height: 48,
      child: Row(
        children: [
          Icon(icon, size: 15, color: isSelected ? const Color(0xFF00D2FF) : const Color(0xFF94A3B8)),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text(title, style: TextStyle(fontSize: 12, fontWeight: isSelected ? FontWeight.bold : FontWeight.w500, color: Colors.white)),
                Text(desc, style: const TextStyle(fontSize: 10, color: Color(0xFF64748B))),
              ],
            ),
          ),
          if (isSelected) const Icon(Icons.check, size: 14, color: Color(0xFF00D2FF)),
        ],
      ),
    );
  }

  PopupMenuItem<String> _buildModelItem(String model, bool isSelected) {
    return PopupMenuItem<String>(
      value: model,
      height: 34,
      child: Row(
        children: [
          Text(model, style: TextStyle(fontSize: 12, color: isSelected ? const Color(0xFF00D2FF) : const Color(0xFFE2E8F0))),
          const Spacer(),
          if (isSelected) const Icon(Icons.check, size: 14, color: Color(0xFF00D2FF)),
        ],
      ),
    );
  }

  PopupMenuItem<String> _buildThoughtItem(String level, bool isSelected) {
    return PopupMenuItem<String>(
      value: level,
      height: 32,
      child: Row(
        children: [
          Text(level, style: TextStyle(fontSize: 12, color: isSelected ? const Color(0xFF00D2FF) : const Color(0xFFE2E8F0))),
          const Spacer(),
          if (isSelected) const Icon(Icons.check, size: 14, color: Color(0xFF00D2FF)),
        ],
      ),
    );
  }
}
