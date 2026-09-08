import 'package:flutter/material.dart';
import 'web_iframe_stub.dart'
    if (dart.library.html) 'web_iframe_web.dart';

/// Web iframe renderer for OmnesAgent Web ADE with platform-conditional implementation.
class WebIframeView extends StatelessWidget {
  final String url;

  const WebIframeView({
    super.key,
    required this.url,
  });

  @override
  Widget build(BuildContext context) {
    return buildIframeWidget(url);
  }
}
