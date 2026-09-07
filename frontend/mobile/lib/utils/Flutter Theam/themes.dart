/*
you need to add this to your pubspec.yaml under dependency section
    get: ^4.6.1
    get_storage: ^2.0.3

put the theme file under the utils section then and you need to initilize the get storage in the main.dart file

example:
    void main() async {
      await GetStorage.init(); // initializing getStorage
      runApp(const MyApp());
    }

then you need to add this 3 line of code in the material app function 
      theme: Themes.light,
      darkTheme: Themes.dark,
      themeMode: Themes().theme,

if you need to use any condition due to light and dark mode you can use
use exmaple:
 backgroundColor: Get.isDarkMode
          ? Theme.of(context).scaffoldBackgroundColor
          : Theme.of(context).scaffoldBackgroundColor,

and all the colors you need to add from customColor.dart file.
*/

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';

import '../custom_color.dart';

class Themes {
  final _box = GetStorage();
  final _key = 'isDarkMode';

  _saveThemeToBox(bool isDarkMode) => _box.write(_key, isDarkMode);

  bool _loadThemeFromBox() => _box.read(_key) ?? true;

  ThemeMode get theme => _loadThemeFromBox() ? ThemeMode.dark : ThemeMode.light;

  void switchTheme() {
    Get.changeThemeMode(_loadThemeFromBox() ? ThemeMode.light : ThemeMode.dark);
    _saveThemeToBox(!_loadThemeFromBox());
  }

  static final light = ThemeData.light().copyWith(
    primaryColor: CustomColor.blackColor,
    scaffoldBackgroundColor: CustomColor.whiteColor,
    // cardColor: CustomColor.secondaryColor,

    brightness: Brightness.light,
    appBarTheme: const AppBarTheme(
        color: CustomColor.whiteColor,
      iconTheme: IconThemeData(
          color: CustomColor.blackColor
      )
    ),
    textTheme: ThemeData.light().textTheme.apply(
          bodyColor: Colors.black,
        ),
  );

  static final dark = ThemeData.dark().copyWith(
    primaryColor: CustomColor.whiteColor,
    scaffoldBackgroundColor: CustomColor.bgColor,
    // cardColor: CustomColor.accentColor,

    brightness: Brightness.dark,
    appBarTheme: const AppBarTheme(
        color: CustomColor.blackColor,
        iconTheme: IconThemeData(
            color: CustomColor.whiteColor
        )
    ),
    textTheme: ThemeData.dark().textTheme.apply(
          bodyColor: Colors.white,
        ),
  );
}
