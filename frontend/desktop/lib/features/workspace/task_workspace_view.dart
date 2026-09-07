// ZCode ADE Task Workspace View matching official ZCode Desktop screenshots.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';

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

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFF16181D),
      child: Obx(() {
        final hasMessages = widget.controller.messages.isNotEmpty;

        return Column(
          children: [
            // Top Navigation Bar
            _buildTopNavBar(hasMessages),

            // Main Body: Either Empty State (New Task) or Chat Timeline
            Expanded(
              child: hasMessages
                  ? _buildChatTimeline()
                  : _buildZCodeNewTaskScreen(),
            ),

            // Bottom Floating Composer when messages exist
            if (hasMessages)
              _buildBottomFloatingComposer(),
          ],
        );
      }),
    );
  }

  // ==========================================
  // TOP NAV BAR MATCHING ZCODE
  // ==========================================
  Widget _buildTopNavBar(bool hasMessages) {
    return Container(
      height: 48,
      padding: const EdgeInsets.symmetric(horizontal: 20),
      decoration: const BoxDecoration(
        color: Color(0xFF16181D),
        border: Border(bottom: BorderSide(color: Color(0xFF22262E), width: 0.8)),
      ),
      child: Row(
        children: [
          if (hasMessages) ...[
            Flexible(
              child: Text(
                widget.controller.activeTaskTitle.value,
                style: const TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: Colors.white,
                ),
                overflow: TextOverflow.ellipsis,
                maxLines: 1,
              ),
            ),
            const SizedBox(width: 8),
            _buildProjectChip(),
            const SizedBox(width: 6),
            _buildBranchChip(),
            const SizedBox(width: 6),
            const Icon(Icons.more_horiz, size: 16, color: Color(0xFF94A3B8)),
          ],

          const Spacer(),

          // Right Window Actions
          // User profile avatar badge
          Container(
            width: 26,
            height: 26,
            decoration: BoxDecoration(
              color: const Color(0xFF00D2FF).withOpacity(0.2),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.4)),
            ),
            child: const Center(
              child: Text(
                'O',
                style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Color(0xFF00D2FF)),
              ),
            ),
          ),
          const SizedBox(width: 10),

          // Terminal quick toggle (>_)
          InkWell(
            onTap: widget.onToggleTerminal,
            borderRadius: BorderRadius.circular(4),
            child: Container(
              padding: const EdgeInsets.all(5),
              decoration: BoxDecoration(
                color: const Color(0xFF22262E),
                borderRadius: BorderRadius.circular(5),
              ),
              child: const Icon(FontAwesomeIcons.terminal, size: 12, color: Color(0xFF94A3B8)),
            ),
          ),
          const SizedBox(width: 8),

          // Inspector / Tools Drawer toggle ([|])
          InkWell(
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
        ],
      ),
    );
  }

  // ==========================================
  // ZCODE NEW TASK / WELCOME SCREEN
  // ==========================================
  Widget _buildZCodeNewTaskScreen() {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 20),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 780),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              // Subtle background watermark & Title
              const SizedBox(height: 10),
              const Text(
                'Start a new task in the omnes-agent project',
                style: TextStyle(
                  fontSize: 28,
                  fontWeight: FontWeight.w400,
                  fontFamily: 'Georgia',
                  letterSpacing: 0.4,
                  color: Colors.white,
                ),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 32),

              // Central Composer Card (Exact ZCode match)
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
                      const Expanded(
                        child: Text(
                          'New feature for subscribers: Create "Idle-time task", We will complete your assigned task for free during periods of surplus computing power.',
                          style: TextStyle(
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

              // 3 Quick Action Cards matching ZCode
              Row(
                children: [
                  Expanded(
                    child: _buildTemplateCard(
                      title: 'Standup Git Summary',
                      desc: 'A Friday summary of what happened this week.',
                      onTap: () {
                        widget.controller.inputController.text = 'Сформируй Git Standup summary за последнюю неделю.';
                      },
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _buildTemplateCard(
                      title: 'CI Failures & Flaky Test Report',
                      desc: 'A report on recent CI failures, flaky tests, and likely causes.',
                      onTap: () {
                        widget.controller.inputController.text = 'Проанализируй недавние падения CI тестов и сформируй отчет.';
                      },
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _buildTemplateCard(
                      title: 'Customize',
                      desc: 'Skip the template and tell it directly what you want to do.',
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
                    ? 'Ask ZCode, type @ to add files, / for commands, \$ for skills, # to link chats'
                    : 'Ask for follow-up changes...',
                hintStyle: const TextStyle(fontSize: 13, color: Color(0xFF6B7280)),
                border: InputBorder.none,
                isDense: true,
                contentPadding: EdgeInsets.zero,
              ),
              onSubmitted: (_) => widget.controller.sendMessage(),
            ),
          ),

          // 4. Bottom Controls Row: [+] [✋ Mode ⌵] ... [Model ⌵] [🧠 Max ⌵] [↑]
          Padding(
            padding: const EdgeInsets.only(left: 12, right: 12, bottom: 10),
            child: Row(
              children: [
                // [+] Add Context Button & Popup
                _buildAddMenuButton(),
                const SizedBox(width: 8),

                // [✋ Permission Mode ⌵]
                _buildPermissionModeMenuButton(),

                const Spacer(),

                // [Model ⌵]
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

                    SelectableText(
                      msg.text,
                      style: const TextStyle(fontSize: 13, height: 1.5, color: Color(0xFFE2E8F0)),
                    ),

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
                                const Text(
                                  '✓ done',
                                  style: TextStyle(fontSize: 10, color: Color(0xFF10B981)),
                                ),
                              ],
                            ),
                          )),
                    ],
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  // ==========================================
  // ZCODE POPUP MENUS
  // ==========================================

  // 1. [+] Add Menu
  Widget _buildAddMenuButton() {
    return PopupMenuButton<String>(
      tooltip: 'Add context (@file, #chat, /cmd)',
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
        _buildPopupItem('attachment', Icons.attach_file, 'Add attachment'),
        _buildPopupItem('mention', Icons.alternate_email, 'Insert @ mention'),
        _buildPopupItem('chat', Icons.chat_bubble_outline, 'Insert # conversation'),
        _buildPopupItem('command', Icons.terminal, 'Insert / command'),
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

  // 2. [✋ Ask before changes ⌵] Popup Menu
  Widget _buildPermissionModeMenuButton() {
    return Obx(() {
      final mode = widget.controller.permissionMode.value;

      return PopupMenuButton<PermissionMode>(
        tooltip: 'Permission Mode',
        offset: const Offset(0, -220),
        color: const Color(0xFF22252A),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
        onSelected: widget.controller.setPermissionMode,
        itemBuilder: (context) => [
          _buildPermissionItem(
            PermissionMode.askBeforeChanges,
            Icons.pan_tool_outlined,
            'Ask before changes',
            'Ask before file changes.',
            mode == PermissionMode.askBeforeChanges,
          ),
          _buildPermissionItem(
            PermissionMode.editAutomatically,
            Icons.shield_outlined,
            'Edit automatically',
            'Edit files automatically.',
            mode == PermissionMode.editAutomatically,
          ),
          _buildPermissionItem(
            PermissionMode.planMode,
            Icons.calendar_today_outlined,
            'Plan mode',
            'Plan before editing.',
            mode == PermissionMode.planMode,
          ),
          _buildPermissionItem(
            PermissionMode.fullAccess,
            Icons.security,
            'Full access',
            'Run with fewer confirmations.',
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

  // 3. [Model ⌵] Popup Menu
  Widget _buildModelMenuButton() {
    return Obx(() {
      final active = widget.controller.activeModel.value;

      return PopupMenuButton<String>(
        tooltip: 'Select Model',
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
          const PopupMenuItem(
            enabled: false,
            height: 28,
            child: Text('Z.ai API', style: TextStyle(fontSize: 10, color: Color(0xFF64748B), fontWeight: FontWeight.bold)),
          ),
          _buildModelItem('GLM-5.3', active == 'GLM-5.3'),
          _buildModelItem('Claude 3.5 Sonnet', active == 'Claude 3.5 Sonnet'),
          _buildModelItem('DeepSeek V3', active == 'DeepSeek V3'),
          _buildModelItem('Local Ollama', active == 'Local Ollama'),
          const PopupMenuDivider(height: 1),
          const PopupMenuItem(
            value: 'manage',
            height: 36,
            child: Row(
              children: [
                Icon(Icons.settings_outlined, size: 14, color: Color(0xFF94A3B8)),
                SizedBox(width: 8),
                Text('Manage models', style: TextStyle(fontSize: 12, color: Color(0xFFCBD5E1))),
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
              Text(active, style: const TextStyle(fontSize: 12, color: Colors.white, fontWeight: FontWeight.w500)),
              const SizedBox(width: 4),
              const Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF94A3B8)),
            ],
          ),
        ),
      );
    });
  }

  // 4. [🧠 Thought Level ⌵] Popup Menu
  Widget _buildThoughtLevelMenuButton() {
    return Obx(() {
      final level = widget.controller.thoughtLevel.value;

      return PopupMenuButton<String>(
        tooltip: 'Thought Level',
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

  // 5. Send button
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

  // Helper popup item builders
  PopupMenuItem<String> _buildPopupItem(String val, IconData icon, String text) {
    return PopupMenuItem(
      value: val,
      height: 38,
      child: Row(
        children: [
          Icon(icon, size: 14, color: const Color(0xFF94A3B8)),
          const SizedBox(width: 10),
          Text(text, style: const TextStyle(fontSize: 13, color: Colors.white)),
        ],
      ),
    );
  }

  PopupMenuItem<PermissionMode> _buildPermissionItem(
    PermissionMode mode,
    IconData icon,
    String title,
    String subtitle,
    bool isSelected,
  ) {
    return PopupMenuItem(
      value: mode,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 16, color: isSelected ? const Color(0xFF00D2FF) : const Color(0xFF94A3B8)),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(title, style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: Colors.white)),
                const SizedBox(height: 2),
                Text(subtitle, style: const TextStyle(fontSize: 11, color: Color(0xFF8B949E))),
              ],
            ),
          ),
          if (isSelected)
            const Icon(Icons.check, size: 16, color: Color(0xFF00D2FF)),
        ],
      ),
    );
  }

  PopupMenuItem<String> _buildModelItem(String model, bool isSelected) {
    return PopupMenuItem(
      value: model,
      height: 36,
      child: Row(
        children: [
          Text(model, style: const TextStyle(fontSize: 13, color: Colors.white)),
          const Spacer(),
          if (isSelected)
            const Icon(Icons.check, size: 16, color: Color(0xFF00D2FF)),
        ],
      ),
    );
  }

  PopupMenuItem<String> _buildThoughtItem(String level, bool isSelected) {
    return PopupMenuItem(
      value: level,
      height: 34,
      child: Row(
        children: [
          Text(level, style: const TextStyle(fontSize: 13, color: Colors.white)),
          const Spacer(),
          if (isSelected)
            const Icon(Icons.check, size: 16, color: Color(0xFF00D2FF)),
        ],
      ),
    );
  }

  Widget _buildProjectChip() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: const Color(0xFF22262E),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: const [
          Icon(FontAwesomeIcons.folder, size: 11, color: Color(0xFF00D2FF)),
          SizedBox(width: 6),
          Text('omnes-agent', style: TextStyle(fontSize: 12, color: Colors.white, fontWeight: FontWeight.w500)),
          SizedBox(width: 4),
          Icon(Icons.keyboard_arrow_down, size: 13, color: Color(0xFF94A3B8)),
        ],
      ),
    );
  }

  Widget _buildBranchChip() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: const Color(0xFF22262E),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: const [
          Icon(FontAwesomeIcons.codeBranch, size: 11, color: Color(0xFF94A3B8)),
          SizedBox(width: 6),
          Text('main', style: TextStyle(fontSize: 12, color: Colors.white, fontWeight: FontWeight.w500)),
          SizedBox(width: 4),
          Icon(Icons.keyboard_arrow_down, size: 13, color: Color(0xFF94A3B8)),
        ],
      ),
    );
  }
}
