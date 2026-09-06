import 'package:flutter/material.dart';
import 'package:get/get.dart';


import '../../utils/custom_color.dart';
import '../../utils/custom_style.dart';
import '../../utils/dimensions.dart';

class PasswordInputWidget extends StatefulWidget {
  final String hint;
  final int maxLines;
  final TextEditingController controller;

  const PasswordInputWidget({
    super.key,
    required this.controller,
    required this.hint,
    this.maxLines = 1,
  });

  @override
  State<PasswordInputWidget> createState() => _PrimaryInputWidgetState();
}

class _PrimaryInputWidgetState extends State<PasswordInputWidget> {
  FocusNode? focusNode;
  bool obscureText = true;

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
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            widget.hint.tr,
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
              maxLines: widget.maxLines,
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
              textInputAction: TextInputAction.next,
              obscureText: obscureText,
              decoration: InputDecoration(
                hintText: widget.hint.tr,
                hintStyle: TextStyle(
                  color: focusNode!.hasFocus
                      ? CustomColor.primaryColor.withOpacity(0.2)
                      : Theme.of(context).primaryColor.withOpacity(0.1),
                  fontSize: Dimensions.defaultTextSize * 2,
                  fontWeight: FontWeight.w500,
                ),
                enabledBorder: InputBorder.none,
                focusedBorder: InputBorder.none,
                suffixIcon: GestureDetector(
                  onTap: () {
                    setState(() {
                      obscureText = !obscureText;
                    });
                  },
                  child: Icon(
                    obscureText ? Icons.visibility_off : Icons.visibility,
                    color: focusNode!.hasFocus
                        ? CustomColor.primaryColor
                        : Theme.of(context).primaryColor.withOpacity(0.7),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
