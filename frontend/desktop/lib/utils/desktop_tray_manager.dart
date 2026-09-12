// System tray manager for OmnesAgent Desktop ADE.
// Implements system tray icon, click events, and context menu matching the screenshot specification.

import 'dart:io';
import 'package:flutter/material.dart';
import 'package:tray_manager/tray_manager.dart';
import 'package:window_manager/window_manager.dart';

class DesktopTrayManager with TrayListener {
  static final DesktopTrayManager instance = DesktopTrayManager._();
  DesktopTrayManager._();

  VoidCallback? onOpenOA;
  VoidCallback? onNewTask;
  VoidCallback? onOpenWorkspace;
  VoidCallback? onCheckUpdates;
  VoidCallback? onAboutOA;
  VoidCallback? onClearData;
  VoidCallback? onQuit;

  bool _initialized = false;

  /// Initializes system tray icon and sets up context menu.
  Future<void> init({
    VoidCallback? onOpenOA,
    VoidCallback? onNewTask,
    VoidCallback? onOpenWorkspace,
    VoidCallback? onCheckUpdates,
    VoidCallback? onAboutOA,
    VoidCallback? onClearData,
    VoidCallback? onQuit,
  }) async {
    this.onOpenOA = onOpenOA;
    this.onNewTask = onNewTask;
    this.onOpenWorkspace = onOpenWorkspace;
    this.onCheckUpdates = onCheckUpdates;
    this.onAboutOA = onAboutOA;
    this.onClearData = onClearData;
    this.onQuit = onQuit;

    if (_initialized) return;
    _initialized = true;

    trayManager.addListener(this);

    try {
      // Set tray icon (assets/app_icon.ico packaged in flutter_assets)
      if (Platform.isWindows) {
        await trayManager.setIcon('assets/app_icon.ico');
      } else {
        await trayManager.setIcon('assets/Logo/app_icon.png');
      }
      await trayManager.setToolTip('OmnesAgent (OA)');

      final menu = Menu(
        items: [
          MenuItem(
            key: 'open_oa',
            label: 'Open OA',
          ),
          MenuItem.separator(),
          MenuItem(
            key: 'new_task',
            label: 'New task',
          ),
          MenuItem(
            key: 'open_workspace',
            label: 'Open workspace',
          ),
          MenuItem.separator(),
          MenuItem(
            key: 'check_updates',
            label: 'Check for updates',
          ),
          MenuItem(
            key: 'about_oa',
            label: 'About OA',
          ),
          MenuItem(
            key: 'clear_data',
            label: 'Clear all data',
          ),
          MenuItem.separator(),
          MenuItem(
            key: 'quit',
            label: 'Quit',
          ),
        ],
      );
      await trayManager.setContextMenu(menu);
    } catch (_) {}
  }

  @override
  void onTrayIconMouseDown() async {
    // Single click on tray icon restores and focuses window
    await windowManager.show();
    await windowManager.focus();
    onOpenOA?.call();
  }

  @override
  void onTrayIconRightMouseDown() {
    // Right click opens tray context menu
    trayManager.popUpContextMenu();
  }

  @override
  void onTrayMenuItemClick(MenuItem menuItem) async {
    switch (menuItem.key) {
      case 'open_oa':
        await windowManager.show();
        await windowManager.focus();
        onOpenOA?.call();
        break;
      case 'new_task':
        await windowManager.show();
        await windowManager.focus();
        onNewTask?.call();
        break;
      case 'open_workspace':
        await windowManager.show();
        await windowManager.focus();
        onOpenWorkspace?.call();
        break;
      case 'check_updates':
        await windowManager.show();
        await windowManager.focus();
        onCheckUpdates?.call();
        break;
      case 'about_oa':
        await windowManager.show();
        await windowManager.focus();
        onAboutOA?.call();
        break;
      case 'clear_data':
        await windowManager.show();
        await windowManager.focus();
        onClearData?.call();
        break;
      case 'quit':
        onQuit?.call();
        break;
    }
  }

  /// Cleans up tray resources on application shutdown.
  Future<void> destroy() async {
    try {
      trayManager.removeListener(this);
      await trayManager.destroy();
    } catch (_) {}
  }
}
