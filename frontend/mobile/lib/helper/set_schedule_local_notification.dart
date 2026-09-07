
import 'package:flutter/material.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:get/get.dart';
import 'package:timezone/timezone.dart' as tz;

import '../model/schedule/schedule_model.dart';
import '../utils/strings.dart';
import '../widgets/api/toast_message.dart';
import 'permissions.dart';

class SetScheduleNotification{

  static setSchedule(ScheduleModel schedule)async{
    final permissionGranted = await requestScheduleExactAlarmPermission();
    if (permissionGranted) {
      await checkScheduleNotification(schedule);
    } else {
      debugPrint("Need to permission alarm");
    }
  }

  static checkScheduleNotification(ScheduleModel schedule) {
      _scheduleNotification(schedule);
  }

  static Future<void> _scheduleNotification(ScheduleModel schedule) async {

    const androidNotificationDetails = AndroidNotificationDetails(
        'channel id',
        'channel name',
        channelDescription: "channel description",
        sound: RawResourceAndroidNotificationSound("alarm"),
        enableVibration: true,
        playSound: true
    );

    const iosNotificationDetails = DarwinNotificationDetails(
      presentAlert: true,
      presentSound: true,
      presentBadge: true,
      // sound:
    );
    const notificationDetails = NotificationDetails(
        android: androidNotificationDetails,
        iOS: iosNotificationDetails
    );

    /// once
    if(schedule.scheduleType == "Once"){
      var now = DateTime.now();
      // print(now);
      DateTime date = DateTime.parse("${schedule.date} ${schedule.time}");

      if (date.isBefore(now)) {
        await removeIDNotification(schedule.id);
      } else {
        final tzTime = tz.TZDateTime.from(date, tz.local);
        await _set(schedule, tzTime, notificationDetails);
      }
    }

    /// daily todo
    else if(schedule.scheduleType == "Daily"){
      // DateTime date = DateTime.parse("${schedule.date} ${schedule.time}");

      // _nextInstanceOfTime(date),
      await _setDaily(schedule,  notificationDetails);
    }

    ///weekly todo
    else if(schedule.scheduleType == "Weekly"){
      await _setWeekly(schedule,  notificationDetails);
    }
  }

  static Future<void> _set(ScheduleModel schedule, tz.TZDateTime tzTime, NotificationDetails notificationDetails) async{
    await FlutterLocalNotificationsPlugin().zonedSchedule(
      schedule.id,
      schedule.title,
      schedule.description,
      tzTime,
      notificationDetails,
      // ignore: deprecated_member_use
      androidAllowWhileIdle: true,
      uiLocalNotificationDateInterpretation: UILocalNotificationDateInterpretation.absoluteTime,
    );

    debugPrint("${schedule.title} is setted =>  ${tzTime.day}:${tzTime.hour}:${tzTime.minute}");

    ToastMessage.success(Strings.setScheduleReminder.tr);
  }

  static Future<void> _setDaily(ScheduleModel schedule, NotificationDetails notificationDetails) async{
    await FlutterLocalNotificationsPlugin().periodicallyShow(
      schedule.id,
      schedule.title,
      schedule.description,
      RepeatInterval.daily,
      notificationDetails,
    );


    ToastMessage.success(Strings.setScheduleReminder.tr);
  }

  static Future<void> _setWeekly(ScheduleModel schedule, NotificationDetails notificationDetails) async{
    await FlutterLocalNotificationsPlugin().periodicallyShow(
      schedule.id,
      schedule.title,
      schedule.description,
      RepeatInterval.weekly,
      notificationDetails,
    );


    ToastMessage.success(Strings.setScheduleReminder.tr);
  }

  /// Calculates the next instance of a specific time within 24 hours.
  // static tz.TZDateTime _nextInstanceOfTime(DateTime time) {
  //   final now = DateTime.now();
  //   final scheduledTime = DateTime(now.year, now.month, now.day, time.hour, time.minute, time.second);
  //   // final location = tz.getLocation('Asia/Dhaka');
  //   // If the scheduled time has already passed for today, calculate the time for tomorrow
  //   if (scheduledTime.isBefore(now)) {
  //     return tz.TZDateTime.from(scheduledTime.add(const Duration(days: 1)), tz.local);
  //   } else {
  //     return tz.TZDateTime.from(scheduledTime, tz.local);
  //   }
  // }


  static Future<void> removeIDNotification(int id) async {
      final FlutterLocalNotificationsPlugin notifications = FlutterLocalNotificationsPlugin();
      await notifications.cancel(id);
  }

  static Future<void> cancelAllNotifications() async {
    final FlutterLocalNotificationsPlugin notifications = FlutterLocalNotificationsPlugin();
    await notifications.cancelAll();
  }
}