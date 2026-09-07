import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../utils/assets.dart';


class AppNameWidget extends StatelessWidget {
  const AppNameWidget({super.key, this.isDark});

  final bool? isDark;

  @override
  Widget build(BuildContext context) {
    bool darkMood = isDark ?? Get.isDarkMode;
    return Image.asset(
      darkMood ? Assets.botLight : Assets.botDark,
      width: MediaQuery.of(context).size.width * .7,
    );
  }
}
