import 'package:flutter/material.dart';

import 'custom_color.dart';
import 'dimensions.dart';

class CustomStyle {

  static var primaryTextStyle = TextStyle(
    color: CustomColor.primaryColor,
    fontSize: Dimensions.defaultTextSize,
    fontWeight: FontWeight.w600,
  );

  static var secondaryTextStyle = TextStyle(
    color: CustomColor.secondaryColor,
    fontSize: Dimensions.defaultTextSize,
    fontWeight: FontWeight.w600,
  );

  static var secondary2TextStyle = TextStyle(
    color: CustomColor.secondaryColor2,
    fontSize: Dimensions.defaultTextSize,
    fontWeight: FontWeight.w600,
  );

  static var blackTextStyle = TextStyle(
    color: Colors.black,
    fontSize: Dimensions.defaultTextSize,
    fontWeight: FontWeight.w600,
  );

  static var grayTextStyle = TextStyle(
    color: Colors.grey,
    fontSize: Dimensions.defaultTextSize,
    fontWeight: FontWeight.w600,
  );

  static var whiteTextStyle = TextStyle(
    color: Colors.white,
    fontSize: Dimensions.defaultTextSize,
    fontWeight: FontWeight.w600,
  );


  static var redTextStyle = TextStyle(
    color: Colors.red,
    fontSize: Dimensions.defaultTextSize,
    fontWeight: FontWeight.w400,
  );

}
