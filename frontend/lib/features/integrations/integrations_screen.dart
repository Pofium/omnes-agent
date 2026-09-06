import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../design_system/design_system.dart';
import 'integrations_controller.dart';

class IntegrationsScreen extends StatelessWidget {
  const IntegrationsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(IntegrationsController());

    final predefinedIntegrations = [
      {'name': 'Telegram Bot', 'type': 'telegram', 'icon': FontAwesomeIcons.telegram, 'desc': 'Интеграция с ботом Telegram через Long-polling/Webhook'},
      {'name': 'Discord Gateway', 'type': 'discord', 'icon': FontAwesomeIcons.discord, 'desc': 'Подключение к серверам Discord через WebSocket-гейтвей'},
      {'name': 'Slack App', 'type': 'slack', 'icon': FontAwesomeIcons.slack, 'desc': 'Приложение Slack Socket Mode / Events API'},
      {'name': 'WhatsApp Cloud', 'type': 'whatsapp', 'icon': FontAwesomeIcons.whatsapp, 'desc': 'Официальный WhatsApp Business API шлюз'},
      {'name': 'HTTP Webhooks', 'type': 'webhook', 'icon': FontAwesomeIcons.networkWired, 'desc': 'Входящие события и триггеры для сторонних систем'},
      {'name': 'Matrix / Element', 'type': 'matrix', 'icon': FontAwesomeIcons.comments, 'desc': 'Децентрализованный E2EE протокол связи Matrix'},
    ];

    return Scaffold(
      backgroundColor: ShadcnColors.background,
      appBar: AppBar(
        title: const Text('Каналы и интеграции'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh, color: ShadcnColors.primary),
            onPressed: () => controller.fetchChannels(),
          ),
        ],
      ),
      body: SafeArea(
        child: Obx(() {
          if (controller.isLoading.value) {
            return const Center(
              child: CircularProgressIndicator(color: ShadcnColors.primary),
            );
          }

          final activeChannels = controller.channels;

          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              const Text(
                'ДОСТУПНЫЕ КАНАЛЫ СВЯЗИ АГЕНТА',
                style: TextStyle(
                  color: ShadcnColors.primary,
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  letterSpacing: 1,
                ),
              ),
              const SizedBox(height: 12),

              ...predefinedIntegrations.map((item) {
                final type = item['type'] as String;
                final name = item['name'] as String;
                final icon = item['icon'] as IconData;
                final desc = item['desc'] as String;

                final isActive = activeChannels.any((c) {
                  final cType = (c['type'] ?? c['channel_type'] ?? '').toString().toLowerCase();
                  return cType.contains(type);
                });

                return Padding(
                  padding: const EdgeInsets.only(bottom: 12),
                  child: ShadcnCard(
                    isActive: isActive,
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Container(
                          width: 42,
                          height: 42,
                          decoration: BoxDecoration(
                            color: isActive
                                ? ShadcnColors.primary.withOpacity(0.15)
                                : ShadcnColors.cardElevated,
                            borderRadius: BorderRadius.circular(10),
                            border: Border.all(
                              color: isActive
                                  ? ShadcnColors.primary.withOpacity(0.4)
                                  : ShadcnColors.border,
                            ),
                          ),
                          child: Center(
                            child: Icon(
                              icon,
                              size: 20,
                              color: isActive ? ShadcnColors.primary : ShadcnColors.foregroundMuted,
                            ),
                          ),
                        ),
                        const SizedBox(width: 14),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Row(
                                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                children: [
                                  Text(
                                    name,
                                    style: const TextStyle(
                                      color: ShadcnColors.foreground,
                                      fontSize: 15,
                                      fontWeight: FontWeight.bold,
                                    ),
                                  ),
                                  ShadcnBadge(
                                    label: isActive ? 'ПОДКЛЮЧЕНО' : 'НЕ АКТИВЕН',
                                    variant: isActive ? ShadcnBadgeVariant.cyber : ShadcnBadgeVariant.neutral,
                                    showDot: isActive,
                                  ),
                                ],
                              ),
                              const SizedBox(height: 6),
                              Text(
                                desc,
                                style: const TextStyle(
                                  color: ShadcnColors.foregroundMuted,
                                  fontSize: 12,
                                  height: 1.4,
                                ),
                              ),
                            ],
                          ),
                        ),
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
}
