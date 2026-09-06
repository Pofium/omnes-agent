// WebSocket client for OmnesAgent Gateway (/ws/chat).
// Implements streaming protocol matching web/src/lib/ws.ts and web/src/contexts/turnStream.logic.ts.

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'gateway_config.dart';
import 'models/gateway_frame.dart';

class GatewayWsClient {
  WebSocket? _socket;
  StreamController<GatewayFrame>? _streamController;
  Timer? _reconnectTimer;
  bool _intentionallyClosed = false;
  bool _isConnecting = false;

  final String _agentAlias;
  final String _sessionId;
  final String? _workspaceDir;

  Stream<GatewayFrame> get stream => _streamController?.stream ?? const Stream.empty();
  bool get isConnected => _socket != null && _socket!.readyState == WebSocket.open;
  String get sessionId => _sessionId;
  String get agentAlias => _agentAlias;

  GatewayWsClient({
    String? agentAlias,
    String? sessionId,
    String? workspaceDir,
  })  : _agentAlias = agentAlias ?? GatewayConfig.getAgentAlias(),
        _sessionId = sessionId ?? DateTime.now().millisecondsSinceEpoch.toString(),
        _workspaceDir = workspaceDir {
    _streamController = StreamController<GatewayFrame>.broadcast();
  }

  /// Connects to gateway /ws/chat.
  Future<void> connect() async {
    if (isConnected || _isConnecting) return;
    _isConnecting = true;
    _intentionallyClosed = false;
    _cancelReconnect();

    try {
      final token = await GatewayConfig.getToken();
      final wsBase = GatewayConfig.getWsBaseUrl();

      final queryParams = <String, String>{
        'agent': _agentAlias,
        'session_id': _sessionId,
        if (token.isNotEmpty) 'token': token,
        if (_workspaceDir != null && _workspaceDir.isNotEmpty) 'workspaceDir': _workspaceDir,
      };

      final queryString = queryParams.entries
          .map((e) => '${Uri.encodeComponent(e.key)}=${Uri.encodeComponent(e.value)}')
          .join('&');

      final uri = Uri.parse('$wsBase/ws/chat?$queryString');

      final protocols = <String>['omnesagent.v1'];
      if (token.isNotEmpty) {
        protocols.add('bearer.$token');
      }

      _socket = await WebSocket.connect(
        uri.toString(),
        protocols: protocols,
      ).timeout(const Duration(seconds: 10));

      _socket!.listen(
        _onData,
        onError: _onError,
        onDone: _onDone,
        cancelOnError: false,
      );

      _isConnecting = false;
    } catch (e) {
      _isConnecting = false;
      _streamController?.add(ErrorFrame(message: 'Connection failed: $e'));
      _scheduleReconnect();
    }
  }

  void _onData(dynamic data) {
    if (data is! String) return;
    try {
      final json = jsonDecode(data);
      if (json is Map<String, dynamic>) {
        final frame = GatewayFrame.fromJson(json);
        _streamController?.add(frame);
      }
    } catch (e) {
      // Ignore non-JSON or malformed frames
    }
  }

  void _onError(dynamic error) {
    _streamController?.add(ErrorFrame(message: error.toString()));
  }

  void _onDone() {
    _socket = null;
    if (!_intentionallyClosed) {
      _scheduleReconnect();
    }
  }

  /// Sends a text message to the agent turn loop.
  bool sendMessage(String content) {
    if (!isConnected) return false;
    final payload = jsonEncode({
      'type': 'message',
      'content': content,
    });
    _socket!.add(payload);
    return true;
  }

  /// Aborts active generation over WebSocket.
  bool abort() {
    if (!isConnected) return false;
    final payload = jsonEncode({
      'type': 'abort',
    });
    _socket!.add(payload);
    return true;
  }

  void _scheduleReconnect() {
    if (_intentionallyClosed) return;
    _cancelReconnect();
    _reconnectTimer = Timer(const Duration(seconds: 3), () {
      if (!_intentionallyClosed && !isConnected) {
        connect();
      }
    });
  }

  void _cancelReconnect() {
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
  }

  /// Closes the connection intentionally.
  Future<void> disconnect() async {
    _intentionallyClosed = true;
    _cancelReconnect();
    try {
      await _socket?.close();
    } catch (_) {}
    _socket = null;
  }

  void dispose() {
    disconnect();
    _streamController?.close();
  }
}
