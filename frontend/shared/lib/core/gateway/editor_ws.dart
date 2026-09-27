// WebSocket client for the editor channel /ws/editor/{buffer_id}.
// BACKEND_SPEC.md §9.9, FRONTEND_SPEC.md §3.3 (EditorWsClient).
// Mirrors the connection discipline of gateway_ws.dart (bearer subprotocol,
// auto-reconnect), but carries editor frames only.

import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

import 'gateway_config.dart';
import '../../model/editor/editor_frames.dart';

class EditorWsClient {
  WebSocketChannel? _channel;
  StreamSubscription? _subscription;
  StreamController<EditorFrame>? _streamController;
  Timer? _reconnectTimer;
  bool _isConnected = false;
  bool _intentionallyClosed = false;
  bool _isConnecting = false;

  final int bufferId;

  Stream<EditorFrame> get stream => _streamController?.stream ?? const Stream.empty();
  bool get isConnected => _isConnected && _channel != null;

  EditorWsClient({required this.bufferId}) {
    _streamController = StreamController<EditorFrame>.broadcast();
  }

  Future<void> connect() async {
    if (isConnected || _isConnecting) return;
    _isConnecting = true;
    _intentionallyClosed = false;
    _cancelReconnect();

    try {
      final token = await GatewayConfig.getToken();
      final wsBase = GatewayConfig.getWsBaseUrl();
      final queryParams = <String, String>{
        if (token.isNotEmpty) 'token': token,
      };
      final query = queryParams.entries
          .map((e) => '${Uri.encodeComponent(e.key)}=${Uri.encodeComponent(e.value)}')
          .join('&');
      final uri = Uri.parse('$wsBase/ws/editor/$bufferId${query.isNotEmpty ? '?$query' : ''}');

      final protocols = <String>['omnesagent.v1'];
      if (token.isNotEmpty) {
        protocols.add('bearer.$token');
      }

      _channel = WebSocketChannel.connect(uri, protocols: protocols);
      await _channel!.ready.timeout(const Duration(seconds: 10));
      _isConnected = true;
      _isConnecting = false;

      _subscription = _channel!.stream.listen(
        _onData,
        onError: _onError,
        onDone: _onDone,
        cancelOnError: false,
      );
    } catch (e) {
      _isConnected = false;
      _isConnecting = false;
      _streamController?.add(EditorFrame(
        type: 'error',
        code: 'connection_failed',
        message: '$e',
      ));
      _scheduleReconnect();
    }
  }

  void _onData(dynamic data) {
    if (data is! String) return;
    try {
      final json = jsonDecode(data);
      if (json is Map<String, dynamic>) {
        _streamController?.add(EditorFrame.fromJson(json));
      }
    } catch (_) {
      // Ignore malformed frames; `rows_request` heals any desync (§9.9 inv. 3).
    }
  }

  void _onError(dynamic error) {
    _isConnected = false;
    _streamController?.add(EditorFrame(
      type: 'error',
      code: 'connection_error',
      message: '$error',
    ));
  }

  void _onDone() {
    _isConnected = false;
    _channel = null;
    _subscription?.cancel();
    _subscription = null;
    if (!_intentionallyClosed) {
      _scheduleReconnect();
    }
  }

  /// Sends a raw frame; returns false when the socket is down (the caller
  /// buffers or blocks — FRONTEND_SPEC §3.3 "буферизация при реконнекте").
  bool send(Map<String, dynamic> frame) {
    if (!isConnected) return false;
    try {
      _channel!.sink.add(jsonEncode(frame));
      return true;
    } catch (_) {
      return false;
    }
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

  Future<void> disconnect() async {
    _intentionallyClosed = true;
    _isConnected = false;
    _cancelReconnect();
    try {
      await _subscription?.cancel();
      await _channel?.sink.close();
    } catch (_) {}
    _subscription = null;
    _channel = null;
  }

  void dispose() {
    disconnect();
    _streamController?.close();
    _streamController = null;
  }
}
