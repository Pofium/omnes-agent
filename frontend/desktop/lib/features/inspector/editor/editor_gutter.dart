// Editor gutter: line numbers for the visible window, current-row highlight.
// FRONTEND_SPEC.md §5.5 (editor_gutter.dart). Git/diagnostic/agent markers and
// fold indicators arrive with F1/F4 (BACKEND_SPEC §9.3, §9.8).

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
    final muted = theme.colorScheme.outline;
    final active = theme.colorScheme.onSurface;

    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return CustomPaint(
          size: Size(width, double.infinity),
          painter: _GutterPainter(
            controller: controller,
            metrics: metrics,
            scrollY: scrollY,
            background: bg,
            borderColor: border,
            mutedColor: muted,
            activeColor: active,
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

  _GutterPainter({
    required this.controller,
    required this.metrics,
    required this.scrollY,
    required this.background,
    required this.borderColor,
    required this.mutedColor,
    required this.activeColor,
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
    final firstRow = math.max(0, (scrollY / m.lineHeight).floor());
    final visibleCount = (size.height / m.lineHeight).ceil() + 1;
    final lastRow = math.min(controller.rows.length - 1, firstRow + visibleCount);

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

    for (var row = firstRow; row <= lastRow; row++) {
      final y = row * m.lineHeight - scrollY;
      final isActive = row == controller.caretRow;
      final tp = TextPainter(
        text: TextSpan(text: '${row + 1}', style: isActive ? activeStyle : baseStyle),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(canvas, Offset(size.width - tp.width - 10, y + (m.lineHeight - tp.height) / 2));
    }
  }

  @override
  bool shouldRepaint(covariant _GutterPainter oldDelegate) {
    return oldDelegate.scrollY != scrollY || oldDelegate.controller != controller;
  }
}
