import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../design_system/design_system.dart';
import 'skills_controller.dart';

class SkillsScreen extends StatelessWidget {
  const SkillsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(SkillsController());

    return Scaffold(
      backgroundColor: ShadcnColors.background,
      appBar: AppBar(
        title: const Text('Навыки агента (Skills)'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh, color: ShadcnColors.primary),
            onPressed: () => controller.fetchSkills(),
          ),
        ],
      ),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: ShadcnInput(
                hintText: 'Поиск навыков и бандлов...',
                prefixIcon: const Icon(Icons.search, color: ShadcnColors.foregroundMuted, size: 18),
                onChanged: (val) => controller.searchQuery.value = val,
              ),
            ),
            Expanded(
              child: Obx(() {
                if (controller.isLoading.value) {
                  return const Center(
                    child: CircularProgressIndicator(color: ShadcnColors.primary),
                  );
                }

                final list = controller.filteredBundles;
                if (list.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const Icon(FontAwesomeIcons.puzzlePiece, color: ShadcnColors.foregroundSubtle, size: 40),
                        const SizedBox(height: 12),
                        const Text(
                          'Навыки не найдены',
                          style: TextStyle(color: ShadcnColors.foregroundMuted, fontSize: 14),
                        ),
                        const SizedBox(height: 16),
                        ShadcnButton(
                          label: 'Обновить',
                          size: ShadcnButtonSize.sm,
                          variant: ShadcnButtonVariant.outline,
                          onPressed: () => controller.fetchSkills(),
                        ),
                      ],
                    ),
                  );
                }

                return ListView.separated(
                  padding: const EdgeInsets.all(16),
                  itemCount: list.length,
                  separatorBuilder: (_, __) => const SizedBox(height: 12),
                  itemBuilder: (ctx, idx) {
                    final bundle = list[idx];
                    final name = (bundle['name'] ?? bundle['id'] ?? 'Unnamed Skill').toString();
                    final desc = (bundle['description'] ?? 'Набор процедур и сценариев поведения агента').toString();
                    final enabled = bundle['enabled'] == true || bundle['active'] == true;
                    final skillsCount = bundle['skills'] is List ? (bundle['skills'] as List).length : null;

                    return ShadcnCard(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              Row(
                                children: [
                                  const Icon(
                                    FontAwesomeIcons.wandMagicSparkles,
                                    size: 15,
                                    color: ShadcnColors.primary,
                                  ),
                                  const SizedBox(width: 8),
                                  Text(
                                    name,
                                    style: const TextStyle(
                                      color: ShadcnColors.foreground,
                                      fontSize: 16,
                                      fontWeight: FontWeight.bold,
                                    ),
                                  ),
                                ],
                              ),
                              ShadcnBadge(
                                label: enabled ? 'ACTIVE' : 'READY',
                                variant: enabled ? ShadcnBadgeVariant.cyber : ShadcnBadgeVariant.neutral,
                                showDot: enabled,
                              ),
                            ],
                          ),
                          const SizedBox(height: 8),
                          Text(
                            desc,
                            style: const TextStyle(
                              color: ShadcnColors.foregroundMuted,
                              fontSize: 13,
                              height: 1.4,
                            ),
                          ),
                          if (skillsCount != null) ...[
                            const SizedBox(height: 10),
                            Row(
                              children: [
                                const Icon(Icons.layers_outlined, size: 14, color: ShadcnColors.foregroundSubtle),
                                const SizedBox(width: 4),
                                Text(
                                  '$skillsCount подсистем / процедур',
                                  style: const TextStyle(
                                    color: ShadcnColors.foregroundSubtle,
                                    fontSize: 12,
                                  ),
                                ),
                              ],
                            ),
                          ],
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
}
