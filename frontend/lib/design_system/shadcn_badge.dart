// ShadcnBadge — Micro status badge chips for health, models, and categories
import 'package:flutter/material.dart';
import 'shadcn_colors.dart';

enum ShadcnBadgeVariant {
  cyber,
  success,
  warning,
  destructive,
  neutral,
}

class ShadcnBadge extends StatelessWidget {
  final String label;
  final ShadcnBadgeVariant variant;
  final bool showDot;
  final IconData? icon;

  const ShadcnBadge({
    super.key,
    required this.label,
    this.variant = ShadcnBadgeVariant.neutral,
    this.showDot = false,
    this.icon,
  });

  @override
  Widget build(BuildContext context) {
    Color bg;
    Color fg;
    Color border;

    switch (variant) {
      case ShadcnBadgeVariant.cyber:
        bg = ShadcnColors.primary.withOpacity(0.12);
        fg = ShadcnColors.primary;
        border = ShadcnColors.primary.withOpacity(0.3);
        break;
      case ShadcnBadgeVariant.success:
        bg = ShadcnColors.successMuted;
        fg = ShadcnColors.success;
        border = ShadcnColors.success.withOpacity(0.3);
        break;
      case ShadcnBadgeVariant.warning:
        bg = ShadcnColors.warningMuted;
        fg = ShadcnColors.warning;
        border = ShadcnColors.warning.withOpacity(0.3);
        break;
      case ShadcnBadgeVariant.destructive:
        bg = ShadcnColors.destructiveMuted;
        fg = ShadcnColors.destructive;
        border = ShadcnColors.destructive.withOpacity(0.3);
        break;
      case ShadcnBadgeVariant.neutral:
        bg = ShadcnColors.cardElevated;
        fg = ShadcnColors.foregroundMuted;
        border = ShadcnColors.border;
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: bg,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: border, width: 1),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          if (showDot) ...[
            Container(
              width: 6,
              height: 6,
              decoration: BoxDecoration(
                color: fg,
                shape: BoxShape.circle,
                boxShadow: [
                  BoxShadow(
                    color: fg.withOpacity(0.6),
                    blurRadius: 4,
                    spreadRadius: 1,
                  ),
                ],
              ),
            ),
            const SizedBox(width: 5),
          ] else if (icon != null) ...[
            Icon(icon, size: 11, color: fg),
            const SizedBox(width: 4),
          ],
          Text(
            label,
            style: TextStyle(
              color: fg,
              fontSize: 11,
              fontWeight: FontWeight.w600,
              letterSpacing: 0.3,
            ),
          ),
        ],
      ),
    );
  }
}
