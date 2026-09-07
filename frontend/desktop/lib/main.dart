// Main entry point for OmnesAgent Desktop Workstation.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:omnes_shared/omnes_shared.dart';

import 'features/desktop_shell.dart';
import 'theme/desktop_theme.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await GetStorage.init();
  Get.put(DesktopThemeController());

  runApp(const OmnesDesktopApp());
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
      home: const DesktopShell(),
    );
  }
}
