import 'package:flutter/material.dart';
import '../core/gateway/models/context_budget.dart';
import 'shadcn_colors.dart';

/// ContextBudgetBar renders a horizontal composite stack bar representing
/// LLM context window breakdown and compression savings counter.
class ContextBudgetBar extends StatelessWidget {
  final ContextBudgetSnapshot budget;

  const ContextBudgetBar({
    super.key,
    required this.budget,
  });

  String _formatK(int tokens) {
    if (tokens >= 1000) {
      return '${(tokens / 1000).toStringAsFixed(1)}k';
    }
    return '$tokens';
  }

  @override
  Widget build(BuildContext context) {
    final maxTokens = budget.maxWindowTokens > 0 ? budget.maxWindowTokens : 128000;
    final totalUsed = budget.totalUsedTokens > 0
        ? budget.totalUsedTokens
        : (budget.systemTokens +
            budget.kagAstTokens +
            budget.memoryTokens +
            budget.dialogHistoryTokens +
            budget.toolOutputsTokens);

    final usedPercent = (totalUsed / maxTokens * 100).clamp(0.0, 100.0);
    final sysFlex = (budget.systemTokens / maxTokens * 1000).round();
    final kagFlex = (budget.kagAstTokens / maxTokens * 1000).round();
    final memFlex = (budget.memoryTokens / maxTokens * 1000).round();
    final histFlex = (budget.dialogHistoryTokens / maxTokens * 1000).round();
    final toolFlex = (budget.toolOutputsTokens / maxTokens * 1000).round();
    final freeFlex = ((maxTokens - totalUsed).clamp(0, maxTokens) / maxTokens * 1000).round();

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: ShadcnColors.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: ShadcnColors.border),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Header info
          Row(
            children: [
              const Icon(Icons.speed, size: 14, color: ShadcnColors.primary),
              const SizedBox(width: 6),
              Text(
                'Контекст: ${_formatK(totalUsed)} / ${_formatK(maxTokens)} (${usedPercent.toStringAsFixed(1)}%)',
                style: const TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.w600,
                  color: ShadcnColors.foreground,
                  fontFamily: 'JetBrains Mono',
                ),
              ),
              const Spacer(),
              if (budget.compressionSavedTokens > 0)
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: ShadcnColors.successMuted,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: ShadcnColors.success.withOpacity(0.3)),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Icon(Icons.compress, size: 11, color: ShadcnColors.success),
                      const SizedBox(width: 4),
                      Text(
                        'Экономия: -${_formatK(budget.compressionSavedTokens)} (${budget.savingsPercent.toStringAsFixed(1)}%)',
                        style: const TextStyle(
                          fontSize: 10,
                          fontWeight: FontWeight.w600,
                          color: ShadcnColors.success,
                          fontFamily: 'JetBrains Mono',
                        ),
                      ),
                    ],
                  ),
                ),
            ],
          ),
          const SizedBox(height: 6),

          // Composite stacked bar
          ClipRRect(
            borderRadius: BorderRadius.circular(3),
            child: SizedBox(
              height: 6,
              child: Row(
                children: [
                  if (sysFlex > 0)
                    Expanded(
                      flex: sysFlex,
                      child: Container(color: const Color(0xFF38BDF8)), // System
                    ),
                  if (kagFlex > 0)
                    Expanded(
                      flex: kagFlex,
                      child: Container(color: const Color(0xFFA855F7)), // KAG
                    ),
                  if (memFlex > 0)
                    Expanded(
                      flex: memFlex,
                      child: Container(color: const Color(0xFFF59E0B)), // Memory
                    ),
                  if (histFlex > 0)
                    Expanded(
                      flex: histFlex,
                      child: Container(color: const Color(0xFF00E5FF)), // Dialog
                    ),
                  if (toolFlex > 0)
                    Expanded(
                      flex: toolFlex,
                      child: Container(color: const Color(0xFF10B981)), // Tools
                    ),
                  if (freeFlex > 0)
                    Expanded(
                      flex: freeFlex,
                      child: Container(color: ShadcnColors.cardElevated), // Free
                    ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 6),

          // Legend
          Wrap(
            spacing: 12,
            runSpacing: 4,
            children: [
              _buildLegendItem('Промпт', const Color(0xFF38BDF8), budget.systemTokens),
              _buildLegendItem('KAG AST', const Color(0xFFA855F7), budget.kagAstTokens),
              _buildLegendItem('Память', const Color(0xFFF59E0B), budget.memoryTokens),
              _buildLegendItem('История', const Color(0xFF00E5FF), budget.dialogHistoryTokens),
              _buildLegendItem('Инструменты', const Color(0xFF10B981), budget.toolOutputsTokens),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildLegendItem(String label, Color color, int tokens) {
    if (tokens <= 0) return const SizedBox.shrink();
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 6,
          height: 6,
          decoration: BoxDecoration(
            color: color,
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 4),
        Text(
          '$label (${_formatK(tokens)})',
          style: const TextStyle(
            fontSize: 9,
            color: ShadcnColors.foregroundMuted,
            fontFamily: 'JetBrains Mono',
          ),
        ),
      ],
    );
  }
}
