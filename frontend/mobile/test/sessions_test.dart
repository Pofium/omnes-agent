import 'package:flutter_test/flutter_test.dart';
import 'package:omagent_front/features/sessions/models/session_info.dart';

void main() {
  group('Session models parsing', () {
    test('parses SessionInfo from JSON', () {
      final json = {
        'session_id': 'session_abc',
        'agent_alias': 'chief',
        'created_at': '2026-09-05T10:00:00.000Z',
        'last_activity': '2026-09-05T10:15:00.000Z',
        'message_count': 5,
      };

      final session = SessionInfo.fromJson(json);
      expect(session.sessionId, 'session_abc');
      expect(session.agentAlias, 'chief');
      expect(session.messageCount, 5);
      expect(session.createdAt.isUtc, isTrue);
      expect(session.lastActivity.isUtc, isTrue);
    });

    test('parses SessionHistoryMessage from JSON', () {
      final userJson = {
        'role': 'user',
        'content': 'Hello agent',
        'created_at': '2026-09-05T10:00:00.000Z',
      };
      final userMsg = SessionHistoryMessage.fromJson(userJson);
      expect(userMsg.role, 'user');
      expect(userMsg.content, 'Hello agent');

      final assistantJson = {
        'role': 'assistant',
        'content': 'Hello user! I am chief.',
      };
      final botMsg = SessionHistoryMessage.fromJson(assistantJson);
      expect(botMsg.role, 'assistant');
      expect(botMsg.content, 'Hello user! I am chief.');
    });
  });
}
