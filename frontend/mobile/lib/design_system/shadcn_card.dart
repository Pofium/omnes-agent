// ShadcnCard — Modern rounded container with subtle 1px border and optional cyber glow.
import 'package:flutter/material.dart';
import 'shadcn_colors.dart';

class ShadcnCard extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final VoidCallback? onTap;
  final bool isActive;
  final bool isGlowing;
  final Color? backgroundColor;
  final Color? borderColor;
  final double borderRadius;

  const ShadcnCard({
    super.key,
    required this.child,
    this.padding = const EdgeInsets.all(16.0),
    this.margin,
    this.onTap,
    this.isActive = false,
    this.isGlowing = false,
    this.backgroundColor,
    this.borderColor,
    this.borderRadius = 14.0,
  });

  @override
  Widget build(BuildContext context) {
    final effectiveBorderColor = borderColor ??
        (isActive
            ? ShadcnColors.primary
            : (isGlowing ? ShadcnColors.primary.withOpacity(0.5) : ShadcnColors.border));

    final effectiveBg = backgroundColor ?? ShadcnColors.card;

    final cardWidget = Container(
      margin: margin,
      padding: padding,
      decoration: BoxDecoration(
        color: effectiveBg,
        borderRadius: BorderRadius.circular(borderRadius),
        border: Border.all(
          color: effectiveBorderColor,
          width: isActive || isGlowing ? 1.5 : 1.0,
        ),
        boxShadow: (isActive || isGlowing)
            ? [
                BoxShadow(
                  color: ShadcnColors.primaryGlow,
                  blurRadius: 14,
                  spreadRadius: 1,
                  offset: const Offset(0, 2),
                ),
              ]
            : null,
      ),
      child: child,
    );

    if (onTap != null) {
      return Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(borderRadius),
          hoverColor: ShadcnColors.primary.withOpacity(0.05),
          splashColor: ShadcnColors.primary.withOpacity(0.1),
          child: cardWidget,
        ),
      );
    }

    return cardWidget;
  }
}
