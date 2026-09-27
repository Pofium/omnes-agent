import 'dart:convert';

/// Represents an execution trajectory step emitted by the agent runtime.
class TrajectoryStep {
  final String id;
  final String sessionId;
  final String turnId;
  final int stepIndex;
  final String? parentStepId;
  final String stepType; // 's1_gate', 'context_inject', 'cot_thinking', 'tool_call', 'tool_observation', 'synthesis'
  final String payloadJson;
  final int tokensUsed;
  final int durationUs;
  final String createdAt;

  const TrajectoryStep({
    required this.id,
    required this.sessionId,
    required this.turnId,
    required this.stepIndex,
    this.parentStepId,
    required this.stepType,
    required this.payloadJson,
    this.tokensUsed = 0,
    this.durationUs = 0,
    required this.createdAt,
  });

  factory TrajectoryStep.fromJson(Map<String, dynamic> json) {
    return TrajectoryStep(
      id: json['id'] as String? ?? '',
      sessionId: json['session_id'] as String? ?? '',
      turnId: json['turn_id'] as String? ?? '',
      stepIndex: json['step_index'] as int? ?? 0,
      parentStepId: json['parent_step_id'] as String?,
      stepType: json['step_type'] as String? ?? 'synthesis',
      payloadJson: json['payload_json'] as String? ??
          (json['payload'] != null ? jsonEncode(json['payload']) : '{}'),
      tokensUsed: json['tokens_used'] as int? ?? 0,
      durationUs: json['duration_us'] as int? ?? 0,
      createdAt: json['created_at'] as String? ?? '',
    );
  }

  Map<String, dynamic> toJson() => {
    'id': id,
    'session_id': sessionId,
    'turn_id': turnId,
    'step_index': stepIndex,
    if (parentStepId != null) 'parent_step_id': parentStepId,
    'step_type': stepType,
    'payload_json': payloadJson,
    'tokens_used': tokensUsed,
    'duration_us': durationUs,
    'created_at': createdAt,
  };
}
