import 'package:omnes_shared/omnes_shared.dart';

/// Represents an independent chat session in OmnesAgent ADE
class TaskSession {
  final String id;
  String title;
  String group;
  String? project;
  String? projectPath;
  String? branch;
  List<ChatMessage> messages;
  PermissionMode permissionMode;
  AgentExecutionMode executionMode;
  String thoughtLevel;
  String activeModel;
  String tokenCount;
  GoalStatus? activeGoal;
  List<Map<String, dynamic>> runTimelineSteps;
  bool isTerminalOpen;
  int additions;
  int deletions;
  bool hasGitRepo;
  bool isCustomNamed;

  TaskSession({
    required this.id,
    required this.title,
    this.group = 'Разное',
    this.project,
    this.projectPath,
    this.branch,
    List<ChatMessage>? messages,
    this.permissionMode = PermissionMode.fullAccess,
    this.executionMode = AgentExecutionMode.fast,
    this.thoughtLevel = 'Max',
    this.activeModel = 'GLM-5.3-Flash',
    this.tokenCount = '28.4k tokens',
    this.activeGoal,
    List<Map<String, dynamic>>? runTimelineSteps,
    this.isTerminalOpen = false,
    this.additions = 0,
    this.deletions = 0,
    this.hasGitRepo = false,
    this.isCustomNamed = false,
  })  : messages = messages ?? [],
        runTimelineSteps = runTimelineSteps ?? [];
}
