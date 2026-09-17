// Top-Level OmnesAgent ADE Desktop Shell: Windows layout matching Screenshot 2.
// Comprises Left Sidebar (Projects/Groups), Central Task Workspace, Right Tool Canvas,
// Modal Settings Dialog, and User Onboarding/Profile Dialog.

import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:omnes_shared/omnes_shared.dart';

import '../theme/desktop_theme.dart';
import '../widgets/desktop_sidebar.dart';
import 'automations/automations_view.dart';
import 'command_palette/command_palette_dialog.dart';
import 'inspector/inspector_panel.dart';
import 'settings/desktop_settings_dialog.dart';
import 'workspace/task_workspace_controller.dart';
import 'workspace/task_workspace_view.dart';

class DesktopShell extends StatefulWidget {
  const DesktopShell({super.key});

  @override
  State<DesktopShell> createState() => _DesktopShellState();
}

class _DesktopShellState extends State<DesktopShell> {
  final workspaceController = Get.put(DesktopTaskWorkspaceController());

  // User Profile state
  UserProfileData userProfile = UserProfileData();

  int selectedNavIndex = 0;
  bool isSidebarVisible = true;
  double sidebarWidth = 260.0;
  double centerWidth = 856.0;
  double inspectorWidth = 420.0;
  bool isInspectorOpen = false; // Closed by default
  bool isCenterCollapsed = false; // When true, inspector occupies the entire middle area
  static const double targetCenterWidth = 856.0;
  int inspectorTabIndex = -1; // -1 opens the 'Open tab' chooser from Screenshot 2
  String? sideChatInitialText;
  Key inspectorKey = UniqueKey();

  @override
  void initState() {
    super.initState();
    try {
      final saved = GetStorage().read<Map<String, dynamic>>('user_profile');
      if (saved != null) {
        userProfile = UserProfileData.fromJson(saved);
      }
    } catch (_) {}
    _checkInitialState();
  }

  Future<void> _checkInitialState() async {
    try {
      final http = GatewayHttpClient();
      final qs = await http.getQuickstartState();
      final memList = await http.memoryList(category: 'core');
      final profileMem = memList.firstWhereOrNull((m) => m['key'] == 'user_profile');
      if (profileMem != null && profileMem['content'] != null) {
        final decoded = jsonDecode(profileMem['content'].toString());
        if (decoded is Map<String, dynamic>) {
          final fullName = (decoded['name'] ?? '').toString();
          final parts = fullName.split(' ');
          if (mounted) {
            setState(() {
              userProfile = UserProfileData(
                firstName: parts.isNotEmpty && parts[0].isNotEmpty ? parts[0] : userProfile.firstName,
                lastName: parts.length > 1 ? parts.sublist(1).join(' ') : userProfile.lastName,
                role: decoded['role']?.toString() ?? userProfile.role,
                primaryStack: decoded['stack']?.toString() ?? userProfile.primaryStack,
                autonomyStyle: decoded['autonomy']?.toString() ?? userProfile.autonomyStyle,
                language: decoded['language']?.toString() ?? userProfile.language,
                enableAstMemory: decoded['ast_memory'] == true,
              );
            });
          }
        }
      } else if (qs != null && qs['ready'] == false) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted) _openOnboardingDialog();
        });
      }
    } catch (_) {}
  }

  void _openInspectorWithTab(int index, {String? sideChatText}) {
    setState(() {
      isInspectorOpen = true;
      isCenterCollapsed = false;
      centerWidth = targetCenterWidth;
      inspectorTabIndex = index;
      sideChatInitialText = sideChatText;
      inspectorKey = UniqueKey();
    });
  }

  void _toggleInspector() {
    setState(() {
      isInspectorOpen = !isInspectorOpen;
      if (isInspectorOpen) {
        isCenterCollapsed = false;
        centerWidth = targetCenterWidth;
        inspectorTabIndex = -1; // Show Open Tab Chooser
        inspectorKey = UniqueKey();
      }
    });
  }

  void _openCommandPalette() {
    DesktopCommandPaletteDialog.show(
      context,
      controller: workspaceController,
      onToggleInspector: _toggleInspector,
      onSelectInspectorTab: (tabIdx) {
        _openInspectorWithTab(tabIdx);
      },
    );
  }

  /// Opens the Settings modal dialog over the workspace.
  void _openSettingsDialog([String initialSection = 'Общие']) {
    showDialog(
      context: context,
      barrierColor: Colors.black.withOpacity(0.7),
      barrierDismissible: true,
      builder: (ctx) {
        return Dialog(
          backgroundColor: Colors.transparent,
          insetPadding: const EdgeInsets.symmetric(horizontal: 40, vertical: 30),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(14),
            child: Container(
              constraints: const BoxConstraints(maxWidth: 1060, maxHeight: 720),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurface,
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: DesktopTheme.borderSubtle, width: 1.2),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withOpacity(0.65),
                    blurRadius: 36,
                    spreadRadius: 6,
                    offset: const Offset(0, 10),
                  ),
                ],
              ),
              child: DesktopSettingsDialog(
                initialSection: initialSection,
                userProfile: userProfile,
                onUpdateProfile: (updated) {
                  setState(() => userProfile = updated);
                },
                onBackToWorkspace: () => Navigator.of(ctx).pop(),
              ),
            ),
          ),
        );
      },
    );
  }

  /// Opens the Registration / Onboarding questionnaire dialog.
  void _openOnboardingDialog() {
    UserOnboardingDialog.show(
      context,
      initialProfile: userProfile,
      onSave: (updated) {
        setState(() => userProfile = updated);
      },
    );
  }

  void _onNewTask() {
    workspaceController.createNewTask();
    setState(() => selectedNavIndex = -1);
  }

  int mobileNavIndex = 0; // 0: Chat/Workspace, 1: Inspector, 2: Terminal, 3: Automations, 4: Settings

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final width = constraints.maxWidth;
        final isTablet = width >= 768 && width < 1280;
        final isMobile = width < 768;

        return KeyboardListener(
          focusNode: FocusNode(),
          autofocus: true,
          onKeyEvent: (event) {
            if (event is KeyDownEvent && HardwareKeyboard.instance.isControlPressed) {
              // Global Ctrl + K: Command Palette
              if (event.logicalKey == LogicalKeyboardKey.keyK) {
                _openCommandPalette();
              }
              // Global Ctrl + J: Open Terminal in Inspector
              else if (event.logicalKey == LogicalKeyboardKey.keyJ) {
                if (isMobile) {
                  setState(() => mobileNavIndex = 2);
                } else {
                  _openInspectorWithTab(1); // Terminal tab
                }
              }
              // Global Ctrl + B: Toggle Inspector Panel
              else if (event.logicalKey == LogicalKeyboardKey.keyB) {
                _toggleInspector();
              }
              // Global Ctrl + N: New Task
              else if (event.logicalKey == LogicalKeyboardKey.keyN) {
                _onNewTask();
              }
              // Global Ctrl + ,: Open Settings
              else if (event.logicalKey == LogicalKeyboardKey.comma) {
                _openSettingsDialog();
              }
            }
          },
          child: Obx(() {
            if (isMobile) {
              return _buildMobileScaffold();
            } else if (isTablet) {
              return _buildTabletScaffold();
            } else {
              return _buildDesktopScaffold();
            }
          }),
        );
      },
    );
  }

  /// 1. Mobile Layout (< 768px): One view at a time with BottomNavigationBar & Drawer
  Widget _buildMobileScaffold() {
    Widget currentBody;
    switch (mobileNavIndex) {
      case 1:
        // Inspector (Browser / Tools)
        currentBody = DesktopInspectorPanel(
          key: inspectorKey,
          controller: workspaceController,
          initialTabIndex: inspectorTabIndex == -1 ? 0 : inspectorTabIndex,
          onClose: () => setState(() => mobileNavIndex = 0),
        );
        break;
      case 2:
        // Terminal in Inspector
        currentBody = DesktopInspectorPanel(
          key: inspectorKey,
          controller: workspaceController,
          initialTabIndex: 1, // Terminal tab
          onClose: () => setState(() => mobileNavIndex = 0),
        );
        break;
      case 3:
        // Automations
        currentBody = AutomationsView(
          onBackToWorkspace: () => setState(() => mobileNavIndex = 0),
        );
        break;
      case 4:
        // Settings embedded view
        currentBody = Container(
          color: DesktopTheme.bgSurface,
          child: DesktopSettingsDialog(
            initialSection: 'Общие',
            userProfile: userProfile,
            onUpdateProfile: (updated) => setState(() => userProfile = updated),
            onBackToWorkspace: () => setState(() => mobileNavIndex = 0),
          ),
        );
        break;
      case 0:
      default:
        currentBody = DesktopTaskWorkspaceView(
          controller: workspaceController,
          isToolsOpen: isInspectorOpen,
          onToggleTools: () => setState(() => mobileNavIndex = 1),
          onToggleTerminal: () => setState(() => mobileNavIndex = 2),
          onOpenSettings: () => setState(() => mobileNavIndex = 4),
        );
        break;
    }

    return Scaffold(
      backgroundColor: DesktopTheme.bgCanvas,
      appBar: AppBar(
        backgroundColor: DesktopTheme.bgSidebar,
        elevation: 0,
        titleSpacing: 0,
        leading: Builder(
          builder: (ctx) => IconButton(
            icon: const Icon(Icons.menu, size: 20, color: Color(0xFF00D2FF)),
            onPressed: () => Scaffold.of(ctx).openDrawer(),
          ),
        ),
        title: Row(
          children: [
            Text(
              'OmnesAgent',
              style: TextStyle(
                color: DesktopTheme.textPrimary,
                fontWeight: FontWeight.bold,
                fontSize: 14,
              ),
            ),
            const SizedBox(width: 6),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
              decoration: BoxDecoration(
                color: DesktopTheme.accentSky.withOpacity(0.12),
                borderRadius: BorderRadius.circular(3),
              ),
              child: const Text(
                'WEB',
                style: TextStyle(
                  color: DesktopTheme.accentSky,
                  fontSize: 9,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            tooltip: 'Палитра команд (Ctrl+K)',
            icon: const Icon(Icons.search, size: 20, color: Color(0xFF94A3B8)),
            onPressed: _openCommandPalette,
          ),
          IconButton(
            tooltip: 'Новая задача',
            icon: const Icon(Icons.add, size: 20, color: Color(0xFF00D2FF)),
            onPressed: _onNewTask,
          ),
        ],
      ),
      drawer: Drawer(
        width: 280,
        backgroundColor: DesktopTheme.bgSidebar,
        child: SafeArea(
          child: DesktopSidebar(
            selectedIndex: selectedNavIndex,
            userProfile: userProfile,
            onSelectTask: (id, title, project) {
              setState(() {
                selectedNavIndex = 0;
                mobileNavIndex = 0;
              });
              Navigator.of(context).pop(); // close drawer
              workspaceController.switchToSession(id, title: title, project: project);
            },
            onNewTask: () {
              Navigator.of(context).pop();
              _onNewTask();
            },
            onOpenSearch: () {
              Navigator.of(context).pop();
              _openCommandPalette();
            },
            onOpenSettings: () {
              Navigator.of(context).pop();
              setState(() => mobileNavIndex = 4);
            },
            onOpenSkills: () {
              Navigator.of(context).pop();
              _openSettingsDialog('Навыки');
            },
            onOpenAutomations: () {
              Navigator.of(context).pop();
              setState(() => mobileNavIndex = 3);
            },
            onOpenProfile: () {
              Navigator.of(context).pop();
              _openOnboardingDialog();
            },
            onToggleSidebar: () => Navigator.of(context).pop(),
          ),
        ),
      ),
      body: SafeArea(child: currentBody),
      bottomNavigationBar: BottomNavigationBar(
        backgroundColor: DesktopTheme.bgSidebar,
        selectedItemColor: const Color(0xFF00D2FF),
        unselectedItemColor: const Color(0xFF64748B),
        currentIndex: mobileNavIndex,
        type: BottomNavigationBarType.fixed,
        selectedFontSize: 11,
        unselectedFontSize: 11,
        onTap: (idx) {
          setState(() => mobileNavIndex = idx);
        },
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.chat_bubble_outline, size: 18),
            activeIcon: Icon(Icons.chat_bubble, size: 18),
            label: 'Чат',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.travel_explore, size: 18),
            activeIcon: Icon(Icons.travel_explore, size: 18),
            label: 'Инспектор',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.terminal, size: 18),
            activeIcon: Icon(Icons.terminal, size: 18),
            label: 'Терминал',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.auto_mode, size: 18),
            activeIcon: Icon(Icons.auto_mode, size: 18),
            label: 'Авто',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.settings_outlined, size: 18),
            activeIcon: Icon(Icons.settings, size: 18),
            label: 'Настройки',
          ),
        ],
      ),
    );
  }

  /// 2. Tablet Layout (768px – 1279px): Two-panel layout with adaptive sidebar/inspector
  Widget _buildTabletScaffold() {
    return Scaffold(
      backgroundColor: DesktopTheme.bgCanvas,
      body: SafeArea(
        child: Row(
          children: [
            // If inspector is closed, sidebar is visible by default. If inspector opens, sidebar auto-collapses unless forced
            if (isSidebarVisible && !isInspectorOpen)
              DesktopSidebar(
                selectedIndex: selectedNavIndex,
                userProfile: userProfile,
                onSelectTask: (id, title, project) {
                  setState(() => selectedNavIndex = 0);
                  workspaceController.switchToSession(id, title: title, project: project);
                },
                onNewTask: _onNewTask,
                onOpenSearch: _openCommandPalette,
                onOpenSettings: () => _openSettingsDialog('Общие'),
                onOpenSkills: () => _openSettingsDialog('Навыки'),
                onOpenAutomations: () {
                  setState(() => selectedNavIndex = -3);
                },
                onOpenProfile: _openOnboardingDialog,
                onToggleSidebar: () {
                  setState(() => isSidebarVisible = !isSidebarVisible);
                },
                onOpenFile: (filePath) {
                  workspaceController.openProjectFile(filePath);
                  if (filePath.toLowerCase().endsWith('.md')) {
                    _openInspectorWithTab(2); // Canvas tab
                  } else {
                    _openInspectorWithTab(5); // File Viewer tab
                  }
                },
                onSelectInspectorTab: (tabIdx) => _openInspectorWithTab(tabIdx),
              ),

            // Central Area
            Expanded(
              child: selectedNavIndex == -3
                  ? AutomationsView(
                      onBackToWorkspace: () => setState(() => selectedNavIndex = 0),
                    )
                  : DesktopTaskWorkspaceView(
                      controller: workspaceController,
                      isToolsOpen: isInspectorOpen,
                      onToggleTools: _toggleInspector,
                      onToggleTerminal: () => _openInspectorWithTab(1),
                      onOpenCanvas: () => _openInspectorWithTab(2),
                      onBranchSideChat: (text) => _openInspectorWithTab(4, sideChatText: text),
                      onOpenSettings: () => _openSettingsDialog('Провайдеры'),
                    ),
            ),

            // Inspector Panel
            if (isInspectorOpen)
              SizedBox(
                width: 440,
                child: DesktopInspectorPanel(
                  key: inspectorKey,
                  controller: workspaceController,
                  initialTabIndex: inspectorTabIndex,
                  initialSideChatText: sideChatInitialText,
                  onClose: () => setState(() => isInspectorOpen = false),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildVerticalResizer({required Function(double delta) onDrag}) {
    return MouseRegion(
      cursor: SystemMouseCursors.resizeColumn,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onHorizontalDragUpdate: (details) => onDrag(details.delta.dx),
        child: Container(
          width: 5,
          color: Colors.transparent,
          alignment: Alignment.center,
          child: Container(
            width: 1,
            color: DesktopTheme.borderSubtle,
          ),
        ),
      ),
    );
  }

  /// 3. Desktop Layout (≥ 1280px): Full 3-panel ADE (Sidebar + Composer/Workspace + Inspector)
  Widget _buildDesktopScaffold() {
    return Scaffold(
      backgroundColor: DesktopTheme.bgCanvas,
      body: SafeArea(
        child: LayoutBuilder(
          builder: (context, constraints) {
            final totalWidth = constraints.maxWidth;
            const minCenterWidth = targetCenterWidth;
            final currentLeft = isSidebarVisible ? sidebarWidth : 0.0;
            final availableSpace = totalWidth - currentLeft;

            return Row(
              children: [
                // 1. Left Sidebar (Projects / Groups)
                if (isSidebarVisible) ...[
                  DesktopSidebar(
                    width: sidebarWidth,
                    selectedIndex: selectedNavIndex,
                    userProfile: userProfile,
                    onSelectTask: (id, title, project) {
                      setState(() => selectedNavIndex = 0);
                      workspaceController.switchToSession(id, title: title, project: project);
                    },
                    onNewTask: _onNewTask,
                    onOpenSearch: _openCommandPalette,
                    onOpenSettings: () => _openSettingsDialog('Общие'),
                    onOpenSkills: () => _openSettingsDialog('Навыки'),
                    onOpenAutomations: () {
                      setState(() => selectedNavIndex = -3);
                    },
                    onOpenProfile: _openOnboardingDialog,
                    onToggleSidebar: () {
                      setState(() => isSidebarVisible = !isSidebarVisible);
                    },
                    onOpenFile: (filePath) {
                      workspaceController.openProjectFile(filePath);
                      if (filePath.toLowerCase().endsWith('.md')) {
                        _openInspectorWithTab(2); // Canvas tab
                      } else {
                        _openInspectorWithTab(5); // File Viewer tab
                      }
                    },
                    onSelectInspectorTab: (tabIdx) => _openInspectorWithTab(tabIdx),
                  ),
                  _buildVerticalResizer(
                    onDrag: (dx) {
                      final maxAllowedSidebar = isCenterCollapsed
                          ? (totalWidth - 320.0).clamp(200.0, 440.0)
                          : (totalWidth - (isInspectorOpen ? 280.0 : 0.0) - minCenterWidth).clamp(200.0, 440.0);
                      setState(() {
                        sidebarWidth = (sidebarWidth + dx).clamp(200.0, maxAllowedSidebar >= 200.0 ? maxAllowedSidebar : 200.0);
                      });
                    },
                  ),
                ],

                // 2. Central Task Canvas & Composer (Width: 856px when inspector is open)
                if (!isCenterCollapsed)
                  isInspectorOpen
                      ? SizedBox(
                          width: centerWidth.clamp(320.0, (availableSpace - 280.0).clamp(320.0, double.infinity)),
                          child: selectedNavIndex == -3
                              ? AutomationsView(
                                  onBackToWorkspace: () => setState(() => selectedNavIndex = 0),
                                )
                              : DesktopTaskWorkspaceView(
                                  controller: workspaceController,
                                  isToolsOpen: isInspectorOpen,
                                  isSidebarVisible: isSidebarVisible,
                                  onToggleSidebar: () => setState(() => isSidebarVisible = !isSidebarVisible),
                                  onToggleTools: _toggleInspector,
                                  onToggleTerminal: () => _openInspectorWithTab(1),
                                  onOpenCanvas: () => _openInspectorWithTab(2),
                                  onBranchSideChat: (text) => _openInspectorWithTab(4, sideChatText: text),
                                  onReviewChanges: () => _openInspectorWithTab(3),
                                  onOpenSettings: () => _openSettingsDialog('Провайдеры'),
                                ),
                        )
                      : Expanded(
                          child: ConstrainedBox(
                            constraints: const BoxConstraints(minWidth: minCenterWidth),
                            child: selectedNavIndex == -3
                                ? AutomationsView(
                                    onBackToWorkspace: () => setState(() => selectedNavIndex = 0),
                                  )
                                : DesktopTaskWorkspaceView(
                                    controller: workspaceController,
                                    isToolsOpen: isInspectorOpen,
                                    isSidebarVisible: isSidebarVisible,
                                    onToggleSidebar: () => setState(() => isSidebarVisible = !isSidebarVisible),
                                    onToggleTools: _toggleInspector,
                                    onToggleTerminal: () => _openInspectorWithTab(1),
                                    onOpenCanvas: () => _openInspectorWithTab(2),
                                    onBranchSideChat: (text) => _openInspectorWithTab(4, sideChatText: text),
                                    onReviewChanges: () => _openInspectorWithTab(3),
                                    onOpenSettings: () => _openSettingsDialog('Провайдеры'),
                                  ),
                          ),
                        ),

                // 3. Right Tool Canvas / Inspector
                if (isInspectorOpen) ...[
                  if (!isCenterCollapsed)
                    _buildVerticalResizer(
                      onDrag: (dx) {
                        final newCenterWidth = centerWidth + dx;
                        // If dragging inspector to the left reduces center width below 856px,
                        // inspector snaps to occupy the entire middle area.
                        if (newCenterWidth < targetCenterWidth) {
                          setState(() {
                            isCenterCollapsed = true;
                            centerWidth = targetCenterWidth;
                          });
                        } else {
                          setState(() {
                            centerWidth = newCenterWidth.clamp(targetCenterWidth, availableSpace - 280.0);
                          });
                        }
                      },
                    ),
                  Expanded(
                    child: Column(
                      children: [
                        if (isCenterCollapsed)
                          _buildCenterPanelRestoreBar(),
                        Expanded(
                          child: DesktopInspectorPanel(
                            key: inspectorKey,
                            width: inspectorWidth,
                            controller: workspaceController,
                            initialTabIndex: inspectorTabIndex,
                            initialSideChatText: sideChatInitialText,
                            onClose: () => setState(() {
                              isInspectorOpen = false;
                              isCenterCollapsed = false;
                            }),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ],
            );
          },
        ),
      ),
    );
  }

  /// Compact header displayed at the top of the expanded inspector when the center
  /// workspace panel is collapsed. Allows the user to restore the center panel to 856px.
  Widget _buildCenterPanelRestoreBar() {
    return Container(
      height: 38,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurfaceElevated,
        border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1.0)),
      ),
      child: Row(
        children: [
          InkWell(
            onTap: () {
              setState(() {
                isCenterCollapsed = false;
                centerWidth = targetCenterWidth;
              });
            },
            borderRadius: BorderRadius.circular(6),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                color: const Color(0xFF00D2FF).withOpacity(0.12),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.5), width: 1),
              ),
              child: const Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.view_sidebar_outlined, size: 15, color: Color(0xFF00D2FF)),
                  SizedBox(width: 8),
                  Text(
                    'Открыть рабочую область (856 px)',
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: Color(0xFF00D2FF),
                    ),
                  ),
                ],
              ),
            ),
          ),
          const Spacer(),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: DesktopTheme.borderSubtle, width: 0.8),
            ),
            child: Text(
              'Инспектор на всю ширину',
              style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
            ),
          ),
        ],
      ),
    );
  }
}
