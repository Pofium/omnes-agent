import 'package:flutter/material.dart';
import 'package:flutter_screenutil/flutter_screenutil.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:get/get.dart';
import 'package:omagent_front/core/gateway/gateway_config.dart';
import 'package:omagent_front/core/supabase/profiles_repository.dart';
import 'package:omagent_front/core/supabase/settings_repository.dart';
import 'package:omagent_front/core/supabase/suggested_prompts_repository.dart';
import 'package:omagent_front/core/supabase/supabase_config.dart';
import 'package:omagent_front/helper/admob_helper.dart';
import 'package:omagent_front/helper/unity_ad.dart';
import 'package:omagent_front/features/system_status/system_status_screen.dart';

void main() {
  setUpAll(() {
    Get.testMode = true;
    GatewayConfig.setMockOverrides(
      httpUrl: 'http://127.0.0.1:42617',
      agentAlias: 'chief',
      token: 'test-token',
    );
    SupabaseConfig.setMockOverrides(
      url: 'https://supabase.omnes.local',
      anonKey: 'test-anon-key',
    );
  });

  group('Phase 6 - Supabase Models & Configuration', () {
    test('ProfilesRepository and SettingsRepository instantiate cleanly', () {
      final profilesRepo = ProfilesRepository();
      final settingsRepo = SettingsRepository();
      final promptsRepo = SuggestedPromptsRepository();

      expect(profilesRepo, isNotNull);
      expect(settingsRepo, isNotNull);
      expect(promptsRepo, isNotNull);
    });

    test('SuggestedPrompt JSON parsing and defaults', () {
      final json = {
        'id': 'prompt_1',
        'title': 'Код на Rust',
        'prompt': 'Напиши HTTP-сервер на Axum',
        'category': 'development',
        'sort': 1,
      };

      final item = SuggestedPrompt.fromJson(json);
      expect(item.id, 'prompt_1');
      expect(item.title, 'Код на Rust');
      expect(item.prompt, 'Напиши HTTP-сервер на Axum');
      expect(item.category, 'development');
      expect(item.sort, 1);

      // Verify default fallback prompts exist
      expect(SuggestedPromptsRepository.defaultPrompts.isNotEmpty, isTrue);
      expect(SuggestedPromptsRepository.defaultPrompts.length, greaterThanOrEqualTo(4));
    });

    test('SupabaseConfig properties and default fallback values', () {
      expect(SupabaseConfig.defaultUrl.isNotEmpty, isTrue);
      expect(SupabaseConfig.defaultAnonKey.isNotEmpty, isTrue);
      expect(SupabaseConfig.getSupabaseUrl(), contains('supabase'));
      expect(SupabaseConfig.getAnonKey().isNotEmpty, isTrue);
      expect(SupabaseConfig.redirectUrl, contains('omagent'));
    });
  });

  group('Phase 6 - Ads and Payments Cleanup', () {
    test('AdManager stub methods complete asynchronously without errors', () async {
      await AdManager.init();
      await AdManager.loadUnityIntAd();
      await AdManager.loadUnityRewardedAd();
      await AdManager.showIntAd();
      await AdManager.showRewardedAd();
      expect(true, isTrue);
    });

    test('AdMobHelper stub methods complete without errors', () async {
      AdMobHelper.initialization();
      await AdMobHelper.getInterstitialAdLoad();
      expect(true, isTrue);
    });
  });

  group('Phase 6 - PurchasePlanScreen (Omnes Status Screen) Widget Test', () {
    testWidgets('renders Omnes Status Screen with self-hosted active card', (WidgetTester tester) async {
      await tester.pumpWidget(
        ScreenUtilInit(
          designSize: const Size(414, 896),
          builder: (context, child) => const GetMaterialApp(
            home: PurchasePlanScreen(),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('SELF-HOSTED'), findsOneWidget);
      expect(find.text('Active'), findsOneWidget);
      expect(find.text('Omnes Enterprise / Personal'), findsOneWidget);
      expect(find.text('Статус шлюза Omnes'), findsOneWidget);
      expect(find.text('http://127.0.0.1:42617'), findsOneWidget);

      await tester.scrollUntilVisible(find.text('Вернуться в приложение'), 200);
      expect(find.text('Вернуться в приложение'), findsOneWidget);
    });
  });
}
