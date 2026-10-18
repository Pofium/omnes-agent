import 'gateway_http_transport.dart';

/// Domain API client for Gateway memory and knowledge graph operations (/api/memory/*).
class MemoryApiClient {
  final GatewayHttpTransport transport;

  MemoryApiClient({GatewayHttpTransport? transport})
      : transport = transport ?? GatewayHttpTransport();

  /// Fetches memory entries from /api/memory.
  Future<List<Map<String, dynamic>>> memoryList({
    String? agentAlias,
    String? query,
    String? category,
  }) async {
    try {
      final queryParams = <String, dynamic>{
        if (agentAlias != null && agentAlias.isNotEmpty) 'agent': agentAlias,
        if (query != null && query.isNotEmpty) 'query': query,
        if (category != null && category.isNotEmpty && category != 'all') 'category': category,
      };
      final res = await transport.get('/api/memory', queryParameters: queryParams);
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        if (data is Map && data['entries'] is List) {
          return (data['entries'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Stores a new memory entry via POST /api/memory.
  Future<bool> memoryStore({
    required String key,
    required String content,
    String category = 'core',
    String? agentAlias,
  }) async {
    try {
      final res = await transport.post(
        '/api/memory',
        body: {
          'key': key,
          'content': content,
          'category': category,
          if (agentAlias != null && agentAlias.isNotEmpty) 'agent': agentAlias,
        },
      );
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Deletes a memory entry via DELETE /api/memory/{key}.
  Future<bool> memoryDelete(String key, {String? agentAlias}) async {
    try {
      final path = '/api/memory/${Uri.encodeComponent(key)}${agentAlias != null && agentAlias.isNotEmpty ? "?agent=${Uri.encodeComponent(agentAlias)}" : ""}';
      final res = await transport.delete(path);
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }
}
