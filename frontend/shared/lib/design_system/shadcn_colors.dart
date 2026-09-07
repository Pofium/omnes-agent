// Design System Color Tokens — shadcn/ui + reui Cyber Aesthetic
import 'package:flutter/material.dart';

class ShadcnColors {
  // Backgrounds
  static const Color background = Color(0xFF09090B);
  static const Color surface = Color(0xFF12141A);
  static const Color card = Color(0xFF141720);
  static const Color cardElevated = Color(0xFF1C202C);

  // Borders
  static const Color border = Color(0xFF242836);
  static const Color borderSubtle = Color(0x1AFFFFFF); // rgba(255,255,255,0.1)
  static const Color borderActive = Color(0xFF00E5FF);

  // Brand / Primary Cyber Accent (matching Omnes Agent metallic cyan wings)
  static const Color primary = Color(0xFF00E5FF);
  static const Color primaryMuted = Color(0xFF0284C7);
  static const Color primaryGlow = Color(0x4000E5FF);
  static const Color primaryGradientStart = Color(0xFF00E5FF);
  static const Color primaryGradientEnd = Color(0xFF0284C7);

  // Text / Foregrounds
  static const Color foreground = Color(0xFFF4F4F5);
  static const Color foregroundMuted = Color(0xFF94A3B8);
  static const Color foregroundSubtle = Color(0xFF64748B);

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
