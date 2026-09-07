// OmnesAgent ADE Task Workspace View matching authentic Windows ADE layout (Screenshot 2).
// Features Git Tools status card popover, bottom terminal toggle, multi-project breadcrumbs,
// git diff chips, and localized Russian assistant prompts.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';

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
    return Container(
      color: const Color(0xFF16181D),
      child: Obx(() {
        final _ = DesktopI18n.currentLanguage.value;
        final hasMessages = widget.controller.messages.isNotEmpty;

        return Stack(
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
        );
      }),
    );
  }

  // ==========================================
  // TOP NAV BAR MATCHING SCREENSHOT 2
  // ==========================================
  Widget _buildTopNavBar(bool hasMessages) {
    return Container(
      height: 44,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      decoration: const BoxDecoration(
        color: Color(0xFF16181D),
        border: Border(bottom: BorderSide(color: Color(0xFF22262E), width: 0.8)),
      ),
      child: Row(
        children: [
          if (hasMessages) ...[
            // Task Title
            Flexible(
              child: Text(
                widget.controller.activeTaskTitle.value,
                style: const TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.w600,
                  color: Colors.white,
                ),
                overflow: TextOverflow.ellipsis,
                maxLines: 1,
              ),
            ),
            const SizedBox(width: 10),

            // Project folder chip
            _buildProjectChip(),
            const SizedBox(width: 6),

            // Branch chip with toggle for Git Tools
            _buildBranchChip(),
            const SizedBox(width: 6),

            // Menu dots
            InkWell(
              onTap: () => setState(() => isGitToolsOpen = !isGitToolsOpen),
              borderRadius: BorderRadius.circular(4),
              child: const Padding(
                padding: EdgeInsets.all(4),
                child: Icon(Icons.more_horiz, size: 16, color: Color(0xFF94A3B8)),
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
                color: const Color(0xFF22262E),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: const [
                  Icon(Icons.folder_outlined, size: 13, color: Color(0xFF00D2FF)),
                  SizedBox(width: 4),
                  Icon(Icons.keyboard_arrow_down, size: 12, color: Color(0xFF94A3B8)),
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
                  color: widget.controller.isTerminalOpen.value ? const Color(0xFF00D2FF).withOpacity(0.18) : const Color(0xFF22262E),
                  borderRadius: BorderRadius.circular(5),
                  border: Border.all(
                    color: widget.controller.isTerminalOpen.value ? const Color(0xFF00D2FF) : Colors.transparent,
                    width: 0.8,
                  ),
                ),
                child: Icon(
                  FontAwesomeIcons.terminal,
                  size: 12,
                  color: widget.controller.isTerminalOpen.value ? const Color(0xFF00D2FF) : const Color(0xFF94A3B8),
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
                  color: widget.isToolsOpen ? const Color(0xFF00D2FF).withOpacity(0.18) : const Color(0xFF22262E),
                  borderRadius: BorderRadius.circular(5),
                  border: Border.all(
                    color: widget.isToolsOpen ? const Color(0xFF00D2FF) : Colors.transparent,
                    width: 0.8,
                  ),
                ),
                child: Icon(
                  Icons.view_sidebar_outlined,
                  size: 14,
                  color: widget.isToolsOpen ? const Color(0xFF00D2FF) : const Color(0xFF94A3B8),
                ),
              ),
            ),
          ),
          const SizedBox(width: 14),

          // Windows native window controls: [ — ] [ 🗖 ] [ ✕ ]
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildWinButton(Icons.remove, DesktopI18n.minimize, () {}),
              const SizedBox(width: 6),
              _buildWinButton(Icons.crop_square, DesktopI18n.maximize, () {}),
              const SizedBox(width: 6),
              _buildWinButton(Icons.close, DesktopI18n.close, () {}, isClose: true),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildWinButton(IconData icon, String tooltip, VoidCallback onTap, {bool isClose = false}) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(3),
      hoverColor: isClose ? const Color(0xFFE81123) : const Color(0xFF2A2E37),
      child: Padding(
        padding: const EdgeInsets.all(4),
        child: Icon(icon, size: 12, color: const Color(0xFF94A3B8)),
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
        color: const Color(0xFF1B1D22),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xFF2A2F3B), width: 1.2),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.7),
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
                style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: Colors.white),
              ),
              const Spacer(),
              InkWell(
                onTap: () {},
                child: const Icon(Icons.more_horiz, size: 14, color: Color(0xFF94A3B8)),
              ),
              const SizedBox(width: 8),
              InkWell(
                onTap: () => setState(() => isGitToolsOpen = false),
                child: const Icon(Icons.close, size: 14, color: Color(0xFF94A3B8)),
              ),
            ],
          ),
          const SizedBox(height: 12),

          // Changes
          Row(
            children: [
              const Icon(Icons.assignment_outlined, size: 14, color: Color(0xFF94A3B8)),
              const SizedBox(width: 8),
              Text(DesktopI18n.tr('Изменения', 'Changes'), style: const TextStyle(fontSize: 12, color: Color(0xFFE2E8F0))),
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
              const Icon(FontAwesomeIcons.codeBranch, size: 12, color: Color(0xFF94A3B8)),
              const SizedBox(width: 8),
              const Text('desktop-brand-ru', style: TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Color(0xFF00D2FF))),
              const SizedBox(width: 4),
              const Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF94A3B8)),
            ],
          ),
          const SizedBox(height: 10),

          // Commit or push
          Row(
            children: [
              const Icon(Icons.commit, size: 14, color: Color(0xFF94A3B8)),
              const SizedBox(width: 8),
              Text(DesktopI18n.tr('Коммит или пуш', 'Commit or push'), style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8))),
            ],
          ),
          const Divider(height: 20, color: Color(0xFF262B34)),

          // Progress 5/5
          Row(
            children: [
              Text(DesktopI18n.tr('Прогресс', 'Progress'), style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: Colors.white)),
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
              style: const TextStyle(fontSize: 11, color: Color(0xFF94A3B8), height: 1.3),
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
        color: const Color(0xFF0E1015),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: const Color(0xFF242934)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            height: 30,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: const BoxDecoration(
              color: Color(0xFF16181F),
              borderRadius: BorderRadius.vertical(top: Radius.circular(9)),
            ),
            child: Row(
              children: [
                const Icon(FontAwesomeIcons.terminal, size: 11, color: Color(0xFF00D2FF)),
                const SizedBox(width: 8),
                Text(DesktopI18n.terminalConsole, style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Colors.white)),
                const Spacer(),
                InkWell(
                  onTap: () => widget.controller.isTerminalOpen.value = false,
                  child: const Icon(Icons.close, size: 13, color: Color(0xFF94A3B8)),
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
                style: const TextStyle(
                  fontSize: 28,
                  fontWeight: FontWeight.w400,
                  fontFamily: 'Georgia',
                  letterSpacing: 0.4,
                  color: Colors.white,
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
                    color: const Color(0xFF1B1D22),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: const Color(0xFF252A33)),
                  ),
                  child: Row(
                    children: [
                       const Icon(Icons.campaign_outlined, size: 16, color: Color(0xFF94A3B8)),
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
          color: const Color(0xFF1B1D22),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: const Color(0xFF262B34)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.nightlight_round, size: 14, color: Color(0xFF94A3B8)),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    title,
                    style: const TextStyle(
                      fontSize: 13,
                      fontWeight: FontWeight.bold,
                      color: Colors.white,
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
              style: const TextStyle(
                fontSize: 11,
                color: Color(0xFF8B949E),
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
        color: const Color(0xFF1E2127),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: const Color(0xFF2E333D), width: 1.2),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.35),
            blurRadius: 24,
            offset: const Offset(0, 8),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 1. Top inside row: Project & Branch switchers (Hero mode)
          if (isHero)
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
                      color: const Color(0xFF282D37),
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
                          style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Colors.white),
                        ),
                        const SizedBox(width: 4),
                        InkWell(
                          onTap: () => widget.controller.removeAttachment(idx),
                          child: const Icon(Icons.close, size: 12, color: Color(0xFF94A3B8)),
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
              style: const TextStyle(fontSize: 14, color: Colors.white, height: 1.4),
              decoration: InputDecoration(
                hintText: isHero
                    ? DesktopI18n.heroInputHint
                    : DesktopI18n.promptPlaceholder,
                hintStyle: const TextStyle(fontSize: 13, color: Color(0xFF6B7280)),
                border: InputBorder.none,
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
                const SizedBox(width: 10),

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
                    style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: Colors.white),
                  ),
                  if (!isUser) ...[
                    const SizedBox(width: 8),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                      decoration: BoxDecoration(
                        color: const Color(0xFF22262E),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        widget.controller.activeModel.value,
                        style: const TextStyle(fontSize: 10, color: Color(0xFF94A3B8), fontFamily: 'Consolas'),
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
                  color: isUser ? const Color(0xFF1B1E24) : const Color(0xFF14161A),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: const Color(0xFF242832)),
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
                          color: const Color(0xFF191C22),
                          borderRadius: BorderRadius.circular(6),
                          border: const Border(left: BorderSide(color: Color(0xFF00D2FF), width: 2.5)),
                        ),
                        child: Text(
                          msg.thinking!,
                          style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF94A3B8)),
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
                              color: const Color(0xFF191C22),
                              borderRadius: BorderRadius.circular(4),
                              border: Border.all(color: const Color(0xFF262A34)),
                            ),
                            child: Row(
                              children: [
                                const Icon(Icons.terminal, size: 13, color: Color(0xFF00D2FF)),
                                const SizedBox(width: 8),
                                Text(
                                  tool.name,
                                  style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Colors.white),
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
                          color: const Color(0xFF191C22),
                          borderRadius: BorderRadius.circular(6),
                          border: Border.all(color: const Color(0xFF262A34)),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            const Icon(Icons.chevron_right, size: 14, color: Color(0xFF94A3B8)),
                            const SizedBox(width: 4),
                            Text('${DesktopI18n.oneFileChanged} ', style: const TextStyle(fontSize: 11, color: Color(0xFFCBD5E1))),
                            const Text('+9', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFF10B981))),
                            const Text(' -0', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', fontWeight: FontWeight.bold, color: Color(0xFFEF4444))),
                            const SizedBox(width: 14),
                            InkWell(
                              onTap: () {},
                              child: Row(
                                children: [
                                  const Icon(Icons.undo, size: 12, color: Color(0xFF94A3B8)),
                                  const SizedBox(width: 4),
                                  Text(DesktopI18n.undo, style: const TextStyle(fontSize: 11, color: Color(0xFF94A3B8))),
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
        style: const TextStyle(fontSize: 13, height: 1.5, color: Color(0xFFE2E8F0)),
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
              style: const TextStyle(fontSize: 13, height: 1.5, color: Color(0xFFE2E8F0)),
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
              color: const Color(0xFF191C22),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: const Color(0xFF282F3B)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                  decoration: const BoxDecoration(
                    color: Color(0xFF20252F),
                    borderRadius: BorderRadius.vertical(top: Radius.circular(7)),
                  ),
                  child: Row(
                    children: [
                      const Icon(Icons.description_outlined, size: 14, color: Color(0xFF00D2FF)),
                      const SizedBox(width: 8),
                      Text(
                        lang,
                        style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFFCBD5E1)),
                      ),
                      const Spacer(),
                      InkWell(
                        onTap: () {
                          Clipboard.setData(ClipboardData(text: codeBody));
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text(DesktopI18n.commandCopied), duration: const Duration(seconds: 1)),
                          );
                        },
                        child: const Padding(
                          padding: EdgeInsets.all(2),
                          child: Icon(Icons.copy, size: 13, color: Color(0xFF94A3B8)),
                        ),
                      ),
                      const SizedBox(width: 8),
                      InkWell(
                        onTap: () {
                          widget.controller.isTerminalOpen.value = true;
                        },
                        child: const Padding(
                          padding: EdgeInsets.all(2),
                          child: Icon(Icons.play_arrow_outlined, size: 16, color: Color(0xFF94A3B8)),
                        ),
                      ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.all(12),
                  child: SelectableText(
                    codeBody,
                    style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Color(0xFFE2E8F0)),
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

  // Helpers
  Widget _buildProjectChip() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: const Color(0xFF20242D),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xFF2C323E)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: const [
          Icon(Icons.folder_outlined, size: 12, color: Color(0xFF00D2FF)),
          SizedBox(width: 6),
          Text('deepseek-harness-mas...', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Colors.white)),
          SizedBox(width: 4),
          Icon(Icons.keyboard_arrow_down, size: 13, color: Color(0xFF94A3B8)),
        ],
      ),
    );
  }

  Widget _buildBranchChip() {
    return InkWell(
      onTap: () => setState(() => isGitToolsOpen = !isGitToolsOpen),
      borderRadius: BorderRadius.circular(6),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        decoration: BoxDecoration(
          color: const Color(0xFF20242D),
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: isGitToolsOpen ? const Color(0xFF00D2FF) : const Color(0xFF2C323E)),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: const [
            Icon(FontAwesomeIcons.codeBranch, size: 10, color: Color(0xFF00D2FF)),
            SizedBox(width: 6),
            Text('desktop-bran...', style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF00D2FF))),
            SizedBox(width: 4),
            Icon(Icons.keyboard_arrow_down, size: 13, color: Color(0xFF94A3B8)),
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
