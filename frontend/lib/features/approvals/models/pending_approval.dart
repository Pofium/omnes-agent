// Model representing an approval gate waiting for human confirmation from /admin/sop/pending.

class PendingApproval {
  final String runId;
  final String sopName;
  final int step;
  final int totalSteps;
  final String kind;

  const PendingApproval({
    required this.runId,
    required this.sopName,
    required this.step,
    required this.totalSteps,
    required this.kind,
  });

  factory PendingApproval.fromJson(Map<String, dynamic> json) {
    return PendingApproval(
      runId: (json['run_id'] ?? '').toString(),
      sopName: (json['sop_name'] ?? 'Действие агента').toString(),
      step: json['step'] != null ? (json['step'] as num).toInt() : 1,
      totalSteps: json['total_steps'] != null ? (json['total_steps'] as num).toInt() : 1,
      kind: (json['kind'] ?? 'approval').toString(),
    );
  }
}
