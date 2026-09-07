// Desktop Workstation Theme: OmnesAgent ADE Dual Theme (Dark & Light).

import 'package:flutter/material.dart';
import 'package:get/get.dart';

class DesktopThemeController extends GetxController {
  static DesktopThemeController get to => Get.find();
  final isDarkMode = true.obs;

  void toggleTheme() {
    isDarkMode.value = !isDarkMode.value;
    Get.changeThemeMode(isDarkMode.value ? ThemeMode.dark : ThemeMode.light);
  }
}

class DesktopTheme {
  // Dark Palette (Default)
  static const Color bgCanvasDark = Color(0xFF0D1117);
  static const Color bgSurfaceDark = Color(0xFF161D26);
  static const Color bgSurfaceElevatedDark = Color(0xFF1C2430);
  static const Color bgSidebarDark = Color(0xFF0E131A);

  static const Color borderSubtleDark = Color(0xFF222B38);
  static const Color borderMediumDark = Color(0xFF30363D);
  static const Color borderFocusDark = Color(0xFF00D2FF);

  static const Color textPrimaryDark = Color(0xFFF1F5F9);
  static const Color textSecondaryDark = Color(0xFF94A3B8);
  static const Color textMutedDark = Color(0xFF64748B);

  // Light Palette
  static const Color bgCanvasLight = Color(0xFFFFFFFF);
  static const Color bgSurfaceLight = Color(0xFFF8FAFC);
  static const Color bgSurfaceElevatedLight = Color(0xFFFFFFFF);
  static const Color bgSidebarLight = Color(0xFFF1F5F9);

  static const Color borderSubtleLight = Color(0xFFE2E8F0);
  static const Color borderMediumLight = Color(0xFFCBD5E1);
  static const Color borderFocusLight = Color(0xFF0284C7);

  static const Color textPrimaryLight = Color(0xFF0F172A);
  static const Color textSecondaryLight = Color(0xFF475569);
  static const Color textMutedLight = Color(0xFF94A3B8);

  // Brand Accents
  static const Color accentCyan = Color(0xFF00D2FF);
  static const Color accentSky = Color(0xFF38BDF8);
  static const Color accentSapphire = Color(0xFF0284C7);
  static const Color accentBlue = Color(0xFF2563EB);

  // Semantic Status Colors
  static const Color statusSuccess = Color(0xFF10B981);
  static const Color statusWarning = Color(0xFFF59E0B);
  static const Color statusError = Color(0xFFEF4444);
  static const Color statusInfo = Color(0xFF38BDF8);

  // Dynamic getters based on active theme
  static bool get _isDark =>
      Get.isRegistered<DesktopThemeController>()
          ? DesktopThemeController.to.isDarkMode.value
          : true;

  static Color get bgCanvas => _isDark ? bgCanvasDark : bgCanvasLight;
  static Color get bgSurface => _isDark ? bgSurfaceDark : bgSurfaceLight;
  static Color get bgSurfaceElevated =>
      _isDark ? bgSurfaceElevatedDark : bgSurfaceElevatedLight;
  static Color get bgSidebar => _isDark ? bgSidebarDark : bgSidebarLight;

  static Color get borderSubtle =>
      _isDark ? borderSubtleDark : borderSubtleLight;
  static Color get borderMedium =>
      _isDark ? borderMediumDark : borderMediumLight;
  static Color get borderFocus => _isDark ? borderFocusDark : borderFocusLight;

  static Color get textPrimary => _isDark ? textPrimaryDark : textPrimaryLight;
  static Color get textSecondary =>
      _isDark ? textSecondaryDark : textSecondaryLight;
  static Color get textMuted => _isDark ? textMutedDark : textMutedLight;

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: bgCanvasDark,
      primaryColor: accentCyan,
      colorScheme: const ColorScheme.dark(
        primary: accentCyan,
        secondary: accentSky,
        surface: bgSurfaceDark,
        error: statusError,
      ),
      fontFamily: 'Segoe UI',
      dividerColor: borderSubtleDark,
      cardTheme: CardTheme(
        color: bgSurfaceDark,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
          side: const BorderSide(color: borderSubtleDark),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: bgSurfaceElevatedDark,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: borderSubtleDark),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: borderSubtleDark),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: borderFocusDark, width: 1.5),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      ),
    );
  }

  static ThemeData get lightTheme {
    return ThemeData(
      brightness: Brightness.light,
      scaffoldBackgroundColor: bgCanvasLight,
      primaryColor: accentSapphire,
      colorScheme: const ColorScheme.light(
        primary: accentSapphire,
        secondary: accentSky,
        surface: bgSurfaceLight,
        error: statusError,
      ),
      fontFamily: 'Segoe UI',
      dividerColor: borderSubtleLight,
      cardTheme: CardTheme(
        color: bgSurfaceLight,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
          side: const BorderSide(color: borderSubtleLight),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: bgSurfaceElevatedLight,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: borderSubtleLight),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: borderSubtleLight),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(6),
          borderSide: const BorderSide(color: borderFocusLight, width: 1.5),
        ),
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      ),
    );
  }
}
