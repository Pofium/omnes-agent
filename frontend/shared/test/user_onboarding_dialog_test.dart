import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_shared/omnes_shared.dart';

void main() {
  group('UserOnboardingDialog Widget Tests', () {
    testWidgets('renders step 0 and validates required user name', (WidgetTester tester) async {
      tester.view.physicalSize = const Size(1200, 900);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);

      final profile = UserProfileData();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: UserOnboardingDialog(
              initialProfile: profile,
              onSave: (_) {},
            ),
          ),
        ),
      );

      // Verify Step 0 welcome text is displayed
      expect(find.text('Здравствуйте! Я ваш агент Omnes.'), findsOneWidget);
      expect(find.text('Далее'), findsOneWidget);

      // Attempt to proceed with empty name -> should show validation error
      await tester.tap(find.text('Далее'));
      await tester.pumpAndSettle();

      expect(find.text('Пожалуйста, введите ваше имя'), findsOneWidget);

      // Enter name
      final textFields = find.byType(TextField);
      expect(textFields, findsAtLeastNWidgets(1));
      await tester.enterText(textFields.first, 'Илья');
      await tester.pumpAndSettle();

      // Proceed to Step 1
      await tester.tap(find.text('Далее'));
      await tester.pumpAndSettle();

      // Verify Step 1 is rendered
      expect(find.text('Кто вы и с чем работаете?'), findsOneWidget);
      expect(find.text('Ваша инженерная роль'), findsOneWidget);
      expect(find.text('Назад'), findsOneWidget);
    });
  });
}
