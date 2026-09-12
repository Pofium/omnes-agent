import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:get_storage/get_storage.dart';
import 'package:http/http.dart' as http;
import 'package:omnes_shared/omnes_shared.dart';

/// Message in the isolated Side Chat session.
class SideChatMessage {
  final String text;
  final bool isUser;
  final DateTime timestamp;

  SideChatMessage({
    required this.text,
    required this.isUser,
    DateTime? timestamp,
  }) : timestamp = timestamp ?? DateTime.now();
}

/// Controller managing isolated Side Chat (/side, /btw) conversations.
class SideChatController extends ChangeNotifier {
  final GatewayHttpClient httpClient;
  final List<SideChatMessage> _messages = [];
  bool _isLoading = false;

  SideChatController({required this.httpClient}) {
    _initDefaultMessages();
  }

  List<SideChatMessage> get messages => List.unmodifiable(_messages);
  bool get isLoading => _isLoading;

  void _initDefaultMessages() {
    _messages.addAll([
      SideChatMessage(
        text: 'Привет! Я вспомогательный Side Chat. Задавай вопросы по коду или архитектуре (/btw), я отвечу изолированно от контекста основной задачи.',
        isUser: false,
      ),
    ]);
  }

  /// Sends a message without polluting the main task session.
  Future<void> sendMessage(String text) async {
    final query = text.trim();
    if (query.isEmpty) return;

    _messages.add(SideChatMessage(text: query, isUser: true));
    _isLoading = true;
    notifyListeners();

    try {
      // Use quick question / btw endpoint or direct tool query
      final answer = await _queryIsolatedResponse(query);
      _messages.add(SideChatMessage(text: answer, isUser: false));
    } catch (e) {
      _messages.add(SideChatMessage(
        text: 'Ошибка получения ответа шлюза: $e',
        isUser: false,
      ));
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<String> _queryIsolatedResponse(String query) async {
    try {
      final storage = GetStorage();
      final activeProvId = storage.read<String>('selected_active_provider') ?? 'deepseek';
      final activeModel = storage.read<String>('selected_active_model') ?? 'deepseek-chat';

      String key = (storage.read<String>('provider_key_$activeProvId') ?? '').trim();
      String url = (storage.read<String>('provider_url_$activeProvId') ?? '').trim();

      if (key.isEmpty) {
        final savedCustom = storage.read<List>('custom_providers_registry') ?? [];
        for (final item in savedCustom) {
          if (item is Map && item['id'] == activeProvId) {
            key = (item['key']?.toString() ?? '').trim();
            if (url.isEmpty) url = (item['url']?.toString() ?? '').trim();
          }
        }
      }

      if (url.isEmpty) {
        if (activeProvId == 'deepseek') {
          url = 'https://api.deepseek.com/v1';
        } else if (activeProvId == 'openai') {
          url = 'https://api.openai.com/v1';
        } else if (activeProvId == 'ollama') {
          url = 'http://localhost:11434/v1';
        } else {
          url = 'https://api.deepseek.com/v1';
        }
      }

      if (key.isEmpty && activeProvId != 'ollama') {
        return 'Ответ на вопрос "$query":\nПровайдер не настроен. Укажите API ключ в настройках для получения ответов в Side Chat.';
      }

      final sanitizedBase = url.endsWith('/') ? url.substring(0, url.length - 1) : url;
      final endpoint = sanitizedBase.endsWith('/chat/completions') ? sanitizedBase : '$sanitizedBase/chat/completions';

      final history = <Map<String, String>>[
        {
          'role': 'system',
          'content': 'Ты — изолированный Side Chat в OmnesAgent. Отвечай кратко, ёмко, по существу на русском языке. Ответ должен быть лаконичным.',
        },
      ];

      for (final m in _messages.take(6)) {
        history.add({
          'role': m.isUser ? 'user' : 'assistant',
          'content': m.text,
        });
      }

      final resp = await http.post(
        Uri.parse(endpoint),
        headers: {
          'Content-Type': 'application/json; charset=utf-8',
          if (key.isNotEmpty) 'Authorization': 'Bearer $key',
        },
        body: jsonEncode({
          'model': activeModel.isNotEmpty ? activeModel : 'default',
          'messages': history,
          'stream': false,
        }),
      ).timeout(const Duration(seconds: 15));

      if (resp.statusCode == 200) {
        final data = jsonDecode(utf8.decode(resp.bodyBytes));
        final content = data['choices']?[0]?['message']?['content']?.toString() ?? '';
        return content.replaceAll(RegExp(r'<think>[\s\S]*?</think>'), '').trim();
      } else {
        return 'Ошибка шлюза (${resp.statusCode}): ${resp.body}';
      }
    } catch (e) {
      return 'Не удалось связаться с моделью: $e';
    }
  }

  /// Adds a branch context message from the main chat.
  void addBranchContext(String contextText) {
    final snippet = contextText.length > 300 ? '${contextText.substring(0, 300)}...' : contextText;
    _messages.add(SideChatMessage(
      text: 'Ветка ответа ассистента:\n"$snippet"\n\nКонтекст перенесен в Side Chat. Задайте вопрос или команду по этой ветке.',
      isUser: false,
    ));
    notifyListeners();
  }

  /// Clears side chat history.
  void clearChat() {
    _messages.clear();
    _initDefaultMessages();
    notifyListeners();
  }
}
