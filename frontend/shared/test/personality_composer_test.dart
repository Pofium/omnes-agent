import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_shared/features/onboarding/personality_composer.dart';

void main() {
  group('PersonalityComposer Tests', () {
    final testDate = DateTime(2026, 9, 14);

    test('generates all 4 expected personality files', () {
      final answers = OnboardingAnswers(
        userName: 'Илья',
        role: 'Tech Lead',
        primaryStack: 'Rust / Dart',
        language: 'Русский',
        autonomyStyle: 'Ask before changes (спрашивать перед правками)',
        agentTone: 'Дружелюбный',
        agentName: 'omnes',
        createdDate: testDate,
      );

      final files = PersonalityComposer.compose(answers);

      expect(files.keys, containsAll(['SOUL.md', 'IDENTITY.md', 'USER.md', 'MEMORY.md']));
      expect(files['SOUL.md'], contains('Дружелюбный'));
      expect(files['SOUL.md'], contains('спрашиваю'));
      expect(files['SOUL.md'], contains('Илья'));

      expect(files['IDENTITY.md'], contains('omnes'));
      expect(files['IDENTITY.md'], contains('Илья'));
      expect(files['IDENTITY.md'], contains('14.09.2026'));

      expect(files['USER.md'], contains('Имя: Илья'));
      expect(files['USER.md'], contains('Роль: Tech Lead'));
      expect(files['USER.md'], contains('Основной стек: Rust / Dart'));

      expect(files['MEMORY.md'], contains('14.09.2026'));
      expect(files['MEMORY.md'], contains('Илья'));
    });

    test('generates English personality when selected', () {
      final answers = OnboardingAnswers(
        userName: 'Sarah Connor',
        role: 'DevOps Engineer',
        primaryStack: 'Kubernetes / Go',
        language: 'English',
        autonomyStyle: 'Full access',
        agentTone: 'Concise',
        agentName: 'omnes',
        createdDate: testDate,
      );

      final files = PersonalityComposer.compose(answers);

      expect(files['SOUL.md'], contains('I am omnes'));
      expect(files['SOUL.md'], contains('Sarah Connor'));
      expect(files['SOUL.md'], contains('Extremely concise'));
      expect(files['SOUL.md'], contains('Full access'));

      expect(files['IDENTITY.md'], contains('DevOps Engineer (Kubernetes / Go)'));
      expect(files['USER.md'], contains('Preferred Language: English'));
    });

    test('is strictly deterministic: identical inputs yield identical output', () {
      final answers1 = OnboardingAnswers(
        userName: 'Alex',
        role: 'Frontend Dev',
        primaryStack: 'Flutter',
        language: 'English',
        autonomyStyle: 'Plan mode',
        agentTone: 'Business',
        createdDate: testDate,
      );

      final answers2 = OnboardingAnswers(
        userName: 'Alex',
        role: 'Frontend Dev',
        primaryStack: 'Flutter',
        language: 'English',
        autonomyStyle: 'Plan mode',
        agentTone: 'Business',
        createdDate: testDate,
      );

      final files1 = PersonalityComposer.compose(answers1);
      final files2 = PersonalityComposer.compose(answers2);

      expect(files1, equals(files2));
    });

    test('guarantees maximum character limit is never exceeded', () {
      final giantString = 'A' * 30000;
      final answers = OnboardingAnswers(
        userName: giantString,
        role: giantString,
        primaryStack: giantString,
        language: 'Русский',
        autonomyStyle: 'Full access',
        agentTone: 'Деловой',
        createdDate: testDate,
      );

      final files = PersonalityComposer.compose(answers);

      for (final entry in files.entries) {
        expect(
          entry.value.length,
          lessThanOrEqualTo(PersonalityComposer.maxFileChars),
          reason: 'File ${entry.key} exceeded maximum file characters',
        );
      }
    });
  });
}
