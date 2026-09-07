// Repository for managing long-term agent memory via /api/memory.

import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import 'models/memory_entry.dart';

class MemoryRepository {
  final GatewayHttpClient _http;

  MemoryRepository({GatewayHttpClient? http}) : _http = http ?? GatewayHttpClient();

  Future<String> _resolveAgent(String? agentAlias) async {
    if (agentAlias != null && agentAlias.isNotEmpty) return agentAlias;
    return GatewayConfig.getActiveAgent();
  }

  /// Lists or recalls memory entries.
  Future<List<MemoryEntry>> listMemory({
    String? query,
    String? category,
    String? agentAlias,
  }) async {
    final alias = await _resolveAgent(agentAlias);
    final rawList = await _http.memoryList(
      agentAlias: alias,
      query: query,
      category: category,
    );
    return rawList.map((j) => MemoryEntry.fromJson(j)).toList();
  }

  /// Stores a new memory entry.
  Future<bool> storeMemory({
    required String key,
    required String content,
    String category = 'core',
    String? agentAlias,
  }) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.memoryStore(
      key: key,
      content: content,
      category: category,
      agentAlias: alias,
    );
  }

  /// Deletes a memory entry.
  Future<bool> deleteMemory(String key, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.memoryDelete(key, agentAlias: alias);
  }

  void dispose() {
    _http.dispose();
  }
}
