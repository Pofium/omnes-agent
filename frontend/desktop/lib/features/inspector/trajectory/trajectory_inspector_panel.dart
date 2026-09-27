import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:omnes_shared/omnes_shared.dart';

/// TrajectoryInspectorPanel renders the execution trajectory timeline
/// and detailed step inspector inspired by DeepSeek Harness.
class TrajectoryInspectorPanel extends StatefulWidget {
  final List<TrajectoryStep> steps;
  final void Function(TrajectoryStep step)? onForkAtStep;

  const TrajectoryInspectorPanel({
    super.key,
    required this.steps,
    this.onForkAtStep,
  });

  @override
  State<TrajectoryInspectorPanel> createState() => _TrajectoryInspectorPanelState();
}

class _TrajectoryInspectorPanelState extends State<TrajectoryInspectorPanel> {
  String? _selectedStepId;

  IconData _iconForStepType(String type) {
    switch (type) {
      case 's1_gate':
        return Icons.bolt;
      case 'context_inject':
        return Icons.account_tree_outlined;
      case 'cot_thinking':
        return Icons.psychology_outlined;
      case 'tool_call':
        return Icons.build_outlined;
      case 'tool_observation':
        return Icons.visibility_outlined;
      case 'synthesis':
      default:
        return Icons.chat_bubble_outline;
    }
  }

  Color _colorForStepType(String type) {
    switch (type) {
      case 's1_gate':
        return const Color(0xFF00E5FF); // Cyan
      case 'context_inject':
        return const Color(0xFFA855F7); // Purple
      case 'cot_thinking':
        return const Color(0xFF818CF8); // Indigo
      case 'tool_call':
        return const Color(0xFFF59E0B); // Amber
      case 'tool_observation':
        return const Color(0xFF10B981); // Emerald
      case 'synthesis':
      default:
        return const Color(0xFF38BDF8); // Blue
    }
  }

  String _labelForStepType(String type) {
    switch (type) {
      case 's1_gate':
        return 'S1 Gate';
      case 'context_inject':
        return 'KAG Context';
      case 'cot_thinking':
        return 'CoT Thinking';
      case 'tool_call':
        return 'Tool Call';
      case 'tool_observation':
        return 'Observation';
      case 'synthesis':
      default:
        return 'Synthesis';
    }
  }

  @override
  Widget build(BuildContext context) {
    if (widget.steps.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.timeline, size: 36, color: ShadcnColors.foregroundSubtle),
              const SizedBox(height: 12),
              const Text(
                'Траектория сессии пуста',
                style: TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.w600,
                  color: ShadcnColors.foregroundMuted,
                  fontFamily: 'JetBrains Mono',
                ),
              ),
              const SizedBox(height: 6),
              const Text(
                'Шаги выполнения агента (S1 Gate, KAG, CoT, Tool Calls) будут отображаться здесь в реальном времени.',
                textAlign: TextAlign.center,
                style: TextStyle(fontSize: 11, color: ShadcnColors.foregroundSubtle),
              ),
            ],
          ),
        ),
      );
    }

    return Column(
      children: [
        // Header
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
          decoration: const BoxDecoration(
            color: ShadcnColors.surface,
            border: Border(bottom: BorderSide(color: ShadcnColors.border)),
          ),
          child: Row(
            children: [
              const Icon(Icons.hub_outlined, size: 16, color: ShadcnColors.primary),
              const SizedBox(width: 8),
              Text(
                'Траектория (${widget.steps.length} шагов)',
                style: const TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  color: ShadcnColors.foreground,
                  fontFamily: 'JetBrains Mono',
                ),
              ),
              const Spacer(),
              Text(
                '${widget.steps.fold<int>(0, (sum, s) => sum + s.tokensUsed)} токенов',
                style: const TextStyle(
                  fontSize: 11,
                  color: ShadcnColors.foregroundMuted,
                  fontFamily: 'JetBrains Mono',
                ),
              ),
            ],
          ),
        ),

        // Steps list
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            itemCount: widget.steps.length,
            itemBuilder: (context, index) {
              final step = widget.steps[index];
              final isSelected = step.id == _selectedStepId;
              final color = _colorForStepType(step.stepType);
              final ms = (step.durationUs / 1000).toStringAsFixed(1);

              return Container(
                margin: const EdgeInsets.only(bottom: 8),
                decoration: BoxDecoration(
                  color: isSelected ? ShadcnColors.cardElevated : ShadcnColors.card,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(
                    color: isSelected ? color : ShadcnColors.border,
                    width: isSelected ? 1.5 : 1.0,
                  ),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    InkWell(
                      onTap: () {
                        setState(() {
                          _selectedStepId = isSelected ? null : step.id;
                        });
                      },
                      borderRadius: BorderRadius.circular(8),
                      child: Padding(
                        padding: const EdgeInsets.all(10),
                        child: Row(
                          children: [
                            Container(
                              padding: const EdgeInsets.all(5),
                              decoration: BoxDecoration(
                                color: color.withOpacity(0.15),
                                borderRadius: BorderRadius.circular(6),
                              ),
                              child: Icon(_iconForStepType(step.stepType), size: 14, color: color),
                            ),
                            const SizedBox(width: 8),
                            Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Row(
                                  children: [
                                    Text(
                                      '#${step.stepIndex + 1} ${_labelForStepType(step.stepType)}',
                                      style: TextStyle(
                                        fontSize: 12,
                                        fontWeight: FontWeight.bold,
                                        color: color,
                                        fontFamily: 'JetBrains Mono',
                                      ),
                                    ),
                                    const SizedBox(width: 6),
                                    Text(
                                      '(${step.id})',
                                      style: const TextStyle(
                                        fontSize: 10,
                                        color: ShadcnColors.foregroundSubtle,
                                        fontFamily: 'JetBrains Mono',
                                      ),
                                    ),
                                  ],
                                ),
                                const SizedBox(height: 2),
                                Text(
                                  'Задержка: $ms мс • ${step.tokensUsed} токенов',
                                  style: const TextStyle(
                                    fontSize: 10,
                                    color: ShadcnColors.foregroundMuted,
                                    fontFamily: 'JetBrains Mono',
                                  ),
                                ),
                              ],
                            ),
                            const Spacer(),
                            if (widget.onForkAtStep != null)
                              IconButton(
                                icon: const Icon(Icons.fork_right, size: 16),
                                tooltip: 'Ветвить от этого шага',
                                color: ShadcnColors.primary,
                                onPressed: () => widget.onForkAtStep!(step),
                              ),
                            Icon(
                              isSelected ? Icons.expand_less : Icons.expand_more,
                              size: 16,
                              color: ShadcnColors.foregroundMuted,
                            ),
                          ],
                        ),
                      ),
                    ),

                    // Step payload inspector drawer
                    if (isSelected)
                      Container(
                        padding: const EdgeInsets.all(10),
                        decoration: const BoxDecoration(
                          color: ShadcnColors.surface,
                          border: Border(top: BorderSide(color: ShadcnColors.border)),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Row(
                              children: [
                                const Text(
                                  'PAYLOAD INSPECTOR',
                                  style: TextStyle(
                                    fontSize: 10,
                                    fontWeight: FontWeight.bold,
                                    color: ShadcnColors.foregroundMuted,
                                    letterSpacing: 0.5,
                                  ),
                                ),
                                const Spacer(),
                                if (widget.onForkAtStep != null)
                                  InkWell(
                                    onTap: () => widget.onForkAtStep!(step),
                                    child: Container(
                                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                      decoration: BoxDecoration(
                                        color: ShadcnColors.primary.withOpacity(0.15),
                                        borderRadius: BorderRadius.circular(4),
                                        border: Border.all(color: ShadcnColors.primary),
                                      ),
                                      child: const Text(
                                        'Разветвить сессию отсюда',
                                        style: TextStyle(
                                          fontSize: 10,
                                          fontWeight: FontWeight.bold,
                                          color: ShadcnColors.primary,
                                        ),
                                      ),
                                    ),
                                  ),
                              ],
                            ),
                            const SizedBox(height: 6),
                            Container(
                              width: double.infinity,
                              padding: const EdgeInsets.all(8),
                              decoration: BoxDecoration(
                                color: ShadcnColors.background,
                                borderRadius: BorderRadius.circular(6),
                                border: Border.all(color: ShadcnColors.border),
                              ),
                              child: SelectableText(
                                _formatJson(step.payloadJson),
                                style: const TextStyle(
                                  fontSize: 11,
                                  height: 1.4,
                                  color: ShadcnColors.foreground,
                                  fontFamily: 'JetBrains Mono',
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                  ],
                ),
              );
            },
          ),
        ),
      ],
    );
  }

  String _formatJson(String raw) {
    try {
      final decoded = jsonDecode(raw);
      return const JsonEncoder.withIndent('  ').convert(decoded);
    } catch (_) {
      return raw;
    }
  }
}
