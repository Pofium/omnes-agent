// ShadcnButton — Cyber-styled button variants (primary, secondary, outline, ghost, destructive)
import 'package:flutter/material.dart';
import 'shadcn_colors.dart';

enum ShadcnButtonVariant {
  primary,
  secondary,
  outline,
  ghost,
  destructive,
}

enum ShadcnButtonSize {
  sm,
  md,
  lg,
}

class ShadcnButton extends StatelessWidget {
  final String label;
  final VoidCallback? onPressed;
  final ShadcnButtonVariant variant;
  final ShadcnButtonSize size;
  final IconData? icon;
  final bool isLoading;
  final bool fullWidth;

  const ShadcnButton({
    super.key,
    required this.label,
    required this.onPressed,
    this.variant = ShadcnButtonVariant.primary,
    this.size = ShadcnButtonSize.md,
    this.icon,
    this.isLoading = false,
    this.fullWidth = false,
  });

  @override
  Widget build(BuildContext context) {
    Color bg;
    Color fg;
    Border? border;

    switch (variant) {
      case ShadcnButtonVariant.primary:
        bg = ShadcnColors.primary;
        fg = const Color(0xFF09090B);
        border = null;
        break;
      case ShadcnButtonVariant.secondary:
        bg = ShadcnColors.cardElevated;
        fg = ShadcnColors.foreground;
        border = Border.all(color: ShadcnColors.border);
        break;
      case ShadcnButtonVariant.outline:
        bg = Colors.transparent;
        fg = ShadcnColors.foreground;
        border = Border.all(color: ShadcnColors.border);
        break;
      case ShadcnButtonVariant.ghost:
        bg = Colors.transparent;
        fg = ShadcnColors.foregroundMuted;
        border = null;
        break;
      case ShadcnButtonVariant.destructive:
        bg = ShadcnColors.destructive;
        fg = Colors.white;
        border = null;
        break;
    }

    double height;
    double fontSize;
    EdgeInsets padding;

    switch (size) {
      case ShadcnButtonSize.sm:
        height = 36;
        fontSize = 13;
        padding = const EdgeInsets.symmetric(horizontal: 12);
        break;
      case ShadcnButtonSize.md:
        height = 44;
        fontSize = 14;
        padding = const EdgeInsets.symmetric(horizontal: 16);
        break;
      case ShadcnButtonSize.lg:
        height = 52;
        fontSize = 16;
        padding = const EdgeInsets.symmetric(horizontal: 22);
        break;
    }

    final content = Row(
      mainAxisSize: fullWidth ? MainAxisSize.max : MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        if (isLoading) ...[
          SizedBox(
            width: fontSize * 1.2,
            height: fontSize * 1.2,
            child: CircularProgressIndicator(strokeWidth: 2, color: fg),
          ),
          const SizedBox(width: 8),
        ] else if (icon != null) ...[
          Icon(icon, size: fontSize * 1.2, color: fg),
          const SizedBox(width: 8),
        ],
        Text(
          label,
          style: TextStyle(
            color: fg,
            fontSize: fontSize,
            fontWeight: FontWeight.w600,
            letterSpacing: 0.2,
          ),
        ),
      ],
    );

    return AnimatedOpacity(
      duration: const Duration(milliseconds: 150),
      opacity: onPressed == null || isLoading ? 0.6 : 1.0,
      child: Container(
        height: height,
        width: fullWidth ? double.infinity : null,
        decoration: BoxDecoration(
          color: bg,
          borderRadius: BorderRadius.circular(10),
          border: border,
          boxShadow: variant == ShadcnButtonVariant.primary && onPressed != null && !isLoading
              ? [
                  BoxShadow(
                    color: ShadcnColors.primaryGlow,
                    blurRadius: 10,
                    offset: const Offset(0, 2),
                  ),
                ]
              : null,
        ),
        child: Material(
          color: Colors.transparent,
          child: InkWell(
            borderRadius: BorderRadius.circular(10),
            onTap: (onPressed == null || isLoading) ? null : onPressed,
            child: Padding(
              padding: padding,
              child: content,
            ),
          ),
        ),
      ),
    );
  }
}
