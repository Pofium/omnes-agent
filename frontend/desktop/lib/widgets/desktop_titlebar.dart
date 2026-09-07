// Desktop TitleBar with Daemon Status, Active Model Selector, and Command Palette Trigger.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import '../theme/desktop_theme.dart';

class DesktopTitleBar extends StatelessWidget {
  final VoidCallback onOpenCommandPalette;
  final VoidCallback onToggleInspector;
  final bool isInspectorOpen;

  const DesktopTitleBar({
    super.key,
    required this.onOpenCommandPalette,
    required this.onToggleInspector,
    required this.isInspectorOpen,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 46,
      padding: const EdgeInsets.symmetric(horizontal: 14),
      decoration: const BoxDecoration(
        color: DesktopTheme.bgSidebar,
        border: Border(
          bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Row(
        children: [
          // 1. Logo and Brand Name
          Container(
            width: 26,
            height: 26,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(6),
              boxShadow: [
                BoxShadow(
                  color: const Color(0xFF00D2FF).withOpacity(0.35),
                  blurRadius: 10,
                  offset: const Offset(0, 2),
                ),
              ],
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: Image.asset(
                'assets/Logo/app_launcher.png',
                fit: BoxFit.cover,
              ),
            ),
          ),
          const SizedBox(width: 10),
          const Text(
            'OmnesAgent',
            style: TextStyle(
              color: DesktopTheme.textPrimary,
              fontWeight: FontWeight.bold,
              fontSize: 14,
              letterSpacing: 0.5,
            ),
          ),
          const SizedBox(width: 6),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: DesktopTheme.accentSky.withOpacity(0.12),
              borderRadius: BorderRadius.circular(4),
              border: Border.all(
                color: DesktopTheme.accentSky.withOpacity(0.3),
                width: 0.8,
              ),
            ),
            child: const Text(
              'DESKTOP',
              style: TextStyle(
                color: DesktopTheme.accentSky,
                fontSize: 10,
                fontWeight: FontWeight.w700,
                letterSpacing: 0.6,
              ),
            ),
          ),
          const SizedBox(width: 16),

          // 2. Workspace Breadcrumb
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              borderRadius: BorderRadius.circular(4),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.folder_outlined, size: 13, color: DesktopTheme.textMuted),
                const SizedBox(width: 6),
                const Text(
                  'workspace/chief',
                  style: TextStyle(
                    color: DesktopTheme.textSecondary,
                    fontSize: 12,
                    fontFamily: 'Consolas',
                  ),
                ),
              ],
            ),
          ),

          const Spacer(),

          // 3. Command Palette Trigger Button (Ctrl + K)
          InkWell(
            onTap: onOpenCommandPalette,
            borderRadius: BorderRadius.circular(6),
            child: Container(
              height: 30,
              width: 280,
              padding: const EdgeInsets.symmetric(horizontal: 10),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: DesktopTheme.borderSubtle),
              ),
              child: Row(
                children: [
                  const Icon(Icons.search, size: 15, color: DesktopTheme.textMuted),
                  const SizedBox(width: 8),
                  const Expanded(
                    child: Text(
                      'Search tasks, tools, models...',
                      style: TextStyle(
                        color: DesktopTheme.textMuted,
                        fontSize: 12,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                    decoration: BoxDecoration(
                      color: Colors.black26,
                      borderRadius: BorderRadius.circular(3),
                      border: Border.all(color: Colors.white12),
                    ),
                    child: const Text(
                      'Ctrl K',
                      style: TextStyle(
                        fontSize: 10,
                        color: DesktopTheme.textSecondary,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),

          const Spacer(),

          // 4. Active Model Chip
          _buildModelChip(context),
          const SizedBox(width: 12),

          // 5. Gateway Daemon Status
          _buildGatewayStatusBadge(),
          const SizedBox(width: 10),

          // 6. Inspector Toggle Button
          IconButton(
            tooltip: isInspectorOpen ? 'Hide Inspector' : 'Show Inspector',
            icon: Icon(
              Icons.view_sidebar_outlined,
              size: 18,
              color: isInspectorOpen
                  ? DesktopTheme.accentSky
                  : DesktopTheme.textMuted,
            ),
            onPressed: onToggleInspector,
          ),
        ],
      ),
    );
  }

  Widget _buildModelChip(BuildContext context) {
    return Container(
      height: 28,
      padding: const EdgeInsets.symmetric(horizontal: 10),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(
            FontAwesomeIcons.brain,
            size: 12,
            color: DesktopTheme.accentSky,
          ),
          const SizedBox(width: 6),
          const Text(
            'Claude 3.5 Sonnet',
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w600,
              color: DesktopTheme.textPrimary,
            ),
          ),
          const SizedBox(width: 4),
          Icon(Icons.arrow_drop_down, size: 14, color: DesktopTheme.textMuted),
        ],
      ),
    );
  }

  Widget _buildGatewayStatusBadge() {
    return Container(
      height: 28,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 7,
            height: 7,
            decoration: const BoxDecoration(
              shape: BoxShape.circle,
              color: DesktopTheme.statusSuccess,
            ),
          ),
          const SizedBox(width: 6),
          const Text(
            '127.0.0.1:42617',
            style: TextStyle(
              fontSize: 11,
              fontFamily: 'Consolas',
              color: DesktopTheme.textSecondary,
            ),
          ),
          const SizedBox(width: 6),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
            decoration: BoxDecoration(
              color: DesktopTheme.statusSuccess.withOpacity(0.12),
              borderRadius: BorderRadius.circular(3),
            ),
            child: const Text(
              '3ms',
              style: TextStyle(
                fontSize: 10,
                color: DesktopTheme.statusSuccess,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
