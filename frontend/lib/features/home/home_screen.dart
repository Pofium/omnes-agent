import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../core/gateway/gateway_config.dart';
import '../../routes/routes.dart';
import '../../utils/custom_color.dart';
import '../../utils/dimensions.dart';
import '../agents/agent_picker_dialog.dart';
import 'drawer_screen.dart';
import 'home_controller.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final controller = Get.put(HomeController());

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final activeAgent = GatewayConfig.getAgentAlias();

    return Scaffold(
      drawer: DrawerWidget(isDark: isDark.obs),
      appBar: AppBar(
        title: Row(
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
              decoration: BoxDecoration(
                color: CustomColor.primaryColor.withOpacity(0.15),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: CustomColor.primaryColor.withOpacity(0.3)),
              ),
              child: const Text(
                'OMNES',
                style: TextStyle(
                  color: CustomColor.primaryColor,
                  fontWeight: FontWeight.w900,
                  fontSize: 14,
                  letterSpacing: 1.2,
                ),
              ),
            ),
            const SizedBox(width: 8),
            Text(
              'ZeroClaw Client',
              style: TextStyle(
                fontSize: 14,
                color: isDark ? Colors.white70 : Colors.black54,
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            tooltip: 'Выбрать агента',
            icon: const Icon(Icons.smart_toy_outlined),
            onPressed: () => AgentPickerDialog.show(context),
          ),
          IconButton(
            tooltip: 'Настройки',
            icon: const Icon(Icons.settings_outlined),
            onPressed: () => Get.toNamed(Routes.settingsScreen),
          ),
        ],
      ),
      body: SafeArea(
        child: ListView(
          padding: EdgeInsets.all(Dimensions.defaultPaddingSize),
          children: [
            _buildActiveAgentBanner(context, activeAgent, isDark),
            SizedBox(height: Dimensions.heightSize * 1.2),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.comments,
              iconColor: const Color(0xFF00B091),
              title: 'Чат с агентом',
              subtitle: 'Потоковый диалог, reasoning и выполнение инструментов',
              badge: 'LIVE WS',
              onTap: () => Get.toNamed(Routes.chatScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.folderTree,
              iconColor: const Color(0xFF00D1FF),
              title: 'Проекты и файлы',
              subtitle: 'Рабочие директории, файлы проекта, заметки NOTES.md',
              onTap: () => Get.toNamed(Routes.projectsScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.clockRotateLeft,
              iconColor: const Color(0xFFFFB800),
              title: 'История сессий',
              subtitle: 'Возобновление диалогов и управление тредами агента',
              onTap: () => Get.toNamed(Routes.sessionsScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.brain,
              iconColor: const Color(0xFF9D00FF),
              title: 'Память агента',
              subtitle: 'Долговременная семантическая память и сохраненные факты',
              onTap: () => Get.toNamed(Routes.memoryScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.clock,
              iconColor: const Color(0xFFFF5252),
              title: 'Автоматизация (Cron)',
              subtitle: 'Фоновые периодические задачи и история выполнения',
              onTap: () => Get.toNamed(Routes.cronScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.chartPie,
              iconColor: const Color(0xFF00E676),
              title: 'Стоимость и токены',
              subtitle: 'Детализация расхода токенов и затрат по моделям',
              onTap: () => Get.toNamed(Routes.statsScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.wrench,
              iconColor: const Color(0xFF00E5FF),
              title: 'Инструменты (Tools)',
              subtitle: 'Каталог инструментов агента и схемы параметров',
              onTap: () => Get.toNamed(Routes.toolsScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.wandMagicSparkles,
              iconColor: const Color(0xFFE040FB),
              title: 'Навыки (Skills)',
              subtitle: 'Установленные пакеты и бандлы процедур агента',
              onTap: () => Get.toNamed(Routes.skillsScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.stethoscope,
              iconColor: const Color(0xFF00E5FF),
              title: 'Диагностика (Doctor)',
              subtitle: 'Проверка целостности шлюза, провайдеров и памяти',
              badge: 'HEALTH',
              onTap: () => Get.toNamed(Routes.doctorScreen),
            ),
            _buildFeatureCard(
              context,
              isDark: isDark,
              icon: FontAwesomeIcons.receipt,
              iconColor: const Color(0xFF38BDF8),
              title: 'Логи сервера',
              subtitle: 'Журнал событий шлюза в реальном времени',
              onTap: () => Get.toNamed(Routes.logsScreen),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildActiveAgentBanner(BuildContext context, String agent, bool isDark) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [
            CustomColor.primaryColor,
            CustomColor.primaryColor.withOpacity(0.8),
          ],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        borderRadius: BorderRadius.circular(Dimensions.radius * 1.2),
        boxShadow: [
          BoxShadow(
            color: CustomColor.primaryColor.withOpacity(0.25),
            blurRadius: 10,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: Colors.white.withOpacity(0.2),
              shape: BoxShape.circle,
            ),
            child: const Icon(FontAwesomeIcons.robot, color: Colors.white, size: 20),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    const Text(
                      'Активный агент: ',
                      style: TextStyle(color: Colors.white70, fontSize: 13),
                    ),
                    Text(
                      agent.toUpperCase(),
                      style: const TextStyle(
                        color: Colors.white,
                        fontWeight: FontWeight.bold,
                        fontSize: 14,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 2),
                Text(
                  GatewayConfig.getBaseUrl(),
                  style: TextStyle(
                    color: Colors.white.withOpacity(0.85),
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
          TextButton(
            style: TextButton.styleFrom(
              backgroundColor: Colors.white.withOpacity(0.2),
              foregroundColor: Colors.white,
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
            ),
            onPressed: () => AgentPickerDialog.show(context),
            child: const Text('Сменить', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
          ),
        ],
      ),
    );
  }

  Widget _buildFeatureCard(
    BuildContext context, {
    required bool isDark,
    required IconData icon,
    required Color iconColor,
    required String title,
    required String subtitle,
    required VoidCallback onTap,
    String? badge,
  }) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(Dimensions.radius),
        child: Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: isDark ? CustomColor.bgColor : Colors.white,
            borderRadius: BorderRadius.circular(Dimensions.radius),
            border: Border.all(
              color: isDark ? Colors.white12 : Colors.black12,
            ),
          ),
          child: Row(
            children: [
              Container(
                width: 44,
                height: 44,
                decoration: BoxDecoration(
                  color: iconColor.withOpacity(0.12),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Icon(icon, color: iconColor, size: 20),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(
                          title,
                          style: const TextStyle(
                            fontSize: 15,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        if (badge != null) ...[
                          const SizedBox(width: 8),
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: Colors.green.withOpacity(0.15),
                              borderRadius: BorderRadius.circular(6),
                            ),
                            child: Text(
                              badge,
                              style: const TextStyle(
                                color: Colors.green,
                                fontSize: 10,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ),
                        ],
                      ],
                    ),
                    const SizedBox(height: 3),
                    Text(
                      subtitle,
                      style: TextStyle(
                        fontSize: 12,
                        color: isDark ? Colors.white60 : Colors.black54,
                      ),
                    ),
                  ],
                ),
              ),
              Icon(
                Icons.chevron_right_rounded,
                color: isDark ? Colors.white38 : Colors.black26,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
