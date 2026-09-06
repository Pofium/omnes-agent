import 'package:get/get.dart';
import '../../core/gateway/gateway_http.dart';

class ToolsController extends GetxController {
  final GatewayHttpClient _http = GatewayHttpClient();

  final RxList<Map<String, dynamic>> tools = <Map<String, dynamic>>[].obs;
  final RxList<Map<String, dynamic>> filteredTools = <Map<String, dynamic>>[].obs;
  final RxBool isLoading = true.obs;
  final RxString searchQuery = ''.obs;
  final RxString selectedCategory = 'all'.obs;

  @override
  void onInit() {
    super.onInit();
    fetchTools();
  }

  Future<void> fetchTools() async {
    isLoading.value = true;
    try {
      final res = await _http.getTools();
      tools.value = res;
      _applyFilter();
    } finally {
      isLoading.value = false;
    }
  }

  void onSearch(String query) {
    searchQuery.value = query;
    _applyFilter();
  }

  void selectCategory(String category) {
    selectedCategory.value = category;
    _applyFilter();
  }

  void _applyFilter() {
    final query = searchQuery.value.toLowerCase().trim();
    final cat = selectedCategory.value;

    filteredTools.value = tools.where((tool) {
      final name = (tool['name'] ?? '').toString().toLowerCase();
      final desc = (tool['description'] ?? '').toString().toLowerCase();
      final toolCat = (tool['category'] ?? 'general').toString().toLowerCase();

      final matchesQuery = query.isEmpty || name.contains(query) || desc.contains(query);
      final matchesCategory = cat == 'all' || toolCat == cat;

      return matchesQuery && matchesCategory;
    }).toList();
  }

  List<String> get categories {
    final cats = {'all'};
    for (final t in tools) {
      final c = (t['category'] ?? 'general').toString();
      cats.add(c);
    }
    return cats.toList();
  }

  @override
  void onClose() {
    _http.dispose();
    super.onClose();
  }
}
