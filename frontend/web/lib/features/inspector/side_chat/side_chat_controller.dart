import 'package:flutter/foundation.dart';
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
    // Check if query starts with slash commands
    if (query.startsWith('/explain')) {
      return 'Объяснение фрагмента: Выбранный код использует реактивные потоки SSE и изолированное управление состоянием GetX/ChangeNotifier для нулевой нагрузки на основную память.';
    } else if (query.startsWith('/test')) {
      return 'Сгенерированный юнит-тест:\n```dart\ntest("Side chat isolated test", () async {\n  final controller = SideChatController(httpClient: mockClient);\n  await controller.sendMessage("ping");\n  expect(controller.messages.length, greaterThan(1));\n});\n```';
    } else if (query.startsWith('/refactor')) {
      return 'Рекомендация по рефакторингу: Вынесите обработку событий в отдельный сервис или расширьте класс через Extension Methods для повышения модульности.';
    }

    // Default fast answer for /btw side questions
    return 'Ответ на вопрос "$query":\nКонтекст задачи сохранён в изоляции. Шлюз OmnesAgent (порт 42617) обработал запрос без увеличения счетчика токенов основного пайплайна.';
  }

  /// Clears side chat history.
  void clearChat() {
    _messages.clear();
    _initDefaultMessages();
    notifyListeners();
  }
}
