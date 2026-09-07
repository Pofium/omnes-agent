// Desktop Workstation ADE Sidebar with Task Groups, Git Diff Badges, and Navigation.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:omnes_shared/omnes_shared.dart';
import '../theme/desktop_theme.dart';

enum TaskViewMode { workspace, grouped, timeline }

class DesktopSidebar extends StatefulWidget {
  final int selectedIndex;
  final ValueChanged<int> onSelectIndex;

  const DesktopSidebar({
    super.key,
    required this.selectedIndex,
    required this.onSelectIndex,
  });

  @override
  State<DesktopSidebar> createState() => _DesktopSidebarState();
}

class _DesktopSidebarState extends State<DesktopSidebar> {
  bool isCollapsed = false;
  TaskViewMode viewMode = TaskViewMode.workspace;

  final List<_MockTaskItem> mockTasks = const [
    _MockTaskItem(
      id: 'task-1',
      title: 'Goal: Рефакторинг UI под ZCode ADE',
      time: '3m',
      status: 'running',
      diff: DiffBadgeInfo(addedLines: 142, deletedLines: 28),
    ),
    _MockTaskItem(
      id: 'task-2',
      title: 'Интеграция Live Browser & Element Picker',
      time: '24m',
      status: 'done',
      diff: DiffBadgeInfo(addedLines: 84, deletedLines: 12),
    ),
    _MockTaskItem(
      id: 'task-3',
      title: 'Поддержка Permission Mode (Shift+Tab)',
      time: '1h',
      status: 'done',
      diff: DiffBadgeInfo(addedLines: 52, deletedLines: 6),
    ),
    _MockTaskItem(
      id: 'task-4',
      title: 'Проверка компиляции Gateway WS фреймов',
      time: '1d',
      status: 'waiting',
      diff: DiffBadgeInfo(addedLines: 18, deletedLines: 4),
    ),
  ];

  final List<_SidebarItem> items = const [
    _SidebarItem(icon: FontAwesomeIcons.terminal, label: 'Agent Workspace'),
    _SidebarItem(icon: FontAwesomeIcons.listCheck, label: 'Task Runs'),
    _SidebarItem(icon: FontAwesomeIcons.folderTree, label: 'Files & Artifacts'),
    _SidebarItem(icon: FontAwesomeIcons.brain, label: 'Semantic Memory'),
    _SidebarItem(icon: FontAwesomeIcons.clock, label: 'Automations (Cron)'),
    _SidebarItem(icon: FontAwesomeIcons.wrench, label: 'Tools & MCP'),
    _SidebarItem(icon: FontAwesomeIcons.stethoscope, label: 'Doctor Diagnostics'),
    _SidebarItem(icon: FontAwesomeIcons.gear, label: 'Settings'),
  ];

  @override
  Widget build(BuildContext context) {
    final width = isCollapsed ? 60.0 : 280.0;

    return AnimatedContainer(
      duration: const Duration(milliseconds: 180),
      curve: Curves.easeInOut,
      width: width,
      decoration: BoxDecoration(
        color: DesktopTheme.bgSidebar,
        border: Border(
          right: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 1. Sidebar Header with Actions
          _buildSidebarHeader(),

          if (!isCollapsed) ...[
            // 2. View Mode Switcher [Workspace | Grouped | Timeline]
            _buildViewModeSwitcher(),
            const SizedBox(height: 6),

            // 3. Task List with Diff Badges
            _buildTaskListHeader(),
            Expanded(
              flex: 4,
              child: _buildTaskList(),
            ),

            const Divider(height: 1, color: DesktopTheme.borderSubtleDark),

            // 4. Secondary Navigation
            Expanded(
              flex: 3,
              child: _buildNavigationList(),
            ),
          ] else ...[
            // Collapsed Icons
            Expanded(
              child: _buildCollapsedIcons(),
            ),
          ],

          // 5. Telemetry Footer
          if (!isCollapsed) _buildTelemetryFooter(),
        ],
      ),
    );
  }

  Widget _buildSidebarHeader() {
    if (isCollapsed) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 12),
        child: Center(
          child: IconButton(
            tooltip: 'Развернуть сайдбар',
            icon: const Icon(Icons.menu, size: 18, color: DesktopTheme.textMutedDark),
            onPressed: () => setState(() => isCollapsed = false),
          ),
        ),
      );
    }

    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 10, 8, 8),
      child: Row(
        children: [
          // + New Task Button
          Expanded(
            child: ElevatedButton.icon(
              style: ElevatedButton.styleFrom(
                backgroundColor: DesktopTheme.accentCyan,
                foregroundColor: Colors.black,
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(6),
                ),
                elevation: 0,
              ),
              icon: const Icon(Icons.add, size: 16),
              label: const Text(
                'New Task',
                style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
              ),
              onPressed: () {},
            ),
          ),
          const SizedBox(width: 6),

          // Search / Skills
          IconButton(
            tooltip: 'Поиск (⌘K)',
            icon: const Icon(Icons.search, size: 16, color: DesktopTheme.textSecondaryDark),
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
            onPressed: () {},
          ),
          IconButton(
            tooltip: 'Свернуть сайдбар',
            icon: const Icon(Icons.menu_open, size: 16, color: DesktopTheme.textMutedDark),
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
            onPressed: () => setState(() => isCollapsed = true),
          ),
        ],
      ),
    );
  }

  Widget _buildViewModeSwitcher() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 10),
      child: Container(
        height: 26,
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(
          color: DesktopTheme.bgSurface,
          borderRadius: BorderRadius.circular(5),
          border: Border.all(color: DesktopTheme.borderSubtle),
        ),
        child: Row(
          children: [
            _buildModeTab('Workspace', TaskViewMode.workspace),
            _buildModeTab('Grouped', TaskViewMode.grouped),
            _buildModeTab('Timeline', TaskViewMode.timeline),
          ],
        ),
      ),
    );
  }

  Widget _buildModeTab(String label, TaskViewMode mode) {
    final isActive = viewMode == mode;
    return Expanded(
      child: InkWell(
        onTap: () => setState(() => viewMode = mode),
        borderRadius: BorderRadius.circular(4),
        child: Container(
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: isActive ? DesktopTheme.bgSurfaceElevated : Colors.transparent,
            borderRadius: BorderRadius.circular(4),
          ),
          child: Text(
            label,
            style: TextStyle(
              fontSize: 10,
              fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
              color: isActive ? DesktopTheme.accentCyan : DesktopTheme.textMuted,
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildTaskListHeader() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 8, 12, 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: const [
          Text(
            'TASKS',
            style: TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.bold,
              letterSpacing: 0.8,
              color: DesktopTheme.textMutedDark,
            ),
          ),
          Text(
            'DIFF',
            style: TextStyle(
              fontSize: 9,
              fontFamily: 'Consolas',
              color: DesktopTheme.textMutedDark,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTaskList() {
    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 8),
      itemCount: mockTasks.length,
      itemBuilder: (context, index) {
        final task = mockTasks[index];
        Color dotColor;
        switch (task.status) {
          case 'running':
            dotColor = DesktopTheme.statusInfo;
            break;
          case 'waiting':
            dotColor = DesktopTheme.statusWarning;
            break;
          case 'done':
          default:
            dotColor = DesktopTheme.statusSuccess;
            break;
        }

        final isSelected = index == 0;

        return InkWell(
          onTap: () {},
          borderRadius: BorderRadius.circular(6),
          child: Container(
            margin: const EdgeInsets.symmetric(vertical: 2),
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
            decoration: BoxDecoration(
              color: isSelected
                  ? DesktopTheme.accentSky.withOpacity(0.1)
                  : Colors.transparent,
              borderRadius: BorderRadius.circular(6),
              border: isSelected
                  ? Border.all(color: DesktopTheme.accentSky.withOpacity(0.25))
                  : null,
            ),
            child: Row(
              children: [
                // Status dot
                Container(
                  width: 6,
                  height: 6,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: dotColor,
                  ),
                ),
                const SizedBox(width: 8),

                // Task title & time
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        task.title,
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                          color: isSelected ? DesktopTheme.textPrimary : DesktopTheme.textSecondary,
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                      Text(
                        task.time,
                        style: const TextStyle(
                          fontSize: 10,
                          color: DesktopTheme.textMutedDark,
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 6),

                // Git Diff Badge (+142 -28)
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
                  decoration: BoxDecoration(
                    color: DesktopTheme.bgSurfaceElevated,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: DesktopTheme.borderSubtle),
                  ),
                  child: Text(
                    task.diff.formatted,
                    style: TextStyle(
                      fontSize: 9,
                      fontFamily: 'Consolas',
                      fontWeight: FontWeight.bold,
                      color: task.diff.addedLines > 0
                          ? DesktopTheme.statusSuccess
                          : DesktopTheme.textMuted,
                    ),
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildNavigationList() {
    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      itemCount: items.length,
      itemBuilder: (context, index) {
        final item = items[index];
        final isSelected = widget.selectedIndex == index;

        return InkWell(
          onTap: () => widget.onSelectIndex(index),
          borderRadius: BorderRadius.circular(6),
          child: Container(
            height: 32,
            margin: const EdgeInsets.symmetric(vertical: 1),
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(
              color: isSelected
                  ? DesktopTheme.accentSky.withOpacity(0.12)
                  : Colors.transparent,
              borderRadius: BorderRadius.circular(6),
            ),
            child: Row(
              children: [
                Icon(
                  item.icon,
                  size: 13,
                  color: isSelected
                      ? DesktopTheme.accentCyan
                      : DesktopTheme.textSecondary,
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    item.label,
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                      color: isSelected
                          ? DesktopTheme.textPrimary
                          : DesktopTheme.textSecondary,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildCollapsedIcons() {
    return ListView.builder(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: items.length,
      itemBuilder: (context, index) {
        final item = items[index];
        final isSelected = widget.selectedIndex == index;

        return IconButton(
          tooltip: item.label,
          icon: Icon(
            item.icon,
            size: 14,
            color: isSelected
                ? DesktopTheme.accentCyan
                : DesktopTheme.textSecondary,
          ),
          onPressed: () => widget.onSelectIndex(index),
        );
      },
    );
  }

  Widget _buildTelemetryFooter() {
    return Container(
      padding: const EdgeInsets.all(10),
      margin: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 6,
                height: 6,
                decoration: const BoxDecoration(
                  shape: BoxShape.circle,
                  color: DesktopTheme.statusSuccess,
                ),
              ),
              const SizedBox(width: 6),
              Text(
                'Daemon 127.0.0.1:42617',
                style: TextStyle(
                  fontSize: 10,
                  fontFamily: 'Consolas',
                  fontWeight: FontWeight.bold,
                  color: DesktopTheme.textPrimary,
                ),
              ),
              const Spacer(),
              const Text(
                'Memory: OK',
                style: TextStyle(
                  fontSize: 9,
                  color: DesktopTheme.statusSuccess,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _MockTaskItem {
  final String id;
  final String title;
  final String time;
  final String status;
  final DiffBadgeInfo diff;

  const _MockTaskItem({
    required this.id,
    required this.title,
    required this.time,
    required this.status,
    required this.diff,
  });
}

class _SidebarItem {
  final IconData icon;
  final String label;

  const _SidebarItem({required this.icon, required this.label});
}
