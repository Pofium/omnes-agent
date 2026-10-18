import '../../../model/session_info.dart';
import '../models/trajectory_step.dart';
import 'gateway_http_transport.dart';

/// Domain API client for Gateway session operations (/api/sessions/*).
class SessionApiClient {
  final GatewayHttpTransport transport;

  SessionApiClient({GatewayHttpTransport? transport})
      : transport = transport ?? GatewayHttpTransport();

  /// Fetches raw sessions list from /api/sessions.
  Future<List<SessionInfo>> getSessionsList() async {
    try {
      final res = await transport.get('/api/sessions');
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
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

  /// Fetches session messages from /api/sessions/{id}/messages.
  Future<List<SessionHistoryMessage>> getSessionMessages(String sessionId) async {
    try {
      final res = await transport.get('/api/sessions/$sessionId/messages');
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        List? rawList;
        if (data is List) {
          rawList = data;
        } else if (data is Map && data['messages'] is List) {
          rawList = data['messages'] as List;
        }
        if (rawList != null) {
          return rawList
              .whereType<Map<String, dynamic>>()
              .map((item) => SessionHistoryMessage.fromJson(item))
              .toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Fetches recorded trajectory steps for a session via GET /api/v1/sessions/{id}/steps.
  Future<List<TrajectoryStep>> getSessionSteps(String sessionId) async {
    try {
      final res = await transport.get('/api/v1/sessions/$sessionId/steps');
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        List? rawList;
        if (data is List) {
          rawList = data;
        } else if (data is Map && data['steps'] is List) {
          rawList = data['steps'] as List;
        }
        if (rawList != null) {
          return rawList
              .whereType<Map<String, dynamic>>()
              .map((item) => TrajectoryStep.fromJson(item))
              .toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Deletes a session on the gateway via DELETE /api/sessions/{id}.
  Future<bool> deleteSession(String sessionId) async {
    try {
      final res = await transport.delete('/api/sessions/$sessionId');
      return res.statusCode == 200 || res.statusCode == 204;
    } catch (_) {
      return false;
    }
  }

  /// Aborts an active session generation via POST /api/sessions/{id}/abort.
  Future<bool> abortSession(String sessionId) async {
    try {
      final res = await transport.post('/api/sessions/$sessionId/abort');
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Renames a session via POST /api/sessions/{id}/rename.
  Future<bool> sessionRename(String sessionId, String name) async {
    try {
      final res = await transport.post(
        '/api/sessions/$sessionId/rename',
        body: {'name': name},
      );
      return res.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  /// Fetches session state from /api/sessions/{id}/state.
  Future<Map<String, dynamic>?> getSessionState(String sessionId) async {
    try {
      final res = await transport.get('/api/sessions/$sessionId/state');
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Forks a session from a specific point via POST /api/sessions/{id}/fork.
  Future<Map<String, dynamic>?> forkSession({
    required String sessionId,
    String? forkPointMessageId,
    String? newTitle,
  }) async {
    try {
      final res = await transport.post(
        '/api/sessions/$sessionId/fork',
        body: {
          if (forkPointMessageId != null) 'fork_point_message_id': forkPointMessageId,
          if (newTitle != null) 'title': newTitle,
        },
      );
      if (res.statusCode == 200 || res.statusCode == 201) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Exports session trace for offline debugging via GET /api/sessions/{id}/trace.
  Future<Map<String, dynamic>?> exportSessionTrace(String sessionId) async {
    try {
      final res = await transport.get('/api/sessions/$sessionId/trace');
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Replays session trace via POST /api/sessions/{id}/replay.
  Future<Map<String, dynamic>?> replaySessionTrace({
    required String sessionId,
    int? speedMultiplier,
  }) async {
    try {
      final res = await transport.post(
        '/api/sessions/$sessionId/replay',
        body: {
          if (speedMultiplier != null) 'speed_multiplier': speedMultiplier,
        },
      );
      if (res.statusCode == 200) {
        return transport.decodeBody(res) as Map<String, dynamic>?;
      }
    } catch (_) {}
    return null;
  }

  /// Fetches list of actively running sessions.
  Future<List<Map<String, dynamic>>> getSessionsRunning() async {
    try {
      final res = await transport.get('/api/sessions/running');
      if (res.statusCode == 200) {
        final data = transport.decodeBody(res);
        if (data is List) {
          return data.whereType<Map<String, dynamic>>().toList();
        } else if (data is Map && data['running'] is List) {
          return (data['running'] as List).whereType<Map<String, dynamic>>().toList();
        }
      }
    } catch (_) {}
    return [];
  }

  /// Sends a single message to a session via HTTP POST /api/sessions/{id}/messages.
  Future<bool> postSessionMessage(String sessionId, String content) async {
    try {
      final res = await transport.post(
        '/api/sessions/$sessionId/messages',
        body: {'content': content},
      );
      return res.statusCode == 200 || res.statusCode == 201;
    } catch (_) {
      return false;
    }
  }
}
