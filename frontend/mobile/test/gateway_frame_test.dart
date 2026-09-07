import 'package:flutter_test/flutter_test.dart';
import 'package:omagent_front/core/gateway/models/gateway_frame.dart';
import 'package:omagent_front/core/gateway/gateway_config.dart';

void main() {
  group('GatewayFrame parsing', () {
    test('parses chunk frame', () {
      final json = {'type': 'chunk', 'content': 'Hello world'};
      final frame = GatewayFrame.fromJson(json);
      expect(frame, isA<ChunkFrame>());
      expect((frame as ChunkFrame).content, 'Hello world');
    });

    test('parses thinking frame', () {
      final json = {'type': 'thinking', 'content': 'I am reasoning...'};
      final frame = GatewayFrame.fromJson(json);
      expect(frame, isA<ThinkingFrame>());
      expect((frame as ThinkingFrame).content, 'I am reasoning...');
    });

    test('parses chunk_reset frame', () {
      final json = {'type': 'chunk_reset'};
      final frame = GatewayFrame.fromJson(json);
      expect(frame, isA<ChunkResetFrame>());
    });

    test('parses tool_call frame', () {
      final json = {
        'type': 'tool_call',
        'id': 'call_123',
        'name': 'fs_list',
        'args': {'path': '/workspace'},
      };
      final frame = GatewayFrame.fromJson(json);
      expect(frame, isA<ToolCallFrame>());
      final toolFrame = frame as ToolCallFrame;
      expect(toolFrame.id, 'call_123');
      expect(toolFrame.name, 'fs_list');
      expect(toolFrame.args['path'], '/workspace');
    });

    test('parses done frame', () {
      final json = {
        'type': 'done',
        'full_response': 'Full completed text',
        'max_context_tokens': 128000,
      };
      final frame = GatewayFrame.fromJson(json);
      expect(frame, isA<DoneFrame>());
      final done = frame as DoneFrame;
      expect(done.fullResponse, 'Full completed text');
      expect(done.maxContextTokens, 128000);
    });

    test('parses error frame', () {
      final json = {
        'type': 'error',
        'message': 'Failed to reach model provider',
        'code': 'PROVIDER_TIMEOUT',
      };
      final frame = GatewayFrame.fromJson(json);
      expect(frame, isA<ErrorFrame>());
      final err = frame as ErrorFrame;
      expect(err.message, 'Failed to reach model provider');
      expect(err.code, 'PROVIDER_TIMEOUT');
    });
  });

  group('GatewayConfig URLs', () {
    setUpAll(() {
      GatewayConfig.setMockOverrides(httpUrl: 'http://127.0.0.1:42617', agentAlias: 'chief');
    });

    test('converts http to ws URL', () {
      final wsUrl = GatewayConfig.getWsBaseUrl();
      expect(wsUrl.startsWith('ws://'), isTrue);
      expect(wsUrl.contains('42617'), isTrue);
    });
  });
}
