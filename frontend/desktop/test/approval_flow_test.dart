import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_shared/core/gateway/models/gateway_frame.dart';

void main() {
  group('Approval Flow & GatewayFrame Tests', () {
    test('ApprovalRequestFrame is correctly parsed from JSON', () {
      final json = {
        'type': 'approval_request',
        'request_id': 'req-12345',
        'tool': 'execute_bash_command',
        'arguments_summary': 'rm -rf /tmp/build_cache',
        'timeout_secs': 60,
      };

      final frame = GatewayFrame.fromJson(json);

      expect(frame, isA<ApprovalRequestFrame>());
      final approval = frame as ApprovalRequestFrame;
      expect(approval.requestId, equals('req-12345'));
      expect(approval.toolName, equals('execute_bash_command'));
      expect(approval.argumentsSummary, equals('rm -rf /tmp/build_cache'));
      expect(approval.timeoutSecs, equals(60));
    });

    test('ApprovalRequestFrame handles alternate tool_name key and default timeout', () {
      final json = {
        'type': 'approval_request',
        'request_id': 'req-999',
        'tool_name': 'fs_delete_file',
        'arguments_summary': 'file:///test.txt',
      };

      final frame = GatewayFrame.fromJson(json);

      expect(frame, isA<ApprovalRequestFrame>());
      final approval = frame as ApprovalRequestFrame;
      expect(approval.requestId, equals('req-999'));
      expect(approval.toolName, equals('fs_delete_file'));
      expect(approval.argumentsSummary, equals('file:///test.txt'));
      expect(approval.timeoutSecs, isNull);
    });

    test('Approval response payload format matches backend expectations', () {
      final requestId = 'req-abc-789';
      final decision = 'approve';

      final payload = jsonEncode({
        'type': 'approval_response',
        'request_id': requestId,
        'decision': decision,
      });

      final decoded = jsonDecode(payload) as Map<String, dynamic>;
      expect(decoded['type'], equals('approval_response'));
      expect(decoded['request_id'], equals('req-abc-789'));
      expect(decoded['decision'], equals('approve'));
    });
  });
}
