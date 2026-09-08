// ShadcnTheme — Global dark cyber theme for Flutter MaterialApp
import 'package:flutter/material.dart';
import 'shadcn_colors.dart';

class ShadcnTheme {
  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: ShadcnColors.background,
      primaryColor: ShadcnColors.primary,
      cardColor: ShadcnColors.card,
      dividerColor: ShadcnColors.border,
      colorScheme: const ColorScheme.dark(
        primary: ShadcnColors.primary,
        secondary: ShadcnColors.primaryMuted,
        surface: ShadcnColors.card,
        error: ShadcnColors.destructive,
        onPrimary: Color(0xFF09090B),
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
          color: ShadcnColors.foreground,
          fontSize: 17,
          fontWeight: FontWeight.w600,
          letterSpacing: 0.2,
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
