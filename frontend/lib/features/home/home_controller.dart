import 'package:get/get.dart';

import '../../core/supabase/supabase_config.dart';
import '../../helper/local_storage.dart';
import '../../routes/routes.dart';
import '../../utils/constants.dart';
import '../../utils/language/english.dart';
import '../../utils/strings.dart';
import '../../widgets/api/toast_message.dart';
import '../auth/login_controller.dart';

class HomeController extends GetxController {
  var selectedLanguage = "".obs;
  final loginController = Get.put(LoginController());

  @override
  void onInit() {
    if (LocalStorage.isLoggedIn()) {
      checkPremium();
    }
    selectedLanguage.value = languageStateName;
    super.onInit();
  }

  onChangeLanguage(var language, int index) {
    selectedLanguage.value = language;
    if (index == 0) {
      LocalStorage.saveLanguage(
        langSmall: 'en',
        langCap: 'US',
        languageName: English.english,
      );
      languageStateName = English.english;
    } else if (index == 1) {
      LocalStorage.saveLanguage(
        langSmall: 'sp',
        langCap: 'SP',
        languageName: English.spanish,
      );
      languageStateName = English.spanish;
    } else if (index == 2) {
      LocalStorage.saveLanguage(
        langSmall: 'ar',
        langCap: 'AR',
        languageName: English.arabic,
      );
      languageStateName = English.arabic;
    } else if (index == 3) {
      LocalStorage.saveLanguage(
        langSmall: 'bn',
        langCap: 'BN',
        languageName: English.bengali,
      );
      languageStateName = English.bengali;
    } else if (index == 4) {
      LocalStorage.saveLanguage(
        langSmall: 'hn',
        langCap: 'HN',
        languageName: English.hindi,
      );
      languageStateName = English.hindi;
    } else if (index == 5) {
      LocalStorage.saveLanguage(
        langSmall: 'ru',
        langCap: 'RU',
        languageName: English.russian,
      );
      languageStateName = English.russian;
    }
  }

  final List<String> moreList = [
    Strings.english,
    Strings.spanish,
    Strings.arabic,
    Strings.bengali,
    Strings.hindi,
    Strings.russian,
  ];

  final List<String> menuList = [
    Strings.deleteAccount,
  ];

  logout() {
    _removeStorage();
    Get.offAllNamed(Routes.loginScreen);
  }

  deleteAccount() async {
    try {
      final client = SupabaseConfig.client;
      if (client != null) {
        await client.auth.signOut();
      }
    } catch (_) {}

    _removeStorage();
    ToastMessage.success("Аккаунт успешно удален");
    Get.offAllNamed(Routes.splashScreen);
  }

  _removeStorage() {
    LocalStorage.logout();
  }

  checkPremium() {
    LocalStorage.showIsFreeUser(isShowAdYes: false);
    update();
  }
}
