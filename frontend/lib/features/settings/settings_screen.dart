import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';
import '../../utils/strings.dart';
import '../../widgets/appbar/appbar_widget.dart';
import 'settings_controller.dart';

class SettingsScreen extends StatelessWidget {
  SettingsScreen({super.key});

  final controller = Get.put(SettingsController());

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Scaffold(
      appBar: AppBarWidget(
        context: context,
        appTitle: Strings.settings.tr,
        onPressed: () {},
        moreVisible: false,
        onBackClick: () {
          Get.back();
        },
      ),
      body: SafeArea(
        child: ListView(
          padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
          children: [
            _buildSectionHeader(context, 'Подключение к шлюзу ZeroClaw', Icons.dns_rounded),
            SizedBox(height: Dimensions.heightSize * 0.8),
            _buildGatewayCard(context, isDark),
            SizedBox(height: Dimensions.heightSize * 1.5),

            _buildSectionHeader(context, 'Голос и синтез речи', Icons.record_voice_over_rounded),
            SizedBox(height: Dimensions.heightSize * 0.8),
            _buildVoiceCard(context, isDark),
            SizedBox(height: Dimensions.heightSize * 1.5),

            _buildSectionHeader(context, 'Язык и локализация', Icons.language_rounded),
            SizedBox(height: Dimensions.heightSize * 0.8),
            _buildLanguageCard(context, isDark),
            SizedBox(height: Dimensions.heightSize * 2),

            ElevatedButton.icon(
              style: ElevatedButton.styleFrom(
                backgroundColor: CustomColor.primaryColor,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(vertical: 14),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(Dimensions.radius),
                ),
              ),
              icon: const Icon(Icons.save_rounded),
              label: Text(
                Strings.save.tr,
                style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
              ),
              onPressed: () {
                controller.saveSettings();
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title, IconData icon) {
    return Row(
      children: [
        Icon(icon, size: 18, color: CustomColor.primaryColor),
        const SizedBox(width: 8),
        Text(
          title,
          style: const TextStyle(fontSize: 15, fontWeight: FontWeight.bold),
        ),
      ],
    );
  }

  Widget _buildGatewayCard(BuildContext context, bool isDark) {
    return Container(
      padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
      decoration: BoxDecoration(
        color: isDark ? CustomColor.bgColor : Colors.white,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        border: Border.all(color: isDark ? Colors.white12 : Colors.black12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          TextField(
            controller: controller.urlController,
            decoration: const InputDecoration(
              labelText: 'URL шлюза (HTTP/WS)',
              hintText: 'http://127.0.0.1:42617',
              prefixIcon: Icon(Icons.link_rounded),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: controller.tokenController,
            obscureText: true,
            decoration: const InputDecoration(
              labelText: 'Bearer токен авторизации',
              hintText: 'omnes-token-secret-12345',
              prefixIcon: Icon(Icons.key_rounded),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: controller.agentController,
            decoration: const InputDecoration(
              labelText: 'Активный агент по умолчанию',
              hintText: 'chief / coder / writer',
              prefixIcon: Icon(Icons.smart_toy_rounded),
            ),
          ),
          const SizedBox(height: 14),
          Obx(
            () => SizedBox(
              width: double.infinity,
              child: OutlinedButton.icon(
                icon: controller.isTestingConnection.value
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.wifi_tethering_rounded, size: 18),
                label: Text(
                  controller.isTestingConnection.value
                      ? 'Проверка шлюза...'
                      : 'Проверить соединение',
                ),
                onPressed: controller.isTestingConnection.value
                    ? null
                    : () => controller.testConnection(),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildVoiceCard(BuildContext context, bool isDark) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: Dimensions.defaultPaddingSize,
        vertical: 8,
      ),
      decoration: BoxDecoration(
        color: isDark ? CustomColor.bgColor : Colors.white,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        border: Border.all(color: isDark ? Colors.white12 : Colors.black12),
      ),
      child: Obx(
        () => SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: const Text('Авто-озвучка ответов (TTS)'),
          subtitle: const Text('Озвучивать финальные ответы агента голосом'),
          value: controller.autoTts.value,
          activeColor: CustomColor.primaryColor,
          onChanged: (val) => controller.toggleAutoTts(val),
        ),
      ),
    );
  }

  Widget _buildLanguageCard(BuildContext context, bool isDark) {
    return Container(
      padding: EdgeInsets.symmetric(
        horizontal: Dimensions.defaultPaddingSize,
        vertical: 8,
      ),
      decoration: BoxDecoration(
        color: isDark ? CustomColor.bgColor : Colors.white,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        border: Border.all(color: isDark ? Colors.white12 : Colors.black12),
      ),
      child: Obx(
        () => Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text('Язык приложения:'),
            DropdownButton<String>(
              value: controller.selectedLanguage.value,
              underline: const SizedBox(),
              items: const [
                DropdownMenuItem(value: 'ru', child: Text('🇷🇺 Русский')),
                DropdownMenuItem(value: 'en', child: Text('🇬🇧 English')),
              ],
              onChanged: (val) {
                if (val != null) controller.setLanguage(val);
              },
            ),
          ],
        ),
      ),
    );
  }
}
