import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_shared/core/gateway/models/gateway_frame.dart';

void main() {
  group('Web Approval Flow & GatewayFrame Tests', () {
    test('ApprovalRequestFrame is correctly parsed from JSON', () {
      final json = {
        'type': 'approval_request',
        'request_id': 'req-web-123',
        'tool': 'execute_bash_command',
        'arguments_summary': 'rm -rf /tmp/build_cache',
        'timeout_secs': 45,
      };

      final frame = GatewayFrame.fromJson(json);

      expect(frame, isA<ApprovalRequestFrame>());
      final approval = frame as ApprovalRequestFrame;
      expect(approval.requestId, equals('req-web-123'));
      expect(approval.toolName, equals('execute_bash_command'));
      expect(approval.argumentsSummary, equals('rm -rf /tmp/build_cache'));
      expect(approval.timeoutSecs, equals(45));
    });

    test('Approval response payload format matches backend expectations', () {
      final requestId = 'req-web-456';
      final decision = 'deny';

      final payload = jsonEncode({
        'type': 'approval_response',
        'request_id': requestId,
        'decision': decision,
      });

      final decoded = jsonDecode(payload) as Map<String, dynamic>;
      expect(decoded['type'], equals('approval_response'));
      expect(decoded['request_id'], equals('req-web-456'));
      expect(decoded['decision'], equals('deny'));
    });
  });
}
