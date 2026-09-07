// Top-Level OmnesAgent ADE Desktop Shell: Windows layout matching Screenshot 2.
// Comprises Left Sidebar (Projects/Groups), Central Task Workspace, Right Tool Canvas,
// Modal Settings Dialog, and User Onboarding/Profile Dialog.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';

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

class _DesktopShellState extends State<DesktopShell> {
  final workspaceController = Get.put(DesktopTaskWorkspaceController());

  // User Profile state (Screenshot 2: Ilya Presnyakov, Lite)
  UserProfileData userProfile = UserProfileData(
    firstName: 'Илья',
    lastName: 'Пресняков',
    tier: 'Lite',
    role: 'Tech Lead / AI Engineer',
    primaryStack: 'Rust / Dart / Python',
    autonomyStyle: 'Full access (максимальная автономность)',
    language: 'Русский',
    enableAstMemory: true,
  );

  int selectedNavIndex = 0;
  bool isSidebarVisible = true;
  bool isInspectorOpen = false; // Closed by default
  int inspectorTabIndex = -1; // -1 opens the 'Open tab' chooser from Screenshot 2
  Key inspectorKey = UniqueKey();

  void _openInspectorWithTab(int index) {
    setState(() {
      isInspectorOpen = true;
      inspectorTabIndex = index;
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
                color: const Color(0xFF131518),
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: const Color(0xFF262A33), width: 1.2),
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
    workspaceController.messages.clear();
    workspaceController.activeTaskTitle.value = 'New Task';
    workspaceController.inputController.clear();
    setState(() => selectedNavIndex = -1);
  }

  @override
  Widget build(BuildContext context) {
    return RawKeyboardListener(
      focusNode: FocusNode(),
      autofocus: true,
      onKey: (event) {
        if (event is RawKeyDownEvent && event.isControlPressed) {
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
      child: Scaffold(
        backgroundColor: const Color(0xFF141619),
        body: SafeArea(
          child: Row(
            children: [
              // 1. Left Sidebar (Projects / Groups)
              if (isSidebarVisible)
                DesktopSidebar(
                  selectedIndex: selectedNavIndex,
                  userProfile: userProfile,
                  onSelectTask: (idx, title) {
                    setState(() => selectedNavIndex = idx);
                    workspaceController.activeTaskTitle.value = title;
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
                ),

              // 2. Central Task Canvas & Composer or Automations Screen (Screenshot 1 Match)
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
                        onOpenSettings: () => _openSettingsDialog('Провайдеры'),
                      ),
              ),

              // 3. Right Tool Canvas / Inspector (Open tab: Side conversation, Review, Terminal, Browser)
              if (isInspectorOpen)
                DesktopInspectorPanel(
                  key: inspectorKey,
                  controller: workspaceController,
                  initialTabIndex: inspectorTabIndex,
                  onClose: () => setState(() => isInspectorOpen = false),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
