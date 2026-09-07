// Screen for viewing, searching, and managing agent long-term memory facts.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'memory_controller.dart';
import 'models/memory_entry.dart';

class MemoryScreen extends StatelessWidget {
  const MemoryScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(MemoryController());

    return Scaffold(
      appBar: AppBar(
        title: const Text('Память агента'),
        actions: [
          IconButton(
            tooltip: 'Обновить',
            icon: const Icon(Icons.refresh),
            onPressed: controller.loadMemory,
          ),
        ],
      ),
      body: Column(
        children: [
          _buildSearchBar(controller),
          _buildCategoryFilter(context, controller),
          const Divider(height: 1),
          Expanded(
            child: Obx(() {
              if (controller.isLoading.value && controller.entries.isEmpty) {
                return const Center(child: CircularProgressIndicator());
              }

              if (controller.errorMessage.isNotEmpty) {
                return Center(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Icon(Icons.error_outline, size: 48, color: Colors.red),
                        const SizedBox(height: 8),
                        Text(controller.errorMessage.value, textAlign: TextAlign.center),
                        const SizedBox(height: 12),
                        ElevatedButton(
                          onPressed: controller.loadMemory,
                          child: const Text('Повторить'),
                        ),
                      ],
                    ),
                  ),
                );
              }

              if (controller.entries.isEmpty) {
                return Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.psychology_outlined, size: 64, color: Colors.grey.shade400),
                      const SizedBox(height: 16),
                      const Text(
                        'Нет сохранённых воспоминаний',
                        style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                      ),
                      const SizedBox(height: 8),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 32),
                        child: Text(
                          'Агент автоматически сохраняет важные факты из диалогов, либо вы можете добавить их вручную.',
                          textAlign: TextAlign.center,
                          style: TextStyle(color: Colors.grey.shade600, fontSize: 12),
                        ),
                      ),
                    ],
                  ),
                );
              }

              return RefreshIndicator(
                onRefresh: controller.loadMemory,
                child: ListView.builder(
                  padding: const EdgeInsets.all(12),
                  itemCount: controller.entries.length,
                  itemBuilder: (context, index) {
                    final entry = controller.entries[index];
                    return _buildMemoryCard(context, controller, entry);
                  },
                ),
              );
            }),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _showAddMemoryDialog(context, controller),
        icon: const Icon(Icons.add),
        label: const Text('Добавить факт'),
      ),
    );
  }

  Widget _buildSearchBar(MemoryController controller) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 8, 12, 4),
      child: TextField(
        decoration: InputDecoration(
          hintText: 'Поиск по ключевым словам и фактам...',
          prefixIcon: const Icon(Icons.search, size: 20),
          isDense: true,
          contentPadding: const EdgeInsets.symmetric(vertical: 10),
          border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
        ),
        onChanged: (val) => controller.search(val),
      ),
    );
  }

  Widget _buildCategoryFilter(BuildContext context, MemoryController controller) {
    final categories = [
      {'id': 'all', 'label': 'Все'},
      {'id': 'core', 'label': 'Важные (Core)'},
      {'id': 'daily', 'label': 'Ежедневные'},
      {'id': 'conversation', 'label': 'Диалоги'},
    ];

    return Obx(() {
      final selected = controller.selectedCategory.value;
      return SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
        child: Row(
          children: categories.map((cat) {
            final isSelected = selected == cat['id'];
            return Padding(
              padding: const EdgeInsets.only(right: 8),
              child: FilterChip(
                label: Text(cat['label']!),
                selected: isSelected,
                onSelected: (_) => controller.setCategory(cat['id']!),
              ),
            );
          }).toList(),
        ),
      );
    });
  }

  Widget _buildMemoryCard(
    BuildContext context,
    MemoryController controller,
    MemoryEntry entry,
  ) {
    Color categoryColor = Colors.blueGrey;
    if (entry.category == 'core') categoryColor = Colors.purple;
    if (entry.category == 'daily') categoryColor = Colors.teal;
    if (entry.category == 'conversation') categoryColor = Colors.indigo;

    return Card(
      elevation: 1,
      margin: const EdgeInsets.only(bottom: 10),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: categoryColor.withOpacity(0.15),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    entry.category.toUpperCase(),
                    style: TextStyle(
                      fontSize: 10,
                      fontWeight: FontWeight.bold,
                      color: categoryColor,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    entry.key,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontWeight: FontWeight.bold,
                      fontSize: 13,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Text(
                  entry.formattedTime,
                  style: TextStyle(fontSize: 11, color: Colors.grey.shade500),
                ),
                const SizedBox(width: 4),
                IconButton(
                  icon: const Icon(Icons.delete_outline, size: 18, color: Colors.grey),
                  tooltip: 'Удалить факт',
                  onPressed: () => controller.deleteMemory(entry),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(),
                ),
              ],
            ),
            const SizedBox(height: 8),
            SelectableText(
              entry.content,
              style: const TextStyle(fontSize: 14, height: 1.35),
            ),
          ],
        ),
      ),
    );
  }

  void _showAddMemoryDialog(BuildContext context, MemoryController controller) {
    final keyCtrl = TextEditingController();
    final contentCtrl = TextEditingController();
    String selectedCat = 'core';

    Get.dialog(
      StatefulBuilder(
        builder: (context, setDialogState) {
          return AlertDialog(
            title: const Text('Новый факт в память'),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextField(
                    controller: keyCtrl,
                    autofocus: true,
                    decoration: const InputDecoration(
                      labelText: 'Ключ / Тема',
                      hintText: 'например: user_preference, tech_stack',
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: contentCtrl,
                    maxLines: 3,
                    decoration: const InputDecoration(
                      labelText: 'Содержимое факта',
                      hintText: 'Что агент должен помнить всегда',
                    ),
                  ),
                  const SizedBox(height: 12),
                  DropdownButtonFormField<String>(
                    value: selectedCat,
                    decoration: const InputDecoration(labelText: 'Категория'),
                    items: const [
                      DropdownMenuItem(value: 'core', child: Text('Важное (Core)')),
                      DropdownMenuItem(value: 'daily', child: Text('Ежедневное (Daily)')),
                      DropdownMenuItem(value: 'conversation', child: Text('Диалоги')),
                    ],
                    onChanged: (val) {
                      if (val != null) {
                        setDialogState(() => selectedCat = val);
                      }
                    },
                  ),
                ],
              ),
            ),
            actions: [
              TextButton(onPressed: () => Get.back(), child: const Text('Отмена')),
              ElevatedButton(
                onPressed: () async {
                  final k = keyCtrl.text.trim();
                  final c = contentCtrl.text.trim();
                  if (k.isNotEmpty && c.isNotEmpty) {
                    Get.back();
                    await controller.addMemory(key: k, content: c, category: selectedCat);
                  }
                },
                child: const Text('Сохранить'),
              ),
            ],
          );
        },
      ),
    );
  }
}
