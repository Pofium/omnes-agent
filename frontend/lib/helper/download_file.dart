import 'package:universal_io/io.dart';
import 'package:flutter/foundation.dart';
import 'package:omagent_front/widgets/api/toast_message.dart';
import 'package:device_info_plus/device_info_plus.dart';
import 'package:http/http.dart' as http;
import 'package:path_provider/path_provider.dart';
import 'package:permission_handler/permission_handler.dart';


mixin DownloadFile {
  Future<bool> checkPermission() async {
    if (kIsWeb) return false;
    bool checkPermission1 = await Permission.storage.isGranted;

    //For android
    DeviceInfoPlugin plugin = DeviceInfoPlugin();
    AndroidDeviceInfo android = await plugin.androidInfo;
    debugPrint(android.version.sdkInt.toString());
    if (android.version.sdkInt < 33) {
      debugPrint("SDK Version < 33");
      if (await Permission.storage.request().isGranted) {
        checkPermission1 = true;
      } else if (await Permission.storage.request().isPermanentlyDenied) {
        await openAppSettings();
      }
    } else {

      debugPrint("SDK Version > 33");
      if (await Permission.photos.request().isGranted) {
        checkPermission1 = true;
      } else if (await Permission.photos.request().isPermanentlyDenied) {
        await openAppSettings();
      } else if (await Permission.photos.request().isDenied) {
        checkPermission1 = false;
      }
    }

    debugPrint(checkPermission1.toString());

    return checkPermission1;
  }

  Future<void> downloadFile({required String url, required String name}) async {
    if (kIsWeb) return;
    bool isChecked = Platform.isIOS ? true : await checkPermission();
    if (isChecked) {
      final http.Response response = await http.get(Uri.parse(url));

      debugPrint("Url ->>>   $url");
      debugPrint("Status Code ->>>   ${response.statusCode}");
      debugPrint("Save ->>>   Start");

      if (response.statusCode == 200) {
        debugPrint("Save ->>>   1");
        Directory? downloadsDirectory;
        if (Platform.isIOS) {
          downloadsDirectory = await getApplicationDocumentsDirectory();
        } else {
          debugPrint("Save ->>>   2");
          String directory;
          if (Platform.isAndroid) {
            debugPrint("Save ->>>   3");

            directory = "/storage/emulated/0/";
            debugPrint("Save ->>>   4");

            final bool dirDownloadExists =
            await Directory("$directory/Download").exists();
            debugPrint("Save ->>>   5");

            directory += dirDownloadExists ? "Download" : "Downloads";
          } else {
            // Handle other platforms here, if applicable
            return;
          }
          debugPrint("Save ->>>   6");

          downloadsDirectory = Directory(directory);
        }

        debugPrint("Save ->>>   7");

        final File file = File('${downloadsDirectory.path}/$name');
        debugPrint("Save ->>>   ${file.path}");
        debugPrint("Save ->>>   8");
        try {
          await file.writeAsBytes(response.bodyBytes).then((value) {
            ToastMessage.success(
                'File downloaded successfully at ${file.path}!');
            debugPrint(file.path);
          });
        } catch (e, s) {
          debugPrint("Save ->>>   Error");
          debugPrint("@@ $e");
          debugPrint("## $s");
          ToastMessage.error('Failed to download the file.');
        }
      } else {
        ToastMessage.error('Failed to download the file.');
      }
    } else {
      ToastMessage.error('Permission is not granted.');
    }
  }

  Future<void> downloadFile2(
      {required Uint8List pdfData, required String name}) async {

    bool isChecked = Platform.isIOS ? true : await checkPermission();


    if (isChecked) {
      Directory? downloadsDirectory;
      if (Platform.isIOS) {
        downloadsDirectory = await getApplicationDocumentsDirectory();
      } else {
        String directory;
        if (Platform.isAndroid) {
          directory = "/storage/emulated/0/";
          final bool dirDownloadExists =
          await Directory("$directory/Download").exists();
          directory += dirDownloadExists ? "Download" : "Downloads";
        } else {
          return;
        }
        downloadsDirectory = Directory(directory);
      }
      final File file = File('${downloadsDirectory.path}/$name');
      try {
        await file.writeAsBytes(pdfData).then((value) {
          ToastMessage.success(
              'File downloaded successfully at ${file.path}!');
          debugPrint(file.path);
        });
      } catch (e, s) {
        debugPrint("Save ->>>   Error");
        debugPrint("@@ $e");
        debugPrint("## $s");
        ToastMessage.error('Failed to download the file.');
      }
    } else {
      ToastMessage.error('Permission is not granted.');
    }
  }
}
