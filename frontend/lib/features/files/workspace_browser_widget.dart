// Reusable widget for browsing workspace files and folders.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'file_viewer_screen.dart';
import 'models/workspace_entry.dart';
import 'workspace_browser_controller.dart';

class WorkspaceBrowserWidget extends StatelessWidget {
  final String rootPath;
  final String? agentAlias;
  final bool showHeader;

  const WorkspaceBrowserWidget({
    super.key,
    this.rootPath = '',
    this.agentAlias,
    this.showHeader = true,
  });

  @override
  Widget build(BuildContext context) {
    // Unique tag per rootPath to allow multiple browser instances
    final tag = 'browser_${rootPath}_${agentAlias ?? "default"}';
    final controller = Get.put(
      WorkspaceBrowserController(rootPath: rootPath, agentAlias: agentAlias),
      tag: tag,
    );

    return Column(
      children: [
        if (showHeader) _buildTopBar(context, controller),
        _buildBreadcrumb(context, controller),
        Expanded(
          child: Obx(() {
            if (controller.isLoading.value) {
              return const Center(child: CircularProgressIndicator());
            }

            if (controller.errorMessage.isNotEmpty) {
              return Center(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      const Icon(Icons.error_outline, size: 48, color: Colors.red),
                      const SizedBox(height: 8),
                      Text(controller.errorMessage.value, textAlign: TextAlign.center),
                      const SizedBox(height: 12),
                      ElevatedButton(
                        onPressed: controller.loadEntries,
                        child: const Text('Повторить'),
                      ),
                    ],
                  ),
                ),
              );
            }

            if (controller.entries.isEmpty) {
              return Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(Icons.folder_open, size: 56, color: Colors.grey.shade400),
                    const SizedBox(height: 12),
                    Text(
                      'Папка пуста',
                      style: TextStyle(color: Colors.grey.shade600, fontSize: 16),
                    ),
                    const SizedBox(height: 12),
                    ElevatedButton.icon(
                      onPressed: () => _promptCreateFile(context, controller),
                      icon: const Icon(Icons.add),
                      label: const Text('Создать файл'),
                    ),
                  ],
                ),
              );
            }

            return RefreshIndicator(
              onRefresh: controller.loadEntries,
              child: ListView.separated(
                itemCount: controller.entries.length,
                separatorBuilder: (_, __) => const Divider(height: 1),
                itemBuilder: (context, index) {
                  final entry = controller.entries[index];
                  return _buildEntryTile(context, controller, entry);
                },
              ),
            );
          }),
        ),
      ],
    );
  }

  Widget _buildTopBar(BuildContext context, WorkspaceBrowserController controller) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      color: Theme.of(context).cardColor,
      child: Row(
        children: [
          IconButton(
            tooltip: 'На уровень вверх',
            icon: const Icon(Icons.arrow_upward, size: 20),
            onPressed: controller.canGoUp ? controller.goUp : null,
          ),
          const Spacer(),
          IconButton(
            tooltip: 'Создать папку',
            icon: const Icon(Icons.create_new_folder_outlined, size: 22),
            onPressed: () => _promptCreateFolder(context, controller),
          ),
          IconButton(
            tooltip: 'Создать файл',
            icon: const Icon(Icons.note_add_outlined, size: 22),
            onPressed: () => _promptCreateFile(context, controller),
          ),
          IconButton(
            tooltip: 'Обновить',
            icon: const Icon(Icons.refresh, size: 22),
            onPressed: controller.loadEntries,
          ),
        ],
      ),
    );
  }

  Widget _buildBreadcrumb(BuildContext context, WorkspaceBrowserController controller) {
    return Obx(() {
      final path = controller.currentPath.value;
      final display = path.isEmpty ? '/' : '/$path';
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        color: Theme.of(context).dividerColor.withOpacity(0.05),
        child: Row(
          children: [
            const Icon(Icons.folder_outlined, size: 16, color: Colors.blueGrey),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                display,
                style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
      );
    });
  }

  Widget _buildEntryTile(
    BuildContext context,
    WorkspaceBrowserController controller,
    WorkspaceEntry entry,
  ) {
    return ListTile(
      leading: _entryIcon(entry),
      title: Text(
        entry.name,
        style: TextStyle(
          fontWeight: entry.isDir ? FontWeight.bold : FontWeight.normal,
        ),
      ),
      subtitle: entry.isDir
          ? null
          : Text(
              entry.formattedSize,
              style: TextStyle(fontSize: 11, color: Colors.grey.shade500),
            ),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (entry.protected)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: Colors.blueGrey.withOpacity(0.2),
                borderRadius: BorderRadius.circular(4),
              ),
              child: const Text('системный', style: TextStyle(fontSize: 10, color: Colors.blueGrey)),
            ),
          PopupMenuButton<String>(
            icon: const Icon(Icons.more_vert, size: 18),
            onSelected: (action) {
              if (action == 'rename') {
                _promptRename(context, controller, entry);
              } else if (action == 'delete') {
                _confirmDelete(context, controller, entry);
              }
            },
            itemBuilder: (context) => [
              const PopupMenuItem(value: 'rename', child: Text('Переименовать')),
              if (!entry.protected)
                const PopupMenuItem(
                  value: 'delete',
                  child: Text('Удалить', style: TextStyle(color: Colors.red)),
                ),
            ],
          ),
        ],
      ),
      onTap: () {
        if (entry.isDir) {
          controller.openDirectory(entry);
        } else {
          Get.to(() => FileViewerScreen(
                filePath: entry.path,
                fileName: entry.name,
                agentAlias: controller.activeAgent.value,
                isReadOnly: entry.protected,
              ));
        }
      },
    );
  }

  Widget _entryIcon(WorkspaceEntry entry) {
    if (entry.isDir) {
      return const Icon(Icons.folder, color: Colors.amber, size: 28);
    }
    final ext = entry.extension;
    if (ext == 'md') {
      return const Icon(Icons.description, color: Colors.blue, size: 28);
    }
    if (['json', 'toml', 'yaml', 'yml'].contains(ext)) {
      return const Icon(Icons.data_object, color: Colors.orange, size: 28);
    }
    if (['png', 'jpg', 'jpeg', 'gif', 'webp'].contains(ext)) {
      return const Icon(Icons.image, color: Colors.teal, size: 28);
    }
    return const Icon(Icons.insert_drive_file_outlined, color: Colors.blueGrey, size: 28);
  }

  void _promptCreateFolder(BuildContext context, WorkspaceBrowserController controller) {
    final textCtrl = TextEditingController();
    Get.dialog(
      AlertDialog(
        title: const Text('Новая папка'),
        content: TextField(
          controller: textCtrl,
          autofocus: true,
          decoration: const InputDecoration(hintText: 'Имя папки'),
        ),
        actions: [
          TextButton(onPressed: () => Get.back(), child: const Text('Отмена')),
          ElevatedButton(
            onPressed: () {
              final name = textCtrl.text.trim();
              if (name.isNotEmpty) {
                Get.back();
                controller.createDirectory(name);
              }
            },
            child: const Text('Создать'),
          ),
        ],
      ),
    );
  }

  void _promptCreateFile(BuildContext context, WorkspaceBrowserController controller) {
    final textCtrl = TextEditingController();
    Get.dialog(
      AlertDialog(
        title: const Text('Новый файл'),
        content: TextField(
          controller: textCtrl,
          autofocus: true,
          decoration: const InputDecoration(hintText: 'Имя файла (например, notes.md)'),
        ),
        actions: [
          TextButton(onPressed: () => Get.back(), child: const Text('Отмена')),
          ElevatedButton(
            onPressed: () {
              final name = textCtrl.text.trim();
              if (name.isNotEmpty) {
                Get.back();
                controller.createFile(name);
              }
            },
            child: const Text('Создать'),
          ),
        ],
      ),
    );
  }

  void _promptRename(BuildContext context, WorkspaceBrowserController controller, WorkspaceEntry entry) {
    final textCtrl = TextEditingController(text: entry.name);
    Get.dialog(
      AlertDialog(
        title: const Text('Переименовать'),
        content: TextField(
          controller: textCtrl,
          autofocus: true,
          decoration: const InputDecoration(hintText: 'Новое имя'),
        ),
        actions: [
          TextButton(onPressed: () => Get.back(), child: const Text('Отмена')),
          ElevatedButton(
            onPressed: () {
              final name = textCtrl.text.trim();
              if (name.isNotEmpty && name != entry.name) {
                Get.back();
                controller.renameEntry(entry, name);
              }
            },
            child: const Text('Сохранить'),
          ),
        ],
      ),
    );
  }

  void _confirmDelete(BuildContext context, WorkspaceBrowserController controller, WorkspaceEntry entry) {
    Get.dialog(
      AlertDialog(
        title: Text('Удалить ${entry.isDir ? "папку" : "файл"}?'),
        content: Text('Вы уверены, что хотите удалить "${entry.name}"?'),
        actions: [
          TextButton(onPressed: () => Get.back(), child: const Text('Отмена')),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            onPressed: () {
              Get.back();
              controller.deleteEntry(entry);
            },
            child: const Text('Удалить', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }
}
