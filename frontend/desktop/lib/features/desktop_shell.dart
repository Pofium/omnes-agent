// Top-Level OmnesAgent ADE Desktop Shell: Windows layout matching Screenshot 2.
// Comprises Left Sidebar (Projects/Groups), Central Task Workspace, Right Tool Canvas,
// Modal Settings Dialog, and User Onboarding/Profile Dialog.

import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:omnes_shared/omnes_shared.dart';
import 'package:window_manager/window_manager.dart';

import '../theme/desktop_theme.dart';
import '../utils/desktop_app_lifecycle.dart';
import '../utils/desktop_tray_manager.dart';
import '../widgets/desktop_sidebar.dart';
import 'automations/automations_view.dart';
import 'command_palette/command_palette_dialog.dart';
import 'inspector/inspector_panel.dart';
import 'onboarding/user_onboarding_dialog.dart';
import 'settings/desktop_settings_dialog.dart';
import 'workspace/task_workspace_controller.dart';
import 'workspace/task_workspace_view.dart';

class DesktopShell extends StatefulWidget {
  const DesktopShell({super.key});

  @override
  State<DesktopShell> createState() => _DesktopShellState();
}

class _DesktopShellState extends State<DesktopShell> with WindowListener {
  final workspaceController = Get.put(DesktopTaskWorkspaceController());

  // User Profile state (Screenshot 2: Ilya Presnyakov, Lite)
  UserProfileData userProfile = UserProfileData(
    firstName: 'Илья',
    lastName: 'Пресняков',
    role: 'Tech Lead / AI Engineer',
    primaryStack: 'Rust / Dart / Python',
    autonomyStyle: 'Full access (максимальная автономность)',
    language: 'Русский',
    enableAstMemory: true,
  );

  int selectedNavIndex = 0;
  bool isSidebarVisible = true;
  double sidebarWidth = 260.0;
  double inspectorWidth = 420.0;
  bool isInspectorOpen = false; // Closed by default
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
    windowManager.addListener(this);
    _initTray();
    _checkInitialState();
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    super.dispose();
  }

  /// Handles close button click on the window.
  /// If minimize_to_tray is enabled, hides to tray; otherwise, cleanly quits.
  @override
  void onWindowClose() async {
    final minimizeToTray = GetStorage().read<bool>('minimize_to_tray') ?? true;
    if (minimizeToTray) {
      await windowManager.hide();
    } else {
      await DesktopAppLifecycle.quitApp();
    }
  }

  /// Initializes system tray icon and attaches callbacks matching the OA mini menu.
  void _initTray() {
    DesktopTrayManager.instance.init(
      onOpenOA: () async {
        await windowManager.show();
        await windowManager.focus();
      },
      onNewTask: () async {
        await windowManager.show();
        await windowManager.focus();
        _onNewTask();
      },
      onOpenWorkspace: () async {
        await windowManager.show();
        await windowManager.focus();
        _openCommandPalette();
      },
      onCheckUpdates: () async {
        await windowManager.show();
        await windowManager.focus();
        _showCheckUpdatesDialog();
      },
      onAboutOA: () async {
        await windowManager.show();
        await windowManager.focus();
        _showAboutOADialog();
      },
      onClearData: () async {
        await windowManager.show();
        await windowManager.focus();
        _showClearDataDialog();
      },
      onQuit: () async {
        await DesktopAppLifecycle.quitApp();
      },
    );
  }

  void _showCheckUpdatesDialog() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: Row(
          children: [
            const Icon(Icons.system_update_alt, color: Color(0xFF00D2FF), size: 20),
            const SizedBox(width: 8),
            Text('Обновление OmnesAgent', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 16)),
          ],
        ),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Версия: 1.0.0 (Latest)', style: TextStyle(color: DesktopTheme.textPrimary, fontWeight: FontWeight.bold, fontSize: 13)),
            const SizedBox(height: 8),
            Text('У вас установлена последняя стабильная версия OmnesAgent (OA). Новых обновлений не обнаружено.', style: TextStyle(color: DesktopTheme.textMuted, fontSize: 12)),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('OK', style: TextStyle(color: Color(0xFF00D2FF))),
          ),
        ],
      ),
    );
  }

  void _showAboutOADialog() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: Row(
          children: [
            Container(
              width: 28,
              height: 28,
              decoration: BoxDecoration(
                gradient: const LinearGradient(colors: [Color(0xFF00D2FF), Color(0xFF0055FF)]),
                borderRadius: BorderRadius.circular(6),
              ),
              alignment: Alignment.center,
              child: const Text('OA', style: TextStyle(color: Colors.white, fontWeight: FontWeight.w900, fontSize: 13)),
            ),
            const SizedBox(width: 10),
            Text('О программе OmnesAgent (OA)', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 16)),
          ],
        ),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('OmnesAgent Workstation v1.0.0', style: TextStyle(color: DesktopTheme.textPrimary, fontWeight: FontWeight.bold, fontSize: 14)),
            const SizedBox(height: 6),
            Text('Автономное инженерное рабочее место с поддержкой Rust Gateway Runtime, детерминированного AST графа и умной маршрутизации LLM.', style: TextStyle(color: DesktopTheme.textMuted, fontSize: 12)),
            const SizedBox(height: 12),
            Text('Локальный шлюз: 127.0.0.1:42617', style: TextStyle(color: DesktopTheme.textSecondary, fontFamily: 'Consolas', fontSize: 11)),
            Text('Репозиторий: github.com/Pofium/omnes-agent', style: TextStyle(color: DesktopTheme.textMuted, fontFamily: 'Consolas', fontSize: 11)),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Закрыть', style: TextStyle(color: Color(0xFF00D2FF))),
          ),
        ],
      ),
    );
  }

  void _showClearDataDialog() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: DesktopTheme.bgSurface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: DesktopTheme.borderSubtle),
        ),
        title: Row(
          children: [
            const Icon(Icons.delete_forever, color: Colors.redAccent, size: 20),
            const SizedBox(width: 8),
            Text('Очистить все данные?', style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 16)),
          ],
        ),
        content: Text(
          'Это действие сбросит локальный кэш, историю интерфейса и сохранённые временные параметры в GetStorage. Продолжить?',
          style: TextStyle(color: DesktopTheme.textMuted, fontSize: 12),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: Text('Отмена', style: TextStyle(color: DesktopTheme.textMuted)),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.redAccent),
            onPressed: () async {
              Navigator.of(ctx).pop();
              await GetStorage().erase();
              if (mounted) {
                Get.snackbar('Очистка данных', 'Все локальные данные успешно очищены',
                    backgroundColor: Colors.black87, colorText: Colors.white);
              }
            },
            child: const Text('Очистить', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
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
      inspectorTabIndex = index;
      sideChatInitialText = sideChatText;
      inspectorKey = UniqueKey();
    });
  }

  void _toggleInspector() {
    setState(() {
      isInspectorOpen = !isInspectorOpen;
      if (isInspectorOpen) {
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

  @override
  Widget build(BuildContext context) {
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
            _openInspectorWithTab(1); // Terminal tab
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
      child: Obx(() => Scaffold(
        backgroundColor: DesktopTheme.bgCanvas,
        body: SafeArea(
          child: LayoutBuilder(
            builder: (context, constraints) {
              final totalWidth = constraints.maxWidth;
              const minCenterWidth = 845.0;

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
                        final currentRight = isInspectorOpen ? inspectorWidth : 0.0;
                        final maxAllowedSidebar = (totalWidth - currentRight - minCenterWidth).clamp(200.0, 440.0);
                        setState(() {
                          sidebarWidth = (sidebarWidth + dx).clamp(200.0, maxAllowedSidebar >= 200.0 ? maxAllowedSidebar : 200.0);
                        });
                      },
                    ),
                  ],

                  // 2. Central Task Canvas & Composer (Min Width: 845px)
                  Expanded(
                    child: ConstrainedBox(
                      constraints: const BoxConstraints(minWidth: minCenterWidth),
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
                              onReviewChanges: () => _openInspectorWithTab(3),
                              onOpenSettings: () => _openSettingsDialog('Провайдеры'),
                            ),
                    ),
                  ),

                  // 3. Right Tool Canvas / Inspector
                  if (isInspectorOpen) ...[
                    _buildVerticalResizer(
                      onDrag: (dx) {
                        final currentLeft = isSidebarVisible ? sidebarWidth : 0.0;
                        final maxAllowedInspector = (totalWidth - currentLeft - minCenterWidth).clamp(280.0, 650.0);
                        setState(() {
                          inspectorWidth = (inspectorWidth - dx).clamp(280.0, maxAllowedInspector >= 280.0 ? maxAllowedInspector : 280.0);
                        });
                      },
                    ),
                    DesktopInspectorPanel(
                      key: inspectorKey,
                      width: inspectorWidth,
                      controller: workspaceController,
                      initialTabIndex: inspectorTabIndex,
                      initialSideChatText: sideChatInitialText,
                      onClose: () => setState(() => isInspectorOpen = false),
                    ),
                  ],
                ],
              );
            },
          ),
        ),
      )),
    );
  }
}
