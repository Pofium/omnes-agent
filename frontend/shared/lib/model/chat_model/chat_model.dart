enum ChatMessageType { user, bot }

class ToolCallInfo {
  final String? id;
  final String name;
  final dynamic args;
  String? output;

  ToolCallInfo({
    this.id,
    required this.name,
    this.args,
    this.output,
  });
}

class ChatMessage {
  ChatMessage({
    required this.text,
    required this.chatMessageType,
    this.thinking,
    List<ToolCallInfo>? toolCalls,
    this.isStreaming = false,
    this.isError = false,
  }) : toolCalls = toolCalls ?? [];

  String text;
  final ChatMessageType chatMessageType;
  String? thinking;
  final List<ToolCallInfo> toolCalls;
  bool isStreaming;
  bool isError;
}