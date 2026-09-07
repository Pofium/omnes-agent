import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';

import 'chat_controller.dart';
import '../../helper/local_storage.dart';
import '../../routes/routes.dart';
import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';
import '../../utils/strings.dart';
import '../../widgets/api/custom_loading_api.dart';
import '../../widgets/api/toast_message.dart';
import '../../widgets/appbar/appbar_widget.dart';
import 'widgets/chat_message_widget.dart';
import '../approvals/approvals_banner_widget.dart';
import 'widgets/send_input_field.dart';

class ChatScreen extends StatelessWidget {
  ChatScreen({super.key});

  final controller = Get.put(ChatController());

  @override
  Widget build(BuildContext context) {
    // ignore: deprecated_member_use
    return WillPopScope(
      onWillPop: () async {
        await controller.speechStopMethod();
        return true;
      },
      child: Scaffold(
        appBar: AppBarWidget(
          context: context,
          onBackClick: () {
            controller.speechStopMethod();
            Get.back();
          },
          appTitle: Strings.omnesAgent.tr,
          extraActions: [
            Obx(() => IconButton(
                  tooltip: controller.autoTts.value
                      ? 'Озвучка включена'
                      : 'Озвучка выключена',
                  icon: Icon(
                    controller.autoTts.value
                        ? Icons.volume_up
                        : Icons.volume_off_outlined,
                    color: controller.autoTts.value
                        ? CustomColor.primaryColor
                        : Theme.of(context).primaryColor.withOpacity(0.4),
                  ),
                  onPressed: () => controller.toggleAutoTts(),
                )),
          ],
          onPressed: () {
            _showDialog(context);
          },
        ),
        body: _mainBody(context),
      ),
    );
  }

  _mainBody(BuildContext context) {
    return Obx(
      () => Column(
        children: [
          const ApprovalsBannerWidget(),
          Expanded(
            flex: 5,
            child: controller.messages.value.isEmpty && !controller.isLoading
                ? _buildEmptyState(context)
                : _buildList(),
          ),
          Expanded(
            flex: 0,
            child: Obx(() => Visibility(
                visible: controller.isLoading,
                child: const CustomLoadingAPI())),
          ),
          Expanded(flex: 0, child: _submitButton(context)),
          SizedBox(height: Dimensions.heightSize)
        ],
      ),
    );
  }

  _submitButton(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        children: [
          _suggestedWidget(context),
          SendInputField(
            isLoading: controller.isLoading,
            onAbort: () => controller.abortGeneration(),
            icon: Icon(
              controller.isListening.value ? Icons.mic : Icons.mic_none_sharp,
              color: controller.isListening.value
                  ? Colors.redAccent
                  : Theme.of(context).primaryColor.withOpacity(0.5),
            ),
            hintText: controller.isListening.value
                ? "Слушаю речь..."
                : Strings.typeYour.tr,
            onTap: () {
              if (controller.chatController.text.trim().isNotEmpty) {
                controller.proccessChat();
                Future.delayed(const Duration(milliseconds: 50))
                    .then((_) => controller.scrollDown());
              } else {
                ToastMessage.error(Strings.writeSomething.tr);
              }
            },
            voiceTab: () {
              controller.listen(context);
            },
            controller: controller.chatController,
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    final alias = controller.currentAgentAlias ?? "chief";
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 20),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: CustomColor.primaryColor.withOpacity(0.12),
              ),
              child: const Icon(
                Icons.smart_toy_outlined,
                size: 52,
                color: CustomColor.primaryColor,
              ),
            ),
            const SizedBox(height: 16),
            Text(
              "OmnesAgent • $alias",
              style: TextStyle(
                fontSize: Dimensions.defaultTextSize * 1.8,
                fontWeight: FontWeight.bold,
                color: Theme.of(context).primaryColor,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              "Готов к работе. Вы можете написать сообщение или воспользоваться голосовым вводом.",
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: Dimensions.defaultTextSize * 1.3,
                color: Theme.of(context).primaryColor.withOpacity(0.6),
              ),
            ),
            const SizedBox(height: 24),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              alignment: WrapAlignment.center,
              children: [
                _buildPromptChip(context, "Создай план проекта"),
                _buildPromptChip(context, "Проанализируй файлы в воркспейсе"),
                _buildPromptChip(context, "Напиши скрипт автоматизации"),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPromptChip(BuildContext context, String promptText) {
    return ActionChip(
      avatar: const Icon(Icons.bolt, size: 16, color: CustomColor.primaryColor),
      label: Text(
        promptText,
        style: TextStyle(
          fontSize: Dimensions.defaultTextSize * 1.2,
          color: Theme.of(context).primaryColor,
        ),
      ),
      backgroundColor: Theme.of(context).cardColor,
      side: BorderSide(color: CustomColor.primaryColor.withOpacity(0.25)),
      onPressed: () {
        controller.chatController.text = promptText;
        controller.proccessChat();
      },
    );
  }

  _suggestedWidget(BuildContext context) {
    return Obx(() => controller.isLoading2.value
        ? const CustomLoadingAPI()
        : SizedBox(
            height: 28,
            width: double.infinity,
            child: ListView.separated(
                padding: EdgeInsets.symmetric(horizontal: Dimensions.widthSize * 1),
                shrinkWrap: true,
                physics: const BouncingScrollPhysics(),
                scrollDirection: Axis.horizontal,
                itemBuilder: (context, index) {
                  final prompt = controller.suggestedData[index];
                  return InkWell(
                    borderRadius: BorderRadius.circular(Dimensions.radius * .7),
                    onTap: () {
                      controller.chatController.text = prompt.prompt;
                    },
                    child: Container(
                      alignment: Alignment.center,
                      padding: const EdgeInsets.symmetric(horizontal: 10),
                      decoration: BoxDecoration(
                          borderRadius:
                              BorderRadius.circular(Dimensions.radius * .7),
                          color: Get.isDarkMode
                              ? CustomColor.primaryColor.withOpacity(0.2)
                              : CustomColor.primaryColor
                      ),
                      child: Text(
                        prompt.title.isNotEmpty ? prompt.title : prompt.category,
                        style: const TextStyle(color: CustomColor.whiteColor, fontSize: 12),
                      ),
                    ),
                  );
                },
                separatorBuilder: (_, i) => const SizedBox(width: 6),
                itemCount: controller.suggestedData.length),
          ));
  }

  void openCustomBottomSheet(BuildContext context, data) {
    showModalBottomSheet(
      context: context,

      builder: (BuildContext context) {
        return Container(
          color: Theme.of(context).scaffoldBackgroundColor,
          padding: EdgeInsets.all(Dimensions.widthSize),
           child: Stack(
             children: [
               Column(
                // mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  const Text(
                    "SUGGESTED QUESTIONS",
                    style: TextStyle(
                      // fontSize: 14.sp,
                      color: CustomColor.primaryColor,
                      fontWeight: FontWeight.bold
                    ),
                  ),

                  const Divider(),

                  Expanded(
                    child: ListView.builder(
                      shrinkWrap: true,
                      physics: const BouncingScrollPhysics(),
                      itemCount: data["questions"].length,
                        itemBuilder: (context, index){
                          return ListTile(
                            title: Text(
                                data["questions"][index],
                              style: TextStyle(
                                color: Theme.of(context).primaryColor.withOpacity(.6)
                              ),
                            ),
                            onTap: () {
                              controller.chatController.text = "";
                              controller.chatController.text = data["questions"][index];
                              Navigator.pop(context);
                            },
                          );
                        }
                    ),
                  )
                ],
          ),
               Positioned(
                 right: -5,
                   top: -15,
                   child: IconButton(
                     icon: const Icon(
                         Icons.close,
                       color: Colors.red,
                     ),
                     onPressed: (){
                       Get.back();
                     },
                   )
               )
             ],
           ),
        );
      },
    );
  }
  _buildList() {
    var languageList = LocalStorage.getLanguage();
    return Obx(() => ListView.builder(
          controller: controller.scrollController,
          itemCount: controller.itemCount.value,
          shrinkWrap: true,
          physics: const BouncingScrollPhysics(),
          itemBuilder: (context, index) {
            return ChatMessageWidget(
                message: controller.messages.value[index],
                onStop: () {
                  controller.speechStopMethod();
                },
                onLongPress: () {
                  Clipboard.setData(ClipboardData(
                      text: controller.messages.value[index].text));
                },
                onSpeech: () {
                  controller.speechMethod(controller.messages.value[index].text,
                      '${languageList[0]}-${languageList[1]}');
                  controller.voiceSelectedIndex.value = index;
                },
                text: controller.messages.value[index].text,
                chatMessageType:
                    controller.messages.value[index].chatMessageType,
                index: index);
          },
        ));
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
              mainAxisSize: MainAxisSize.min,
              mainAxisAlignment: MainAxisAlignment.start,
              crossAxisAlignment: CrossAxisAlignment.end,
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
                              if (LocalStorage.isFreeUser()) {
                                // FacebookAdHelper.initAd();
                              }

                              if (index == 0) {
                                if (controller.textInput.value.isNotEmpty) {
                                  controller.proccessChat2();
                                }
                                Get.back();
                              } else if (index == 1) {
                                controller.clearConversation();
                                Get.back();
                              } else if (index == 2) {
                                if (controller.shareMessages.isEmpty) {
                                  Get.snackbar(
                                      'OH No!!', 'No Conversation yet');
                                  Get.back();
                                } else {
                                  controller.shareChat(context);
                                }
                                Get.back();
                              } else if (index == 3) {
                                Get.back();
                                Get.toNamed(Routes.settingsScreen);
                              }
                            },
                            child: Text(
                              controller.moreList[index],
                              style: const TextStyle(
                                  color: CustomColor.blackColor),
                            )),
                      )),
            ),
          );
        });
  }
}