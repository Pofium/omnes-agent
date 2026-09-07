import 'package:permission_handler/permission_handler.dart';
import 'package:device_info_plus/device_info_plus.dart';

Future<bool> requestScheduleExactAlarmPermission() async {

  final int sdkInt = await checkAndroidVersion();
  if (sdkInt >= 33) {
    final permissionStatus = await Permission.scheduleExactAlarm.request();

    if (permissionStatus.isDenied) {

      return false;
    } else if (permissionStatus.isPermanentlyDenied) {

      return false;
    } else {
      return true;
    }
  }
  else {
    var status = await Permission.scheduleExactAlarm.status;
    if(status == PermissionStatus.denied){
      // return openAppSettings();
      return true;
    }else{
      return true;
    }
  }

}

Future<int> checkAndroidVersion() async {
  final DeviceInfoPlugin deviceInfoPlugin = DeviceInfoPlugin();
  final AndroidDeviceInfo androidDeviceInfo = await deviceInfoPlugin.androidInfo;
  final int sdkInt = androidDeviceInfo.version.sdkInt;
  return sdkInt;
}