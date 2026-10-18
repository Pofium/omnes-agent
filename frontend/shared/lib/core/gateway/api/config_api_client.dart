import 'gateway_http_transport.dart';

/// Domain API client for Gateway configuration operations (/api/config/*).
class ConfigApiClient {
  final GatewayHttpTransport transport;

  ConfigApiClient({GatewayHttpTransport? transport})
      : transport = transport ?? GatewayHttpTransport();

  /// Fetches LLM provider catalog from GET /api/config/catalog.
  Future<Map<String, dynamic>?> getConfigCatalog() async {
    try {
      final res = await transport.get('/api/config/catalog');
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches active gateway configuration from GET /api/config.
  Future<Map<String, dynamic>?> getFullConfig() async {
    try {
      final res = await transport.get('/api/config');
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Patches active gateway configuration via PATCH /api/config.
  Future<bool> updateConfig(Map<String, dynamic> patch) async {
    try {
      final res = await transport.patch('/api/config', body: patch);
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Gets a specific config property from GET /api/config/prop?path={path}.
  Future<Map<String, dynamic>?> getConfigProp(String path) async {
    try {
      final res = await transport.get(
        '/api/config/prop',
        queryParameters: {'path': path},
      );
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Sets a specific config property via PUT /api/config/prop.
  Future<bool> setConfigProp(String path, dynamic value) async {
    try {
      final res = await transport.put(
        '/api/config/prop',
        body: {'path': path, 'value': value},
      );
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Deletes/unsets a specific config property via DELETE /api/config/prop?path={path}.
  Future<bool> deleteConfigProp(String path) async {
    try {
      final res = await transport.delete(
        '/api/config/prop?path=${Uri.encodeComponent(path)}',
      );
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Fetches map keys for a config path via GET /api/config/map-keys?path={path}.
  Future<List<String>> getConfigMapKeys(String path) async {
    try {
      final res = await transport.get(
        '/api/config/map-keys',
        queryParameters: {'path': path},
      );
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        if (data is Map && data['keys'] is List) {
          return (data['keys'] as List).map((e) => e.toString()).toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Adds a key to a map property via POST /api/config/map-key.
  Future<bool> addConfigMapKey(String path, String key, dynamic value) async {
    try {
      final res = await transport.post(
        '/api/config/map-key',
        body: {'path': path, 'key': key, 'value': value},
      );
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Fetches configured agent aliases via GET /api/config/map-keys?path=agents.
  Future<List<String>> getAgentAliases() async {
    final keys = await getConfigMapKeys('agents');
    return keys.isNotEmpty ? keys : ['chief'];
  }
}
