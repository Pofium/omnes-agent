// Controller for managing chat sessions and thread history.
import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import '../../routes/routes.dart';
import 'models/session_info.dart';

class SessionsController extends GetxController {
  final GatewayHttpClient _http = GatewayHttpClient();

  final RxList<SessionInfo> sessions = <SessionInfo>[].obs;
  final RxBool isLoading = false.obs;
  final RxString activeAgent = 'chief'.obs;

  @override
  void onInit() {
    super.onInit();
    activeAgent.value = GatewayConfig.getAgentAlias();
    loadSessions();
  }

  @override
  void onClose() {
    _http.dispose();
    super.onClose();
  }

  /// Loads sessions from the ZeroClaw gateway.
  Future<void> loadSessions() async {
    isLoading.value = true;
    update();
    try {
      final list = await _http.getSessionsList();
      sessions.assignAll(list);
    } catch (_) {}
    isLoading.value = false;
    update();
  }

  /// Deletes a session by ID.
  Future<void> deleteSession(String sessionId) async {
    final success = await _http.deleteSession(sessionId);
    if (success) {
      sessions.removeWhere((s) => s.sessionId == sessionId);
      update();
    }
  }

  /// Opens an existing session in the chat screen.
  void openSession(SessionInfo session) {
    Get.toNamed(
      Routes.chatScreen,
      arguments: {
        'sessionId': session.sessionId,
        'agentAlias': session.agentAlias,
      },
    );
  }

  /// Starts a fresh session with the active agent.
  void startNewChat() {
    final newSessionId = "chat_${DateTime.now().millisecondsSinceEpoch}";
    Get.toNamed(
      Routes.chatScreen,
      arguments: {
        'sessionId': newSessionId,
        'agentAlias': activeAgent.value,
        'isNew': true,
      },
    );
  }
}
