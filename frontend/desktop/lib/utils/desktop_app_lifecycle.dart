// Manages top-level application lifecycle and clean shutdown of frontend + backend.

import 'dart:io';
import 'package:window_manager/window_manager.dart';
import 'desktop_backend_manager.dart';
import 'desktop_tray_manager.dart';

class DesktopAppLifecycle {
  static bool _isQuitting = false;

  /// Performs clean shutdown: stops backend gateway, destroys tray icon, closes window, and terminates process.
  static Future<void> quitApp() async {
    if (_isQuitting) return;
    _isQuitting = true;

    // 1. Terminate backend gateway process
    await DesktopBackendManager.stopBackend();

    // 2. Remove tray icon
    await DesktopTrayManager.instance.destroy();

    // 3. Destroy window and exit
    try {
      await windowManager.destroy();
    } catch (_) {}
    exit(0);
  }
}
