import 'package:flutter/services.dart';
// OmnesAgent ADE Desktop Sidebar matching authentic Windows ADE layout.
// Features Windows native header, # Group vs Project Project switcher, real projects hierarchy,
// resizable width support, default groups, project creation pop-up with auto-suggestions,
// project file tree browser, and high-contrast light theme buttons (solid black in light mode).

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:universal_io/io.dart' as universal_io;
import '../features/onboarding/user_onboarding_dialog.dart';
import '../features/workspace/task_workspace_controller.dart';
import '../theme/desktop_theme.dart';
import '../utils/desktop_i18n.dart';
import '../utils/project_scaffolding.dart';

class DesktopProject {
  final String id;
  String name;
  String path;
  String domain;
  List<String> suggestedSkills;
  List<String> suggestedAgents;
  List<String> suggestedTools;
  String? customInstructions;
  String colorHex;
  String iconName;
  bool gitInitialized;
  bool ob2hIndexed;

  DesktopProject({
    required this.id,
    required this.name,
    required this.path,
    this.domain = 'Общее',
    this.suggestedSkills = const [],
    this.suggestedAgents = const [],
    this.suggestedTools = const [],
    this.customInstructions,
    this.colorHex = '#00D2FF',
    this.iconName = 'code',
    this.gitInitialized = false,
    this.ob2hIndexed = false,
  });

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'path': path,
        'domain': domain,
        'suggestedSkills': suggestedSkills,
        'suggestedAgents': suggestedAgents,
        'suggestedTools': suggestedTools,
        'customInstructions': customInstructions,
        'colorHex': colorHex,
        'iconName': iconName,
        'gitInitialized': gitInitialized,
        'ob2hIndexed': ob2hIndexed,
      };

  factory DesktopProject.fromJson(Map<String, dynamic> j) => DesktopProject(
        id: j['id']?.toString() ?? '',
        name: j['name']?.toString() ?? '',
        path: j['path']?.toString() ?? '',
        domain: j['domain']?.toString() ?? 'Общее',
        suggestedSkills: List<String>.from(j['suggestedSkills'] ?? []),
        suggestedAgents: List<String>.from(j['suggestedAgents'] ?? []),
        suggestedTools: List<String>.from(j['suggestedTools'] ?? []),
        customInstructions: j['customInstructions']?.toString(),
        colorHex: j['colorHex']?.toString() ?? '#00D2FF',
        iconName: j['iconName']?.toString() ?? 'code',
        gitInitialized: j['gitInitialized'] == true,
        ob2hIndexed: j['ob2hIndexed'] == true,
      );
}

class DesktopGroup {
  final String id;
  String name;
  IconData icon;
  String? boundProjectId;
  String? artifactsFolder;

  DesktopGroup({
    required this.id,
    required this.name,
    required this.icon,
    this.boundProjectId,
    this.artifactsFolder,
  });

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'codePoint': icon.codePoint,
    'fontFamily': icon.fontFamily,
    'fontPackage': icon.fontPackage,
    'boundProjectId': boundProjectId,
    'artifactsFolder': artifactsFolder,
  };

  factory DesktopGroup.fromJson(Map<String, dynamic> json) => DesktopGroup(
    id: json['id']?.toString() ?? 'misc',
    name: json['name']?.toString() ?? 'Разное',
    icon: IconData(
      json['codePoint'] as int? ?? FontAwesomeIcons.shapes.codePoint,
      fontFamily: json['fontFamily']?.toString() ?? 'FontAwesomeSolid',
      fontPackage: json['fontPackage']?.toString() ?? 'font_awesome_flutter',
    ),
    boundProjectId: json['boundProjectId']?.toString(),
    artifactsFolder: json['artifactsFolder']?.toString(),
  );
}

class DesktopSidebar extends StatefulWidget {
  final double width;
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
  final Function(String filePath)? onOpenFile;
  final void Function(int tabIndex)? onSelectInspectorTab;

  const DesktopSidebar({
    super.key,
    this.width = 260.0,
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
    this.onOpenFile,
    this.onSelectInspectorTab,
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
  final Set<String> collapsedSections = {};
  final Set<String> expandedProjectDirs = {};

  // Active Project for File Tree mode
  DesktopProject? activeFileTreeProject;
  String fileTreeSearchQuery = '';
  final TextEditingController fileTreeSearchCtrl = TextEditingController();

  // 20 Group icons catalog
  static const List<Map<String, dynamic>> groupIconsCatalog = [
    {'name': 'Разное', 'icon': FontAwesomeIcons.shapes},
    {'name': 'Дом', 'icon': FontAwesomeIcons.house},
    {'name': 'Работа', 'icon': FontAwesomeIcons.briefcase},
    {'name': 'Семья', 'icon': FontAwesomeIcons.peopleGroup},
    {'name': 'Игры', 'icon': FontAwesomeIcons.gamepad},
    {'name': 'Поездки', 'icon': FontAwesomeIcons.planeDeparture},
    {'name': 'Время', 'icon': FontAwesomeIcons.clock},
    {'name': 'Папка', 'icon': FontAwesomeIcons.folder},
    {'name': 'Код', 'icon': FontAwesomeIcons.code},
    {'name': 'Сервер', 'icon': FontAwesomeIcons.server},
    {'name': 'Защита', 'icon': FontAwesomeIcons.shieldHalved},
    {'name': 'Молния', 'icon': FontAwesomeIcons.bolt},
    {'name': 'Сердце', 'icon': FontAwesomeIcons.heart},
    {'name': 'График', 'icon': FontAwesomeIcons.chartLine},
    {'name': 'Книга', 'icon': FontAwesomeIcons.bookOpen},
    {'name': 'Идея', 'icon': FontAwesomeIcons.lightbulb},
    {'name': 'Авто', 'icon': FontAwesomeIcons.car},
    {'name': 'Компас', 'icon': FontAwesomeIcons.compass},
    {'name': 'Музыка', 'icon': FontAwesomeIcons.music},
    {'name': 'Ключ', 'icon': FontAwesomeIcons.wrench},
  ];

  // Default Groups list
  late final List<DesktopGroup> groups = [
    DesktopGroup(id: 'misc', name: 'Разное', icon: FontAwesomeIcons.shapes),
    DesktopGroup(id: 'home', name: 'Дом', icon: FontAwesomeIcons.house),
    DesktopGroup(id: 'work', name: 'Работа', icon: FontAwesomeIcons.briefcase),
    DesktopGroup(id: 'family', name: 'Семья', icon: FontAwesomeIcons.peopleGroup),
    DesktopGroup(id: 'fun', name: 'Развлечения', icon: FontAwesomeIcons.gamepad),
    DesktopGroup(id: 'travel', name: 'Поездки и путешествия', icon: FontAwesomeIcons.planeDeparture),
    DesktopGroup(id: 'temp', name: 'Временное', icon: FontAwesomeIcons.clock),
  ];

  // Projects list
  final List<DesktopProject> projects = [];
  final GetStorage _storage = GetStorage();

  @override
  void initState() {
    super.initState();
    _loadProjects();
    _loadGroups();
  }

  void _saveGroups() {
    try {
      _storage.write(
        'user_groups_registry',
        groups.map((g) => g.toJson()).toList(),
      );
    } catch (_) {}
  }

  void _loadGroups() {
    try {
      final raw = _storage.read<List>('user_groups_registry');
      if (raw != null && raw.isNotEmpty) {
        groups.clear();
        for (final item in raw) {
          if (item is Map) {
            groups.add(DesktopGroup.fromJson(Map<String, dynamic>.from(item)));
          }
        }
      }
    } catch (_) {}
  }

  void _loadProjects() {
    final raw = _storage.read<List>('user_projects_registry');
    if (raw != null && raw.isNotEmpty) {
      projects.clear();
      for (final item in raw) {
        if (item is Map) {
          projects.add(DesktopProject.fromJson(Map<String, dynamic>.from(item)));
        }
      }
    } else {
      projects.addAll([
        DesktopProject(
          id: 'omnes_agent',
          name: 'Omnes agent',
          path: r'C:\Projects\Omnes-agent',
          domain: 'Rust / Системная разработка',
          suggestedAgents: ['chief', 'code-agent'],
          suggestedTools: ['cargo', 'git'],
          suggestedSkills: ['ob2h'],
        ),
        DesktopProject(
          id: 'deepseek_harness',
          name: 'deepseek-harness-master',
          path: r'C:\Projects\Omnes-agent',
          domain: 'AI / Data Science',
          suggestedAgents: ['chief'],
          suggestedTools: ['python'],
          suggestedSkills: ['science'],
        ),
      ]);
    }
  }

  void _saveProjects() {
    _storage.write(
      'user_projects_registry',
      projects.map((p) => p.toJson()).toList(),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Obx(() {
      return Container(
        width: widget.width,
        decoration: BoxDecoration(
          color: DesktopTheme.bgSidebar,
          border: Border(right: BorderSide(color: DesktopTheme.borderSubtle, width: 1)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 1. Top Windows ADE Header (Logo + Back/Forward + Collapse)
            Padding(
              padding: const EdgeInsets.only(left: 12, right: 10, top: 12, bottom: 8),
              child: Row(
                children: [
                  SvgPicture.asset(
                    'assets/Logo/OA_icon.svg',
                    width: 22,
                    height: 22,
                    fit: BoxFit.contain,
                  ),
                  const SizedBox(width: 8),
                  Text(
                    'OmnesAgent',
                    style: TextStyle(
                      fontSize: 13,
                      fontWeight: FontWeight.bold,
                      letterSpacing: 0.2,
                      color: DesktopTheme.textPrimary,
                    ),
                  ),
                  const Spacer(),
                  Tooltip(
                    message: DesktopI18n.tr('Скрыть боковую панель', 'Collapse sidebar'),
                    child: InkWell(
                      onTap: widget.onToggleSidebar,
                      borderRadius: BorderRadius.circular(4),
                      child: Transform.flip(
                        flipX: true,
                        child: Icon(Icons.view_sidebar_outlined, size: 15, color: DesktopTheme.textMuted),
                      ),
                    ),
                  ),
                ],
              ),
            ),

            // If Project File Tree mode is active, render the file tree instead of lists
            if (activeFileTreeProject != null)
              Expanded(child: _buildProjectFileTree(activeFileTreeProject!))
            else ...[
              // 2. Primary Actions: + New session, Search, Automations
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                child: Column(
                  children: [
                    _buildActionRow(
                      icon: FontAwesomeIcons.plus,
                      label: 'Новая сессия',
                      shortcut: 'Ctrl+N',
                      isPrimary: true,
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

              // 3. Tab Switcher: [ # Группа ]  [ Project Проект ]
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                child: Row(
                  children: [
                    // Group button
                    InkWell(
                      onTap: () => setState(() => isGroupView = true),
                      borderRadius: BorderRadius.circular(6),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
                        decoration: BoxDecoration(
                          color: isGroupView
                              ? (DesktopTheme.isDark ? const Color(0xFF282D37) : const Color(0xFF0F172A))
                              : Colors.transparent,
                          borderRadius: BorderRadius.circular(6),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(
                              FontAwesomeIcons.hashtag,
                              size: 11,
                              color: isGroupView ? Colors.white : DesktopTheme.textMuted,
                            ),
                            const SizedBox(width: 5),
                            Text(
                              DesktopI18n.group,
                              style: TextStyle(
                                fontSize: 12,
                                fontWeight: isGroupView ? FontWeight.bold : FontWeight.w500,
                                color: isGroupView ? Colors.white : DesktopTheme.textSecondary,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                    const SizedBox(width: 6),

                    // Project button
                    InkWell(
                      onTap: () => setState(() => isGroupView = false),
                      borderRadius: BorderRadius.circular(6),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 5),
                        decoration: BoxDecoration(
                          color: !isGroupView
                              ? (DesktopTheme.isDark ? const Color(0xFF282D37) : const Color(0xFF0F172A))
                              : Colors.transparent,
                          borderRadius: BorderRadius.circular(6),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(
                              Icons.folder_outlined,
                              size: 13,
                              color: !isGroupView
                                  ? (DesktopTheme.isDark ? const Color(0xFF00D2FF) : Colors.white)
                                  : DesktopTheme.textMuted,
                            ),
                            const SizedBox(width: 5),
                            Text(
                              DesktopI18n.project,
                              style: TextStyle(
                                fontSize: 12,
                                fontWeight: !isGroupView ? FontWeight.bold : FontWeight.w500,
                                color: !isGroupView ? Colors.white : DesktopTheme.textSecondary,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),

                    const Spacer(),

                    // Expand / Collapse all
                    IconButton(
                      tooltip: isAllExpanded ? DesktopI18n.collapseAll : DesktopI18n.expandAll,
                      icon: Icon(
                        isAllExpanded ? Icons.close_fullscreen : Icons.open_in_full,
                        size: 13,
                        color: DesktopTheme.textMuted,
                      ),
                      onPressed: () {
                        setState(() {
                          isAllExpanded = !isAllExpanded;
                          if (!isAllExpanded) {
                            if (isGroupView) {
                              collapsedSections.addAll(groups.map((g) => g.name));
                            } else {
                              collapsedSections.addAll(projects.map((p) => p.name));
                            }
                          } else {
                            collapsedSections.clear();
                          }
                        });
                      },
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(minWidth: 22, minHeight: 22),
                    ),
                    const SizedBox(width: 4),

                    // Working Filter & Sort Popover Menu
                    _buildFilterPopupMenu(),
                  ],
                ),
              ),

              // 4. Section Header with Add Project (+) or AI Grouping (✨)
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                child: Row(
                  children: [
                    Text(
                      isGroupView
                          ? DesktopI18n.groups.toUpperCase()
                          : (viewMode == SidebarViewMode.timeline ? DesktopI18n.timeline : DesktopI18n.projects).toUpperCase(),
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w700,
                        color: DesktopTheme.textMuted,
                        letterSpacing: 0.6,
                      ),
                    ),
                    const Spacer(),
                    if (isGroupView) ...[
                      // AI Grouping button
                      IconButton(
                        tooltip: 'Сгруппировать через ИИ',
                        icon: const Icon(Icons.auto_awesome, size: 14, color: Color(0xFF00D2FF)),
                        onPressed: _showAiGroupingDialog,
                        padding: EdgeInsets.zero,
                        constraints: const BoxConstraints(minWidth: 22, minHeight: 22),
                      ),
                      const SizedBox(width: 4),
                      IconButton(
                        tooltip: 'Добавить группу',
                        icon: const Icon(Icons.add, size: 14, color: Color(0xFF94A3B8)),
                        onPressed: _showAddGroupDialog,
                        padding: EdgeInsets.zero,
                        constraints: const BoxConstraints(minWidth: 22, minHeight: 22),
                      ),
                    ] else ...[
                      // Add Project (+) button
                      IconButton(
                        tooltip: 'Создать проект',
                        icon: const Icon(Icons.add, size: 15, color: Color(0xFF00D2FF)),
                        onPressed: _showAddProjectDialog,
                        padding: EdgeInsets.zero,
                        constraints: const BoxConstraints(minWidth: 22, minHeight: 22),
                      ),
                    ],
                  ],
                ),
              ),

              // 5. Scrollable Projects or Groups List with Real Sessions
              Expanded(
                child: Obx(() {
                  final workspaceCtrl = Get.isRegistered<DesktopTaskWorkspaceController>()
                      ? Get.find<DesktopTaskWorkspaceController>()
                      : null;

                  final allSessions = workspaceCtrl?.sessions.values.toList() ?? [];

                  if (viewMode == SidebarViewMode.timeline) {
                    return ListView(
                      padding: const EdgeInsets.symmetric(horizontal: 8),
                      children: _buildTimelineItems(allSessions),
                    );
                  }

                  if (isGroupView) {
                    return ListView(
                      padding: const EdgeInsets.symmetric(horizontal: 8),
                      children: groups.map((g) {
                        final groupSessions = allSessions.where((s) => s.group == g.name).toList();
                        return _buildGroupItem(g, groupSessions);
                      }).toList(),
                    );
                  }

                  return ListView(
                    padding: const EdgeInsets.symmetric(horizontal: 8),
                    children: projects.map((p) {
                      final projSessions = allSessions
                          .where((s) => s.project == p.name || (s.projectPath != null && s.projectPath == p.path))
                          .toList();
                      return _buildProjectItem(p, projSessions);
                    }).toList(),
                  );
                }),
              ),
            ],

            // 6. Fixed Bottom User Profile, Remote & Settings Bar
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

  // ==========================================
  // ACTION ROW (NEW SESSION / SEARCH / AUTO)
  // HIGH CONTRAST IN LIGHT THEME
  // ==========================================
  Widget _buildActionRow({
    required IconData icon,
    required String label,
    required String shortcut,
    required VoidCallback onTap,
    bool isSelected = false,
    bool isPrimary = false,
  }) {
    final isDark = DesktopTheme.isDark;
    final Color bgColor;
    final Color fgColor;
    final Color iconColor;
    final Border? border;

    if (isPrimary) {
      bgColor = isDark ? const Color(0xFF282D37) : const Color(0xFF0F172A);
      fgColor = Colors.white;
      iconColor = isDark ? const Color(0xFF00D2FF) : Colors.white;
      border = Border.all(color: isDark ? const Color(0xFF00D2FF).withOpacity(0.3) : const Color(0xFF0F172A));
    } else if (isSelected) {
      bgColor = isDark ? const Color(0xFF282D37) : const Color(0xFF0F172A);
      fgColor = Colors.white;
      iconColor = isDark ? const Color(0xFF00D2FF) : Colors.white;
      border = Border.all(color: isDark ? const Color(0xFF3B82F6) : const Color(0xFF0F172A));
    } else {
      bgColor = Colors.transparent;
      fgColor = isDark ? const Color(0xFFE2E8F0) : const Color(0xFF0F172A);
      iconColor = isDark ? const Color(0xFFCBD5E1) : const Color(0xFF475569);
      border = null;
    }

    return Container(
      margin: const EdgeInsets.only(bottom: 3),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(6),
        child: Container(
          decoration: BoxDecoration(
            color: bgColor,
            borderRadius: BorderRadius.circular(6),
            border: border,
          ),
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
          child: Row(
            children: [
              Icon(icon, size: 13, color: iconColor),
              const SizedBox(width: 10),
              Expanded(
                child: Text(
                  label,
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: (isPrimary || isSelected) ? FontWeight.w600 : FontWeight.w500,
                    color: fgColor,
                  ),
                ),
              ),
              if (shortcut.isNotEmpty)
                Text(
                  shortcut,
                  style: TextStyle(
                    fontSize: 11,
                    fontFamily: 'Consolas',
                    color: (isPrimary || isSelected) ? Colors.white70 : const Color(0xFF64748B),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  // ==========================================
  // PROJECT ITEM WITH FILE TREE BUTTON
  // ==========================================
  Widget _buildProjectItem(DesktopProject proj, List<TaskSession> sessions) {
    final isCollapsed = collapsedSections.contains(proj.name);

    Color projColor = const Color(0xFF00D2FF);
    try {
      final clean = proj.colorHex.replaceAll('#', '');
      if (clean.length == 6) {
        projColor = Color(int.parse('0xFF$clean'));
      }
    } catch (_) {}

    IconData projIcon = Icons.folder_outlined;
    switch (proj.iconName) {
      case 'code':
        projIcon = Icons.code_rounded;
        break;
      case 'terminal':
        projIcon = Icons.terminal_rounded;
        break;
      case 'globe':
        projIcon = Icons.language_rounded;
        break;
      case 'cpu':
        projIcon = Icons.memory_rounded;
        break;
      case 'layers':
        projIcon = Icons.layers_outlined;
        break;
      default:
        projIcon = Icons.folder_outlined;
    }

    return Container(
      margin: const EdgeInsets.only(bottom: 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InkWell(
            onTap: () {
              setState(() {
                if (isCollapsed) {
                  collapsedSections.remove(proj.name);
                } else {
                  collapsedSections.add(proj.name);
                }
              });
            },
            borderRadius: BorderRadius.circular(6),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
              child: Row(
                children: [
                  Icon(
                    isCollapsed ? Icons.chevron_right : Icons.keyboard_arrow_down,
                    size: 14,
                    color: DesktopTheme.textMuted,
                  ),
                  const SizedBox(width: 4),
                  Icon(projIcon, size: 13, color: projColor),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      proj.name,
                      style: TextStyle(
                        fontSize: 12,
                        fontWeight: FontWeight.bold,
                        color: DesktopTheme.textPrimary,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  // Button to create session in project (left of folder)
                  IconButton(
                    tooltip: 'Создать сессию в проекте',
                    icon: const Icon(Icons.add, size: 13, color: Color(0xFF94A3B8)),
                    onPressed: () {
                      final ctrl = Get.find<DesktopTaskWorkspaceController>();
                      ctrl.switchToSession(
                        'sess_${DateTime.now().millisecondsSinceEpoch}',
                        group: proj.name,
                        project: proj.name,
                        projectPath: proj.path,
                      );
                    },
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                  ),
                  // Button to browse project file tree
                  IconButton(
                    tooltip: 'Открыть дерево файлов проекта',
                    icon: Icon(Icons.folder_open, size: 14, color: projColor),
                    onPressed: () {
                      setState(() {
                        activeFileTreeProject = proj;
                      });
                    },
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 22, minHeight: 22),
                  ),
                  // Project Context Menu Button (3 dots)
                  PopupMenuButton<String>(
                    tooltip: 'Действия с проектом',
                    offset: const Offset(0, 24),
                    color: DesktopTheme.bgSurface,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(8),
                      side: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
                    ),
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                    icon: Icon(Icons.more_vert, size: 13, color: DesktopTheme.textMuted),
                    onSelected: (val) {
                      if (val == 'tree') {
                        setState(() => activeFileTreeProject = proj);
                      } else if (val == 'explorer') {
                        ProjectScaffoldingService.openInExplorer(proj.path);
                      } else if (val == 'code') {
                        ProjectScaffoldingService.openInCode(proj.path);
                      } else if (val == 'terminal') {
                        ProjectScaffoldingService.openInTerminal(proj.path);
                      } else if (val == 'ob2h') {
                        ProjectScaffoldingService.runOb2hScanAsync(proj.path);
                        Get.snackbar(
                          'OB2H AST-сканирование',
                          'Индексация проекта запущена в фоновом режиме',
                          backgroundColor: const Color(0xFF0F172A),
                          colorText: const Color(0xFFA855F7),
                          snackPosition: SnackPosition.BOTTOM,
                        );
                      } else if (val == 'rename') {
                        _showRenameProjectDialog(proj);
                      } else if (val == 'path') {
                        _showChangeProjectPathDialog(proj);
                      } else if (val == 'delete') {
                        setState(() {
                          projects.remove(proj);
                          _saveProjects();
                        });
                      }
                    },
                    itemBuilder: (ctx) => [
                      PopupMenuItem(
                        value: 'tree',
                        child: Row(children: [
                          Icon(Icons.folder_open, size: 14, color: projColor),
                          const SizedBox(width: 8),
                          const Text('Открыть дерево файлов проекта'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'explorer',
                        child: Row(children: const [
                          Icon(Icons.folder_special_outlined, size: 14, color: Color(0xFF00D2FF)),
                          SizedBox(width: 8),
                          Text('Открыть в Проводнике'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'code',
                        child: Row(children: const [
                          Icon(Icons.code, size: 14, color: Color(0xFF38BDF8)),
                          SizedBox(width: 8),
                          Text('Открыть в VS Code / IDE'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'terminal',
                        child: Row(children: const [
                          Icon(Icons.terminal, size: 14, color: Color(0xFF10B981)),
                          SizedBox(width: 8),
                          Text('Открыть терминал в папке'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'ob2h',
                        child: Row(children: const [
                          Icon(Icons.psychology_outlined, size: 14, color: Color(0xFFA855F7)),
                          SizedBox(width: 8),
                          Text('Переиндексировать в OB2H'),
                        ]),
                      ),
                      const PopupMenuDivider(height: 1),
                      PopupMenuItem(
                        value: 'rename',
                        child: Row(children: const [
                          Icon(Icons.edit_outlined, size: 14),
                          SizedBox(width: 8),
                          Text('Переименовать'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'path',
                        child: Row(children: const [
                          Icon(Icons.link, size: 14),
                          SizedBox(width: 8),
                          Text('Сменить путь'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'delete',
                        child: Row(children: const [
                          Icon(Icons.delete_outline, color: Colors.redAccent, size: 14),
                          SizedBox(width: 8),
                          Text('Удалить проект', style: TextStyle(color: Colors.redAccent)),
                        ]),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),

          if (!isCollapsed) ...[
            if (sessions.isEmpty)
              Padding(
                padding: const EdgeInsets.only(left: 28, top: 2, bottom: 4),
                child: Text(
                  'Нет активных сессий',
                  style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted, fontStyle: FontStyle.italic),
                ),
              )
            else
              ...sessions.map((s) => _buildSessionRow(s, proj.name)),
          ],
        ],
      ),
    );
  }

  // ==========================================
  // GROUP ITEM
  // ==========================================
  Widget _buildGroupItem(DesktopGroup group, List<TaskSession> sessions) {
    final isCollapsed = collapsedSections.contains(group.name);

    return Container(
      margin: const EdgeInsets.only(bottom: 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InkWell(
            onTap: () {
              setState(() {
                if (isCollapsed) {
                  collapsedSections.remove(group.name);
                } else {
                  collapsedSections.add(group.name);
                }
              });
            },
            onSecondaryTapDown: (details) => _showGroupContextMenu(context, details.globalPosition, group),
            borderRadius: BorderRadius.circular(6),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
              child: Row(
                children: [
                  Icon(
                    isCollapsed ? Icons.chevron_right : Icons.keyboard_arrow_down,
                    size: 14,
                    color: DesktopTheme.textMuted,
                  ),
                  const SizedBox(width: 4),
                  Icon(group.icon, size: 12, color: const Color(0xFF00D2FF)),
                  const SizedBox(width: 7),
                  Expanded(
                    child: Text(
                      group.name,
                      style: TextStyle(
                        fontSize: 12,
                        fontWeight: FontWeight.bold,
                        color: DesktopTheme.textPrimary,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  IconButton(
                    tooltip: 'Создать сессию в группе',
                    icon: const Icon(Icons.add, size: 13, color: Color(0xFF94A3B8)),
                    onPressed: () {
                      final ctrl = Get.find<DesktopTaskWorkspaceController>();
                      ctrl.switchToSession('sess_${DateTime.now().millisecondsSinceEpoch}', group: group.name);
                    },
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                  ),
                  // Group Context Menu Button (3 dots)
                  PopupMenuButton<String>(
                    tooltip: 'Действия с группой',
                    offset: const Offset(0, 24),
                    color: DesktopTheme.bgSurface,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(8),
                      side: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
                    ),
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                    icon: Icon(Icons.more_vert, size: 13, color: DesktopTheme.textMuted),
                    onSelected: (val) {
                      if (val == 'new_session') {
                        final ctrl = Get.find<DesktopTaskWorkspaceController>();
                        ctrl.switchToSession('sess_${DateTime.now().millisecondsSinceEpoch}', group: group.name);
                      } else if (val == 'rename') {
                        _showRenameGroupDialog(group);
                        _saveGroups();
                      } else if (val == 'icon') {
                        _showChangeGroupIconDialog(group);
                        _saveGroups();
                      } else if (val == 'artifacts') {
                        _showGroupArtifactsDialog(group);
                      } else if (val == 'delete') {
                        setState(() { groups.remove(group); _saveGroups(); });
                      }
                    },
                    itemBuilder: (ctx) => [
                      PopupMenuItem(
                        value: 'new_session',
                        child: Row(children: const [
                          Icon(Icons.add, size: 14),
                          SizedBox(width: 8),
                          Text('Новая сессия в группе'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'rename',
                        child: Row(children: const [
                          Icon(Icons.edit_outlined, size: 14),
                          SizedBox(width: 8),
                          Text('Переименовать'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'icon',
                        child: Row(children: const [
                          Icon(Icons.palette_outlined, size: 14),
                          SizedBox(width: 8),
                          Text('Сменить иконку (20 иконок)'),
                        ]),
                      ),
                      PopupMenuItem(
                        value: 'artifacts',
                        child: Row(children: const [
                          Icon(Icons.folder_zip_outlined, size: 14),
                          SizedBox(width: 8),
                          Text('Папка артефактов'),
                        ]),
                      ),
                      if (group.name != 'Разное')
                        PopupMenuItem(
                          value: 'delete',
                          child: Row(children: const [
                            Icon(Icons.delete_outline, color: Colors.redAccent, size: 14),
                            SizedBox(width: 8),
                            Text('Удалить группу', style: TextStyle(color: Colors.redAccent)),
                          ]),
                        ),
                    ],
                  ),
                ],
              ),
            ),
          ),

          if (!isCollapsed) ...[
            if (sessions.isEmpty)
              Padding(
                padding: const EdgeInsets.only(left: 28, top: 2, bottom: 4),
                child: Text(
                  'Нет сессий в группе',
                  style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted, fontStyle: FontStyle.italic),
                ),
              )
            else
              ...sessions.map((s) => _buildSessionRow(s, group.name)),
          ],
        ],
      ),
    );
  }

  // ==========================================
  // SESSION ROW
  // ==========================================
  Widget _buildSessionRow(TaskSession session, String parentContext) {
    final workspaceCtrl = Get.find<DesktopTaskWorkspaceController>();
    final isSelected = workspaceCtrl.activeSessionId.value == session.id;

    return InkWell(
      onTap: () {
        widget.onSelectTask(session.id, session.title, parentContext);
      },
      onSecondaryTapDown: (details) => _showSessionContextMenu(context, details.globalPosition, session),
      borderRadius: BorderRadius.circular(6),
      child: Container(
        margin: const EdgeInsets.only(bottom: 1, left: 16, right: 4),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        decoration: BoxDecoration(
          color: isSelected
              ? (DesktopTheme.isDark ? const Color(0xFF282D37) : const Color(0xFF0F172A).withOpacity(0.1))
              : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
          border: isSelected ? Border.all(color: const Color(0xFF00D2FF).withOpacity(0.4), width: 0.8) : null,
        ),
        child: Row(
          children: [
            Icon(
              Icons.chat_bubble_outline,
              size: 11,
              color: isSelected ? const Color(0xFF00D2FF) : DesktopTheme.textMuted,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                session.title,
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                  color: isSelected ? DesktopTheme.textPrimary : DesktopTheme.textSecondary,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
            // Session Context Menu Button (3 dots)
            PopupMenuButton<String>(
              tooltip: 'Действия с сессией',
              offset: const Offset(0, 24),
              color: DesktopTheme.bgSurface,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(8),
                side: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
              ),
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
              icon: Icon(Icons.more_vert, size: 12, color: DesktopTheme.textMuted),
              onSelected: (val) {
                if (val == 'rename') {
                  _showRenameSessionDialog(session);
                } else if (val == 'move') {
                  _showMoveSessionToGroupDialog(session);
                } else if (val == 'bind') {
                  _showBindSessionToProjectDialog(session);
                } else if (val == 'delete') {
                  _showDeleteSessionDialog(session);
                }
              },
              itemBuilder: (ctx) => [
                PopupMenuItem(
                  value: 'rename',
                  child: Row(children: const [
                    Icon(Icons.edit_outlined, size: 14),
                    SizedBox(width: 8),
                    Text('Переименовать'),
                  ]),
                ),
                PopupMenuItem(
                  value: 'move',
                  child: Row(children: const [
                    Icon(Icons.drive_file_move_outlined, size: 14),
                    SizedBox(width: 8),
                    Text('Переместить в группу'),
                  ]),
                ),
                PopupMenuItem(
                  value: 'bind',
                  child: Row(children: const [
                    Icon(Icons.folder_outlined, size: 14),
                    SizedBox(width: 8),
                    Text('Привязать к проекту'),
                  ]),
                ),
                PopupMenuItem(
                  value: 'delete',
                  child: Row(children: const [
                    Icon(Icons.delete_outline, color: Colors.redAccent, size: 14),
                    SizedBox(width: 8),
                    Text('Удалить сессию', style: TextStyle(color: Colors.redAccent)),
                  ]),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  // ==========================================
  // PROJECT FILE TREE VIEW
  // ==========================================
  Widget _buildProjectFileTree(DesktopProject proj) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Top Back to Projects header
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
          ),
          child: Row(
            children: [
              InkWell(
                onTap: () => setState(() => activeFileTreeProject = null),
                borderRadius: BorderRadius.circular(4),
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
                  child: Row(
                    children: [
                      const Icon(Icons.arrow_back, size: 13, color: Color(0xFF00D2FF)),
                      const SizedBox(width: 5),
                      Text(
                        'К проектам',
                        style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
                      ),
                    ],
                  ),
                ),
              ),
              const Spacer(),
              Icon(Icons.folder_open, size: 14, color: DesktopTheme.accentCyan),
              const SizedBox(width: 4),
              ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 100),
                child: Text(
                  proj.name,
                  style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 4),
              IconButton(
                icon: const Icon(Icons.refresh, size: 13),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                tooltip: 'Обновить дерево',
                color: DesktopTheme.textMuted,
                onPressed: () => setState(() {}),
              ),
            ],
          ),
        ),

        // File Search & Filter Bar
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurfaceElevated,
            border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
          ),
          child: Row(
            children: [
              const Icon(Icons.filter_list, size: 13, color: Color(0xFF94A3B8)),
              const SizedBox(width: 6),
              Expanded(
                child: TextField(
                  controller: fileTreeSearchCtrl,
                  style: TextStyle(fontSize: 11, color: DesktopTheme.textPrimary),
                  decoration: InputDecoration(
                    hintText: 'Фильтр файлов (*.dart, *.rs)...',
                    hintStyle: TextStyle(fontSize: 10, color: DesktopTheme.textMuted),
                    isDense: true,
                    contentPadding: const EdgeInsets.symmetric(vertical: 4),
                    border: InputBorder.none,
                  ),
                  onChanged: (val) {
                    setState(() {
                      fileTreeSearchQuery = val.trim().toLowerCase();
                    });
                  },
                ),
              ),
              if (fileTreeSearchQuery.isNotEmpty)
                IconButton(
                  icon: const Icon(Icons.close, size: 12),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 16, minHeight: 16),
                  color: DesktopTheme.textMuted,
                  onPressed: () {
                    setState(() {
                      fileTreeSearchCtrl.clear();
                      fileTreeSearchQuery = '';
                    });
                  },
                ),
            ],
          ),
        ),

        // File list with recursive folder expansion
        Expanded(
          child: ListView(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 6),
            children: _buildFileTreeNodes(proj.path, proj.path, 0),
          ),
        ),
      ],
    );
  }

  List<Widget> _buildFileTreeNodes(String rootPath, String dirPath, int depth) {
    List<universal_io.FileSystemEntity> entities = [];
    try {
      final dir = universal_io.Directory(dirPath);
      if (dir.existsSync()) {
        entities = dir.listSync().toList()
          ..sort((a, b) {
            final aIsDir = universal_io.FileSystemEntity.isDirectorySync(a.path);
            final bIsDir = universal_io.FileSystemEntity.isDirectorySync(b.path);
            if (aIsDir && !bIsDir) return -1;
            if (!aIsDir && bIsDir) return 1;
            return a.path.toLowerCase().compareTo(b.path.toLowerCase());
          });
      }
    } catch (_) {}

    if (entities.isEmpty && depth == 0) {
      return [
        Center(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Text(
              'Папка пуста или недоступна:\n$dirPath',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
            ),
          ),
        ),
      ];
    }

    final List<Widget> widgets = [];
    final hasSearch = fileTreeSearchQuery.isNotEmpty;

    for (final ent in entities) {
      final isDir = universal_io.FileSystemEntity.isDirectorySync(ent.path);
      final cleanPath = ent.path.replaceAll(r'\', '/');
      final name = cleanPath.split('/').lastWhere((s) => s.isNotEmpty, orElse: () => cleanPath);

      // Apply search filter if active
      if (hasSearch && !isDir) {
        final matchesName = name.toLowerCase().contains(fileTreeSearchQuery);
        final matchesPath = cleanPath.toLowerCase().contains(fileTreeSearchQuery);
        if (!matchesName && !matchesPath) {
          continue;
        }
      }

      final isExpanded = hasSearch || expandedProjectDirs.contains(ent.path);
      final cleanRoot = rootPath.replaceAll(r'\', '/');
      final relativePath = cleanPath.startsWith(cleanRoot)
          ? cleanPath.substring(cleanRoot.length).replaceAll(RegExp(r'^/+'), '')
          : name;

      final isUntracked = name.startsWith('.tmp') || (name.endsWith('.json') && name.contains('dump'));
      final isGitActive = name == '.codegraph' || name == '.tmp-diag' || name == 'logs';

      widgets.add(
        GestureDetector(
          onSecondaryTapDown: (details) {
            _showFileTreeContextMenu(context, details.globalPosition, ent.path, rootPath, isDir);
          },
          child: Tooltip(
            message: relativePath,
            waitDuration: const Duration(milliseconds: 600),
            child: InkWell(
              onTap: () {
              if (isDir) {
                setState(() {
                  if (expandedProjectDirs.contains(ent.path)) {
                    expandedProjectDirs.remove(ent.path);
                  } else {
                    expandedProjectDirs.add(ent.path);
                  }
                });
              } else {
                widget.onOpenFile?.call(ent.path);
              }
            },
            borderRadius: BorderRadius.circular(4),
            child: Container(
              padding: EdgeInsets.only(left: 6.0 + depth * 12.0, right: 6, top: 4, bottom: 4),
              child: Row(
                children: [
                  if (isDir)
                    Padding(
                      padding: const EdgeInsets.only(right: 3),
                      child: Icon(
                        isExpanded ? Icons.keyboard_arrow_down : Icons.chevron_right,
                        size: 13,
                        color: const Color(0xFF94A3B8),
                      ),
                    )
                  else
                    const SizedBox(width: 16),
                  _getFileIcon(name, isDir, isExpanded),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      name,
                      style: TextStyle(
                        fontSize: 11,
                        fontFamily: 'Consolas',
                        color: isDir
                            ? DesktopTheme.textPrimary
                            : (name.endsWith('.md') ? const Color(0xFF38BDF8) : DesktopTheme.textSecondary),
                        fontWeight: isDir ? FontWeight.w500 : FontWeight.normal,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  if (isGitActive)
                    Container(
                      width: 6,
                      height: 6,
                      margin: const EdgeInsets.only(left: 6),
                      decoration: const BoxDecoration(
                        color: Color(0xFF10B981),
                        shape: BoxShape.circle,
                      ),
                    ),
                  if (isUntracked)
                    Container(
                      margin: const EdgeInsets.only(left: 6),
                      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                      decoration: BoxDecoration(
                        color: const Color(0xFF10B981).withOpacity(0.15),
                        borderRadius: BorderRadius.circular(3),
                      ),
                      child: const Text(
                        'U',
                        style: TextStyle(
                          fontSize: 9,
                          fontFamily: 'Consolas',
                          fontWeight: FontWeight.bold,
                          color: Color(0xFF10B981),
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ),
        ),
      ),
    );

      if (isDir && isExpanded) {
        widgets.addAll(_buildFileTreeNodes(rootPath, ent.path, depth + 1));
      }
    }
    return widgets;
  }

  Widget _getFileIcon(String name, bool isDir, bool isExpanded) {
    if (isDir) {
      return Icon(
        isExpanded ? Icons.folder_open_outlined : Icons.folder_outlined,
        size: 13,
        color: const Color(0xFF00D2FF),
      );
    }
    final lower = name.toLowerCase();
    if (lower.endsWith('.json')) {
      return const Text(
        '{}',
        style: TextStyle(
          fontSize: 11,
          fontFamily: 'Consolas',
          fontWeight: FontWeight.bold,
          color: Color(0xFFF59E0B),
        ),
      );
    } else if (lower.endsWith('.md')) {
      return const Icon(Icons.description_outlined, size: 13, color: Color(0xFF38BDF8));
    } else if (lower.endsWith('.dart') || lower.endsWith('.rs') || lower.endsWith('.py') || lower.endsWith('.js') || lower.endsWith('.ts')) {
      return const Icon(Icons.code, size: 13, color: Color(0xFF06B6D4));
    } else if (lower.endsWith('.png') || lower.endsWith('.jpg') || lower.endsWith('.svg') || lower.endsWith('.ico')) {
      return const Icon(Icons.image_outlined, size: 13, color: Color(0xFFA855F7));
    } else {
      return const Icon(Icons.insert_drive_file_outlined, size: 13, color: Color(0xFF94A3B8));
    }
  }

  void _showFileTreeContextMenu(
    BuildContext ctx,
    Offset position,
    String fullPath,
    String rootPath,
    bool isDir,
  ) {
    final cleanFull = fullPath.replaceAll(r'\', '/');
    final cleanRoot = rootPath.replaceAll(r'\', '/');
    final relativePath = cleanFull.startsWith(cleanRoot)
        ? cleanFull.substring(cleanRoot.length).replaceAll(RegExp(r'^/+'), '')
        : cleanFull.split('/').lastWhere((s) => s.isNotEmpty, orElse: () => cleanFull);

    showMenu<String>(
      context: ctx,
      position: RelativeRect.fromLTRB(
        position.dx,
        position.dy,
        position.dx + 220,
        position.dy + 320,
      ),
      color: const Color(0xFF181C24),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: const BorderSide(color: Color(0xFF2B3240), width: 1),
      ),
      elevation: 8,
      items: isDir
          ? [
              PopupMenuItem(
                value: 'reveal_explorer',
                height: 32,
                child: Row(
                  children: const [
                    Icon(Icons.folder_outlined, size: 14, color: Color(0xFF94A3B8)),
                    SizedBox(width: 10),
                    Text('Показать в Проводнике', style: TextStyle(fontSize: 12, color: Colors.white)),
                  ],
                ),
              ),
              PopupMenuItem(
                value: 'copy_path',
                height: 32,
                child: Row(
                  children: const [
                    Icon(Icons.copy, size: 13, color: Color(0xFF94A3B8)),
                    SizedBox(width: 10),
                    Text('Копировать путь', style: TextStyle(fontSize: 12, color: Colors.white)),
                  ],
                ),
              ),
              PopupMenuItem(
                value: 'add_to_chat',
                height: 32,
                child: Row(
                  children: const [
                    Icon(Icons.chat_bubble_outline, size: 13, color: Color(0xFF94A3B8)),
                    SizedBox(width: 10),
                    Text('Добавить в контекст чата', style: TextStyle(fontSize: 12, color: Colors.white)),
                  ],
                ),
              ),
            ]
          : [
              PopupMenuItem(
                value: 'open',
                height: 32,
                child: Row(
                  children: const [
                    Icon(Icons.open_in_new, size: 13, color: Color(0xFF94A3B8)),
                    SizedBox(width: 10),
                    Text('Открыть', style: TextStyle(fontSize: 12, color: Colors.white)),
                  ],
                ),
              ),
              const PopupMenuDivider(height: 1),
              PopupMenuItem(
                value: 'open_canvas',
                height: 30,
                child: Row(
                  children: const [
                    Icon(FontAwesomeIcons.wandMagicSparkles, size: 12, color: Color(0xFF00D2FF)),
                    SizedBox(width: 10),
                    Text('Открыть в Холсте', style: TextStyle(fontSize: 11.5, color: Colors.white)),
                  ],
                ),
              ),
              PopupMenuItem(
                value: 'open_code',
                height: 30,
                child: Row(
                  children: const [
                    Icon(Icons.code, size: 13, color: Color(0xFF38BDF8)),
                    SizedBox(width: 10),
                    Text('Открыть в Редакторе кода', style: TextStyle(fontSize: 11.5, color: Colors.white)),
                  ],
                ),
              ),
              const PopupMenuDivider(height: 1),
              PopupMenuItem(
                value: 'reveal_explorer',
                height: 32,
                child: Row(
                  children: const [
                    Icon(Icons.folder_outlined, size: 14, color: Color(0xFF94A3B8)),
                    SizedBox(width: 10),
                    Text('Показать в Проводнике', style: TextStyle(fontSize: 12, color: Colors.white)),
                  ],
                ),
              ),
              PopupMenuItem(
                value: 'copy_abs_path',
                height: 32,
                child: Row(
                  children: const [
                    Icon(Icons.copy, size: 13, color: Color(0xFF94A3B8)),
                    SizedBox(width: 10),
                    Text('Копировать абсолютный путь', style: TextStyle(fontSize: 12, color: Colors.white)),
                  ],
                ),
              ),
              PopupMenuItem(
                value: 'copy_rel_path',
                height: 32,
                child: Row(
                  children: const [
                    Icon(Icons.copy_all_outlined, size: 13, color: Color(0xFF94A3B8)),
                    SizedBox(width: 10),
                    Text('Копировать относительный путь', style: TextStyle(fontSize: 12, color: Colors.white)),
                  ],
                ),
              ),
              const PopupMenuDivider(height: 1),
              PopupMenuItem(
                value: 'add_to_chat',
                height: 32,
                child: Row(
                  children: const [
                    Icon(Icons.chat_bubble_outline, size: 13, color: Color(0xFF00D2FF)),
                    SizedBox(width: 10),
                    Text('Добавить в контекст чата', style: TextStyle(fontSize: 12, color: Colors.white)),
                  ],
                ),
              ),
            ],
    ).then((val) {
      if (val == null) return;
      if (val == 'open') {
        widget.onOpenFile?.call(fullPath);
      } else if (val == 'open_canvas') {
        final ctrl = Get.isRegistered<DesktopTaskWorkspaceController>() ? Get.find<DesktopTaskWorkspaceController>() : null;
        ctrl?.openProjectFile(fullPath);
        widget.onSelectInspectorTab?.call(2); // Canvas tab
      } else if (val == 'open_code') {
        final ctrl = Get.isRegistered<DesktopTaskWorkspaceController>() ? Get.find<DesktopTaskWorkspaceController>() : null;
        ctrl?.openProjectFile(fullPath);
        widget.onSelectInspectorTab?.call(5); // File viewer tab
      } else if (val == 'reveal_explorer') {
        try {
          if (universal_io.Platform.isWindows) {
            universal_io.Process.run('explorer.exe', isDir ? [fullPath] : ['/select,', fullPath]);
          }
        } catch (_) {}
      } else if (val == 'copy_path' || val == 'copy_abs_path') {
        Clipboard.setData(ClipboardData(text: fullPath));
        Get.snackbar('Буфер обмена', 'Абсолютный путь скопирован');
      } else if (val == 'copy_rel_path') {
        Clipboard.setData(ClipboardData(text: relativePath));
        Get.snackbar('Буфер обмена', 'Относительный путь скопирован');
      } else if (val == 'add_to_chat') {
        final ctrl = Get.isRegistered<DesktopTaskWorkspaceController>() ? Get.find<DesktopTaskWorkspaceController>() : null;
        if (ctrl != null) {
          ctrl.addAttachment('@$relativePath');
          ctrl.inputController.text = '${ctrl.inputController.text.trim()} @$relativePath '.trimLeft();
          Get.snackbar('Контекст чата', 'Файл @$relativePath добавлен в поле ввода');
        }
      }
    });
  }

  // ==========================================
  // OS-AWARE FOLDER HELPERS & ADD PROJECT MODAL
  // ==========================================
  /// Opens native OS directory picker dialog (Windows PowerShell/FolderBrowserDialog, macOS osascript, Linux zenity/kdialog).
  Future<String?> _pickFolderNative({String? initialPath}) async {
    try {
      if (universal_io.Platform.isWindows) {
        final initPathClean = (initialPath != null && initialPath.trim().isNotEmpty)
            ? initialPath.trim().replaceAll("'", "''")
            : '';
        final script = '''
Add-Type -AssemblyName System.Windows.Forms
\$dlg = New-Object System.Windows.Forms.FolderBrowserDialog
\$dlg.Description = "Выберите папку для проекта"
\$dlg.ShowNewFolderButton = \$true
if ('$initPathClean' -ne '' -and (Test-Path -LiteralPath '$initPathClean')) {
    \$dlg.SelectedPath = '$initPathClean'
}
\$top = New-Object System.Windows.Forms.Form
\$top.TopMost = \$true
if (\$dlg.ShowDialog(\$top) -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output \$dlg.SelectedPath
}
\$top.Dispose()
''';
        final res = await universal_io.Process.run('powershell', ['-NoProfile', '-Command', script]);
        if (res.exitCode == 0) {
          final out = (res.stdout as String).trim();
          if (out.isNotEmpty) return out;
        }
      } else if (universal_io.Platform.isMacOS) {
        final res = await universal_io.Process.run('osascript', [
          '-e',
          'POSIX path of (choose folder with prompt "Выберите папку проекта")',
        ]);
        if (res.exitCode == 0) {
          final out = (res.stdout as String).trim();
          if (out.isNotEmpty) return out;
        }
      } else if (universal_io.Platform.isLinux) {
        try {
          final res = await universal_io.Process.run('zenity', [
            '--file-selection',
            '--directory',
            '--title=Выберите папку проекта',
          ]);
          if (res.exitCode == 0) {
            final out = (res.stdout as String).trim();
            if (out.isNotEmpty) return out;
          }
        } catch (_) {
          final res = await universal_io.Process.run('kdialog', [
            '--getexistingdirectory',
          ]);
          if (res.exitCode == 0) {
            final out = (res.stdout as String).trim();
            if (out.isNotEmpty) return out;
          }
        }
      }
    } catch (e) {
      debugPrint('Error opening native folder picker: $e');
    }
    return null;
  }

  /// Creates directory recursively on disk (OS-aware) if it does not exist.
  Future<String?> _createDirectory(String path) async {
    final clean = path.trim();
    if (clean.isEmpty) return 'Укажите путь к папке';
    try {
      final dir = universal_io.Directory(clean);
      if (!dir.existsSync()) {
        dir.createSync(recursive: true);
      }
      return null;
    } catch (e) {
      return 'Не удалось создать папку: $e';
    }
  }

  /// Checks whether directory exists on disk.
  bool _doesFolderExist(String path) {
    final clean = path.trim();
    if (clean.isEmpty) return false;
    try {
      return universal_io.Directory(clean).existsSync();
    } catch (_) {
      return false;
    }
  }

  /// Returns OS-aware default initial path.
  String _getDefaultInitialPath() {
    try {
      if (universal_io.Platform.isWindows) {
        if (universal_io.Directory(r'C:\Projects').existsSync()) {
          return r'C:\Projects\';
        }
        final userProfile = universal_io.Platform.environment['USERPROFILE'];
        if (userProfile != null && userProfile.isNotEmpty) {
          return '$userProfile\\Projects\\';
        }
        return r'C:\Projects\';
      } else {
        final home = universal_io.Platform.environment['HOME'] ?? '';
        if (home.isNotEmpty) {
          return '$home/Projects/';
        }
        return '/home/Projects/';
      }
    } catch (_) {
      return r'C:\Projects\';
    }
  }

  void _showAddProjectDialog() {
    final nameCtrl = TextEditingController();
    final pathCtrl = TextEditingController(text: _getDefaultInitialPath());

    final instructionsCtrl = TextEditingController();
    bool initGit = true;
    bool createAgentsMd = true;
    bool createReadme = true;
    bool runOb2hScan = true;
    String selectedColor = '#00D2FF';
    String selectedIcon = 'code';
    bool isScaffolding = false;

    // Initial pre-production analysis
    ProjectAnalysisResult analysis = ProjectAnalyzer.analyze(pathCtrl.text);
    if (analysis.hasGit) initGit = false;
    if (analysis.hasAgentsMd) createAgentsMd = false;

    final availableColors = [
      '#00D2FF',
      '#10B981',
      '#8B5CF6',
      '#F59E0B',
      '#EC4899',
      '#38BDF8',
    ];

    final availableIcons = [
      {'name': 'code', 'icon': Icons.code},
      {'name': 'terminal', 'icon': Icons.terminal},
      {'name': 'rocket', 'icon': Icons.rocket_launch_outlined},
      {'name': 'shield', 'icon': Icons.security_outlined},
      {'name': 'cpu', 'icon': Icons.memory_outlined},
      {'name': 'box', 'icon': Icons.inventory_2_outlined},
    ];

    IconData getStackIcon(String iconKey) {
      switch (iconKey) {
        case 'rust':
          return FontAwesomeIcons.rust;
        case 'web':
          return FontAwesomeIcons.code;
        case 'flutter':
          return Icons.flutter_dash;
        case 'python':
          return FontAwesomeIcons.python;
        case 'go':
          return FontAwesomeIcons.golang;
        case 'php':
          return FontAwesomeIcons.php;
        default:
          return Icons.folder_open_outlined;
      }
    }

    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDlgState) {
          final folderExists = _doesFolderExist(pathCtrl.text);

          return AlertDialog(
            backgroundColor: DesktopTheme.bgSurface,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
              side: BorderSide(color: DesktopTheme.borderSubtle),
            ),
            title: Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(6),
                  decoration: BoxDecoration(
                    color: const Color(0xFF00D2FF).withOpacity(0.12),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: const Icon(Icons.psychology_outlined, color: Color(0xFF00D2FF), size: 20),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Подключение и анализ проекта', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 16, fontWeight: FontWeight.bold)),
                      const SizedBox(height: 2),
                      Text('Предпроизводственный анализ, архитектурные правила и интеграция с агентом', style: TextStyle(color: DesktopTheme.textMuted, fontSize: 11)),
                    ],
                  ),
                ),
              ],
            ),
            content: SizedBox(
              width: 580,
              child: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // 1. Имя проекта
                    Text('Название проекта:', style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: DesktopTheme.textSecondary)),
                    const SizedBox(height: 6),
                    TextField(
                      controller: nameCtrl,
                      style: TextStyle(fontSize: 13, color: DesktopTheme.textPrimary),
                      decoration: InputDecoration(
                        hintText: 'Например: omnes-agent-core',
                        hintStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 12),
                        filled: true,
                        fillColor: DesktopTheme.bgSurfaceElevated,
                        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
                        border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                      ),
                    ),
                    const SizedBox(height: 12),

                    // 2. Путь на диске
                    Text('Папка на диске:', style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: DesktopTheme.textSecondary)),
                    const SizedBox(height: 6),
                    TextField(
                      controller: pathCtrl,
                      style: TextStyle(fontSize: 12, fontFamily: 'Consolas', color: DesktopTheme.textPrimary),
                      decoration: InputDecoration(
                        hintText: universal_io.Platform.isWindows ? r'C:\Projects\my-project' : '/home/user/projects/my-project',
                        hintStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 12),
                        filled: true,
                        fillColor: DesktopTheme.bgSurfaceElevated,
                        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
                        border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                        suffixIcon: Tooltip(
                          message: 'Выбрать папку через проводник',
                          child: IconButton(
                            icon: const Icon(Icons.folder_open_outlined, color: Color(0xFF00D2FF), size: 18),
                            onPressed: () async {
                              final picked = await _pickFolderNative(initialPath: pathCtrl.text);
                              if (picked != null && picked.isNotEmpty) {
                                pathCtrl.text = picked;
                                if (nameCtrl.text.trim().isEmpty) {
                                  final normalized = picked.replaceAll(r'\', '/');
                                  final segments = normalized.split('/').where((s) => s.isNotEmpty).toList();
                                  if (segments.isNotEmpty) {
                                    nameCtrl.text = segments.last;
                                  }
                                }
                                analysis = ProjectAnalyzer.analyze(picked);
                                if (analysis.hasGit) initGit = false;
                                if (analysis.hasAgentsMd) createAgentsMd = false;
                                setDlgState(() {});
                              }
                            },
                          ),
                        ),
                      ),
                      onChanged: (val) {
                        analysis = ProjectAnalyzer.analyze(val);
                        if (analysis.hasGit) initGit = false;
                        if (analysis.hasAgentsMd) createAgentsMd = false;
                        setDlgState(() {});
                      },
                    ),
                    if (pathCtrl.text.trim().isNotEmpty) ...[
                      const SizedBox(height: 5),
                      Row(
                        children: [
                          Icon(
                            folderExists ? Icons.check_circle_rounded : Icons.info_outline_rounded,
                            size: 13,
                            color: folderExists ? const Color(0xFF10B981) : const Color(0xFFF59E0B),
                          ),
                          const SizedBox(width: 6),
                          Expanded(
                            child: Text(
                              folderExists ? 'Директория обнаружена на диске' : 'Директория не существует — будет создана автоматически',
                              style: TextStyle(
                                fontSize: 11,
                                color: folderExists ? const Color(0xFF10B981) : const Color(0xFFF59E0B),
                              ),
                            ),
                          ),
                          if (!folderExists)
                            InkWell(
                              onTap: () async {
                                final err = await _createDirectory(pathCtrl.text.trim());
                                if (err == null) {
                                  analysis = ProjectAnalyzer.analyze(pathCtrl.text.trim());
                                  setDlgState(() {});
                                }
                              },
                              child: Container(
                                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                                decoration: BoxDecoration(
                                  color: const Color(0xFF00D2FF).withOpacity(0.12),
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.3)),
                                ),
                                child: const Text('Создать папку', style: TextStyle(fontSize: 10, color: Color(0xFF00D2FF))),
                              ),
                            ),
                        ],
                      ),
                    ],
                    const SizedBox(height: 14),

                    // 3. Блок предпроизводственного анализа агента
                    Container(
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgSurfaceElevated,
                        borderRadius: BorderRadius.circular(8),
                        border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.35)),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Icon(getStackIcon(analysis.iconKey), size: 15, color: const Color(0xFF00D2FF)),
                              const SizedBox(width: 8),
                              Expanded(
                                child: Text(
                                  'Предпроизводственный анализ: ${analysis.detectedStack}',
                                  style: const TextStyle(
                                    fontSize: 12,
                                    fontWeight: FontWeight.bold,
                                    color: Color(0xFF00D2FF),
                                  ),
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: 6),
                          Text(
                            analysis.summary,
                            style: TextStyle(fontSize: 11, color: DesktopTheme.textSecondary),
                          ),
                          const SizedBox(height: 10),

                          // Индикаторы состояния проекта
                          Wrap(
                            spacing: 8,
                            runSpacing: 6,
                            children: [
                              Container(
                                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                                decoration: BoxDecoration(
                                  color: (analysis.hasGit ? const Color(0xFF10B981) : const Color(0xFFF59E0B)).withOpacity(0.12),
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(color: (analysis.hasGit ? const Color(0xFF10B981) : const Color(0xFFF59E0B)).withOpacity(0.4)),
                                ),
                                child: Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    Icon(FontAwesomeIcons.codeBranch, size: 10, color: analysis.hasGit ? const Color(0xFF10B981) : const Color(0xFFF59E0B)),
                                    const SizedBox(width: 5),
                                    Text(
                                      analysis.hasGit ? 'Git: активен' : 'Git: инициализировать',
                                      style: TextStyle(fontSize: 10, color: analysis.hasGit ? const Color(0xFF10B981) : const Color(0xFFF59E0B)),
                                    ),
                                  ],
                                ),
                              ),
                              Container(
                                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                                decoration: BoxDecoration(
                                  color: (analysis.hasAgentsMd ? const Color(0xFF10B981) : const Color(0xFF38BDF8)).withOpacity(0.12),
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(color: (analysis.hasAgentsMd ? const Color(0xFF10B981) : const Color(0xFF38BDF8)).withOpacity(0.4)),
                                ),
                                child: Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    Icon(Icons.shield_outlined, size: 11, color: analysis.hasAgentsMd ? const Color(0xFF10B981) : const Color(0xFF38BDF8)),
                                    const SizedBox(width: 5),
                                    Text(
                                      analysis.hasAgentsMd ? 'AGENTS.md: настроен' : 'AGENTS.md: сгенерировать',
                                      style: TextStyle(fontSize: 10, color: analysis.hasAgentsMd ? const Color(0xFF10B981) : const Color(0xFF38BDF8)),
                                    ),
                                  ],
                                ),
                              ),
                              Container(
                                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                                decoration: BoxDecoration(
                                  color: const Color(0xFF8B5CF6).withOpacity(0.12),
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(color: const Color(0xFF8B5CF6).withOpacity(0.4)),
                                ),
                                child: Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: const [
                                    Icon(Icons.hub_outlined, size: 11, color: Color(0xFFA78BFA)),
                                    SizedBox(width: 5),
                                    Text(
                                      'OB2H Граф памяти',
                                      style: TextStyle(fontSize: 10, color: Color(0xFFA78BFA)),
                                    ),
                                  ],
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: 10),

                          // Автоматически подобранные агенты и тулзы
                          Text('Рекомендованный агентский стек:', style: TextStyle(fontSize: 10, fontWeight: FontWeight.bold, color: DesktopTheme.accentCyan)),
                          const SizedBox(height: 4),
                          Wrap(
                            spacing: 4,
                            runSpacing: 4,
                            children: [
                              ...analysis.recommendedAgents.map((a) => Chip(
                                    backgroundColor: const Color(0xFF3B82F6).withOpacity(0.18),
                                    label: Text('Агент: $a', style: const TextStyle(fontSize: 9, color: Color(0xFF60A5FA))),
                                    visualDensity: VisualDensity.compact,
                                    padding: EdgeInsets.zero,
                                  )),
                              ...analysis.recommendedTools.map((t) => Chip(
                                    backgroundColor: const Color(0xFF10B981).withOpacity(0.18),
                                    label: Text('MCP: $t', style: const TextStyle(fontSize: 9, color: Color(0xFF34D399))),
                                    visualDensity: VisualDensity.compact,
                                    padding: EdgeInsets.zero,
                                  )),
                              ...analysis.recommendedSkills.map((s) => Chip(
                                    backgroundColor: const Color(0xFF8B5CF6).withOpacity(0.18),
                                    label: Text('Скилл: $s', style: const TextStyle(fontSize: 9, color: Color(0xFFA78BFA))),
                                    visualDensity: VisualDensity.compact,
                                    padding: EdgeInsets.zero,
                                  )),
                            ],
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 14),

                    // 4. Опции автоматизации
                    Text('Опции инициализации и интеграции:', style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: DesktopTheme.textSecondary)),
                    const SizedBox(height: 6),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgSurfaceElevated,
                        borderRadius: BorderRadius.circular(8),
                        border: Border.all(color: DesktopTheme.borderSubtle),
                      ),
                      child: Column(
                        children: [
                          _buildScaffoldCheckbox(
                            title: 'Инициализировать Git репозиторий',
                            subtitle: 'Выполнит git init и создаст адаптированный .gitignore',
                            value: initGit,
                            onChanged: (v) => setDlgState(() => initGit = v ?? true),
                          ),
                          Divider(height: 8, color: DesktopTheme.borderSubtle.withOpacity(0.4)),
                          _buildScaffoldCheckbox(
                            title: 'Создать архитектурный файл AGENTS.md',
                            subtitle: 'Инструкции для агентов оркестратора, стек и правила кода',
                            value: createAgentsMd,
                            onChanged: (v) => setDlgState(() => createAgentsMd = v ?? true),
                          ),
                          Divider(height: 8, color: DesktopTheme.borderSubtle.withOpacity(0.4)),
                          _buildScaffoldCheckbox(
                            title: 'Сгенерировать README.md',
                            subtitle: 'Описание проекта, структура каталогов и команды запуска',
                            value: createReadme,
                            onChanged: (v) => setDlgState(() => createReadme = v ?? true),
                          ),
                          Divider(height: 8, color: DesktopTheme.borderSubtle.withOpacity(0.4)),
                          _buildScaffoldCheckbox(
                            title: 'Индексация в графе знаний OB2H',
                            subtitle: 'Фоновое сканирование структуры проекта в память агента',
                            value: runOb2hScan,
                            onChanged: (v) => setDlgState(() => runOb2hScan = v ?? true),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 14),

                    // 5. Персонализация (Цвет и иконка)
                    Row(
                      children: [
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text('Цвет акцента:', style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: DesktopTheme.textSecondary)),
                              const SizedBox(height: 6),
                              Row(
                                children: availableColors.map((colHex) {
                                  final colorVal = Color(int.parse(colHex.replaceFirst('#', '0xFF')));
                                  final isSel = selectedColor == colHex;
                                  return GestureDetector(
                                    onTap: () => setDlgState(() => selectedColor = colHex),
                                    child: Container(
                                      margin: const EdgeInsets.only(right: 8),
                                      width: 22,
                                      height: 22,
                                      decoration: BoxDecoration(
                                        color: colorVal,
                                        shape: BoxShape.circle,
                                        border: Border.all(
                                          color: isSel ? Colors.white : Colors.transparent,
                                          width: 2,
                                        ),
                                        boxShadow: isSel
                                            ? [BoxShadow(color: colorVal.withOpacity(0.6), blurRadius: 6)]
                                            : null,
                                      ),
                                      child: isSel ? const Icon(Icons.check, size: 12, color: Colors.black) : null,
                                    ),
                                  );
                                }).toList(),
                              ),
                            ],
                          ),
                        ),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text('Иконка проекта:', style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: DesktopTheme.textSecondary)),
                              const SizedBox(height: 6),
                              Row(
                                children: availableIcons.map((ic) {
                                  final isSel = selectedIcon == ic['name'];
                                  return GestureDetector(
                                    onTap: () => setDlgState(() => selectedIcon = ic['name'] as String),
                                    child: Container(
                                      margin: const EdgeInsets.only(right: 6),
                                      padding: const EdgeInsets.all(4),
                                      decoration: BoxDecoration(
                                        color: isSel ? const Color(0xFF00D2FF).withOpacity(0.2) : DesktopTheme.bgSurfaceElevated,
                                        borderRadius: BorderRadius.circular(4),
                                        border: Border.all(color: isSel ? const Color(0xFF00D2FF) : DesktopTheme.borderSubtle),
                                      ),
                                      child: Icon(ic['icon'] as IconData, size: 14, color: isSel ? const Color(0xFF00D2FF) : DesktopTheme.textSecondary),
                                    ),
                                  );
                                }).toList(),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 14),

                    // 6. Индивидуальные инструкции
                    Text('Индивидуальные инструкции агенту для проекта (опционально):', style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: DesktopTheme.textSecondary)),
                    const SizedBox(height: 6),
                    TextField(
                      controller: instructionsCtrl,
                      maxLines: 2,
                      style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary),
                      decoration: InputDecoration(
                        hintText: 'Например: Строго следовать TDD, архитектурный стиль Hexagonal...',
                        hintStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 11),
                        filled: true,
                        fillColor: DesktopTheme.bgSurfaceElevated,
                        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                        border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            actions: [
              TextButton(
                onPressed: isScaffolding ? null : () => Navigator.of(ctx).pop(),
                child: Text(DesktopI18n.cancel, style: TextStyle(color: DesktopTheme.textMuted)),
              ),
              ElevatedButton(
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF00D2FF),
                  foregroundColor: const Color(0xFF0F172A),
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                ),
                onPressed: isScaffolding
                    ? null
                    : () async {
                        final name = nameCtrl.text.trim();
                        final path = pathCtrl.text.trim();
                        if (name.isEmpty || path.isEmpty) {
                          Get.snackbar(
                            'Не все поля заполнены',
                            'Укажите название проекта и путь к папке',
                            backgroundColor: const Color(0xFF450A0A),
                            colorText: const Color(0xFFEF4444),
                            snackPosition: SnackPosition.BOTTOM,
                          );
                          return;
                        }

                        setDlgState(() => isScaffolding = true);

                        try {
                          final dirErr = await _createDirectory(path);
                          if (dirErr != null) {
                            setDlgState(() => isScaffolding = false);
                            Get.snackbar('Ошибка создания папки', dirErr, backgroundColor: const Color(0xFF450A0A), colorText: const Color(0xFFEF4444), snackPosition: SnackPosition.BOTTOM);
                            return;
                          }

                          // Запуск скаффолдинга и регистрации на базе предпроизводственного анализа
                          final scaffoldOptions = ProjectScaffoldOptions(
                            initGit: initGit,
                            createAgentsMd: createAgentsMd,
                            createReadme: createReadme,
                            runOb2hScan: runOb2hScan,
                            customInstructions: instructionsCtrl.text.trim().isNotEmpty ? instructionsCtrl.text.trim() : null,
                            colorHex: selectedColor,
                            iconName: selectedIcon,
                          );

                          final scaffoldRes = await ProjectScaffoldingService.scaffold(
                            projectPath: path,
                            projectName: name,
                            options: scaffoldOptions,
                          );

                          final newP = DesktopProject(
                            id: 'proj_${DateTime.now().millisecondsSinceEpoch}',
                            name: name,
                            path: path,
                            domain: scaffoldRes.analysis.domain,
                            suggestedAgents: scaffoldRes.analysis.recommendedAgents,
                            suggestedTools: scaffoldRes.analysis.recommendedTools,
                            suggestedSkills: scaffoldRes.analysis.recommendedSkills,
                            customInstructions: instructionsCtrl.text.trim().isNotEmpty ? instructionsCtrl.text.trim() : null,
                            colorHex: selectedColor,
                            iconName: selectedIcon,
                            gitInitialized: initGit || scaffoldRes.analysis.hasGit,
                            ob2hIndexed: runOb2hScan,
                          );

                          if (mounted) {
                            setState(() {
                              projects.add(newP);
                              activeFileTreeProject = newP;
                              _saveProjects();
                            });
                          }

                          if (ctx.mounted) {
                            Navigator.of(ctx).pop();
                          }

                          Get.snackbar(
                            'Проект подключен',
                            scaffoldRes.message,
                            backgroundColor: const Color(0xFF0F172A),
                            colorText: const Color(0xFF00D2FF),
                            icon: const Icon(Icons.check_circle_outline, color: Color(0xFF00D2FF)),
                            snackPosition: SnackPosition.BOTTOM,
                            duration: const Duration(seconds: 4),
                          );
                        } catch (e) {
                          setDlgState(() => isScaffolding = false);
                          Get.snackbar('Ошибка', 'Не удалось завершить создание проекта: $e', backgroundColor: const Color(0xFF450A0A), colorText: const Color(0xFFEF4444), snackPosition: SnackPosition.BOTTOM);
                        }
                      },
                child: isScaffolding
                    ? Row(
                        mainAxisSize: MainAxisSize.min,
                        children: const [
                          SizedBox(width: 14, height: 14, child: CircularProgressIndicator(strokeWidth: 2, color: Color(0xFF0F172A))),
                          SizedBox(width: 8),
                          Text('Анализ и настройка...', style: TextStyle(fontWeight: FontWeight.bold)),
                        ],
                      )
                    : const Text('Подключить к агенту', style: TextStyle(fontWeight: FontWeight.bold)),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildScaffoldCheckbox({
    required String title,
    required String subtitle,
    required bool value,
    required ValueChanged<bool?> onChanged,
  }) {
    return InkWell(
      onTap: () => onChanged(!value),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Row(
          children: [
            SizedBox(
              width: 20,
              height: 20,
              child: Checkbox(
                value: value,
                onChanged: onChanged,
                activeColor: const Color(0xFF00D2FF),
                checkColor: const Color(0xFF0F172A),
                materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(title, style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: DesktopTheme.textPrimary)),
                  Text(subtitle, style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted)),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ==========================================
  // ADD GROUP MODAL
  // ==========================================
  void _showAddGroupDialog() {
    final ctrl = TextEditingController();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: Text('Новая группа', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 16)),
        content: TextField(
          controller: ctrl,
          style: TextStyle(fontSize: 13, color: DesktopTheme.textPrimary),
          decoration: InputDecoration(
            hintText: 'Название группы (например: Исследования)',
            hintStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 12),
            filled: true,
            fillColor: DesktopTheme.bgSurfaceElevated,
            border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
          ),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.of(ctx).pop(), child: Text(DesktopI18n.cancel, style: TextStyle(color: DesktopTheme.textMuted))),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF00D2FF), foregroundColor: const Color(0xFF0F172A)),
            onPressed: () {
              final name = ctrl.text.trim();
              if (name.isNotEmpty) {
                setState(() {
                  groups.add(DesktopGroup(
                    id: 'grp_${DateTime.now().millisecondsSinceEpoch}',
                    name: name,
                    icon: FontAwesomeIcons.folder,
                  ));
                });
              }
              Navigator.of(ctx).pop();
            },
            child: const Text('Добавить', style: TextStyle(fontWeight: FontWeight.bold)),
          ),
        ],
      ),
    );
  }

  // ==========================================
  // CHANGE GROUP ICON DIALOG (20 ICONS PICKER)
  // ==========================================
  void _showChangeGroupIconDialog(DesktopGroup group) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: Text('Выберите иконку для "${group.name}"', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: SizedBox(
          width: 320,
          child: GridView.builder(
            shrinkWrap: true,
            gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: 5,
              crossAxisSpacing: 10,
              mainAxisSpacing: 10,
            ),
            itemCount: groupIconsCatalog.length,
            itemBuilder: (ctx, idx) {
              final item = groupIconsCatalog[idx];
              final ic = item['icon'] as IconData;
              final isSelected = group.icon == ic;

              return InkWell(
                onTap: () {
                  setState(() => group.icon = ic);
                  Navigator.of(ctx).pop();
                },
                borderRadius: BorderRadius.circular(8),
                child: Container(
                  decoration: BoxDecoration(
                    color: isSelected ? const Color(0xFF00D2FF).withOpacity(0.2) : DesktopTheme.bgSurfaceElevated,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: isSelected ? const Color(0xFF00D2FF) : DesktopTheme.borderSubtle),
                  ),
                  child: Center(
                    child: Icon(ic, size: 16, color: isSelected ? const Color(0xFF00D2FF) : DesktopTheme.textSecondary),
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );
  }

  // ==========================================
  // AI GROUPING TRIGGER
  // ==========================================
  void _showAiGroupingDialog() {
    Get.snackbar(
      '✨ ИИ Группировка',
      'Агент анализирует несгруппированные сессии и распределяет их по смысловым категориям...',
      backgroundColor: Colors.black87,
      colorText: Colors.white,
      duration: const Duration(seconds: 3),
      snackPosition: SnackPosition.BOTTOM,
    );
  }

  // ==========================================
  // CONTEXT MENUS (GROUP & SESSION)
  // ==========================================
  void _showGroupContextMenu(BuildContext ctx, Offset pos, DesktopGroup group) {
    showMenu<String>(
      context: ctx,
      position: RelativeRect.fromLTRB(pos.dx, pos.dy, pos.dx + 1, pos.dy + 1),
      color: DesktopTheme.bgSurfaceElevated,
      items: [
        PopupMenuItem(value: 'icon', child: Row(children: [const Icon(Icons.palette_outlined, size: 14), const SizedBox(width: 8), const Text('Сменить иконку (20 иконок)')])),
        PopupMenuItem(value: 'rename', child: Row(children: [const Icon(Icons.edit_outlined, size: 14), const SizedBox(width: 8), const Text('Переименовать')])),
        if (group.name != 'Разное')
          PopupMenuItem(value: 'delete', child: Row(children: [const Icon(Icons.delete_outline, color: Colors.redAccent, size: 14), const SizedBox(width: 8), const Text('Удалить группу', style: TextStyle(color: Colors.redAccent))])),
      ],
    ).then((val) {
      if (val == 'icon') {
        _showChangeGroupIconDialog(group);
                        _saveGroups();
      } else if (val == 'delete') {
        setState(() { groups.remove(group); _saveGroups(); });
      }
    });
  }

  void _showSessionContextMenu(BuildContext ctx, Offset pos, TaskSession session) {
    showMenu<String>(
      context: ctx,
      position: RelativeRect.fromLTRB(pos.dx, pos.dy, pos.dx + 1, pos.dy + 1),
      color: DesktopTheme.bgSurfaceElevated,
      items: [
        PopupMenuItem(value: 'rename', child: Row(children: [const Icon(Icons.edit_outlined, size: 14), const SizedBox(width: 8), const Text('Переименовать')])),
        PopupMenuItem(value: 'clear', child: Row(children: [const Icon(Icons.cleaning_services_outlined, size: 14), const SizedBox(width: 8), const Text('Очистить историю')])),
        PopupMenuItem(value: 'delete', child: Row(children: [const Icon(Icons.delete_outline, color: Colors.redAccent, size: 14), const SizedBox(width: 8), const Text('Удалить сессию', style: TextStyle(color: Colors.redAccent))])),
      ],
    ).then((val) {
      final ctrl = Get.find<DesktopTaskWorkspaceController>();
      if (val == 'rename') {
        _showRenameSessionDialog(session);
      } else if (val == 'clear') {
        ctrl.clearSessionHistory(session.id);
      } else if (val == 'delete') {
        _showDeleteSessionDialog(session);
      }
    });
  }

  void _showDeleteSessionDialog(TaskSession session) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        title: Text('Удалить сессию?', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: Text(
          'Вы уверены, что хотите удалить сессию «${session.title}»?',
          style: TextStyle(color: DesktopTheme.textSecondary, fontSize: 13),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text(DesktopI18n.cancel),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.redAccent),
            onPressed: () {
              Navigator.of(ctx).pop();
              final ctrl = Get.find<DesktopTaskWorkspaceController>();
              ctrl.deleteSession(session.id);
            },
            child: const Text('Удалить', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }

  void _showRenameSessionDialog(TaskSession session) {
    final ctrl = TextEditingController(text: session.title);
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        title: Text('Переименовать сессию', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: TextField(controller: ctrl, style: TextStyle(color: DesktopTheme.textPrimary)),
        actions: [
          TextButton(onPressed: () => Navigator.of(ctx).pop(), child: Text(DesktopI18n.cancel)),
          ElevatedButton(
            onPressed: () {
              if (ctrl.text.trim().isNotEmpty) {
                Get.find<DesktopTaskWorkspaceController>().renameSession(session.id, ctrl.text.trim());
              }
              Navigator.of(ctx).pop();
            },
            child: const Text('Сохранить'),
          ),
        ],
      ),
    );
  }

  List<Widget> _buildTimelineItems(List<TaskSession> all) {
    return all.map((session) {
      return _buildSessionRow(session, session.project ?? session.group);
    }).toList();
  }

  Widget _buildFilterPopupMenu() {
    return PopupMenuButton<String>(
      tooltip: DesktopI18n.tr('Вид и сортировка', 'View and sort'),
      offset: const Offset(0, 26),
      color: DesktopTheme.bgSurface,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
        side: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
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
        PopupMenuItem(value: 'by_project', child: Text(DesktopI18n.tr('Иерархия', 'Hierarchy'))),
        PopupMenuItem(value: 'timeline', child: Text(DesktopI18n.tr('Хронология', 'Timeline'))),
      ],
      child: Container(
        padding: const EdgeInsets.all(4),
        child: Icon(Icons.filter_list, size: 14, color: DesktopTheme.textMuted),
      ),
    );
  }

  void _showMoveSessionToGroupDialog(TaskSession session) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: Text('Переместить в группу', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: SizedBox(
          width: 280,
          child: ListView(
            shrinkWrap: true,
            children: groups.map((g) {
              return ListTile(
                dense: true,
                leading: Icon(g.icon, size: 14, color: const Color(0xFF00D2FF)),
                title: Text(g.name, style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 13)),
                onTap: () {
                  final ctrl = Get.find<DesktopTaskWorkspaceController>();
                  ctrl.moveSessionToGroup(session.id, g.name);
                  Navigator.of(ctx).pop();
                },
              );
            }).toList(),
          ),
        ),
      ),
    );
  }

  void _showBindSessionToProjectDialog(TaskSession session) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: Text('Привязать к проекту', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: SizedBox(
          width: 320,
          child: ListView(
            shrinkWrap: true,
            children: projects.map((p) {
              return ListTile(
                dense: true,
                leading: const Icon(Icons.folder_outlined, size: 14, color: Color(0xFF00D2FF)),
                title: Text(p.name, style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 13, fontWeight: FontWeight.bold)),
                subtitle: Text(p.path, style: TextStyle(color: DesktopTheme.textMuted, fontSize: 11, fontFamily: 'Consolas')),
                onTap: () {
                  final ctrl = Get.find<DesktopTaskWorkspaceController>();
                  ctrl.bindSessionToProject(session.id, p.name, p.path);
                  Navigator.of(ctx).pop();
                },
              );
            }).toList(),
          ),
        ),
      ),
    );
  }

  void _showRenameGroupDialog(DesktopGroup group) {
    final ctrl = TextEditingController(text: group.name);
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        title: Text('Переименовать группу', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: TextField(controller: ctrl, style: TextStyle(color: DesktopTheme.textPrimary)),
        actions: [
          TextButton(onPressed: () => Navigator.of(ctx).pop(), child: Text(DesktopI18n.cancel)),
          ElevatedButton(
            onPressed: () {
              final newName = ctrl.text.trim();
              if (newName.isNotEmpty) {
                setState(() => group.name = newName);
              }
              Navigator.of(ctx).pop();
            },
            child: const Text('Сохранить'),
          ),
        ],
      ),
    );
  }

  void _showGroupArtifactsDialog(DesktopGroup group) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        title: Text('Артефакты группы "${group.name}"', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Директория артефактов:', style: TextStyle(color: DesktopTheme.textSecondary, fontSize: 12)),
            const SizedBox(height: 6),
            Text('~/.omnesagent/data/artifacts/${group.id}', style: const TextStyle(fontFamily: 'Consolas', fontSize: 12, color: Color(0xFF00D2FF))),
          ],
        ),
        actions: [
          ElevatedButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Закрыть'),
          ),
        ],
      ),
    );
  }

  void _showRenameProjectDialog(DesktopProject proj) {
    final ctrl = TextEditingController(text: proj.name);
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        title: Text('Переименовать проект', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
        content: TextField(controller: ctrl, style: TextStyle(color: DesktopTheme.textPrimary)),
        actions: [
          TextButton(onPressed: () => Navigator.of(ctx).pop(), child: Text(DesktopI18n.cancel)),
          ElevatedButton(
            onPressed: () {
              final newName = ctrl.text.trim();
              if (newName.isNotEmpty) {
                setState(() {
                  proj.name = newName;
                  _saveProjects();
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

  void _showChangeProjectPathDialog(DesktopProject proj) {
    final ctrl = TextEditingController(text: proj.path);
    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDlgState) {
          final exists = _doesFolderExist(ctrl.text);
          return AlertDialog(
            backgroundColor: DesktopTheme.bgSurface,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(12),
              side: BorderSide(color: DesktopTheme.borderSubtle),
            ),
            title: Row(
              children: [
                const Icon(Icons.edit_location_alt_outlined, color: Color(0xFF00D2FF), size: 18),
                const SizedBox(width: 8),
                Text('Сменить путь к проекту', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 16)),
              ],
            ),
            content: SizedBox(
              width: 480,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextField(
                    controller: ctrl,
                    style: TextStyle(color: DesktopTheme.textPrimary, fontFamily: 'Consolas', fontSize: 13),
                    decoration: InputDecoration(
                      filled: true,
                      fillColor: DesktopTheme.bgSurfaceElevated,
                      contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                      border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                      suffixIcon: Tooltip(
                        message: 'Выбрать папку через проводник',
                        child: IconButton(
                          icon: const Icon(Icons.folder_open_outlined, color: Color(0xFF00D2FF), size: 18),
                          onPressed: () async {
                            final picked = await _pickFolderNative(initialPath: ctrl.text);
                            if (picked != null && picked.isNotEmpty) {
                              ctrl.text = picked;
                              setDlgState(() {});
                            }
                          },
                        ),
                      ),
                    ),
                    onChanged: (_) => setDlgState(() {}),
                  ),
                  if (ctrl.text.trim().isNotEmpty) ...[
                    const SizedBox(height: 8),
                    Row(
                      children: [
                        Icon(
                          exists ? Icons.check_circle_rounded : Icons.info_outline_rounded,
                          size: 14,
                          color: exists ? const Color(0xFF10B981) : const Color(0xFFF59E0B),
                        ),
                        const SizedBox(width: 6),
                        Expanded(
                          child: Text(
                            exists ? 'Папка найдена на диске' : 'Папка не существует на диске',
                            style: TextStyle(
                              fontSize: 11,
                              color: exists ? const Color(0xFF10B981) : const Color(0xFFF59E0B),
                            ),
                          ),
                        ),
                        if (!exists)
                          InkWell(
                            onTap: () async {
                              final err = await _createDirectory(ctrl.text.trim());
                              if (err == null) {
                                setDlgState(() {});
                                Get.snackbar(
                                  'Папка создана',
                                  'Папка успешно создана на диске',
                                  backgroundColor: const Color(0xFF0F172A),
                                  colorText: const Color(0xFF00D2FF),
                                  snackPosition: SnackPosition.BOTTOM,
                                );
                              } else {
                                Get.snackbar('Ошибка создания папки', err,
                                    backgroundColor: const Color(0xFF450A0A),
                                    colorText: const Color(0xFFEF4444),
                                    snackPosition: SnackPosition.BOTTOM);
                              }
                            },
                            child: Container(
                              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                              decoration: BoxDecoration(
                                color: const Color(0xFF00D2FF).withOpacity(0.15),
                                borderRadius: BorderRadius.circular(4),
                                border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.4)),
                              ),
                              child: Row(
                                mainAxisSize: MainAxisSize.min,
                                children: const [
                                  Icon(Icons.create_new_folder_outlined, size: 12, color: Color(0xFF00D2FF)),
                                  SizedBox(width: 4),
                                  Text(
                                    'Создать папку',
                                    style: TextStyle(fontSize: 11, color: Color(0xFF00D2FF), fontWeight: FontWeight.w600),
                                  ),
                                ],
                              ),
                            ),
                          ),
                      ],
                    ),
                  ],
                ],
              ),
            ),
            actions: [
              TextButton(onPressed: () => Navigator.of(ctx).pop(), child: Text(DesktopI18n.cancel, style: TextStyle(color: DesktopTheme.textMuted))),
              ElevatedButton(
                style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF00D2FF), foregroundColor: const Color(0xFF0F172A)),
                onPressed: () async {
                  final newPath = ctrl.text.trim();
                  if (newPath.isNotEmpty) {
                    if (!_doesFolderExist(newPath)) {
                      final confirm = await showDialog<bool>(
                        context: ctx,
                        builder: (c) => AlertDialog(
                          backgroundColor: DesktopTheme.bgSurface,
                          shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.circular(12),
                            side: BorderSide(color: DesktopTheme.borderSubtle),
                          ),
                          title: Row(
                            children: [
                              const Icon(Icons.create_new_folder_outlined, color: Color(0xFF00D2FF), size: 18),
                              const SizedBox(width: 8),
                              Text('Создать папку?', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 15)),
                            ],
                          ),
                          content: Text('Папка «$newPath» не существует.\nСоздать её автоматически?', style: TextStyle(color: DesktopTheme.textSecondary, fontSize: 13)),
                          actions: [
                            TextButton(onPressed: () => Navigator.of(c).pop(false), child: Text(DesktopI18n.cancel)),
                            ElevatedButton(
                              style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF00D2FF), foregroundColor: const Color(0xFF0F172A)),
                              onPressed: () => Navigator.of(c).pop(true),
                              child: const Text('Создать и продолжить', style: TextStyle(fontWeight: FontWeight.bold)),
                            ),
                          ],
                        ),
                      );
                      if (confirm != true) return;
                      final err = await _createDirectory(newPath);
                      if (err != null) {
                        Get.snackbar('Ошибка', err,
                            backgroundColor: const Color(0xFF450A0A),
                            colorText: const Color(0xFFEF4444),
                            snackPosition: SnackPosition.BOTTOM);
                        return;
                      }
                    }
                    if (mounted) {
                      setState(() {
                        proj.path = newPath;
                        _saveProjects();
                      });
                    }
                    if (ctx.mounted) {
                      Navigator.of(ctx).pop();
                    }
                  }
                },
                child: const Text('Сохранить', style: TextStyle(fontWeight: FontWeight.bold)),
              ),
            ],
          );
        },
      ),
    );
  }

}
