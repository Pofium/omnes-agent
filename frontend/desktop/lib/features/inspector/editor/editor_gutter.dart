// Editor gutter: line numbers, current-row highlight and fold triangles
// (click — свернуть/развернуть кандидат; FRONTEND_SPEC §5.5). Git/diagnostic
// маркеры arrive with F4.

import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'editor_controller.dart';
import 'editor_viewport.dart';

class EditorGutter extends StatelessWidget {
  final EditorBufferController controller;
  final EditorMetrics metrics;
  final double scrollY;
  final double width;

  const EditorGutter({
    super.key,
    required this.controller,
    required this.metrics,
    required this.scrollY,
    this.width = 56,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    final bg = isDark ? const Color(0xFF0B1120) : const Color(0xFFF1F5F9);
    final border = isDark ? const Color(0xFF1E293B) : const Color(0xFFE2E8F0);

    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapUp: (details) {
            // Клик по стрелке фолда — сворачивание/разворачивание.
            final m = metrics;
            final dy = details.localPosition.dy - 6.0;
            if (dy < 0) return;
            final ordinal =
                (((scrollY + dy) / m.lineHeight).floor())
                    .clamp(0, math.max(0, controller.visibleRowCount - 1))
                    .toInt();
            final row = controller.rowAtVisible(ordinal);
            if (controller.isRowContinuation(row)) {
              controller.setSelection(row: row, col: 0);
              return;
            }
            final isFoldCandidate = controller.foldEndForRow(row) != null ||
                controller.foldCandidates.any((c) => c[0] == row);
            if (isFoldCandidate) {
              controller.toggleFoldAt(row);
            } else {
              // Клик по номеру строки — просто каретка.
              controller.setSelection(row: row, col: 0);
            }
          },
          child: CustomPaint(
            size: Size(width, double.infinity),
            painter: _GutterPainter(
              controller: controller,
              metrics: metrics,
              scrollY: scrollY,
              background: bg,
              borderColor: border,
              mutedColor: theme.colorScheme.outline,
              activeColor: theme.colorScheme.onSurface,
              foldColor: theme.colorScheme.primary,
            ),
          ),
        );
      },
    );
  }
}

class _GutterPainter extends CustomPainter {
  final EditorBufferController controller;
  final EditorMetrics metrics;
  final double scrollY;
  final Color background;
  final Color borderColor;
  final Color mutedColor;
  final Color activeColor;
  final Color foldColor;

  _GutterPainter({
    required this.controller,
    required this.metrics,
    required this.scrollY,
    required this.background,
    required this.borderColor,
    required this.mutedColor,
    required this.activeColor,
    required this.foldColor,
  });

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = background);
    canvas.drawLine(
      Offset(size.width - 1, 0),
      Offset(size.width - 1, size.height),
      Paint()..color = borderColor..strokeWidth = 1,
    );

    if (controller.rows.isEmpty) return;
    final m = metrics;
    final ordinal0 =
        ((scrollY / m.lineHeight).floor().clamp(0, math.max(0, controller.visibleRowCount - 1)))
            .toInt();
    var y = ordinal0 * m.lineHeight - scrollY + 6.0;
    var row = controller.rowAtVisible(ordinal0);
    final maxRow = controller.rows.length - 1;

    final baseStyle = TextStyle(
      fontFamily: 'Consolas',
      fontSize: m.fontSize - 1,
      color: mutedColor,
    );
    final activeStyle = TextStyle(
      fontFamily: 'Consolas',
      fontSize: m.fontSize - 1,
      fontWeight: FontWeight.bold,
      color: activeColor,
    );

    while (row <= maxRow && y < size.height + m.lineHeight) {
      final isContinuation = controller.isRowContinuation(row);
      final bufferRow = controller.bufferRowFor(row);
      final isActive = !isContinuation && bufferRow == controller.bufferRowFor(controller.caretRow);
      final lineLabel = isContinuation ? '↳' : '${bufferRow + 1}';

      final tp = TextPainter(
        text: TextSpan(
          text: lineLabel,
          style: isContinuation
              ? baseStyle.copyWith(color: mutedColor.withOpacity(0.45))
              : (isActive ? activeStyle : baseStyle),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(canvas, Offset(size.width - tp.width - 22, y + (m.lineHeight - tp.height) / 2));

      // Стрелка фолда: только на основных строках (не на продолжениях переноса)
      if (!isContinuation) {
        final foldEnd = controller.foldEndForRow(row);
        final isCandidate =
            foldEnd != null || controller.foldCandidates.any((c) => c[0] == row);
        if (isCandidate) {
          _drawFoldArrow(canvas, Offset(size.width - 10, y + m.lineHeight / 2),
              filled: foldEnd != null);
        }
      }

      y += m.lineHeight;
      final nextFold = controller.foldEndForRow(row);
      row = nextFold != null ? nextFold + 1 : row + 1;
    }
  }

  void _drawFoldArrow(Canvas canvas, Offset center, {required bool filled}) {
    final w = 4.0;
    final h = 4.5;
    final path = Path()
      ..moveTo(center.dx - w / 2, center.dy - h / 2)
      ..lineTo(center.dx + w / 2, center.dy)
      ..lineTo(center.dx - w / 2, center.dy + h / 2)
      ..close();
    canvas.drawPath(
      path,
      Paint()
        ..color = filled ? foldColor : foldColor.withOpacity(0.5)
        ..style = filled ? PaintingStyle.fill : PaintingStyle.stroke
        ..strokeWidth = 1.2,
    );
  }

  @override
  bool shouldRepaint(covariant _GutterPainter oldDelegate) {
    return oldDelegate.scrollY != scrollY || oldDelegate.controller != controller;
  }
}
