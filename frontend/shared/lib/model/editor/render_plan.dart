// Render-plan models for the file editor viewport.
// BACKEND_SPEC.md §9.3 (render plan) and FRONTEND_SPEC.md §5.5.
// F0: rows only; syntax `runs` arrive with F1 (server-side tree-sitter).

/// One display row of the buffer. `row` is the 0-based display row index;
/// `bufferRow` is the source buffer line index; `isWrapContinuation` indicates
/// visual continuation rows caused by soft-wrapping (BACKEND_SPEC §9.3).
/// `text` carries no line terminator. `runs` is the run-length syntax
/// coloring: segments of `(lengthUtf16, styleId)` summing to the row's
/// UTF-16 length; empty when there is no highlighter.
class EditorRow {
  final int row;
  final int bufferRow;
  final String text;
  final List<List<int>> runs;

  /// Строка открывает свёрнутый диапазон: сколько строк скрыто под маркером
  /// (null = строка не свёрнута).
  final int? foldHidden;

  /// Строка является продолжением предыдущей буферной строки из-за переноса.
  final bool isWrapContinuation;

  const EditorRow({
    required this.row,
    int? bufferRow,
    required this.text,
    this.runs = const [],
    this.foldHidden,
    this.isWrapContinuation = false,
  }) : bufferRow = bufferRow ?? row;

  factory EditorRow.fromJson(Map<String, dynamic> json) {
    int asInt(dynamic v) => v is num ? v.toInt() : 0;
    final rawRuns = json['runs'];
    final runs = rawRuns is List
        ? [
            for (final seg in rawRuns)
              if (seg is List && seg.isNotEmpty)
                [asInt(seg[0]), seg.length > 1 ? asInt(seg[1]) : 0],
          ]
        : const <List<int>>[];
    final fold = json['fold'];
    final row = asInt(json['row']);
    final bufferRow = json['buffer_row'] is num
        ? (json['buffer_row'] as num).toInt()
        : row;
    final isContinuation = (json['is_wrap_continuation'] as bool?) ?? false;

    return EditorRow(
      row: row,
      bufferRow: bufferRow,
      text: (json['text'] as String?) ?? '',
      runs: runs,
      foldHidden: fold is Map && fold['hidden'] is num
          ? (fold['hidden'] as num).toInt()
          : null,
      isWrapContinuation: isContinuation,
    );
  }

  Map<String, dynamic> toJson() => {
        'row': row,
        'buffer_row': bufferRow,
        'text': text,
        'runs': runs,
        if (foldHidden != null) 'fold': {'hidden': foldHidden},
        if (isWrapContinuation) 'is_wrap_continuation': true,
      };
}

/// Server-side editor settings delivered in the `hello` frame.
class EditorSettings {
  final int tabSize;
  final bool hardTabs;
  final String softWrap;
  final int wrapColumn;

  const EditorSettings({
    this.tabSize = 4,
    this.hardTabs = false,
    this.softWrap = 'none',
    this.wrapColumn = 80,
  });

  factory EditorSettings.fromJson(Map<String, dynamic>? json) {
    if (json == null) return const EditorSettings();
    return EditorSettings(
      tabSize: (json['tab_size'] as num?)?.toInt() ?? 4,
      hardTabs: (json['hard_tabs'] as bool?) ?? false,
      softWrap: (json['soft_wrap'] as String?) ?? 'none',
      wrapColumn: (json['wrap_column'] as num?)?.toInt() ?? 80,
    );
  }
}

/// Shared style table entry (`styles` frame payload, BACKEND_SPEC §9.4):
/// один JSON темы для шлюза и клиента.
class EditorStyleEntry {
  final int id;
  final String name;
  /// `#RRGGBB`; null = клиентский цвет по умолчанию.
  final String? fg;
  final bool bold;
  final bool italic;

  const EditorStyleEntry({
    required this.id,
    required this.name,
    this.fg,
    this.bold = false,
    this.italic = false,
  });

  factory EditorStyleEntry.fromJson(Map<String, dynamic> json) {
    return EditorStyleEntry(
      id: (json['id'] as num?)?.toInt() ?? 0,
      name: (json['name'] as String?) ?? 'text',
      fg: json['fg'] as String?,
      bold: (json['bold'] as bool?) ?? false,
      italic: (json['italic'] as bool?) ?? false,
    );
  }
}
