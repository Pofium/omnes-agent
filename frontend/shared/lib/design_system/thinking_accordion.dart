import 'package:flutter/material.dart';
import 'shadcn_colors.dart';

/// ThinkingAccordion widget for DeepSeek-R1 / CoT reasoning streaming.
/// Provides a collapsible cyber container with elapsed duration and pulsing glow.
class ThinkingAccordion extends StatefulWidget {
  final String thinkingContent;
  final bool isThinking;
  final int? elapsedSeconds;
  final bool initiallyExpanded;

  const ThinkingAccordion({
    super.key,
    required this.thinkingContent,
    this.isThinking = false,
    this.elapsedSeconds,
    this.initiallyExpanded = false,
  });

  @override
  State<ThinkingAccordion> createState() => _ThinkingAccordionState();
}

class _ThinkingAccordionState extends State<ThinkingAccordion>
    with SingleTickerProviderStateMixin {
  late bool _expanded;
  late AnimationController _pulseController;
  late Animation<double> _pulseAnimation;

  @override
  void initState() {
    super.initState();
    _expanded = widget.initiallyExpanded || widget.isThinking;
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1400),
    )..repeat(reverse: true);
    _pulseAnimation = Tween<double>(begin: 0.3, end: 1.0).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );
  }

  @override
  void didUpdateWidget(ThinkingAccordion oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.isThinking && !widget.isThinking && _expanded) {
      // Auto-collapse after thinking finishes to keep answer clean
      setState(() {
        _expanded = false;
      });
    } else if (!oldWidget.isThinking && widget.isThinking && !_expanded) {
      setState(() {
        _expanded = true;
      });
    }
  }

  @override
  void dispose() {
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (widget.thinkingContent.trim().isEmpty && !widget.isThinking) {
      return const SizedBox.shrink();
    }

    final durationText = widget.elapsedSeconds != null
        ? '${widget.elapsedSeconds} сек'
        : (widget.isThinking ? 'идёт анализ...' : '');

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 6),
      decoration: BoxDecoration(
        color: ShadcnColors.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: widget.isThinking
              ? ShadcnColors.primary.withOpacity(0.4)
              : ShadcnColors.border,
          width: 1,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Header / Toggle
          InkWell(
            onTap: () => setState(() => _expanded = !_expanded),
            borderRadius: BorderRadius.circular(8),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              child: Row(
                children: [
                  if (widget.isThinking)
                    AnimatedBuilder(
                      animation: _pulseAnimation,
                      builder: (context, child) {
                        return Container(
                          width: 8,
                          height: 8,
                          margin: const EdgeInsets.only(right: 8),
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            color: ShadcnColors.primary.withOpacity(
                              _pulseAnimation.value,
                            ),
                            boxShadow: [
                              BoxShadow(
                                color: ShadcnColors.primaryGlow,
                                blurRadius: 4 * _pulseAnimation.value,
                              ),
                            ],
                          ),
                        );
                      },
                    )
                  else
                    const Padding(
                      padding: EdgeInsets.only(right: 8),
                      child: Icon(
                        Icons.psychology_outlined,
                        size: 16,
                        color: ShadcnColors.primary,
                      ),
                    ),
                  Text(
                    widget.isThinking
                        ? 'DeepThink: размышляет'
                        : 'Ход мыслей агента',
                    style: const TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w600,
                      color: ShadcnColors.foreground,
                      fontFamily: 'JetBrains Mono',
                    ),
                  ),
                  if (durationText.isNotEmpty) ...[
                    const SizedBox(width: 8),
                    Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 6,
                        vertical: 2,
                      ),
                      decoration: BoxDecoration(
                        color: ShadcnColors.cardElevated,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        durationText,
                        style: const TextStyle(
                          fontSize: 10,
                          color: ShadcnColors.foregroundMuted,
                          fontFamily: 'JetBrains Mono',
                        ),
                      ),
                    ),
                  ],
                  const Spacer(),
                  Icon(
                    _expanded
                        ? Icons.keyboard_arrow_up
                        : Icons.keyboard_arrow_down,
                    size: 16,
                    color: ShadcnColors.foregroundMuted,
                  ),
                ],
              ),
            ),
          ),

          // Body
          if (_expanded)
            Container(
              padding: const EdgeInsets.all(12),
              decoration: const BoxDecoration(
                border: Border(
                  top: BorderSide(color: ShadcnColors.border, width: 1),
                ),
              ),
              child: SelectableText(
                widget.thinkingContent.trim().isEmpty
                    ? 'Формирование цепочки рассуждений...'
                    : widget.thinkingContent,
                style: const TextStyle(
                  fontSize: 11,
                  height: 1.5,
                  color: ShadcnColors.foregroundMuted,
                  fontFamily: 'JetBrains Mono',
                ),
              ),
            ),
        ],
      ),
    );
  }
}
