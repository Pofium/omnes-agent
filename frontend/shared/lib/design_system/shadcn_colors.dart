// Design System Color Tokens — OmnesProducts v1.1 (Dark Charcoal + Off-White + Brand Orange)
import 'package:flutter/material.dart';

class ShadcnColors {
  // Backgrounds (60% Dark Charcoal base)
  static const Color background = Color(0xFF0D1117);
  static const Color surface = Color(0xFF111827);
  static const Color card = Color(0xFF18202F);
  static const Color cardElevated = Color(0xFF1F2937);

  // Borders
  static const Color border = Color(0xFF283244);
  static const Color borderSubtle = Color(0x1AFFFFFF); // rgba(255,255,255,0.1)
  static const Color borderActive = Color(0xFFFF6B00);

  // Brand / Signal Accent (<=10% Brand Orange #FF6B00)
  static const Color primary = Color(0xFFFF6B00);
  static const Color primaryMuted = Color(0xFFE05E00);
  static const Color primaryGlow = Color(0x26FF6B00);
  static const Color primaryGradientStart = Color(0xFFFF6B00);
  static const Color primaryGradientEnd = Color(0xFFE05E00);

  // Text / Foregrounds (Off-White #F4F5F7 primary)
  static const Color foreground = Color(0xFFF4F5F7);
  static const Color foregroundMuted = Color(0xFF94A3B8);
  static const Color foregroundSubtle = Color(0xFF5B6472);

  // Statuses
  static const Color success = Color(0xFF10B981);
  static const Color successMuted = Color(0x2610B981);
  static const Color warning = Color(0xFFF59E0B);
  static const Color warningMuted = Color(0x26F59E0B);
  static const Color destructive = Color(0xFFEF4444);
  static const Color destructiveMuted = Color(0x26EF4444);
  static const Color info = Color(0xFF38BDF8);
  static const Color infoMuted = Color(0x2638BDF8);
}
