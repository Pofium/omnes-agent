// Automations View for OmnesAgent ADE:
// - Header with Title and Subtitle
// - Scheduled tasks empty-state container with 'Create scheduled task' and 'Create idle-time task'
// - 'Keep your computer awake while OmnesAgent is running a chat' setting
// - 'Idle-time task template' cards
// - 'Scheduled task template' cards with cron intervals

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import '../../theme/desktop_theme.dart';
import '../../utils/desktop_i18n.dart';
import 'sop_studio_controller.dart';

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
  int selectedTab = 0; // 0: SOP Studio, 1: Scheduled Tasks
  late final SopStudioController sopController;
  bool isKeepAwakeEnabled = true;

  // Active scheduled tasks list
  final List<Map<String, dynamic>> scheduledTasks = [];

  @override
  void initState() {
    super.initState();
    sopController = Get.put(SopStudioController());
    _loadPersistedTasks();
  }

  void _loadPersistedTasks() {
    try {
      final saved = GetStorage().read<List>('scheduled_tasks');
      if (saved != null) {
        for (final item in saved) {
          if (item is Map) {
            scheduledTasks.add(Map<String, dynamic>.from(item));
          }
        }
      }
      isKeepAwakeEnabled = GetStorage().read<bool>('keep_awake_enabled') ?? true;
    } catch (_) {}
  }

  void _saveScheduledTasks() {
    try {
      GetStorage().write('scheduled_tasks', scheduledTasks);
    } catch (_) {}
  }

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
                          _saveScheduledTasks();
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

  Widget _buildTabButton(int index, String label, IconData icon) {
    final isSelected = selectedTab == index;
    return InkWell(
      onTap: () => setState(() => selectedTab = index),
      borderRadius: BorderRadius.circular(8),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
        decoration: BoxDecoration(
          color: isSelected ? DesktopTheme.accentCyan.withOpacity(0.12) : Colors.transparent,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: isSelected ? DesktopTheme.accentCyan : DesktopTheme.borderSubtle,
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              icon,
              size: 13,
              color: isSelected ? DesktopTheme.accentCyan : DesktopTheme.textMuted,
            ),
            const SizedBox(width: 8),
            Text(
              label,
              style: TextStyle(
                fontSize: 12.5,
                fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                color: isSelected ? DesktopTheme.textPrimary : DesktopTheme.textMuted,
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: DesktopTheme.bgCanvas,
      body: Column(
        children: [
          // Top Navigation & Tab Bar
          Container(
            padding: const EdgeInsets.fromLTRB(32, 20, 32, 16),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    if (widget.onBackToWorkspace != null) ...[
                      IconButton(
                        icon: const Icon(Icons.arrow_back, size: 18),
                        color: DesktopTheme.textMuted,
                        onPressed: widget.onBackToWorkspace,
                        tooltip: 'Назад в рабочую область',
                      ),
                      const SizedBox(width: 8),
                    ],
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          DesktopI18n.automationsTitle,
                          style: TextStyle(
                            fontSize: 22,
                            fontWeight: FontWeight.bold,
                            color: DesktopTheme.textPrimary,
                            letterSpacing: -0.3,
                          ),
                        ),
                        const SizedBox(height: 2),
                        Text(
                          DesktopI18n.automationsSubtitle,
                          style: TextStyle(
                            fontSize: 12,
                            color: DesktopTheme.textMuted,
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
                const SizedBox(height: 14),
                Row(
                  children: [
                    _buildTabButton(0, 'Студия SOP (Workflow Studio)', FontAwesomeIcons.diagramProject),
                    const SizedBox(width: 8),
                    _buildTabButton(1, 'Задачи по расписанию (Cron)', FontAwesomeIcons.clock),
                  ],
                ),
              ],
            ),
          ),

          // Tab Content
          if (selectedTab == 0)
            Expanded(child: _buildSopStudioTab())
          else
            Expanded(
              child: SingleChildScrollView(
                padding: const EdgeInsets.fromLTRB(48, 24, 48, 48),
                child: ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 980),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // Top Empty State Card or Active Tasks List
                      _buildScheduledTasksCard(),
                      const SizedBox(height: 16),

                      // Keep Awake Row
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
  }

  // ==========================================
  // SOP WORKFLOW STUDIO TAB (EPIC P1 - 5.5)
  // ==========================================
  Widget _buildSopStudioTab() {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Left Column: SOP List
        Container(
          width: 310,
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            border: Border(right: BorderSide(color: DesktopTheme.borderSubtle)),
          ),
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 14, 16, 10),
                child: Row(
                  children: [
                    Text(
                      'Пайплайны (SOP)',
                      style: TextStyle(
                        fontSize: 13.5,
                        fontWeight: FontWeight.bold,
                        color: DesktopTheme.textPrimary,
                      ),
                    ),
                    const Spacer(),
                    IconButton(
                      icon: const Icon(Icons.refresh, size: 16),
                      color: DesktopTheme.textMuted,
                      tooltip: 'Обновить',
                      onPressed: () => sopController.loadSops(),
                    ),
                    IconButton(
                      icon: const Icon(Icons.add, size: 16),
                      color: DesktopTheme.accentCyan,
                      tooltip: 'Создать SOP',
                      onPressed: _showCreateSopDialog,
                    ),
                  ],
                ),
              ),
              const Divider(height: 1),
              Expanded(
                child: Obx(() {
                  if (sopController.isLoading.value && sopController.sops.isEmpty) {
                    return const Center(child: CircularProgressIndicator(strokeWidth: 2));
                  }
                  return ListView.builder(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
                    itemCount: sopController.sops.length,
                    itemBuilder: (ctx, i) {
                      final sop = sopController.sops[i];
                      final name = sop['name']?.toString() ?? '';
                      final title = sop['title']?.toString() ?? name;
                      final isSelected = sopController.selectedSopName.value == name;
                      final mode = sop['execution_mode']?.toString() ?? 'autonomous';
                      return Container(
                        margin: const EdgeInsets.only(bottom: 6),
                        decoration: BoxDecoration(
                          color: isSelected ? DesktopTheme.accentCyan.withOpacity(0.12) : Colors.transparent,
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(
                            color: isSelected ? DesktopTheme.accentCyan : DesktopTheme.borderSubtle.withOpacity(0.5),
                          ),
                        ),
                        child: ListTile(
                          dense: true,
                          title: Text(
                            title,
                            style: TextStyle(
                              fontSize: 12.5,
                              fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                              color: DesktopTheme.textPrimary,
                            ),
                          ),
                          subtitle: Text(
                            name,
                            style: TextStyle(
                              fontSize: 11,
                              fontFamily: 'Consolas',
                              color: DesktopTheme.textMuted,
                            ),
                          ),
                          trailing: Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: mode == 'supervised'
                                  ? Colors.amber.withOpacity(0.15)
                                  : const Color(0xFF10B981).withOpacity(0.15),
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              mode,
                              style: TextStyle(
                                fontSize: 10,
                                fontWeight: FontWeight.w600,
                                color: mode == 'supervised' ? Colors.amber : const Color(0xFF10B981),
                              ),
                            ),
                          ),
                          onTap: () => sopController.selectSop(name),
                        ),
                      );
                    },
                  );
                }),
              ),
            ],
          ),
        ),

        // Right Column: DAG Graph & Execution Panel
        Expanded(
          child: Obx(() {
            final selectedName = sopController.selectedSopName.value;
            if (selectedName == null) {
              return Center(
                child: Text(
                  'Выберите SOP пайплайн из списка слева',
                  style: TextStyle(color: DesktopTheme.textMuted),
                ),
              );
            }
            final graph = sopController.selectedGraph.value;
            final nodes = (graph?['nodes'] as List<dynamic>?) ?? [];

            return Column(
              children: [
                // SOP Header Bar
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
                  decoration: BoxDecoration(
                    color: DesktopTheme.bgSurfaceElevated,
                    border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
                  ),
                  child: Row(
                    children: [
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              const Icon(FontAwesomeIcons.diagramProject, size: 15, color: DesktopTheme.accentCyan),
                              const SizedBox(width: 8),
                              Text(
                                selectedName,
                                style: TextStyle(
                                  fontSize: 15,
                                  fontWeight: FontWeight.bold,
                                  color: DesktopTheme.textPrimary,
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: 4),
                          Text(
                            'Интерактивный DAG граф исполнения процедур OmnesAgent Gateway',
                            style: TextStyle(fontSize: 11.5, color: DesktopTheme.textMuted),
                          ),
                        ],
                      ),
                      const Spacer(),
                      OutlinedButton.icon(
                        icon: const Icon(FontAwesomeIcons.robot, size: 12),
                        label: const Text('AI Wire-Draft', style: TextStyle(fontSize: 12)),
                        style: OutlinedButton.styleFrom(
                          foregroundColor: DesktopTheme.accentCyan,
                          side: BorderSide(color: DesktopTheme.accentCyan.withOpacity(0.5)),
                        ),
                        onPressed: () => _showAiWireDraftDialog(selectedName),
                      ),
                      const SizedBox(width: 10),
                      ElevatedButton.icon(
                        icon: const Icon(FontAwesomeIcons.play, size: 11),
                        label: const Text('Запустить SOP', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: DesktopTheme.accentCyan,
                          foregroundColor: Colors.black,
                          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                        ),
                        onPressed: () => sopController.runActiveSop(),
                      ),
                    ],
                  ),
                ),

                // Main Graph Area
                Expanded(
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.all(28),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        // Visual DAG Nodes Flow
                        _buildDagGraphNodes(nodes),
                        const SizedBox(height: 24),

                        // Approval Gate (if any node has status 'pending')
                        if (nodes.any((n) => n['kind'] == 'gate' && n['status'] == 'pending'))
                          _buildApprovalGateCard(selectedName),

                        const SizedBox(height: 32),
                        _buildRunsTimeline(),
                      ],
                    ),
                  ),
                ),
              ],
            );
          }),
        ),
      ],
    );
  }

  Widget _buildDagGraphNodes(List<dynamic> nodes) {
    if (nodes.isEmpty) {
      return Center(
        child: Text('Граф узлов пуст', style: TextStyle(color: DesktopTheme.textMuted)),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Узлы пайплайна (DAG Pipeline)',
          style: TextStyle(
            fontSize: 14,
            fontWeight: FontWeight.bold,
            color: DesktopTheme.textPrimary,
          ),
        ),
        const SizedBox(height: 16),
        ...nodes.asMap().entries.map((entry) {
          final idx = entry.key;
          final node = entry.value as Map<String, dynamic>;
          final isLast = idx == nodes.length - 1;

          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildNodeCard(node),
              if (!isLast) ...[
                Padding(
                  padding: const EdgeInsets.only(left: 36),
                  child: Container(
                    width: 2,
                    height: 24,
                    color: DesktopTheme.accentCyan.withOpacity(0.4),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(left: 32),
                  child: Icon(
                    Icons.arrow_downward,
                    size: 14,
                    color: DesktopTheme.accentCyan.withOpacity(0.6),
                  ),
                ),
                const SizedBox(height: 4),
              ],
            ],
          );
        }),
      ],
    );
  }

  Widget _buildNodeCard(Map<String, dynamic> node) {
    final kind = node['kind']?.toString() ?? 'step';
    final title = node['title']?.toString() ?? node['id']?.toString() ?? '';
    final status = node['status']?.toString() ?? 'idle';

    IconData icon;
    Color iconColor;
    switch (kind) {
      case 'trigger':
        icon = FontAwesomeIcons.bolt;
        iconColor = const Color(0xFF38BDF8);
        break;
      case 'tool':
        icon = FontAwesomeIcons.wrench;
        iconColor = const Color(0xFFA78BFA);
        break;
      case 'gate':
        icon = FontAwesomeIcons.shieldHalved;
        iconColor = const Color(0xFFF59E0B);
        break;
      default:
        icon = FontAwesomeIcons.gears;
        iconColor = DesktopTheme.accentCyan;
    }

    Color statusColor;
    String statusLabel;
    switch (status) {
      case 'completed':
        statusColor = const Color(0xFF10B981);
        statusLabel = 'Выполнено';
        break;
      case 'running':
        statusColor = DesktopTheme.accentCyan;
        statusLabel = 'В процессе';
        break;
      case 'pending':
        statusColor = const Color(0xFFF59E0B);
        statusLabel = 'Ожидает одобрения';
        break;
      default:
        statusColor = const Color(0xFF64748B);
        statusLabel = 'Ожидание';
    }

    return Container(
      width: double.infinity,
      constraints: const BoxConstraints(maxWidth: 720),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurfaceElevated,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(
          color: status == 'pending' ? const Color(0xFFF59E0B) : DesktopTheme.borderSubtle,
          width: status == 'pending' ? 1.5 : 1,
        ),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: iconColor.withOpacity(0.12),
              shape: BoxShape.circle,
            ),
            child: Icon(icon, size: 14, color: iconColor),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.bold,
                    color: DesktopTheme.textPrimary,
                  ),
                ),
                const SizedBox(height: 3),
                Text(
                  'Тип: $kind • ID: ${node['id']}',
                  style: TextStyle(
                    fontSize: 11,
                    fontFamily: 'Consolas',
                    color: DesktopTheme.textMuted,
                  ),
                ),
              ],
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: statusColor.withOpacity(0.15),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 6,
                  height: 6,
                  decoration: BoxDecoration(
                    color: statusColor,
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 6),
                Text(
                  statusLabel,
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: statusColor,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildApprovalGateCard(String sopName) {
    return Container(
      width: double.infinity,
      constraints: const BoxConstraints(maxWidth: 720),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: const Color(0xFFF59E0B).withOpacity(0.08),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: const Color(0xFFF59E0B)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(FontAwesomeIcons.shieldHalved, size: 16, color: Color(0xFFF59E0B)),
              const SizedBox(width: 10),
              Text(
                'Human-in-the-loop: Требуется подтверждение оператора',
                style: TextStyle(
                  fontSize: 13.5,
                  fontWeight: FontWeight.bold,
                  color: DesktopTheme.textPrimary,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            'Выполнение пайплайна "$sopName" остановлено на шаге согласования деструктивных операций.',
            style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted),
          ),
          const SizedBox(height: 14),
          Row(
            children: [
              ElevatedButton.icon(
                icon: const Icon(Icons.check, size: 14),
                label: const Text('Одобрить (Approve)', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF10B981),
                  foregroundColor: Colors.white,
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
                  padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                ),
                onPressed: () async {
                  await sopController.httpClient.sopApprove(sopName);
                  sopController.selectSop(sopName);
                  Get.snackbar('SOP согласован', 'Шаг пайплайна утвержден оператором');
                },
              ),
              const SizedBox(width: 10),
              OutlinedButton.icon(
                icon: const Icon(Icons.close, size: 14),
                label: const Text('Отклонить (Deny)', style: TextStyle(fontSize: 12)),
                style: OutlinedButton.styleFrom(
                  foregroundColor: const Color(0xFFEF4444),
                  side: const BorderSide(color: Color(0xFFEF4444)),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
                  padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                ),
                onPressed: () async {
                  await sopController.httpClient.sopDeny(sopName);
                  sopController.selectSop(sopName);
                  Get.snackbar('SOP отклонен', 'Шаг пайплайна заблокирован');
                },
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildRunsTimeline() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Text(
              'История запусков (Recent Runs)',
              style: TextStyle(
                fontSize: 14,
                fontWeight: FontWeight.bold,
                color: DesktopTheme.textPrimary,
              ),
            ),
            const Spacer(),
            TextButton.icon(
              icon: const Icon(Icons.refresh, size: 14),
              label: const Text('Обновить', style: TextStyle(fontSize: 12)),
              onPressed: () => sopController.loadRuns(),
            ),
          ],
        ),
        const SizedBox(height: 10),
        Obx(() {
          if (sopController.sopRuns.isEmpty) {
            return Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: DesktopTheme.borderSubtle),
              ),
              child: Text(
                'Запусков пока не зафиксировано',
                style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted),
              ),
            );
          }
          return Column(
            children: sopController.sopRuns.map((run) {
              final id = run['run_id']?.toString() ?? run['id']?.toString() ?? 'unknown';
              final status = run['status']?.toString() ?? 'completed';
              return Container(
                margin: const EdgeInsets.only(bottom: 8),
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                decoration: BoxDecoration(
                  color: DesktopTheme.bgSurfaceElevated,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: DesktopTheme.borderSubtle),
                ),
                child: Row(
                  children: [
                    Icon(
                      status == 'completed' ? Icons.check_circle : Icons.error,
                      size: 16,
                      color: status == 'completed' ? const Color(0xFF10B981) : const Color(0xFFEF4444),
                    ),
                    const SizedBox(width: 10),
                    Text(
                      'Run #$id',
                      style: TextStyle(
                        fontSize: 12.5,
                        fontFamily: 'Consolas',
                        fontWeight: FontWeight.bold,
                        color: DesktopTheme.textPrimary,
                      ),
                    ),
                    const Spacer(),
                    Text(
                      status,
                      style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                    ),
                  ],
                ),
              );
            }).toList(),
          );
        }),
      ],
    );
  }

  void _showCreateSopDialog() {
    final nameCtrl = TextEditingController();
    final titleCtrl = TextEditingController();
    final descCtrl = TextEditingController();

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurfaceElevated,
        title: Text('Новый SOP пайплайн', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 16)),
        content: SizedBox(
          width: 420,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: nameCtrl,
                decoration: InputDecoration(
                  labelText: 'Идентификатор (латиница)',
                  labelStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 12),
                  hintText: 'security-scan',
                ),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: titleCtrl,
                decoration: InputDecoration(
                  labelText: 'Название',
                  labelStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 12),
                  hintText: 'Сканирование безопасности',
                ),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: descCtrl,
                maxLines: 2,
                decoration: InputDecoration(
                  labelText: 'Описание',
                  labelStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 12),
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(DesktopI18n.cancel, style: TextStyle(color: DesktopTheme.textMuted)),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: DesktopTheme.accentCyan, foregroundColor: Colors.black),
            onPressed: () async {
              if (nameCtrl.text.trim().isNotEmpty) {
                final name = nameCtrl.text.trim();
                final sopData = {
                  'name': name,
                  'title': titleCtrl.text.trim().isNotEmpty ? titleCtrl.text.trim() : name,
                  'description': descCtrl.text.trim(),
                  'execution_mode': 'autonomous',
                  'triggers': ['manual'],
                };
                Navigator.of(ctx).pop();
                await sopController.httpClient.saveSop(name, sopData);
                sopController.loadSops();
              }
            },
            child: const Text('Создать'),
          ),
        ],
      ),
    );
  }

  void _showAiWireDraftDialog(String sopName) {
    final promptCtrl = TextEditingController(text: 'Сгенерируй пайплайн для автоматической проверки $sopName');

    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurfaceElevated,
        title: Text('AI Wire-Draft для $sopName', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 16)),
        content: SizedBox(
          width: 460,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Опишите словами желаемые этапы и инструменты. LLM-агент сформирует DAG-граф и спецификацию.',
                style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted),
              ),
              const SizedBox(height: 14),
              TextField(
                controller: promptCtrl,
                maxLines: 3,
                decoration: InputDecoration(
                  labelText: 'Промпт генерации',
                  labelStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 12),
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(DesktopI18n.cancel, style: TextStyle(color: DesktopTheme.textMuted)),
          ),
          ElevatedButton.icon(
            icon: const Icon(FontAwesomeIcons.robot, size: 12),
            label: const Text('Сгенерировать'),
            style: ElevatedButton.styleFrom(backgroundColor: DesktopTheme.accentCyan, foregroundColor: Colors.black),
            onPressed: () async {
              Navigator.of(ctx).pop();
              Get.snackbar('AI Wire-Draft', 'Промпт отправлен в OmnesAgent runtime...');
              await sopController.httpClient.sopWireDraft({'prompt': promptCtrl.text.trim(), 'name': sopName});
              sopController.selectSop(sopName);
            },
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
                        _saveScheduledTasks();
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
            onChanged: (val) {
              setState(() => isKeepAwakeEnabled = val);
              try {
                GetStorage().write('keep_awake_enabled', val);
              } catch (_) {}
            },
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
          color: DesktopTheme.bgSurfaceElevated,
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: DesktopTheme.borderSubtle),
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
            const SizedBox(height: 10),
            Text(
              desc,
              style: TextStyle(
                fontSize: 11.5,
                color: DesktopTheme.textMuted,
                height: 1.45,
              ),
            ),
            const SizedBox(height: 16),
            Text(
              badge,
              style: TextStyle(
                fontSize: 11,
                fontFamily: 'Consolas',
                color: DesktopTheme.textMuted,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
