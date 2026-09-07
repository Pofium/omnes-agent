// Model representing a long-term memory entry from GET /api/memory.

class MemoryEntry {
  final String id;
  final String key;
  final String content;
  final String category;
  final DateTime timestamp;
  final double? importance;
  final bool pinned;
  final String? agentAlias;

  const MemoryEntry({
    required this.id,
    required this.key,
    required this.content,
    required this.category,
    required this.timestamp,
    this.importance,
    this.pinned = false,
    this.agentAlias,
  });

  String get formattedTime {
    final diff = DateTime.now().difference(timestamp);
    if (diff.inMinutes < 1) return 'только что';
    if (diff.inHours < 1) return '${diff.inMinutes} мин. назад';
    if (diff.inDays < 1) return '${diff.inHours} ч. назад';
    if (diff.inDays < 7) return '${diff.inDays} дн. назад';
    return '${timestamp.day}.${timestamp.month}.${timestamp.year}';
  }

  factory MemoryEntry.fromJson(Map<String, dynamic> json) {
    DateTime parseDate(dynamic val) {
      if (val is String) {
        return DateTime.tryParse(val) ?? DateTime.now();
      }
      return DateTime.now();
    }

    return MemoryEntry(
      id: (json['id'] ?? '').toString(),
      key: (json['key'] ?? '').toString(),
      content: (json['content'] ?? '').toString(),
      category: (json['category'] ?? 'core').toString(),
      timestamp: parseDate(json['timestamp']),
      importance: json['importance'] != null ? (json['importance'] as num).toDouble() : null,
      pinned: json['pinned'] == true,
      agentAlias: json['agent_alias']?.toString(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'key': key,
      'content': content,
      'category': category,
      'timestamp': timestamp.toIso8601String(),
      if (importance != null) 'importance': importance,
      'pinned': pinned,
      if (agentAlias != null) 'agent_alias': agentAlias,
    };
  }
}
