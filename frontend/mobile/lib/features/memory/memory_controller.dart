// Controller managing state for agent memory screen.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import 'models/memory_entry.dart';
import 'memory_repository.dart';

class MemoryController extends GetxController {
  final MemoryRepository _repo;

  MemoryController({MemoryRepository? repo}) : _repo = repo ?? MemoryRepository();

  final RxList<MemoryEntry> entries = <MemoryEntry>[].obs;
  final RxBool isLoading = false.obs;
  final RxString errorMessage = ''.obs;
  final RxString activeAgent = ''.obs;
  final RxString selectedCategory = 'all'.obs;
  final RxString searchQuery = ''.obs;

  @override
  void onInit() {
    super.onInit();
    loadMemory();
  }

  /// Loads memory entries with optional search and category filter.
  Future<void> loadMemory() async {
    isLoading.value = true;
    errorMessage.value = '';
    try {
      activeAgent.value = await GatewayConfig.getActiveAgent();
      final list = await _repo.listMemory(
        query: searchQuery.value.isNotEmpty ? searchQuery.value : null,
        category: selectedCategory.value != 'all' ? selectedCategory.value : null,
        agentAlias: activeAgent.value,
      );
      // Sort newest first
      list.sort((a, b) => b.timestamp.compareTo(a.timestamp));
      entries.assignAll(list);
    } catch (e) {
      errorMessage.value = 'Ошибка загрузки памяти: $e';
    } finally {
      isLoading.value = false;
    }
  }

  void setCategory(String category) {
    if (selectedCategory.value == category) return;
    selectedCategory.value = category;
    loadMemory();
  }

  void search(String query) {
    searchQuery.value = query.trim();
    loadMemory();
  }

  /// Stores a new memory fact.
  Future<bool> addMemory({
    required String key,
    required String content,
    String category = 'core',
  }) async {
    if (key.trim().isEmpty || content.trim().isEmpty) return false;
    isLoading.value = true;
    try {
      final success = await _repo.storeMemory(
        key: key.trim(),
        content: content.trim(),
        category: category,
        agentAlias: activeAgent.value,
      );
      if (success) {
        await loadMemory();
        Get.snackbar(
          'Память обновлена',
          'Факт "$key" сохранён в долгосрочную память агента',
          snackPosition: SnackPosition.BOTTOM,
          backgroundColor: Colors.green.shade800,
          colorText: Colors.white,
          duration: const Duration(seconds: 2),
        );
        return true;
      }
    } catch (_) {} finally {
      isLoading.value = false;
    }
    return false;
  }

  /// Deletes a memory fact.
  Future<void> deleteMemory(MemoryEntry entry) async {
    final confirmed = await Get.dialog<bool>(
      AlertDialog(
        title: const Text('Удалить факт?'),
        content: Text('Удалить "${entry.key}" из долгосрочной памяти агента?'),
        actions: [
          TextButton(onPressed: () => Get.back(result: false), child: const Text('Отмена')),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            onPressed: () => Get.back(result: true),
            child: const Text('Удалить', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    try {
      final success = await _repo.deleteMemory(entry.key, agentAlias: activeAgent.value);
      if (success) {
        entries.removeWhere((e) => e.key == entry.key);
        Get.snackbar(
          'Удалено',
          'Запись удалена из памяти',
          snackPosition: SnackPosition.BOTTOM,
        );
      }
    } catch (_) {}
  }

  @override
  void onClose() {
    _repo.dispose();
    super.onClose();
  }
}
