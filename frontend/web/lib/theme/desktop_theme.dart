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
  // Typography Triad (OmnesProducts v1.1 §06)
  static const String fontHeading = 'Montserrat';
  static const String fontBody = 'Segoe UI';
  static const String fontMono = 'JetBrains Mono';

  // Dark Palette (60% Dark Charcoal base around #111827)
  static const Color bgCanvasDark = Color(0xFF111827);
  static const Color bgSurfaceDark = Color(0xFF161E2E);
  static const Color bgSurfaceElevatedDark = Color(0xFF1F2937);
  static const Color bgSidebarDark = Color(0xFF0D1117);

  static const Color borderSubtleDark = Color(0xFF232C3D);
  static const Color borderMediumDark = Color(0xFF334155);
  static const Color borderFocusDark = Color(0xFFFF6B00);

  static const Color textPrimaryDark = Color(0xFFF4F5F7);
  static const Color textSecondaryDark = Color(0xFFCBD5E1);
  static const Color textMutedDark = Color(0xFF94A3B8);

  // Light Palette (30% Off-White / Paper neutral around #F4F5F7)
  static const Color bgCanvasLight = Color(0xFFFFFFFF);
  static const Color bgSurfaceLight = Color(0xFFFAFBFC);
  static const Color bgSurfaceElevatedLight = Color(0xFFFFFFFF);
  static const Color bgSidebarLight = Color(0xFFF4F5F7);

  static const Color borderSubtleLight = Color(0xFFD8DCE3);
  static const Color borderMediumLight = Color(0xFF9AA3B2);
  static const Color borderFocusLight = Color(0xFFE05E00);

  static const Color textPrimaryLight = Color(0xFF111827);
  static const Color textSecondaryLight = Color(0xFF1E293B);
  static const Color textMutedLight = Color(0xFF5B6472);

  // Brand Accents (<=10% Brand Orange #FF6B00; secondary code blue preserved)
  static const Color accentBrand = Color(0xFFFF6B00);
  static const Color accentBrandDark = Color(0xFFE05E00);
  static const Color accentCyan = Color(0xFFFF6B00);
  static const Color accentSky = Color(0xFF38BDF8);
  static const Color accentSapphire = Color(0xFF0284C7);
  static const Color accentBlue = Color(0xFF2563EB);

  // Semantic Status Colors
  static const Color statusSuccess = Color(0xFF10B981);
  static const Color statusWarning = Color(0xFFF59E0B);
  static const Color statusError = Color(0xFFEF4444);
  static const Color statusInfo = Color(0xFF38BDF8);

  // Public theme check
  static bool get isDark =>
      Get.isRegistered<DesktopThemeController>()
          ? DesktopThemeController.to.isDarkMode.value
          : true;

  // Dynamic getters based on active theme
  static Color get bgCanvas => isDark ? bgCanvasDark : bgCanvasLight;
  static Color get bgSurface => isDark ? bgSurfaceDark : bgSurfaceLight;
  static Color get bgSurfaceElevated =>
      isDark ? bgSurfaceElevatedDark : bgSurfaceElevatedLight;
  static Color get bgSidebar => isDark ? bgSidebarDark : bgSidebarLight;

  static Color get borderSubtle =>
      isDark ? borderSubtleDark : borderSubtleLight;
  static Color get borderMedium =>
      isDark ? borderMediumDark : borderMediumLight;
  static Color get borderFocus => isDark ? borderFocusDark : borderFocusLight;

  static Color get textPrimary => isDark ? textPrimaryDark : textPrimaryLight;
  static Color get textSecondary =>
      isDark ? textSecondaryDark : textSecondaryLight;
  static Color get textMuted => isDark ? textMutedDark : textMutedLight;

  // Button styling (Dark: charcoal #1F2937, solid #FF6B00 reserved for main CTA)
  static Color get btnPrimaryBg =>
      isDark ? const Color(0xFF1F2937) : const Color(0xFF111827);
  static Color get btnPrimaryFg => const Color(0xFFF4F5F7);
  static Color get btnCtaBg => isDark ? accentBrand : accentBrandDark;
  static Color get btnSecondaryBg =>
      isDark ? const Color(0xFF18202F) : const Color(0xFFE5E7EB);
  static Color get btnSecondaryFg =>
      isDark ? const Color(0xFFF4F5F7) : const Color(0xFF111827);

  static const TextTheme _brandTextTheme = TextTheme(
    displayLarge: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    displayMedium: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    displaySmall: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    headlineLarge: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    headlineMedium: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    headlineSmall: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    titleLarge: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    titleMedium: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    titleSmall: TextStyle(fontFamily: fontHeading, fontFamilyFallback: ['Segoe UI', 'Inter', 'sans-serif']),
    bodyLarge: TextStyle(fontFamily: fontBody, fontFamilyFallback: ['Inter', 'Segoe UI', 'sans-serif']),
    bodyMedium: TextStyle(fontFamily: fontBody, fontFamilyFallback: ['Inter', 'Segoe UI', 'sans-serif']),
    bodySmall: TextStyle(fontFamily: fontBody, fontFamilyFallback: ['Inter', 'Segoe UI', 'sans-serif']),
    labelLarge: TextStyle(fontFamily: fontBody, fontFamilyFallback: ['Inter', 'Segoe UI', 'sans-serif']),
    labelMedium: TextStyle(fontFamily: fontBody, fontFamilyFallback: ['Inter', 'Segoe UI', 'sans-serif']),
    labelSmall: TextStyle(fontFamily: fontBody, fontFamilyFallback: ['Inter', 'Segoe UI', 'sans-serif']),
  );

  static ThemeData get darkTheme {
    return ThemeData(
      brightness: Brightness.dark,
      scaffoldBackgroundColor: bgCanvasDark,
      primaryColor: accentBrand,
      colorScheme: const ColorScheme.dark(
        primary: accentBrand,
        secondary: accentSky,
        surface: bgSurfaceDark,
        error: statusError,
      ),
      fontFamily: fontBody,
      fontFamilyFallback: const ['Inter', 'Segoe UI', 'Roboto', 'sans-serif'],
      textTheme: _brandTextTheme,
      dividerColor: borderSubtleDark,
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: const Color(0xFF1F2937),
          foregroundColor: const Color(0xFFF4F5F7),
          elevation: 0,
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
          textStyle: const TextStyle(fontWeight: FontWeight.w600, fontSize: 13),
        ),
      ),
      cardTheme: CardTheme(
        color: bgSurfaceDark,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
          side: const BorderSide(color: borderSubtleDark),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: false,
        border: InputBorder.none,
        enabledBorder: InputBorder.none,
        focusedBorder: InputBorder.none,
        errorBorder: InputBorder.none,
        focusedErrorBorder: InputBorder.none,
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      ),
    );
  }

  static ThemeData get lightTheme {
    return ThemeData(
      brightness: Brightness.light,
      scaffoldBackgroundColor: bgCanvasLight,
      primaryColor: const Color(0xFF111827),
      colorScheme: const ColorScheme.light(
        primary: accentBrandDark,
        secondary: accentSky,
        surface: bgSurfaceLight,
        error: statusError,
      ),
      fontFamily: fontBody,
      fontFamilyFallback: const ['Inter', 'Segoe UI', 'Roboto', 'sans-serif'],
      textTheme: _brandTextTheme,
      dividerColor: borderSubtleLight,
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: const Color(0xFF111827),
          foregroundColor: const Color(0xFFF4F5F7),
          elevation: 0,
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
          textStyle: const TextStyle(fontWeight: FontWeight.w600, fontSize: 13),
        ),
      ),
      cardTheme: CardTheme(
        color: bgSurfaceLight,
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
          side: const BorderSide(color: borderSubtleLight),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: false,
        border: InputBorder.none,
        enabledBorder: InputBorder.none,
        focusedBorder: InputBorder.none,
        errorBorder: InputBorder.none,
        focusedErrorBorder: InputBorder.none,
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      ),
    );
  }
}
