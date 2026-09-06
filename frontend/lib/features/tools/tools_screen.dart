import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../design_system/design_system.dart';
import 'tools_controller.dart';

class ToolsScreen extends StatelessWidget {
  const ToolsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(ToolsController());

    return Scaffold(
      backgroundColor: ShadcnColors.background,
      appBar: AppBar(
        title: const Text('Инструменты агента'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh, color: ShadcnColors.primary),
            onPressed: () => controller.fetchTools(),
          ),
        ],
      ),
      body: SafeArea(
        child: Column(
          children: [
            // Search Bar
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: ShadcnInput(
                hintText: 'Поиск по названию или описанию...',
                prefixIcon: const Icon(Icons.search, color: ShadcnColors.foregroundMuted, size: 18),
                onChanged: controller.onSearch,
              ),
            ),

            // Category selector chips
            Obx(() {
              final cats = controller.categories;
              if (cats.length <= 1) return const SizedBox.shrink();

              return SizedBox(
                height: 38,
                child: ListView.separated(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  scrollDirection: Axis.horizontal,
                  itemCount: cats.length,
                  separatorBuilder: (_, __) => const SizedBox(width: 8),
                  itemBuilder: (ctx, idx) {
                    final cat = cats[idx];
                    final isSelected = controller.selectedCategory.value == cat;
                    return GestureDetector(
                      onTap: () => controller.selectCategory(cat),
                      child: ShadcnBadge(
                        label: cat.toUpperCase(),
                        variant: isSelected ? ShadcnBadgeVariant.cyber : ShadcnBadgeVariant.neutral,
                      ),
                    );
                  },
                ),
              );
            }),

            const SizedBox(height: 8),

            // Tools list
            Expanded(
              child: Obx(() {
                if (controller.isLoading.value) {
                  return const Center(
                    child: CircularProgressIndicator(color: ShadcnColors.primary),
                  );
                }

                final list = controller.filteredTools;
                if (list.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Icon(FontAwesomeIcons.wrench, color: ShadcnColors.foregroundSubtle, size: 40),
                        const SizedBox(height: 12),
                        const Text(
                          'Инструменты не найдены',
                          style: TextStyle(color: ShadcnColors.foregroundMuted, fontSize: 14),
                        ),
                        const SizedBox(height: 16),
                        ShadcnButton(
                          label: 'Обновить',
                          size: ShadcnButtonSize.sm,
                          variant: ShadcnButtonVariant.outline,
                          onPressed: () => controller.fetchTools(),
                        ),
                      ],
                    ),
                  );
                }

                return ListView.separated(
                  padding: const EdgeInsets.all(16),
                  itemCount: list.length,
                  separatorBuilder: (_, __) => const SizedBox(height: 10),
                  itemBuilder: (ctx, idx) {
                    final tool = list[idx];
                    final name = (tool['name'] ?? 'Unnamed Tool').toString();
                    final desc = (tool['description'] ?? 'Без описания').toString();
                    final category = (tool['category'] ?? 'general').toString();

                    return ShadcnCard(
                      onTap: () => _showToolDetails(context, tool),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Row(
                                children: [
                                  const Icon(
                                    FontAwesomeIcons.terminal,
                                    size: 14,
                                    color: ShadcnColors.primary,
                                  ),
                                  const SizedBox(width: 8),
                                  Text(
                                    name,
                                    style: const TextStyle(
                                      color: ShadcnColors.foreground,
                                      fontSize: 15,
                                      fontWeight: FontWeight.bold,
                                      fontFamily: 'monospace',
                                    ),
                                  ),
                                ],
                              ),
                              ShadcnBadge(
                                label: category,
                                variant: ShadcnBadgeVariant.cyber,
                              ),
                            ],
                          ),
                          const SizedBox(height: 8),
                          Text(
                            desc,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            style: const TextStyle(
                              color: ShadcnColors.foregroundMuted,
                              fontSize: 13,
                              height: 1.4,
                            ),
                          ),
                        ],
                      ),
                    );
                  },
                );
              }),
            ),
          ],
        ),
      ),
    );
  }

  void _showToolDetails(BuildContext context, Map<String, dynamic> tool) {
    final name = (tool['name'] ?? '').toString();
    final desc = (tool['description'] ?? '').toString();
    final params = tool['parameters'] ?? tool['schema'] ?? {};

    String prettyParams = '';
    try {
      prettyParams = const JsonEncoder.withIndent('  ').convert(params);
    } catch (_) {
      prettyParams = params.toString();
    }

    ShadcnDialog.showBottomSheet(
      context: context,
      title: name,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          const Text(
            'ОПИСАНИЕ',
            style: TextStyle(
              color: ShadcnColors.primary,
              fontSize: 11,
              fontWeight: FontWeight.bold,
              letterSpacing: 1,
            ),
          ),
          const SizedBox(height: 6),
          Text(
            desc,
            style: const TextStyle(color: ShadcnColors.foreground, fontSize: 13, height: 1.4),
          ),
          const SizedBox(height: 16),
          if (params is Map && params.isNotEmpty) ...[
            const Text(
              'ПАРАМЕТРЫ (JSON SCHEMA)',
              style: TextStyle(
                color: ShadcnColors.primary,
                fontSize: 11,
                fontWeight: FontWeight.bold,
                letterSpacing: 1,
              ),
            ),
            const SizedBox(height: 6),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: ShadcnColors.background,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: ShadcnColors.border),
              ),
              child: SelectableText(
                prettyParams,
                style: const TextStyle(
                  color: ShadcnColors.info,
                  fontFamily: 'monospace',
                  fontSize: 12,
                  height: 1.3,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}
