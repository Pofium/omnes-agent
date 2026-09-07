import 'package:get/get.dart';
import '../../core/gateway/gateway_http.dart';

class SkillsController extends GetxController {
  final GatewayHttpClient _http = GatewayHttpClient();

  final RxList<Map<String, dynamic>> bundles = <Map<String, dynamic>>[].obs;
  final RxBool isLoading = true.obs;
  final RxString searchQuery = ''.obs;

  @override
  void onInit() {
    super.onInit();
    fetchSkills();
  }

  Future<void> fetchSkills() async {
    isLoading.value = true;
    try {
      final res = await _http.getSkillsBundles();
      bundles.value = res;
    } finally {
      isLoading.value = false;
    }
  }

  List<Map<String, dynamic>> get filteredBundles {
    final query = searchQuery.value.toLowerCase().trim();
    if (query.isEmpty) return bundles;

    return bundles.where((b) {
      final name = (b['name'] ?? b['id'] ?? '').toString().toLowerCase();
      final desc = (b['description'] ?? '').toString().toLowerCase();
      return name.contains(query) || desc.contains(query);
    }).toList();
  }

  @override
  void onClose() {
    _http.dispose();
    super.onClose();
  }
}
