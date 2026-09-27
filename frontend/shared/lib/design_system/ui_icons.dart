// Shared inline SVG icon set (normative UI icon source — see AGENTS.md
// "UI Conventions — Icons"): no emoji anywhere in the UI, icons are SVG only.
//
// Every glyph is a minimal 24×24 stroke-style SVG string (Feather/Lucide
// style). `UiIcon` renders them via flutter_svg; `color` maps to the SVG
// `stroke`, so icons follow theme colors. No asset files, no icon fonts.

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

/// Icon identifiers → inline SVG bodies (viewBox 0 0 24 24, stroke style).
const Map<String, String> kUiIconSvg = <String, String>{
  // Check mark — success / done.
  'check': '<path d="M4 12.5l5 5L20 6.5" fill="none"/>',
  // Warning triangle.
  'warning':
      '<path d="M12 3.5L2.5 20h19L12 3.5z" fill="none"/><path d="M12 10v4.5" fill="none"/><path d="M12 17.2v.4" fill="none"/>',
  // Link (chain).
  'link':
      '<path d="M10 13.5a4.2 4.2 0 006.4.5l2.8-2.8a4.2 4.2 0 00-6-6L11.6 6.8" fill="none"/><path d="M14 10.5a4.2 4.2 0 00-6.4-.5l-2.8 2.8a4.2 4.2 0 006 6l1.6-1.6" fill="none"/>',
  // Git branch — forks / sessions.
  'branch':
      '<circle cx="6" cy="5" r="2.4" fill="none"/><circle cx="6" cy="19" r="2.4" fill="none"/><circle cx="18" cy="8" r="2.4" fill="none"/><path d="M6 7.5v9" fill="none"/><path d="M18 10.5c0 3.5-4 3-7 4.5" fill="none"/>',
  // Package box — compacted context.
  'package':
      '<path d="M12 2.8l8.5 4.4v9.6L12 21.2l-8.5-4.4V7.2L12 2.8z" fill="none"/><path d="M3.5 7.2L12 11.6l8.5-4.4" fill="none"/><path d="M12 11.6v9.6" fill="none"/>',
  // Pencil — commit / edit.
  'pencil':
      '<path d="M16.8 3.8a2.3 2.3 0 013.3 3.3L7.5 19.7 3 21l1.3-4.5L16.8 3.8z" fill="none"/>',
  // Magnifier — search / inspect.
  'search':
      '<circle cx="10.5" cy="10.5" r="6.5" fill="none"/><path d="M15.3 15.3L21 21" fill="none"/>',
  // Shield — permission modes.
  'shield': '<path d="M12 2.8l7.5 3v6c0 4.6-3.2 8-7.5 9.4-4.3-1.4-7.5-4.8-7.5-9.4v-6l7.5-3z" fill="none"/>',
  // Filled dot — status / model indicator.
  'dot': '<circle cx="12" cy="12" r="5" fill="currentColor" stroke="none"/>',
  // Repeat — loops.
  'repeat':
      '<path d="M17 2.5l3.5 3.5L17 9.5" fill="none"/><path d="M20.5 6H8a4.5 4.5 0 00-4.5 4.5v1" fill="none"/><path d="M7 21.5L3.5 18 7 14.5" fill="none"/><path d="M3.5 18H16a4.5 4.5 0 004.5-4.5v-1" fill="none"/>',
  // Bolt — fast mode.
  'bolt': '<path d="M13 2.5L4.5 13.5H11l-1 8 8.5-11H12l1-8z" fill="none"/>',
  // Laptop with code — deep code mode.
  'code':
      '<path d="M8.5 8.5L5 12l3.5 3.5" fill="none"/><path d="M15.5 8.5L19 12l-3.5 3.5" fill="none"/><path d="M13.2 6l-2.4 12" fill="none"/>',
  // Brain-ish head — architect / reasoning.
  'brain':
      '<path d="M12 4a4.2 4.2 0 00-4.2 4.2c-1.9.5-3.3 2.2-3.3 4.3 0 2.4 2 4.4 4.4 4.4h.6V21" fill="none"/><path d="M12 4a4.2 4.2 0 014.2 4.2c1.9.5 3.3 2.2 3.3 4.3 0 2.4-2 4.4-4.4 4.4h-.6V21" fill="none"/>',
  // Corner down left — Enter key.
  'enter':
      '<path d="M20 5.5v6a3 3 0 01-3 3H5" fill="none"/><path d="M8.5 10.5L4.5 14.5l4 4" fill="none"/>',
  // X — close / cancel.
  'x': '<path d="M5.5 5.5l13 13" fill="none"/><path d="M18.5 5.5l-13 13" fill="none"/>',
  // Sparkles — AI features.
  'sparkles':
      '<path d="M12 3.5l1.8 4.7L18.5 10l-4.7 1.8L12 16.5l-1.8-4.7L5.5 10l4.7-1.8L12 3.5z" fill="none"/><path d="M19 15.5l.9 2.1 2.1.9-2.1.9-.9 2.1-.9-2.1-2.1-.9 2.1-.9.9-2.1z" fill="none"/>',
};

/// SVG icon widget for the OmnesAgent UI (AGENTS.md: emoji are forbidden;
/// icons are SVG only). Tint via [color]; size defaults to 14 px.
class UiIcon extends StatelessWidget {
  final String icon;
  final double size;
  final Color? color;

  const UiIcon(this.icon, {super.key, this.size = 14, this.color});

  @override
  Widget build(BuildContext context) {
    final body = kUiIconSvg[icon];
    assert(body != null, 'Unknown UiIcon: $icon');
    if (body == null) return SizedBox(width: size, height: size);
    return SvgPicture.string(
      '<svg viewBox="0 0 24 24" stroke-linecap="round" '
      'stroke-linejoin="round" stroke-width="1.8">$body</svg>',
      width: size,
      height: size,
      colorFilter: color == null
          ? null
          : ColorFilter.mode(color!, BlendMode.srcIn),
    );
  }
}
