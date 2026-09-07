// Command Palette Modal Dialog (Ctrl + K).

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import '../../theme/desktop_theme.dart';

class DesktopCommandPaletteDialog extends StatefulWidget {
  const DesktopCommandPaletteDialog({super.key});

  static Future<void> show(BuildContext context) {
    return showDialog(
      context: context,
      barrierColor: Colors.black.withOpacity(0.6),
      builder: (_) => const DesktopCommandPaletteDialog(),
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

  final List<_CommandAction> allActions = const [
    _CommandAction(
      category: 'TASKS',
      title: 'Create New Task Run',
      shortcut: 'Ctrl + N',
      icon: FontAwesomeIcons.plus,
    ),
    _CommandAction(
      category: 'TASKS',
      title: 'Clear Current Workspace Session',
      shortcut: 'Ctrl + L',
      icon: FontAwesomeIcons.trashCan,
    ),
    _CommandAction(
      category: 'MODELS',
      title: 'Switch to Claude 3.5 Sonnet',
      shortcut: 'Alt + 1',
      icon: FontAwesomeIcons.brain,
    ),
    _CommandAction(
      category: 'MODELS',
      title: 'Switch to DeepSeek V3 / Reasoner',
      shortcut: 'Alt + 2',
      icon: FontAwesomeIcons.brain,
    ),
    _CommandAction(
      category: 'MODELS',
      title: 'Switch to Local Ollama Llama 3.2',
      shortcut: 'Alt + 3',
      icon: FontAwesomeIcons.server,
    ),
    _CommandAction(
      category: 'SYSTEM',
      title: 'Run Doctor System Diagnostics',
      shortcut: 'F5',
      icon: FontAwesomeIcons.stethoscope,
    ),
    _CommandAction(
      category: 'SYSTEM',
      title: 'Toggle Supervised / Autonomous Mode',
      shortcut: 'Ctrl + M',
      icon: Icons.security,
    ),
    _CommandAction(
      category: 'SETTINGS',
      title: 'Security & PIN Code Settings',
      shortcut: '',
      icon: Icons.lock_outline,
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
          width: 580,
          constraints: const BoxConstraints(maxHeight: 460),
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
                        style: const TextStyle(
                          fontSize: 15,
                          color: DesktopTheme.textPrimary,
                        ),
                        decoration: const InputDecoration(
                          hintText: 'Type a command or search action...',
                          hintStyle: TextStyle(
                            color: DesktopTheme.textMuted,
                            fontSize: 14,
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
                      icon: const Icon(Icons.close, size: 18, color: DesktopTheme.textMuted),
                      onPressed: () => Get.back(),
                    ),
                  ],
                ),
              ),
              const Divider(height: 1, color: DesktopTheme.borderSubtle),

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
                      },
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                        child: Row(
                          children: [
                            Icon(item.icon, size: 14, color: DesktopTheme.textSecondary),
                            const SizedBox(width: 12),
                            Expanded(
                              child: Text(
                                item.title,
                                style: const TextStyle(
                                  fontSize: 13,
                                  color: DesktopTheme.textPrimary,
                                ),
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
                                  style: const TextStyle(
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

  const _CommandAction({
    required this.category,
    required this.title,
    required this.shortcut,
    required this.icon,
  });
}
