// Models for Gateway Sessions and History Messages

class SessionInfo {
  final String sessionId;
  final String agentAlias;
  final String? workspaceDir;
  final String previewText;
  final DateTime createdAt;
  final DateTime lastActivity;
  final int messageCount;

  SessionInfo({
    required this.sessionId,
    required this.agentAlias,
    this.workspaceDir,
    this.previewText = '',
    required this.createdAt,
    required this.lastActivity,
    required this.messageCount,
  });

  String get formattedLastActivity {
    final diff = DateTime.now().difference(lastActivity);
    if (diff.inMinutes < 1) return 'только что';
    if (diff.inHours < 1) return '${diff.inMinutes} мин. назад';
    if (diff.inDays < 1) return '${diff.inHours} ч. назад';
    if (diff.inDays < 7) return '${diff.inDays} дн. назад';
    return '${lastActivity.day}.${lastActivity.month}.${lastActivity.year}';
  }

  factory SessionInfo.fromJson(Map<String, dynamic> json) {
    DateTime parseDate(dynamic val) {
      if (val is String) {
        return DateTime.tryParse(val) ?? DateTime.now();
      }
      return DateTime.now();
    }

    return SessionInfo(
      sessionId: json['session_id'] as String? ?? '',
      agentAlias: json['agent_alias'] as String? ?? 'chief',
      workspaceDir: json['workspace_dir'] as String? ?? json['workspaceDir'] as String?,
      previewText: json['preview_text'] as String? ?? json['preview'] as String? ?? '',
      createdAt: parseDate(json['created_at']),
      lastActivity: parseDate(json['last_activity']),
      messageCount: json['message_count'] as int? ?? 0,
    );
  }
}

class SessionHistoryMessage {
  final String role;
  final String content;
  final DateTime? createdAt;

  SessionHistoryMessage({
    required this.role,
    required this.content,
    this.createdAt,
  });

  factory SessionHistoryMessage.fromJson(Map<String, dynamic> json) {
    DateTime? dt;
    if (json['created_at'] != null) {
      dt = DateTime.tryParse(json['created_at'] as String);
    }
    return SessionHistoryMessage(
      role: json['role'] as String? ?? 'assistant',
      content: json['content'] as String? ?? '',
      createdAt: dt,
    );
  }
}
