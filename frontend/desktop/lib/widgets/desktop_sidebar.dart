// Desktop Workstation Collapsible Sidebar.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import '../theme/desktop_theme.dart';

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

  final List<_SidebarItem> items = const [
    _SidebarItem(icon: FontAwesomeIcons.terminal, label: 'Agent Workspace'),
    _SidebarItem(icon: FontAwesomeIcons.listCheck, label: 'Task Runs'),
    _SidebarItem(icon: FontAwesomeIcons.folderTree, label: 'Files & Artifacts'),
    _SidebarItem(icon: FontAwesomeIcons.brain, label: 'Semantic Memory'),
    _SidebarItem(icon: FontAwesomeIcons.clock, label: 'Automations (Cron)'),
    _SidebarItem(icon: FontAwesomeIcons.wrench, label: 'Tools & MCP'),
    _SidebarItem(icon: FontAwesomeIcons.stethoscope, label: 'Doctor Diagnostics'),
    _SidebarItem(icon: FontAwesomeIcons.microchip, label: 'LLM Studio & Models'),
    _SidebarItem(icon: FontAwesomeIcons.gear, label: 'Settings'),
  ];

  @override
  Widget build(BuildContext context) {
    final width = isCollapsed ? 60.0 : 250.0;

    return AnimatedContainer(
      duration: const Duration(milliseconds: 180),
      curve: Curves.easeInOut,
      width: width,
      decoration: const BoxDecoration(
        color: DesktopTheme.bgSidebar,
        border: Border(
          right: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Sidebar header toggle
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 10),
            child: Row(
              children: [
                if (!isCollapsed) ...[
                  const Expanded(
                    child: Text(
                      'NAVIGATION',
                      style: TextStyle(
                        fontSize: 10,
                        fontWeight: FontWeight.bold,
                        letterSpacing: 0.8,
                        color: DesktopTheme.textMuted,
                      ),
                    ),
                  ),
                ],
                IconButton(
                  tooltip: isCollapsed ? 'Expand Sidebar' : 'Collapse Sidebar',
                  icon: Icon(
                    isCollapsed ? Icons.menu : Icons.menu_open,
                    size: 16,
                    color: DesktopTheme.textMuted,
                  ),
                  onPressed: () => setState(() => isCollapsed = !isCollapsed),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(),
                ),
              ],
            ),
          ),

          // Navigation Links
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              itemCount: items.length,
              itemBuilder: (context, index) {
                final item = items[index];
                final isSelected = widget.selectedIndex == index;

                return InkWell(
                  onTap: () => widget.onSelectIndex(index),
                  borderRadius: BorderRadius.circular(6),
                  child: Container(
                    height: 36,
                    margin: const EdgeInsets.symmetric(vertical: 2),
                    padding: const EdgeInsets.symmetric(horizontal: 10),
                    decoration: BoxDecoration(
                      color: isSelected
                          ? DesktopTheme.accentSky.withOpacity(0.12)
                          : Colors.transparent,
                      borderRadius: BorderRadius.circular(6),
                      border: isSelected
                          ? Border.all(
                              color: DesktopTheme.accentSky.withOpacity(0.25),
                            )
                          : null,
                    ),
                    child: Row(
                      children: [
                        Icon(
                          item.icon,
                          size: 14,
                          color: isSelected
                              ? DesktopTheme.accentSky
                              : DesktopTheme.textSecondary,
                        ),
                        if (!isCollapsed) ...[
                          const SizedBox(width: 10),
                          Expanded(
                            child: Text(
                              item.label,
                              style: TextStyle(
                                fontSize: 13,
                                fontWeight: isSelected
                                    ? FontWeight.w600
                                    : FontWeight.normal,
                                color: isSelected
                                    ? DesktopTheme.textPrimary
                                    : DesktopTheme.textSecondary,
                              ),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                );
              },
            ),
          ),

          // Daemon telemetry footer
          if (!isCollapsed) _buildTelemetryFooter(),
        ],
      ),
    );
  }

  Widget _buildTelemetryFooter() {
    return Container(
      padding: const EdgeInsets.all(12),
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
              const Text(
                'Daemon Online',
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  color: DesktopTheme.textPrimary,
                ),
              ),
              const Spacer(),
              const Text(
                'PID 42617',
                style: TextStyle(
                  fontSize: 10,
                  fontFamily: 'Consolas',
                  color: DesktopTheme.textMuted,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          const Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('CPU: 0.4%', style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted)),
              Text('RAM: 42 MB', style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted)),
              Text('Channels: 4', style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted)),
            ],
          ),
        ],
      ),
    );
  }
}

class _SidebarItem {
  final IconData icon;
  final String label;

  const _SidebarItem({required this.icon, required this.label});
}
