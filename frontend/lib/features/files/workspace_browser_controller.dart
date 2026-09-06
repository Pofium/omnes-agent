// Controller managing workspace files and directories browsing.

import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import 'models/workspace_entry.dart';
import 'workspace_repository.dart';

class WorkspaceBrowserController extends GetxController {
  final WorkspaceRepository _repo;
  final String rootPath;
  final String? agentAlias;

  WorkspaceBrowserController({
    this.rootPath = '',
    this.agentAlias,
    WorkspaceRepository? repo,
  }) : _repo = repo ?? WorkspaceRepository();

  final RxString currentPath = ''.obs;
  final RxList<WorkspaceEntry> entries = <WorkspaceEntry>[].obs;
  final RxBool isLoading = false.obs;
  final RxString errorMessage = ''.obs;
  final RxString activeAgent = ''.obs;

  @override
  void onInit() {
    super.onInit();
    currentPath.value = rootPath;
    loadEntries();
  }

  /// Whether current directory is below rootPath and can go up.
  bool get canGoUp {
    final cur = currentPath.value.trim().replaceAll(RegExp(r'^/+|/+$'), '');
    final root = rootPath.trim().replaceAll(RegExp(r'^/+|/+$'), '');
    return cur.isNotEmpty && cur != root;
  }

  /// Navigates one directory level up.
  void goUp() {
    if (!canGoUp) return;
    final parts = currentPath.value.split('/');
    if (parts.length > 1) {
      parts.removeLast();
      currentPath.value = parts.join('/');
    } else {
      currentPath.value = '';
    }
    loadEntries();
  }

  /// Opens a subdirectory.
  void openDirectory(WorkspaceEntry entry) {
    if (!entry.isDir) return;
    currentPath.value = entry.path;
    loadEntries();
  }

  /// Loads entries in currentPath.
  Future<void> loadEntries() async {
    isLoading.value = true;
    errorMessage.value = '';
    try {
      activeAgent.value = agentAlias ?? await GatewayConfig.getActiveAgent();
      final list = await _repo.listDirectory(currentPath.value, agentAlias: activeAgent.value);
      // Sort directories first, then alphabetically
      list.sort((a, b) {
        if (a.isDir && !b.isDir) return -1;
        if (!a.isDir && b.isDir) return 1;
        return a.name.toLowerCase().compareTo(b.name.toLowerCase());
      });
      entries.assignAll(list);
    } catch (e) {
      errorMessage.value = 'Ошибка загрузки файлов: $e';
    } finally {
      isLoading.value = false;
    }
  }

  /// Creates a new directory inside currentPath.
  Future<bool> createDirectory(String dirName) async {
    final cleanName = dirName.trim();
    if (cleanName.isEmpty) return false;
    final fullPath = currentPath.value.isEmpty ? cleanName : '${currentPath.value}/$cleanName';
    isLoading.value = true;
    try {
      final success = await _repo.createDirectory(fullPath, agentAlias: activeAgent.value);
      if (success) {
        await loadEntries();
        return true;
      }
    } catch (_) {} finally {
      isLoading.value = false;
    }
    return false;
  }

  /// Creates a new text file inside currentPath.
  Future<bool> createFile(String fileName, {String content = ''}) async {
    final cleanName = fileName.trim();
    if (cleanName.isEmpty) return false;
    final fullPath = currentPath.value.isEmpty ? cleanName : '${currentPath.value}/$cleanName';
    isLoading.value = true;
    try {
      final success = await _repo.writeFile(fullPath, content, agentAlias: activeAgent.value);
      if (success) {
        await loadEntries();
        return true;
      }
    } catch (_) {} finally {
      isLoading.value = false;
    }
    return false;
  }

  /// Reads a file content.
  Future<WorkspaceFileContent?> readFile(String path) async {
    try {
      return await _repo.readFile(path, agentAlias: activeAgent.value);
    } catch (_) {
      return null;
    }
  }

  /// Writes/saves file content.
  Future<bool> saveFile(String path, String content) async {
    try {
      return await _repo.writeFile(path, content, agentAlias: activeAgent.value);
    } catch (_) {
      return false;
    }
  }

  /// Renames or moves an entry.
  Future<bool> renameEntry(WorkspaceEntry entry, String newName) async {
    final clean = newName.trim();
    if (clean.isEmpty || clean == entry.name) return false;
    final parent = currentPath.value;
    final newPath = parent.isEmpty ? clean : '$parent/$clean';
    isLoading.value = true;
    try {
      final success = await _repo.movePath(entry.path, newPath, agentAlias: activeAgent.value);
      if (success) {
        await loadEntries();
        return true;
      }
    } catch (_) {} finally {
      isLoading.value = false;
    }
    return false;
  }

  /// Deletes an entry.
  Future<bool> deleteEntry(WorkspaceEntry entry) async {
    isLoading.value = true;
    try {
      final success = await _repo.deletePath(entry.path, agentAlias: activeAgent.value);
      if (success) {
        entries.removeWhere((e) => e.path == entry.path);
        return true;
      }
    } catch (_) {} finally {
      isLoading.value = false;
    }
    return false;
  }

  @override
  void onClose() {
    _repo.dispose();
    super.onClose();
  }
}
