// Command Palette Modal Dialog (Ctrl + K) with OmnesAgent ADE Shortcuts and Actions.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';
import '../../theme/desktop_theme.dart';
import '../workspace/task_workspace_controller.dart';

class DesktopCommandPaletteDialog extends StatefulWidget {
  final DesktopTaskWorkspaceController? controller;
  final VoidCallback? onToggleInspector;
  final Function(int tabIndex)? onSelectInspectorTab;

  const DesktopCommandPaletteDialog({
    super.key,
    this.controller,
    this.onToggleInspector,
    this.onSelectInspectorTab,
  });

  static Future<void> show(
    BuildContext context, {
    DesktopTaskWorkspaceController? controller,
    VoidCallback? onToggleInspector,
    Function(int tabIndex)? onSelectInspectorTab,
  }) {
    return showDialog(
      context: context,
      barrierColor: Colors.black.withOpacity(0.6),
      builder: (_) => DesktopCommandPaletteDialog(
        controller: controller,
        onToggleInspector: onToggleInspector,
        onSelectInspectorTab: onSelectInspectorTab,
      ),
    );
  }

  @override
  State<DesktopCommandPaletteDialog> createState() =>
      _DesktopCommandPaletteDialogState();
}

class _DesktopCommandPaletteDialogState
    extends State<DesktopCommandPaletteDialog> {
  final searchController = TextEditingController();
  String query = '';

  List<_CommandAction> get allActions => [
        _CommandAction(
          category: 'GOAL & TASKS',
          title: 'Set Goal Mode (/goal)',
          shortcut: '/goal',
          icon: FontAwesomeIcons.bullseye,
          onExecute: () {
            if (widget.controller != null) {
              widget.controller!.inputController.text = '/goal ';
            }
          },
        ),
        _CommandAction(
          category: 'GOAL & TASKS',
          title: 'Create New Task Run',
          shortcut: 'Ctrl + N',
          icon: FontAwesomeIcons.plus,
          onExecute: () {
            widget.controller?.messages.clear();
            widget.controller?.activeTaskTitle.value = 'New Task Session';
          },
        ),
        _CommandAction(
          category: 'GOAL & TASKS',
          title: 'Clear Current Workspace Session',
          shortcut: 'Ctrl + L',
          icon: FontAwesomeIcons.trashCan,
          onExecute: () {
            widget.controller?.messages.clear();
          },
        ),
        _CommandAction(
          category: 'PERMISSION MODE',
          title: 'Cycle Permission Mode (Ask -> Edit -> Plan -> Full)',
          shortcut: 'Shift + Tab',
          icon: Icons.shield_outlined,
          onExecute: () {
            widget.controller?.cyclePermissionMode();
          },
        ),
        _CommandAction(
          category: 'PERMISSION MODE',
          title: 'Set Mode: Ask before changes (Supervised)',
          shortcut: '',
          icon: Icons.lock_clock,
          onExecute: () {
            widget.controller?.setPermissionMode(PermissionMode.askBeforeChanges);
          },
        ),
        _CommandAction(
          category: 'PERMISSION MODE',
          title: 'Set Mode: Edit automatically',
          shortcut: '',
          icon: Icons.edit_note,
          onExecute: () {
            widget.controller?.setPermissionMode(PermissionMode.editAutomatically);
          },
        ),
        _CommandAction(
          category: 'PERMISSION MODE',
          title: 'Set Mode: Plan mode (Architecture First)',
          shortcut: '',
          icon: Icons.architecture,
          onExecute: () {
            widget.controller?.setPermissionMode(PermissionMode.planMode);
          },
        ),
        _CommandAction(
          category: 'PERMISSION MODE',
          title: 'Set Mode: Full access (Unrestricted)',
          shortcut: '',
          icon: Icons.bolt,
          onExecute: () {
            widget.controller?.setPermissionMode(PermissionMode.fullAccess);
          },
        ),
        _CommandAction(
          category: 'TOOLS',
          title: 'Open Integrated Terminal',
          shortcut: 'Ctrl + J',
          icon: FontAwesomeIcons.terminal,
          onExecute: () {
            widget.onSelectInspectorTab?.call(1);
          },
        ),
        _CommandAction(
          category: 'TOOLS',
          title: 'Open Live Browser & Element Picker',
          shortcut: '',
          icon: FontAwesomeIcons.globe,
          onExecute: () {
            widget.onSelectInspectorTab?.call(0);
          },
        ),
        _CommandAction(
          category: 'TOOLS',
          title: 'Toggle Tool Canvas Panel',
          shortcut: 'Ctrl + B',
          icon: Icons.view_sidebar_outlined,
          onExecute: () {
            widget.onToggleInspector?.call();
          },
        ),
        _CommandAction(
          category: 'APPEARANCE',
          title: 'Toggle Dark / Light Theme',
          shortcut: 'Ctrl + T',
          icon: Icons.brightness_6_outlined,
          onExecute: () {
            DesktopThemeController.to.toggleTheme();
          },
        ),
        _CommandAction(
          category: 'MODELS',
          title: 'Switch to GLM-5.3 ADE Agent',
          shortcut: 'Alt + 1',
          icon: FontAwesomeIcons.brain,
          onExecute: () {
            widget.controller?.setModel('GLM-5.3');
          },
        ),
        _CommandAction(
          category: 'MODELS',
          title: 'Switch to Claude 3.5 Sonnet',
          shortcut: 'Alt + 2',
          icon: FontAwesomeIcons.brain,
          onExecute: () {
            widget.controller?.setModel('Claude 3.5 Sonnet');
          },
        ),
        _CommandAction(
          category: 'MODELS',
          title: 'Switch to DeepSeek V3 / Reasoner',
          shortcut: 'Alt + 3',
          icon: FontAwesomeIcons.brain,
          onExecute: () {
            widget.controller?.setModel('DeepSeek V3');
          },
        ),
        _CommandAction(
          category: 'SYSTEM',
          title: 'Run Diagnostic Doctor & Gateway Health',
          shortcut: 'F5',
          icon: FontAwesomeIcons.stethoscope,
          onExecute: () {},
        ),
      ];

  @override
  Widget build(BuildContext context) {
    final filtered = allActions.where((a) {
      if (query.isEmpty) return true;
      return a.title.toLowerCase().contains(query.toLowerCase()) ||
          a.category.toLowerCase().contains(query.toLowerCase());
    }).toList();

    return Center(
      child: Material(
        color: Colors.transparent,
        child: Container(
          width: 600,
          constraints: const BoxConstraints(maxHeight: 480),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            borderRadius: BorderRadius.circular(10),
            border: Border.all(color: DesktopTheme.borderMedium, width: 1.2),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withOpacity(0.5),
                blurRadius: 32,
                offset: const Offset(0, 12),
              ),
            ],
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Search input
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                child: Row(
                  children: [
                    const Icon(Icons.search, size: 20, color: DesktopTheme.accentSky),
                    const SizedBox(width: 12),
                    Expanded(
                      child: TextField(
                        controller: searchController,
                        autofocus: true,
                        style: TextStyle(
                          fontSize: 14,
                          color: DesktopTheme.textPrimary,
                        ),
                        decoration: InputDecoration(
                          hintText: 'Type a command, tool or search actions...',
                          hintStyle: TextStyle(
                            color: DesktopTheme.textMuted,
                            fontSize: 13,
                          ),
                          filled: false,
                          border: InputBorder.none,
                          enabledBorder: InputBorder.none,
                          focusedBorder: InputBorder.none,
                          contentPadding: EdgeInsets.zero,
                        ),
                        onChanged: (val) => setState(() => query = val),
                      ),
                    ),
                    IconButton(
                      icon: Icon(Icons.close, size: 18, color: DesktopTheme.textMuted),
                      onPressed: () => Get.back(),
                    ),
                  ],
                ),
              ),
              Divider(height: 1, color: DesktopTheme.borderSubtle),

              // Results List
              Flexible(
                child: ListView.separated(
                  shrinkWrap: true,
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  itemCount: filtered.length,
                  separatorBuilder: (_, __) => const SizedBox(height: 2),
                  itemBuilder: (context, index) {
                    final item = filtered[index];
                    return InkWell(
                      onTap: () {
                        Get.back();
                        item.onExecute?.call();
                      },
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 9),
                        child: Row(
                          children: [
                            Icon(item.icon, size: 14, color: DesktopTheme.textSecondary),
                            const SizedBox(width: 12),
                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Text(
                                    item.title,
                                    style: TextStyle(
                                      fontSize: 13,
                                      fontWeight: FontWeight.w500,
                                      color: DesktopTheme.textPrimary,
                                    ),
                                  ),
                                  Text(
                                    item.category,
                                    style: TextStyle(
                                      fontSize: 10,
                                      color: DesktopTheme.textMuted,
                                      letterSpacing: 0.5,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                            if (item.shortcut.isNotEmpty)
                              Container(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 6,
                                  vertical: 2,
                                ),
                                decoration: BoxDecoration(
                                  color: DesktopTheme.bgSurfaceElevated,
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(color: DesktopTheme.borderSubtle),
                                ),
                                child: Text(
                                  item.shortcut,
                                  style: TextStyle(
                                    fontSize: 10,
                                    fontFamily: 'Consolas',
                                    color: DesktopTheme.textSecondary,
                                  ),
                                ),
                              ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _CommandAction {
  final String category;
  final String title;
  final String shortcut;
  final IconData icon;
  final VoidCallback? onExecute;

  const _CommandAction({
    required this.category,
    required this.title,
    required this.shortcut,
    required this.icon,
    this.onExecute,
  });
}
