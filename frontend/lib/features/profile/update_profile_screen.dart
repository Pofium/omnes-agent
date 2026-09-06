import 'dart:io';

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:image_picker/image_picker.dart';

import 'update_profile_controller.dart';
import '../../utils/assets.dart';
import '../../utils/dimensions.dart';
import '../../utils/strings.dart';
import '../../widgets/api/custom_loading_api.dart';
import '../../widgets/api/toast_message.dart';
import '../../widgets/appbar/appbar_widget.dart';
import '../../widgets/inputs_widgets/input_field.dart';

class UpdateProfileScreen extends StatelessWidget {
  UpdateProfileScreen({super.key});

  final controller = Get.put(UpdateProfileController());

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBarWidget(
        context: context,
        appTitle: Strings.updateProfile.tr,
        onPressed: () {},
        moreVisible: false,
        onBackClick: () {
          Get.back();
        },
      ),
      body: _inputFields(context),
    );
  }

  _inputFields(BuildContext context) {
    return Obx(
      () => controller.isLoading
          ? const CustomLoadingAPI()
          : ListView(
              padding: EdgeInsets.symmetric(
                  horizontal: Dimensions.widthSize,
                  vertical: Dimensions.heightSize),
              physics: const BouncingScrollPhysics(),
              children: [
                _imageWidget(context),


                SizedBox(height: Dimensions.heightSize * 1.8),

                Text(
                  Strings.fullName.tr,
                  style: TextStyle(
                      fontSize: Dimensions.mediumTextSize * .8,
                      fontWeight: FontWeight.w600,
                      color: Theme.of(context).primaryColor
                  ),
                ),
                InputField(
                  controller: controller.nameController,
                  hintText: Strings.fullName.tr,
                ),


                Visibility(
                  visible: controller.isEmailHave.value,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      SizedBox(height: Dimensions.heightSize * 1.6),

                      Text(
                        Strings.emailAddress.tr,
                        style: TextStyle(
                            fontSize: Dimensions.mediumTextSize * .8,
                            fontWeight: FontWeight.w600,
                            color: Theme.of(context).primaryColor
                        ),
                      ),
                      InputField(
                            readOnly: true,
                            controller: controller.emailController,
                            hintText: Strings.emailAddress.tr,
                          ),
                    ],
                  ),
                ),


                SizedBox(height: Dimensions.heightSize * 1.6),


                Text(
                  Strings.mobileNumber.tr,
                  style: TextStyle(
                      fontSize: Dimensions.mediumTextSize * .8,
                      fontWeight: FontWeight.w600,
                      color: Theme.of(context).primaryColor
                  ),
                ),

                InputField(
                  controller: controller.numberController,
                  hintText: Strings.mobileNumber.tr,
                ),


                SizedBox(height: Dimensions.heightSize * 2.5),

                SizedBox(
                  height: Dimensions.buttonHeight * .8,
                  child: ElevatedButton(
                    onPressed: () {
                      if(controller.imageSelected.value){
                        controller.updateUserDataWithImage();
                      }else{
                        controller.updateUserData();
                      }

                    },
                    child: Text(Strings.updateProfile.tr),
                  ),
                ),
              ],
            ),
    );
  }

  _imageWidget(BuildContext context) {
    return InkWell(
      onTap: (){
        debugPrint('Select');
        pickImageFromGallery(context);
      },
      child: Container(
        alignment: Alignment.center,
        height: Dimensions.buttonHeight * 2,
        width: Dimensions.buttonHeight * 3,
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(Dimensions.radius * 20)
        ),
        child: CircleAvatar(
          radius: 80,
          backgroundImage: controller.imageSelected.value
              ? FileImage(
            File(
              controller.filePathString.value,
            ),
          )
              : controller.userModel.imageUrl == ''
                ? const AssetImage(Assets.menCartoon)
                : NetworkImage(
                    controller.userModel.imageUrl,
                  )
          as ImageProvider,
        ),
      ),
    );
  }


  Future<File?> pickImageFromGallery(BuildContext context) async {
    File? image;
    try {
      debugPrint('Start');

      final pickedImage = await ImagePicker().pickImage(source: ImageSource.gallery);
      if (pickedImage != null) {
        image = File(pickedImage.path);

        controller.imageSelected.value = true;
        controller.file = image;
        controller.filePathString.value = image.path;
        debugPrint('Selected');

        ToastMessage.success('Image Selected');
      }

    } catch (e) {
      debugPrint(e.toString());
      ToastMessage.error(e.toString());
    }
    return image;
  }

}
