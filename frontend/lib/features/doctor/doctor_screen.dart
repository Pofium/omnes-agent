import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../design_system/design_system.dart';
import 'doctor_controller.dart';

class DoctorScreen extends StatelessWidget {
  const DoctorScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(DoctorController());

    return Scaffold(
      backgroundColor: ShadcnColors.background,
      appBar: AppBar(
        title: const Text('Диагностика шлюза (Doctor)'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh, color: ShadcnColors.primary),
            onPressed: () => controller.runDiagnostics(),
          ),
        ],
      ),
      body: SafeArea(
        child: Obx(() {
          if (controller.isLoading.value) {
            return const Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  CircularProgressIndicator(color: ShadcnColors.primary),
                  SizedBox(height: 16),
                  Text(
                    'Диагностика шлюза и компонентов...',
                    style: TextStyle(color: ShadcnColors.foregroundMuted, fontSize: 14),
                  ),
                ],
              ),
            );
          }

          final list = controller.results;
          if (list.isEmpty) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(FontAwesomeIcons.circleCheck, color: ShadcnColors.success, size: 48),
                  const SizedBox(height: 16),
                  const Text(
                    'Шлюз работает в штатном режиме',
                    style: TextStyle(
                      color: ShadcnColors.foreground,
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'Все проверенные сервисы и провайдеры активны',
                    style: TextStyle(color: ShadcnColors.foregroundMuted, fontSize: 13),
                  ),
                  const SizedBox(height: 20),
                  ShadcnButton(
                    label: 'Запустить проверку снова',
                    size: ShadcnButtonSize.md,
                    onPressed: () => controller.runDiagnostics(),
                  ),
                ],
              ),
            );
          }

          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              // Summary Stat Card
              ShadcnCard(
                isGlowing: controller.failCount == 0,
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceAround,
                  children: [
                    _buildStatItem('В норме', controller.passCount, ShadcnColors.success),
                    Container(width: 1, height: 32, color: ShadcnColors.border),
                    _buildStatItem('Замечания', controller.warnCount, ShadcnColors.warning),
                    Container(width: 1, height: 32, color: ShadcnColors.border),
                    _buildStatItem('Ошибки', controller.failCount, ShadcnColors.destructive),
                  ],
                ),
              ),
              const SizedBox(height: 16),

              const Text(
                'РЕЗУЛЬТАТЫ ПРОВЕРКИ СИСТЕМЫ',
                style: TextStyle(
                  color: ShadcnColors.primary,
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  letterSpacing: 1,
                ),
              ),
              const SizedBox(height: 10),

              ...list.map((res) {
                final name = (res['name'] ?? res['target'] ?? 'Diagnostic check').toString();
                final status = (res['status'] ?? 'ok').toString().toLowerCase();
                final message = (res['message'] ?? res['description'] ?? '').toString();
                final remedy = (res['remedy'] ?? res['recommendation'] ?? '').toString();

                ShadcnBadgeVariant badgeVariant;
                IconData statusIcon;
                Color statusColor;

                if (status == 'fail' || status == 'error') {
                  badgeVariant = ShadcnBadgeVariant.destructive;
                  statusIcon = Icons.cancel;
                  statusColor = ShadcnColors.destructive;
                } else if (status == 'warn' || status == 'warning') {
                  badgeVariant = ShadcnBadgeVariant.warning;
                  statusIcon = Icons.warning_amber_rounded;
                  statusColor = ShadcnColors.warning;
                } else {
                  badgeVariant = ShadcnBadgeVariant.success;
                  statusIcon = Icons.check_circle;
                  statusColor = ShadcnColors.success;
                }

                return Padding(
                  padding: const EdgeInsets.only(bottom: 10),
                  child: ShadcnCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Row(
                              children: [
                                Icon(statusIcon, color: statusColor, size: 18),
                                const SizedBox(width: 8),
                                Text(
                                  name,
                                  style: const TextStyle(
                                    color: ShadcnColors.foreground,
                                    fontSize: 15,
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                              ],
                            ),
                            ShadcnBadge(
                              label: status.toUpperCase(),
                              variant: badgeVariant,
                            ),
                          ],
                        ),
                        if (message.isNotEmpty) ...[
                          const SizedBox(height: 8),
                          Text(
                            message,
                            style: const TextStyle(
                              color: ShadcnColors.foregroundMuted,
                              fontSize: 13,
                              height: 1.4,
                            ),
                          ),
                        ],
                        if (remedy.isNotEmpty) ...[
                          const SizedBox(height: 8),
                          Container(
                            padding: const EdgeInsets.all(8),
                            decoration: BoxDecoration(
                              color: ShadcnColors.background,
                              borderRadius: BorderRadius.circular(6),
                              border: Border.all(color: ShadcnColors.border),
                            ),
                            child: Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                const Icon(Icons.lightbulb_outline, color: ShadcnColors.primary, size: 16),
                                const SizedBox(width: 6),
                                Expanded(
                                  child: Text(
                                    remedy,
                                    style: const TextStyle(
                                      color: ShadcnColors.primary,
                                      fontSize: 12,
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                );
              }),
            ],
          );
        }),
      ),
    );
  }

  Widget _buildStatItem(String label, int count, Color color) {
    return Column(
      children: [
        Text(
          '$count',
          style: TextStyle(
            color: color,
            fontSize: 22,
            fontWeight: FontWeight.bold,
          ),
        ),
        const SizedBox(height: 2),
        Text(
          label,
          style: const TextStyle(
            color: ShadcnColors.foregroundMuted,
            fontSize: 12,
          ),
        ),
      ],
    );
  }
}
