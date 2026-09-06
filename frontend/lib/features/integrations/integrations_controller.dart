import 'package:get/get.dart';
import '../../core/gateway/gateway_http.dart';

class IntegrationsController extends GetxController {
  final GatewayHttpClient _http = GatewayHttpClient();

  final RxList<Map<String, dynamic>> channels = <Map<String, dynamic>>[].obs;
  final RxBool isLoading = true.obs;

  @override
  void onInit() {
    super.onInit();
    fetchChannels();
  }

  Future<void> fetchChannels() async {
    isLoading.value = true;
    try {
      final res = await _http.getChannels();
      channels.value = res;
    } finally {
      isLoading.value = false;
    }
  }

  @override
  void onClose() {
    _http.dispose();
    super.onClose();
  }
}
