import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import '../../../theme/desktop_theme.dart';
import 'artifacts_viewer_controller.dart';

/// Interactive widget displaying generated artifacts (Markdown, SVG, Diff, Code).
class DiffViewerWidget extends StatelessWidget {
  final ArtifactsViewerController controller;

  const DiffViewerWidget({super.key, required this.controller});

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: controller,
      builder: (context, _) {
        final active = controller.activeArtifact;
        if (active == null) {
          return Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(FontAwesomeIcons.fileCode, size: 36, color: DesktopTheme.textMuted.withOpacity(0.5)),
                const SizedBox(height: 12),
                Text(
                  'Нет сгенерированных артефактов и diff-изменений',
                  style: TextStyle(fontSize: 13, color: DesktopTheme.textSecondary),
                ),
                const SizedBox(height: 6),
                Text(
                  'Изменения и созданные файлы появятся здесь во время работы агента',
                  style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                ),
              ],
            ),
          );
        }

        return Column(
          children: [
            // Top artifact selector strip
            Container(
              height: 40,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurface,
                border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
              ),
              child: Row(
                children: [
                  Expanded(
                    child: SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: Row(
                        children: List.generate(controller.artifacts.length, (idx) {
                          final item = controller.artifacts[idx];
                          final isSelected = (idx == controller.selectedIndex);
                          return Padding(
                            padding: const EdgeInsets.only(right: 6),
                            child: InkWell(
                              onTap: () => controller.selectArtifact(idx),
                              borderRadius: BorderRadius.circular(4),
                              child: Container(
                                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                                decoration: BoxDecoration(
                                  color: isSelected
                                      ? DesktopTheme.accentSky.withOpacity(0.15)
                                      : DesktopTheme.bgCanvas,
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(
                                    color: isSelected ? DesktopTheme.accentSky : DesktopTheme.borderSubtle,
                                  ),
                                ),
                                child: Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    Icon(
                                      _getArtifactIcon(item.type),
                                      size: 11,
                                      color: isSelected ? DesktopTheme.accentSky : DesktopTheme.textMuted,
                                    ),
                                    const SizedBox(width: 6),
                                    Text(
                                      item.name,
                                      style: TextStyle(
                                        fontSize: 11,
                                        fontFamily: 'Consolas',
                                        fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                                        color: isSelected ? DesktopTheme.accentSky : DesktopTheme.textPrimary,
                                      ),
                                    ),
                                    const SizedBox(width: 6),
                                    InkWell(
                                      onTap: () => controller.removeArtifact(idx),
                                      borderRadius: BorderRadius.circular(3),
                                      child: Padding(
                                        padding: const EdgeInsets.all(2),
                                        child: Icon(
                                          Icons.close,
                                          size: 11,
                                          color: isSelected ? DesktopTheme.textSecondary : DesktopTheme.textMuted,
                                        ),
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                            ),
                          );
                        }),
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  // Action buttons: Copy / Apply / Rollback / Clear
                  IconButton(
                    icon: const Icon(Icons.clear_all, size: 16),
                    tooltip: 'Очистить все артефакты',
                    color: DesktopTheme.textMuted,
                    onPressed: controller.clearArtifacts,
                  ),
                  IconButton(
                    icon: const Icon(Icons.copy, size: 14),
                    tooltip: 'Скопировать содержимое',
                    color: DesktopTheme.textMuted,
                    onPressed: () {
                      Clipboard.setData(ClipboardData(text: active.diff ?? active.content));
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text('Скопировано в буфер обмена'), duration: Duration(seconds: 1)),
                      );
                    },
                  ),
                  if (active.type == 'diff') ...[
                    IconButton(
                      icon: const Icon(Icons.check_circle_outline, size: 15),
                      tooltip: 'Применить изменение',
                      color: DesktopTheme.statusSuccess,
                      onPressed: () async {
                        final ok = await controller.applyDiff(active);
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text(ok ? 'Изменения успешно применены' : 'Ошибка применения изменений'),
                              duration: const Duration(seconds: 2),
                            ),
                          );
                        }
                      },
                    ),
                    IconButton(
                      icon: const Icon(Icons.undo, size: 15),
                      tooltip: 'Откатить файл',
                      color: DesktopTheme.statusError,
                      onPressed: () => controller.rollbackArtifact(active),
                    ),
                  ],
                ],
              ),
            ),

            // Content Area
            Expanded(
              child: Container(
                color: DesktopTheme.bgCanvas,
                child: _buildArtifactBody(active),
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _buildArtifactBody(AgentArtifact item) {
    if (item.type == 'diff' && item.diff != null) {
      return _buildDiffView(item.diff!);
    } else if (item.type == 'svg') {
      return _buildSvgView(item.content);
    } else {
      return _buildCodeOrMarkdownView(item.content);
    }
  }

  Widget _buildDiffView(String diffText) {
    final lines = diffText.split('\n');
    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: lines.length,
      itemBuilder: (context, i) {
        final line = lines[i];
        Color? bgColor;
        Color textColor = DesktopTheme.textPrimary;
        FontWeight weight = FontWeight.normal;

        if (line.startsWith('+') && !line.startsWith('+++')) {
          bgColor = DesktopTheme.statusSuccess.withOpacity(0.15);
          textColor = const Color(0xFF10B981);
        } else if (line.startsWith('-') && !line.startsWith('---')) {
          bgColor = DesktopTheme.statusError.withOpacity(0.15);
          textColor = const Color(0xFFEF4444);
        } else if (line.startsWith('@@')) {
          bgColor = DesktopTheme.accentSky.withOpacity(0.12);
          textColor = DesktopTheme.accentSky;
          weight = FontWeight.bold;
        }

        return Container(
          color: bgColor,
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              SizedBox(
                width: 32,
                child: Text(
                  '${i + 1}',
                  style: TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.textMuted),
                ),
              ),
              Expanded(
                child: SelectableText(
                  line,
                  style: TextStyle(
                    fontSize: 11,
                    fontFamily: 'Consolas',
                    color: textColor,
                    fontWeight: weight,
                  ),
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildSvgView(String svgString) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurfaceElevated,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: DesktopTheme.borderSubtle),
          ),
          child: SvgPicture.string(
            svgString,
            fit: BoxFit.contain,
          ),
        ),
      ),
    );
  }

  Widget _buildCodeOrMarkdownView(String text) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(12),
      child: SelectableText(
        text,
        style: TextStyle(
          fontSize: 12,
          height: 1.5,
          fontFamily: 'Consolas',
          color: DesktopTheme.textPrimary,
        ),
      ),
    );
  }

  IconData _getArtifactIcon(String type) {
    switch (type) {
      case 'diff':
        return Icons.difference_outlined;
      case 'svg':
        return Icons.image_outlined;
      case 'markdown':
        return Icons.description_outlined;
      default:
        return Icons.code;
    }
  }
}
