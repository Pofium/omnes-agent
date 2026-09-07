// OmnesAgent ADE Desktop Sidebar matching authentic Windows ADE layout (Screenshot 2).
// Features Windows native header, # Group vs 📁 Project switcher, real projects hierarchy,
// and fixed bottom user profile with avatar, 'Lite' badge, mobile remote, and settings gear.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import '../features/onboarding/user_onboarding_dialog.dart';

class DesktopSidebar extends StatefulWidget {
  final int selectedIndex;
  final Function(int, String) onSelectTask;
  final VoidCallback onNewTask;
  final VoidCallback onOpenSearch;
  final VoidCallback onOpenSettings;
  final VoidCallback onToggleSidebar;
  final VoidCallback? onOpenSkills;
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
    required this.userProfile,
    required this.onOpenProfile,
  });

  @override
  State<DesktopSidebar> createState() => _DesktopSidebarState();
}

class _DesktopSidebarState extends State<DesktopSidebar> {
  bool isGroupView = false; // false: Project, true: Group

  final List<Map<String, dynamic>> projects = [
    {
      'name': 'Omnes agent',
      'tasks': [
        {'title': 'План миграции Dart фронтенда в агента Zero...', 'time': '2d', 'unread': false},
      ],
    },
    {
      'name': 'deepseek-harness-master',
      'tasks': [
        {'title': 'Запуск exe-файла', 'time': '1h', 'unread': false},
        {'title': 'Переключение на ветку desktop и запуск exe', 'time': '1d', 'unread': false},
      ],
    },
    {
      'name': 'us proxy vps',
      'tasks': [
        {'title': 'реши проблему с прокси нихера не работает', 'time': '7d', 'unread': false},
        {'title': 'Отладка прокси Hermes и LLM API', 'time': '7d', 'unread': false},
      ],
    },
    {
      'name': 'aprh.serv',
      'tasks': [
        {'title': 'проанализируй проект посмотри логи напи...', 'time': '15d', 'unread': false},
      ],
    },
    {
      'name': 'AmoParallel',
      'tasks': [
        {'title': 'Диагностика ошибки агента Antigravity IDE', 'time': '9d', 'unread': false},
        {'title': 'выполный план, только поставь веху и остан...', 'time': '19d', 'unread': true},
      ],
    },
  ];

  final List<Map<String, dynamic>> groups = [
    {
      'name': 'Личные диалоги и черновики',
      'tasks': [
        {'title': 'Идеи для оптимизации рендеринга Flutter Web', 'time': '3h', 'unread': false},
        {'title': 'Сравнение моделей GLM-5.3 и Claude 3.5 Sonnet', 'time': '1d', 'unread': false},
      ],
    },
    {
      'name': 'Исследования и архитектура',
      'tasks': [
        {'title': 'Дизайн-документ ob2h графа памяти', 'time': '2d', 'unread': false},
        {'title': 'Интеграция протокола MCP через WebSocket', 'time': '4d', 'unread': false},
      ],
    },
  ];

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 260,
      decoration: const BoxDecoration(
        color: Color(0xFF141619),
        border: Border(right: BorderSide(color: Color(0xFF22262E), width: 1)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 1. Top Windows ADE Header (Logo + Back/Forward + Collapse)
          Padding(
            padding: const EdgeInsets.only(left: 14, top: 12, right: 12, bottom: 8),
            child: Row(
              children: [
                // OmnesAgent Brand Logo / 'Z' Symbol
                Container(
                  width: 22,
                  height: 22,
                  decoration: BoxDecoration(
                    gradient: const LinearGradient(
                      colors: [Color(0xFF00D2FF), Color(0xFF0072FF)],
                      begin: Alignment.topLeft,
                      end: Alignment.bottomRight,
                    ),
                    borderRadius: BorderRadius.circular(5),
                    boxShadow: [
                      BoxShadow(
                        color: const Color(0xFF00D2FF).withOpacity(0.3),
                        blurRadius: 6,
                        offset: const Offset(0, 1),
                      ),
                    ],
                  ),
                  child: const Center(
                    child: Text(
                      'Ω',
                      style: TextStyle(fontSize: 13, color: Colors.white, fontWeight: FontWeight.bold),
                    ),
                  ),
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
                  label: 'New task',
                  shortcut: 'Ctrl+N',
                  onTap: widget.onNewTask,
                ),
                _buildActionRow(
                  icon: FontAwesomeIcons.magnifyingGlass,
                  label: 'Search',
                  shortcut: 'Ctrl+K',
                  onTap: widget.onOpenSearch,
                ),
                _buildActionRow(
                  icon: FontAwesomeIcons.bolt,
                  label: 'Automations',
                  shortcut: '',
                  onTap: () {},
                ),
                _buildActionRow(
                  icon: FontAwesomeIcons.puzzlePiece,
                  label: 'Plugin Marketplace',
                  shortcut: '',
                  onTap: widget.onOpenSkills ?? widget.onOpenSettings,
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
                          'Group',
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
                          'Project',
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

                // Quick actions right of tabs
                IconButton(
                  tooltip: 'Фильтр',
                  icon: const Icon(Icons.filter_list, size: 13, color: Color(0xFF64748B)),
                  onPressed: () {},
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                ),
                const SizedBox(width: 4),
                IconButton(
                  tooltip: 'Свернуть все',
                  icon: const Icon(Icons.unfold_less, size: 13, color: Color(0xFF64748B)),
                  onPressed: () {},
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                ),
              ],
            ),
          ),

          // 4. Projects / Groups Header
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
            child: Text(
              isGroupView ? 'Groups' : 'Projects',
              style: const TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.w600,
                color: Color(0xFF64748B),
                letterSpacing: 0.4,
              ),
            ),
          ),

          // 5. Scrollable Projects List
          Expanded(
            child: ListView(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              children: (isGroupView ? groups : projects).map((proj) {
                return _buildProjectItem(proj);
              }).toList(),
            ),
          ),

          // 6. Fixed Bottom User, Remote & Settings Bar (NEVER SCROLLED!)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: const BoxDecoration(
              color: Color(0xFF141619),
              border: Border(top: BorderSide(color: Color(0xFF22262E), width: 1)),
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
                            style: const TextStyle(fontSize: 11, color: Colors.white, fontWeight: FontWeight.bold),
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      ConstrainedBox(
                        constraints: const BoxConstraints(maxWidth: 110),
                        child: Text(
                          widget.userProfile.fullName,
                          style: const TextStyle(fontSize: 12, color: Color(0xFFE2E8F0), fontWeight: FontWeight.w500),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 6),

                // Lite Badge
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                  decoration: BoxDecoration(
                    color: const Color(0xFF22262E),
                    borderRadius: BorderRadius.circular(3),
                  ),
                  child: Text(
                    widget.userProfile.tier,
                    style: const TextStyle(fontSize: 10, color: Color(0xFF94A3B8), fontWeight: FontWeight.bold),
                  ),
                ),

                const Spacer(),

                // Mobile Remote Control
                IconButton(
                  tooltip: 'Удалённое управление (Mobile Remote)',
                  icon: const Icon(Icons.phone_iphone, size: 16, color: Color(0xFF94A3B8)),
                  onPressed: () {},
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 26, minHeight: 26),
                ),
                const SizedBox(width: 4),

                // Settings Gear Button
                IconButton(
                  tooltip: 'Настройки (Провайдеры, MCP, Память, Язык)',
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
  }

  Widget _buildProjectItem(Map<String, dynamic> proj) {
    final tasks = proj['tasks'] as List<dynamic>;

    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Project Folder Header
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            child: Row(
              children: [
                Icon(
                  isGroupView ? Icons.chat_bubble_outline : Icons.folder_outlined,
                  size: 13,
                  color: const Color(0xFF64748B),
                ),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    proj['name'],
                    style: const TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: Color(0xFFCBD5E1),
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),

          // Tasks under project
          ...tasks.map((task) {
            final hasUnread = task['unread'] == true;

            return InkWell(
              onTap: () {
                widget.onSelectTask(0, task['title']);
              },
              borderRadius: BorderRadius.circular(6),
              child: Container(
                margin: const EdgeInsets.only(bottom: 1, left: 6, right: 6),
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
                        style: const TextStyle(
                          fontSize: 12,
                          color: Color(0xFF94A3B8),
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    const SizedBox(width: 6),
                    Text(
                      task['time'],
                      style: const TextStyle(
                        fontSize: 10,
                        color: Color(0xFF64748B),
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

  Widget _buildActionRow({
    required IconData icon,
    required String label,
    required String shortcut,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(6),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        child: Row(
          children: [
            Icon(icon, size: 13, color: const Color(0xFFCBD5E1)),
            const SizedBox(width: 10),
            Expanded(
              child: Text(
                label,
                style: const TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.w500,
                  color: Color(0xFFE2E8F0),
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
