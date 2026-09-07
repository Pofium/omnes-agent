// Server-Sent Events (SSE) client for OmnesAgent Gateway (/api/events).

import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'gateway_config.dart';

/// Represents a parsed SSE event from OmnesAgent Gateway.
class GatewaySystemEvent {
  final String type;
  final String? sessionId;
  final Map<String, dynamic> data;
  final DateTime timestamp;

  GatewaySystemEvent({
    required this.type,
    this.sessionId,
    required this.data,
    DateTime? timestamp,
  }) : timestamp = timestamp ?? DateTime.now();

  factory GatewaySystemEvent.fromJson(Map<String, dynamic> json) {
    return GatewaySystemEvent(
      type: json['type']?.toString() ?? 'system',
      sessionId: json['session_id']?.toString(),
      data: json,
      timestamp: json['timestamp'] != null
          ? DateTime.tryParse(json['timestamp'].toString()) ?? DateTime.now()
          : DateTime.now(),
    );
  }
}

/// Global SSE client managing the persistent connection to /api/events.
class GatewaySseClient {
  http.Client? _client;
  StreamSubscription? _subscription;
  final StreamController<GatewaySystemEvent> _controller =
      StreamController<GatewaySystemEvent>.broadcast();
  Timer? _reconnectTimer;
  bool _isConnected = false;
  bool _disposed = false;

  Stream<GatewaySystemEvent> get events => _controller.stream;
  bool get isConnected => _isConnected;

  static final GatewaySseClient instance = GatewaySseClient._internal();
  GatewaySseClient._internal();

  factory GatewaySseClient() => instance;

  /// Starts listening to /api/events.
  Future<void> connect() async {
    if (_isConnected || _disposed) return;
    _cancelReconnect();

    try {
      _client = http.Client();
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/events');
      final token = await GatewayConfig.getToken();

      final req = http.Request('GET', uri);
      req.headers['Accept'] = 'text/event-stream';
      req.headers['Cache-Control'] = 'no-cache';
      if (token.isNotEmpty) {
        req.headers['Authorization'] = 'Bearer $token';
      }

      final res = await _client!.send(req);
      if (res.statusCode != 200) {
        _scheduleReconnect();
        return;
      }

      _isConnected = true;

      // Stream lines
      _subscription = res.stream
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .listen(
        _onLine,
        onError: (err) {
          _isConnected = false;
          _scheduleReconnect();
        },
        onDone: () {
          _isConnected = false;
          _scheduleReconnect();
        },
        cancelOnError: true,
      );
    } catch (_) {
      _isConnected = false;
      _scheduleReconnect();
    }
  }

  void _onLine(String line) {
    final trimmed = line.trim();
    if (trimmed.isEmpty || trimmed.startsWith(':')) return; // Comments / keepalive

    if (trimmed.startsWith('data:')) {
      final jsonStr = trimmed.substring(5).trim();
      if (jsonStr.isEmpty) return;
      try {
        final decoded = jsonDecode(jsonStr);
        if (decoded is Map<String, dynamic>) {
          _controller.add(GatewaySystemEvent.fromJson(decoded));
        }
      } catch (_) {
        // Ignore unparseable frames
      }
    }
  }

  void _scheduleReconnect() {
    if (_disposed) return;
    _cancelReconnect();
    _reconnectTimer = Timer(const Duration(seconds: 4), () {
      if (!_disposed && !_isConnected) {
        connect();
      }
    });
  }

  void _cancelReconnect() {
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
  }

  /// Disconnects the SSE stream.
  void disconnect() {
    _cancelReconnect();
    _isConnected = false;
    _subscription?.cancel();
    _subscription = null;
    _client?.close();
    _client = null;
  }

  void dispose() {
    _disposed = true;
    disconnect();
    _controller.close();
  }
}
