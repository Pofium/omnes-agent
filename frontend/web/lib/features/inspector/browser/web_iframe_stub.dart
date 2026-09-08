import 'package:flutter/material.dart';

Widget buildIframeWidget(String url) {
  return Center(
    child: Text(
      'Web Browser IFrame Standby ($url)',
      style: const TextStyle(color: Colors.grey),
    ),
  );
}
