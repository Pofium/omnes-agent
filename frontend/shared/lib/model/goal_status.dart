// Goal Mode tracking model for ZCode ADE.

class GoalChecklistItem {
  final String id;
  final String title;
  final int iteration;
  bool isCompleted;

  GoalChecklistItem({
    required this.id,
    required this.title,
    required this.iteration,
    this.isCompleted = false,
  });

  Map<String, dynamic> toJson() => {
        'id': id,
        'title': title,
        'iteration': iteration,
        'is_completed': isCompleted,
      };

  factory GoalChecklistItem.fromJson(Map<String, dynamic> json) =>
      GoalChecklistItem(
        id: json['id'] as String? ?? '',
        title: json['title'] as String? ?? '',
        iteration: json['iteration'] as int? ?? 1,
        isCompleted: json['is_completed'] as bool? ?? false,
      );
}

class GoalStatus {
  final String objective;
  final int elapsedSeconds;
  final int currentIteration;
  final int maxIterations;
  final bool isPaused;
  final bool isCompleted;
  final List<GoalChecklistItem> checklist;

  GoalStatus({
    required this.objective,
    this.elapsedSeconds = 0,
    this.currentIteration = 1,
    this.maxIterations = 10,
    this.isPaused = false,
    this.isCompleted = false,
    this.checklist = const [],
  });

  String get formattedElapsed {
    final minutes = elapsedSeconds ~/ 60;
    final seconds = elapsedSeconds % 60;
    return '${minutes}m ${seconds.toString().padLeft(2, '0')}s';
  }

  GoalStatus copyWith({
    String? objective,
    int? elapsedSeconds,
    int? currentIteration,
    int? maxIterations,
    bool? isPaused,
    bool? isCompleted,
    List<GoalChecklistItem>? checklist,
  }) {
    return GoalStatus(
      objective: objective ?? this.objective,
      elapsedSeconds: elapsedSeconds ?? this.elapsedSeconds,
      currentIteration: currentIteration ?? this.currentIteration,
      maxIterations: maxIterations ?? this.maxIterations,
      isPaused: isPaused ?? this.isPaused,
      isCompleted: isCompleted ?? this.isCompleted,
      checklist: checklist ?? this.checklist,
    );
  }
}
