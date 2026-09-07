// OmnesAgent ADE Desktop Sidebar matching authentic Windows ADE layout (Screenshot 2).
// Features Windows native header, # Group vs 📁 Project switcher, real projects hierarchy,
// and fixed bottom user profile with avatar, 'Lite' badge, mobile remote, and settings gear.

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import '../features/onboarding/user_onboarding_dialog.dart';
import '../features/workspace/task_workspace_controller.dart';
import '../theme/desktop_theme.dart';
import '../utils/desktop_i18n.dart';

class DesktopSidebar extends StatefulWidget {
  final int selectedIndex;
  final Function(String id, String title, String project) onSelectTask;
  final VoidCallback onNewTask;
  final VoidCallback onOpenSearch;
  final VoidCallback onOpenSettings;
  final VoidCallback onToggleSidebar;
  final VoidCallback? onOpenSkills;
  final VoidCallback? onOpenAutomations;
  final UserProfileData userProfile;
  final VoidCallback onOpenProfile;

  const DesktopSidebar({
    super.key,
    required this.selectedIndex,
    required this.onSelectTask,
    required this.onNewTask,
    required this.onOpenSearch,
    required this.onOpenSettings,
    required this.onToggleSidebar,
    this.onOpenSkills,
    this.onOpenAutomations,
    required this.userProfile,
    required this.onOpenProfile,
  });

  @override
  State<DesktopSidebar> createState() => _DesktopSidebarState();
}

enum SidebarViewMode { byProject, timeline }
enum SidebarSortMode { updated, created }

class _DesktopSidebarState extends State<DesktopSidebar> {
  bool isGroupView = false; // false: Project, true: Group
  SidebarViewMode viewMode = SidebarViewMode.byProject;
  SidebarSortMode sortMode = SidebarSortMode.updated;
  bool isAllExpanded = true;
  final Set<String> collapsedProjects = {};

  final List<Map<String, dynamic>> projects = [
    {
      'name': 'Omnes agent',
      'tasks': [
        {'id': 'omnes-1', 'title': 'План миграции Dart фронтенда в агента Zero...', 'time': '2d', 'unread': false},
      ],
    },
    {
      'name': 'deepseek-harness-master',
      'tasks': [
        {'id': 'deepseek-1', 'title': 'Запуск exe-файла', 'time': '1h', 'unread': false},
        {'id': 'deepseek-2', 'title': 'Переключение на ветку desktop и запуск exe', 'time': '1d', 'unread': false},
      ],
    },
    {
      'name': 'us proxy vps',
      'tasks': [
        {'id': 'proxy-1', 'title': 'реши проблему с прокси нихера не работает', 'time': '7d', 'unread': false},
        {'id': 'proxy-2', 'title': 'Отладка прокси Hermes и LLM API', 'time': '7d', 'unread': false},
      ],
    },
    {
      'name': 'aprh.serv',
      'tasks': [
        {'id': 'aprh-1', 'title': 'проанализируй проект посмотри логи напи...', 'time': '15d', 'unread': false},
      ],
    },
    {
      'name': 'AmoParallel',
      'tasks': [
        {'id': 'amo-1', 'title': 'Диагностика ошибки агента Antigravity IDE', 'time': '9d', 'unread': false},
        {'id': 'amo-2', 'title': 'выполный план, только поставь веху и остан...', 'time': '19d', 'unread': true},
      ],
    },
  ];

  final List<Map<String, dynamic>> groups = [
    {
      'name': 'Личные диалоги и черновики',
      'tasks': [
        {'id': 'group-1', 'title': 'Идеи для оптимизации рендеринга Flutter Web', 'time': '3h', 'unread': false},
        {'id': 'group-2', 'title': 'Сравнение моделей GLM-5.3 и Claude 3.5 Sonnet', 'time': '1d', 'unread': false},
      ],
    },
    {
      'name': 'Исследования и архитектура',
      'tasks': [
        {'id': 'group-3', 'title': 'Дизайн-документ ob2h графа памяти', 'time': '2d', 'unread': false},
        {'id': 'group-4', 'title': 'Интеграция протокола MCP через WebSocket', 'time': '4d', 'unread': false},
      ],
    },
  ];

  @override
  Widget build(BuildContext context) {
    return Obx(() {
      return Container(
        width: 260,
        decoration: BoxDecoration(
          color: DesktopTheme.bgSidebar,
          border: Border(right: BorderSide(color: DesktopTheme.borderSubtle, width: 1)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 1. Top Windows ADE Header (Logo + Back/Forward + Collapse)
            Padding(
              padding: const EdgeInsets.only(left: 14, top: 12, right: 12, bottom: 8),
              child: Row(
                children: [
                  // OmnesAgent Brand Logo (SVG emblem from OA_icon.svg)
                  SvgPicture.asset(
                    'assets/Logo/OA_icon.svg',
                    width: 22,
                    height: 22,
                    fit: BoxFit.contain,
                  ),
                  const SizedBox(width: 14),

                // Navigation Arrows
                InkWell(
                  onTap: () {},
                  borderRadius: BorderRadius.circular(3),
                  child: const Icon(Icons.arrow_back, size: 14, color: Color(0xFF94A3B8)),
                ),
                const SizedBox(width: 8),
                InkWell(
                  onTap: () {},
                  borderRadius: BorderRadius.circular(3),
                  child: const Icon(Icons.arrow_forward, size: 14, color: Color(0xFF475569)),
                ),

                const Spacer(),

                // Collapse sidebar button
                InkWell(
                  onTap: widget.onToggleSidebar,
                  borderRadius: BorderRadius.circular(4),
                  child: const Icon(Icons.view_sidebar_outlined, size: 15, color: Color(0xFF94A3B8)),
                ),
              ],
            ),
          ),

          // 2. Primary Actions: + New task, Search, Automations, Plugin Marketplace
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            child: Column(
              children: [
                _buildActionRow(
                  icon: FontAwesomeIcons.plus,
                  label: DesktopI18n.newTask,
                  shortcut: 'Ctrl+N',
                  onTap: widget.onNewTask,
                ),
                _buildActionRow(
                  icon: FontAwesomeIcons.magnifyingGlass,
                  label: DesktopI18n.search,
                  shortcut: 'Ctrl+K',
                  onTap: widget.onOpenSearch,
                ),
                _buildActionRow(
                  icon: FontAwesomeIcons.bolt,
                  label: DesktopI18n.automations,
                  shortcut: '',
                  isSelected: widget.selectedIndex == -3,
                  onTap: widget.onOpenAutomations ?? () {},
                ),
              ],
            ),
          ),

          const SizedBox(height: 6),

          // 3. Tab Switcher: [ # Group ]  [ 📁 Project ]
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            child: Row(
              children: [
                // Group button
                InkWell(
                  onTap: () => setState(() => isGroupView = true),
                  borderRadius: BorderRadius.circular(6),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: isGroupView ? const Color(0xFF242933) : Colors.transparent,
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(FontAwesomeIcons.hashtag, size: 11, color: isGroupView ? Colors.white : const Color(0xFF64748B)),
                        const SizedBox(width: 5),
                        Text(
                          DesktopI18n.group,
                          style: TextStyle(
                            fontSize: 12,
                            fontWeight: isGroupView ? FontWeight.bold : FontWeight.w500,
                            color: isGroupView ? Colors.white : const Color(0xFF94A3B8),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(width: 4),

                // Project button
                InkWell(
                  onTap: () => setState(() => isGroupView = false),
                  borderRadius: BorderRadius.circular(6),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: !isGroupView ? const Color(0xFF242933) : Colors.transparent,
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(Icons.folder_outlined, size: 13, color: !isGroupView ? const Color(0xFF00D2FF) : const Color(0xFF64748B)),
                        const SizedBox(width: 5),
                        Text(
                          DesktopI18n.project,
                          style: TextStyle(
                            fontSize: 12,
                            fontWeight: !isGroupView ? FontWeight.bold : FontWeight.w500,
                            color: !isGroupView ? Colors.white : const Color(0xFF94A3B8),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),

                const Spacer(),

                // Quick actions right of tabs: Expand / Collapse all & Filter menu (Image 1 match)
                IconButton(
                  tooltip: isAllExpanded ? DesktopI18n.collapseAll : DesktopI18n.expandAll,
                  icon: Icon(
                    isAllExpanded ? Icons.close_fullscreen : Icons.open_in_full,
                    size: 13,
                    color: const Color(0xFF94A3B8),
                  ),
                  onPressed: () {
                    setState(() {
                      isAllExpanded = !isAllExpanded;
                      if (!isAllExpanded) {
                        for (final p in (isGroupView ? groups : projects)) {
                          collapsedProjects.add(p['name'] as String);
                        }
                      } else {
                        collapsedProjects.clear();
                      }
                    });
                  },
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 22, minHeight: 22),
                ),
                const SizedBox(width: 4),

                // Working Filter & Sort Popover Menu (Image 1 match)
                _buildFilterPopupMenu(),
              ],
            ),
          ),

          // 4. Projects / Groups Header
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
            child: Row(
              children: [
                Text(
                  isGroupView
                      ? DesktopI18n.groups
                      : (viewMode == SidebarViewMode.timeline ? DesktopI18n.timeline : DesktopI18n.projects),
                  style: const TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: Color(0xFF64748B),
                    letterSpacing: 0.4,
                  ),
                ),
                if (viewMode == SidebarViewMode.timeline) ...[
                  const SizedBox(width: 6),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                    decoration: BoxDecoration(
                      color: const Color(0xFF00D2FF).withOpacity(0.15),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      sortMode == SidebarSortMode.updated ? 'Updated' : 'Created',
                      style: const TextStyle(fontSize: 9, color: Color(0xFF00D2FF), fontWeight: FontWeight.bold),
                    ),
                  ),
                ],
              ],
            ),
          ),

          // 5. Scrollable Projects / Timeline List
          Expanded(
            child: Obx(() {
              final workspaceCtrl = Get.isRegistered<DesktopTaskWorkspaceController>()
                  ? Get.find<DesktopTaskWorkspaceController>()
                  : null;

              final currentProjects = List<Map<String, dynamic>>.from(projects);
              if (workspaceCtrl != null && workspaceCtrl.backendSessions.isNotEmpty) {
                currentProjects.insert(0, {
                  'name': '⚡ Шлюз 42617',
                  'tasks': workspaceCtrl.backendSessions.map((s) => {
                    'id': s.sessionId,
                    'title': s.previewText.isNotEmpty ? s.previewText : s.sessionId,
                    'time': s.formattedLastActivity,
                    'unread': false,
                  }).toList(),
                });
              }

              return ListView(
                padding: const EdgeInsets.symmetric(horizontal: 8),
                children: viewMode == SidebarViewMode.byProject
                    ? (isGroupView ? groups : currentProjects).map((proj) => _buildProjectItem(proj)).toList()
                    : _buildTimelineItems(),
              );
            }),
          ),

          // 6. Fixed Bottom User, Remote & Settings Bar (NEVER SCROLLED!)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSidebar,
              border: Border(top: BorderSide(color: DesktopTheme.borderSubtle, width: 1)),
            ),
            child: Row(
              children: [
                // Profile Avatar & Name
                InkWell(
                  onTap: widget.onOpenProfile,
                  borderRadius: BorderRadius.circular(6),
                  child: Row(
                    children: [
                      Container(
                        width: 26,
                        height: 26,
                        decoration: BoxDecoration(
                          gradient: const LinearGradient(
                            colors: [Color(0xFF00D2FF), Color(0xFF0072FF)],
                          ),
                          borderRadius: BorderRadius.circular(13),
                        ),
                        child: Center(
                          child: Text(
                            widget.userProfile.initials,
                            style: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Colors.white),
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      ConstrainedBox(
                        constraints: const BoxConstraints(maxWidth: 105),
                        child: Text(
                          widget.userProfile.fullName,
                          style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary, fontWeight: FontWeight.w500),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    ],
                  ),
                ),
                const Spacer(),

                // Settings Gear Button
                IconButton(
                  tooltip: DesktopI18n.settings,
                  icon: const Icon(Icons.settings_outlined, size: 17, color: Color(0xFF00D2FF)),
                  onPressed: widget.onOpenSettings,
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 26, minHeight: 26),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  });
}

  Widget _buildFilterPopupMenu() {
    return PopupMenuButton<String>(
      tooltip: DesktopI18n.tr('Вид и сортировка', 'View and sort'),
      offset: const Offset(0, 26),
      color: const Color(0xFF1E2127),
      elevation: 12,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(10),
        side: const BorderSide(color: Color(0xFF2C323E), width: 1),
      ),
      onSelected: (val) {
        setState(() {
          if (val == 'by_project') viewMode = SidebarViewMode.byProject;
          if (val == 'timeline') viewMode = SidebarViewMode.timeline;
          if (val == 'updated') sortMode = SidebarSortMode.updated;
          if (val == 'created') sortMode = SidebarSortMode.created;
        });
      },
      itemBuilder: (ctx) => [
        // View header
        PopupMenuItem<String>(
          enabled: false,
          height: 28,
          child: Text(DesktopI18n.view, style: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Color(0xFF64748B))),
        ),
        _buildFilterItem('by_project', Icons.folder_outlined, DesktopI18n.byProject, viewMode == SidebarViewMode.byProject),
        _buildFilterItem('timeline', Icons.access_time, DesktopI18n.timeline, viewMode == SidebarViewMode.timeline),
        const PopupMenuDivider(height: 10),
        // Sort by header
        PopupMenuItem<String>(
          enabled: false,
          height: 28,
          child: Text(DesktopI18n.sortBy, style: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Color(0xFF64748B))),
        ),
        _buildFilterItem('updated', Icons.chat_bubble_outline, DesktopI18n.updated, sortMode == SidebarSortMode.updated),
        _buildFilterItem('created', Icons.add_comment_outlined, DesktopI18n.created, sortMode == SidebarSortMode.created),
      ],
      child: Container(
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(
          color: (viewMode == SidebarViewMode.timeline || sortMode == SidebarSortMode.created)
              ? const Color(0xFF00D2FF).withOpacity(0.18)
              : Colors.transparent,
          borderRadius: BorderRadius.circular(4),
        ),
        child: const Icon(Icons.filter_list, size: 14, color: Color(0xFF94A3B8)),
      ),
    );
  }

  PopupMenuItem<String> _buildFilterItem(String value, IconData icon, String label, bool isSelected) {
    return PopupMenuItem<String>(
      value: value,
      height: 36,
      child: Row(
        children: [
          Icon(icon, size: 14, color: isSelected ? const Color(0xFF00D2FF) : const Color(0xFFCBD5E1)),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              label,
              style: TextStyle(
                fontSize: 12,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                color: isSelected ? Colors.white : const Color(0xFFCBD5E1),
              ),
            ),
          ),
          if (isSelected)
            const Icon(Icons.check, size: 14, color: Color(0xFF00D2FF)),
        ],
      ),
    );
  }

  List<Widget> _buildTimelineItems() {
    final all = <Map<String, dynamic>>[];
    for (final proj in (isGroupView ? groups : projects)) {
      for (final t in proj['tasks']) {
        all.add({
          ...t,
          'project': proj['name'],
        });
      }
    }

    if (sortMode == SidebarSortMode.created) {
      all.sort((a, b) => (a['title'] as String).compareTo(b['title'] as String));
    }

    return all.map((task) {
      final hasUnread = task['unread'] == true;
      return InkWell(
        onTap: () {
          widget.onSelectTask(
            task['id'] as String? ?? 'task-timeline',
            task['title'] as String,
            task['project'] as String? ?? '',
          );
        },
        onSecondaryTapDown: (details) => _showTaskContextMenu(
          context,
          details.globalPosition,
          task,
          task['project'] as String? ?? '',
        ),
        borderRadius: BorderRadius.circular(6),
        child: Container(
          margin: const EdgeInsets.only(bottom: 2, left: 6, right: 6),
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
          decoration: BoxDecoration(borderRadius: BorderRadius.circular(6)),
          child: Row(
            children: [
              if (hasUnread) ...[
                Container(
                  width: 5,
                  height: 5,
                  decoration: const BoxDecoration(shape: BoxShape.circle, color: Color(0xFFEF4444)),
                ),
                const SizedBox(width: 6),
              ] else ...[
                Icon(Icons.chat_bubble_outline, size: 12, color: DesktopTheme.textMuted),
                const SizedBox(width: 6),
              ],
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      task['title'],
                      style: TextStyle(fontSize: 12, color: DesktopTheme.textSecondary),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    Text(
                      task['project'],
                      style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted, fontFamily: 'Consolas'),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 6),
              Text(
                task['time'],
                style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted, fontFamily: 'Consolas'),
              ),
            ],
          ),
        ),
      );
    }).toList();
  }

  Widget _buildProjectItem(Map<String, dynamic> proj) {
    final tasks = proj['tasks'] as List<dynamic>;
    final isCollapsed = collapsedProjects.contains(proj['name']);

    return Container(
      margin: const EdgeInsets.only(bottom: 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Project Folder Header
          InkWell(
            onTap: () {
              setState(() {
                if (isCollapsed) {
                  collapsedProjects.remove(proj['name']);
                } else {
                  collapsedProjects.add(proj['name'] as String);
                }
              });
            },
            borderRadius: BorderRadius.circular(6),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              child: Row(
                children: [
                  Icon(
                    isCollapsed ? Icons.chevron_right : Icons.keyboard_arrow_down,
                    size: 14,
                    color: DesktopTheme.textMuted,
                  ),
                  const SizedBox(width: 4),
                  Icon(
                    isGroupView ? Icons.chat_bubble_outline : Icons.folder_outlined,
                    size: 13,
                    color: isCollapsed ? DesktopTheme.textMuted : DesktopTheme.accentCyan,
                  ),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      proj['name'],
                      style: TextStyle(
                        fontSize: 12,
                        fontWeight: FontWeight.bold,
                        color: DesktopTheme.textPrimary,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            ),
          ),

          // Tasks under project (if not collapsed)
          if (!isCollapsed)
            ...tasks.map((task) {
              final hasUnread = task['unread'] == true;

              return InkWell(
                onTap: () {
                  widget.onSelectTask(
                    task['id'] as String? ?? 'task-${task['title']}',
                    task['title'] as String,
                    proj['name'] as String,
                  );
                },
                onSecondaryTapDown: (details) => _showTaskContextMenu(
                  context,
                  details.globalPosition,
                  task as Map<String, dynamic>,
                  proj['name'] as String,
                ),
                borderRadius: BorderRadius.circular(6),
                child: Container(
                  margin: const EdgeInsets.only(bottom: 1, left: 18, right: 6),
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: Row(
                    children: [
                      if (hasUnread) ...[
                        Container(
                          width: 5,
                          height: 5,
                          decoration: const BoxDecoration(shape: BoxShape.circle, color: Color(0xFFEF4444)),
                        ),
                        const SizedBox(width: 6),
                      ],
                      Expanded(
                        child: Text(
                          task['title'],
                          style: TextStyle(
                            fontSize: 12,
                            color: DesktopTheme.textSecondary,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      const SizedBox(width: 6),
                      Text(
                        task['time'],
                        style: TextStyle(
                          fontSize: 10,
                          color: DesktopTheme.textMuted,
                          fontFamily: 'Consolas',
                        ),
                      ),
                    ],
                  ),
                ),
              );
            }),
        ],
      ),
    );
  }

  void _showTaskContextMenu(
    BuildContext context,
    Offset position,
    Map<String, dynamic> task,
    String projectName,
  ) async {
    final taskId = task['id'] as String? ?? '';
    final taskTitle = task['title'] as String? ?? '';

    final value = await showMenu<String>(
      context: context,
      position: RelativeRect.fromLTRB(position.dx, position.dy, position.dx + 1, position.dy + 1),
      color: DesktopTheme.bgSurfaceElevated,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: BorderSide(color: DesktopTheme.borderSubtle),
      ),
      items: [
        PopupMenuItem(
          value: 'rename',
          height: 32,
          child: Row(
            children: [
              Icon(Icons.edit_outlined, size: 14, color: DesktopTheme.textSecondary),
              const SizedBox(width: 8),
              Text('Переименовать', style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary)),
            ],
          ),
        ),
        PopupMenuItem(
          value: 'clear',
          height: 32,
          child: Row(
            children: [
              Icon(Icons.cleaning_services_outlined, size: 14, color: DesktopTheme.textSecondary),
              const SizedBox(width: 8),
              Text('Очистить историю', style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary)),
            ],
          ),
        ),
        const PopupMenuDivider(height: 1),
        PopupMenuItem(
          value: 'delete',
          height: 32,
          child: Row(
            children: [
              const Icon(Icons.delete_outline, size: 14, color: Color(0xFFEF4444)),
              const SizedBox(width: 8),
              const Text('Удалить задачу', style: TextStyle(fontSize: 12, color: Color(0xFFEF4444))),
            ],
          ),
        ),
      ],
    );

    if (value == null || !mounted || !context.mounted) return;
    if (!Get.isRegistered<DesktopTaskWorkspaceController>()) return;
    final controller = Get.find<DesktopTaskWorkspaceController>();

    if (value == 'rename') {
      _promptRenameTask(context, taskId, taskTitle, controller);
    } else if (value == 'clear') {
      controller.clearSessionHistory(taskId);
      Get.snackbar('История очищена', 'Сообщения задачи $taskId очищены');
    } else if (value == 'delete') {
      controller.deleteSession(taskId);
      setState(() {
        for (final p in projects) {
          (p['tasks'] as List).removeWhere((t) => t['id'] == taskId);
        }
      });
      Get.snackbar('Задача удалена', 'Сессия $taskId удалена');
    }
  }

  void _promptRenameTask(
    BuildContext context,
    String taskId,
    String currentTitle,
    DesktopTaskWorkspaceController controller,
  ) {
    final ctrl = TextEditingController(text: currentTitle);
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurfaceElevated,
        title: Text('Переименовать задачу', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: TextField(
          controller: ctrl,
          autofocus: true,
          style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 13),
          decoration: InputDecoration(
            hintText: 'Новое название...',
            hintStyle: TextStyle(color: DesktopTheme.textMuted),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(DesktopI18n.cancel, style: TextStyle(color: DesktopTheme.textMuted)),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: DesktopTheme.accentCyan, foregroundColor: Colors.black),
            onPressed: () {
              final newName = ctrl.text.trim();
              if (newName.isNotEmpty) {
                controller.renameSession(taskId, newName);
                setState(() {
                  for (final p in projects) {
                    for (final t in (p['tasks'] as List)) {
                      if (t['id'] == taskId) t['title'] = newName;
                    }
                  }
                });
              }
              Navigator.of(ctx).pop();
            },
            child: const Text('Сохранить'),
          ),
        ],
      ),
    );
  }

  Widget _buildActionRow({
    required IconData icon,
    required String label,
    required String shortcut,
    required VoidCallback onTap,
    bool isSelected = false,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(6),
      child: Container(
        decoration: BoxDecoration(
          color: isSelected ? const Color(0xFF282D37) : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        child: Row(
          children: [
            Icon(icon, size: 13, color: isSelected ? Colors.white : const Color(0xFFCBD5E1)),
            const SizedBox(width: 10),
            Expanded(
              child: Text(
                label,
                style: TextStyle(
                  fontSize: 13,
                  fontWeight: isSelected ? FontWeight.w600 : FontWeight.w500,
                  color: isSelected ? Colors.white : const Color(0xFFE2E8F0),
                ),
              ),
            ),
            if (shortcut.isNotEmpty)
              Text(
                shortcut,
                style: const TextStyle(
                  fontSize: 11,
                  fontFamily: 'Consolas',
                  color: Color(0xFF64748B),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
