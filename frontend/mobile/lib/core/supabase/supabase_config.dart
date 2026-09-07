// Configuration and initialization for Supabase client in OmnesAgent.

import 'package:flutter/foundation.dart';
import 'package:get_storage/get_storage.dart';
import 'package:supabase_flutter/supabase_flutter.dart';

class SupabaseConfig {
  static const String _storageKeyUrl = 'omnes_supabase_url';
  static const String _storageKeyAnonKey = 'omnes_supabase_anon_key';

  // Default demo / placeholder endpoint or self-hosted Supabase URL.
  // Can be configured by user in Settings or via environment.
  static const String defaultUrl = 'https://supabase.omnes.local';
  static const String defaultAnonKey = 'public-anon-key-placeholder';

  static const String redirectUrl = 'com.omagent.front://login';

  static bool _isInitialized = false;
  static bool get isInitialized => _isInitialized;

  static String? _mockUrl;
  static String? _mockAnonKey;

  /// Sets mock values for unit testing.
  static void setMockOverrides({String? url, String? anonKey}) {
    _mockUrl = url;
    _mockAnonKey = anonKey;
  }

  static String getSupabaseUrl() {
    if (_mockUrl != null) return _mockUrl!;
    try {
      final box = GetStorage();
      return box.read<String>(_storageKeyUrl) ?? defaultUrl;
    } catch (_) {
      return defaultUrl;
    }
  }

  static String getAnonKey() {
    if (_mockAnonKey != null) return _mockAnonKey!;
    try {
      final box = GetStorage();
      return box.read<String>(_storageKeyAnonKey) ?? defaultAnonKey;
    } catch (_) {
      return defaultAnonKey;
    }
  }

  static Future<void> saveConfig({required String url, required String anonKey}) async {
    final box = GetStorage();
    await box.write(_storageKeyUrl, url);
    await box.write(_storageKeyAnonKey, anonKey);
  }

  static Future<bool> initialize() async {
    if (_isInitialized) return true;

    final url = getSupabaseUrl();
    final anonKey = getAnonKey();

    try {
      await Supabase.initialize(
        url: url,
        anonKey: anonKey,
        authOptions: const FlutterAuthClientOptions(
          authFlowType: AuthFlowType.pkce,
        ),
        debug: kDebugMode,
      );
      _isInitialized = true;
      return true;
    } catch (e) {
      debugPrint('Supabase initialization warning (running in offline/mock mode): $e');
      _isInitialized = false;
      return false;
    }
  }

  static SupabaseClient? get client {
    if (!_isInitialized) return null;
    try {
      return Supabase.instance.client;
    } catch (_) {
      return null;
    }
  }

  /// Current user ID if signed in with Supabase Auth
  static String? get currentUserId => client?.auth.currentUser?.id;

  /// Current user email if signed in with Supabase Auth
  static String? get currentUserEmail => client?.auth.currentUser?.email;

  /// Whether user is signed in to Supabase
  static bool get isAuthenticated => client?.auth.currentUser != null;
}
