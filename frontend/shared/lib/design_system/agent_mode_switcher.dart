import 'package:flutter/material.dart';
import '../core/gateway/models/agent_mode.dart';
import 'shadcn_colors.dart';
import 'ui_icons.dart';

/// AgentModeSwitcher presents interactive cyber chips for selecting the
/// execution mode (Speed/Fast, Deep Code, Architect, Ralph Loop).
class AgentModeSwitcher extends StatelessWidget {
  final AgentExecutionMode currentMode;
  final ValueChanged<AgentExecutionMode> onModeChanged;

  const AgentModeSwitcher({
    super.key,
    required this.currentMode,
    required this.onModeChanged,
  });

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: AgentExecutionMode.values.map((mode) {
          final isSelected = mode == currentMode;
          return Padding(
            padding: const EdgeInsets.only(right: 6),
            child: Tooltip(
              message: mode.description,
              preferBelow: false,
              textStyle: const TextStyle(
                color: Color(0xFFF8FAFC),
                fontSize: 11,
                fontFamily: 'Segoe UI',
                fontWeight: FontWeight.w500,
              ),
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: BoxDecoration(
                color: const Color(0xFF131A29),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.6), width: 0.8),
                boxShadow: const [
                  BoxShadow(color: Color(0x66000000), blurRadius: 10, offset: Offset(0, 3)),
                ],
              ),
              child: InkWell(
                onTap: () => onModeChanged(mode),
                borderRadius: BorderRadius.circular(6),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 150),
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    color: isSelected
                        ? ShadcnColors.primary.withOpacity(0.15)
                        : ShadcnColors.card,
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(
                      color: isSelected
                          ? ShadcnColors.primary
                          : ShadcnColors.border,
                      width: 1,
                    ),
                    boxShadow: isSelected
                        ? [
                            BoxShadow(
                              color: ShadcnColors.primaryGlow,
                              blurRadius: 6,
                            ),
                          ]
                        : null,
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      UiIcon(
                        mode.icon,
                        size: 12,
                        color: isSelected
                            ? ShadcnColors.primary
                            : ShadcnColors.foregroundMuted,
                      ),
                      const SizedBox(width: 5),
                      Text(
                        mode.label,
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                          color: isSelected
                              ? ShadcnColors.primary
                              : ShadcnColors.foregroundMuted,
                          fontFamily: 'JetBrains Mono',
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}
