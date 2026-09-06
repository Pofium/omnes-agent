// Local Notification helper for OmnesAgent (v1 local notifications without FCM).

import 'package:flutter/material.dart';
import 'local_notification_helper.dart';

class NotificationHelper {
  static Future<void> requestPermission() async {
    debugPrint('Notification permission requested');
  }

  static Future<void> initialization() async {
    await NotificationService.init();
  }

  static void localNotification() {
    // Local notifications managed by NotificationService
  }

  static void getBackgroundNotification() {
    // Background notifications handler
  }
}