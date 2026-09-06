// SettingsController for Omnes Agent: Gateway URL/token, active agent, voice TTS, and language.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import '../../helper/local_storage.dart';
import '../../widgets/api/toast_message.dart';

class SettingsController extends GetxController {
  final urlController = TextEditingController();
  final tokenController = TextEditingController();
  final agentController = TextEditingController();

  final isTestingConnection = false.obs;
  final autoTts = false.obs;
  final selectedLanguage = 'ru'.obs;

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

  Future<void> testConnection() async {
    isTestingConnection.value = true;
    try {
      final client = GatewayHttpClient();
      final ok = await client.checkHealth();
      client.dispose();

      if (ok) {
        ToastMessage.success("Шлюз Omnes доступен и отвечает!");
      } else {
        ToastMessage.error("Шлюз недоступен по указанному адресу.");
      }
    } catch (e) {
      ToastMessage.error("Ошибка подключения: $e");
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

    ToastMessage.success("Настройки успешно сохранены");
    Get.back();
  }
}