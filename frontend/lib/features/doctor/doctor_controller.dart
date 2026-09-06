import 'package:get/get.dart';
import '../../core/gateway/gateway_http.dart';

class DoctorController extends GetxController {
  final GatewayHttpClient _http = GatewayHttpClient();

  final RxList<Map<String, dynamic>> results = <Map<String, dynamic>>[].obs;
  final RxBool isLoading = true.obs;

  @override
  void onInit() {
    super.onInit();
    runDiagnostics();
  }

  Future<void> runDiagnostics() async {
    isLoading.value = true;
    try {
      final res = await _http.runDoctor();
      results.value = res;
    } finally {
      isLoading.value = false;
    }
  }

  int get passCount => results.where((r) {
        final s = (r['status'] ?? '').toString().toLowerCase();
        return s == 'pass' || s == 'ok' || s == 'healthy';
      }).length;

  int get warnCount => results.where((r) {
        final s = (r['status'] ?? '').toString().toLowerCase();
        return s == 'warn' || s == 'warning';
      }).length;

  int get failCount => results.where((r) {
        final s = (r['status'] ?? '').toString().toLowerCase();
        return s == 'fail' || s == 'error';
      }).length;

  @override
  void onClose() {
    _http.dispose();
    super.onClose();
  }
}
