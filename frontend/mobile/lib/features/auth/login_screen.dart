import '../../utils/custom_style.dart';
import '../../widgets/api/toast_message.dart';
import '../home/main_controller.dart';
import '../../services/apple_sign_in/apple_sign_in_available.dart';
import '../../widgets/api/custom_loading_api.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

import 'login_controller.dart';
import '../../utils/assets.dart';
import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';
import '../../utils/strings.dart';
import '../../widgets/app_name_widget.dart';
import '../../widgets/buttons/button.dart';
import '../../widgets/inputs_widgets/password_input_widget.dart';
import '../../widgets/inputs_widgets/secondary_input_field.dart';

class LogInScreen extends StatelessWidget {
  LogInScreen({super.key});

  final controller = Get.put(LoginController());

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        scrolledUnderElevation: 0,
        elevation: 0,
        backgroundColor: Colors.transparent,
        actions: [
          IconButton(
            onPressed: () {
              showModalBottomSheet(
                  shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.only(
                        topLeft: Radius.circular(Dimensions.radius * 2),
                        topRight: Radius.circular(Dimensions.radius * 2),
                      )),
                  backgroundColor: Theme.of(context).scaffoldBackgroundColor,
                  context: context,
                  isScrollControlled: true,
                  builder: (context) {
                    return _supportField(context);
                  });
            },
            icon: Icon(
              Icons.help_outline_outlined,
              color: Theme.of(context).primaryColor,
            ),
          )
        ],
      ),
      body: Obx(() => controller.isLoading
          ? const CustomLoadingAPI()
          : _bodyWidget(context)),
    );
  }

  _supportField(BuildContext context) {
    return Padding(
      padding: EdgeInsets.only(
        bottom: MediaQuery.of(context).viewInsets.bottom,
        top: Dimensions.heightSize * 2,
        left: Dimensions.widthSize * 2,
        right: Dimensions.widthSize * 2,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(
            Strings.faceAnyProblem.tr,
            style: CustomStyle.primaryTextStyle
                .copyWith(fontSize: Dimensions.defaultTextSize * 2),
          ),
          SecondaryInputField(
            controller: controller.nameController,
            hintText: Strings.enterYourName.tr,
          ),
          SecondaryInputField(
            controller: controller.userEmailController,
            hintText: Strings.enterYourEmail.tr,
          ),
          SecondaryInputField(
            controller: controller.noteController,
            hintText: Strings.describeYourIssue.tr,
            maxLine: 3,
          ),
          GestureDetector(
            onTap: () {
              if (controller.nameController.text.isNotEmpty &&
                  controller.userEmailController.text.isNotEmpty &&
                  controller.noteController.text.isNotEmpty) {
                Get.back();
                MainController.sendSupportTicket(
                    name: controller.nameController.text,
                    email: controller.userEmailController.text,
                    note: controller.noteController.text);
              } else {
                ToastMessage.error('!! Fill the Form');
              }
            },
            child: Card(
              color: CustomColor.primaryColor,
              shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(Dimensions.radius * 2)),
              child: Container(
                alignment: Alignment.center,
                padding: EdgeInsets.symmetric(vertical: Dimensions.heightSize),
                child: const Icon(
                  Icons.send_outlined,
                  color: CustomColor.whiteColor,
                ),
              ),
            ),
          ),
          SizedBox(
            height: Dimensions.heightSize,
          )
        ],
      ),
    );
  }

  _animatedTextWidget() {
    return const Padding(
        padding: EdgeInsets.only(top: 18.0), child: AppNameWidget());
  }

  _bodyWidget(BuildContext context) {
    return ListView(
      physics: const BouncingScrollPhysics(),
      // mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        SizedBox(height: Dimensions.buttonHeight),
        Row(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.center,
          mainAxisAlignment: MainAxisAlignment.center,
          children: [_animatedTextWidget()],
        ),
        SizedBox(height: Dimensions.buttonHeight),
        inputWidgetAndButtons(context),
        SizedBox(height: Dimensions.buttonHeight * .1),
        Align(
          alignment: Alignment.bottomRight,
          child: Padding(
            padding: EdgeInsets.symmetric(
                horizontal: Dimensions.defaultPaddingSize * .5),
            child: TextButton(
                onPressed: () => controller.forgotPassword(context),
                child: Text(Strings.forgotPassword.tr)),
          ),
        ),
        SizedBox(height: Dimensions.buttonHeight * .6),
        footerWidget(context),
      ],
    );
  }

  inputWidgetAndButtons(BuildContext context) {
    return Padding(
      padding:
      EdgeInsets.symmetric(horizontal: Dimensions.defaultPaddingSize * .5),
      child: Column(
        children: [
          SecondaryInputField(
            controller: controller.emailController,
            hintText: Strings.enterYourEmail,
          ),
          SizedBox(
            height: Dimensions.defaultPaddingSize * .2,
          ),
          PasswordInputWidget(
            controller: controller.passwordController,
            hint: Strings.enterYourPassword,
          ),
          SizedBox(
            height: Dimensions.defaultPaddingSize * .6,
          ),
          Row(
            children: [
              Expanded(
                child: Button(
                  onClick: () => controller.createBTN(context),
                  child: Text(
                    Strings.create.tr,
                    style: const TextStyle(color: CustomColor.whiteColor,
                        fontWeight: FontWeight.bold
                    ),
                  ),
                ),
              ),
              SizedBox(width: Dimensions.widthSize),
              Expanded(
                child: Button(
                  onClick: () => controller.loginBTN(context),
                  child: Text(Strings.logIn.tr,
                    style: const TextStyle(
                        color: CustomColor.whiteColor,
                        fontWeight: FontWeight.bold
                    ),
                  ),
                ),
              ),
            ],
          )
        ],
      ),
    );
  }

  footerWidget(BuildContext context) {
    return Column(
      children: [
        googleAndAppleLogin(context),
        SizedBox(height: Dimensions.heightSize),
        TextButton(
            onPressed: () {
              controller.goToHomePage();
            },
            child: Text(
              Strings.continueAsGuest.tr,
              style: const TextStyle(
                color: CustomColor.primaryColor,
              ),
            )),
      ],
    );
  }

  googleAndAppleLogin(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Container(
          padding: EdgeInsets.all(Dimensions.defaultPaddingSize * 0.5),
          margin: EdgeInsets.symmetric(
            horizontal: Dimensions.widthSize * 3,
          ),
          decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(Dimensions.radius),
              border: Border.all(color: CustomColor.primaryColor, width: 1)),
          child: InkWell(
            onTap: () {
              controller.signInWithGoogle(context);
            },
            borderRadius: BorderRadius.circular(Dimensions.radius),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.center,
              mainAxisSize: MainAxisSize.min,
              children: [
                Image.asset(Assets.google),
              ],
            ),
          ),
        ),
        SizedBox(height: Dimensions.heightSize),
        Visibility(
          visible: appleSignInAvailable.isAvailable.value,
          child: Container(
            padding: EdgeInsets.all(Dimensions.defaultPaddingSize * 0.5),
            margin: EdgeInsets.symmetric(
              horizontal: Dimensions.widthSize * 3,
            ),
            decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(Dimensions.radius),
                border: Border.all(color: CustomColor.primaryColor, width: 1)),
            child: InkWell(
              onTap: () {
                controller.signInWithApple(context);
              },
              borderRadius: BorderRadius.circular(Dimensions.radius),
              child: const Row(
                mainAxisSize: MainAxisSize.min,
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  Icon(Icons.apple),
                ],
              ),
            ),
          ),
        )
      ],
    );
  }
}
