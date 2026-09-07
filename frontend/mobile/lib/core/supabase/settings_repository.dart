// User settings repository for syncing user preferences across devices via Supabase table 'user_settings'.

import 'package:flutter/foundation.dart';
import 'package:supabase_flutter/supabase_flutter.dart';
import 'supabase_config.dart';

class SettingsRepository {
  final SupabaseClient? _client;

  SettingsRepository({SupabaseClient? client})
      : _client = client ?? SupabaseConfig.client;

  /// Fetches settings JSON map for a user.
  Future<Map<String, dynamic>?> getUserSettings(String userId) async {
    final client = _client ?? SupabaseConfig.client;
    if (client == null) return null;

    try {
      final response = await client
          .from('user_settings')
          .select('settings')
          .eq('user_id', userId)
          .maybeSingle();

      if (response != null && response['settings'] is Map) {
        return Map<String, dynamic>.from(response['settings']);
      }
      return null;
    } catch (e) {
      debugPrint('SettingsRepository.getUserSettings error: $e');
      return null;
    }
  }

  /// Saves settings JSON map for a user.
  Future<bool> saveUserSettings(String userId, Map<String, dynamic> settings) async {
    final client = _client ?? SupabaseConfig.client;
    if (client == null) return false;

    try {
      await client.from('user_settings').upsert({
        'user_id': userId,
        'settings': settings,
        'updated_at': DateTime.now().toUtc().toIso8601String(),
      });
      return true;
    } catch (e) {
      debugPrint('SettingsRepository.saveUserSettings error: $e');
      return false;
    }
  }
}
