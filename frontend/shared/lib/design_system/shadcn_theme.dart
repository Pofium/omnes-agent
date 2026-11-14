// ShadcnTheme — Global OmnesProducts v1.1 dark theme for Flutter MaterialApp
import 'package:flutter/material.dart';
import 'shadcn_colors.dart';

class ShadcnTheme {
  static const String fontHeading = 'Montserrat';
  static const String fontBody = 'Segoe UI';
  static const String fontMono = 'JetBrains Mono';

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: ShadcnColors.background,
      primaryColor: ShadcnColors.primary,
      cardColor: ShadcnColors.card,
      dividerColor: ShadcnColors.border,
      fontFamily: fontBody,
      fontFamilyFallback: const ['Inter', 'Segoe UI', 'Roboto', 'sans-serif'],
      colorScheme: const ColorScheme.dark(
        primary: ShadcnColors.primary,
        secondary: ShadcnColors.primaryMuted,
        surface: ShadcnColors.card,
        error: ShadcnColors.destructive,
        onPrimary: ShadcnColors.foreground,
        onSurface: ShadcnColors.foreground,
        onError: Colors.white,
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: ShadcnColors.background,
        elevation: 0,
        centerTitle: true,
        scrolledUnderElevation: 0,
        iconTheme: IconThemeData(color: ShadcnColors.foreground),
        titleTextStyle: TextStyle(
          fontFamily: fontHeading,
          fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif'],
          color: ShadcnColors.foreground,
          fontSize: 17,
          fontWeight: FontWeight.w600,
          letterSpacing: 0.3,
        ),
      ),
      dialogTheme: DialogTheme(
        backgroundColor: ShadcnColors.card,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: const BorderSide(color: ShadcnColors.border),
        ),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: ShadcnColors.cardElevated,
        contentTextStyle: const TextStyle(color: ShadcnColors.foreground),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(10),
          side: const BorderSide(color: ShadcnColors.border),
        ),
        behavior: SnackBarBehavior.floating,
      ),
    );
  }
}
