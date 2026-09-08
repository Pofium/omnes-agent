// ignore_for_file: avoid_web_libraries_in_flutter

import 'package:flutter/material.dart';
import 'dart:html' as html;
import 'dart:ui_web' as ui_web;

Widget buildIframeWidget(String url) {
  return _WebIframeHtmlView(url: url);
}

class _WebIframeHtmlView extends StatefulWidget {
  final String url;

  const _WebIframeHtmlView({
    required this.url,
  });

  @override
  State<_WebIframeHtmlView> createState() => _WebIframeHtmlViewState();
}

class _WebIframeHtmlViewState extends State<_WebIframeHtmlView> {
  late final String _viewType;
  html.IFrameElement? _iframeElement;

  @override
  void initState() {
    super.initState();
    _viewType = 'omnes-iframe-${DateTime.now().microsecondsSinceEpoch}';
    _iframeElement = html.IFrameElement()
      ..src = widget.url
      ..style.border = 'none'
      ..style.width = '100%'
      ..style.height = '100%'
      ..allow = 'camera; microphone; clipboard-read; clipboard-write; autoplay; fullscreen';

    ui_web.platformViewRegistry.registerViewFactory(
      _viewType,
      (int viewId) => _iframeElement!,
    );
  }

  @override
  void didUpdateWidget(covariant _WebIframeHtmlView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.url != widget.url && _iframeElement != null) {
      _iframeElement!.src = widget.url;
    }
  }

  @override
  Widget build(BuildContext context) {
    return HtmlElementView(viewType: _viewType);
  }
}
