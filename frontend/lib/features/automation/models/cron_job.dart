// Models for Cron Jobs and Execution Runs from /api/cron.

class CronJob {
  final String id;
  final String name;
  final String expression;
  final String command;
  final String? prompt;
  final String agentAlias;
  final bool enabled;
  final DateTime? nextRun;
  final DateTime? lastRun;
  final String? lastStatus;
  final String? lastOutput;

  const CronJob({
    required this.id,
    required this.name,
    required this.expression,
    this.command = '',
    this.prompt,
    required this.agentAlias,
    this.enabled = true,
    this.nextRun,
    this.lastRun,
    this.lastStatus,
    this.lastOutput,
  });

  String get displayName => name.isNotEmpty ? name : (prompt != null && prompt!.isNotEmpty ? prompt! : expression);

  factory CronJob.fromJson(Map<String, dynamic> json) {
    DateTime? parseDate(dynamic val) {
      if (val is String) {
        return DateTime.tryParse(val);
      }
      return null;
    }

    String expr = '';
    if (json['expression'] != null) {
      expr = json['expression'].toString();
    } else if (json['schedule'] != null) {
      final s = json['schedule'];
      if (s is Map && s['cron'] != null) {
        expr = s['cron'].toString();
      } else if (s is String) {
        expr = s;
      }
    }

    return CronJob(
      id: (json['id'] ?? '').toString(),
      name: (json['name'] ?? '').toString(),
      expression: expr,
      command: (json['command'] ?? '').toString(),
      prompt: json['prompt']?.toString(),
      agentAlias: (json['agent_alias'] ?? 'chief').toString(),
      enabled: json['enabled'] != false,
      nextRun: parseDate(json['next_run']),
      lastRun: parseDate(json['last_run']),
      lastStatus: json['last_status']?.toString(),
      lastOutput: json['last_output']?.toString(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'expression': expression,
      'command': command,
      if (prompt != null) 'prompt': prompt,
      'agent_alias': agentAlias,
      'enabled': enabled,
      if (nextRun != null) 'next_run': nextRun!.toIso8601String(),
      if (lastRun != null) 'last_run': lastRun!.toIso8601String(),
      if (lastStatus != null) 'last_status': lastStatus,
      if (lastOutput != null) 'last_output': lastOutput,
    };
  }

  CronJob copyWith({
    bool? enabled,
    String? lastStatus,
    DateTime? lastRun,
  }) {
    return CronJob(
      id: id,
      name: name,
      expression: expression,
      command: command,
      prompt: prompt,
      agentAlias: agentAlias,
      enabled: enabled ?? this.enabled,
      nextRun: nextRun,
      lastRun: lastRun ?? this.lastRun,
      lastStatus: lastStatus ?? this.lastStatus,
      lastOutput: lastOutput,
    );
  }
}

class CronRun {
  final int id;
  final String jobId;
  final DateTime startedAt;
  final DateTime finishedAt;
  final String status;
  final String? output;
  final int? durationMs;

  const CronRun({
    required this.id,
    required this.jobId,
    required this.startedAt,
    required this.finishedAt,
    required this.status,
    this.output,
    this.durationMs,
  });

  factory CronRun.fromJson(Map<String, dynamic> json) {
    DateTime parseDate(dynamic val) {
      if (val is String) {
        return DateTime.tryParse(val) ?? DateTime.now();
      }
      return DateTime.now();
    }

    return CronRun(
      id: json['id'] != null ? (json['id'] as num).toInt() : 0,
      jobId: (json['job_id'] ?? '').toString(),
      startedAt: parseDate(json['started_at']),
      finishedAt: parseDate(json['finished_at']),
      status: (json['status'] ?? 'completed').toString(),
      output: json['output']?.toString(),
      durationMs: json['duration_ms'] != null ? (json['duration_ms'] as num).toInt() : null,
    );
  }
}
