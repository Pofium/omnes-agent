import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_desktop/features/onboarding/user_onboarding_dialog.dart';

void main() {
  group('UserProfileData Persistence & Serialization Tests', () {
    test('UserProfileData initializes with default values', () {
      final profile = UserProfileData();
      expect(profile.firstName, equals('Илья'));
      expect(profile.lastName, equals('Пресняков'));
      expect(profile.fullName, equals('Илья Пресняков'));
      expect(profile.initials, equals('ИП'));
      expect(profile.role, contains('Tech Lead'));
      expect(profile.enableAstMemory, isTrue);
    });

    test('UserProfileData toJson and fromJson roundtrip correctly', () {
      final original = UserProfileData(
        firstName: 'Алексей',
        lastName: 'Смирнов',
        role: 'Senior Rust Engineer',
        primaryStack: 'Rust / Actix / Tokio',
        autonomyStyle: 'Supervised',
        language: 'English',
        enableAstMemory: false,
      );

      final json = original.toJson();
      expect(json['firstName'], equals('Алексей'));
      expect(json['lastName'], equals('Смирнов'));
      expect(json['enableAstMemory'], isFalse);

      final restored = UserProfileData.fromJson(json);
      expect(restored.firstName, equals('Алексей'));
      expect(restored.lastName, equals('Смирнов'));
      expect(restored.fullName, equals('Алексей Смирнов'));
      expect(restored.initials, equals('АС'));
      expect(restored.role, equals('Senior Rust Engineer'));
      expect(restored.primaryStack, equals('Rust / Actix / Tokio'));
      expect(restored.autonomyStyle, equals('Supervised'));
      expect(restored.language, equals('English'));
      expect(restored.enableAstMemory, isFalse);
    });

    test('UserProfileData fromJson handles missing or null fields gracefully', () {
      final partialJson = <String, dynamic>{
        'firstName': 'Елена',
      };

      final profile = UserProfileData.fromJson(partialJson);
      expect(profile.firstName, equals('Елена'));
      expect(profile.lastName, equals('Пресняков')); // fallback default
      expect(profile.enableAstMemory, isTrue);
    });
  });
}
