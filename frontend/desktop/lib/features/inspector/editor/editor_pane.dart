// Editor pane: composition of header, gutter + viewport and the status bar.
// FRONTEND_SPEC.md §5.5. Created per open file path; reopens when the path
// changes. The gateway is the source of truth — the pane is a viewport.

import 'dart:math' as math;

import 'package:flutter/material.dart';

import 'editor_client.dart';
import 'editor_controller.dart';
import 'editor_gutter.dart';
import 'editor_viewport.dart';

/// Pane for one open file. [root] registers the project root on the gateway
/// on first use (BACKEND_SPEC §9.9, invariant 5).
class EditorPane extends StatefulWidget {
  final String filePath;
  final String? projectRoot;

  const EditorPane({
    super.key,
    required this.filePath,
    this.projectRoot,
  });

  @override
  State<EditorPane> createState() => _EditorPaneState();
}

class _EditorPaneState extends State<EditorPane> {
  late final EditorBufferController controller;
  late final EditorClient client;
  double _scrollY = 0;

  @override
  void initState() {
    super.initState();
    controller = EditorBufferController();
    client = EditorClient(controller: controller);
    _openCurrentPath();
  }

  @override
  void didUpdateWidget(covariant EditorPane oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.filePath != widget.filePath) {
      _openCurrentPath();
    }
  }

  Future<void> _openCurrentPath() async {
    _scrollY = 0;
    await client.open(path: widget.filePath, root: widget.projectRoot);
  }

  @override
  void dispose() {
    client.dispose();
    controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;
    final metrics = EditorMetrics.measure(context);
    final bg = isDark ? const Color(0xFF0B1120) : const Color(0xFFF8FAFC);

    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        if (controller.status == EditorConnectionStatus.idle ||
            (controller.path.isEmpty && controller.lastError == null)) {
          return Center(
            child: Text(
              'Открытие файла…',
              style: TextStyle(color: theme.colorScheme.outline, fontSize: 12),
            ),
          );
        }

        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _buildHeader(context),
            Expanded(
              child: Container(
                color: bg,
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    EditorGutter(
                      controller: controller,
                      metrics: metrics,
                      scrollY: _scrollY,
                    ),
                    Expanded(
                      child: EditorViewport(
                        key: ValueKey(controller.path),
                        controller: controller,
                        client: client,
                        metrics: metrics,
                        onScrollChanged: (offset) {
                          if (mounted && offset != _scrollY) {
                            setState(() => _scrollY = offset);
                          }
                        },
                      ),
                    ),
                  ],
                ),
              ),
            ),
            _buildStatusBar(context),
          ],
        );
      },
    );
  }

  Widget _buildHeader(BuildContext context) {
    final theme = Theme.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        border: Border(
          bottom: BorderSide(color: theme.dividerColor.withOpacity(0.6), width: 0.8),
        ),
      ),
      child: Row(
        children: [
          Icon(Icons.code, size: 14, color: theme.colorScheme.primary),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              controller.path.isEmpty ? widget.filePath : controller.path,
              style: TextStyle(
                fontSize: 11,
                fontFamily: 'Consolas',
                color: theme.colorScheme.onSurface.withOpacity(0.75),
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          if (controller.dirty)
            Container(
              margin: const EdgeInsets.only(right: 8),
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: theme.colorScheme.secondaryContainer,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                'не сохранено',
                style: TextStyle(
                  fontSize: 9,
                  color: theme.colorScheme.onSecondaryContainer,
                ),
              ),
            ),
          Text(
            '${controller.rows.length} строк',
            style: TextStyle(
              fontSize: 10,
              color: theme.colorScheme.outline,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusBar(BuildContext context) {
    final theme = Theme.of(context);
    final statusColor = switch (controller.status) {
      EditorConnectionStatus.online => Colors.green.shade400,
      EditorConnectionStatus.connecting => Colors.orange.shade300,
      EditorConnectionStatus.offline => Colors.red.shade300,
      _ => theme.colorScheme.outline,
    };
    final statusLabel = switch (controller.status) {
      EditorConnectionStatus.online => 'шлюз',
      EditorConnectionStatus.connecting => 'подключение',
      EditorConnectionStatus.offline => 'нет связи',
      EditorConnectionStatus.closed => 'буфер закрыт',
      EditorConnectionStatus.idle => '—',
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 5),
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        border: Border(
          top: BorderSide(color: theme.dividerColor.withOpacity(0.6), width: 0.8),
        ),
      ),
      child: Row(
        children: [
          Container(
            width: 7,
            height: 7,
            margin: const EdgeInsets.only(right: 6),
            decoration: BoxDecoration(color: statusColor, shape: BoxShape.circle),
          ),
          Text(statusLabel, style: const TextStyle(fontSize: 10)),
          const SizedBox(width: 14),
          Text(
            controller.language,
            style: TextStyle(fontSize: 10, color: theme.colorScheme.outline),
          ),
          const SizedBox(width: 14),
          Text(
            'Стр ${math.min(controller.caretRow + 1, math.max(controller.rows.length, 1))}, '
            'Кол ${controller.caretCol + 1}',
            style: TextStyle(fontSize: 10, color: theme.colorScheme.outline),
          ),
          const Spacer(),
          if (controller.lastError != null)
            Flexible(
              child: Padding(
                padding: const EdgeInsets.only(right: 10),
                child: Text(
                  controller.lastError!,
                  style: TextStyle(fontSize: 10, color: Colors.red.shade300),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ),
          _StatusAction(
            label: 'Сохранить',
            enabled: controller.dirty && controller.status == EditorConnectionStatus.online,
            onTap: () => client.save(),
          ),
        ],
      ),
    );
  }
}

class _StatusAction extends StatelessWidget {
  final String label;
  final bool enabled;
  final VoidCallback onTap;

  const _StatusAction({required this.label, required this.enabled, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return InkWell(
      onTap: enabled ? onTap : null,
      borderRadius: BorderRadius.circular(4),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        child: Text(
          label,
          style: TextStyle(
            fontSize: 10,
            fontWeight: FontWeight.w600,
            color: enabled
                ? theme.colorScheme.primary
                : theme.colorScheme.outline.withOpacity(0.5),
          ),
        ),
      ),
    );
  }
}
