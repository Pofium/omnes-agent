import 'dart:io';
import 'package:get/get.dart';

final appleSignInAvailable = AppleSignInAvailable();

class AppleSignInAvailable {
  RxBool isAvailable = false.obs;

  Future<bool> check() async {
    try {
      isAvailable.value = Platform.isIOS || Platform.isMacOS;
    } catch (_) {
      isAvailable.value = false;
    }
    return isAvailable.value;
  }
}
