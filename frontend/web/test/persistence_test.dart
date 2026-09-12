import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_web/features/onboarding/user_onboarding_dialog.dart';

void main() {
  group('Web UserProfileData Persistence & Serialization Tests', () {
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
        firstName: 'Иван',
        lastName: 'Петров',
        role: 'Fullstack Flutter / Rust Dev',
        primaryStack: 'Flutter / Rust / Docker',
        autonomyStyle: 'Full access',
        language: 'Русский',
        enableAstMemory: true,
      );

      final json = original.toJson();
      expect(json['firstName'], equals('Иван'));
      expect(json['lastName'], equals('Петров'));

      final restored = UserProfileData.fromJson(json);
      expect(restored.firstName, equals('Иван'));
      expect(restored.lastName, equals('Петров'));
      expect(restored.fullName, equals('Иван Петров'));
      expect(restored.initials, equals('ИП'));
      expect(restored.role, equals('Fullstack Flutter / Rust Dev'));
      expect(restored.primaryStack, equals('Flutter / Rust / Docker'));
    });
  });
}
