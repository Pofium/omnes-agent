// Backend process lifecycle manager for OmnesAgent Desktop Workstation.
// Starts omnesagent gateway daemon on desktop launch and terminates it on exit.

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import 'package:path/path.dart' as path;

class DesktopBackendManager {
  static Process? _backendProcess;
  static bool _startedByDesktop = false;

  /// Checks whether OmnesAgent gateway is currently responding on port 42617.
  static Future<bool> isBackendRunning() async {
    try {
      final res = await http
          .get(Uri.parse('http://127.0.0.1:42617/health'))
          .timeout(const Duration(milliseconds: 600));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Locates the Rust backend omnesagent binary in development or distribution locations.
  /// Explicitly avoids resolving to the desktop Flutter runner itself (OmnesAgent.exe).
  static String? findOmnesAgentBinary() {
    final currentAppExe = path.canonicalize(Platform.resolvedExecutable);

    // 1. Environment variable override
    final envBin = Platform.environment['OMNESAGENT_BIN'];
    if (envBin != null && File(envBin).existsSync() && path.canonicalize(envBin) != currentAppExe) {
      return envBin;
    }

    final exeDir = path.dirname(Platform.resolvedExecutable);

    // 2. Production packaging subdirectories (backend/omnesagent.exe or bin/omnesagent.exe)
    final subBackend = path.join(exeDir, 'backend', 'omnesagent.exe');
    if (File(subBackend).existsSync() && path.canonicalize(subBackend) != currentAppExe) {
      return subBackend;
    }

    final subBin = path.join(exeDir, 'bin', 'omnesagent.exe');
    if (File(subBin).existsSync() && path.canonicalize(subBin) != currentAppExe) {
      return subBin;
    }

    // 3. Dev tree: climb up directories to find workspace root and backend/target
    Directory dir = Directory(exeDir);
    for (int i = 0; i < 7; i++) {
      final debugExe = path.join(dir.path, 'backend', 'target', 'debug', 'omnesagent.exe');
      if (File(debugExe).existsSync() && path.canonicalize(debugExe) != currentAppExe) {
        return debugExe;
      }

      final releaseExe = path.join(dir.path, 'backend', 'target', 'release', 'omnesagent.exe');
      if (File(releaseExe).existsSync() && path.canonicalize(releaseExe) != currentAppExe) {
        return releaseExe;
      }

      final parent = dir.parent;
      if (parent.path == dir.path) break;
      dir = parent;
    }

    // 4. Default hardcoded workspace dev paths
    const devPathDebug = r'C:\Projects\Omnes-agent\backend\target\debug\omnesagent.exe';
    if (File(devPathDebug).existsSync() && path.canonicalize(devPathDebug) != currentAppExe) {
      return devPathDebug;
    }

    const devPathRelease = r'C:\Projects\Omnes-agent\backend\target\release\omnesagent.exe';
    if (File(devPathRelease).existsSync() && path.canonicalize(devPathRelease) != currentAppExe) {
      return devPathRelease;
    }

    return null;
  }

  /// Determines the workspace root directory from binary location.
  static String findWorkspaceRoot(String binPath) {
    Directory d = Directory(path.dirname(binPath));
    while (d.path != d.parent.path) {
      if (Directory(path.join(d.path, 'backend')).existsSync() ||
          File(path.join(d.path, 'Cargo.toml')).existsSync()) {
        return d.path;
      }
      d = d.parent;
    }
    return path.dirname(binPath);
  }

  /// Starts backend gateway process if it is not already running.
  static Future<bool> startBackendIfNeeded() async {
    if (await isBackendRunning()) {
      debugPrint('[DesktopBackendManager] Backend gateway is already active on port 42617');
      return true;
    }

    final binPath = findOmnesAgentBinary();
    if (binPath == null) {
      debugPrint('[DesktopBackendManager] Could not find omnesagent backend binary');
      return false;
    }

    final workingDir = findWorkspaceRoot(binPath);
    debugPrint('[DesktopBackendManager] Spawning backend: $binPath (cwd: $workingDir)');

    try {
      _backendProcess = await Process.start(
        binPath,
        ['gateway', 'start', '-p', '42617'],
        workingDirectory: workingDir,
        mode: ProcessStartMode.normal,
      );
      _startedByDesktop = true;

      // Drain stdout and stderr
      _backendProcess!.stdout.transform(utf8.decoder).listen((line) {
        if (kDebugMode) debugPrint('[Gateway stdout] $line');
      });
      _backendProcess!.stderr.transform(utf8.decoder).listen((line) {
        if (kDebugMode) debugPrint('[Gateway stderr] $line');
      });

      // Wait up to 8 seconds for gateway to report healthy
      for (int i = 0; i < 16; i++) {
        await Future.delayed(const Duration(milliseconds: 500));
        if (await isBackendRunning()) {
          debugPrint('[DesktopBackendManager] Backend gateway is online and verified healthy!');
          return true;
        }
      }
    } catch (e) {
      debugPrint('[DesktopBackendManager] Error starting backend: $e');
      return false;
    }
    return false;
  }

  /// Terminates backend gateway process when desktop application closes.
  static Future<void> stopBackend() async {
    debugPrint('[DesktopBackendManager] Stopping backend gateway process...');
    if (_backendProcess != null) {
      try {
        _backendProcess!.kill(ProcessSignal.sigterm);
      } catch (_) {}
      try {
        _backendProcess!.kill();
      } catch (_) {}
      _backendProcess = null;
    }

    // If started by desktop, ensure background gateway is stopped cleanly
    if (_startedByDesktop) {
      try {
        if (Platform.isWindows) {
          await Process.run('taskkill', ['/F', '/IM', 'omnesagent.exe']);
        }
      } catch (_) {}
    }
  }
}
