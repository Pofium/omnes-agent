import 'package:flutter/material.dart';

import '../../../utils/custom_color.dart';
import '../../../utils/custom_style.dart';
import '../../../utils/dimensions.dart';

class SendInputField extends StatefulWidget {
  final TextEditingController controller;
  final String hintText;
  final VoidCallback? onTap, voiceTab, onAbort;
  final Widget icon;
  final bool isLoading;

  const SendInputField({
    super.key,
    required this.controller,
    required this.hintText,
    required this.icon,
    this.onTap,
    this.voiceTab,
    this.onAbort,
    this.isLoading = false,
  });

  @override
  State<SendInputField> createState() => _SendInputFieldState();
}

class _SendInputFieldState extends State<SendInputField> {
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
        horizontal: Dimensions.widthSize * 1.2,
        vertical: Dimensions.heightSize * 1,
      ),
      child: Row(
        children: [
          Expanded(
            child: Container(
              padding: EdgeInsets.only(
                left: Dimensions.widthSize * 1.2,
              ),
              decoration: BoxDecoration(
                border: Border.all(
                    color: focusNode!.hasFocus
                        ? CustomColor.primaryColor
                        : Theme.of(context).primaryColor.withOpacity(0.1),
                    width: 1
                ),
                color: Colors.transparent,
                borderRadius: BorderRadius.circular(Dimensions.radius * 0.7),
              ),
              child: TextFormField(
                controller: widget.controller,
                style: CustomStyle.primaryTextStyle.copyWith(fontSize: 15.0),
                textAlign: TextAlign.left,
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
                  hintText: widget.hintText,
                  hintStyle: TextStyle(
                    color: focusNode!.hasFocus
                        ? CustomColor.primaryColor.withOpacity(0.2)
                        : Theme.of(context).primaryColor.withOpacity(0.25),
                    fontSize: 15.0,
                    fontWeight: FontWeight.w400,
                  ),
                  enabledBorder: InputBorder.none,
                  focusedBorder: InputBorder.none,

                  suffixIcon: IconButton(
                     onPressed: widget.voiceTab,
                      icon: widget.icon
                  )
                ),

              ),
            ),
          ),
          const SizedBox(width: 10),
          Expanded(
            flex: 0,
            child: CircleAvatar(
              backgroundColor: widget.isLoading ? Colors.redAccent : CustomColor.primaryColor,
              child: InkWell(
                borderRadius: BorderRadius.circular(20),
                onTap: widget.isLoading ? widget.onAbort : widget.onTap,
                child: Padding(
                  padding: EdgeInsets.only(left: widget.isLoading ? 0 : 4.0),
                  child: Icon(
                    widget.isLoading ? Icons.stop : Icons.send,
                    color: CustomColor.whiteColor,
                    size: 20,
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
