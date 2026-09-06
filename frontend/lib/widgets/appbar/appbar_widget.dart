import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:get/get.dart';

import '../../utils/assets.dart';
import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';

// ignore_for_file: deprecated_member_use
class AppBarWidget extends AppBar {
  final BuildContext context;
  final bool moreVisible, actionVisible;
  final String appTitle,buttonText;
  final VoidCallback onPressed, onBackClick;
  final Widget? actionWidget;
  final List<Widget>? extraActions;
  AppBarWidget(
      {super.key,
      required this.appTitle,
      required this.context,
      required this.onBackClick,
      this.moreVisible = true,
      this.actionVisible = true,
      this.buttonText = '',
        this.actionWidget,
        this.extraActions,
        required this.onPressed})
      : super(
          backgroundColor: Colors.transparent,
          title: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                appTitle.tr,
                style: TextStyle(
                  color: Theme.of(context).primaryColor,
                  fontSize: Dimensions.defaultTextSize * 1.8,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ],
          ),
          elevation: 0,
          automaticallyImplyLeading: false,
          centerTitle: true,
          actions: actionVisible ? [
            if (extraActions != null) ...extraActions,
            Visibility(
              visible: moreVisible,
              child: IconButton(
                  onPressed: onPressed,
                  icon: Icon(Icons.more_vert,
                      color: Theme.of(context).primaryColor
                  )),
            ),

            Visibility(
              visible: !moreVisible,
              child: TextButton(
                  onPressed: onPressed,
                  child: Text(buttonText,
                      style: const TextStyle(
                          color: CustomColor.primaryColor
                      ),
                  )),
            ),
          ] : [
            actionWidget ?? const SizedBox()
          ],
          leading: IconButton(
              onPressed: onBackClick,
              icon: SvgPicture.asset(
                Assets.backSVG,
                color: Theme.of(context).primaryColor,
              )),
        );
}
