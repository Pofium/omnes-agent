import '../../../utils/dimensions.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../chat_controller.dart';
import '../../../helper/local_storage.dart';
import '../../../model/chat_model/chat_model.dart';
import '../../../utils/assets.dart';
import '../../../utils/custom_color.dart';
import '../../../widgets/markdown_preview_widget.dart';

class ChatMessageWidget extends StatelessWidget {
  final controller = Get.find<ChatController>();

  ChatMessageWidget({
    super.key,
    required this.text,
    required this.onSpeech,
    required this.onStop,
    required this.index,
    required this.onLongPress,
    required this.chatMessageType,
    this.message,
  });

  final String text;
  final int index;
  final ChatMessageType chatMessageType;
  final VoidCallback onSpeech, onStop, onLongPress;
  final ChatMessage? message;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      child: Row(
        mainAxisAlignment: chatMessageType == ChatMessageType.bot
            ? MainAxisAlignment.start
            : MainAxisAlignment.end,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _robotIcon(),
          _chatBody(context),
          _userIcon(),
        ],
      ),
    );
  }

  Widget _buildThinkingWidget(BuildContext context, String thinking) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.black.withOpacity(0.08),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.white.withOpacity(0.15)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.psychology_outlined, size: 16, color: Colors.amberAccent),
              const SizedBox(width: 6),
              Text(
                "Ход мыслей",
                style: TextStyle(
                  fontSize: Dimensions.defaultTextSize * 1.1,
                  fontWeight: FontWeight.bold,
                  color: Colors.amberAccent,
                ),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Text(
            thinking.trim(),
            style: TextStyle(
              fontSize: Dimensions.defaultTextSize * 1.2,
              fontStyle: FontStyle.italic,
              color: Colors.white.withOpacity(0.85),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildToolCallWidget(BuildContext context, ToolCallInfo tool) {
    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.teal.withOpacity(0.2),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: Colors.teal.withOpacity(0.4)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.build_circle_outlined, size: 14, color: Colors.tealAccent),
          const SizedBox(width: 6),
          Text(
            "Инструмент: ${tool.name}",
            style: TextStyle(
              fontSize: Dimensions.defaultTextSize * 1.1,
              color: Colors.tealAccent,
              fontWeight: FontWeight.w500,
            ),
          ),
          if (tool.output != null) ...[
            const SizedBox(width: 6),
            const Icon(Icons.check, size: 12, color: Colors.greenAccent),
          ],
        ],
      ),
    );
  }

  Widget _chatBody(BuildContext context) {
    final msg = message;
    final isStreaming = msg?.isStreaming ?? false;
    final thinking = msg?.thinking;
    final toolCalls = msg?.toolCalls ?? [];
    final displayText = text.isEmpty && isStreaming ? "..." : text;

    return Expanded(
      flex: 1,
      child: InkWell(
        onTap: onStop,
        onLongPress: onLongPress,
        child: Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: chatMessageType == ChatMessageType.bot
                ? Get.isDarkMode
                    ? CustomColor.primaryColor.withOpacity(0.2)
                    : CustomColor.primaryColor
                : Get.isDarkMode
                    ? Colors.white.withOpacity(0.5)
                    : Colors.black.withOpacity(0.2),
            borderRadius: BorderRadius.only(
              bottomRight: const Radius.circular(15),
              bottomLeft: const Radius.circular(15),
              topRight: Radius.circular(chatMessageType == ChatMessageType.user ? 0 : 15),
              topLeft: Radius.circular(chatMessageType == ChatMessageType.bot ? 0 : 15),
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (thinking != null && thinking.isNotEmpty)
                _buildThinkingWidget(context, thinking),
              if (toolCalls.isNotEmpty)
                ...toolCalls.map((t) => _buildToolCallWidget(context, t)),
              if (chatMessageType == ChatMessageType.bot && displayText != "...")
                MarkdownPreviewWidget(
                  text: displayText,
                  shrinkWrap: true,
                  textColor: CustomColor.whiteColor,
                )
              else
                SelectableText(
                  displayText,
                  style: TextStyle(
                    fontWeight: FontWeight.w400,
                    fontSize: Dimensions.defaultTextSize * 1.4,
                    color: chatMessageType == ChatMessageType.bot
                        ? CustomColor.whiteColor
                        : Theme.of(context).primaryColor,
                  ),
                ),
              if (isStreaming && text.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: SizedBox(
                    width: 12,
                    height: 12,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      valueColor: AlwaysStoppedAnimation<Color>(
                        chatMessageType == ChatMessageType.bot
                            ? Colors.white70
                            : CustomColor.primaryColor,
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _userIcon() {
    return Expanded(
      flex: 0,
      child: chatMessageType == ChatMessageType.user
          ? LocalStorage.isLoggedIn()
              ? Container(
                  margin: EdgeInsets.symmetric(horizontal: Dimensions.widthSize),
                  child: CircleAvatar(
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(Dimensions.radius * 5),
                      child: controller.userModel.imageUrl != ""
                          ? FadeInImage(
                              image: NetworkImage(controller.userModel.imageUrl),
                              placeholder: const AssetImage(Assets.smileSVG),
                              fit: BoxFit.fill,
                            )
                          : const CircleAvatar(
                              backgroundImage: AssetImage(Assets.menCartoon),
                            ),
                    ),
                  ),
                )
              : Container(
                  margin: EdgeInsets.symmetric(horizontal: Dimensions.widthSize),
                  child: Image.asset(Assets.smileSVG),
                )
          : IconButton(
              onPressed: onSpeech,
              icon: Obx(
                () => Icon(
                  Icons.record_voice_over_outlined,
                  color: controller.isSpeechLoading
                      ? controller.voiceSelectedIndex.value == index
                          ? CustomColor.primaryColor
                          : Get.isDarkMode
                              ? Colors.white24
                              : Colors.black26
                      : Get.isDarkMode
                          ? Colors.white24
                          : Colors.black26,
                ),
              ),
            ),
    );
  }

  Widget _robotIcon() {
    return Expanded(
      flex: 0,
      child: chatMessageType == ChatMessageType.bot
          ? Container(
              margin: EdgeInsets.symmetric(horizontal: Dimensions.widthSize),
              child: CircleAvatar(
                backgroundColor: Colors.transparent,
                child: Image.asset(
                  Assets.bot,
                ),
              ),
            )
          : SizedBox(width: Dimensions.widthSize * 5),
    );
  }
}
