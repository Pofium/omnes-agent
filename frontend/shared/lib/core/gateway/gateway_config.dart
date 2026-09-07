// Configuration management for OmnesAgent Gateway connection.
import 'package:get_storage/get_storage.dart';

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

  /// Returns configured HTTP base URL (e.g. http://127.0.0.1:42617).
  static String getBaseUrl() {
    if (_overrideHttpUrl != null) return _overrideHttpUrl!;
    try {
      return _box.read<String>(_urlKey) ?? defaultHttpUrl;
    } catch (_) {
      return defaultHttpUrl;
    }
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
