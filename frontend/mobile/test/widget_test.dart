import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:omagent_front/widgets/markdown_preview_widget.dart';

void main() {
  testWidgets('MarkdownPreviewWidget renders markdown headings and content', (WidgetTester tester) async {
    const md = '''
# Заголовок проекта
## Подзаголовок
- Пункт 1
- Пункт 2
```dart
void main() {}
```
''';
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: MarkdownPreviewWidget(text: md),
        ),
      ),
    );

    expect(find.text('Заголовок проекта'), findsOneWidget);
    expect(find.text('Подзаголовок'), findsOneWidget);
    expect(find.text('Пункт 1'), findsOneWidget);
    expect(find.text('Пункт 2'), findsOneWidget);
    expect(find.text('void main() {}'), findsOneWidget);
  });
}
