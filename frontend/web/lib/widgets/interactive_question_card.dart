import 'dart:convert';
import 'package:flutter/material.dart';
import '../theme/desktop_theme.dart';

/// Модель данных для интерактивного вопроса от агента к пользователю
class InteractiveQuestionData {
  InteractiveQuestionData({
    required this.question,
    this.options = const [],
    this.allowCustom = true,
    this.rawBlock = '',
  });

  final String question;
  final List<String> options;
  final bool allowCustom;
  final String rawBlock;

  static InteractiveQuestionData? tryParse(String text) {
    final trimmed = text.trim();
    // 1. Try parsing JSON directly
    try {
      final decoded = jsonDecode(trimmed);
      if (decoded is Map<String, dynamic>) {
        final q = decoded['question']?.toString() ?? decoded['title']?.toString();
        if (q != null && q.isNotEmpty) {
          final optsRaw = decoded['options'];
          final opts = <String>[];
          if (optsRaw is List) {
            for (final item in optsRaw) {
              if (item is String) {
                opts.add(item);
              } else if (item is Map) {
                final label = item['text'] ?? item['label'] ?? item['title'] ?? item.values.firstOrNull;
                if (label != null) opts.add(label.toString());
              }
            }
          }
          final allowCustom = decoded['allow_custom'] ?? decoded['allowCustom'] ?? true;
          return InteractiveQuestionData(
            question: q,
            options: opts,
            allowCustom: allowCustom == true,
            rawBlock: text,
          );
        }
      }
    } catch (_) {}

    // 2. Try parsing <question>...</question>
    final xmlMatch = RegExp(r'<question>([\s\S]*?)<\/question>').firstMatch(trimmed);
    if (xmlMatch != null) {
      final inner = xmlMatch.group(1)!.trim();
      final fromJson = tryParse(inner);
      if (fromJson != null) return fromJson;
    }

    return null;
  }
}

/// Виджет интерактивной карточки вопроса от LLM с возможностью выбора ответов кликом
class InteractiveQuestionCard extends StatefulWidget {
  const InteractiveQuestionCard({
    super.key,
    required this.data,
    this.initialAnswer,
    required this.onAnswer,
  });

  final InteractiveQuestionData data;
  final String? initialAnswer;
  final ValueChanged<String> onAnswer;

  @override
  State<InteractiveQuestionCard> createState() => _InteractiveQuestionCardState();
}

class _InteractiveQuestionCardState extends State<InteractiveQuestionCard> {
  String? _selectedOption;
  final TextEditingController _customInputController = TextEditingController();
  bool _isSubmitting = false;

  @override
  void initState() {
    super.initState();
    _selectedOption = widget.initialAnswer;
  }

  @override
  void didUpdateWidget(covariant InteractiveQuestionCard oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.initialAnswer != oldWidget.initialAnswer) {
      _selectedOption = widget.initialAnswer;
    }
  }

  @override
  void dispose() {
    _customInputController.dispose();
    super.dispose();
  }

  bool get isAnswered => widget.initialAnswer != null && widget.initialAnswer!.isNotEmpty;

  void _submitAnswer(String answer) {
    final trimmed = answer.trim();
    if (trimmed.isEmpty || isAnswered || _isSubmitting) return;

    setState(() {
      _selectedOption = trimmed;
      _isSubmitting = true;
    });

    widget.onAnswer(trimmed);
  }

  @override
  Widget build(BuildContext context) {
    final hasOptions = widget.data.options.isNotEmpty;
    final answered = isAnswered;

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: DesktopTheme.isDark ? const Color(0xFF1B1D22) : Colors.white,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(
          color: answered
              ? const Color(0xFF10B981).withOpacity(0.5)
              : const Color(0xFF00D2FF).withOpacity(0.6),
          width: 1.0,
        ),
        boxShadow: [
          BoxShadow(
            color: (answered ? const Color(0xFF10B981) : const Color(0xFF00D2FF))
                .withOpacity(DesktopTheme.isDark ? 0.08 : 0.04),
            blurRadius: 14,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // 1. Верхний бейдж / заголовок вопроса
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurfaceElevated,
              borderRadius: const BorderRadius.vertical(top: Radius.circular(9)),
              border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
            ),
            child: Row(
              children: [
                Icon(
                  answered ? Icons.check_circle : Icons.help_outline,
                  size: 15,
                  color: answered ? const Color(0xFF10B981) : const Color(0xFF00D2FF),
                ),
                const SizedBox(width: 8),
                Text(
                  'Вопрос от OmnesAgent',
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.bold,
                    color: DesktopTheme.textPrimary,
                  ),
                ),
                const Spacer(),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 2),
                  decoration: BoxDecoration(
                    color: (answered ? const Color(0xFF10B981) : const Color(0xFFF59E0B)).withOpacity(0.15),
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(
                      color: (answered ? const Color(0xFF10B981) : const Color(0xFFF59E0B)).withOpacity(0.4),
                      width: 0.8,
                    ),
                  ),
                  child: Text(
                    answered ? '✓ Отвечено' : 'Ожидает выбора',
                    style: TextStyle(
                      fontSize: 10,
                      fontWeight: FontWeight.bold,
                      color: answered ? const Color(0xFF10B981) : const Color(0xFFF59E0B),
                    ),
                  ),
                ),
              ],
            ),
          ),

          // 2. Тело вопроса
          Padding(
            padding: const EdgeInsets.fromLTRB(14, 12, 14, 10),
            child: Text(
              widget.data.question,
              style: TextStyle(
                fontSize: 13,
                fontWeight: FontWeight.w600,
                color: DesktopTheme.textPrimary,
                height: 1.4,
              ),
            ),
          ),

          // 3. Список кликабельных вариантов
          if (hasOptions)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 14),
              child: Column(
                children: List.generate(widget.data.options.length, (idx) {
                  final opt = widget.data.options[idx];
                  final isSelected = _selectedOption == opt;

                  return Padding(
                    padding: const EdgeInsets.only(bottom: 6),
                    child: InkWell(
                      onTap: answered || _isSubmitting ? null : () => _submitAnswer(opt),
                      borderRadius: BorderRadius.circular(6),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                        decoration: BoxDecoration(
                          color: isSelected
                              ? const Color(0xFF00D2FF).withOpacity(0.12)
                              : (answered ? DesktopTheme.bgSurface : DesktopTheme.bgSurfaceElevated),
                          borderRadius: BorderRadius.circular(6),
                          border: Border.all(
                            color: isSelected
                                ? (answered ? const Color(0xFF10B981) : const Color(0xFF00D2FF))
                                : DesktopTheme.borderSubtle,
                            width: isSelected ? 1.2 : 0.8,
                          ),
                        ),
                        child: Row(
                          children: [
                            Container(
                              width: 20,
                              height: 20,
                              alignment: Alignment.center,
                              decoration: BoxDecoration(
                                color: isSelected
                                    ? (answered ? const Color(0xFF10B981) : const Color(0xFF00D2FF))
                                    : Colors.transparent,
                                shape: BoxShape.circle,
                                border: Border.all(
                                  color: isSelected
                                      ? (answered ? const Color(0xFF10B981) : const Color(0xFF00D2FF))
                                      : DesktopTheme.textMuted,
                                  width: 1.2,
                                ),
                              ),
                              child: isSelected
                                  ? const Icon(Icons.check, size: 12, color: Color(0xFF090D14))
                                  : Text(
                                      '${idx + 1}',
                                      style: TextStyle(
                                        fontSize: 10,
                                        fontWeight: FontWeight.bold,
                                        fontFamily: 'Consolas',
                                        color: DesktopTheme.textMuted,
                                      ),
                                    ),
                            ),
                            const SizedBox(width: 10),
                            Expanded(
                              child: Text(
                                opt,
                                style: TextStyle(
                                  fontSize: 12,
                                  fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                                  color: isSelected
                                      ? (answered ? const Color(0xFF10B981) : const Color(0xFF00D2FF))
                                      : (answered ? DesktopTheme.textMuted : DesktopTheme.textSecondary),
                                ),
                              ),
                            ),
                            if (!answered)
                              Icon(
                                Icons.arrow_forward_ios,
                                size: 10,
                                color: DesktopTheme.textMuted.withOpacity(0.6),
                              ),
                          ],
                        ),
                      ),
                    ),
                  );
                }),
              ),
            ),

          // 4. Поле для собственного ответа (если не отвечено и разрешено)
          if (!answered && widget.data.allowCustom)
            Padding(
              padding: const EdgeInsets.fromLTRB(14, 4, 14, 12),
              child: Row(
                children: [
                  Expanded(
                    child: Container(
                      height: 32,
                      padding: const EdgeInsets.symmetric(horizontal: 10),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgSurfaceElevated,
                        borderRadius: BorderRadius.circular(6),
                        border: Border.all(color: DesktopTheme.borderSubtle, width: 0.8),
                      ),
                      child: TextField(
                        controller: _customInputController,
                        style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary),
                        decoration: InputDecoration(
                          hintText: 'Или напишите свой ответ...',
                          hintStyle: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                          border: InputBorder.none,
                          isDense: true,
                          contentPadding: const EdgeInsets.symmetric(vertical: 7),
                        ),
                        onSubmitted: (text) => _submitAnswer(text),
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  ElevatedButton(
                    onPressed: () => _submitAnswer(_customInputController.text),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: const Color(0xFF00D2FF).withOpacity(0.18),
                      foregroundColor: const Color(0xFF00D2FF),
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
                      minimumSize: Size.zero,
                      tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(6),
                        side: const BorderSide(color: Color(0xFF00D2FF), width: 0.8),
                      ),
                    ),
                    child: const Text('Ответить', style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold)),
                  ),
                ],
              ),
            ),

          // 5. Если уже отвечено: статусная плашка
          if (answered)
            Container(
              margin: const EdgeInsets.fromLTRB(14, 4, 14, 12),
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: BoxDecoration(
                color: const Color(0xFF10B981).withOpacity(0.08),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: const Color(0xFF10B981).withOpacity(0.3), width: 0.8),
              ),
              child: Row(
                children: [
                  const Icon(Icons.check, size: 13, color: Color(0xFF10B981)),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      'Выбран ответ: ${widget.initialAnswer}',
                      style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: Color(0xFF10B981)),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}
