// HTTP REST client for OmnesAgent Gateway endpoints.
import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../model/workspace_entry.dart';
import '../../model/session_info.dart';
import 'gateway_config.dart';

class GatewayHttpClient {
  final http.Client _client;

  GatewayHttpClient({http.Client? client}) : _client = client ?? http.Client();

  /// Headers with bearer authentication.
  Future<Map<String, String>> _authHeaders() async {
    final token = await GatewayConfig.getToken();
    return {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      if (token.isNotEmpty) 'Authorization': 'Bearer $token',
    };
  }

  /// Checks whether gateway is reachable and responsive.
  /// Safely decodes UTF-8 JSON response, avoiding Latin-1 mojibake.
  dynamic _decodeBody(http.Response res) {
    return jsonDecode(utf8.decode(res.bodyBytes));
  }

  Future<bool> checkHealth() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/health');
      final res = await _client.get(uri).timeout(const Duration(seconds: 5));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        return data['status'] == 'ok';
      }
      return false;
    } catch (_) {
      return false;
    }
  }

  /// Fetches system status from /api/status.
  Future<Map<String, dynamic>?> getStatus() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/status');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches list of configured agent aliases from /api/config/map-keys?path=agents.
  Future<List<String>> getAgentAliases() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/map-keys?path=agents');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['keys'] is List) {
          final list = (data['keys'] as List).map((e) => e.toString()).toList();
          if (list.isNotEmpty) return list;
        }
      }
    } catch (_) {}
    return ['chief'];
  }

  /// Fetches raw sessions list from /api/sessions.
  Future<List<SessionInfo>> getSessionsList() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sessions');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        List? rawList;
        if (data is List) {
          rawList = data;
        } else if (data is Map && data['sessions'] is List) {
          rawList = data['sessions'] as List;
        }
        if (rawList != null) {
          final sessions = rawList
              .whereType<Map<String, dynamic>>()
              .map((item) => SessionInfo.fromJson(item))
              .toList();
          // Sort newest activity first
          sessions.sort((a, b) => b.lastActivity.compareTo(a.lastActivity));
          return sessions;
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches message history for a specific session from /api/sessions/{id}/messages.
  Future<List<SessionHistoryMessage>> getSessionMessages(String sessionId) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sessions/$sessionId/messages');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['messages'] is List) {
          final list = (data['messages'] as List)
              .whereType<Map<String, dynamic>>()
              .map((m) => SessionHistoryMessage.fromJson(m))
              .toList();
          return list;
        }
      }
    } catch (_) {}
    return [];
  }

  /// Deletes a session from /api/sessions/{id}.
  Future<bool> deleteSession(String sessionId) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sessions/$sessionId');
      final headers = await _authHeaders();
      final res = await _client.delete(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Aborts an active session turn via /api/sessions/{id}/abort.
  Future<bool> abortSession(String sessionId) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sessions/$sessionId/abort');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 5));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Renames a session via PUT /api/sessions/{id}.
  Future<bool> sessionRename(String sessionId, String name) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sessions/$sessionId');
      final headers = await _authHeaders();
      final body = jsonEncode({'name': name.trim()});
      final res = await _client.put(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Fetches session state from GET /api/sessions/{id}/state.
  Future<Map<String, dynamic>?> getSessionState(String sessionId) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sessions/$sessionId/state');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Lists actively running sessions from GET /api/sessions/running.
  Future<List<Map<String, dynamic>>> getSessionsRunning() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sessions/running');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['sessions'] is List) {
          return (data['sessions'] as List).whereType<Map<String, dynamic>>().toList();
        } else if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Pushes a visible message into a session transcript via POST /api/sessions/{id}/messages.
  Future<bool> postSessionMessage(String sessionId, String content) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sessions/$sessionId/messages');
      final headers = await _authHeaders();
      final body = jsonEncode({'content': content});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Fetches personality files for agent from /api/personality?agent={alias}.
  Future<Map<String, dynamic>?> getAgentPersonality(String agentAlias) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/personality?agent=${Uri.encodeComponent(agentAlias)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Lists workspace entries for an agent at a given relative path.
  Future<List<WorkspaceEntry>> workspaceList(String agentAlias, {String path = ''}) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final cleanPath = path.trim().replaceAll(RegExp(r'^/+|/+$'), '');
      final queryParam = cleanPath.isNotEmpty ? '?path=${Uri.encodeComponent(cleanPath)}' : '';
      final uri = Uri.parse('$base/api/agents/${Uri.encodeComponent(agentAlias)}/workspace/list$queryParam');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['entries'] is List) {
          final parentPath = (data['path'] ?? cleanPath).toString();
          return (data['entries'] as List)
              .whereType<Map<String, dynamic>>()
              .map((e) => WorkspaceEntry.fromJson(e, parentPath: parentPath))
              .toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Reads a file from an agent's workspace.
  Future<WorkspaceFileContent?> workspaceReadFile(String agentAlias, String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final cleanPath = path.trim().replaceAll(RegExp(r'^/+|/+$'), '');
      final uri = Uri.parse('$base/api/agents/${Uri.encodeComponent(agentAlias)}/workspace/read?path=${Uri.encodeComponent(cleanPath)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map<String, dynamic>) {
          return WorkspaceFileContent.fromJson(data);
        }
      }
    } catch (_) {}
    return null;
  }

  /// Downloads raw file bytes (binary or text) from an agent's workspace via /raw.
  Future<List<int>?> workspaceRawFileBytes(String agentAlias, String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final cleanPath = path.trim().replaceAll(RegExp(r'^/+|/+$'), '');
      final uri = Uri.parse('$base/api/agents/${Uri.encodeComponent(agentAlias)}/workspace/raw?path=${Uri.encodeComponent(cleanPath)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 15));
      if (res.statusCode == 200) {
        return res.bodyBytes;
      }
    } catch (_) {}
    return null;
  }

  /// Writes text content to a file in an agent's workspace.
  Future<bool> workspaceWriteFile(String agentAlias, String path, String content) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final cleanPath = path.trim().replaceAll(RegExp(r'^/+|/+$'), '');
      final uri = Uri.parse('$base/api/agents/${Uri.encodeComponent(agentAlias)}/workspace/write');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': cleanPath, 'content': content});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Creates a directory in an agent's workspace.
  Future<bool> workspaceMkdir(String agentAlias, String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final cleanPath = path.trim().replaceAll(RegExp(r'^/+|/+$'), '');
      final uri = Uri.parse('$base/api/agents/${Uri.encodeComponent(agentAlias)}/workspace/mkdir');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': cleanPath});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Moves or renames a path inside an agent's workspace.
  Future<bool> workspaceMove(String agentAlias, String from, String to) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final cleanFrom = from.trim().replaceAll(RegExp(r'^/+|/+$'), '');
      final cleanTo = to.trim().replaceAll(RegExp(r'^/+|/+$'), '');
      final uri = Uri.parse('$base/api/agents/${Uri.encodeComponent(agentAlias)}/workspace/move');
      final headers = await _authHeaders();
      final body = jsonEncode({'from': cleanFrom, 'to': cleanTo});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Deletes a path (file or directory) in an agent's workspace.
  Future<bool> workspaceDelete(String agentAlias, String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final cleanPath = path.trim().replaceAll(RegExp(r'^/+|/+$'), '');
      final uri = Uri.parse('$base/api/agents/${Uri.encodeComponent(agentAlias)}/workspace/path');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': cleanPath});
      final req = http.Request('DELETE', uri)..headers.addAll(headers)..body = body;
      final streamed = await _client.send(req).timeout(const Duration(seconds: 10));
      return streamed.statusCode == 200 || streamed.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Uploads binary/image bytes to an agent's workspace via POST /api/upload?agent=<alias>.
  Future<String?> uploadFile(String agentAlias, List<int> bytes) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/upload?agent=${Uri.encodeComponent(agentAlias)}');
      final token = await GatewayConfig.getToken();
      final req = http.Request('POST', uri)
        ..headers.addAll({
          if (token.isNotEmpty) 'Authorization': 'Bearer $token',
        })
        ..bodyBytes = bytes;
      final streamed = await _client.send(req).timeout(const Duration(seconds: 30));
      final res = await http.Response.fromStream(streamed);
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map) {
          return data['path']?.toString() ?? data['marker']?.toString();
        }
      }
    } catch (_) {}
    return null;
  }

  // ── Memory API (/api/memory) ─────────────────────────────────────────────

  /// Fetches memory entries from /api/memory.
  Future<List<Map<String, dynamic>>> memoryList({
    String? agentAlias,
    String? query,
    String? category,
  }) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final queryParams = <String, String>{
        if (agentAlias != null && agentAlias.isNotEmpty) 'agent': agentAlias,
        if (query != null && query.isNotEmpty) 'query': query,
        if (category != null && category.isNotEmpty && category != 'all') 'category': category,
      };
      final qs = queryParams.entries
          .map((e) => '${Uri.encodeComponent(e.key)}=${Uri.encodeComponent(e.value)}')
          .join('&');
      final uri = Uri.parse('$base/api/memory${qs.isNotEmpty ? "?$qs" : ""}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
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
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/memory');
      final headers = await _authHeaders();
      final body = jsonEncode({
        'key': key,
        'content': content,
        'category': category,
        if (agentAlias != null && agentAlias.isNotEmpty) 'agent': agentAlias,
      });
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Deletes a memory entry via DELETE /api/memory/{key}.
  Future<bool> memoryDelete(String key, {String? agentAlias}) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final agentParam = (agentAlias != null && agentAlias.isNotEmpty)
          ? '?agent=${Uri.encodeComponent(agentAlias)}'
          : '';
      final uri = Uri.parse('$base/api/memory/${Uri.encodeComponent(key)}$agentParam');
      final headers = await _authHeaders();
      final res = await _client.delete(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  // ── Cron API (/api/cron) ──────────────────────────────────────────────────

  /// Lists all configured cron jobs from GET /api/cron.
  Future<List<Map<String, dynamic>>> cronList() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cron');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['jobs'] is List) {
          return (data['jobs'] as List).whereType<Map<String, dynamic>>().toList();
        } else if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Adds a new cron job via POST /api/cron.
  Future<bool> cronAdd(Map<String, dynamic> body) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cron');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(body)).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Updates or toggles a cron job via PATCH /api/cron/{id}.
  Future<bool> cronPatch(String id, Map<String, dynamic> patch) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cron/${Uri.encodeComponent(id)}');
      final headers = await _authHeaders();
      final res = await _client.patch(uri, headers: headers, body: jsonEncode(patch)).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Triggers immediate execution of a cron job via POST /api/cron/{id}/run.
  Future<bool> cronRun(String id) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cron/${Uri.encodeComponent(id)}/run');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 202;
    } catch (_) {
      return false;
    }
  }

  /// Deletes a cron job via DELETE /api/cron/{id}.
  Future<bool> cronDelete(String id) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cron/${Uri.encodeComponent(id)}');
      final headers = await _authHeaders();
      final res = await _client.delete(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Lists execution history for a cron job from GET /api/cron/{id}/runs.
  Future<List<Map<String, dynamic>>> cronRuns(String id) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cron/${Uri.encodeComponent(id)}/runs');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['runs'] is List) {
          return (data['runs'] as List).whereType<Map<String, dynamic>>().toList();
        } else if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }


  // ── Cost API (/api/cost) ─────────────────────────────────────────────────

  /// Fetches cost summary from GET /api/cost.
  Future<Map<String, dynamic>?> costSummary({
    String? agentAlias,
    String? from,
    String? to,
  }) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final queryParams = <String, String>{
        if (agentAlias != null && agentAlias.isNotEmpty) 'agent': agentAlias,
        if (from != null && from.isNotEmpty) 'from': from,
        if (to != null && to.isNotEmpty) 'to': to,
      };
      final qs = queryParams.entries
          .map((e) => '${Uri.encodeComponent(e.key)}=${Uri.encodeComponent(e.value)}')
          .join('&');
      final uri = Uri.parse('$base/api/cost${qs.isNotEmpty ? "?$qs" : ""}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['cost'] is Map) {
          return data['cost'] as Map<String, dynamic>;
        } else if (data is Map<String, dynamic>) {
          return data;
        }
      }
    } catch (_) {}
    return null;
  }

  // ── Tools API (/api/tools) ───────────────────────────────────────────────

  /// Fetches available tools from /api/tools.
  Future<List<Map<String, dynamic>>> getTools({String? agentAlias}) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final agentParam = (agentAlias != null && agentAlias.isNotEmpty)
          ? '?agent=${Uri.encodeComponent(agentAlias)}'
          : '';
      final uri = Uri.parse('$base/api/tools$agentParam');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['tools'] is List) {
          return (data['tools'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── Skills API (/api/skills/bundles) ──────────────────────────────────────

  /// Fetches skills bundles from /api/skills/bundles.
  Future<List<Map<String, dynamic>>> getSkillsBundles() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/skills/bundles');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['bundles'] is List) {
          return (data['bundles'] as List).whereType<Map<String, dynamic>>().toList();
        } else if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── Doctor / Diagnostics API (/api/doctor) ────────────────────────────────

  /// Runs diagnostic checks on the gateway and components from /api/doctor.
  Future<List<Map<String, dynamic>>> runDoctor() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/doctor');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 15));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['results'] is List) {
          return (data['results'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── Logs API (/api/logs) ──────────────────────────────────────────────────

  /// Fetches server logs from /api/logs.
  Future<List<Map<String, dynamic>>> getLogs({String? level, int limit = 100}) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final queryParams = <String, String>{
        if (level != null && level.isNotEmpty && level != 'all') 'level': level,
        'limit': limit.toString(),
      };
      final qs = queryParams.entries
          .map((e) => '${Uri.encodeComponent(e.key)}=${Uri.encodeComponent(e.value)}')
          .join('&');
      final uri = Uri.parse('$base/api/logs${qs.isNotEmpty ? "?$qs" : ""}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['entries'] is List) {
          return (data['entries'] as List).whereType<Map<String, dynamic>>().toList();
        } else if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── Channels / Integrations API (/api/channels) ───────────────────────────

  /// Fetches active channel integrations from /api/channels.
  Future<List<Map<String, dynamic>>> getChannels() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/channels');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['channels'] is List) {
          return (data['channels'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── Pairing API (/api/pair) ───────────────────────────────────────────────

  /// Pairs device via pairing code through POST /api/pair.
  Future<String?> pairDevice(String code) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/pair');
      final body = jsonEncode({
        'code': code.trim(),
        'device_name': 'Omnes Client (Mobile/Desktop)',
        'device_type': 'flutter',
      });
      final res = await _client.post(
        uri,
        headers: {'Content-Type': 'application/json'},
        body: body,
      ).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['token'] != null) {
          final token = data['token'].toString();
          await GatewayConfig.setToken(token);
          return token;
        }
      }
    } catch (_) {}
    return null;
  }

  // ── Config Catalog API (/api/config/catalog) ──────────────────────────────

  /// Fetches LLM provider catalog from GET /api/config/catalog.
  Future<Map<String, dynamic>?> getConfigCatalog() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/catalog');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches active gateway configuration from GET /api/config.
  Future<Map<String, dynamic>?> getFullConfig() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Patches active gateway configuration via PATCH /api/config.
  Future<bool> updateConfig(Map<String, dynamic> patch) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config');
      final headers = await _authHeaders();
      final res = await _client.patch(uri, headers: headers, body: jsonEncode(patch)).timeout(const Duration(seconds: 15));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Gets a specific config property from GET /api/config/prop?path={path}.
  Future<Map<String, dynamic>?> getConfigProp(String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/prop?path=${Uri.encodeComponent(path)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Sets a specific config property via PUT /api/config/prop.
  Future<bool> setConfigProp(String path, dynamic value) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/prop');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': path, 'value': value});
      final res = await _client.put(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Deletes/unsets a specific config property via DELETE /api/config/prop?path={path}.
  Future<bool> deleteConfigProp(String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/prop?path=${Uri.encodeComponent(path)}');
      final headers = await _authHeaders();
      final res = await _client.delete(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Fetches map keys for a config path via GET /api/config/map-keys?path={path}.
  Future<List<String>> getConfigMapKeys(String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/map-keys?path=${Uri.encodeComponent(path)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
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
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/map-key');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': path, 'key': key, 'value': value});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Deletes a key from a map property via DELETE /api/config/map-key.
  Future<bool> deleteConfigMapKey(String path, String key) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/map-key');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': path, 'key': key});
      final req = http.Request('DELETE', uri)..headers.addAll(headers)..body = body;
      final streamedRes = await _client.send(req).timeout(const Duration(seconds: 10));
      return streamedRes.statusCode == 200 || streamedRes.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Renames a key in a map property via POST /api/config/rename-map-key.
  Future<bool> renameConfigMapKey(String path, String oldKey, String newKey) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/rename-map-key');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': path, 'old_key': oldKey, 'new_key': newKey});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Fetches model catalog list from GET /api/config/catalog/models.
  Future<List<Map<String, dynamic>>> getConfigCatalogModels() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/catalog/models');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['models'] is List) {
          return (data['models'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches config sections overview from GET /api/config/sections.
  Future<List<Map<String, dynamic>>> getConfigSections() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/sections');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['sections'] is List) {
          return (data['sections'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches reload status from GET /api/config/reload-status.
  Future<Map<String, dynamic>?> getConfigReloadStatus() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/reload-status');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches config drift from GET /api/config/drift.
  Future<List<Map<String, dynamic>>> getConfigDrift() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/drift');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['drifted'] is List) {
          return (data['drifted'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── Live Canvas (A2UI) API (/api/canvas) ──────────────────────────────────

  /// Fetches active canvases list from GET /api/canvas.
  Future<List<Map<String, dynamic>>> getCanvasList() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/canvas');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['canvases'] is List) {
          return (data['canvases'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches a specific canvas state from GET /api/canvas/{id}.
  Future<Map<String, dynamic>?> getCanvas(String id) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/canvas/${Uri.encodeComponent(id)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Updates or pushes canvas content via POST /api/canvas/{id}.
  Future<bool> postCanvas(String id, Map<String, dynamic> canvasData) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/canvas/${Uri.encodeComponent(id)}');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(canvasData)).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Clears canvas content via DELETE /api/canvas/{id}.
  Future<bool> clearCanvas(String id) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/canvas/${Uri.encodeComponent(id)}');
      final headers = await _authHeaders();
      final res = await _client.delete(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Fetches canvas update history from GET /api/canvas/{id}/history.
  Future<List<Map<String, dynamic>>> getCanvasHistory(String id) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/canvas/${Uri.encodeComponent(id)}/history');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['history'] is List) {
          return (data['history'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── SOP Author & Graph API (/api/sops) ────────────────────────────────────

  /// Fetches SOPs list from GET /api/sops.
  Future<List<Map<String, dynamic>>> getSopsList() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['sops'] is List) {
          return (data['sops'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches SOP graph representation from GET /api/sops/{name}/graph.
  Future<Map<String, dynamic>?> getSopGraph(String name) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops/${Uri.encodeComponent(name)}/graph');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Saves SOP definition via PUT /api/sops/{name}.
  Future<bool> saveSop(String name, Map<String, dynamic> sopData) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops/${Uri.encodeComponent(name)}');
      final headers = await _authHeaders();
      final res = await _client.put(uri, headers: headers, body: jsonEncode(sopData)).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Runs an SOP via POST /api/sops/{name}/run.
  Future<Map<String, dynamic>?> runSop(String name, {Map<String, dynamic>? inputs}) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops/${Uri.encodeComponent(name)}/run');
      final headers = await _authHeaders();
      final body = jsonEncode(inputs ?? {});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 30));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches recent SOP runs from GET /api/sops/runs.
  Future<List<Map<String, dynamic>>> getSopRuns() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops/runs');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['runs'] is List) {
          return (data['runs'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches pending approval gates from GET /admin/sop/pending.
  Future<List<Map<String, dynamic>>> sopPending() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/sop/pending');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['pending'] is List) {
          return (data['pending'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Approves a pending SOP run gate via POST /admin/sop/approve.
  Future<bool> sopApprove(String runId, {String? reason}) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/sop/approve');
      final headers = await _authHeaders();
      final body = jsonEncode({'run_id': runId, if (reason != null) 'reason': reason});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 15));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Denies a pending SOP run gate via POST /admin/sop/deny.
  Future<bool> sopDeny(String runId, {String? reason}) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/sop/deny');
      final headers = await _authHeaders();
      final body = jsonEncode({'run_id': runId, if (reason != null) 'reason': reason});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 15));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Sends operator decision for an SOP run via POST /api/sops/{name}/runs/{run_id}/decide.
  Future<bool> sopDecide(String name, String runId, String decision, {String? reason}) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops/${Uri.encodeComponent(name)}/runs/${Uri.encodeComponent(runId)}/decide');
      final headers = await _authHeaders();
      final body = jsonEncode({'decision': decision, if (reason != null) 'reason': reason});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 15));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Cancels an active SOP run via POST /api/sops/{name}/runs/{run_id}/cancel.
  Future<bool> sopCancel(String name, String runId) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops/${Uri.encodeComponent(name)}/runs/${Uri.encodeComponent(runId)}/cancel');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode({})).timeout(const Duration(seconds: 15));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Generates pure SOP wire draft via POST /api/sops/wire-draft.
  Future<Map<String, dynamic>?> sopWireDraft(Map<String, dynamic> spec) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops/wire-draft');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(spec)).timeout(const Duration(seconds: 20));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Generates pure SOP graph draft via POST /api/sops/graph-draft.
  Future<Map<String, dynamic>?> sopGraphDraft(Map<String, dynamic> spec) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/sops/graph-draft');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(spec)).timeout(const Duration(seconds: 20));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }


  // ── Quickstart Wizard API (/api/quickstart) ───────────────────────────────

  /// Fetches quickstart wizard state from GET /api/quickstart/state.
  Future<Map<String, dynamic>?> getQuickstartState() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/quickstart/state');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Validates quickstart configuration via POST /api/quickstart/validate.
  Future<Map<String, dynamic>?> validateQuickstart(Map<String, dynamic> fields) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/quickstart/validate');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(fields)).timeout(const Duration(seconds: 15));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Applies quickstart configuration via POST /api/quickstart/apply.
  Future<bool> applyQuickstart(Map<String, dynamic> fields) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/quickstart/apply');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(fields)).timeout(const Duration(seconds: 20));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── Personality Editor API (/api/personality) ─────────────────────────────

  /// Fetches personality templates from GET /api/personality/templates.
  Future<List<Map<String, dynamic>>> getPersonalityTemplates() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/personality/templates');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['templates'] is List) {
          return (data['templates'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches personality file content from GET /api/personality/{filename}.
  Future<String?> getPersonalityFile(String filename) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/personality/${Uri.encodeComponent(filename)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['content'] != null) {
          return data['content'].toString();
        }
        return res.body;
      }
    } catch (_) {}
    return null;
  }

  /// Saves personality file content via PUT /api/personality/{filename}.
  Future<bool> savePersonalityFile(String filename, String content) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/personality/${Uri.encodeComponent(filename)}');
      final headers = await _authHeaders();
      final body = jsonEncode({'content': content});
      final res = await _client.put(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── Version & Update API (/api/version) ───────────────────────────────────

  /// Checks for gateway updates from GET /api/version/check.
  Future<Map<String, dynamic>?> checkVersion() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/version/check');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Triggers runtime upgrade via POST /api/version/upgrade.
  Future<bool> upgradeVersion() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/version/upgrade');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 30));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── Device Management API (/api/devices) ──────────────────────────────────

  /// Lists connected paired devices from GET /api/devices.
  Future<List<Map<String, dynamic>>> getDevices() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/devices');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['devices'] is List) {
          return (data['devices'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Revokes a device token via DELETE /api/devices/{id}.
  Future<bool> revokeDevice(String id) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/devices/${Uri.encodeComponent(id)}');
      final headers = await _authHeaders();
      final res = await _client.delete(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  // ── Channels Management API (/api/channels) ─────────────────────────────

  /// Binds or configures a communication channel via POST /api/channels/bind.
  Future<bool> channelBind(Map<String, dynamic> config) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/channels/bind');
      final headers = await _authHeaders();
      final body = jsonEncode(config);
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 15));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Relinks / reconnects a channel via POST /api/channels/{channel}/relink.
  Future<bool> channelRelink(String channel) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/channels/${Uri.encodeComponent(channel)}/relink');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 15));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── Skills CRUD API (/api/skills) ─────────────────────────────────────────

  /// Lists skills within a specific bundle via GET /api/skills/bundles/{alias}/skills.
  Future<List<Map<String, dynamic>>> listSkillsInBundle(String alias) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/skills/bundles/${Uri.encodeComponent(alias)}/skills');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['skills'] is List) {
          return (data['skills'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Creates a new skill via POST /api/skills/bundles/{alias}/skills.
  Future<bool> createSkill(String alias, String name, String bodyContent) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/skills/bundles/${Uri.encodeComponent(alias)}/skills');
      final headers = await _authHeaders();
      final body = jsonEncode({'name': name, 'body': bodyContent});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Reads a skill content via GET /api/skills/bundles/{alias}/skills/{name}.
  Future<String?> readSkill(String alias, String name) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/skills/bundles/${Uri.encodeComponent(alias)}/skills/${Uri.encodeComponent(name)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['body'] != null) {
          return data['body'].toString();
        }
        return res.body;
      }
    } catch (_) {}
    return null;
  }

  /// Updates a skill via PUT /api/skills/bundles/{alias}/skills/{name}.
  Future<bool> writeSkill(String alias, String name, String bodyContent) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/skills/bundles/${Uri.encodeComponent(alias)}/skills/${Uri.encodeComponent(name)}');
      final headers = await _authHeaders();
      final body = jsonEncode({'body': bodyContent});
      final res = await _client.put(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Deletes a skill via DELETE /api/skills/bundles/{alias}/skills/{name}.
  Future<bool> deleteSkill(String alias, String name) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/skills/bundles/${Uri.encodeComponent(alias)}/skills/${Uri.encodeComponent(name)}');
      final headers = await _authHeaders();
      final res = await _client.delete(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Retrieves slash option kinds via GET /api/skills/slash-option-kinds.
  Future<List<String>> getSlashOptionKinds() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/skills/slash-option-kinds');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.map((e) => e.toString()).toList();
        }
      }
    } catch (_) {}
    return ['goal', 'schedule', 'grill-me', 'learn'];
  }

  /// Retrieves skills associated with a specific agent via GET /api/agents/{alias}/skills.
  Future<List<Map<String, dynamic>>> getAgentSkills(String alias) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/agents/${Uri.encodeComponent(alias)}/skills');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── Host Filesystem Browser API (/api/browse) ─────────────────────────────

  /// Browses host directory entries via GET /api/browse?path=...
  Future<List<Map<String, dynamic>>> browse(String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final clean = path.trim();
      final query = clean.isNotEmpty ? '?path=${Uri.encodeComponent(clean)}' : '';
      final uri = Uri.parse('$base/api/browse$query');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['entries'] is List) {
          return (data['entries'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Creates directory on host via POST /api/browse/mkdir.
  Future<bool> browseMkdir(String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/browse/mkdir');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': path});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Removes directory on host via POST /api/browse/rmdir.
  Future<bool> browseRmdir(String path) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/browse/rmdir');
      final headers = await _authHeaders();
      final body = jsonEncode({'path': path});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── Admin Operations API (/admin/*) ───────────────────────────────────────

  /// Shuts down gateway server daemon via POST /admin/shutdown.
  Future<bool> adminShutdown() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/shutdown');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 5));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Reloads gateway server daemon configuration via POST /admin/reload.
  Future<bool> adminReload() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/reload');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Generates a new one-time pairing code via POST /admin/paircode/new.
  Future<String?> adminPaircodeNew() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/paircode/new');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['code'] != null) {
          return data['code'].toString();
        }
        return res.body.trim();
      }
    } catch (_) {}
    return null;
  }

  /// Fetches pairing code info via GET /admin/paircode.
  Future<String?> adminPaircode() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/paircode');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 5));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is Map && data['code'] != null) {
          return data['code'].toString();
        }
        return res.body.trim();
      }
    } catch (_) {}
    return null;
  }

  // ── Config Sections Wizard & Advanced Config API ──────────────────────────

  /// Fetches picker choices for a specific config section via GET /api/config/sections/{section}.
  Future<Map<String, dynamic>?> configSectionPicker(String section) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/sections/${Uri.encodeComponent(section)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Selects an item in a config section via POST /api/config/sections/{section}/items/{key}.
  Future<bool> configSectionSelect(String section, String key) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/sections/${Uri.encodeComponent(section)}/items/${Uri.encodeComponent(key)}');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Fetches agent options via GET /api/config/agent-options.
  Future<Map<String, dynamic>?> configAgentOptions() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/agent-options');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches overall config section status via GET /api/config/status.
  Future<Map<String, dynamic>?> configStatus() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/status');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Refreshes context window size for a model provider via POST /api/config/model-providers/{type}/{alias}/refresh-context-window.
  Future<bool> configRefreshContextWindow(String type, String alias) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/model-providers/${Uri.encodeComponent(type)}/${Uri.encodeComponent(alias)}/refresh-context-window');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 15));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Initializes default configuration via POST /api/config/init.
  Future<bool> configInit() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/init');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  /// Fetches delete plan simulation for config via GET /api/config/delete-plan.
  Future<Map<String, dynamic>?> configDeletePlan() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/delete-plan');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches list of config files or properties via GET /api/config/list.
  Future<List<dynamic>> configList() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/config/list');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) return data;
        if (data is Map && data['items'] is List) return data['items'] as List;
      }
    } catch (_) {}
    return [];
  }

  /// Dismisses the quickstart banner via POST /api/quickstart/dismiss.
  Future<bool> quickstartDismiss() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/quickstart/dismiss');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 5));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── Version Upgrade Status & Device Capabilities ─────────────────────────

  /// Fetches background upgrade progress via GET /api/version/upgrade/status.
  Future<Map<String, dynamic>?> versionUpgradeStatus([String? handoffId]) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final q = handoffId != null ? '?handoff_id=${Uri.encodeComponent(handoffId)}' : '';
      final uri = Uri.parse('$base/api/version/upgrade/status$q');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Rotates security token for device via POST /api/devices/{id}/token/rotate.
  Future<Map<String, dynamic>?> rotateToken(String deviceId) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/devices/${Uri.encodeComponent(deviceId)}/token/rotate');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Updates current client device capabilities via POST /api/devices/me/capabilities.
  Future<bool> updateCapabilities(Map<String, dynamic> caps) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/devices/me/capabilities');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(caps)).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── Integrations & Cron Settings ──────────────────────────────────────────

  /// Fetches integrations configuration settings via GET /api/integrations/settings.
  Future<Map<String, dynamic>?> getIntegrationsSettings() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/integrations/settings');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches global cron scheduler settings via GET /api/cron/settings.
  Future<Map<String, dynamic>?> cronSettings() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cron/settings');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Updates global cron scheduler settings via PATCH /api/cron/settings.
  Future<bool> cronSettingsPatch(Map<String, dynamic> patch) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cron/settings');
      final headers = await _authHeaders();
      final res = await _client.patch(uri, headers: headers, body: jsonEncode(patch)).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── System Utilities, CLI Tools, TUIs & Metrics ──────────────────────────

  /// Fetches list of discovered host CLI tools via GET /api/cli-tools.
  Future<List<Map<String, dynamic>>> getCliTools() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/cli-tools');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['tools'] is List) {
          return (data['tools'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches list of active TUI sessions via GET /api/tuis.
  Future<List<Map<String, dynamic>>> getTuis() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/tuis');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['sessions'] is List) {
          return (data['sessions'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches raw Prometheus metrics via GET /metrics.
  Future<String?> getMetrics() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/metrics');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return res.body;
      }
    } catch (_) {}
    return null;
  }

  /// Resolves dynamic parameter options for tool authoring via POST /api/tools/param-options.
  Future<Map<String, dynamic>?> toolsParamOptions(Map<String, dynamic> body) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/tools/param-options');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(body)).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  // ── WebAuthn Hardware Key Authentication ─────────────────────────────────

  /// Starts WebAuthn credential registration via POST /api/webauthn/register/start.
  Future<Map<String, dynamic>?> webauthnRegisterStart(String username) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/webauthn/register/start');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode({'username': username})).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Finishes WebAuthn credential registration via POST /api/webauthn/register/finish.
  Future<bool> webauthnRegisterFinish(Map<String, dynamic> data) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/webauthn/register/finish');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(data)).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Starts WebAuthn authentication via POST /api/webauthn/auth/start.
  Future<Map<String, dynamic>?> webauthnAuthStart() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/webauthn/auth/start');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: '{}').timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return _decodeBody(res) as Map<String, dynamic>;
      }
    } catch (_) {}
    return null;
  }

  /// Finishes WebAuthn authentication via POST /api/webauthn/auth/finish.
  Future<bool> webauthnAuthFinish(Map<String, dynamic> data) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/webauthn/auth/finish');
      final headers = await _authHeaders();
      final res = await _client.post(uri, headers: headers, body: jsonEncode(data)).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Lists registered WebAuthn credentials via GET /api/webauthn/credentials.
  Future<List<Map<String, dynamic>>> webauthnListCredentials() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/webauthn/credentials');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Deletes a WebAuthn credential via DELETE /api/webauthn/credentials/{id}.
  Future<bool> webauthnDeleteCredential(String id) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/webauthn/credentials/${Uri.encodeComponent(id)}');
      final headers = await _authHeaders();
      final res = await _client.delete(uri, headers: headers).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // ── WASM Plugins API ──────────────────────────────────────────────────────

  /// Lists loaded WASM plugins via GET /api/plugins.
  Future<List<Map<String, dynamic>>> listPlugins() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/plugins');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        final data = _decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['plugins'] is List) {
          return (data['plugins'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  void dispose() {
    _client.close();
  }
}
