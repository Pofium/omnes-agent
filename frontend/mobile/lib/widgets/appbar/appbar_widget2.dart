import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';
import '../../utils/strings.dart';

class AppBarWidget2 extends AppBar {
  final BuildContext context;
  final VoidCallback onTap;
  final String appTitle;
  final double elv;
  AppBarWidget2({
    super.key,
    required this.appTitle,
    required this.context,
    required this.onTap,
    this.elv = 0,
  }) : super(
            backgroundColor: Colors.transparent,
            centerTitle: true,
            title: Text(appTitle.tr,
                style: TextStyle(
                    color: Theme.of(context).primaryColor,
                    fontSize: Dimensions.defaultTextSize * 2,
                    fontWeight: FontWeight.w600)),
            elevation: elv,
            actions: [
              InkWell(
                onTap: onTap,
                child: Container(
                  alignment: Alignment.center,
                  height: Dimensions.heightSize * 2,
                  width: Dimensions.widthSize * 5.5,
                  margin: EdgeInsets.symmetric(
                    horizontal: Dimensions.widthSize,
                    vertical: Dimensions.heightSize * 1,
                  ),
                  decoration: BoxDecoration(
                      borderRadius:
                          BorderRadius.circular(Dimensions.radius * 1.3),
                      color: CustomColor.primaryColor.withOpacity(0.1)),
                  child: Text(Strings.skip,
                      style: TextStyle(
                          color: CustomColor.primaryColor,
                          fontSize: Dimensions.defaultTextSize * 1.2,
                          fontWeight: FontWeight.w600)),
                ),
              )
            ]);
}
