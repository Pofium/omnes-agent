// Top-Level ZCode ADE Desktop Shell: Minimalist Layout matching official ZCode Desktop screenshots.
// Comprises Left Sidebar, Central Task Workspace Canvas, Collapsible Right Tool Canvas,
// and Modal Settings Popup.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';

import '../widgets/desktop_sidebar.dart';
import 'command_palette/command_palette_dialog.dart';
import 'inspector/inspector_panel.dart';
import 'settings/zcode_settings_view.dart';
import 'workspace/task_workspace_controller.dart';
import 'workspace/task_workspace_view.dart';

class DesktopShell extends StatefulWidget {
  const DesktopShell({super.key});

  @override
  State<DesktopShell> createState() => _DesktopShellState();
}

class _DesktopShellState extends State<DesktopShell> {
  final workspaceController = Get.put(DesktopTaskWorkspaceController());

  int selectedNavIndex = 0;
  bool isSidebarVisible = true;
  bool isInspectorOpen = false; // Closed by default, exactly like ZCode!
  int inspectorTabIndex = 0;
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

  /// Opens the ZCode Settings screen as a modal popup dialog over the workspace.
  void _openSettingsDialog([String initialSection = 'General']) {
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
              child: ZCodeSettingsView(
                initialSection: initialSection,
                onBackToWorkspace: () => Navigator.of(ctx).pop(),
              ),
            ),
          ),
        );
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
              // 1. Left Sidebar (ZCode ADE Left Panel)
              if (isSidebarVisible)
                DesktopSidebar(
                  selectedIndex: selectedNavIndex,
                  onSelectIndex: (idx) {
                    setState(() => selectedNavIndex = idx);
                    // Load sample task or reset
                    if (idx >= 0) {
                      workspaceController.activeTaskTitle.value = 'Task #$idx';
                    }
                  },
                  onNewTask: _onNewTask,
                  onOpenSearch: _openCommandPalette,
                  onOpenSettings: () => _openSettingsDialog('General'),
                  onOpenSkills: () => _openSettingsDialog('Skills'),
                  onToggleSidebar: () {
                    setState(() => isSidebarVisible = !isSidebarVisible);
                  },
                ),

              // 2. Central Task Canvas & Composer (Full Width by Default!)
              Expanded(
                child: DesktopTaskWorkspaceView(
                  controller: workspaceController,
                  isToolsOpen: isInspectorOpen,
                  onToggleTools: _toggleInspector,
                  onToggleTerminal: () => _openInspectorWithTab(1),
                  onOpenSettings: () => _openSettingsDialog('Model Settings'),
                ),
              ),

              // 3. Right Tool Canvas / Inspector (Browser, Terminal, Preview, Side Chat)
              // Hidden by default, toggled smoothly via top right buttons or Ctrl+B / Ctrl+J
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
