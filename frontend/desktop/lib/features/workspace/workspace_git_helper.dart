import 'package:path/path.dart' as p;
import 'package:universal_io/io.dart' as universal_io;

/// Utility methods for Git repository discovery and branch inspection without sub-processes.
class WorkspaceGitHelper {
  /// Resolves the initial project root path dynamically.
  static String resolveInitialProjectPath() {
    try {
      final envPath = universal_io.Platform.environment['OMNES_WORKSPACE_ROOT'];
      if (envPath != null && envPath.trim().isNotEmpty && universal_io.Directory(envPath).existsSync()) {
        return envPath.trim();
      }
      final currentDir = universal_io.Directory.current.path;
      if (currentDir.isNotEmpty && universal_io.Directory(currentDir).existsSync()) {
        return currentDir;
      }
      final userProfile = universal_io.Platform.environment['USERPROFILE'] ?? universal_io.Platform.environment['HOME'];
      if (userProfile != null && userProfile.isNotEmpty) {
        final devDir = universal_io.Directory(p.join(userProfile, 'Projects'));
        if (devDir.existsSync()) return devDir.path;
        return userProfile;
      }
    } catch (_) {}
    return universal_io.Directory.current.path;
  }

  /// Resolves the initial project name dynamically from the directory path.
  static String resolveInitialProjectName([String? projectPath]) {
    final targetPath = projectPath ?? resolveInitialProjectPath();
    final name = p.basename(targetPath);
    return name.isNotEmpty ? name : 'Мой проект';
  }

  /// Synchronously reads current Git branch from .git/HEAD in 0ms without running sub-processes.
  static String? readGitHeadBranchSync(String? projectPath) {
    if (projectPath == null || projectPath.isEmpty) return null;
    try {
      final headFile = universal_io.File(p.join(projectPath, '.git', 'HEAD'));
      if (!headFile.existsSync()) return null;
      final headContent = headFile.readAsStringSync().trim();
      if (headContent.startsWith('ref: refs/heads/')) {
        return headContent.substring('ref: refs/heads/'.length).trim();
      }
      if (headContent.length >= 7) {
        return headContent.substring(0, 7);
      }
    } catch (_) {}
    return null;
  }

  /// Synchronously reads available local branches from .git/refs/heads in 0ms.
  static List<String> readGitLocalBranchesSync(String? projectPath) {
    if (projectPath == null || projectPath.isEmpty) return [];
    try {
      final headsDir = universal_io.Directory(p.join(projectPath, '.git', 'refs', 'heads'));
      if (!headsDir.existsSync()) return [];
      final branches = <String>[];
      for (final entity in headsDir.listSync(recursive: true)) {
        if (entity is universal_io.File) {
          final relative = p.relative(entity.path, from: headsDir.path);
          branches.add(relative.replaceAll('\\', '/'));
        }
      }
      return branches;
    } catch (_) {
      return [];
    }
  }
}
