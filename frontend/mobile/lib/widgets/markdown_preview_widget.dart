// Lightweight Markdown preview widget built with core Flutter widgets (no external dependencies).

import 'package:flutter/material.dart';

class MarkdownPreviewWidget extends StatelessWidget {
  final String text;
  final bool shrinkWrap;
  final Color? textColor;

  const MarkdownPreviewWidget({
    super.key,
    required this.text,
    this.shrinkWrap = false,
    this.textColor,
  });

  @override
  Widget build(BuildContext context) {
    if (text.trim().isEmpty) {
      return const Center(
        child: Text(
          'Нет содержимого',
          style: TextStyle(color: Colors.grey),
        ),
      );
    }

    final effectiveColor = textColor ?? Theme.of(context).textTheme.bodyMedium?.color ?? Colors.white;
    final lines = text.split('\n');
    final widgets = <Widget>[];

    bool inCodeBlock = false;
    final codeBuffer = StringBuffer();

    for (int i = 0; i < lines.length; i++) {
      final line = lines[i];

      if (line.trim().startsWith('```')) {
        if (inCodeBlock) {
          // Close code block
          widgets.add(_buildCodeBlock(context, codeBuffer.toString()));
          codeBuffer.clear();
          inCodeBlock = false;
        } else {
          inCodeBlock = true;
        }
        continue;
      }

      if (inCodeBlock) {
        codeBuffer.writeln(line);
        continue;
      }

      final trimmed = line.trim();

      if (trimmed.isEmpty) {
        widgets.add(const SizedBox(height: 6));
        continue;
      }

      // Headers
      if (trimmed.startsWith('# ')) {
        widgets.add(Padding(
          padding: const EdgeInsets.only(top: 8, bottom: 4),
          child: SelectableText(
            trimmed.substring(2),
            style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold, color: effectiveColor),
          ),
        ));
      } else if (trimmed.startsWith('## ')) {
        widgets.add(Padding(
          padding: const EdgeInsets.only(top: 6, bottom: 3),
          child: SelectableText(
            trimmed.substring(3),
            style: TextStyle(fontSize: 17, fontWeight: FontWeight.bold, color: effectiveColor),
          ),
        ));
      } else if (trimmed.startsWith('### ')) {
        widgets.add(Padding(
          padding: const EdgeInsets.only(top: 4, bottom: 2),
          child: SelectableText(
            trimmed.substring(4),
            style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: effectiveColor),
          ),
        ));
      } else if (trimmed.startsWith('> ')) {
        // Blockquote
        widgets.add(Container(
          margin: const EdgeInsets.symmetric(vertical: 4),
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
          decoration: BoxDecoration(
            border: Border(left: BorderSide(color: Colors.blue.shade400, width: 3)),
            color: Colors.black12,
            borderRadius: const BorderRadius.only(
              topRight: Radius.circular(4),
              bottomRight: Radius.circular(4),
            ),
          ),
          child: SelectableText(
            trimmed.substring(2),
            style: TextStyle(
              fontStyle: FontStyle.italic,
              color: effectiveColor.withOpacity(0.85),
            ),
          ),
        ));
      } else if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
        // Bullet list
        widgets.add(Padding(
          padding: const EdgeInsets.symmetric(vertical: 2),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('•  ', style: TextStyle(fontWeight: FontWeight.bold, color: effectiveColor)),
              Expanded(
                child: SelectableText(
                  trimmed.substring(2),
                  style: TextStyle(fontSize: 14, height: 1.3, color: effectiveColor),
                ),
              ),
            ],
          ),
        ));
      } else {
        // Regular paragraph
        widgets.add(Padding(
          padding: const EdgeInsets.symmetric(vertical: 2),
          child: SelectableText(
            line,
            style: TextStyle(fontSize: 14, height: 1.4, color: effectiveColor),
          ),
        ));
      }
    }

    if (inCodeBlock && codeBuffer.isNotEmpty) {
      widgets.add(_buildCodeBlock(context, codeBuffer.toString()));
    }

    if (shrinkWrap) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: widgets,
      );
    }

    return ListView(
      physics: const ClampingScrollPhysics(),
      padding: const EdgeInsets.all(16),
      children: widgets,
    );
  }

  Widget _buildCodeBlock(BuildContext context, String code) {
    return Container(
      margin: const EdgeInsets.symmetric(vertical: 6),
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: Colors.grey.shade900,
        borderRadius: BorderRadius.circular(8),
      ),
      width: double.infinity,
      child: SelectableText(
        code.trimRight(),
        style: const TextStyle(
          fontFamily: 'monospace',
          fontSize: 12,
          color: Colors.lightGreenAccent,
          height: 1.3,
        ),
      ),
    );
  }
}
