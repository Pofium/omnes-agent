import 'dart:async';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';

/// Coordinates Gateway WebSocket streaming and SSE event subscriptions for active sessions.
class WorkspaceStreamCoordinator {
  GatewayWsClient? _wsClient;
  StreamSubscription<GatewayFrame>? _wsSubscription;
  StreamSubscription<GatewaySystemEvent>? _sseSubscription;

  /// Connection status: 'disconnected', 'connecting', 'connected', 'error'
  final wsStatus = 'disconnected'.obs;

  final void Function(GatewayFrame frame)? onFrameReceived;
  final void Function(GatewaySystemEvent event)? onSseReceived;

  WorkspaceStreamCoordinator({
    this.onFrameReceived,
    this.onSseReceived,
  });

  bool get isConnected => _wsClient?.isConnected ?? false;

  /// Connects or reconnects the WebSocket stream to the given session.
  Future<void> connectWebSocket(String sessionId) async {
    await _wsSubscription?.cancel();
    _wsSubscription = null;
    await _wsClient?.disconnect();

    _wsClient = GatewayWsClient(
      sessionId: sessionId,
      agentAlias: GatewayConfig.getAgentAlias(),
    );

    wsStatus.value = 'connecting';

    _wsSubscription = _wsClient!.stream.listen(
      (frame) {
        if (frame is ConnectedFrame) {
          wsStatus.value = 'connected';
        } else if (frame is ErrorFrame) {
          wsStatus.value = 'error';
        }
        onFrameReceived?.call(frame);
      },
      onError: (err) {
        wsStatus.value = 'error';
      },
      onDone: () {
        wsStatus.value = 'disconnected';
      },
    );

    await _wsClient!.connect();
    if (_wsClient!.isConnected) {
      wsStatus.value = 'connected';
    }
  }

  /// Sends a message into the active WebSocket session.
  bool sendMessage(String text, {String? mode}) {
    if (_wsClient != null && _wsClient!.isConnected) {
      return _wsClient!.sendMessage(text, mode: mode);
    }
    return false;
  }

  /// Sends an abort signal to cancel current model generation.
  void abort() {
    _wsClient?.abort();
  }

  /// Sends HITL approval response (approve / deny) back to the gateway.
  void sendApprovalResponse(String requestId, String decision) {
    _wsClient?.sendApprovalResponse(requestId, decision);
  }

  /// Subscribes to global Server-Sent Events stream from the Gateway.
  void startSse() {
    try {
      GatewaySseClient.instance.connect();
      _sseSubscription?.cancel();
      _sseSubscription = GatewaySseClient.instance.events.listen((event) {
        onSseReceived?.call(event);
      });
    } catch (_) {}
  }

  /// Cancels SSE subscription.
  void stopSse() {
    _sseSubscription?.cancel();
    _sseSubscription = null;
  }

  /// Disconnects current WebSocket session.
  Future<void> disconnectWebSocket() async {
    await _wsSubscription?.cancel();
    _wsSubscription = null;
    await _wsClient?.disconnect();
    wsStatus.value = 'disconnected';
  }

  /// Cleans up all active subscriptions and sockets.
  void dispose() {
    stopSse();
    _wsSubscription?.cancel();
    _wsSubscription = null;
    _wsClient?.disconnect();
    _wsClient?.dispose();
    _wsClient = null;
  }
}
