import 'gateway_http_transport.dart';

/// Domain API client for Gateway tools and skills bundles (/api/tools, /api/skills/*).
class ToolsApiClient {
  final GatewayHttpTransport transport;

  ToolsApiClient({GatewayHttpTransport? transport})
      : transport = transport ?? GatewayHttpTransport();

  /// Fetches available tools from /api/tools.
  Future<List<Map<String, dynamic>>> getTools({String? agentAlias}) async {
    try {
      final queryParams = <String, dynamic>{
        if (agentAlias != null && agentAlias.isNotEmpty) 'agent': agentAlias,
      };
      final res = await transport.get('/api/tools', queryParameters: queryParams);
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['tools'] is List) {
          return (data['tools'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches skills bundles from /api/skills/bundles.
  Future<List<Map<String, dynamic>>> getSkillsBundles() async {
    try {
      final res = await transport.get('/api/skills/bundles');
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        if (data is Map && data['bundles'] is List) {
          return (data['bundles'] as List).whereType<Map<String, dynamic>>().toList();
        } else if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }
}
