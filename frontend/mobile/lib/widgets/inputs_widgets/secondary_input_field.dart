import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../utils/custom_color.dart';
import '../../utils/custom_style.dart';
import '../../utils/dimensions.dart';

class SecondaryInputField extends StatefulWidget {
  final TextEditingController controller;
  final String hintText;
  final int maxLine;


  const SecondaryInputField({
    super.key,
    required this.controller,
    required this.hintText,
    this.maxLine = 1
  });

  @override
  State<SecondaryInputField> createState() => _SecondaryInputFieldState();
}

class _SecondaryInputFieldState extends State<SecondaryInputField> {

  FocusNode? focusNode;

  @override
  void initState() {
    super.initState();
    focusNode = FocusNode();
  }

  @override
  void dispose() {
    focusNode!.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: EdgeInsets.symmetric(
        vertical: Dimensions.heightSize * .3,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            widget.hintText.tr,
            style: TextStyle(
              color: Theme.of(context).primaryColor
            ),
          ),

          SizedBox(
            height: Dimensions.heightSize * 0.5,
          ),

          Container(
            padding: EdgeInsets.only(
              left: Dimensions.widthSize * 1.2,
            ),
            decoration: BoxDecoration(
              border: Border.all(
                  color: focusNode!.hasFocus
                      ? CustomColor.primaryColor
                      : Theme.of(context).primaryColor.withOpacity(0.7),
                  width: 1
              ),
              color: Colors.transparent,
              borderRadius: BorderRadius.circular(Dimensions.radius * 0.7),
            ),
            child: TextFormField(
              controller: widget.controller,
              style: CustomStyle.primaryTextStyle.copyWith(fontSize: Dimensions.defaultTextSize * 1.6),
              textAlign: TextAlign.left,
              maxLines: widget.maxLine,
              onTap: (){
                setState(() {
                  focusNode!.requestFocus();
                });
              },
              onFieldSubmitted: (value){
                setState(() {
                  focusNode!.unfocus();
                });
              },
              focusNode: focusNode,
              decoration: InputDecoration(
                  hintText: widget.hintText.tr,
                  hintStyle: TextStyle(
                    color: focusNode!.hasFocus
                        ? CustomColor.primaryColor.withOpacity(0.2)
                        : Theme.of(context).primaryColor.withOpacity(0.1),
                    fontSize: Dimensions.defaultTextSize * 2,
                    fontWeight: FontWeight.w500,
                  ),
                  enabledBorder: InputBorder.none,
                  focusedBorder: InputBorder.none,
              ),

            ),
          ),
        ],
      ),
    );
  }
}
