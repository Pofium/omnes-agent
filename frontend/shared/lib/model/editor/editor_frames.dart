// DTO канала редактора `/ws/editor/{buffer_id}`.
// Формат фреймов: BACKEND_SPEC.md §9.9; FRONTEND_SPEC.md §3.3.
// F0-подмножество: hello, rows_snapshot, rows_changed, edit_ack, save_state,
// error (S→C); rows_request, edit_ops, save, cursor (C→S).

import 'render_plan.dart';

/// Позиция в буфере: 0-я строка, 0-я колонка в UTF-16 code units —
/// совпадает и с индексами Dart-строк, и с LSP-координатами.
class EditorPosition {
  final int line;
  final int col;

  const EditorPosition({required this.line, required this.col});

  factory EditorPosition.fromJson(Map<String, dynamic>? json) {
    if (json == null) return const EditorPosition(line: 0, col: 0);
    return EditorPosition(
      line: (json['line'] as num?)?.toInt() ?? 0,
      col: (json['col'] as num?)?.toInt() ?? 0,
    );
  }

  Map<String, dynamic> toJson() => {'line': line, 'col': col};

  @override
  String toString() => 'EditorPosition($line, $col)';
}

/// Одна правка: заменить диапазон [start, end) на `text`
/// (пустой text = удаление, start == end = вставка).
class EditorEditOp {
  final EditorPosition start;
  final EditorPosition end;
  final String text;

  const EditorEditOp({
    required this.start,
    required this.end,
    this.text = '',
  });

  Map<String, dynamic> toJson() => {
        'start': start.toJson(),
        'end': end.toJson(),
        'text': text,
      };
}

/// Входящий фрейм канала. Одни поля заполнены в зависимости от `type`.
class EditorFrame {
  final String type;
  final int? bufferId;
  final int? rev;
  final int? from;
  final int? totalLines;
  final int? startLine;
  final int? invalCount;
  final int? newCount;
  final int? opId;
  final bool? applied;
  final bool? readOnly;
  final bool? dirty;
  final String? path;
  final String? language;
  final String? eol;
  final String? encoding;
  final String? code;
  final String? message;
  final String? reason;
  final String? savedAt;
  final List<EditorRow> rows;
  final EditorSettings? settings;
  final List<EditorStyleEntry> styles;

  /// Свёрнутые диапазоны буфера [[start, end], ...] (hello / folds_state).
  final List<List<int>> folds;

  const EditorFrame({
    required this.type,
    this.bufferId,
    this.rev,
    this.from,
    this.totalLines,
    this.startLine,
    this.invalCount,
    this.newCount,
    this.opId,
    this.applied,
    this.readOnly,
    this.dirty,
    this.path,
    this.language,
    this.eol,
    this.encoding,
    this.code,
    this.message,
    this.reason,
    this.savedAt,
    this.rows = const [],
    this.settings,
    this.styles = const [],
    this.folds = const [],
  });

  factory EditorFrame.fromJson(Map<String, dynamic> json) {
    List<EditorRow> parseRows(dynamic raw) {
      if (raw is! List) return const [];
      return raw
          .whereType<Map<String, dynamic>>()
          .map(EditorRow.fromJson)
          .toList(growable: false);
    }

    return EditorFrame(
      type: (json['type'] as String?) ?? 'unknown',
      bufferId: (json['buffer_id'] as num?)?.toInt(),
      rev: (json['rev'] as num?)?.toInt(),
      from: (json['from'] as num?)?.toInt(),
      totalLines: (json['total_lines'] as num?)?.toInt(),
      startLine: (json['start_line'] as num?)?.toInt(),
      invalCount: (json['inval_count'] as num?)?.toInt(),
      newCount: (json['new_count'] as num?)?.toInt(),
      opId: (json['op_id'] as num?)?.toInt(),
      applied: json['applied'] as bool?,
      readOnly: json['read_only'] as bool?,
      dirty: json['dirty'] as bool?,
      path: json['path'] as String?,
      language: json['language'] as String?,
      eol: json['eol'] as String?,
      encoding: json['encoding'] as String?,
      code: json['code'] as String?,
      message: json['message'] as String?,
      reason: json['reason'] as String?,
      savedAt: json['saved_at'] as String?,
      rows: parseRows(json['rows']),
      settings: EditorSettings.fromJson(json['settings'] as Map<String, dynamic>?),
      styles: (json['styles'] as List?)
              ?.whereType<Map<String, dynamic>>()
              .map(EditorStyleEntry.fromJson)
              .toList(growable: false) ??
          const [],
      folds: (json['folds'] as List?)
              ?.whereType<List>()
              .map((pair) => [
                    (pair.length > 0 ? pair[0] : 0) is num
                        ? (pair[0] as num).toInt()
                        : 0,
                    pair.length > 1
                        ? ((pair[1] as num?)?.toInt() ?? 0)
                        : 0,
                  ])
              .toList(growable: false) ??
          const [],
    );
  }
}

/// Исходящий фрейм `edit_ops` (оптимистично применённая правка клиента).
Map<String, dynamic> editorEditOpsFrame({
  required int baseRev,
  required int opId,
  required List<EditorEditOp> ops,
}) {
  return {
    'type': 'edit_ops',
    'base_rev': baseRev,
    'op_id': opId,
    'ops': ops.map((e) => e.toJson()).toList(),
  };
}

/// Исходящий фрейм запроса строк (идемпотентный, BACKEND_SPEC §9.9 инвариант 3).
Map<String, dynamic> editorRowsRequestFrame({required int from, required int count}) {
  return {'type': 'rows_request', 'from': from, 'count': count};
}

Map<String, dynamic> editorSaveFrame() => {'type': 'save'};

Map<String, dynamic> editorCursorFrame({required int line, required int col}) {
  return {'type': 'cursor', 'line': line, 'col': col};
}

/// Фрейм `folds` (C→S, §9.9): op = fold|unfold (start/end) | fold_all | unfold_all.
Map<String, dynamic> editorFoldsFrame({
  required String op,
  int? start,
  int? end,
}) {
  return {
    'type': 'folds',
    'op': op,
    if (start != null) 'start': start,
    if (end != null) 'end': end,
  };
}

Map<String, dynamic> editorPingFrame() => {'type': 'ping'};
