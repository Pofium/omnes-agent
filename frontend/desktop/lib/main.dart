// Main entry point for OmnesAgent Desktop Workstation.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:omnes_shared/omnes_shared.dart';
import 'package:window_manager/window_manager.dart';

import 'features/splash/desktop_splash_screen.dart';
import 'theme/desktop_theme.dart';
import 'utils/desktop_i18n.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await windowManager.ensureInitialized();
  await GetStorage.init();
  DesktopI18n.init();
  Get.put(DesktopThemeController());

  runApp(const OmnesDesktopApp());

  const windowOptions = WindowOptions(
    size: Size(1360, 860),
    minimumSize: Size(1024, 640),
    center: true,
    backgroundColor: Color(0xFF090D16),
    skipTaskbar: false,
    titleBarStyle: TitleBarStyle.normal,
    title: 'OmnesAgent Desktop ADE',
  );

  windowManager.waitUntilReadyToShow(windowOptions, () async {
    await windowManager.show();
    await windowManager.focus();
    await windowManager.setPreventClose(true);
    await windowManager.maximize();
  });
}

class OmnesDesktopApp extends StatelessWidget {
  const OmnesDesktopApp({super.key});

  @override
  Widget build(BuildContext context) {
    return GetMaterialApp(
      title: 'OmnesAgent Desktop ADE',
      debugShowCheckedModeBanner: false,
      theme: DesktopTheme.lightTheme,
      darkTheme: DesktopTheme.darkTheme,
      themeMode: ThemeMode.dark,
      translations: LocalString(),
      locale: const Locale('ru', 'RU'),
      fallbackLocale: const Locale('en', 'US'),
      home: const DesktopSplashScreen(),
    );
  }
}
