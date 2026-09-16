// Centralized bilingual (RU / EN) localization system for omnes_shared.
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';

class DesktopI18n {
  static final RxString currentLanguage = 'ru'.obs;

  static bool get isRu => currentLanguage.value == 'ru';
  static bool get isEn => currentLanguage.value == 'en';

  /// Returns [ru] if current language is Russian, otherwise [en].
  static String tr(String ru, String en) {
    return isRu ? ru : en;
  }

  /// Initialize language from persistent storage.
  static void init() {
    try {
      final storage = GetStorage();
      final saved = storage.read<String>('desktop_language');
      if (saved == 'en' || saved == 'ru') {
        currentLanguage.value = saved!;
      }
    } catch (_) {}
  }

  /// Change active language and update GetX locale.
  static void setLanguage(String lang) {
    if (lang != 'ru' && lang != 'en') return;
    currentLanguage.value = lang;
    try {
      GetStorage().write('desktop_language', lang);
      Get.updateLocale(Locale(lang));
    } catch (_) {}
  }

  // Common buttons & actions
  static String get finishBtn => tr('Завершить и сохранить', 'Finish & Save');
  static String get cancelBtn => tr('Отмена', 'Cancel');
  static String get nextBtn => tr('Далее', 'Next');
  static String get backBtn => tr('Назад', 'Back');
  static String get applyBtn => tr('Применить', 'Apply');
  static String get skipBtn => tr('Пропустить', 'Skip');
}
