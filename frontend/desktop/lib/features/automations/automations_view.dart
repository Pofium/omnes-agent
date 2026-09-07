// Automations View for OmnesAgent ADE:
// - Header with Title and Subtitle
// - Scheduled tasks empty-state container with 'Create scheduled task' and 'Create idle-time task'
// - 'Keep your computer awake while OmnesAgent is running a chat' setting
// - 'Idle-time task template' cards
// - 'Scheduled task template' cards with cron intervals

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import '../../utils/desktop_i18n.dart';

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
    final scheduleController = TextEditingController(text: initialSchedule ?? (isIdleTime ? DesktopI18n.soonestAvailable : 'Daily at 10:00'));

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
                      isIdleTime
                          ? DesktopI18n.tr('Новая idle-time задача', 'New idle-time task')
                          : DesktopI18n.tr('Новая задача по расписанию', 'New scheduled task'),
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

                Text(DesktopI18n.taskName, style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8), fontWeight: FontWeight.w500)),
                const SizedBox(height: 6),
                TextField(
                  controller: titleController,
                  style: const TextStyle(fontSize: 13, color: Colors.white),
                  decoration: InputDecoration(
                    hintText: DesktopI18n.tr('Например: Standup Git Summary', 'E.g.: Standup Git Summary'),
                    hintStyle: const TextStyle(fontSize: 13, color: Color(0xFF64748B)),
                    filled: true,
                    fillColor: const Color(0xFF131519),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF2B313D))),
                    focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF00D2FF))),
                  ),
                ),
                const SizedBox(height: 14),

                Text(DesktopI18n.taskPrompt, style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8), fontWeight: FontWeight.w500)),
                const SizedBox(height: 6),
                TextField(
                  controller: promptController,
                  maxLines: 4,
                  style: const TextStyle(fontSize: 12, color: Colors.white),
                  decoration: InputDecoration(
                    hintText: DesktopI18n.tr('Опишите, что именно агент должен проверить или сформировать...', 'Describe what the agent should verify or generate...'),
                    hintStyle: const TextStyle(fontSize: 12, color: Color(0xFF64748B)),
                    filled: true,
                    fillColor: const Color(0xFF131519),
                    contentPadding: const EdgeInsets.all(12),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF2B313D))),
                    focusedBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: const BorderSide(color: Color(0xFF00D2FF))),
                  ),
                ),
                const SizedBox(height: 14),

                Text(DesktopI18n.taskSchedule, style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8), fontWeight: FontWeight.w500)),
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
                      child: Text(DesktopI18n.cancel, style: const TextStyle(color: Color(0xFF94A3B8))),
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
                              content: Text(
                                DesktopI18n.tr(
                                  'Задача "${titleController.text.trim()}" успешно добавлена в Automations!',
                                  'Task "${titleController.text.trim()}" added to Automations!',
                                ),
                              ),
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
                      child: Text(DesktopI18n.scheduleAction, style: const TextStyle(fontWeight: FontWeight.bold)),
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
    return Obx(() {
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
                      Text(
                        DesktopI18n.automationsTitle,
                        style: const TextStyle(
                          fontSize: 26,
                          fontWeight: FontWeight.bold,
                          color: Colors.white,
                          letterSpacing: -0.3,
                        ),
                      ),
                      const SizedBox(height: 6),
                      Text(
                        DesktopI18n.automationsSubtitle,
                        style: const TextStyle(
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
                      Text(
                        DesktopI18n.idleTimeTemplates,
                        style: const TextStyle(
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
                      Text(
                        DesktopI18n.scheduledTemplates,
                        style: const TextStyle(
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
    });
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
                  '${DesktopI18n.activeTasks} (${scheduledTasks.length})',
                  style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: Colors.white),
                ),
                const Spacer(),
                TextButton.icon(
                  onPressed: () => _showCreateTaskDialog(),
                  icon: const Icon(Icons.add, size: 14, color: Color(0xFF00D2FF)),
                  label: Text(DesktopI18n.add, style: const TextStyle(color: Color(0xFF00D2FF), fontSize: 12)),
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
          Text(
            DesktopI18n.noScheduledTasks,
            style: const TextStyle(
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
                    children: [
                      Text(
                        DesktopI18n.createScheduledTask,
                        style: const TextStyle(
                          fontSize: 12.5,
                          fontWeight: FontWeight.w600,
                          color: Color(0xFF0F172A),
                        ),
                      ),
                      const SizedBox(width: 6),
                      const Icon(Icons.keyboard_arrow_down, size: 14, color: Color(0xFF0F172A)),
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
                  child: Text(
                    DesktopI18n.createIdleTask,
                    style: const TextStyle(
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
          Expanded(
            child: Text(
              DesktopI18n.keepAwakeText,
              style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8)),
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
            'title': DesktopI18n.tr('Итоги недели в Git (Standup Summary)', 'Standup Git Summary'),
            'desc': DesktopI18n.tr(
              "Сформировать краткую сводку активности git за неделю к пятничному стендапу: главные коммиты, смерженные PR и изменения.",
              "Summarize this week's git activity into a Friday standup: notable commits, merged PRs, and what changed. Keep it concise.",
            ),
            'badge': DesktopI18n.soonestAvailable,
          },
          {
            'icon': FontAwesomeIcons.chartLine,
            'title': DesktopI18n.tr('Отчет о сбоях CI и нестабильных тестах', 'CI Failures & Flaky Test Report'),
            'desc': DesktopI18n.tr(
              'Просканировать недавние прогоны CI, составить список упавших тестов с вероятными причинами и предложить исправления.',
              'Scan recent CI runs, list failing and flaky tests with likely causes, and propose fixes ranked by impact.',
            ),
            'badge': DesktopI18n.soonestAvailable,
          },
          {
            'icon': FontAwesomeIcons.fileLines,
            'title': DesktopI18n.tr('Синхронизация документации с кодом', 'Documentation sync check'),
            'desc': DesktopI18n.tr(
              'На основе актуального кода и коммитов проверить, не устарели ли файлы README, документация и примеры использования.',
              'Using the current implementation and recent commits as evidence, check whether README files, docs, configuration guidance, and usage examples are outdated or inconsistent with the code. Modify only content that can be directly verified from code, configuration, or commit history; do not infer unverified behavior, and preserve the existing document structure, terminology, and writing style. When finished, list the files changed and the evidence for each change. If no update is needed, report the scope checked, the evidence reviewed, and the conclusion.',
            ),
            'badge': DesktopI18n.soonestAvailable,
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
            'title': DesktopI18n.tr('Утренний брифинг разработчика', 'Morning dev brief'),
            'desc': DesktopI18n.tr(
              'Суммировать коммиты, изменения модулей, статус CI и задачи со вчерашнего дня в виде до пяти тезисов к утреннему созвону.',
              'Summarize commits, module changes, CI status, and follow-ups since the previous workday, then produce no more than five stand-up-ready bullets. Perform read-only analysis using only verifiable repository facts; state when evidence is insufficient, do not speculate, and do not modify code or external state.',
            ),
            'badge': DesktopI18n.tr('Каждый будний день в 09:00', 'Every weekday at 09:00'),
          },
          {
            'icon': FontAwesomeIcons.shieldHalved,
            'title': DesktopI18n.tr('Сканирование рисков и регрессий', 'Risk scan'),
            'desc': DesktopI18n.tr(
              'Проверить изменения за 24 часа на предмет сбоев в рантайме, утечек ресурсов или потери кроссплатформенной совместимости.',
              'Inspect code changes from the last 24 hours for high-confidence risks involving runtime failures, data loss, authorization bypasses, resource leaks, or cross-platform compatibility, and attach code and commit/diff evidence. Perform read-only analysis using only verifiable repository facts; state when evidence is insufficient, do not speculate, and do not modify code or external state.',
            ),
            'badge': DesktopI18n.tr('Ежедневно в 10:00', 'Daily at 10:00'),
          },
          {
            'icon': FontAwesomeIcons.rocket,
            'title': DesktopI18n.tr('Сводка релиза (Release Notes)', 'Release brief'),
            'desc': DesktopI18n.tr(
              'Сгруппировать PR и коммиты недели по фичам, багфиксам и улучшениям, подготовив заметки к релизу для команды и пользователей.',
              'Organize PRs and commits merged this week into Features, Fixes, Experience improvements, and Engineering improvements, then produce both a team brief and concise user-facing release notes. Perform read-only analysis using only verifiable repository facts; state when evidence is insufficient, do not speculate, and do not modify code or external state.',
            ),
            'badge': DesktopI18n.tr('Еженедельно по пятницам в 16:00', 'Weekly on Fri at 16:00'),
          },
          {
            'icon': FontAwesomeIcons.bookBookmark,
            'title': DesktopI18n.tr('Ревизия актуальности документации', 'Documentation sync check'),
            'desc': DesktopI18n.tr(
              'Сравнить изменения в кодовой базе и API за последние 7 дней и составить список документов, требующих обновления.',
              'Compare code, configuration, API, and documentation changes from the last seven days. Identify high-confidence cases where public behavior changed without matching documentation, attach file paths and commit/diff evidence, and name the documentation locations and key points that should be updated. Perform read-only analysis using only verifiable repository facts; state when evidence is insufficient, do not speculate, and do not modify code or external state.',
            ),
            'badge': DesktopI18n.tr('Еженедельно по средам в 15:00', 'Weekly on Wed at 15:00'),
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
