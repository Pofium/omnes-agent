enum ChatMessageType { user, bot }

enum AgentActionType {
  readingFile,
  editingFile,
  analyzingCode,
  runningTerminal,
  searchingFiles,
}

enum TodoTaskStatus { completed, running, pending }

class TodoTaskItem {
  final String title;
  final TodoTaskStatus status;

  TodoTaskItem({
    required this.title,
    required this.status,
  });

  Map<String, dynamic> toJson() => {
    'title': title,
    'status': status.name,
  };

  factory TodoTaskItem.fromJson(Map<String, dynamic> json) => TodoTaskItem(
    title: json['title']?.toString() ?? '',
    status: TodoTaskStatus.values.firstWhere(
      (s) => s.name == json['status'],
      orElse: () => TodoTaskStatus.pending,
    ),
  );
}

class TodoBlockData {
  final String title;
  final int completedCount;
  final int totalCount;
  final List<TodoTaskItem> items;
  bool isExpanded;

  TodoBlockData({
    required this.title,
    required this.completedCount,
    required this.totalCount,
    required this.items,
    this.isExpanded = true,
  });

  Map<String, dynamic> toJson() => {
    'title': title,
    'completedCount': completedCount,
    'totalCount': totalCount,
    'items': items.map((i) => i.toJson()).toList(),
    'isExpanded': isExpanded,
  };

  factory TodoBlockData.fromJson(Map<String, dynamic> json) => TodoBlockData(
    title: json['title']?.toString() ?? '',
    completedCount: json['completedCount'] as int? ?? 0,
    totalCount: json['totalCount'] as int? ?? 0,
    items: (json['items'] as List?)
            ?.map((i) => TodoTaskItem.fromJson(Map<String, dynamic>.from(i as Map)))
            .toList() ??
        [],
    isExpanded: json['isExpanded'] != false,
  );
}

class AgentActionStep {
  final String title;
  final AgentActionType type;
  final String? filePath;
  String? details;
  bool isRunning;
  bool isError;
  bool isExpanded;
  int? additions;
  int? deletions;
  String? command;
  String? failureReason;

  AgentActionStep({
    required this.title,
    required this.type,
    this.filePath,
    this.details,
    this.isRunning = false,
    this.isError = false,
    this.isExpanded = false,
    this.additions,
    this.deletions,
    this.command,
    this.failureReason,
  });

  Map<String, dynamic> toJson() => {
    'title': title,
    'type': type.name,
    'filePath': filePath,
    'details': details,
    'isRunning': isRunning,
    'isError': isError,
    'isExpanded': isExpanded,
    'additions': additions,
    'deletions': deletions,
    'command': command,
    'failureReason': failureReason,
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
    additions: json['additions'] as int?,
    deletions: json['deletions'] as int?,
    command: json['command']?.toString(),
    failureReason: json['failureReason']?.toString(),
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
    List<TodoBlockData>? todoBlocks,
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
    this.metadata,
  })  : toolCalls = toolCalls ?? [],
        steps = steps ?? [],
        todoBlocks = todoBlocks ?? [],
        suggestedActions = suggestedActions ?? [];

  String text;
  final ChatMessageType chatMessageType;
  String? thinking;
  final List<ToolCallInfo> toolCalls;
  final List<AgentActionStep> steps;
  final List<TodoBlockData> todoBlocks;
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
  Map<String, dynamic>? metadata;

  Map<String, dynamic> toJson() => {
    'text': text,
    'isUser': chatMessageType == ChatMessageType.user,
    'thinking': thinking,
    'thinkingSeconds': thinkingSeconds,
    'isError': isError,
    'steps': steps.map((s) => s.toJson()).toList(),
    'todoBlocks': todoBlocks.map((t) => t.toJson()).toList(),
    'filesChangedCount': filesChangedCount,
    'additions': additions,
    'deletions': deletions,
    'selectedQuestionAnswer': selectedQuestionAnswer,
    'checkpointHash': checkpointHash,
    'suggestedActions': suggestedActions,
    'metadata': metadata,
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
    todoBlocks: (json['todoBlocks'] as List?)
            ?.map((t) => TodoBlockData.fromJson(Map<String, dynamic>.from(t as Map)))
            .toList() ??
        [],
    filesChangedCount: json['filesChangedCount'] as int?,
    additions: json['additions'] as int?,
    deletions: json['deletions'] as int?,
    selectedQuestionAnswer: json['selectedQuestionAnswer'] as String?,
    checkpointHash: json['checkpointHash'] as String?,
    suggestedActions: (json['suggestedActions'] as List?)?.map((e) => e.toString()).toList() ?? [],
    metadata: json['metadata'] is Map ? Map<String, dynamic>.from(json['metadata'] as Map) : null,
  );
}