import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';

import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';


class Button extends StatelessWidget {
  final Widget child;
  final VoidCallback onClick;

  const Button({super.key, required this.child, required this.onClick});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: MediaQuery.of(context).size.width,
      height: 45.h,
      child: Material(
        color: CustomColor.primaryColor,
          borderRadius: BorderRadius.circular(Dimensions.radius),
          child: InkWell(
            onTap: onClick,
            child: Center(
              child: child,
            )),
      ),
    );
  }
}
