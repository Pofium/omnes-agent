// ignore_for_file: avoid_print
import 'dart:async';
import 'dart:convert';
import 'dart:io';

void main() async {
  print('--- Testing OmnesAgent Live Gateway (127.0.0.1:42617) ---');

  // 1. Test HTTP GET /health
  final client = HttpClient();
  final healthReq = await client.getUrl(Uri.parse('http://127.0.0.1:42617/health'));
  final healthRes = await healthReq.close();
  final healthBody = await utf8.decodeStream(healthRes);
  print('1. Health check: status=${healthRes.statusCode}, body=$healthBody');
  if (healthRes.statusCode != 200) {
    exitCode = 1;
    return;
  }

  // 2. Test HTTP GET /api/status with Bearer token
  final statusReq = await client.getUrl(Uri.parse('http://127.0.0.1:42617/api/status'));
  statusReq.headers.add('Authorization', 'Bearer omnes-token-secret-12345');
  final statusRes = await statusReq.close();
  final statusBody = await utf8.decodeStream(statusRes);
  print('2. Status check: status=${statusRes.statusCode}, version in body=${statusBody.contains("0.8.4")}');

  // 3. Test WebSocket /ws/chat
  final wsUrl = 'ws://127.0.0.1:42617/ws/chat?agent=chief&session_id=live_test_1&token=omnes-token-secret-12345';
  print('3. Connecting WebSocket to $wsUrl ...');
  final ws = await WebSocket.connect(wsUrl, protocols: ['omnesagent.v1', 'bearer.omnes-token-secret-12345']);
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
  print('4. Sending message: "Привет! Ответь коротко: Ты Omnes?"');
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
