// Model representing a project stored under projects/<id>/project.json.

class Project {
  final String id;
  final String name;
  final String description;
  final DateTime createdAt;
  final DateTime updatedAt;
  final String? agentAlias;

  const Project({
    required this.id,
    required this.name,
    this.description = '',
    required this.createdAt,
    required this.updatedAt,
    this.agentAlias,
  });

  /// The workspace directory path passed to WebSocket chat session (e.g. "projects/demo-123").
  String get workspaceDir => 'projects/$id';

  factory Project.fromJson(Map<String, dynamic> json, String id) {
    return Project(
      id: id,
      name: (json['name'] ?? id).toString(),
      description: (json['description'] ?? '').toString(),
      createdAt: json['created_at'] != null
          ? DateTime.tryParse(json['created_at'].toString()) ?? DateTime.now()
          : DateTime.now(),
      updatedAt: json['updated_at'] != null
          ? DateTime.tryParse(json['updated_at'].toString()) ?? DateTime.now()
          : DateTime.now(),
      agentAlias: json['agent_alias']?.toString(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'description': description,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
      if (agentAlias != null) 'agent_alias': agentAlias,
    };
  }

  Project copyWith({
    String? name,
    String? description,
    DateTime? updatedAt,
    String? agentAlias,
  }) {
    return Project(
      id: id,
      name: name ?? this.name,
      description: description ?? this.description,
      createdAt: createdAt,
      updatedAt: updatedAt ?? DateTime.now(),
      agentAlias: agentAlias ?? this.agentAlias,
    );
  }
}
