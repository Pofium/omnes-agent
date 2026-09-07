import 'package:flutter/material.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';

class WebviewWidget extends StatelessWidget {
  const WebviewWidget({super.key, required this.mainUrl, required this.appBarTitle});
  final String mainUrl;
  final String appBarTitle;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(
            appBarTitle,
          style: TextStyle(
            color: Theme.of(context).primaryColor
          ),
        ),
        leading: GestureDetector(
          child: Icon(
            Icons.arrow_back_ios,
            color: Theme.of(context).primaryColor,
          ),
          onTap: () => Navigator.pop(context),
        ),
      ),
      body: InAppWebView(
        initialUrlRequest: URLRequest(url: WebUri(mainUrl)), // Replace with your URL
        onWebViewCreated: (InAppWebViewController controller) {
          // You can use the controller to interact with the WebView
        },
        initialOptions: InAppWebViewGroupOptions(
          android: AndroidInAppWebViewOptions(
            useWideViewPort: true,
          ),
          ios: IOSInAppWebViewOptions(
            allowsInlineMediaPlayback: true,
          ),
        ),
        onProgressChanged: (InAppWebViewController controller, int progress) {
          // Track progress changes if needed
        },
      ),
    );
  }
}
