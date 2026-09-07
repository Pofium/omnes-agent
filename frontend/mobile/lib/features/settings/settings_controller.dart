// SettingsController for Omnes Agent: Gateway URL/token, active agent, voice TTS, security/PIN, and language.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import '../../helper/local_storage.dart';
import '../../utils/strings.dart';
import '../../widgets/api/toast_message.dart';

class SettingsController extends GetxController {
  final urlController = TextEditingController();
  final tokenController = TextEditingController();
  final agentController = TextEditingController();

  final isTestingConnection = false.obs;
  final autoTts = false.obs;
  final selectedLanguage = 'ru'.obs;

  // Security & PIN protection
  final isPinProtectionEnabled = false.obs;
  final isBiometricEnabled = false.obs;
  final hasPinSet = false.obs;

  // Expandable network settings toggle
  final isNetworkSettingsExpanded = false.obs;

  @override
  void onInit() {
    super.onInit();
    urlController.text = GatewayConfig.getBaseUrl();
    agentController.text = GatewayConfig.getAgentAlias();
    autoTts.value = LocalStorage.getAutoTts();

    final langList = LocalStorage.getLanguage();
    if (langList.isNotEmpty) {
      selectedLanguage.value = langList[0].toString();
    }

    GatewayConfig.getToken().then((token) {
      tokenController.text = token;
    });

    isPinProtectionEnabled.value = LocalStorage.isPinProtectionEnabled();
    hasPinSet.value = LocalStorage.getPinCode().isNotEmpty;
    isBiometricEnabled.value = LocalStorage.isBiometricAuthEnabled();
  }

  @override
  void onClose() {
    urlController.dispose();
    tokenController.dispose();
    agentController.dispose();
    super.onClose();
  }

  Future<void> toggleAutoTts(bool value) async {
    autoTts.value = value;
    await LocalStorage.saveAutoTts(value: value);
  }

  Future<void> setLanguage(String lang) async {
    selectedLanguage.value = lang;
    if (lang == 'ru') {
      await LocalStorage.saveLanguage(
        langSmall: 'ru',
        langCap: 'RU',
        languageName: 'Russian',
      );
      Get.updateLocale(const Locale('ru', 'RU'));
    } else {
      await LocalStorage.saveLanguage(
        langSmall: 'en',
        langCap: 'US',
        languageName: 'English',
      );
      Get.updateLocale(const Locale('en', 'US'));
    }
  }

  Future<void> togglePinProtection(bool value, BuildContext context) async {
    if (value && !hasPinSet.value) {
      // Must set PIN first
      showSetPinDialog(context);
    } else {
      isPinProtectionEnabled.value = value;
      await LocalStorage.savePinProtection(value: value);
      if (!value) {
        ToastMessage.success(Strings.pinDisabled.tr);
      }
    }
  }

  Future<void> toggleBiometrics(bool value) async {
    isBiometricEnabled.value = value;
    await LocalStorage.saveBiometricAuth(value: value);
  }

  void showSetPinDialog(BuildContext context) {
    final pinCtrl = TextEditingController();
    final confirmCtrl = TextEditingController();

    Get.dialog(
      AlertDialog(
        title: Text(Strings.setPinCode.tr),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: pinCtrl,
              keyboardType: TextInputType.number,
              obscureText: true,
              maxLength: 6,
              decoration: InputDecoration(
                labelText: Strings.enterNewPin.tr,
                counterText: '',
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: confirmCtrl,
              keyboardType: TextInputType.number,
              obscureText: true,
              maxLength: 6,
              decoration: InputDecoration(
                labelText: Strings.confirmPin.tr,
                counterText: '',
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Get.back(),
            child: Text(Strings.cancel.tr),
          ),
          ElevatedButton(
            onPressed: () async {
              final pin = pinCtrl.text.trim();
              final confirm = confirmCtrl.text.trim();
              if (pin.length < 4) {
                ToastMessage.error('PIN must be 4-6 digits');
                return;
              }
              if (pin != confirm) {
                ToastMessage.error(Strings.pinsDoNotMatch.tr);
                return;
              }
              await LocalStorage.savePinCode(pin: pin);
              await LocalStorage.savePinProtection(value: true);
              hasPinSet.value = true;
              isPinProtectionEnabled.value = true;
              Get.back();
              ToastMessage.success(Strings.pinSaved.tr);
            },
            child: Text(Strings.save.tr),
          ),
        ],
      ),
    );
  }

  Future<void> testConnection() async {
    isTestingConnection.value = true;
    try {
      final client = GatewayHttpClient();
      final ok = await client.checkHealth();
      client.dispose();

      if (ok) {
        ToastMessage.success(Strings.connectionSuccess.tr);
      } else {
        ToastMessage.error(Strings.connectionFailed.tr);
      }
    } catch (e) {
      ToastMessage.error("${Strings.connectionFailed.tr}: $e");
    } finally {
      isTestingConnection.value = false;
    }
  }

  Future<void> saveSettings() async {
    final url = urlController.text.trim();
    final token = tokenController.text.trim();
    final agent = agentController.text.trim();

    if (url.isNotEmpty) {
      await GatewayConfig.setBaseUrl(url);
    }
    await GatewayConfig.setToken(token);
    if (agent.isNotEmpty) {
      await GatewayConfig.setAgentAlias(agent);
    }

    await LocalStorage.saveAutoTts(value: autoTts.value);

    ToastMessage.success(Strings.save.tr);
    Get.back();
  }
}