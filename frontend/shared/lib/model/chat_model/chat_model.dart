enum ChatMessageType { user, bot }

enum AgentActionType {
  readingFile,
  editingFile,
  analyzingCode,
  runningTerminal,
  searchingFiles,
}

class AgentActionStep {
  final String title;
  final AgentActionType type;
  final String? filePath;
  String? details;
  bool isRunning;
  bool isError;
  bool isExpanded;

  AgentActionStep({
    required this.title,
    required this.type,
    this.filePath,
    this.details,
    this.isRunning = false,
    this.isError = false,
    this.isExpanded = false,
  });

  Map<String, dynamic> toJson() => {
    'title': title,
    'type': type.name,
    'filePath': filePath,
    'details': details,
    'isRunning': isRunning,
    'isError': isError,
    'isExpanded': isExpanded,
  };

  factory AgentActionStep.fromJson(Map<String, dynamic> json) => AgentActionStep(
    title: json['title']?.toString() ?? '',
    type: AgentActionType.values.firstWhere(
      (e) => e.name == json['type'],
      orElse: () => AgentActionType.analyzingCode,
    ),
    filePath: json['filePath']?.toString(),
    details: json['details']?.toString(),
    isRunning: json['isRunning'] == true,
    isError: json['isError'] == true,
    isExpanded: json['isExpanded'] == true,
  );
}

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

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'args': args,
    'output': output,
  };

  factory ToolCallInfo.fromJson(Map<String, dynamic> json) => ToolCallInfo(
    id: json['id']?.toString(),
    name: json['name']?.toString() ?? '',
    args: json['args'],
    output: json['output']?.toString(),
  );
}

class ChatMessage {
  ChatMessage({
    required this.text,
    required this.chatMessageType,
    this.thinking,
    List<ToolCallInfo>? toolCalls,
    List<AgentActionStep>? steps,
    this.isStreaming = false,
    this.isError = false,
    this.thinkingSeconds = 0,
    this.isThinkingFinished = false,
    this.isThinkingExpanded = false,
    this.filesChangedCount,
    this.additions,
    this.deletions,
    this.selectedQuestionAnswer,
    this.checkpointHash,
    List<String>? suggestedActions,
  })  : toolCalls = toolCalls ?? [],
        steps = steps ?? [],
        suggestedActions = suggestedActions ?? [];

  String text;
  final ChatMessageType chatMessageType;
  String? thinking;
  final List<ToolCallInfo> toolCalls;
  final List<AgentActionStep> steps;
  bool isStreaming;
  bool isError;
  int thinkingSeconds;
  bool isThinkingFinished;
  bool isThinkingExpanded;
  int? filesChangedCount;
  int? additions;
  int? deletions;
  String? selectedQuestionAnswer;
  String? checkpointHash;
  List<String> suggestedActions;

  Map<String, dynamic> toJson() => {
    'text': text,
    'isUser': chatMessageType == ChatMessageType.user,
    'thinking': thinking,
    'thinkingSeconds': thinkingSeconds,
    'isError': isError,
    'steps': steps.map((s) => s.toJson()).toList(),
    'filesChangedCount': filesChangedCount,
    'additions': additions,
    'deletions': deletions,
    'selectedQuestionAnswer': selectedQuestionAnswer,
    'checkpointHash': checkpointHash,
    'suggestedActions': suggestedActions,
  };

  factory ChatMessage.fromJson(Map<String, dynamic> json) => ChatMessage(
    text: json['text']?.toString() ?? '',
    chatMessageType: json['isUser'] == true ? ChatMessageType.user : ChatMessageType.bot,
    thinking: json['thinking']?.toString(),
    thinkingSeconds: json['thinkingSeconds'] is int ? json['thinkingSeconds'] as int : 0,
    isError: json['isError'] == true,
    isThinkingFinished: true,
    isThinkingExpanded: false,
    steps: (json['steps'] as List?)
            ?.map((s) => AgentActionStep.fromJson(Map<String, dynamic>.from(s as Map)))
            .toList() ??
        [],
    filesChangedCount: json['filesChangedCount'] as int?,
    additions: json['additions'] as int?,
    deletions: json['deletions'] as int?,
    selectedQuestionAnswer: json['selectedQuestionAnswer'] as String?,
    checkpointHash: json['checkpointHash'] as String?,
    suggestedActions: (json['suggestedActions'] as List?)?.map((e) => e.toString()).toList() ?? [],
  );
}