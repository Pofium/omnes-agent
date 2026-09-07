// Permission Modes for ADE (Agentic Development Environment).

enum PermissionMode {
  /// Asks for confirmation before every file edit or shell command (Default).
  askBeforeChanges,

  /// Applies file edits automatically, while shell commands still require confirmation.
  editAutomatically,

  /// Reviews code and proposes an architectural plan first; edits only after user approval.
  planMode,

  /// Autonomous continuous execution without confirmations for lower-risk tasks.
  fullAccess,
}

extension PermissionModeExtension on PermissionMode {
  String get label {
    switch (this) {
      case PermissionMode.askBeforeChanges:
        return 'Ask before changes';
      case PermissionMode.editAutomatically:
        return 'Edit automatically';
      case PermissionMode.planMode:
        return 'Plan mode';
      case PermissionMode.fullAccess:
        return 'Full access';
    }
  }

  String get description {
    switch (this) {
      case PermissionMode.askBeforeChanges:
        return 'Запрос подтверждения на правки файлов и команды';
      case PermissionMode.editAutomatically:
        return 'Авто-правка файлов, подтверждение шелл-команд';
      case PermissionMode.planMode:
        return 'Сначала план, исполнение только после ревью';
      case PermissionMode.fullAccess:
        return 'Полная автономность без лишних остановок';
    }
  }

  PermissionMode next() {
    switch (this) {
      case PermissionMode.askBeforeChanges:
        return PermissionMode.editAutomatically;
      case PermissionMode.editAutomatically:
        return PermissionMode.planMode;
      case PermissionMode.planMode:
        return PermissionMode.fullAccess;
      case PermissionMode.fullAccess:
        return PermissionMode.askBeforeChanges;
    }
  }
}
