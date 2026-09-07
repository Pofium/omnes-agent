// Real-time Node Discovery WebSocket client for OmnesAgent Gateway (/ws/nodes).

import 'dart:async';
import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'gateway_config.dart';

class NodePeerInfo {
  final String nodeId;
  final String? alias;
  final String status;
  final int latencyMs;
  final List<String> capabilities;
  final int activeSessions;
  final DateTime lastSeen;

  NodePeerInfo({
    required this.nodeId,
    this.alias,
    required this.status,
    required this.latencyMs,
    required this.capabilities,
    required this.activeSessions,
    required this.lastSeen,
  });

  factory NodePeerInfo.fromJson(Map<String, dynamic> json) {
    return NodePeerInfo(
      nodeId: json['node_id']?.toString() ?? json['id']?.toString() ?? '',
      alias: json['alias']?.toString() ?? json['name']?.toString(),
      status: json['status']?.toString() ?? 'online',
      latencyMs: json['latency_ms'] is int ? json['latency_ms'] as int : 0,
      capabilities: (json['capabilities'] is List)
          ? (json['capabilities'] as List).map((e) => e.toString()).toList()
          : [],
      activeSessions: json['active_sessions'] is int ? json['active_sessions'] as int : 0,
      lastSeen: json['last_seen'] != null
          ? DateTime.tryParse(json['last_seen'].toString()) ?? DateTime.now()
          : DateTime.now(),
    );
  }
}

class NodesWsClient {
  WebSocketChannel? _channel;
  StreamSubscription? _subscription;
  StreamController<List<NodePeerInfo>>? _streamController;
  Timer? _reconnectTimer;
  bool _isConnected = false;
  bool _intentionallyClosed = false;

  final Map<String, NodePeerInfo> _peers = {};

  Stream<List<NodePeerInfo>> get stream => _streamController?.stream ?? const Stream.empty();
  List<NodePeerInfo> get currentPeers => _peers.values.toList();
  bool get isConnected => _isConnected && _channel != null;

  NodesWsClient() {
    _streamController = StreamController<List<NodePeerInfo>>.broadcast();
  }

  /// Connects to gateway /ws/nodes.
  Future<void> connect() async {
    if (isConnected) return;
    _intentionallyClosed = false;

    try {
      final token = await GatewayConfig.getToken();
      final wsBase = GatewayConfig.getWsBaseUrl();
      final query = token.isNotEmpty ? '?token=${Uri.encodeComponent(token)}' : '';
      final uri = Uri.parse('$wsBase/ws/nodes$query');

      _channel = WebSocketChannel.connect(
        uri,
        protocols: ['omnesagent.node.v1', 'omnesagent.v1'],
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
        if (json['type'] == 'nodes_snapshot' && json['nodes'] is List) {
          _peers.clear();
          for (final n in json['nodes']) {
            if (n is Map<String, dynamic>) {
              final peer = NodePeerInfo.fromJson(n);
              _peers[peer.nodeId] = peer;
            }
          }
        } else if (json['type'] == 'node_joined' || json['type'] == 'node_heartbeat') {
          final peer = NodePeerInfo.fromJson(json);
          _peers[peer.nodeId] = peer;
        } else if (json['type'] == 'node_left') {
          final id = json['node_id']?.toString();
          if (id != null) _peers.remove(id);
        } else {
          final peer = NodePeerInfo.fromJson(json);
          if (peer.nodeId.isNotEmpty) {
            _peers[peer.nodeId] = peer;
          }
        }
        _streamController?.add(_peers.values.toList());
      } else if (json is List) {
        _peers.clear();
        for (final n in json) {
          if (n is Map<String, dynamic>) {
            final peer = NodePeerInfo.fromJson(n);
            _peers[peer.nodeId] = peer;
          }
        }
        _streamController?.add(_peers.values.toList());
      }
    } catch (_) {}
  }

  void _scheduleReconnect() {
    if (_intentionallyClosed) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 4), () {
      connect();
    });
  }

  /// Sends a peer invocation message.
  void sendInvocation(String targetNodeId, String capability, Map<String, dynamic> params) {
    if (!isConnected) return;
    final msg = jsonEncode({
      'type': 'invoke',
      'target': targetNodeId,
      'capability': capability,
      'params': params,
    });
    _channel?.sink.add(msg);
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
