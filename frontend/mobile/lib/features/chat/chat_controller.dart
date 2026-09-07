import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_tts/flutter_tts.dart';
import 'package:get/get.dart';
import 'package:share_plus/share_plus.dart';
import 'package:speech_to_text/speech_to_text.dart' as stt;

import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import '../../core/gateway/gateway_ws.dart';
import '../../core/gateway/models/gateway_frame.dart';
import '../../core/supabase/suggested_prompts_repository.dart';
import '../../core/supabase/supabase_config.dart';
import '../../helper/local_storage.dart';
import '../../helper/speech_helper.dart';
import '../../model/chat_model/chat_model.dart';
import '../../model/user_model/user_model.dart';
import '../../utils/strings.dart';

class ChatController extends GetxController {
  Timer? timer;
  List<SuggestedPrompt> suggestedData = [];

  GatewayWsClient? _gatewayWs;
  StreamSubscription<GatewayFrame>? _gatewaySub;
  final GatewayHttpClient _gatewayHttp = GatewayHttpClient();

  String? currentSessionId;
  String? currentAgentAlias;
  String? currentWorkspaceDir;

  final chatController = TextEditingController();
  final scrollController = ScrollController();

  Rx<List<ChatMessage>> messages = Rx<List<ChatMessage>>([]);
  List<String> shareMessages = [
    '--THIS IS CONVERSATION with ${Strings.appName}--\n\n'
  ];
  RxInt itemCount = 0.obs;
  RxInt voiceSelectedIndex = 0.obs;
  final RxBool autoTts = false.obs;

  final _isLoading = false.obs;
  bool get isLoading => _isLoading.value;
  RxBool isLoading2 = false.obs;

  late UserModel userModel;

  final List<String> moreList = [
    Strings.regenerateResponse.tr,
    Strings.clearConversation.tr,
    Strings.share.tr,
    Strings.changeTextModel.tr,
  ];

  @override
  void onInit() {
    getSuggestedCategory();
    _initGateway();

    autoTts.value = LocalStorage.getAutoTts();
    speech = stt.SpeechToText();
    LocalStorage.isLoggedIn() ? _getUserData() : _setGesutUser();

    count.value = LocalStorage.getTextCount();

    super.onInit();
  }

  @override
  void onClose() {
    _gatewaySub?.cancel();
    _gatewayWs?.dispose();
    _gatewayHttp.dispose();
    super.onClose();
  }

  void _initGateway() async {
    if (Get.arguments is Map) {
      currentSessionId = Get.arguments['sessionId'] as String?;
      currentAgentAlias = Get.arguments['agentAlias'] as String?;
      currentWorkspaceDir = Get.arguments['workspaceDir'] as String?;
    }

    final sid = currentSessionId ?? "chat_${DateTime.now().millisecondsSinceEpoch}";
    currentSessionId = sid;
    final alias = currentAgentAlias ?? GatewayConfig.getAgentAlias();
    currentAgentAlias = alias;

    final isNew = (Get.arguments is Map) ? (Get.arguments['isNew'] == true) : false;
    if (!isNew && currentSessionId != null) {
      await _loadSessionHistory(sid);
    }

    _gatewayWs = GatewayWsClient(
      sessionId: sid,
      agentAlias: alias,
      workspaceDir: currentWorkspaceDir,
    );
    _gatewayWs!.connect();
    _gatewaySub = _gatewayWs!.stream.listen(_handleGatewayFrame);
  }

  Future<void> _loadSessionHistory(String sid) async {
    try {
      final history = await _gatewayHttp.getSessionMessages(sid);
      if (history.isNotEmpty) {
        messages.value.clear();
        shareMessages.clear();
        shareMessages.add('--THIS IS CONVERSATION with ${Strings.appName}--\n\n');

        for (final m in history) {
          if (m.role == 'system') continue;
          final type = m.role == 'user' ? ChatMessageType.user : ChatMessageType.bot;
          String text = m.content;
          if (text.startsWith('[CURRENT DATE & TIME:') && text.contains(']\n\n')) {
            text = text.substring(text.indexOf(']\n\n') + 3);
          }
          messages.value.add(ChatMessage(
            text: text,
            chatMessageType: type,
          ));
          shareMessages.add("$text - ${type == ChatMessageType.user ? 'Myself' : 'By BOT'}\n");
        }
        itemCount.value = messages.value.length;
        update();
        Future.delayed(const Duration(milliseconds: 100)).then((_) => scrollDown());
      }
    } catch (_) {}
  }

  void _handleGatewayFrame(GatewayFrame frame) {
    debugPrint("Gateway Frame: ${frame.type}");
    switch (frame) {
      case ChunkFrame(:final content):
        _appendStreamingChunk(content);
        break;
      case ThinkingFrame(:final content):
        _appendStreamingThinking(content);
        break;
      case ChunkResetFrame():
        _resetStreamingBuffer();
        break;
      case ToolCallFrame(:final id, :final name, :final args):
        _addToolCall(id, name, args);
        break;
      case ToolResultFrame(:final id, :final name, :final output):
        _updateToolResult(id, name, output);
        break;
      case DoneFrame(:final fullResponse):
        _completeTurn(fullResponse);
        break;
      case MessageFrame(:final content, :final fullResponse):
        _completeTurn(fullResponse ?? content);
        break;
      case AbortedFrame(:final reason):
        _handleAborted(reason);
        break;
      case ErrorFrame(:final message):
        _handleError(message);
        break;
      default:
        break;
    }
  }

  ChatMessage? get _currentStreamingMessage {
    if (messages.value.isEmpty) return null;
    final last = messages.value.last;
    if (last.chatMessageType == ChatMessageType.bot && last.isStreaming) {
      return last;
    }
    return null;
  }

  void _appendStreamingChunk(String content) {
    final msg = _currentStreamingMessage;
    if (msg != null) {
      msg.text += content;
      update();
      _autoScroll();
    }
  }

  void _appendStreamingThinking(String content) {
    final msg = _currentStreamingMessage;
    if (msg != null) {
      msg.thinking = (msg.thinking ?? '') + content;
      update();
      _autoScroll();
    }
  }

  void _resetStreamingBuffer() {
    final msg = _currentStreamingMessage;
    if (msg != null) {
      msg.text = '';
      update();
    }
  }

  void _addToolCall(String? id, String name, dynamic args) {
    final msg = _currentStreamingMessage;
    if (msg != null) {
      msg.toolCalls.add(ToolCallInfo(id: id, name: name, args: args));
      update();
      _autoScroll();
    }
  }

  void _updateToolResult(String? id, String? name, String output) {
    final msg = _currentStreamingMessage;
    if (msg != null) {
      for (final tool in msg.toolCalls) {
        if ((id != null && tool.id == id) || (name != null && tool.name == name)) {
          tool.output = output;
          break;
        }
      }
      update();
    }
  }

  void _completeTurn(String fullResponse) {
    final msg = _currentStreamingMessage;
    if (msg != null) {
      if (fullResponse.isNotEmpty) {
        msg.text = fullResponse;
      }
      msg.isStreaming = false;
      shareMessages.add("${msg.text} -By BOT\n");
    }
    _isLoading.value = false;
    update();
    _autoScroll();

    if (autoTts.value && fullResponse.trim().isNotEmpty) {
      final langCode = '${languageList[0]}-${languageList[1]}';
      speechMethod(fullResponse, langCode);
    }
  }

  void _handleAborted(String? reason) {
    final msg = _currentStreamingMessage;
    if (msg != null) {
      msg.isStreaming = false;
      if (msg.text.isEmpty) {
        msg.text = 'Генерация прервана';
      }
    }
    _isLoading.value = false;
    update();
  }

  void _handleError(String errorMsg) {
    final msg = _currentStreamingMessage;
    if (msg != null) {
      msg.isStreaming = false;
      msg.isError = true;
      if (msg.text.isEmpty) {
        msg.text = 'Ошибка шлюза: $errorMsg';
      }
    }
    _isLoading.value = false;
    update();
  }

  void _autoScroll() {
    if (scrollController.hasClients) {
      final pos = scrollController.position;
      if (pos.maxScrollExtent - pos.pixels < 200) {
        scrollController.animateTo(
          pos.maxScrollExtent,
          duration: const Duration(milliseconds: 100),
          curve: Curves.easeOut,
        );
      }
    }
  }

  void proccessChat() async {
    speechStopMethod();

    final input = chatController.text.trim();
    if (input.isEmpty) return;

    addTextCount();

    messages.value.add(
      ChatMessage(
        text: input,
        chatMessageType: ChatMessageType.user,
      ),
    );
    shareMessages.add("$input - Myself\n");

    // Add placeholder bot message in streaming mode
    final botMessage = ChatMessage(
      text: '',
      chatMessageType: ChatMessageType.bot,
      isStreaming: true,
    );
    messages.value.add(botMessage);

    itemCount.value = messages.value.length;
    _isLoading.value = true;
    textInput.value = input;
    chatController.clear();
    update();

    Future.delayed(const Duration(milliseconds: 50)).then((_) => scrollDown());

    // Send to Gateway via WebSocket
    if (_gatewayWs == null || !_gatewayWs!.isConnected) {
      _gatewayWs ??= GatewayWsClient();
      await _gatewayWs!.connect();
    }

    final sent = _gatewayWs!.sendMessage(input);
    if (!sent) {
      botMessage.text = "Не удалось отправить сообщение шлюзу OmnesAgent (проверьте подключение)";
      botMessage.isStreaming = false;
      botMessage.isError = true;
      _isLoading.value = false;
      update();
    }
  }

  RxString textInput = ''.obs;

  void proccessChat2() async {
    speechStopMethod();

    if (textInput.value.isEmpty) return;
    chatController.text = textInput.value;
    proccessChat();
  }

  void abortGeneration() {
    _gatewayWs?.abort();
    _handleAborted(null);
  }

  void toggleAutoTts() {
    autoTts.value = !autoTts.value;
    LocalStorage.saveAutoTts(value: autoTts.value);
    if (!autoTts.value) {
      speechStopMethod();
    }
    update();
  }

  void scrollDown() {
    if (scrollController.hasClients) {
      scrollController.animateTo(
        scrollController.position.maxScrollExtent,
        duration: const Duration(milliseconds: 300),
        curve: Curves.easeOut,
      );
    }
  }

  RxString userInput = "".obs;
  RxString result = "".obs;
  RxBool isListening = false.obs;
  var languageList = LocalStorage.getLanguage();
  late stt.SpeechToText speech;

  void listen(BuildContext context) async {
    speechStopMethod();

    if (!isListening.value) {
      chatController.text = '';
      result.value = '';
      userInput.value = '';
      try {
        bool available = await speech.initialize(
          onStatus: (val) {
            debugPrint('STT status: $val');
            if (val == 'done' || val == 'notListening') {
              isListening.value = false;
              update();
            }
          },
          onError: (val) {
            debugPrint('STT error: $val');
            isListening.value = false;
            update();
          },
        );
        if (available) {
          isListening.value = true;
          update();
          speech.listen(
            localeId: languageList[0],
            onResult: (val) {
              chatController.text = val.recognizedWords.toString();
              userInput.value = val.recognizedWords.toString();
              update();
            },
          );
        } else {
          isListening.value = false;
          update();
        }
      } catch (e) {
        debugPrint('STT error: $e');
        isListening.value = false;
        update();
      }
    } else {
      isListening.value = false;
      speech.stop();
      update();
    }
  }

  final FlutterTts flutterTts = FlutterTts();
  final _isSpeechLoading = false.obs;
  bool get isSpeechLoading => _isSpeechLoading.value;
  final _isSpeech = false.obs;
  bool get isSpeech => _isSpeech.value;

  speechMethod(String text, String language) async {
    final cleanText = SpeechHelper.stripMarkdownForSpeech(text);
    if (cleanText.isEmpty) return;

    _isSpeechLoading.value = true;
    _isSpeech.value = true;
    update();

    try {
      await flutterTts.setLanguage(language);
      await flutterTts.setPitch(1.0);
      await flutterTts.setSpeechRate(0.48);
      await flutterTts.speak(cleanText);
    } catch (e) {
      debugPrint("TTS error: $e");
      _isSpeech.value = false;
    }

    Future.delayed(
        const Duration(seconds: 2), () => _isSpeechLoading.value = false);
    update();
  }

  speechStopMethod() async {
    _isSpeech.value = false;
    try {
      await flutterTts.stop();
    } catch (_) {}
    update();
  }

  clearConversation() {
    speechStopMethod();
    messages.value.clear();
    shareMessages.clear();
    shareMessages.add('--THIS IS CONVERSATION with ${Strings.appName}--\n\n');
    textInput.value = '';
    itemCount.value = 0;
    speechStopMethod();
    update();
  }

  _getUserData() async {
    final uid = LocalStorage.getId() ?? SupabaseConfig.currentUserId ?? '';
    final name = LocalStorage.getName() ?? 'Omnes User';
    final email = LocalStorage.getEmail() ?? SupabaseConfig.currentUserEmail ?? '';
    final imageUrl = LocalStorage.getImage() ?? '';

    UserModel userData = UserModel(
      name: name,
      uniqueId: uid,
      email: email,
      phoneNumber: "",
      isActive: true,
      imageUrl: imageUrl,
    );

    userModel = userData;

    messages.value.add(
      ChatMessage(
        text: '${Strings.hello.tr} ${userData.name}',
        chatMessageType: ChatMessageType.bot,
      ),
    );
    shareMessages
        .add("${Strings.hello.tr} ${userData.name} -By ${Strings.appName}\n");

    Future.delayed(const Duration(milliseconds: 50)).then((_) => scrollDown());
    itemCount.value = messages.value.length;
    update();
  }

  void _setGesutUser() async {
    UserModel userData = UserModel(
        name: "Guest",
        uniqueId: '',
        email: '',
        phoneNumber: '',
        isActive: false,
        imageUrl: '');

    userModel = userData;

    messages.value.add(
      ChatMessage(
        text: Strings.helloGuest.tr,
        chatMessageType: ChatMessageType.bot,
      ),
    );
    shareMessages.add("${Strings.helloGuest.tr} -By ${Strings.appName}\n\n");

    Future.delayed(const Duration(milliseconds: 50)).then((_) => scrollDown());
    itemCount.value = messages.value.length;
    update();
  }

  void shareChat(BuildContext context) {
    debugPrint(shareMessages.toString());
    Share.share("${shareMessages.toString()}\n\n --CONVERSATION END--",
        subject: "I'm sharing Conversation with ${Strings.appName}");
  }

  RxInt count = 0.obs;

  addTextCount() async {
    count.value++;
  }

  getSuggestedCategory() async {
    isLoading2.value = true;
    update();

    try {
      suggestedData = await SuggestedPromptsRepository().getPrompts();
    } catch (_) {
      suggestedData = SuggestedPromptsRepository.defaultPrompts;
    }

    isLoading2.value = false;
    update();
  }
}