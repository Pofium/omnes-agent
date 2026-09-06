import 'package:flutter/material.dart';

import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';

class InputField extends StatefulWidget {
  final TextEditingController controller;
  final String hintText;
  final bool readOnly;
  final TextInputType keyboardType;
  final int maxLines;
  final VoidCallback? onTap;

  const InputField(
      {super.key,
        required this.controller,
        required this.hintText,
        this.readOnly = false,
        this.keyboardType = TextInputType.text,
        this.maxLines = 1,
        this.onTap});

  @override
  State<InputField> createState() => _PrimaryInputFieldState();
}

class _PrimaryInputFieldState extends State<InputField> {
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
    return Container(
      margin: EdgeInsets.symmetric(vertical: Dimensions.heightSize * 0.4),
      child: TextFormField(
        readOnly: widget.readOnly,
        controller: widget.controller,
        keyboardType: widget.keyboardType,
        maxLines: widget.maxLines,
        style: TextStyle(
          color: Theme.of(context).primaryColor.withOpacity(1),
          fontSize: Dimensions.defaultTextSize * 1.6,
          fontWeight: FontWeight.w600,
        ),
        onTap: widget.readOnly
            ? widget.onTap
            : () {
          setState(() {
            focusNode!.requestFocus();
          });
        },
        onFieldSubmitted: (value) {
          setState(() {
            focusNode!.unfocus();
          });
        },
        validator: (value) {
          if (value == null || value.isEmpty) {
            return 'Please enter some text';
          }
          return null;
        },
        focusNode: focusNode,
        decoration: InputDecoration(
            hintText: widget.hintText,
            hintStyle: TextStyle(
              color: Theme.of(context).primaryColor.withOpacity(0.1),
              fontSize: Dimensions.defaultTextSize * 1.5,
              fontWeight: FontWeight.w500,
            ),
            enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(Dimensions.radius * .8),
                borderSide: BorderSide(
                  color: focusNode!.hasFocus
                      ? CustomColor.primaryColor
                      : Theme.of(context).primaryColor.withOpacity(0.4),
                )
            ),
            focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(Dimensions.radius * .8),
                borderSide: BorderSide(
                  color: focusNode!.hasFocus
                      ? CustomColor.primaryColor
                      : Theme.of(context).primaryColor.withOpacity(0.4),
                )
            ),
            errorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(Dimensions.radius * .8),
                borderSide: const BorderSide(
                  color: Colors.red,
                )
            ),
            focusedErrorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(Dimensions.radius * .8),
                borderSide: const BorderSide(
                  color: Colors.red,
                )
            ),

            contentPadding: const EdgeInsets.symmetric(
                horizontal: 10,
                vertical: 5
            ),

            suffixIcon: Visibility(
              visible: widget.readOnly,
              child: Icon(
                widget.onTap == null
                    ? Icons.check_circle_outline
                    : Icons.calendar_month_outlined,
                color: widget.onTap == null
                    ? Colors.green
                    : Theme.of(context).primaryColor.withOpacity(1),
              ),
            )

          // contentPadding: EdgeInsets.zero
        ),
      ),
    );
  }
}
