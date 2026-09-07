// Settings Screen for OmnesAgent: Security/PIN, LLM Providers, Gateway connection, TTS, and Language.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../core/gateway/gateway_config.dart';
import '../../routes/routes.dart';
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
        onBackClick: () => Get.back(),
      ),
      body: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 800),
            child: ListView(
              padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
              children: [
                // 1. LLM Providers Card
                _buildLlmProvidersBanner(context, isDark),
                SizedBox(height: Dimensions.heightSize * 1.5),

                // 2. Security & Lock
                _buildSectionHeader(
                  context,
                  Strings.securityAndLock.tr,
                  Icons.shield_outlined,
                ),
                SizedBox(height: Dimensions.heightSize * 0.8),
                _buildSecurityCard(context, isDark),
                SizedBox(height: Dimensions.heightSize * 1.5),

                // 3. Gateway Daemon Connection (Default Hardcoded with Advanced expansion)
                _buildSectionHeader(
                  context,
                  Strings.gatewaySettings.tr,
                  Icons.dns_rounded,
                ),
                SizedBox(height: Dimensions.heightSize * 0.8),
                _buildGatewayCard(context, isDark),
                SizedBox(height: Dimensions.heightSize * 1.5),

                // 4. Voice & Speech Synthesis
                _buildSectionHeader(
                  context,
                  Strings.voiceAndSpeech.tr,
                  Icons.record_voice_over_rounded,
                ),
                SizedBox(height: Dimensions.heightSize * 0.8),
                _buildVoiceCard(context, isDark),
                SizedBox(height: Dimensions.heightSize * 1.5),

                // 5. Language Selection
                _buildSectionHeader(
                  context,
                  Strings.appLanguage.tr,
                  Icons.language_rounded,
                ),
                SizedBox(height: Dimensions.heightSize * 0.8),
                _buildLanguageCard(context, isDark),
                SizedBox(height: Dimensions.heightSize * 2),

                // Save button
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
                  onPressed: () => controller.saveSettings(),
                ),
                SizedBox(height: Dimensions.heightSize * 2),
              ],
            ),
          ),
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

  Widget _buildLlmProvidersBanner(BuildContext context, bool isDark) {
    return InkWell(
      onTap: () => Get.toNamed(Routes.llmProvidersScreen),
      borderRadius: BorderRadius.circular(Dimensions.radius),
      child: Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          gradient: LinearGradient(
            colors: isDark
                ? [const Color(0xFF1E293B), const Color(0xFF0F172A)]
                : [const Color(0xFFEEF2FF), const Color(0xFFE0E7FF)],
          ),
          borderRadius: BorderRadius.circular(Dimensions.radius),
          border: Border.all(color: CustomColor.primaryColor.withOpacity(0.3)),
          boxShadow: [
            BoxShadow(
              color: CustomColor.primaryColor.withOpacity(0.08),
              blurRadius: 12,
              offset: const Offset(0, 4),
            ),
          ],
        ),
        child: Row(
          children: [
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: CustomColor.primaryColor.withOpacity(0.15),
                shape: BoxShape.circle,
              ),
              child: Icon(
                FontAwesomeIcons.robot,
                color: CustomColor.primaryColor,
                size: 22,
              ),
            ),
            const SizedBox(width: 14),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    Strings.llmProviders.tr,
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.bold,
                      color: isDark ? Colors.white : Colors.black87,
                    ),
                  ),
                  const SizedBox(height: 3),
                  Text(
                    Strings.llmProvidersSubtitle.tr,
                    style: TextStyle(
                      fontSize: 12,
                      color: isDark ? Colors.white60 : Colors.black54,
                    ),
                  ),
                ],
              ),
            ),
            const Icon(Icons.arrow_forward_ios, size: 16, color: Colors.grey),
          ],
        ),
      ),
    );
  }

  Widget _buildSecurityCard(BuildContext context, bool isDark) {
    return Container(
      padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
      decoration: BoxDecoration(
        color: isDark ? CustomColor.bgColor : Colors.white,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        border: Border.all(color: isDark ? Colors.white12 : Colors.black12),
      ),
      child: Column(
        children: [
          // PIN protection switch
          Obx(() => SwitchListTile(
                contentPadding: EdgeInsets.zero,
                secondary: Icon(
                  Icons.lock_outline_rounded,
                  color: CustomColor.primaryColor,
                ),
                title: Text(
                  Strings.pinProtection.tr,
                  style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
                ),
                subtitle: Text(
                  Strings.pinProtectionSubtitle.tr,
                  style: const TextStyle(fontSize: 12, color: Colors.grey),
                ),
                value: controller.isPinProtectionEnabled.value,
                onChanged: (val) => controller.togglePinProtection(val, context),
              )),

          // Set/Change PIN button if enabled
          Obx(() {
            if (!controller.isPinProtectionEnabled.value) return const SizedBox.shrink();
            return Padding(
              padding: const EdgeInsets.only(left: 40.0, top: 4.0, bottom: 8.0),
              child: Align(
                alignment: Alignment.centerLeft,
                child: TextButton.icon(
                  style: TextButton.styleFrom(
                    foregroundColor: CustomColor.primaryColor,
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                  ),
                  icon: const Icon(Icons.pin, size: 16),
                  label: Text(
                    Strings.setPinCode.tr,
                    style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold),
                  ),
                  onPressed: () => controller.showSetPinDialog(context),
                ),
              ),
            );
          }),

          const Divider(height: 16),

          // Biometrics switch
          Obx(() => SwitchListTile(
                contentPadding: EdgeInsets.zero,
                secondary: Icon(
                  Icons.fingerprint_rounded,
                  color: CustomColor.primaryColor,
                ),
                title: Text(
                  Strings.biometricAuth.tr,
                  style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
                ),
                subtitle: Text(
                  Strings.biometricAuthSubtitle.tr,
                  style: const TextStyle(fontSize: 12, color: Colors.grey),
                ),
                value: controller.isBiometricEnabled.value,
                onChanged: (val) => controller.toggleBiometrics(val),
              )),
        ],
      ),
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
          // Default connection overview
          Row(
            children: [
              Container(
                width: 10,
                height: 10,
                decoration: const BoxDecoration(
                  color: Colors.greenAccent,
                  shape: BoxShape.circle,
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      '127.0.0.1:42617',
                      style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
                    ),
                    Text(
                      Strings.gatewaySettingsSubtitle.tr,
                      style: const TextStyle(fontSize: 12, color: Colors.grey),
                    ),
                  ],
                ),
              ),
              Obx(() => OutlinedButton(
                    onPressed: controller.isTestingConnection.value
                        ? null
                        : controller.testConnection,
                    style: OutlinedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                      side: BorderSide(color: CustomColor.primaryColor.withOpacity(0.5)),
                    ),
                    child: controller.isTestingConnection.value
                        ? const SizedBox(
                            width: 14,
                            height: 14,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : Text(
                            Strings.testConnection.tr,
                            style: const TextStyle(fontSize: 11),
                          ),
                  )),
            ],
          ),

          const SizedBox(height: 10),

          // Collapsible Advanced network settings
          Obx(() => InkWell(
                onTap: () {
                  controller.isNetworkSettingsExpanded.value =
                      !controller.isNetworkSettingsExpanded.value;
                },
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  child: Row(
                    children: [
                      Icon(
                        controller.isNetworkSettingsExpanded.value
                            ? Icons.keyboard_arrow_up
                            : Icons.keyboard_arrow_down,
                        size: 18,
                        color: Colors.grey,
                      ),
                      const SizedBox(width: 6),
                      Text(
                        Strings.advancedNetworkSettings.tr,
                        style: const TextStyle(
                          fontSize: 12,
                          color: Colors.grey,
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                    ],
                  ),
                ),
              )),

          Obx(() {
            if (!controller.isNetworkSettingsExpanded.value) {
              return const SizedBox.shrink();
            }

            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const SizedBox(height: 12),
                TextField(
                  controller: controller.urlController,
                  decoration: InputDecoration(
                    labelText: Strings.baseUrl.tr,
                    hintText: GatewayConfig.defaultHttpUrl,
                    border: const OutlineInputBorder(),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                  ),
                ),
                SizedBox(height: Dimensions.heightSize * 0.8),
                TextField(
                  controller: controller.tokenController,
                  obscureText: true,
                  decoration: InputDecoration(
                    labelText: Strings.token.tr,
                    hintText: 'Bearer token',
                    border: const OutlineInputBorder(),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                  ),
                ),
                SizedBox(height: Dimensions.heightSize * 0.8),
                TextField(
                  controller: controller.agentController,
                  decoration: InputDecoration(
                    labelText: Strings.activeAgent.tr,
                    hintText: 'chief',
                    border: const OutlineInputBorder(),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                  ),
                ),
              ],
            );
          }),
        ],
      ),
    );
  }

  Widget _buildVoiceCard(BuildContext context, bool isDark) {
    return Container(
      padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
      decoration: BoxDecoration(
        color: isDark ? CustomColor.bgColor : Colors.white,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        border: Border.all(color: isDark ? Colors.white12 : Colors.black12),
      ),
      child: Obx(
        () => SwitchListTile(
          contentPadding: EdgeInsets.zero,
          secondary: Icon(
            Icons.volume_up_rounded,
            color: CustomColor.primaryColor,
          ),
          title: Text(
            Strings.autoTtsTitle.tr,
            style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
          ),
          subtitle: Text(
            Strings.autoTtsSubtitle.tr,
            style: const TextStyle(fontSize: 12, color: Colors.grey),
          ),
          value: controller.autoTts.value,
          onChanged: (val) => controller.toggleAutoTts(val),
        ),
      ),
    );
  }

  Widget _buildLanguageCard(BuildContext context, bool isDark) {
    return Container(
      padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
      decoration: BoxDecoration(
        color: isDark ? CustomColor.bgColor : Colors.white,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        border: Border.all(color: isDark ? Colors.white12 : Colors.black12),
      ),
      child: Obx(
        () => Column(
          children: [
            RadioListTile<String>(
              contentPadding: EdgeInsets.zero,
              title: const Text('Русский (Russian)', style: TextStyle(fontSize: 14)),
              value: 'ru',
              groupValue: controller.selectedLanguage.value,
              onChanged: (val) {
                if (val != null) controller.setLanguage(val);
              },
            ),
            const Divider(height: 8),
            RadioListTile<String>(
              contentPadding: EdgeInsets.zero,
              title: const Text('English', style: TextStyle(fontSize: 14)),
              value: 'en',
              groupValue: controller.selectedLanguage.value,
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
