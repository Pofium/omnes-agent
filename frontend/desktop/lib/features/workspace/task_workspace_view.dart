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
import 'task_workspace_controller.dart';

class DesktopTaskWorkspaceView extends StatefulWidget {
  final DesktopTaskWorkspaceController controller;
  final VoidCallback? onToggleTools;
  final VoidCallback? onToggleTerminal;
  final VoidCallback? onOpenSettings;
  final bool isToolsOpen;

  const DesktopTaskWorkspaceView({
    super.key,
    required this.controller,
    this.onToggleTools,
    this.onToggleTerminal,
    this.onOpenSettings,
    this.isToolsOpen = false,
  });

  @override
  State<DesktopTaskWorkspaceView> createState() => _DesktopTaskWorkspaceViewState();
}

class _DesktopTaskWorkspaceViewState extends State<DesktopTaskWorkspaceView> {
  bool isBannerDismissed = false;
  bool isGitToolsOpen = false;

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

            // Floating Git Tools Popover Card (Screenshot 2 Match)
            if (isGitToolsOpen)
              Positioned(
                top: 48,
                left: 180,
                child: _buildGitToolsCard(),
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
      padding: const EdgeInsets.symmetric(horizontal: 16),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 0.8)),
      ),
      child: Row(
        children: [
          if (!isNewTask && hasMessages) ...[
            // Task Title
            Flexible(
              child: Text(
                widget.controller.activeTaskTitle.value,
                style: TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.w600,
                  color: DesktopTheme.textPrimary,
                ),
                overflow: TextOverflow.ellipsis,
                maxLines: 1,
              ),
            ),
            if (widget.controller.activeProject.value != null && widget.controller.activeProject.value!.isNotEmpty) ...[
              const SizedBox(width: 10),
              _buildProjectChip(),
            ],
            if (widget.controller.activeBranch.value != null && widget.controller.activeBranch.value!.isNotEmpty) ...[
              const SizedBox(width: 6),
              _buildBranchChip(),
            ],
            const SizedBox(width: 6),

            // Menu dots
            InkWell(
              onTap: () => setState(() => isGitToolsOpen = !isGitToolsOpen),
              borderRadius: BorderRadius.circular(4),
              child: Padding(
                padding: const EdgeInsets.all(4),
                child: Icon(Icons.more_horiz, size: 16, color: DesktopTheme.textMuted),
              ),
            ),
          ] else ...[
            // Clean New Task Mode - no stale pinned branch or project!
            Text(
              DesktopI18n.newTask,
              style: TextStyle(
                fontSize: 13,
                fontWeight: FontWeight.w600,
                color: DesktopTheme.textMuted,
              ),
            ),
          ],

          const Spacer(),

          // Right Toolbar Controls
          // Workspace / folder switcher
          InkWell(
            onTap: () {},
            borderRadius: BorderRadius.circular(4),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.folder_outlined, size: 13, color: Color(0xFF00D2FF)),
                  const SizedBox(width: 4),
                  Icon(Icons.keyboard_arrow_down, size: 12, color: DesktopTheme.textMuted),
                ],
              ),
            ),
          ),
          // Gateway Live Status Badge
          InkWell(
            onTap: () => widget.controller.initGatewayConnection(),
            borderRadius: BorderRadius.circular(4),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: widget.controller.wsStatus.value == 'connected'
                    ? const Color(0xFF10B981).withOpacity(0.15)
                    : widget.controller.wsStatus.value == 'connecting'
                        ? const Color(0xFFF59E0B).withOpacity(0.15)
                        : const Color(0xFFEF4444).withOpacity(0.15),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(
                  color: widget.controller.wsStatus.value == 'connected'
                      ? const Color(0xFF10B981)
                      : widget.controller.wsStatus.value == 'connecting'
                          ? const Color(0xFFF59E0B)
                          : const Color(0xFFEF4444),
                  width: 0.8,
                ),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Container(
                    width: 6,
                    height: 6,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: widget.controller.wsStatus.value == 'connected'
                          ? const Color(0xFF10B981)
                          : widget.controller.wsStatus.value == 'connecting'
                              ? const Color(0xFFF59E0B)
                              : const Color(0xFFEF4444),
                    ),
                  ),
                  const SizedBox(width: 5),
                  Text(
                    widget.controller.wsStatus.value == 'connected'
                        ? 'Gateway 42617'
                        : widget.controller.wsStatus.value == 'connecting'
                            ? 'Шлюз...'
                            : 'Шлюз оффлайн',
                    style: TextStyle(
                      fontSize: 11,
                      fontWeight: FontWeight.w600,
                      color: widget.controller.wsStatus.value == 'connected'
                          ? const Color(0xFF10B981)
                          : widget.controller.wsStatus.value == 'connecting'
                              ? const Color(0xFFF59E0B)
                              : const Color(0xFFEF4444),
                    ),
                  ),
                ],
              ),
            ),
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

  // ==========================================
  // GIT TOOLS POPOVER CARD (Screenshot 2 Match)
  // ==========================================
  Widget _buildGitToolsCard() {
    return Container(
      width: 380,
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: DesktopTheme.borderSubtle, width: 1.2),
        boxShadow: [
          BoxShadow(
            color: DesktopTheme.isDark ? Colors.black.withOpacity(0.7) : Colors.black.withOpacity(0.12),
            blurRadius: 28,
            offset: const Offset(0, 10),
          ),
        ],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Row(
            children: [
              Text(
                DesktopI18n.gitTools,
                style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
              ),
              const Spacer(),
              InkWell(
                onTap: () {},
                child: Icon(Icons.more_horiz, size: 14, color: DesktopTheme.textMuted),
              ),
              const SizedBox(width: 8),
              InkWell(
                onTap: () => setState(() => isGitToolsOpen = false),
                child: Icon(Icons.close, size: 14, color: DesktopTheme.textMuted),
              ),
            ],
          ),
          const SizedBox(height: 12),

          // Changes
          Row(
            children: [
              Icon(Icons.assignment_outlined, size: 14, color: DesktopTheme.textMuted),
              const SizedBox(width: 8),
              Text(DesktopI18n.tr('Изменения', 'Changes'), style: TextStyle(fontSize: 12, color: DesktopTheme.textSecondary)),
              const Spacer(),
              const Text(
                '+72347 -0',
                style: TextStyle(fontSize: 12, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFF10B981)),
              ),
            ],
          ),
          const SizedBox(height: 10),

          // Branch
          Row(
            children: [
              Icon(FontAwesomeIcons.codeBranch, size: 12, color: DesktopTheme.textMuted),
              const SizedBox(width: 8),
              const Text('desktop-brand-ru', style: TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Color(0xFF00D2FF))),
              const SizedBox(width: 4),
              Icon(Icons.keyboard_arrow_down, size: 14, color: DesktopTheme.textMuted),
            ],
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
          Divider(height: 20, color: DesktopTheme.borderSubtle),

          // Progress 5/5
          Row(
            children: [
              Text(DesktopI18n.tr('Прогресс', 'Progress'), style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary)),
              const SizedBox(width: 8),
              const Text('5/5', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF10B981), fontWeight: FontWeight.bold)),
            ],
          ),
          const SizedBox(height: 10),

          _buildCheckItem('Graft + merge upstream dsh v0.1.3-alpha.1, resolve 519 conflicts'),
          _buildCheckItem('typecheck/i18n/tests/build green; ru dictionaries completed'),
          _buildCheckItem('Runtime synced to 0.1.3; root cause of fileUploads pending found and fixed'),
          _buildCheckItem('Exe smoke: boots with live URL on synced 0.1.3 runtime'),
          _buildCheckItem('ob2h подключён (project dsh-desktop), факты в памяти, всё запущено'),
        ],
      ),
    );
  }

  Widget _buildCheckItem(String text) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.check_circle_outline, size: 14, color: Color(0xFF10B981)),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              text,
              style: TextStyle(fontSize: 11, color: DesktopTheme.textSecondary, height: 1.3),
            ),
          ),
        ],
      ),
    );
  }

  // ==========================================
  // BOTTOM INLINE TERMINAL
  // ==========================================
  Widget _buildBottomTerminal() {
    return Container(
      height: 180,
      margin: const EdgeInsets.fromLTRB(24, 0, 24, 10),
      decoration: BoxDecoration(
        color: DesktopTheme.isDark ? const Color(0xFF0E1015) : const Color(0xFFF8FAFC),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            height: 30,
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
                Text(DesktopI18n.terminalConsole, style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textPrimary)),
                const Spacer(),
                InkWell(
                  onTap: () => widget.controller.isTerminalOpen.value = false,
                  child: Icon(Icons.close, size: 13, color: DesktopTheme.textMuted),
                ),
              ],
            ),
          ),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(10),
              child: ListView(
                children: const [
                  Text('\$ cargo check --workspace', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF00D2FF))),
                  Text('   Compiling omnesagent-gateway v0.1.0 (C:\\Projects\\Omnes-agent)...', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF94A3B8))),
                  Text('   Finished dev [unoptimized + debuginfo] in 1.8s', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF10B981))),
                  Text('\$ flutter analyze', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF00D2FF))),
                  Text('No issues found! (ran in 1.2s)', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF10B981))),
                ],
              ),
            ),
          ),
        ],
      ),
    );
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
                      onTap: () {},
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
                        const Icon(Icons.attach_file, size: 12, color: Color(0xFF00D2FF)),
                        const SizedBox(width: 4),
                        Text(
                          att,
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

                const Spacer(),

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

  Widget _buildBottomFloatingComposer() {
    return Container(
      padding: const EdgeInsets.fromLTRB(24, 0, 24, 16),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 820),
        child: _buildComposerCard(isHero: false),
      ),
    );
  }

  // ==========================================
  // CHAT TIMELINE (Active Task Run)
  // ==========================================
  Widget _buildChatTimeline() {
    return ListView.builder(
      controller: widget.controller.scrollController,
      padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
      itemCount: widget.controller.messages.length,
      itemBuilder: (context, index) {
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
                    // Thinking accordion if bot
                    if (!isUser && msg.thinking != null && msg.thinking!.isNotEmpty) ...[
                      Container(
                        padding: const EdgeInsets.all(10),
                        margin: const EdgeInsets.only(bottom: 12),
                        decoration: BoxDecoration(
                          color: DesktopTheme.bgSurfaceElevated,
                          borderRadius: BorderRadius.circular(6),
                          border: const Border(left: BorderSide(color: Color(0xFF00D2FF), width: 2.5)),
                        ),
                        child: Text(
                          msg.thinking!,
                          style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textSecondary),
                        ),
                      ),
                    ],

                    _buildMessageContent(context, msg),

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

                    // Git diff chip (Screenshot 2 match)
                    if (!isUser && msg.toolCalls.isNotEmpty) ...[
                      const SizedBox(height: 12),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                        decoration: BoxDecoration(
                          color: DesktopTheme.bgSurfaceElevated,
                          borderRadius: BorderRadius.circular(6),
                          border: Border.all(color: DesktopTheme.borderSubtle),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Icons.chevron_right, size: 14, color: DesktopTheme.textMuted),
                            const SizedBox(width: 4),
                            Text('${DesktopI18n.oneFileChanged} ', style: TextStyle(fontSize: 11, color: DesktopTheme.textSecondary)),
                            const Text('+9', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFF10B981))),
                            const Text(' -0', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFFEF4444))),
                            const SizedBox(width: 14),
                            InkWell(
                              onTap: () {},
                              child: Row(
                                children: [
                                  Icon(Icons.undo, size: 12, color: DesktopTheme.textMuted),
                                  const SizedBox(width: 4),
                                  Text(DesktopI18n.undo, style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted)),
                                ],
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],

                    // Message Reactions Row (Image 2 match: [⎘ Copy] [👍] [👎] [🔀 Branch] 9/5, 10:07 AM)
                    if (!isUser)
                      _buildMessageReactionsRow(context, msg),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildMessageContent(BuildContext context, ChatMessage msg) {
    if (!msg.text.contains('```')) {
      return SelectableText(
        msg.text,
        style: TextStyle(fontSize: 13, height: 1.5, color: DesktopTheme.textPrimary),
      );
    }

    final parts = msg.text.split('```');
    final widgets = <Widget>[];

    for (int i = 0; i < parts.length; i++) {
      final part = parts[i];
      if (i % 2 == 0) {
        if (part.trim().isNotEmpty) {
          widgets.add(
            SelectableText(
              part.trim(),
              style: TextStyle(fontSize: 13, height: 1.5, color: DesktopTheme.textPrimary),
            ),
          );
        }
      } else {
        final lines = part.split('\n');
        final lang = lines.isNotEmpty && lines.first.trim().isNotEmpty ? lines.first.trim() : 'text';
        final codeBody = lines.length > 1 ? lines.sublist(1).join('\n').trim() : part.trim();

        widgets.add(
          Container(
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
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                  decoration: BoxDecoration(
                    color: DesktopTheme.isDark ? const Color(0xFF1B1E28) : const Color(0xFFE2E8F0),
                    borderRadius: const BorderRadius.vertical(top: Radius.circular(7)),
                  ),
                  child: Row(
                    children: [
                      const Icon(Icons.description_outlined, size: 14, color: Color(0xFF00D2FF)),
                      const SizedBox(width: 8),
                      Text(
                        lang,
                        style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textPrimary),
                      ),
                      const Spacer(),
                      InkWell(
                        onTap: () {
                          Clipboard.setData(ClipboardData(text: codeBody));
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text(DesktopI18n.commandCopied), duration: const Duration(seconds: 1)),
                          );
                        },
                        child: Padding(
                          padding: const EdgeInsets.all(2),
                          child: Icon(Icons.copy, size: 13, color: DesktopTheme.textMuted),
                        ),
                      ),
                      const SizedBox(width: 8),
                      InkWell(
                        onTap: () {
                          widget.controller.isTerminalOpen.value = true;
                        },
                        child: Padding(
                          padding: const EdgeInsets.all(2),
                          child: Icon(Icons.play_arrow_outlined, size: 16, color: DesktopTheme.textMuted),
                        ),
                      ),
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
                      color: DesktopTheme.isDark ? const Color(0xFFE2E8F0) : const Color(0xFF0F172A),
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
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
                onTap: () {},
                borderRadius: BorderRadius.circular(4),
                child: const Padding(
                  padding: EdgeInsets.all(4),
                  child: Icon(FontAwesomeIcons.codeBranch, size: 11, color: Color(0xFF94A3B8)),
                ),
              ),
              const SizedBox(width: 8),

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

  // ==========================================
  // POPUP MENUS
  // ==========================================
  Widget _buildAddMenuButton() {
    return PopupMenuButton<String>(
      tooltip: DesktopI18n.addContextTooltip,
      offset: const Offset(0, -170),
      color: const Color(0xFF22252A),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      onSelected: (val) {
        if (val == 'attachment') {
          widget.controller.addAttachment('attachment_spec.md');
        } else if (val == 'mention') {
          widget.controller.addAttachment('@lib/main.dart');
        } else if (val == 'chat') {
          widget.controller.addAttachment('#Task #41');
        } else if (val == 'command') {
          widget.controller.inputController.text = '/goal ';
        }
      },
      itemBuilder: (context) => [
        _buildPopupItem('attachment', Icons.attach_file, DesktopI18n.attachFileItem),
        _buildPopupItem('mention', Icons.alternate_email, DesktopI18n.mentionFileItem),
        _buildPopupItem('chat', Icons.chat_bubble_outline, DesktopI18n.linkChatItem),
        _buildPopupItem('command', Icons.terminal, DesktopI18n.insertCommandItem),
      ],
      child: Container(
        width: 28,
        height: 28,
        decoration: BoxDecoration(
          color: const Color(0xFF282D36),
          borderRadius: BorderRadius.circular(6),
        ),
        child: const Icon(Icons.add, size: 16, color: Color(0xFFCBD5E1)),
      ),
    );
  }

  Widget _buildPermissionModeMenuButton() {
    return Obx(() {
      final mode = widget.controller.permissionMode.value;

      return PopupMenuButton<PermissionMode>(
        tooltip: DesktopI18n.permissionModeTooltip,
        offset: const Offset(0, -220),
        color: const Color(0xFF22252A),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
        onSelected: widget.controller.setPermissionMode,
        itemBuilder: (context) => [
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
      );
    });
  }

  Widget _buildModelMenuButton() {
    return Obx(() {
      final active = widget.controller.activeModel.value;

      return PopupMenuButton<String>(
        tooltip: DesktopI18n.modelTooltip,
        offset: const Offset(0, -230),
        color: const Color(0xFF22252A),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        onSelected: (model) {
          if (model == 'manage') {
            widget.onOpenSettings?.call();
          } else {
            widget.controller.setModel(model);
          }
        },
        itemBuilder: (context) => [
          PopupMenuItem(
            enabled: false,
            height: 28,
            child: Text(DesktopI18n.modelProvidersHeader, style: const TextStyle(fontSize: 10, color: Color(0xFF64748B), fontWeight: FontWeight.bold)),
          ),
          _buildModelItem('GLM-5.3-Flash', active == 'GLM-5.3-Flash'),
          _buildModelItem('GLM-5.3', active == 'GLM-5.3'),
          _buildModelItem('Claude 3.5 Sonnet', active == 'Claude 3.5 Sonnet'),
          _buildModelItem('DeepSeek V3', active == 'DeepSeek V3'),
          _buildModelItem('Local Ollama', active == 'Local Ollama'),
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
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: const Color(0xFF282D36),
            borderRadius: BorderRadius.circular(6),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Green active status dot (Screenshot 2 Match)
              Container(
                width: 6,
                height: 6,
                decoration: const BoxDecoration(
                  shape: BoxShape.circle,
                  color: Color(0xFF10B981),
                ),
              ),
              const SizedBox(width: 6),
              Text(active, style: const TextStyle(fontSize: 12, color: Colors.white, fontWeight: FontWeight.w500)),
              const SizedBox(width: 4),
              const Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF94A3B8)),
            ],
          ),
        ),
      );
    });
  }

  Widget _buildThoughtLevelMenuButton() {
    return Obx(() {
      final level = widget.controller.thoughtLevel.value;

      return PopupMenuButton<String>(
        tooltip: DesktopI18n.thoughtLevelTooltip,
        offset: const Offset(0, -140),
        color: const Color(0xFF22252A),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        onSelected: widget.controller.setThoughtLevel,
        itemBuilder: (context) => [
          _buildThoughtItem('Low', level == 'Low'),
          _buildThoughtItem('High', level == 'High'),
          _buildThoughtItem('Max', level == 'Max'),
        ],
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
      final active = widget.controller.isVoiceDuplexActive.value;
      return IconButton(
        icon: Icon(
          active ? Icons.mic : Icons.mic_none,
          size: 18,
          color: active ? Colors.redAccent : DesktopTheme.textMuted,
        ),
        tooltip: active ? 'Голосовой дуплекс активен (нажмите для остановки)' : 'Голосовой дуплекс (микрофон + синтез речи)',
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
      height: 38,
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
