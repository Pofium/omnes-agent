import 'dart:convert';
import 'package:http/http.dart' as http;
import '../gateway_config.dart';

/// Base HTTP transport for Gateway API clients, handling URL resolution,
/// authentication headers, UTF-8 decoding, and timeouts.
class GatewayHttpTransport {
  final http.Client client;

  GatewayHttpTransport({http.Client? client}) : client = client ?? http.Client();

  /// Headers with dynamic bearer authentication.
  Future<Map<String, String>> authHeaders({Map<String, String>? extraHeaders}) async {
    final token = await GatewayConfig.getToken();
    return {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      if (token.isNotEmpty) 'Authorization': 'Bearer $token',
      if (extraHeaders != null) ...extraHeaders,
    };
  }

  /// Safely decodes UTF-8 JSON response, avoiding Latin-1 mojibake.
  dynamic decodeBody(http.Response res) {
    if (res.bodyBytes.isEmpty) return null;
    return jsonDecode(utf8.decode(res.bodyBytes));
  }

  /// Resolves an API path or full URL against GatewayConfig.getBaseUrl().
  Uri resolveUri(String path, [Map<String, dynamic>? queryParameters]) {
    final base = GatewayConfig.getBaseUrl().replaceAll(RegExp(r'/+$'), '');
    final cleanPath = path.startsWith('/') ? path : '/$path';
    final fullUrl = '$base$cleanPath';
    final uri = Uri.parse(fullUrl);
    if (queryParameters != null && queryParameters.isNotEmpty) {
      return uri.replace(queryParameters: {
        ...uri.queryParameters,
        ...queryParameters.map((k, v) => MapEntry(k, v?.toString() ?? '')),
      });
    }
    return uri;
  }

  /// Performs an authenticated GET request.
  Future<http.Response> get(String path, {
    Map<String, dynamic>? queryParameters,
    Duration timeout = const Duration(seconds: 10),
  }) async {
    final uri = resolveUri(path, queryParameters);
    final headers = await authHeaders();
    return await client.get(uri, headers: headers).timeout(timeout);
  }

  /// Performs an authenticated POST request with JSON payload.
  Future<http.Response> post(String path, {
    dynamic body,
    Map<String, dynamic>? queryParameters,
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final uri = resolveUri(path, queryParameters);
    final headers = await authHeaders();
    final encoded = body != null ? (body is String ? body : jsonEncode(body)) : null;
    return await client.post(uri, headers: headers, body: encoded).timeout(timeout);
  }

  /// Performs an authenticated PUT request with JSON payload.
  Future<http.Response> put(String path, {
    dynamic body,
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final uri = resolveUri(path);
    final headers = await authHeaders();
    final encoded = body != null ? (body is String ? body : jsonEncode(body)) : null;
    return await client.put(uri, headers: headers, body: encoded).timeout(timeout);
  }

  /// Performs an authenticated PATCH request with JSON payload.
  Future<http.Response> patch(String path, {
    dynamic body,
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final uri = resolveUri(path);
    final headers = await authHeaders();
    final encoded = body != null ? (body is String ? body : jsonEncode(body)) : null;
    return await client.patch(uri, headers: headers, body: encoded).timeout(timeout);
  }

  /// Performs an authenticated DELETE request.
  Future<http.Response> delete(String path, {
    Duration timeout = const Duration(seconds: 10),
  }) async {
    final uri = resolveUri(path);
    final headers = await authHeaders();
    return await client.delete(uri, headers: headers).timeout(timeout);
  }

  void close() {
    client.close();
  }
}
