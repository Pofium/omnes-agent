// Top-Level 3-Pane Desktop Shell with Titlebar, Sidebar, Workspace Canvas, and Inspector.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';

import '../theme/desktop_theme.dart';
import '../widgets/desktop_sidebar.dart';
import '../widgets/desktop_titlebar.dart';
import 'command_palette/command_palette_dialog.dart';
import 'inspector/inspector_panel.dart';
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
  bool isInspectorOpen = true;
  int inspectorTabIndex = 0;
  Key inspectorKey = UniqueKey();

  void _openInspectorWithTab(int index) {
    setState(() {
      isInspectorOpen = true;
      inspectorTabIndex = index;
      inspectorKey = UniqueKey();
    });
  }

  void _openCommandPalette() {
    DesktopCommandPaletteDialog.show(
      context,
      controller: workspaceController,
      onToggleInspector: () {
        setState(() => isInspectorOpen = !isInspectorOpen);
      },
      onSelectInspectorTab: (tabIdx) {
        _openInspectorWithTab(tabIdx);
      },
    );
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
            setState(() => isInspectorOpen = !isInspectorOpen);
          }
        }
      },
      child: Scaffold(
        backgroundColor: DesktopTheme.bgCanvas,
        body: SafeArea(
          child: Column(
            children: [
              // 1. Frameless Custom Window Titlebar
              DesktopTitleBar(
                onOpenCommandPalette: _openCommandPalette,
                onToggleInspector: () {
                  setState(() => isInspectorOpen = !isInspectorOpen);
                },
                isInspectorOpen: isInspectorOpen,
              ),

              // 2. Main 3-Pane Multi-Column Workstation Layout
              Expanded(
                child: Row(
                  children: [
                    // Pane 1: Collapsible Left Sidebar (280px / 60px)
                    DesktopSidebar(
                      selectedIndex: selectedNavIndex,
                      onSelectIndex: (idx) {
                        setState(() => selectedNavIndex = idx);
                      },
                    ),

                    // Pane 2: Central Task Canvas & Chat (Flex)
                    Expanded(
                      child: DesktopTaskWorkspaceView(
                        controller: workspaceController,
                      ),
                    ),

                    // Pane 3: Right Inspector Panel (460px)
                    if (isInspectorOpen)
                      DesktopInspectorPanel(
                        key: inspectorKey,
                        controller: workspaceController,
                        initialTabIndex: inspectorTabIndex,
                      ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
