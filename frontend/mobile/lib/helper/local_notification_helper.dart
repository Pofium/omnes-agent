import 'package:flutter/foundation.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:uuid/uuid.dart';

import '../../utils/strings.dart';
class NotificationService {
  /// set channel info
  static const String channelId = '1';
  static const String channelName = Strings.appName;
  /// Initialize Local Notification
  static init() {
    if (kIsWeb) return;
    final FlutterLocalNotificationsPlugin flutterLocalNotificationsPlugin =
    FlutterLocalNotificationsPlugin();
    flutterLocalNotificationsPlugin.resolvePlatformSpecificImplementation<AndroidFlutterLocalNotificationsPlugin>()?.requestNotificationsPermission();
    flutterLocalNotificationsPlugin.resolvePlatformSpecificImplementation<AndroidFlutterLocalNotificationsPlugin>()?.requestExactAlarmsPermission();
    flutterLocalNotificationsPlugin.initialize(
      const InitializationSettings(
        // Android settings.
        android: AndroidInitializationSettings('@mipmap/launcher_icon'),
        // iOS settings.
        iOS: DarwinInitializationSettings(
          requestAlertPermission: true,
          requestBadgePermission: true,
          requestSoundPermission: true,
        ),
      ),
    );
  }
  static FlutterLocalNotificationsPlugin flutterLocalNotificationsPlugin =
  FlutterLocalNotificationsPlugin();
  static NotificationDetails notificationDetails = const NotificationDetails(
    // Android details.
    android: AndroidNotificationDetails(
      channelId,
      channelName,
    ),
    // iOS details.
    iOS: DarwinNotificationDetails(
      presentAlert: true,
      presentBadge: true,
      presentSound: true,
    ),
  );

  static showLocalNotification({  required String title, required String body}) {
    if (kIsWeb) return;
    flutterLocalNotificationsPlugin.show(
      const Uuid().v4().hashCode, // Use the unique ID as the notification ID
      title,
      body,
      notificationDetails,
    );
    debugPrint("<<< Success! >>>");
  }

  Future<void> cancelNotification(int id) async {
    await flutterLocalNotificationsPlugin.cancel(id);
  }

  Future<void> cancelAllNotifications() async {
    await flutterLocalNotificationsPlugin.cancelAll();
  }
}