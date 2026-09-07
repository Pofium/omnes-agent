import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../helper/local_storage.dart';
import '../../helper/unity_ad.dart';
import '../../routes/routes.dart';
import '../../utils/Flutter Theam/themes.dart';
import '../../utils/assets.dart';
import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';
import '../../utils/strings.dart';
import '../agents/agent_picker_dialog.dart';
import '../support/support_screen.dart';
import '../../design_system/design_system.dart';
import 'home_controller.dart';

class DrawerWidget extends StatefulWidget {
  const DrawerWidget({super.key, required this.isDark});

  // ignore: prefer_typing_uninitialized_variables
  final isDark;

  @override
  State<DrawerWidget> createState() => _DrawerWidgetState();
}

class _DrawerWidgetState extends State<DrawerWidget> {
  final controller = Get.put(HomeController());

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((timeStamp) async {
      await AdManager.loadUnityIntAd();
      await AdManager.loadUnityRewardedAd();
    });
  }

  @override
  Widget build(BuildContext context) {
    return _drawerWidget(context, widget.isDark);
  }

  _drawerWidget(BuildContext context, RxBool isDark) {
    return Drawer(
      backgroundColor: Theme.of(context).scaffoldBackgroundColor,
      width: MediaQuery.of(context).size.width * .7,
      shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.only(
        topRight: Radius.circular(Dimensions.radius * 5),
      )),
      child: SafeArea(
        child: Column(
          children: [
            SizedBox(height: Dimensions.heightSize),
            SizedBox(
              width: MediaQuery.of(context).size.width * .6,
              height: Dimensions.buttonHeight * 2,
              child: _drawerIconAndTitle(),
            ),
            const Padding(
              padding: EdgeInsets.symmetric(horizontal: 16.0, vertical: 8.0),
              child: Divider(
                color: ShadcnColors.border,
                height: 1,
                thickness: 1,
              ),
            ),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.symmetric(horizontal: 8),
                children: [
                  _drawerListTileWidget(context,
                      text: Strings.sessionHistory.tr,
                      icon: FontAwesomeIcons.comments, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.sessionsScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.projectsAndFiles.tr,
                      icon: FontAwesomeIcons.folderTree, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.projectsScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.selectAgent.tr,
                      icon: FontAwesomeIcons.robot, onTap: () {
                    Get.back();
                    AgentPickerDialog.show(context);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.agentMemory.tr,
                      icon: FontAwesomeIcons.brain, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.memoryScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.cronTasks.tr,
                      icon: FontAwesomeIcons.clock, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.cronScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.toolsAndSkills.tr,
                      icon: FontAwesomeIcons.wrench, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.toolsScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: "Навыки (Skills)",
                      icon: FontAwesomeIcons.wandMagicSparkles, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.skillsScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.systemStatus.tr,
                      icon: FontAwesomeIcons.stethoscope, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.doctorScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: "Логи сервера",
                      icon: FontAwesomeIcons.receipt, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.logsScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: "Каналы и интеграции",
                      icon: FontAwesomeIcons.networkWired, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.integrationsScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.costAndTokens.tr,
                      icon: FontAwesomeIcons.chartPie, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.statsScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: "Сопряжение (Pairing)",
                      icon: FontAwesomeIcons.qrcode, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.pairingScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.llmProviders.tr,
                      icon: FontAwesomeIcons.microchip, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.llmProvidersScreen);
                  }),
                  _drawerListTileWidget(context,
                      text: Strings.settings.tr,
                      icon: FontAwesomeIcons.gear, onTap: () {
                    Get.back();
                    Get.toNamed(Routes.settingsScreen);
                  }),
                  Visibility(
                    visible: false,
                    child: _drawerListTileWidget(context,
                        text: Strings.logIn.tr,
                        icon: FontAwesomeIcons.rightFromBracket, onTap: () {
                      Get.offAllNamed(Routes.loginScreen);
                    }),
                  ),
                  Visibility(
                    visible: LocalStorage.isLoggedIn(),
                    child: _drawerListTileWidget(context,
                        text: Strings.updateProfile.tr,
                        icon: FontAwesomeIcons.user, onTap: () async {
                      Get.toNamed(Routes.updateProfileScreen);
                    }),
                  ),
                  _drawerListTileWidget(context,
                      text: Strings.supportAndFeedback.tr,
                      icon: FontAwesomeIcons.penToSquare, onTap: () {
                    Get.back();
                    showModalBottomSheet(
                        shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.only(
                          topLeft: Radius.circular(Dimensions.radius * 2),
                          topRight: Radius.circular(Dimensions.radius * 2),
                        )),
                        backgroundColor: ShadcnColors.card,
                        context: context,
                        isScrollControlled: true,
                        builder: (context) {
                          return const SupportFieldWidget();
                        });
                  }),
                  Obx(() => _themeChangeListTileWidget(context,
                      text: Strings.themeChange.tr,
                      icon: isDark.value
                          ? FontAwesomeIcons.sun
                          : FontAwesomeIcons.moon)),
                  Visibility(
                    visible: LocalStorage.isLoggedIn(),
                    child: _drawerListTileWidget(context,
                        text: Strings.logOut.tr,
                        icon: FontAwesomeIcons.rightFromBracket, onTap: () {
                      controller.logout();
                    }),
                  ),
                ],
              ),
            ),
            Visibility(
              visible: LocalStorage.isLoggedIn(),
              child: _drawerListTileWidget(context,
                  text: Strings.logOut.tr,
                  icon: FontAwesomeIcons.rightFromBracket, onTap: () {
                controller.logout();
              }),
            ),
            const Spacer(),
            GestureDetector(
              onTap: () {
                _showDialog(context);
              },
              child: Container(
                padding: EdgeInsets.symmetric(
                    horizontal: Dimensions.widthSize * 2,
                    vertical: Dimensions.heightSize * 0.7),
                width: MediaQuery.of(context).size.width * 0.5,
                margin: EdgeInsets.symmetric(
                    vertical: Dimensions.heightSize * 2, horizontal: 5),
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  color: (isDark.value
                          ? CustomColor.whiteColor
                          : CustomColor.primaryColor)
                      .withOpacity(0.05),
                  borderRadius: BorderRadius.circular(Dimensions.radius * 2),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.center,
                  mainAxisSize: MainAxisSize.max,
                  children: [
                    Obx(() => Text(
                          controller.selectedLanguage.value.tr,
                          style: TextStyle(
                              fontSize: Dimensions.defaultTextSize * 1.8,
                              fontWeight: FontWeight.w500,
                              color: (isDark.value
                                  ? CustomColor.whiteColor
                                  : CustomColor.primaryColor)),
                        )),
                    SizedBox(width: Dimensions.widthSize * .7),
                    Icon(
                      Icons.arrow_drop_down,
                      color: (isDark.value
                          ? CustomColor.whiteColor
                          : CustomColor.primaryColor),
                    )
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  _drawerListTileWidget(BuildContext context,
      {required String text,
      required VoidCallback onTap,
      required IconData icon}) {
    return ListTile(
      dense: false,
      onTap: onTap,
      leading: Icon(icon),
      title: Text(
        text,
        style: TextStyle(color: Theme.of(context).primaryColor),
      ),
    );
  }

  _themeChangeListTileWidget(BuildContext context,
      {required String text, required IconData icon}) {
    return ListTile(
      dense: false,
      leading: Icon(icon),
      title: Text(
        text,
        style: TextStyle(color: Theme.of(context).primaryColor),
      ),
      trailing: Obx(() => Switch(
            activeColor: CustomColor.primaryColor,
            onChanged: (value) {
              Themes().switchTheme();
              widget.isDark.value = !widget.isDark.value;
            },
            value: widget.isDark.value,
          )),
    );
  }

  _drawerIconAndTitle() {
    return FittedBox(
      child: Center(
        child: Image.asset(
          Assets.bot,
          scale: 6,
        ),
      ),
    );
  }

  _showDialog(BuildContext context) {
    showDialog(
        context: context,
        barrierDismissible: true,
        builder: (context) {
          return Padding(
            padding: EdgeInsets.symmetric(
                horizontal: Dimensions.widthSize * 3,
                vertical: Dimensions.heightSize),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.center,
              mainAxisAlignment: MainAxisAlignment.center,
              mainAxisSize: MainAxisSize.min,
              children: List.generate(
                  controller.moreList.length,
                  (index) => Container(
                        alignment: Alignment.centerLeft,
                        color: Colors.white,
                        width: MediaQuery.of(context).size.width * 0.5,
                        padding: EdgeInsets.symmetric(
                            horizontal: Dimensions.widthSize * 1,
                            vertical: Dimensions.heightSize * 0.5),
                        child: TextButton(
                            onPressed: () {
                              controller.onChangeLanguage(
                                  controller.moreList[index], index);
                              Get.back();
                            },
                            child: Text(
                              controller.moreList[index],
                              style: TextStyle(
                                  color: controller.selectedLanguage.value ==
                                          controller.moreList[index]
                                      ? CustomColor.primaryColor
                                      : CustomColor.blackColor),
                            )),
                      )),
            ),
          );
        });
  }
}
