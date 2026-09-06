// Controller for discovering and selecting active AI agents on the gateway.
import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';

class AgentController extends GetxController {
  final GatewayHttpClient _http = GatewayHttpClient();

  final RxList<String> availableAgents = <String>[].obs;
  final RxString currentAgent = 'chief'.obs;
  final RxBool isLoading = false.obs;

  @override
  void onInit() {
    super.onInit();
    currentAgent.value = GatewayConfig.getAgentAlias();
    loadAgents();
  }

  @override
  void onClose() {
    _http.dispose();
    super.onClose();
  }

  /// Loads list of configured agent aliases from gateway.
  Future<void> loadAgents() async {
    isLoading.value = true;
    update();
    try {
      final list = await _http.getAgentAliases();
      availableAgents.assignAll(list);
    } catch (_) {}
    isLoading.value = false;
    update();
  }

  /// Sets the active agent alias.
  Future<void> selectAgent(String alias) async {
    currentAgent.value = alias;
    await GatewayConfig.setAgentAlias(alias);
    update();
  }
}
