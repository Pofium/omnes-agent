// REST client for the editor API /api/v1/editor/*.
// BACKEND_SPEC.md §3.2, FRONTEND_SPEC.md §3.4 (api_editor.dart).
// F0 endpoints: open/read file into a gateway buffer, atomic write,
// buffer registry listing/close; raw bytes for non-text previews (images).

import 'dart:convert';
import 'dart:typed_data';

import 'package:http/http.dart' as http;

import 'gateway_config.dart';

/// Result of opening a file in a gateway buffer.
class EditorOpenResult {
  final int bufferId;
  final String path;
  final int rev;
  final String content;
  final String language;
  final String eol;
  final String encoding;
  final bool readOnly;
  final bool dirty;
  final int numLines;
  final int sizeBytes;

  const EditorOpenResult({
    required this.bufferId,
    required this.path,
    required this.rev,
    required this.content,
    required this.language,
    required this.eol,
    required this.encoding,
    required this.readOnly,
    required this.dirty,
    required this.numLines,
    required this.sizeBytes,
  });

  factory EditorOpenResult.fromJson(Map<String, dynamic> json) {
    return EditorOpenResult(
      bufferId: (json['buffer_id'] as num?)?.toInt() ?? 0,
      path: (json['path'] as String?) ?? '',
      rev: (json['rev'] as num?)?.toInt() ?? 0,
      content: (json['content'] as String?) ?? '',
      language: (json['language'] as String?) ?? 'plaintext',
      eol: (json['eol'] as String?) ?? 'lf',
      encoding: (json['encoding'] as String?) ?? 'utf-8',
      readOnly: (json['read_only'] as bool?) ?? false,
      dirty: (json['dirty'] as bool?) ?? false,
      numLines: (json['num_lines'] as num?)?.toInt() ?? 0,
      sizeBytes: (json['size_bytes'] as num?)?.toInt() ?? 0,
    );
  }
}

/// Error surfaced by the editor REST API (status + gateway message).
class EditorApiException implements Exception {
  final int statusCode;
  final String message;

  const EditorApiException(this.statusCode, this.message);

  bool get isConflict => statusCode == 409;

  @override
  String toString() => 'EditorApiException($statusCode): $message';
}

class EditorApi {
  final http.Client _client;

  EditorApi({http.Client? client}) : _client = client ?? http.Client();

  Future<Map<String, String>> _authHeaders() async {
    final token = await GatewayConfig.getToken();
    return {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      if (token.isNotEmpty) 'Authorization': 'Bearer $token',
    };
  }

  Uri _uri(String path, Map<String, String> query) {
    final base = GatewayConfig.getBaseUrl();
    return Uri.parse('$base$path').replace(queryParameters: query);
  }

  EditorApiException _fromResponse(http.Response res) {
    String message = 'HTTP ${res.statusCode}';
    try {
      final body = jsonDecode(utf8.decode(res.bodyBytes));
      if (body is Map && body['error'] is String) message = body['error'] as String;
    } catch (_) {}
    return EditorApiException(res.statusCode, message);
  }

  /// `GET /api/v1/editor/file` — opens (or reuses) the gateway buffer and
  /// returns its metadata + content. `root` registers the project root on
  /// first use (BACKEND_SPEC §9.9, invariant 5).
  Future<EditorOpenResult> openFile({
    required String path,
    String? root,
  }) async {
    final res = await _client.get(
      _uri('/api/v1/editor/file', {
        'path': path,
        if (root != null && root.isNotEmpty) 'root': root,
      }),
      headers: await _authHeaders(),
    );
    if (res.statusCode != 200) throw _fromResponse(res);
    return EditorOpenResult.fromJson(
      jsonDecode(utf8.decode(res.bodyBytes)) as Map<String, dynamic>,
    );
  }

  /// `PUT /api/v1/editor/file` — atomic write of [content]. Throws
  /// [EditorApiException] with `isConflict` when a dirty open buffer would be
  /// clobbered (молчаливая потеря правок запрещена).
  Future<void> writeFile({
    required String path,
    required String content,
    String? root,
    bool createParents = false,
    bool force = false,
  }) async {
    final res = await _client.put(
      _uri('/api/v1/editor/file', {
        'path': path,
        if (root != null && root.isNotEmpty) 'root': root,
        if (force) 'force': 'true',
      }),
      headers: await _authHeaders(),
      body: jsonEncode({
        'content': content,
        'create_parents': createParents,
      }),
    );
    if (res.statusCode != 200) throw _fromResponse(res);
  }

  /// `GET /api/v1/editor/raw` — raw bytes of a project file (image previews
  /// and other non-text assets served straight from the gateway).
  Future<Uint8List> fetchRaw({
    required String path,
    String? root,
  }) async {
    final res = await _client.get(
      _uri('/api/v1/editor/raw', {
        'path': path,
        if (root != null && root.isNotEmpty) 'root': root,
      }),
      headers: await _authHeaders(),
    );
    if (res.statusCode != 200) throw _fromResponse(res);
    return res.bodyBytes;
  }

  /// `DELETE /api/v1/editor/buffer` — close a buffer.
  Future<void> closeBuffer({
    int? bufferId,
    String? path,
    bool force = false,
  }) async {
    final res = await _client.delete(
      _uri('/api/v1/editor/buffer', {
        if (bufferId != null) 'buffer_id': '$bufferId',
        if (path != null) 'path': path,
        if (force) 'force': 'true',
      }),
      headers: await _authHeaders(),
    );
    if (res.statusCode != 200) throw _fromResponse(res);
  }

  void dispose() {
    _client.close();
  }
}
