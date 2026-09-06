import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../core/gateway/gateway_config.dart';
import '../../routes/routes.dart';
import '../../utils/custom_color.dart';
import '../../utils/custom_style.dart';
import '../../utils/dimensions.dart';
import '../../utils/strings.dart';
import '../../widgets/appbar/appbar_widget2.dart';

/// Screen displaying Omnes Server & Subscription Status.
/// Omnes is an open-source, self-hosted system where all features are free and unlimited.
class PurchasePlanScreen extends StatelessWidget {
  const PurchasePlanScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBarWidget2(
        context: context,
        appTitle: Strings.subscriptionPlan.tr,
        onTap: () {
          Get.back();
        },
      ),
      body: SafeArea(
        child: ListView(
          padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
          children: [
            _buildPlanCard(context),
            SizedBox(height: Dimensions.heightSize * 1.5),
            _buildGatewayCard(context),
            SizedBox(height: Dimensions.heightSize * 1.5),
            _buildFeaturesCard(context),
            SizedBox(height: Dimensions.heightSize * 2),
            _buildActionButtons(context),
          ],
        ),
      ),
    );
  }

  Widget _buildPlanCard(BuildContext context) {
    return Container(
      padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            CustomColor.primaryColor,
            CustomColor.primaryColor.withOpacity(0.8),
          ],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        borderRadius: BorderRadius.circular(Dimensions.radius * 1.5),
        boxShadow: [
          BoxShadow(
            color: CustomColor.primaryColor.withOpacity(0.3),
            blurRadius: 12,
            offset: const Offset(0, 6),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                decoration: BoxDecoration(
                  color: Colors.white.withOpacity(0.25),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: const Text(
                  'SELF-HOSTED',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.bold,
                    fontSize: 11,
                    letterSpacing: 1.1,
                  ),
                ),
              ),
              const Row(
                children: [
                  Icon(Icons.check_circle, color: Colors.greenAccent, size: 18),
                  SizedBox(width: 4),
                  Text(
                    'Active',
                    style: TextStyle(
                      color: Colors.white,
                      fontWeight: FontWeight.w600,
                      fontSize: 13,
                    ),
                  ),
                ],
              ),
            ],
          ),
          SizedBox(height: Dimensions.heightSize),
          const Text(
            'Omnes Enterprise / Personal',
            style: TextStyle(
              color: Colors.white,
              fontSize: 22,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            'Неограниченный персональный доступ',
            style: TextStyle(
              color: Colors.white.withOpacity(0.9),
              fontSize: 14,
            ),
          ),
          SizedBox(height: Dimensions.heightSize * 1.2),
          const Divider(color: Colors.white24),
          SizedBox(height: Dimensions.heightSize * 0.8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Стоимость:',
                style: TextStyle(
                  color: Colors.white.withOpacity(0.8),
                  fontSize: 13,
                ),
              ),
              const Text(
                '0 ₽ / Free & Open Source',
                style: TextStyle(
                  color: Colors.white,
                  fontWeight: FontWeight.bold,
                  fontSize: 14,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildGatewayCard(BuildContext context) {
    final gatewayUrl = GatewayConfig.getBaseUrl();
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Container(
      padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
      decoration: BoxDecoration(
        color: isDark ? CustomColor.bgColor : Colors.white,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        border: Border.all(
          color: isDark ? Colors.white12 : Colors.black12,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(
                Icons.dns_rounded,
                color: CustomColor.primaryColor,
                size: 20,
              ),
              const SizedBox(width: 8),
              Text(
                'Статус шлюза Omnes',
                style: CustomStyle.primaryTextStyle.copyWith(
                  fontWeight: FontWeight.bold,
                  fontSize: 16,
                ),
              ),
            ],
          ),
          SizedBox(height: Dimensions.heightSize),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text('Хост шлюза:', style: TextStyle(color: Colors.grey)),
              Text(
                gatewayUrl,
                style: const TextStyle(fontWeight: FontWeight.w600),
              ),
            ],
          ),
          const SizedBox(height: 8),
          const Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('Протокол:', style: TextStyle(color: Colors.grey)),
              Text(
                'WebSocket & REST (ZeroClaw)',
                style: TextStyle(fontWeight: FontWeight.w600),
              ),
            ],
          ),
          const SizedBox(height: 8),
          const Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text('Приватность данных:', style: TextStyle(color: Colors.grey)),
              Text(
                '100% Локально / Без рекламы',
                style: TextStyle(color: Colors.green, fontWeight: FontWeight.w600),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildFeaturesCard(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;

    final features = [
      'Неограниченная генерация и стриминг ответов',
      'Инструменты агента (Bash, чтение/запись файлов, веб-поиск)',
      'Управление проектами и сессиями в реальном времени',
      'Голосовой синтез (TTS) и распознавание (STT)',
      'Долговременная семантическая память и планировщик Cron',
      'Синхронизация профиля через Supabase',
    ];

    return Container(
      padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
      decoration: BoxDecoration(
        color: isDark ? CustomColor.bgColor : Colors.white,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        border: Border.all(
          color: isDark ? Colors.white12 : Colors.black12,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Включенные возможности',
            style: CustomStyle.primaryTextStyle.copyWith(
              fontWeight: FontWeight.bold,
              fontSize: 16,
            ),
          ),
          SizedBox(height: Dimensions.heightSize),
          ...features.map(
            (feature) => Padding(
              padding: const EdgeInsets.only(bottom: 10),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Icon(Icons.check_circle_outline, color: Colors.green, size: 18),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      feature,
                      style: const TextStyle(fontSize: 13, height: 1.3),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionButtons(BuildContext context) {
    return ElevatedButton.icon(
      style: ElevatedButton.styleFrom(
        backgroundColor: CustomColor.primaryColor,
        foregroundColor: Colors.white,
        padding: const EdgeInsets.symmetric(vertical: 14),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(Dimensions.radius),
        ),
      ),
      icon: const Icon(Icons.check),
      label: const Text(
        'Вернуться в приложение',
        style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold),
      ),
      onPressed: () {
        Get.offAllNamed(Routes.homeScreen);
      },
    );
  }
}
