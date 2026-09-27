// Editor buffer session controller (client side).
// FRONTEND_SPEC.md §5.5 — editor_controller.dart + editor_client.dart roles.
//
// The gateway owns the text (BACKEND_SPEC §9.1); this controller keeps the
// viewport state: row cache, caret/selection, optimistic local edits with a
// client-side undo stack (our edits only — remote edits never enter it,
// BACKEND_SPEC §9.9 invariant 4). Positions are (line, col-in-UTF-16) —
// Dart string indices map 1:1.

import 'dart:async';
import 'dart:collection';

import 'package:flutter/foundation.dart';
import 'package:omnes_shared/omnes_shared.dart';

enum EditorConnectionStatus { idle, connecting, online, offline, closed }

/// One entry of the local undo stack: our op plus its inverse, both in
/// positions of the rows they transform.
class _UndoEntry {
  final EditorEditOp op;
  final EditorEditOp inverse;

  _UndoEntry(this.op, this.inverse);
}

class EditorBufferController extends ChangeNotifier {
  /// Buffer identity (filled after a successful open).
  int? bufferId;
  String path = '';
  String language = 'plaintext';
  String eol = 'lf';
  bool readOnly = false;

  /// Row cache. F0 keeps the whole document client-side; the viewport paints
  /// only the visible window. Incremental windows arrive in F1+.
  final List<String> rows = <String>[];

  /// Syntax runs per row (from `rows_snapshot` / `rows_changed`), valid for
  /// [_runsRev]; empty segments mean "paint plain".
  final Map<int, List<List<int>>> _runsByRow = <int, List<List<int>>>{};
  int _runsRev = -1;

  /// Runs for [row] at buffer revision [rev]; stale revisions paint plain.
  List<List<int>> rowRuns(int row, int rev) {
    if (rev != _runsRev) return const [];
    return _runsByRow[row] ?? const [];
  }

  void _storeRuns(Iterable<EditorRow> rows, int? rev) {
    if (rev != null) _runsRev = rev;
    for (final r in rows) {
      if (r.runs.isEmpty) {
        _runsByRow.remove(r.row);
      } else {
        _runsByRow[r.row] = r.runs;
      }
    }
  }

  /// Shift the runs map after a rows patch (`startLine`, removedCount rows
  /// replaced by `incoming`), keeping unchanged rows' runs.
  void _applyRunsPatch(int startLine, int removedCount, List<EditorRow> incoming) {
    final delta = incoming.length - removedCount;
    if (delta != 0 || removedCount > 0) {
      final shifted = <int, List<List<int>>>{};
      _runsByRow.forEach((k, v) {
        if (k < startLine) {
          shifted[k] = v;
        } else if (k >= startLine + removedCount) {
          shifted[k + delta] = v;
        }
      });
      _runsByRow
        ..clear()
        ..addAll(shifted);
    }
    _storeRuns(incoming, null);
  }

  /// Gateway revision of the last state reflected in [rows].
  int rev = 0;
  int totalLines = 0;
  bool dirty = false;
  String? savedAt;

  EditorConnectionStatus status = EditorConnectionStatus.idle;
  String? lastError;

  /// Shared syntax style table from the gateway `hello` frame
  /// (BACKEND_SPEC §9.4 — одна тема для шлюза и клиента).
  final List<EditorStyleEntry> styleTable = <EditorStyleEntry>[];

  /// Свёрнутые диапазоны [[start, end], ...] — buffer-строки включительно,
  /// отсортированы. Единственный источник — шлюз (folds_state).
  final List<List<int>> folds = <List<int>>[];

  /// Кандидаты фолдов по индентации (для стрелок в гуттере), пересчитываются
  /// вместе с размерами строк.
  final List<List<int>> foldCandidates = <List<int>>[];

  /// Команды фолдов и прочие сырые фреймы → EditorClient → шлюз.
  final StreamController<Map<String, dynamic>> _frameSink =
      StreamController<Map<String, dynamic>>.broadcast();

  Stream<Map<String, dynamic>> get frameStream => _frameSink.stream;

  // ── Caret / selection ────────────────────────────────────────────────
  int caretRow = 0;
  int caretCol = 0;
  int? anchorRow;
  int? anchorCol;

  bool get hasSelection =>
      anchorRow != null && (anchorRow != caretRow || anchorCol != caretCol);

  /// Normalized selection (start before end), or null when it is a caret.
  (EditorPosition, EditorPosition)? get selectionRange {
    if (!hasSelection || anchorRow == null || anchorCol == null) return null;
    final a = EditorPosition(line: anchorRow!, col: anchorCol!);
    final b = EditorPosition(line: caretRow, col: caretCol);
    final aFlat = posToFlat(a);
    final bFlat = posToFlat(b);
    return aFlat <= bFlat ? (a, b) : (b, a);
  }

  /// Emits ops that must go to the gateway (subscribed by EditorClient).
  final StreamController<EditorEditOp> _opSink =
      StreamController<EditorEditOp>.broadcast();

  Stream<EditorEditOp> get opStream => _opSink.stream;

  /// Public change notification for external collaborators (EditorClient
  /// triggers repaints after frame-driven state updates).
  void notifyChanged() => notifyListeners();

  // ── Local undo (own replica only) ────────────────────────────────────
  final ListQueue<_UndoEntry> _undoStack = ListQueue<_UndoEntry>();
  final ListQueue<_UndoEntry> _redoStack = ListQueue<_UndoEntry>();
  static const int _maxUndoEntries = 200;

  bool get canUndo => _undoStack.isNotEmpty;
  bool get canRedo => _redoStack.isNotEmpty;

  /// Longest row length — horizontal scroll extent for the viewport.
  int maxRowLen = 0;

  void loadFromOpen(EditorOpenResult open) {
    bufferId = open.bufferId;
    path = open.path;
    language = open.language;
    eol = open.eol;
    readOnly = open.readOnly;
    dirty = open.dirty;
    rev = open.rev;
    rows
      ..clear()
      ..addAll(open.content.split('\n'));
    totalLines = rows.length;
    caretRow = 0;
    caretCol = 0;
    anchorRow = null;
    anchorCol = null;
    folds.clear();
    _runsByRow.clear();
    _runsRev = rev;
    _undoStack.clear();
    _redoStack.clear();
    _recomputeMaxRowLen();
    status = EditorConnectionStatus.connecting;
    notifyListeners();
  }

  // ── Frame application (called by EditorClient) ───────────────────────

  void applyHello(EditorFrame frame) {
    status = EditorConnectionStatus.online;
    readOnly = frame.readOnly ?? readOnly;
    totalLines = frame.totalLines ?? totalLines;
    if (frame.styles.isNotEmpty) {
      styleTable
        ..clear()
        ..addAll(frame.styles);
    }
    if (frame.rev != null && frame.rev != rev) {
      // Buffer moved on while we were opening — full resync.
      rev = frame.rev!;
    }
    if (frame.folds.isNotEmpty || frame.type == 'hello') {
      folds
        ..clear()
        ..addAll(frame.folds);
    }
    lastError = null;
    notifyListeners();
  }

  /// `folds_state`: новое состояние фолдов от шлюза. Каретка, попавшая в
  /// скрытую область, подтягивается к началу диапазона.
  void applyFoldsState(EditorFrame frame) {
    folds
      ..clear()
      ..addAll(frame.folds);
    _clampCaret();
    notifyListeners();
  }

  // ── Фолды: видимая геометрия ─────────────────────────────────────────

  bool isRowHidden(int row) {
    for (final f in folds) {
      if (row > f[0] && row <= f[1]) return true;
      if (f[0] >= row) break;
    }
    return false;
  }

  /// Конец свёрнутого диапазона, если [row] — строка-маркер, иначе null.
  int? foldEndForRow(int row) {
    for (final f in folds) {
      if (f[0] == row) return f[1];
      if (f[0] > row) break;
    }
    return null;
  }

  /// Сколько скрытых строк выше [row] (не считая самой строки).
  int _hiddenAbove(int row) {
    var hidden = 0;
    for (final f in folds) {
      if (f[0] >= row) break;
      hidden += (f[1] < row ? f[1] : row - 1) - f[0] + 1;
    }
    return hidden;
  }

  /// Видимый ординал строки (сколько видимых строк выше неё).
  int visibleBefore(int row) => row - _hiddenAbove(row);

  /// Buffer-строка по видимому ординалу; ординал внутри свёрнутого диапазона
  /// схлопывается к его концу.
  int rowAtVisible(int ordinal) {
    var r = ordinal;
    for (final f in folds) {
      if (r >= f[0]) {
        r = f[1] + 1 + (r - f[0]);
      } else {
        break;
      }
    }
    return r.clamp(0, rows.isEmpty ? 0 : rows.length - 1);
  }

  int get hiddenRowCount {
    var hidden = 0;
    for (final f in folds) {
      hidden += f[1] - f[0] + 1;
    }
    return hidden;
  }

  int get visibleRowCount => rows.length - hiddenRowCount;

  /// Пересчитать кандидатов фолдов по индентации (вызывается из
  /// [_recomputeMaxRowLen]).
  void _recomputeFoldCandidates() {
    foldCandidates.clear();
    int? indentOf(int line) {
      if (line >= rows.length) return null;
      final text = rows[line];
      if (text.trim().isEmpty) return null;
      var width = 0;
      for (final cu in text.codeUnits) {
        if (cu == 0x20) {
          width++;
        } else if (cu == 0x09) {
          width += 4;
        } else {
          break;
        }
      }
      return width;
    }
    var line = 0;
    while (line < rows.length) {
      final indent = indentOf(line);
      if (indent == null) {
        line++;
        continue;
      }
      int? blockEnd;
      var probe = line + 1;
      while (probe < rows.length) {
        final w = indentOf(probe);
        if (w != null && w > indent) {
          blockEnd = probe;
          probe++;
        } else if (w == null) {
          probe++;
        } else {
          break;
        }
      }
      if (blockEnd != null) {
        foldCandidates.add([line, blockEnd]);
      }
      line++;
    }
  }

  /// Свернуть/развернуть кандидат на [row]; отправляет `folds` на шлюз —
  /// состояние вернётся рассылкой `folds_state`.
  void toggleFoldAt(int row) {
    if (readOnly || rows.isEmpty) return;
    // Уже свёрнут здесь — развернуть.
    for (final f in folds) {
      if (f[0] == row) {
        if (!_frameSink.isClosed) {
          _frameSink.add(editorFoldsFrame(op: 'unfold', start: f[0], end: f[1]));
        }
        return;
      }
    }
    for (final c in foldCandidates) {
      if (c[0] == row) {
        // Каретка внутрь свёрнутого не должна проваливаться.
        if (caretRow > row && caretRow <= c[1]) {
          setSelection(row: row, col: caretCol.clamp(0, rows[row].length));
        }
        if (!_frameSink.isClosed) {
          _frameSink.add(editorFoldsFrame(op: 'fold', start: c[0], end: c[1]));
        }
        return;
      }
    }
  }

  void applySnapshot(EditorFrame frame) {
    if (frame.rev != null) rev = frame.rev!;
    if (frame.totalLines != null) totalLines = frame.totalLines!;
    final from = frame.from ?? 0;
    final incoming = frame.rows.map((r) => r.text).toList(growable: false);
    if (from == 0 && incoming.length >= rows.length) {
      rows
        ..clear()
        ..addAll(incoming);
      _runsByRow.clear();
      _runsRev = rev;
      _storeRuns(frame.rows, null);
    } else {
      _patchRows(from, incoming.length, incoming);
      _applyRunsPatch(from, incoming.length, frame.rows);
      _runsRev = rev;
    }
    totalLines = rows.length;
    if (frame.reason == 'reloaded') {
      _undoStack.clear();
      _redoStack.clear();
      dirty = false;
    }
    _clampCaret();
    _recomputeMaxRowLen();
    notifyListeners();
  }

  /// `rows_changed`: authoritative rows for [startLine, startLine+invalCount).
  void applyRowsChanged(EditorFrame frame) {
    if (frame.rev != null) rev = frame.rev!;
    final start = frame.startLine ?? 0;
    final incoming = frame.rows.map((r) => r.text).toList(growable: false);
    _patchRows(start, frame.invalCount ?? incoming.length, incoming);
    _applyRunsPatch(start, frame.invalCount ?? incoming.length, frame.rows);
    _runsRev = rev;
    totalLines = rows.length;
    _clampCaret();
    _recomputeMaxRowLen();
    notifyListeners();
  }

  void applySaveState(EditorFrame frame) {
    if (frame.rev != null) rev = frame.rev!;
    dirty = frame.dirty ?? false;
    savedAt = frame.savedAt;
    notifyListeners();
  }

  void markOffline() {
    status = EditorConnectionStatus.offline;
    notifyListeners();
  }

  void markClosed() {
    status = EditorConnectionStatus.closed;
    notifyListeners();
  }

  void setError(String message) {
    lastError = message;
    notifyListeners();
  }

  // ── Caret & selection API (called by the viewport) ───────────────────

  void setSelection({required int row, required int col, bool keepAnchor = false}) {
    caretRow = row.clamp(0, rows.isEmpty ? 0 : rows.length - 1);
    caretCol = col.clamp(0, rows.isEmpty ? 0 : rows[caretRow].length);
    if (!keepAnchor) {
      anchorRow = null;
      anchorCol = null;
    }
    notifyListeners();
  }

  /// Vertical caret movement preserving a "goal column" is handled by the
  /// viewport (it owns pixel metrics); here we only shift by rows.
  void moveCaret({required int deltaRow, int? targetCol, required bool extend}) {
    var newRow = (caretRow + deltaRow).clamp(0, rows.isEmpty ? 0 : rows.length - 1);
    // Не заезжать в свёрнутую область — подтягиваемся к началу фолда.
    if (isRowHidden(newRow)) {
      for (final f in folds) {
        if (newRow > f[0] && newRow <= f[1]) {
          newRow = deltaRow > 0 ? f[1] + 1 : f[0];
          break;
        }
      }
      newRow = newRow.clamp(0, rows.isEmpty ? 0 : rows.length - 1);
    }
    final col = (targetCol ?? caretCol).clamp(0, rows[newRow].length);
    if (extend && anchorRow == null) {
      anchorRow = caretRow;
      anchorCol = caretCol;
    }
    caretRow = newRow;
    caretCol = col;
    if (!extend) {
      anchorRow = null;
      anchorCol = null;
    }
    notifyListeners();
  }

  void selectAll() {
    if (rows.isEmpty) return;
    anchorRow = 0;
    anchorCol = 0;
    caretRow = rows.length - 1;
    caretCol = rows.last.length;
    notifyListeners();
  }

  // ── Editing primitives ───────────────────────────────────────────────

  /// Insert [text] at the caret (replacing the selection, if any).
  void insertText(String text, {bool pushUndo = true}) {
    if (readOnly || rows.isEmpty) return;
    final sel = selectionRange;
    final EditorPosition start;
    if (sel != null) {
      start = sel.$1;
    } else {
      start = EditorPosition(line: caretRow, col: caretCol);
    }
    final end = sel?.$2 ?? start;
    _applyLocalOp(EditorEditOp(start: start, end: end, text: text), pushUndo: pushUndo);
  }

  /// Backspace: delete the selection, or one character left of the caret
  /// (surrogate-pair aware), or merge with the previous line at col 0.
  void backspace() {
    if (readOnly || rows.isEmpty) return;
    final sel = selectionRange;
    if (sel != null) {
      _applyLocalOp(EditorEditOp(start: sel.$1, end: sel.$2, text: ''));
      return;
    }
    if (caretCol > 0) {
      final row = rows[caretRow];
      var deleteLen = 1;
      // Low surrogate at col-1 → delete the full surrogate pair.
      if (caretCol >= 2) {
        final prevUnit = row.codeUnitAt(caretCol - 1);
        if (prevUnit >= 0xDC00 && prevUnit <= 0xDFFF) {
          final prevPrev = row.codeUnitAt(caretCol - 2);
          if (prevPrev >= 0xD800 && prevPrev <= 0xDBFF) deleteLen = 2;
        }
      }
      _applyLocalOp(EditorEditOp(
        start: EditorPosition(line: caretRow, col: caretCol - deleteLen),
        end: EditorPosition(line: caretRow, col: caretCol),
        text: '',
      ));
    } else if (caretRow > 0) {
      final prevLen = rows[caretRow - 1].length;
      _applyLocalOp(EditorEditOp(
        start: EditorPosition(line: caretRow - 1, col: prevLen),
        end: EditorPosition(line: caretRow, col: 0),
        text: '',
      ));
    }
  }

  /// Delete key: mirror of [backspace].
  void deleteForward() {
    if (readOnly || rows.isEmpty) return;
    final sel = selectionRange;
    if (sel != null) {
      _applyLocalOp(EditorEditOp(start: sel.$1, end: sel.$2, text: ''));
      return;
    }
    final row = rows[caretRow];
    if (caretCol < row.length) {
      var deleteLen = 1;
      final unit = row.codeUnitAt(caretCol);
      if (unit >= 0xD800 && unit <= 0xDBFF && caretCol + 1 < row.length) {
        final next = row.codeUnitAt(caretCol + 1);
        if (next >= 0xDC00 && next <= 0xDFFF) deleteLen = 2;
      }
      _applyLocalOp(EditorEditOp(
        start: EditorPosition(line: caretRow, col: caretCol),
        end: EditorPosition(line: caretRow, col: caretCol + deleteLen),
        text: '',
      ));
    } else if (caretRow < rows.length - 1) {
      _applyLocalOp(EditorEditOp(
        start: EditorPosition(line: caretRow, col: caretCol),
        end: EditorPosition(line: caretRow + 1, col: 0),
        text: '',
      ));
    }
  }

  void undo() {
    if (_undoStack.isEmpty) return;
    final entry = _undoStack.removeLast();
    _applyOpToRows(entry.inverse);
    // The inverse edit must reach the gateway too — otherwise the server
    // buffer keeps the undone change and the next save/resync diverges.
    if (!_opSink.isClosed) {
      _opSink.add(entry.inverse);
    }
    _redoStack.addLast(entry);
    dirty = true;
    notifyListeners();
  }

  void redo() {
    if (_redoStack.isEmpty) return;
    final entry = _redoStack.removeLast();
    _applyOpToRows(entry.op);
    if (!_opSink.isClosed) {
      _opSink.add(entry.op);
    }
    _undoStack.addLast(entry);
    dirty = true;
    notifyListeners();
  }

  /// Join rows the way the gateway represents content (LF-joined; EOL is
  /// preserved server-side). Used by the REST save fallback.
  String joinedText() => rows.join('\n');

  // ── Internals ────────────────────────────────────────────────────────

  void _applyLocalOp(EditorEditOp op, {bool pushUndo = true}) {
    if (pushUndo) {
      final inverse = _inverseOp(op);
      _undoStack.addLast(_UndoEntry(op, inverse));
      while (_undoStack.length > _maxUndoEntries) {
        _undoStack.removeFirst();
      }
      _redoStack.clear();
    }
    _applyOpToRows(op);
    if (!_opSink.isClosed) {
      _opSink.add(op);
    }
    dirty = true;
    notifyListeners();
  }

  /// Applies [op] to the row cache and places the caret at the end of the
  /// inserted text (typing) or at the collapse point (deletion).
  void _applyOpToRows(EditorEditOp op) {
    // Fast path: single-row edit without newlines — splice that row only,
    // no whole-document join/split per keystroke.
    final singleRow = op.start.line == op.end.line &&
        !op.text.contains('\n') &&
        op.start.line >= 0 &&
        op.start.line < rows.length;
    if (singleRow) {
      final rowIndex = op.start.line;
      final line = rows[rowIndex];
      final start = op.start.col.clamp(0, line.length);
      final end = op.end.col.clamp(start, line.length);
      final updated = line.replaceRange(start, end, op.text);
      rows[rowIndex] = updated;
      caretRow = rowIndex;
      caretCol = (start + op.text.length).clamp(0, updated.length);
      anchorRow = null;
      anchorCol = null;
      if (updated.length > maxRowLen) {
        maxRowLen = updated.length;
      } else if (line.length >= maxRowLen && updated.length < maxRowLen) {
        _recomputeMaxRowLen();
      }
      totalLines = rows.length;
      return;
    }

    final startFlat = posToFlat(op.start);
    final endFlat = posToFlat(op.end);
    final text = op.text;

    final flat = _flatText();
    final replaced = text.isEmpty
        ? flat.replaceRange(startFlat, endFlat, '')
        : flat.replaceRange(startFlat, endFlat, text);
    rows
      ..clear()
      ..addAll(replaced.split('\n'));
    totalLines = rows.length;
    _recomputeMaxRowLen();

    final caretFlat = startFlat + text.length;
    final pos = flatToPos(EditorPosition(line: 0, col: 0), caretFlat);
    caretRow = pos.line;
    caretCol = pos.col;
    anchorRow = null;
    anchorCol = null;
    _clampCaret();
  }

  EditorEditOp _inverseOp(EditorEditOp op) {
    final startFlat = posToFlat(op.start);
    final endFlat = posToFlat(op.end);
    final flat = _flatText();
    return EditorEditOp(
      start: op.start,
      end: flatToPos(op.start, startFlat + op.text.length),
      text: flat.substring(startFlat, endFlat),
    );
  }

  String _flatText() => rows.join('\n');

  int posToFlat(EditorPosition p) {
    var flat = 0;
    final line = p.line.clamp(0, rows.isEmpty ? 0 : rows.length - 1);
    for (var i = 0; i < line; i++) {
      flat += rows[i].length + 1;
    }
    return flat + p.col.clamp(0, rows.isEmpty ? 0 : rows[line].length);
  }

  EditorPosition flatToPos(EditorPosition _, int flatOffset) {
    var remaining = flatOffset;
    for (var i = 0; i < rows.length; i++) {
      final lineLen = rows[i].length;
      if (remaining <= lineLen) {
        return EditorPosition(line: i, col: remaining);
      }
      remaining -= lineLen + 1;
    }
    return EditorPosition(
      line: rows.isEmpty ? 0 : rows.length - 1,
      col: rows.isEmpty ? 0 : rows.last.length,
    );
  }

  void _patchRows(int startLine, int removedCount, List<String> incoming) {
    final start = startLine.clamp(0, rows.length);
    var end = (startLine + removedCount).clamp(0, rows.length);
    if (end < start) end = start;
    rows.replaceRange(start, end, incoming);
  }

  void _clampCaret() {
    if (rows.isEmpty) {
      caretRow = 0;
      caretCol = 0;
      anchorRow = null;
      anchorCol = null;
      return;
    }
    caretRow = caretRow.clamp(0, rows.length - 1);
    // Каретка не остаётся в свёрнутой области — подтягивается к началу фолда.
    if (isRowHidden(caretRow)) {
      for (final f in folds) {
        if (caretRow > f[0] && caretRow <= f[1]) {
          caretRow = f[0];
          break;
        }
      }
    }
    caretCol = caretCol.clamp(0, rows[caretRow].length);
    if (anchorRow != null) {
      anchorRow = anchorRow!.clamp(0, rows.length - 1);
      if (isRowHidden(anchorRow!)) {
        anchorRow = null;
        anchorCol = null;
      } else {
        anchorCol = anchorCol!.clamp(0, rows[anchorRow!].length);
      }
    }
  }

  void _recomputeMaxRowLen() {
    var maxLen = 0;
    for (final row in rows) {
      if (row.length > maxLen) maxLen = row.length;
    }
    maxRowLen = maxLen;
    _recomputeFoldCandidates();
  }

  @override
  void dispose() {
    _opSink.close();
    super.dispose();
  }
}
