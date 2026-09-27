// Turn rail («палочки» слева от чата, как в ZCode): одна засечка на ход
// пользователя, активная подсвечена, клик прыгает к ходу. Показывает длину
// диалога и позицию в нём; наведение — подсказка с текстом хода.

import 'package:flutter/material.dart';

/// Тик-маркер одного хода диалога на вертикальной линейке.
class TurnRail extends StatelessWidget {
  /// Тексты ходов для подсказок (по одному на засечку).
  final List<String> turnPreviews;
  final int activeIndex;
  final ValueChanged<int> onTapTurn;

  const TurnRail({
    super.key,
    required this.turnPreviews,
    required this.activeIndex,
    required this.onTapTurn,
  });

  @override
  Widget build(BuildContext context) {
    final count = turnPreviews.length;
    if (count == 0) return const SizedBox.shrink();

    return LayoutBuilder(
      builder: (context, constraints) {
        final height = constraints.maxHeight;
        // Равномерная сетка засечек: шаг 6..18 px, по центру колонки.
        final step = count > 1
            ? ((height - 24) / (count - 1)).clamp(6.0, 18.0)
            : 10.0;
        final span = step * (count - 1);
        final top = (height - span) / 2;

        return SizedBox(
          width: 18,
          child: Stack(
            clipBehavior: Clip.none,
            children: [
              for (var i = 0; i < count; i++)
                Positioned(
                  left: 0,
                  top: top + step * i - 5,
                  child: _TurnTick(
                    label: 'Ход ${i + 1} из $count\n'
                        '${turnPreviews[i]}',
                    active: i == activeIndex.clamp(0, count - 1),
                    onTap: () => onTapTurn(i),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}

class _TurnTick extends StatelessWidget {
  final String label;
  final bool active;
  final VoidCallback onTap;

  const _TurnTick({
    required this.label,
    required this.active,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: label,
      preferBelow: false,
      waitDuration: const Duration(milliseconds: 250),
      margin: const EdgeInsets.only(left: 24),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: Container(
          width: 18,
          height: 12,
          alignment: Alignment.centerLeft,
          padding: const EdgeInsets.symmetric(horizontal: 2),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 150),
            width: active ? 13.0 : 8.0,
            height: active ? 3.0 : 2.0,
            decoration: BoxDecoration(
              color: active
                  ? const Color(0xFF00D2FF)
                  : Colors.white.withOpacity(0.22),
              borderRadius: BorderRadius.circular(1.5),
            ),
          ),
        ),
      ),
    );
  }
}
