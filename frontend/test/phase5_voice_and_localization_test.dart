import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:omagent_front/helper/speech_helper.dart';
import 'package:omagent_front/utils/language/local_string.dart';
import 'package:omagent_front/utils/strings.dart';
import 'package:omagent_front/widgets/markdown_preview_widget.dart';

void main() {
  group('Phase 5 - SpeechHelper Markdown Stripping', () {
    test('strips headers and bold/italic formatting', () {
      const input = '# Header 1\n## Header 2\n**bold text** and *italic text* and __underline__';
      final result = SpeechHelper.stripMarkdownForSpeech(input);

      expect(result, isNot(contains('#')));
      expect(result, isNot(contains('*')));
      expect(result, isNot(contains('__')));
      expect(result, contains('Header 1'));
      expect(result, contains('Header 2'));
      expect(result, contains('bold text and italic text and underline'));
    });

    test('strips fenced code blocks and inline code', () {
      const input = 'Here is code:\n```dart\nvoid main() {\n  print("hello");\n}\n```\nAnd inline `test_func()` here.';
      final result = SpeechHelper.stripMarkdownForSpeech(input);

      expect(result, isNot(contains('```')));
      expect(result, isNot(contains('print("hello")')));
      expect(result, contains('test_func()'));
      expect(result, isNot(contains('`')));
    });

    test('converts markdown links and removes image tags', () {
      const input = 'Check out [Omnes Agent](https://omnes.ai) and image: ![logo](https://omnes.ai/logo.png)';
      final result = SpeechHelper.stripMarkdownForSpeech(input);

      expect(result, isNot(contains('https://omnes.ai')));
      expect(result, contains('Check out Omnes Agent and image:'));
      expect(result, isNot(contains('![')));
    });

    test('strips blockquotes and bullet points', () {
      const input = '> Important quote\n- Item one\n- Item two\n1. Numbered item';
      final result = SpeechHelper.stripMarkdownForSpeech(input);

      expect(result, isNot(contains('>')));
      expect(result, isNot(contains('- Item')));
      expect(result, contains('Important quote'));
      expect(result, contains('Item one'));
      expect(result, contains('Item two'));
      expect(result, contains('Numbered item'));
    });
  });

  group('Phase 5 - Localization and Keys', () {
    test('en_US and ru_RU translations exist and contain required keys', () {
      final translations = LocalString();
      final keys = translations.keys;

      expect(keys.containsKey('en_US'), isTrue);
      expect(keys.containsKey('ru_RU'), isTrue);

      final en = keys['en_US']!;
      final ru = keys['ru_RU']!;

      final testKeys = [
        Strings.sessions,
        Strings.projects,
        Strings.memory,
        Strings.automation,
        Strings.approvals,
        Strings.stats,
        Strings.voiceAutoTts,
        Strings.listening,
      ];

      for (final k in testKeys) {
        expect(en.containsKey(k), isTrue, reason: 'en_US missing key $k');
        expect(ru.containsKey(k), isTrue, reason: 'ru_RU missing key $k');
        expect(en[k]!.isNotEmpty, isTrue);
        expect(ru[k]!.isNotEmpty, isTrue);
      }

      // Check Russian specific translations
      expect(ru[Strings.sessions], 'Чаты');
      expect(ru[Strings.projects], 'Проекты');
      expect(ru[Strings.memory], 'Память');
      expect(ru[Strings.automation], 'Автоматизация');
      expect(ru[Strings.approvals], 'Подтверждения');
      expect(ru[Strings.stats], 'Статистика');
    });
  });

  group('Phase 5 - MarkdownPreviewWidget Widget Tests', () {
    testWidgets('renders in shrinkWrap mode without ListView', (tester) async {
      const markdown = '# Title\n- Bullet 1\n- Bullet 2\n```\ncode snippet\n```';
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: MarkdownPreviewWidget(
              text: markdown,
              shrinkWrap: true,
              textColor: Colors.white,
            ),
          ),
        ),
      );

      expect(find.text('Title'), findsOneWidget);
      expect(find.text('Bullet 1'), findsOneWidget);
      expect(find.text('code snippet'), findsOneWidget);
    });
  });
}
