// Real-time Canvas WebSocket client for OmnesAgent Gateway (/ws/canvas/{id}).

import 'dart:async';
import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'gateway_config.dart';

class CanvasFrame {
  final String canvasId;
  final String contentType;
  final String content;
  final int version;

  CanvasFrame({
    required this.canvasId,
    required this.contentType,
    required this.content,
    required this.version,
  });

  factory CanvasFrame.fromJson(Map<String, dynamic> json) {
    return CanvasFrame(
      canvasId: json['canvas_id']?.toString() ?? '',
      contentType: json['content_type']?.toString() ?? 'html',
      content: json['content']?.toString() ?? '',
      version: json['version'] is int ? json['version'] as int : 1,
    );
  }
}

class CanvasWsClient {
  WebSocketChannel? _channel;
  StreamSubscription? _subscription;
  StreamController<CanvasFrame>? _streamController;
  Timer? _reconnectTimer;
  bool _isConnected = false;
  bool _intentionallyClosed = false;

  final String canvasId;

  Stream<CanvasFrame> get stream => _streamController?.stream ?? const Stream.empty();
  bool get isConnected => _isConnected && _channel != null;

  CanvasWsClient({required this.canvasId}) {
    _streamController = StreamController<CanvasFrame>.broadcast();
  }

  /// Connects to gateway /ws/canvas/{canvasId}.
  Future<void> connect() async {
    if (isConnected) return;
    _intentionallyClosed = false;

    try {
      final token = await GatewayConfig.getToken();
      final wsBase = GatewayConfig.getWsBaseUrl();
      final query = token.isNotEmpty ? '?token=${Uri.encodeComponent(token)}' : '';
      final uri = Uri.parse('$wsBase/ws/canvas/${Uri.encodeComponent(canvasId)}$query');

      _channel = WebSocketChannel.connect(
        uri,
        protocols: ['omnesagent.v1'],
      );

      _subscription = _channel?.stream.listen(
        (data) {
          _isConnected = true;
          _handleMessage(data);
        },
        onError: (err) {
          _isConnected = false;
          _scheduleReconnect();
        },
        onDone: () {
          _isConnected = false;
          if (!_intentionallyClosed) {
            _scheduleReconnect();
          }
        },
      );
    } catch (_) {
      _isConnected = false;
      _scheduleReconnect();
    }
  }

  void _handleMessage(dynamic data) {
    try {
      final text = data is String ? data : utf8.decode(data as List<int>);
      final json = jsonDecode(text);
      if (json is Map<String, dynamic>) {
        if (json.containsKey('frame') && json['frame'] is Map) {
          _streamController?.add(CanvasFrame.fromJson(json['frame'] as Map<String, dynamic>));
        } else {
          _streamController?.add(CanvasFrame.fromJson(json));
        }
      }
    } catch (_) {}
  }

  /// Sends action event back to canvas runner.
  void sendAction(String action, Map<String, dynamic> payload) {
    if (!isConnected) return;
    try {
      final msg = jsonEncode({
        'type': 'action',
        'action': action,
        'payload': payload,
      });
      _channel?.sink.add(msg);
    } catch (_) {}
  }

  void _scheduleReconnect() {
    if (_intentionallyClosed) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 3), () {
      if (!_intentionallyClosed) {
        connect();
      }
    });
  }

  void disconnect() {
    _intentionallyClosed = true;
    _reconnectTimer?.cancel();
    _subscription?.cancel();
    _channel?.sink.close();
    _channel = null;
    _isConnected = false;
  }

  void dispose() {
    disconnect();
    _streamController?.close();
  }
}
