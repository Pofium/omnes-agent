import 'gateway_http_transport.dart';

/// Domain API client for Gateway system health, diagnostics, logs, and device pairing.
class SystemApiClient {
  final GatewayHttpTransport transport;

  SystemApiClient({GatewayHttpTransport? transport})
      : transport = transport ?? GatewayHttpTransport();

  /// Checks whether gateway is reachable and responsive.
  Future<bool> checkHealth() async {
    try {
      final res = await transport.get('/health', timeout: const Duration(seconds: 5));
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        return data is Map && data['status'] == 'ok';
      }
      return false;
    } catch (_) {
      return false;
    }
  }

  /// Fetches system status from /api/status.
  Future<Map<String, dynamic>?> getStatus() async {
    try {
      final res = await transport.get('/api/status');
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches configured LLM providers and available models from /api/models.
  Future<Map<String, dynamic>?> getModelProviders() async {
    try {
      final res = await transport.get('/api/models');
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Runs diagnostic checks on the gateway and components from /api/doctor.
  Future<List<Map<String, dynamic>>> runDoctor() async {
    try {
      final res = await transport.get('/api/doctor', timeout: const Duration(seconds: 15));
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['results'] is List) {
          return (data['results'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches server logs from /api/logs.
  Future<List<Map<String, dynamic>>> getLogs({String? level, int limit = 100}) async {
    try {
      final queryParams = <String, dynamic>{
        if (level != null && level.isNotEmpty && level != 'all') 'level': level,
        'limit': limit.toString(),
      };
      final res = await transport.get('/api/logs', queryParameters: queryParams);
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['logs'] is List) {
          return (data['logs'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Pairs this device with the gateway using a pairing code.
  Future<String?> pairDevice(String code) async {
    try {
      final res = await transport.post('/api/pair', body: {'code': code});
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        if (data is Map && data['token'] != null) {
          return data['token'].toString();
        }
      }
    } catch (_) {}
    return null;
  }

  /// Triggers full AST project reindexing on the gateway.
  Future<bool> reindexAst({String? projectPath}) async {
    try {
      final res = await transport.post(
        '/api/ast/reindex',
        body: {
          if (projectPath != null) 'project_path': projectPath,
        },
      );
      return res.statusCode == 200 || res.statusCode == 202;
    } catch (_) {
      return false;
    }
  }

  /// Fetches S1 fast-inference model download/runtime status.
  Future<Map<String, dynamic>?> getS1Status() async {
    try {
      final res = await transport.get('/api/s1/status');
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Initiates downloading of S1 model weights.
  Future<bool> downloadS1Model() async {
    try {
      final res = await transport.post('/api/s1/download');
      return res.statusCode == 200 || res.statusCode == 202;
    } catch (_) {
      return false;
    }
  }
}
