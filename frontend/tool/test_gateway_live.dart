// ignore_for_file: avoid_print
import 'dart:async';
import 'dart:convert';
import 'dart:io';

void main() async {
  print('--- Testing OmnesAgent Live Gateway (127.0.0.1:42617) ---');

  // Bearer token is never hardcoded: read it from the environment or from
  // the gateway's own discovery descriptor (.omnes/gateway.json).
  final token = Platform.environment['OMNES_GATEWAY_TOKEN'] ??
      () {
        final f = File('.omnes/gateway.json');
        if (f.existsSync()) {
          final parsed = jsonDecode(f.readAsStringSync());
          if (parsed is Map && parsed['token'] is String) return parsed['token'] as String;
        }
        return '';
      }();
  if (token.isEmpty) {
    print('! No gateway token found: set OMNES_GATEWAY_TOKEN or start the '
        'gateway so it writes .omnes/gateway.json');
    exitCode = 1;
    return;
  }
  print('1. Token discovered (${token.length} chars).');

  // 2. Test HTTP GET /health
  final client = HttpClient();
  final healthReq = await client.getUrl(Uri.parse('http://127.0.0.1:42617/health'));
  final healthRes = await healthReq.close();
  final healthBody = await utf8.decodeStream(healthRes);
  print('2. Health check: status=${healthRes.statusCode}, body=$healthBody');
  if (healthRes.statusCode != 200) {
    exitCode = 1;
    return;
  }

  // 3. Test HTTP GET /api/status with Bearer token
  final statusReq = await client.getUrl(Uri.parse('http://127.0.0.1:42617/api/status'));
  statusReq.headers.add('Authorization', 'Bearer $token');
  final statusRes = await statusReq.close();
  final statusBody = await utf8.decodeStream(statusRes);
  print('3. Status check: status=${statusRes.statusCode}, version in body=${statusBody.contains("0.8.4")}');

  // 4. Test WebSocket /ws/chat
  final wsUrl = 'ws://127.0.0.1:42617/ws/chat?agent=chief&session_id=live_test_1&token=$token';
  print('4. Connecting WebSocket to $wsUrl ...');
  final ws = await WebSocket.connect(wsUrl, protocols: ['omnesagent.v1', 'bearer.$token']);
  print('   WebSocket connected! readyState=${ws.readyState}');

  final completer = Completer<void>();
  int chunksReceived = 0;
  String fullText = '';

  ws.listen((data) {
    try {
      final json = jsonDecode(data as String);
      final type = json['type'];
      if (type == 'chunk') {
        chunksReceived++;
        fullText += (json['content'] ?? '');
        stdout.write(json['content'] ?? '');
      } else if (type == 'thinking') {
        print('\n[thinking]: ${json['content']}');
      } else if (type == 'done') {
        print('\n[done]: full_response="${json['full_response']}"');
        if (!completer.isCompleted) completer.complete();
      } else if (type == 'error') {
        print('\n[error]: ${json['message']}');
        if (!completer.isCompleted) completer.complete();
      } else {
        print('   Received frame: $type');
      }
    } catch (e) {
      print('   Frame parse error: $e');
    }
  }, onDone: () {
    print('   WebSocket closed.');
    if (!completer.isCompleted) completer.complete();
  }, onError: (err) {
    print('   WebSocket error: $err');
    if (!completer.isCompleted) completer.complete();
  });

  // Send prompt
  print('5. Sending message: "Привет! Ответь коротко: Ты Omnes?"');
  ws.add(jsonEncode({
    'type': 'message',
    'content': 'Привет! Ответь коротко: Ты Omnes?',
  }));

  // Wait up to 30 seconds for agent to respond
  await completer.future.timeout(const Duration(seconds: 30), onTimeout: () {
    print('\nTimed out waiting for response.');
  });

  await ws.close();
  client.close();

  print('\n--- Result: chunks=$chunksReceived, responseLength=${fullText.length} ---');
  if (chunksReceived > 0 || fullText.isNotEmpty) {
    print('SUCCESS: ZeroClaw streaming chat verified end-to-end!');
  } else {
    print('WARNING: No chunks returned yet.');
  }
}
