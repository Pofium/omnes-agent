import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../utils/dimensions.dart';

class InputDropDown extends StatelessWidget {
  final RxString selectMethod;
  final List<String> itemsList;
  final void Function(String?)? onChanged;

  const InputDropDown({
    required this.itemsList,
    super.key,
    required this.selectMethod,
    this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Obx(() => Container(
      height: 52,
      margin: EdgeInsets.symmetric(
        horizontal: Dimensions.widthSize * .5,
      ),
      decoration: BoxDecoration(
        border: Border.all(
          color: Theme.of(context).primaryColor.withOpacity(0.3),
          width: 2,
        ),
        borderRadius: BorderRadius.circular(Dimensions.radius * 0.5),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton(
          hint: Padding(
            padding: EdgeInsets.only(left: Dimensions.defaultPaddingSize * 0.7),
            child: Text(
              selectMethod.value,
              style: TextStyle(
                  fontSize: Dimensions.mediumTextSize * .8,
                  fontWeight: FontWeight.w600,
                  color: Theme.of(context).primaryColor.withOpacity(.8)
              ),
            ),
          ),
          icon: Padding(
            padding: const EdgeInsets.only(right: 4),
            child: Icon(
              Icons.arrow_drop_down,
              color: Theme.of(context).primaryColor,
            ),
          ),
          isExpanded: true,
          underline: Container(),
          borderRadius: BorderRadius.circular(Dimensions.radius * .5),
          items: itemsList.map<DropdownMenuItem<String>>((value) {
            return DropdownMenuItem<String>(
              value: value.toString(),
              child: Text(value.toString(),
                  style: TextStyle(
                    color: Theme.of(context).primaryColor
                  )
              ),
            );
          }).toList(),
          onChanged: onChanged,
        ),
      ),
    ));
  }
}