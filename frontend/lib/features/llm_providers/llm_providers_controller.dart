// Controller for LLM Providers and Model Configuration.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:http/http.dart' as http;

import '../../core/gateway/gateway_http.dart';
import '../../helper/local_storage.dart';
import '../../utils/strings.dart';
import '../../widgets/api/toast_message.dart';

class LlmProvidersController extends GetxController {
  final GatewayHttpClient _http = GatewayHttpClient();

  final isLoading = false.obs;
  final isTesting = false.obs;
  final activeModel = 'anthropic/claude-3-5-sonnet'.obs;

  final catalogProviders = <Map<String, dynamic>>[].obs;
  final customProviders = <Map<String, dynamic>>[].obs;

  // Form controllers for custom provider
  final nameController = TextEditingController();
  final baseUrlController = TextEditingController();
  final apiKeyController = TextEditingController();
  final modelController = TextEditingController();
  final selectedFormat = 'OpenAI compatible'.obs;

  final List<String> wireFormats = [
    'OpenAI compatible',
    'Anthropic',
    'Ollama',
    'Custom REST',
  ];

  @override
  void onInit() {
    super.onInit();
    loadProviders();
  }

  Future<void> loadProviders() async {
    isLoading.value = true;
    try {
      // 1. Load custom providers from LocalStorage
      final savedCustom = LocalStorage.getCustomProviders();
      customProviders.assignAll(savedCustom);

      // 2. Fetch gateway catalog
      final catalog = await _http.getConfigCatalog();
      if (catalog != null && catalog['providers'] is List) {
        final list = (catalog['providers'] as List)
            .whereType<Map<String, dynamic>>()
            .toList();
        catalogProviders.assignAll(list);
      } else {
        // Fallback standard catalog
        catalogProviders.assignAll([
          {
            'name': 'OpenRouter',
            'type': 'openrouter',
            'doc_url': 'https://openrouter.ai',
            'models': ['anthropic/claude-3-5-sonnet', 'openai/gpt-4o', 'deepseek/deepseek-chat'],
            'is_configured': true,
          },
          {
            'name': 'Anthropic',
            'type': 'anthropic',
            'doc_url': 'https://anthropic.com',
            'models': ['claude-3-5-sonnet-20241022', 'claude-3-haiku-20240307'],
            'is_configured': false,
          },
          {
            'name': 'DeepSeek',
            'type': 'deepseek',
            'doc_url': 'https://deepseek.com',
            'models': ['deepseek-chat', 'deepseek-reasoner'],
            'is_configured': true,
          },
          {
            'name': 'OpenAI',
            'type': 'openai',
            'doc_url': 'https://platform.openai.com',
            'models': ['gpt-4o', 'gpt-4o-mini', 'o1-preview'],
            'is_configured': false,
          },
          {
            'name': 'Ollama (Local)',
            'type': 'ollama',
            'doc_url': 'http://localhost:11434',
            'models': ['llama3.2', 'qwen2.5-coder', 'mistral'],
            'is_configured': true,
          },
        ]);
      }

      // 3. Load active model if saved
      final savedModel = LocalStorage.getSelectedModel();
      if (savedModel.isNotEmpty) {
        activeModel.value = savedModel;
      }
    } finally {
      isLoading.value = false;
    }
  }

  void setActiveModel(String model) {
    activeModel.value = model;
    LocalStorage.saveSelectedModel(value: model);
    ToastMessage.success(Strings.providerSaved.tr);
  }

  Future<bool> testConnection() async {
    final url = baseUrlController.text.trim();
    if (url.isEmpty) {
      ToastMessage.error('Please enter Base URL');
      return false;
    }

    isTesting.value = true;
    try {
      final uri = Uri.parse(url);
      final headers = <String, String>{
        'Content-Type': 'application/json',
      };
      final key = apiKeyController.text.trim();
      if (key.isNotEmpty) {
        headers['Authorization'] = 'Bearer $key';
      }

      // Simple GET or OPTIONS check
      final response = await http
          .get(uri, headers: headers)
          .timeout(const Duration(seconds: 5));

      // Any HTTP response (including 401/404/200) means server is reachable
      if (response.statusCode < 500) {
        ToastMessage.success(Strings.connectionSuccess.tr);
        return true;
      } else {
        ToastMessage.error('${Strings.connectionFailed.tr}: HTTP ${response.statusCode}');
        return false;
      }
    } catch (e) {
      ToastMessage.error('${Strings.connectionFailed.tr}: $e');
      return false;
    } finally {
      isTesting.value = false;
    }
  }

  Future<void> saveCustomProvider() async {
    final name = nameController.text.trim();
    final url = baseUrlController.text.trim();
    final key = apiKeyController.text.trim();
    final model = modelController.text.trim();
    final format = selectedFormat.value;

    if (name.isEmpty || url.isEmpty || model.isEmpty) {
      ToastMessage.error('Please fill name, URL, and model');
      return;
    }

    final newProvider = {
      'id': DateTime.now().millisecondsSinceEpoch.toString(),
      'name': name,
      'base_url': url,
      'api_key': key,
      'model': model,
      'format': format,
      'created_at': DateTime.now().toIso8601String(),
    };

    customProviders.add(newProvider);
    await LocalStorage.saveCustomProviders(customProviders.toList());

    // Clear form
    nameController.clear();
    baseUrlController.clear();
    apiKeyController.clear();
    modelController.clear();

    ToastMessage.success(Strings.providerSaved.tr);
  }

  Future<void> deleteCustomProvider(String id) async {
    customProviders.removeWhere((p) => p['id'] == id);
    await LocalStorage.saveCustomProviders(customProviders.toList());
  }

  @override
  void onClose() {
    nameController.dispose();
    baseUrlController.dispose();
    apiKeyController.dispose();
    modelController.dispose();
    _http.dispose();
    super.onClose();
  }
}
