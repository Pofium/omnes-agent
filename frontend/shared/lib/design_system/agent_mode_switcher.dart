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
    final isLight = Theme.of(context).brightness == Brightness.light;

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: AgentExecutionMode.values.map((mode) {
          final isSelected = mode == currentMode;
          final unselectedBg = isLight ? const Color(0xFF334155) : const Color(0xFF282D37);
          final unselectedFg = Colors.white;
          final unselectedBorder = isLight ? const Color(0xFF334155) : const Color(0xFF384152);

          return Padding(
            padding: const EdgeInsets.only(right: 6),
            child: Tooltip(
              message: mode.description,
              preferBelow: false,
              textStyle: TextStyle(
                color: isLight ? const Color(0xFF020617) : const Color(0xFFF8FAFC),
                fontSize: 11,
                fontFamily: 'Segoe UI',
                fontWeight: FontWeight.w500,
              ),
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: BoxDecoration(
                color: isLight ? Colors.white : const Color(0xFF131A29),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.6), width: 0.8),
                boxShadow: [
                  BoxShadow(color: isLight ? Colors.black.withOpacity(0.1) : const Color(0x66000000), blurRadius: 10, offset: const Offset(0, 3)),
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
                        ? (isLight ? const Color(0xFF00D2FF).withOpacity(0.18) : ShadcnColors.primary.withOpacity(0.15))
                        : unselectedBg,
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(
                      color: isSelected
                          ? (isLight ? const Color(0xFF0284C7) : ShadcnColors.primary)
                          : unselectedBorder,
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
                            ? (isLight ? const Color(0xFF0284C7) : ShadcnColors.primary)
                            : unselectedFg,
                      ),
                      const SizedBox(width: 5),
                      Text(
                        mode.label,
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: isSelected ? FontWeight.bold : FontWeight.w500,
                          color: isSelected
                              ? (isLight ? const Color(0xFF0284C7) : ShadcnColors.primary)
                              : unselectedFg,
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
