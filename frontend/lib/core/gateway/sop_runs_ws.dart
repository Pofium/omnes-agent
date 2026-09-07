// Real-time SOP Runs feed WebSocket client for OmnesAgent Gateway (/ws/sops/runs).

import 'dart:async';
import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'gateway_config.dart';

class SopRunEvent {
  final String runId;
  final String sopName;
  final String status;
  final String? currentStep;
  final Map<String, dynamic>? extra;
  final DateTime timestamp;

  SopRunEvent({
    required this.runId,
    required this.sopName,
    required this.status,
    this.currentStep,
    this.extra,
    required this.timestamp,
  });

  factory SopRunEvent.fromJson(Map<String, dynamic> json) {
    return SopRunEvent(
      runId: json['run_id']?.toString() ?? json['id']?.toString() ?? '',
      sopName: json['sop_name']?.toString() ?? json['name']?.toString() ?? '',
      status: json['status']?.toString() ?? 'running',
      currentStep: json['current_step']?.toString(),
      extra: json['extra'] is Map<String, dynamic> ? json['extra'] as Map<String, dynamic> : null,
      timestamp: json['timestamp'] != null
          ? DateTime.tryParse(json['timestamp'].toString()) ?? DateTime.now()
          : DateTime.now(),
    );
  }
}

class SopRunsWsClient {
  WebSocketChannel? _channel;
  StreamSubscription? _subscription;
  StreamController<SopRunEvent>? _streamController;
  Timer? _reconnectTimer;
  bool _isConnected = false;
  bool _intentionallyClosed = false;

  Stream<SopRunEvent> get stream => _streamController?.stream ?? const Stream.empty();
  bool get isConnected => _isConnected && _channel != null;

  SopRunsWsClient() {
    _streamController = StreamController<SopRunEvent>.broadcast();
  }

  /// Connects to gateway /ws/sops/runs.
  Future<void> connect() async {
    if (isConnected) return;
    _intentionallyClosed = false;

    try {
      final token = await GatewayConfig.getToken();
      final wsBase = GatewayConfig.getWsBaseUrl();
      final query = token.isNotEmpty ? '?token=${Uri.encodeComponent(token)}' : '';
      final uri = Uri.parse('$wsBase/ws/sops/runs$query');

      _channel = WebSocketChannel.connect(
        uri,
        protocols: ['omnesagent.v1'],
      );

      _subscription = _channel?.stream.listen(
        (data) {
          _isConnected = true;
          _handleMessage(data);
        },
        onError: (_) {
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
        if (json['type'] == 'disabled') return;
        _streamController?.add(SopRunEvent.fromJson(json));
      }
    } catch (_) {}
  }

  void _scheduleReconnect() {
    if (_intentionallyClosed) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 5), () {
      connect();
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
