import 'dart:async';
import 'dart:convert';
import 'package:get/get.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:omnes_shared/omnes_shared.dart';

/// Bidirectional Terminal Session over WebSocket for OmnesAgent Web ADE.
class TerminalSession {
  final String id;
  final String title;
  final WebSocketChannel? channel;
  final List<String> lines = [];
  final StreamController<String> outputController = StreamController<String>.broadcast();
  StreamSubscription? _sub;

  TerminalSession({
    required this.id,
    required this.title,
    this.channel,
  }) {
    if (channel != null) {
      _sub = channel!.stream.listen(
        (data) {
          try {
            if (data is String) {
              if (data.startsWith('{')) {
                final json = jsonDecode(data);
                if (json['type'] == 'stdout' || json['type'] == 'data') {
                  appendLine(json['data']?.toString() ?? '');
                  return;
                } else if (json['type'] == 'exit') {
                  appendLine('[Process exited with code ${json['code']}]');
                  return;
                }
              }
              appendLine(data);
            }
          } catch (_) {
            appendLine(data.toString());
          }
        },
        onError: (err) => appendLine('[PTY WS ERROR] $err'),
        onDone: () => appendLine('[PTY WS connection closed]'),
      );
    }
  }

  void appendLine(String line) {
    lines.add(line);
    if (lines.length > 2000) {
      lines.removeAt(0);
    }
    outputController.add(line);
  }

  void writeInput(String input) {
    if (channel != null) {
      try {
        final payload = jsonEncode({
          'type': 'stdin',
          'data': '$input\n',
        });
        channel!.sink.add(payload);
      } catch (e) {
        appendLine('[PTY SEND ERROR] $e');
      }
    } else {
      appendLine('[Web Terminal Standby] Command received: $input');
    }
  }

  void kill() {
    if (channel != null) {
      try {
        channel!.sink.add(jsonEncode({'type': 'signal', 'signal': 'SIGINT'}));
      } catch (_) {}
    }
  }

  void dispose() {
    kill();
    _sub?.cancel();
    channel?.sink.close();
    outputController.close();
  }
}

/// Web-compatible Terminal Service communicating with Gateway PTY WebSocket.
class WebTerminalService extends GetxController {
  static WebTerminalService get to => Get.isRegistered<WebTerminalService>()
      ? Get.find<WebTerminalService>()
      : Get.put(WebTerminalService());

  final sessions = <TerminalSession>[].obs;
  final activeSessionIndex = 0.obs;

  TerminalSession? get activeSession =>
      sessions.isNotEmpty && activeSessionIndex.value < sessions.length
          ? sessions[activeSessionIndex.value]
          : null;

  @override
  void onInit() {
    super.onInit();
    startNewSession(title: '1: Web Terminal');
  }

  /// Starts a new PTY session over WebSocket.
  Future<TerminalSession?> startNewSession({String? title, String? workingDir}) async {
    try {
      final id = DateTime.now().millisecondsSinceEpoch.toString();
      final sessionTitle = title ?? '${sessions.length + 1}: Remote Shell';

      // Connect to Gateway PTY endpoint
      final wsBaseUrl = GatewayConfig.getWsBaseUrl();
      final wsUri = Uri.parse('$wsBaseUrl/ws/terminal/$id');

      WebSocketChannel? channel;
      try {
        channel = WebSocketChannel.connect(wsUri);
      } catch (_) {
        // Fallback to local session emulation if gateway is offline
      }

      final session = TerminalSession(
        id: id,
        title: sessionTitle,
        channel: channel,
      );

      session.appendLine('OmnesAgent Web ADE Terminal connected via PTY-over-WebSocket');
      session.appendLine('Gateway Endpoint: $wsUri');
      session.appendLine('Session ID: $id');
      session.appendLine('---');

      sessions.add(session);
      activeSessionIndex.value = sessions.length - 1;
      return session;
    } catch (e) {
      Get.snackbar('Ошибка запуска терминала', '$e');
      return null;
    }
  }

  /// Sends a command line to the active session.
  void sendCommand(String cmd) {
    final session = activeSession;
    if (session == null) return;
    session.appendLine('> $cmd');
    session.writeInput(cmd);
  }

  /// Interrupts active process.
  void interruptActiveSession() {
    activeSession?.kill();
    activeSession?.appendLine('^C [Process interrupted]');
  }

  /// Clears lines in active session.
  void clearActiveSession() {
    activeSession?.lines.clear();
  }

  /// Closes session.
  void closeSession(int index) {
    if (index >= 0 && index < sessions.length) {
      final s = sessions.removeAt(index);
      s.dispose();
      if (activeSessionIndex.value >= sessions.length) {
        activeSessionIndex.value = (sessions.length - 1).clamp(0, 999);
      }
    }
  }

  @override
  void onClose() {
    for (final s in sessions) {
      s.dispose();
    }
    sessions.clear();
    super.onClose();
  }
}
