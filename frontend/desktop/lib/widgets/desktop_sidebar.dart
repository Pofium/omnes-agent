// ZCode ADE Desktop Sidebar matching official ZCode UI screenshots.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';

class DesktopSidebar extends StatefulWidget {
  final int selectedIndex;
  final Function(int) onSelectIndex;
  final VoidCallback onNewTask;
  final VoidCallback onOpenSearch;
  final VoidCallback onOpenSettings;
  final VoidCallback onToggleSidebar;
  final VoidCallback? onOpenSkills;

  const DesktopSidebar({
    super.key,
    required this.selectedIndex,
    required this.onSelectIndex,
    required this.onNewTask,
    required this.onOpenSearch,
    required this.onOpenSettings,
    required this.onToggleSidebar,
    this.onOpenSkills,
  });

  @override
  State<DesktopSidebar> createState() => _DesktopSidebarState();
}

class _DesktopSidebarState extends State<DesktopSidebar> {
  final List<Map<String, dynamic>> sampleTasks = [
    {
      'title': 'Reproduce paper "Sparse Attention..."',
      'time': '1 day',
      'added': 142,
      'deleted': 28,
      'status': 'done',
    },
    {
      'title': 'Bot Channel Feishu binding flow...',
      'time': '3 days',
      'added': 84,
      'deleted': 12,
      'status': 'done',
    },
    {
      'title': 'MCP, Plugin and Command...',
      'time': '3 days',
      'added': 52,
      'deleted': 6,
      'status': 'done',
    },
    {
      'title': 'Edit history conversation: continue...',
      'time': '3 days',
      'added': 18,
      'deleted': 4,
      'status': 'done',
    },
    {
      'title': 'Safety confirmation: high-risk first...',
      'time': '3 days',
      'added': 36,
      'deleted': 9,
      'status': 'done',
    },
    {
      'title': 'Memory and output style: keep team...',
      'time': '3 days',
      'added': 12,
      'deleted': 0,
      'status': 'done',
    },
    {
      'title': 'Usage Stats: long-task usage, tools...',
      'time': '3 days',
      'added': 64,
      'deleted': 15,
      'status': 'done',
    },
  ];

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 250,
      decoration: const BoxDecoration(
        color: Color(0xFF141619),
        border: Border(right: BorderSide(color: Color(0xFF22262E), width: 1)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 1. Top Window Controls / Traffic Lights & Navigation
          Padding(
            padding: const EdgeInsets.only(left: 14, top: 12, right: 12, bottom: 8),
            child: Row(
              children: [
                // macOS traffic lights
                Row(
                  children: [
                    Container(width: 11, height: 11, decoration: const BoxDecoration(shape: BoxShape.circle, color: Color(0xFFFF5F56))),
                    const SizedBox(width: 6),
                    Container(width: 11, height: 11, decoration: const BoxDecoration(shape: BoxShape.circle, color: Color(0xFFFFBD2E))),
                    const SizedBox(width: 6),
                    Container(width: 11, height: 11, decoration: const BoxDecoration(shape: BoxShape.circle, color: Color(0xFF27C93F))),
                  ],
                ),
                const SizedBox(width: 14),
                // Counter badge
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                  decoration: BoxDecoration(
                    color: const Color(0xFF20232A),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: const Text(
                    '13',
                    style: TextStyle(fontSize: 10, color: Color(0xFF94A3B8), fontFamily: 'Consolas'),
                  ),
                ),
                const Spacer(),
                // Back / Forward
                Icon(Icons.arrow_back, size: 14, color: const Color(0xFF64748B)),
                const SizedBox(width: 6),
                Icon(Icons.arrow_forward, size: 14, color: const Color(0xFF475569)),
                const SizedBox(width: 10),
                // Collapse sidebar button
                InkWell(
                  onTap: widget.onToggleSidebar,
                  borderRadius: BorderRadius.circular(4),
                  child: const Icon(Icons.view_sidebar_outlined, size: 15, color: Color(0xFF94A3B8)),
                ),
              ],
            ),
          ),

          // 2. Primary Actions: + New Task, Search, Skills
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            child: Column(
              children: [
                _buildActionRow(
                  icon: FontAwesomeIcons.plus,
                  label: 'New Task',
                  shortcut: '⌘ N',
                  onTap: widget.onNewTask,
                ),
                _buildActionRow(
                  icon: FontAwesomeIcons.magnifyingGlass,
                  label: 'Search',
                  shortcut: '⌘ K',
                  onTap: widget.onOpenSearch,
                ),
                _buildActionRow(
                  icon: Icons.auto_awesome,
                  label: 'Skills',
                  shortcut: '',
                  onTap: widget.onOpenSkills ?? widget.onOpenSettings,
                ),
              ],
            ),
          ),

          const SizedBox(height: 6),

          // 3. Workspace Header with actions
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
            child: Row(
              children: [
                const Text(
                  'Workspace ↗',
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                    color: Color(0xFF64748B),
                  ),
                ),
                const Spacer(),
                Icon(Icons.menu, size: 14, color: const Color(0xFF64748B)),
                const SizedBox(width: 8),
                Icon(Icons.search, size: 14, color: const Color(0xFF64748B)),
                const SizedBox(width: 8),
                Icon(Icons.archive_outlined, size: 14, color: const Color(0xFF64748B)),
              ],
            ),
          ),

          // 4. Project Folder Header
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
            child: Row(
              children: const [
                Icon(FontAwesomeIcons.folder, size: 13, color: Color(0xFF00D2FF)),
                SizedBox(width: 8),
                Text(
                  'omnes-agent',
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.bold,
                    color: Colors.white,
                  ),
                ),
              ],
            ),
          ),

          // 5. Tasks List
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
              itemCount: sampleTasks.length,
              itemBuilder: (context, index) {
                final task = sampleTasks[index];
                final isSelected = widget.selectedIndex == index;

                return InkWell(
                  onTap: () => widget.onSelectIndex(index),
                  borderRadius: BorderRadius.circular(6),
                  child: Container(
                    margin: const EdgeInsets.only(bottom: 2),
                    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    decoration: BoxDecoration(
                      color: isSelected ? const Color(0xFF22262E) : Colors.transparent,
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Row(
                      children: [
                        Expanded(
                          child: Text(
                            task['title'],
                            style: TextStyle(
                              fontSize: 12,
                              color: isSelected ? Colors.white : const Color(0xFF94A3B8),
                              fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        const SizedBox(width: 6),
                        Text(
                          task['time'],
                          style: const TextStyle(
                            fontSize: 11,
                            color: Color(0xFF64748B),
                            fontFamily: 'Consolas',
                          ),
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),

          // Show less / more
          Padding(
            padding: const EdgeInsets.only(left: 20, bottom: 12),
            child: const Text(
              'Show less',
              style: TextStyle(fontSize: 11, color: Color(0xFF64748B)),
            ),
          ),

          // 6. Bottom User, Remote and Settings Bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            decoration: const BoxDecoration(
              border: Border(top: BorderSide(color: Color(0xFF22262E))),
            ),
            child: Row(
              children: [
                // Profile Avatar & Name
                Container(
                  width: 24,
                  height: 24,
                  decoration: BoxDecoration(
                    color: const Color(0xFF2B303C),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: const Center(
                    child: Text(
                      'O',
                      style: TextStyle(fontSize: 11, color: Colors.white, fontWeight: FontWeight.bold),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                const Expanded(
                  child: Text(
                    'OmnesAgent',
                    style: TextStyle(fontSize: 12, color: Color(0xFFCBD5E1), fontWeight: FontWeight.w500),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),

                // Mobile Remote button
                IconButton(
                  tooltip: 'Mobile Remote Control',
                  icon: const Icon(Icons.phone_iphone, size: 16, color: Color(0xFF94A3B8)),
                  onPressed: () {},
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 26, minHeight: 26),
                ),
                const SizedBox(width: 6),

                // Settings Gear Button
                IconButton(
                  tooltip: 'Settings (General, Appearance, Models, Memory)',
                  icon: const Icon(Icons.settings_outlined, size: 16, color: Color(0xFF00D2FF)),
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
