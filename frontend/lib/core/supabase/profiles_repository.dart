// Profiles repository for Supabase table 'profiles' and Storage bucket 'avatars'.

import 'package:flutter/foundation.dart';
import 'package:supabase_flutter/supabase_flutter.dart';
import 'supabase_config.dart';

class ProfilesRepository {
  final SupabaseClient? _client;

  ProfilesRepository({SupabaseClient? client})
      : _client = client ?? SupabaseConfig.client;

  /// Fetches the profile for a given user ID.
  Future<Map<String, dynamic>?> getProfile(String userId) async {
    final client = _client ?? SupabaseConfig.client;
    if (client == null) return null;

    try {
      final response = await client
          .from('profiles')
          .select()
          .eq('id', userId)
          .maybeSingle();
      return response;
    } catch (e) {
      debugPrint('ProfilesRepository.getProfile error: $e');
      return null;
    }
  }

  /// Upserts user profile information.
  Future<bool> upsertProfile({
    required String userId,
    String? displayName,
    String? email,
    String? avatarUrl,
  }) async {
    final client = _client ?? SupabaseConfig.client;
    if (client == null) return false;

    try {
      final data = <String, dynamic>{
        'id': userId,
        'updated_at': DateTime.now().toUtc().toIso8601String(),
        if (displayName != null) 'display_name': displayName,
        if (email != null) 'email': email,
        if (avatarUrl != null) 'avatar_url': avatarUrl,
      };

      await client.from('profiles').upsert(data);
      return true;
    } catch (e) {
      debugPrint('ProfilesRepository.upsertProfile error: $e');
      return false;
    }
  }

  /// Uploads avatar image bytes to Supabase Storage bucket 'avatars'
  /// and returns the public download URL.
  Future<String?> uploadAvatar({
    required String userId,
    required Uint8List bytes,
    required String fileExtension,
  }) async {
    final client = _client ?? SupabaseConfig.client;
    if (client == null) return null;

    try {
      final filePath = '$userId/avatar_${DateTime.now().millisecondsSinceEpoch}.$fileExtension';
      await client.storage.from('avatars').uploadBinary(
            filePath,
            bytes,
            fileOptions: const FileOptions(upsert: true),
          );

      final publicUrl = client.storage.from('avatars').getPublicUrl(filePath);
      return publicUrl;
    } catch (e) {
      debugPrint('ProfilesRepository.uploadAvatar error: $e');
      return null;
    }
  }
}
