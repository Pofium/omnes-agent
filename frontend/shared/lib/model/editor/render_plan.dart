// Render-plan models for the file editor viewport.
// BACKEND_SPEC.md §9.3 (render plan) and FRONTEND_SPEC.md §5.5.
// F0: rows only; syntax `runs` arrive with F1 (server-side tree-sitter).

/// One display row of the buffer. `row` is the 0-based buffer line index;
/// `text` carries no line terminator. `runs` is the run-length syntax
/// coloring: segments of `(lengthUtf16, styleId)` summing to the row's
/// UTF-16 length (BACKEND_SPEC §9.3); empty when there is no highlighter.
class EditorRow {
  final int row;
  final String text;
  final List<List<int>> runs;

  const EditorRow({
    required this.row,
    required this.text,
    this.runs = const [],
  });

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
    return EditorRow(
      row: asInt(json['row']),
      text: (json['text'] as String?) ?? '',
      runs: runs,
    );
  }

  Map<String, dynamic> toJson() => {
        'row': row,
        'text': text,
        'runs': runs,
      };
}

/// Server-side editor settings delivered in the `hello` frame.
class EditorSettings {
  final int tabSize;
  final bool hardTabs;
  final String softWrap;

  const EditorSettings({
    this.tabSize = 4,
    this.hardTabs = false,
    this.softWrap = 'none',
  });

  factory EditorSettings.fromJson(Map<String, dynamic>? json) {
    if (json == null) return const EditorSettings();
    return EditorSettings(
      tabSize: (json['tab_size'] as num?)?.toInt() ?? 4,
      hardTabs: (json['hard_tabs'] as bool?) ?? false,
      softWrap: (json['soft_wrap'] as String?) ?? 'none',
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
