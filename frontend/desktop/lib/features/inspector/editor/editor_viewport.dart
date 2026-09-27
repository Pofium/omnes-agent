// Virtualized editor viewport: CustomPaint + TextInputClient.
// FRONTEND_SPEC.md §5.5 (editor_viewport.dart). Ф0: monospace text layer,
// caret with blink, drag selection, wheel scrolling, IME-safe typing via the
// platform text input (diff of the current-line editing value), hardware key
// shortcuts. Only visible rows are painted.

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:omnes_shared/omnes_shared.dart' show EditorPosition, EditorStyleEntry;

import 'editor_client.dart';
import 'editor_controller.dart';

/// Shared text metrics for the viewport and the gutter.
class EditorMetrics {
  final double charWidth;
  final double lineHeight;
  final double fontSize;

  const EditorMetrics({
    required this.charWidth,
    required this.lineHeight,
    required this.fontSize,
  });

  static EditorMetrics measure(BuildContext context, {double fontSize = 13}) {
    final style = TextStyle(
      fontFamily: 'Consolas',
      fontFamilyFallback: const ['Cascadia Mono', 'Courier New', 'monospace'],
      fontSize: fontSize,
      height: 1.4,
    );
    final tp = TextPainter(
      text: TextSpan(text: '0' * 20, style: style),
      textDirection: TextDirection.ltr,
    )..layout();
    final lineHeight = TextPainter(
      text: TextSpan(text: 'M', style: style),
      textDirection: TextDirection.ltr,
    )..layout();
    return EditorMetrics(
      charWidth: tp.width / 20,
      lineHeight: lineHeight.height,
      fontSize: fontSize,
    );
  }

  TextStyle textStyle(Color color) => TextStyle(
        fontFamily: 'Consolas',
        fontFamilyFallback: const ['Cascadia Mono', 'Courier New', 'monospace'],
        fontSize: fontSize,
        height: 1.4,
        color: color,
      );
}

class EditorViewport extends StatefulWidget {
  final EditorBufferController controller;
  final EditorClient client;
  final EditorMetrics metrics;

  /// Notifies the pane about vertical scroll changes so the gutter stays in
  /// sync (the viewport owns the scroll offset).
  final ValueChanged<double>? onScrollChanged;

  const EditorViewport({
    super.key,
    required this.controller,
    required this.client,
    required this.metrics,
    this.onScrollChanged,
  });

  @override
  State<EditorViewport> createState() => _EditorViewportState();
}

class _EditorViewportState extends State<EditorViewport> {
  static const double _scrollbarWidth = 10;

  final FocusNode _focusNode = FocusNode(debugLabel: 'editor-viewport');

  double _scrollY = 0;
  double _scrollX = 0;
  Timer? _blinkTimer;
  bool _caretVisible = true;
  bool _dragSelecting = false;

  // Current-line editing value maintained for the platform text input.
  TextEditingValue _editingValue = TextEditingValue.empty;
  int _editingRow = -1;

  @override
  void initState() {
    super.initState();
    widget.controller.addListener(_onControllerChanged);
    _focusNode.addListener(_onFocusChanged);
    _startBlink();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_focusNode.hasFocus) _attachInput();
    });
  }

  @override
  void didUpdateWidget(covariant EditorViewport oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != widget.controller) {
      oldWidget.controller.removeListener(_onControllerChanged);
      widget.controller.addListener(_onControllerChanged);
      _scrollY = 0;
      _scrollX = 0;
    }
  }

  @override
  void dispose() {
    _blinkTimer?.cancel();
    _detachInput();
    _focusNode.removeListener(_onFocusChanged);
    _focusNode.dispose();
    widget.controller.removeListener(_onControllerChanged);
    super.dispose();
  }

  void _onControllerChanged() {
    // While a platform editing value is being processed, the notify cascade
    // from controller mutations must NOT push state back: the platform is
    // already in the correct state.
    if (_handlingPlatformValue) return;
    if (mounted && _focusNode.hasFocus) {
      _pushEditingState(force: false);
    }
    if (mounted) setState(() {});
  }

  void _startBlink() {
    _blinkTimer?.cancel();
    _blinkTimer = Timer.periodic(const Duration(milliseconds: 530), (_) {
      if (mounted && _focusNode.hasFocus) {
        setState(() => _caretVisible = !_caretVisible);
      }
    });
  }

  // ── IME / TextInputClient plumbing ───────────────────────────────────

  TextInputConnection? _inputConnection;

  /// True while a platform `updateEditingValue` is being applied — blocks
  /// re-entrant connection churn from controller notifications.
  bool _handlingPlatformValue = false;

  /// The client instance that owns [_inputConnection]; `connectionClosed`
  /// from a stale client must not detach a newer connection.
  _EditorTextInput? _activeInputClient;

  void _onFocusChanged() {
    if (_focusNode.hasFocus) {
      _attachInput();
      _startBlink();
    } else {
      _detachInput();
    }
    if (mounted) setState(() {});
  }

  void _attachInput() {
    _detachInput();
    _editingRow = widget.controller.caretRow;
    _editingValue = _buildEditingValue();
    final client = _EditorTextInput(this);
    _activeInputClient = client;
    _inputConnection = TextInput.attach(
      client,
      const TextInputConfiguration(
        inputType: TextInputType.multiline,
        autocorrect: false,
        enableSuggestions: false,
        inputAction: TextInputAction.newline,
      ),
    );
    _inputConnection!.show();
    // The framework does NOT push the editing state to the embedder on
    // attach — without this the platform text model starts empty and the
    // next keystroke arrives as a diff against the wrong line (caret jumps
    // to the line start, typed text replaces the row).
    _inputConnection!.setEditingState(_editingValue);
  }

  void _detachInput() {
    final connection = _inputConnection;
    _inputConnection = null;
    _activeInputClient = null;
    connection?.close();
  }

  TextEditingValue _buildEditingValue() {
    final c = widget.controller;
    final rowText = c.rows.isEmpty ? '' : c.rows[c.caretRow];
    final col = c.caretCol.clamp(0, rowText.length);
    return TextEditingValue(
      text: rowText,
      selection: TextSelection.collapsed(offset: col),
    );
  }

  /// Publishes the current line to the platform input model WITHOUT
  /// re-attaching (`TextInputConnection.setEditingState`). Re-attach resets
  /// the embedder's text model to empty on Windows and must only happen on
  /// focus changes — every other sync goes through here.
  void _pushEditingState({required bool force}) {
    final c = widget.controller;
    final connection = _inputConnection;
    if (connection == null || !connection.attached) {
      if (_focusNode.hasFocus) _attachInput();
      return;
    }
    final value = _buildEditingValue();
    final rowChanged = _editingRow != c.caretRow;
    final textChanged = value.text != _editingValue.text;
    if (!force && !rowChanged && !textChanged) return;
    _editingRow = c.caretRow;
    _editingValue = value;
    connection.setEditingState(value);
  }

  /// Called by the platform with the new current-line value; diffed against
  /// our cached value to produce the buffer op (IME-safe: composition and
  /// Cyrillic input produce correct replace-ranges).
  void _handleEditingValue(TextEditingValue value) {
    _handlingPlatformValue = true;
    try {
      final c = widget.controller;
      final oldText = _editingValue.text;
      final newText = value.text;
      if (oldText != newText) {
        // The platform edit belongs to the row the editing value was built
        // for — not necessarily the controller's current caret row.
        final row = (_editingRow >= 0 && _editingRow < c.rows.length)
            ? _editingRow
            : c.caretRow;
        final prefix = _commonPrefixLen(oldText, newText);
        final suffix = _commonSuffixLen(oldText, newText, prefix);
        final oldEnd = oldText.length - suffix;
        final insert = newText.substring(prefix, newText.length - suffix);
        if (oldEnd < prefix) {
          // Degenerate overlap (should not happen) — adopt the value as-is.
          _editingValue = value;
          _editingRow = c.caretRow;
          return;
        }
        c.setSelection(row: row, col: prefix);
        c.insertText(insert, pushUndo: true);
        _editingValue = _buildEditingValue();
        _editingRow = c.caretRow;
        // Keep the platform model canonical after our clamping. Never push
        // during an active IME composition — setEditingState would cancel
        // it (CJK input); composition updates are adopted as-is above.
        if (value.composing == TextRange.empty) {
          _pushEditingState(force: true);
        }
        return;
      }
      // Text unchanged — selection-only update from the platform. Windows
      // delivers values with an unset (negative) selection in various IME /
      // focus states; moving the caret on such updates (or on composition
      // internals) snaps it to the line start, so they only refresh the
      // cached value.
      final extent = value.selection.extentOffset;
      if (extent >= 0 && value.composing == TextRange.empty) {
        final col = extent.clamp(0, newText.length);
        if (col != c.caretCol || c.hasSelection) {
          c.setSelection(row: c.caretRow, col: col);
        }
      }
      _editingValue = value;
    } finally {
      _handlingPlatformValue = false;
    }
  }

  static int _commonPrefixLen(String a, String b) {
    final n = math.min(a.length, b.length);
    var i = 0;
    while (i < n && a.codeUnitAt(i) == b.codeUnitAt(i)) {
      i++;
    }
    return i;
  }

  static int _commonSuffixLen(String a, String b, int prefix) {
    var i = 0;
    final maxSuffix = math.min(a.length - prefix, b.length - prefix);
    while (i < maxSuffix &&
        a.codeUnitAt(a.length - 1 - i) == b.codeUnitAt(b.length - 1 - i)) {
      i++;
    }
    return i;
  }

  // ── Keyboard ─────────────────────────────────────────────────────────

  KeyEventResult _onKeyEvent(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }
    final c = widget.controller;
    final key = event.logicalKey;
    final ctrl = HardwareKeyboard.instance.isControlPressed;
    final shift = HardwareKeyboard.instance.isShiftPressed;

    final alt = HardwareKeyboard.instance.isAltPressed;

    if (ctrl) {
      switch (key) {
        case LogicalKeyboardKey.keyS:
          widget.client.save();
          return KeyEventResult.handled;
        case LogicalKeyboardKey.keyZ:
          shift ? c.redo() : c.undo();
          _pushEditingState(force: true);
          return KeyEventResult.handled;
        case LogicalKeyboardKey.keyY:
          c.redo();
          _pushEditingState(force: true);
          return KeyEventResult.handled;
        case LogicalKeyboardKey.keyA:
          c.selectAll();
          return KeyEventResult.handled;
        case LogicalKeyboardKey.keyC:
          if (c.hasSelection) _copySelection();
          return KeyEventResult.handled;
        case LogicalKeyboardKey.keyX:
          if (c.hasSelection) {
            _copySelection();
            c.insertText('', pushUndo: true);
            _pushEditingState(force: true);
          }
          return KeyEventResult.handled;
        case LogicalKeyboardKey.keyV:
          _paste();
          return KeyEventResult.handled;
        case LogicalKeyboardKey.keyD:
          c.selectNextOccurrence();
          return KeyEventResult.handled;
        case LogicalKeyboardKey.slash:
          c.toggleComment();
          _pushEditingState(force: true);
          return KeyEventResult.handled;
        case LogicalKeyboardKey.arrowLeft:
          _moveWordHorizontal(-1, extend: shift);
          return KeyEventResult.handled;
        case LogicalKeyboardKey.arrowRight:
          _moveWordHorizontal(1, extend: shift);
          return KeyEventResult.handled;
        default:
          return KeyEventResult.ignored;
      }
    }

    if (alt) {
      if (key == LogicalKeyboardKey.arrowUp) {
        shift ? c.duplicateLines(delta: -1) : c.moveLines(delta: -1);
        _pushEditingState(force: true);
        _ensureCaretVisible();
        return KeyEventResult.handled;
      } else if (key == LogicalKeyboardKey.arrowDown) {
        shift ? c.duplicateLines(delta: 1) : c.moveLines(delta: 1);
        _pushEditingState(force: true);
        _ensureCaretVisible();
        return KeyEventResult.handled;
      }
    }

    final ch = event.character;
    if (!ctrl && !alt && ch != null && ch.isNotEmpty) {
      if (c.hasSelection && (ch == '(' || ch == '[' || ch == '{' || ch == '"' || ch == "'" || ch == '`')) {
        final pair = _matchingPair(ch);
        _wrapSelection(ch, pair);
        return KeyEventResult.handled;
      }
      if (!c.hasSelection) {
        if (ch == '(' || ch == '[' || ch == '{') {
          final pair = _matchingPair(ch);
          c.insertText('$ch$pair', pushUndo: true);
          c.setSelection(row: c.caretRow, col: c.caretCol - 1);
          _pushEditingState(force: true);
          _ensureCaretVisible();
          return KeyEventResult.handled;
        }
        if (ch == ')' || ch == ']' || ch == '}' || ch == '"' || ch == "'" || ch == '`') {
          final row = c.rows.isEmpty ? '' : c.rows[c.caretRow];
          if (c.caretCol < row.length && row[c.caretCol] == ch) {
            c.setSelection(row: c.caretRow, col: c.caretCol + 1);
            _pushEditingState(force: true);
            _ensureCaretVisible();
            return KeyEventResult.handled;
          }
          if (ch == '"' || ch == "'" || ch == '`') {
            c.insertText('$ch$ch', pushUndo: true);
            c.setSelection(row: c.caretRow, col: c.caretCol - 1);
            _pushEditingState(force: true);
            _ensureCaretVisible();
            return KeyEventResult.handled;
          }
        }
      }
    }

    switch (key) {
      case LogicalKeyboardKey.arrowLeft:
        _moveHorizontal(-1, extend: shift);
        return KeyEventResult.handled;
      case LogicalKeyboardKey.arrowRight:
        _moveHorizontal(1, extend: shift);
        return KeyEventResult.handled;
      case LogicalKeyboardKey.arrowUp:
        c.moveCaret(deltaRow: -1, extend: shift);
        _ensureCaretVisible();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.arrowDown:
        c.moveCaret(deltaRow: 1, extend: shift);
        _ensureCaretVisible();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.home:
        _handleHome(extend: shift);
        return KeyEventResult.handled;
      case LogicalKeyboardKey.end:
        c.setSelection(
          row: c.caretRow,
          col: c.rows.isEmpty ? 0 : c.rows[c.caretRow].length,
          keepAnchor: shift,
        );
        _ensureCaretVisible();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.pageUp:
      case LogicalKeyboardKey.pageDown:
        final pageRows = math.max(1, ((context.size?.height ?? 400) / widget.metrics.lineHeight).floor() - 1);
        c.moveCaret(
          deltaRow: key == LogicalKeyboardKey.pageUp ? -pageRows : pageRows,
          extend: shift,
        );
        _ensureCaretVisible();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.backspace:
        _handleBackspace();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.delete:
        c.deleteForward();
        _pushEditingState(force: true);
        return KeyEventResult.handled;
      case LogicalKeyboardKey.enter:
      case LogicalKeyboardKey.numpadEnter:
        _handleEnter();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.tab:
        if (shift) {
          c.outdentSelection();
        } else if (c.hasSelection && c.selectionRange!.$1.line != c.selectionRange!.$2.line) {
          c.indentSelection();
        } else {
          c.insertText('    ', pushUndo: true);
        }
        _pushEditingState(force: true);
        _ensureCaretVisible();
        return KeyEventResult.handled;
      case LogicalKeyboardKey.escape:
        if (c.secondarySelections.isNotEmpty) {
          c.removeSecondaryCursors();
        } else {
          _focusNode.unfocus();
        }
        return KeyEventResult.handled;
      default:
        return KeyEventResult.ignored;
    }
  }

  static String _matchingPair(String ch) {
    switch (ch) {
      case '(':
        return ')';
      case '[':
        return ']';
      case '{':
        return '}';
      case '"':
        return '"';
      case "'":
        return "'";
      case '`':
        return '`';
      default:
        return '';
    }
  }

  void _wrapSelection(String open, String close) {
    final c = widget.controller;
    final sel = c.selectionRange;
    if (sel == null) return;
    final startFlat = c.posToFlat(sel.$1);
    final endFlat = c.posToFlat(sel.$2);
    final text = c.joinedText().substring(startFlat, endFlat);
    c.insertText('$open$text$close', pushUndo: true);
    final newStart = sel.$1;
    final newEnd = EditorPosition(line: sel.$2.line, col: sel.$2.col + 1);
    c.setSelection(row: newEnd.line, col: newEnd.col);
    c.anchorRow = newStart.line;
    c.anchorCol = newStart.col + 1;
    _pushEditingState(force: true);
    _ensureCaretVisible();
  }

  void _handleEnter() {
    final c = widget.controller;
    if (c.readOnly || c.rows.isEmpty) return;
    final row = c.rows[c.caretRow];
    final col = c.caretCol.clamp(0, row.length);
    final wsCount = row.length - row.trimLeft().length;
    var indent = row.substring(0, math.min(wsCount, col));

    final charBefore = col > 0 ? row[col - 1] : '';
    final charAfter = col < row.length ? row[col] : '';

    if (charBefore == '{' && charAfter == '}') {
      final insert = '\n$indent    \n$indent';
      c.insertText(insert, pushUndo: true);
      c.setSelection(row: c.caretRow - 1, col: '$indent    '.length);
      _pushEditingState(force: true);
      _ensureCaretVisible();
      return;
    }

    if (charBefore == '{' || charBefore == '[' || charBefore == '(' || charBefore == ':') {
      indent += '    ';
    }

    c.insertText('\n$indent', pushUndo: true);
    _pushEditingState(force: true);
    _ensureCaretVisible();
  }

  void _handleBackspace() {
    final c = widget.controller;
    if (c.readOnly || c.rows.isEmpty) return;
    if (!c.hasSelection && c.caretCol > 0) {
      final row = c.rows[c.caretRow];
      if (c.caretCol < row.length) {
        final before = row[c.caretCol - 1];
        final after = row[c.caretCol];
        if ((before == '(' && after == ')') ||
            (before == '[' && after == ']') ||
            (before == '{' && after == '}') ||
            (before == '"' && after == '"') ||
            (before == "'" && after == "'") ||
            (before == '`' && after == '`')) {
          c.setSelection(row: c.caretRow, col: c.caretCol + 1);
          c.backspace();
          c.backspace();
          _pushEditingState(force: true);
          _ensureCaretVisible();
          return;
        }
      }
    }
    c.backspace();
    _pushEditingState(force: true);
    _ensureCaretVisible();
  }

  void _handleHome({required bool extend}) {
    final c = widget.controller;
    if (c.rows.isEmpty) return;
    final row = c.rows[c.caretRow];
    final firstNonWs = row.indexOf(RegExp(r'\S'));
    final targetCol = (firstNonWs >= 0 && c.caretCol != firstNonWs) ? firstNonWs : 0;
    c.setSelection(row: c.caretRow, col: targetCol, keepAnchor: extend);
    _ensureCaretVisible();
  }

  void _moveWordHorizontal(int direction, {required bool extend}) {
    final c = widget.controller;
    if (c.rows.isEmpty) return;
    final row = c.rows[c.caretRow];
    var col = c.caretCol;
    bool isWord(String ch) => RegExp(r'^[a-zA-Z0-9_]$').hasMatch(ch);
    if (direction < 0) {
      if (col == 0) {
        if (c.caretRow > 0) {
          c.setSelection(row: c.caretRow - 1, col: c.rows[c.caretRow - 1].length, keepAnchor: extend);
        }
        return;
      }
      col--;
      while (col > 0 && !isWord(row[col])) {
        col--;
      }
      while (col > 0 && isWord(row[col - 1])) {
        col--;
      }
    } else {
      if (col >= row.length) {
        if (c.caretRow < c.rows.length - 1) {
          c.setSelection(row: c.caretRow + 1, col: 0, keepAnchor: extend);
        }
        return;
      }
      while (col < row.length && isWord(row[col])) {
        col++;
      }
      while (col < row.length && !isWord(row[col])) {
        col++;
      }
    }
    c.setSelection(row: c.caretRow, col: col, keepAnchor: extend);
    _ensureCaretVisible();
  }

  void _moveHorizontal(int delta, {required bool extend}) {
    final c = widget.controller;
    if (delta < 0 && c.caretCol > 0) {
      c.setSelection(row: c.caretRow, col: c.caretCol - 1, keepAnchor: extend);
    } else if (delta > 0 && c.caretCol < (c.rows.isEmpty ? 0 : c.rows[c.caretRow].length)) {
      c.setSelection(row: c.caretRow, col: c.caretCol + 1, keepAnchor: extend);
    } else if (delta < 0 && c.caretRow > 0) {
      c.setSelection(
        row: c.caretRow - 1,
        col: c.rows[c.caretRow - 1].length,
        keepAnchor: extend,
      );
    } else if (delta > 0 && c.caretRow < c.rows.length - 1) {
      c.setSelection(row: c.caretRow + 1, col: 0, keepAnchor: extend);
    }
    _ensureCaretVisible();
  }

  void _copySelection() {
    final c = widget.controller;
    final sel = c.selectionRange;
    if (sel == null) return;
    final startFlat = c.posToFlat(sel.$1);
    final endFlat = c.posToFlat(sel.$2);
    final text = c.joinedText().substring(startFlat, endFlat);
    Clipboard.setData(ClipboardData(text: text));
  }

  Future<void> _paste() async {
    final data = await Clipboard.getData('text/plain');
    final text = data?.text;
    if (text == null || text.isEmpty) return;
    widget.controller.insertText(text, pushUndo: true);
    _pushEditingState(force: true);
    _ensureCaretVisible();
  }

  // ── Scroll ───────────────────────────────────────────────────────────

  double get _maxScrollY => math.max(
        0,
        widget.controller.visibleRowCount * widget.metrics.lineHeight -
            (context.size?.height ?? 0) +
            widget.metrics.lineHeight,
      );

  double get _maxScrollX => math.max(
        0,
        widget.controller.maxRowLen * widget.metrics.charWidth -
            ((context.size?.width ?? 400) - _scrollbarWidth) +
            2 * widget.metrics.charWidth,
      );

  void _scrollBy(double dy, double dx) {
    setState(() {
      _scrollY = (_scrollY + dy).clamp(0.0, _maxScrollY);
      _scrollX = (_scrollX + dx).clamp(0.0, _maxScrollX);
    });
    widget.onScrollChanged?.call(_scrollY);
  }

  void _ensureCaretVisible() {
    final m = widget.metrics;
    final caretTop =
        widget.controller.visibleBefore(widget.controller.caretRow) * m.lineHeight + 6.0;
    final viewportH = context.size?.height ?? 400;
    if (caretTop - 6.0 < _scrollY) {
      setState(() => _scrollY = math.max(0.0, caretTop - 6.0));
    } else if (caretTop + m.lineHeight > _scrollY + viewportH) {
      setState(() => _scrollY = caretTop + m.lineHeight - viewportH);
    }
    widget.onScrollChanged?.call(_scrollY);
    final caretLeft = widget.controller.caretCol * m.charWidth;
    final viewportW = (context.size?.width ?? 400) - _scrollbarWidth;
    if (caretLeft < _scrollX) {
      setState(() => _scrollX = math.max(0, caretLeft - 4 * m.charWidth));
    } else if (caretLeft > _scrollX + viewportW) {
      setState(() => _scrollX = caretLeft - viewportW + 4 * m.charWidth);
    }
  }

  // ── Hit testing ──────────────────────────────────────────────────────

  (int, int) _hitTest(Offset local) {
    final m = widget.metrics;
    final maxRow = math.max(0, widget.controller.rows.length - 1);
    final adjustedY = local.dy - 6.0;
    final ordinal =
        (((_scrollY + math.max(0.0, adjustedY)) / m.lineHeight).floor().clamp(0, math.max(0, widget.controller.visibleRowCount - 1)))
            .toInt();
    final row = widget.controller.rowAtVisible(ordinal).clamp(0, maxRow);
    final maxCol = widget.controller.rows.isEmpty ? 0 : widget.controller.rows[row].length;
    final col = (((_scrollX + local.dx) / m.charWidth).round().clamp(0, maxCol)).toInt();
    return (row, col);
  }

  // ── Build ────────────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    final bg = isDark ? const Color(0xFF0B1120) : const Color(0xFFF8FAFC);
    final selectionColor = theme.colorScheme.primary.withOpacity(0.28);
    final caretColor = theme.colorScheme.primary;
    final currentLineColor = isDark ? const Color(0x14FFFFFF) : const Color(0x0A0F172A);

    return Focus(
      focusNode: _focusNode,
      onKeyEvent: _onKeyEvent,
      canRequestFocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: (d) {
          _focusNode.requestFocus();
          final (row, col) = _hitTest(d.localPosition);
          final alt = HardwareKeyboard.instance.isAltPressed;
          if (alt) {
            widget.controller.addCursor(row, col);
          } else {
            widget.controller.removeSecondaryCursors();
            widget.controller.setSelection(row: row, col: col);
          }
          _pushEditingState(force: true);
          _ensureCaretVisible();
        },
        onPanStart: (d) {
          _dragSelecting = true;
          final (row, col) = _hitTest(d.localPosition);
          widget.controller.setSelection(row: row, col: col, keepAnchor: true);
          if (widget.controller.anchorRow == null) {
            widget.controller.setSelection(row: row, col: col);
          }
        },
        onPanUpdate: (d) {
          if (!_dragSelecting) return;
          final (row, col) = _hitTest(d.localPosition);
          widget.controller.setSelection(row: row, col: col, keepAnchor: true);
          if (d.localPosition.dy < 0 || d.localPosition.dy > (context.size?.height ?? 0)) {
            _scrollBy(-d.delta.dy, 0);
          }
        },
        onPanEnd: (_) => _dragSelecting = false,
        child: Listener(
          onPointerSignal: (event) {
            if (event is PointerScrollEvent) {
              _scrollBy(event.scrollDelta.dy, event.scrollDelta.dx);
            }
          },
          child: LayoutBuilder(
            builder: (context, constraints) {
              return ListenableBuilder(
                listenable: widget.controller,
                builder: (context, _) {
                  return CustomPaint(
                    size: Size(constraints.maxWidth, constraints.maxHeight),
                    painter: _EditorPainter(
                      controller: widget.controller,
                      metrics: widget.metrics,
                      scrollY: _scrollY,
                      scrollX: _scrollX,
                      background: bg,
                      textColor: theme.colorScheme.onSurface,
                      selectionColor: selectionColor,
                      caretColor: caretColor,
                      currentLineColor: currentLineColor,
                      showCaret: _focusNode.hasFocus && _caretVisible,
                    ),
                    child: _buildScrollbarOverlay(constraints, theme),
                  );
                },
              );
            },
          ),
        ),
      ),
    );
  }

  Widget _buildScrollbarOverlay(BoxConstraints constraints, ThemeData theme) {
    final viewportH = constraints.maxHeight;
    final docH = widget.controller.visibleRowCount * widget.metrics.lineHeight;
    if (docH <= viewportH) return const SizedBox.expand();
    final thumbH = math.max(32.0, viewportH * viewportH / docH);
    final maxY = docH - viewportH;
    final t = maxY > 0 ? _scrollY / maxY : 0.0;
    final thumbY = t * (viewportH - thumbH);
    return Stack(
      children: [
        Positioned(
          right: 1,
          top: thumbY,
          child: GestureDetector(
            behavior: HitTestBehavior.translucent,
            onVerticalDragUpdate: (d) {
              final range = viewportH - thumbH;
              if (range <= 0) return;
              final newT = ((thumbY + d.delta.dy) / range).clamp(0.0, 1.0);
              setState(() => _scrollY = newT * maxY);
              widget.onScrollChanged?.call(_scrollY);
            },
            onVerticalDragStart: (_) {},
            child: Container(
              width: 8,
              height: thumbH,
              decoration: BoxDecoration(
                color: theme.colorScheme.outline.withOpacity(0.45),
                borderRadius: BorderRadius.circular(4),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

/// Renders the visible window: current-line background, selection rects,
/// cached row text and the blinking caret.
class _EditorPainter extends CustomPainter {
  final EditorBufferController controller;
  final EditorMetrics metrics;
  final double scrollY;
  final double scrollX;
  final Color background;
  final Color textColor;
  final Color selectionColor;
  final Color caretColor;
  final Color currentLineColor;
  final bool showCaret;

  _EditorPainter({
    required this.controller,
    required this.metrics,
    required this.scrollY,
    required this.scrollX,
    required this.background,
    required this.textColor,
    required this.selectionColor,
    required this.caretColor,
    required this.currentLineColor,
    required this.showCaret,
  });

  final Map<int, TextPainter> _painterCache = {};
  final Map<int, String> _painterKeys = {};

  /// Build the row's TextSpan: syntax `runs` (len_utf16, styleId) split the
  /// line into colored segments from the shared style table (§9.4); falls
  /// back to the plain style when there are no runs.
  TextSpan _rowSpan(String text, TextStyle base, List<List<int>> runs) {
    if (runs.isEmpty) return TextSpan(text: text, style: base);
    final children = <InlineSpan>[];
    var cursor = 0; // UTF-16 offset into the row.
    for (final seg in runs) {
      if (seg.length < 2) continue;
      final len = seg[0];
      final styleId = seg[1];
      if (len <= 0) continue;
      final end = math.min(cursor + len, text.length);
      if (cursor >= end) {
        // Runs longer than the row (race with an edit) — stop gracefully.
        break;
      }
      children.add(TextSpan(
        text: text.substring(cursor, end),
        style: _styleForId(styleId, base),
      ));
      cursor = end;
      if (cursor >= text.length) break;
    }
    if (cursor < text.length) {
      children.add(TextSpan(text: text.substring(cursor), style: base));
    }
    return TextSpan(style: base, children: children);
  }

  final Map<int, TextStyle> _styleCache = {};

  TextStyle _styleForId(int styleId, TextStyle base) {
    final cached = _styleCache[styleId];
    if (cached != null) return cached;
    EditorStyleEntry? entry;
    for (final s in controller.styleTable) {
      if (s.id == styleId) {
        entry = s;
        break;
      }
    }
    TextStyle style = base;
    if (entry != null && entry.fg != null) {
      final fg = _parseHex(entry.fg!);
      style = style.copyWith(
        color: fg ?? base.color,
        fontWeight: entry.bold ? FontWeight.bold : null,
        fontStyle: entry.italic ? FontStyle.italic : null,
      );
    }
    _styleCache[styleId] = style;
    return style;
  }

  static Color? _parseHex(String hex) {
    final h = hex.replaceFirst('#', '');
    if (h.length != 6) return null;
    final value = int.tryParse(h, radix: 16);
    if (value == null) return null;
    return Color(0xFF000000 | value);
  }

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = background);
    final m = metrics;
    if (controller.rows.isEmpty) return;

    final style = m.textStyle(textColor);
    final revKey = controller.rev;

    // Обход видимых строк: скрытые пропускаются, на строке-маркере фолда
    // рисуется «… N строк», затем прыжок к концу диапазона.
    final ordinal0 = ((scrollY / m.lineHeight)
            .floor()
            .clamp(0, math.max(0, controller.visibleRowCount - 1)))
        .toInt();
    var y = ordinal0 * m.lineHeight - scrollY + 6.0;
    var row = controller.rowAtVisible(ordinal0);
    final maxRow = controller.rows.length - 1;

    while (row <= maxRow && y < size.height + m.lineHeight) {
      final text = controller.rows[row];
      final fold = controller.foldEndForRow(row);
      if (row == controller.caretRow && !controller.hasSelection) {
        canvas.drawRect(
          Rect.fromLTWH(0, y, size.width, m.lineHeight),
          Paint()..color = currentLineColor,
        );
      }
      for (final s in controller.allSelections) {
        final sRange = s.getRange(controller);
        if (sRange != null) {
          final startRow = sRange.$1.line;
          final endRow = sRange.$2.line;
          if (row >= startRow && row <= endRow) {
            final startCol = row == startRow ? sRange.$1.col : 0;
            final endCol = row == endRow ? sRange.$2.col : text.length;
            final x1 = startCol * m.charWidth - scrollX;
            final x2 = endCol * m.charWidth - scrollX;
            canvas.drawRect(
              Rect.fromLTWH(x1, y, math.max(2, x2 - x1), m.lineHeight),
              Paint()..color = selectionColor,
            );
          }
        }
      }
      _paintRowText(canvas, row, text, style, revKey, -scrollX, y);
      if (fold != null) {
        final marker = TextPainter(
          text: TextSpan(
            text: '  … ${fold - row} строк скрыто',
            style: m.textStyle(textColor.withOpacity(0.4)).copyWith(fontStyle: FontStyle.italic),
          ),
          textDirection: TextDirection.ltr,
        )..layout();
        marker.paint(canvas, Offset(text.length * m.charWidth - scrollX + 6, y));
      }

      y += m.lineHeight;
      row = fold != null ? fold + 1 : row + 1;
    }

    // Carets.
    if (showCaret) {
      for (final s in controller.allSelections) {
        if (!controller.isRowHidden(s.caretRow)) {
          final cx = s.caretCol * m.charWidth - scrollX;
          final cy = controller.visibleBefore(s.caretRow) * m.lineHeight - scrollY + 6.0;
          canvas.drawRect(
            Rect.fromLTWH(cx, cy, 2, m.lineHeight),
            Paint()..color = caretColor,
          );
        }
      }
    }
  }

  /// Отрисовка текста строки с кэшем пейнтеров (ключ — ревизия + runs).
  void _paintRowText(Canvas canvas, int row, String text, TextStyle style,
      int revKey, double dx, double dy) {
    final runs = controller.rowRuns(row, revKey);
    final runsKey = '$revKey|${runs.map((s) => '${s[0]}:${s[1]}').join(',')}';
    var tp = _painterCache[row];
    if (tp == null || _painterKeys[row] != runsKey) {
      tp = _painterFor(row, _rowSpan(text, style, runs));
      _painterKeys[row] = runsKey;
    }
    canvas.save();
    canvas.translate(dx, dy);
    tp.paint(canvas, Offset.zero);
    canvas.restore();
  }

  TextPainter _painterFor(int row, TextSpan span) {
    final cached = _painterCache[row];
    if (cached != null) return cached;
    final tp = TextPainter(
      text: span,
      textDirection: TextDirection.ltr,
    )..layout();
    if (_painterCache.length > 512) {
      _painterCache.clear();
      _painterKeys.clear();
    }
    _painterCache[row] = tp;
    return tp;
  }

  @override
  bool shouldRepaint(covariant _EditorPainter oldDelegate) {
    return oldDelegate.scrollY != scrollY ||
        oldDelegate.scrollX != scrollX ||
        oldDelegate.showCaret != showCaret ||
        oldDelegate.controller != controller ||
        oldDelegate.selectionColor != selectionColor ||
        oldDelegate.controller.secondarySelections.length != controller.secondarySelections.length;
  }
}

/// Bridge between the platform input connection and the viewport state.
class _EditorTextInput with TextInputClient {
  final _EditorViewportState state;

  _EditorTextInput(this.state);

  @override
  void updateEditingValue(TextEditingValue value) {
    state._handleEditingValue(value);
  }

  @override
  void performAction(TextInputAction action) {}

  @override
  void showAutocorrectionPromptRect(int start, int end) {}

  @override
  void connectionClosed() {
    // A stale client instance must not detach a newer connection.
    if (identical(state._activeInputClient, this)) {
      state._inputConnection = null;
      state._activeInputClient = null;
    }
  }

  // DeltaTextInputClient-compatible stubs are not required: the plain client
  // receives full values, which the state diffs itself.
  @override
  AutofillScope? get currentAutofillScope => null;

  @override
  TextEditingValue get currentTextEditingValue => state._editingValue;

  @override
  void performPrivateCommand(String action, Map<String, dynamic> data) {}

  @override
  void performSelector(String selectorName) {}

  @override
  void insertContent(KeyboardInsertedContent content) {}

  @override
  void updateFloatingCursor(RawFloatingCursorPoint point) {}
}
