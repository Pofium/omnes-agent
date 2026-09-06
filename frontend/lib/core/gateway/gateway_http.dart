// HTTP REST client for OmnesAgent Gateway endpoints.
import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../features/files/models/workspace_entry.dart';
import '../../features/sessions/models/session_info.dart';
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
  Future<bool> checkHealth() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/health');
      final res = await _client.get(uri).timeout(const Duration(seconds: 5));
      if (res.statusCode == 200) {
        final data = jsonDecode(res.body);
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
        return jsonDecode(res.body) as Map<String, dynamic>;
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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

  /// Fetches personality files for agent from /api/personality?agent={alias}.
  Future<Map<String, dynamic>?> getAgentPersonality(String agentAlias) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/api/personality?agent=${Uri.encodeComponent(agentAlias)}');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 10));
      if (res.statusCode == 200) {
        return jsonDecode(res.body) as Map<String, dynamic>;
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
        if (data is Map && data['runs'] is List) {
          return (data['runs'] as List).whereType<Map<String, dynamic>>().toList();
        } else if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  // ── Approvals / SOP API (/admin/sop) ──────────────────────────────────────

  /// Checks for pending approval actions from GET /admin/sop/pending.
  Future<List<Map<String, dynamic>>> sopPending() async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/sop/pending');
      final headers = await _authHeaders();
      final res = await _client.get(uri, headers: headers).timeout(const Duration(seconds: 5));
      if (res.statusCode == 200) {
        final data = jsonDecode(res.body);
        if (data is Map && data['pending'] is List) {
          return (data['pending'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Approves a gated action via POST /admin/sop/approve.
  Future<bool> sopApprove(String runId) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/sop/approve');
      final headers = await _authHeaders();
      final body = jsonEncode({'run_id': runId});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Denies/cancels a gated action via POST /admin/sop/deny.
  Future<bool> sopDeny(String runId) async {
    try {
      final base = GatewayConfig.getBaseUrl();
      final uri = Uri.parse('$base/admin/sop/deny');
      final headers = await _authHeaders();
      final body = jsonEncode({'run_id': runId});
      final res = await _client.post(uri, headers: headers, body: body).timeout(const Duration(seconds: 10));
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
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
        final data = jsonDecode(res.body);
        if (data is Map && data['token'] != null) {
          final token = data['token'].toString();
          await GatewayConfig.setToken(token);
          return token;
        }
      }
    } catch (_) {}
    return null;
  }

  void dispose() {
    _client.close();
  }
}
