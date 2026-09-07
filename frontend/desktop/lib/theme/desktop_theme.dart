// Desktop Workstation Theme: ReUI / Cyber Dark-First Theme.

import 'package:flutter/material.dart';

class DesktopTheme {
  // Base Surface Colors (Graphite & Slate)
  static const Color bgCanvas = Color(0xFF090D16);
  static const Color bgSurface = Color(0xFF0F172A);
  static const Color bgSurfaceElevated = Color(0xFF1E293B);
  static const Color bgSidebar = Color(0xFF0B1120);

  // Border & Dividers
  static const Color borderSubtle = Color(0xFF1E293B);
  static const Color borderMedium = Color(0xFF334155);
  static const Color borderFocus = Color(0xFF38BDF8);

  // Brand & Accent Colors (Electric Cyan & Sapphire)
  static const Color accentCyan = Color(0xFF00F2FE);
  static const Color accentSky = Color(0xFF38BDF8);
  static const Color accentBlue = Color(0xFF2563EB);
  static const Color accentPurple = Color(0xFF8B5CF6);

  // Semantic Status
  static const Color statusSuccess = Color(0xFF10B981);
  static const Color statusWarning = Color(0xFFF59E0B);
  static const Color statusError = Color(0xFFEF4444);
  static const Color statusInfo = Color(0xFF06B6D4);

  // Text Hierarchy
  static const Color textPrimary = Color(0xFFF8FAFC);
  static const Color textSecondary = Color(0xFF94A3B8);
  static const Color textMuted = Color(0xFF64748B);

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: bgCanvas,
      primaryColor: accentSky,
      colorScheme: const ColorScheme.dark(
        primary: accentSky,
        secondary: accentCyan,
        surface: bgSurface,
        error: statusError,
      ),
      fontFamily: 'Segoe UI',
      dividerColor: borderSubtle,
      cardTheme: CardTheme(
        color: bgSurface,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
          side: const BorderSide(color: borderSubtle),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: bgSurfaceElevated,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: borderSubtle),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: borderSubtle),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: accentSky, width: 1.5),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      ),
    );
  }
}
