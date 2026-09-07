// Models for OmnesAgent/ZeroClaw WebSocket Gateway frames.
// Mirrors web/src/types/api.ts and web/src/contexts/turnStream.logic.ts

/// Base sealed class for all frames received over /ws/chat.
sealed class GatewayFrame {
  final String type;
  const GatewayFrame(this.type);

  factory GatewayFrame.fromJson(Map<String, dynamic> json) {
    final type = json['type'] as String? ?? 'unknown';
    switch (type) {
      case 'chunk':
        return ChunkFrame(
          content: json['content'] as String? ?? '',
        );
      case 'thinking':
        return ThinkingFrame(
          content: json['content'] as String? ?? '',
        );
      case 'chunk_reset':
        return const ChunkResetFrame();
      case 'tool_call':
        return ToolCallFrame(
          id: json['id'] as String?,
          name: json['name'] as String? ?? '',
          args: json['args'],
        );
      case 'tool_result':
        return ToolResultFrame(
          id: json['id'] as String?,
          name: json['name'] as String?,
          output: json['output'] as String? ?? '',
        );
      case 'done':
        return DoneFrame(
          fullResponse: json['full_response'] as String? ?? '',
          content: json['content'] as String?,
          maxContextTokens: json['max_context_tokens'] as int?,
        );
      case 'message':
        return MessageFrame(
          content: json['content'] as String? ?? '',
          fullResponse: json['full_response'] as String?,
        );
      case 'aborted':
        return AbortedFrame(
          reason: json['reason'] as String?,
        );
      case 'error':
        return ErrorFrame(
          message: json['message'] as String? ?? json['error'] as String? ?? 'Unknown error',
          code: json['code'] as String?,
        );
      case 'session_start':
      case 'connected':
        return ConnectedFrame(
          sessionId: json['session_id'] as String?,
          resumed: json['resumed'] as bool? ?? false,
        );
      default:
        return UnknownFrame(type, json);
    }
  }
}

/// A delta chunk of the model response.
class ChunkFrame extends GatewayFrame {
  final String content;
  const ChunkFrame({required this.content}) : super('chunk');
}

/// A delta chunk of reasoning / chain-of-thought.
class ThinkingFrame extends GatewayFrame {
  final String content;
  const ThinkingFrame({required this.content}) : super('thinking');
}

/// Signals that prior chunk deltas are being replaced by an authoritative answer.
class ChunkResetFrame extends GatewayFrame {
  const ChunkResetFrame() : super('chunk_reset');
}

/// Tool invocation initiated by the agent.
class ToolCallFrame extends GatewayFrame {
  final String? id;
  final String name;
  final dynamic args;
  const ToolCallFrame({this.id, required this.name, this.args}) : super('tool_call');
}

/// Result of a tool execution.
class ToolResultFrame extends GatewayFrame {
  final String? id;
  final String? name;
  final String output;
  const ToolResultFrame({this.id, this.name, required this.output}) : super('tool_result');
}

/// Terminal completion frame of a turn.
class DoneFrame extends GatewayFrame {
  final String fullResponse;
  final String? content;
  final int? maxContextTokens;
  const DoneFrame({required this.fullResponse, this.content, this.maxContextTokens}) : super('done');
}

/// Complete assistant message frame.
class MessageFrame extends GatewayFrame {
  final String content;
  final String? fullResponse;
  const MessageFrame({required this.content, this.fullResponse}) : super('message');
}

/// Generation was aborted by client or system.
class AbortedFrame extends GatewayFrame {
  final String? reason;
  const AbortedFrame({this.reason}) : super('aborted');
}

/// An error reported by the gateway.
class ErrorFrame extends GatewayFrame {
  final String message;
  final String? code;
  const ErrorFrame({required this.message, this.code}) : super('error');
}

/// Connected/handshake frame.
class ConnectedFrame extends GatewayFrame {
  final String? sessionId;
  final bool resumed;
  const ConnectedFrame({this.sessionId, required this.resumed}) : super('connected');
}

/// Fallback for unhandled or experimental frame types.
class UnknownFrame extends GatewayFrame {
  final Map<String, dynamic> raw;
  const UnknownFrame(super.type, this.raw);
}
