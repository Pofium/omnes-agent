// Automations View matching ZCode Screenshot 1:
// - Header with Title and Subtitle
// - Scheduled tasks empty-state container with 'Create scheduled task' and 'Create idle-time task'
// - 'Keep your computer awake while OmnesAgent is running a chat' setting
// - 'Idle-time task template' cards
// - 'Scheduled task template' cards with cron intervals

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';

class AutomationsView extends StatefulWidget {
  final VoidCallback? onBackToWorkspace;

  const AutomationsView({
    super.key,
    this.onBackToWorkspace,
  });

  @override
  State<AutomationsView> createState() => _AutomationsViewState();
}

class _AutomationsViewState extends State<AutomationsView> {
  bool isKeepAwakeEnabled = true;

  // Active scheduled tasks list
  final List<Map<String, dynamic>> scheduledTasks = [];

  void _showCreateTaskDialog({
    String? initialTitle,
    String? initialPrompt,
    String? initialSchedule,
    bool isIdleTime = false,
  }) {
    final titleController = TextEditingController(text: initialTitle ?? '');
    final promptController = TextEditingController(text: initialPrompt ?? '');
    final scheduleController = TextEditingController(text: initialSchedule ?? (isIdleTime ? 'Soonest available' : 'Daily at 10:00'));

    showDialog(
      context: context,
      builder: (ctx) {
        return Dialog(
          backgroundColor: Colors.transparent,
          insetPadding: const EdgeInsets.symmetric(horizontal: 40, vertical: 24),
          child: Container(
            width: 580,
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: const Color(0xFF1B1D23),
              borderRadius: BorderRadius.circular(14),
              border: Border.all(color: const Color(0xFF2C323E)),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.7),
                  blurRadius: 30,
                  offset: const Offset(0, 12),
                ),
              ],
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Icon(
                      isIdleTime ? FontAwesomeIcons.moon : FontAwesomeIcons.clock,
                      size: 16,
                      color: const Color(0xFF00D2FF),
                    ),
                    const SizedBox(width: 10),
                    Text(
                      isIdleTime ? 'Новая idle-time задача' : 'Новая cron/scheduled задача',
                      style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: Colors.white),
                    ),
                    const Spacer(),
                    IconButton(
                      icon: const Icon(Icons.close, size: 16, color: Color(0xFF94A3B8)),
                      onPressed: () => Navigator.of(ctx).pop(),
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
                    ),
                  ],
                ),
                const SizedBox(height: 16),

                const Text('Название задачи', style: TextStyle(fontSize: 12, color: Color(0xFF94A3B8), fontWeight: FontWeight.w500)),
                const SizedBox(height: 6),
                TextField(
                  controller: titleController,
                  style: const TextStyle(fontSize: 13, color: Colors.white),
                  decoration: InputDecoration(
                    hintText: 'Например: Standup Git Summary',
                    hintStyle: const TextStyle(fontSize: 13, color: Color(0xFF64748B)),
                    filled: true,
                    fillColor: const Color(0xFF131519),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF2B313D))),
                    focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF00D2FF))),
                  ),
                ),
                const SizedBox(height: 14),

                const Text('Промпт / Инструкция агенту', style: TextStyle(fontSize: 12, color: Color(0xFF94A3B8), fontWeight: FontWeight.w500)),
                const SizedBox(height: 6),
                TextField(
                  controller: promptController,
                  maxLines: 4,
                  style: const TextStyle(fontSize: 12, color: Colors.white),
                  decoration: InputDecoration(
                    hintText: 'Опишите, что именно агент должен проверить или сформировать...',
                    hintStyle: const TextStyle(fontSize: 12, color: Color(0xFF64748B)),
                    filled: true,
                    fillColor: const Color(0xFF131519),
                    contentPadding: const EdgeInsets.all(12),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF2B313D))),
                    focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF00D2FF))),
                  ),
                ),
                const SizedBox(height: 14),

                const Text('Расписание выполнения', style: TextStyle(fontSize: 12, color: Color(0xFF94A3B8), fontWeight: FontWeight.w500)),
                const SizedBox(height: 6),
                TextField(
                  controller: scheduleController,
                  style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Color(0xFF00D2FF)),
                  decoration: InputDecoration(
                    hintText: isIdleTime ? 'Soonest available' : 'Every weekday at 09:00',
                    hintStyle: const TextStyle(fontSize: 12, color: Color(0xFF64748B)),
                    filled: true,
                    fillColor: const Color(0xFF131519),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF2B313D))),
                    focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF00D2FF))),
                  ),
                ),
                const SizedBox(height: 24),

                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    TextButton(
                      onPressed: () => Navigator.of(ctx).pop(),
                      child: const Text('Отмена', style: TextStyle(color: Color(0xFF94A3B8))),
                    ),
                    const SizedBox(width: 10),
                    ElevatedButton(
                      onPressed: () {
                        if (titleController.text.trim().isNotEmpty) {
                          setState(() {
                            scheduledTasks.add({
                              'title': titleController.text.trim(),
                              'prompt': promptController.text.trim(),
                              'schedule': scheduleController.text.trim(),
                              'isIdleTime': isIdleTime,
                              'active': true,
                            });
                          });
                          Navigator.of(ctx).pop();
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text('Задача "${titleController.text.trim()}" успешно добавлена в Automations!'),
                              backgroundColor: const Color(0xFF1E293B),
                              behavior: SnackBarBehavior.floating,
                            ),
                          );
                        }
                      },
                      style: ElevatedButton.styleFrom(
                        backgroundColor: const Color(0xFF00D2FF),
                        foregroundColor: Colors.black,
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                      ),
                      child: const Text('Запланировать', style: TextStyle(fontWeight: FontWeight.bold)),
                    ),
                  ],
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF141619),
      body: Column(
        children: [
          // Top Window Controls Bar
          Container(
            height: 38,
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Row(
              children: [
                const Spacer(),
                _buildWinButton(Icons.remove, () {}),
                const SizedBox(width: 6),
                _buildWinButton(Icons.crop_square, () {}),
                const SizedBox(width: 6),
                _buildWinButton(Icons.close, () {}, isClose: true),
              ],
            ),
          ),

          // Main Scrollable Automations Content
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.fromLTRB(48, 12, 48, 48),
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 980),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Header
                    const Text(
                      'Automations',
                      style: TextStyle(
                        fontSize: 26,
                        fontWeight: FontWeight.bold,
                        color: Colors.white,
                        letterSpacing: -0.3,
                      ),
                    ),
                    const SizedBox(height: 6),
                    const Text(
                      'Schedule recurring tasks or queue background work that runs during idle time.',
                      style: TextStyle(
                        fontSize: 13,
                        color: Color(0xFF94A3B8),
                      ),
                    ),
                    const SizedBox(height: 24),

                    // Top Empty State Card or Active Tasks List
                    _buildScheduledTasksCard(),
                    const SizedBox(height: 16),

                    // Keep Awake Row (Image 1 match)
                    _buildKeepAwakeSettingRow(),
                    const SizedBox(height: 36),

                    // Section 1: Idle-time task template
                    const Text(
                      'Idle-time task template',
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.bold,
                        color: Color(0xFF94A3B8),
                        letterSpacing: 0.2,
                      ),
                    ),
                    const SizedBox(height: 14),
                    _buildIdleTimeTemplatesGrid(),
                    const SizedBox(height: 36),

                    // Section 2: Scheduled task template
                    const Text(
                      'Scheduled task template',
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.bold,
                        color: Color(0xFF94A3B8),
                        letterSpacing: 0.2,
                      ),
                    ),
                    const SizedBox(height: 14),
                    _buildScheduledTemplatesGrid(),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  // ==========================================
  // TOP SCHEDULED TASKS CONTAINER (Image 1 match)
  // ==========================================
  Widget _buildScheduledTasksCard() {
    if (scheduledTasks.isNotEmpty) {
      return Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: const Color(0xFF171A20),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: const Color(0xFF262B35)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(FontAwesomeIcons.circleCheck, size: 14, color: Color(0xFF00D2FF)),
                const SizedBox(width: 8),
                Text(
                  'Активные задачи (${scheduledTasks.length})',
                  style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: Colors.white),
                ),
                const Spacer(),
                TextButton.icon(
                  onPressed: () => _showCreateTaskDialog(),
                  icon: const Icon(Icons.add, size: 14, color: Color(0xFF00D2FF)),
                  label: const Text('Добавить', style: TextStyle(color: Color(0xFF00D2FF), fontSize: 12)),
                ),
              ],
            ),
            const Divider(color: Color(0xFF262B35), height: 16),
            ...scheduledTasks.map((t) {
              return Padding(
                padding: const EdgeInsets.symmetric(vertical: 6),
                child: Row(
                  children: [
                    Icon(
                      t['isIdleTime'] == true ? FontAwesomeIcons.moon : FontAwesomeIcons.clock,
                      size: 13,
                      color: const Color(0xFF94A3B8),
                    ),
                    const SizedBox(width: 10),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            t['title'] as String,
                            style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600, color: Colors.white),
                          ),
                          Text(
                            t['schedule'] as String,
                            style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF00D2FF)),
                          ),
                        ],
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.delete_outline, size: 15, color: Color(0xFF64748B)),
                      onPressed: () {
                        setState(() => scheduledTasks.remove(t));
                      },
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
                    ),
                  ],
                ),
              );
            }),
          ],
        ),
      );
    }

    // Empty state container matching Image 1
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(vertical: 42, horizontal: 20),
      decoration: BoxDecoration(
        color: const Color(0xFF171A20),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xFF262B35)),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text(
            'No scheduled tasks yet.',
            style: TextStyle(
              fontSize: 13,
              color: Color(0xFF64748B),
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 20),

          // Two CTA Buttons
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              // Create scheduled task (white card button with dropdown arrow)
              InkWell(
                onTap: () => _showCreateTaskDialog(isIdleTime: false),
                borderRadius: BorderRadius.circular(8),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  decoration: BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: const [
                      Text(
                        'Create scheduled task',
                        style: TextStyle(
                          fontSize: 12.5,
                          fontWeight: FontWeight.w600,
                          color: Color(0xFF0F172A),
                        ),
                      ),
                      SizedBox(width: 6),
                      Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF0F172A)),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 12),

              // Create idle-time task (dark card button)
              InkWell(
                onTap: () => _showCreateTaskDialog(isIdleTime: true),
                borderRadius: BorderRadius.circular(8),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  decoration: BoxDecoration(
                    color: const Color(0xFF232731),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: const Color(0xFF333846)),
                  ),
                  child: const Text(
                    'Create idle-time task',
                    style: TextStyle(
                      fontSize: 12.5,
                      fontWeight: FontWeight.w500,
                      color: Color(0xFFE2E8F0),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  // ==========================================
  // KEEP AWAKE SETTING ROW (Image 1 match)
  // ==========================================
  Widget _buildKeepAwakeSettingRow() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
      decoration: BoxDecoration(
        color: const Color(0xFF16181E),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          const Icon(Icons.info_outline, size: 15, color: Color(0xFF64748B)),
          const SizedBox(width: 10),
          const Expanded(
            child: Text(
              'Keep your computer awake while OmnesAgent is running a chat.',
              style: TextStyle(fontSize: 12, color: Color(0xFF94A3B8)),
            ),
          ),
          Switch(
            value: isKeepAwakeEnabled,
            onChanged: (val) => setState(() => isKeepAwakeEnabled = val),
            activeColor: const Color(0xFF00D2FF),
            activeTrackColor: const Color(0xFF00D2FF).withOpacity(0.3),
            inactiveThumbColor: const Color(0xFF64748B),
            inactiveTrackColor: const Color(0xFF242934),
          ),
        ],
      ),
    );
  }

  // ==========================================
  // IDLE-TIME TASK TEMPLATES (Image 1 match)
  // ==========================================
  Widget _buildIdleTimeTemplatesGrid() {
    return LayoutBuilder(
      builder: (context, constraints) {
        final isWide = constraints.maxWidth > 640;
        final cardWidth = isWide ? (constraints.maxWidth - 16) / 2 : constraints.maxWidth;

        final templates = [
          {
            'icon': FontAwesomeIcons.listCheck,
            'title': 'Standup Git Summary',
            'desc':
                "Summarize this week's git activity into a Friday standup: notable commits, merged PRs, and what changed. Keep it concise.",
            'badge': 'Soonest available',
          },
          {
            'icon': FontAwesomeIcons.chartLine,
            'title': 'CI Failures & Flaky Test Report',
            'desc':
                'Scan recent CI runs, list failing and flaky tests with likely causes, and propose fixes ranked by impact.',
            'badge': 'Soonest available',
          },
          {
            'icon': FontAwesomeIcons.fileLines,
            'title': 'Documentation sync check',
            'desc':
                'Using the current implementation and recent commits as evidence, check whether README files, docs, configuration guidance, and usage examples are outdated or inconsistent with the code. Modify only content that can be directly verified from code, configuration, or commit history; do not infer unverified behavior, and preserve the existing document structure, terminology, and writing style. When finished, list the files changed and the evidence for each change. If no update is needed, report the scope checked, the evidence reviewed, and the conclusion.',
            'badge': 'Soonest available',
          },
        ];

        return Wrap(
          spacing: 16,
          runSpacing: 16,
          children: templates.map((t) {
            return SizedBox(
              width: cardWidth,
              child: _buildTemplateCard(
                icon: t['icon'] as IconData,
                title: t['title'] as String,
                desc: t['desc'] as String,
                badge: t['badge'] as String,
                isIdleTime: true,
              ),
            );
          }).toList(),
        );
      },
    );
  }

  // ==========================================
  // SCHEDULED TASK TEMPLATES (Image 1 match)
  // ==========================================
  Widget _buildScheduledTemplatesGrid() {
    return LayoutBuilder(
      builder: (context, constraints) {
        final isWide = constraints.maxWidth > 640;
        final cardWidth = isWide ? (constraints.maxWidth - 16) / 2 : constraints.maxWidth;

        final templates = [
          {
            'icon': FontAwesomeIcons.sun,
            'title': 'Morning dev brief',
            'desc':
                'Summarize commits, module changes, CI status, and follow-ups since the previous workday, then produce no more than five stand-up-ready bullets. Perform read-only analysis using only verifiable repository facts; state when evidence is insufficient, do not speculate, and do not modify code or external state.',
            'badge': 'Every weekday at 09:00',
          },
          {
            'icon': FontAwesomeIcons.shieldHalved,
            'title': 'Risk scan',
            'desc':
                'Inspect code changes from the last 24 hours for high-confidence risks involving runtime failures, data loss, authorization bypasses, resource leaks, or cross-platform compatibility, and attach code and commit/diff evidence. Perform read-only analysis using only verifiable repository facts; state when evidence is insufficient, do not speculate, and do not modify code or external state.',
            'badge': 'Daily at 10:00',
          },
          {
            'icon': FontAwesomeIcons.rocket,
            'title': 'Release brief',
            'desc':
                'Organize PRs and commits merged this week into Features, Fixes, Experience improvements, and Engineering improvements, then produce both a team brief and concise user-facing release notes. Perform read-only analysis using only verifiable repository facts; state when evidence is insufficient, do not speculate, and do not modify code or external state.',
            'badge': 'Weekly on Fri at 16:00',
          },
          {
            'icon': FontAwesomeIcons.bookBookmark,
            'title': 'Documentation sync check',
            'desc':
                'Compare code, configuration, API, and documentation changes from the last seven days. Identify high-confidence cases where public behavior changed without matching documentation, attach file paths and commit/diff evidence, and name the documentation locations and key points that should be updated. Perform read-only analysis using only verifiable repository facts; state when evidence is insufficient, do not speculate, and do not modify code or external state.',
            'badge': 'Weekly on Wed at 15:00',
          },
        ];

        return Wrap(
          spacing: 16,
          runSpacing: 16,
          children: templates.map((t) {
            return SizedBox(
              width: cardWidth,
              child: _buildTemplateCard(
                icon: t['icon'] as IconData,
                title: t['title'] as String,
                desc: t['desc'] as String,
                badge: t['badge'] as String,
                isIdleTime: false,
              ),
            );
          }).toList(),
        );
      },
    );
  }

  // ==========================================
  // SINGLE TEMPLATE CARD
  // ==========================================
  Widget _buildTemplateCard({
    required IconData icon,
    required String title,
    required String desc,
    required String badge,
    required bool isIdleTime,
  }) {
    return InkWell(
      onTap: () {
        _showCreateTaskDialog(
          initialTitle: title,
          initialPrompt: desc,
          initialSchedule: badge,
          isIdleTime: isIdleTime,
        );
      },
      borderRadius: BorderRadius.circular(10),
      hoverColor: const Color(0xFF1E232C),
      child: Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: const Color(0xFF181B22),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: const Color(0xFF262B36)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, size: 13, color: const Color(0xFF00D2FF)),
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
            const SizedBox(height: 10),
            Text(
              desc,
              style: const TextStyle(
                fontSize: 11.5,
                color: Color(0xFF94A3B8),
                height: 1.45,
              ),
            ),
            const SizedBox(height: 16),
            Text(
              badge,
              style: const TextStyle(
                fontSize: 11,
                fontFamily: 'Consolas',
                color: Color(0xFF64748B),
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildWinButton(IconData icon, VoidCallback onTap, {bool isClose = false}) {
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
}
