// Model representing a workspace file or directory entry from OmnesAgent Gateway.

class WorkspaceEntry {
  final String name;
  final String path;
  final String kind;
  final int? size;
  final bool protected;

  const WorkspaceEntry({
    required this.name,
    required this.path,
    required this.kind,
    this.size,
    this.protected = false,
  });

  bool get isDir => kind == 'dir';
  bool get isFile => kind == 'file';

  String get extension => name.contains('.') ? name.split('.').last.toLowerCase() : '';

  String get formattedSize {
    if (isDir) return '';
    final bytes = size ?? 0;
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  factory WorkspaceEntry.fromJson(Map<String, dynamic> json, {String parentPath = ''}) {
    final name = (json['name'] ?? '').toString();
    final cleanParent = parentPath.trim().replaceAll(RegExp(r'^/+|/+$'), '');
    final fullPath = cleanParent.isEmpty ? name : '$cleanParent/$name';
    return WorkspaceEntry(
      name: name,
      path: fullPath,
      kind: (json['kind'] ?? 'file').toString(),
      size: json['size'] != null ? (json['size'] as num).toInt() : null,
      protected: json['protected'] == true,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'name': name,
      'path': path,
      'kind': kind,
      if (size != null) 'size': size,
      'protected': protected,
    };
  }
}

/// Model representing the response from GET /api/agents/{alias}/workspace/read.
class WorkspaceFileContent {
  final String path;
  final int size;
  final bool isText;
  final String content;
  final String encoding;

  const WorkspaceFileContent({
    required this.path,
    required this.size,
    required this.isText,
    required this.content,
    required this.encoding,
  });

  factory WorkspaceFileContent.fromJson(Map<String, dynamic> json) {
    return WorkspaceFileContent(
      path: (json['path'] ?? '').toString(),
      size: json['size'] != null ? (json['size'] as num).toInt() : 0,
      isText: json['is_text'] == true,
      content: (json['content'] ?? '').toString(),
      encoding: (json['encoding'] ?? 'utf8').toString(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'path': path,
      'size': size,
      'is_text': isText,
      'content': content,
      'encoding': encoding,
    };
  }
}
