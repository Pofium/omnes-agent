// Image viewer pane for the right inspector panel.
// Opens for image extensions instead of the code editor: bytes come from the
// gateway (`GET /api/v1/editor/raw`, FRONTEND_SPEC §3.4), rendering is native
// Flutter (InteractiveViewer: pan + wheel zoom, fit-to-window on load).

import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:omnes_shared/omnes_shared.dart';

/// Extensions routed to the image viewer by the file-open flow.
/// svg is rendered via flutter_svg (не входит во встроенные кодеки Flutter).
const Set<String> kImageViewerExtensions = {
  'png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'wbmp', 'avif', 'svg',
};

bool isImagePath(String path) {
  final dot = path.lastIndexOf('.');
  if (dot < 0 || dot == path.length - 1) return false;
  return kImageViewerExtensions
      .contains(path.substring(dot + 1).toLowerCase());
}

class ImagePane extends StatefulWidget {
  final String filePath;
  final String? projectRoot;

  const ImagePane({super.key, required this.filePath, this.projectRoot});

  @override
  State<ImagePane> createState() => _ImagePaneState();
}

class _ImagePaneState extends State<ImagePane> {
  final EditorApi _api = EditorApi();
  final TransformationController _transform = TransformationController();

  Uint8List? _bytes;
  int? _width;
  int? _height;
  bool _loading = true;
  bool _fitMode = true;
  bool _isSvg = false;
  String? _error;

  bool _extensionIs(String path, String ext) =>
      path.toLowerCase().endsWith('.$ext');

  @override
  void initState() {
    super.initState();
    _isSvg = _extensionIs(widget.filePath, 'svg');
    _load();
  }

  @override
  void didUpdateWidget(covariant ImagePane oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.filePath != widget.filePath) {
      _isSvg = _extensionIs(widget.filePath, 'svg');
      _load();
    }
  }

  @override
  void dispose() {
    _transform.dispose();
    _api.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
      _bytes = null;
      _width = null;
      _height = null;
    });
    try {
      final bytes = await _api.fetchRaw(
        path: widget.filePath,
        root: widget.projectRoot,
      );
      // SVG обходят встроенные кодеки — размеры берёт сам flutter_svg
      // из viewBox; вписывание = сброс трансформации.
      if (_isSvg) {
        if (!mounted) return;
        setState(() {
          _bytes = bytes;
          _width = null;
          _height = null;
          _loading = false;
        });
        return;
      }
      ui.Codec codec;
      try {
        codec = await ui.instantiateImageCodec(bytes);
      } catch (_) {
        // Gateway served the bytes, but the format is not decodable by
        // Flutter codecs (e.g. ico variants) — show a graceful error.
        if (!mounted) return;
        setState(() {
          _bytes = bytes;
          _loading = false;
          _error = 'Формат не поддерживается встроенным просмотрщиком';
        });
        return;
      }
      final frame = await codec.getNextFrame();
      if (!mounted) return;
      final image = frame.image;
      setState(() {
        _bytes = bytes;
        _width = image.width;
        _height = image.height;
        _loading = false;
      });
      WidgetsBinding.instance.addPostFrameCallback((_) => _applyFit());
    } on EditorApiException catch (e) {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _error = e.message;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _error = '$e';
      });
    }
  }

  void _applyFit() {
    final box = context.size;
    final w = _width, h = _height;
    if (box == null || w == null || h == null || w == 0 || h == 0) return;
    final scale = math.min(box.width / w, box.height / h);
    _transform.value = Matrix4.identity()..scale(scale.clamp(0.01, 32.0));
  }

  void _onWheel(PointerScrollEvent event) {
    final current = _transform.value.getMaxScaleOnAxis();
    final factor = event.scrollDelta.dy > 0 ? 1 / 1.12 : 1.12;
    final next = (current * factor).clamp(0.01, 32.0);
    setState(() {
      _fitMode = false;
      _transform.value = Matrix4.identity()..scale(next);
    });
  }

  void _toggleFit() {
    if (_fitMode) {
      // 1:1 pixels.
      setState(() {
        _fitMode = false;
        _transform.value = Matrix4.identity()..scale(1.0);
      });
    } else {
      setState(() => _fitMode = true);
      _applyFit();
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final name = widget.filePath.split(RegExp(r'[\\/]')).last;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Header
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: theme.colorScheme.surface,
            border: Border(
              bottom: BorderSide(
                  color: theme.dividerColor.withOpacity(0.6), width: 0.8),
            ),
          ),
          child: Row(
            children: [
              Icon(Icons.image_outlined, size: 14, color: theme.colorScheme.primary),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  widget.filePath,
                  style: TextStyle(
                    fontSize: 11,
                    fontFamily: 'Consolas',
                    color: theme.colorScheme.onSurface.withOpacity(0.75),
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (_width != null && _height != null)
                Padding(
                  padding: const EdgeInsets.only(right: 10),
                  child: Text(
                    '$_width × $_height',
                    style: TextStyle(fontSize: 10, color: theme.colorScheme.outline),
                  ),
                ),
              if (_bytes != null)
                Padding(
                  padding: const EdgeInsets.only(right: 10),
                  child: Text(
                    '${(_bytes!.lengthInBytes / 1024).toStringAsFixed(1)} КБ',
                    style: TextStyle(fontSize: 10, color: theme.colorScheme.outline),
                  ),
                ),
              _HeaderAction(
                label: _fitMode ? '1:1' : 'Вписать',
                // Для SVG «1:1» бессмыслен (нет пиксельных размеров).
                onTap: (_bytes == null || _isSvg) ? null : _toggleFit,
              ),
            ],
          ),
        ),
        // Viewer
        Expanded(
          child: Container(
            color: const Color(0xFF0B1120),
            child: _loading
                ? Center(
                    child: SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(strokeWidth: 2, color: theme.colorScheme.primary),
                    ),
                  )
                : _bytes == null || _error != null
                    ? Center(
                        child: Padding(
                          padding: const EdgeInsets.all(24),
                          child: Text(
                            _error ?? 'Не удалось загрузить файл',
                            textAlign: TextAlign.center,
                            style: TextStyle(color: theme.colorScheme.outline, fontSize: 12),
                          ),
                        ),
                      )
                    : GestureDetector(
                        onDoubleTap: () {
                          if (_isSvg) {
                            // «Вписать» для SVG = сброс трансформации.
                            setState(() => _transform.value = Matrix4.identity());
                          } else {
                            _toggleFit();
                          }
                        },
                        child: Listener(
                          onPointerSignal: (event) {
                            if (event is PointerScrollEvent) _onWheel(event);
                          },
                          child: InteractiveViewer(
                            transformationController: _transform,
                            maxScale: 32,
                            minScale: 0.01,
                            panEnabled: true,
                            scaleEnabled: true,
                            alignment: Alignment.center,
                            child: _isSvg
                                ? FittedBox(
                                    fit: BoxFit.contain,
                                    child: SvgPicture.memory(_bytes!),
                                  )
                                : Image.memory(
                                    _bytes!,
                                    fit: BoxFit.contain,
                                    filterQuality: FilterQuality.medium,
                                    gaplessPlayback: true,
                                  ),
                          ),
                        ),
                      ),
          ),
        ),
        // Footer
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 5),
          decoration: BoxDecoration(
            color: theme.colorScheme.surface,
            border: Border(
              top: BorderSide(color: theme.dividerColor.withOpacity(0.6), width: 0.8),
            ),
          ),
          child: Row(
            children: [
              Text(name, style: const TextStyle(fontSize: 10)),
              const SizedBox(width: 14),
              Text(
                'через шлюз • колесо — зум, двойной клик — вписать',
                style: TextStyle(fontSize: 10, color: theme.colorScheme.outline),
              ),
              const Spacer(),
              Text(widget.filePath, style: TextStyle(fontSize: 10, color: theme.colorScheme.outline)),
            ],
          ),
        ),
      ],
    );
  }
}

class _HeaderAction extends StatelessWidget {
  final String label;
  final VoidCallback? onTap;

  const _HeaderAction({required this.label, this.onTap});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        decoration: BoxDecoration(
          border: Border.all(color: theme.colorScheme.outline.withOpacity(0.35), width: 0.8),
          borderRadius: BorderRadius.circular(4),
        ),
        child: Text(
          label,
          style: TextStyle(
            fontSize: 10,
            fontWeight: FontWeight.w600,
            color: onTap == null
                ? theme.colorScheme.outline.withOpacity(0.5)
                : theme.colorScheme.primary,
          ),
        ),
      ),
    );
  }
}
