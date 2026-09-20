import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:get_storage/get_storage.dart';
import 'package:universal_io/io.dart' as universal_io;

class GatewayConfig {
  static const String defaultHttpUrl = "http://127.0.0.1:42617";
  static const String defaultAgentAlias = "chief";
  static const String defaultToken = "omnes-token-secret-12345";

  static const String _urlKey = "gateway_url";
  static const String _agentKey = "gateway_agent_alias";
  static const String _tokenKey = "gateway_bearer_token";

  static String? _overrideHttpUrl;
  static String? _overrideAgentAlias;
  static String? _overrideToken;

  /// Sets mock overrides for unit testing without platform channel storage.
  static void setMockOverrides({String? httpUrl, String? agentAlias, String? token}) {
    _overrideHttpUrl = httpUrl;
    _overrideAgentAlias = agentAlias;
    _overrideToken = token;
  }

  static GetStorage get _box => GetStorage();

  /// Returns configured HTTP base URL (e.g. http://127.0.0.1:42617 or origin in browser).
  static String getBaseUrl() {
    if (_overrideHttpUrl != null) return _overrideHttpUrl!;
    const envUrl = String.fromEnvironment('GATEWAY_BASE_URL', defaultValue: '');
    if (envUrl.isNotEmpty) {
      return envUrl.endsWith('/') ? envUrl.substring(0, envUrl.length - 1) : envUrl;
    }
    try {
      final stored = _box.read<String>(_urlKey);
      if (stored != null && stored.isNotEmpty) return stored;
    } catch (_) {}

    if (!kIsWeb) {
      try {
        final envUrl = universal_io.Platform.environment['OMNES_GATEWAY_URL'];
        if (envUrl != null && envUrl.isNotEmpty) {
          return envUrl.endsWith('/') ? envUrl.substring(0, envUrl.length - 1) : envUrl;
        }

        final envPort = universal_io.Platform.environment['OMNES_GATEWAY_PORT'];
        if (envPort != null && envPort.isNotEmpty) {
          return 'http://127.0.0.1:$envPort';
        }

        final discFile = universal_io.File('.omnes/gateway.json');
        if (discFile.existsSync()) {
          final parsed = jsonDecode(discFile.readAsStringSync());
          if (parsed is Map && parsed['port'] != null) {
            return 'http://127.0.0.1:${parsed['port']}';
          }
        }
      } catch (_) {}
    }

    if (kIsWeb) {
      try {
        final origin = Uri.base.origin;
        if (origin.isNotEmpty && origin != 'null') {
          return origin;
        }
      } catch (_) {}
    }
    return defaultHttpUrl;
  }

  /// Sets HTTP base URL.
  static Future<void> setBaseUrl(String url) async {
    final sanitized = url.endsWith('/') ? url.substring(0, url.length - 1) : url;
    try {
      await _box.write(_urlKey, sanitized);
    } catch (_) {}
  }

  /// Returns WebSocket URL for gateway (e.g. ws://127.0.0.1:42617).
  static String getWsBaseUrl() {
    final httpUrl = getBaseUrl();
    if (httpUrl.startsWith('https://')) {
      return httpUrl.replaceFirst('https://', 'wss://');
    } else if (httpUrl.startsWith('http://')) {
      return httpUrl.replaceFirst('http://', 'ws://');
    }
    return "ws://$httpUrl";
  }

  /// Returns port of configured gateway
  static int getPort() {
    final u = Uri.tryParse(getBaseUrl());
    return u?.port != null && u!.port > 0 ? u.port : 42617;
  }

  /// Returns currently selected agent alias (default: chief).
  static String getAgentAlias() {
    if (_overrideAgentAlias != null) return _overrideAgentAlias!;
    try {
      return _box.read<String>(_agentKey) ?? defaultAgentAlias;
    } catch (_) {
      return defaultAgentAlias;
    }
  }

  /// Sets active agent alias.
  static Future<void> setAgentAlias(String alias) async {
    try {
      await _box.write(_agentKey, alias);
    } catch (_) {}
  }

  /// Alias for getAgentAlias returning Future for consistency.
  static Future<String> getActiveAgent() async {
    return getAgentAlias();
  }

  /// Retrieves Bearer token for gateway auth.
  static Future<String> getToken() async {
    if (_overrideToken != null) return _overrideToken!;
    try {
      return _box.read<String>(_tokenKey) ?? defaultToken;
    } catch (_) {
      return defaultToken;
    }
  }

  /// Stores Bearer token.
  static Future<void> setToken(String token) async {
    try {
      await _box.write(_tokenKey, token);
    } catch (_) {}
  }
}
