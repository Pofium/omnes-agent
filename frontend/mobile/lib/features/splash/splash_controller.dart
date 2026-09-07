import 'dart:async';
import 'dart:ui';

import 'package:get/get.dart';

import '../../helper/local_storage.dart';
import '../../routes/routes.dart';
import '../../utils/constants.dart';
import '../home/main_controller.dart';


class SplashController extends GetxController {
  // final homeController = Get.put(HomeController());


  @override
  void onReady() {
    // homeController.getCredentials();
    var languageList = LocalStorage.getLanguage();
    var locale = Locale(languageList[0], languageList[1]);

    languageStateName = languageList[2];

    Get.updateLocale(locale);


    MainController.getCredentials();

    _goToScreen();

    super.onReady();
  }

  _goToScreen() async {
    Timer(const Duration(seconds: 2), () {
      if (LocalStorage.isPinProtectionEnabled()) {
        Get.offAndToNamed(Routes.lockScreen);
      } else {
        Get.offAndToNamed(Routes.homeScreen);
      }
    });
  }
}
