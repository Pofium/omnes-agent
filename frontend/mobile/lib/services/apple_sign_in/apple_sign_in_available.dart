import 'package:universal_io/io.dart';
import 'package:flutter/foundation.dart';
import 'package:get/get.dart';

final appleSignInAvailable = AppleSignInAvailable();

class AppleSignInAvailable {
  RxBool isAvailable = false.obs;

  Future<bool> check() async {
    try {
      if (kIsWeb) {
        isAvailable.value = false;
      } else {
        isAvailable.value = Platform.isIOS || Platform.isMacOS;
      }
    } catch (_) {
      isAvailable.value = false;
    }
    return isAvailable.value;
  }
}
